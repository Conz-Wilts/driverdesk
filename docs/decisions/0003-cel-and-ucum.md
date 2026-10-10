# 0003. CEL for expressions, UCUM for units

- Status: accepted · Date: 2026-10-10 · Class: one-way

## Decision
Conversions, guards and computed values use Common Expression Language (CEL): non-Turing-complete, sandboxed, used by Kubernetes. Multi-step math uses a `compute:` list of named CEL steps. Units use UCUM codes (`Cel`, `Pa`, `%`, `mA`), validated against a curated subset table.

## Why
Never invent an expression language; never leave units as free text (unit errors are a classic driver bug).

## Consequences
If CEL cannot express something reasonably, it belongs in a Wasm logic module (P7).
