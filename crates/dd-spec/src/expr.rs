//! CEL expressions: parsing, reference extraction and the DriverDesk function library (spec §10).

use std::collections::BTreeSet;
use std::sync::Arc;

use cel::common::ast::Expr;
use cel::{Context, ExecutionError, IdedExpr, Program, Value};

/// Error returned when a library function name clashes with the CEL standard library.
pub use cel::DeclarationError;

/// DriverDesk functions every conforming runtime provides (spec §10.3).
pub const LIBRARY: &[&str] = &[
    "shl",
    "shr",
    "ushr",
    "band",
    "bor",
    "bxor",
    "bnot",
    "bit",
    "field",
    "sign_extend",
    "clamp",
    "min",
    "max",
    "abs",
    "round",
    "floor",
    "ceil",
    "trunc",
    "pow",
    "sqrt",
    "log",
    "log10",
    "exp",
    "be16",
    "le16",
    "be24",
    "le24",
    "be32",
    "le32",
    "be16s",
    "le16s",
    "be24s",
    "le24s",
    "be32s",
    "le32s",
    "bytes_of",
    "crc",
];

/// Standard CEL functions and macros the DriverDesk profile allows.
pub const STANDARD: &[&str] = &[
    "size",
    "int",
    "uint",
    "double",
    "string",
    "bytes",
    "bool",
    "type",
    "has",
    "all",
    "exists",
    "exists_one",
    "map",
    "filter",
    "contains",
    "startsWith",
    "endsWith",
    "matches",
];

/// An expression that compiled, with what it references.
#[derive(Debug, Clone)]
pub struct Checked {
    /// Root identifiers used (`raw`, `vars`, `instance`, …).
    pub roots: BTreeSet<String>,
    /// Dotted paths of `root.field` selections (`vars.t1`, `settings.oversampling`).
    pub selections: BTreeSet<String>,
    /// Functions called (excluding operators).
    pub functions: BTreeSet<String>,
}

/// Why an expression was rejected.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprError {
    /// Does not parse.
    Syntax(String),
    /// Calls a function outside the profile.
    UnknownFunction(String),
}

/// Parse `src` and extract its references.
pub fn check(src: &str) -> Result<Checked, ExprError> {
    let program = Program::compile(src)
        .map_err(|e| ExprError::Syntax(syntax_message(src, &e.to_string())))?;
    let mut c = Checked {
        roots: BTreeSet::new(),
        selections: BTreeSet::new(),
        functions: BTreeSet::new(),
    };
    collect(program.expression(), &mut c, &mut BTreeSet::new());
    if let Some(f) = c
        .functions
        .iter()
        .find(|f| !LIBRARY.contains(&f.as_str()) && !STANDARD.contains(&f.as_str()))
    {
        return Err(ExprError::UnknownFunction(f.clone()));
    }
    Ok(c)
}

fn syntax_message(src: &str, raw: &str) -> String {
    for op in ["|", "&", "^", "<<", ">>", "~"] {
        let bitwise = match op {
            "|" => src.contains('|') && !src.contains("||"),
            "&" => src.contains('&') && !src.contains("&&"),
            _ => src.contains(op),
        };
        if bitwise {
            return format!(
                "CEL has no `{op}` operator; use the DriverDesk functions band, bor, bxor, bnot, shl, shr"
            );
        }
    }
    raw.lines()
        .find(|l| l.contains("ERROR"))
        .unwrap_or(raw)
        .trim()
        .to_string()
}

fn collect(e: &IdedExpr, c: &mut Checked, bound: &mut BTreeSet<String>) {
    match &e.expr {
        Expr::Ident(n) => {
            if !bound.contains(n) {
                c.roots.insert(n.clone());
            }
        }
        Expr::Select(s) => {
            if let Expr::Ident(root) = &s.operand.expr {
                if !bound.contains(root) {
                    c.roots.insert(root.clone());
                    c.selections.insert(format!("{root}.{}", s.field));
                }
            } else {
                collect(&s.operand, c, bound);
            }
        }
        Expr::Call(call) => {
            let name = call.func_name.as_str();
            if !name.starts_with('_') && !name.starts_with('@') && name != "!_" && name != "-_" {
                c.functions.insert(name.to_string());
            }
            if let Some(t) = &call.target {
                collect(t, c, bound);
            }
            for a in &call.args {
                collect(a, c, bound);
            }
        }
        Expr::List(l) => {
            for el in &l.elements {
                collect(el, c, bound);
            }
        }
        Expr::Map(m) => {
            for entry in &m.entries {
                if let cel::common::ast::EntryExpr::MapEntry(me) = &entry.expr {
                    collect(&me.key, c, bound);
                    collect(&me.value, c, bound);
                }
            }
        }
        Expr::Comprehension(comp) => {
            collect(&comp.iter_range, c, bound);
            let mut inner = bound.clone();
            inner.insert(comp.iter_var.clone());
            inner.insert(comp.accu_var.clone());
            collect(&comp.accu_init, c, &mut inner);
            collect(&comp.loop_cond, c, &mut inner);
            collect(&comp.loop_step, c, &mut inner);
            collect(&comp.result, c, &mut inner);
        }
        _ => {}
    }
}

