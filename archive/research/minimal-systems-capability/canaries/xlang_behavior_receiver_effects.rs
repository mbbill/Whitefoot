use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Bound, Deref, DerefMut, RangeBounds};

static EXTERNAL_SHARED_ROOT: u8 = 9;

struct Operand<'a> {
    leaf: RefCell<Option<&'a mut u8>>,
}

impl PartialEq for Operand<'_> {
    fn eq(&self, other: &Self) -> bool {
        *other.leaf.borrow_mut() = self.leaf.borrow_mut().take();
        true
    }
}

impl Eq for Operand<'_> {}

impl PartialOrd for Operand<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        *other.leaf.borrow_mut() = self.leaf.borrow_mut().take();
        Some(Ordering::Equal)
    }
}

impl Ord for Operand<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        *other.leaf.borrow_mut() = self.leaf.borrow_mut().take();
        Ordering::Equal
    }
}

struct Bounds<'a> {
    leaf: RefCell<Option<&'a mut u8>>,
}

struct HashOperand<'a> {
    source_leaf: RefCell<Option<&'a mut u8>>,
    destination_leaf: RefCell<Option<&'a mut u8>>,
}

impl Hash for HashOperand<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        *self.destination_leaf.borrow_mut() = self.source_leaf.borrow_mut().take();
        state.write_u8(1);
    }
}

#[derive(Default)]
struct SinkHasher(u64);

impl Hasher for SinkHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 += bytes.len() as u64;
    }
}

impl RangeBounds<usize> for Bounds<'_> {
    fn start_bound(&self) -> Bound<&usize> {
        let _ended = self.leaf.borrow_mut().take();
        Bound::Unbounded
    }

    fn end_bound(&self) -> Bound<&usize> {
        Bound::Unbounded
    }
}

struct Cursor<'a> {
    leaf: RefCell<Option<&'a mut u8>>,
}

struct SharedView<'a> {
    leaf: RefCell<Option<&'a mut u8>>,
}

impl Deref for SharedView<'_> {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        let _ended = self.leaf.borrow_mut().take();
        &EXTERNAL_SHARED_ROOT
    }
}

impl AsRef<u8> for SharedView<'_> {
    fn as_ref(&self) -> &u8 {
        let _ended = self.leaf.borrow_mut().take();
        &EXTERNAL_SHARED_ROOT
    }
}

impl Borrow<u8> for SharedView<'_> {
    fn borrow(&self) -> &u8 {
        let _ended = self.leaf.borrow_mut().take();
        &EXTERNAL_SHARED_ROOT
    }
}

struct UniqueView<'a> {
    leaf: Option<&'a mut u8>,
}

impl Deref for UniqueView<'_> {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &EXTERNAL_SHARED_ROOT
    }
}

impl DerefMut for UniqueView<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.leaf.take().expect("one unique result")
    }
}

impl Borrow<u8> for UniqueView<'_> {
    fn borrow(&self) -> &u8 {
        &EXTERNAL_SHARED_ROOT
    }
}

impl BorrowMut<u8> for UniqueView<'_> {
    fn borrow_mut(&mut self) -> &mut u8 {
        self.leaf.take().expect("one unique result")
    }
}

impl Iterator for Cursor<'_> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let _ended = self.leaf.borrow_mut().take();
        (0, Some(0))
    }
}

struct Payload<'a> {
    source_leaf: RefCell<Option<&'a mut u8>>,
    destination_leaf: RefCell<Option<&'a mut u8>>,
}

fn invoke_predicate<'a, F>(payload: &Payload<'a>, mut predicate: F) -> bool
where
    F: FnMut(&Payload<'a>) -> bool,
{
    predicate(payload)
}

fn main() {
    let mut eq_value = 1;
    let left = Operand { leaf: RefCell::new(Some(&mut eq_value)) };
    let right = Operand { leaf: RefCell::new(None) };
    assert!(left == right);
    assert!(left.leaf.borrow().is_none());
    assert!(right.leaf.borrow().is_some());

    let mut ord_value = 2;
    let low = Operand { leaf: RefCell::new(Some(&mut ord_value)) };
    let high = Operand { leaf: RefCell::new(None) };
    assert_eq!(low.cmp(&high), Ordering::Equal);
    assert!(low.leaf.borrow().is_none());
    assert!(high.leaf.borrow().is_some());

    let mut hash_value = 6;
    let hash_operand = HashOperand {
        source_leaf: RefCell::new(Some(&mut hash_value)),
        destination_leaf: RefCell::new(None),
    };
    let mut hasher = SinkHasher::default();
    hash_operand.hash(&mut hasher);
    assert!(hash_operand.source_leaf.borrow().is_none());
    assert!(hash_operand.destination_leaf.borrow().is_some());

    let mut bound_value = 3;
    let bounds = Bounds { leaf: RefCell::new(Some(&mut bound_value)) };
    assert!(matches!(bounds.start_bound(), Bound::Unbounded));
    assert!(bounds.leaf.borrow().is_none());

    let mut cursor_value = 4;
    let cursor = Cursor { leaf: RefCell::new(Some(&mut cursor_value)) };
    assert_eq!(cursor.size_hint(), (0, Some(0)));
    assert!(cursor.leaf.borrow().is_none());

    let mut shared_view_value = 7;
    let shared_view = SharedView { leaf: RefCell::new(Some(&mut shared_view_value)) };
    assert_eq!(*shared_view, EXTERNAL_SHARED_ROOT);
    assert!(shared_view.leaf.borrow().is_none());

    let mut unique_view_value = 8;
    let mut unique_view = UniqueView { leaf: Some(&mut unique_view_value) };
    *DerefMut::deref_mut(&mut unique_view) = 10;
    assert!(unique_view.leaf.is_none());

    let mut payload_value = 5;
    let payload = Payload {
        source_leaf: RefCell::new(Some(&mut payload_value)),
        destination_leaf: RefCell::new(None),
    };
    let predicate = move |item: &Payload<'_>| {
        *item.destination_leaf.borrow_mut() = item.source_leaf.borrow_mut().take();
        true
    };
    assert!(invoke_predicate(&payload, predicate));
    assert!(payload.source_leaf.borrow().is_none());
    assert!(payload.destination_leaf.borrow().is_some());

    println!("PASS");
}
