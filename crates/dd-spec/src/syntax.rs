//! YAML 1.2 parsing into a tree where every node carries its byte span.
//!
//! Built on `saphyr-parser` events (R-006) so that duplicate keys, aliases and limits are
//! under our control. The tree converts to a `serde_json::Value` for schema validation plus a
//! [`SpanMap`] from JSON pointers back to source spans.

use std::collections::HashMap;

use saphyr_parser::{Event, Parser, ScalarStyle, SpannedEventReceiver};

use crate::diag::{Diagnostic, Span};

/// Maximum accepted document size in bytes (spec §19).
pub const MAX_BYTES: usize = 1024 * 1024;
/// Maximum nesting depth (spec §19).
pub const MAX_DEPTH: usize = 64;
/// Maximum number of alias expansions (spec §19).
pub const MAX_ALIASES: usize = 1000;

/// A parsed YAML node.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// What the node holds.
    pub kind: NodeKind,
    /// Byte span of the node in the source.
    pub span: Span,
}

/// The content of a [`Node`].
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// A scalar with its resolved value.
    Scalar(Scalar),
    /// A sequence.
    Seq(Vec<Node>),
    /// A mapping, in document order.
    Map(Vec<(Node, Node)>),
}

/// A scalar value with the text it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct Scalar {
    /// Resolved value under the YAML 1.2 core schema.
    pub value: ScalarValue,
    /// Source text of the scalar (after unquoting).
    pub text: String,
    /// True for unquoted (plain) scalars.
    pub plain: bool,
}

/// YAML 1.2 core schema scalar types.
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarValue {
    /// `null`, `~` or empty.
    Null,
    /// `true` / `false`.
    Bool(bool),
    /// Integer that fits in `i64`.
    Int(i64),
    /// Integer above `i64::MAX`.
    UInt(u64),
    /// Floating point.
    Float(f64),
    /// Anything else.
    Str(String),
}

/// Spans recorded for each JSON pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeSpans {
    /// Span of the mapping key, when the node is a mapping value.
    pub key: Option<Span>,
    /// Span of the value.
    pub value: Span,
}

/// Map from RFC 6901 JSON pointer to source spans.
#[derive(Debug, Default, Clone)]
pub struct SpanMap {
    map: HashMap<String, NodeSpans>,
}

impl SpanMap {
    /// Spans for `pointer`, if recorded.
    pub fn get(&self, pointer: &str) -> Option<NodeSpans> {
        self.map.get(pointer).copied()
    }

    /// Value span for `pointer`, or of its nearest recorded ancestor.
    pub fn value_or_ancestor(&self, pointer: &str) -> Option<Span> {
        let mut p = pointer;
        loop {
            if let Some(s) = self.map.get(p) {
                return Some(s.value);
            }
            p = &p[..p.rfind('/')?];
        }
    }

    /// Span of the first top-level key (used to anchor document-level findings).
    pub fn first_key(&self) -> Option<Span> {
        self.map
            .iter()
            .filter(|(p, _)| p.len() > 1 && !p[1..].contains('/'))
            .filter_map(|(_, s)| s.key)
            .min_by_key(|s| s.start)
    }

    /// Key span if available, else value span, else nearest ancestor.
    pub fn key_or_value(&self, pointer: &str) -> Option<Span> {
        match self.map.get(pointer) {
            Some(NodeSpans { key: Some(k), .. }) => Some(*k),
            Some(s) => Some(s.value),
            None => self.value_or_ancestor(pointer),
        }
    }
}

/// Escape one JSON pointer token (RFC 6901).
pub fn escape_token(t: &str) -> String {
    t.replace('~', "~0").replace('/', "~1")
}

/// A parsed document.
#[derive(Debug, Clone)]
pub struct Document {
    /// Root node.
    pub root: Node,
    /// JSON form for schema validation.
    pub json: serde_json::Value,
    /// Pointer → span index.
    pub spans: SpanMap,
    /// The source text.
    pub source: String,
}

