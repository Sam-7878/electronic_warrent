# WP17 External Transactional Adapter

`adapter-sqlite-asset` implements the unchanged `EnforcementAdapter` API using
SQLite with WAL mode, `synchronous=FULL`, and atomic transactions. `prepare` is
read-only; `commit` checks version/digest, applies the full candidate, records
the command/receipt, and commits once.

Seven tests cover bounded freeze, prepare/rollback side-effect freedom, stale
commit, duplicate command, execute/release conservation, expiry, and process
restart recovery. The Adapter has no dependency on the warrant or agent domain,
credential crate, or `poa-core`.

The comparison campaign used 30 runs × 1,000 operations per Adapter:

| Adapter | Mean total ns | Median ns | P95 ns | P99 ns |
|---|---:|---:|---:|---:|
| Simulated | 18,895 | 17,796 | 25,316 | 38,076 |
| SQLite | 42,061 | 38,341 | 60,549 | 112,621 |

These WSL2 timings are comparative calibration, not certified storage SLAs.
