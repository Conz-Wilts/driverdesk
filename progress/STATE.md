# STATE

> Read this first every session. Keep it short: it is a handoff note, not a diary. Older session log entries move to `progress/log/YYYY-MM.md` when this file passes ~150 lines.

## Now

- **Current stage:** P1 (Linux runtime) — `plans/P1-linux-runtime.md`; B still open (B.T6, B.T9 doable; B.T7/B.T8 wait on H-001). P0 gate passed 2026-10-10 (`docs/reports/P0.md`, tag `gate/P0`).
- **Current task:** none. Next: P1.T0 (re-validate P1 plan; include D-002 stack note), B.T6 (virtual USB lab, needed by P1 transport tests). Done: B.T1–B.T5, P0.T0–T7, T9–T11. Next: `/gate P0`, then B.T6 (virtual USB lab) and P1. B.T7/B.T8 wait on H-001.
- **Branches:** `stage/b` (PR #1), `stage/p0` (branched from stage/b). Remote https://github.com/Conz-Wilts/driverdesk

## Current task notes

(none)

## Blockers / waiting on humans

- H-001 (buy hardware, ≈NZ$1,792) open — blocks B.T7, B.T8.

## Open questions being researched

(none yet — see each plan's Research questions)

## Session log (newest first)
- **2026-10-10** — P0 gate PASS (`docs/reports/P0.md`): spec 0.1, 29 lint rules, 57 fixtures, persona 98.2%, fuzz clean after D-001, benchmark 50/50 verified. PR #2 merged, tag `gate/P0`.
- **2026-10-10** — Owner asked to build the whole standard incl. complex devices. P0 built on `stage/p0`: spec text `spec/driverdesk-0.1.md` (stable core + provisional §20–22 for PCI/DMA/queues/streams/firmware/logic), JSON Schemas `spec/schema/0.1/`, 7 examples (incl. BME280 full compensation, HID keyboard, NVMe admin queue), `dd-spec` validator (YAML spans, schema, 29 lint rules, CEL library, UCUM, CRC catalogue), `dd validate`, 57 fixtures + snapshots, `cargo xtask spec-check|bench-check|docs`, proptest 3×100k pass, mdBook with tutorial + 38 error pages, benchmark 50 chips (online-verified), persona usability 98.2% (`docs/reports/P0-persona.md`). Research R-006 (crates). Fuzz: first run found D-001 (arithmetic on invalid input) → fixed; rerun 261,363 execs / 600 s clean. Windows stack overflow D-002 fixed. Security review: no findings ≥ 8/10; hardening applied (control-char escaping, tag rejection). PR #2 CI green on 3 OSes.
- **2026-10-10** — P0 started on `stage/p0` (owner asked whether the driver standard exists: not yet, it is P0). P0.T1: read Linux driver source (TMP117, BMP280/BME280, lm75, i2c-tiny-usb, cp210x, regmap, IIO, usb_device_id, DT bindings) → R-004; survey of SVD, SystemRDL, IP-XACT, Zephyr, ESPHome, SunSpec, HID, device-driver → R-005 with 10 decisions for spec 0.1. P0.T0: plan updated (variants, instance parameters, CEL bitwise library).
- **2026-10-10** — B.T5: public repo https://github.com/Conz-Wilts/driverdesk (owner's choice, H-002 done). `ci.yml` green on ubuntu/macos/windows (run 38089187304, draft PR #1). `main` protected (required check `ci`, admins included). Private vuln reporting on; fork-PR workflows need approval for all external contributors (self-hosted runner safety).
- **2026-10-10** — Stage B started on `stage/b`. B.T1 skeleton (licences, README, CONTRIBUTING, SECURITY, CoC); B.T2 H-001 shopping list (R-002 hubs, R-003 RP2350); B.T3 toolchain pinned (Rust 1.99.0, docs/toolchain.md, `cargo xtask doctor` green); B.T4 workspace with 15 stub crates + xtask + ai/ (`cargo xtask ci` green locally).
- **2026-10-10** — Added docs/vision/SCOPE.md (three users: hardware makers, OS writers, every OS) and stage P8 (OS porting kit, OS conformance, catalogue runs, second client). P4 gains a client protocol spec; R1 gains an OS developer portal.

- **2026-10-10** — Planning session. Created operating manual (CLAUDE.md), autonomy protocols (docs/autonomy), stage plans B, P0–P8, R1–R4, ADR-0001…0012, research note R-001, human action queue, project skills (/next /research /debug /decide /gate). North star: docs/vision/driverdesk-plan.html. No code yet.
