//! Semantic lint rules `DD-Lxxx` (spec Appendix B).
//!
//! Lints read the JSON form of the document defensively (every lookup is optional), so they
//! still give useful results when the schema check failed.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use serde_json::{Map, Value};

use crate::diag::{Diagnostic, Span, suggest};
use crate::expr::{self, ExprError};
use crate::syntax::{Document, NodeKind, ScalarValue, escape_token, walk};
use crate::units;

/// The spec version this validator implements.
pub const SPEC: &str = "driverdesk/0.1";

/// Top-level sections marked provisional (spec §3).
const PROVISIONAL_SECTIONS: &[&str] = &[
    "memory",
    "queues",
    "interrupts",
    "streams",
    "firmware",
    "logic",
    "hints",
];
const PROVISIONAL_TRANSPORTS: &[&str] = &["spi", "pci"];
const STABLE_CLASSES: &[&str] = &[
    "sensor.temperature",
    "sensor.humidity",
    "sensor.pressure",
    "sensor.generic",
    "light.indicator",
    "bridge.i2c",
    "bridge.spi",
    "bridge.uart",
    "bridge.gpio",
];
const KNOWN_CLASSES: &[&str] = &[
    "sensor.temperature",
    "sensor.humidity",
    "sensor.pressure",
    "sensor.generic",
    "light.indicator",
    "bridge.i2c",
    "bridge.spi",
    "bridge.uart",
    "bridge.gpio",
    "input.keyboard",
    "input.pointer",
    "input.gamepad",
    "serial.uart",
    "net.ethernet",
    "storage.block",
    "camera.video",
    "audio.pcm",
    "display.basic",
    "gpu.compute",
];
const ACTUATOR_PREFIXES: &[&str] = &["light.", "motor.", "power.", "heater.", "actuator."];
/// Step operations that do not touch the device.
const LOCAL_OPS: &[&str] = &[
    "let", "store", "check", "wait_us", "wait_ms", "call", "logic", "dma_addr",
];
/// Step operations that write to the device.
const WRITE_OPS: &[&str] = &[
    "write",
    "update",
    "transfer",
    "hid_set",
    "hid_write",
    "bus",
    "enqueue",
    "write_memory",
];
/// Keys whose string values are always CEL.
const CEL_KEYS: &[&str] = &["cel", "until", "check", "match", "data", "when"];
/// Keys whose string values are CEL when the schema allows `int_or_cel`.
const INT_OR_CEL_KEYS: &[&str] = &[
    "value",
    "length",
    "wait_us",
    "wait_ms",
    "index",
    "address",
    "size",
    "depth",
    "rate",
    "startup_us",
    "reset_us",
    "conversion_us",
];
const YAML11_BOOLS: &[&str] = &[
    "y", "Y", "yes", "Yes", "YES", "n", "N", "no", "No", "NO", "on", "On", "ON", "off", "Off",
    "OFF",
];

/// Run all lint rules. `schema_ok` tells whether the schema check passed (some rules only
/// make sense on a well-formed document).
pub fn run(doc: &Document, schema_ok: bool) -> Vec<Diagnostic> {
    let mut l = Linter::new(doc);
    l.yaml11(); // L027, L028 — purely syntactic
    l.spec_version(); // L010
    if !doc.json.is_object() {
        return l.out;
    }
    l.units(); // L026
    l.cel_compiles(); // L012
    l.provisional(); // L023
    l.registers_overlap(); // L005
    l.register_fields(); // L021
    l.commands(); // L003, L004, L014, L015, L019, L020, L022, L025, L029
    l.sequences(); // L015, L020, L022, L029
    l.states(); // L001, L002, L029
    l.expose(); // L006, L007
    l.tests(); // L008, L017
    l.safety(); // L009, L016, L020
    l.matching(); // L011, L018
    l.variants(); // L020, L024
    l.framing(); // L013
    l.references(); // L029 (provides, settings.apply, events, queues, firmware, logic)
    l.expressions_outside_commands(); // L022 in timing, transport, events, tests
    let _ = schema_ok;
    l.out
}

struct Linter<'a> {
    doc: &'a Document,
    j: &'a Value,
    out: Vec<Diagnostic>,
    regs: BTreeMap<String, Reg>,
    commands: BTreeMap<String, &'a Value>,
    sequences: BTreeSet<String>,
    vars: BTreeSet<String>,
    settings: BTreeSet<String>,
    instance: BTreeSet<String>,
    variant_params: BTreeSet<String>,
    events: BTreeSet<String>,
    states: BTreeSet<String>,
    types: BTreeMap<String, Vec<String>>,
    memory: BTreeSet<String>,
    queues: BTreeMap<String, &'a Value>,
}

#[derive(Debug, Clone)]
struct Reg {
    access: String,
    /// Field name → (hi, lo, access override).
    fields: BTreeMap<String, (u32, u32, Option<String>)>,
}

fn obj(v: &Value) -> Option<&Map<String, Value>> {
    v.as_object()
}

fn keys(v: Option<&Value>) -> BTreeSet<String> {
    v.and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

fn ptr(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|p| format!("/{}", escape_token(p)))
        .collect()
}

/// Parse `"H:L"` or `"N"` into (hi, lo).
pub fn parse_bits(s: &str) -> Option<(u32, u32)> {
    match s.split_once(':') {
        Some((h, l)) => Some((h.trim().parse().ok()?, l.trim().parse().ok()?)),
        None => {
            let n = s.trim().parse().ok()?;
            Some((n, n))
        }
    }
}

/// The single operation key of a step object.
fn step_op(step: &Value) -> Option<&str> {
    const OPS: &[&str] = &[
        "read",
        "read_block",
        "write",
        "update",
        "wait_us",
        "wait_ms",
        "poll",
        "call",
        "let",
        "store",
        "check",
        "send",
        "receive",
        "control",
        "transfer",
        "hid_get",
        "hid_set",
        "hid_read",
        "hid_write",
        "bus",
        "logic",
        "dma_addr",
        "enqueue",
        "await",
        "read_memory",
        "write_memory",
    ];
    obj(step)?
        .keys()
        .find(|k| OPS.contains(&k.as_str()))
        .map(String::as_str)
}

fn step_writes(step: &Value) -> bool {
    match step_op(step) {
        Some("control") => {
            step.pointer("/control/direction").and_then(Value::as_str) == Some("out")
        }
        Some("send") => true,
        Some(op) => WRITE_OPS.contains(&op),
        None => false,
    }
}

fn step_touches_device(step: &Value) -> bool {
    step_op(step).is_some_and(|op| !LOCAL_OPS.contains(&op))
}

