//! Diagnostics: stable codes (spec Appendix B), source spans, one-line help and a docs link.

use std::fmt;

use serde::Serialize;

/// Base URL for per-code documentation pages (provisional domain, spec Appendix B).
pub const DOCS_BASE: &str = "https://driverdesk.dev/errors/";

/// A byte range in the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    /// Start byte offset.
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
}

impl Span {
    /// New span; `end` is clamped to be at least `start`.
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end: end.max(start),
        }
    }
}

/// How bad a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The description is still valid.
    Warning,
    /// The description is invalid.
    Error,
}

/// One finding about a description.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    /// Stable code, e.g. `DD-S004` or `DD-L003`.
    pub code: &'static str,
    /// Severity.
    pub severity: Severity,
    /// One-sentence statement of the problem.
    pub message: String,
    /// Primary location.
    pub span: Option<Span>,
    /// Label shown at the primary location.
    pub label: Option<String>,
    /// One-line advice on how to fix it.
    pub help: Option<String>,
    /// Secondary locations.
    pub related: Vec<(Span, String)>,
    /// JSON pointer of the offending node, when known.
    pub pointer: Option<String>,
}

impl Diagnostic {
    /// New error.
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            span: None,
            label: None,
            help: None,
            related: Vec::new(),
            pointer: None,
        }
    }

    /// New warning.
    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(code, message)
        }
    }

    /// Attach the primary location.
    pub fn at(mut self, span: Span, label: impl Into<String>) -> Self {
        self.span = Some(span);
        self.label = Some(label.into());
        self
    }

    /// Attach the primary location if known.
    pub fn at_opt(self, span: Option<Span>, label: impl Into<String>) -> Self {
        match span {
            Some(s) => self.at(s, label),
            None => self,
        }
    }

    /// Attach help text.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Attach a secondary location.
    pub fn with_related(mut self, span: Span, label: impl Into<String>) -> Self {
        self.related.push((span, label.into()));
        self
    }

    /// Record the JSON pointer.
    pub fn with_pointer(mut self, pointer: impl Into<String>) -> Self {
        self.pointer = Some(pointer.into());
        self
    }

    /// Documentation URL for this code.
    pub fn url(&self) -> String {
        format!("{DOCS_BASE}{}", self.code)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// A diagnostic bound to its source, renderable by miette.
#[derive(Debug)]
pub struct Rendered {
    diag: Diagnostic,
    src: miette::NamedSource<String>,
}

impl Rendered {
    /// Bind `diag` to the file `name` with contents `src`. Control characters from the
    /// (untrusted) file are neutralised so they cannot drive the terminal.
    pub fn new(diag: &Diagnostic, name: &str, src: &str) -> Self {
        let mut diag = diag.clone();
        diag.message = clean_text(&diag.message);
        diag.label = diag.label.as_deref().map(clean_text);
        diag.help = diag.help.as_deref().map(clean_text);
        for (_, l) in &mut diag.related {
            *l = clean_text(l);
        }
        Self {
            diag,
            src: miette::NamedSource::new(clean_text(name), clean_source(src)),
        }
    }
}

fn is_dangerous(c: char) -> bool {
    c.is_control() && c != '\n' && c != '\t'
}

/// Escape control characters (ESC, BEL, C1 …) as `\u{..}`; keep newlines and tabs.
pub fn clean_text(s: &str) -> String {
    s.chars()
        .map(|c| {
            if is_dangerous(c) && c != '\r' {
                c.escape_unicode().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// Replace control characters with `?` of the same UTF-8 length, so byte spans stay valid.
pub fn clean_source(s: &str) -> String {
    s.chars()
        .map(|c| {
            if is_dangerous(c) && c != '\r' {
                "?".repeat(c.len_utf8())
            } else {
                c.to_string()
            }
        })
        .collect()
}

impl fmt::Display for Rendered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.diag, f)
    }
}

impl std::error::Error for Rendered {}

impl miette::Diagnostic for Rendered {
    fn code<'b>(&'b self) -> Option<Box<dyn fmt::Display + 'b>> {
        Some(Box::new(self.diag.code))
    }
    fn severity(&self) -> Option<miette::Severity> {
        Some(match self.diag.severity {
            Severity::Error => miette::Severity::Error,
            Severity::Warning => miette::Severity::Warning,
        })
    }
    fn help<'b>(&'b self) -> Option<Box<dyn fmt::Display + 'b>> {
        self.diag
            .help
            .as_ref()
            .map(|h| Box::new(h) as Box<dyn fmt::Display>)
    }
    fn url<'b>(&'b self) -> Option<Box<dyn fmt::Display + 'b>> {
        Some(Box::new(self.diag.url()))
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.diag.span.map(|_| &self.src as &dyn miette::SourceCode)
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let primary = self.diag.span.map(|s| {
            miette::LabeledSpan::new_primary_with_span(
                self.diag.label.clone(),
                (s.start, s.end - s.start),
            )
        });
        let related = self.diag.related.iter().map(|(s, l)| {
            miette::LabeledSpan::new_with_span(Some(l.clone()), (s.start, s.end - s.start))
        });
        Some(Box::new(primary.into_iter().chain(related)))
    }
}

/// Render diagnostics as plain text without colour (for snapshots and non-TTY output).
pub fn render_plain(diags: &[Diagnostic], name: &str, src: &str) -> String {
    let handler =
        miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor())
            .with_width(100)
            .with_links(false);
    let mut out = String::new();
    for d in diags {
        let r = Rendered::new(d, name, src);
        // Rendering into a String cannot fail.
        let _ = handler.render_report(&mut out, &r);
    }
    out
}

/// Levenshtein distance, for "did you mean" suggestions.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != *cb);
            cur.push(sub.min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Closest candidate to `word` within a distance proportional to its length.
pub fn suggest<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (word.chars().count() / 3).max(1) + 1;
    candidates
        .into_iter()
        .map(|c| (edit_distance(&word.to_lowercase(), &c.to_lowercase()), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, c)| (*d, c.len()))
        .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_characters_cannot_reach_the_terminal() {
        let src = "\u{1b}]0;x\u{7}: 1\n";
        let d = Diagnostic::error("DD-S004", "unknown field `\u{1b}]0;x\u{7}`")
            .at(Span::new(0, 7), "here")
            .with_help("\u{1b}[2K");
        let out = render_plain(&[d], "f\u{1b}.yaml", src);
        assert!(!out.contains('\u{1b}') && !out.contains('\u{7}'), "{out:?}");
        assert_eq!(
            clean_source("a\u{1b}b\u{85}c").len(),
            "a\u{1b}b\u{85}c".len()
        );
    }

    #[test]
    fn suggests_close_names() {
        assert_eq!(
            suggest("timeout", ["timeout_ms", "retries"]),
            Some("timeout_ms")
        );
        assert_eq!(
            suggest("timout_ms", ["timeout_ms", "retries"]),
            Some("timeout_ms")
        );
        assert_eq!(suggest("venodr", ["vendor", "version"]), Some("vendor"));
        assert_eq!(suggest("zzz", ["vendor"]), None);
    }

    #[test]
    fn renders_code_help_and_label() {
        let src = "spec: driverdesk/0.1\nvendr: x\n";
        let d = Diagnostic::error("DD-S004", "unknown field `vendr`")
            .at(Span::new(21, 26), "not allowed here")
            .with_help("did you mean `vendor`?");
        let out = render_plain(&[d], "x.yaml", src);
        assert!(out.contains("DD-S004"), "{out}");
        assert!(out.contains("did you mean `vendor`?"), "{out}");
        assert!(out.contains("x.yaml:2:1"), "{out}");
    }
}
