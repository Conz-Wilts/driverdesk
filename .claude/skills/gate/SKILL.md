---
name: gate
description: Run a DriverDesk stage gate - verify every exit criterion of a stage plan with evidence, write the stage report, merge the stage branch to main and tag it, then prepare the next stage. Use when all tasks in a stage are done, or when asked "/gate <stage>".
---

# /gate <stage> — prove the claim, then ship it

1. Read the stage plan's **Exit criteria**. For each one, run its check (`cargo xtask gate <ID>` runs the scripted ones; HIL criteria need a fresh CI run on the named lab nodes — trigger with `gh workflow run`, wait, download artifacts).
2. A criterion passes only with **evidence**: command output, CI run URL, report file, recording. Sim-only (`[~]`) does not pass a hardware criterion.
3. Any failure → back to `next`/`debug` on that item; the gate is not partially passed.
4. Write `docs/reports/<ID>.md`:
   - claim, result (PASS), date, commit SHA
   - table: criterion → evidence link → notes
   - metrics (test counts, coverage of lint rules, benchmark numbers, timings)
   - what surprised us; decisions made (ADR links); debt created (with task ids in the next stages)
   - demo recording links
5. Run the `security-review` skill on the stage diff if the stage touched agent/registry/signing/sandbox/parsers; fix high findings before merging.
6. Open PR `stage/<id>` → `main` (body = report summary; end with the PR attribution line from the session's system instructions); wait for CI green; merge (squash not allowed — keep task commits); tag `gate/<ID>`; push tag.
7. Update `progress/STATE.md` (Now → next stage) and `plans/README.md` status column. Check the next stage's research questions and pull forward any human actions so they're queued early.
8. Tell the owner in ≤ 6 lines: claim proven, key numbers, link to report, anything they need to do.
