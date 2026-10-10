# R4 — Platform

- Status: not started · Depends on: R3 gate · Horizon: years 3–5+
- **R4.T0 is mandatory: expand this plan before building.**

Claim: DriverDesk is how hardware talks to software, including complex devices.

## Directions (each becomes its own stage when R4.T0 runs)

1. **Complex logic modules** for Wi-Fi (fullMAC devices first), cameras (ISP pipelines as Wasm), and GPU research (start with display-only / simple compute devices; no promises).
2. **On-device descriptions**: devices carry their description digest (and optionally the description itself) in a USB BOS capability descriptor or vendor request, signed by the vendor, so they work offline from first boot. Propose as a USB-IF / industry standard.
3. **Larger OS partnerships**: preinstallation agreements, joint pre-release testing, OS-integrated agents.
4. **Performance path**: description → generated native drivers per OS (kernel or DriverKit) for performance-critical classes, verified against the interpreted runtime by differential testing.
5. **Research partnerships** on formal verification of descriptions (Termite-style synthesis, Devil-style checks) with universities.
