$ErrorActionPreference = "Stop"
cargo check --tests -p praborrow-lease 2>&1 | Out-File -Encoding utf8 debug_lease.txt
