# PraBorrow (Bahasa Indonesia)

[English](./README.md) | Indonesia

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
