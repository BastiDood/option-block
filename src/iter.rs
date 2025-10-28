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

use core::ops::Range;

macro_rules! impl_iterator_outer {
	($name:ident $into_iter:ident $iter:ident) => {
		/// By-value iterator that consumes the block allocation.
		pub struct $into_iter<T> {
			pub(crate) block: $crate::$name<T>,
			pub(crate) index: Range<usize>,
		}

		impl<T> Iterator for $into_iter<T> {
			type Item = T;
			fn next(&mut self) -> Option<Self::Item> {
				Some(loop {
					let idx = self.index.next()?;
					if let Some(val) = self.block.remove(idx) {
						break val;
					}
				})
			}
		}

		/// By-reference iterator that borrows from the block allocation.
		pub struct $iter<'a, T> {
			pub(crate) block: &'a $crate::$name<T>,
			pub(crate) index: Range<usize>,
		}

		impl<'a, T> Iterator for $iter<'a, T> {
			type Item = &'a T;
			fn next(&mut self) -> Option<Self::Item> {
				Some(loop {
					let idx = self.index.next()?;
					if let Some(val) = self.block.get(idx) {
						break val;
					}
				})
			}
		}
	};
}

impl_iterator_outer!(Block8 Block8IntoIter Block8Iter);
impl_iterator_outer!(Block16 Block16IntoIter Block16Iter);
impl_iterator_outer!(Block32 Block32IntoIter Block32Iter);
impl_iterator_outer!(Block64 Block64IntoIter Block64Iter);
impl_iterator_outer!(Block128 Block128IntoIter Block128Iter);
