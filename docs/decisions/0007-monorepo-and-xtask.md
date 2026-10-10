# 0007. Monorepo layout and `cargo xtask`

- Status: accepted · Date: 2026-10-10 · Class: two-way

## Decision
One repository. Layout: `spec/ crates/ xtask/ ai/ firmware/ shims/ portal/ devices/ lab/ deploy/ packaging/ benchmark/ docs/ plans/ progress/`. Project commands are `cargo xtask <cmd>` (doctor, ci, gate, vlab, spec-check, bench-check, lint-generic) — Rust-native, no Make/just dependency, works on Windows.

Crates: dd-spec, dd-spec-py, dd-runtime, dd-transport-{usb,serial,hid,i2c,vfio}, dd-surface-{linux,macos,windows}, dd-sandbox, dd-agent, dd-harness, dd-registry, dd-sim, dd-cli (binary `dd`).
