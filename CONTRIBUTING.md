# Contributing to DriverDesk

Thanks for helping. A few rules keep the project healthy.

## Developer Certificate of Origin (DCO)

We do not use a CLA. Every commit must carry a `Signed-off-by` line certifying the [Developer Certificate of Origin 1.1](https://developercertificate.org/):

```
git commit -s -m "feat(runtime): add register cache (P1.T3)"
```

The name and email must be yours. Commits without a sign-off cannot be merged.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/): `type(scope): summary`, with the task id in parentheses when there is one.

- Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.
- Scopes: crate or area name without the `dd-` prefix (`spec`, `runtime`, `agent`, `registry`, `lab`, `ai`, …).

## Before opening a pull request

- `cargo xtask ci` passes (formatting, clippy with `-D warnings`, tests, `cargo-deny`, `gitleaks`).
- Python changes under `ai/`: `uv run ruff check && uv run pytest`.
- New dependencies have a permissive licence (Apache-2.0, MIT, BSD, ISC, Zlib, MPL-2.0). No GPL in runtime crates.
- Never commit secrets, datasheet PDFs or other third-party copyrighted documents (store a URL and SHA-256 instead).
- No `TODO` without a task id, e.g. `TODO(P2.T4)`.

## Safety

Never write to a device outside its description's safety envelope and never disable safety enforcement, including in tests. Lab hardware is USB-powered and low-energy only.

## Code of Conduct

Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).
