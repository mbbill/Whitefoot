use std::cell::RefCell;
use std::hash::{BuildHasher, Hasher};

struct Builder<'a> {
    leaf: RefCell<Option<&'a mut u64>>,
}

struct Generated<'a> {
    leaf: &'a mut u64,
}

impl Hasher for Generated<'_> {
    fn finish(&self) -> u64 {
        *self.leaf
    }

    fn write(&mut self, bytes: &[u8]) {
        *self.leaf += bytes.len() as u64;
    }
}

impl<'a> BuildHasher for Builder<'a> {
    type Hasher = Generated<'a>;

    fn build_hasher(&self) -> Self::Hasher {
        Generated {
            leaf: self.leaf.borrow_mut().take().unwrap(),
        }
    }
}

fn main() {
    let mut root = 7;
    let builder = Builder {
        leaf: RefCell::new(Some(&mut root)),
    };
    {
        let mut generated = builder.build_hasher();
        generated.write(&[1, 2, 3]);
        assert_eq!(generated.finish(), 10);
    }
    assert!(builder.leaf.borrow().is_none());
    drop(builder);
    assert_eq!(root, 10);
}
