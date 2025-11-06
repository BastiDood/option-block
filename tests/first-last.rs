//! Comprehensive tests for occupied/vacant element access methods.
//!
//! Tests cover:
//! - Occupied: `lowest_occupied_index()`, `highest_occupied_index()`, `first_occupied()`, `last_occupied()`, etc.
//! - Vacant: `lowest_vacant_index()`, `highest_vacant_index()`, `insert_at_first_vacancy()`, `insert_at_last_vacancy()`
//! - Edge cases: empty blocks, full blocks, single elements, boundaries, sparse blocks
//! - Safety: proper Drop handling and no memory leaks (MIRI-compatible)

#![cfg(test)]

extern crate alloc;

use alloc::rc::Rc;
use option_block::Block8;

mod empty_block_tests {
	use super::*;

	#[test]
	fn first_returns_none() {
		let block = Block8::<u32>::new();
		assert!(block.first_occupied().is_none());
	}

	#[test]
	fn first_mut_returns_none() {
		let mut block = Block8::<u32>::new();
		assert!(block.first_occupied_mut().is_none());
	}

	#[test]
	fn last_returns_none() {
		let block = Block8::<u32>::new();
		assert!(block.last_occupied().is_none());
	}

	#[test]
	fn last_mut_returns_none() {
		let mut block = Block8::<u32>::new();
		assert!(block.last_occupied_mut().is_none());
	}

	#[test]
	fn lowest_index_returns_none() {
		let block = Block8::<u32>::new();
		assert!(block.lowest_occupied_index().is_none());
	}

	#[test]
	fn highest_index_returns_none() {
		let block = Block8::<u32>::new();
		assert!(block.highest_occupied_index().is_none());
	}
}

mod single_element_tests {
	use super::*;

	#[test]
	fn at_index_zero() {
		let mut block = Block8::new();
		block.insert(0, 42);

		assert_eq!(block.first_occupied(), Some(&42));
		assert_eq!(block.last_occupied(), Some(&42));
		assert_eq!(block.lowest_occupied_index(), Some(0));
		assert_eq!(block.highest_occupied_index(), Some(0));
	}

	#[test]
	fn at_middle_index() {
		let mut block = Block8::new();
		block.insert(3, 100);

		assert_eq!(block.first_occupied(), Some(&100));
		assert_eq!(block.last_occupied(), Some(&100));
		assert_eq!(block.lowest_occupied_index(), Some(3));
		assert_eq!(block.highest_occupied_index(), Some(3));
	}

	#[test]
	fn at_last_index() {
		let mut block = Block8::new();
		block.insert(7, 999);

		assert_eq!(block.first_occupied(), Some(&999));
		assert_eq!(block.last_occupied(), Some(&999));
		assert_eq!(block.lowest_occupied_index(), Some(7));
		assert_eq!(block.highest_occupied_index(), Some(7));
	}

	#[test]
	fn first_and_last_are_same_reference() {
		let mut block = Block8::new();
		block.insert(4, 555);

		let first_ref = block.first_occupied().unwrap();
		let last_ref = block.last_occupied().unwrap();

		assert_eq!(first_ref, last_ref);
		assert_eq!(*first_ref, 555);
	}
}

mod boundary_position_tests {
	use super::*;

	#[test]
	fn elements_only_at_boundaries() {
		let mut block = Block8::new();
		block.insert(0, 10);
		block.insert(7, 20);

		assert_eq!(block.first_occupied(), Some(&10));
		assert_eq!(block.last_occupied(), Some(&20));
		assert_eq!(block.lowest_occupied_index(), Some(0));
		assert_eq!(block.highest_occupied_index(), Some(7));
	}

	#[test]
	fn element_only_at_first_boundary() {
		let mut block = Block8::new();
		block.insert(0, 777);

		assert_eq!(block.first_occupied(), Some(&777));
		assert_eq!(block.last_occupied(), Some(&777));
		assert_eq!(block.lowest_occupied_index(), Some(0));
		assert_eq!(block.highest_occupied_index(), Some(0));
	}

	#[test]
	fn element_only_at_last_boundary() {
		let mut block = Block8::new();
		block.insert(7, 888);

		assert_eq!(block.first_occupied(), Some(&888));
		assert_eq!(block.last_occupied(), Some(&888));
		assert_eq!(block.lowest_occupied_index(), Some(7));
		assert_eq!(block.highest_occupied_index(), Some(7));
	}
}

