use crate::raw::RawResource;
use praborrow_core::Sovereign;

/// The "Hilirisasi" (Refinement) Pipeline.
///
/// Defines how Raw Resources (upstream) are refined into Sovereign Products (downstream).
pub trait Hilirisasi {
    type Raw;
    type Product;

    fn refine(raw: RawResource<Self::Raw>) -> Sovereign<Self::Product>;
}
