use std::cell::RefCell;
use std::rc::Rc;

struct DefaultLeaf<'a> {
    leaf: RefCell<Option<&'a mut u64>>,
}

impl<'a> DefaultLeaf<'a> {
    fn full(root: &'a mut u64) -> Self {
        Self { leaf: RefCell::new(Some(root)) }
    }

    fn has_leaf(&self) -> bool {
        self.leaf.borrow().is_some()
    }
}

impl Clone for DefaultLeaf<'_> {
    fn clone(&self) -> Self {
        Self { leaf: RefCell::new(self.leaf.borrow_mut().take()) }
    }
    // Intentionally inherit Clone::clone_from's default implementation.
}

fn main() {
    // Default clone_from first invokes source.clone(), so the shared source can evolve.
    let mut source_root = 1;
    let mut old_destination_root = 2;
    let source = DefaultLeaf::full(&mut source_root);
    let mut destination = DefaultLeaf::full(&mut old_destination_root);
    destination.clone_from(&source);
    assert!(!source.has_leaf());
    assert!(destination.has_leaf());
    drop(destination);
    drop(source);

    // Array Clone recursively invokes element Clone through shared array elements.
    let mut array_root = 3;
    let source_array = [DefaultLeaf::full(&mut array_root)];
    let result_array = source_array.clone();
    assert!(!source_array[0].has_leaf());
    assert!(result_array[0].has_leaf());
    drop(result_array);
    drop(source_array);

    // Box Clone recursively invokes payload Clone; Box clone_from similarly delegates.
    let mut box_root = 4;
    let source_box = Box::new(DefaultLeaf::full(&mut box_root));
    let result_box = source_box.clone();
    assert!(!source_box.has_leaf());
    assert!(result_box.has_leaf());
    drop(result_box);
    drop(source_box);

    // Rc handle clone is the narrower fixed branch: it changes counts, not payload leaves.
    // Rc::make_mut's multiple-strong branch then invokes payload Clone and can evolve the old
    // allocation visible through the surviving Rc.
    let mut make_mut_root = 5;
    let mut detached = Rc::new(DefaultLeaf::full(&mut make_mut_root));
    let survivor = Rc::clone(&detached);
    assert!(detached.has_leaf());
    assert!(survivor.has_leaf());
    let detached_payload = Rc::make_mut(&mut detached);
    assert!(detached_payload.has_leaf());
    assert!(!survivor.has_leaf());
    drop(detached);
    drop(survivor);

    // Rc::unwrap_or_clone's non-unique fallback has the same source effect.
    let mut unwrap_root = 6;
    let consumed = Rc::new(DefaultLeaf::full(&mut unwrap_root));
    let unwrap_survivor = Rc::clone(&consumed);
    let unwrapped = Rc::unwrap_or_clone(consumed);
    assert!(unwrapped.has_leaf());
    assert!(!unwrap_survivor.has_leaf());
    drop(unwrapped);
    drop(unwrap_survivor);

    println!("PASS default clone_from and array/Box/Rc source effects");
}
