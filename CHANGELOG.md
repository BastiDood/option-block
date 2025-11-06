# 0.6.0 (November 7, 2025)

This release introduces brand new helpers for getting the first and last elements of a block.

## New Getters for the First Occupied Element

- `lowest_occupied_index` returns the index of the first occupied element in the block.
- `first_occupied` returns a shared reference to the first occupied element in the block.
- `first_occupied_mut` returns an exclusive reference to the first occupied element in the block.

## New Getters for the Last Occupied Element

- `highest_occupied_index` returns the index of the last occupied element in the block.
- `last_occupied` returns a shared reference to the last occupied element in the block.
- `last_occupied_mut` returns an exclusive reference to the last occupied element in the block.

## New Getters for the First Vacant Element

- `lowest_vacant_index` returns the index of the first vacant element in the block.
- `first_vacant` returns a shared reference to the first vacant element in the block.
- `first_vacant_mut` returns an exclusive reference to the first vacant element in the block.

## New Getters for the Last Vacant Element

- `highest_vacant_index` returns the index of the last vacant element in the block.
- `last_vacant` returns a shared reference to the last vacant element in the block.
- `last_vacant_mut` returns an exclusive reference to the last vacant element in the block.

## New Inserters into Vacant Slots

- `insert_at_first_vacancy` inserts a value into the first vacancy of the block.
- `insert_at_last_vacancy` inserts a value into the last vacancy of the block.

# 0.5.0 (November 6, 2025)

## Now available in `const` contexts!

This release introduces a brand `new` 🥁 constructor that is compatible in `const` contexts. The old `Default` implementation is still available, but moving forward, the `const` constructor is now recommended wherever possible.

```rust
// You can now use blocks in `static` variables!
static BLOCK: Block8<u8> = Block8::new();
```

Various methods throughout the library have also been `const`-ified, allowing basic compile-time initialization of blocks.

```rust
// Bespoke `const`-initialization is also allowed!
static BLOCK: Block8<u8> = {
    let mut block = Block8::new();
    block.insert(0, 1);
    block.insert(1, 2);
    block.insert(2, 3);
    block
};
```

# 0.4.1 (October 29, 2025)

A minor corrective release that includes the integration tests into the bundled package. No crate behaviors are affected. This is mainly for documentation.

# 0.4.0 (October 29, 2025)

## `iter_mut` is now supported!

A long-standing shortcoming of the crate is its missing `iter_mut` implementation. This version finally adds the missing piece to the iteration story.

> [!NOTE]
> Internally, the crate has been refactored to leverage [`core::array`] iterator primitives for improved safety.

[`core::array`]: https://doc.rust-lang.org/stable/core/array/index.html

## Rust 2024 Edition

The crate now uses Rust 2024, which bumps the `rust-version` to `1.85.0`. This is arguably an invisible change as most `no_std` use cases tend to require the nightly toolchain anyway. 😅

# 0.3.0 (July 22, 2022)

## New Unchecked Getters

Users now have the option to skip the validation step when getting a reference to a value in the block. However, this should be sparingly used because it is `unsafe`. If improperly used, the method returns garbage memory, which may invoke undefined behavior.

```rust
let mut block = option_block::Block8::default();
block.insert(0, 100);

// Safe! 👍
assert_eq!(block.get(0), Some(&100));
assert_eq!(unsafe { block.get_unchecked(0) }, &100);

// Undefined Behavior! ⚠
let _ = unsafe { block.get_unchecked(1) };
```

# 0.2.2 (July 2, 2022)

## Documentation Changes

- Outdated documentation regarding the `Clone` implementation has been removed.
- Added doc-comment about the `iter` method.
- Clarified that the `is_vacant` method may panic when the given `index` is out of bounds.

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

- The main addition is the `get_or_else` method, which attempts to retrieve a value and return an exclusive reference to it. If the slot is vacant, then it constructs a new value based on the given closure.
- Next is the `get_or` method, which is simply a special case of the `get_or_else` method where the value is ready upfront.
- Finally, the `get_or_default` method provides a wrapper around `get_or_else` for inserting the default value if the slot is vacant.

```rust
let mut block = option_block::Block8::default();
assert_eq!(block.get_or_else(0, || 100), &mut 100);
assert_eq!(block.get_or(1, 200), &mut 200);
assert_eq!(block.get_or_default(2), &mut 0);
```

# 0.1.0 (July 1, 2022)

This is the initial release. Note that this has since been yanked due to undefined behavior.
