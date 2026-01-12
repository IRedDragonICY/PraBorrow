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

---

# PraBorrow (Bahasa Indonesia)

**The PraBorrow Framework** - Sebuah framework sistem terdistribusi untuk Rust yang menegakkan jaminan keamanan memori (memory safety) lintas batas jaringan.

> "Keamanan memori dengan integritas kedaulatan (sovereign integrity)."

## Ikhtisar (Overview)

PraBorrow memperluas model kepemilikan (ownership model) Rust ke sistem terdistribusi, menyediakan:

- **`Sovereign<T>`** - Tipe wrapper yang melacak kepemilikan antar node.
- **`Constitution`** - Macro derive untuk penegakan invarian.
- **`RawResource`** - Manajemen buffer zero-copy.
- **Leasing** - Pembagian sumber daya dengan batas waktu.

## Memulai Cepat (Quick Start)

```toml
[dependencies]
praborrow = "0.9.0"
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
    
    // Akses aman selama statusnya domestic
    println!("Saldo: {}", account.balance);
    
    // Annex memindahkan sumber daya ke yurisdiksi asing
    account.annex().unwrap();
    
    // Akses lebih lanjut akan memicu panic dengan "SOVEREIGNTY VIOLATION"
}
```

## Crate

| Crate | Deskripsi |
|-------|-----------|
| [`praborrow-core`](https://crates.io/crates/praborrow-core) | Primitif inti: `Sovereign<T>`, `CheckProtocol` |
| [`praborrow-defense`](https://crates.io/crates/praborrow-defense) | Macro `#[derive(Constitution)]` |
| [`praborrow-logistics`](https://crates.io/crates/praborrow-logistics) | Buffer `RawResource` zero-copy |
| [`praborrow-diplomacy`](https://crates.io/crates/praborrow-diplomacy) | Protokol jaringan |
| [`praborrow-lease`](https://crates.io/crates/praborrow-lease) | Leasing terdistribusi |
| [`praborrow-prover`](https://crates.io/crates/praborrow-prover) | Verifikasi formal berbasis SMT |

## Lisensi (License)

Lisensi MIT - Lihat [LICENSE](https://github.com/ireddragonicy/PraBorrow/blob/main/LICENSE)

