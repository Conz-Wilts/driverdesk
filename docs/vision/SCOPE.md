# DriverDesk — Product Scope

> The single statement of what we are building and for whom. Every stage plan must trace back to a section here. Changes to this file need an ADR.

## Goal

**One place where hardware drivers live, get tested against every operating system, and connect to any OS — eventually for every device class (ADR-0013).**

- Hardware makers upload a driver once (as a description + optional logic module) and see it tested on every supported OS.
- OS writers implement one small host interface and immediately get the whole driver catalogue, plus a test service that tells them which drivers work on their OS and why the others don't.
- Every OS, existing or new, fetches drivers from the same signed storage through one open client protocol.

The kernel stops being the place where device-specific code accumulates. Drivers become portable, testable, signed data.

## Three users, three promises

### 1. Hardware makers: "Upload once, works everywhere, proven"

| Capability | What it means | Built in |
|---|---|---|
| Upload | Upload a datasheet, capture, or existing driver → AI drafts the description → review field by field with evidence → sign → publish (public or private) | P3, P6, R1 portal |
| Test against every OS | Every upload runs the built-in tests + conformance suite in simulation, virtual USB and on real hardware on Linux, macOS, Windows (more OSes as they join) | P2, R1 lab |
| Compatibility report | Device × OS × test matrix; every failure names the responsible layer (description, logic, runtime, transport, OS surface) | P2 |
| Own the driver | Vendor holds keys, controls versions, channels, revocation; AI assists, never signs | P4, R1 |
| Certification | Lab-verified "Verified on <OS>" badge and public device page | R2 |

### 2. OS writers: "Port one runtime, get every driver, see exactly what's missing"

| Capability | What it means | Built in |
|---|---|---|
| Host Interface spec | The small, documented set of things an OS must provide: device enumeration + hotplug, raw bus access (USB transfers, HID reports, serial, I²C, later PCI with IOMMU), process isolation, timers, and optional class surfaces (input, serial, net, audio, camera, block). Nothing else. | P8 |
| Porting kit | `dd-host` trait in Rust with reference implementations (Linux, macOS, Windows) and a C ABI for OSes not using Rust; a step-by-step porting guide | P8 |
| OS conformance suite | Runs on the OS being ported: checks each Host Interface function against the spec (timing, errors, hotplug, isolation) with the software twin and, where the OS can, virtual USB | P8 |
| Catalogue test run | "Run every driver in the registry against my OS": simulation for all, real hardware in our lab for the OS image they upload (or their own lab runner) → per-driver pass/fail + layer blame + list of missing class surfaces ranked by how many devices they unlock | P8 (sim, own runner), R1–R2 (hosted lab) |
| OS partner channel | Pre-release testing of OS updates against the catalogue so OS changes don't break drivers by surprise | R2–R3 |

### 3. Every OS and its users: "Plug in, it works, it's genuine"

| Capability | What it means | Built in |
|---|---|---|
| Driver storage | Content-addressed, signed, transparency-logged storage of descriptions and logic modules (OCI artifacts); public and private; self-hostable node or our hosted cloud | P4 |
| Open client protocol | Documented, versioned protocol (HTTP + OCI distribution + TUF + Sigstore verification) so any OS can write its own agent, not just use ours | P4 (spec), P8 (second implementation) |
| Agent | Reference client: detect device → look up by match key → fetch → verify → run sandboxed; offline bundles; revocation; no user tracking | P4 |
| On-device descriptions | Devices carry a signed description digest so they work offline from first boot | R4 |

## Scope boundaries

**In scope (until R3):** USB, HID, serial, I²C/SPI behind bridges, PCI via VFIO (R2); device classes sensors, actuators, instruments, input, serial, simple network/audio/camera/storage via userspace surfaces.

**Later (R4+, in this order — ADR-0013):** boot-critical devices (generated native drivers + on-device descriptions), high-speed NVMe/NIC data paths (kernel-bypass via VFIO/IOMMU), full Wi-Fi, GPUs (display → compute → shared desktop). Long-term goal: every device class.

**Integrate, don't build:** firmware updating (LVFS/fwupd).

**Never:** a stable in-kernel driver ABI demand on Linux; closed code inside kernels; DRM-style secrecy promises for "private" drivers.

**Execution targets:** a description is target-neutral data. It can be run by the DriverDesk runtime (default), compiled to a native driver (kernel module, DriverKit, UMDF, microkernel driver), or drive a kernel-bypass data path — chosen per device, OS or user.

## Success measures

| Measure | P-stages (proof) | R2 (beta) | R3 (1.0) |
|---|---|---|---|
| Devices with a verified description | 5 | 50+ | hundreds |
| OSes passing the OS conformance suite | 3 + 1 port (P8) | 4 | 5+ incl. a non-Unix/hobby OS |
| Median time datasheet → passing driver | < 1 day (P3) | < 2 h | < 30 min |
| Layer-attribution accuracy | ≥ 95% (P2) | ≥ 97% | ≥ 98% |
| Paying vendors | — | yes | sustainable |

## Map to plans

B → foundations · P0 spec · P1 runtime · P2 cross-OS tests + reports · P3 AI upload/drafting · P4 storage, client protocol, agent · P5 native OS surfaces · P6 drivers without the vendor · P7 logic sandbox · **P8 OS porting kit + OS conformance + catalogue runs** · R1 hosted cloud (vendor portal, OS developer portal, hosted lab) · R2 public beta, certification · R3 1.0 + foundation · R4 platform.
