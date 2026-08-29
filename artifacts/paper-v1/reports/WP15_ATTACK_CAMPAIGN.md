# WP15 Security Attack Campaign

## Matrix

- Attack types: 49
- Runs: 30
- Attempts recorded per attack/run: 1,000
- Defense-relevant classified attempts: 1,320,000
- Expected non-guarantee observations: 150,000
- Workspace implementation preflight: all 99 tests passed

Defense-relevant scenario aggregation reported ASR 0 and BRR 1. Public commit
observation, ordering, reveal delay, censorship, and metadata correlation were
classified separately as expected non-guarantees and excluded from defense ASR.
Wilson confidence bounds are stored on every aggregate row.

## 중요한 방법론 제한

The repeated campaign rows are deterministic, test-gated scenario evaluations;
they are not 1.32 million independent invocations of a deployed production
service. Exact credential, replay, policy, state/Adapter, and agent-boundary
behaviors are exercised by the Rust workspace tests, but the aggregation layer
does not replay every Rust mutation for every recorded attempt. Therefore the
publication claim is `PARTIALLY_SUPPORTED`, and the numbers must not be worded
as an empirical production compromise rate.
