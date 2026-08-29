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
