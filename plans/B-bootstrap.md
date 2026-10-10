# B — Bootstrap: the project can build, test and touch hardware without a human

- Status: in progress
- Depends on: —
- Branch: `stage/b`
- Estimate: 1–2 weeks of work; hardware shipping runs in parallel

## Exit criteria (the gate)

1. Repo on GitHub with licences, contribution rules, CI; `main` protected (CI must pass to merge). *Check:* `gh api repos/{owner}/{repo}/branches/main/protection`.
2. Toolchains installed and pinned on the Mac; `docs/toolchain.md` lists versions. *Check:* `cargo xtask doctor` exits 0.
3. Cargo workspace with all planned crates as compiling stubs; `cargo xtask ci` green locally and on hosted ubuntu/macos/windows runners.
4. **Virtual USB lab works:** in CI, a Rust raw-gadget program inside a virtme-ng VM enumerates a fake device with VID:PID `1209:0001` that `lsusb` and `nusb` inside the VM can see. *Check:* `vlab.yml` run green.
5. **Hardware lab works** (needs H-001…H-004 done): on `lab-linux`, CI flashes the Pico 2 via the Debug Probe, power-cycles it via `uhubctl`, and reads the firmware's "hello" log over RTT. *Check:* `hil-smoke.yml` run green on `hil-linux`. `hil-macos` and `hil-windows` smoke runs are required by the P2 gate, not this one.
6. `progress/STATE.md`, `progress/HUMAN-ACTIONS.md`, all protocol docs and ADR-0001…0012 committed.

If hardware has not arrived, criteria 1–4 and 6 are enough to **start P0**. Criterion 5 must be met before P1.T9.

## Pre-made decisions

- Rust workspace + `xtask` pattern for project commands (no Make/just). ADR-0007.
- GitHub + GitHub Actions; self-hosted runners for HIL. ADR-0011.
- pid.codes test VID:PIDs `1209:0001`–`1209:000F` for internal fixtures until we request our own PIDs (R1). ADR-0012.
- Lab nodes: Raspberry Pi 5 (Linux), owner's Mac (macOS), x86 mini PC (Windows). `docs/autonomy/LAB.md`.

## Research questions

