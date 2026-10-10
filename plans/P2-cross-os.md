# P2 — One description, every OS; failures name their layer

- Status: not started
- Depends on: P1 gate
- Branch: `stage/p2`
- Estimate: 3–4 weeks

Claim: the same description files work unchanged on Linux, macOS and Windows, and a test harness produces a compatibility report that pins every failure on one layer.

## Exit criteria (the gate)

1. The identical files from P1 pass `dd test` on `hil-linux`, `hil-macos`, `hil-windows`. No OS-specific fields in the descriptions. *Evidence:* three HIL runs.
2. On Windows, the Pico binds to WinUSB **automatically** on first plug-in (no INF, no Zadig) because of MS OS 2.0 descriptors. *Evidence:* HIL Windows job that removes the device node (`pnputil /remove-device`), power-cycles, and asserts WinUSB is bound.
3. `dd-harness` produces `report.json` + `report.html` + `report.md` per run: device × OS × test matrix, layer attribution for each failure, environment info (OS version, runtime version, firmware version, description digest).
4. **Mutation score ≥ 90%**: harness auto-generates ≥ 200 mutated descriptions (wrong register address, wrong scale, flipped endianness, removed timeout, loosened safety limit, wrong CRC, swapped state) and the built-in tests + conformance suite catch ≥ 90%; survivors are listed with an explanation or a new test.
5. **Layer attribution accuracy ≥ 95%** on a labelled set of ≥ 40 injected faults (sim + HIL): each fault is injected at a known layer; the harness must name it.
6. Conformance suite (runtime behaviour independent of any device) runs against the twin on all three hosted OS runners: timeouts, retries, safety enforcement, state transitions, rate limits, reconnect.
7. Public compatibility page generated from the latest reports: `docs/book/compat.md`.

## Pre-made decisions

- Same `nusb` code path on all OSes; OS differences live only in `dd-transport-usb/src/os/*` behind cfg and are listed in a table in the crate README.
- macOS: interfaces are vendor-class (0xFF), so no kernel driver claims them; no DriverKit needed in P2.
- Harness report format = JSON (schema in `spec/schema/report/0.1.json`), HTML is a static render of it.
- Layer attribution is **rule-based and explainable** (not ML): decision table over (sim result, vlab result, per-OS HIL result, error layer reported by runtime, capture analysis).

## Research questions

- **R-P2-1** nusb on macOS: claiming interface 1 while interface 0 is claimed by another process / same process; permissions; does macOS ever auto-match a driver to a vendor interface? · needed by P2.T1 · Default: works for vendor-class interfaces without entitlements. · Status: open
- **R-P2-2** MS OS 2.0 descriptors for a composite device: function subset headers per interface, `DeviceInterfaceGUIDs` registry property, Windows version support. · needed by P2.T3 · Default: per-function compatible ID `WINUSB` for both interfaces; DeviceInterfaceGUID per interface. · Status: open
- **R-P2-3** How to force Windows to "forget" a device to test first-plug behaviour (`pnputil /remove-device`, clearing the `usbflags` registry cache for MS OS descriptors). · needed by P2.T3 · Default: remove device + delete `HKLM\SYSTEM\CurrentControlSet\Control\usbflags\<VIDPIDREV>` + port cycle. · Status: open
- **R-P2-4** Mutation testing approach for YAML (existing tools?) · needed by P2.T6 · Default: write our own mutator over the typed model with ~15 operators. · Status: open

## Tasks

- [ ] **P2.T0 Re-validate this plan.**
- [ ] **P2.T1 macOS** (deps: R-P2-1) — build, sim tests on hosted macOS; HIL on `hil-macos` (needs H-005). Fix OS-specific transport issues.
- [ ] **P2.T2 Windows build + sim** — hosted Windows runner green.
- [ ] **P2.T3 Windows HIL + WinUSB auto-bind** (deps: R-P2-2, R-P2-3, H-004) — exit criterion 2. If descriptors need changes, change firmware and re-run Linux/macOS HIL too.
- [ ] **P2.T4 Conformance suite** — `crates/dd-harness/conformance/`: device-independent tests using a synthetic "conformance device" in the twin with every feature. Exit criterion 6.
- [ ] **P2.T5 Harness reporting + attribution** — data model, decision table (in `docs/book/blame-model.md`, tested), JSON/HTML/MD renderers, environment capture. Exit criterion 3.
- [ ] **P2.T6 Description mutation testing** (deps: P2.T5; R-P2-4) — exit criterion 4.
- [ ] **P2.T7 Fault-injection attribution set** (deps: P2.T5) — labelled faults across layers incl. HIL-only faults (power-cut via uhubctl mid-transfer, firmware that stalls on purpose: build `firmware/pico-bridge` with a `faulty` feature). Exit criterion 5.
- [ ] **P2.T8 CI matrix** — `hil-matrix.yml` runs all three HIL nodes in parallel nightly and on PRs to main; aggregates into one report artifact; posts summary to the PR.
- [ ] **P2.T9 Compat page + gate** — exit criterion 7, `/gate P2`.

## Risks and debugging hints

- **Windows caches MS OS descriptor results per VID/PID/bcdDevice** — bump `bcdDevice` in firmware when changing descriptors, or clear `usbflags` (R-P2-3). This is the #1 time sink here.
- **macOS "Allow accessory to connect" prompt** on Apple Silicon for new USB devices may block HIL on first plug: handled in H-005 (security setting for accessories).
- **Timing differences**: Windows default USB timeouts and scheduling differ; tests must use description-declared timeouts only.

## Human actions

H-004 (Windows node), H-005 (macOS node) must be done.

## Gate procedure

`/gate P2`, report `docs/reports/P2.md`, merge, tag `gate/P2`.