fn literal_wait_ms(step: &Value) -> f64 {
    match step_op(step) {
        Some("wait_us") => step["wait_us"].as_f64().unwrap_or(0.0) / 1000.0,
        Some("wait_ms") => step["wait_ms"].as_f64().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Names declared by a payload struct (`[{name: type}, …]` or `{name: type}`).
fn struct_names(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(obj)
            .flat_map(|m| m.keys().cloned())
            .collect(),
        Some(Value::Object(m)) => m.keys().cloned().collect(),
        _ => Vec::new(),
    }
}

impl<'a> Linter<'a> {
    fn new(doc: &'a Document) -> Self {
        let j = &doc.json;
        let mut l = Linter {
            doc,
            j,
            out: Vec::new(),
            regs: BTreeMap::new(),
            commands: BTreeMap::new(),
            sequences: keys(j.get("sequences")),
            vars: keys(j.get("vars")),
            settings: keys(j.get("settings")),
            instance: keys(j.get("instance")),
            variant_params: BTreeSet::new(),
            events: keys(j.get("events")),
            states: BTreeSet::new(),
            types: BTreeMap::new(),
            memory: keys(j.get("memory")),
            queues: BTreeMap::new(),
        };
        if let Some(m) = j.get("commands").and_then(obj) {
            l.commands = m.iter().map(|(k, v)| (k.clone(), v)).collect();
        }
        if let Some(m) = j.get("queues").and_then(obj) {
            l.queues = m.iter().map(|(k, v)| (k.clone(), v)).collect();
        }
        if let Some(m) = j.get("types").and_then(obj) {
            l.types = m
                .iter()
                .map(|(k, v)| (k.clone(), struct_names(Some(v))))
                .collect();
        }
        if let Some(vs) = j.get("variants").and_then(obj) {
            for v in vs.values() {
                l.variant_params.extend(keys(v.get("params")));
            }
        }
        l.index_registers();
        l
    }

    fn span(&self, p: &str) -> Option<Span> {
        self.doc.spans.value_or_ancestor(p)
    }

    fn key(&self, p: &str) -> Option<Span> {
        self.doc.spans.key_or_value(p)
    }

    fn push(&mut self, d: Diagnostic) {
        self.out.push(d);
    }

    fn index_registers(&mut self) {
        let Some(r) = self.j.get("registers") else {
            return;
        };
        let fieldsets: HashMap<String, &Value> = r
            .get("fieldsets")
            .and_then(obj)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v)).collect())
            .unwrap_or_default();
        let Some(map) = r.get("map").and_then(obj) else {
            return;
        };
        for (name, reg) in map {
            let access = reg
                .get("access")
                .and_then(Value::as_str)
                .unwrap_or("rw")
                .to_string();
            let fsrc = reg.get("fields").or_else(|| {
                reg.get("fieldset")
                    .and_then(Value::as_str)
                    .and_then(|f| fieldsets.get(f).copied())
            });
            let mut fields = BTreeMap::new();
            if let Some(fm) = fsrc.and_then(obj) {
                for (fname, f) in fm {
                    // Unparsable bits are reported by the schema; still index the name.
                    let (h, lo) = f
                        .get("bits")
                        .and_then(Value::as_str)
                        .and_then(parse_bits)
                        .unwrap_or((0, 0));
                    fields.insert(
                        fname.clone(),
                        (
                            h,
                            lo,
                            f.get("access").and_then(Value::as_str).map(str::to_string),
                        ),
                    );
                }
            }
            let entry = Reg { access, fields };
            if let Some(count) = reg.pointer("/repeat/count").and_then(Value::as_u64) {
                for i in 0..count {
                    self.regs.insert(format!("{name}{i}"), entry.clone());
                }
            }
            self.regs.insert(name.clone(), entry);
        }
    }

    // ---------------------------------------------------------------- L027, L028
    fn yaml11(&mut self) {
        let mut found = Vec::new();
        walk(&self.doc.root, "", None, &mut |_, node, key| {
            if let Some(k) = key
                && let NodeKind::Scalar(s) = &k.kind
                && s.plain
                && YAML11_BOOLS.contains(&s.text.as_str())
            {
                found.push(("DD-L027", k.span, s.text.clone(), true));
            }
            if let NodeKind::Scalar(s) = &node.kind
                && s.plain
            {
                if YAML11_BOOLS.contains(&s.text.as_str()) {
                    found.push(("DD-L027", node.span, s.text.clone(), false));
                } else if is_leading_zero(&s.text) && matches!(s.value, ScalarValue::Int(_)) {
                    found.push(("DD-L028", node.span, s.text.clone(), false));
                }
            }
        });
        for (code, span, text, is_key) in found {
            if code == "DD-L027" {
                let what = if is_key { "key" } else { "value" };
                let alt = match text.to_lowercase().as_str() {
                    "on" => "`enabled`, `trigger` or `\"on\"`",
                    "off" => "`disabled`, `unbound` or `\"off\"`",
                    "yes" | "y" => "`true` or `\"yes\"`",
                    _ => "`false` or `\"no\"`",
                };
                self.push(
                    Diagnostic::error(
                        "DD-L027",
                        format!("unquoted `{text}` as a {what} means true/false to YAML 1.1 tools"),
                    )
                    .at(span, "ambiguous between YAML versions")
                    .with_help(format!(
                        "use {alt} so every YAML parser reads the same thing (spec §3)"
                    )),
                );
            } else {
                let digits = text.trim_start_matches(['+', '-']).trim_start_matches('0');
                self.push(
                    Diagnostic::error(
                        "DD-L028",
                        format!(
                            "`{text}` has a leading zero: octal in YAML 1.1, decimal in YAML 1.2"
                        ),
                    )
                    .at(span, "ambiguous number")
                    .with_help(format!(
                        "write `{}` for decimal or `0o{}` for octal",
                        if digits.is_empty() { "0" } else { digits },
                        if digits.is_empty() { "0" } else { digits }
                    )),
                );
            }
        }
    }

    // ---------------------------------------------------------------- L010
    fn spec_version(&mut self) {
        let Some(v) = self.j.get("spec") else { return };
        let got = v.as_str().map_or_else(|| v.to_string(), str::to_string);
        if got == SPEC {
            return;
        }
        let help = if got.trim() == "0.1"
            || got == "driverdesk/0.1.0"
            || got.eq_ignore_ascii_case("DriverDesk/0.1")
            || got == "driverdesk-0.1"
        {
            format!("write `spec: {SPEC}`")
        } else if got.starts_with("driverdesk/") {
            format!(
                "this validator implements `{SPEC}`; update the validator or migrate the description to `{SPEC}`"
            )
        } else {
            format!("the first line of a description is `spec: {SPEC}`")
        };
        self.push(
            Diagnostic::error("DD-L010", format!("unknown spec version `{got}`"))
                .at_opt(self.span("/spec"), "not a known version")
                .with_help(help)
                .with_pointer("/spec"),
        );
    }

    // ---------------------------------------------------------------- L026
    fn units(&mut self) {
        let mut bad = Vec::new();
        visit_json(self.j, "", &mut |p, k, v| {
            if k == Some("unit")
                && let Some(u) = v.as_str()
                && units::parse(u).is_none()
            {
                bad.push((p.to_string(), u.to_string()));
            }
        });
        for (p, u) in bad {
            let codes = units::codes();
            let fixes: HashMap<&str, &str> = [
                ("degC", "Cel"),
                ("°C", "Cel"),
                ("C", "Cel"),
                ("celsius", "Cel"),
                ("degF", "[degF]"),
                ("percent", "%"),
                ("RH", "%"),
                ("%RH", "%"),
                ("lux", "lx"),
                ("ohm", "Ohm"),
                ("psi", "[psi]"),
                ("ppm", "[ppm]"),
                ("sec", "s"),
            ]
            .into_iter()
            .collect();
            let help = match fixes
                .get(u.as_str())
                .copied()
                .or_else(|| suggest(&u, codes.iter().copied()))
            {
                Some(s) => {
                    format!("did you mean `{s}`? Units are case-sensitive UCUM codes (spec §11)")
                }
                None => {
                    "use a UCUM code such as `Cel`, `Pa`, `%`, `m/s2`, `mA` (spec §11)".to_string()
                }
            };
            self.push(
                Diagnostic::error("DD-L026", format!("unknown unit `{u}`"))
                    .at_opt(self.span(&p), "not a UCUM code")
                    .with_help(help)
                    .with_pointer(p),
            );
        }
    }

    // ---------------------------------------------------------------- L012
    fn cel_compiles(&mut self) {
        let mut found = Vec::new();
        visit_json(self.j, "", &mut |p, k, v| {
            let Some(k) = k else { return };
            let Some(s) = v.as_str() else { return };
            if in_struct(p) {
                return;
            }
            if CEL_KEYS.contains(&k) || INT_OR_CEL_KEYS.contains(&k) {
                // `match` in the top-level list is the match section, not CEL.
                if k == "match" && p == "/match" {
                    return;
                }
                if k == "data"
                    && !p.contains("/steps/")
                    && !p.contains("/sequences/")
                    && !p.contains("/transitions/")
                    && !p.contains("/before/")
                {
                    return;
                }
                if k == "rate" && p.starts_with("/safety/") {
                    return;
                }
                found.push((p.to_string(), s.to_string()));
            }
        });
        for (p, s) in found {
            if let Err(e) = expr::check(&s) {
                let (msg, help) = match e {
                    ExprError::Syntax(m) => ("expression does not compile".to_string(), m),
                    ExprError::UnknownFunction(f) => {
                        let all: Vec<&str> = expr::LIBRARY
                            .iter()
                            .chain(expr::STANDARD)
                            .copied()
                            .collect();
                        let h = suggest(&f, all.iter().copied()).map_or_else(
                            || "available functions are listed in spec §10.3".to_string(),
                            |s| format!("did you mean `{s}`? (spec §10.3)"),
                        );
                        (format!("unknown function `{f}` in expression"), h)
                    }
                };
                self.push(
                    Diagnostic::error("DD-L012", msg)
                        .at_opt(self.span(&p), "CEL")
                        .with_help(help)
                        .with_pointer(p),
                );
            }
        }
    }

    // ---------------------------------------------------------------- L023
    fn provisional(&mut self) {
        let mut used: Vec<(String, String)> = Vec::new();
        for s in PROVISIONAL_SECTIONS {
            if self.j.get(*s).is_some() {
                used.push((format!("/{s}"), format!("`{s}`")));
            }
        }
        for t in PROVISIONAL_TRANSPORTS {
            if self.j.pointer(&format!("/transport/{t}")).is_some() {
                used.push((format!("/transport/{t}"), format!("transport `{t}`")));
            }
        }
        if let Some(classes) = self.j.pointer("/device/classes").and_then(Value::as_array) {
            for (i, c) in classes.iter().enumerate() {
                if let Some(c) = c.as_str() {
                    if !KNOWN_CLASSES.contains(&c) {
                        let help = suggest(c, KNOWN_CLASSES.iter().copied()).map_or_else(
                            || format!("known classes: {}", KNOWN_CLASSES.join(", ")),
                            |s| format!("did you mean `{s}`?"),
                        );
                        self.push(
                            Diagnostic::error("DD-L007", format!("unknown device class `{c}`"))
                                .at_opt(self.span(&format!("/device/classes/{i}")), "unknown class")
                                .with_help(help)
                                .with_pointer(format!("/device/classes/{i}")),
                        );
                    } else if !STABLE_CLASSES.contains(&c) {
                        used.push((format!("/device/classes/{i}"), format!("class `{c}`")));
                    }
                }
            }
        }
        for (p, what) in used {
            self.push(
                Diagnostic::warning(
                    "DD-L023",
                    format!("{what} is provisional in spec 0.1 and may change"),
                )
                .at_opt(self.key(&p), "provisional")
                .with_help(
                    "fine for experiments; expect to migrate when 0.2 is published (spec §3, §20)",
                )
                .with_pointer(p),
            );
        }
    }

    // ---------------------------------------------------------------- L005
    fn registers_overlap(&mut self) {
        let Some(r) = self.j.get("registers") else {
            return;
        };
        let val_bits = r
            .pointer("/config/val_bits")
            .and_then(Value::as_u64)
            .unwrap_or(8);
        let stride = r
            .pointer("/config/stride")
            .and_then(Value::as_u64)
            .unwrap_or(1);
        let Some(map) = r.get("map").and_then(obj) else {
            return;
        };
        let mut ranges: Vec<(u64, u64, String, bool)> = Vec::new();
        for (name, reg) in map {
            let Some(addr) = reg.get("address").and_then(Value::as_u64) else {
                continue;
            };
            let bits = reg.get("bits").and_then(Value::as_u64).unwrap_or(val_bits);
            let width = (bits.div_ceil(val_bits)).max(1) * stride;
            let (count, rstride) = match (
                reg.pointer("/repeat/count").and_then(Value::as_u64),
                reg.pointer("/repeat/stride").and_then(Value::as_u64),
            ) {
                (Some(c), Some(s)) => (c, s),
                _ => (1, width),
            };
            let end = addr + (count - 1) * rstride + width;
            ranges.push((
                addr,
                end,
                name.clone(),
                reg.get("overlap").and_then(Value::as_bool).unwrap_or(false),
            ));
        }
        ranges.sort();
        for i in 0..ranges.len() {
            for j in (i + 1)..ranges.len() {
                let (a, b) = (&ranges[i], &ranges[j]);
                if b.0 >= a.1 {
                    break;
                }
                if a.3 && b.3 {
                    continue;
                }
                let pa = ptr(&["registers", "map", &a.2]);
                let pb = ptr(&["registers", "map", &b.2]);
                let mut d = Diagnostic::error("DD-L005", format!("registers `{}` (0x{:X}..0x{:X}) and `{}` (0x{:X}..0x{:X}) overlap", a.2, a.0, a.1, b.2, b.0, b.1))
                    .at_opt(self.key(&pb), "overlaps")
                    .with_help("fix the address, `bits`, or `repeat`; if the overlap is real (aliased registers) set `overlap: true` on both (spec §8.2)")
                    .with_pointer(pb);
                if let Some(s) = self.key(&pa) {
                    d = d.with_related(s, "this register");
                }
                self.push(d);
            }
        }
    }

    // ---------------------------------------------------------------- L021
    fn register_fields(&mut self) {
        let Some(r) = self.j.get("registers") else {
            return;
        };
        let val_bits = r
            .pointer("/config/val_bits")
            .and_then(Value::as_u64)
            .unwrap_or(8) as u32;
        let mut sets: Vec<(String, &Value, u32)> = Vec::new();
        if let Some(map) = r.get("map").and_then(obj) {
            for (name, reg) in map {
                let bits = reg
                    .get("bits")
                    .and_then(Value::as_u64)
                    .map_or(val_bits, |b| b as u32);
                if let Some(f) = reg.get("fields") {
                    sets.push((ptr(&["registers", "map", name, "fields"]), f, bits));
                }
                if let Some(fs) = reg.get("fieldset").and_then(Value::as_str)
                    && r.pointer(&format!("/fieldsets/{}", escape_token(fs)))
                        .is_none()
                {
                    let p = ptr(&["registers", "map", name, "fieldset"]);
                    self.push(
                        Diagnostic::error("DD-L020", format!("unknown fieldset `{fs}`"))
                            .at_opt(self.span(&p), "not in registers.fieldsets")
                            .with_pointer(p),
                    );
                }
            }
        }
        for (base, fields, width) in sets {
            let Some(fm) = obj(fields) else { continue };
            let mut used: Vec<(u32, u32, String)> = Vec::new();
            for (fname, f) in fm {
                let p = format!("{base}/{}/bits", escape_token(fname));
                let Some((h, lo)) = f.get("bits").and_then(Value::as_str).and_then(parse_bits)
                else {
                    continue;
                };
                if h < lo {
                    self.push(
                        Diagnostic::error(
                            "DD-L021",
                            format!("field `{fname}` has bits \"{h}:{lo}\" with high < low"),
                        )
                        .at_opt(self.span(&p), "reversed")
                        .with_help(format!("write \"{lo}:{h}\" (high bit first)"))
                        .with_pointer(p),
                    );
                    continue;
                }
                if h >= width {
                    self.push(
                        Diagnostic::error(
                            "DD-L021",
                            format!(
                                "field `{fname}` uses bit {h} but the register is {width} bits wide"
                            ),
                        )
                        .at_opt(self.span(&p), "outside the register")
                        .with_help(format!(
                            "bits run 0..{}; set the register's `bits` if it is wider (spec §8.3)",
                            width - 1
                        ))
                        .with_pointer(p.clone()),
                    );
                }
                if let Some((_, _, other)) = used.iter().find(|(oh, ol, _)| lo <= *oh && *ol <= h) {
                    self.push(
                        Diagnostic::error(
                            "DD-L021",
                            format!("field `{fname}` overlaps field `{other}`"),
                        )
                        .at_opt(self.span(&p), "overlap")
                        .with_help("each bit belongs to at most one field (spec §8.3)")
                        .with_pointer(p),
                    );
                }
                used.push((h, lo, fname.clone()));
            }
        }
    }

    // ---------------------------------------------------------------- register references
    fn check_reg_ref(&mut self, target: &str, p: &str, writing: bool) {
        let (reg, field) = match target.split_once('.') {
            Some((r, f)) => (r, Some(f)),
            None => (target, None),
        };
        let Some(r) = self.regs.get(reg).cloned() else {
            let help = suggest(reg, self.regs.keys().map(String::as_str)).map_or_else(
                || "declare it under `registers.map` (spec §8)".to_string(),
                |s| format!("did you mean `{s}`?"),
            );
            self.push(
                Diagnostic::error("DD-L020", format!("unknown register `{reg}`"))
                    .at_opt(self.span(p), "not in registers.map")
                    .with_help(help)
                    .with_pointer(p.to_string()),
            );
            return;
        };
        if let Some(f) = field {
            match r.fields.get(f) {
                None => {
                    let help = suggest(f, r.fields.keys().map(String::as_str)).map_or_else(
                        || {
                            format!(
                                "fields of `{reg}`: {}",
                                r.fields.keys().cloned().collect::<Vec<_>>().join(", ")
                            )
                        },
                        |s| format!("did you mean `{reg}.{s}`?"),
                    );
                    self.push(
                        Diagnostic::error(
                            "DD-L020",
                            format!("register `{reg}` has no field `{f}`"),
                        )
                        .at_opt(self.span(p), "unknown field")
                        .with_help(help)
                        .with_pointer(p.to_string()),
                    );
                }
                Some((_, _, acc)) if writing => {
                    let a = acc.as_deref().unwrap_or(&r.access);
                    if matches!(a, "ro" | "rc") {
                        self.ro_write(&format!("{reg}.{f}"), a, p);
                    }
                }
                _ => {}
            }
        } else if writing && matches!(r.access.as_str(), "ro" | "rc") {
            self.ro_write(reg, &r.access, p);
        }
    }

    fn ro_write(&mut self, what: &str, access: &str, p: &str) {
        self.push(
            Diagnostic::error("DD-L015", format!("write to `{what}`, which is read-only (`access: {access}`)"))
                .at_opt(self.span(p), "read-only")
                .with_help("check the register name, or fix `access` if the datasheet says it is writable (spec §8.2)")
                .with_pointer(p.to_string()),
        );
    }

    // ---------------------------------------------------------------- steps (shared by commands and sequences)
    /// Lint a step list. `scope` holds names visible to expressions and grows as steps define
    /// locals. Returns (writes?, touches device?, literal wait ms).
    fn steps(
        &mut self,
        base: &str,
        steps: &'a Value,
        scope: &mut Scope,
        reads_only: bool,
    ) -> (bool, bool, f64) {
        let mut writes = false;
        let mut touches = false;
        let mut wait = 0.0;
        let Some(list) = steps.as_array() else {
            return (false, false, 0.0);
        };
        for (i, step) in list.iter().enumerate() {
            let sp = format!("{base}/{i}");
            let Some(op) = step_op(step) else {
                // Schema reports the unknown operation; keep its result name to avoid cascades.
                if let Some(a) = step.get("as").and_then(Value::as_str) {
                    scope.names.insert(a.to_string());
                }
                continue;
            };
            let opp = format!("{sp}/{}", escape_token(op));
            wait += literal_wait_ms(step);
            if step_writes(step) {
                writes = true;
                if reads_only {
                    self.push(
                        Diagnostic::error("DD-L019", format!("command is declared `reads_only` but step {i} (`{op}`) writes to the device"))
                            .at_opt(self.key(&opp), "writes")
                            .with_help("remove `reads_only: true`, or remove the write (spec §9.1)")
                            .with_pointer(opp.clone()),
                    );
                }
            }
            if step_touches_device(step) {
                touches = true;
            }
            // Register references.
            match op {
                "read" | "read_block" | "poll" => {
                    if let Some(t) = step[op].as_str() {
                        self.check_reg_ref(t, &opp, false);
                    }
                }
                "write" | "update" => {
                    if let Some(t) = step[op].as_str() {
                        self.check_reg_ref(t, &opp, true);
                        if op == "update"
                            && let Some(fm) = step.get("fields").and_then(obj)
                        {
                            for (f, v) in fm {
                                let fp = format!("{sp}/fields/{}", escape_token(f));
                                self.check_reg_ref(&format!("{t}.{f}"), &fp, true);
                                if let Some(name) = v.as_str() {
                                    self.check_enum_value(t, f, name, &fp);
                                }
                            }
                        }
                    }
                }
                "call" => {
                    if let Some(c) = step["call"].as_str() {
                        if !self.commands.contains_key(c) {
                            self.unknown_ref(
                                "command",
                                c,
                                &opp,
                                self.commands.keys().cloned().collect(),
                            );
                        } else if let Some(args) = step.get("args").and_then(obj) {
                            let declared = keys(self.commands[c].get("args"));
                            for a in args.keys() {
                                if !declared.contains(a) {
                                    let ap = format!("{sp}/args/{}", escape_token(a));
                                    self.unknown_ref(
                                        &format!("argument of `{c}`"),
                                        a,
                                        &ap,
                                        declared.clone(),
                                    );
                                }
                            }
                        }
                    }
                }
                "enqueue" | "await" => {
                    if let Some(q) = step[op].as_str()
                        && !self.queues.contains_key(q)
                    {
                        self.unknown_ref("queue", q, &opp, self.queues.keys().cloned().collect());
                    }
                }
                "dma_addr" | "read_memory" | "write_memory" => {
                    if let Some(m) = step[op].as_str()
                        && !self.memory.contains(m)
                    {
                        self.unknown_ref("memory region", m, &opp, self.memory.clone());
                    }
                }
                "transfer" => {
                    if let Some(e) = step.pointer("/transfer/endpoint").and_then(Value::as_str) {
                        let eps = keys(self.j.pointer("/transport/usb/endpoints"));
                        if !eps.contains(e) {
                            self.unknown_ref("endpoint", e, &format!("{opp}/endpoint"), eps);
                        }
                    }
                }
                "hid_get" | "hid_set" | "hid_read" | "hid_write" => {
                    if let Some(r) = step[op].as_str() {
                        let reports = keys(self.j.pointer("/transport/hid/reports"));
                        if !reports.contains(r) {
                            self.unknown_ref("HID report", r, &opp, reports);
                        }
                    }
                }
                "store" => {
                    if let Some(v) = step["store"].as_str()
                        && !self.vars.contains(v)
                    {
                        self.unknown_ref("variable", v, &opp, self.vars.clone());
                    }
                }
                _ => {}
            }
            // Expressions in this step see the scope so far (+ `value` in poll.until).
            let mut local = scope.clone();
            if op == "poll" {
                local.names.insert("value".into());
            }
            if op == "await"
                && let Some(q) = step["await"].as_str()
                && let Some(e) = self
                    .queues
                    .get(q)
                    .and_then(|q| q.get("entry"))
                    .and_then(Value::as_str)
            {
                local.names.insert(e.to_string());
            }
            self.exprs_in(step, &sp, &local);
            // Locals defined by this step.
            if let Some(a) = step.get("as").and_then(Value::as_str) {
                scope.names.insert(a.to_string());
                if op == "call"
                    && let Some(c) = step["call"].as_str()
                    && let Some(cmd) = self.commands.get(c)
                {
                    scope
                        .call_outputs
                        .insert(a.to_string(), keys(cmd.get("output")));
                }
            }
            if let Some(n) = step.get("let").and_then(Value::as_str) {
                scope.names.insert(n.to_string());
            }
            if op == "receive" {
                for n in struct_names(step.pointer("/receive/payload")) {
                    if step.get("as").is_none() {
                        scope.names.insert(n);
                    }
                }
                if let Some(a) = step.get("as").and_then(Value::as_str) {
                    scope.call_outputs.insert(
                        a.to_string(),
                        struct_names(step.pointer("/receive/payload"))
                            .into_iter()
                            .collect(),
                    );
                }
            }
        }
        (writes, touches, wait)
    }

    fn check_enum_value(&mut self, reg: &str, field: &str, value: &str, p: &str) {
        let Some(map) = self.j.pointer("/registers/map").and_then(obj) else {
            return;
        };
        let Some(r) = map.get(reg) else { return };
        let fdef = r
            .pointer(&format!("/fields/{}", escape_token(field)))
            .or_else(|| {
                let fs = r.get("fieldset")?.as_str()?;
                self.j.pointer(&format!(
                    "/registers/fieldsets/{}/{}",
                    escape_token(fs),
                    escape_token(field)
                ))
            });
        let names = keys(fdef.and_then(|f| f.get("enum")));
        if !names.contains(value) {
            let help = if names.is_empty() {
                format!("`{reg}.{field}` has no `enum`; write a number or `{{ cel: \"...\" }}`")
            } else {
                suggest(value, names.iter().map(String::as_str)).map_or_else(
                    || {
                        format!(
                            "named values: {}",
                            names.iter().cloned().collect::<Vec<_>>().join(", ")
                        )
                    },
                    |s| format!("did you mean `{s}`?"),
                )
            };
            self.push(
                Diagnostic::error(
                    "DD-L020",
                    format!("`{value}` is not a named value of `{reg}.{field}`"),
                )
                .at_opt(self.span(p), "unknown value")
                .with_help(help)
                .with_pointer(p.to_string()),
            );
        }
    }

    fn unknown_ref(&mut self, what: &str, name: &str, p: &str, candidates: BTreeSet<String>) {
        let help = suggest(name, candidates.iter().map(String::as_str)).map_or_else(
            || {
                if candidates.is_empty() {
                    format!("no {what} is declared")
                } else {
                    format!(
                        "declared: {}",
                        candidates.iter().cloned().collect::<Vec<_>>().join(", ")
                    )
                }
            },
            |s| format!("did you mean `{s}`?"),
        );
        self.push(
            Diagnostic::error("DD-L029", format!("unknown {what} `{name}`"))
                .at_opt(self.span(p), "not declared")
                .with_help(help)
                .with_pointer(p.to_string()),
        );
    }

    /// Check identifiers in every CEL string directly inside `v` (not nested step lists).
    fn exprs_in(&mut self, v: &Value, base: &str, scope: &Scope) {
        let mut found = Vec::new();
        visit_json(v, base, &mut |p, k, val| {
            let Some(k) = k else { return };
            if in_struct(p) || p[base.len()..].contains("/steps/") {
                return;
            }
            if let Some(s) = val.as_str()
                && (CEL_KEYS.contains(&k) || INT_OR_CEL_KEYS.contains(&k))
            {
                found.push((p.to_string(), s.to_string()));
            }
        });
        for (p, s) in found {
            self.check_identifiers(&s, &p, scope);
        }
    }

    fn check_identifiers(&mut self, src: &str, p: &str, scope: &Scope) {
        let Ok(c) = expr::check(src) else { return }; // L012 reports it
        for root in &c.roots {
            let known = scope.names.contains(root)
                || matches!(
                    root.as_str(),
                    "vars" | "instance" | "variant" | "quirks" | "settings" | "args"
                );
            if !known {
                let candidates: Vec<String> = scope
                    .names
                    .iter()
                    .cloned()
                    .chain(["vars", "instance", "variant", "settings"].map(String::from))
                    .collect();
                let help = suggest(root, candidates.iter().map(String::as_str)).map_or_else(
                    || {
                        format!(
                            "names available here: {}",
                            if scope.names.is_empty() {
                                "(none; define one with `as:` or `let:`)".to_string()
                            } else {
                                scope.names.iter().cloned().collect::<Vec<_>>().join(", ")
                            }
                        )
                    },
                    |s| format!("did you mean `{s}`?"),
                );
                self.push(
                    Diagnostic::error("DD-L022", format!("unknown name `{root}` in expression"))
                        .at_opt(self.span(p), "not defined here")
                        .with_help(help)
                        .with_pointer(p.to_string()),
                );
            }
        }
        for sel in &c.selections {
            let (root, field) = sel.split_once('.').expect("dotted");
            let set: Option<&BTreeSet<String>> = match root {
                "vars" => Some(&self.vars),
                "settings" => Some(&self.settings),
                "instance" => Some(&self.instance),
                "variant" if field != "id" => Some(&self.variant_params),
                _ => scope.call_outputs.get(root),
            };
            if let Some(set) = set
                && !set.contains(field)
            {
                let help = suggest(field, set.iter().map(String::as_str)).map_or_else(
                    || {
                        if set.is_empty() {
                            format!("nothing is declared for `{root}`")
                        } else {
                            format!(
                                "available: {}",
                                set.iter().cloned().collect::<Vec<_>>().join(", ")
                            )
                        }
                    },
                    |s| format!("did you mean `{root}.{s}`?"),
                );
                self.push(
                    Diagnostic::error("DD-L022", format!("`{root}` has no `{field}`"))
                        .at_opt(self.span(p), "unknown")
                        .with_help(help)
                        .with_pointer(p.to_string()),
                );
            }
        }
    }

    // ---------------------------------------------------------------- commands
    fn commands(&mut self) {
        let cmds: Vec<(String, &'a Value)> =
            self.commands.iter().map(|(k, v)| (k.clone(), *v)).collect();
        let limits: BTreeSet<String> = keys(self.j.pointer("/safety/limits"));
        let has_framing = self.j.pointer("/transport/framing").is_some();
        for (name, cmd) in cmds {
            let base = ptr(&["commands", &name]);
            let args = keys(cmd.get("args"));
            let reads_only = cmd
                .get("reads_only")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let mut scope = Scope::default();
            scope.names.extend(args.iter().cloned());
            let (mut writes, mut touches, wait) = match cmd.get("steps") {
                Some(steps) => self.steps(&format!("{base}/steps"), steps, &mut scope, reads_only),
                None => (false, false, 0.0),
            };
            if let Some(send) = cmd.get("send") {
                touches = true;
                if !has_framing {
                    self.push(
                        Diagnostic::error(
                            "DD-L029",
                            format!(
                                "command `{name}` uses `send` but the transport has no `framing`"
                            ),
                        )
                        .at_opt(self.key(&format!("{base}/send")), "needs framing")
                        .with_help("add `transport.framing` (spec §7.3) or use `steps`")
                        .with_pointer(format!("{base}/send")),
                    );
                }
                let payload = struct_names(send.get("payload"));
                writes |= !payload.is_empty();
                for f in &payload {
                    if !args.contains(f) {
                        let p = format!("{base}/send/payload");
                        self.push(
                            Diagnostic::error("DD-L022", format!("payload field `{f}` of `{name}` has no matching argument"))
                                .at_opt(self.span(&p), "no argument with this name")
                                .with_help(format!("declare `args: {{ {f}: {{ type: ... }} }}` (payload fields take the argument of the same name, spec §9.4)"))
                                .with_pointer(p),
                        );
                    }
                }
                if reads_only && !payload.is_empty() {
                    self.push(
                        Diagnostic::error(
                            "DD-L019",
                            format!(
                                "command `{name}` is declared `reads_only` but sends a payload"
                            ),
                        )
                        .at_opt(self.key(&format!("{base}/reads_only")), "reads_only")
                        .with_pointer(format!("{base}/reads_only")),
                    );
                }
                scope
                    .names
                    .extend(struct_names(cmd.pointer("/expect/payload")));
            }
            // L003 timeout
            let timeout = cmd.get("timeout_ms").and_then(Value::as_f64);
            let near_miss = cmd.as_object().is_some_and(|m| {
                m.keys()
                    .any(|k| k != "timeout_ms" && suggest(k, ["timeout_ms"]).is_some())
            });
            if touches && cmd.get("timeout_ms").is_none() && !near_miss {
                self.push(
                    Diagnostic::error("DD-L003", format!("command `{name}` talks to the device but has no `timeout_ms`"))
                        .at_opt(self.key(&base), "add timeout_ms")
                        .with_help("every device interaction needs a deadline so a stuck device cannot hang the host (spec §9.1)")
                        .with_pointer(base.clone()),
                );
            }
            // L014 timeout vs waits
            if let Some(t) = timeout
                && wait > t
            {
                let p = format!("{base}/timeout_ms");
                self.push(
                    Diagnostic::error(
                        "DD-L014",
                        format!(
                            "`timeout_ms: {t}` is shorter than the {wait} ms of waits in `{name}`"
                        ),
                    )
                    .at_opt(self.span(&p), "too short")
                    .with_help(format!("raise `timeout_ms` to at least {}", wait.ceil()))
                    .with_pointer(p),
                );
            }
            // L004 numeric write arguments need bounds
            if writes && let Some(am) = cmd.get("args").and_then(obj) {
                for (a, def) in am {
                    let ty = def.get("type").and_then(Value::as_str).unwrap_or("");
                    if matches!(ty, "bool" | "string" | "bytes" | "") {
                        continue;
                    }
                    let bounded = (def.get("min").is_some() && def.get("max").is_some())
                        || def.get("enum").is_some();
                    if !bounded && !limits.contains(&format!("{name}.{a}")) {
                        let p = format!("{base}/args/{}", escape_token(a));
                        self.push(
                            Diagnostic::error("DD-L004", format!("argument `{a}` of `{name}` is written to the device but has no bounds"))
                                .at_opt(self.key(&p), "unbounded")
                                .with_help(format!("add `min`/`max` or `enum` to the argument, or a limit `safety.limits.{name}.{a}` (spec §9.2, §14.2)"))
                                .with_pointer(p),
                        );
                    }
                }
            }
            // L025 retries on non-idempotent
            let retries = cmd.get("retries").and_then(Value::as_u64).unwrap_or(0);
            let idempotent = cmd
                .get("idempotent")
                .and_then(Value::as_bool)
                .unwrap_or(!writes);
            if retries > 0 && !idempotent {
                let p = format!("{base}/retries");
                self.push(
                    Diagnostic::error("DD-L025", format!("`{name}` retries a command that writes to the device"))
                        .at_opt(self.key(&p), "unsafe retry")
                        .with_help("set `idempotent: true` if running it twice is harmless, or remove `retries` (spec §9.1)")
                        .with_pointer(p),
                );
            }
            // Outputs see every local.
            if let Some(om) = cmd.get("output").and_then(obj) {
                for (o, def) in om {
                    if let Some(c) = def.get("cel").and_then(Value::as_str) {
                        let p = format!("{base}/output/{}/cel", escape_token(o));
                        self.check_identifiers(c, &p, &scope);
                        self.int_division(c, def.get("unit").and_then(Value::as_str), &p);
                    }
                }
            }
            if let Some(states) = cmd.get("requires_state").and_then(Value::as_array) {
                let declared = self.state_names();
                for (i, s) in states.iter().enumerate() {
                    if let Some(s) = s.as_str()
                        && !declared.contains(s)
                    {
                        self.push(
                            Diagnostic::error("DD-L002", format!("unknown state `{s}`"))
                                .at_opt(
                                    self.span(&format!("{base}/requires_state/{i}")),
                                    "not a state",
                                )
                                .with_pointer(format!("{base}/requires_state/{i}")),
                        );
                    }
                }
            }
        }
    }

    fn int_division(&mut self, src: &str, unit: Option<&str>, p: &str) {
        let Some(unit) = unit else { return };
        if unit == "1"
            || !src.contains('/')
            || src.contains("double(")
            || src.contains('.')
                && src.chars().any(|c| c.is_ascii_digit())
                && has_float_literal(src)
        {
            return;
        }
        self.push(
            Diagnostic::warning(
                "DD-L006",
                format!("integer division in an output with unit `{unit}` drops the fraction"),
            )
            .at_opt(self.span(p), "integer arithmetic")
            .with_help("CEL has no implicit conversion: write `double(raw) / 100.0` (spec §10.2)")
            .with_pointer(p.to_string()),
        );
    }

    // ---------------------------------------------------------------- sequences
    fn sequences(&mut self) {
        let Some(m) = self.j.get("sequences").and_then(obj) else {
            return;
        };
        for (name, steps) in m {
            let mut scope = Scope::default();
            self.steps(&ptr(&["sequences", name]), steps, &mut scope, false);
        }
    }

    // ---------------------------------------------------------------- states
    fn state_names(&self) -> BTreeSet<String> {
        let mut s: BTreeSet<String> = BTreeSet::new();
        let Some(st) = self.j.get("states") else {
            return s;
        };
        if let Some(i) = st.get("initial").and_then(Value::as_str) {
            s.insert(i.to_string());
        }
        if let Some(l) = st.get("list").and_then(Value::as_array) {
            s.extend(l.iter().filter_map(Value::as_str).map(str::to_string));
        }
        for t in st
            .get("transitions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for k in ["from", "to"] {
                match t.get(k) {
                    Some(Value::String(x)) if x != "*" => {
                        s.insert(x.clone());
                    }
                    Some(Value::Array(a)) => {
                        s.extend(a.iter().filter_map(Value::as_str).map(str::to_string))
                    }
                    _ => {}
                }
            }
        }
        s
    }

    fn states(&mut self) {
        let Some(st) = self.j.get("states") else {
            return;
        };
        self.states = self.state_names();
        let list: Option<BTreeSet<String>> = st.get("list").and_then(Value::as_array).map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        });
        let initial = st
            .get("initial")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let transitions: Vec<&Value> = st
            .get("transitions")
            .and_then(Value::as_array)
            .map(|a| a.iter().collect())
            .unwrap_or_default();
        // L002 against an explicit list
        if let Some(list) = &list {
            if !initial.is_empty() && !list.contains(&initial) {
                self.push(
                    Diagnostic::error(
                        "DD-L002",
                        format!("initial state `{initial}` is not in `states.list`"),
                    )
                    .at_opt(self.span("/states/initial"), "undeclared")
                    .with_pointer("/states/initial"),
                );
            }
            for (i, t) in transitions.iter().enumerate() {
                for k in ["from", "to"] {
                    let names: Vec<(String, String)> = match t.get(k) {
                        Some(Value::String(x)) if x != "*" => {
                            vec![(x.clone(), format!("/states/transitions/{i}/{k}"))]
                        }
                        Some(Value::Array(a)) => a
                            .iter()
                            .enumerate()
                            .filter_map(|(j, v)| {
                                v.as_str().map(|s| {
                                    (s.to_string(), format!("/states/transitions/{i}/{k}/{j}"))
                                })
                            })
                            .collect(),
                        _ => vec![],
                    };
                    for (n, p) in names {
                        if !list.contains(&n) {
                            let help = suggest(&n, list.iter().map(String::as_str)).map_or_else(
                                || {
                                    format!(
                                        "declared states: {}",
                                        list.iter().cloned().collect::<Vec<_>>().join(", ")
                                    )
                                },
                                |s| format!("did you mean `{s}`?"),
                            );
                            self.push(
                                Diagnostic::error(
                                    "DD-L002",
                                    format!("transition uses undefined state `{n}`"),
                                )
                                .at_opt(self.span(&p), "not in states.list")
                                .with_help(help)
                                .with_pointer(p),
                            );
                        }
                    }
                }
            }
        }
        // L001 reachability
        let mut edges: HashMap<String, Vec<String>> = HashMap::new();
        for t in &transitions {
            let Some(to) = t.get("to").and_then(Value::as_str) else {
                continue;
            };
            let froms: Vec<String> = match t.get("from") {
                Some(Value::String(x)) if x == "*" => self.states.iter().cloned().collect(),
                Some(Value::String(x)) => vec![x.clone()],
                Some(Value::Array(a)) => a
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect(),
                _ => vec![],
            };
            for f in froms {
                edges.entry(f).or_default().push(to.to_string());
            }
        }
        let mut seen: HashSet<String> = HashSet::new();
        let mut q = VecDeque::from([initial.clone()]);
        while let Some(s) = q.pop_front() {
            if seen.insert(s.clone()) {
                for n in edges.get(&s).into_iter().flatten() {
                    q.push_back(n.clone());
                }
            }
        }
        for s in self.states.clone() {
            if !seen.contains(&s) && !initial.is_empty() {
                let p = self.first_state_mention(&s);
                self.push(
                    Diagnostic::error(
                        "DD-L001",
                        format!("state `{s}` can never be reached from `{initial}`"),
                    )
                    .at_opt(p.as_deref().and_then(|p| self.span(p)), "unreachable")
                    .with_help("add a transition into it, or remove it (spec §12.1)")
                    .with_pointer(p.unwrap_or_else(|| "/states".into())),
                );
            }
        }
        // `do` references and trigger events
        for (i, t) in transitions.iter().enumerate() {
            let p = format!("/states/transitions/{i}");
            match t.get("do") {
                Some(Value::String(seq)) if !self.sequences.contains(seq) => {
                    self.unknown_ref("sequence", seq, &format!("{p}/do"), self.sequences.clone());
                }
                Some(steps @ Value::Array(_)) => {
                    let mut scope = Scope::default();
                    self.steps(&format!("{p}/do"), steps, &mut scope, false);
                }
                _ => {}
            }
            if let Some(e) = t.pointer("/trigger/event").and_then(Value::as_str)
                && !self.events.contains(e)
            {
                self.unknown_ref(
                    "event",
                    e,
                    &format!("{p}/trigger/event"),
                    self.events.clone(),
                );
            }
            if let Some(c) = t.pointer("/trigger/call").and_then(Value::as_str)
                && !self.commands.contains_key(c)
            {
                self.unknown_ref(
                    "command",
                    c,
                    &format!("{p}/trigger/call"),
                    self.commands.keys().cloned().collect(),
                );
            }
        }
    }

    fn first_state_mention(&self, s: &str) -> Option<String> {
        let st = self.j.get("states")?;
        if let Some(l) = st.get("list").and_then(Value::as_array)
            && let Some(i) = l.iter().position(|v| v.as_str() == Some(s))
        {
            return Some(format!("/states/list/{i}"));
        }
        for (i, t) in st.get("transitions")?.as_array()?.iter().enumerate() {
            if t.get("from").and_then(Value::as_str) == Some(s) {
                return Some(format!("/states/transitions/{i}/from"));
            }
            if let Some(a) = t.get("from").and_then(Value::as_array)
                && let Some(j) = a.iter().position(|v| v.as_str() == Some(s))
            {
                return Some(format!("/states/transitions/{i}/from/{j}"));
            }
        }
        None
    }

    // ---------------------------------------------------------------- expose
    fn expose(&mut self) {
        let Some(list) = self.j.get("expose").and_then(Value::as_array) else {
            return;
        };
        let classes: BTreeSet<String> = self
            .j
            .pointer("/device/classes")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        // Class cross-checks only make sense when `device.classes` is a well-formed list.
        let classes_ok = self
            .j
            .pointer("/device/classes")
            .is_some_and(Value::is_array);
        let mut channels: HashSet<(String, String)> = HashSet::new();
        let mut covered: BTreeSet<String> = BTreeSet::new();
        for (i, e) in list.iter().enumerate() {
            let p = format!("/expose/{i}");
            let class = e
                .get("class")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if classes_ok && !class.is_empty() && !classes.contains(&class) {
                self.push(
                    Diagnostic::error(
                        "DD-L007",
                        format!("`expose` uses class `{class}`, which is not in `device.classes`"),
                    )
                    .at_opt(self.span(&format!("{p}/class")), "not claimed")
                    .with_help(format!(
                        "add `{class}` to `device.classes`, or fix the class name"
                    ))
                    .with_pointer(format!("{p}/class")),
                );
            }
            covered.insert(class.clone());
            if let Some(ch) = e.get("channel").and_then(Value::as_str)
                && !channels.insert((class.clone(), ch.to_string()))
            {
                self.push(
                    Diagnostic::error(
                        "DD-L007",
                        format!("channel `{ch}` is exposed twice for class `{class}`"),
                    )
                    .at_opt(self.span(&format!("{p}/channel")), "duplicate")
                    .with_pointer(format!("{p}/channel")),
                );
            }
            if let Some(r) = e.get("read").and_then(Value::as_str) {
                let rp = format!("{p}/read");
                match r.split_once('.') {
                    None => self.push(
                        Diagnostic::error(
                            "DD-L007",
                            format!("`read: {r}` must name a command output as `command.output`"),
                        )
                        .at_opt(self.span(&rp), "needs an output")
                        .with_help(self.commands.get(r).map_or_else(
                            || "e.g. `read_temp.celsius`".to_string(),
                            |c| {
                                format!(
                                    "outputs of `{r}`: {}",
                                    keys(c.get("output"))
                                        .into_iter()
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            },
                        ))
                        .with_pointer(rp),
                    ),
                    Some((c, o)) => match self.commands.get(c) {
                        None => self.unknown_ref_code(
                            "DD-L007",
                            "command",
                            c,
                            &rp,
                            self.commands.keys().cloned().collect(),
                        ),
                        Some(cmd) => {
                            let outs = keys(cmd.get("output"));
                            if !outs.contains(o) {
                                self.unknown_ref_code(
                                    "DD-L007",
                                    &format!("output of `{c}`"),
                                    o,
                                    &rp,
                                    outs,
                                );
                            } else {
                                self.profile_unit(
                                    &class,
                                    cmd.pointer(&format!("/output/{}/unit", escape_token(o)))
                                        .and_then(Value::as_str),
                                    &rp,
                                );
                            }
                        }
                    },
                }
            }
            if let Some(w) = e.get("write").and_then(Value::as_str)
                && !self.commands.contains_key(w)
            {
                self.unknown_ref_code(
                    "DD-L007",
                    "command",
                    w,
                    &format!("{p}/write"),
                    self.commands.keys().cloned().collect(),
                );
            }
            if let Some(ev) = e.get("event").and_then(Value::as_str)
                && !self.events.contains(ev)
            {
                self.unknown_ref_code(
                    "DD-L007",
                    "event",
                    ev,
                    &format!("{p}/event"),
                    self.events.clone(),
                );
            }
            if class == "light.indicator"
                && let Some(w) = e.get("write").and_then(Value::as_str)
                && let Some(cmd) = self.commands.get(w)
                && !keys(cmd.get("args")).contains("enabled")
            {
                self.push(
                    Diagnostic::error("DD-L006", format!("`light.indicator` needs `{w}` to take an `enabled: bool` argument"))
                        .at_opt(self.span(&format!("{p}/write")), "profile mismatch")
                        .with_help("the light.indicator profile requires `enabled: bool` and an optional `level` in `%` (spec §15.1)")
                        .with_pointer(format!("{p}/write")),
                );
            }
        }
        for (i, c) in classes.iter().enumerate() {
            let needs_expose = !list.is_empty()
                && !c.starts_with("bridge.")
                && !matches!(
                    c.as_str(),
                    "net.ethernet"
                        | "storage.block"
                        | "camera.video"
                        | "audio.pcm"
                        | "display.basic"
                        | "gpu.compute"
                );
            if needs_expose && !covered.contains(c) {
                self.push(
                    Diagnostic::error(
                        "DD-L007",
                        format!("class `{c}` is claimed but nothing is exposed for it"),
                    )
                    .at_opt(self.span(&format!("/device/classes/{i}")), "no channel")
                    .with_help(format!(
                        "add an `expose` entry with `class: {c}` (spec §15)"
                    ))
                    .with_pointer("/device/classes"),
                );
            }
            if let Some(kind) = c.strip_prefix("bridge.")
                && self.j.pointer(&format!("/provides/{kind}")).is_none()
            {
                self.push(
                    Diagnostic::error("DD-L007", format!("class `{c}` requires `provides.{kind}`"))
                        .at_opt(self.span("/device/classes"), "missing provides")
                        .with_help(
                            "declare which commands implement the bus primitives (spec §6.7)",
                        )
                        .with_pointer("/provides"),
                );
            }
        }
    }

    fn unknown_ref_code(
        &mut self,
        code: &'static str,
        what: &str,
        name: &str,
        p: &str,
        cands: BTreeSet<String>,
    ) {
        self.unknown_ref(what, name, p, cands);
        if let Some(d) = self.out.last_mut() {
            d.code = code;
        }
    }

    fn profile_unit(&mut self, class: &str, unit: Option<&str>, p: &str) {
        let want = match class {
            "sensor.temperature" => "Cel",
            "sensor.pressure" => "Pa",
            "sensor.humidity" => "%",
            _ => return,
        };
        match unit {
            None => self.push(
                Diagnostic::error("DD-L006", format!("`{class}` needs a unit convertible to `{want}`, but the output has no `unit`"))
                    .at_opt(self.span(p), "no unit")
                    .with_help(format!("add `unit: {want}` to the output (spec §15.1)"))
                    .with_pointer(p.to_string()),
            ),
            Some(u) if units::parse(u).is_some() && !units::convertible(u, want) => self.push(
                Diagnostic::error("DD-L006", format!("`{class}` needs {} (`{want}`), but the output is in `{u}` ({})", units::describe(want), units::describe(u)))
                    .at_opt(self.span(p), "wrong kind of unit")
                    .with_help("expose the output that carries this quantity, or fix the unit (spec §15.1)")
                    .with_pointer(p.to_string()),
            ),
            _ => {}
        }
    }

    // ---------------------------------------------------------------- tests
    fn tests(&mut self) {
        let Some(list) = self.j.get("tests").and_then(Value::as_array) else {
            return;
        };
        let variants = keys(self.j.get("variants"));
        let mut rejected: BTreeSet<String> = BTreeSet::new();
        let mut names = HashSet::new();
        for (i, t) in list.iter().enumerate() {
            let p = format!("/tests/{i}");
            if let Some(n) = t.get("name").and_then(Value::as_str)
                && !names.insert(n.to_string())
            {
                self.push(
                    Diagnostic::error("DD-L008", format!("two tests are named `{n}`"))
                        .at_opt(self.span(&format!("{p}/name")), "duplicate")
                        .with_pointer(format!("{p}/name")),
                );
            }
            let Some(call) = t.get("call").and_then(Value::as_str) else {
                continue;
            };
            let Some(cmd) = self.commands.get(call).copied() else {
                self.unknown_ref_code(
                    "DD-L008",
                    "command",
                    call,
                    &format!("{p}/call"),
                    self.commands.keys().cloned().collect(),
                );
                continue;
            };
            if t.get("expect").and_then(Value::as_str) == Some("rejected_by_safety") {
                rejected.insert(call.to_string());
            }
            let outs = keys(cmd.get("output"));
            if let Some(m) = t.get("expect").and_then(obj) {
                if let Some(c) = m.get("cel").and_then(Value::as_str) {
                    let scope = Scope {
                        names: outs.clone(),
                        ..Scope::default()
                    };
                    self.check_identifiers(c, &format!("{p}/expect/cel"), &scope);
                } else if m.get("error").is_none() {
                    for o in m.keys() {
                        if !outs.contains(o) {
                            self.unknown_ref_code(
                                "DD-L008",
                                &format!("output of `{call}`"),
                                o,
                                &format!("{p}/expect/{}", escape_token(o)),
                                outs.clone(),
                            );
                        }
                    }
                }
            }
            let args = keys(cmd.get("args"));
            for a in keys(t.get("args")) {
                if !args.contains(&a) {
                    self.unknown_ref_code(
                        "DD-L008",
                        &format!("argument of `{call}`"),
                        &a,
                        &format!("{p}/args/{}", escape_token(&a)),
                        args.clone(),
                    );
                }
            }
            if let Some(v) = t.get("variant").and_then(Value::as_str)
                && !variants.contains(v)
            {
                self.unknown_ref_code(
                    "DD-L008",
                    "variant",
                    v,
                    &format!("{p}/variant"),
                    variants.clone(),
                );
            }
            if let Some(before) = t.get("before") {
                let mut scope = Scope::default();
                self.steps(&format!("{p}/before"), before, &mut scope, false);
            }
        }
        for l in keys(self.j.pointer("/safety/limits")) {
            let cmd = l.split('.').next().unwrap_or("");
            if !rejected.contains(cmd) {
                let p = format!("/safety/limits/{}", escape_token(&l));
                self.push(
                    Diagnostic::warning("DD-L017", format!("safety limit `{l}` has no test proving it is enforced"))
                        .at_opt(self.key(&p), "untested limit")
                        .with_help(format!("add a test that calls `{cmd}` with an out-of-range value and `expect: rejected_by_safety` (spec §16)"))
                        .with_pointer(p),
                );
            }
        }
    }

    // ---------------------------------------------------------------- safety
    fn safety(&mut self) {
        let classes: Vec<String> = self
            .j
            .pointer("/device/classes")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let safety = self.j.get("safety");
        if safety.is_none() {
            if let Some(c) = classes
                .iter()
                .find(|c| ACTUATOR_PREFIXES.iter().any(|p| c.starts_with(p)))
            {
                self.push(
                    Diagnostic::error("DD-L009", format!("`{c}` is an actuator class but the description has no `safety` block"))
                        .at_opt(self.span("/device/classes"), "actuator")
                        .with_help("add `safety:` with limits for every argument that drives the output, e.g. `safety: { limits: { set_led.level: { max: 80, reason: \"...\" } } }` (spec §14)")
                        .with_pointer("/device/classes"),
                );
            }
            return;
        }
        let safety = safety.expect("checked");
        if safety.get("signed_by").is_none() {
            self.push(
                Diagnostic::warning("DD-L016", "safety block is not signed")
                    .at_opt(self.key("/safety"), "unsigned")
                    .with_help("fine for local development; the registry refuses to publish vendor descriptions with an unsigned safety block (spec §14.6)")
                    .with_pointer("/safety"),
            );
        }
        for l in keys(safety.get("limits")) {
            let p = format!("/safety/limits/{}", escape_token(&l));
            match l.split_once('.') {
                Some((c, a)) => match self.commands.get(c) {
                    None => {
                        self.unknown_ref("command", c, &p, self.commands.keys().cloned().collect())
                    }
                    Some(cmd) => {
                        let args = keys(cmd.get("args"));
                        if !args.contains(a) {
                            self.unknown_ref(&format!("argument of `{c}`"), a, &p, args);
                        }
                    }
                },
                None => self.push(
                    Diagnostic::error(
                        "DD-L029",
                        format!("safety limit `{l}` must be `command.argument`"),
                    )
                    .at_opt(self.key(&p), "not a path")
                    .with_pointer(p),
                ),
            }
        }
        for c in keys(safety.get("rates")) {
            if !self.commands.contains_key(&c) {
                self.unknown_ref(
                    "command",
                    &c,
                    &format!("/safety/rates/{}", escape_token(&c)),
                    self.commands.keys().cloned().collect(),
                );
            }
        }
        for r in keys(safety.get("registers")) {
            let p = format!("/safety/registers/{}", escape_token(&r));
            self.check_reg_ref(&r, &p, false);
        }
        for (i, fs) in safety
            .get("forbidden_sequences")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            for (j, c) in fs
                .get("sequence")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                if let Some(c) = c.as_str()
                    && !self.commands.contains_key(c)
                {
                    self.unknown_ref(
                        "command",
                        c,
                        &format!("/safety/forbidden_sequences/{i}/sequence/{j}"),
                        self.commands.keys().cloned().collect(),
                    );
                }
            }
        }
        for (i, il) in safety
            .get("interlocks")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            for (j, c) in il
                .get("deny")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                if let Some(c) = c.as_str()
                    && !self.commands.contains_key(c)
                {
                    self.unknown_ref(
                        "command",
                        c,
                        &format!("/safety/interlocks/{i}/deny/{j}"),
                        self.commands.keys().cloned().collect(),
                    );
                }
            }
            if let Some(w) = il.get("when").and_then(Value::as_str) {
                self.check_identifiers(
                    w,
                    &format!("/safety/interlocks/{i}/when"),
                    &Scope::default(),
                );
            }
        }
    }

    // ---------------------------------------------------------------- matching
    fn matching(&mut self) {
        let Some(list) = self.j.get("match").and_then(Value::as_array) else {
            return;
        };
        let variants = keys(self.j.get("variants"));
        let has_id_check = self
            .j
            .get("variants")
            .and_then(obj)
            .is_some_and(|m| m.values().any(|v| v.get("id_check").is_some()));
        for (i, m) in list.iter().enumerate() {
            let p = format!("/match/{i}");
            if m.get("bus").and_then(Value::as_str) == Some("usb") {
                let narrowing = [
                    "pid",
                    "device_class",
                    "interface_class",
                    "interface_subclass",
                    "interface_protocol",
                    "interface_number",
                    "product",
                ];
                if m.get("vid").is_some() && !narrowing.iter().any(|k| m.get(*k).is_some()) {
                    self.push(
                        Diagnostic::warning("DD-L011", "USB match uses only the vendor id; it would claim every product of that vendor")
                            .at_opt(self.key(&format!("{p}/vid")), "too broad")
                            .with_help("add `pid` (and `interface_class` for vendor-specific interfaces) (spec §5.2)")
                            .with_pointer(format!("{p}/vid")),
                    );
                }
            }
            if m.get("probe").is_some() && !has_id_check {
                self.push(
                    Diagnostic::error("DD-L018", "probing needs a read-only identity check, but no variant has `id_check`")
                        .at_opt(self.key(&format!("{p}/probe")), "unsafe probe")
                        .with_help("add `variants: { <name>: { name: ..., id_check: { register: ID, equals: 0x.. } } }` so probing only reads (spec §5.4)")
                        .with_pointer(format!("{p}/probe")),
                );
            }
            if let Some(v) = m.get("variant").and_then(Value::as_str)
                && !variants.contains(v)
            {
                self.unknown_ref("variant", v, &format!("{p}/variant"), variants.clone());
            }
            if let Some(inst) = m.get("instance").and_then(obj) {
                for k in inst.keys() {
                    if !self.instance.contains(k) {
                        self.unknown_ref(
                            "instance parameter",
                            k,
                            &format!("{p}/instance/{}", escape_token(k)),
                            self.instance.clone(),
                        );
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------- variants
    fn variants(&mut self) {
        let Some(vs) = self.j.get("variants").and_then(obj) else {
            return;
        };
        let mut defaults = 0;
        let mut param_sets: Vec<(String, BTreeSet<String>)> = Vec::new();
        let channels: BTreeSet<String> = self
            .j
            .get("expose")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|e| e.get("channel").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        for (name, v) in vs {
            let p = ptr(&["variants", name]);
            if v.get("default").and_then(Value::as_bool) == Some(true) {
                defaults += 1;
                if defaults > 1 {
                    self.push(
                        Diagnostic::error(
                            "DD-L024",
                            "more than one variant is marked `default: true`",
                        )
                        .at_opt(self.span(&format!("{p}/default")), "second default")
                        .with_pointer(format!("{p}/default")),
                    );
                }
            }
            if let Some(r) = v.pointer("/id_check/register").and_then(Value::as_str) {
                self.check_reg_ref(r, &format!("{p}/id_check/register"), false);
            }
            if let Some(c) = v.pointer("/id_check/command").and_then(Value::as_str) {
                match self.commands.get(c) {
                    None => self.unknown_ref("command", c, &format!("{p}/id_check/command"), self.commands.keys().cloned().collect()),
                    Some(cmd) if cmd.get("reads_only").and_then(Value::as_bool) != Some(true) => self.push(
                        Diagnostic::error("DD-L018", format!("identity check command `{c}` is not declared `reads_only`"))
                            .at_opt(self.span(&format!("{p}/id_check/command")), "may write")
                            .with_help("identity checks must not change device state; add `reads_only: true` to the command (spec §4.3)")
                            .with_pointer(format!("{p}/id_check/command")),
                    ),
                    _ => {}
                }
            }
            for (i, e) in v
                .get("exclude")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                let Some(path) = e.as_str() else { continue };
                let ok = match path.split_once('.') {
                    None => {
                        self.commands.contains_key(path)
                            || self.regs.contains_key(path)
                            || self.settings.contains(path)
                            || channels.contains(path)
                    }
                    Some((a, b)) => {
                        self.commands
                            .get(a)
                            .is_some_and(|c| keys(c.get("output")).contains(b))
                            || self.regs.get(a).is_some_and(|r| r.fields.contains_key(b))
                    }
                };
                if !ok {
                    self.push(
                        Diagnostic::error("DD-L024", format!("variant `{name}` excludes `{path}`, which does not exist"))
                            .at_opt(self.span(&format!("{p}/exclude/{i}")), "unknown")
                            .with_help("exclude names commands, `command.output`, registers, settings or exposed channels (spec §4.3)")
                            .with_pointer(format!("{p}/exclude/{i}")),
                    );
                }
            }
            param_sets.push((name.clone(), keys(v.get("params"))));
        }
        if let Some((first, set)) = param_sets.first().cloned() {
            for (n, s) in &param_sets[1..] {
                if *s != set {
                    self.push(
                        Diagnostic::error(
                            "DD-L024",
                            format!("variants `{first}` and `{n}` define different `params`"),
                        )
                        .at_opt(
                            self.key(&ptr(&["variants", n, "params"]))
                                .or_else(|| self.key(&ptr(&["variants", n]))),
                            "different parameter names",
                        )
                        .with_help("every variant must define the same parameter names (spec §4.3)")
                        .with_pointer(ptr(&["variants", n])),
                    );
                }
            }
        }
    }

    // ---------------------------------------------------------------- L013
    fn framing(&mut self) {
        let Some(list) = self
            .j
            .pointer("/transport/framing")
            .and_then(Value::as_array)
        else {
            return;
        };
        let names: BTreeSet<String> = list
            .iter()
            .filter_map(|f| f.get("name").and_then(Value::as_str))
            .map(str::to_string)
            .collect();
        for (i, f) in list.iter().enumerate() {
            let p = format!("/transport/framing/{i}");
            if let Some(c) = f.get("checksum").and_then(Value::as_str)
                && !crate::checksum::NAMES.contains(&c)
            {
                let help = suggest(c, crate::checksum::NAMES.iter().copied()).map_or_else(
                    || format!("known: {}", crate::checksum::NAMES.join(", ")),
                    |s| format!("did you mean `{s}`?"),
                );
                self.push(
                    Diagnostic::error("DD-L013", format!("unknown checksum algorithm `{c}`"))
                        .at_opt(self.span(&format!("{p}/checksum")), "unknown")
                        .with_help(help)
                        .with_pointer(format!("{p}/checksum")),
                );
            }
            if let Some(n) = f.get("counts").and_then(Value::as_str)
                && !names.contains(n)
            {
                self.unknown_ref("frame field", n, &format!("{p}/counts"), names.clone());
            }
            for (j, n) in f
                .get("over")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                if let Some(n) = n.as_str()
                    && !names.contains(n)
                {
                    self.unknown_ref("frame field", n, &format!("{p}/over/{j}"), names.clone());
                }
            }
        }
    }

    // ---------------------------------------------------------------- L029 misc references
    fn references(&mut self) {
        let cmds: BTreeSet<String> = self.commands.keys().cloned().collect();
        if let Some(pv) = self.j.get("provides").and_then(obj) {
            for (bus, def) in pv {
                for (k, v) in def.as_object().into_iter().flatten() {
                    if matches!(
                        k.as_str(),
                        "write" | "read" | "write_read" | "transfer" | "set" | "get" | "set_baud"
                    ) && let Some(c) = v.as_str()
                        && !cmds.contains(c)
                    {
                        self.unknown_ref("command", c, &ptr(&["provides", bus, k]), cmds.clone());
                    }
                }
            }
        }
        if let Some(st) = self.j.get("settings").and_then(obj) {
            for (n, s) in st {
                if let Some(a) = s.get("apply").and_then(Value::as_str)
                    && !self.sequences.contains(a)
                    && !cmds.contains(a)
                {
                    let all: BTreeSet<String> = self.sequences.union(&cmds).cloned().collect();
                    self.unknown_ref(
                        "sequence or command",
                        a,
                        &ptr(&["settings", n, "apply"]),
                        all,
                    );
                }
            }
        }
        if let Some(evs) = self.j.get("events").and_then(obj) {
            for (n, e) in evs {
                if let Some(c) = e.pointer("/source/poll/command").and_then(Value::as_str)
                    && !cmds.contains(c)
                {
                    self.unknown_ref(
                        "command",
                        c,
                        &ptr(&["events", n, "source", "poll", "command"]),
                        cmds.clone(),
                    );
                }
                if let Some(r) = e.pointer("/source/hid_report").and_then(Value::as_str) {
                    let reports = keys(self.j.pointer("/transport/hid/reports"));
                    if !reports.contains(r) {
                        self.unknown_ref(
                            "HID report",
                            r,
                            &ptr(&["events", n, "source", "hid_report"]),
                            reports,
                        );
                    }
                }
                if let Some(r) = e.pointer("/source/usb_interrupt").and_then(Value::as_str) {
                    let eps = keys(self.j.pointer("/transport/usb/endpoints"));
                    if !eps.contains(r) {
                        self.unknown_ref(
                            "endpoint",
                            r,
                            &ptr(&["events", n, "source", "usb_interrupt"]),
                            eps,
                        );
                    }
                }
                if let Some(irq) = e.pointer("/source/interrupt").and_then(Value::as_str) {
                    let irqs = keys(self.j.get("interrupts"));
                    if !irqs.contains(irq) {
                        self.unknown_ref(
                            "interrupt",
                            irq,
                            &ptr(&["events", n, "source", "interrupt"]),
                            irqs,
                        );
                    }
                }
            }
        }
        for (n, q) in self.queues.clone() {
            if let Some(m) = q.get("memory").and_then(Value::as_str)
                && !self.memory.contains(m)
            {
                self.unknown_ref(
                    "memory region",
                    m,
                    &ptr(&["queues", &n, "memory"]),
                    self.memory.clone(),
                );
            }
            if let Some(t) = q.get("entry").and_then(Value::as_str)
                && !self.types.contains_key(t)
            {
                self.unknown_ref(
                    "type",
                    t,
                    &ptr(&["queues", &n, "entry"]),
                    self.types.keys().cloned().collect(),
                );
            }
            if let Some(pq) = q.get("pair").and_then(Value::as_str)
                && !self.queues.contains_key(pq)
            {
                self.unknown_ref(
                    "queue",
                    pq,
                    &ptr(&["queues", &n, "pair"]),
                    self.queues.keys().cloned().collect(),
                );
            }
            if let Some(r) = q.pointer("/doorbell/write").and_then(Value::as_str) {
                self.check_reg_ref(r, &ptr(&["queues", &n, "doorbell", "write"]), true);
            }
        }
        for (i, f) in self
            .j
            .get("firmware")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            if let Some(l) = f.get("load").and_then(Value::as_str)
                && !self.sequences.contains(l)
            {
                self.unknown_ref(
                    "sequence",
                    l,
                    &format!("/firmware/{i}/load"),
                    self.sequences.clone(),
                );
            }
        }
        for (i, c) in self
            .j
            .pointer("/logic/permissions/commands")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            if let Some(c) = c.as_str()
                && !cmds.contains(c)
            {
                self.unknown_ref(
                    "command",
                    c,
                    &format!("/logic/permissions/commands/{i}"),
                    cmds.clone(),
                );
            }
        }
        if let Some(irqs) = self.j.get("interrupts").and_then(obj) {
            for (n, irq) in irqs {
                if let Some(s) = irq.get("status").and_then(Value::as_str) {
                    self.check_reg_ref(s, &ptr(&["interrupts", n, "status"]), false);
                }
                if let Some(w) = irq.pointer("/ack/write").and_then(Value::as_str) {
                    self.check_reg_ref(w, &ptr(&["interrupts", n, "ack", "write"]), true);
                }
            }
        }
    }

    // ---------------------------------------------------------------- L022 outside commands
    fn expressions_outside_commands(&mut self) {
        let global = Scope::default();
        for section in ["timing", "transport"] {
            if let Some(v) = self.j.get(section) {
                self.exprs_in(v, &format!("/{section}"), &global);
            }
        }
        if let Some(evs) = self.j.get("events").and_then(obj) {
            for (n, e) in evs {
                let scope = Scope {
                    names: struct_names(e.get("decode")).into_iter().collect(),
                    ..Scope::default()
                };
                if let Some(om) = e.get("output").and_then(obj) {
                    for (o, d) in om {
                        if let Some(c) = d.get("cel").and_then(Value::as_str) {
                            self.check_identifiers(
                                c,
                                &ptr(&["events", n, "output", o, "cel"]),
                                &scope,
                            );
                        }
                    }
                }
            }
        }
        for (n, q) in self.queues.clone() {
            let scope = Scope {
                names: ["head", "tail"].map(String::from).into_iter().collect(),
                ..Scope::default()
            };
            self.exprs_in(q, &ptr(&["queues", &n]), &scope);
        }
    }
}

