# fuzz/

`cargo fuzz` targets (nightly only, outside the main workspace).

```
rustup toolchain install nightly
cargo install cargo-fuzz
cd fuzz && cargo +nightly fuzz run validate -- -max_total_time=600
```

Seed corpus: `devices/examples/` and `spec/fixtures/invalid/` (copy them into `fuzz/corpus/validate/` before a run).
