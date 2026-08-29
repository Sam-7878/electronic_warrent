# HETE Machine-Verifiable Policy Object

`hete-policy::MachinePolicyObject` is the domain-neutral authorization envelope. JSON inputs fail closed on unknown fields. A policy is usable only after semantic validation and comparison of `policy_digest` with the SHA-256 digest of its canonical payload (all fields except the digest itself).

Security assumptions: the caller supplies a deterministic clock, validates the relevant JSON Schema, and obtains policy bytes through a trusted ingress. Canonicalization does not establish legal validity. It only prevents key-order ambiguity and binds material fields.

Inheritance may narrow roles, threshold, actions, amount, and duration. `ensure_no_privilege_expansion` rejects new roles/actions, lower thresholds, larger amounts, or longer durations. Explicit expansion authorization is deliberately not implemented in this milestone; expansion fails closed.

Failure behavior uses stable policy reason families: schema invalid, digest mismatch, invalid time/authorization, canonicalization failure, and privilege expansion.
