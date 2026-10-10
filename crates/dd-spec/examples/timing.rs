//! Print validation time per example (P0 exit criterion 5: < 50 ms each, release build).
fn main() {
    let dir = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../devices/examples"
    ));
    let _ = dd_spec::validate("spec: driverdesk/0.1\n"); // compile schemas once
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .expect("examples")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    files.sort();
    for p in files {
        let src = std::fs::read_to_string(&p).expect("readable");
        let t = std::time::Instant::now();
        let n = 20;
        for _ in 0..n {
            let _ = dd_spec::validate(&src);
        }
        println!(
            "{:24} {:>8.2} ms",
            p.file_name().unwrap().to_string_lossy(),
            t.elapsed().as_secs_f64() * 1000.0 / f64::from(n)
        );
    }
}
