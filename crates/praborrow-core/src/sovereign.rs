use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// A unique identifier for a Lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeaseId(pub u128);

/// An Epoch counter for logical clocks / lease versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u64);

/// The internal state of the Sovereign resource.
///
/// We compress state into atomics where possible, or use a "State Cell" pattern.
/// For this high-perf architecture, we might ideally want a single AtomicU64 pointer
/// to a state struct if we want lock-free, but that requires more complex memory management (e.g. crossbeam-epoch).
///
/// For the "Foundation" step, we will use a dedicated internal state with a spin-lock or
/// standard Mutex if we strictly follow "No Arc<Mutex<T>>" for the *user* API, 
/// but internally we need *some* synchronization for the metadata.
/// 
/// However, the prompt asks for "LeaseId, Epoch, Timeout".
/// 
/// Let's implement a `State` enum that is swapped atomically if possible, 
/// or protected by a very thin internal lock (like a `parking_lot::RwLock`, 
/// but we want to avoid deps if "core" implies std-only).
///
/// Wait, standard `std::sync::Mutex` is allowed *internally* if it's not exposed as the API.
/// The user said "Do not suggest Arc<Mutex<T>>". `Sovereign<T>` itself is the replacement.
///
/// Let's use `std::sync::RwLock` for the *metadata* (State), while `T` is in `UnsafeCell`.
/// This is a common pattern for "smart pointers" that manage access.
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Jurisdiction {
    /// The resource is under local control.
    Domestic,
    /// The resource is leased to a remote entity.
    Foreign {
        holder: u128,
        lease_id: LeaseId,
        epoch: Epoch,
        expires_at: Instant,
    },
}

/// A wrapper that enforces ownership semantics across network boundaries.
///
/// # The Sovereign Promise
/// "Memory safety with sovereign integrity."
///
/// If `Jurisdiction` is `Foreign`, local access panics (Sovereignty Violation).
pub struct Sovereign<T> {
    /// The protected data.
    inner: UnsafeCell<T>,
    /// The "Law" (State Metadata).
    /// improved for concurrency over AtomicU8.
    meta: RwLock<Jurisdiction>,
}

impl<T> Sovereign<T> {
    /// Creates a new Sovereign resource under domestic jurisdiction.
    pub fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            meta: RwLock::new(Jurisdiction::Domestic),
        }
    }

    /// Checks jurisdiction and returns reference if Domestic.
    /// Panics if Foreign.
    fn ensure_domestic(&self) {
        let meta = self.meta.read().expect("Sovereignty metadata corrupted");
        match *meta {
            Jurisdiction::Domestic => {}, // OK
            Jurisdiction::Foreign { holder, expires_at, .. } => {
                if Instant::now() < expires_at {
                    panic!("SOVEREIGNTY VIOLATION: Resource leased to Peer {:?} until {:?}", holder, expires_at);
                } else {
                    // Lazy expiry handling? 
                    // Ideally we should reclaim here, but for now we panic or we need `&mut self` to reclaim?
                    // If we just read, we can't reclaim state.
                    // This is a design decision: simpler to strictly fail until explicit `reclaim()` is called?
                    // Or treat expired lease as Domestic?
                    // "Network timeouts as lifetime annotation" -> If timeout passed, it IS valid locally?
                    // Only if we trust the clock absolutely.
                    // Let's allow access if expired, conceptually.
                    // But we need to update state to Domestic to avoid readers seeing Foreign.
                    // We can't upgrade read lock to write lock easily.
                    // For safety in Step 1: PANIC if Foreign, even if technically expired, 
                    // requiring an explicit `reclaim()` step to clean up.
                    panic!("SOVEREIGNTY VIOLATION: Resource lease expired but not reclaimed.");
                }
            }
        }
    }
    
    /// Explicitly reclaim a resource after lease expiration.
    pub fn reclaim(&self) {
        let mut meta = self.meta.write().expect("Sovereignty metadata corrupted");
        match *meta {
            Jurisdiction::Domestic => {}, // Already domestic
            Jurisdiction::Foreign { expires_at, .. } => {
                if Instant::now() >= expires_at {
                    *meta = Jurisdiction::Domestic;
                } else {
                    // Cannot reclaim yet
                }
            }
        }
    }

    /// Grant a lease to a peer.
    ///
    /// # Returns
    /// `Ok(LeaseId)` if successful.
    pub fn grant_lease(&self, peer: u128, duration: Duration) -> Result<(LeaseId, Epoch), String> {
        let mut meta = self.meta.write().expect("Sovereignty metadata corrupted");
        
        match *meta {
            Jurisdiction::Domestic => {
                let lease_id = LeaseId(uuid_pseudo()); // Placeholder for UUID
                let epoch = Epoch(1); // Placeholder for clock
                let expires_at = Instant::now() + duration;
                
                *meta = Jurisdiction::Foreign {
                    holder: peer,
                    lease_id,
                    epoch,
                    expires_at,
                };
                
                Ok((lease_id, epoch))
            }
            Jurisdiction::Foreign { .. } => Err("Resource already leased".to_string()),
        }
    }
}

// Helper for unique ID (placeholder)
fn uuid_pseudo() -> u128 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNT: AtomicU64 = AtomicU64::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) as u128
}

impl<T> Deref for Sovereign<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ensure_domestic();
        // SAFETY: We checked jurisdiction. If Domestic, no remote writer exists (by contract).
        // If we are here, we have shared read access locally.
        unsafe { &*self.inner.get() }
    }
}

impl<T> DerefMut for Sovereign<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ensure_domestic();
        // SAFETY: We checked jurisdiction.
        unsafe { &mut *self.inner.get() }
    }
}


use crate::traits::{DistributedBorrow, Lease, LeaseError};

impl<T> DistributedBorrow for Sovereign<T> {
    type Target = T;

    fn try_hire(&self, peer_id: u128, duration: Duration) -> Result<Lease<Self::Target>, LeaseError> {
        match self.grant_lease(peer_id, duration) {
            Ok((lease_id, _epoch)) => Ok(Lease {
                lease_id: lease_id.0,
                holder: peer_id,
                duration,
                _marker: std::marker::PhantomData,
            }),
            Err(_) => Err(LeaseError::AlreadyLeased),
        }
    }
}


unsafe impl<T: Send> Send for Sovereign<T> {}
unsafe impl<T: Sync> Sync for Sovereign<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::DistributedBorrow;

    #[test]
    fn test_domestic_access() {
        let sov = Sovereign::new(42);
        assert_eq!(*sov, 42);
    }

    #[test]
    fn test_lease_grant() {
        let sov = Sovereign::new(100);
        let lease = sov.try_hire(1, Duration::from_secs(10));
        assert!(lease.is_ok());
        
        // Should panic now
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = *sov;
        }));
        assert!(result.is_err(), "Access should be denied when leased");
    }

    #[test]
    fn test_reclaim_after_expiry() {
        let sov = Sovereign::new(200);
        // Short lease for testing
        // Note: Real time testing is flaky, but we can rely on Instant passing
        let _ = sov.grant_lease(2, Duration::from_millis(1));
        
        std::thread::sleep(Duration::from_millis(10));
        
        // Should still panic strictly speaking before reclaim is implicitly or explicitly handled
        // Our implementation currently panics if Foreign, unless we call reclaim (?)
        // Let's check the logic: ensure_domestic calls read(). If Foreign, checks expiry.
        // If expired -> Panic "expired but not reclaimed".
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = *sov; 
        }));
        assert!(result.is_err(), "Access denied until reclaim");

        sov.reclaim();
        
        // Now should work
        assert_eq!(*sov, 200); 
    }
}
