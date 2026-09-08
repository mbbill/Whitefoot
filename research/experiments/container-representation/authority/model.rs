#![forbid(unsafe_code)]

// An experimental verifier of concrete finite interval certificates, not a
// Whitefoot extension. The independent oracle below stores per-slot live bits;
// this verifier stores linear range capabilities and no payload occupancy map.
use std::collections::BTreeMap;

type Token = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Raw,
    Init,
}

#[derive(Clone, Copy, Debug)]
struct Range {
    lo: usize,
    hi: usize,
    kind: Kind,
}

impl Range {
    fn overlaps(self, other: Self) -> bool {
        self.lo < other.hi && other.lo < self.hi
    }
}

#[derive(Clone, Debug)]
struct Checker {
    capacity: usize,
    next: Token,
    ranges: BTreeMap<Token, Range>,
    loans: BTreeMap<Token, Range>,
    // These counters track linear values in the function's own authority.
    // They do not encode which physical payload slots are initialized.
    values: usize,
    returned: usize,
    released: bool,
    steps: usize,
    peak_ranges: usize,
}

impl Checker {
    fn new(capacity: usize, values: usize) -> Self {
        assert!(capacity > 0);
        Self {
            capacity,
            next: 1,
            ranges: BTreeMap::from([(
                0,
                Range {
                    lo: 0,
                    hi: capacity,
                    kind: Kind::Raw,
                },
            )]),
            loans: BTreeMap::new(),
            values,
            returned: 0,
            released: false,
            steps: 0,
            peak_ranges: 1,
        }
    }

    fn mint(&mut self, range: Range) -> Token {
        let id = self.next;
        self.next += 1;
        self.ranges.insert(id, range);
        self.peak_ranges = self.peak_ranges.max(self.ranges.len());
        id
    }

    fn unloaned(&self, range: Range) -> bool {
        !self.loans.values().any(|loan| range.overlaps(*loan))
    }

    fn split(&mut self, id: Token, at: usize) -> Option<(Token, Token)> {
        let range = *self.ranges.get(&id)?;
        if !(range.lo < at && at < range.hi) || !self.unloaned(range) {
            return None;
        }
        self.ranges.remove(&id);
        let a = self.mint(Range { hi: at, ..range });
        let b = self.mint(Range { lo: at, ..range });
        self.steps += 1;
        Some((a, b))
    }

    fn join(&mut self, left: Token, right: Token) -> Option<Token> {
        let a = *self.ranges.get(&left)?;
        let b = *self.ranges.get(&right)?;
        if left == right
            || a.hi != b.lo
            || a.kind != b.kind
            || !self.unloaned(a)
            || !self.unloaned(b)
        {
            return None;
        }
        self.ranges.remove(&left);
        self.ranges.remove(&right);
        self.steps += 1;
        Some(self.mint(Range { hi: b.hi, ..a }))
    }

    fn construct(&mut self, id: Token) -> Option<Token> {
        let range = *self.ranges.get(&id)?;
        if range.kind != Kind::Raw
            || range.hi - range.lo != 1
            || self.values == 0
            || !self.unloaned(range)
        {
            return None;
        }
        self.ranges.remove(&id);
        self.values -= 1;
        self.steps += 1;
        Some(self.mint(Range {
            kind: Kind::Init,
            ..range
        }))
    }

    fn extract(&mut self, id: Token) -> Option<Token> {
        let range = *self.ranges.get(&id)?;
        if range.kind != Kind::Init || range.hi - range.lo != 1 || !self.unloaned(range) {
            return None;
        }
        self.ranges.remove(&id);
        self.values += 1;
        self.steps += 1;
        Some(self.mint(Range {
            kind: Kind::Raw,
            ..range
        }))
    }

    fn borrow(&mut self, id: Token) -> Option<Token> {
        let range = *self.ranges.get(&id)?;
        if range.kind != Kind::Init || !self.unloaned(range) {
            return None;
        }
        let loan = self.next;
        self.next += 1;
        self.loans.insert(loan, range);
        self.steps += 1;
        Some(loan)
    }

    fn read_loan(&self, id: Token, at: usize) -> bool {
        self.loans.get(&id).is_some_and(|r| r.lo <= at && at < r.hi)
    }

    fn end_loan(&mut self, id: Token) -> bool {
        if self.loans.remove(&id).is_none() {
            return false;
        }
        self.steps += 1;
        true
    }

