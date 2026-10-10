# The lab: how an AI tests real hardware

Claude cannot plug in cables. So the lab is built once (with human hands) and from then on driven entirely by software: CI jobs flash firmware, power-cycle ports, run tests and upload evidence.

## Layout

```
                 GitHub Actions (hosted)                    rungs 1–5
        ubuntu / macos / windows runners: unit, sim, virtual-USB (Linux VM)

                 Self-hosted lab nodes                      rung 6 (HIL)
 ┌──────────────────────┐ ┌──────────────────────┐ ┌──────────────────────┐
 │ lab-linux            │ │ lab-macos            │ │ lab-windows          │
 │ Raspberry Pi 5       │ │ owner's Mac (or a    │ │ x86 mini PC,         │
 │ Ubuntu Server arm64  │ │ Mac mini later)      │ │ Windows 11           │
 │ runner label:        │ │ runner label:        │ │ runner label:        │
 │   hil-linux          │ │   hil-macos          │ │   hil-windows        │
 └──────────┬───────────┘ └──────────┬───────────┘ └──────────┬───────────┘
   uhubctl-capable hub      uhubctl-capable hub      uhubctl-capable hub
   ├─ port 1: DUT Pico 2 (pico-bridge firmware) + BME280 + TMP117 on I²C
   ├─ port 2: Raspberry Pi Debug Probe (SWD to the DUT: flash, reset, RTT logs)
   └─ port 3: spare / device under trace (P5, P6 devices)
```

## Rules

- Each node is a GitHub Actions **self-hosted runner** registered to the project repo with labels above. Workflows `hil-*.yml` target them. Claude starts runs with `gh workflow run` and reads results with `gh run view --log` / `gh run download`.
- HIL jobs always: (1) power-cycle DUT port, (2) flash firmware under test with `probe-rs`, (3) run tests, (4) upload JSON report + logs + capture, (5) leave the DUT flashed with `known-good` firmware.
- Self-hosted runners only run workflows from the repo's own branches (no fork PRs) — configure `Require approval for all outside collaborators` and restrict runner groups. Security matters: the runner has physical device access.
- `lab-macos` on the owner's daily Mac runs as a separate macOS user with the runner as a launch agent, only during agreed hours (set in the runner's service config). Replace with a dedicated Mac mini when R1 budget allows.
- Lab inventory lives in `lab/inventory.yaml` (node, hub model, port map, DUT serial numbers). Workflows read port numbers from it; nothing hard-coded.

## Virtual lab (no hardware needed)

- `crates/dd-sim`: the software twin — a model of each lab device (pico-thermo, pico-bridge, BME280 register file with real calibration values, TMP117) with fault injection.
- `cargo xtask vlab`: boots a virtme-ng VM with a generic Ubuntu kernel that has `dummy_hcd` + `raw_gadget`; the twin runs inside as a real USB gadget; tests run against it through the real kernel USB stack. Same command in CI and locally (locally needs a Linux host or VM — on the owner's Mac use the Pi 5 over SSH once it exists).

## Setup is a stage B deliverable

`lab/setup/linux-node.sh`, `lab/setup/windows-node.ps1`, `lab/setup/macos-node.sh` are written by Claude and are idempotent. The human runs one command per node (given in HUMAN-ACTIONS) and never has to edit anything.
