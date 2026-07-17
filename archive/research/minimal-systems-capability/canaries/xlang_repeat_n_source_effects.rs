use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct Tracked<'a> {
    leaf: RefCell<Option<&'a mut u64>>,
    clones: Rc<Cell<usize>>,
    drops: Rc<Cell<usize>>,
}

impl<'a> Tracked<'a> {
    fn new(root: &'a mut u64, clones: Rc<Cell<usize>>, drops: Rc<Cell<usize>>) -> Self {
        Self { leaf: RefCell::new(Some(root)), clones, drops }
    }

    fn has_leaf(&self) -> bool {
        self.leaf.borrow().is_some()
    }
}

impl Clone for Tracked<'_> {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);
        Self {
            leaf: RefCell::new(self.leaf.borrow_mut().take()),
            clones: Rc::clone(&self.clones),
            drops: Rc::clone(&self.drops),
        }
    }
}

impl Drop for Tracked<'_> {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

fn main() {
    // count=0 destroys the seed during construction and calls Clone zero times.
    let zero_clones = Rc::new(Cell::new(0));
    let zero_drops = Rc::new(Cell::new(0));
    let mut zero_root = 0;
    let zero = std::iter::repeat_n(
        Tracked::new(&mut zero_root, Rc::clone(&zero_clones), Rc::clone(&zero_drops)),
        0,
    );
    assert_eq!(zero_clones.get(), 0);
    assert_eq!(zero_drops.get(), 1);
    drop(zero);
    assert_eq!(zero_drops.get(), 1);

    // count=1 moves the retained seed on the only yield and calls Clone zero times.
    let one_clones = Rc::new(Cell::new(0));
    let one_drops = Rc::new(Cell::new(0));
    let mut one_root = 1;
    let mut one = std::iter::repeat_n(
        Tracked::new(&mut one_root, Rc::clone(&one_clones), Rc::clone(&one_drops)),
        1,
    );
    let one_last = one.next().unwrap();
    assert!(one_last.has_leaf());
    assert_eq!(one_clones.get(), 0);
    assert!(one.next().is_none());
    drop(one);
    assert_eq!(one_drops.get(), 0);
    drop(one_last);
    assert_eq!(one_drops.get(), 1);

    // count=3 clones the current retained seed twice, then moves its evolved state.
    let three_clones = Rc::new(Cell::new(0));
    let three_drops = Rc::new(Cell::new(0));
    let mut three_root = 3;
    let mut three = std::iter::repeat_n(
        Tracked::new(&mut three_root, Rc::clone(&three_clones), Rc::clone(&three_drops)),
        3,
    );
    let first = three.next().unwrap();
    let second = three.next().unwrap();
    let final_value = three.next().unwrap();
    assert!(first.has_leaf());
    assert!(!second.has_leaf());
    assert!(!final_value.has_leaf());
    assert_eq!(three_clones.get(), 2);
    assert!(three.next().is_none());
    drop(three);
    assert_eq!(three_drops.get(), 0);
    drop(first);
    drop(second);
    drop(final_value);
    assert_eq!(three_drops.get(), 3);

    println!("PASS repeat_n source effects and final evolved move");
}
