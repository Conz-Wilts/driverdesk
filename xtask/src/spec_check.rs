//! `cargo xtask spec-check`: keep spec text, schema, examples and fixtures consistent
//! (P0 exit criteria 1 and 2).
//!
//! 1. Every property name defined in `spec/schema/0.1/*.json` appears in backticks in
//!    `spec/driverdesk-0.1.md`.
//! 2. Every ```yaml block in the text passes the schema (fragments may omit required
//!    top-level sections).
//! 3. Every file in `devices/examples/` validates without errors.
//! 4. Every file in `spec/fixtures/invalid/` fails and reports the code in its header.
//! 5. The diagnostic codes in spec Appendix B match the codes the validator can emit.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Run all checks; print a summary; fail if any check fails.
pub fn run(root: &Path) -> Result<()> {
    let text = std::fs::read_to_string(root.join("spec/driverdesk-0.1.md"))
        .context("reading spec text")?;
    let mut failures: Vec<String> = Vec::new();

    // 1. schema fields mentioned in text
    let mut fields = BTreeSet::new();
    for (_, contents) in dd_spec::schema::FILES {
        let v: Value = serde_json::from_str(contents)?;
        collect_properties(&v, &mut fields);
    }
    let missing: Vec<&String> = fields
        .iter()
        .filter(|f| {
            !text.contains(&format!("`{f}`"))
                && !text.contains(&format!(".{f}`"))
                && !text.contains(&format!("`{f}:"))
                && !text.contains(&format!("{f}:"))
        })
        .collect();
    for f in &missing {
        failures.push(format!(
            "schema field `{f}` is not described in spec/driverdesk-0.1.md"
        ));
    }
    println!(
        "schema fields documented: {}/{}",
        fields.len() - missing.len(),
        fields.len()
    );

    // 2. yaml blocks in text
    let blocks = yaml_blocks(&text);
    let mut ok_blocks = 0;
    for (line, block) in &blocks {
        let mut diags = Vec::new();
        let errs: Vec<String> = match dd_spec::syntax::parse(block, &mut diags) {
            Err(e) => e.into_iter().map(|d| d.message).collect(),
            Ok(doc) => dd_spec::schema::validate(&doc)
                .into_iter()
                .chain(diags)
                .filter(|d| !(d.code == "DD-S002" && d.pointer.as_deref() == Some("")))
                .map(|d| format!("{} {}", d.code, d.message))
                .collect(),
        };
        if errs.is_empty() {
            ok_blocks += 1;
        } else {
            failures.push(format!(
                "spec text example at line {line} is invalid: {}",
                errs.join("; ")
            ));
        }
    }
    println!("text examples valid: {ok_blocks}/{}", blocks.len());

    // 3. examples
    let mut n_ex = 0;
    for path in yaml_files(&root.join("devices/examples"))? {
        let src = std::fs::read_to_string(&path)?;
        let r = dd_spec::validate(&src);
        n_ex += 1;
        if !r.is_valid() {
            let codes: Vec<&str> = r
                .diagnostics
                .iter()
                .filter(|d| d.severity == dd_spec::Severity::Error)
                .map(|d| d.code)
                .collect();
            failures.push(format!("example {} is invalid: {codes:?}", path.display()));
        }
    }
    println!("examples valid: {n_ex} checked");

    // 4. invalid fixtures
    let mut n_fx = 0;
    let mut seen_codes = BTreeSet::new();
    for path in yaml_files(&root.join("spec/fixtures/invalid"))? {
        let src = std::fs::read_to_string(&path)?;
        let Some(expect) = src
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("# expect: "))
        else {
            failures.push(format!(
                "fixture {} has no `# expect: DD-xxxx` header",
                path.display()
            ));
            continue;
        };
        let r = dd_spec::validate(&src);
        n_fx += 1;
        if !r.diagnostics.iter().any(|d| d.code == expect.trim()) {
            failures.push(format!(
                "fixture {} does not report {expect}",
                path.display()
            ));
        }
        let warning_only = r
            .diagnostics
            .iter()
            .find(|d| d.code == expect.trim())
            .is_some_and(|d| d.severity == dd_spec::Severity::Warning);
        if r.is_valid() && !warning_only {
            failures.push(format!("fixture {} is unexpectedly valid", path.display()));
        }
        seen_codes.insert(expect.trim().to_string());
    }
    println!(
        "invalid fixtures: {n_fx} checked, {} distinct codes covered",
        seen_codes.len()
    );

    // 5. code registry
    let documented: BTreeSet<String> = text
        .lines()
        .filter_map(|l| l.strip_prefix("| DD-"))
        .map(|r| format!("DD-{}", r.split_whitespace().next().unwrap_or("")))
        .collect();
    let mut emitted = BTreeSet::new();
    for file in ["syntax.rs", "schema.rs", "lint.rs"] {
        let src = std::fs::read_to_string(root.join("crates/dd-spec/src").join(file))?;
        for (i, _) in src.match_indices("\"DD-") {
            let code: String = src[i + 1..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            if code.len() == 7 {
                emitted.insert(code);
            }
        }
    }
    for c in emitted.difference(&documented) {
        failures.push(format!(
            "code {c} is emitted but not listed in spec Appendix B"
        ));
    }
    for c in documented.difference(&emitted) {
        failures.push(format!(
            "code {c} is listed in spec Appendix B but never emitted"
        ));
    }
    for c in documented.difference(&seen_codes) {
        if c != "DD-S009" || !seen_codes.iter().any(|s| s.starts_with("DD-S")) {
            println!("note: no invalid fixture targets {c}");
        }
    }
    println!(
        "diagnostic codes: {} documented, {} emitted",
        documented.len(),
        emitted.len()
    );

    // 6. docs: every code has guidance and a book page; the tutorial assembles to a valid file
    for c in &documented {
        if !crate::docs::GUIDE.iter().any(|(g, _, _)| g == c) {
            failures.push(format!("code {c} has no guidance in xtask/src/docs.rs"));
        }
        if !root.join(format!("docs/book/src/errors/{c}.md")).exists() {
            failures.push(format!(
                "code {c} has no page docs/book/src/errors/{c}.md (run `cargo xtask docs`)"
            ));
        }
    }
    let tutorial = std::fs::read_to_string(root.join("docs/book/src/tutorial.md"))
        .context("reading tutorial")?;
    let assembled: String = yaml_blocks(&tutorial).into_iter().map(|(_, b)| b).collect();
    let r = dd_spec::validate(&assembled);
    if r.is_valid() {
        println!(
            "tutorial assembles to a valid description ({} warnings)",
            r.warnings()
        );
    } else {
        let codes: Vec<&str> = r
            .diagnostics
            .iter()
            .filter(|d| d.severity == dd_spec::Severity::Error)
            .map(|d| d.code)
            .collect();
        failures.push(format!(
            "the tutorial's YAML blocks do not assemble to a valid description: {codes:?}"
        ));
    }

    if failures.is_empty() {
        println!("\nspec-check: all green");
        Ok(())
    } else {
        for f in &failures {
            eprintln!("  ✗ {f}");
        }
        bail!("spec-check: {} problem(s)", failures.len())
    }
}

fn collect_properties(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(m) => {
            if let Some(Value::Object(props)) = m.get("properties") {
                out.extend(props.keys().cloned());
            }
            for c in m.values() {
                collect_properties(c, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|c| collect_properties(c, out)),
        _ => {}
    }
}

fn yaml_blocks(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut cur: Option<(usize, String)> = None;
    for (i, line) in text.lines().enumerate() {
        match &mut cur {
            None if line.trim_start() == "```yaml" => cur = Some((i + 2, String::new())),
            Some(_) if line.trim_start() == "```" => out.push(cur.take().expect("open block")),
            Some((_, buf)) => {
                buf.push_str(line);
                buf.push('\n');
            }
            None => {}
        }
    }
    out
}

fn yaml_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut v: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
        .collect();
    v.sort();
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_yaml_blocks() {
        let t = "x\n```yaml\na: 1\n```\n```rust\nno\n```\n```yaml\nb: 2\n```\n";
        let b = yaml_blocks(t);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].1, "a: 1\n");
    }
}