mod sparse_block_tests {
	use super::*;

	#[test]
	fn with_gaps() {
		let mut block = Block8::new();
		block.insert(2, 200);
		block.insert(5, 500);
		block.insert(7, 700);

		assert_eq!(block.first_occupied(), Some(&200));
		assert_eq!(block.last_occupied(), Some(&700));
		assert_eq!(block.lowest_occupied_index(), Some(2));
		assert_eq!(block.highest_occupied_index(), Some(7));
	}

	#[test]
	fn alternating_pattern() {
		let mut block = Block8::new();
		block.insert(1, 1);
		block.insert(3, 3);
		block.insert(5, 5);

		assert_eq!(block.first_occupied(), Some(&1));
		assert_eq!(block.last_occupied(), Some(&5));
		assert_eq!(block.lowest_occupied_index(), Some(1));
		assert_eq!(block.highest_occupied_index(), Some(5));
	}

	#[test]
	fn clustered_at_start() {
		let mut block = Block8::new();
		block.insert(0, 10);
		block.insert(1, 11);
		block.insert(2, 12);
		block.insert(6, 60);

		assert_eq!(block.first_occupied(), Some(&10));
		assert_eq!(block.last_occupied(), Some(&60));
		assert_eq!(block.lowest_occupied_index(), Some(0));
		assert_eq!(block.highest_occupied_index(), Some(6));
	}

	#[test]
	fn clustered_at_end() {
		let mut block = Block8::new();
		block.insert(1, 10);
		block.insert(5, 50);
		block.insert(6, 60);
		block.insert(7, 70);

		assert_eq!(block.first_occupied(), Some(&10));
		assert_eq!(block.last_occupied(), Some(&70));
		assert_eq!(block.lowest_occupied_index(), Some(1));
		assert_eq!(block.highest_occupied_index(), Some(7));
	}
}

mod full_block_tests {
	use super::*;

	#[test]
	fn first_and_last() {
		let mut block = Block8::new();
		for i in 0..8 {
			block.insert(i, i * 10);
		}

		assert_eq!(block.first_occupied(), Some(&0));
		assert_eq!(block.last_occupied(), Some(&70));
		assert_eq!(block.lowest_occupied_index(), Some(0));
		assert_eq!(block.highest_occupied_index(), Some(7));
	}

	#[test]
	fn all_same_value() {
		let mut block = Block8::new();
		for i in 0..8 {
			block.insert(i, 42);
		}

		assert_eq!(block.first_occupied(), Some(&42));
		assert_eq!(block.last_occupied(), Some(&42));
		assert_eq!(block.lowest_occupied_index(), Some(0));
		assert_eq!(block.highest_occupied_index(), Some(7));
	}
}

mod post_removal_tests {
	use super::*;

	#[test]
	fn after_removing_first_element() {
		let mut block = Block8::new();
		block.insert(0, 10);
		block.insert(3, 30);
		block.insert(5, 50);

		block.remove(0);

		assert_eq!(block.first_occupied(), Some(&30));
		assert_eq!(block.last_occupied(), Some(&50));
		assert_eq!(block.lowest_occupied_index(), Some(3));
		assert_eq!(block.highest_occupied_index(), Some(5));
	}

	#[test]
	fn after_removing_last_element() {
		let mut block = Block8::new();
		block.insert(1, 10);
		block.insert(4, 40);
		block.insert(7, 70);

		block.remove(7);

		assert_eq!(block.first_occupied(), Some(&10));
		assert_eq!(block.last_occupied(), Some(&40));
		assert_eq!(block.lowest_occupied_index(), Some(1));
		assert_eq!(block.highest_occupied_index(), Some(4));
	}

	#[test]
	fn after_removing_middle_elements() {
		let mut block = Block8::new();
		block.insert(0, 10);
		block.insert(3, 30);
		block.insert(5, 50);
		block.insert(7, 70);

		block.remove(3);
		block.remove(5);

		assert_eq!(block.first_occupied(), Some(&10));
		assert_eq!(block.last_occupied(), Some(&70));
		assert_eq!(block.lowest_occupied_index(), Some(0));
		assert_eq!(block.highest_occupied_index(), Some(7));
	}

