$ErrorActionPreference = "Stop"
$env:RUST_BACKTRACE = "1"
cargo test -p praborrow-defense 2>&1 | Out-File -Encoding utf8 debug_panic.txt
