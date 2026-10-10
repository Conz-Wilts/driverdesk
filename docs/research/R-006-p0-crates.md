# R-006. Rust crates for the P0 validator: YAML spans, JSON Schema, CEL, UCUM, diagnostics
- Date: 2026-10-10 · Stage/task: P0.T5/T6 · Plan questions: R-P0-3, R-P0-4, R-P0-5, R-P0-6
- Confidence: high (each snippet compiled and run with rustc 1.99.0 in a scratch spike)
- Answer: **saphyr 0.1** (+ `saphyr-parser` event receiver) for YAML 1.2 with spans; **jsonschema 0.58.6** (`default-features = false`, `Registry` + `.offline()`); **cel 0.15** (renamed from cel-interpreter); **curated UCUM subset table** (octofhir-ucum works but has a tiny user base and Regenstrief data terms); **miette 7.6** + **insta 1.49** for diagnostics and snapshots.

## Evidence (versions on crates.io 2026-10-10)
| crate | version | licence | notes |
|---|---|---|---|
| saphyr / saphyr-parser | 0.1.0 (2026-09-19) | MIT OR Apache-2.0 | every node has `span {start,end}`; `Marker::index()` counts **chars, not bytes** → convert for miette; `MarkedYaml` silently drops duplicate keys (last wins) → detect with a `SpannedEventReceiver` |
| jsonschema | 0.58.6 (2026-10-06) | MIT | `Registry::new().add(uri, &schema)?.prepare()?`, `options().with_registry(&r).offline().build(&root)?`; `ValidationErrorKind` lives in `jsonschema::error`; `instance_path()`, `schema_path()`, `evaluation_path()` |
| cel | 0.15.0 (2026-10-06) | MIT | `Program::compile`, `program.expression()` AST (`Expr::Call/Ident/Select/...`, operators as `_+_`), `references().variables()/functions()`, `ctx.add_function("shl", \|a: i64, b: i64\| ...)`; int overflow → `ExecutionError::Overflow`; no bitwise operators (parse error) |
| octofhir-ucum | 0.6.1 (2026-10-02) | Apache-2.0 | `validate`, `analyse`, `is_comparable` work; not adopted (see answer) |
| miette | 7.6.0 (2025-04-27) | Apache-2.0 | **last release > 12 months** — exception to the dependency rule: mandated by CLAUDE.md conventions, repo active (2026-06); re-check at R1 |
| insta | 1.49.0 | Apache-2.0 | |

Excluded: `serde_yaml` (deprecated), `marked-yaml` (last release 2025-05, older yaml-rust2), `ucum` 0.1 (dead, CeCILL-B), `cel-interpreter` (frozen at 0.10).

## Implications
- `dd_spec::syntax` builds its own spanned tree on saphyr-parser events (also catches duplicate keys → `DD-S007`), with byte offsets.
- Schemas are embedded with `include_str!` and registered by `$id`; validation is offline.
- Spike lives (gitignored) in the session scratchpad; snippets reproduced in the crates' code.

## Re-check when
saphyr 0.2 · jsonschema 0.6x API change · miette release or archival (switch to `annotate-snippets` if archived).