/// Parse `src` into a [`Document`]. Fatal problems (syntax, limits) return `Err` with
/// diagnostics; non-fatal ones (duplicate keys) are pushed to `diags`.
pub fn parse(src: &str, diags: &mut Vec<Diagnostic>) -> Result<Document, Vec<Diagnostic>> {
    if src.len() > MAX_BYTES {
        return Err(vec![
            Diagnostic::error("DD-S008", format!("document is {} bytes; the limit is {MAX_BYTES}", src.len()))
                .with_help("split the description or remove embedded data (datasheets are referenced by URL, never embedded)"),
        ]);
    }
    let offsets = CharOffsets::new(src);
    let mut b = Builder {
        offsets: &offsets,
        stack: Vec::new(),
        anchors: HashMap::new(),
        docs: Vec::new(),
        errors: Vec::new(),
        aliases: 0,
    };
    let mut parser = Parser::new_from_str(src);
    if let Err(e) = parser.load(&mut b, true) {
        let at = offsets.byte(e.marker().index());
        let d = Diagnostic::error("DD-S001", format!("YAML syntax error: {}", e.info()))
            .at(Span::new(at, (at + 1).min(src.len())), "here");
        let d = match unclosed_flow(src, at) {
            Some((open, ch)) => {
                let close = if ch == '[' { ']' } else { '}' };
                d.with_related(Span::new(open, open + 1), format!("this `{ch}` is never closed"))
                    .with_help(format!("add the missing `{close}` to close the `{ch}` opened earlier"))
            }
            None => d.with_help(
                "check indentation (spaces only, no tabs) and that every `key:` is followed by a space",
            ),
        };
        return Err(vec![d]);
    }
    if !b.errors.is_empty() {
        return Err(b.errors);
    }
    let mut docs = b.docs;
    let root = match docs.len() {
        0 => {
            return Err(vec![
                Diagnostic::error("DD-S001", "the file is empty")
                    .with_help("a description starts with `spec: driverdesk/0.1`"),
            ]);
        }
        1 => docs.pop().expect("one document"),
        _ => {
            return Err(vec![
                Diagnostic::error(
                    "DD-S001",
                    "the file contains more than one YAML document (`---`)",
                )
                .at(docs[1].span, "second document starts here")
                .with_help("put each device description in its own file"),
            ]);
        }
    };
    let mut spans = SpanMap::default();
    let json = to_json(&root, String::new(), None, &mut spans, diags);
    Ok(Document {
        root,
        json,
        spans,
        source: src.to_string(),
    })
}

/// The last `[` or `{` before `upto` that is still open (ignoring quotes and comments).
fn unclosed_flow(src: &str, upto: usize) -> Option<(usize, char)> {
    let mut stack: Vec<(usize, char)> = Vec::new();
    let mut quote: Option<char> = None;
    let mut comment = false;
    for (i, c) in src[..upto.min(src.len())].char_indices() {
        if comment {
            comment = c != '\n';
            continue;
        }
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '#') => comment = true,
            (None, '[' | '{') => stack.push((i, c)),
            (None, ']' | '}') => {
                let opener = if c == ']' { '[' } else { '{' };
                match stack.last() {
                    Some(&(_, top)) if top == opener => {
                        stack.pop();
                    }
                    // `{ a: [1, 2 }`: the `[` on top is the one never closed.
                    Some(&top) => return Some(top),
                    None => {}
                }
            }
            _ => {}
        }
    }
    stack.pop()
}

/// Convert char indices (from saphyr) to byte offsets (for miette).
struct CharOffsets {
    bytes: Vec<usize>,
}

impl CharOffsets {
    fn new(src: &str) -> Self {
        let mut bytes: Vec<usize> = src.char_indices().map(|(b, _)| b).collect();
        bytes.push(src.len());
        Self { bytes }
    }
    fn byte(&self, char_index: usize) -> usize {
        *self
            .bytes
            .get(char_index)
            .unwrap_or_else(|| self.bytes.last().expect("non-empty"))
    }
}

enum Frame {
    Seq {
        items: Vec<Node>,
        start: usize,
        anchor: usize,
    },
    Map {
        entries: Vec<(Node, Node)>,
        pending_key: Option<Node>,
        start: usize,
        anchor: usize,
    },
}

struct Builder<'a> {
    offsets: &'a CharOffsets,
    stack: Vec<Frame>,
    anchors: HashMap<usize, Node>,
    docs: Vec<Node>,
    errors: Vec<Diagnostic>,
    aliases: usize,
}

impl Builder<'_> {
    fn span(&self, s: saphyr_parser::Span) -> Span {
        Span::new(
            self.offsets.byte(s.start.index()),
            self.offsets.byte(s.end.index()),
        )
    }

    fn push_node(&mut self, node: Node) {
        match self.stack.last_mut() {
            None => self.docs.push(node),
            Some(Frame::Seq { items, .. }) => items.push(node),
            Some(Frame::Map {
                entries,
                pending_key,
                ..
            }) => match pending_key.take() {
                None => *pending_key = Some(node),
                Some(k) => entries.push((k, node)),
            },
        }
    }
}