	#[test]
	fn after_removing_all_but_one() {
		let mut block = Block8::new();
		block.insert(0, 10);
		block.insert(3, 30);
		block.insert(7, 70);

		block.remove(0);
		block.remove(7);

		assert_eq!(block.first_occupied(), Some(&30));
		assert_eq!(block.last_occupied(), Some(&30));
		assert_eq!(block.lowest_occupied_index(), Some(3));
		assert_eq!(block.highest_occupied_index(), Some(3));
	}

	#[test]
	fn after_removing_all_elements() {
		let mut block = Block8::new();
		block.insert(2, 20);
		block.insert(5, 50);

		block.remove(2);
		block.remove(5);

		assert!(block.first_occupied().is_none());
		assert!(block.last_occupied().is_none());
		assert!(block.lowest_occupied_index().is_none());
		assert!(block.highest_occupied_index().is_none());
	}

	#[test]
	fn sequential_removal_from_front() {
		let mut block = Block8::new();
		for i in 0..5 {
			block.insert(i, i * 10);
		}

		// Remove 0, 1, 2 sequentially
		block.remove(0);
		assert_eq!(block.lowest_occupied_index(), Some(1));

		block.remove(1);
		assert_eq!(block.lowest_occupied_index(), Some(2));

		block.remove(2);
		assert_eq!(block.lowest_occupied_index(), Some(3));
		assert_eq!(block.first_occupied(), Some(&30));
	}

	#[test]
	fn sequential_removal_from_back() {
		let mut block = Block8::new();
		for i in 3..8 {
			block.insert(i, i * 10);
		}

		// Remove 7, 6, 5 sequentially
		block.remove(7);
		assert_eq!(block.highest_occupied_index(), Some(6));

		block.remove(6);
		assert_eq!(block.highest_occupied_index(), Some(5));

		block.remove(5);
		assert_eq!(block.highest_occupied_index(), Some(4));
		assert_eq!(block.last_occupied(), Some(&40));
	}
}

mod mutation_tests {
	use super::*;

	#[test]
	fn first_mut_modifies_first_element() {
		let mut block = Block8::new();
		block.insert(2, 20);
		block.insert(5, 50);
		block.insert(7, 70);

		if let Some(first) = block.first_occupied_mut() {
			*first = 999;
		}

		assert_eq!(block.get(2), Some(&999));
		assert_eq!(block.first_occupied(), Some(&999));
		// Other elements unchanged
		assert_eq!(block.get(5), Some(&50));
		assert_eq!(block.get(7), Some(&70));
	}

	#[test]
	fn last_mut_modifies_last_element() {
		let mut block = Block8::new();
		block.insert(1, 10);
		block.insert(4, 40);
		block.insert(6, 60);

		if let Some(last) = block.last_occupied_mut() {
			*last = 888;
		}

		assert_eq!(block.get(6), Some(&888));
		assert_eq!(block.last_occupied(), Some(&888));
		// Other elements unchanged
		assert_eq!(block.get(1), Some(&10));
		assert_eq!(block.get(4), Some(&40));
	}

	#[test]
	fn first_mut_on_single_element() {
		let mut block = Block8::new();
		block.insert(3, 30);

		if let Some(val) = block.first_occupied_mut() {
			*val += 5;
		}

		assert_eq!(block.first_occupied(), Some(&35));
		assert_eq!(block.last_occupied(), Some(&35));
	}

	#[test]
	fn last_mut_on_single_element() {
		let mut block = Block8::new();
		block.insert(5, 50);

		if let Some(val) = block.last_occupied_mut() {
			*val *= 2;
		}

		assert_eq!(block.first_occupied(), Some(&100));
		assert_eq!(block.last_occupied(), Some(&100));
	}

	#[test]
	fn alternating_first_and_last_mut() {
		let mut block = Block8::new();
		block.insert(0, 1);
		block.insert(7, 2);

		if let Some(first) = block.first_occupied_mut() {
			*first = 10;
		}
		assert_eq!(block.get(0), Some(&10));

		if let Some(last) = block.last_occupied_mut() {
			*last = 20;
		}
		assert_eq!(block.get(7), Some(&20));

		assert_eq!(block.first_occupied(), Some(&10));
		assert_eq!(block.last_occupied(), Some(&20));
	}
}

mod drop_safety_tests {
	use super::*;