fn err(f: &str, m: impl ToString) -> ExecutionError {
    ExecutionError::function_error(f, m)
}

fn shift(f: &str, n: i64) -> Result<u32, ExecutionError> {
    u32::try_from(n)
        .ok()
        .filter(|n| *n < 64)
        .ok_or_else(|| err(f, format!("shift amount {n} outside 0..63")))
}

fn read_be(
    f: &str,
    b: &[u8],
    off: i64,
    len: usize,
    signed: bool,
    little: bool,
) -> Result<i64, ExecutionError> {
    let off = usize::try_from(off).map_err(|_| err(f, "negative offset"))?;
    let slice = b.get(off..off + len).ok_or_else(|| {
        err(
            f,
            format!("needs {len} bytes at offset {off}, have {}", b.len()),
        )
    })?;
    let mut v: u64 = 0;
    for i in 0..len {
        let byte = if little { slice[len - 1 - i] } else { slice[i] };
        v = (v << 8) | u64::from(byte);
    }
    let bits = (len * 8) as u32;
    Ok(if signed {
        sign_ext(v as i64, bits)
    } else {
        v as i64
    })
}

fn sign_ext(v: i64, bits: u32) -> i64 {
    if bits >= 64 {
        return v;
    }
    let s = 64 - bits;
    (v << s) >> s
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Int(i) => Some(*i as f64),
        Value::UInt(u) => Some(*u as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

/// A CEL context with the DriverDesk library registered.
pub fn context() -> Context<'static, 'static> {
    let mut ctx = Context::default();
    install(&mut ctx).expect("DriverDesk library names do not clash with the CEL standard library");
    ctx
}

macro_rules! byte_fn {
    ($ctx:ident, $name:literal, $len:expr, $signed:expr, $little:expr) => {
        $ctx.add_function($name, |b: Arc<Vec<u8>>, off: i64| {
            read_be($name, &b, off, $len, $signed, $little)
        })?;
    };
}

/// Register the library into `ctx`.
pub fn install(ctx: &mut Context<'static, 'static>) -> Result<(), DeclarationError> {
    ctx.add_function("shl", |a: i64, n: i64| -> Result<i64, ExecutionError> {
        let n = shift("shl", n)?;
        let r = a.checked_shl(n).ok_or_else(|| err("shl", "overflow"))?;
        if (r >> n) != a {
            return Err(err("shl", format!("{a} << {n} overflows 64 bits")));
        }
        Ok(r)
    })?;
    ctx.add_function("shr", |a: i64, n: i64| -> Result<i64, ExecutionError> {
        Ok(a >> shift("shr", n)?)
    })?;
    ctx.add_function("ushr", |a: i64, n: i64| -> Result<i64, ExecutionError> {
        Ok(((a as u64) >> shift("ushr", n)?) as i64)
    })?;
    ctx.add_function("band", |a: i64, b: i64| a & b)?;
    ctx.add_function("bor", |a: i64, b: i64| a | b)?;
    ctx.add_function("bxor", |a: i64, b: i64| a ^ b)?;
    ctx.add_function("bnot", |a: i64| !a)?;
    ctx.add_function("bit", |a: i64, n: i64| -> Result<bool, ExecutionError> {
        Ok((a >> shift("bit", n)?) & 1 == 1)
    })?;
    ctx.add_function(
        "field",
        |a: i64, hi: i64, lo: i64| -> Result<i64, ExecutionError> {
            let (h, l) = (shift("field", hi)?, shift("field", lo)?);
            if h < l {
                return Err(err("field", "hi must be >= lo"));
            }
            let width = h - l + 1;
            let mask = if width == 64 {
                u64::MAX
            } else {
                (1u64 << width) - 1
            };
            Ok((((a as u64) >> l) & mask) as i64)
        },
    )?;
    ctx.add_function(
        "sign_extend",
        |a: i64, bits: i64| -> Result<i64, ExecutionError> {
            let b = u32::try_from(bits)
                .ok()
                .filter(|b| (1..=64).contains(b))
                .ok_or_else(|| err("sign_extend", "bits must be 1..64"))?;
            Ok(sign_ext(a, b))
        },
    )?;
    ctx.add_function(
        "clamp",
        |x: Value, lo: Value, hi: Value| -> Result<Value, ExecutionError> {
            match (&x, &lo, &hi) {
                (Value::Int(x), Value::Int(l), Value::Int(h)) => {
                    Ok(Value::Int((*x).clamp(*l, (*h).max(*l))))
                }
                _ => {
                    let (x, l, h) = (num(&x), num(&lo), num(&hi));
                    match (x, l, h) {
                        (Some(x), Some(l), Some(h)) => Ok(Value::Float(x.clamp(l, h.max(l)))),
                        _ => Err(err("clamp", "arguments must be numbers")),
                    }
                }
            }
        },
    )?;
    ctx.add_function(
        "min",
        |a: Value, b: Value| -> Result<Value, ExecutionError> { pick("min", a, b, true) },
    )?;
    ctx.add_function(
        "max",
        |a: Value, b: Value| -> Result<Value, ExecutionError> { pick("max", a, b, false) },
    )?;
    ctx.add_function("abs", |a: Value| -> Result<Value, ExecutionError> {
        match a {
            Value::Int(i) => i
                .checked_abs()
                .map(Value::Int)
                .ok_or_else(|| err("abs", "overflow")),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err(err("abs", "argument must be a number")),
        }
    })?;
    ctx.add_function("round", |a: f64| -> Result<i64, ExecutionError> {
        to_int("round", a.round())
    })?;
    ctx.add_function("floor", |a: f64| -> Result<i64, ExecutionError> {
        to_int("floor", a.floor())
    })?;
    ctx.add_function("ceil", |a: f64| -> Result<i64, ExecutionError> {
        to_int("ceil", a.ceil())
    })?;
    ctx.add_function("trunc", |a: f64| -> Result<i64, ExecutionError> {
        to_int("trunc", a.trunc())
    })?;
    ctx.add_function("pow", |a: f64, b: f64| a.powf(b))?;
    ctx.add_function("sqrt", |a: f64| a.sqrt())?;
    ctx.add_function("log", |a: f64| a.ln())?;
    ctx.add_function("log10", |a: f64| a.log10())?;
    ctx.add_function("exp", |a: f64| a.exp())?;
    byte_fn!(ctx, "be16", 2, false, false);
    byte_fn!(ctx, "le16", 2, false, true);
    byte_fn!(ctx, "be24", 3, false, false);
    byte_fn!(ctx, "le24", 3, false, true);
    byte_fn!(ctx, "be32", 4, false, false);
    byte_fn!(ctx, "le32", 4, false, true);
    byte_fn!(ctx, "be16s", 2, true, false);
    byte_fn!(ctx, "le16s", 2, true, true);
    byte_fn!(ctx, "be24s", 3, true, false);
    byte_fn!(ctx, "le24s", 3, true, true);
    byte_fn!(ctx, "be32s", 4, true, false);
    byte_fn!(ctx, "le32s", 4, true, true);
    ctx.add_function(
        "bytes_of",
        |l: Arc<Vec<Value>>| -> Result<Value, ExecutionError> {
            let mut out = Vec::with_capacity(l.len());
            for v in l.iter() {
                match v {
                    Value::Int(i) if (0..=255).contains(i) => out.push(*i as u8),
                    Value::UInt(u) if *u <= 255 => out.push(*u as u8),
                    other => {
                        return Err(err("bytes_of", format!("{other:?} is not a byte (0..255)")));
                    }
                }
            }
            Ok(Value::Bytes(Arc::new(out)))
        },
    )?;
    ctx.add_function(
        "crc",
        |name: Arc<String>, b: Arc<Vec<u8>>| -> Result<i64, ExecutionError> {
            crate::checksum::compute(&name, &b)
                .map(|v| v as i64)
                .ok_or_else(|| err("crc", format!("unknown checksum `{name}`")))
        },
    )?;
    Ok(())
}

fn to_int(f: &str, x: f64) -> Result<i64, ExecutionError> {
    if x.is_finite() && x >= i64::MIN as f64 && x <= i64::MAX as f64 {
        Ok(x as i64)
    } else {
        Err(err(f, format!("{x} does not fit in an int")))
    }
}

fn pick(f: &str, a: Value, b: Value, smaller: bool) -> Result<Value, ExecutionError> {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            Ok(Value::Int(if smaller { *x.min(y) } else { *x.max(y) }))
        }
        _ => match (num(&a), num(&b)) {
            (Some(x), Some(y)) => Ok(Value::Float(if smaller { x.min(y) } else { x.max(y) })),
            _ => Err(err(f, "arguments must be numbers")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(src: &str) -> Result<Value, ExecutionError> {
        Program::compile(src).expect("compiles").execute(&context())
    }

    #[test]
    fn library_registers_without_conflicts() {
        let _ = context();
    }

    #[test]
    fn bit_operations() {
        assert_eq!(eval("shl(1, 4)").unwrap(), Value::Int(16));
        assert_eq!(eval("shr(-16, 2)").unwrap(), Value::Int(-4));
        assert_eq!(eval("ushr(-1, 60)").unwrap(), Value::Int(15));
        assert_eq!(
            eval("bor(band(0xF0, 0x3C), bxor(1, 3))").unwrap(),
            Value::Int(0x32)
        );
        assert_eq!(eval("field(0x0220, 11, 10)").unwrap(), Value::Int(0));
        assert_eq!(eval("field(0x0C00, 11, 10)").unwrap(), Value::Int(3));
        assert_eq!(eval("bit(8, 3)").unwrap(), Value::Bool(true));
        assert_eq!(eval("sign_extend(0xFF80, 16)").unwrap(), Value::Int(-128));
        assert!(eval("shl(1, 64)").is_err());
        assert!(
            eval("shl(0x4000000000000000, 2)").is_err(),
            "overflow is an error, not wrap"
        );
    }

    #[test]
    fn byte_decoding() {
        assert_eq!(eval("le16(b'\\x34\\x12', 0)").unwrap(), Value::Int(0x1234));
        assert_eq!(eval("be16s(b'\\xff\\x80', 0)").unwrap(), Value::Int(-128));
        assert_eq!(
            eval("le24(b'\\x01\\x02\\x03', 0)").unwrap(),
            Value::Int(0x030201)
        );
        assert!(eval("le32(b'\\x01', 0)").is_err());
        assert_eq!(
            eval("bytes_of([1, 2, 255])").unwrap(),
            Value::Bytes(Arc::new(vec![1, 2, 255]))
        );
        assert!(eval("bytes_of([256])").is_err());
    }

    #[test]
    fn numeric_helpers() {
        assert_eq!(eval("clamp(500, 0, 100)").unwrap(), Value::Int(100));
        assert_eq!(eval("clamp(-1.5, 0.0, 1.0)").unwrap(), Value::Float(0.0));
        assert_eq!(eval("round(2.5)").unwrap(), Value::Int(3));
        assert_eq!(eval("max(2, 7)").unwrap(), Value::Int(7));
        assert_eq!(eval("abs(-3)").unwrap(), Value::Int(3));
    }

    #[test]
    fn crc8_smbus_check_value() {
        // CRC RevEng check value for "123456789".
        assert_eq!(
            eval("crc('crc8-smbus', b'123456789')").unwrap(),
            Value::Int(0xF4)
        );
    }

    #[test]
    fn check_extracts_references() {
        let c = check("double(sign_extend(raw, 16)) * vars.scale + instance.offset").unwrap();
        assert!(
            c.roots.contains("raw") && c.roots.contains("vars") && c.roots.contains("instance")
        );
        assert!(c.selections.contains("vars.scale"));
        assert!(c.functions.contains("sign_extend") && c.functions.contains("double"));
        let c = check("[1, 2].all(x, x > lim)").unwrap();
        assert!(
            c.roots.contains("lim") && !c.roots.contains("x"),
            "{:?}",
            c.roots
        );
    }

    #[test]
    fn check_rejects_bitwise_operators_with_advice() {
        match check("a | b") {
            Err(ExprError::Syntax(m)) => assert!(m.contains("bor"), "{m}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn check_rejects_unknown_functions() {
        assert_eq!(
            check("sin(x)").unwrap_err(),
            ExprError::UnknownFunction("sin".into())
        );
    }

    #[test]
    fn overflow_is_an_error() {
        assert!(eval("9223372036854775807 + 1").is_err());
    }

    #[test]
    fn bme280_temperature_compensation_matches_datasheet_example() {
        // Bosch BME280 datasheet worked example values (dig_T1=27504, dig_T2=26435, dig_T3=-1000,
        // adc_T=519888) → T = 2508 (25.08 °C).
        let mut ctx = context();
        ctx.add_variable_from_value("adc_t", Value::Int(519_888));
        ctx.add_variable_from_value("t1", Value::Int(27_504));
        ctx.add_variable_from_value("t2", Value::Int(26_435));
        ctx.add_variable_from_value("t3", Value::Int(-1_000));
        let tv1 = Program::compile("shr((shr(adc_t, 3) - shl(t1, 1)) * t2, 11)")
            .unwrap()
            .execute(&ctx)
            .unwrap();
        let tv2 =
            Program::compile("shr(shr((shr(adc_t, 4) - t1) * (shr(adc_t, 4) - t1), 12) * t3, 14)")
                .unwrap()
                .execute(&ctx)
                .unwrap();
        let (Value::Int(a), Value::Int(b)) = (tv1, tv2) else {
            panic!()
        };
        assert_eq!(((a + b) * 5 + 128) >> 8, 2508);
    }
}
