# HETE Authority Credential Profile

This profile is a deliberately limited Ed25519 credential envelope, not a complete W3C VC implementation.

- DID resolution and role status use a local deterministic registry by default.
- Credential issuer key activation/revocation and credential expiry are checked at the injected timestamp.
- Approval signatures cover the `HETE-EW-V1` domain-separated message assembled by the warrant domain.
- The message binds environment, policy digest, warrant, pseudonymous target, asset scope, maximum amount, validity, action, and nonce.
- Threshold, sequential order, and mutually exclusive roles are policy data; jurisdiction-specific institution names are fixtures, not enums in the core.
- A nonce is recorded only after all required approvals verify.

No raw credential is included in errors or audit records. External network DID resolution, JSON-LD processing, selective disclosure, and legal authority adjudication are outside this profile.
