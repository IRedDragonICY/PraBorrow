pub use praborrow_macros::Constitution;

/// The Constitution Trait.
///
/// Types implementing this trait have invariants that are enforced as "Law".
/// In Year 3, this enforcement moves from runtime panic to compile-time proof.
pub trait Constitution {
    /// Checks all invariants and panics if any are violated.
    fn enforce_law(&self);
}
