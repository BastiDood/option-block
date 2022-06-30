# A Block of Optionals!
The `option-block` crate provides a simple primitive for fixed-size blocks of optional types. Formally speaking, it's a direct-address table with a fixed-size array as the storage medium.

Importantly, this is not to be confused with the popular [`slab`](https://github.com/tokio-rs/slab) crate, which internally uses the dynamically-sized, heap-allocated [`Vec`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html). Although both crates provide indexed accesses and map-like features, `option-block` operates at a lower level.

Specifically, `option-block` does not keep track of the next empty slot in the allocation upon insertion (unlike `slab`). Instead, `option-block` is simply a wrapper around an array and a bit mask. The array contains the (maybe uninitialized) data while the bit mask keeps track of the valid (i.e. initialized) entries in the allocation. Again, it's basically a direct-address table.

> This crate is compatible with [`no_std` environments](https://docs.rust-embedded.org/book/intro/no-std.html)! Neither `std` nor `alloc` is necessary.

# Motivation
## The Nullable Pointer Optimization
Sometimes, a direct-address table with a fixed-size allocation on the stack is sufficient for simple look-ups. That is, a heap-allocated `HashMap` and `Vec` may be overkill. Intuitively, one may be inclined to implement such a table using an array of `Option<T>` (for some type `T`). This is not ideal, however, because for most types, the size of an `Option<T>` (in bytes) is unnecessarily large.

Certain types in Rust take advantage of the [nullable pointer optimization](https://doc.rust-lang.org/nomicon/ffi.html#the-nullable-pointer-optimization). For some `enum` types (like `Option`), the compiler can do clever tricks to minimize its memory footprint. For instance, consider an `Option<&T>`. Assuming a 64-bit target without the nullable pointer optimization enabled, the compiler may naively allocate 16 bytes for a single `Option<&T>`: 8 bytes for the reference (i.e. the actual pointer) plus 8 bytes for the `enum` discriminant. This is indeed rather wasteful.

To resolve these issues, recall that all references in Rust are never null. The compiler can take advantage of this fact by assigning the `None::<&T>` variant to be the actual null pointer instead. Hence, we say that `&T` is `None` if the reference is null; otherwise, it is the `Some` variant (which has a valid reference). The `enum` discriminant is thus no longer necessary. An `Option<&T>` is now just 8 bytes!

The Rustonomicon discusses more examples that enable the optimization. The point is: some types have properties and assumptions that allow the compiler to forego some size overhead. _But what if this size optimization cannot happen?_

## Double the Memory Footprint
Consider an `Option<u64>`. The [`core::mem::size_of`](https://doc.rust-lang.org/nightly/core/mem/fn.size_of.html) function tells us that a single `Option<u64>` takes up 16 bytes of memory! The first 8 bytes belong to the `u64` itself while the other 8 bytes belong to the `enum` discriminant. Again, this is rather wasteful.

To resolve the `enum` discriminant overhead, the standard library provides the [`core::num::NonZeroU64`](https://doc.rust-lang.org/nightly/core/num/struct.NonZeroU64.html) type. The `NonZeroU64` is a zero-cost wrapper for `u64` that is assumed to be non-zero (as its name suggests).

This assumption makes `NonZeroU64` eligible for the nullable pointer optimization. That is, an `Option<NonZeroU64>` is `None` if it contains `0`; otherwise, it is the `Some` variant (which a valid non-zero value). We may thus remove the overhead since the value already implicitly encodes the discriminant. An `Option<NonZeroU64>` is now just 8 bytes!

```rust
use core::{mem::size_of, num::NonZeroU64};
let _ = size_of::<Option<u64>>();        // 16
let _ = size_of::<Option<NonZeroU64>>(); // 8
```

# Examples

# Stack Limitations
Since `option-block` allocates on the stack, one must handle the `Block64` and `Block128` types with care. In the extreme case of the `Block128` type, it allocates 128 instances of the inner data type plus 16 more bytes for the bit mask. Stack memory usage can easily skyrocket if too many are created. Thus, it is advised to use the larger block variants sparingly.
