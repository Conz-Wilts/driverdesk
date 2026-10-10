# P4 — Private, signed and automatic

- Status: not started
- Depends on: P1 gate (P2 for the macOS/Windows agent)
- Branch: `stage/p4`
- Estimate: 3–4 weeks

Claim: a description can be published privately and signed, and on a fresh machine the agent detects the device, fetches the driver, verifies it, and runs it automatically. Tampering, rollback and revoked drivers are rejected.

## Exit criteria (the gate)

1. `docker compose up` in `deploy/registry-node/` starts a complete registry node (index service + OCI registry + Postgres) on a laptop in < 60 s; health endpoint green.
2. `dd publish devices/examples/pico-thermo.yaml --visibility private` signs (Sigstore/cosign-compatible), uploads as an OCI artifact, records in the transparency log (Rekor; public staging instance during development), and updates TUF metadata.
3. **Plug-and-play**: on a fresh vlab VM and on `lab-linux` with a fresh user, with only `dd-agent` installed and pointed at the node, plugging in the device (power-cycle via uhubctl / gadget attach) results in a running driver and a successful `dd read temp0` within **5 s**, no commands typed. *Evidence:* timed HIL run.
4. **Security suite** (automated, all must reject with a clear error): flipped byte in artifact; signature from an unknown key; valid signature but missing transparency-log entry; TUF metadata rollback (older version); expired metadata (freeze-attack window); revoked description; description for a different match key; private artifact fetched by an unauthorised client. ≥ 12 tests.
5. **Revocation** propagates: revoke on the node → running agent stops the driver within the configured refresh interval (default 5 min; test with 10 s).
6. **Offline bundle**: `dd bundle create` + `dd bundle import` on an air-gapped vlab VM; works with no network; bundle expiry enforced.
7. **Privacy**: agent lookups contain only the match key and the TUF snapshot version — no user/machine identifiers. Asserted by a test that inspects requests.
8. Threat model `docs/security/threat-model.md` (STRIDE per component) and `security-review` skill run over the stage diff; all high findings fixed.

## Pre-made decisions

- Distribution = OCI artifacts (ADR-0005). Media types: `application/vnd.driverdesk.description.v0+yaml`, `application/vnd.driverdesk.logic.v0+wasm`; manifest annotations carry the match keys.
- Index service: Rust + `axum` + Postgres (`sqlx`). OCI registry: **Zot** in the compose file (any OCI registry must work; CI also tests against `registry:2`).
- Signing: Sigstore. Vendors: key-based signing (cosign key pair or KMS). Project CI: keyless (GitHub OIDC). Verification policy per registry node: trusted identities/keys list.
- Freshness/revocation: TUF (`tough` crate on the client) with targets = description digests; short-lived `timestamp` role.
- Private content: envelope encryption with `age` (X25519) to recipient keys of authorised clients; the node stores only ciphertext. Vendor KMS integration is R1.
- Agent runs drivers as separate unprivileged processes (one per device) and supervises them.

## Research questions

- **R-P4-1** Rust OCI client (`oci-client` / `oci-distribution`) — push/pull artifacts with custom media types, referrers API for signatures. · Default: `oci-client`. · Status: open
- **R-P4-2** `sigstore-rs` maturity for sign + verify (bundle format v0.3, Rekor v2?) vs shelling out to `cosign`. · Default: verify in-process with `sigstore-rs`; sign via `sigstore-rs` if it supports our flow, else `cosign` CLI in `dd publish` for P4 only (ADR if so). · Status: open
- **R-P4-3** `tough` for TUF client and repository editing; how other projects (e.g. Bottlerocket) lay out targets. · Default: `tough` for both. · Status: open
- **R-P4-4** nusb hotplug (`watch_devices`) on all three OSes vs udev/IOKit/CM_Register_Notification. · Default: nusb hotplug. · Status: open
- **R-P4-5** Running drivers as unprivileged processes on Linux with access to exactly one USB device (fd passing from agent vs udev permissions). · Default: agent opens the device and passes the fd to the child over a Unix socket (`SCM_RIGHTS`); child has no other device access (landlock + seccomp in P7 hardening, documented now). · Status: open

## Tasks

- [ ] **P4.T0 Re-validate this plan.**
- [ ] **P4.T1 Artifact format** — spec appendix "Distribution" + Rust types + tests (round trip push/pull against Zot in CI service container).
- [ ] **P4.T2 Index service** — publish, lookup by match key (`usb:v1209p0001` with specificity ranking), list versions, visibility, revocation list, authN (API tokens for publishers; client tokens for private content), rate limits. OpenAPI spec generated.
- [ ] **P4.T3 Compose + deployment** — `deploy/registry-node/` with healthchecks, volumes, backups doc. Exit criterion 1.
- [ ] **P4.T4 Signing + transparency** (R-P4-2) — `dd publish`, `dd verify`; transparency entries stored with the artifact; verification policy file.
- [ ] **P4.T5 TUF metadata** (R-P4-3) — repository editing on publish/revoke; key roles documented; root keys for dev generated offline and stored outside git (human action H-007 for the production root later in R1).
- [ ] **P4.T6 Private encryption** — age envelope, client key enrollment, access checks. Exit criterion 4 (private part).
- [ ] **P4.T7 `dd-agent`** (R-P4-4, R-P4-5) — hotplug → lookup → fetch → verify (signature, log inclusion, TUF) → cache → spawn per-device runtime process → supervise (restart with backoff) → refresh/revoke loop. Systemd unit; launchd plist and Windows service stubs (completed in P5/R2). Local API socket for `dd read` to reach running drivers.
- [ ] **P4.T8 Offline bundles** — exit criterion 6.
- [ ] **P4.T9 Security suite + privacy test** — exit criteria 4, 5, 7.
- [ ] **P4.T10 Plug-and-play HIL** — exit criterion 3 on vlab and `lab-linux`.
- [ ] **P4.T11 Threat model + security review + gate** — exit criterion 8, `/gate P4`.

## Risks and debugging hints

- **Clock skew** breaks TUF expiry and Sigstore certificate validity: lab nodes run NTP; tests use an injected clock.
- **Rekor availability** in CI: cache verification material; mark tests that need the public staging instance and run them nightly, use a local Rekor container for PR CI if available.
- Keep the agent's verification code small and boring; it is the most security-critical code in the project.

## Human actions

None required for development (uses public Sigstore staging). Production trust roots are R1 (H-007).

## Gate procedure

`/gate P4`, report `docs/reports/P4.md`, merge, tag `gate/P4`.
