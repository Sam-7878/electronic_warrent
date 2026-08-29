# POA RiskEvidence Quarantine Extension

Version: 1.0.0

The key words **MUST**, **MUST NOT**, **SHOULD**, and **MAY** are normative.

## Scope

RiskEvidence is an optional POA policy extension for routing a domain-neutral Risk failure to Reject or Quarantine. A conforming POA implementation MUST NOT require this extension for ordinary Commit, Reject, Quarantine, or Abort processing. Direct Quarantine decisions defined outside this extension remain valid.

## Evidence

A RiskEvidence value MUST contain:

- `category`: one of `replay_attack`, `geofence_violation`, `authentication_failure`, `policy_escalation`, `fraud_signal`, or `anomaly_signal`;
- `severity`: integer basis points in `0..=10000`;
- `confidence`: integer basis points in `0..=10000`;
- `occurrences`: integer of at least one;
- `source`: one of `replay_window`, `geofence_engine`, `authentication_verifier`, `policy_validator`, `pattern_detector`, or `external_oracle`;
- `observed_at_ms`: unsigned Unix epoch milliseconds;
- `correlation_id`: non-empty UTF-8, no leading/trailing whitespace or control characters, and no more than 128 bytes.

Unknown object fields and unknown category/source values MUST be rejected. Implementations MUST NOT use floating-point values for basis points. `observed_at_ms` is provenance only and MUST NOT affect the threshold decision.

## Policy

The protocol's top-level `risk_evidence` property MAY be absent. When present, it MUST contain exactly:

- `enabled`: boolean;
- `minimum_occurrences`: integer `1..=1000000`;
- `minimum_severity_bps`: integer `0..=10000`;
- `minimum_confidence_bps`: integer `0..=10000`;
- `aggregation`: `all_thresholds` or `any_threshold`.

An absent or disabled policy MUST route a Risk failure to Reject. It MUST NOT silently route Risk to Commit or Quarantine.

The effective policy, including these five fields, MUST be included in the deterministic policy digest. Runtime evidence MUST NOT be included in the policy digest. A specification with the property absent MUST retain the canonical representation it had before this extension.

If an inherited child omits the extension, it MUST inherit the parent's value. Enabling, increasing a threshold, and changing `any_threshold` to `all_thresholds` are restrictive changes. Disabling, reducing a threshold, and changing `all_thresholds` to `any_threshold` are privilege expansions and MUST use the protocol's explicit approved-expansion mechanism.

## Evaluation

The evaluator MUST be pure and deterministic for identical evidence and policy. It MUST compare occurrences, severity, and confidence using inclusive `>=` comparisons.

- `all_thresholds` succeeds only if all comparisons pass.
- `any_threshold` succeeds if one or more comparisons pass.
- Disabled evaluation returns `extension_disabled`.
- Failed enabled evaluation returns `insufficient` and the failed threshold names.
- Successful evaluation returns `quarantine`, the matched threshold names, and `score_bps` equal to `min(severity, confidence)`.

Threshold names MUST be reported in the stable order occurrences, severity, confidence.

## Failure routing and state

A Policy failure MUST route to Reject. An Internal failure MUST route to Abort. A sufficient Risk failure MUST route to Quarantine. An insufficient, absent-policy, or disabled-policy Risk failure MUST route to Reject.

Risk MAY arise during authorize, validate, candidate mutation, reconciliation, or commit. Implementations MUST keep a candidate uncommitted until authorization, validation, mutation, and reconciliation have succeeded. A Risk failure before commit MUST NOT alter committed domain state. Commit implementations SHOULD be atomic because an extension evaluator cannot roll back externally published effects.

## Audit

For every Risk failure, the caller MUST be able to associate the terminal outcome with the original typed evidence, evaluator decision, stable reason code, and effective policy digest. A Quarantine audit MUST preserve category, severity, confidence, occurrences, source, observation time, and correlation ID. An insufficient or disabled Risk audit MUST preserve the same evidence and its decision.

Audit records MUST NOT include the request payload, candidate object, or domain state solely to support this extension. The extension MAY coexist with an independently computed payload hash.

## Compatibility

Existing protocol files without `risk_evidence` MUST remain schema-valid. Existing non-Risk execution behavior MUST remain unchanged. Existing direct Quarantine reasons MUST NOT be rewritten as fabricated RiskEvidence.
