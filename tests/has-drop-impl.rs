//! This test module checks the `Drop` implementation for the `Block` variants.
//! Since [`MaybeUninit`](core::mem::MaybeUninit) is used internally, we must
//! manually drop the contents of the blocks.
//!
//! This is separated from the unit tests simply because we require the `alloc`
//! crate to run these tests.

use option_block::{Block8, Block128};

#[test]
fn block_of_optional_strings() {
	let mut block = Block8::<String>::default();

	assert!(block.insert(0, String::from("Hello")).is_none());
	assert!(block.insert(1, String::from("World")).is_none());
	assert!(block.insert(2, String::from("Rust")).is_none());
	assert!(block.insert(7, String::from("Ferris")).is_none());

	use core::ops::Deref;
	assert_eq!(block.get(0).map(Deref::deref), Some("Hello"));
	assert_eq!(block.get(1).map(Deref::deref), Some("World"));
	assert_eq!(block.get(2).map(Deref::deref), Some("Rust"));
	assert!(block.get(3).is_none());
	assert!(block.get(4).is_none());
	assert!(block.get(5).is_none());
	assert!(block.get(6).is_none());
	assert_eq!(block.get(7).map(Deref::deref), Some("Ferris"));

	assert_eq!(block.remove(0).as_deref(), Some("Hello"));
	assert_eq!(block.remove(1).as_deref(), Some("World"));
	assert_eq!(block.remove(2).as_deref(), Some("Rust"));
	assert!(block.remove(3).is_none());
	assert!(block.remove(4).is_none());
	assert!(block.remove(5).is_none());
	assert!(block.remove(6).is_none());
	assert_eq!(block.remove(7).as_deref(), Some("Ferris"));
}

#[test]
fn insert_strings_twice() {
	let mut block = Block8::<String>::default();
	assert!(block.insert(0, String::from("Hello")).is_none());
	assert_eq!(block.insert(0, String::from("World")).as_deref(), Some("Hello"));
}

#[test]
fn ensure_zero_resource_leaks() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Hello World");
	let mut block = Block8::default();
	for i in 0..Block8::<Rc<str>>::CAPACITY as usize {
		assert!(block.insert(i, resource.clone()).is_none());
	}

	assert_eq!(Rc::strong_count(&resource), 9);
	let other = block.clone();
	assert_eq!(Rc::strong_count(&resource), 17);
	drop(block);
	assert_eq!(Rc::strong_count(&resource), 9);
	drop(other);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn partial_cloning() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Hello World");
	let mut block = Block8::default();
	assert!(block.insert(4, resource.clone()).is_none());
	assert!(block.insert(6, resource.clone()).is_none());

	assert_eq!(Rc::strong_count(&resource), 3);
	let other = block.clone();
	assert_eq!(Rc::strong_count(&resource), 5);
	drop(other);
	assert_eq!(Rc::strong_count(&resource), 3);
	drop(block);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn default_getters() {
	use std::rc::Rc;
	let mut block = Block8::<Rc<u8>>::default();
	let resource = Rc::new(10);

	assert!(Rc::ptr_eq(block.get_or_else(0, || resource.clone()), &resource));
	assert_eq!(Rc::strong_count(&resource), 2);
	assert!(Rc::ptr_eq(block.get_or(1, resource.clone()), &resource));
	assert_eq!(Rc::strong_count(&resource), 3);

	let other = block.get_or_default(2).clone();
	assert!(!Rc::ptr_eq(&other, &resource));
	assert_eq!(Rc::strong_count(&resource), 3);
	assert_eq!(Rc::strong_count(&other), 2);

	assert!(Rc::ptr_eq(block.get_or_else(0, || resource.clone()), &resource));
	assert_eq!(Rc::strong_count(&resource), 3);
	assert_eq!(Rc::strong_count(&other), 2);
	assert!(Rc::ptr_eq(block.get_or(1, resource.clone()), &resource));
	assert_eq!(Rc::strong_count(&resource), 3);
	assert_eq!(Rc::strong_count(&other), 2);
	assert!(Rc::ptr_eq(block.get_or_default(2), &other));
	assert_eq!(Rc::strong_count(&resource), 3);
	assert_eq!(Rc::strong_count(&other), 2);

	drop(block);
	assert_eq!(Rc::strong_count(&resource), 1);
	assert_eq!(Rc::strong_count(&other), 1);
}

