# 0.2.2 (July 2, 2022)
## Documentation Changes
* Outdated documentation regarding the `Clone` implementation has been removed.
* Added doc-comment about the `iter` method.
* Clarified that the `is_vacant` method may panic when the given `index` is out of bounds.

# 0.2.1 (July 2, 2022)
This patch release mainly features documentation-related improvements. In particular, it has been made clearer that the `iter` module is not meant to be directly used. Rather, it is only part of the public interface so that users have the option to explicitly "name" the iterator objects in their code.

# 0.2.0 (July 1, 2022)
## Undefined Behavior Resolved
This release fixes a critical oversight in the use of [`core::mem::MaybeUninit`](https://doc.rust-lang.org/nightly/core/mem/union.MaybeUninit.html). Internally, `option-block` uses `MaybeUninit` to allocate an array which serves as the direct-address table on the stack.

However, the original implementation did _not_ implement the `Drop` trait for the block variants. For non-trivial types with destructors (i.e. types that implement `Drop`), this leads to leaked memory and resources (at best). This is because `MaybeUninit` requires its contents to be manually dropped by the owner. In the worst case, however, the failure to invoke the `Drop` implementation leads to various (implementation-specific) undefined behavior.

```rust
let mut block = option_block::Block8::default();
block.insert(0, String::from("Hello"));

// This leaks the string because `Drop` was (originally) not implemented!
// Internally, the `MaybeUninit` will simply ignore the `String`.
// No destructors will be invoked.
drop(block);
```

To address this, the various block variants now implement `Drop`. The implementation basically drops any valid elements left in the block.

## Changes in `Clone` and `Copy` Bounds
Originally, all block variants implemented `Clone` and `Copy` as long as the inner data type `T` implements `Copy`. This is fine, but it is too restrictive. This release loosens the `Clone` trait bound. Now, as long as `T` implements `Clone` (no `Copy` necessary), the block variant will also implement `Clone`. In line with the resolved undefined behavior above, the `Clone` implementation is careful to only values that have been _explicitly_ initialized (via `insert` or otherwise).

Note that since all block variants now implement `Drop`, it is now impossible to implement `Copy`. The compiler forbids types with destructors from implementing `Copy` (for good reason). Therefore, all block variants are no longer trivially `Copy`-able. This was an oversight from the original implementation.

## `FromIterator` Implementation
For convenience, `FromIterator<(usize, T)` (for some `T`) has been implemented for all block variants. It is now possible to initialize a block from an iterator of key-value pairs.

```rust
let block: option_block::Block8<_> = [10, 8, 1]
    .into_iter()
    .enumerate()
    .collect();
assert_eq!(block.get(0), Some(&10));
assert_eq!(block.get(1), Some(&8));
assert_eq!(block.get(2), Some(&1));
assert!(block.get(3).is_none());
```

## `IntoIterator` Implementation
The `IntoIterator` trait has also been implemented for `Block` (see `into_iter` mehotd) and `&Block` (see `iter` method). At the moment, there is no equivalent implementation for `iter_mut` due to some strange lifetime annotation issues. This will be sorted out in future releases.

```rust
let block: option_block::Block8<_> = [10, 8, 1]
    .into_iter()
    .enumerate()
    .collect();

for val in &block {
    // Do stuff by-reference...
}

for val in block {
    // Do stuff by-value...
}
```

## New Getter Methods
For convenience, new getters with default inserters have been added.

* The main addition is the `get_or_else` method, which attempts to retrieve a value and return an exclusive reference to it. If the slot is vacant, then it constructs a new value based on the given closure.
* Next is the `get_or` method, which is simply a special case of the `get_or_else` method where the value is ready upfront.
* Finally, the `get_or_default` method provides a wrapper around `get_or_else` for inserting the default value if the slot is vacant.

```rust
let mut block = option_block::Block8::default();
assert_eq!(block.get_or_else(0, || 100), &mut 100);
assert_eq!(block.get_or(1, 200), &mut 200);
assert_eq!(block.get_or_default(2), &mut 0);
```

# 0.1.0 (July 1, 2022)
This is the initial release. Note that this has since been yanked due to undefined behavior.
