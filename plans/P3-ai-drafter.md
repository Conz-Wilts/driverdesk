# P3 — AI writes the description

- Status: not started
- Depends on: P0 gate (benchmark list); P1 gate for the hardware check
- Branch: `stage/p3`
- Estimate: 3–4 weeks

Claim: from a datasheet PDF alone, the AI drafter produces a description that validates and passes tests on real hardware, and we can measure how good it is.

## Exit criteria (the gate)

1. `dd draft <datasheet.pdf> --class sensor.pressure` (CLI wrapper over `ai/`) produces `bme280.draft.yaml` + `bme280.evidence.json` (per-field value, confidence 0–1, evidence: page + quote).
2. The BME280 draft, **with zero human edits**, after the automated repair loop and bench-agent loop, passes all of its own tests and the hand-written P1 description's tests on `hil-linux`. Same for TMP117.
3. Benchmark run on the 15 held-out chips, reported in `benchmark/runs/` and `docs/reports/P3-benchmark.md`:
   - field accuracy first pass and after repair loop (overall and per section: identity, registers, conversions, timing),
   - **confident-and-wrong rate** (fields with confidence ≥ 0.9 that are wrong),
   - validation pass rate, repair iterations, cost (USD) and latency per chip.
   Targets: post-loop field accuracy ≥ 85%, confident-and-wrong ≤ 3%, cost ≤ US$3 per chip. **Kill signal**: first-pass accuracy < 60% after two prompt/pipeline iterations → run DECIDING on pivoting the drafter toward trace + import tools (north star §14).
4. Calibration: a reliability plot (confidence bucket vs. actual accuracy) in the report; ECE ≤ 0.1.
5. Safety rule enforced in code: drafts never contain a signed safety block; `safety.signed_by` is always null and `dd validate --publish` rejects unsigned safety.
6. Every eval run is reproducible: model id, prompt hashes, dataset version, seed recorded.

## Pre-made decisions

- Python package `ai/driverdesk_ai` (uv, pydantic). Calls the Claude API; load the `claude-api` skill before writing API code. Drafting model `claude-opus-5-5`, cheap passes (classification, table detection) `claude-haiku-5-5`. Use structured outputs with the JSON Schema of each spec section.
- The Rust validator is used from Python through a PyO3 binding (`crates/dd-spec-py`, built with maturin) — one validator everywhere (ADR-0001/0009).
- Datasheets are never committed: `benchmark/chips.yaml` has URLs + hashes; `cache/datasheets/` is gitignored.
- Pipeline stages (north star §08): ingest → section extraction with evidence → deterministic cross-checks → validate/repair loop (≤ 4 rounds) → sim self-consistency → bench agent (HIL) → report. Each stage is a pure function over files so stages are cacheable and testable.
- Ground truth for the benchmark comes from Linux drivers (register addresses, chip IDs, scale factors, init sequences), extracted by AI then **verified by a second independent pass and spot checks against the datasheet**; disagreements are resolved by reading the source code, recorded per field.

## Research questions

- **R-P3-1** PDF extraction: which of PyMuPDF, Docling, pdfplumber, Claude PDF input (native document support), or page-image vision best recovers register tables? · needed by P3.T2 · Default: spike on 5 datasheets scoring register-table cell accuracy; pick the best, or a combination (text-native PDF support for prose + page images for tables). · Status: open
- **R-P3-2** Claude API features now: structured outputs, citations, PDF support, prompt caching, batch API, token limits for 100-page PDFs. · needed by P3.T3 · Default: use native PDF input + citations if available; prompt caching for the datasheet across section passes; batch API for benchmark runs. · Status: open
- **R-P3-3** Licensing/ToS of downloading and processing vendor datasheets for evaluation. · needed by P3.T1 · Default: evaluation use of publicly downloadable datasheets, not redistributed; record per vendor in `chips.yaml`. · Status: open
- **R-P3-4** Confidence estimation method: model self-report vs. agreement across samples vs. evidence-quote verification. · needed by P3.T4 · Default: combine — self-report, plus check that the quote exists verbatim on the cited page, plus 3-sample agreement for numeric fields; calibrate with isotonic regression on the dev set. · Status: open

## Tasks

- [ ] **P3.T0 Re-validate this plan.**
- [ ] **P3.T1 PyO3 binding** — `dd_spec.validate(yaml) -> list[Diagnostic]`; wheels built in CI for macOS/Linux.
- [ ] **P3.T2 Ingest** (R-P3-1) — PDF → pages (text + image), table detection, section index (register map, electrical characteristics, timing, conversions). Cache by SHA-256.
- [ ] **P3.T3 Section extractors** (R-P3-2) — one prompt + schema per spec section, each returning values with evidence. Prompts in `ai/prompts/*.md`, versioned by hash.
- [ ] **P3.T4 Confidence + evidence verification** (R-P3-4) — quote verification against page text; multi-sample agreement; calibration.
- [ ] **P3.T5 Cross-checks + repair loop** — deterministic checks (register overlap, unit sanity, chip-ID plausibility, timing consistency), validator diagnostics fed back to the model with the failing YAML span; max 4 rounds; stop early if no progress.
- [ ] **P3.T6 Ground truth** — for all 50 chips, `benchmark/truth/<chip>.yaml` from Linux drivers with double-pass verification and a `verification:` record per field.
- [ ] **P3.T7 Scoring** — `uv run dd-bench --split dev|heldout`: field-level comparison with tolerance rules (addresses exact, scale factors ±0.5%, timings ±10% or ≥), per-section metrics, calibration, cost. Output JSON + MD.
- [ ] **P3.T8 Iterate on dev split** — up to 3 improvement rounds; log every run. Never look at held-out results until the final run.
- [ ] **P3.T9 Bench agent v0** (deps: P1 gate) — runs on `hil-linux` via a workflow: read-only probes (chip ID, reset values of registers declared readable and non-volatile), built-in tests; compares to draft; proposes field fixes with evidence "measured"; loop ≤ 3 times. Writes only within the draft's limits **and** only to registers the datasheet marks safe (configuration registers), never to anything flagged by the cross-checks as uncertain.
- [ ] **P3.T10 End-to-end BME280 + TMP117** — exit criterion 2.
- [ ] **P3.T11 Held-out run + report + gate** — exit criteria 3–6, `/gate P3`.

## Risks and debugging hints

- **Table extraction errors dominate** register accuracy — measure extraction separately from reasoning so you know which to fix.
- **Hallucinated registers**: require evidence quotes; reject fields whose quote is not found on the page.
- **Overfitting to BME280** (it is in many training sets): that is why the held-out split exists. Also include 5 obscure chips in held-out.
- **Cost blow-ups**: cache datasheet tokens (prompt caching), use Haiku for routing, batch API for benchmark runs.

## Human actions

H-006: Anthropic API key stored in macOS Keychain and as a GitHub Actions secret (Claude provides the commands; budget cap set in the Anthropic console by the human).

## Gate procedure

`/gate P3`, report `docs/reports/P3.md`, merge, tag `gate/P3`.
