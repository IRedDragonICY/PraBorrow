use std::time::Duration;

/// Represents a standardized Lease for a resource.
///
/// A Lease is a contract that guarantees exclusive access (or shared, depending on semantics)
/// for a specific duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lease<T> {
    pub lease_id: u128,
    pub holder: u128, // explicit PeerId type later?
    pub duration: Duration,
    pub _marker: std::marker::PhantomData<T>,
}

/// The fundamental law of the Distributed Borrow Checker.
///
/// This trait allows a local resource to be "hired" (leased) to a remote peer.
pub trait DistributedBorrow {
    type Target;

    /// Attempt to lease the resource to a peer.
    ///
    /// # Arguments
    /// * `peer_id` - The ID of the requesting peer.
    /// * `duration` - Requested lease duration.
    ///
    /// # Returns
    /// * `Ok(Lease)` if the resource is free and successfully leased.
    /// * `Err(LeaseError)` if already leased or other policy failure.
    fn try_hire(&self, peer_id: u128, duration: Duration) -> Result<Lease<Self::Target>, LeaseError>;
}

#[derive(Debug)]
pub enum LeaseError {
    AlreadyLeased,
    ResourceLocked,
    SystemFailure,
}
