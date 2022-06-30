# A Block of Optionals!
The `option-block` crate provides a simple primitive for fixed-size blocks of optional types. Formally speaking, it's a direct-address table with a fixed-size array as the storage medium.

Importantly, this is not to be confused with the popular [`slab`](https://github.com/tokio-rs/slab) crate, which internally uses the dynamically-sized, heap-allocated [`Vec`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html). Although both crates provide indexed accesses and map-like features, `option-block` operates at a lower level.

Specifically, `option-block` does not keep track of the next empty slot in the allocation upon insertion (unlike `slab`). Instead, `option-block` is simply a wrapper around an array and a bit mask. The array contains the (maybe uninitialized) data while the bit mask keeps track of the valid (i.e. initialized) entries in the allocation. Again, it's basically a direct-address table.

> This crate is compatible with [`no_std` environments](https://docs.rust-embedded.org/book/intro/no-std.html)! Neither `std` nor `alloc` is necessary.

# Motivation

# Examples

# Stack Limitations
Since `option-block` allocates on the stack, one must handle the `Block64` and `Block128` types with care. In the extreme case of the `Block128` type, it allocates 128 instances of the inner data type plus 16 more bytes for the bit mask. Stack memory usage can easily skyrocket if too many are created. Thus, it is advised to use the larger block variants sparingly.
