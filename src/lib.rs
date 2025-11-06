#![no_std]
#![deny(warnings)]
#![doc = include_str!("../README.md")]

/// By-value and by-reference iterator objects for the various block variants.
pub mod iter;

use core::{
	mem::{ManuallyDrop, MaybeUninit},
	ops::{Index, IndexMut},
	ptr,
};

macro_rules! impl_blocked_optional {
    ($(#[$attrs:meta])* $name:ident $into_iter:ident $iter:ident $iter_mut:ident $int:ty) => {
        $(#[$attrs])*
        #[derive(Debug)]
        pub struct $name<T> {
            data: [MaybeUninit<T>; <$int>::BITS as usize],
            mask: $int,
        }

        /// Ensure that all remaining items in the block are dropped. Since the implementation
        /// internally uses [`MaybeUninit`](MaybeUninit), we **must** manually drop the valid
        /// (i.e., initialized) contents ourselves.
        impl<T> Drop for $name<T> {
            fn drop(&mut self) {
                for i in 0..Self::CAPACITY as usize {
                    self.remove(i); // No memory leaks!
                }
            }
        }

        impl<T: Clone> Clone for $name<T> {
            fn clone(&self) -> Self {
                let mut block = Self::default();
                block.mask = self.mask;

                for idx in 0..Self::CAPACITY as usize {
                    if self.is_vacant(idx) {
                        continue;
                    }

                    // SAFETY: This slot is not vacant, and hence initialized.
                    // To ensure that no resources are leaked or aliased, we
                    // must manually invoke the `clone` method ourselves.
                    let data = unsafe { self.get_unchecked(idx) };
                    block.data[idx] = MaybeUninit::new(data.clone());
                }

                block
            }
        }

        impl<T> Default for $name<T> {
            fn default() -> Self {
                Self::new()
            }
        }

        /// Create a fully initialized direct-access table.
        impl<T> From<[T; <$int>::BITS as usize]> for $name<T> {
            fn from(vals: [T; <$int>::BITS as usize]) -> Self {
                Self {
                    data: vals.map(MaybeUninit::new),
                    mask: <$int>::MAX,
                }
            }
        }

        impl<T> Index<usize> for $name<T> {
            type Output = T;
            fn index(&self, idx: usize) -> &Self::Output {
                self.get(idx).expect("slot is vacant")
            }
        }

        impl<T> IndexMut<usize> for $name<T> {
            fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
                self.get_mut(idx).expect("slot is vacant")
            }
        }

        impl<T> FromIterator<(usize, T)> for $name<T> {
            fn from_iter<I>(iter: I) -> Self
            where
                I: IntoIterator<Item = (usize, T)>
            {
                let mut block = Self::default();

                for (idx, val) in iter {
                    // SAFETY: The `insert` method internally invokes `MaybeUninit::assume_init`.
                    // Since it returns the old data by-value (if any), the `Drop` implementation
                    // should be implicitly invoked. No resources can be leaked here.
                    block.insert(idx, val);
                }

                block
            }
        }

        impl<T> IntoIterator for $name<T> {
            type Item = T;
            type IntoIter = iter::$into_iter<T>;
            fn into_iter(self) -> Self::IntoIter {
                // We need to prevent `self` from invoking `Drop` prematurely when this scope
                // finishes. We thus wrap `self` in `ManuallyDrop` to progressively drop
                // each element as the iterator is consumed.
                let this = ManuallyDrop::new(self);
                let mask = this.mask;

                // SAFETY: Reading the data pointer effectively "moves" the data out of `this`,
                // which allows us to pass ownership of the `data` to `Self::IntoIter` without
                // invoking the `Drop` impl prematurely (thanks to `ManuallyDrop` from earlier).
                let iter = unsafe { ptr::read(&this.data) }.into_iter().enumerate();
                Self::IntoIter { iter, mask }
            }
        }

        impl<'a, T> IntoIterator for &'a $name<T> {
            type Item = &'a T;
            type IntoIter = iter::$iter<'a, T>;
            fn into_iter(self) -> Self::IntoIter {
                Self::IntoIter {
                    iter: self.data.iter().enumerate(),
                    mask: self.mask,
                }
            }
        }

        impl<'a, T> IntoIterator for &'a mut $name<T> {
            type Item = &'a mut T;
            type IntoIter = iter::$iter_mut<'a, T>;
            fn into_iter(self) -> Self::IntoIter {
                Self::IntoIter {
                    iter: self.data.iter_mut().enumerate(),
                    mask: self.mask,
                }
            }
        }

        impl<T> $name<T> {
            /// Maximum capacity of the fixed-size block.
            pub const CAPACITY: u32 = <$int>::BITS;

            /// Creates a new empty block. Useful in `const` contexts.
            pub const fn new() -> Self {
                let block = MaybeUninit::<[MaybeUninit<T>; <$int>::BITS as usize]>::uninit();
                Self {
                    // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
                    // This is supported by the nightly feature: `maybe_uninit_uninit_array`.
                    // When this feature stabilizes, we may use the `MaybeUninit::uninit_array`
                    // wrapper method instead, which effectively does the same transformation.
                    data: unsafe { block.assume_init() },
                    mask: 0,
                }
            }

            /// Checks whether the item at the `index` is vacant (i.e. contains `None`).
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub const fn is_vacant(&self, index: usize) -> bool {
                assert!(index < Self::CAPACITY as usize);
                self.mask & (1 << index) == 0
            }

            /// Returns the number of non-null elements in the block.
            pub const fn len(&self) -> u32 {
                self.mask.count_ones()
            }

            /// Returns `true` if the block contains zero elements.
            pub const fn is_empty(&self) -> bool {
                self.mask == 0
            }

            /// Returns an immutable reference to the value at `index`.
            /// See the [`get`](Self::get) method for the safe, checked
            /// version of this method.
            ///
            /// # Safety
            /// The queried value **must** be properly initialized. Otherwise,
            /// the behavior is undefined.
            pub const unsafe fn get_unchecked(&self, index: usize) -> &T {
                unsafe { self.data[index].assume_init_ref() }
            }

            /// Attempts to retrieve a shared reference to the element at `index`.
            /// Returns `None` if the slot is vacant (i.e. uninitialized).
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub const fn get(&self, index: usize) -> Option<&T> {
                if self.is_vacant(index) {
                    None
                } else {
                    // SAFETY: We have already verified that the current `index` is not vacant.
                    Some(unsafe { self.get_unchecked(index) })
                }
            }

            /// Returns a mutable reference to the value at `index`.
            /// See the [`get_mut`](Self::get_mut) method for the safe,
            /// checked version of this method.
            ///
            /// # Safety
            /// The queried value **must** be properly initialized. Otherwise,
            /// the behavior is undefined.
            pub const unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
                unsafe { self.data[index].assume_init_mut() }
            }

            /// Attempts to retrieve an exclusive reference to the element at
            /// `index`. Returns `None` if the slot is vacant (i.e. uninitialized).
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub const fn get_mut(&mut self, index: usize) -> Option<&mut T> {
                if self.is_vacant(index) {
                    None
                } else {
                    // SAFETY: We have already verified that the current `index` is not vacant.
                    Some(unsafe { self.get_unchecked_mut(index) })
                }
            }

            /// If the slot at the given `index` is already occupied, this method returns a mutable
            /// reference to the inner data. Otherwise, if the slot is vacant, then this method
            /// inserts the value constructed by `func`. A mutable reference to the inner data is
            /// nevertheless returned.
            pub fn get_or_else(&mut self, index: usize, func: impl FnOnce() -> T) -> &mut T {
                if self.is_vacant(index) {
                    // SAFETY: Since this slot is initially vacant, then there are no destructors
                    // that need to be run. It should be impossible to leak resources here.
                    self.mask |= 1 << index;
                    self.data[index].write(func())
                } else {
                    // SAFETY: We have already verified that the current `index` is not vacant.
                    unsafe { self.get_unchecked_mut(index) }
                }
            }

            /// Convenience wrapper for the [`get_or_else`](Self::get_or_else) method.
            pub fn get_or(&mut self, index: usize, val: T) -> &mut T {
                self.get_or_else(index, || val)
            }

            const fn lowest_index(mask: $int) -> Option<u32> {
                // TODO: Use `lowest_one` when that stabilizes.
                let index = mask.trailing_zeros();
                if index < Self::CAPACITY {
                    Some(index)
                } else {
                    None
                }
            }

            const fn highest_index(mask: $int) -> Option<u32> {
                // TODO: Use `highest_one` when that stabilizes.
                let index = Self::CAPACITY - mask.leading_zeros();
                if index == 0 {
                    None
                } else {
                    Some(index - 1)
                }
            }

            /// Returns the index of the first (i.e., lowest index) non-vacant element in the block.
            /// Note that a [`u32`] is returned for maximum flexibility, but its value will never
            /// exceed [`Self::CAPACITY`]. It should be safe to cast to a [`usize`] without loss of
            /// information. You may also safely `unwrap` the conversion via the [`TryFrom`] trait.
            pub const fn lowest_occupied_index(&self) -> Option<u32> {
                Self::lowest_index(self.mask)
            }

            /// Returns a shared reference to the first non-vacant element in the block.
            /// Convenience wrapper around [`Self::lowest_occupied_index`] followed by [`Self::get_unchecked`].
            pub const fn first_occupied(&self) -> Option<&T> {
                if let Some(index) = self.lowest_occupied_index() {
                    // SAFETY: This is a valid index according to the bitmask.
                    Some(unsafe { self.get_unchecked(index as usize) })
                } else {
                    None
                }
            }

            /// Returns an exclusive reference to the first non-vacant element in the block.
            /// Convenience wrapper around [`Self::lowest_occupied_index`] followed by [`Self::get_unchecked_mut`].
            pub const fn first_occupied_mut(&mut self) -> Option<&mut T> {
                if let Some(index) = self.lowest_occupied_index() {
                    // SAFETY: This is a valid index according to the bitmask.
                    Some(unsafe { self.get_unchecked_mut(index as usize) })
                } else {
                    None
                }
            }

            /// Returns the index of the last (i.e., highest index) non-vacant element in the block.
            /// Note that a [`u32`] is returned for maximum flexibility, but its value will never
            /// exceed [`Self::CAPACITY`]. It should be safe to cast to a [`usize`] without loss of
            /// information. You may also safely `unwrap` the conversion via the [`TryFrom`] trait.
            pub const fn highest_occupied_index(&self) -> Option<u32> {
                Self::highest_index(self.mask)
            }

            /// Returns a shared reference to the last non-vacant element in the block.
            /// Convenience wrapper around [`Self::highest_occupied_index`] followed by [`Self::get_unchecked`].
            pub const fn last_occupied(&self) -> Option<&T> {
                if let Some(index) = self.highest_occupied_index() {
                    // SAFETY: This is a valid index according to the bitmask.
                    Some(unsafe { self.get_unchecked(index as usize) })
                } else {
                    None
                }
            }

            /// Returns an exclusive reference to the last non-vacant element in the block.
            /// Convenience wrapper around [`Self::highest_occupied_index`] followed by [`Self::get_unchecked_mut`].
            pub const fn last_occupied_mut(&mut self) -> Option<&mut T> {
                if let Some(index) = self.highest_occupied_index() {
                    // SAFETY: This is a valid index according to the bitmask.
                    Some(unsafe { self.get_unchecked_mut(index as usize) })
                } else {
                    None
                }
            }

            /// Returns the index of the first (i.e., lowest index) vacant element in the block.
            /// Note that a [`u32`] is returned for maximum flexibility, but its value will never
            /// exceed [`Self::CAPACITY`]. It should be safe to cast to a [`usize`] without loss of
            /// information. You may also safely `unwrap` the conversion via the [`TryFrom`] trait.
            pub const fn lowest_vacant_index(&self) -> Option<u32> {
                Self::lowest_index(!self.mask)
            }

            /// Attempts to insert `value` at the first vacant slot in the block.
            /// Convenience wrapper around [`Self::lowest_vacant_index`] followed by [`Self::insert`].
            ///
            /// # Return Value
            /// - `Ok(option)` if a vacant slot was found, where `option` is the return value from [`Self::insert`].
            /// - `Err(value)` if the block is full, returning the original `value` back to the caller.
            pub const fn insert_at_first_vacancy(&mut self, value: T) -> Result<Option<T>, T> {
                if let Some(index) = self.lowest_vacant_index() {
                    Ok(self.insert(index as usize, value))
                } else {
                    Err(value)
                }
            }

            /// Returns the index of the last (i.e., highest index) vacant element in the block.
            /// Note that a [`u32`] is returned for maximum flexibility, but its value will never
            /// exceed [`Self::CAPACITY`]. It should be safe to cast to a [`usize`] without loss of
            /// information. You may also safely `unwrap` the conversion via the [`TryFrom`] trait.
            pub const fn highest_vacant_index(&self) -> Option<u32> {
                Self::highest_index(!self.mask)
            }

            /// Attempts to insert `value` at the last vacant slot in the block.
            /// Convenience wrapper around [`Self::highest_vacant_index`] followed by [`Self::insert`].
            ///
            /// # Return Value
            /// - `Ok(option)` if a vacant slot was found, where `option` is the return value from [`Self::insert`].
            /// - `Err(value)` if the block is full, returning the original `value` back to the caller.
            pub const fn insert_at_last_vacancy(&mut self, value: T) -> Result<Option<T>, T> {
                if let Some(index) = self.highest_vacant_index() {
                    Ok(self.insert(index as usize, value))
                } else {
                    Err(value)
                }
            }

            /// Inserts the `value` at the `index`. If a value already exists, it returns `Some`
            /// containing the old value. Otherwise, it returns `None`.
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub const fn insert(&mut self, index: usize, value: T) -> Option<T> {
                let vacant = self.is_vacant(index);
                let uninit_value = core::mem::replace(&mut self.data[index], MaybeUninit::new(value));
                self.mask |= 1 << index;

                if vacant {
                    None
                } else {
                    // SAFETY: The slot was occupied before replacement.
                    // Therefore, it has been initialized properly.
                    Some(unsafe { uninit_value.assume_init() })
                }
            }

            /// Removes the value at the `index`. If a value already exists, it returns `Some`
            /// containing that value. Otherwise, it returns `None`.
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub const fn remove(&mut self, index: usize) -> Option<T> {
                if self.is_vacant(index) {
                    return None;
                }

                let uninit_val = core::mem::replace(&mut self.data[index], MaybeUninit::uninit());
                self.mask &= !(1 << index);

                // SAFETY: We have already verified that the current `index` is not vacant.
                Some(unsafe { uninit_val.assume_init() })
            }

            /// Create a by-reference iterator for this block.
            pub fn iter(&self) -> iter::$iter<'_, T> {
                iter::$iter {
                    iter: self.data.iter().enumerate(),
                    mask: self.mask,
                }
            }

            /// Create a mutable by-reference iterator for this block.
            pub fn iter_mut(&mut self) -> iter::$iter_mut<'_, T> {
                iter::$iter_mut {
                    iter: self.data.iter_mut().enumerate(),
                    mask: self.mask,
                }
            }
        }

        impl<T: Default> $name<T> {
            /// Convenience wrapper for the [`get_or_else`](Self::get_or_else) method.
            pub fn get_or_default(&mut self, index: usize) -> &mut T {
                self.get_or_else(index, Default::default)
            }
        }
    };
}

impl_blocked_optional! {
	/// A fixed block of optionals masked by a [`u8`](u8),
	/// which may thus contain at most 8 elements.
	Block8 Block8IntoIter Block8Iter Block8IterMut u8
}

impl_blocked_optional! {
	/// A fixed block of optionals masked by a [`u16`](u16),
	/// which may thus contain at most 16 elements.
	Block16 Block16IntoIter Block16Iter Block16IterMut u16
}

impl_blocked_optional! {
	/// A fixed block of optionals masked by a [`u32`](u32),
	/// which may thus contain at most 32 elements.
	Block32 Block32IntoIter Block32Iter Block32IterMut u32
}

impl_blocked_optional! {
	/// A fixed block of optionals masked by a [`u64`](u64),
	/// which may thus contain at most 64 elements.
	Block64 Block64IntoIter Block64Iter Block64IterMut u64
}

impl_blocked_optional! {
	/// A fixed block of optionals masked by a [`u128`](u128),
	/// which may thus contain at most 128 elements.
	Block128 Block128IntoIter Block128Iter Block128IterMut u128
}
