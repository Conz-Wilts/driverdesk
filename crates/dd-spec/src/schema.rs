//! JSON Schema validation against the embedded `spec/schema/0.1` files, with errors mapped to
//! spanned [`Diagnostic`]s (`DD-S002`…`DD-S009`).

use std::collections::HashMap;
use std::sync::OnceLock;

use jsonschema::error::{TypeKind, ValidationErrorKind};
use jsonschema::{Registry, ValidationError, Validator};
use serde_json::Value;

use crate::diag::{Diagnostic, suggest};
use crate::syntax::{Document, escape_token};

/// `$id` prefix of every 0.1 schema file.
pub const BASE: &str = "https://driverdesk.dev/schema/0.1/";

/// The schema files, embedded at build time: `(file name, contents)`.
pub const FILES: &[(&str, &str)] = &[
    (
        "description.json",
        include_str!("../../../spec/schema/0.1/description.json"),
    ),
    (
        "common.json",
        include_str!("../../../spec/schema/0.1/common.json"),
    ),
    (
        "identity.json",
        include_str!("../../../spec/schema/0.1/identity.json"),
    ),
    (
        "match.json",
        include_str!("../../../spec/schema/0.1/match.json"),
    ),
    (
        "transport.json",
        include_str!("../../../spec/schema/0.1/transport.json"),
    ),
    (
        "registers.json",
        include_str!("../../../spec/schema/0.1/registers.json"),
    ),
    (
        "commands.json",
        include_str!("../../../spec/schema/0.1/commands.json"),
    ),
    (
        "states.json",
        include_str!("../../../spec/schema/0.1/states.json"),
    ),
    (
        "safety.json",
        include_str!("../../../spec/schema/0.1/safety.json"),
    ),
    (
        "expose.json",
        include_str!("../../../spec/schema/0.1/expose.json"),
    ),
    (
        "complex.json",
        include_str!("../../../spec/schema/0.1/complex.json"),
    ),
];

struct Compiled {
    validator: Validator,
    resources: HashMap<String, Value>,
}

fn compiled() -> &'static Compiled {
    static C: OnceLock<Compiled> = OnceLock::new();
    C.get_or_init(|| {
        let mut resources = HashMap::new();
        for (name, text) in FILES {
            let v: Value = serde_json::from_str(text)
                .unwrap_or_else(|e| panic!("embedded schema {name} is not JSON: {e}"));
            resources.insert(format!("{BASE}{name}"), v);
        }
        let validator = {
            let mut registry = Registry::new();
            for (uri, v) in &resources {
                registry = registry
                    .add(uri.as_str(), v)
                    .unwrap_or_else(|e| panic!("schema {uri}: {e}"));
            }
            let registry = registry.prepare().expect("schema registry");
            let root = &resources[&format!("{BASE}description.json")];
            jsonschema::options()
                .with_registry(&registry)
                .offline()
                .build(root)
                .expect("description schema compiles")
        };
        Compiled {
            validator,
            resources,
        }
    })
}

/// Validate a parsed document; returns spanned diagnostics (empty if valid).
pub fn validate(doc: &Document) -> Vec<Diagnostic> {
    let c = compiled();
    let mut out = Vec::new();
    for e in c.validator.iter_errors(&doc.json) {
        convert(&e, doc, c, &mut out);
    }
    dedup(out)
}

fn dedup(mut v: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let mut seen = std::collections::HashSet::new();
    v.retain(|d| seen.insert((d.code, d.message.clone(), d.pointer.clone())));
    v
}

/// Resolve a schema URI (with optional `#/json/pointer` fragment) to its JSON, following `$ref`.
fn resolve<'a>(c: &'a Compiled, uri: &str) -> Option<(&'a Value, String)> {
    let (base, frag) = uri.split_once('#').unwrap_or((uri, ""));
    let doc = c.resources.get(base)?;
    let v = if frag.is_empty() {
        doc
    } else {
        doc.pointer(frag)?
    };
    Some(follow_ref(c, v, base))
}

