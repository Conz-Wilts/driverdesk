# Toolchain

Pinned versions on the development Mac (Apple Silicon, macOS). `cargo xtask doctor` checks that each tool is present and reports its version; update this table when a version is bumped deliberately.

| Tool | Version | Install | Purpose |
|---|---|---|---|
| Rust (rustc/cargo) | 1.99.0 | `brew install rustup`, pinned by `rust-toolchain.toml` | everything |
| targets | thumbv8m.main-none-eabihf, thumbv6m-none-eabi, wasm32-wasip2, wasm32-unknown-unknown | `rust-toolchain.toml` | Pico 2, Pico 1 fallback, Wasm logic |
| cargo-nextest | 0.9.148 | `cargo binstall cargo-nextest` | test runner |
| cargo-deny | 0.20.2 | `cargo binstall cargo-deny` | licences, advisories, bans |
| cargo-insta | 1.49.0 | `cargo binstall cargo-insta` | snapshot tests |
| probe-rs | 0.32.0 | `cargo binstall probe-rs-tools` | flash + RTT via Debug Probe |
| flip-link | latest | `cargo binstall flip-link` | stack-overflow protection for firmware |
| cargo-component | 0.21.1 | `cargo binstall cargo-component` | Wasm components (P7) |
| uv | 0.13.0 | `brew install uv` | Python (`ai/`) |
| Python | ≥ 3.12 (3.14.7 system) | uv-managed | `ai/` |
| gitleaks | 8.30.1 | `brew install gitleaks` | secret scanning |
| shellcheck | 0.11.0 | `brew install shellcheck` | lab setup scripts |
| cargo-binstall | 1.25.3 | `brew install cargo-binstall` | fast installs of the above |

PATH: `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"` (rustup from Homebrew is keg-only; added to `~/.zshrc`).
