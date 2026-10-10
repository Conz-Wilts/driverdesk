# P1 — A description alone can drive a device (Linux)

- Status: not started
- Depends on: P0 gate; B exit criterion 5 (hardware lab) before P1.T9
- Branch: `stage/p1`
- Estimate: 2–3 weeks

Claim: the generic runtime reads YAML descriptions and drives real hardware with **zero device-specific code**, including chips behind a USB-to-I²C bridge.

## Exit criteria (the gate)

1. On `lab-linux` (HIL): `dd read temp0`, `dd read humidity0`, `dd read pressure0` return values in Cel, %, Pa within plausible room ranges; `dd write led0 on` / `off` toggles the LED (verified by reading back the firmware's LED state command); BME280 values agree with TMP117 temperature within 2 °C. *Evidence:* `hil-linux.yml` run + JSON report.
2. `cargo xtask lint-generic` passes: no chip names, VID/PIDs, register addresses or device strings in `dd-runtime`, `dd-transport-*`, `dd-cli`.
3. Swapping BME280 for TMP117 on the bus requires only selecting a different YAML file (both on the bus simultaneously at different addresses; tests read both).
4. Safety: `dd write led0 level=255` is rejected by the runtime **before** any USB traffic (asserted by twin traffic log and usbmon capture), with error layer `description/safety`.
5. All descriptions pass their built-in tests on sim (rung 4), virtual USB (rung 5) and HIL (rung 6).
6. Fault handling (sim + vlab): timeout, wrong CRC, short read, endpoint stall and unplug during a transfer each produce a typed error naming the layer, and the runtime recovers (re-opens) after replug. ≥ 10 fault tests.
7. Runs as a non-root user on Linux via documented udev rules.
8. Demo recording (`asciinema`) of the whole flow committed under `docs/reports/media/` and linked from the report.

## Pre-made decisions

- Firmware in Rust with Embassy (`embassy-rp`, `embassy-usb`) on Pico 2; one composite USB device: interface 0 = pico-thermo protocol (bulk, framed: header/len/payload/crc8), interface 1 = I²C bridge (vendor control requests modelled on i2c-tiny-usb, **with our own VID:PID** `1209:0001` so Linux's `i2c-tiny-usb` kernel driver does not bind). MS OS 2.0 descriptors from day one (needed in P2). ADR-0012.
- Thermo uses the RP2350's **internal temperature sensor** + one LED, so interface 0 needs no external parts.
- USB from Rust via `nusb` (ADR-0008). Async runtime: `tokio`.
- Runtime architecture: `Description` (validated model) → `Device` actor per matched device (state machine + command queue) → `Transport` trait stack (`UsbTransport` → `I2cBridgeTransport` → `I2cDevice`) → safety check is a mandatory step inside `Device::call`, not optional middleware.
- CLI is session-based in P1 (`dd run` holds the device while a command runs); the long-lived daemon arrives with the agent in P4.

## Research questions

- **R-P1-1** i2c-tiny-usb protocol: request codes, how reads/writes/repeated starts and NACKs are signalled; limits (max transfer size). · needed by P1.T1 · Default: read `drivers/i2c/busses/i2c-tiny-usb.c` in the kernel and the original i2c-tiny-usb firmware; implement CMD_ECHO, GET_FUNC, SET_DELAY, GET_STATUS, I2C_IO with BEGIN/END flags. · Status: open
- **R-P1-2** embassy-usb: composite devices with a vendor interface + MS OS 2.0 descriptors with per-function (function subset) compatible IDs. · needed by P1.T1 · Default: embassy-usb `msos` module supports it; verify with an example. · Status: open
- **R-P1-3** nusb: current API for control/bulk transfers, interface claiming, hotplug (`watch_devices`), cancellation, timeouts. · needed by P1.T4 · Default: nusb latest (0.2.x as of mid-2026). · Status: open
- **R-P1-4** RP2350 internal temperature sensor formula and accuracy (datasheet section on ADC/temperature). · needed by P1.T1 · Default: 27 − (V − 0.706)/0.001721 °C; ±few °C accuracy, so tests use wide bounds. · Status: open
- **R-P1-5** BME280 calibration readout and compensation (datasheet §4.2.3, Bosch reference code licence). · needed by P1.T7 · Default: implement from the datasheet formulas in CEL `compute` steps; do not copy Bosch code. · Status: open

## Tasks

- [ ] **P1.T0 Re-validate this plan.**
- [ ] **P1.T1 Firmware `firmware/pico-bridge`** (deps: B.T8; R-P1-1, R-P1-2, R-P1-4)
  - Do: composite device as above. Interface 0 commands: hello(0x01), read_temp(0x10, i16le centi-°C), set_led(0x20, on:bool, level:u8), get_led(0x21). Interface 1: i2c bridge on I²C0 (GP4/GP5) at 100/400 kHz. `defmt` RTT logs. A firmware version string in a vendor control request. Serial number = unique chip ID.
  - Accept: HIL smoke: enumerates; `lsusb -v` shows two vendor interfaces + MS OS 2.0 BOS descriptor; echo/hello works from a 20-line nusb test.
- [ ] **P1.T2 Software twin `dd-sim`** (deps: P0 gate)
  - Do: Rust models of pico-thermo, i2c bridge, BME280 (register file, calibration values from the datasheet example, measurement modes), TMP117 (registers, conversion). In-process `SimTransport`; fault injection API (delay, drop, corrupt CRC, NACK, stall, disconnect) driven by a seeded script. Raw-gadget backend (`dd-sim --gadget`) so the same twin appears as a real USB device in vlab.
  - Accept: unit tests of each model against datasheet example numbers; vlab: twin enumerates as `1209:0001`.
- [ ] **P1.T3 Runtime core** (deps: P0 gate)
  - Do: load + validate description → typed model; codecs (int/float types, endianness, framing, CRC catalogue); CEL evaluation with `compute` steps; command execution with timeout/retry policy; state machine executor with triggers (`on: request`, `idle_ms`); safety envelope enforcement (limits, rate limits) as a mandatory step; structured `tracing` with `device`/`layer` fields; typed `DdError { layer, code, message, source }`. `cargo xtask lint-generic`.
  - Accept: unit + property tests; all against `SimTransport`.
- [ ] **P1.T4 `dd-transport-usb`** (deps: P1.T3; R-P1-3)
  - Do: match → open → claim interface (detach kernel driver only if the description sets `allow_detach: true`) → control/bulk/interrupt transfers with timeouts → typed errors mapped to the transport layer; reopen on reconnect.
  - Accept: vlab tests against the twin gadget, including stall and unplug.
- [ ] **P1.T5 Transport stacking + `dd-transport-i2c`** (deps: P1.T4)
  - Do: `I2cBus` trait; `I2cBridgeTransport` implements it using the bridge description's commands (the bridge protocol is data, not code); `I2cDevice` descriptions declare `transport: { i2c: { bus: <bridge id>, address: 0x76 } }`.
  - Accept: sim + vlab tests reading BME280 and TMP117 chip-ID registers through the bridge.
- [ ] **P1.T6 CLI `dd`** (deps: P1.T3–T5)
  - Do: `dd list` (connected devices + which description matches, with match specificity), `dd run <desc...> [-- cmd]`, `dd read <channel>`, `dd write <channel> <args>`, `dd call <command> [k=v]`, `dd test <desc...>`, `dd trace` (live log). `--json` everywhere; `--sim` to use the twin.
  - Accept: snapshot tests of help and output; works with `--sim` on all three hosted OS runners.
- [ ] **P1.T7 Descriptions** (deps: P1.T5; R-P1-5)
  - Do: finalise `devices/examples/{pico-thermo,pico-bridge,bme280,tmp117}.yaml` with full tests (ranges, round trips, safety rejections). Mark project fixtures' safety blocks signed by the project dev key (allowed for our own low-power fixtures; see ADR-0010).
  - Accept: `dd test` green on sim and vlab.
- [ ] **P1.T8 Linux permissions** (deps: P1.T4) — `packaging/linux/70-driverdesk.rules` (uaccess/plugdev for matched devices), docs page, test in vlab that a non-root user can run `dd test`.
- [ ] **P1.T9 HIL workflow** (deps: P1.T1, P1.T7; B exit 5)
  - Do: `hil-linux.yml`: flash pico-bridge → cycle port → `dd test devices/examples/*.yaml --json` → assert cross-check (BME280 vs TMP117) → capture usbmon during the run → upload. Nightly schedule + on PR to `main`.
  - Accept: exit criteria 1, 3, 4, 5 on hardware.
- [ ] **P1.T10 Fault suite** (deps: P1.T4, P1.T2) — exit criterion 6.
- [ ] **P1.T11 Demo + report** — asciinema recording on lab-linux via SSH (scripted with `asciinema rec -c`), report, `/gate P1`.

## Risks and debugging hints

- **Kernel driver binding**: if anything binds to our interfaces on Linux, `dd list` should say so explicitly ("interface 1 is claimed by kernel driver X") — never silently detach.
- **I²C NACKs** after BME280 soft reset: the datasheet requires a start-up time (≈2 ms); put it in the state machine, not as a retry.
- **Endianness**: BME280 calibration mixes little-endian words and split 12-bit fields (dig_H4/H5). Property-test the field extraction against datasheet examples.
- **Flaky HIL**: always power-cycle before tests; record the hub port state; a flaky HIL run is a bug in the lab or runtime, never re-run-until-green.

## Human actions

None beyond B's (wiring diagram for BME280 + TMP117 + LED is part of H-003; Claude writes it with a photo checklist).

## Gate procedure

`/gate P1`, report `docs/reports/P1.md` with the demo link, merge, tag `gate/P1`.
