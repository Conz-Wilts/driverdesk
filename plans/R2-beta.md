# R2 — Public beta

- Status: not started · Depends on: R1 gate · Branch: `stage/r2` · Horizon: months 13–20
- **R2.T0 is mandatory: expand this plan to P1-level detail before building.**

Claim: anyone can publish, vendors pay, and a Linux distribution ships the agent.

## Exit criteria (draft)

1. Public registry open: free public descriptions; moderation and takedown process; abuse handling.
2. Paid tiers live (Pro, Scale; Enterprise by contract) with usage-based billing (Stripe); prices tested with ≥ 10 paying vendors.
3. Verified badge + certification programme v1 (lab-tested per OS, signed attestation, public page per device).
4. PCI support via VFIO on Linux with IOMMU required; one PCI demo device class (e.g. a simple capture or GPIO card) end to end.
5. Agent packaged: deb, rpm, AUR, Nix, Flatpak-compatible service where relevant, Homebrew, winget; signed packages; reproducible builds verified by a second builder.
6. ≥ 1 smaller Linux distribution ships the agent by default (human-assisted partnership).
7. Spec **v1.0 frozen** with a written compatibility promise and a conformance test suite third parties can run.
8. External security audit completed and findings fixed; public bug bounty launched.
9. 50+ verified devices.

## Workstreams

Public registry + moderation; billing + metering; certification programme; VFIO transport + IOMMU checks; packaging + reproducible builds; distro partnership (Claude prepares packaging PRs and outreach; human sends); spec 1.0 freeze process; audit + bounty; status page + public changelog; opt-in anonymous crash/failure reporting (privacy-reviewed); AI first-line support bot reading harness reports.

## Human actions expected

Stripe account, pricing approval is **not** needed (Claude decides prices via DECIDING with experiment data) but the account setup is; distro and audit contracts; bounty platform account; EV code-signing certificate + Microsoft Partner Center attestation; Apple Developer ID notarisation setup.
