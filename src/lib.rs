#![no_std]
#![doc = include_str!("../README.md")]

mod iter;

use core::mem::MaybeUninit;

macro_rules! impl_blocked_optional {
    ($(#[$attrs:meta])* $name:ident $into_iter:ident $iter:ident $int:ty) => {
        $(#[$attrs])*
        #[derive(Debug)]
        pub struct $name<T> {
            data: [MaybeUninit<T>; <$int>::BITS as usize],
            mask: $int,
        }

        /// Since the current implementation relies on [`MaybeUninit`](MaybeUninit), the
        /// block can only be cloned if the internal data is trivially copyable (bitwise).
        impl<T: Copy> Clone for $name<T> {
            fn clone(&self) -> Self {
                Self {
                    data: self.data,
                    mask: self.mask,
                }
            }
        }

        impl<T: Copy> Copy for $name<T> { }

        impl<T> Default for $name<T> {
            fn default() -> Self {
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

        impl<T> IntoIterator for $name<T> {
            type Item = T;
            type IntoIter = iter::$into_iter<T>;
            fn into_iter(self) -> Self::IntoIter {
                iter::$into_iter {
                    block: self,
                    index: 0..Self::CAPACITY as usize,
                }
            }
        }

        impl<T> $name<T> {
            /// Maximum capacity of the fixed-size block.
            pub const CAPACITY: u32 = <$int>::BITS;

            /// Checks whether the item at the `index` is vacant (i.e. contains `None`).
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
                self.len() == 0
            }

            /// Attempts to retrieve a shared reference to the element at `index`.
            /// Returns `None` if the slot is vacant (i.e. uninitialized).
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub fn get(&self, index: usize) -> Option<&T> {
                if self.is_vacant(index) {
                    None
                } else {
                    // SAFETY: We have already verified that the current `index` is not vacant.
                    Some(unsafe { self.data[index].assume_init_ref() })
                }
            }

            /// Attempts to retrieve an exclusive reference to the element at
            /// `index`. Returns `None` if the slot is vacant (i.e. uninitialized).
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
                if self.is_vacant(index) {
                    None
                } else {
                    // SAFETY: We have already verified that the current `index` is not vacant.
                    Some(unsafe { self.data[index].assume_init_mut() })
                }
            }

            /// Inserts the `val` at the `index`. If a value already exists, it returns `Some`
            /// containing the old value. Otherwise, it returns `None`.
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub fn insert(&mut self, index: usize, val: T) -> Option<T> {
                let vacant = self.is_vacant(index);
                let uninit_val = core::mem::replace(&mut self.data[index], MaybeUninit::new(val));
                self.mask |= 1 << index;

                if vacant {
                    None
                } else {
                    // SAFETY: The slot was occupied before replacement.
                    // Therefore, it has been initialized properly.
                    Some(unsafe { uninit_val.assume_init() })
                }
            }

            /// Removes the value at the `index`. If a value already exists, it returns `Some`
            /// containing that value. Otherwise, it returns `None`.
            ///
            /// # Panic
            /// Panics if `index >= CAPACITY`. See the [maximum capacity](Self::CAPACITY).
            pub fn remove(&mut self, index: usize) -> Option<T> {
                if self.is_vacant(index) {
                    return None;
                }

                let uninit_val = core::mem::replace(&mut self.data[index], MaybeUninit::uninit());
                self.mask &= !(1 << index);

                // SAFETY: We have already verified that the current `index` is not vacant.
                Some(unsafe { uninit_val.assume_init() })
            }

            pub fn iter(&self) -> iter::$iter<T> {
                iter::$iter {
                    block: self,
                    index: 0..Self::CAPACITY as usize,
                }
            }
        }
    };
}

impl_blocked_optional! {
    /// A fixed block of optionals masked by a [`u8`](u8),
    /// which may thus contain at most 8 elements.
    Block8 Block8IntoIter Block8Iter u8
}

impl_blocked_optional! {
    /// A fixed block of optionals masked by a [`u16`](u16),
    /// which may thus contain at most 16 elements.
    Block16 Block16IntoIter Block16Iter u16
}

impl_blocked_optional! {
    /// A fixed block of optionals masked by a [`u32`](u32),
    /// which may thus contain at most 32 elements.
    Block32 Block32IntoIter Block32Iter u32
}

impl_blocked_optional! {
    /// A fixed block of optionals masked by a [`u64`](u64),
    /// which may thus contain at most 64 elements.
    Block64 Block64IntoIter Block64Iter u64
}

impl_blocked_optional! {
    /// A fixed block of optionals masked by a [`u128`](u128),
    /// which may thus contain at most 128 elements.
    Block128 Block128IntoIter Block128Iter u128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_tests() {
        assert_eq!(Block8::<()>::CAPACITY, 8);
        assert_eq!(Block16::<()>::CAPACITY, 16);
        assert_eq!(Block32::<()>::CAPACITY, 32);
        assert_eq!(Block64::<()>::CAPACITY, 64);
        assert_eq!(Block128::<()>::CAPACITY, 128);
    }

    #[test]
    fn size_tests() {
        use core::mem::size_of;
        assert_eq!(size_of::<Block8<u8>>(), 8 + 1);
        assert_eq!(size_of::<Block16<u8>>(), 16 + 2);
        assert_eq!(size_of::<Block32<u8>>(), 32 + 4);
        assert_eq!(size_of::<Block64<u8>>(), 64 + 8);
        assert_eq!(size_of::<Block128<u8>>(), 128 + 16);
    }

    #[test]
    fn insert_replace_semantics() {
        let mut block = Block8::default();
        assert!(block.is_empty());

        assert!(block.insert(0, 32).is_none());
        assert!(block.insert(1, 64).is_none());

        assert_eq!(block.insert(0, 1), Some(32));
        assert_eq!(block.insert(1, 2), Some(64));

        assert_eq!(block.remove(0), Some(1));
        assert_eq!(block.remove(1), Some(2));

        assert!(block.is_empty());
    }

    #[test]
    fn fully_initialized_removals() {
        let mut block = Block8::<u8>::from([0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(block.remove(0), Some(0));
        assert_eq!(block.remove(1), Some(1));
        assert_eq!(block.remove(2), Some(2));
        assert_eq!(block.remove(3), Some(3));
        assert_eq!(block.remove(4), Some(4));
        assert_eq!(block.remove(5), Some(5));
        assert_eq!(block.remove(6), Some(6));
        assert_eq!(block.remove(7), Some(7));
        assert!(block.is_empty());
    }
}
