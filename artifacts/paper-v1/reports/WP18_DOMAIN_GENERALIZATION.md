# WP18 Domain Generalization

The new `domain-agent-delegation` crate demonstrates AI Agent Tool Delegation
using the existing `MachinePolicyObject`, `poa-core::TransitionDescriptor`, and
`EnforcementAdapter` contract.

Nine tests cover valid activation/invocation, wrong policy type, scope expansion,
human-confirmation bypass, call limit, revocation, expiry, and the domain-neutral
descriptor. The pinned aggregate hash of `crates/poa-core` remained
`43245924ae4d00baa50a3f60315074eea3596180e53548057bc4f39831b2501e`.

This supports a two-domain reuse demonstration. It does not prove that every
future administrative, legal, or financial domain can be represented without
new abstractions.