    fn return_values(&mut self, count: usize) -> bool {
        if count > self.values {
            return false;
        }
        self.values -= count;
        self.returned += count;
        self.steps += 1;
        true
    }

    fn release(&mut self, id: Token) -> bool {
        let Some(range) = self.ranges.get(&id) else {
            return false;
        };
        if self.ranges.len() != 1
            || range.lo != 0
            || range.hi != self.capacity
            || range.kind != Kind::Raw
            || !self.loans.is_empty()
        {
            return false;
        }
        self.ranges.remove(&id);
        self.released = true;
        self.steps += 1;
        true
    }

    fn finished(&self) -> bool {
        self.released && self.ranges.is_empty() && self.loans.is_empty() && self.values == 0
    }
}

// Independent representation: enumerate concrete slot occupancy, then verify
// coverage and linear value conservation after accepted certificate operations.
fn agrees(checker: &Checker, live: &[bool], original_values: usize) {
    let mut coverage = vec![0; checker.capacity];
    for range in checker.ranges.values() {
        assert!(range.lo < range.hi && range.hi <= live.len());
        for index in range.lo..range.hi {
            coverage[index] += 1;
            assert_eq!(live[index], range.kind == Kind::Init);
        }
    }
    assert!(
        coverage
            .iter()
            .all(|n| *n == if checker.released { 0 } else { 1 })
    );
    assert_eq!(
        live.iter().filter(|value| **value).count() + checker.values + checker.returned,
        original_values
    );
    for loan in checker.loans.values() {
        assert!(live[loan.lo..loan.hi].iter().all(|value| *value));
    }
    let loans = checker.loans.values().collect::<Vec<_>>();
    for (at, loan) in loans.iter().enumerate() {
        assert!(!loans[at + 1..].iter().any(|other| loan.overlaps(**other)));
    }
}

fn individual_slots(checker: &mut Checker) -> Vec<Token> {
    let mut tail = 0;
    let mut slots = Vec::new();
    for at in 1..checker.capacity {
        let (one, rest) = checker.split(tail, at).unwrap();
        slots.push(one);
        tail = rest;
    }
    slots.push(tail);
    slots
}

fn drain_and_release(
    checker: &mut Checker,
    slots: &mut [Token],
    live: &mut [bool],
    original: usize,
) {
    agrees(checker, live, original);
    for (at, slot) in slots.iter_mut().enumerate() {
        if live[at] {
            *slot = checker.extract(*slot).unwrap();
            live[at] = false;
            agrees(checker, live, original);
        }
    }
    let mut all = slots[0];
    for slot in &slots[1..] {
        all = checker.join(all, *slot).unwrap();
    }
    assert!(checker.return_values(checker.values));
    assert!(checker.release(all));
    assert!(checker.finished());
    agrees(checker, live, original);
}

// This is an exact representability comparison, not a performance simulation.
// A closed run can describe exactly one circular initialized interval. It can
// still encode other applications using additional values/metadata; that cost
// and its contracts must be tested separately, not inferred from this count.
fn native_window(capacity: usize, mask: usize) -> bool {
    (0..capacity).any(|head| {
        (0..=capacity).any(|len| {
            let represented = (0..len).fold(0, |bits, i| bits | (1 << ((head + i) % capacity)));
            represented == mask
        })
    })
}

fn exhaustive_states() -> (usize, usize, usize) {
    let mut states = 0;
    let mut native = 0;
    let mut checked = 0;
    for capacity in 1..=8 {
        for mask in 0..1 << capacity {
            let count = (mask as usize).count_ones() as usize;
            let mut checker = Checker::new(capacity, count);
            let mut slots = individual_slots(&mut checker);
            let mut live = vec![false; capacity];
            for at in 0..capacity {
                if mask & (1 << at) != 0 {
                    slots[at] = checker.construct(slots[at]).unwrap();
                    live[at] = true;
                    agrees(&checker, &live, count);
                }
                // Every slot read is checked against a separately enumerated bit.
                let mut probe = checker.clone();
                let loan = probe.borrow(slots[at]);
                assert_eq!(loan.is_some(), live[at]);
                if let Some(loan) = loan {
                    assert!(probe.read_loan(loan, at));
                    assert!(!probe.read_loan(loan, capacity));
                    assert!(probe.borrow(slots[at]).is_none());
                    assert!(probe.extract(slots[at]).is_none());
                    assert!(probe.end_loan(loan));
                    assert!(!probe.read_loan(loan, at));
                }
            }
            native += usize::from(native_window(capacity, mask));
            drain_and_release(&mut checker, &mut slots, &mut live, count);
            checked += 1;
            states += 1;
        }
    }
    (states, native, checked)
}