fn follow_ref<'a>(c: &'a Compiled, v: &'a Value, base: &str) -> (&'a Value, String) {
    if let Some(r) = v.get("$ref").and_then(Value::as_str) {
        let abs = if r.starts_with('#') {
            format!("{base}{r}")
        } else if r.starts_with("http") {
            r.to_string()
        } else {
            format!("{BASE}{r}")
        };
        let abs_base = abs.split('#').next().unwrap_or(&abs).to_string();
        if let Some((t, _)) = resolve(c, &abs) {
            return follow_ref(c, t, &abs_base);
        }
    }
    (v, base.to_string())
}

/// The schema object that contains the failing keyword.
fn parent_schema<'a>(c: &'a Compiled, e: &ValidationError<'_>) -> Option<(&'a Value, String)> {
    let loc = e.absolute_keyword_location()?.as_str().to_string();
    let parent = loc.rsplit_once('/').map(|(p, _)| p.to_string())?;
    resolve(c, &parent)
}

fn pointer_of(e: &ValidationError<'_>) -> String {
    e.instance_path().as_str().to_string()
}

fn where_(ptr: &str) -> String {
    if ptr.is_empty() {
        "at the top level".to_string()
    } else {
        format!("in `{}`", display_path(ptr))
    }
}

/// `/commands/read_temp/steps/0` → `commands.read_temp.steps[0]`.
pub fn display_path(ptr: &str) -> String {
    let mut s = String::new();
    for tok in ptr.split('/').skip(1) {
        let tok = tok.replace("~1", "/").replace("~0", "~");
        if tok.chars().all(|c| c.is_ascii_digit()) && !tok.is_empty() {
            s.push_str(&format!("[{tok}]"));
        } else {
            if !s.is_empty() {
                s.push('.');
            }
            s.push_str(&tok);
        }
    }
    s
}

fn json_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "nothing (null)",
        Value::Bool(_) => "a boolean",
        Value::Number(n) if n.is_i64() || n.is_u64() => "an integer",
        Value::Number(_) => "a number",
        Value::String(_) => "text",
        Value::Array(_) => "a list",
        Value::Object(_) => "a map",
    }
}

fn type_name(k: &TypeKind) -> String {
    let one = |t: &str| match t {
        "integer" => "an integer".to_string(),
        "number" => "a number".to_string(),
        "string" => "text".to_string(),
        "boolean" => "true or false".to_string(),
        "array" => "a list".to_string(),
        "object" => "a map (`key: value` entries)".to_string(),
        "null" => "null".to_string(),
        o => o.to_string(),
    };
    match k {
        TypeKind::Single(t) => one(&t.to_string()),
        TypeKind::Multiple(set) => {
            let v: Vec<String> = set.iter().map(|t| one(&t.to_string())).collect();
            v.join(" or ")
        }
    }
}

/// Human description of well-known patterns from `common.json`.
fn pattern_help(pattern: &str) -> &'static str {
    match pattern {
        "^[a-z][a-z0-9_]*$" => {
            "names use lower_snake_case: a lower-case letter first, then letters, digits or `_`"
        }
        "^[A-Za-z][A-Za-z0-9_]*$" => {
            "register names start with a letter, then letters, digits or `_`"
        }
        p if p.starts_with("^(bool|string|[ui]") => {
            "scalar types are bool, string, u8, i8, u16, i16, u24, i24, u32, i32, u64, i64, f32, f64 (multi-byte types may end in `le` or `be`, e.g. `i16le`)"
        }
        p if p.starts_with("^(bool|[ui]") => {
            "field types are scalar types (`u8`, `i16le`, `f32be`, …), `bytes`, `bytes(N)`, `string(N)`, `bits(N)` or a name from `types`"
        }
        p if p.starts_with("^[a-z0-9][a-z0-9-]*(\\.") => {
            "device ids are lower-case and dot-separated with the vendor prefix first, e.g. `ti.tmp117`"
        }
        p if p.starts_with("^(0|[1-9]") => "use a semantic version such as `0.1.0`",
        p if p.contains(",[a-z0-9._+-]+$") => {
            "compatible strings are `vendor,part`, e.g. `ti,tmp117` (quote them inside `[ ]` lists)"
        }
        "^[0-9a-f]{64}$" => "a SHA-256 is 64 lower-case hex characters",
        p if p.contains("(:([0-9]") => {
            "bit ranges are `\"H:L\"` (inclusive, LSB = 0) or a single bit `\"N\"`, quoted, for bits 0–63"
        }
        p if p.contains("/(s|min|h)$") => "rates are written `<count>/<s|min|h>`, e.g. `100/s`",
        p if p.contains("(\\.[A-Za-z]") => {
            "references are names joined by dots, e.g. `read_temp.celsius` or `CONFIG.mode`"
        }
        _ => "the value does not have the expected form",
    }
}