impl<'input> SpannedEventReceiver<'input> for Builder<'_> {
    fn on_event(&mut self, ev: Event<'input>, span: saphyr_parser::Span) {
        if !self.errors.is_empty() {
            return;
        }
        let sp = self.span(span);
        match ev {
            Event::Scalar(text, style, anchor, tag) => {
                if let Some(tag) = tag {
                    self.errors.push(
                        Diagnostic::error(
                            "DD-S009",
                            format!(
                                "YAML tags are not supported (`{}{}`)",
                                tag.handle, tag.suffix
                            ),
                        )
                        .at(sp, "remove the tag")
                        .with_help("write the value directly; quote it if it must be a string"),
                    );
                    return;
                }
                let plain = style == ScalarStyle::Plain;
                let value = if plain {
                    resolve_plain(&text)
                } else {
                    ScalarValue::Str(text.to_string())
                };
                let node = Node {
                    kind: NodeKind::Scalar(Scalar {
                        value,
                        text: text.to_string(),
                        plain,
                    }),
                    span: sp,
                };
                if anchor > 0 {
                    self.anchors.insert(anchor, node.clone());
                }
                self.push_node(node);
            }
            Event::SequenceStart(anchor, ref tag) | Event::MappingStart(anchor, ref tag) => {
                if let Some(tag) = tag {
                    self.errors.push(
                        Diagnostic::error(
                            "DD-S009",
                            format!(
                                "YAML tags are not supported (`{}{}`)",
                                tag.handle, tag.suffix
                            ),
                        )
                        .at(sp, "remove the tag")
                        .with_help("write the list or map without a tag"),
                    );
                    return;
                }
                if self.stack.len() >= MAX_DEPTH {
                    self.errors.push(
                        Diagnostic::error(
                            "DD-S008",
                            format!("nesting deeper than {MAX_DEPTH} levels"),
                        )
                        .at(sp, "too deep")
                        .with_help("descriptions never need this much nesting; flatten or remove the value that starts here"),
                    );
                    return;
                }
                let frame = if matches!(ev, Event::SequenceStart(..)) {
                    Frame::Seq {
                        items: Vec::new(),
                        start: sp.start,
                        anchor,
                    }
                } else {
                    Frame::Map {
                        entries: Vec::new(),
                        pending_key: None,
                        start: sp.start,
                        anchor,
                    }
                };
                self.stack.push(frame);
            }
            Event::SequenceEnd | Event::MappingEnd => {
                let Some(frame) = self.stack.pop() else {
                    return;
                };
                let (kind, start, anchor) = match frame {
                    Frame::Seq {
                        items,
                        start,
                        anchor,
                    } => (NodeKind::Seq(items), start, anchor),
                    Frame::Map {
                        entries,
                        start,
                        anchor,
                        ..
                    } => (NodeKind::Map(entries), start, anchor),
                };
                let end = sp.end.max(start);
                let node = Node {
                    kind,
                    span: Span::new(start, end),
                };
                if anchor > 0 {
                    self.anchors.insert(anchor, node.clone());
                }
                self.push_node(node);
            }
            Event::Alias(id) => {
                self.aliases += 1;
                if self.aliases > MAX_ALIASES {
                    self.errors.push(
                        Diagnostic::error(
                            "DD-S008",
                            format!("more than {MAX_ALIASES} alias expansions"),
                        )
                        .at(sp, "alias"),
                    );
                    return;
                }
                match self.anchors.get(&id) {
                    Some(n) => {
                        let mut n = n.clone();
                        n.span = sp;
                        self.push_node(n);
                    }
                    None => self.errors.push(
                        Diagnostic::error("DD-S001", "alias refers to an unknown anchor")
                            .at(sp, "here"),
                    ),
                }
            }
            Event::StreamStart
            | Event::StreamEnd
            | Event::DocumentStart(_)
            | Event::DocumentEnd
            | Event::Nothing => {}
        }
    }
}