#[test]
fn iter_does_not_clone() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Iteration Test");
	let block = Block8::from([
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
	]);

	// Block owns 8 references, plus our original = 9 total
	assert_eq!(Rc::strong_count(&resource), 9);

	// Iterating should not change reference count
	for item in block.iter() {
		assert_eq!(Rc::strong_count(item), 9);
	}

	// Still 9 after iteration
	assert_eq!(Rc::strong_count(&resource), 9);

	drop(block);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn iter_mut_does_not_clone() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Mutable Iteration Test");
	let mut block = Block8::from([
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
	]);

	assert_eq!(Rc::strong_count(&resource), 9);

	// Mutable iteration should not change reference count
	for item in block.iter_mut() {
		assert_eq!(Rc::strong_count(item), 9);
	}

	assert_eq!(Rc::strong_count(&resource), 9);

	drop(block);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn into_iter_transfers_ownership() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Consuming Iteration Test");
	let block = Block8::from([
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
	]);

	assert_eq!(Rc::strong_count(&resource), 9);

	// Create iterator - block is consumed but references still alive
	let iter = block.into_iter();
	assert_eq!(Rc::strong_count(&resource), 9);

	// Consume all items
	let items: Vec<_> = iter.collect();
	assert_eq!(items.len(), 8);
	assert_eq!(Rc::strong_count(&resource), 9);

	// Drop items
	drop(items);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn partial_into_iter_drops_remaining() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Partial Consumption");
	let block = Block8::from([
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
	]);

	assert_eq!(Rc::strong_count(&resource), 9);

	let mut iter = block.into_iter();

	// Consume only 3 items
	let first = iter.next().unwrap();
	let second = iter.next().unwrap();
	let third = iter.next().unwrap();

	assert_eq!(Rc::strong_count(&resource), 9); // All still alive

	// Drop the consumed items
	drop(first);
	assert_eq!(Rc::strong_count(&resource), 8);
	drop(second);
	assert_eq!(Rc::strong_count(&resource), 7);
	drop(third);
	assert_eq!(Rc::strong_count(&resource), 6);

	// Drop iterator - should drop remaining 5 items
	drop(iter);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn sparse_into_iter_no_leaks() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Sparse Block Test");
	let mut block = Block8::default();

	// Only populate some indices
	block.insert(1, resource.clone());
	block.insert(3, resource.clone());
	block.insert(7, resource.clone());

	assert_eq!(Rc::strong_count(&resource), 4); // 3 in block + 1 original

	// Consuming iterator should only yield 3 items
	let items: Vec<_> = block.into_iter().collect();
	assert_eq!(items.len(), 3);
	assert_eq!(Rc::strong_count(&resource), 4);

	drop(items);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn empty_into_iter_no_drops() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Empty Block");
	let block = Block8::<Rc<str>>::default();

	assert_eq!(Rc::strong_count(&resource), 1);

	// Empty block into_iter should not affect any references
	let items: Vec<_> = block.into_iter().collect();
	assert_eq!(items.len(), 0);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn iter_mut_modifications_preserved() {
	use std::rc::Rc;
	let resource1 = Rc::<str>::from("First");
	let resource2 = Rc::<str>::from("Second");

	let mut block = Block8::default();
	block.insert(0, resource1.clone());
	block.insert(1, resource1.clone());

	assert_eq!(Rc::strong_count(&resource1), 3);
	assert_eq!(Rc::strong_count(&resource2), 1);

	// Replace all references via mutable iteration
	for item in block.iter_mut() {
		*item = resource2.clone();
	}

	// resource1 references should be dropped (both replaced), resource2 should gain 2 references
	assert_eq!(Rc::strong_count(&resource1), 1);
	assert_eq!(Rc::strong_count(&resource2), 3); // original + 2 in block

	drop(block);
	assert_eq!(Rc::strong_count(&resource2), 1);
}

#[test]
fn large_block_iteration_no_leaks() {
	use std::rc::Rc;
	let resource = Rc::<str>::from("Large Block");
	let mut block = Block128::default();

	// Populate every 10th index: 0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120 (13 items)
	for i in (0..128).step_by(10) {
		block.insert(i, resource.clone());
	}

	let items_in_block = (0..128).step_by(10).count();
	let expected_count = 1 + items_in_block; // original + items in block
	assert_eq!(Rc::strong_count(&resource), expected_count);

	// Iterate and collect
	let items: Vec<_> = block.into_iter().collect();
	assert_eq!(items.len(), items_in_block);
	assert_eq!(Rc::strong_count(&resource), expected_count);

	drop(items);
	assert_eq!(Rc::strong_count(&resource), 1);
}

#[test]
fn into_iter_with_manually_drop_correctness() {
	use std::rc::Rc;
	// This test specifically validates that our ManuallyDrop + ptr::read
	// approach in into_iter doesn't cause double-drops or leaks

	let resource = Rc::<str>::from("ManuallyDrop Test");
	let block = Block8::from([
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
		resource.clone(),
	]);

	assert_eq!(Rc::strong_count(&resource), 9);

	// into_iter should transfer ownership without dropping
	{
		let mut iter = block.into_iter();

		// Consume one at a time and verify counts
		let item0 = iter.next().unwrap();
		assert_eq!(Rc::strong_count(&resource), 9);

		let item1 = iter.next().unwrap();
		assert_eq!(Rc::strong_count(&resource), 9);

		drop(item0);
		assert_eq!(Rc::strong_count(&resource), 8);
		drop(item1);
		assert_eq!(Rc::strong_count(&resource), 7);

		// Iterator still owns 6 more items
		drop(iter);
		assert_eq!(Rc::strong_count(&resource), 1);
	}

	assert_eq!(Rc::strong_count(&resource), 1);
}
