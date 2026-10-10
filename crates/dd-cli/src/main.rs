//! The `dd` command-line tool.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

/// DriverDesk: device drivers as signed, portable data.
#[derive(Parser)]
#[command(name = "dd", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check device descriptions against the driverdesk/0.1 spec.
    ///
    /// Exit status: 0 = valid (warnings allowed), 1 = at least one description is invalid,
    /// 2 = usage or file error.
    Validate {
        /// Description files (YAML).
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Human)]
        format: Format,
        /// Treat warnings as errors.
        #[arg(long)]
        deny_warnings: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    /// Annotated source snippets.
    Human,
    /// One JSON object per run: `{ "files": [ { "file", "valid", "diagnostics": [...] } ] }`.
    Json,
}

/// Validation needs about 1 MiB of stack in debug builds; Windows gives the main thread only
/// 1 MiB, so work runs on a thread with the Linux/macOS default of 8 MiB (D-002).
const STACK_BYTES: usize = 8 * 1024 * 1024;

fn main() -> ExitCode {
    let cli = Cli::parse();
    std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || match cli.command {
            Command::Validate {
                files,
                format,
                deny_warnings,
            } => validate(&files, format, deny_warnings),
        })
        .expect("spawn worker thread")
        .join()
        .unwrap_or(ExitCode::from(2))
}

fn validate(files: &[PathBuf], format: Format, deny_warnings: bool) -> ExitCode {
    let mut any_invalid = false;
    let mut json_files = Vec::new();
    for path in files {
        let name = path.display().to_string();
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("dd: cannot read {name}: {e}");
                return ExitCode::from(2);
            }
        };
        let report = dd_spec::validate(&src);
        let valid = report.is_valid() && !(deny_warnings && report.warnings() > 0);
        any_invalid |= !valid;
        match format {
            Format::Human => {
                let handler = if std::io::stderr().is_terminal() {
                    miette::GraphicalReportHandler::new()
                } else {
                    miette::GraphicalReportHandler::new_themed(
                        miette::GraphicalTheme::unicode_nocolor(),
                    )
                    .with_links(false)
                };
                for d in &report.diagnostics {
                    let mut out = String::new();
                    let _ = handler
                        .render_report(&mut out, &dd_spec::diag::Rendered::new(d, &name, &src));
                    eprint!("{out}");
                }
                let status = if valid { "ok" } else { "INVALID" };
                eprintln!(
                    "{name}: {status} ({} errors, {} warnings)",
                    report.errors(),
                    report.warnings()
                );
            }
            Format::Json => {
                json_files.push(serde_json::json!({ "file": name, "valid": valid, "diagnostics": report.diagnostics }));
            }
        }
    }
    if let Format::Json = format {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({ "spec": dd_spec::lint::SPEC, "files": json_files })
            )
            .expect("serialisable")
        );
    }
    if any_invalid {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
