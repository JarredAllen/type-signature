# type-signature

Compile-time structural signatures for Rust types.

Every type that implements [`TypeSignature`] exposes a `SIGNATURE` derived from
the type's name, generic arguments, and field/variant layout. If the layout
changes in a way that could be a breaking change for consumers, the hash
changes.

This is useful for:

- **Schema drift detection**: pin a `CONST_HASH` in a `const` assertion so an
  accidental field addition or type change fails the build instead of silently
  shipping a breaking change.
- **Cross-process compatibility checks**: embed the hash in a file header or
  wire protocol handshake so mismatched producers and consumers reject each
  other early.
- **Cache/index invalidation**: key a persisted cache on the signature so
  rebuilding a type's schema automatically invalidates stale data.

This trait is `no-std` compatible! The dependency on `std` and `alloc` are both
controlled by features.

## Example

```rust
use type_signature::TypeSignature;

#[derive(TypeSignature)]
struct Message {
    id: u64,
    body: String,
}

// Lock in the current schema at compile time. Changing `Message` in a
// breaking way will fail this assertion.
const _: () = assert!(Message::CONST_HASH == 0x7190_e284_5c80_29a5);
```

## MSRV Policy

This crate has a minimum supported Rust version in the `Cargo.toml`. It may be
updated on minor version releases of this crate, but will be at least 2 release
cycles old.
