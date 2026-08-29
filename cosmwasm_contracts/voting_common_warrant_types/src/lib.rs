pub mod crypto;
pub mod error;
pub mod types;

pub use crypto::{VotingNullifierLifecycle, VotingNullifierManager};
pub use types::NullifierState;

// Re-export hete-core domain-agnostic protocol interfaces for use by all voting contracts.
// This makes the POA (Protocol Oriented Architecture) layer available without each
// contract needing to directly depend on hete-core.
pub use hete_core::{
    compute_gov_hash,
    derive_nullifier_hash,
    // Core Actor-Asset-Context triplet
    ActorDescriptor,
    AssetStateEngine,
    // Atomic engines
    AtomicStateEngine,
    AtomicTriplet,
    // Context validation
    ContextValidator,
    EngineError,
    FraudWriteOffProtocol,
    // Shared crypto utilities
    IdentityCredential,
    // Generic actor trust lifecycle. Voting's ballot-nullifier lifecycle is
    // exported separately to avoid mixing incompatible state meanings.
    NullifierLifecycle,
    NullifierState as TrustState,
    // OLAP event streaming
    OlapEventStreamer,
    QueryContext,
    ReconciliationEngine,
    // Error hierarchy
    SecurityError,
    // State proof
    StateProof,
    // POA protocol traits (new)
    TransactionProtocol,
    ValidationError,
};
