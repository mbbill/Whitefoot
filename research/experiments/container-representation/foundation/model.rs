#![forbid(unsafe_code)]

// Concrete protocol experiment, not a source checker. Unlike authority/model.rs,
// this models helper failure, full sealing, empty release, and backing retirement.
// No claim about symbolic contracts, allocation layout, or proof erasure follows.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Building,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Loan {
    Published,
    Done,
    Joined,
    Consumed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Owner {
    backing: usize,
    cap: usize,
    head: usize,
    len: usize,
    phase: Phase,
    loan: Option<Loan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct World {
    next: usize,
    owners: BTreeMap<usize, Owner>,
    // Independent per-slot payload oracle. Owner metadata never reads this to
    // decide admission; successful operations must agree with it afterwards.
    memory: BTreeMap<usize, Vec<Option<usize>>>,
    observers: BTreeMap<usize, Vec<Option<usize>>>,
    caller: BTreeSet<usize>,
    discharged: BTreeSet<usize>,
    total: usize,
}

impl World {
    fn new(total: usize) -> Self {
        Self {
            next: 0,
            owners: BTreeMap::new(),
            memory: BTreeMap::new(),
            observers: BTreeMap::new(),
            caller: (0..total).collect(),
            discharged: BTreeSet::new(),
            total,
        }
    }

    fn reserve(&mut self, cap: usize, succeeds: bool) -> Option<usize> {
        if !succeeds {
            return None;
        }
        let id = self.next;
        self.next += 1;
        self.owners.insert(
            id,
            Owner {
                backing: id,
                cap,
                head: 0,
                len: 0,
                phase: Phase::Building,
                loan: None,
            },
        );
        self.memory.insert(id, vec![None; cap]);
        Some(id)
    }

    fn push(&mut self, id: usize, value: usize) -> bool {
        let Some(owner) = self.owners.get_mut(&id) else {
            return false;
        };
        if owner.phase != Phase::Building
            || owner.loan.is_some()
            || owner.head + owner.len == owner.cap
            || !self.caller.remove(&value)
        {
            return false;
        }
        let slot = &mut self.memory.get_mut(&owner.backing).unwrap()[owner.head + owner.len];
        assert!(slot.replace(value).is_none());
        owner.len += 1;
        true
    }

    fn take_front(&mut self, id: usize) -> Option<usize> {
        let owner = self.owners.get_mut(&id)?;
        if owner.phase != Phase::Building || owner.loan.is_some() || owner.len == 0 {
            return None;
        }
        let value = self.memory.get_mut(&owner.backing).unwrap()[owner.head]
            .take()
            .unwrap();
        owner.len -= 1;
        owner.head = if owner.len == 0 { 0 } else { owner.head + 1 };
        assert!(self.caller.insert(value));
        Some(value)
    }

    fn seal(&mut self, id: usize, extent: usize) -> bool {
        let Some(owner) = self.owners.get_mut(&id) else {
            return false;
        };
        // This experiment chooses no outstanding loans at state conversion.
        // It does not establish that address-preserving conversion under a loan
        // is intrinsically unsound; that needs separate provenance transport.
        if owner.phase != Phase::Building
            || owner.loan.is_some()
            || owner.head != 0
            || owner.len != extent
            || owner.cap != extent
        {
            return false;
        }
        owner.phase = Phase::Full;
        true
    }

    fn open(&mut self, id: usize) -> bool {
        let Some(owner) = self.owners.get_mut(&id) else {
            return false;
        };
        if owner.phase != Phase::Full || owner.loan.is_some() {
            return false;
        }
        owner.phase = Phase::Building;
        true
    }

    fn release(&mut self, id: usize) -> bool {
        let Some(owner) = self.owners.get(&id) else {
            return false;
        };
        if owner.len != 0 || owner.loan.is_some() {
            return false;
        }
        let backing = owner.backing;
        self.owners.remove(&id);
        assert!(
            self.memory
                .remove(&backing)
                .unwrap()
                .iter()
                .all(Option::is_none)
        );
        true
    }

    fn discharge(&mut self, value: usize) {
        assert!(self.caller.remove(&value));
        assert!(self.discharged.insert(value));
    }

    fn publish(&mut self, id: usize) -> bool {
        let Some(owner) = self.owners.get_mut(&id) else {
            return false;
        };
        if owner.loan.is_some() {
            return false;
        }
        owner.loan = Some(Loan::Published);
        self.observers
            .insert(owner.backing, self.memory[&owner.backing].clone());
        true
    }

    fn advance_loan(&mut self, id: usize) {
        let owner = self.owners.get_mut(&id).unwrap();
        owner.loan = match owner.loan.unwrap() {
            Loan::Published => Some(Loan::Done),
            Loan::Done => Some(Loan::Joined),
            Loan::Joined => Some(Loan::Consumed),
            Loan::Consumed => None,
        };
        if owner.loan.is_none() {
            self.observers.remove(&owner.backing);
        }
    }

    fn valid(&self) -> bool {
        if self.owners.len() != self.memory.len() {
            return false;
        }
        for (backing, observed) in &self.observers {
            if self.memory.get(backing) != Some(observed) {
                return false;
            }
        }
        let mut counts = vec![0; self.total];
        for value in self.caller.iter().chain(&self.discharged) {
            let Some(count) = counts.get_mut(*value) else {
                return false;
            };
            *count += 1;
        }
        let mut backings = BTreeSet::new();
        for owner in self.owners.values() {
            let Some(slots) = self.memory.get(&owner.backing) else {
                return false;
            };
            if !backings.insert(owner.backing)
                || slots.len() != owner.cap
                || owner.head + owner.len > owner.cap
                || (owner.phase == Phase::Full && (owner.head != 0 || owner.len != owner.cap))
            {
                return false;
            }
            for (index, slot) in slots.iter().enumerate() {
                if slot.is_some() != (owner.head <= index && index < owner.head + owner.len) {
                    return false;
                }
                if let Some(value) = slot {
                    let Some(count) = counts.get_mut(*value) else {
                        return false;
                    };
                    *count += 1;
                }
            }
        }
        counts.iter().all(|count| *count == 1)
    }

    fn drain(&mut self, id: usize) -> Vec<usize> {
        if self.owners[&id].phase == Phase::Full {
            assert!(self.open(id));
        }
        let mut sequence = Vec::new();
        while let Some(value) = self.take_front(id) {
            sequence.push(value);
            self.discharge(value);
            assert!(self.valid());
        }
        assert!(self.release(id));
        sequence
    }
}

// A failing producer returns its unconsumed value plus the current valid builder.
// The finite loop split stands for crossing a helper boundary; this code does not
// verify that a Whitefoot contract can express or transport that relation.
fn construction_traces() -> usize {
    let mut traces = 0;
    for cap in 0..=8 {
        for stop in 0..=cap {
            for split in 0..=stop {
                let mut world = World::new(cap);
                let before = world.clone();
                assert_eq!(world.reserve(cap, false), None);
                assert_eq!(world, before);
                let id = world.reserve(cap, true).unwrap();
                for batch in [0..split, split..stop] {
                    for value in batch {
                        assert!(world.push(id, value));
                        assert!(world.valid());
                    }
                }
                let before_seal = world.clone();
                assert_eq!(world.seal(id, cap), stop == cap);
                if stop != cap {
                    assert_eq!(world, before_seal);
                }
                assert_eq!(world.drain(id), (0..stop).collect::<Vec<_>>());
                for value in stop..cap {
                    world.discharge(value);
                }
                assert!(world.valid());
                assert!(world.owners.is_empty());
                assert_eq!(world.discharged.len(), cap);
                traces += 1;
            }
        }
    }
    traces
}

fn relocation_traces() -> usize {
    let mut traces = 0;
    for old_cap in 0..=6 {
        for new_cap in old_cap..=8 {
            let mut world = World::new(old_cap);
            let old = world.reserve(old_cap, true).unwrap();
            for value in 0..old_cap {
                assert!(world.push(old, value));
            }
            assert!(world.seal(old, old_cap));
            let before = world.clone();
            assert_eq!(world.reserve(new_cap, false), None);
            assert_eq!(world, before);
            let new = world.reserve(new_cap, true).unwrap();
            assert!(world.open(old));
            while let Some(value) = world.take_front(old) {
                assert!(world.valid());
                assert!(world.push(new, value));
                assert!(world.valid());
            }
            assert!(world.release(old));
            assert!(!world.open(old));
            assert!(!world.publish(old));
            assert_eq!(world.drain(new), (0..old_cap).collect::<Vec<_>>());
            assert!(world.valid());
            traces += 1;
        }
    }
    traces
}

fn loan_traces() -> usize {
    for cap in 0..=8 {
        let mut world = World::new(cap);
        let id = world.reserve(cap, true).unwrap();
        for value in 0..cap {
            assert!(world.push(id, value));
        }
        assert!(world.publish(id));
        for _ in 0..4 {
            let before = world.clone();
            assert!(!world.seal(id, cap));
            assert_eq!(world.take_front(id), None);
            assert!(!world.release(id));
            assert_eq!(world, before);
            world.advance_loan(id);
        }
        assert!(world.seal(id, cap));
        world.drain(id);
        assert!(world.valid());
    }
    9
}

fn main() {
    println!(
        "construction/failure/helper splits: {}",
        construction_traces()
    );
    println!(
        "allocation-first relocation traces: {}",
        relocation_traces()
    );
    println!("loan-retirement traces: {}", loan_traces());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_protocols() {
        assert_eq!(construction_traces(), 165);
        assert_eq!(relocation_traces(), 42);
        assert_eq!(loan_traces(), 9);
    }

    #[test]
    fn oracle_detects_weakened_full_init_and_linear_accounting() {
        let mut world = World::new(2);
        let id = world.reserve(2, true).unwrap();
        assert!(world.push(id, 0));
        assert!(!world.seal(id, 1)); // A full prefix is not a full backing.
        let mut raw = world.clone();
        raw.owners.get_mut(&id).unwrap().len = 2;
        raw.owners.get_mut(&id).unwrap().phase = Phase::Full;
        assert!(!raw.valid());
        let mut duplicated = world.clone();
        duplicated.caller.insert(0);
        assert!(!duplicated.valid());
        let mut leaked = world;
        leaked.caller.remove(&1);
        assert!(!leaked.valid());
    }

    #[test]
    fn oracle_detects_wrong_or_duplicated_backing_identity() {
        let mut world = World::new(0);
        let first = world.reserve(0, true).unwrap();
        let second = world.reserve(0, true).unwrap();
        assert!(world.valid());
        world.owners.get_mut(&second).unwrap().backing = first;
        assert!(!world.valid());
        world.owners.get_mut(&second).unwrap().backing = usize::MAX;
        assert!(!world.valid());
    }

    #[test]
    fn no_double_commit_or_early_reuse() {
        let mut world = World::new(2);
        let id = world.reserve(2, true).unwrap();
        assert!(world.push(id, 0));
        let before = world.clone();
        assert!(!world.push(id, 0));
        assert_eq!(world, before);
        assert!(world.publish(id));
        world.advance_loan(id); // DONE still owns the loan.
        assert!(!world.push(id, 1));
        assert!(world.valid());
    }

    #[test]
    fn observer_detects_done_treated_as_retirement() {
        let mut world = World::new(1);
        let id = world.reserve(1, true).unwrap();
        assert!(world.push(id, 0));
        assert!(world.publish(id));
        world.advance_loan(id);
        // Deliberately weaken the protocol guard while retaining the observer.
        world.owners.get_mut(&id).unwrap().loan = None;
        let value = world.take_front(id).unwrap();
        world.discharge(value);
        assert!(!world.valid());
        assert!(world.release(id));
        assert!(!world.valid());
    }
}
