# D-001. Validator panicked on `repeat: { count: 0 }` (found by cargo fuzz)

- Date: 2026-10-10 · Stage/task: P0.T8 · Layer: description (validator)
- Symptom: `cargo fuzz run validate` crashed after ~219k executions: `attempt to subtract with overflow` at `lint.rs` (`registers_overlap`, `count - 1`). Input: bme280.yaml mutated to `repeat: { count: 0, stride: 1 }` (`crates/dd-spec/tests/regressions/D-001-repeat-count-zero.yaml`).
- Why proptest missed it: the structured generator never produced `repeat.count`, and byte mutations rarely turn `7` into `0` in exactly that place.

## Root cause
Lint rules run even when the schema check fails (so authors see all problems at once). They read numbers the schema would reject (`count: 0`, `val_bits: 0`, huge `stride`) and did unchecked arithmetic on them. Auditing the same pattern found three siblings:
1. `bits.div_ceil(val_bits)` with `val_bits: 0` → division by zero.
2. `width - 1` with a register `bits: 0` → underflow.
3. `repeat.count` expanded into `count` register names → a huge count exhausts memory (no upper bound in the schema).
4. Unit exponents (`m2147483647`) multiplied without bounds → i32 overflow.

## Fix
- Saturating arithmetic and zero guards in `registers_overlap` / `register_fields`.
- `repeat.count` capped at 4096 in the schema, the spec text (§8.2) and the expansion loop (`MAX_REPEAT`).
- Unit exponents limited to ±64, checked arithmetic.
- Regression tests: `lint::tests::hostile_register_numbers_do_not_panic`, `units::tests::huge_exponents_are_rejected_not_overflowed`, `tests/robustness.rs::past_crashes_stay_fixed` (replays every file in `tests/regressions/`).

## Lesson
Any code that runs on schema-invalid input must treat every number as hostile. Add new crash inputs to `tests/regressions/`.
