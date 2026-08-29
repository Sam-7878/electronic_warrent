# WP13 Full Benchmark

## 완료된 campaign

- Baselines: B0--B6
- Independent runs: 30
- Operations: 1,000 per baseline per run
- Total raw rows: 210,000
- Build: optimized Rust `release`
- Provenance: source commit and host ID on every row
- Failure rate: 0 for all baselines

The raw CSVs and per-file hashes are in
`evaluation/results/raw/full_benchmark/`. Each row contains parse,
canonicalization, digest, identity, credential, authorization, validation,
risk, prepare, reconciliation, commit, audit, and total latency.

| Baseline | Mean total ns | Median ns | P95 ns | 95% CI of run mean |
|---|---:|---:|---:|---|
| B0 | 94,014 | 94,587 | 100,563 | 92,246–95,782 |
| B1 | 205,761 | 207,032 | 219,971 | 202,199–209,323 |
| B2 | 182,361 | 180,280 | 201,808 | 178,359–186,364 |
| B3 | 205,464 | 204,826 | 225,446 | 201,708–209,219 |
| B4 | 204,200 | 203,467 | 221,853 | 200,337–208,063 |
| B5 | 203,894 | 203,533 | 221,960 | 200,026–207,762 |
| B6 | 204,925 | 205,398 | 224,164 | 201,089–208,760 |

These numbers are generated from independent-run means by the analysis scripts;
the table is a readable copy, not the authoritative data source.

## 제한

The current host is WSL2. These results are calibration evidence and must not be
used as final native-host or cross-system performance claims. The automated
manual publication job is ready for a dedicated native Ubuntu host.
