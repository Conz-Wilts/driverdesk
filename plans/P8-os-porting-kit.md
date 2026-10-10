# P8 — An OS writer can port the runtime and test their OS against the whole catalogue

- Status: not started
- Depends on: P4 gate (client protocol), P7 gate (process sandbox design)
- Branch: `stage/p8`
- Estimate: 3–4 weeks

Claim: a small, documented Host Interface is all an OS needs. Proven by porting to an OS we haven't targeted (FreeBSD by default), running the OS conformance suite on it, and producing a catalogue report for it.

## Exit criteria (the gate)

1. `spec/host-interface-0.1.md`: every function an OS must provide (enumeration, hotplug, USB/HID/serial/I²C access, isolation, timers, optional class surfaces) with required semantics, error mapping and timing expectations. Each item tagged *required* / *optional* / *class-surface*.
2. `crates/dd-host`: the Host Interface as a Rust trait; Linux/macOS/Windows implementations refactored to it (no behaviour change: P2 HIL matrix still green). `dd-host-capi`: C ABI + header (`dd_host.h`) generated with cbindgen, with a C reference stub that compiles.
3. **OS conformance suite** `dd oscheck`: tests every Host Interface function against the spec using the software twin (and virtual USB where the OS supports gadgets); outputs a conformance report in the same format as P2 reports. Passes on Linux, macOS, Windows.
4. **Second-host port**: the runtime + agent build and run on FreeBSD (R-P8-1), pass `dd oscheck` required items, and drive the pico-bridge + BME280 on real hardware (a FreeBSD lab node or a VM with USB passthrough on lab-linux).
5. **Catalogue run**: `dd catalogue-run --os <name>` runs every description in the registry against an OS (sim for all; HIL where a lab runner for that OS exists) and produces a report: per-driver result, failing layer, and **missing class surfaces ranked by number of devices they would unlock**. Demonstrated for all four OSes.
6. **Client protocol spec** (from P4) has a second, independent implementation: a minimal agent in a different language (Python, ~500 lines) that fetches + verifies + hands a description to the runtime, passing the P4 security suite. This proves "any OS can connect to the storage".
7. Porting guide `docs/book/porting.md`: "Port DriverDesk to your OS" with the FreeBSD port as the worked example; a persona review (embedded/OS developer persona) scores ≥ 90% on a set of 15 porting questions answered only from the guide.

## Pre-made decisions

- Host Interface is defined in prose + Rust trait + C ABI; the prose is normative.
- Second host defaults to FreeBSD (mature USB stack, libusb available, plausible OS partner audience). Alternatives if blocked: a BSD-licensed hobby OS with USB support, or Redox (userspace drivers already).
- The second-language client is Python for speed of writing; it is a conformance proof, not a product.

## Research questions

- **R-P8-1** FreeBSD USB access from Rust: does nusb support FreeBSD? If not, `rusb`/libusb backend; hotplug via devd. · needed by P8.T4 · Default: `rusb` backend behind `UsbBackend` (ADR-0008 fallback) + devd for hotplug. · Status: open
- **R-P8-2** FreeBSD lab option: bare-metal on a spare mini PC vs. bhyve/QEMU VM with USB passthrough on lab-linux. · Default: QEMU VM on lab-linux with `usb-host` passthrough of a second Pico on its own hub port. · Status: open
- **R-P8-3** Capsicum (FreeBSD) as the process sandbox for drivers. · Default: Capsicum capability mode after opening the device. · Status: open
- **R-P8-4** What OS writers actually need: survey docs/forums of hobby OS projects (Redox, Haiku, SerenityOS, BSDs) on driver pain points to prioritise class surfaces. · Default: input, serial, storage, network first. · Status: open

## Tasks

- [ ] **P8.T0 Re-validate this plan.**
- [ ] **P8.T1 Host Interface spec** (R-P8-4) — exit criterion 1.
- [ ] **P8.T2 `dd-host` refactor** — move OS-specific code behind the trait; P2 matrix stays green. Exit criterion 2 (Rust part).
- [ ] **P8.T3 C ABI** — `dd-host-capi`, cbindgen header, C stub host that links and runs `dd oscheck` against sim. Exit criterion 2.
- [ ] **P8.T4 `dd oscheck`** — exit criterion 3.
- [ ] **P8.T5 FreeBSD port** (R-P8-1, R-P8-2, R-P8-3) — lab node setup script `lab/setup/freebsd-node.sh`, runner label `hil-freebsd`; exit criterion 4.
- [ ] **P8.T6 Catalogue run** — exit criterion 5; output also feeds the OS-developer view in the R1 portal.
- [ ] **P8.T7 Second-language client** — exit criterion 6.
- [ ] **P8.T8 Porting guide + persona review + gate** — exit criterion 7, `/gate P8`.

## Risks and debugging hints

- Refactoring transports behind `dd-host` can regress P2: run the full HIL matrix before and after; diff reports.
- USB passthrough into a VM adds its own failure layer — label it `lab` in reports so it's not blamed on the port.

## Human actions

Possibly H-014 (spare mini PC for a bare-metal FreeBSD node) if R-P8-2's VM option fails.

## Gate procedure

`/gate P8`, report, merge, tag `gate/P8`.
