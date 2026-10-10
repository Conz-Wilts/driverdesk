//! Fuzz `dd_spec::validate`: it must never panic or hang on any input (P0.T8).
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = dd_spec::validate(s);
    }
});
