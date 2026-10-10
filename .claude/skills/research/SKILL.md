---
name: research
description: DriverDesk research gate - answer an unknown (library API, hardware/OS behaviour, plan research question R-xx, legal/licensing, a number for a document) from primary sources or a quick experiment, then record it in docs/research. Use before first use of any library or OS facility and after 3 failed debugging hypotheses.
---

# /research — answer an unknown and record it

Follow `docs/autonomy/RESEARCHING.md`. Summary:

1. `grep -ri` the topic in `docs/research docs/decisions docs/debug` — maybe it's answered.
2. Write the precise question and what depends on it.
3. Sources in order: the thing itself (source code in `~/.cargo/registry/src`, datasheet, spec, kernel source) → official docs/changelogs → maintainer statements → reputable secondary → forums (hints only). `WebSearch` mode standard first, extended for niche/recent. For broad sweeps, delegate to an `Explore`/`general-purpose` subagent and keep only its conclusion.
4. If a ≤ 30-line experiment can answer it, do that in `spikes/<topic>/` — evidence beats documents.
5. Time box 20 min (deep: 60). Unresolved → take the plan's default, confidence low, add a test that fails loudly if the default is wrong.
6. Write `docs/research/R-NNN-slug.md` (template in RESEARCHING.md) and link it from the plan's research question ("Answered: R-NNN").
