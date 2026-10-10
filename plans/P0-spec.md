# P0 — The spec and the yardstick

- Status: in progress
- Depends on: B.T1–B.T5
- Branch: `stage/p0`
- Estimate: 2 weeks

Claim: a precise, versioned device-description standard exists, a validator enforces it with error messages a hardware engineer can act on, and we have a benchmark to measure AI drafting against.

## Exit criteria (the gate)

1. `spec/driverdesk-0.1.md` covers every section listed in P0.T2, each field with type, MUST/SHOULD wording (RFC 2119/8174) and at least one example. *Check:* `cargo xtask spec-check` verifies every field in the schema is mentioned in the text and every example block in the text validates.
2. JSON Schema 2020-12 files in `spec/schema/0.1/` validate all 5 example descriptions and reject all fixtures in `spec/fixtures/invalid/`. *Check:* `cargo xtask spec-check`.
3. `dd validate <file>` implements schema validation **and** ≥ 14 semantic lint rules (P0.T6) with source spans, a stable error code, a one-line help, and a docs link each. *Check:* ≥ 40 snapshot tests in `crates/dd-spec/tests/`.
4. **Persona usability score ≥ 90%** on the invalid fixtures (TESTING.md "Usability tests without humans"). *Check:* `docs/reports/P0-persona.md` with per-fixture results.
5. Property test: `dd validate` never panics on 100k random YAML documents (proptest + `arbitrary`), and runs < 50 ms on each example.
6. `benchmark/chips.yaml` lists 50 chips with: part number, vendor, class, bus, datasheet URL (HTTP 200 checked), datasheet SHA-256, mainline Linux driver path and commit, licence note. Split 35 dev / 15 held-out. *Check:* `cargo xtask bench-check`.
7. `docs/book/` (mdBook) builds and has "Describe your first device in 30 minutes" using the pico-thermo example.

## Pre-made decisions

