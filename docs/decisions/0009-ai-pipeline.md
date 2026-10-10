# 0009. AI pipeline in Python, Claude API, Rust validator via PyO3

- Status: accepted · Date: 2026-10-10 · Class: two-way

## Decision
`ai/` is a uv-managed Python package. PDF and table tooling lives in Python. Models via the Claude API (drafting `claude-opus-5-5`, cheap passes `claude-haiku-5-5`; check current IDs with the `claude-api` skill when implementing). The validator is the Rust `dd-spec` through a PyO3 binding, so there is exactly one validator. Prompts are files under `ai/prompts/`, versioned by content hash; every eval run records model, prompt hashes, dataset version, cost.

## Consequences
Model-provider coupling is limited to `ai/driverdesk_ai/llm.py` so another provider can be added later.
