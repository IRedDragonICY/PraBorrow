![PraBorrow Banner](banner.png)

# PraBorrow

**High-Performance Distributed Systems Framework**

[![Crates.io](https://img.shields.io/crates/v/praborrow.svg)](https://crates.io/crates/praborrow)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

PraBorrow enforces "Memory safety with sovereign integrity" by combining strict ownership semantics, procedural invariant checking, and zero-copy logistics into a unified framework.

## What's New in v0.6.0

### Safety Overhaul
- **Zero Library Panics**: All critical paths now return `Result` types
- **Memory Leak Fix**: `RawResource` now properly deallocates via `Drop` implementation
- **Input Validation**: Empty buffers, zero-duration leases, and invalid addresses are caught early

### API Improvements  
- **Non-Panicking `enforce_law()`**: Returns `Result<(), String>` instead of panicking
- **New Sovereign Methods**: `new_exiled()`, `is_domestic()`, `is_exiled()`, `repatriate()`
- **Configurable Networks**: `NetworkConfig` for buffer size and timeouts

### Observability
- **Structured Logging**: Full `tracing` integration across all crates
- **Prover Warnings**: Stub backend now warns when Z3 is disabled

### Infrastructure
- **Injectable Storage**: `RaftNode::new()` accepts pluggable storage backends
- **UDP Safety**: Exponential backoff, read timeouts, packet size validation

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
praborrow = "0.6.0"
```

## Quick Start

```rust
use praborrow::core::{Sovereign, SovereigntyError, CheckProtocol};
use praborrow::defense::Constitution;

#[derive(Constitution)]
struct FiscalData {
    #[invariant(self.value >= 0)]
    value: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Establish Sovereign Data
    let data = Sovereign::new(FiscalData { value: 100 });
    
    // 2. Enforce Constitution (now returns Result!)
    data.enforce_law()?;
    
    // 3. Safe access with error handling
    let value = data.try_get()?;
    println!("Value: {}", value.value);
    
    // 4. Check state with helper methods
    assert!(data.is_domestic());
    
    // 5. Annex (Move to remote)
    data.annex()?; // Propagate errors, do not panic
    assert!(data.is_exiled());
    
    // 6. Graceful error instead of panic
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
- **Strictly Typed Errors**: No string-based errors in public API
- **Bounded Caches**: LRU strategy prevents unbounded memory growth
- **Memory Safety**: `RawResource` properly deallocates with `Drop`
- **Input Validation**: Invalid inputs are rejected early with clear errors

## License

MIT


