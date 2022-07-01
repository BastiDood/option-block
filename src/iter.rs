//! By-value and by-reference iterator objects for the various block variants.

use core::{iter::DoubleEndedIterator, ops::Range};

macro_rules! impl_iterator_outer {
    ($name:ident $into_iter:ident $iter:ident) => {
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

        impl<T> DoubleEndedIterator for $into_iter<T> {
            fn next_back(&mut self) -> Option<Self::Item> {
                Some(loop {
                    let idx = self.index.next_back()?;
                    if let Some(val) = self.block.remove(idx) {
                        break val;
                    }
                })
            }
        }

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

        impl<'a, T> DoubleEndedIterator for $iter<'a, T> {
            fn next_back(&mut self) -> Option<Self::Item> {
                Some(loop {
                    let idx = self.index.next_back()?;
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
