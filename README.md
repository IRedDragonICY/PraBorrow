![PraBorrow Banner](banner.png)

# PraBorrow

**High-Performance Distributed Systems Framework**

[![Crates.io](https://img.shields.io/crates/v/praborrow.svg)](https://crates.io/crates/praborrow)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

PraBorrow enforces "Memory safety with sovereign integrity" by combining strict ownership semantics, procedural invariant checking, and zero-copy logistics into a unified framework.

## What's New in v0.5.0

- **Non-Panicking API**: `try_get()` and `try_get_mut()` for graceful error handling
- **LRU Prover Cache**: Bounded cache (10k entries) prevents memory leaks in long-running nodes
- **Raft Storage Abstraction**: Swappable storage backends via `RaftStorage` trait
- **Modernized Macros**: Direct expression parsing with `syn::Expr` (compile-time validation)
- **UDP Supervisor**: Auto-restart on network thread panics
- **RFC: Deadlock Detection**: Wait-For Graph design for distributed deadlock detection

## Architecture

The framework is modularized into specialized crates:

| Crate | Description |
|-------|-------------|
| **`praborrow-core`** | `Sovereign<T>` for Domestic/Exiled state tracking with `try_get()`/`try_get_mut()` |
| **`praborrow-defense`** | `#[derive(Constitution)]` for invariant enforcement ("Garuda" system) |
| **`praborrow-logistics`** | `RawResource` zero-copy buffer with safe `as_slice()` accessor |
| **`praborrow-diplomacy`** | FFI interop for foreign systems |
| **`praborrow-lease`** | Raft consensus with `RaftStorage` abstraction |
| **`praborrow-prover`** | Garuda Proof System (SMT verification with LRU cache) |
| **`praborrow-macros`** | Proc-macro crate for `#[derive(Constitution)]` |
| **`praborrow-sidl`** | Schema IDL for distributed protocols |

## Installation

```toml
[dependencies]
praborrow = "0.5.0"
```

## Quick Start

```rust
use praborrow::core::{Sovereign, SovereigntyError};
use praborrow::defense::Constitution;
use praborrow::core::CheckProtocol;

#[derive(Constitution)]
struct FiscalData {
    #[invariant(self.value >= 0)]
    value: i32,
}

fn main() -> Result<(), SovereigntyError> {
    // 1. Establish Sovereign Data
    let data = Sovereign::new(FiscalData { value: 100 });
    
    // 2. Enforce Constitution
    data.enforce_law();
    
    // 3. Safe access with error handling
    let value = data.try_get()?;
    println!("Value: {}", value.value);
    
    // 4. Annex (Move to remote)
    data.annex().expect("Annexation failed");
    
    // 5. Graceful error instead of panic
    match data.try_get() {
        Ok(_) => unreachable!(),
        Err(SovereigntyError::ForeignJurisdiction) => {
            println!("Resource is exiled - handle gracefully");
        }
    }
    
    Ok(())
}
```

## Safety Features

- **Zero Panics in Library Code**: All critical paths return `Result` types
- **SAFETY Comments**: Every `unsafe` block is documented
- **Strictly Typed Errors**: No string-based errors
- **Bounded Caches**: LRU strategy prevents unbounded memory growth

## License

MIT

