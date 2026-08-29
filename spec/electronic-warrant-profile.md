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
