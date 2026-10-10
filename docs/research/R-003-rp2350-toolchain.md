# R-003. Is RP2350 (Pico 2) supported by probe-rs and embassy-rp?
- Date: 2026-10-10 · Stage/task: B.T2 (B.T8) · Plan question: R-B-4
- Confidence: high (versions), medium-high (Debug Probe flash + RTT in practice)
- Answer: Yes. probe-rs flashes RP235x since v0.27.0 (chip name `RP235x`); current 0.32.0 (installed). embassy-rp 0.10.0 supports it via feature `rp235xa` (Pico 2 = RP2350A). Firmware must include the RP2350 image-definition block and `memory.x` from embassy's `examples/rp235x`, or it will not boot.

## Evidence
- probe-rs 0.27.0 release notes: "Support added for flashing RP235x targets". https://github.com/probe-rs/probe-rs/releases/tag/v0.27.0
- Tracking issue closed 2025-02-01, users confirm defmt RTT. https://github.com/probe-rs/probe-rs/issues/2787
- embassy-rp Cargo.toml (main): features `rp2040`, `rp235xa`, `rp235xb`. https://raw.githubusercontent.com/embassy-rs/embassy/main/embassy-rp/Cargo.toml · https://crates.io/crates/embassy-rp

## Implications
- Pico 2 stays the DUT; no RP2040 fallback purchase needed.
- B.T8: start from embassy `examples/rp235x` linker setup; `probe-rs run --chip RP235x`. Use SWD (Debug Probe), not UF2.

## Re-check when
embassy-rp or probe-rs minor bump · B.T8 first flash.
