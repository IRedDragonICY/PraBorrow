/// A type that can safely cross the FFI boundary using Sovereign IDL (SIDL).
///
/// Types implementing this trait are guaranteed to have a stable memory layout
/// defined by the protocol, not rustc.
pub trait Diplomat {
    /// The stable Type ID (UUID).
    const TYPE_ID: u128;
}

/// Represents a remote interface (RPC Stub).
pub trait ForeignInterface {
    type Protocol;
}
