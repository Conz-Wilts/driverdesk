# R1 — Alpha with design partners + hosted cloud alpha

- Status: not started · Depends on: P0–P8 gates · Branch: `stage/r1` · Horizon: months 7–13
- **R1.T0 is mandatory: expand this plan to P1-level detail before building** (tasks with acceptance criteria, research questions with defaults, exit criteria with checks).

Claim: real vendors describe real products on our platform, mostly without us, and a hosted service runs it for them.

## Exit criteria (draft — finalise in R1.T0)

1. 3–5 design partners, each with ≥ 1 product passing tests on Linux, macOS, Windows; ≥ 1 real end customer using one of them.
2. **At least one partner published a description with zero edits by us** (the "vendor owns the driver" test).
3. Hosted cloud alpha: vendor portal (sign-in, upload datasheet, review AI draft field-by-field with evidence, run lab tests, view reports, publish private/public), managed registry nodes, vendor-held keys (BYOK via cloud KMS).
4. Spec v0.5 with device classes: sensors, simple actuators/controllers, serial instruments (SCPI), HID, simple USB peripherals; validator and runtime support them.
5. Device lab v1: dedicated hardware (Mac mini replaces owner's Mac), ≥ 2 OS versions per OS, lab utilisation dashboard.
6. Production trust roots (TUF root, signing identities) created via a documented key ceremony.
7. ≥ 1 OS project outside Linux/macOS/Windows has used the OS developer portal and published a catalogue report.
8. External security review of agent + registry booked (done before R2 public beta).

## Workstreams (expand into tasks in R1.T0)

- **Partners** (human-assisted): Claude builds a target list (robotics, lab instruments, maker hardware, audio) with reasons, writes personalised outreach drafts in `outreach/`, a one-page pitch, demo videos from P0–P7, and a partner onboarding guide. Human sends messages (H-0xx) and joins calls; Claude prepares briefs and follow-ups, and turns call notes into requirements.
- **Portal** (Next.js + TypeScript; validator in browser via Wasm build of `dd-spec`). Hosting decision via DECIDING (candidates: Vercel for portal, Fly.io/Railway/AWS for registry + Postgres + object storage).
- **OS developer portal**: OS writers register an OS, upload an OS image or connect their own lab runner, run `dd oscheck` and catalogue runs in our hosted lab, and see which class surfaces would unlock the most devices (builds on P8).
- **Hosted registry**: multi-tenant index service, per-tenant OCI namespaces, usage metering (for later billing).
- **AI in the portal**: async drafting jobs (queue), review UI sorted by risk × uncertainty, bench agent via partner's own machine ("bring your own lab runner").
- **Partner-run lab runners** with signed in-toto attestations of results (north star §12).
- **Spec v0.5** proposals process (`spec/proposals/`), compatibility tests for 0.1 descriptions.
- **Security**: key ceremony, threat model update, booking external audit (human signs contract: H-0xx).
- **Legal/business basics** (human actions, Claude drafts): company registration if not done, trademark search + filing for the name, terms of service + privacy policy drafts for counsel review.

## Human actions expected

Send partner outreach; partner calls; company/trademark/legal filings; cloud accounts + billing; audit contract; Mac mini purchase; key ceremony participation (hardware keys).
