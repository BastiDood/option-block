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
fn check_iterators() {
	let block = Block8::<usize>::from([0, 1, 2, 3, 4, 5, 6, 7]);

	for (idx, &val) in block.iter().enumerate() {
		assert_eq!(idx, val);
	}

	for (idx, val) in block.into_iter().enumerate() {
		assert_eq!(idx, val);
	}
}

#[test]
fn indexing_operations() {
	use core::ops::Range;
	type Block = Block8<usize>;
	const RANGE: Range<usize> = 0..Block::CAPACITY as usize;
	let mut block = Block::from([0, 1, 2, 3, 4, 5, 6, 7]);

	for i in RANGE {
		assert_eq!(block[i], i);
	}

	for i in RANGE {
		block[i] *= 2;
	}

	for i in RANGE {
		assert_eq!(block[i], i * 2);
	}
}

#[test]
fn default_getters() {
	let mut block = Block8::<u16>::default();

	assert_eq!(block.get_or_else(0, || 5), &mut 5);
	assert_eq!(block.get_or(1, 10), &mut 10);
	assert_eq!(block.get_or_default(2), &mut 0);

	assert_eq!(block.get_or_else(0, || 3), &mut 5);
	assert_eq!(block.get_or(1, 100), &mut 10);
	assert_eq!(block.get_or_default(2), &mut 0);
}

#[test]
fn mutable_iteration() {
	let mut block = Block8::<usize>::from([0, 1, 2, 3, 4, 5, 6, 7]);

	// Test `iter_mut()` method
	for val in block.iter_mut() {
		*val *= 2;
	}

	for (idx, &val) in block.iter().enumerate() {
		assert_eq!(idx * 2, val);
	}

	// Test `IntoIterator` for `&mut BlockN<T>`
	for val in &mut block {
		*val += 1;
	}

	for (idx, val) in block.into_iter().enumerate() {
		assert_eq!(idx * 2 + 1, val);
	}
}

#[test]
fn mutable_iteration_partial() {
	let mut block = Block8::<usize>::default();
	block.insert(1, 10);
	block.insert(3, 30);
	block.insert(5, 50);

	// Multiply values by 2
	for val in block.iter_mut() {
		*val *= 2;
	}

	assert_eq!(block.get(0), None);
	assert_eq!(block.get(1), Some(&20));
	assert_eq!(block.get(2), None);
	assert_eq!(block.get(3), Some(&60));
	assert_eq!(block.get(4), None);
	assert_eq!(block.get(5), Some(&100));
	assert_eq!(block.get(6), None);
	assert_eq!(block.get(7), None);
}
