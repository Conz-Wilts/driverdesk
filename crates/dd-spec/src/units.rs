//! UCUM unit subset (spec §11): parse unit codes and check that two units are convertible.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde_json::Value;

const TABLE: &str = include_str!("../../../spec/units/ucum-subset.yaml");
const DIMS: [&str; 7] = ["L", "M", "T", "I", "K", "N", "J"];
/// Largest exponent accepted in a unit term (`m3`, `s-2`); real units never need more.
const MAX_EXPONENT: i32 = 64;

/// A dimension vector (exponents of L, M, T, I, K, N, J).
pub type Dim = [i32; 7];

/// A parsed unit: dimension and scale to the coherent SI unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unit {
    /// Dimension exponents.
    pub dim: Dim,
    /// Multiply a value by this to get coherent SI.
    pub factor: f64,
    /// True for units with an offset (Cel, degF); they may not be combined with others.
    pub offset: bool,
}

struct Atom {
    dim: Dim,
    factor: f64,
    offset: bool,
    prefix: bool,
}

struct Table {
    prefixes: Vec<(String, f64)>,
    atoms: HashMap<String, Atom>,
}

fn table() -> &'static Table {
    static T: OnceLock<Table> = OnceLock::new();
    T.get_or_init(|| {
        let mut d = Vec::new();
        let doc = crate::syntax::parse(TABLE, &mut d).expect("ucum-subset.yaml parses");
        let mut prefixes: Vec<(String, f64)> = doc.json["prefixes"]
            .as_object()
            .expect("prefixes")
            .iter()
            .map(|(k, v)| (k.clone(), v.as_f64().expect("prefix factor")))
            .collect();
        prefixes.sort_by_key(|(p, _)| std::cmp::Reverse(p.len()));
        let mut atoms = HashMap::new();
        for (code, u) in doc.json["units"].as_object().expect("units") {
            let mut dim = [0; 7];
            if let Some(m) = u["dim"].as_object() {
                for (k, e) in m {
                    let i = DIMS.iter().position(|x| x == k).expect("known dimension");
                    dim[i] =
                        i32::try_from(e.as_i64().expect("int exponent")).expect("small exponent");
                }
            }
            let factor = u["factor"].as_f64().expect("factor");
            let offset = u.get("offset").and_then(Value::as_f64).is_some();
            let prefix = u.get("prefix").and_then(Value::as_bool).unwrap_or(true);
            atoms.insert(
                code.clone(),
                Atom {
                    dim,
                    factor,
                    offset,
                    prefix,
                },
            );
        }
        Table { prefixes, atoms }
    })
}

/// All unit codes in the table (for suggestions).
pub fn codes() -> Vec<&'static str> {
    let mut v: Vec<&str> = table().atoms.keys().map(String::as_str).collect();
    v.sort_unstable();
    v
}

/// Parse a UCUM expression from the subset, e.g. `m/s2`, `hPa`, `mA`, `Cel`, `%`.
pub fn parse(code: &str) -> Option<Unit> {
    if code.is_empty() || code.chars().any(char::is_whitespace) {
        return None;
    }
    let t = table();
    let mut total = Unit {
        dim: [0; 7],
        factor: 1.0,
        offset: false,
    };
    let mut terms = 0;
    let mut sign = 1;
    let mut rest = code;
    if let Some(r) = rest.strip_prefix('/') {
        sign = -1;
        rest = r;
    }
    loop {
        let end = rest.find(['.', '/']).unwrap_or(rest.len());
        let (term, after) = rest.split_at(end);
        let (atom, exp) = split_exponent(term)?;
        if exp.abs() > MAX_EXPONENT {
            return None;
        }
        let (dim, factor, offset) = resolve_atom(t, atom)?;
        let e = exp * sign;
        for (i, d) in dim.iter().enumerate() {
            total.dim[i] = total.dim[i].checked_add(d.checked_mul(e)?)?;
        }
        total.factor *= factor.powi(e);
        total.offset |= offset;
        terms += 1;
        if after.is_empty() {
            break;
        }
        sign = if after.starts_with('/') { -1 } else { 1 };
        rest = &after[1..];
        if rest.is_empty() {
            return None;
        }
    }
    if total.offset && terms > 1 {
        return None;
    }
    Some(total)
}