- **R-B-1** How to load `dummy_hcd` + `raw_gadget` in GitHub Actions? · needed by B.T6 · Default: virtme-ng booting Ubuntu's generic kernel with `linux-modules-extra` (the approach used in hidapi's CI); build `dummy_hcd` from kernel source if not packaged. · Status: open (lead: hidapi PR #815 workflow)
- **R-B-2** Can the virtual lab run locally on Apple Silicon? · needed by B.T6 (nice-to-have) · Default: no; run vlab locally on `lab-linux` over SSH, and in CI. · Status: open
- **R-B-3** Which per-port-power USB hub is (a) supported by uhubctl, (b) currently buyable in the owner's country (infer from `defaults read -g AppleLocale`; likely New Zealand), (c) ≥4 ports, USB 2.0 or 3.x? · needed by B.T2 · Default: pick from uhubctl's README compatibility table, prefer models listed as "per-port" and recently confirmed. · Status: open
- **R-B-4** Pico 2 (RP2350) support status in `probe-rs` and `embassy-rp` (versions, flashing via Debug Probe). · needed by B.T8 · Default: RP2350 supported by both; if not, fall back to Pico 1 (RP2040) which is certainly supported. · Status: open
- **R-B-5** GitHub self-hosted runner on Raspberry Pi 5 (arm64 Linux) and Windows on ARM vs x86 — supported architectures. · needed by B.T7 · Default: linux-arm64 and win-x64 runners are supported. · Status: open

## Tasks

- [x] **B.T1 Repository skeleton** (deps: —)
  - Do: `.gitignore` (Rust, Python, node, `.env*`, `cache/`, `spikes/*` except README), `LICENSE` (Apache-2.0 full text), `spec/LICENSE` (CC-BY-4.0), `NOTICE`, `README.md` (one-paragraph pitch + status + link to north star), `CONTRIBUTING.md` (DCO sign-off, conventional commits), `SECURITY.md` (private reporting via GitHub advisories), `CODE_OF_CONDUCT.md` (Contributor Covenant). Fetch licence texts from their official sources (research gate: licence texts).
  - Accept: files exist; `git log` has a signed-off commit.
- [>] **B.T2 Hardware shopping list** (deps: —) — *do this first in practice: shipping takes the longest*
  - Do: resolve R-B-3, R-B-4. Produce H-001 in `progress/HUMAN-ACTIONS.md`: exact products, links from a retailer that ships to the owner, quantities, prices, total. Baseline list: 3× Raspberry Pi Pico 2 with headers; 2× Raspberry Pi Debug Probe; 1× Raspberry Pi 5 (8 GB) + official PSU + 64 GB microSD or NVMe; 1× x86 mini PC with Windows 11 (N100-class); 3× uhubctl-compatible per-port-power hubs; BME280 and TMP117 breakouts (STEMMA QT/Qwiic) ×2 each; Qwiic-to-header cables; breadboards, jumper wires; 5× 3 mm LEDs + 330 Ω resistors; 4× good USB-C/micro-USB data cables. Leave P5/P6 devices for those stages.
  - Accept: H-001 written; budget total stated; each item has a reason.
- [x] **B.T3 Toolchain on the Mac** (deps: —)
  - Do: install rustup (stable), targets `thumbv8m.main-none-eabihf` (RP2350), `thumbv6m-none-eabi` (RP2040 fallback), `wasm32-wasip2`, `wasm32-unknown-unknown`; cargo tools `cargo-nextest cargo-deny cargo-insta probe-rs-tools flip-link cargo-component`; `uv`; `gitleaks`. Use Homebrew where the tool has an official formula. Write `rust-toolchain.toml`, `docs/toolchain.md`.
  - Accept: `cargo xtask doctor` (written in B.T4) reports all present with versions.
- [x] **B.T4 Workspace + xtask** (deps: B.T1, B.T3)
  - Do: Cargo workspace per ADR-0007 with stub crates (`dd-spec`, `dd-runtime`, `dd-transport-usb`, `dd-transport-serial`, `dd-transport-hid`, `dd-transport-i2c`, `dd-surface-linux`, `dd-surface-macos`, `dd-surface-windows`, `dd-sandbox`, `dd-agent`, `dd-harness`, `dd-registry`, `dd-sim`, `dd-cli` (binary `dd`)) and `xtask` with subcommands: `doctor`, `ci` (fmt --check, clippy -D warnings, nextest, deny check, gitleaks), `gate <ID>` (runs the stage's scripted checks; stub), `vlab` (stub). `deny.toml` with the licence allowlist from CLAUDE.md. `ai/` as a uv project with ruff + pytest and one passing test.
  - Accept: `cargo xtask ci` green; `cd ai && uv run pytest` green.
- [ ] **B.T5 GitHub repo + CI** (deps: B.T4, H-002)
  - Do: create repo (public, under the org from H-002), push, workflow `ci.yml` (matrix ubuntu-latest, macos-latest, windows-latest; cache cargo; run `cargo xtask ci`; ai tests on ubuntu), branch protection on `main` requiring `ci`. Dependabot for cargo, pip, actions. Enable private vulnerability reporting.
  - Accept: a PR from `stage/b` shows green CI on three OSes.
- [ ] **B.T6 Virtual USB lab** (deps: B.T4; research R-B-1, R-B-2)
  - Do: `lab/vlab/` scripts + `cargo xtask vlab -- <test command>`: boots virtme-ng with a generic Ubuntu kernel, loads `dummy_hcd` and `raw_gadget`, runs a test binary inside. Minimal gadget in `crates/dd-sim/examples/hello_gadget.rs` (raw-gadget ioctls via `nix`/`libc`) exposing `1209:0001`, vendor class, one bulk OUT/IN pair echoing data. Test: from inside the VM, `nusb` lists the device and an echo round-trip succeeds. Workflow `vlab.yml`.
  - Accept: `vlab.yml` green; local run documented (on lab-linux).
- [ ] **B.T7 Lab node setup scripts** (deps: B.T5; research R-B-5; hardware via H-001)
  - Do: idempotent `lab/setup/linux-node.sh` (Ubuntu Server 24.04+ arm64 on Pi 5: packages, udev rules for Pico/Debug Probe/hubs, uhubctl, probe-rs, rustup, GitHub runner as a service with label `hil-linux`, ssh hardening, unattended upgrades), `lab/setup/windows-node.ps1` (OpenSSH server, runner as service `hil-windows`, Rust, probe-rs, uhubctl or vendor tool, USBPcap, Wireshark), `lab/setup/macos-node.sh` (separate user, runner launch agent `hil-macos`, Homebrew deps). `lab/inventory.yaml` template. Write H-003 (Pi 5 imaging + one command) and H-004 (Windows box first boot + one command) in HUMAN-ACTIONS with exact steps, including how to create a short-lived runner registration token with `gh` *run by Claude* and paste nothing secret into chat.
  - Accept: scripts pass `shellcheck` / `PSScriptAnalyzer`; dry-run mode works; after the human runs them, `gh api repos/{o}/{r}/actions/runners` shows the nodes online.
- [ ] **B.T8 Firmware toolchain proof** (deps: B.T3, B.T7; research R-B-4)
  - Do: `firmware/hello/`: embassy-rp blinky that logs "hello <git sha>" over RTT with `defmt`. Workflow `hil-smoke.yml` on `hil-linux`: uhubctl cycle DUT port → `probe-rs run` → assert RTT output contains the sha within 10 s → flash `known-good` (same hello) → upload log.
  - Accept: exit criterion 5.
- [ ] **B.T9 Seed records** (deps: —)
  - Do: verify ADRs 0001–0012, research note R-001, STATE and HUMAN-ACTIONS are consistent with what B actually did; fix drift.
  - Accept: `/gate B` checklist passes.

## Risks and debugging hints

- **dummy_hcd unavailable on hosted runners** (expected): that is why we use virtme-ng. If virtme-ng proves unworkable, fallback is a self-hosted VM on `lab-linux` (QEMU + generic kernel); record as ADR.
- **Pi 5 runner disk wear**: put the runner work dir on NVMe if available; enable log rotation.
- **macOS runner permissions**: probe-rs needs USB access only (no kext); first run may trigger a macOS USB accessory prompt on Apple Silicon ("Allow accessory to connect") — include in H-005 for the human to click once.
- **Windows Defender/SmartScreen** may quarantine probe-rs or the runner: add exclusions in the setup script; document.

## Human actions

H-001 (buy hardware), H-002 (GitHub org + `gh auth`), H-003 (Pi 5 first boot), H-004 (Windows mini PC first boot), H-005 (Mac runner user + accessory prompt). See `progress/HUMAN-ACTIONS.md`.

## Gate procedure

Run `/gate B`. Report: `docs/reports/B.md`. Merge `stage/b` → `main`, tag `gate/B`.
