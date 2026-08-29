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




# Electronic Warrant Domain Profile

The profile extends `MachinePolicyObject` with pseudonymous case/warrant references, jurisdiction, bounded asset scope, freeze and execution rules, configurable authority references, and review/hold rules.

The state machine rejects terminal-state resurrection. Activation follows the existing AACO stages:

1. authorize: verified multi-authority context and RiskEvidence gate;
2. validate: legal profile invariants, domain binding, and adapter capabilities;
3. mutate candidate: adapter `prepare`;
4. reconcile: amount and state conservation;
5. commit: optimistic-version adapter commit followed by domain publication.

Risk evidence reaching the configured threshold returns `Quarantine` before `prepare`, so no asset state is published. The reference applies only to HETE-aware regulated assets and does not decide whether a formally valid warrant is substantively justified.

Public commitments bind a policy digest and nonce but do not lock a target or provide secrecy. `LocalSealedIngress` is explicitly a simulation-only interface illustrating the external confidential transport boundary.




# Electronic Warrant Threat Model

## Protected assets and trust boundaries

Protected assets are authority keys, DID/trust data, policy and credential envelopes, off-chain target mappings, adapter state, audit evidence, revocation status, receipts, and raw evaluation data. Trust crosses authority clients, the HETE process, OS sandbox, resolver/registry, adapter, audit storage, and optional ledger infrastructure.

The requesting authority, legal reviewer, judicial issuer, executor, and auditor are separate configurable roles. A single compromised key is tolerated only when the configured threshold and mutual-exclusion rules still require independent approval.

## Attacker capabilities

The evaluated attacker may forge or mutate credentials, steal one key, replay within or across domains, provide stale identity state, reuse revoked credentials, enumerate target digests, downgrade policy, falsify adapter capability claims, race prepare/commit, duplicate execution, exploit expiry boundaries, suppress logs, submit malicious AI drafts, manipulate public ordering, or probe OS resources.

Mitigations are domain-separated signatures, nonce registry, deterministic time, threshold/sequential policy, fail-closed schemas, optimistic versions, amount conservation, RiskEvidence quarantine before commit, hash-chained minimized audit, and the existing OpenBSD startup/isolation path.

## Explicit non-guarantees

HETE does not guarantee universal freezing of permissionless or non-integrated assets, resistance to malicious consensus majority, substantive legal correctness, natural-language understanding, external resolver availability, cross-chain recovery, or seizure of assets controlled only by an uncompelled private key. Public commitment does not provide target secrecy, instant lock, private ordering, or consensus-level front-running prevention. Pseudonymous references reduce direct disclosure but retain linkability and off-chain re-identification risk.

AI agents may draft, validate, monitor, and request dry-runs. They may not issue credentials, sign for authorities, bypass thresholds, expand amount/duration, invoke adapters directly, release quarantine, or trigger real enforcement without human authorization.








# HETE Enforcement Adapter Contract

An `EnforcementAdapter` exposes `manifest`, `inspect`, `prepare`, `commit`, and `rollback`.

`prepare` must be side-effect free. `commit` checks the prepared snapshot version and either publishes the complete candidate or no change. Unsupported resource/action/capability combinations fail closed. An adapter claiming amount-bounded enforcement must expose an authoritative balance.

The reference simulated adapter maintains balances and per-warrant reservations. Available transfer value saturates at zero:

```text
balance - active_reserved_amount - pending_execution_amount
```

Reservation totals may exceed current balance, but execution cannot exceed either the active reservation or current balance. Expired reservations are excluded. The dry-run adapter commits only to an isolated clone and returns a receipt marked `dry_run=true`.

This contract provides no atomicity across an external system that does not implement the advertised prepare/commit semantics. Capability metadata is an assertion that must be independently certified for production adapters.







# HETE Authority Credential Profile

This profile is a deliberately limited Ed25519 credential envelope, not a complete W3C VC implementation.

- DID resolution and role status use a local deterministic registry by default.
- Credential issuer key activation/revocation and credential expiry are checked at the injected timestamp.
- Approval signatures cover the `HETE-EW-V1` domain-separated message assembled by the warrant domain.
- The message binds environment, policy digest, warrant, pseudonymous target, asset scope, maximum amount, validity, action, and nonce.
- Threshold, sequential order, and mutually exclusive roles are policy data; jurisdiction-specific institution names are fixtures, not enums in the core.
- A nonce is recorded only after all required approvals verify.

No raw credential is included in errors or audit records. External network DID resolution, JSON-LD processing, selective disclosure, and legal authority adjudication are outside this profile.






# HETE Machine-Verifiable Policy Object

`hete-policy::MachinePolicyObject` is the domain-neutral authorization envelope. JSON inputs fail closed on unknown fields. A policy is usable only after semantic validation and comparison of `policy_digest` with the SHA-256 digest of its canonical payload (all fields except the digest itself).

Security assumptions: the caller supplies a deterministic clock, validates the relevant JSON Schema, and obtains policy bytes through a trusted ingress. Canonicalization does not establish legal validity. It only prevents key-order ambiguity and binds material fields.

Inheritance may narrow roles, threshold, actions, amount, and duration. `ensure_no_privilege_expansion` rejects new roles/actions, lower thresholds, larger amounts, or longer durations. Explicit expansion authorization is deliberately not implemented in this milestone; expansion fails closed.

Failure behavior uses stable policy reason families: schema invalid, digest mismatch, invalid time/authorization, canonicalization failure, and privilege expansion.






# warrant_evaluation_manifest.json

{
  "build_profile": "dev",
  "commands": [
    [
      "/mnt/d/_Work/goat_bank/.venv/bin/python",
      "evaluation/check_architecture.py"
    ],
    [
      "/mnt/d/_Work/goat_bank/.venv/bin/python",
      "evaluation/check_warrant_invariants.py",
      "--skip-rust"
    ],
    [
      "/mnt/d/_Work/goat_bank/.venv/bin/python",
      "evaluation/experiments/functional_correctness.py",
      "--smoke",
      "--output",
      "evaluation/results/raw/functional.json"
    ],
    [
      "/mnt/d/_Work/goat_bank/.venv/bin/python",
      "evaluation/experiments/scale_benchmark.py",
      "--smoke",
      "--output",
      "evaluation/results/raw/warrant_scale.csv"
    ],
    [
      "/mnt/d/_Work/goat_bank/.venv/bin/python",
      "evaluation/experiments/attack_simulation.py",
      "--output",
      "evaluation/results/raw/warrant_attack.csv"
    ],
    [
      "/mnt/d/_Work/goat_bank/.venv/bin/python",
      "evaluation/experiments/privacy_surface_audit.py",
      "--output",
      "evaluation/results/processed/privacy_audit.json"
    ]
  ],
  "host": {
    "distribution": "Ubuntu",
    "os": "Ubuntu 24.04.4 LTS",
    "python": "3.12.13",
    "python_environment": "/mnt/d/_Work/goat_bank/.venv",
    "rustc": "1.96.0 (ac68faa20 2026-05-25)",
    "virtualization": "WSL2"
  },
  "mode": "smoke",
  "source_commit": "039e0aaee3be0bc5fe9d8ac1acd2b871005d8538",
  "status": "passed",
  "working_tree": "dirty-development-output"
}





