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
    pub const fn is_vacant(&self, index: u8) -> bool {
        self.mask & (1 << index) == 0
    }

    pub const fn len(&self) -> u32 {
        self.mask.count_ones()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: u8) -> Option<&T> {
        if self.is_vacant(index) {
            return None;
        }

        // SAFETY: We have already verified that the current `index` is not vacant.
        let uninit_ref = self.data.get(index as usize)?;
        Some(unsafe { uninit_ref.assume_init_ref() })
    }

    pub fn get_mut(&mut self, index: u8) -> Option<&mut T> {
        if self.is_vacant(index) {
            return None;
        }

        // SAFETY: We have already verified that the current `index` is not vacant.
        let uninit_ref = self.data.get_mut(index as usize)?;
        Some(unsafe { uninit_ref.assume_init_mut() })
    }

    pub fn insert(&mut self, index: u8, val: T) -> Option<T> {
        let vacant = self.is_vacant(index);
        let uninit_ref = self.data.get_mut(index as usize)?;
        let uninit_val = core::mem::replace(uninit_ref, MaybeUninit::new(val));

        self.mask |= 1 << index;
        if vacant {
            return None;
        }

        // SAFETY: The slot was occupied before replacement.
        // Therefore, it has been initialized properly.
        Some(unsafe { uninit_val.assume_init() })
    }

    pub fn remove(&mut self, index: u8) -> Option<T> {
        if self.is_vacant(index) {
            return None;
        }

        let uninit_ref = self.data.get_mut(index as usize)?;
        let uninit_val = core::mem::replace(uninit_ref, MaybeUninit::uninit());

        // SAFETY: We have already verified that the current `index` is not vacant.
        self.mask &= !(1 << index);
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
