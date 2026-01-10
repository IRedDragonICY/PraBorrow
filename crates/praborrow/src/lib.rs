//! PraBorrow - A distributed systems framework for Rust.
//!
//! Re-exports all sub-crates for convenient access:
//! - `core`: Distributed ownership primitives (`Sovereign<T>`)
//! - `defense`: Invariant verification macros
//! - `logistics`: Zero-copy buffer abstraction
//! - `diplomacy`: FFI bindings
//! - `lease`: Raft/Paxos consensus
//! - `sidl`: Stable IDL generation
//! - `macros`: Additional procedural macros

pub use praborrow_core as core;
pub use praborrow_defense as defense;
pub use praborrow_diplomacy as diplomacy;
pub use praborrow_lease as lease;
pub use praborrow_logistics as logistics;
pub use praborrow_macros as macros;
pub use praborrow_sidl as sidl;
