---
name: decide
description: DriverDesk decision protocol - settle any technical, product or design choice without asking the owner, using existing plans/ADRs first, then a fixed rubric (safety, plan consistency, reversibility, proven tech, fewer parts, speed, consistency), time boxes and spikes, and record one-way-door decisions as ADRs. Use whenever a choice is not already made in the plan or docs/decisions.
---

# /decide — make the call and record it

Follow `docs/autonomy/DECIDING.md`. Summary:

1. Already decided? Check the task text → `docs/decisions/` → north star → CLAUDE.md. If yes, follow it (reopen only with new evidence + superseding ADR).
2. Classify: trivial (just do it) · two-way door (≤ 15 min, one-line `Decision:` in commit; ADR if cross-task) · one-way door (≤ 2 h incl. research/spikes, full ADR with ≥ 2 options + pre-mortem).
3. Rubric in order: safety/security → north-star/ADR consistency → reversibility → proven & maintained → fewer moving parts → speed to exit criteria → consistency with code. Ties: least surprising to a kernel/embedded engineer → easiest to test → first listed.
4. Close call that matters → spike (≤ 2 h, measurable question) instead of debate.
5. Out of time → most reversible option, ADR `accepted (provisional)` with a review trigger and a task where the trigger fires.
6. Only money / legal terms / identity / physical actions go to the owner — and you still choose *what*; queue it in `progress/HUMAN-ACTIONS.md` with exact steps.
