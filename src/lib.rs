#![no_std]
#![feature(maybe_uninit_uninit_array)]

use core::mem::MaybeUninit;

pub struct BlockedOptionals8<T> {
    data: [MaybeUninit<T>; 8],
    mask: u8,
}

impl<T> Default for BlockedOptionals8<T> {
    fn default() -> Self {
        Self {
            data: MaybeUninit::uninit_array(),
            mask: 0,
        }
    }
}

impl<T> BlockedOptionals8<T> {
    /// Checks whether the item at the `index` is vacant (i.e. contains `None`).
    pub const fn is_vacant(&self, index: usize) -> bool {
        self.mask & (1 << index) == 0
    }

    pub const fn len(&self) -> u32 {
        self.mask.count_ones()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if self.is_vacant(index) {
            None
        } else {
            // SAFETY: We have already verified that the current `index` is not vacant.
            Some(unsafe { self.data[index].assume_init_ref() })
        }
    }

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

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if self.is_vacant(index) {
            return None;
        }

        let uninit_val = core::mem::replace(&mut self.data[index], MaybeUninit::uninit());
        self.mask &= !(1 << index);

        // SAFETY: We have already verified that the current `index` is not vacant.
        Some(unsafe { uninit_val.assume_init() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_replace_semantics() {
        let mut block = BlockedOptionals8::default();
        assert!(block.is_empty());

        assert!(block.insert(0, 32).is_none());
        assert!(block.insert(1, 64).is_none());

        assert_eq!(block.insert(0, 1), Some(32));
        assert_eq!(block.insert(1, 2), Some(64));

        assert_eq!(block.remove(0), Some(1));
        assert_eq!(block.remove(1), Some(2));

        assert!(block.is_empty());
    }
}
