# P5 — Unmodified apps see a native OS device

- Status: not started
- Depends on: P2 gate (P4 agent recommended)
- Branch: `stage/p5`
- Estimate: 2–3 weeks

Claim: a device driven by a description shows up as a normal OS device of its class, so existing apps work with it unchanged. Plus: an HID device that only ships a vendor app becomes usable.

## Exit criteria (the gate)

1. **Serial surface**: pico-thermo appears as a serial port on Linux (`/dev/ttyDD0` symlink → pty) and macOS (pty path printed by `dd list`); an unmodified terminal (`screen`/`picocom` driven by `expect`) sends `MEAS:TEMP?` and receives a temperature; `LED ON` toggles the LED. SCPI-style command set is **generated** from the description's `expose` block.
2. **Input surface on Linux**: the HID "vendor device" (default: Elgato Stream Deck, see R-P5-1) works via a description; key presses (from the emulator Pico in HIL, see P5.T6, and once from the real device) produce standard key events, read by a `libevdev` test from the uinput device.
3. **Input surface on macOS and Windows**: same description; key presses become OS input events (macOS: virtual HID if entitlement-free path exists, else CGEvent posting with Accessibility permission; Windows: `SendInput`). Limitations documented honestly in `docs/book/os-surfaces.md`.
4. `dd-transport-hid` passes the P2 conformance suite subset that applies to HID.
5. Surface tests are automated (rung 4–6); the OS surface matrix in the north star is updated with measured results.

## Pre-made decisions

- HID transport via `hidapi` crate (hidraw backend on Linux, IOHIDManager on macOS, Windows HID API) — HID devices are accessible from userspace without detaching drivers on all three OSes.
- Linux input via `uinput` (`evdev` crate); serial via pty + stable symlink (CUSE deferred: needs root and is Linux-only).
- Surfaces are generated from class profiles: `serial.scpi` (text protocol over pty), `input.keys` (button → key code map in the description, user-overridable in a local config file).

## Research questions

- **R-P5-1** Choose the HID demo device: must speak a vendor HID protocol (buttons not delivered as keyboard events), cost < US$100, protocol publicly documented by an open-source project, sold in the owner's country. · needed by P5.T2 · Default: Elgato Stream Deck Mini or MK.2 (protocol documented by python-elgato-streamdeck / node-elgato-stream-deck); buttons only, ignore key images in P5 (images → P7 logic module stretch). · Status: open
- **R-P5-2** macOS virtual input options without kernel extensions: IOHIDUserDevice entitlement requirements; CGEventPost + Accessibility (TCC) permissions for a background process; DriverKit VirtualHIDDevice entitlements. · needed by P5.T5 · Default: CGEventPost with Accessibility permission (human action to grant once); record DriverKit route for R1. · Status: open
- **R-P5-3** How to physically "press" keys in HIL without a human: options — skip physical presses and inject HID input reports via a second Pico emulating the device (USB device emulation of the Stream Deck protocol), or a servo. · needed by P5.T6 · Default: a **Pico running a Stream-Deck-protocol emulator** (firmware feature) for HIL, plus one recorded real-device session replayed in sim; real device verified once via H-009 where the human presses 3 keys while a test watches. · Status: open
- **R-P5-4** Licence/IP check on using Elgato's protocol as documented by open-source projects (interoperability). · Default: interoperability use is fine; don't copy GPL code; describe protocol facts only. · Status: open

## Tasks

- [ ] **P5.T0 Re-validate this plan.**
- [ ] **P5.T1 `dd-transport-hid`** — open by match, read/write input/output/feature reports, timeouts; sim backend.
- [ ] **P5.T2 Choose + buy HID device** (R-P5-1, R-P5-4) — H-008 shopping item.
- [ ] **P5.T3 Stream Deck description** — written from public protocol facts; button reports → `input.keys` class; brightness command with safety limits; tests.
- [ ] **P5.T4 Serial surface** (Linux + macOS) — pty, SCPI generator from `expose`, `dd-agent` integration. Exit criterion 1.
- [ ] **P5.T5 Input surfaces** (R-P5-2) — Linux uinput; macOS CGEvent; Windows SendInput. Exit criteria 2–3.
- [ ] **P5.T6 HID HIL** (R-P5-3) — emulator firmware feature `streamdeck-emu` on a spare Pico; HIL tests on all three nodes; one-off real-device check (H-009).
- [ ] **P5.T7 Docs + matrix update + gate** — exit criteria 4–5, `/gate P5`.

## Risks and debugging hints

- **Accessibility/TCC prompts** on macOS cannot be automated: the human grants once (H-010); the test checks for permission and fails with a clear message if missing.
- **uinput permissions**: needs `/dev/uinput` access → udev rule in `packaging/linux`.
- Don't let P5 grow into a Stream Deck app; buttons → keys is enough.

## Human actions

H-008 (buy HID device), H-009 (press 3 keys once during a watched test), H-010 (grant macOS Accessibility to `dd-agent`).

## Gate procedure

`/gate P5`, report, merge, tag `gate/P5`.
