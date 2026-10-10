//! Every invalid fixture produces its expected code (header `# expect: DD-xxxx`) and a stable
//! rendered message (insta snapshot). Every example validates without errors.

use std::path::Path;

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn invalid_fixtures_report_expected_code() {
    insta::glob!("../../../spec/fixtures/invalid", "*.yaml", |path| {
        let src = std::fs::read_to_string(path).expect("fixture readable");
        let expect = src
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("# expect: "))
            .expect("fixture header `# expect: DD-xxxx`")
            .trim()
            .to_string();
        let report = dd_spec::validate(&src);
        let codes: Vec<&str> = report.diagnostics.iter().map(|d| d.code).collect();
        assert!(
            codes.contains(&expect.as_str()),
            "{}: expected {expect}, got {codes:?}",
            path.display()
        );
        let name = path.file_name().expect("file name").to_string_lossy();
        insta::assert_snapshot!(dd_spec::diag::render_plain(
            &report.diagnostics,
            &name,
            &src
        ));
    });
}

#[test]
fn fixtures_are_numerous_enough() {
    let n = std::fs::read_dir(root().join("spec/fixtures/invalid"))
        .expect("dir")
        .count();
    assert!(n >= 40, "P0 needs at least 40 invalid fixtures, found {n}");
}

#[test]
fn examples_have_no_errors() {
    let dir = root().join("devices/examples");
    let mut n = 0;
    for entry in std::fs::read_dir(&dir).expect("examples dir") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|e| e == "yaml") {
            let src = std::fs::read_to_string(&path).expect("readable");
            let report = dd_spec::validate(&src);
            assert!(
                report.is_valid(),
                "{} is invalid:\n{}",
                path.display(),
                dd_spec::diag::render_plain(&report.diagnostics, "example", &src)
            );
            n += 1;
        }
    }
    assert!(n >= 5, "need at least 5 examples, found {n}");
}

#[test]
fn validation_is_fast_on_examples() {
    let src = std::fs::read_to_string(root().join("devices/examples/bme280.yaml")).expect("bme280");
    let _ = dd_spec::validate(&src); // warm up the compiled schema
    let t = std::time::Instant::now();
    for _ in 0..10 {
        let _ = dd_spec::validate(&src);
    }
    let per = t.elapsed() / 10;
    // Exit criterion 5: < 50 ms per example (debug builds are slower; release is checked in CI).
    let limit = if cfg!(debug_assertions) { 500 } else { 50 };
    assert!(per.as_millis() < limit, "validate took {per:?} per run");
}

#[test]
fn corpus_validates_within_a_2_mib_stack() {
    // Guards stack growth (D-002): `dd` and `xtask` give validation 8 MiB; this keeps a 4x margin.
    let h = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for dir in [
                "devices/examples",
                "spec/fixtures/invalid",
                "crates/dd-spec/tests/regressions",
            ] {
                for e in std::fs::read_dir(root().join(dir)).expect("dir") {
                    let src = std::fs::read_to_string(e.expect("entry").path()).unwrap_or_default();
                    let _ = dd_spec::validate(&src);
                }
            }
        })
        .expect("spawn");
    assert!(h.join().is_ok(), "validation overflowed a 2 MiB stack");
}