- YAML authoring, JSON Schema 2020-12 canonical (ADR-0002). CEL for expressions, UCUM for units (ADR-0003).
- Spec text CC-BY-4.0 (ADR-0004). Spec identifier: `spec: driverdesk/0.1`.
- Shape of the spec follows the north-star example (identity, match, transport, commands, safety, states, expose, tests, visibility). Borrow vocabulary from Linux regmap, IIO, devicetree bindings (see north star §05). Concrete borrow list: R-005 "Decisions for spec 0.1".
- Transports in 0.1: `usb` (control, bulk, interrupt), `i2c` (as a child of a bridge), `i2c-bridge` (USB-to-I²C, protocol described in the bridge's own description), `serial`. HID arrives in P5, PCI in R2.

## Research questions

- **R-P0-1** Survey of prior description formats: devicetree bindings (dt-schema), CMSIS-SVD, SystemRDL, IP-XACT, USB/HID descriptors, Linux regmap + IIO, Devil IDL, ESPHome YAML, Zephyr devicetree, SCPI, Modbus register maps. What do we borrow, what do we import later? · needed by P0.T2 · Default: borrow regmap register flags (readable/writable/volatile/precious), IIO channel model (raw/scale/offset/unit), dt-schema conventions (`$id`, `compatible`-like ids, vendor prefixes). · Answered: R-004 (Linux), R-005 (survey)
- **R-P0-2** Can BME280 compensation (integer formulas using ~18 calibration registers) and TMP117 conversion be expressed in CEL? · needed by P0.T3 · Default: add a `compute:` block = ordered list of named CEL steps (`let`-style), 64-bit int and double, plus a DriverDesk CEL function library `shl shr band bor bxor sign_extend clamp be16 le16 be24` (standard CEL has no bitwise ops and errors on overflow — R-004); if BME280 needs > 40 lines, keep it anyway in 0.1 and move to a logic module in P7 as a comparison. · Status: open
- **R-P0-3** Rust YAML parser with line/column spans for every node, actively maintained (candidates: `saphyr`, `yaml-rust2`, `marked-yaml`, `serde-saphyr`). `serde_yaml` is deprecated — do not use. · needed by P0.T5 · Default: whichever gives spans and passes the YAML 1.2 test suite best; wrap it behind `dd_spec::syntax` so it can be swapped. · Status: open
- **R-P0-4** Rust JSON Schema 2020-12 validator that reports instance paths (to map to spans) — `jsonschema` crate current version, `$ref` across files, format validation. · needed by P0.T5 · Default: `jsonschema`. · Status: open
- **R-P0-5** CEL implementation in Rust (`cel-interpreter` or newer), supports type-checking? custom functions? · needed by P0.T6 · Default: `cel-interpreter` for evaluation; write our own lightweight type/unit checker over its AST for lint L006. · Status: open
- **R-P0-6** UCUM library in Rust, or do we need a table subset? · needed by P0.T6 · Default: ship a curated subset table (SI base + common derived units + prefixes) as data in `spec/units/ucum-subset.yaml`. · Status: open

## Tasks

- [x] **P0.T0 Re-validate this plan** (deps: B gate or B.T1–T5) — read north star §06, ADRs, any new research; adjust tasks. Commit as `docs(plan): ...`.
- [x] **P0.T1 Prior-art survey** (deps: —) — answer R-P0-1 as `docs/research/R-xxx-prior-art.md` with a table: format → what it describes well → what we borrow → importer priority. Use a research subagent for breadth.
- [ ] **P0.T2 Spec text v0.1** (deps: P0.T1)
  - Do: `spec/driverdesk-0.1.md` sections: 1 Scope & goals · 2 Conformance language · 3 Document structure & versioning · 4 Identity & variants (id-register detection, per-variant overrides — R-005) · 4a Instance parameters (bus address options, straps, supplies; separate from chip facts) · 5 Matching (modalias-style keys; precedence; specificity rules) · 6 Transports (usb, serial, i2c-bridge, i2c; stacking) · 7 Data types, byte order, framing, checksums (named CRC catalogue) · 8 Registers (regmap-style map; `access` enum ro/rw/wo/rw1c/rw1s/rc/w1 + volatile/precious/noinc; reset values; fields/bitfields; reusable fieldsets; repeat count/stride — R-005) · 9 Commands (send/expect/output, timeouts, retries, idempotency flag) · 10 Expressions (CEL profile: allowed functions, int/double semantics, `compute` steps) · 11 Units (UCUM subset) · 12 State machine (states, transitions, triggers, actions; suspend/resume mapping) · 13 Timing · 14 Safety envelope (limits, rates, forbidden sequences, `signed_by`; enforcement semantics: runtime MUST reject) · 15 Exposure & class profiles · 16 Built-in tests (call, expect matchers, tolerance, `rejected_by_safety`) · 17 Visibility · 18 Compatibility promise · 19 Security considerations · Appendices: full examples, error code registry.
  - Accept: exit criterion 1 (after P0.T3/T7 exist).
- [ ] **P0.T3 JSON Schemas + class profiles** (deps: P0.T2 draft, R-P0-2)
  - Do: `spec/schema/0.1/description.json` + one file per section, `$id` under `https://driverdesk.dev/schema/0.1/` (domain is provisional; ids are just URIs). Class profiles: `sensor.temperature`, `sensor.humidity`, `sensor.pressure`, `light.indicator`, `bridge.i2c`. Examples: `devices/examples/pico-thermo.yaml`, `pico-bridge.yaml`, `bme280.yaml`, `tmp117.yaml`, `minimal.yaml`.
  - Accept: exit criterion 2.
- [ ] **P0.T4 Invalid fixtures** (deps: P0.T3) — ≥ 40 files in `spec/fixtures/invalid/`, each one mistake, each with a header comment naming the expected error code. Cover: schema errors (missing field, wrong type, unknown key with "did you mean"), every lint rule, YAML syntax errors, wrong spec version.
- [ ] **P0.T5 `dd-spec` parse + schema validation** (deps: P0.T3; R-P0-3, R-P0-4)
  - Do: parse YAML with spans → JSON value + span map; validate with schema; map instance paths to spans; `miette` diagnostics; `dd validate` in `dd-cli` with `--format human|json`. Exit codes: 0 ok, 1 invalid, 2 usage/IO.
  - Accept: snapshot tests for schema-class fixtures; JSON output schema documented.
- [ ] **P0.T6 Semantic lint rules** (deps: P0.T5; R-P0-5, R-P0-6) — implement at least:
  `DD-L001` unreachable state · `L002` transition to undefined state · `L003` command without timeout · `L004` numeric write argument without safety limit · `L005` overlapping register ranges · `L006` unit/type mismatch in expression · `L007` `expose` references unknown command/field · `L008` test references unknown command · `L009` actuator class without safety block · `L010` unknown spec version (with upgrade hint) · `L011` match too broad (VID only) · `L012` CEL does not compile · `L013` unknown checksum algorithm · `L014` timeout shorter than the sum of declared waits · `L015` write to read-only register · `L016` safety block not signed (warning in 0.1).
  - Accept: one fixture + snapshot per rule; codes listed in the spec appendix.
- [ ] **P0.T7 `cargo xtask spec-check`** (deps: P0.T3, P0.T5) — cross-checks text ↔ schema ↔ examples (exit criterion 1 & 2).
- [ ] **P0.T8 Robustness** (deps: P0.T6) — proptest/arbitrary fuzzing; `cargo fuzz` target for the parser (run 10 min in a nightly workflow). Exit criterion 5.
- [ ] **P0.T9 Persona usability loop** (deps: P0.T6)
  - Do: for each invalid fixture, give a fresh subagent (persona: "embedded C engineer, knows the device, has never seen DriverDesk or Rust") only the `dd validate` output and the fixture; ask for the corrected YAML; check by re-validating. Iterate on messages until ≥ 90%. Record scores per iteration in `docs/reports/P0-persona.md`.
  - Accept: exit criterion 4.
- [ ] **P0.T10 Benchmark list** (deps: —)
  - Do: 50 chips with a public datasheet AND a mainline Linux driver, spread across: temperature/humidity/pressure (IIO, hwmon), accelerometers/gyros (IIO), ADCs/DACs, light sensors, GPIO expanders, RTCs, LED drivers. Prefer I²C (works with our bridge). For each: datasheet URL + SHA-256 (download to `cache/`, gitignored), Linux driver path + commit. `cargo xtask bench-check` verifies URLs and hashes. Random 35/15 split with recorded seed.
  - Accept: exit criterion 6.
- [ ] **P0.T11 Docs site** (deps: P0.T3) — mdBook in `docs/book/`: intro, tutorial, spec rendered, error code pages (each lint rule = one page, linked from diagnostics). Exit criterion 7.
- [ ] **P0.T12 Gate** — `/gate P0`.

## Risks and debugging hints

- **Spec bloat**: keep 0.1 to what P1–P2 need. Anything else goes in `spec/proposals/` for later versions.
- **Span mapping** for schema errors is fiddly: build the span map as a `HashMap<JsonPointer, Span>` during parsing; test with deeply nested and flow-style YAML.
- **CEL int vs double** surprises: CEL has no implicit numeric conversion; the linter should flag `raw / 100` when `raw` is int and the output unit expects double — suggest `double(raw) / 100.0`.

## Human actions

None.

## Gate procedure

`/gate P0`, report `docs/reports/P0.md`, merge, tag `gate/P0`.