fn sparse_reuse() -> (usize, usize) {
    let mut checker = Checker::new(4, 3);
    let mut slots = individual_slots(&mut checker);
    let mut live = vec![true, true, true, false];
    for slot in &mut slots[..3] {
        *slot = checker.construct(*slot).unwrap();
    }
    let first = checker.borrow(slots[0]).unwrap();
    let last = checker.borrow(slots[2]).unwrap();
    assert!(checker.extract(slots[0]).is_none());
    assert!(checker.extract(slots[2]).is_none());
    let old_middle = slots[1];
    slots[1] = checker.extract(old_middle).unwrap();
    live[1] = false;
    assert!(!native_window(4, 0b0101));
    agrees(&checker, &live, 3);
    assert!(checker.borrow(old_middle).is_none());
    slots[1] = checker.construct(slots[1]).unwrap();
    live[1] = true;
    assert!(checker.borrow(old_middle).is_none());
    assert!(checker.read_loan(first, 0) && checker.read_loan(last, 2));
    assert!(checker.end_loan(first) && checker.end_loan(last));
    drain_and_release(&mut checker, &mut slots, &mut live, 3);
    (checker.steps, checker.peak_ranges)
}

// This sealed experimental summary is derived from exact capability endpoints,
// not accepted because a library asserts that its representation is a prefix.
// A real language would still need to verify a symbolic implementation and its
// abstraction boundary; concrete instantiation here does not establish that.
#[derive(Clone, Copy)]
struct Prefix {
    initialized: Option<Token>,
    spare: Option<Token>,
    len: usize,
}

fn prefix_valid(checker: &Checker, prefix: Prefix) -> bool {
    let init = prefix.initialized.and_then(|id| checker.ranges.get(&id));
    let raw = prefix.spare.and_then(|id| checker.ranges.get(&id));
    let init_valid = if prefix.len == 0 {
        prefix.initialized.is_none()
    } else {
        init.is_some_and(|r| r.kind == Kind::Init && r.lo == 0 && r.hi == prefix.len)
    };
    let raw_valid = if prefix.len == checker.capacity {
        prefix.spare.is_none()
    } else {
        raw.is_some_and(|r| r.kind == Kind::Raw && r.lo == prefix.len && r.hi == checker.capacity)
    };
    prefix.len <= checker.capacity && !checker.released && init_valid && raw_valid
}

fn append(checker: &mut Checker, prefix: Prefix) -> Option<Prefix> {
    if !prefix_valid(checker, prefix)
        || prefix.len == checker.capacity
        || checker.values == 0
        || !checker.loans.is_empty()
    {
        return None;
    }
    let spare = prefix.spare?;
    let (one, tail) = if prefix.len + 1 == checker.capacity {
        (spare, None)
    } else {
        let (one, tail) = checker.split(spare, prefix.len + 1)?;
        (one, Some(tail))
    };
    let constructed = checker.construct(one)?;
    let initialized = if let Some(front) = prefix.initialized {
        checker.join(front, constructed)?
    } else {
        constructed
    };
    let result = Prefix {
        initialized: Some(initialized),
        spare: tail,
        len: prefix.len + 1,
    };
    assert!(prefix_valid(checker, result));
    Some(result)
}

fn prefix_helpers(capacity: usize, stop: usize) -> (usize, usize) {
    let mut checker = Checker::new(capacity, capacity);
    let mut prefix = Prefix {
        initialized: None,
        spare: Some(0),
        len: 0,
    };
    let mut live = vec![false; capacity];
    for at in 0..stop {
        let before = prefix;
        prefix = append(&mut checker, prefix).unwrap();
        live[at] = true;
        assert!(!prefix_valid(&checker, before));
        assert!(append(&mut checker, before).is_none());
        agrees(&checker, &live, capacity);
    }
    // Simulate an actual producer failure after `stop` successful constructions.
    // Unconsumed inputs and extracted initialized values must all be returned.
    let construction_steps = checker.steps;
    let construction_peak = checker.peak_ranges;
    let mut slots = Vec::new();
    for root in [prefix.initialized, prefix.spare].into_iter().flatten() {
        let mut rest = root;
        let range = checker.ranges[&rest];
        for at in range.lo + 1..range.hi {
            let (one, tail) = checker.split(rest, at).unwrap();
            slots.push(one);
            rest = tail;
        }
        slots.push(rest);
    }
    drain_and_release(&mut checker, &mut slots, &mut live, capacity);
    (construction_steps, construction_peak)
}