	#[test]
	fn first_does_not_drop_element() {
		let val = Rc::new(42);
		let mut block = Block8::new();
		block.insert(0, Rc::clone(&val));

		assert_eq!(Rc::strong_count(&val), 2);

		let _ = block.first_occupied();
		assert_eq!(Rc::strong_count(&val), 2); // No drop occurred

		let _ = block.first_occupied();
		assert_eq!(Rc::strong_count(&val), 2); // Still no drop
	}

	#[test]
	fn last_does_not_drop_element() {
		let val = Rc::new(100);
		let mut block = Block8::new();
		block.insert(7, Rc::clone(&val));

		assert_eq!(Rc::strong_count(&val), 2);

		let _ = block.last_occupied();
		assert_eq!(Rc::strong_count(&val), 2); // No drop occurred

		let _ = block.last_occupied();
		assert_eq!(Rc::strong_count(&val), 2); // Still no drop
	}

	#[test]
	fn first_mut_does_not_cause_leaks() {
		let val1 = Rc::new(1);
		let val2 = Rc::new(2);
		let mut block = Block8::new();
		block.insert(0, Rc::clone(&val1));
		block.insert(5, Rc::clone(&val2));

		assert_eq!(Rc::strong_count(&val1), 2);

		if let Some(first) = block.first_occupied_mut() {
			*first = Rc::clone(&val2); // Replace val1 with val2
		}

		// val1 should be dropped, val2 now has 3 references
		assert_eq!(Rc::strong_count(&val1), 1);
		assert_eq!(Rc::strong_count(&val2), 3);
	}

	#[test]
	fn last_mut_does_not_cause_leaks() {
		let val1 = Rc::new(10);
		let val2 = Rc::new(20);
		let mut block = Block8::new();
		block.insert(2, Rc::clone(&val1));
		block.insert(7, Rc::clone(&val2));

		assert_eq!(Rc::strong_count(&val2), 2);

		if let Some(last) = block.last_occupied_mut() {
			*last = Rc::clone(&val1); // Replace val2 with val1
		}

		// val2 should be dropped, val1 now has 3 references
		assert_eq!(Rc::strong_count(&val2), 1);
		assert_eq!(Rc::strong_count(&val1), 3);
	}

	#[test]
	fn multiple_first_last_accesses_no_double_drop() {
		let val1 = Rc::new(111);
		let val2 = Rc::new(222);
		let mut block = Block8::new();
		block.insert(0, Rc::clone(&val1));
		block.insert(7, Rc::clone(&val2));

		for _ in 0..10 {
			let _ = block.first_occupied();
			let _ = block.last_occupied();
		}

		assert_eq!(Rc::strong_count(&val1), 2);
		assert_eq!(Rc::strong_count(&val2), 2);
	}
}

mod index_correctness_tests {
	use super::*;

	#[test]
	fn lowest_index_returns_correct_u32() {
		let mut block = Block8::new();

		for i in 0..8 {
			block.insert(i, i * 10);
			assert_eq!(block.lowest_occupied_index(), Some(0));
		}
	}

	#[test]
	fn highest_index_returns_correct_u32() {
		let mut block = Block8::new();

		for i in 0..8 {
			block.insert(i, i * 10);
			assert_eq!(block.highest_occupied_index(), Some(i as u32));
		}
	}

	#[test]
	fn index_values_are_within_capacity() {
		let mut block = Block8::new();
		block.insert(0, 1);
		block.insert(3, 2);
		block.insert(7, 3);

		if let Some(idx) = block.lowest_occupied_index() {
			assert!(idx < Block8::<u32>::CAPACITY);
		}

		if let Some(idx) = block.highest_occupied_index() {
			assert!(idx < Block8::<u32>::CAPACITY);
		}
	}

	#[test]
	fn index_can_be_safely_cast_to_usize() {
		let mut block = Block8::new();
		block.insert(5, 50);

		if let Some(idx) = block.lowest_occupied_index() {
			let usize_idx = idx as usize;
			assert_eq!(block.get(usize_idx), Some(&50));
		}

		if let Some(idx) = block.highest_occupied_index() {
			let usize_idx = idx as usize;
			assert_eq!(block.get(usize_idx), Some(&50));
		}
	}

	#[test]
	fn lowest_and_highest_index_consistency() {
		let mut block = Block8::new();
		block.insert(2, 20);
		block.insert(3, 30);
		block.insert(6, 60);

		let lowest = block.lowest_occupied_index().unwrap();
		let highest = block.highest_occupied_index().unwrap();

		assert!(lowest <= highest);
		assert_eq!(lowest, 2);
		assert_eq!(highest, 6);
	}
}

