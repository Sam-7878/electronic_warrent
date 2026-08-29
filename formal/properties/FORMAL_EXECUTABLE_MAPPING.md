# Formal-to-executable mapping

This mapping defines the evidence boundary between the TLC model and Rust. The
model is an abstraction: it proves the listed properties for its finite bounds;
the trace checker demonstrates that representative executable states conform to
that abstraction. Neither result is described as a proof of the entire binary.

| TLA+ variable | Rust source | Serialization rule |
|---|---|---|
| `warrant_state` | `WarrantRecord.state` | Rust variant name |
| `credentials_verified` | lifecycle projection | true after credential verification |
| `authorized` | lifecycle projection | true at and after authorization |
| `domain_valid` | validated `MachinePolicyObject.domain_binding` | true only after validation |
| `nonce_used` | authorization/nonce registry result | true at and after authorization |
| `activation_count` | `WarrantRecord.position` | 0 or 1 |
| `reserved` | `FreezePosition.reserved_amount` | exact integer |
| `executed` | `FreezePosition.executed_amount` | exact integer |
| `released` | `FreezePosition.released_amount` | exact integer |
| `now` | operation timestamp | seconds in the bounded trace epoch |
| `adapter_committed` | published position/receipt | true after adapter commit |
| `audit_written` | `AuditChain` append outcome | true only after durable append |
| `last_action` | operation instrumentation | TLA+ action name |

`domain_electronic_warrant::FormalTraceState` is the single schema. The
`formal_traces` Rust example emits 11 JSONL traces, and
`evaluation/check_trace_conformance.py` checks transitions and SAFE-001--009.
SAFE-010 remains a static dependency-boundary property checked by the
architecture suite because TLA+ cannot inspect the Cargo dependency graph.

Run:

```sh
cargo run --release -p domain-electronic-warrant --example formal_traces -- formal/traces/rust
python3 evaluation/check_trace_conformance.py
```
