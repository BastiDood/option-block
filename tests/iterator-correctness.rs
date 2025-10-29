//! Comprehensive iterator correctness tests that require `alloc` for Vec.

use option_block::{Block8, Block32, Block64, Block128};

#[test]
fn sparse_block_iteration() {
	let mut block = Block128::<usize>::default();
	// Only populate indices that are prime-ish: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31
	let indices = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
	for &idx in &indices {
		block.insert(idx, idx * 100);
	}

	// Verify we get exactly these elements
	let collected: Vec<_> = block.iter().copied().collect();
	assert_eq!(collected.len(), indices.len());
	for (&idx, &value) in indices.iter().zip(collected.iter()) {
		assert_eq!(idx * 100, value);
	}
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

	// Collect from immutable iterator
	let immutable: Vec<_> = block.iter().copied().collect();

	// Collect from mutable iterator (without modifying)
	let mutable: Vec<_> = block.iter_mut().map(|v| *v).collect();

	assert_eq!(immutable, mutable);

	// Verify into_iter matches
	let consuming: Vec<_> = block.into_iter().collect();
	assert_eq!(immutable, consuming);
}

#[test]
fn into_iter_consumes_block() {
	let block = Block8::<usize>::from([10, 20, 30, 40, 50, 60, 70, 80]);
	let values: Vec<_> = block.into_iter().collect();

	assert_eq!(values, vec![10, 20, 30, 40, 50, 60, 70, 80]);
	// block is consumed and can't be used anymore (verified by compilation)
}

#[test]
fn partial_into_iter_with_vec() {
	let block = Block64::<usize>::from([
		0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
		25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
		48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
	]);

	let first_half: Vec<_> = block.into_iter().take(32).collect();
	assert_eq!(first_half.len(), 32);
	for (i, &val) in first_half.iter().enumerate() {
		assert_eq!(i, val);
	}
}

#[test]
fn iterator_chaining() {
	let mut block = Block32::<usize>::default();
	for i in 0..Block32::<usize>::CAPACITY as usize {
		if i % 2 == 0 {
			block.insert(i, i);
		}
	}

	// Chain operations: filter, map, collect
	let result: Vec<_> = block.iter().filter(|&&x| x % 4 == 0).map(|&x| x * 2).collect();

	let expected: Vec<_> = (0..32).step_by(4).map(|x| x * 2).collect();
	assert_eq!(result, expected);
}

#[test]
fn mutable_iter_collect_and_verify() {
	let mut block = Block8::<usize>::from([1, 2, 3, 4, 5, 6, 7, 8]);

	// Collect mutable references and modify through them
	let refs: Vec<_> = block.iter_mut().collect();
	assert_eq!(refs.len(), 8);

	// Verify original values in block are still correct
	// (we can't actually modify through the collected Vec<&mut T> after collection
	// because we'd need to iterate again, so this test mainly verifies collection works)
	drop(refs);

	// Now actually modify
	for val in block.iter_mut() {
		*val *= 10;
	}

	let values: Vec<_> = block.into_iter().collect();
	assert_eq!(values, vec![10, 20, 30, 40, 50, 60, 70, 80]);
}