fn section_for(ptr: &str) -> &'static str {
    match ptr.split('/').nth(1).unwrap_or("") {
        "spec" => "§3.1",
        "device" | "variants" | "instance" => "§4",
        "match" => "§5",
        "transport" => "§6",
        "types" => "§7",
        "registers" => "§8",
        "commands" | "vars" => "§9",
        "states" | "sequences" | "settings" => "§12",
        "timing" => "§13",
        "safety" => "§14",
        "expose" => "§15",
        "tests" => "§16",
        "visibility" => "§17",
        "events" | "interrupts" | "memory" | "queues" | "streams" | "firmware" => "§20",
        "logic" => "§21",
        _ => "§3",
    }
}

fn convert(e: &ValidationError<'_>, doc: &Document, c: &Compiled, out: &mut Vec<Diagnostic>) {
    let ptr = pointer_of(e);
    let spans = &doc.spans;
    let sec = section_for(&ptr);
    match e.kind() {
        ValidationErrorKind::AdditionalProperties { unexpected } => {
            let allowed: Vec<String> = parent_schema(c, e)
                .and_then(|(s, _)| {
                    s.get("properties")
                        .and_then(Value::as_object)
                        .map(|m| m.keys().cloned().collect())
                })
                .unwrap_or_default();
            for u in unexpected {
                let child = format!("{ptr}/{}", escape_token(u));
                let help = match suggest(u, allowed.iter().map(String::as_str)) {
                    Some(s) => format!("did you mean `{s}`?"),
                    None if allowed.is_empty() => format!("remove it (see spec {sec})"),
                    None => format!("allowed here: {} (spec {sec})", short_list(&allowed)),
                };
                out.push(
                    Diagnostic::error("DD-S004", format!("unknown field `{u}` {}", where_(&ptr)))
                        .at_opt(spans.key_or_value(&child), "not allowed here")
                        .with_help(help)
                        .with_pointer(child),
                );
            }
        }
        ValidationErrorKind::Required { property } => {
            let p = property.as_str().unwrap_or("?");
            let sec = section_for(&format!("{ptr}/{p}"));
            let at = if ptr.is_empty() {
                doc.spans.first_key()
            } else {
                spans.key_or_value(&ptr)
            };
            out.push(
                Diagnostic::error(
                    "DD-S002",
                    format!("missing required field `{p}` {}", where_(&ptr)),
                )
                .at_opt(
                    at,
                    format!(
                        "add `{p}:` {}",
                        if ptr.is_empty() {
                            "to the document"
                        } else {
                            "here"
                        }
                    ),
                )
                .with_help(format!("`{p}` is required (spec {sec})"))
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::Type { kind } => {
            out.push(
                Diagnostic::error(
                    "DD-S003",
                    format!(
                        "`{}` must be {}, found {}",
                        display_path(&ptr),
                        type_name(kind),
                        json_type(e.instance())
                    ),
                )
                .at_opt(spans.value_or_ancestor(&ptr), "wrong type")
                .with_help(type_help(kind, e.instance()))
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::Enum { options } => {
            let opts: Vec<String> = options
                .as_array()
                .map(|a| a.iter().map(scalar_text).collect())
                .unwrap_or_default();
            let got = scalar_text(e.instance());
            let help = match suggest(&got, opts.iter().map(String::as_str)) {
                Some(s) => format!("did you mean `{s}`?"),
                None => format!("expected one of: {}", short_list(&opts)),
            };
            out.push(
                Diagnostic::error(
                    "DD-S005",
                    format!(
                        "`{got}` is not an allowed value for `{}`",
                        display_path(&ptr)
                    ),
                )
                .at_opt(spans.value_or_ancestor(&ptr), "not allowed")
                .with_help(help)
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::Constant { expected_value } => {
            if ptr == "/spec" {
                // Reported by lint DD-L010 with an upgrade hint.
                return;
            }
            out.push(
                Diagnostic::error(
                    "DD-S005",
                    format!(
                        "`{}` must be `{}`",
                        display_path(&ptr),
                        scalar_text(expected_value)
                    ),
                )
                .at_opt(spans.value_or_ancestor(&ptr), "wrong value")
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::Pattern { pattern } => {
            let got = scalar_text(e.instance());
            // A pattern failure on a property name comes through PropertyNames below.
            out.push(
                Diagnostic::error(
                    "DD-S005",
                    format!("`{got}` is not a valid value for `{}`", display_path(&ptr)),
                )
                .at_opt(spans.value_or_ancestor(&ptr), "invalid")
                .with_help(pattern_help(pattern))
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::PropertyNames { error } => {
            let name = error.instance().as_str().unwrap_or("?").to_string();
            let child = format!("{ptr}/{}", escape_token(&name));
            let help = match error.kind() {
                ValidationErrorKind::Pattern { pattern } => pattern_help(pattern).to_string(),
                ValidationErrorKind::MaxLength { limit } => {
                    format!("names are at most {limit} characters")
                }
                _ => "rename it".to_string(),
            };
            out.push(
                Diagnostic::error(
                    "DD-S005",
                    format!("`{name}` is not a valid name {}", where_(&ptr)),
                )
                .at_opt(spans.key_or_value(&child), "invalid name")
                .with_help(help)
                .with_pointer(child),
            );
        }
        ValidationErrorKind::Minimum { limit }
        | ValidationErrorKind::ExclusiveMinimum { limit } => {
            out.push(range_diag(
                &ptr,
                e.instance(),
                "below the minimum",
                limit,
                doc,
            ));
        }
        ValidationErrorKind::Maximum { limit }
        | ValidationErrorKind::ExclusiveMaximum { limit } => {
            out.push(range_diag(
                &ptr,
                e.instance(),
                "above the maximum",
                limit,
                doc,
            ));
        }
        ValidationErrorKind::MinItems { .. } | ValidationErrorKind::MinProperties { .. } => {
            out.push(
                Diagnostic::error(
                    "DD-S002",
                    format!("`{}` must not be empty", display_path(&ptr)),
                )
                .at_opt(spans.key_or_value(&ptr), "empty")
                .with_help(format!("add at least one entry (spec {sec})"))
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::MaxItems { limit } | ValidationErrorKind::MaxProperties { limit } => {
            out.push(
                Diagnostic::error(
                    "DD-S006",
                    format!(
                        "`{}` has too many entries (at most {limit})",
                        display_path(&ptr)
                    ),
                )
                .at_opt(spans.key_or_value(&ptr), "too many")
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::MinLength { .. } => {
            out.push(
                Diagnostic::error(
                    "DD-S002",
                    format!("`{}` must not be empty", display_path(&ptr)),
                )
                .at_opt(spans.value_or_ancestor(&ptr), "empty")
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::MaxLength { limit } => {
            out.push(
                Diagnostic::error(
                    "DD-S006",
                    format!("`{}` is longer than {limit} characters", display_path(&ptr)),
                )
                .at_opt(spans.value_or_ancestor(&ptr), "too long")
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::UniqueItems => {
            out.push(
                Diagnostic::error(
                    "DD-S007",
                    format!("`{}` contains the same entry twice", display_path(&ptr)),
                )
                .at_opt(spans.value_or_ancestor(&ptr), "duplicate entries")
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::OneOfNotValid { context } | ValidationErrorKind::AnyOf { context } => {
            one_of(e, context, doc, c, out);
        }
        ValidationErrorKind::OneOfMultipleValid { .. } => {
            out.push(
                Diagnostic::error(
                    "DD-S009",
                    format!(
                        "`{}` is ambiguous: it matches more than one allowed form",
                        display_path(&ptr)
                    ),
                )
                .at_opt(spans.key_or_value(&ptr), "ambiguous")
                .with_help(format!(
                    "use exactly one of the alternatives described in spec {sec}"
                ))
                .with_pointer(ptr),
            );
        }
        ValidationErrorKind::Not { .. } => {
            out.push(
                Diagnostic::error(
                    "DD-S009",
                    format!(
                        "`{}` combines fields that cannot be used together",
                        display_path(&ptr)
                    ),
                )
                .at_opt(spans.key_or_value(&ptr), "conflicting fields")
                .with_help(format!("see spec {sec}"))
                .with_pointer(ptr),
            );
        }
        _ => {
            out.push(
                Diagnostic::error(
                    "DD-S009",
                    format!("`{}` is invalid: {e}", display_path(&ptr)),
                )
                .at_opt(spans.value_or_ancestor(&ptr), "invalid")
                .with_pointer(ptr),
            );
        }
    }
}

fn type_help(kind: &TypeKind, got: &Value) -> String {
    let want = type_name(kind);
    match got {
        Value::String(s) if want.contains("integer") && s.starts_with("0b") => {
            "binary literals are not YAML 1.2; write hex, e.g. `0x0A`".to_string()
        }
        Value::String(s) if want.contains("integer") && s.parse::<f64>().is_ok() => {
            "remove the quotes so the value is a number".to_string()
        }
        Value::Number(_) if want.starts_with("text") => {
            "quote the value if it must be text".to_string()
        }
        _ => format!("write {want}"),
    }
}

fn range_diag(ptr: &str, got: &Value, what: &str, limit: &Value, doc: &Document) -> Diagnostic {
    Diagnostic::error(
        "DD-S006",
        format!(
            "`{}` = {} is {what} ({})",
            display_path(ptr),
            scalar_text(got),
            scalar_text(limit)
        ),
    )
    .at_opt(doc.spans.value_or_ancestor(ptr), "out of range")
    .with_pointer(ptr.to_string())
}

fn scalar_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn short_list(items: &[String]) -> String {
    let mut v: Vec<&str> = items
        .iter()
        .map(String::as_str)
        .filter(|s| !s.starts_with("x-"))
        .collect();
    v.sort_unstable();
    if v.len() > 14 {
        format!("{}, … ({} more)", v[..14].join(", "), v.len() - 14)
    } else {
        v.join(", ")
    }
}

/// The key that tells `oneOf` branches apart: a property with `const`, else the first
/// required property. Returns `(key, Some(const value))`.
fn tag_of(branch: &Value) -> Option<(String, Option<Value>)> {
    if let Some(props) = branch.get("properties").and_then(Value::as_object) {
        for (k, v) in props {
            if let Some(cv) = v.get("const") {
                return Some((k.clone(), Some(cv.clone())));
            }
        }
    }
    let req = branch.get("required")?.as_array()?;
    req.first()
        .and_then(Value::as_str)
        .map(|s| (s.to_string(), None))
}

fn one_of(
    e: &ValidationError<'_>,
    context: &[Vec<ValidationError<'static>>],
    doc: &Document,
    c: &Compiled,
    out: &mut Vec<Diagnostic>,
) {
    let ptr = pointer_of(e);
    let inst = e.instance();
    let branches: Vec<(Value, String)> = e
        .absolute_keyword_location()
        .and_then(|loc| resolve(c, loc.as_str()))
        .and_then(|(v, base)| {
            v.as_array().map(|a| {
                a.iter()
                    .map(|b| {
                        let (r, rb) = follow_ref(c, b, &base);
                        (r.clone(), rb)
                    })
                    .collect()
            })
        })
        .unwrap_or_default();

    // 1. Discriminated union on an object: pick the branch whose tag the author used.
    if let Value::Object(obj) = &**inst {
        let tags: Vec<Option<(String, Option<Value>)>> =
            branches.iter().map(|(b, _)| tag_of(b)).collect();
        if !tags.is_empty() && tags.iter().all(Option::is_some) {
            let tags: Vec<(String, Option<Value>)> = tags.into_iter().map(Option::unwrap).collect();
            let chosen = tags.iter().position(|(k, cv)| match cv {
                Some(cv) => obj.get(k) == Some(cv),
                None => obj.contains_key(k),
            });
            if let Some(errs) = chosen.and_then(|i| context.get(i)) {
                for sub in errs {
                    convert(sub, doc, c, out);
                }
                return;
            }
            // No tag matched: name the tag.
            let const_key = tags
                .iter()
                .find_map(|(k, cv)| cv.as_ref().map(|_| k.clone()));
            if let Some(k) = const_key {
                let allowed: Vec<String> = tags
                    .iter()
                    .filter_map(|(_, cv)| cv.as_ref().map(scalar_text))
                    .collect();
                let child = format!("{ptr}/{}", escape_token(&k));
                match obj.get(&k) {
                    Some(v) => {
                        let got = scalar_text(v);
                        let help = suggest(&got, allowed.iter().map(String::as_str)).map_or_else(
                            || format!("expected one of: {}", short_list(&allowed)),
                            |s| format!("did you mean `{s}`?"),
                        );
                        out.push(
                            Diagnostic::error(
                                "DD-S005",
                                format!("unknown {k} `{got}` {}", where_(&ptr)),
                            )
                            .at_opt(doc.spans.value_or_ancestor(&child), "unknown")
                            .with_help(help)
                            .with_pointer(child),
                        );
                    }
                    None => out.push(
                        Diagnostic::error(
                            "DD-S002",
                            format!("missing required field `{k}` {}", where_(&ptr)),
                        )
                        .at_opt(doc.spans.key_or_value(&ptr), format!("add `{k}:`"))
                        .with_help(format!("one of: {}", short_list(&allowed)))
                        .with_pointer(ptr),
                    ),
                }
                return;
            }
            let keys: Vec<String> = tags.iter().map(|(k, _)| k.clone()).collect();
            // Keys that some branch knows (like `as`) are not the typo; look at the others.
            let known: std::collections::HashSet<&str> = branches
                .iter()
                .filter_map(|(b, _)| b.get("properties").and_then(Value::as_object))
                .flat_map(|m| m.keys().map(String::as_str))
                .collect();
            let present: Vec<&String> =
                obj.keys().filter(|k| !known.contains(k.as_str())).collect();
            let guess = present.iter().find_map(|p| {
                suggest(p, keys.iter().map(String::as_str)).map(|s| (p.to_string(), s.to_string()))
            });
            let (msg, help, at) = match guess {
                Some((p, s)) => (
                    format!("unknown field `{p}` {}", where_(&ptr)),
                    format!("did you mean `{s}`?"),
                    format!("{ptr}/{}", escape_token(&p)),
                ),
                None => (
                    format!(
                        "`{}` must contain exactly one of: {}",
                        display_path(&ptr),
                        short_list(&keys)
                    ),
                    format!("see spec {}", section_for(&ptr)),
                    ptr.clone(),
                ),
            };
            out.push(
                Diagnostic::error("DD-S004", msg)
                    .at_opt(doc.spans.key_or_value(&at), "unrecognised")
                    .with_help(help)
                    .with_pointer(at),
            );
            return;
        }
    }

    // 2. Otherwise: the branch whose errors are all deeper than this node (it fit the shape).
    let depth = |er: &ValidationError<'_>| er.instance_path().as_str().len();
    let here = ptr.len();
    if let Some(best) = context
        .iter()
        .filter(|errs| !errs.is_empty() && errs.iter().all(|er| depth(er) > here))
        .min_by_key(|errs| errs.len())
    {
        for sub in best {
            convert(sub, doc, c, out);
        }
        return;
    }

    // 3. Fall back to a description of the accepted forms.
    let forms: Vec<String> = branches.iter().map(|(b, _)| form_name(b)).collect();
    out.push(
        Diagnostic::error(
            "DD-S003",
            format!(
                "`{}` has the wrong form: found {}",
                display_path(&ptr),
                json_type(inst)
            ),
        )
        .at_opt(doc.spans.value_or_ancestor(&ptr), "wrong form")
        .with_help(format!("expected {}", dedup_forms(forms)))
        .with_pointer(ptr),
    );
}

fn form_name(b: &Value) -> String {
    if b.get("enum").is_some() {
        return "one of the listed words".to_string();
    }
    if let Some(cv) = b.get("const") {
        return format!("`{}`", scalar_text(cv));
    }
    match b.get("type").and_then(Value::as_str) {
        Some("integer") => "an integer".to_string(),
        Some("number") => "a number".to_string(),
        Some("string") => "text (e.g. a CEL expression in quotes)".to_string(),
        Some("boolean") => "true/false".to_string(),
        Some("array") => "a list".to_string(),
        Some("object") => match b
            .get("required")
            .and_then(Value::as_array)
            .and_then(|r| r.first())
            .and_then(Value::as_str)
        {
            Some(k) => format!("a map with `{k}:`"),
            None => "a map".to_string(),
        },
        _ => "another form".to_string(),
    }
}

fn dedup_forms(mut v: Vec<String>) -> String {
    let mut seen = std::collections::HashSet::new();
    v.retain(|s| seen.insert(s.clone()));
    v.join(", or ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax;

    fn check(src: &str) -> Vec<Diagnostic> {
        let mut d = Vec::new();
        let doc = syntax::parse(src, &mut d).expect("yaml");
        validate(&doc)
    }

    #[test]
    fn schemas_compile() {
        let _ = compiled();
    }

    #[test]
    fn display_path_formats() {
        assert_eq!(
            display_path("/commands/read_temp/steps/0"),
            "commands.read_temp.steps[0]"
        );
        assert_eq!(display_path(""), "");
    }

    #[test]
    fn unknown_top_level_field_suggests() {
        let d = check("spec: driverdesk/0.1\ndevcie: {}\n");
        let u = d
            .iter()
            .find(|d| d.code == "DD-S004")
            .expect("unknown field");
        assert_eq!(u.help.as_deref(), Some("did you mean `device`?"));
        assert_eq!(u.pointer.as_deref(), Some("/devcie"));
    }

    #[test]
    fn unknown_bus_is_named() {
        let d = check("match:\n  - bus: usbb\n    vid: 1\n");
        let m = d
            .iter()
            .find(|d| d.pointer.as_deref() == Some("/match/0/bus"))
            .expect("bus diag");
        assert_eq!(m.help.as_deref(), Some("did you mean `usb`?"));
    }

    #[test]
    fn unknown_step_operation_is_named() {
        let d =
            check("commands:\n  c:\n    steps:\n      - { reed: X, as: y }\n    timeout_ms: 5\n");
        let s = d
            .iter()
            .find(|d| d.pointer.as_deref() == Some("/commands/c/steps/0/reed"))
            .unwrap_or_else(|| panic!("{d:#?}"));
        assert_eq!(s.help.as_deref(), Some("did you mean `read`?"));
    }
}