mod vacant_index_tests {
	use super::*;

	#[test]
	fn empty_block_vacant_indices() {
		let block = Block8::<u32>::new();

		// Empty block: first vacancy is at 0, last is at 7
		assert_eq!(block.lowest_vacant_index(), Some(0));
		assert_eq!(block.highest_vacant_index(), Some(7));
	}

	#[test]
	fn full_block_no_vacancies() {
		let mut block = Block8::new();
		for i in 0..8 {
			block.insert(i, i * 10);
		}

		assert_eq!(block.lowest_vacant_index(), None);
		assert_eq!(block.highest_vacant_index(), None);
	}

	#[test]
	fn single_element_vacant_indices() {
		let mut block = Block8::new();
		block.insert(3, 30);

		// First vacancy should be 0 (before occupied slot)
		assert_eq!(block.lowest_vacant_index(), Some(0));
		// Last vacancy should be 7 (after occupied slot)
		assert_eq!(block.highest_vacant_index(), Some(7));
	}

	#[test]
	fn sparse_block_vacant_indices() {
		let mut block = Block8::new();
		block.insert(2, 20);
		block.insert(5, 50);
		block.insert(7, 70);

		// Vacancies at: 0, 1, 3, 4, 6
		assert_eq!(block.lowest_vacant_index(), Some(0));
		assert_eq!(block.highest_vacant_index(), Some(6));
	}

	#[test]
	fn vacancy_at_boundaries() {
		let mut block = Block8::new();
		// Fill middle, leave boundaries vacant
		for i in 1..7 {
			block.insert(i, i * 10);
		}

		assert_eq!(block.lowest_vacant_index(), Some(0));
		assert_eq!(block.highest_vacant_index(), Some(7));
	}

	#[test]
	fn single_vacancy_at_start() {
		let mut block = Block8::new();
		for i in 1..8 {
			block.insert(i, i * 10);
		}

		assert_eq!(block.lowest_vacant_index(), Some(0));
		assert_eq!(block.highest_vacant_index(), Some(0));
	}

	#[test]
	fn single_vacancy_in_middle() {
		let mut block = Block8::new();
		for i in 0..8 {
			if i != 4 {
				block.insert(i, i * 10);
			}
		}

		assert_eq!(block.lowest_vacant_index(), Some(4));
		assert_eq!(block.highest_vacant_index(), Some(4));
	}

	#[test]
	fn single_vacancy_at_end() {
		let mut block = Block8::new();
		for i in 0..7 {
			block.insert(i, i * 10);
		}

		assert_eq!(block.lowest_vacant_index(), Some(7));
		assert_eq!(block.highest_vacant_index(), Some(7));
	}

	#[test]
	fn vacancy_tracking_after_removal() {
		let mut block = Block8::new();
		for i in 0..8 {
			block.insert(i, i * 10);
		}

		// Remove some elements
		block.remove(2);
		block.remove(6);

		// Vacancies at 2 and 6
		assert_eq!(block.lowest_vacant_index(), Some(2));
		assert_eq!(block.highest_vacant_index(), Some(6));
	}
}

mod insert_at_first_vacancy_tests {
	use super::*;

	#[test]
	fn empty_block() {
		let mut block = Block8::new();

		let result = block.insert_at_first_vacancy(100);
		assert_eq!(result, Ok(None)); // Successfully inserted, no previous value
		assert_eq!(block.get(0), Some(&100));
		assert_eq!(block.len(), 1);
	}

	#[test]
	fn full_block() {
		let mut block = Block8::new();
		for i in 0..8 {
			block.insert(i, i * 10);
		}

		let result = block.insert_at_first_vacancy(999);
		assert_eq!(result, Err(999)); // Block is full, value returned
		assert_eq!(block.len(), 8);
	}

	#[test]
	fn sparse_block() {
		let mut block = Block8::new();
		block.insert(3, 30);
		block.insert(5, 50);
		block.insert(7, 70);

		// First vacancy is at index 0
		let result = block.insert_at_first_vacancy(100);
		assert_eq!(result, Ok(None));
		assert_eq!(block.get(0), Some(&100));
		assert_eq!(block.len(), 4);
	}

