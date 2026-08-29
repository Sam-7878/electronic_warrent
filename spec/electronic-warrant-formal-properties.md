# Electronic Warrant Formal Properties

The bounded TLA+ model is in `formal/tla/ElectronicWarrant.tla`; `ElectronicWarrant.cfg` uses a small amount/time domain for exhaustive exploration when TLC is available.

| ID | Property | Executable evidence |
|---|---|---|
| SAFE-001 | unauthorized requests never activate | credential threshold tests and model invariant |
| SAFE-002 | nonce/domain pair is not reused | AUTH replay test |
| SAFE-003 | executed amount does not exceed maximum | Rust property test |
| SAFE-004 | executed plus released does not exceed reserved | 10,000-case property test |
| SAFE-005 | no new execution after expiry | adapter active-reservation rule and model |
| SAFE-006 | revoked state cannot reactivate | state-transition unit test |
| SAFE-007 | signatures are domain-bound | cross-domain test |
| SAFE-008 | prepared state is not partially published | failure-injection test |
| SAFE-009 | terminal outcomes produce minimized evidence | audit-chain implementation/tests |
| SAFE-010 | core contains no warrant symbols | ARCH-005/010 |

Liveness is conditional: a fair scheduler, available adapter, writable audit store, and valid unexpired credentials allow an execution request to terminate as Commit, Reject, Quarantine, or Abort. Expiration requires a scheduler tick; quarantine requires explicit human review. These environmental fairness assumptions are not proven by the Rust implementation.

`evaluation/check_formal_model.py` checks traceability only. It is not a substitute for TLC. A TLC result must be captured before claiming bounded model-check success.
