//! `cargo xtask docs`: generate one book page per diagnostic code (`docs/book/src/errors/`)
//! from spec Appendix B, the fix guidance below, and the fixture snapshots.

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, Result};

/// Why each code matters and how to fix it. Every code in spec Appendix B must have an entry
/// (checked by `cargo xtask spec-check`).
pub const GUIDE: &[(&str, &str, &str)] = &[
    (
        "DD-S001",
        "The file is not valid YAML, so nothing else can be checked.",
        "Look at the marked line: unclosed `[`/`{`, a tab used for indentation, or a missing space after `key:`. Use spaces only.",
    ),
    (
        "DD-S002",
        "A field the spec requires is missing, or a list that must have entries is empty.",
        "Add the field named in the message. The help line names the spec section that describes it.",
    ),
    (
        "DD-S003",
        "A value has the wrong type, e.g. text where a number is expected.",
        "Remove quotes around numbers (`vid: 0x1209`, not `\"0x1209\"`), drop unit suffixes (`timeout_ms: 50`), and use `[ ]` for lists.",
    ),
    (
        "DD-S004",
        "A field name is not part of the spec at this place. Usually a typo.",
        "Use the suggested name. Vendor-specific data goes in keys starting with `x-`.",
    ),
    (
        "DD-S005",
        "A value is not one of the allowed words or does not have the required form.",
        "Use one of the values listed in the help line; names are lower_snake_case, bit ranges are `\"H:L\"`.",
    ),
    (
        "DD-S006",
        "A number is outside the allowed range (for example a USB id above 0xFFFF).",
        "Check the datasheet value; the message shows the limit.",
    ),
    (
        "DD-S007",
        "The same key appears twice in a map, or a list repeats an entry. YAML silently keeps only one.",
        "Delete or rename one of them; both locations are marked.",
    ),
    (
        "DD-S008",
        "The document is too large, too deeply nested, or expands too many aliases.",
        "Descriptions are small: reference datasheets by URL, avoid deep nesting and alias chains.",
    ),
    (
        "DD-S009",
        "A construct the spec does not allow, such as a YAML tag (`!!str`).",
        "Write the value plainly; quote it if it must be text.",
    ),
    (
        "DD-L001",
        "A state can never be entered, so its behaviour is dead code.",
        "Add a transition into the state, or remove it.",
    ),
    (
        "DD-L002",
        "A transition or `requires_state` names a state that is not declared.",
        "Fix the spelling or add the state to `states.list`.",
    ),
    (
        "DD-L003",
        "A command talks to the device without a deadline; a stuck device would hang the host.",
        "Add `timeout_ms:` to the command (a little above the datasheet's worst-case time).",
    ),
    (
        "DD-L004",
        "A numeric value is sent to the device with no bounds, so any value could reach the hardware.",
        "Add `min`/`max` or `enum` to the argument, or a `safety.limits` entry with a reason.",
    ),
    (
        "DD-L005",
        "Two registers occupy the same addresses.",
        "Fix the address, `bits` or `repeat`; if the datasheet really aliases them, set `overlap: true` on both.",
    ),
    (
        "DD-L006",
        "A unit or numeric type does not fit: e.g. a temperature exposed in Pa, or integer division in a real-valued output.",
        "Expose the output that carries the right quantity; write `double(raw) / 100.0` for real numbers.",
    ),
    (
        "DD-L007",
        "`expose` or a class profile refers to something that does not exist or does not match the claimed class.",
        "Fix the reference (`command.output`), add the class to `device.classes`, or provide what the profile requires.",
    ),
    (
        "DD-L008",
        "A built-in test refers to an unknown command, output, argument or variant.",
        "Fix the name in the test so it matches `commands`.",
    ),
    (
        "DD-L009",
        "An actuator (LED, motor, heater, power) has no safety envelope.",
        "Add `safety.limits` for every argument that drives the output, each with a `reason`.",
    ),
    (
        "DD-L010",
        "The `spec:` line names a version this validator does not implement.",
        "Write `spec: driverdesk/0.1`.",
    ),
    (
        "DD-L011",
        "A USB match uses only the vendor id and would claim every product of that vendor.",
        "Add `pid`, and `interface_class` for vendor-specific interfaces.",
    ),
    (
        "DD-L012",
        "An expression is not valid CEL, or calls a function outside the DriverDesk profile.",
        "CEL has no `|`, `&`, `<<`, `>>`: use `bor`, `band`, `shl`, `shr`. See spec §10.3 for the function list.",
    ),
    (
        "DD-L013",
        "The framing names a checksum algorithm the spec does not define.",
        "Use a name from spec §7.5, e.g. `crc8-smbus`, `crc16-modbus`.",
    ),
    (
        "DD-L014",
        "A command's timeout is shorter than the waits it contains, so it would always time out.",
        "Raise `timeout_ms` above the sum of the waits.",
    ),
    (
        "DD-L015",
        "A step writes a register or field that is read-only.",
        "Check the register name, or correct `access` if the datasheet says it is writable.",
    ),
    (
        "DD-L016",
        "The safety envelope is not signed. Fine locally; the registry will not publish it for a vendor.",
        "Sign it when publishing (`dd sign`, stage P4).",
    ),
    (
        "DD-L017",
        "A safety limit has no test proving the runtime enforces it.",
        "Add a test that calls the command with an out-of-range value and `expect: rejected_by_safety`.",
    ),
    (
        "DD-L018",
        "Probing could write to an unknown device on the bus.",
        "Give a variant an `id_check` that only reads an identity register, and keep identity commands `reads_only`.",
    ),
    (
        "DD-L019",
        "A command declared `reads_only` writes to the device.",
        "Remove `reads_only: true` or the write.",
    ),
    (
        "DD-L020",
        "A step or rule refers to a register, field, fieldset or enum value that is not declared.",
        "Use the suggested name or declare it under `registers.map`.",
    ),
    (
        "DD-L021",
        "A bit field lies outside its register, is reversed, or overlaps another field.",
        "Bits are `\"high:low\"`, LSB = 0, inside the register width; each bit belongs to one field.",
    ),
    (
        "DD-L022",
        "An expression uses a name that is not defined at that point.",
        "Define it earlier with `as:` or `let:`, or declare it in `vars`, `settings` or `instance`.",
    ),
    (
        "DD-L023",
        "A provisional part of spec 0.1 is used; it may change in 0.2.",
        "No action needed for experiments; expect to migrate.",
    ),
    (
        "DD-L024",
        "A variant definition is inconsistent (unknown `exclude`, different `params`, two defaults).",
        "Make every variant define the same params and exclude only things that exist.",
    ),
    (
        "DD-L025",
        "A command that writes is retried; a retry could apply the write twice. A command is *idempotent* when running it twice leaves the device exactly as running it once (setting an LED to 50 % is; toggling it is not).",
        "Set `idempotent: true` if a repeat is harmless, otherwise remove `retries`.",
    ),
    (
        "DD-L026",
        "A unit is not a UCUM code, so conversions cannot be checked.",
        "Use the suggested UCUM code: `Cel`, `Pa`, `%`, `m/s2`, `mA`, `lx`.",
    ),
    (
        "DD-L027",
        "An unquoted `on`/`off`/`yes`/`no`/`y`/`n` means true/false to YAML 1.1 tools but text to YAML 1.2.",
        "Quote it or use another word (`enabled`, `trigger`, `unbound`).",
    ),
    (
        "DD-L028",
        "An integer with a leading zero is octal in YAML 1.1 and decimal in YAML 1.2.",
        "Remove the leading zero, or write octal as `0o17`.",
    ),
    (
        "DD-L029",
        "A reference to an unknown command, sequence, variable, event, queue, memory region, endpoint, report, type or frame field.",
        "Use the suggested name or declare the thing it refers to.",
    ),
];

