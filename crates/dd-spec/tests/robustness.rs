//! `dd_spec::validate` must never panic, whatever the input (P0 exit criterion 5).
//!
//! Case counts default to a quick local run; the nightly workflow sets `PROPTEST_CASES=100000`
//! and runs in release mode.

use proptest::prelude::*;

/// Keys from the description vocabulary, so random documents reach deep validation paths.
const KEYS: &[&str] = &[
    "spec",
    "device",
    "id",
    "name",
    "vendor",
    "version",
    "classes",
    "match",
    "bus",
    "vid",
    "pid",
    "transport",
    "usb",
    "i2c",
    "interface",
    "endpoints",
    "address",
    "type",
    "framing",
    "commands",
    "steps",
    "read",
    "write",
    "update",
    "fields",
    "bits",
    "as",
    "let",
    "cel",
    "output",
    "unit",
    "timeout_ms",
    "registers",
    "map",
    "config",
    "access",
    "states",
    "initial",
    "transitions",
    "from",
    "to",
    "trigger",
    "do",
    "safety",
    "limits",
    "rates",
    "expose",
    "class",
    "channel",
    "tests",
    "call",
    "expect",
    "args",
    "variants",
    "id_check",
    "register",
    "equals",
    "instance",
    "sequences",
    "init",
    "events",
    "source",
    "queues",
    "memory",
    "x-ext",
    "repeat",
    "count",
    "stride",
    "val_bits",
    "reg_bits",
    "bits",
    "fieldset",
    "exclude",
    "params",
    "on",
    "off",
];

const SCALARS: &[&str] = &[
    "driverdesk/0.1",
    "0x1209",
    "usb",
    "i2c",
    "ro",
    "rw",
    "Cel",
    "Pa",
    "\"raw / 100\"",
    "\"double(raw)\"",
    "\"7:5\"",
    "on",
    "off",
    "yes",
    "~",
    "null",
    "0",
    "18446744073709551615",
    "-9223372036854775808",
    "\"63:0\"",
    "\"99:0\"",
    "m2147483647",
    "true",
    "-1",
    "0o17",
    "010",
    "1e999",
    "\"\"",
    "\"a | b\"",
    "\"shl(1, 64)\"",
    "&a x",
    "*a",
    "!!str x",
    "'q'",
    "|\n  block",
];

fn yaml_value(depth: u32) -> BoxedStrategy<String> {
    let leaf = prop_oneof![
        prop::sample::select(SCALARS).prop_map(str::to_string),
        any::<i64>().prop_map(|i| i.to_string()),
        "[a-zA-Z0-9_.:/ -]{0,12}".prop_map(|s| format!("{s:?}")),
    ];
    if depth == 0 {
        return leaf.boxed();
    }
    prop_oneof![
        3 => leaf,
        2 => prop::collection::vec((prop::sample::select(KEYS), yaml_value(depth - 1)), 0..6)
            .prop_map(|kv| format!("{{ {} }}", kv.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", "))),
        1 => prop::collection::vec(yaml_value(depth - 1), 0..5).prop_map(|v| format!("[ {} ]", v.join(", "))),
    ]
    .boxed()
}

fn document() -> impl Strategy<Value = String> {
    prop::collection::vec((prop::sample::select(KEYS), yaml_value(4)), 0..10).prop_map(|kv| {
        kv.iter()
            .map(|(k, v)| format!("{k}: {v}\n"))
            .collect::<String>()
    })
}

fn corpus() -> Vec<String> {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let mut v = Vec::new();
    for dir in ["devices/examples", "spec/fixtures/invalid"] {
        for e in std::fs::read_dir(root.join(dir)).expect("dir") {
            let p = e.expect("entry").path();
            if p.extension().is_some_and(|x| x == "yaml") {
                v.push(std::fs::read_to_string(p).expect("readable"));
            }
        }
    }
    v
}

fn cases(default: u32) -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(1500), ..ProptestConfig::default() })]

    #[test]
    fn structured_documents_never_panic(doc in document()) {
        let _ = dd_spec::validate(&doc);
    }

    #[test]
    fn arbitrary_text_never_panics(s in any::<String>()) {
        let _ = dd_spec::validate(&s);
    }

    #[test]
    fn mutated_corpus_never_panics(idx in any::<prop::sample::Index>(), edits in prop::collection::vec((any::<prop::sample::Index>(), any::<u8>(), 0u8..3), 1..8)) {
        let corpus = corpus();
        let mut bytes = corpus[idx.index(corpus.len())].clone().into_bytes();
        for (pos, b, op) in edits {
            if bytes.is_empty() { break; }
            let i = pos.index(bytes.len());
            match op {
                0 => bytes[i] = b,
                1 => bytes.insert(i, b),
                _ => { bytes.remove(i); }
            }
        }
        let s = String::from_utf8_lossy(&bytes);
        let _ = dd_spec::validate(&s);
    }
}

/// Inputs that once crashed the validator (see docs/debug/). They must keep validating.
#[test]
fn past_crashes_stay_fixed() {
    let dir = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/regressions"));
    let mut n = 0;
    for e in std::fs::read_dir(dir).expect("regressions dir") {
        let src = std::fs::read(e.expect("entry").path()).expect("readable");
        let _ = dd_spec::validate(&String::from_utf8_lossy(&src));
        n += 1;
    }
    assert!(n >= 1);
}
