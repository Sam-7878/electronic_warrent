# HETE Enforcement Adapter Contract

An `EnforcementAdapter` exposes `manifest`, `inspect`, `prepare`, `commit`, and `rollback`.

`prepare` must be side-effect free. `commit` checks the prepared snapshot version and either publishes the complete candidate or no change. Unsupported resource/action/capability combinations fail closed. An adapter claiming amount-bounded enforcement must expose an authoritative balance.

The reference simulated adapter maintains balances and per-warrant reservations. Available transfer value saturates at zero:

```text
balance - active_reserved_amount - pending_execution_amount
```

Reservation totals may exceed current balance, but execution cannot exceed either the active reservation or current balance. Expired reservations are excluded. The dry-run adapter commits only to an isolated clone and returns a receipt marked `dry_run=true`.

This contract provides no atomicity across an external system that does not implement the advertised prepare/commit semantics. Capability metadata is an assertion that must be independently certified for production adapters.