fn wrapped_spans() {
    let mut checker = Checker::new(8, 4);
    let mut slots = individual_slots(&mut checker);
    let mut live = vec![false; 8];
    for at in [0, 1, 6, 7] {
        slots[at] = checker.construct(slots[at]).unwrap();
        live[at] = true;
    }
    let high = checker.join(slots[6], slots[7]).unwrap();
    let low = checker.join(slots[0], slots[1]).unwrap();
    let first = checker.borrow(high).unwrap();
    let second = checker.borrow(low).unwrap();
    agrees(&checker, &live, 4);
    assert!(checker.read_loan(first, 6) && checker.read_loan(second, 0));
    assert!(!checker.read_loan(first, 0));
    assert!(checker.borrow(high).is_none());
    assert!(checker.split(high, 7).is_none());
    assert!(checker.end_loan(first) && checker.end_loan(second));
    (slots[6], slots[7]) = checker.split(high, 7).unwrap();
    (slots[0], slots[1]) = checker.split(low, 1).unwrap();
    drain_and_release(&mut checker, &mut slots, &mut live, 4);
}

fn main() {
    let (states, native, checked) = exhaustive_states();
    let (steps, peak) = sparse_reuse();
    println!("case,states,native_window_states,checked_interval_states");
    println!("all_live_sets_capacity_1_to_8,{states},{native},{checked}");
    println!("case,certificate_transitions,peak_range_tokens");
    println!("three_live_slots_capacity_four_middle_reuse,{steps},{peak}");
    for capacity in [4, 16, 256] {
        for stop in [capacity / 2, capacity] {
            let (steps, peak) = prefix_helpers(capacity, stop);
            println!("concrete_prefix_capacity_{capacity}_stop_{stop},{steps},{peak}");
        }
    }
    wrapped_spans();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_bounded_live_sets_and_nearby_invalid_reads() {
        assert_eq!(exhaustive_states(), (510, 184, 510));
    }

    #[test]
    fn middle_slot_can_be_reused_without_invalidating_neighbor_loans() {
        sparse_reuse();
        // This near neighbor refutes the initial paper claim that capacity 3
        // alone separates sparse occupancy from a circular window.
        assert!(native_window(3, 0b101));
    }

    #[test]
    fn split_and_join_do_not_duplicate_or_forge_ownership() {
        let mut checker = Checker::new(8, 8);
        assert!(checker.split(0, 0).is_none());
        assert!(checker.split(0, 8).is_none());
        assert!(checker.split(999, 4).is_none());
        let (a, b) = checker.split(0, 4).unwrap();
        assert!(checker.split(0, 2).is_none());
        assert!(checker.join(a, a).is_none());
        assert!(checker.join(b, a).is_none());
        let all = checker.join(a, b).unwrap();
        assert!(checker.join(a, b).is_none());
        assert!(checker.construct(all).is_none());
        agrees(&checker, &[false; 8], 8);
    }

    #[test]
    fn incomplete_linear_values_and_live_storage_prevent_completion() {
        let mut checker = Checker::new(1, 1);
        assert!(!checker.finished());
        let init = checker.construct(0).unwrap();
        assert!(!checker.release(init));
        assert!(checker.construct(init).is_none());
        let raw = checker.extract(init).unwrap();
        assert!(checker.release(raw));
        assert!(!checker.finished());
        assert!(!checker.return_values(2));
        assert!(checker.return_values(1));
        assert!(!checker.return_values(1));
        assert!(checker.finished());
        assert!(!checker.release(raw));
    }

    #[test]
    fn mixed_initialization_cannot_be_hidden_by_join() {
        let mut checker = Checker::new(2, 1);
        let (a, b) = checker.split(0, 1).unwrap();
        let init = checker.construct(a).unwrap();
        assert!(checker.join(init, b).is_none());
        assert!(checker.borrow(b).is_none());
        assert!(checker.extract(b).is_none());
        agrees(&checker, &[true, false], 1);
    }

    #[test]
    fn concrete_prefix_helpers_preserve_contract_and_failure_ownership() {
        for capacity in 1..=32 {
            for stop in 0..=capacity {
                prefix_helpers(capacity, stop);
            }
        }
    }

    #[test]
    fn wrapped_two_span_loans_share_backing_without_overlapping() {
        wrapped_spans();
    }
}
