![PraBorrow Banner](banner.png)

# PraBorrow

**High-Performance Distributed Systems Framework**

PraBorrow enforces "Memory safety with sovereign integrity" by combining strict ownership semantics, procedural invariant checking, and zero-copy logistics into a unified framework.

## Architecture

The framework is modularized into specialized crates:

- **`praborrow-core`**: Implements `Sovereign<T>` for Domestic/Exiled state tracking.
- **`praborrow-defense`**: Provides `#[derive(Constitution)]` for explicit invariant enforcement ("Garuda" system).
- **`praborrow-logistics`**: Manages `RawResource` refinement ("Hilirisasi" pipeline) for zero-copy operations.
- **`praborrow-diplomacy`**: Handles FFI interop.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
praborrow = "0.2.0"
```

## Quick Start

```rust
use praborrow::core::Sovereign;
use praborrow::defense::Constitution;
use praborrow::logistics::RawResource;

#[derive(Constitution)]
struct FiscalData {
    #[invariant("self.value >= 0")]
    value: i32,
}

fn main() {
    // 1. Establish Sovereign Data
    let data = Sovereign::new(FiscalData { value: 100 });
    
    // 2. Enforce Constitution
    data.enforce_law();
    
    // 3. Annex (Move to remote)
    data.annex().expect("Annexation failed");
    
    // 4. Access Violation Check
    // println!("{}", data.value); // Panics: SOVEREIGNTY VIOLATION
}
```

## License

MIT
