# 0002. Descriptions are authored in YAML and defined by JSON Schema 2020-12

- Status: accepted · Date: 2026-10-10 · Class: one-way · Source: north star §05

## Decision
Authoring format: YAML 1.2 (no anchors/aliases/tags allowed in 0.x — the linter rejects them for predictability). Canonical definition: JSON Schema 2020-12 in `spec/schema/<version>/`. Semantic rules beyond schema live in the validator with stable codes `DD-Lnnn`.

## Why
Same model as Linux devicetree bindings (dt-schema): proven, familiar to kernel people, every language can validate. LLMs write YAML reliably and comments survive.

## Consequences
`serde_yaml` is deprecated; parser choice is research question R-P0-3, wrapped behind `dd_spec::syntax`.
