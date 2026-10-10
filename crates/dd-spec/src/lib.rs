//! Description format: types, parser and validator for DriverDesk device descriptions (P0).
//!
//! Pipeline: [`syntax::parse`] (YAML 1.2 with spans) → [`schema::validate`] (JSON Schema
//! 2020-12, `spec/schema/0.1`) → [`lint::run`] (semantic rules `DD-Lxxx`). Every finding is a
//! [`diag::Diagnostic`] with a stable code, a source span, help text and a docs link.

pub mod checksum;
pub mod diag;
pub mod expr;
pub mod lint;
pub mod schema;
pub mod syntax;
pub mod units;

pub use diag::{Diagnostic, Severity, Span};

/// Result of validating one description.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Report {
    /// All findings, errors first, then by position.
    pub diagnostics: Vec<Diagnostic>,
}

impl Report {
    /// True when no error-severity diagnostic was found.
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Number of errors.
    pub fn errors(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Number of warnings.
    pub fn warnings(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }
}

/// Validate the description text `src`: syntax, schema and semantic lint rules.
pub fn validate(src: &str) -> Report {
    let mut diagnostics = Vec::new();
    let doc = match syntax::parse(src, &mut diagnostics) {
        Ok(doc) => doc,
        Err(fatal) => {
            diagnostics.extend(fatal);
            return finish(diagnostics);
        }
    };
    let schema_diags = schema::validate(&doc);
    let schema_ok = schema_diags.is_empty();
    diagnostics.extend(schema_diags);
    diagnostics.extend(lint::run(&doc, schema_ok));
    finish(diagnostics)
}

fn finish(mut diagnostics: Vec<Diagnostic>) -> Report {
    diagnostics.sort_by_key(|d| {
        (
            std::cmp::Reverse(d.severity),
            d.span.map_or(usize::MAX, |s| s.start),
            d.code,
        )
    });
    Report { diagnostics }
}
