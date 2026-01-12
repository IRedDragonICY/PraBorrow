//! `PraBorrow` - A distributed systems framework for Rust.
//!
//! Re-exports all sub-crates for convenient access:
//! - `core`: Distributed ownership primitives (`Sovereign<T>`)
//! - `defense`: Invariant verification macros
//! - `logistics`: Zero-copy buffer abstraction
//! - `diplomacy`: FFI bindings (requires `diplomacy` feature)
//! - `lease`: Raft/Paxos consensus
//! - `sidl`: Stable IDL generation (requires `sidl` feature)
//! - `macros`: Additional procedural macros
//! - `prover`: SMT-based formal verification (requires `prover` feature)
//!
//! # Feature Flags
//!
//! - `default`: Enables `std` and `full` features
//! - `full`: Enables all optional dependencies (`diplomacy`, `prover`, `sidl`)
//! - `std`: Enables standard library support
//! - `diplomacy`: Enables FFI bindings for foreign systems
//! - `prover`: Enables SMT-based formal verification
//! - `sidl`: Enables Stable IDL generation

#![deny(clippy::all)]
#![warn(clippy::pedantic)]

// Core dependencies - always available
#[doc(inline)]
pub use praborrow_core as core;

#[doc(inline)]
pub use praborrow_defense as defense;

#[doc(inline)]
pub use praborrow_lease as lease;

#[doc(inline)]
pub use praborrow_logistics as logistics;

#[doc(inline)]
pub use praborrow_macros as macros;

// Optional dependencies - feature-gated
#[cfg(feature = "diplomacy")]
#[doc(inline)]
pub use praborrow_diplomacy as diplomacy;

#[cfg(feature = "prover")]
#[doc(inline)]
pub use praborrow_prover as prover;

#[cfg(feature = "sidl")]
#[doc(inline)]
pub use praborrow_sidl as sidl;

pub mod error;
pub use error::PraBorrowError;

#[cfg(feature = "std")]
pub mod telemetry;

/// Common imports for quick access to `PraBorrow` functionality.
///
/// # Usage
///
/// ```rust,ignore
/// use praborrow::prelude::*;
/// ```
pub mod prelude {
    pub use praborrow_core::{CheckProtocol, Sovereign};
    pub use praborrow_defense::Constitution;
    #[cfg(feature = "prover")]
    pub use praborrow_prover::VerifiableSovereign;
    pub use crate::PraBorrowError;
}
