#![forbid(unsafe_code)]
//! Safe Rust control for reusable fixed-capacity construction backing.
//!
//! Keep this probe until a matched Whitefoot fixed-block experiment replaces it.

use std::cell::{Cell, RefCell};
use std::convert::TryInto;
use std::rc::Rc;

const REPEATED_CHECKOUTS: usize = 3;

#[derive(Default)]
struct DropLedger {
    created: Cell<usize>,
    dropped: Cell<usize>,
    drops_by_id: RefCell<Vec<usize>>,
}

impl DropLedger {
    fn make(self: &Rc<Self>) -> Tracked {
        let id = self.created.get();
        self.created.set(id + 1);
        self.drops_by_id.borrow_mut().push(0);
        Tracked {
            id,
            ledger: Rc::clone(self),
        }
    }

    fn assert_exactly_once(&self) {
        assert_eq!(self.dropped.get(), self.created.get());
        assert!(self.drops_by_id.borrow().iter().all(|drops| *drops == 1));
    }
}

struct Tracked {
    id: usize,
    ledger: Rc<DropLedger>,
}

impl Drop for Tracked {
    fn drop(&mut self) {
        let mut drops = self.ledger.drops_by_id.borrow_mut();
        drops[self.id] += 1;
        assert_eq!(
            drops[self.id], 1,
            "value {} dropped more than once",
            self.id
        );
        self.ledger.dropped.set(self.ledger.dropped.get() + 1);
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Observation {
    extent: usize,
    reserved_capacity: usize,
    failure_prefixes: usize,
    incomplete_conversions_returning_pointer: usize,
    failure_clears_retaining_pointer: usize,
    exact_capacity_conversions: usize,
    vec_to_box_retaining_pointer: usize,
    box_to_vec_retaining_pointer: usize,
    checkouts: usize,
    values_created_and_dropped: usize,
}

fn run_extent<const N: usize>() -> Observation {
    let ledger = Rc::new(DropLedger::default());
    let mut slot = Vec::new();
    slot.try_reserve_exact(N)
        .expect("the small research control must reserve its slot");

    let reserved_capacity = slot.capacity();
    assert!(reserved_capacity >= N);
    let reserved_pointer = slot.as_ptr();
    let mut incomplete_conversions_returning_pointer = 0;
    let mut failure_clears_retaining_pointer = 0;

    // Model a producer refusing after every possible initialized prefix. The
    // prefix contains only valid values, so safe clear performs exact cleanup
    // and returns the still-allocated Vec for the next checkout.
    for prefix in 0..=N {
        assert!(slot.is_empty());
        assert_eq!(slot.as_ptr(), reserved_pointer);
        let start_id = ledger.created.get();
        for _ in 0..prefix {
            slot.push(ledger.make());
        }
        assert_eq!(slot.len(), prefix);
        assert_eq!(slot.as_ptr(), reserved_pointer);
        assert!(
            slot.iter()
                .enumerate()
                .all(|(offset, value)| value.id == start_id + offset)
        );

        if prefix < N {
            let returned = match <Vec<Tracked> as TryInto<Box<[Tracked; N]>>>::try_into(slot) {
                Ok(_) => panic!("an incomplete Vec must not publish as a full array"),
                Err(returned) => returned,
            };
            if returned.as_ptr() == reserved_pointer {
                incomplete_conversions_returning_pointer += 1;
            }
            assert_eq!(returned.len(), prefix);
            assert_eq!(returned.capacity(), reserved_capacity);
            slot = returned;
        }

        slot.clear();
        assert!(slot.is_empty());
        assert_eq!(slot.capacity(), reserved_capacity);
        if slot.as_ptr() == reserved_pointer {
            failure_clears_retaining_pointer += 1;
        }
        ledger.assert_exactly_once();
    }

    let mut exact_capacity_conversions = 0;
    let mut vec_to_box_retaining_pointer = 0;
    let mut box_to_vec_retaining_pointer = 0;

    for _ in 0..REPEATED_CHECKOUTS {
        assert!(slot.is_empty());
        let checkout_pointer = slot.as_ptr();
        let start_id = ledger.created.get();
        for _ in 0..N {
            slot.push(ledger.make());
        }

        let capacity_before_conversion = slot.capacity();
        let vector_pointer = slot.as_ptr();
        assert_eq!(vector_pointer, checkout_pointer);
        let had_exact_capacity = slot.len() == capacity_before_conversion;
        if had_exact_capacity {
            exact_capacity_conversions += 1;
        }

        let complete: Box<[Tracked; N]> = match slot.try_into() {
            Ok(complete) => complete,
            Err(_) => panic!("a complete Vec must publish as a matching boxed array"),
        };
        assert!(
            complete
                .iter()
                .enumerate()
                .all(|(offset, value)| value.id == start_id + offset)
        );
        let boxed_pointer = complete.as_ptr();
        if boxed_pointer == vector_pointer {
            vec_to_box_retaining_pointer += 1;
        }
        if had_exact_capacity {
            assert_eq!(boxed_pointer, vector_pointer);
        }

        let complete_slice: Box<[Tracked]> = complete;
        slot = complete_slice.into_vec();
        if slot.as_ptr() == boxed_pointer {
            box_to_vec_retaining_pointer += 1;
        }
        assert_eq!(slot.len(), N);
        assert!(
            slot.iter()
                .enumerate()
                .all(|(offset, value)| value.id == start_id + offset)
        );
        slot.clear();
        assert!(slot.is_empty());
        assert_eq!(slot.as_ptr(), boxed_pointer);
        ledger.assert_exactly_once();
    }

    drop(slot);
    ledger.assert_exactly_once();
    let values_created_and_dropped = ledger.created.get();

    Observation {
        extent: N,
        reserved_capacity,
        failure_prefixes: N + 1,
        incomplete_conversions_returning_pointer,
        failure_clears_retaining_pointer,
        exact_capacity_conversions,
        vec_to_box_retaining_pointer,
        box_to_vec_retaining_pointer,
        checkouts: REPEATED_CHECKOUTS,
        values_created_and_dropped,
    }
}

fn print_observation(observation: &Observation) {
    println!(
        "extent={} reserved_capacity={} failure_prefixes={} incomplete_try_from_pointer_same={}/{} failure_clear_pointer_same={}/{} \
         exact_capacity_conversions={}/{} vec_to_box_pointer_same={}/{} \
         box_to_vec_pointer_same={}/{} created_and_dropped={}",
        observation.extent,
        observation.reserved_capacity,
        observation.failure_prefixes,
        observation.incomplete_conversions_returning_pointer,
        observation.extent,
        observation.failure_clears_retaining_pointer,
        observation.failure_prefixes,
        observation.exact_capacity_conversions,
        observation.checkouts,
        observation.vec_to_box_retaining_pointer,
        observation.checkouts,
        observation.box_to_vec_retaining_pointer,
        observation.checkouts,
        observation.values_created_and_dropped,
    );
}

fn main() {
    let zero = run_extent::<0>();
    let nonzero = run_extent::<8>();
    print_observation(&zero);
    print_observation(&nonzero);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_extent_has_one_empty_failure_prefix_and_no_values() {
        let observed = run_extent::<0>();
        assert_eq!(observed.extent, 0);
        assert_eq!(observed.reserved_capacity, 0);
        assert_eq!(observed.failure_prefixes, 1);
        assert_eq!(observed.values_created_and_dropped, 0);
    }

    #[test]
    fn every_nonzero_failure_prefix_is_cleared_and_reused() {
        let observed = run_extent::<8>();
        assert_eq!(observed.failure_prefixes, 9);
        assert_eq!(observed.failure_clears_retaining_pointer, 9);
        assert_eq!(observed.checkouts, REPEATED_CHECKOUTS);
        assert_eq!(observed.values_created_and_dropped, 60);
    }
}
