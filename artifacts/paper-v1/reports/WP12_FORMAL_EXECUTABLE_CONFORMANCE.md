# WP12 Formal–Executable Conformance

`FormalTraceState` serializes every TLA+ state variable from Rust. The release
example emitted 11 JSONL scenarios and 65 snapshots covering activation,
partial/full execution, release, expiry, revocation, rejection, quarantine,
abort, suspend/revoke, and duplicate/terminal stutter.

`evaluation/check_trace_conformance.py` checks schema equality, action/state
transitions, and SAFE-001--009. All 11 traces passed. Six deliberately mutated
traces (unauthorized, replay, amount, domain, atomicity, and audit) were rejected,
demonstrating checker sensitivity.

The exact field mapping and abstraction boundary are documented in
`formal/properties/FORMAL_EXECUTABLE_MAPPING.md`. This is representative trace
conformance, not exhaustive bisimulation.
