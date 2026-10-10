---
name: debug
description: DriverDesk debugging protocol - reproduce, locate the failing layer (description, logic, runtime, transport, surface, firmware/hardware), test ranked hypotheses one change at a time, enforce strike limits, fix the root cause and add a regression test. Use whenever a test, build, CI run or hardware run fails unexpectedly.
---

# /debug — find the root cause

Follow `docs/autonomy/DEBUGGING.md`. Summary:

1. Read the whole error; rerun with `RUST_LOG=debug RUST_BACKTRACE=1`.
2. Reproduce as the smallest failing test. Intermittent → loop to get a rate.
3. Locate the layer by climbing the test ladder: sim → virtual USB → HIL; one OS vs all; capture USB traffic (usbmon/USBPcap).
4. Write ≤ 3 ranked hypotheses in STATE.md "Current task notes"; test cheapest first; one change per experiment; record results.
5. Regression → `git bisect run`.
6. Strikes: 3 failed hypotheses → `research` skill. 5 failed or 2 hours → write `docs/debug/D-NNN`, then `decide` skill: alternative approach / upstream repro / mark task `[!] blocked: D-NNN` and move on (add an "Unblock D-NNN" task).
7. Fix the cause (no sleeps/retries/ignores as fixes), add the regression test at the lowest rung, write `docs/debug/D-NNN` if it took > 30 min or was surprising.