fn split_exponent(term: &str) -> Option<(&str, i32)> {
    if term.ends_with(']') {
        return Some((term, 1));
    }
    let digits_start = term
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(i, _)| i);
    match digits_start {
        Some(i) => {
            let (base, num) = term.split_at(i);
            let (base, neg) = match base.strip_suffix('-') {
                Some(b) => (b, true),
                None => (base, false),
            };
            if base.is_empty() {
                // A bare number like "1" is the unity atom.
                return Some((term, 1));
            }
            let n: i32 = num.parse().ok()?;
            Some((base, if neg { -n } else { n }))
        }
        None => Some((term, 1)),
    }
}

fn resolve_atom(t: &Table, atom: &str) -> Option<(Dim, f64, bool)> {
    if let Some(a) = t.atoms.get(atom) {
        return Some((a.dim, a.factor, a.offset));
    }
    for (p, pf) in &t.prefixes {
        if let Some(base) = atom.strip_prefix(p.as_str())
            && let Some(a) = t.atoms.get(base)
            && a.prefix
        {
            return Some((a.dim, a.factor * pf, a.offset));
        }
    }
    None
}

/// True if a value in `a` can be converted to `b` (same dimension).
pub fn convertible(a: &str, b: &str) -> bool {
    matches!((parse(a), parse(b)), (Some(x), Some(y)) if x.dim == y.dim)
}

/// Human description of a dimension, for messages (`pressure`, `temperature`, …).
pub fn describe(code: &str) -> String {
    let Some(u) = parse(code) else {
        return "unknown".to_string();
    };
    let known: BTreeMap<Dim, &str> = [
        ([0, 0, 0, 0, 0, 0, 0], "dimensionless"),
        ([0, 0, 0, 0, 1, 0, 0], "temperature"),
        ([-1, 1, -2, 0, 0, 0, 0], "pressure"),
        ([1, 0, 0, 0, 0, 0, 0], "length"),
        ([0, 0, 1, 0, 0, 0, 0], "time"),
        ([0, 0, -1, 0, 0, 0, 0], "frequency"),
        ([0, 0, 0, 1, 0, 0, 0], "current"),
        ([2, 1, -3, -1, 0, 0, 0], "voltage"),
        ([-2, 0, 0, 0, 0, 0, 1], "illuminance"),
        ([1, 0, -2, 0, 0, 0, 0], "acceleration"),
    ]
    .into_iter()
    .collect();
    known
        .get(&u.dim)
        .map_or_else(|| format!("{:?}", u.dim), |s| (*s).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_units() {
        for c in [
            "Cel", "K", "Pa", "hPa", "kPa", "%", "1", "m/s2", "mA", "V", "lx", "Hz", "ms", "us",
            "[ppm]", "Ohm", "W", "kHz", "mm", "/s",
        ] {
            assert!(parse(c).is_some(), "{c}");
        }
    }

    #[test]
    fn rejects_unknown_and_malformed() {
        for c in ["degC", "C°", "", "m/", "m s", "kCel", "Cel/s"] {
            assert!(parse(c).is_none(), "{c}");
        }
    }

    #[test]
    fn huge_exponents_are_rejected_not_overflowed() {
        assert!(parse("m2147483647").is_none());
        assert!(parse("m3.m2147483647").is_none());
        assert!(parse("m-2147483647").is_none());
    }

    #[test]
    fn convertibility() {
        assert!(convertible("hPa", "Pa"));
        assert!(convertible("Cel", "K"));
        assert!(convertible("m/s2", "[g]"));
        assert!(!convertible("Cel", "Pa"));
        assert!(!convertible("%", "Pa"));
        assert!(convertible("%", "1"));
    }

    #[test]
    fn factors() {
        assert!((parse("hPa").unwrap().factor - 100.0).abs() < 1e-9);
        assert!((parse("mA").unwrap().factor - 1e-3).abs() < 1e-12);
        assert!((parse("km/h").unwrap().factor - 1000.0 / 3600.0).abs() < 1e-9);
    }

    #[test]
    fn describes_dimensions() {
        assert_eq!(describe("hPa"), "pressure");
        assert_eq!(describe("Cel"), "temperature");
    }
}
