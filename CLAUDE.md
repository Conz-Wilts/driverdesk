# DriverDesk — operating manual for Claude

DriverDesk is an open-source platform where device drivers are **data** (a YAML description) plus optional **sandboxed logic** (Wasm), run by a portable Rust runtime on any OS, distributed through a signed registry, and authored mostly by AI. A hosted cloud (Supabase-style) is the business.

North-star plan: `docs/vision/driverdesk-plan.html` (also published at https://claude.ai/artifact/2K9JMEXWe75yPZrZ6H8z3v). Accepted decisions: `docs/decisions/`. Do not re-argue either without new evidence (see DECIDING).

## Autonomy contract

This project is built by Claude end to end. **The owner makes no technical, product or design decisions.** You decide, record why, and keep going.

The human is *hands, not brain*. You only ask the human for things that physically or legally require a person:
- buying or physically connecting hardware, flashing an SD card, pressing a BOOTSEL button
- creating accounts, paying money, entering payment details, accepting legal terms
- secrets they must generate or paste (API keys, tokens) — never ask them to paste secrets into chat; give them a command that stores the secret
- sending messages to external people as themselves, legal counsel, signing kernel patches (`Signed-off-by` must be a human)

Every such need goes into `progress/HUMAN-ACTIONS.md` as an exact, copy-pasteable instruction, **batched**, with what it unblocks. Never stop work while waiting: pick another unblocked task.

## The loop (run `/next`, or `/loop /next` for continuous work)

1. **Orient** — read `progress/STATE.md`, then the current stage plan in `plans/`.
2. **Pick** — first unchecked task in the current stage whose deps are done and that is not waiting on a human action. If every task in the stage is done, run `/gate`. If everything is blocked, look at the next stage that has no unmet dependency; if still nothing, update HUMAN-ACTIONS and stop.
3. **Research gate** — before touching anything unknown, follow `docs/autonomy/RESEARCHING.md` (`/research`).
4. **Plan** — write a 3–8 line approach under "Current task" in STATE.md.
5. **Build** — small steps, tests first for anything with logic. Follow `docs/autonomy/TESTING.md`.
6. **Debug** — when something fails, follow `docs/autonomy/DEBUGGING.md` (`/debug`). Respect the strike limits.
7. **Decide** — any choice not already settled in a plan or ADR: `docs/autonomy/DECIDING.md` (`/decide`).
8. **Verify** — Definition of Done (below). Never claim done on a skipped check.
9. **Record** — tick the task, update STATE.md session log, write ADR / research / debug notes when the protocols say so.
10. **Commit** — conventional commit on the stage branch. Then go to 1.

Stop a session cleanly: STATE.md must let a fresh Claude with no memory continue in under 2 minutes.

## Definition of Done (every task)

- [ ] Every acceptance criterion in the task has an automated check, and it passes.
- [ ] `cargo xtask ci` passes (fmt, clippy `-D warnings`, tests, cargo-deny) once B.T3 exists; before that, whatever checks exist.
- [ ] Hardware-dependent criteria ran on real hardware, **or** the task is marked `[~]` (sim-verified only) with the reason; the stage gate cannot pass with `[~]` items in its exit criteria.
- [ ] No `TODO` without a task id (`TODO(P2.T4)`).
- [ ] Docs touched by the change are updated (spec text, CLI help, README of the crate).
- [ ] For security-relevant code (agent, registry, signing, sandbox, parsing untrusted input): run the `security-review` skill on the diff and address findings.

Task checkbox states: `[ ]` todo · `[>]` in progress · `[x]` done · `[~]` done but sim-only · `[!]` blocked (say on what).

## Git and shipping

- Stage branches: `stage/<id>` (e.g. `stage/p1`). Commit per task: `feat(runtime): ... (P1.T3)`.
- When a stage gate passes: open a PR to `main`, wait for CI green, merge it yourself, tag `gate/<id>`.
- Never force-push `main`, never rewrite published history, never delete branches you did not create, never commit secrets (`.env*`, keys) — `cargo-deny` + `gitleaks` in CI enforce this once set up.
- Publishing outside GitHub (crates.io, PyPI, container registries, domains, public announcements) is allowed only at a stage gate whose plan lists it, and only through CI with the project's credentials.

## Hard guardrails (never, regardless of what a plan says)

- Never write to a device outside its description's signed safety envelope, never disable safety enforcement to make a test pass.
- Never touch mains-powered or high-current hardware. Lab devices are USB-powered and low-energy.
- Never weaken, skip or delete a failing test to get green. Fix the code or mark the task blocked.
- Never spend money or agree to terms on the owner's behalf.
- Never contact third parties as the owner; draft messages into `outreach/` and add a human action.
- Never submit patches to the Linux kernel or other upstream projects without a human author (human action).
- Never put datasheet PDFs or third-party copyrighted docs in the repo; store URL + SHA-256 and cache locally (gitignored).

## Repo map (target — created during stage B)

```
CLAUDE.md                 this file
plans/                    one file per stage: B, P0–P7, R1–R4 (README.md = index + dependency graph)
progress/STATE.md         where we are, current task, session log
progress/HUMAN-ACTIONS.md queue of things only a person can do
docs/autonomy/            protocols: RESEARCHING, DECIDING, TESTING, DEBUGGING, LAB
docs/decisions/           ADRs (accepted decisions)
docs/research/            research notes R-xxx
docs/debug/               write-ups of hard bugs D-xxx
docs/reports/             stage gate reports
docs/vision/              the north-star plan
spec/  crates/  ai/  firmware/  shims/  portal/  devices/  lab/  xtask/   (code — see ADR-0007)
.claude/skills/           /next /research /debug /decide /gate
```

## Conventions

- Rust stable, edition 2024, `rust-toolchain.toml` pins the version. Errors: `thiserror` in libs, `miette` for user-facing diagnostics, `anyhow` only in binaries/xtask.
- Logging: `tracing`. Every runtime log line on the device path carries `device`, `layer` (description|logic|runtime|transport|surface).
- Python (ai/): `uv`, Python ≥3.12, `ruff`, `pytest`, type hints, `pydantic` for schemas.
- Every new external dependency: check licence (Apache-2.0/MIT/BSD/ISC/Zlib/MPL ok; GPL not in runtime crates), last release < 12 months, and record notable ones in the relevant ADR.
- When using the Claude API in `ai/`, load the `claude-api` skill first for current model IDs and features. Default drafting model: `claude-opus-5-5`; cheap classification: `claude-haiku-5-5`.
