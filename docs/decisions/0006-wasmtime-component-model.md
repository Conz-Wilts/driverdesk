# 0006. Logic modules: WebAssembly components on Wasmtime

- Status: accepted · Date: 2026-10-10 · Class: one-way · Source: north star §06; WASI drivers paper (arXiv 2410.22919) measured ≤ 8% USB overhead

## Decision
Vendor logic is a Wasm component targeting worlds defined in `spec/wit/driverdesk-logic`. Host: Wasmtime. No WASI capabilities by default; only `driverdesk:logic/device` imports, which route through the runtime's safety-enforced command layer.

## Consequences
"Private" logic is access-controlled, not secret (Wasm can be decompiled). Say so in docs.
