#![forbid(unsafe_code)]

//! Bounded native feasibility control, not a Whitefoot API or proof system.
//! Caller-selected store IDs test rejection only when IDs differ; this model
//! does not supply the uniqueness authority a real factory would need. The
//! affine runtime ticket below models a logical deletion guard. It is not a
//! Rust borrow and can outlive a dropped store, so it does not establish backing
//! lifetime. `borrowed_words` is the separate real-lifetime control.
//! The traces do not exercise abandoned tickets or access-counter saturation.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

const MAX_GENERATION: u8 = 2;

type DropLedger = Rc<RefCell<BTreeMap<u64, u32>>>;

#[derive(Debug)]
struct Payload {
    id: u64,
    words: Box<[u64; 4]>,
    drops: DropLedger,
}

impl Payload {
    fn new(id: u64, drops: &DropLedger) -> Self {
        Self {
            id,
            words: Box::new([id, id + 1, id + 2, id + 3]),
            drops: Rc::clone(drops),
        }
    }

    fn checksum(&self) -> u64 {
        self.words.iter().sum()
    }
}

impl Drop for Payload {
    fn drop(&mut self) {
        *self.drops.borrow_mut().entry(self.id).or_default() += 1;
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Handle {
    store: u64,
    slot: usize,
    generation: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Policy {
    Weak,
    Retained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdentityError {
    WrongStore,
    Expired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InsertError {
    CapacityFull,
    GenerationExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeleteResult {
    Deleted { retired: bool },
    BusyMemberships(u8),
    BusyAccess(u16),
    WrongStore,
    Expired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LookupResult {
    Found { payload_id: u64, checksum: u64 },
    Expired,
    WrongStore,
    MissingKey,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum IndexId {
    First,
    Second,
}

impl IndexId {
    const fn offset(self) -> usize {
        match self {
            Self::First => 0,
            Self::Second => 1,
        }
    }
}

#[derive(Debug)]
struct Entry {
    generation: u8,
    payload: Payload,
    memberships: [bool; 2],
    accesses: u16,
}

#[derive(Debug)]
enum SlotState {
    Vacant,
    Occupied(Entry),
    Retired,
}

#[derive(Debug)]
struct Slot {
    next_generation: u8,
    state: SlotState,
}

#[derive(Debug)]
// An affine logical capability. It drives the delete guard but carries no
// Rust borrow and does not keep the Store backing alive.
struct AccessTicket {
    handle: Handle,
}

#[derive(Debug)]
struct Store {
    id: u64,
    policy: Policy,
    slots: Vec<Slot>,
}

impl Store {
    fn new(id: u64, capacity: usize, policy: Policy) -> Self {
        Self {
            id,
            policy,
            slots: (0..capacity)
                .map(|_| Slot {
                    next_generation: 0,
                    state: SlotState::Vacant,
                })
                .collect(),
        }
    }

    fn insert(&mut self, payload: Payload) -> Result<Handle, (InsertError, Payload)> {
        let Some(slot_index) = self
            .slots
            .iter()
            .position(|slot| matches!(slot.state, SlotState::Vacant))
        else {
            let error = if self
                .slots
                .iter()
                .all(|slot| matches!(slot.state, SlotState::Retired))
            {
                InsertError::GenerationExhausted
            } else {
                InsertError::CapacityFull
            };
            return Err((error, payload));
        };
        let slot = &mut self.slots[slot_index];
        let generation = slot.next_generation;
        slot.state = SlotState::Occupied(Entry {
            generation,
            payload,
            memberships: [false, false],
            accesses: 0,
        });
        Ok(Handle {
            store: self.id,
            slot: slot_index,
            generation,
        })
    }

    fn entry(&self, handle: Handle) -> Result<&Entry, IdentityError> {
        if handle.store != self.id {
            return Err(IdentityError::WrongStore);
        }
        let Some(slot) = self.slots.get(handle.slot) else {
            return Err(IdentityError::Expired);
        };
        match &slot.state {
            SlotState::Occupied(entry) if entry.generation == handle.generation => Ok(entry),
            SlotState::Vacant | SlotState::Retired | SlotState::Occupied(_) => {
                Err(IdentityError::Expired)
            }
        }
    }

    fn entry_mut(&mut self, handle: Handle) -> Result<&mut Entry, IdentityError> {
        if handle.store != self.id {
            return Err(IdentityError::WrongStore);
        }
        let Some(slot) = self.slots.get_mut(handle.slot) else {
            return Err(IdentityError::Expired);
        };
        match &mut slot.state {
            SlotState::Occupied(entry) if entry.generation == handle.generation => Ok(entry),
            SlotState::Vacant | SlotState::Retired | SlotState::Occupied(_) => {
                Err(IdentityError::Expired)
            }
        }
    }

    fn inspect(&self, handle: Handle) -> LookupResult {
        match self.entry(handle) {
            Ok(entry) => LookupResult::Found {
                payload_id: entry.payload.id,
                checksum: entry.payload.checksum(),
            },
            Err(IdentityError::WrongStore) => LookupResult::WrongStore,
            Err(IdentityError::Expired) => LookupResult::Expired,
        }
    }

    fn borrowed_words(&self, handle: Handle) -> Result<&[u64; 4], IdentityError> {
        Ok(self.entry(handle)?.payload.words.as_ref())
    }

    fn add_membership(&mut self, handle: Handle, index: IndexId) -> Result<(), IdentityError> {
        let retained = self.policy == Policy::Retained;
        let entry = self.entry_mut(handle)?;
        if retained {
            assert!(!entry.memberships[index.offset()], "duplicate membership");
            entry.memberships[index.offset()] = true;
        }
        Ok(())
    }

    fn remove_membership(&mut self, handle: Handle, index: IndexId) -> Result<(), IdentityError> {
        if self.policy == Policy::Weak {
            return Ok(());
        }
        let entry = self.entry_mut(handle)?;
        let present = &mut entry.memberships[index.offset()];
        assert!(*present, "index bookkeeping removed absent membership");
        *present = false;
        Ok(())
    }

    fn begin_access(&mut self, handle: Handle) -> Result<(AccessTicket, u64), IdentityError> {
        let checksum = {
            let entry = self.entry_mut(handle)?;
            entry.accesses += 1;
            entry.payload.checksum()
        };
        Ok((AccessTicket { handle }, checksum))
    }

    fn end_access(&mut self, ticket: AccessTicket) -> Result<(), (IdentityError, AccessTicket)> {
        let entry = match self.entry_mut(ticket.handle) {
            Ok(entry) => entry,
            Err(error) => return Err((error, ticket)),
        };
        assert!(entry.accesses > 0, "access counter underflow");
        entry.accesses -= 1;
        Ok(())
    }

    fn delete(&mut self, handle: Handle) -> DeleteResult {
        let entry = match self.entry(handle) {
            Ok(entry) => entry,
            Err(IdentityError::WrongStore) => return DeleteResult::WrongStore,
            Err(IdentityError::Expired) => return DeleteResult::Expired,
        };
        let memberships = entry.memberships.iter().filter(|present| **present).count() as u8;
        if memberships != 0 {
            return DeleteResult::BusyMemberships(memberships);
        }
        if entry.accesses != 0 {
            return DeleteResult::BusyAccess(entry.accesses);
        }
        let generation = entry.generation;
        let retired = generation == MAX_GENERATION;
        let slot = &mut self.slots[handle.slot];
        slot.state = if retired {
            SlotState::Retired
        } else {
            slot.next_generation = generation + 1;
            SlotState::Vacant
        };
        DeleteResult::Deleted { retired }
    }
}

#[derive(Debug)]
struct Protocol {
    store: Store,
    indexes: [BTreeMap<u64, Handle>; 2],
}

impl Protocol {
    fn new(store_id: u64, capacity: usize, policy: Policy) -> Self {
        Self {
            store: Store::new(store_id, capacity, policy),
            indexes: std::array::from_fn(|_| BTreeMap::new()),
        }
    }

    fn link(&mut self, index: IndexId, key: u64, handle: Handle) -> Result<(), IdentityError> {
        self.store.add_membership(handle, index)?;
        assert!(
            self.indexes[index.offset()].insert(key, handle).is_none(),
            "fixture links one object once per index"
        );
        Ok(())
    }

    fn unlink(&mut self, index: IndexId, key: u64) -> bool {
        let Some(handle) = self.indexes[index.offset()].remove(&key) else {
            return false;
        };
        match self.store.remove_membership(handle, index) {
            Ok(()) | Err(IdentityError::Expired) => true,
            Err(IdentityError::WrongStore) => panic!("private index held a foreign handle"),
        }
    }

    fn lookup(&self, index: IndexId, key: u64) -> LookupResult {
        match self.indexes[index.offset()].get(&key) {
            Some(handle) => self.store.inspect(*handle),
            None => LookupResult::MissingKey,
        }
    }
}

fn observe<T: std::fmt::Debug>(trace: &mut Vec<String>, label: &str, value: T) {
    trace.push(format!("{label}={value:?}"));
}

fn run_scenarios() -> (Vec<String>, DropLedger) {
    let drops = Rc::new(RefCell::new(BTreeMap::new()));
    let mut trace = Vec::new();
    {
        let mut weak = Protocol::new(10, 1, Policy::Weak);
        let first = weak.store.insert(Payload::new(1, &drops)).unwrap();
        observe(&mut trace, "weak.insert.first", first);
        let (error, refused) = weak.store.insert(Payload::new(2, &drops)).unwrap_err();
        observe(&mut trace, "weak.insert.full", error);
        drop(refused);
        weak.link(IndexId::First, 100, first).unwrap();
        weak.link(IndexId::Second, 200, first).unwrap();
        observe(
            &mut trace,
            "weak.lookup.first",
            weak.lookup(IndexId::First, 100),
        );
        let borrowed_checksum: u64 = weak.store.borrowed_words(first).unwrap().iter().sum();
        observe(&mut trace, "weak.rust-borrow.checksum", borrowed_checksum);
        let (access, checksum) = weak.store.begin_access(first).unwrap();
        observe(&mut trace, "weak.access.checksum", checksum);
        observe(&mut trace, "weak.delete.borrowed", weak.store.delete(first));
        let mut foreign = Store::new(11, 1, Policy::Weak);
        let (error, access) = foreign.end_access(access).unwrap_err();
        observe(&mut trace, "weak.access.wrong-store", error);
        observe(
            &mut trace,
            "weak.access.returned",
            weak.store.end_access(access).unwrap(),
        );
        observe(&mut trace, "weak.delete", weak.store.delete(first));
        observe(
            &mut trace,
            "weak.lookup.stale.first",
            weak.lookup(IndexId::First, 100),
        );
        observe(
            &mut trace,
            "weak.lookup.stale.second",
            weak.lookup(IndexId::Second, 200),
        );
        let reused = weak.store.insert(Payload::new(3, &drops)).unwrap();
        observe(&mut trace, "weak.insert.reused", reused);
        observe(
            &mut trace,
            "weak.lookup.old.after.reuse",
            weak.lookup(IndexId::First, 100),
        );
        observe(&mut trace, "cross.store", foreign.inspect(reused));

        let mut retained = Protocol::new(20, 1, Policy::Retained);
        let held = retained.store.insert(Payload::new(4, &drops)).unwrap();
        retained.link(IndexId::First, 300, held).unwrap();
        retained.link(IndexId::Second, 400, held).unwrap();
        observe(
            &mut trace,
            "retained.delete.two",
            retained.store.delete(held),
        );
        observe(
            &mut trace,
            "retained.unlink.first",
            retained.unlink(IndexId::First, 300),
        );
        observe(
            &mut trace,
            "retained.delete.one",
            retained.store.delete(held),
        );
        observe(
            &mut trace,
            "retained.lookup.second",
            retained.lookup(IndexId::Second, 400),
        );
        observe(
            &mut trace,
            "retained.unlink.second",
            retained.unlink(IndexId::Second, 400),
        );
        let (access, checksum) = retained.store.begin_access(held).unwrap();
        observe(&mut trace, "retained.access.checksum", checksum);
        observe(
            &mut trace,
            "retained.delete.borrowed",
            retained.store.delete(held),
        );
        retained.store.end_access(access).unwrap();
        observe(&mut trace, "retained.delete", retained.store.delete(held));

        let mut finite = Store::new(30, 1, Policy::Weak);
        let generation0 = finite.insert(Payload::new(5, &drops)).unwrap();
        observe(&mut trace, "finite.insert.g0", generation0);
        let (error, refused) = finite.insert(Payload::new(6, &drops)).unwrap_err();
        observe(&mut trace, "finite.insert.full", error);
        drop(refused);
        observe(&mut trace, "finite.delete.g0", finite.delete(generation0));
        let generation1 = finite.insert(Payload::new(7, &drops)).unwrap();
        observe(&mut trace, "finite.insert.g1", generation1);
        observe(&mut trace, "finite.delete.g1", finite.delete(generation1));
        let generation2 = finite.insert(Payload::new(8, &drops)).unwrap();
        observe(&mut trace, "finite.insert.g2", generation2);
        observe(&mut trace, "finite.delete.g2", finite.delete(generation2));
        let (error, refused) = finite.insert(Payload::new(9, &drops)).unwrap_err();
        observe(&mut trace, "finite.insert.exhausted", error);
        drop(refused);
        observe(&mut trace, "finite.old.g0", finite.inspect(generation0));
    }
    (trace, drops)
}

fn expected_trace() -> Vec<String> {
    [
        "weak.insert.first=Handle { store: 10, slot: 0, generation: 0 }",
        "weak.insert.full=CapacityFull",
        "weak.lookup.first=Found { payload_id: 1, checksum: 10 }",
        "weak.rust-borrow.checksum=10",
        "weak.access.checksum=10",
        "weak.delete.borrowed=BusyAccess(1)",
        "weak.access.wrong-store=WrongStore",
        "weak.access.returned=()",
        "weak.delete=Deleted { retired: false }",
        "weak.lookup.stale.first=Expired",
        "weak.lookup.stale.second=Expired",
        "weak.insert.reused=Handle { store: 10, slot: 0, generation: 1 }",
        "weak.lookup.old.after.reuse=Expired",
        "cross.store=WrongStore",
        "retained.delete.two=BusyMemberships(2)",
        "retained.unlink.first=true",
        "retained.delete.one=BusyMemberships(1)",
        "retained.lookup.second=Found { payload_id: 4, checksum: 22 }",
        "retained.unlink.second=true",
        "retained.access.checksum=22",
        "retained.delete.borrowed=BusyAccess(1)",
        "retained.delete=Deleted { retired: false }",
        "finite.insert.g0=Handle { store: 30, slot: 0, generation: 0 }",
        "finite.insert.full=CapacityFull",
        "finite.delete.g0=Deleted { retired: false }",
        "finite.insert.g1=Handle { store: 30, slot: 0, generation: 1 }",
        "finite.delete.g1=Deleted { retired: false }",
        "finite.insert.g2=Handle { store: 30, slot: 0, generation: 2 }",
        "finite.delete.g2=Deleted { retired: true }",
        "finite.insert.exhausted=GenerationExhausted",
        "finite.old.g0=Expired",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn verify() {
    let (trace, drops) = run_scenarios();
    assert_eq!(
        trace,
        expected_trace(),
        "lifecycle/identity oracle mismatch"
    );
    let actual_drops = drops.borrow();
    let expected_drops: BTreeMap<_, _> = (1..=9).map(|id| (id, 1)).collect();
    assert_eq!(*actual_drops, expected_drops, "resource lifecycle mismatch");
    println!(
        "stable-index cases={} max_generation={MAX_GENERATION}",
        trace.len()
    );
    println!(
        "access_ticket=affine-logical-guard ticket_registry=none real_borrow=rust-lifetime store_identity_authority=unmodeled"
    );
    for line in trace {
        println!("{line}");
    }
    println!("payload_drops=9/9 exactly_once");
}

fn main() {
    verify();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn finite_identity_and_lifecycle_protocols_match_oracles() {
        let (trace, drops) = run_scenarios();
        assert_eq!(trace, expected_trace());
        assert_eq!(
            *drops.borrow(),
            (1..=9).map(|id| (id, 1)).collect::<BTreeMap<_, _>>()
        );
    }

    #[test]
    fn finite_generations_do_not_wrap() {
        let drops = Rc::new(RefCell::new(BTreeMap::new()));
        let mut store = Store::new(44, 1, Policy::Weak);
        let mut generations = BTreeSet::new();
        for id in 20..=22 {
            let handle = store.insert(Payload::new(id, &drops)).unwrap();
            assert!(generations.insert(handle.generation));
            assert_eq!(
                store.delete(handle),
                DeleteResult::Deleted { retired: id == 22 }
            );
        }
        let (error, payload) = store.insert(Payload::new(23, &drops)).unwrap_err();
        assert_eq!(error, InsertError::GenerationExhausted);
        drop(payload);
        assert_eq!(generations, BTreeSet::from([0, 1, 2]));
    }
}
