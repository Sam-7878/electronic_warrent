# Reproduce the HETE electronic-warrant paper artifact

Pinned source: `12c9c8f9fa116809841eff01daa9aaf5fdede358`  
Tag: `v0.2.0-paper-rc1`

Use Ubuntu 24.04, Python 3.12, current stable Rust, and Java 21. TLC is
bootstrapped from the pinned checksum in `formal/tools/README.md`.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
sh formal/scripts/bootstrap_tlc.sh
sh formal/scripts/run_tlc.sh safety reproduce-safety 2
sh formal/scripts/run_tlc.sh liveness reproduce-liveness 2
python evaluation/check_trace_conformance.py
python evaluation/analysis/verify_raw_hashes.py
```

The included WSL2 performance results are calibration evidence. Run
`evaluation/run_full_benchmark.py` on a dedicated native publication host before
using performance values as final cross-system claims.
