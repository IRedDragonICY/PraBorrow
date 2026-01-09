// PraBorrow Facade
// Re-exporting all modules.

pub use praborrow_core as core;
pub use praborrow_defense as defense;
pub use praborrow_logistics as logistics;
pub use praborrow_diplomacy as diplomacy;

/// The manifesto of the framework.
pub fn manifesto() {
    println!("PraBorrow: Memory safety with sovereign integrity.");
}
