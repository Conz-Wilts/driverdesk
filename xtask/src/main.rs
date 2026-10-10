//! Project commands: `cargo xtask <doctor|ci|gate|vlab>` (ADR-0007).

mod bench_check;
mod docs;
mod gate;
mod spec_check;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, bail};

const USAGE: &str = "\
usage: cargo xtask <command>

commands:
  doctor        check that every pinned tool is installed and print versions
  ci            fmt --check, clippy -D warnings, nextest, cargo-deny, gitleaks, ai tests
  gate <ID>     list the exit criteria of stage <ID> (scripted checks arrive per stage)
  spec-check    cross-check spec text, schemas, examples and fixtures (P0)
  docs          regenerate docs/book error pages and build the book (needs mdbook)
  bench-check   verify benchmark/chips.yaml; --online re-downloads datasheets (P0)
  vlab -- <cmd> run <cmd> inside the virtual USB lab (B.T6)";

/// Room for `dd_spec::validate` in debug builds on Windows (1 MiB main-thread stack, D-002).
const STACK_BYTES: usize = 8 * 1024 * 1024;

fn main() -> ExitCode {
    let result = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(run)
        .expect("spawn worker thread")
        .join()
        .unwrap_or_else(|_| Err(anyhow::anyhow!("xtask panicked")));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = workspace_root();
    match args.first().map(String::as_str) {
        Some("doctor") => doctor(),
        Some("ci") => ci(&root),
        Some("gate") => {
            let id = args.get(1).context("usage: cargo xtask gate <ID>")?;
            gate::print_exit_criteria(&root, id)
        }
        Some("spec-check") => spec_check::run(&root),
        Some("docs") => {
            docs::run(&root)?;
            step(&root.join("docs/book"), "mdbook", &["mdbook", "build"])
        }
        Some("bench-check") => bench_check::run(&root, args.iter().any(|a| a == "--online")),
        Some("vlab") => bail!("vlab is not implemented yet (B.T6)"),
        Some("-h" | "--help" | "help") | None => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => bail!("unknown command `{other}`\n\n{USAGE}"),
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the workspace root")
        .to_path_buf()
}

/// A tool `doctor` checks: display name, program, and the arguments that print its version.
struct Tool {
    name: &'static str,
    program: &'static str,
    version_args: &'static [&'static str],
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "rustc",
        program: "rustc",
        version_args: &["--version"],
    },
    Tool {
        name: "cargo",
        program: "cargo",
        version_args: &["--version"],
    },
    Tool {
        name: "rustfmt",
        program: "cargo",
        version_args: &["fmt", "--version"],
    },
    Tool {
        name: "clippy",
        program: "cargo",
        version_args: &["clippy", "--version"],
    },
    Tool {
        name: "cargo-nextest",
        program: "cargo",
        version_args: &["nextest", "--version"],
    },
    Tool {
        name: "cargo-deny",
        program: "cargo",
        version_args: &["deny", "--version"],
    },
    Tool {
        name: "cargo-insta",
        program: "cargo",
        version_args: &["insta", "--version"],
    },
    Tool {
        name: "cargo-component",
        program: "cargo",
        version_args: &["component", "--version"],
    },
    Tool {
        name: "probe-rs",
        program: "probe-rs",
        version_args: &["--version"],
    },
    Tool {
        name: "flip-link",
        program: "flip-link",
        version_args: &["--version"],
    },
    Tool {
        name: "uv",
        program: "uv",
        version_args: &["--version"],
    },
    Tool {
        name: "gitleaks",
        program: "gitleaks",
        version_args: &["version"],
    },
    Tool {
        name: "shellcheck",
        program: "shellcheck",
        version_args: &["--version"],
    },
];

const TARGETS: &[&str] = &[
    "thumbv8m.main-none-eabihf",
    "thumbv6m-none-eabi",
    "wasm32-wasip2",
    "wasm32-unknown-unknown",
];

fn doctor() -> Result<()> {
    let mut missing = Vec::new();
    for tool in TOOLS {
        match Command::new(tool.program).args(tool.version_args).output() {
            // flip-link has no --version flag; any successful spawn proves it is installed.
            Ok(out) if out.status.success() || tool.name == "flip-link" => {
                let text = String::from_utf8_lossy(&out.stdout);
                println!("ok       {:<16} {}", tool.name, first_version_line(&text));
            }
            _ => {
                println!("MISSING  {}", tool.name);
                missing.push(tool.name.to_string());
            }
        }
    }
    let installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    for target in TARGETS {
        if installed.lines().any(|l| l.trim() == *target) {
            println!("ok       target {target}");
        } else {
            println!("MISSING  target {target}");
            missing.push(format!("target {target}"));
        }
    }
    if missing.is_empty() {
        println!("\nall tools present (see docs/toolchain.md)");
        Ok(())
    } else {
        bail!(
            "missing: {} (install per docs/toolchain.md)",
            missing.join(", ")
        )
    }
}

fn first_version_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("(installed)")
}

fn ci(root: &Path) -> Result<()> {
    let steps: &[(&str, &[&str])] = &[
        ("fmt", &["cargo", "fmt", "--all", "--check"]),
        (
            "clippy",
            &[
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
        (
            "test",
            &["cargo", "nextest", "run", "--workspace", "--no-tests=pass"],
        ),
        ("doctest", &["cargo", "test", "--workspace", "--doc"]),
        ("deny", &["cargo", "deny", "check"]),
        (
            "gitleaks",
            &["gitleaks", "git", "--no-banner", "--redact", "."],
        ),
    ];
    for (name, cmd) in steps {
        step(root, name, cmd)?;
    }
    println!("==> spec-check");
    spec_check::run(root)?;
    let ai = root.join("ai");
    if ai.join("pyproject.toml").exists() {
        step(&ai, "ai-ruff", &["uv", "run", "ruff", "check", "."])?;
        step(
            &ai,
            "ai-ruff-format",
            &["uv", "run", "ruff", "format", "--check", "."],
        )?;
        step(&ai, "ai-pytest", &["uv", "run", "pytest", "-q"])?;
    }
    println!("\nxtask ci: all green");
    Ok(())
}

fn step(dir: &Path, name: &str, cmd: &[&str]) -> Result<()> {
    println!("==> {name}: {}", cmd.join(" "));
    let status = Command::new(cmd[0])
        .args(&cmd[1..])
        .current_dir(dir)
        .status()
        .with_context(|| format!("could not start `{}` (run `cargo xtask doctor`)", cmd[0]))?;
    if !status.success() {
        bail!("step `{name}` failed ({status})");
    }
    Ok(())
}
