use std::cell::RefCell;

struct Leaf<'a> {
    slot: RefCell<Option<&'a mut u64>>,
}

// The source effect does not require violating Clone's documented equality
// expectation: this type's logical equality deliberately ignores scratch authority.
impl PartialEq for Leaf<'_> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for Leaf<'_> {}

impl<'a> Leaf<'a> {
    fn full(value: &'a mut u64) -> Self {
        Self { slot: RefCell::new(Some(value)) }
    }

    fn empty() -> Self {
        Self { slot: RefCell::new(None) }
    }

    fn has_leaf(&self) -> bool {
        self.slot.borrow().is_some()
    }

    fn increment_leaf(&self) {
        let mut slot = self.slot.borrow_mut();
        **slot.as_mut().unwrap() += 1;
    }
}

impl<'a> Clone for Leaf<'a> {
    fn clone(&self) -> Self {
        // Safe interior mutability moves unique authority out of the shared source.
        Self { slot: RefCell::new(self.slot.borrow_mut().take()) }
    }

    fn clone_from(&mut self, source: &Self) {
        // Safe interior mutability moves unique authority out of the shared source
        // while ending the old destination leaf.
        *self.slot.get_mut() = source.slot.borrow_mut().take();
    }
}

fn main() {
    // Direct Clone::clone: the source remains a valid Leaf but loses its unique leaf.
    let mut direct_root = 10;
    let direct_source = Leaf::full(&mut direct_root);
    let direct_result = direct_source.clone();
    assert!(direct_source == direct_result);
    assert!(!direct_source.has_leaf());
    assert!(direct_result.has_leaf());
    direct_result.increment_leaf();
    drop(direct_result);
    drop(direct_source);
    assert_eq!(direct_root, 11);

    // Stable slice::to_vec propagates the same source effect through T::clone.
    let mut vec_root = 20;
    let source_array = [Leaf::full(&mut vec_root)];
    let cloned_vec = source_array.as_slice().to_vec();
    assert!(!source_array[0].has_leaf());
    assert!(cloned_vec[0].has_leaf());
    cloned_vec[0].increment_leaf();
    drop(cloned_vec);
    drop(source_array);
    assert_eq!(vec_root, 21);

    // Stable slice::clone_from_slice propagates clone_from's source effect.
    let mut source_root = 30;
    let mut old_destination_root = 40;
    let source = [Leaf::full(&mut source_root)];
    let mut destination = [Leaf::full(&mut old_destination_root)];
    destination.clone_from_slice(&source);
    assert!(!source[0].has_leaf());
    assert!(destination[0].has_leaf());
    destination[0].increment_leaf();
    drop(destination);
    drop(source);
    assert_eq!(source_root, 31);
    assert_eq!(old_destination_root, 40);

    // The empty form remains a valid state after authority has moved out.
    let empty = Leaf::empty();
    assert!(!empty.has_leaf());

    println!("PASS clone-source behavior effects");
}
