//! Render `dd_spec::validate` output for a file (development aid).
fn main() {
    let path = std::env::args().nth(1).expect("usage: dbg <file>");
    let src = std::fs::read_to_string(&path).expect("readable file");
    let r = dd_spec::validate(&src);
    print!(
        "{}",
        dd_spec::diag::render_plain(&r.diagnostics, &path, &src)
    );
}
