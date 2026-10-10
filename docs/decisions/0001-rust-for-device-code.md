# 0001. Rust for all device-touching and untrusted-input code

- Status: accepted · Date: 2026-10-10 · Class: one-way · Source: north-star plan §07

## Context
The runtime, transports, agent, validator and registry handle untrusted input (descriptions, USB traffic, network data) and run with device access. Memory-safety bugs here become security bugs.

## Decision
Rust (stable, edition 2024) for: spec parser/validator, runtime, transports, OS surfaces where the platform allows, sandbox host, agent, harness, registry index service, firmware for lab devices (Embassy). C/C++ only where an OS demands it (Windows KMDF/UMDF, macOS DriverKit shims), kept tiny and audited.

## Consequences
One shared `dd-spec` crate is reused by CLI, runtime, Python (PyO3), browser (Wasm). Rust is now permanent in the Linux kernel (Dec 2025), so kernel-facing work in R3 is culturally aligned.
