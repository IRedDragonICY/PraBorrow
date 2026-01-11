use thiserror::Error;

/// Unified error type for the `PraBorrow` framework.
///
/// Aggregates errors from various sub-crates into a single type for
/// application-level error handling.
#[derive(Error, Debug)]
pub enum PraBorrowError {
    /// Error related to lease consensus or management.
    #[error("Lease consensus error: {0}")]
    Lease(#[from] praborrow_lease::ConsensusError),

    /// Error related to network operations.
    #[error("Network error: {0}")]
    Network(#[from] praborrow_lease::NetworkError),

    /// Error related to formal verification proof.
    #[cfg(feature = "prover")]
    #[error("Formal verification error: {0}")]
    Proof(#[from] praborrow_prover::ProofError),

    /// Error related to distributed ownership enforcement.
    #[error("Sovereignty error: {0}")]
    Sovereignty(#[from] praborrow_core::SovereigntyError),

    /// Error related to constitutional invariants.
    #[error("Constitution error: {0}")]
    Constitution(#[from] praborrow_core::ConstitutionError),

    /// Standard IO error.
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}
