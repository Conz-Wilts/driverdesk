# Researching

Your training data is stale for fast-moving tools and can be wrong about hardware details. Research is cheap; building on a wrong assumption is expensive.

## When research is mandatory

Run the research gate **before** writing code when any of these is true:

1. **First use** of a crate, library, API, tool, protocol or OS facility in this repo. Check the current version, current API, and known issues. (Do not trust remembered APIs for: nusb, wasmtime, jsonschema, sigstore, tough/TUF, embassy, probe-rs, PyMuPDF/Docling, the Claude API.)
2. **Hardware or OS behaviour** you are going to rely on: USB descriptor rules, driver binding, permissions, kernel interfaces, timing, electrical limits.
3. **A stage plan lists research questions** (`R-<stage>-n`) that the current task depends on.
4. **A one-way-door decision** (see DECIDING).
5. **Debugging escalation** — after the third failed hypothesis (see DEBUGGING).
6. **Legal, licensing, pricing, or policy** questions (licences of dependencies, reverse-engineering law, signing programmes, store/entitlement rules).
7. **A number goes into a document** (a statistic, a price, a limit). Numbers need a source.

Not required for: pure logic you can test locally, code style, things already answered in `docs/research/` within the last 90 days (re-check if older and the topic moves fast).

## How

1. **Check memory first**: `grep -ri "<topic>" docs/research docs/decisions docs/debug`.
2. **Write the question** precisely, with what decision or code depends on the answer.
3. **Source order** (stop when the answer is solid):
   1. The thing itself: source code (read the crate/library source, `cargo doc`, `~/.cargo/registry/src`), datasheets, the spec (USB 2.0, HID, MS OS 2.0, JSON Schema, CEL, WIT), kernel source and `Documentation/`.
   2. Official docs and changelogs/release notes.
   3. Maintainer statements: issue trackers, mailing lists (lore.kernel.org), PR discussions.
   4. Reputable secondary sources (LWN, well-known blogs).
   5. Forums / Q&A — only as hints to verify with 1–3.
4. **Empirical beats reading**: if a 30-line experiment answers it, write the experiment (`spikes/`). Run it. A passing experiment outranks any document.
5. **Time box**: standard 20 min, deep 60 min. If not resolved: take the plan's stated default (every R-question in a plan has one), mark confidence low, add a validation test that will fail loudly if the default is wrong, and continue.
6. Use `WebSearch` mode `standard` first; `extended` for niche/recent topics. Use `WebFetch` for specific pages. Use subagents (`Agent`, type `Explore` or `general-purpose`) for broad sweeps so your context stays clean.

## Record it

Every non-trivial answer goes into `docs/research/R-NNN-slug.md`:

```markdown
# R-NNN. Question in one line
- Date: YYYY-MM-DD · Stage/task: P2.T1 · Plan question: R-P2-2 (if any)
- Confidence: high | medium | low
- Answer: two or three sentences a future Claude can act on.
## Evidence
- Source 1 (URL or path@commit) — what it says
- Experiment: spikes/xyz — result
## Implications
What we will do because of this. Links to ADRs/tasks created.
## Re-check when
Version bump of X / date / event.
```

Then link the note from the plan's research question (replace its "Status: open" with "Answered: R-NNN").
