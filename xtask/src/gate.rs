//! `cargo xtask gate <ID>`: read a stage plan and list its exit criteria.
//!
//! Scripted checks per criterion are added by each stage's gate task.

use std::path::Path;

use anyhow::{Context, Result, bail};

/// Print the exit criteria of stage `id` from `plans/<ID>-*.md`.
pub fn print_exit_criteria(root: &Path, id: &str) -> Result<()> {
    let plans = root.join("plans");
    let prefix = format!("{}-", id.to_ascii_uppercase());
    let entry = std::fs::read_dir(&plans)
        .with_context(|| format!("reading {}", plans.display()))?
        .filter_map(Result::ok)
        .find(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .with_context(|| format!("no plan file plans/{prefix}*.md"))?;
    let text = std::fs::read_to_string(entry.path())?;
    let criteria = exit_criteria(&text);
    if criteria.is_empty() {
        bail!(
            "{} has no `## Exit criteria` section",
            entry.path().display()
        );
    }
    println!(
        "Exit criteria for stage {id} ({}):",
        entry.file_name().to_string_lossy()
    );
    for c in &criteria {
        println!("  {c}");
    }
    println!("\nScripted checks are not wired yet; verify each criterion via /gate {id}.");
    Ok(())
}

/// Numbered items under the `## Exit criteria` heading, up to the next `## ` heading.
fn exit_criteria(plan: &str) -> Vec<&str> {
    plan.lines()
        .skip_while(|l| !l.starts_with("## Exit criteria"))
        .skip(1)
        .take_while(|l| !l.starts_with("## "))
        .map(str::trim)
        .filter(|l| {
            l.split_once(". ")
                .is_some_and(|(n, _)| n.parse::<u32>().is_ok())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::exit_criteria;

    #[test]
    fn extracts_numbered_items_in_section_only() {
        let plan = "# B\n1. not this\n## Exit criteria (the gate)\n\n1. Repo exists.\n2. CI green.\nIf hardware ...\n## Tasks\n3. not this either\n";
        assert_eq!(exit_criteria(plan), vec!["1. Repo exists.", "2. CI green."]);
    }

    #[test]
    fn empty_when_section_missing() {
        assert!(exit_criteria("# P9\n## Tasks\n1. x\n").is_empty());
    }

    #[test]
    fn real_stage_b_plan_has_six_criteria() {
        let root = super::super::workspace_root();
        let text = std::fs::read_to_string(root.join("plans/B-bootstrap.md")).unwrap();
        assert_eq!(exit_criteria(&text).len(), 6);
    }
}
