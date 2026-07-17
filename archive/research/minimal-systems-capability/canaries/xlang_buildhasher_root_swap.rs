use std::cell::{Cell, RefCell};
use std::hash::{BuildHasher, Hasher};

struct Builder<'a> {
    first: &'a u64,
    second: &'a u64,
    current: RefCell<&'a u64>,
    use_second_next: Cell<bool>,
}

struct Generated<'a> {
    seed: &'a u64,
    state: u64,
}

impl Hasher for Generated<'_> {
    fn finish(&self) -> u64 {
        self.state ^ *self.seed
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state = self.state.wrapping_mul(31).wrapping_add(u64::from(*byte));
        }
    }
}

impl<'a> BuildHasher for Builder<'a> {
    type Hasher = Generated<'a>;

    fn build_hasher(&self) -> Self::Hasher {
        let next = if self.use_second_next.replace(!self.use_second_next.get()) {
            self.second
        } else {
            self.first
        };
        *self.current.borrow_mut() = next;
        Generated { seed: next, state: 0 }
    }
}

fn hash(builder: &Builder<'_>, bytes: &[u8]) -> u64 {
    let mut generated = builder.build_hasher();
    generated.write(bytes);
    generated.finish()
}

fn main() {
    let first = 17;
    let second = 17;
    let builder = Builder {
        first: &first,
        second: &second,
        current: RefCell::new(&first),
        use_second_next: Cell::new(false),
    };
    let one = hash(&builder, b"same input");
    assert!(std::ptr::eq(*builder.current.borrow(), &first));
    let two = hash(&builder, b"same input");
    assert!(std::ptr::eq(*builder.current.borrow(), &second));
    assert_eq!(one, two);
}
