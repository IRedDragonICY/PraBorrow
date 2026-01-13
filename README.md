![PraBorrow Banner](banner.png)

# PraBorrow

**High-Performance Distributed Systems Framework**

[![Crates.io](https://img.shields.io/crates/v/praborrow.svg)](https://crates.io/crates/praborrow)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
English | [Indonesia](./README_ID.md)

PraBorrow enforces "Memory safety with sovereign integrity" by combining strict ownership semantics, procedural invariant checking, and zero-copy logistics into a unified framework.

## What's New in v0.8.0

### Ecosystem & Logistics
- **Garuda Dashboard**: Integrated Deadlock Detection (`WaitForGraph`) in `praborrow-lease`.
- **Micro-Kernel Architecture**: Strict separation of concerns (Core, Lease, Defense).
- **Safety**: `no_std` compatible `ConstitutionError` with `alloc::collections`.

### Tooling
- **xtask**: Automated verification, release, and submodule management.
- **Parallel Publish**: (Stub) Foundation for parallel crate publishing.

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
praborrow = "0.8.0"
```

## Quick Start

### 1. Sovereign Integrity & Constitution

```rust
use praborrow::prelude::*;

#[derive(Constitution)]
struct FiscalData {
    /// Invariants are checked at compile-time/runtime
    #[invariant("self.value >= 0")]
    value: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Establish Sovereign Data
    let data = Sovereign::new(FiscalData { value: 100 });
    
    // Invariants enforced automatically on access
    if let Some(valid_data) = data.try_get() {
        println!("Value: {}", valid_data.value);
    }
    
    Ok(())
}
```

### 2. Garuda Deadlock Detection

```rust
use praborrow::lease::deadlock::WaitForGraph;

let mut graph = WaitForGraph::new();
graph.add_wait(1, 200); // Tx 1 waits for Res 200
graph.add_wait(200, 1); // Res 200 held by Tx 1 (Deadlock if cycle)

if graph.detect_cycle() {
    println!("Deadlock detected!");
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




