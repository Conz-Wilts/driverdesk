# STATE

> Read this first every session. Keep it short: it is a handoff note, not a diary. Older session log entries move to `progress/log/YYYY-MM.md` when this file passes ~150 lines.

## Now

- **Current stages:** B (bootstrap, B.T6/T7/T8/T9 left) — `plans/B-bootstrap.md`; P0 (spec) started in parallel — `plans/P0-spec.md`
- **Current task:** none. Done: B.T1–B.T5, P0.T0, P0.T1. Next: P0.T2 (spec text v0.1, using R-005 decisions) or B.T6 (virtual USB lab). B.T7/B.T8 wait on H-001.
- **Branches:** `stage/b` (PR #1), `stage/p0` (branched from stage/b). Remote https://github.com/Conz-Wilts/driverdesk

## Current task notes

(none)

## Blockers / waiting on humans

- H-001 (buy hardware, ≈NZ$1,792) open — blocks B.T7, B.T8.

## Open questions being researched

(none yet — see each plan's Research questions)

## Session log (newest first)
- **2026-10-10** — P0 started on `stage/p0` (owner asked whether the driver standard exists: not yet, it is P0). P0.T1: read Linux driver source (TMP117, BMP280/BME280, lm75, i2c-tiny-usb, cp210x, regmap, IIO, usb_device_id, DT bindings) → R-004; survey of SVD, SystemRDL, IP-XACT, Zephyr, ESPHome, SunSpec, HID, device-driver → R-005 with 10 decisions for spec 0.1. P0.T0: plan updated (variants, instance parameters, CEL bitwise library).
- **2026-10-10** — B.T5: public repo https://github.com/Conz-Wilts/driverdesk (owner's choice, H-002 done). `ci.yml` green on ubuntu/macos/windows (run 38089187304, draft PR #1). `main` protected (required check `ci`, admins included). Private vuln reporting on; fork-PR workflows need approval for all external contributors (self-hosted runner safety).
- **2026-10-10** — Stage B started on `stage/b`. B.T1 skeleton (licences, README, CONTRIBUTING, SECURITY, CoC); B.T2 H-001 shopping list (R-002 hubs, R-003 RP2350); B.T3 toolchain pinned (Rust 1.99.0, docs/toolchain.md, `cargo xtask doctor` green); B.T4 workspace with 15 stub crates + xtask + ai/ (`cargo xtask ci` green locally).
- **2026-10-10** — Added docs/vision/SCOPE.md (three users: hardware makers, OS writers, every OS) and stage P8 (OS porting kit, OS conformance, catalogue runs, second client). P4 gains a client protocol spec; R1 gains an OS developer portal.

- **2026-10-10** — Planning session. Created operating manual (CLAUDE.md), autonomy protocols (docs/autonomy), stage plans B, P0–P8, R1–R4, ADR-0001…0012, research note R-001, human action queue, project skills (/next /research /debug /decide /gate). North star: docs/vision/driverdesk-plan.html. No code yet.
