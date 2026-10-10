# Human actions

Things only a person can do (physical, money, legal, identity). Claude has already made every decision; each item says exactly what to do. Do the **open** items in any order; tell Claude "done H-00x" (or just run `/next` — it checks).

Rules for Claude: batch requests; give copy-pasteable commands; never ask for secrets in chat (give a command that stores them); say what each item unblocks; mark items `done` with the date when verified.

| ID | Status | What | Unblocks |
|---|---|---|---|
| H-001 | pending (Claude writes it in B.T2) | Buy lab hardware from the list Claude prepares | B.T7, B.T8, P1 HIL |
| H-002 | **open** | Create GitHub home + let Claude use `gh` | B.T5 (CI) |
| H-003 | pending (B.T7) | Set up Raspberry Pi 5 lab node (flash card, plug in, run one command) | HIL Linux |
| H-004 | pending (B.T7) | Set up Windows mini PC lab node (first boot, run one command) | HIL Windows |
| H-005 | pending (B.T7) | Create macOS lab-runner user on your Mac, allow USB accessories | HIL macOS |
| H-006 | pending (P3) | Anthropic API key + spend cap | P3 AI drafter |
| H-007 | pending (R1) | Production key ceremony (hardware security keys) | R1 |
| H-008 | pending (P5) | Buy HID demo device | P5 |
| H-009 | pending (P5) | Press 3 buttons on the HID device during a watched test | P5 gate |
| H-010 | pending (P5) | Grant macOS Accessibility to `dd-agent` | P5 |
| H-011 | pending (P6) | Ask a lawyer to review the reverse-engineering checklist | P6 publishing |
| H-012 | pending (P6) | Buy trace target device + USB webcam | P6 |

---

## H-002 — GitHub home for the project (open)

1. Decide nothing: Claude's choice is a **GitHub organisation** named `driverdesk` (if taken, use `driverdesk-dev`, then `driverdesk-org`). Create it (free plan) at https://github.com/organizations/plan while signed in as `Conz-Wilts`.
2. In this terminal, make sure `gh` can create repos and manage Actions for the org:
   ```
   gh auth refresh -h github.com -s repo,workflow,admin:org
   ```
   (`admin:org` lets Claude register the lab machines as self-hosted runners.)
3. Tell Claude the org name you got.
