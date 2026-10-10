# Deciding

The owner does not make decisions. You do. This protocol makes your decisions consistent, fast, and reviewable later.

## 0. Is it already decided?

Check, in order, and stop at the first hit:
1. The task text in the stage plan (a "Decided:" line or a default).
2. `docs/decisions/` (ADRs). Accepted ADRs are binding.
3. The north-star plan `docs/vision/driverdesk-plan.html`.
4. `CLAUDE.md` conventions.

If found, follow it. Reopening an accepted decision requires **new evidence** (a failed spike, a benchmark, a dead dependency, a security issue) and a superseding ADR — never taste.

## 1. Classify

| Class | Examples | Process |
|---|---|---|
| **Trivial** | variable names, file layout inside a crate, which helper to write | Just decide. No record. |
| **Two-way door** (cheap to reverse, < 1 day to undo) | choice of a utility crate, internal API shape, test structure, CLI flag names before v1 | Decide in ≤ 15 min with the rubric. One line in the commit message: `Decision: X over Y because Z.` If it affects more than one task, add a short ADR. |
| **One-way door** (expensive to reverse, or visible to outsiders) | spec field semantics, on-disk/wire formats, trust roots and key handling, licences, public names, anything published, hardware purchases, paid services | Full ADR (template below) with ≥ 2 real options, research gate done, pre-mortem written. Then decide and proceed. |

When unsure which class: treat it as one class heavier.

## 2. Rubric (apply in this order; earlier criteria dominate)

1. **Safety and security.** Never pick an option that weakens the safety envelope, signing, sandboxing or privacy.
2. **Consistency with the north-star plan and accepted ADRs.**
3. **Reversibility.** Prefer the option that keeps future options open.
4. **Proven and maintained.** Prefer widely used, actively maintained tools (release in last 12 months, real users, responsive issues). Prefer standards over inventions (JSON Schema, CEL, OCI, Sigstore, TUF, WIT, UCUM).
5. **Fewer moving parts.** Fewer dependencies, fewer languages, fewer services.
6. **Speed to the stage exit criteria.**
7. **Consistency with existing code.**

Tie-breakers, in order: the option a competent kernel/embedded engineer would find least surprising → the option that is cheaper to test automatically → the first option you listed.

## 3. Evidence beats argument

If two options are close and the difference matters, **spike** instead of debating: a throwaway experiment in `spikes/<topic>/` (gitignored except README), max 2 hours, with a measurable question ("does nusb claim interface 1 on macOS while interface 0 is in use?"). Record the result in the ADR.

## 4. Time box

- Two-way door: 15 minutes, then pick the most reversible option.
- One-way door: 2 hours including research and spikes. If still unclear, pick the most reversible option, mark the ADR `Status: accepted (provisional)` with a **review trigger** ("revisit when P3 benchmark results exist") and add that review as a task in the plan where the trigger happens.

## 5. Things you must not decide alone

There are very few. They are not decisions — they are actions that legally or physically need a person:
spending money, accepting terms of service, creating accounts in the owner's name, contacting people as the owner, human sign-off required by external policy (kernel `Signed-off-by`). For these you still make the choice (which hub to buy, which wording to send), then queue the action in `progress/HUMAN-ACTIONS.md` with the exact instruction.

## ADR template

File: `docs/decisions/NNNN-short-slug.md` (next free number).

```markdown
# NNNN. Title

- Status: proposed | accepted | accepted (provisional) | superseded by NNNN
- Date: YYYY-MM-DD
- Class: two-way | one-way
- Stage/task: P1.T4
- Review trigger: (provisional only) what event reopens this

## Context
What forces this decision. Link research notes.

## Options
### A. ...
Pros / cons / evidence.
### B. ...

## Decision
We choose X.

## Rubric check
1 safety ✓ … 6 speed ✓ (one line each, only where relevant)

## Pre-mortem (one-way only)
It is 12 months later and this decision failed. Most likely reasons: ...
Early warning signs we will watch: ...

## Consequences
What becomes easier, harder, and what follow-up tasks this creates.
```