/// Names visible to an expression.
#[derive(Debug, Clone, Default)]
struct Scope {
    names: BTreeSet<String>,
    /// For locals bound by `call … as: x` or `receive … as: x`: the fields `x.<field>` offers.
    call_outputs: HashMap<String, BTreeSet<String>>,
}

fn has_float_literal(s: &str) -> bool {
    let b = s.as_bytes();
    (1..b.len().saturating_sub(1))
        .any(|i| b[i] == b'.' && b[i - 1].is_ascii_digit() && b[i + 1].is_ascii_digit())
}

fn is_leading_zero(t: &str) -> bool {
    let d = t.trim_start_matches(['+', '-']);
    d.len() > 1 && d.starts_with('0') && d.bytes().all(|b| b.is_ascii_digit())
}

/// True for pointers inside payload/decode structs, type definitions or framing, whose
/// string values are type names rather than expressions.
fn in_struct(p: &str) -> bool {
    p.contains("/payload")
        || p.contains("/decode")
        || p.starts_with("/types")
        || p.starts_with("/transport/framing")
}

/// Visit every JSON node with its pointer and parent key.
fn visit_json(v: &Value, p: &str, f: &mut dyn FnMut(&str, Option<&str>, &Value)) {
    fn go(v: &Value, p: &str, key: Option<&str>, f: &mut dyn FnMut(&str, Option<&str>, &Value)) {
        f(p, key, v);
        match v {
            Value::Object(m) => {
                for (k, c) in m {
                    go(c, &format!("{p}/{}", escape_token(k)), Some(k), f);
                }
            }
            Value::Array(a) => {
                for (i, c) in a.iter().enumerate() {
                    // Array items inherit no key.
                    go(c, &format!("{p}/{i}"), None, f);
                }
            }
            _ => {}
        }
    }
    go(v, p, None, f);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint(src: &str) -> Vec<Diagnostic> {
        let mut d = Vec::new();
        let doc = crate::syntax::parse(src, &mut d).expect("yaml");
        run(&doc, true)
    }

    fn codes(src: &str) -> Vec<&'static str> {
        lint(src).into_iter().map(|d| d.code).collect()
    }

    #[test]
    fn parse_bits_forms() {
        assert_eq!(parse_bits("7:5"), Some((7, 5)));
        assert_eq!(parse_bits("3"), Some((3, 3)));
        assert_eq!(parse_bits("x"), None);
    }

    #[test]
    fn yaml11_words_flagged() {
        assert!(codes("a: on\n").contains(&"DD-L027"));
        assert!(codes("on: 1\n").contains(&"DD-L027"));
        assert!(!codes("a: \"on\"\n").contains(&"DD-L027"));
        assert!(codes("a: 010\n").contains(&"DD-L028"));
        assert!(!codes("a: 10\n").contains(&"DD-L028"));
    }

    #[test]
    fn spec_version() {
        let d = lint("spec: driverdesk/0.1.0\n");
        assert_eq!(d[0].code, "DD-L010");
        assert_eq!(d[0].help.as_deref(), Some("write `spec: driverdesk/0.1`"));
    }

    #[test]
    fn unknown_unit_suggests_ucum() {
        let d = lint("x: { unit: degC }\n");
        assert_eq!(d[0].code, "DD-L026");
        assert!(d[0].help.as_deref().unwrap().contains("`Cel`"));
    }

    #[test]
    fn integer_division_warning() {
        let src = "commands:\n  r:\n    steps: [ { read: T, as: raw } ]\n    output: { c: { cel: \"raw / 100\", unit: Cel } }\n    timeout_ms: 5\nregisters: { map: { T: { address: 0, access: ro } } }\n";
        let d = lint(src);
        assert!(
            d.iter()
                .any(|d| d.code == "DD-L006" && d.severity == crate::Severity::Warning),
            "{d:#?}"
        );
        let ok = src.replace("raw / 100", "double(raw) / 100.0");
        assert!(!codes(&ok).contains(&"DD-L006"));
    }
}
