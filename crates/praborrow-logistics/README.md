# Praborrow Logistics

Zero-copy data abstraction layer for the "Hilirisasi" pipeline. Handles the refinement of raw byte streams into downstream-ready resources without unnecessary cloning or allocation overhead.

## RawResource

Provides a stable pointer/length view into a byte vector. Uses `ManuallyDrop` to ensure the underlying buffer is not deallocated while the resource is active, strictly adhering to zero-copy principles during refinement.

### Usage

```rust
use praborrow_logistics::RawResource;

let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
let resource = RawResource::refine(data);

unsafe {
    // Access via raw pointer
    let slice = std::slice::from_raw_parts(resource.ptr, resource.len);
    assert_eq!(slice[0], 0xDE);
}
```

### Safety

This crate facilitates raw pointer manipulation. The caller is responsible for ensuring the lifetime of the `RawResource` does not violate memory safety guarantees of the consumptive system.
