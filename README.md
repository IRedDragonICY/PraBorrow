![PraBorrow Banner](banner.png)

# PraBorrow

**High-Performance Distributed Systems Framework**

[![Crates.io](https://img.shields.io/crates/v/praborrow.svg)](https://crates.io/crates/praborrow)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

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

---

# PraBorrow (Bahasa Indonesia)

**Framework Sistem Terdistribusi Berkinerja Tinggi**

PraBorrow menegakkan "Keamanan memori dengan integritas kedaulatan" dengan menggabungkan semantik kepemilikan yang ketat, pemeriksaan invarian prosedural, dan logistik zero-copy ke dalam satu framework yang terpadu.

## Apa yang Baru di v0.9.0

### Ekosistem & Logistik
- **Sistem Pembuktian Garuda**: Verifikasi formal berbasis SMT dengan Z3.
- **Parallel Publish**: Workflow rilis otomatis yang sadar akan ketergantungan.
- **Garuda Dashboard**: Deteksi Deadlock terintegrasi (`WaitForGraph`) dalam `praborrow-lease`.

### Tooling
- **xtask**: Verifikasi otomatis, rilis, dan manajemen submodule.

## Arsitektur (Architecture)

Framework ini dimodularisasi menjadi crate khusus:

| Crate | Deskripsi |
|-------|-----------|
| **`praborrow-core`** | `Sovereign<T>` untuk pelacakan status Domestic/Exiled. |
| **`praborrow-defense`** | `#[derive(Constitution)]` untuk penegakan invarian (sistem "Garuda"). |
| **`praborrow-logistics`** | Buffer zero-copy `RawResource` dengan akses aman. |
| **`praborrow-diplomacy`** | Interop FFI untuk sistem asing. |
| **`praborrow-lease`** | Konsensus Raft dengan abstraksi `RaftStorage`. |
| **`praborrow-prover`** | Sistem Pembuktian Garuda (Verifikasi SMT dengan cache LRU). |
| **`praborrow-macros`** | Crate proc-macro untuk `#[derive(Constitution)]`. |
| **`praborrow-sidl`** | Schema IDL untuk protokol terdistribusi. |

## Instalasi (Installation)

```toml
[dependencies]
praborrow = "0.9.0"
```

## Keamanan (Safety Features)

- **Zero Panics di Kode Library**: Semua jalur kritis mengembalikan tipe `Result`.
- **Komentar SAFETY**: Setiap blok `unsafe` didokumentasikan.
- **Error dengan Tipe Ketat**: Tidak ada error berbasis string di API publik.
- **Cache Terbatas**: Strategi LRU mencegah pertumbuhan memori yang tidak terbatas.
- **Keamanan Memori**: `RawResource` melakukan dealokasi dengan benar menggunakan `Drop`.

## Lisensi (License)

MIT



