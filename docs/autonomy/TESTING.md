# Testing

DriverDesk's promise is "it works on every OS, and when it doesn't, we say which layer failed". Tests are how we keep that promise, so testing is designed for an AI developer that cannot look at hardware: everything observable must be machine-checkable.

## The test ladder

Run the cheapest rung that can catch the problem; climb only when needed. Every feature needs at least the rungs marked **required**.

| Rung | What | Where it runs | Required for |
|---|---|---|---|
| 1. Unit | Pure functions, parsers, encoders, CEL evaluation, state machine steps | `cargo nextest`, every OS | all logic |
| 2. Property | `proptest` on parsers/encoders/framing (round trip: encode→decode = id), validator never panics on arbitrary YAML | every OS | parsers, codecs, validator |
| 3. Snapshot | `insta` snapshots of user-facing output: validator diagnostics, CLI output, reports | every OS | anything a human reads |
| 4. Simulation | Runtime + description against the **software twin** (`dd-sim`) through an in-process transport | every OS, CI hosted | every description, every runtime feature |
| 5. Virtual USB | Twin exposed as a real USB device via Linux raw-gadget + dummy_hcd inside a virtme-ng VM; runtime talks to it through the real USB stack | Linux VM (CI + local) | transports, agent hotplug, permissions |
| 6. Hardware-in-loop (HIL) | Real Pico + sensors on lab nodes, flashed and power-cycled by CI | self-hosted runners `hil-linux`, `hil-macos`, `hil-windows` | stage exit criteria that mention hardware |
| 7. Cross-OS matrix | Rungs 1–4 on hosted ubuntu/macos/windows runners; 6 on each lab node | CI | everything from P2 on |

Fault-injection and mutation testing are part of rungs 4–6, not extras: the twin must be able to drop packets, delay, return wrong CRCs, stall endpoints and disconnect, and the harness must show the runtime handles each.

## Rules

1. **Write the test first** for bug fixes (reproduce → red → fix → green) and for every acceptance criterion.
2. **Never weaken a test to pass.** Not by loosening tolerances, adding sleeps, skipping, or `#[ignore]`. If the test is wrong, fix it in a separate commit whose message says why it was wrong.
3. **Flaky = failing.** A test that fails 1 in 50 runs is a bug (usually timing). Reproduce with `cargo nextest run --retries 0 -j1` in a loop (`for i in $(seq 50)`), then fix the cause.
4. **Hardware results are only real if recorded.** HIL jobs upload logs, a JSON report, and (where relevant) the usbmon/USBPcap capture as CI artifacts. Link the run in the task when ticking it.
5. **No device-specific code in generic crates.** `cargo xtask lint-generic` (P1.T3) fails if runtime/transport/surface crates contain chip names, VIDs/PIDs, or register addresses.
6. **Every failure message names a layer**: description, logic, runtime, transport, surface (or `lab` for infrastructure faults). Tests assert the layer, not just failure.
7. **Determinism**: seed all randomness (print the seed on failure); no wall-clock in logic (inject a clock).
8. **Speed budget**: rungs 1–4 for the whole workspace under 3 minutes on a laptop. If over, split or optimise before adding more.

## What "verified" means for a task

- Pure software task: rungs 1–4 green on all hosted OS runners.
- Task touching USB/OS behaviour: + rung 5 green.
- Task whose acceptance criteria name real hardware: + rung 6 green on the named lab nodes. If the lab is not available yet, tick as `[~]` and add the HIL check to the stage gate checklist.

## Evaluating AI components (P3, P6)

AI output is tested like code, against fixed datasets:
- Golden datasets live in `benchmark/` (inputs as URL + SHA-256, ground truth in YAML).
- Every eval run records model id, prompt hash, dataset version, scores and cost in `benchmark/runs/<date>-<id>.json`.
- A prompt or pipeline change must not reduce any headline metric by more than 2 points without an ADR explaining the trade.
- Separate dev and held-out sets. Never tune prompts on the held-out set.

## Usability tests without humans

For human-facing text (validator errors, docs, CLI help), run a **persona review**: spawn a subagent with a persona ("hardware engineer, writes C firmware, never used Rust or YAML schemas") that sees *only* the output, and ask it to state the fix. Score = fraction of fixtures where its fix is correct. Threshold is set per stage plan.
