//! Basic functionality tests for option-block.
//! These tests cover core operations, iterators, and basic correctness.

use option_block::{Block8, Block16, Block32, Block64, Block128};

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

#[test]
fn empty_block_iteration() {
	let block = Block8::<usize>::default();

	// Empty block should yield no items
	assert_eq!(block.iter().count(), 0);
	assert_eq!(block.into_iter().count(), 0);
}

#[test]
fn empty_block_mutable_iteration() {
	let mut block = Block8::<usize>::default();
	assert_eq!(block.iter_mut().count(), 0);
}

#[test]
fn sparse_block_iteration() {
	let mut block = Block128::<usize>::default();
	// Only populate indices that are prime-ish: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31
	let indices = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
	for &idx in &indices {
		block.insert(idx, idx * 100);
	}

	// Verify we get exactly these elements in order
	let mut iter = block.iter();
	for &idx in &indices {
		assert_eq!(iter.next(), Some(&(idx * 100)));
	}
	assert_eq!(iter.next(), None);
}

#[test]
fn sparse_block_mutable_iteration() {
	let mut block = Block64::<usize>::default();
	let indices = [0, 10, 20, 30, 40, 50, 60, 63];
	for &idx in &indices {
		block.insert(idx, idx);
	}

	// Double all values
	for val in block.iter_mut() {
		*val *= 2;
	}

	// Verify only the expected indices exist and have correct values
	for i in 0..Block64::<usize>::CAPACITY as usize {
		if indices.contains(&i) {
			assert_eq!(block.get(i), Some(&(i * 2)));
		} else {
			assert_eq!(block.get(i), None);
		}
	}
}

#[test]
fn partial_into_iter_consumption() {
	let block = Block8::<usize>::from([0, 1, 2, 3, 4, 5, 6, 7]);
	let mut iter = block.into_iter();

	// Consume only first 3 elements
	assert_eq!(iter.next(), Some(0));
	assert_eq!(iter.next(), Some(1));
	assert_eq!(iter.next(), Some(2));

	// Drop iterator early - remaining elements should be dropped automatically
	drop(iter);
}

#[test]
fn iter_count_matches_length() {
	let block = Block8::<usize>::from([0, 1, 2, 3, 4, 5, 6, 7]);
	assert_eq!(block.iter().count(), 8);
	assert_eq!(block.len(), 8);

	let mut sparse = Block8::<usize>::default();
	sparse.insert(0, 10);
	sparse.insert(3, 30);
	sparse.insert(7, 70);
	assert_eq!(sparse.iter().count(), 3);
	assert_eq!(sparse.len(), 3);
}

#[test]
fn all_iterator_types_agree() {
	let mut block = Block32::<usize>::default();
	// Populate with pattern
	for i in 0..Block32::<usize>::CAPACITY as usize {
		if i % 3 == 0 {
			block.insert(i, i * 10);
		}
	}

	// Count from immutable iterator
	let immutable_count = block.iter().count();

	// Count from mutable iterator (without modifying)
	let mutable_count = block.iter_mut().count();

	assert_eq!(immutable_count, mutable_count);

	// Verify into_iter matches
	let consuming_count = block.into_iter().count();
	assert_eq!(immutable_count, consuming_count);
}

#[test]
fn iterator_correctness_full_block() {
	// Test all block sizes with full population
	let block8 = Block8::<u8>::from([0, 1, 2, 3, 4, 5, 6, 7]);
	assert_eq!(block8.iter().count(), 8);

	let block16 = Block16::<u16>::from([
		0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
	]);
	assert_eq!(block16.iter().count(), 16);

	// Verify values are correct
	for (i, &val) in block8.iter().enumerate() {
		assert_eq!(i as u8, val);
	}
	for (i, &val) in block16.iter().enumerate() {
		assert_eq!(i as u16, val);
	}
}

#[test]
fn mutable_iterator_independence() {
	let mut block = Block8::<usize>::from([0, 1, 2, 3, 4, 5, 6, 7]);

	// Create iterator and modify only some elements
	let mut count = 0;
	for val in block.iter_mut() {
		if *val % 2 == 0 {
			*val += 100;
			count += 1;
		}
	}

	assert_eq!(count, 4); // Modified 0, 2, 4, 6

	// Verify modifications
	assert_eq!(block.get(0), Some(&100));
	assert_eq!(block.get(1), Some(&1));
	assert_eq!(block.get(2), Some(&102));
	assert_eq!(block.get(3), Some(&3));
	assert_eq!(block.get(4), Some(&104));
	assert_eq!(block.get(5), Some(&5));
	assert_eq!(block.get(6), Some(&106));
	assert_eq!(block.get(7), Some(&7));
}

#[test]
fn into_iter_consumes_block() {
	let block = Block8::<usize>::from([10, 20, 30, 40, 50, 60, 70, 80]);
	let mut iter = block.into_iter();

	// Verify all values are yielded in order
	assert_eq!(iter.next(), Some(10));
	assert_eq!(iter.next(), Some(20));
	assert_eq!(iter.next(), Some(30));
	assert_eq!(iter.next(), Some(40));
	assert_eq!(iter.next(), Some(50));
	assert_eq!(iter.next(), Some(60));
	assert_eq!(iter.next(), Some(70));
	assert_eq!(iter.next(), Some(80));
	assert_eq!(iter.next(), None);
	// block is consumed and can't be used anymore (verified by compilation)
}