/// Resolve a plain scalar with the YAML 1.2 core schema.
pub fn resolve_plain(t: &str) -> ScalarValue {
    match t {
        "" | "~" | "null" | "Null" | "NULL" => return ScalarValue::Null,
        "true" | "True" | "TRUE" => return ScalarValue::Bool(true),
        "false" | "False" | "FALSE" => return ScalarValue::Bool(false),
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => {
            return ScalarValue::Float(f64::INFINITY);
        }
        "-.inf" | "-.Inf" | "-.INF" => return ScalarValue::Float(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => return ScalarValue::Float(f64::NAN),
        _ => {}
    }
    if let Some(v) = parse_int(t) {
        return v;
    }
    if is_float(t)
        && let Ok(f) = t.parse::<f64>()
    {
        return ScalarValue::Float(f);
    }
    ScalarValue::Str(t.to_string())
}

fn parse_int(t: &str) -> Option<ScalarValue> {
    let (digits, radix) = if let Some(h) = t.strip_prefix("0x") {
        (h, 16)
    } else if let Some(o) = t.strip_prefix("0o") {
        (o, 8)
    } else {
        let d = t.strip_prefix('+').unwrap_or(t);
        let body = d.strip_prefix('-').unwrap_or(d);
        if body.is_empty() || !body.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        return match d.parse::<i64>() {
            Ok(v) => Some(ScalarValue::Int(v)),
            Err(_) => d.parse::<u64>().ok().map(ScalarValue::UInt),
        };
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    let v = u64::from_str_radix(digits, radix).ok()?;
    Some(i64::try_from(v).map_or(ScalarValue::UInt(v), ScalarValue::Int))
}

fn is_float(t: &str) -> bool {
    // [-+]? ( \. [0-9]+ | [0-9]+ ( \. [0-9]* )? ) ( [eE] [-+]? [0-9]+ )?
    let b = t.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let had_int = i > int_start;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if !had_int && i == fs {
            return false;
        }
    } else if !had_int {
        return false;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let es = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return false;
        }
    }
    i == b.len()
}

fn to_json(
    node: &Node,
    pointer: String,
    key: Option<Span>,
    spans: &mut SpanMap,
    diags: &mut Vec<Diagnostic>,
) -> serde_json::Value {
    use serde_json::Value;
    spans.map.insert(
        pointer.clone(),
        NodeSpans {
            key,
            value: node.span,
        },
    );
    match &node.kind {
        NodeKind::Scalar(s) => match &s.value {
            ScalarValue::Null => Value::Null,
            ScalarValue::Bool(b) => Value::Bool(*b),
            ScalarValue::Int(i) => Value::from(*i),
            ScalarValue::UInt(u) => Value::from(*u),
            ScalarValue::Float(f) => {
                serde_json::Number::from_f64(*f).map_or(Value::Null, Value::Number)
            }
            ScalarValue::Str(t) => Value::String(t.clone()),
        },
        NodeKind::Seq(items) => Value::Array(
            items
                .iter()
                .enumerate()
                .map(|(i, n)| to_json(n, format!("{pointer}/{i}"), None, spans, diags))
                .collect(),
        ),
        NodeKind::Map(entries) => {
            let mut obj = serde_json::Map::new();
            let mut first: HashMap<String, Span> = HashMap::new();
            for (k, v) in entries {
                let name = match &k.kind {
                    NodeKind::Scalar(s) => s.text.clone(),
                    _ => {
                        diags.push(
                            Diagnostic::error(
                                "DD-S003",
                                "mapping keys must be plain names, not lists or maps",
                            )
                            .at(k.span, "complex key"),
                        );
                        continue;
                    }
                };
                if let Some(prev) = first.get(&name) {
                    diags.push(
                        Diagnostic::error("DD-S007", format!("duplicate key `{name}`"))
                            .at(k.span, "duplicate")
                            .with_related(*prev, "first defined here")
                            .with_help(
                                "the validator uses the first value and ignores this one (some YAML tools keep the last instead); delete the one you do not want",
                            ),
                    );
                    continue;
                }
                first.insert(name.clone(), k.span);
                let child = format!("{pointer}/{}", escape_token(&name));
                let val = to_json(v, child, Some(k.span), spans, diags);
                obj.insert(name, val);
            }
            Value::Object(obj)
        }
    }
}

