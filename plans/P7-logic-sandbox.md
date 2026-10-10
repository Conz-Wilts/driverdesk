# P7 — Vendor logic is portable and contained

- Status: not started
- Depends on: P2 gate (P4 for signed module digests)
- Branch: `stage/p7`
- Estimate: 3 weeks

Claim: vendor algorithms compiled to WebAssembly run identically on all three OSes, can only do what the description grants, and cannot harm the host even when malicious. Also: the runtime process is sandboxed by the OS as a second wall.

## Exit criteria (the gate)

1. WIT package `spec/wit/driverdesk-logic@0.1.0` published in the spec (worlds: `sensor-filter`, `actuator-effect`, `protocol-helper`), plus spec section "Logic modules" (module digest, world, granted commands, CPU/memory limits).
2. Three example modules: EMA smoothing filter (Rust), LED effect generator (C via wasi-sdk), BME280 compensation (Rust) whose output is **bit-identical** to the CEL version from P1 on 10k recorded raw samples.
3. **Determinism**: each example produces identical outputs on Linux/macOS/Windows for the same inputs (hash of 10k outputs compared in CI matrix).
4. **Containment suite** (all contained, runtime keeps serving other devices): infinite loop (fuel/epoch stops it within the declared CPU budget), memory bomb (limit), calling an ungranted command, out-of-envelope arguments (rejected by safety, never reaches transport), trapping, deep recursion, attempting WASI filesystem/network/clock (not linked), huge return values, re-entrancy. ≥ 12 tests.
5. **OS sandbox** around per-device runtime processes: Linux landlock + seccomp profile, macOS sandbox profile, Windows AppContainer (or restricted token + job object if AppContainer blocks USB — research). A test proves the driver process cannot open files or sockets outside its grant.
6. Overhead measured and reported: per-call latency (p50/p99) and throughput vs. CEL path; budget p99 < 200 µs per filter call on the lab Pi 5.
7. Module digests are part of the signed description; the agent refuses a module whose digest does not match (P4 integration test).

## Pre-made decisions

- `wasmtime` with the Component Model; host bindings generated with `wasmtime::component::bindgen!` from the WIT (ADR-0006).
- No WASI capabilities linked by default; only `driverdesk:logic/device` imports. Clocks/randomness only if a future world grants them explicitly.
- Fuel metering for determinism tests; epoch interruption for production timeouts (both supported).
- Guest toolchains: Rust (`cargo component` or `wit-bindgen`), C (wasi-sdk + wit-bindgen C). Others are documentation-only in P7.

## Research questions

- **R-P7-1** Current wasmtime version, component model API, `ResourceLimiter`, fuel vs epoch costs, async support. · Default: latest stable wasmtime. · Status: open
- **R-P7-2** Float determinism in Wasm across hosts (NaN canonicalisation setting in wasmtime). · Default: enable NaN canonicalisation; test on all OSes. · Status: open
- **R-P7-3** Windows sandboxing of a process that must use WinUSB handles passed from the agent (AppContainer capability for USB? handle inheritance?). · Default: agent opens the device and duplicates the handle into a restricted-token + job-object child; AppContainer only if it works with passed handles. · Status: open
- **R-P7-4** macOS: sandbox profile for a child process with an inherited IOKit user client / USB interface. · Default: `sandbox_init` with a custom profile denying file/network; verify USB I/O still works on the passed interface. · Status: open

## Tasks

- [ ] **P7.T0 Re-validate this plan.**
- [ ] **P7.T1 WIT + spec section** — exit criterion 1.
- [ ] **P7.T2 `dd-sandbox` host** (R-P7-1, R-P7-2) — instantiate per device, bind host functions to the runtime's command layer (safety enforced there), limits, error mapping to layer `logic`.
- [ ] **P7.T3 Example modules + guest docs** — exit criterion 2; tutorial "Write a logic module in Rust/C".
- [ ] **P7.T4 Containment suite** — exit criterion 4 (write each malicious module as a fixture).
- [ ] **P7.T5 Cross-OS determinism** — exit criterion 3 in the CI matrix.
- [ ] **P7.T6 OS process sandbox** (R-P7-3, R-P7-4) — exit criterion 5.
- [ ] **P7.T7 Signed module digests** (deps: P4) — exit criterion 7.
- [ ] **P7.T8 Performance** — `criterion` benches; exit criterion 6.
- [ ] **P7.T9 Security review + gate** — run `security-review` over the stage; `/gate P7`; then write `docs/reports/POC-complete.md` summarising P0–P7 with all demo recordings (this is the fundraising/partner pack).

## Risks and debugging hints

- **"Private logic" expectations**: Wasm can be decompiled; the docs must say plainly that private means access-controlled distribution, not secrecy (north star §13).
- **Sandbox breaks USB** on some OS: fd/handle passing design (R-P7-3/4) must be validated in a spike before building the full suite.

## Human actions

None expected.

## Gate procedure

`/gate P7`, report, merge, tag `gate/P7`, then start R1.T0.
