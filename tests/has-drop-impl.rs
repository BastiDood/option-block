//! This test module checks the `Drop` implementation for the `Block` variants.
//! Since [`MaybeUninit`](core::mem::MaybeUninit) is used internally, we must
//! manually drop the contents of the blocks.

use option_block::Block8;

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
