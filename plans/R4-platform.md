# R4 — Platform

- Status: not started · Depends on: R3 gate · Horizon: years 3–5+
- **R4.T0 is mandatory: expand this plan before building.**

Claim: DriverDesk is how hardware talks to software, for every device class (ADR-0013).

## Directions (each becomes its own stage when R4.T0 runs)

1. **Complex logic modules** for Wi-Fi (fullMAC devices first), cameras (ISP pipelines as Wasm), and GPUs (display-only → compute → full shared desktop, in that order).
2. **On-device descriptions**: devices carry their description digest (and optionally the description itself) in a USB BOS capability descriptor or vendor request, signed by the vendor, so they work offline from first boot. Propose as a USB-IF / industry standard.
3. **Larger OS partnerships**: preinstallation agreements, joint pre-release testing, OS-integrated agents.
4. **Performance path**: description → generated native drivers per OS (kernel or DriverKit) for performance-critical classes, verified against the interpreted runtime by differential testing.
5. **Research partnerships** on formal verification of descriptions (Termite-style synthesis, Devil-style checks) with universities.
6. **Boot-critical devices**: generated native drivers (direction 4) plus on-device descriptions (direction 2) so storage/display work before userspace starts.
7. **Kernel-bypass data paths** for high-speed NVMe and NICs: VFIO + IOMMU + polling (DPDK/SPDK-style), with a multiplexer so several apps can share one device. Extends the P8 Host Interface with DMA, interrupt and sharing primitives.
