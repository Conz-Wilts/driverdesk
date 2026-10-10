# 0011. Test ladder, virtual USB lab, self-hosted HIL runners

- Status: accepted · Date: 2026-10-10 · Class: two-way · Details: docs/autonomy/TESTING.md, docs/autonomy/LAB.md

## Decision
Seven-rung test ladder (unit → property → snapshot → simulation twin → virtual USB via raw-gadget/dummy_hcd in a virtme-ng VM → hardware-in-loop → cross-OS matrix). Hardware lab = GitHub Actions self-hosted runners (Pi 5 Linux, Mac, Windows mini PC) with Debug Probes for flashing and uhubctl hubs for power cycling, so an AI can run real-hardware tests with no human present.