	#[test]
	fn sequential() {
		let mut block = Block8::new();

		// Insert 5 elements sequentially from first vacancy
		for i in 0..5 {
			let result = block.insert_at_first_vacancy((i * 10) as u32);
			assert_eq!(result, Ok(None));
			assert_eq!(block.len(), i + 1);
		}

		// Verify they were inserted at indices 0-4
		for i in 0..5 {
			assert_eq!(block.get(i), Some(&((i * 10) as u32)));
		}
	}

	#[test]
	fn after_removal() {
		let mut block = Block8::new();
		block.insert(0, 10);
		block.insert(1, 20);
		block.insert(2, 30);

		// Remove first element
		block.remove(0);

		// First vacancy should now be at 0
		let result = block.insert_at_first_vacancy(999);
		assert_eq!(result, Ok(None));
		assert_eq!(block.get(0), Some(&999));
	}

	#[test]
	fn replace_existing() {
		let mut block = Block8::new();
		block.insert(1, 10);
		block.insert(2, 20);

		// First vacancy at 0, insert there
		let result = block.insert_at_first_vacancy(100);
		assert_eq!(result, Ok(None));

		// Now fill the rest except index 3
		block.insert(4, 40);
		block.insert(5, 50);
		block.insert(6, 60);
		block.insert(7, 70);

		// First vacancy at 3, but let's say it was previously occupied
		// (This scenario tests if insert returns Some when replacing)
		block.insert(3, 30);
		block.remove(3); // Create vacancy again

		let result = block.insert_at_first_vacancy(300);
		assert_eq!(result, Ok(None)); // No previous value at this vacant slot
		assert_eq!(block.get(3), Some(&300));
	}

	#[test]
	fn fills_gaps() {
		let mut block = Block8::new();
		block.insert(2, 20);
		block.insert(5, 50);

		// Gaps at: 0, 1, 3, 4, 6, 7
		let result1 = block.insert_at_first_vacancy(100);
		assert_eq!(result1, Ok(None));
		assert_eq!(block.get(0), Some(&100));

		let result2 = block.insert_at_first_vacancy(200);
		assert_eq!(result2, Ok(None));
		assert_eq!(block.get(1), Some(&200));
	}
}

mod insert_at_last_vacancy_tests {
	use super::*;

	#[test]
	fn empty_block() {
		let mut block = Block8::new();

		let result = block.insert_at_last_vacancy(100);
		assert_eq!(result, Ok(None));
		assert_eq!(block.get(7), Some(&100));
		assert_eq!(block.len(), 1);
	}

	#[test]
	fn full_block() {
		let mut block = Block8::new();
		for i in 0..8 {
			block.insert(i, i * 10);
		}

		let result = block.insert_at_last_vacancy(999);
		assert_eq!(result, Err(999));
		assert_eq!(block.len(), 8);
	}

	#[test]
	fn sparse_block() {
		let mut block = Block8::new();
		block.insert(0, 10);
		block.insert(2, 20);
		block.insert(4, 40);

		// Last vacancy is at index 7
		let result = block.insert_at_last_vacancy(700);
		assert_eq!(result, Ok(None));
		assert_eq!(block.get(7), Some(&700));
		assert_eq!(block.len(), 4);
	}

	#[test]
	fn sequential() {
		let mut block = Block8::new();

		// Insert 5 elements sequentially from last vacancy
		for i in 0..5 {
			let result = block.insert_at_last_vacancy(((7 - i) * 10) as u32);
			assert_eq!(result, Ok(None));
			assert_eq!(block.len(), i + 1);
		}

		// Verify they were inserted at indices 7, 6, 5, 4, 3
		for i in 0..5 {
			assert_eq!(block.get(7 - i), Some(&(((7 - i) * 10) as u32)));
		}
	}

	#[test]
	fn after_removal() {
		let mut block = Block8::new();
		block.insert(5, 50);
		block.insert(6, 60);
		block.insert(7, 70);

		// Remove last element
		block.remove(7);

		// Last vacancy should now be at 7
		let result = block.insert_at_last_vacancy(777);
		assert_eq!(result, Ok(None));
		assert_eq!(block.get(7), Some(&777));
	}

