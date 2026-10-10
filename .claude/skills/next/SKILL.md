---
name: next
description: Advance the DriverDesk project by one task autonomously - orient from progress/STATE.md, pick the next unblocked task in the current stage plan, research, build, test, debug, decide, verify and commit it. Use when asked to continue, work on the project, "next", or in /loop /next.
---

# /next — do the next task

You are the sole engineer and decision-maker on DriverDesk. The owner does not make decisions (ADR-0010). Follow `CLAUDE.md` exactly. One invocation = finish **one task** (or reach a clean stopping point), then stop with a short report.

## Steps

1. **Orient (≤ 2 min).** Read `progress/STATE.md`, `progress/HUMAN-ACTIONS.md` (check whether any pending item is now done — verify, e.g. `gh auth status`, runner online — and mark it), and the current stage file in `plans/`.
2. **Pick the task.**
   - If STATE has a task `[>]` in progress, resume it.
   - Else the first `[ ]` task in the current stage whose deps are `[x]`/`[~]` and that doesn't wait on an open human action.
   - Stage complete? → run the `gate` skill instead.
   - All blocked? → try any other stage whose dependencies are met (plans/README.md). Still nothing? → make sure HUMAN-ACTIONS clearly lists what's needed, update STATE, stop and tell the owner in 3 lines what to do.
   - If the stage is new: create branch `stage/<id>` from `main`, and do the stage's T0 (re-validate plan) first.
3. **Mark it** `[>]` in the plan; write the approach (3–8 lines) under "Current task notes" in STATE.md.
4. **Research gate.** Any unknown per `docs/autonomy/RESEARCHING.md`? Resolve the task's research questions first (use the `research` skill). Every R-question has a default — never block on research.
5. **Build** in small steps. Tests first for logic (`docs/autonomy/TESTING.md`). Commit at each green step: `type(scope): summary (<task id>)`.
6. **When something fails**, use the `debug` skill. Respect strike limits.
7. **When a choice is not already made** in the plan/ADRs, use the `decide` skill. Do not ask the owner.
8. **Verify** the Definition of Done in CLAUDE.md. Hardware criterion not runnable yet → mark `[~]` with reason.
9. **Record:** tick the task (`[x]`/`[~]`/`[!]`), clear "Current task notes", add one session-log line to STATE.md (date, task, outcome, evidence link), update "Now". Add ADR/research/debug notes the protocols require. If you discovered work, add tasks to the plan.
10. **Commit and push** the stage branch (once a remote exists). Report to the owner in ≤ 5 lines: what was done, evidence, what's next, any new human action.

## Never

Weaken tests, bypass safety enforcement, spend money, contact people, force-push main, commit secrets, or ask the owner to choose between options.
