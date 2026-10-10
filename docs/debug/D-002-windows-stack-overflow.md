# D-002. `cargo xtask spec-check` overflowed the stack on Windows CI

- Date: 2026-10-10 · Stage/task: P0 (CI on PR #2) · Layer: runtime (tooling)
- Symptom: `thread 'main' has overflowed its stack` in `spec-check` on `windows-latest` only; Linux/macOS green; `nextest` (2 MiB test threads) green on all three.
- Reproduction (macOS): validating the corpus on a thread with an explicit stack: 512–896 KiB overflow, 1 MiB passes (debug build).
- Root cause: debug builds of `dd_spec::validate` (jsonschema evaluation + our recursive walkers, depth ≤ 64) need ≈ 1 MiB of stack. Windows gives the main thread 1 MiB; Linux/macOS give 8 MiB.
- Fix: `dd` and `xtask` run their work on a thread with an 8 MiB stack. Regression test `corpus_validates_within_a_2_mib_stack` (fixtures.rs) keeps a 4× margin for the 8 MiB worker.
- Follow-up: library users (runtime, Python binding, Wasm) must give validation ≥ 2 MiB of stack; noted for P1/P3 bindings. Reducing frame sizes (boxing large `Diagnostic`s in the oneOf path) is the long-term fix.