/// Visit every node with its JSON pointer and, for mapping values, the key node.
pub fn walk<'a>(
    node: &'a Node,
    pointer: &str,
    key: Option<&'a Node>,
    f: &mut dyn FnMut(&str, &'a Node, Option<&'a Node>),
) {
    f(pointer, node, key);
    match &node.kind {
        NodeKind::Scalar(_) => {}
        NodeKind::Seq(items) => {
            for (i, n) in items.iter().enumerate() {
                walk(n, &format!("{pointer}/{i}"), None, f);
            }
        }
        NodeKind::Map(entries) => {
            for (k, v) in entries {
                if let NodeKind::Scalar(s) = &k.kind {
                    walk(
                        v,
                        &format!("{pointer}/{}", escape_token(&s.text)),
                        Some(k),
                        f,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(src: &str) -> (Document, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let doc = parse(src, &mut d).expect("parses");
        (doc, d)
    }

    #[test]
    fn resolves_yaml12_core_scalars() {
        assert_eq!(resolve_plain("0x1F"), ScalarValue::Int(31));
        assert_eq!(resolve_plain("0o17"), ScalarValue::Int(15));
        assert_eq!(resolve_plain("-42"), ScalarValue::Int(-42));
        assert_eq!(resolve_plain("1.5e3"), ScalarValue::Float(1500.0));
        assert_eq!(
            resolve_plain("on"),
            ScalarValue::Str("on".into()),
            "YAML 1.2: `on` is a string"
        );
        assert_eq!(
            resolve_plain("0b101"),
            ScalarValue::Str("0b101".into()),
            "binary is not YAML 1.2 core"
        );
        assert_eq!(
            resolve_plain("18446744073709551615"),
            ScalarValue::UInt(u64::MAX)
        );
        assert_eq!(resolve_plain("~"), ScalarValue::Null);
        assert_eq!(resolve_plain("1.2.3"), ScalarValue::Str("1.2.3".into()));
    }

    #[test]
    fn spans_are_byte_offsets_after_multibyte_text() {
        let src = "name: \"µ°C\"\nid: 7\n";
        let (doc, _) = ok(src);
        let s = doc.spans.get("/id").unwrap();
        assert_eq!(&src[s.key.unwrap().start..s.key.unwrap().end], "id");
        assert_eq!(&src[s.value.start..s.value.end], "7");
    }

    #[test]
    fn duplicate_keys_are_reported_with_both_locations() {
        let (doc, d) = ok("a: 1\nb: 2\na: 3\n");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].code, "DD-S007");
        assert_eq!(d[0].related.len(), 1);
        assert_eq!(doc.json["a"], 1, "first value wins");
    }

    #[test]
    fn syntax_error_has_position() {
        let mut d = Vec::new();
        let e = parse("a: [1, 2\nb: 3\n", &mut d).unwrap_err();
        assert_eq!(e[0].code, "DD-S001");
        assert!(e[0].span.is_some());
    }

    #[test]
    fn unclosed_bracket_inside_braces_is_the_bracket() {
        let src = "a: { b: [1, 2 }\nc: 3\n";
        let mut d = Vec::new();
        let e = parse(src, &mut d).unwrap_err();
        let (span, label) = &e[0].related[0];
        assert_eq!(&src[span.start..span.end], "[", "{label}");
        assert!(e[0].help.as_deref().unwrap().contains("`]`"));
    }

    #[test]
    fn aliases_expand_and_are_limited() {
        let (doc, _) = ok("a: &x { k: 1 }\nb: *x\n");
        assert_eq!(doc.json["b"]["k"], 1);
        let mut bomb = String::from("a: &a [1]\n");
        for i in 0..1100 {
            bomb.push_str(&format!("k{i}: *a\n"));
        }
        let mut d = Vec::new();
        assert_eq!(parse(&bomb, &mut d).unwrap_err()[0].code, "DD-S008");
    }

    #[test]
    fn depth_is_limited() {
        let src = "[".repeat(100) + &"]".repeat(100);
        let mut d = Vec::new();
        assert_eq!(parse(&src, &mut d).unwrap_err()[0].code, "DD-S008");
    }

    #[test]
    fn tags_are_rejected() {
        let mut d = Vec::new();
        assert_eq!(
            parse("a: !!binary aGk=\n", &mut d).unwrap_err()[0].code,
            "DD-S009"
        );
        assert_eq!(
            parse("a: !!set { x: 1 }\n", &mut d).unwrap_err()[0].code,
            "DD-S009"
        );
        assert_eq!(
            parse("a: !custom [1]\n", &mut d).unwrap_err()[0].code,
            "DD-S009"
        );
    }

    #[test]
    fn multiple_documents_rejected() {
        let mut d = Vec::new();
        assert_eq!(
            parse("a: 1\n---\nb: 2\n", &mut d).unwrap_err()[0].code,
            "DD-S001"
        );
    }

    #[test]
    fn pointer_escaping() {
        let (doc, _) = ok("\"a/b\": { \"c~d\": 1 }\n");
        assert!(doc.spans.get("/a~1b/c~0d").is_some());
    }
}
