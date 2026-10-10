//! Semantic lint rules DD-Lxxx (spec Appendix B).

use crate::diag::Diagnostic;
use crate::syntax::Document;

/// Run all lint rules.
pub fn run(_doc: &Document, _schema_ok: bool) -> Vec<Diagnostic> {
    Vec::new()
}