/// Regenerate `docs/book/src/errors/` and its index.
pub fn run(root: &Path) -> Result<()> {
    let spec = std::fs::read_to_string(root.join("spec/driverdesk-0.1.md"))?;
    let rows: Vec<(String, String, String)> = spec
        .lines()
        .filter_map(|l| l.strip_prefix("| DD-"))
        .filter_map(|r| {
            let cells: Vec<&str> = r.split('|').map(str::trim).collect();
            (cells.len() >= 3).then(|| {
                (
                    format!("DD-{}", cells[0]),
                    cells[1].to_string(),
                    cells[2].to_string(),
                )
            })
        })
        .collect();
    let dir = root.join("docs/book/src/errors");
    std::fs::create_dir_all(&dir)?;
    let snaps = root.join("crates/dd-spec/tests/snapshots");
    let mut index = String::from(
        "# Diagnostic codes\n\nEvery message from `dd validate` has a stable code. `DD-S` codes come from the schema check, `DD-L` codes from semantic lint rules.\n\n| Code | Severity | Meaning |\n|---|---|---|\n",
    );
    for (code, sev, meaning) in &rows {
        let (_, why, fix) = GUIDE
            .iter()
            .find(|(c, _, _)| c == code)
            .with_context(|| format!("no guidance for {code} in xtask/src/docs.rs"))?;
        let _ = writeln!(index, "| [{code}]({code}.md) | {sev} | {meaning} |");
        let mut page = format!(
            "# {code}: {meaning}\n\nSeverity: **{sev}**\n\n## What it means\n\n{why}\n\n## How to fix it\n\n{fix}\n"
        );
        let mut examples = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&snaps) {
            let mut files: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
            files.sort();
            for p in files {
                let fixture = root.join("spec/fixtures/invalid").join(
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .and_then(|n| n.split('@').nth(1))
                        .map(|n| n.trim_end_matches(".snap"))
                        .unwrap_or(""),
                );
                let header = std::fs::read_to_string(&fixture).unwrap_or_default();
                if header.lines().next() == Some(&format!("# expect: {code}")) {
                    let snap = std::fs::read_to_string(&p)?;
                    let body = snap.splitn(3, "---\n").nth(2).unwrap_or("").to_string();
                    let first: String = body
                        .split(&format!("{code} ("))
                        .nth(1)
                        .map(|rest| {
                            format!("{code} ({}", rest.split("\nDD-").next().unwrap_or(rest))
                        })
                        .unwrap_or(body);
                    examples.push((
                        fixture
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        header
                            .lines()
                            .nth(1)
                            .unwrap_or("")
                            .trim_start_matches("# ")
                            .to_string(),
                        first,
                    ));
                }
            }
        }
        for (name, desc, out) in examples {
            let _ = write!(
                page,
                "\n## Example: {desc}\n\nFixture `spec/fixtures/invalid/{name}`:\n\n```text\n{}\n```\n",
                out.trim_end()
            );
        }
        std::fs::write(dir.join(format!("{code}.md")), page)?;
    }
    std::fs::write(dir.join("README.md"), index)?;
    println!(
        "docs: wrote {} error pages to docs/book/src/errors/",
        rows.len()
    );
    Ok(())
}
