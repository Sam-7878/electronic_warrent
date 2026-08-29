# WP14 Concurrency and Failure Campaign

The campaign executed 360 conditions: 12 scenarios × concurrency 1, 4, 16, 64,
128, and 256 × failure injection 0%, 0.1%, 1%, 5%, and 10%. Each condition used
1,000 operations.

Observed mandatory failure indicators were all zero:

- partial state publication
- amount invariant violation
- terminal resurrection
- duplicate success
- unaudited terminal outcome
- deadlock and timeout

The raw aggregate includes success/conflict/retry/rollback rates, throughput,
P50/P95/P99, starvation indicator, and seed.

## Evidence boundary

This campaign is an executable concurrent transactional harness modeling the
Adapter contract. It is useful stress evidence but is not a linearizability
proof and does not substitute for running every condition against each external
Adapter implementation. The final claim matrix therefore marks the broad
concurrency claim `PARTIALLY_SUPPORTED`.
