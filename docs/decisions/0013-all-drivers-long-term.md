# 0013. Long-term goal: every driver can live on DriverDesk

- Status: accepted · Date: 2026-10-10 · Class: one-way (product direction) · Source: owner direction
- Amends: `docs/vision/SCOPE.md` ("Scope boundaries"), `plans/R4-platform.md`

## Context
SCOPE.md listed GPUs, high-speed NIC/NVMe data paths, boot-critical devices and complex Wi-Fi as "out of scope until R4 (or never)". The owner set the direction that **all** drivers should eventually be able to live on DriverDesk. Evidence that this is technically possible: userspace data paths already beat the kernel for NICs (DPDK) and NVMe (SPDK); most GPU driver code is already userspace (Mesa/Vulkan); microkernel OSes (seL4, QNX, MINIX 3, Fuchsia) run nearly all drivers outside the kernel; IOMMU/VFIO makes DMA from userspace safe.

## Decision
1. The long-term goal is that **every device class** has its driver described, tested, signed and distributed through DriverDesk. "Never" no longer applies to any device class.
2. A DriverDesk description has **multiple execution targets**, chosen per device, per OS, or by the user:
   - interpreted by the DriverDesk runtime (default, sandboxed, portable);
   - compiled to a native driver (kernel module, DriverKit, UMDF, or a microkernel driver) — R3/R4;
   - a kernel-bypass data path (VFIO + polling, DPDK/SPDK-style) for high-throughput devices — R4.
3. **Order does not change**: simple devices first (P0–R3), because they prove the model with the least risk. Hard classes come in this order in R4+: boot-critical (via generated native drivers + on-device descriptions), high-speed storage and network (bypass data paths), Wi-Fi full stack, GPUs (display → compute → full desktop sharing).
4. Firmware *updating* stays integrated with LVFS/fwupd (it is not a driver).
5. Unchanged principles ("never" list now covers only these): no demand for a stable in-kernel ABI on Linux; no closed code inside kernels; no DRM-style secrecy promises.

## Consequences
- Spec 0.1 stays small (P0 "spec bloat" risk), but must not paint us into a corner: the description is target-neutral data; nothing in it may assume "interpreted by our runtime only". Added as a P0.T2 review point.
- The Host Interface (P8) will later need DMA/IOMMU, interrupt and shared-device multiplexing primitives; P8 notes this as future extension, not 0.1 work.
- Kernel-resident targets still reach Linux only through normal upstream review with a human author (CLAUDE.md guardrail).
- Risk: much larger long-term scope. Mitigation: success measures for P–R3 are unchanged; hard classes are gated on R3 adoption.
