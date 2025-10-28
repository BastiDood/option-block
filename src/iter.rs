//! By-value and by-reference iterator objects for the various block variants. Note that these
//! types aren't meant to be used directly. They are simply part of the public interface just
//! in case one needs to explicitly "name" the iterator object in their code.
//!
//! # Example
//!
//! ```rust
//! let block: option_block::Block8<_> = [10, 8, 1].into_iter().enumerate().collect();
//! assert_eq!(block.get(0), Some(&10));
//! assert_eq!(block.get(1), Some(&8));
//! assert_eq!(block.get(2), Some(&1));
//! assert!(block.get(3).is_none());
//! ```

use core::{array, iter::Enumerate, mem::MaybeUninit, slice};

macro_rules! impl_iterator_outer {
	($name:ident $into_iter:ident $iter:ident $iter_mut:ident $int:ty) => {
		/// By-value iterator that consumes the block allocation.
		pub struct $into_iter<T> {
			pub(crate) iter: Enumerate<array::IntoIter<MaybeUninit<T>, { <$int>::BITS as usize }>>,
			pub(crate) mask: $int,
		}

		impl<T> Iterator for $into_iter<T> {
			type Item = T;
			fn next(&mut self) -> Option<Self::Item> {
				loop {
					let (i, item) = self.iter.next()?;
					if self.mask & (1 << i) != 0 {
						// SAFETY: The bitmask guarantees this slot is initialized.
						return Some(unsafe { item.assume_init() });
					}
					// Skip vacant slots: `item` is uninitialized, so no drop needed.
				}
			}
		}

		/// By-reference iterator that borrows from the block allocation.
		pub struct $iter<'a, T> {
			pub(crate) iter: Enumerate<slice::Iter<'a, MaybeUninit<T>>>,
			pub(crate) mask: $int,
		}

		impl<'a, T> Iterator for $iter<'a, T> {
			type Item = &'a T;
			fn next(&mut self) -> Option<Self::Item> {
				loop {
					let (i, item) = self.iter.next()?;
					if self.mask & (1 << i) != 0 {
						// SAFETY: The bitmask guarantees this slot is initialized.
						return Some(unsafe { item.assume_init_ref() });
					}
				}
			}
		}

		/// Mutable by-reference iterator that borrows mutably from the block allocation.
		pub struct $iter_mut<'a, T> {
			pub(crate) iter: Enumerate<slice::IterMut<'a, MaybeUninit<T>>>,
			pub(crate) mask: $int,
		}

		impl<'a, T> Iterator for $iter_mut<'a, T> {
			type Item = &'a mut T;
			fn next(&mut self) -> Option<Self::Item> {
				loop {
					let (i, item) = self.iter.next()?;
					if self.mask & (1 << i) != 0 {
						// SAFETY: The bitmask guarantees this slot is initialized.
						return Some(unsafe { item.assume_init_mut() });
					}
				}
			}
		}
	};
}

impl_iterator_outer!(Block8 Block8IntoIter Block8Iter Block8IterMut u8);
impl_iterator_outer!(Block16 Block16IntoIter Block16Iter Block16IterMut u16);
impl_iterator_outer!(Block32 Block32IntoIter Block32Iter Block32IterMut u32);
impl_iterator_outer!(Block64 Block64IntoIter Block64Iter Block64IterMut u64);
impl_iterator_outer!(Block128 Block128IntoIter Block128Iter Block128IterMut u128);
