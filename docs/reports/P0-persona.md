# P0 persona usability report (P0.T9, exit criterion 4)

- Date: 2026-10-10 · Validator: `dd validate` at commits `5edbcd9` (round 1) and `2a7a9c0` + bracket fix (round 2)
- Threshold: **≥ 90 %** of invalid fixtures corrected by a persona that sees only the fixture and the validator output.
- Result: **round 1 55/55 (100 %)**, **round 2 15/16 (94 %)** on the messages changed after round 1. Score on the latest messages: **54/55 = 98.2 %**. **Pass.**

## Method

- Persona (subagent, Claude Sonnet 5.5): *"embedded C engineer; knows the devices, USB and I²C; has never seen DriverDesk, its spec or Rust."*
- Input per fixture: the fixture with its `# expect:` header removed, and the human-format `dd validate` output (no colour, no links). Nothing else.
- Task: write the corrected YAML with the smallest sensible change; rate clarity 1–5; say what was unclear.
- Scoring (automatic, `dd validate --format json` on the persona's file): **fixed** = the fixture's expected code is gone **and** no error-severity diagnostic remains.
- Scored fixtures: 55 of 57. Excluded, with reason: `41-L016-unsigned-safety` (resolving it requires a signing key, not an edit) and `50-L023-provisional-section` (informational; no fix is expected).

### Threats to validity (recorded honestly)

- Fixtures were batched 6–7 per subagent in round 1 and 4 per subagent in round 2, so later fixtures in a batch could benefit from earlier ones. One round-1 persona said so explicitly for `04-missing-tests` ("I could only do that because I had the other fixtures' content in context"). Round 2 re-tested that fixture with the new example-bearing help and it passed with clarity 5.
- Three subagents broke the Read/Write-only rule (one used `sed` once to fix its own file in round 1; two wrote outputs with `sed` in round 2). None ran the validator; their edits were checked by the scorer like everyone else's.
- Both rounds use the same model family as the author of the messages. A human study is planned for R1 (design partners).

## Round 1 (55 fixtures)

All 55 fixed. Clarity ratings ≤ 3 drove the round-2 changes:

| Fixture | Clarity | Persona's complaint | Change made |
|---|---|---|---|
| 01-syntax-unclosed-bracket | 2 | error points at the next line, says "invalid indentation", help talks about tabs | name the unclosed bracket at its opening position, help says which bracket to add |
| 04-missing-tests | 2 | no shape for an entry; caret on line 1 | required sections show a minimal example |
| 05-empty-expose | 2 | "add at least one entry" doesn't say what an entry looks like | example + (round 2) "one entry per class" with the class list |
| 17-bad-device-id | 3 | doesn't say what to turn `LedThermo` into | concrete suggestion `example.led-thermo` from vendor prefix |
| 18-vid-too-large | 3 | limit given in decimal only | "must fit in 16 bits (0x0000–0xFFFF); check for an extra digit" |
| 22-deep-nesting | 3 | points into a long line | help says to flatten/remove the value starting there |
| 35-L011-vid-only-match | 3 | where to get the pid | "the product id from the device descriptor, e.g. from `lsusb`" |
| 40-L015-write-read-only | 3 | two options, intent unknowable without datasheet | none (genuinely ambiguous; both options are listed) |
| 47-L021-field-outside-register | 3 | where to widen the register | "add `bits: 32` (or 24/64) to that register" |
| 20-duplicate-key | 4 | which value is kept? (persona assumed last; we keep first) | help states the validator keeps the first |
| 14-bad-command-name | 4 | L007/L008 suggested the invalid name `ReadTemp` | invalid names are never offered as suggestions |
| 36, 37, 48, 49 (CEL) | 4–5 | underline covers the whole expression | span narrowed to the operator / function / name |
| 51-L024, 52-L025 | 4 | list existing names; where `idempotent` goes | help lists commands; "add `idempotent: true` next to `retries`" |

## Round 2 (16 changed messages, fresh subagents)

| Fixture | Result | Clarity | Note |
|---|---|---|---|
| 01-syntax-unclosed-bracket | fixed | 3 | marker pointed at the `{`, not the `[` → **bug found**: mismatched closer now blames the bracket left open (regression test `unclosed_bracket_inside_braces_is_the_bracket`) |
| 04-missing-tests | fixed | 5 | |
| 05-empty-expose | **not fixed** | 4 | added one entry for two classes → help now lists every class that needs an entry (not re-tested) |
| 14-bad-command-name | fixed | 5 | |
| 17-bad-device-id | fixed | 5 | |
| 18-vid-too-large | fixed | 4 | |
| 20-duplicate-key | fixed | 5 | |
| 22-deep-nesting | fixed | 4 | |
| 35-L011-vid-only-match | fixed | 4 | |
| 36-L012-bitwise-operator | fixed | 5 | |
| 37-L012-unknown-function | fixed | 5 | |
| 47-L021-field-outside-register | fixed | 3 | intent genuinely ambiguous (wrong bit vs wider register) |
| 48-L022-unknown-name | fixed | 5 | |
| 49-L022-unknown-instance-param | fixed | 5 | |
| 51-L024-variant-excludes-unknown | fixed | 4 | |
| 52-L025-retry-non-idempotent | fixed | 4 | asked for a plain definition of "idempotent" → follow-up for the error page |

## Follow-ups

- Error page DD-L025: define "idempotent" in one plain sentence.
- Re-run the persona loop at every spec minor version and whenever a diagnostic text changes (`P0.T9` procedure; scorer and inputs are reproducible from `spec/fixtures/invalid/`).