	#[test]
	fn fills_gaps_from_end() {
		let mut block = Block8::new();
		block.insert(1, 10);
		block.insert(4, 40);

		// Gaps at: 0, 2, 3, 5, 6, 7
		let result1 = block.insert_at_last_vacancy(700);
		assert_eq!(result1, Ok(None));
		assert_eq!(block.get(7), Some(&700));

		let result2 = block.insert_at_last_vacancy(600);
		assert_eq!(result2, Ok(None));
		assert_eq!(block.get(6), Some(&600));

		let result3 = block.insert_at_last_vacancy(500);
		assert_eq!(result3, Ok(None));
		assert_eq!(block.get(5), Some(&500));
	}

	#[test]
	fn first_and_last_vacancy_meet_in_middle() {
		let mut block = Block8::new();

		// Insert from first vacancy 3 times
		for _ in 0..3 {
			let _ = block.insert_at_first_vacancy(1);
		}

		// Insert from last vacancy 3 times
		for _ in 0..3 {
			let _ = block.insert_at_last_vacancy(2);
		}

		assert_eq!(block.len(), 6);

		// Two vacancies left at indices 3 and 4
		assert_eq!(block.lowest_vacant_index(), Some(3));
		assert_eq!(block.highest_vacant_index(), Some(4));
	}
}

mod vacant_insert_drop_safety_tests {
	use super::*;

	#[test]
	fn insert_at_first_vacancy_no_leak_on_err() {
		let val = Rc::new(42);
		let mut block = Block8::new();

		// Fill the block
		for i in 0..8 {
			block.insert(i, Rc::new(i));
		}

		assert_eq!(Rc::strong_count(&val), 1);

		// Try to insert into full block
		let result = block.insert_at_first_vacancy(Rc::clone(&val));

		// Should get Err with the value back
		assert!(result.is_err());
		assert_eq!(Rc::strong_count(&val), 2); // Original + returned value

		// Drop the returned value
		drop(result);
		assert_eq!(Rc::strong_count(&val), 1); // Back to original only
	}

	#[test]
	fn insert_at_last_vacancy_no_leak_on_err() {
		let val = Rc::new(100);
		let mut block = Block8::new();

		// Fill the block
		for i in 0..8 {
			block.insert(i, Rc::new(i));
		}

		assert_eq!(Rc::strong_count(&val), 1);

		// Try to insert into full block
		let result = block.insert_at_last_vacancy(Rc::clone(&val));

		assert!(result.is_err());
		assert_eq!(Rc::strong_count(&val), 2);

		drop(result);
		assert_eq!(Rc::strong_count(&val), 1);
	}

	#[test]
	fn insert_at_first_vacancy_proper_ownership() {
		let val = Rc::new(123);
		let mut block = Block8::new();

		assert_eq!(Rc::strong_count(&val), 1);

		let result = block.insert_at_first_vacancy(Rc::clone(&val));
		assert_eq!(result, Ok(None));

		// Value is now owned by the block
		assert_eq!(Rc::strong_count(&val), 2);

		// Verify it's retrievable
		assert_eq!(block.get(0).map(|rc| **rc), Some(123));
	}

	#[test]
	fn insert_at_last_vacancy_proper_ownership() {
		let val = Rc::new(456);
		let mut block = Block8::new();

		assert_eq!(Rc::strong_count(&val), 1);

		let result = block.insert_at_last_vacancy(Rc::clone(&val));
		assert_eq!(result, Ok(None));

		assert_eq!(Rc::strong_count(&val), 2);
		assert_eq!(block.get(7).map(|rc| **rc), Some(456));
	}

	#[test]
	fn vacant_insert_replaces_correctly() {
		let val1 = Rc::new(1);
		let val2 = Rc::new(2);
		let mut block = Block8::new();

		// Insert val1 at index 3
		block.insert(3, Rc::clone(&val1));
		assert_eq!(Rc::strong_count(&val1), 2);

		// Remove it to create vacancy
		block.remove(3);
		assert_eq!(Rc::strong_count(&val1), 1); // Dropped from block

		// Now insert at first vacancy (which might be index 0 or 3 depending on mask)
		// Let's make sure we know where first vacancy is
		let first_vacant = block.lowest_vacant_index().unwrap() as usize;

		let result = block.insert_at_first_vacancy(Rc::clone(&val2));
		assert_eq!(result, Ok(None)); // No old value at vacant slot
		assert_eq!(Rc::strong_count(&val2), 2);
		assert_eq!(block.get(first_vacant).map(|rc| **rc), Some(2));
	}
}
