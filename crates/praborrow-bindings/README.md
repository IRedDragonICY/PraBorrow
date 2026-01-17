# PraBorrow Python Bindings

**PraBorrow** is a formal distributed ownership system based on Rust's borrow checker rules, designed for high-reliability distributed consensus.

This package (`praborrow`) provides Python bindings for the core `Sovereign<T>` primitives, allowing you to leverage PraBorrow's safety guarantees in your Python applications.

## Installation

```bash
pip install praborrow
```

## Usage

```python
from praborrow import SovereignString

# Create a domestic sovereign resource
sov = SovereignString.new("Hello World")

print(f"Domestic? {sov.is_domestic()}")  # True
print(f"Value: {sov.get_value()}")       # "Hello World"

# Annex the resource (take strict ownership/lock)
sov.annex()
# Now it might be exiled/moved depending on rules (simplified example)
```

## Features

- **Formal Verification**: Core logic verified with Z3 Prover.
- **Thread Safety**: Rust-backed concurrency guarantees.
- **Type Safety**: Strong typing for ownership states.

## License

MIT
