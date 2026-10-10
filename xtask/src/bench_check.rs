//! `cargo xtask bench-check [--online]`: verify `benchmark/chips.yaml` (P0 exit criterion 6).
//!
//! Offline: 50 chips, required fields, 35 dev / 15 heldout, unique parts, SHA-256 of any
//! datasheet already cached in `cache/datasheets/` matches. `--online` also downloads every
//! datasheet (HTTP 200, `%PDF` magic) and checks its hash.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

const REQUIRED: &[&str] = &[
    "/part",
    "/vendor",
    "/class",
    "/kind",
    "/bus",
    "/datasheet/url",
    "/datasheet/sha256",
    "/linux/path",
    "/split",
    "/licence",
];

/// Run the checks.
pub fn run(root: &Path, online: bool) -> Result<()> {
    let path = root.join("benchmark/chips.yaml");
    let src =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut diags = Vec::new();
    let doc = dd_spec::syntax::parse(&src, &mut diags)
        .map_err(|e| anyhow::anyhow!("chips.yaml: {}", e[0].message))?;
    let chips = doc
        .json
        .get("chips")
        .and_then(Value::as_array)
        .context("chips.yaml has no `chips` list")?;
    let mut failures = Vec::new();
    if chips.len() != 50 {
        failures.push(format!("expected 50 chips, found {}", chips.len()));
    }
    let (mut dev, mut held) = (0, 0);
    let mut parts = BTreeSet::new();
    let cache = root.join("cache/datasheets");
    let (mut cached, mut verified) = (0, 0);
    for (i, c) in chips.iter().enumerate() {
        let part = c.get("part").and_then(Value::as_str).unwrap_or("?");
        for r in REQUIRED {
            if c.pointer(r).is_none() {
                failures.push(format!("chip #{i} ({part}): missing {r}"));
            }
        }
        if !parts.insert(part.to_string()) {
            failures.push(format!("duplicate part {part}"));
        }
        match c.get("split").and_then(Value::as_str) {
            Some("dev") => dev += 1,
            Some("heldout") => held += 1,
            other => failures.push(format!(
                "{part}: split must be dev or heldout, got {other:?}"
            )),
        }
        let sha = c
            .pointer("/datasheet/sha256")
            .and_then(Value::as_str)
            .unwrap_or("");
        if sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
            failures.push(format!("{part}: datasheet sha256 is not 64 hex characters"));
        }
        let file = cache.join(format!("{part}.pdf"));
        if file.exists() {
            cached += 1;
            if sha256(&file)? != sha {
                failures.push(format!(
                    "{part}: cached datasheet hash differs from chips.yaml"
                ));
            }
        }
        if online {
            let url = c
                .pointer("/datasheet/url")
                .and_then(Value::as_str)
                .unwrap_or("");
            let tmp = std::env::temp_dir().join(format!("dd-bench-{part}.pdf"));
            let status = Command::new("curl")
                .args(["-sSL", "-A", "Mozilla/5.0", "--max-time", "90", "-o"])
                .arg(&tmp)
                .args(["-w", "%{http_code}", url])
                .output()?;
            let code = String::from_utf8_lossy(&status.stdout).to_string();
            let magic = std::fs::read(&tmp)
                .map(|b| b.starts_with(b"%PDF"))
                .unwrap_or(false);
            if code != "200" || !magic {
                failures.push(format!("{part}: {url} → HTTP {code}, pdf={magic}"));
            } else if sha256(&tmp)? != sha {
                failures.push(format!(
                    "{part}: downloaded datasheet hash differs (vendor may have revised it)"
                ));
            } else {
                verified += 1;
            }
            let _ = std::fs::remove_file(&tmp);
        }
    }
    if dev != 35 || held != 15 {
        failures.push(format!(
            "split must be 35 dev / 15 heldout, found {dev} / {held}"
        ));
    }
    println!(
        "chips: {}, dev {dev}, heldout {held}, cached datasheets checked {cached}{}",
        chips.len(),
        if online {
            format!(", downloaded and verified {verified}")
        } else {
            String::new()
        }
    );
    if failures.is_empty() {
        println!("bench-check: all green");
        Ok(())
    } else {
        for f in &failures {
            eprintln!("  ✗ {f}");
        }
        bail!("bench-check: {} problem(s)", failures.len())
    }
}

fn sha256(p: &Path) -> Result<String> {
    let out = Command::new("shasum")
        .args(["-a", "256"])
        .arg(p)
        .output()
        .or_else(|_| Command::new("sha256sum").arg(p).output())?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string())
}
