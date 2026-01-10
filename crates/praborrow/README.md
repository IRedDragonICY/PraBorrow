# PraBorrow

![PraBorrow Banner](https://raw.githubusercontent.com/ireddragonicy/PraBorrow/main/banner.png)

[![Crates.io](https://img.shields.io/crates/v/praborrow.svg)](https://crates.io/crates/praborrow)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**The PraBorrow Framework** - A distributed systems framework for Rust that enforces memory safety guarantees across network boundaries.

> "Memory safety with sovereign integrity."

## Overview

PraBorrow extends Rust's ownership model to distributed systems, providing:

- **`Sovereign<T>`** - Wrapper type that tracks ownership across nodes
- **`Constitution`** - Derive macro for invariant enforcement
- **`RawResource`** - Zero-copy buffer management
- **Leasing** - Time-bounded resource sharing

## Quick Start

```toml
[dependencies]
praborrow = "0.2"
```

```rust
use praborrow::prelude::*;

#[derive(Constitution)]
struct Account {
    #[invariant("self.balance >= 0")]
    balance: i64,
}

fn main() {
    let account = Sovereign::new(Account { balance: 100 });
    
    // Access is safe while domestic
    println!("Balance: {}", account.balance);
    
    // Annex moves resource to foreign jurisdiction
    account.annex().unwrap();
    
    // Further access would panic with "SOVEREIGNTY VIOLATION"
}
```

## Crates

| Crate | Description |
|-------|-------------|
| [`praborrow-core`](https://crates.io/crates/praborrow-core) | Core primitives: `Sovereign<T>`, `CheckProtocol` |
| [`praborrow-defense`](https://crates.io/crates/praborrow-defense) | `#[derive(Constitution)]` macro |
| [`praborrow-logistics`](https://crates.io/crates/praborrow-logistics) | Zero-copy `RawResource` buffers |
| [`praborrow-diplomacy`](https://crates.io/crates/praborrow-diplomacy) | Networking protocols |
| [`praborrow-lease`](https://crates.io/crates/praborrow-lease) | Distributed leasing |
| [`praborrow-prover`](https://crates.io/crates/praborrow-prover) | SMT-based formal verification |

## License

MIT License - See [LICENSE](https://github.com/ireddragonicy/PraBorrow/blob/main/LICENSE)
