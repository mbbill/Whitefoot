//! Exact local resource-state comparison for the fixed-object experiment.
//! The ordinary model checker is the subject; the physical machine plus a
//! test-only binding/ownership ledger is its reference. Keep with RESULTS.md.

use super::*;
use std::collections::VecDeque;

const OBJECTS: usize = 2;
const BINDINGS: usize = 4;

#[derive(Clone)]
struct Oracle {
    machine: Machine<ObservedPointer>,
    owners: BTreeSet<Reg>,
}

impl Oracle {
    fn step(&mut self, op: &Op) -> Checked<()> {
        // The physical interpreter trusts binding discipline and disposal
        // authority. This ledger makes those concrete obligations observable
        // in the experiment; it does not read any Checker state.
        match op {
            Op::Alias(dst, src) => {
                self.machine.pointer(*src)?;
                if self.machine.bindings.contains_key(dst) {
                    return Err(Error::OccupiedBinding);
                }
            }
            Op::Rebind(dst, src) => {
                self.machine.pointer(*src)?;
                self.machine.pointer(*dst)?;
                if self.owners.contains(dst) {
                    return Err(Error::OwnerOverwrite);
                }
            }
            Op::Free(reg) => {
                self.machine.pointer(*reg)?;
                if !self.owners.contains(reg) {
                    return Err(Error::MissingOwner);
                }
            }
            Op::Read(_) | Op::Store(..) | Op::Take(_) => {}
            _ => panic!("operation outside the local experiment: {op:?}"),
        }
        self.machine.step(op, &[])?;
        if let Op::Free(reg) = op {
            self.owners.remove(reg);
        }
        Ok(())
    }

    fn finish(&self) -> Checked<()> {
        if self.owners.is_empty() {
            Ok(())
        } else {
            Err(Error::RemainingOwners)
        }
    }
}

fn entry() -> (Checker, Oracle) {
    let mut checker = Checker::new(Fault::None);
    let mut machine = Machine::<ObservedPointer>::new(OBJECTS);
    let declarations = [Op::New(0, 10), Op::New(1, 20)];
    checker.run(&declarations, &[]).unwrap();
    machine.run(&declarations, &[]).unwrap();
    (
        checker,
        Oracle {
            machine,
            owners: BTreeSet::from([0, 1]),
        },
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Slot {
    Initialized,
    Empty,
    Dead,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    bindings: [Option<(usize, bool)>; BINDINGS],
    slots: [Slot; OBJECTS],
}

fn checked_state(checker: &Checker) -> State {
    State {
        bindings: std::array::from_fn(|reg| {
            checker.bindings.get(&reg).map(|b| (b.target, b.owner))
        }),
        slots: std::array::from_fn(|target| {
            let cell = &checker.cells[target];
            match (cell.life, cell.init) {
                (Life::Dead, _) => Slot::Dead,
                (Life::Live, Init::Yes) => Slot::Initialized,
                (Life::Live, Init::No) => Slot::Empty,
                _ => Slot::Unknown,
            }
        }),
    }
}

fn concrete_state(oracle: &Oracle) -> State {
    State {
        bindings: std::array::from_fn(|reg| {
            oracle
                .machine
                .bindings
                .get(&reg)
                .map(|p| (p.slot, oracle.owners.contains(&reg)))
        }),
        slots: std::array::from_fn(|slot| match &oracle.machine.heap[slot] {
            None => Slot::Dead,
            Some(node) if node.data.is_some() => Slot::Initialized,
            Some(_) => Slot::Empty,
        }),
    }
}

fn compare_step(checker: &mut Checker, oracle: &mut Oracle, op: &Op) -> Checked<()> {
    let actual = checker.step(op, &[]);
    let expected = oracle.step(op);
    assert_eq!(actual, expected, "admission mismatch at {op:?}");
    assert_eq!(
        checked_state(checker),
        concrete_state(oracle),
        "resource-state mismatch at {op:?}"
    );
    actual
}

#[derive(Debug)]
enum Expected {
    Complete(&'static [i32]),
    Reject(usize, Error),
    Exit(Error),
}

struct Example {
    name: &'static str,
    body: Vec<Op>,
    expected: Expected,
}

fn examples() -> Vec<Example> {
    use Expected::{Complete, Exit, Reject};
    use Op::{Free, Read, Rebind, Store, Take};
    vec![
        Example {
            name: "rebinding_preserves_the_prior_copy",
            body: vec![
                Rebind(2, 1),
                Store(2, 9),
                Read(3),
                Read(2),
                Free(0),
                Free(1),
            ],
            expected: Complete(&[10, 9]),
        },
        Example {
            name: "sequential_alias_writes",
            body: vec![Store(2, 5), Store(3, 7), Read(2), Free(0), Free(1)],
            expected: Complete(&[7]),
        },
        Example {
            name: "read_after_take",
            body: vec![Take(2), Read(3)],
            expected: Reject(2, Error::NotInitialized),
        },
        Example {
            name: "restore_through_an_alias",
            body: vec![Take(2), Store(3, 7), Read(2), Free(0), Free(1)],
            expected: Complete(&[10, 7]),
        },
        Example {
            name: "copy_an_empty_slot_locator_then_restore",
            body: vec![
                Take(2),
                Rebind(3, 2),
                Store(3, 7),
                Read(2),
                Free(0),
                Free(1),
            ],
            expected: Complete(&[10, 7]),
        },
        Example {
            name: "read_after_release",
            body: vec![Free(0), Read(3)],
            expected: Reject(2, Error::NotLive),
        },
        Example {
            name: "write_cannot_revive_released_storage",
            body: vec![Free(0), Store(3, 7)],
            expected: Reject(2, Error::NotLive),
        },
        Example {
            name: "take_after_release",
            body: vec![Free(0), Take(3)],
            expected: Reject(2, Error::NotLive),
        },
        Example {
            name: "a_locator_has_no_disposal_authority",
            body: vec![Free(2)],
            expected: Reject(1, Error::MissingOwner),
        },
        Example {
            name: "owner_consumption_cannot_repeat",
            body: vec![Free(0), Free(0)],
            expected: Reject(2, Error::MissingBinding),
        },
        Example {
            name: "rebinding_cannot_erase_an_owner",
            body: vec![Rebind(0, 1)],
            expected: Reject(1, Error::OwnerOverwrite),
        },
        Example {
            name: "self_assignment_preserves_the_target",
            body: vec![Rebind(2, 2), Read(3), Free(0), Free(1)],
            expected: Complete(&[10]),
        },
        Example {
            name: "an_empty_copy_slot_can_be_released",
            body: vec![Take(2), Free(0), Free(1)],
            expected: Complete(&[10]),
        },
        Example {
            name: "inert_locators_can_be_copied_without_access",
            body: vec![Free(0), Rebind(3, 2), Rebind(2, 1), Read(2), Free(1)],
            expected: Complete(&[20]),
        },
        Example {
            name: "procedure_exit_checks_every_owner",
            body: vec![Free(0)],
            expected: Exit(Error::RemainingOwners),
        },
        Example {
            name: "take_cannot_repeat_without_restoration",
            body: vec![Take(2), Take(3)],
            expected: Reject(2, Error::NotInitialized),
        },
    ]
}

fn shared_locators(checker: &mut Checker, oracle: &mut Oracle) {
    compare_step(checker, oracle, &Op::Alias(2, 0)).unwrap();
    compare_step(checker, oracle, &Op::Alias(3, 2)).unwrap();
}

#[test]
fn specified_examples_match_their_first_error_or_values() {
    for example in examples() {
        let (mut checker, mut oracle) = entry();
        shared_locators(&mut checker, &mut oracle);
        let mut rejected = None;
        for (index, op) in example.body.iter().enumerate() {
            if let Err(error) = compare_step(&mut checker, &mut oracle, op) {
                rejected = Some((index + 1, error));
                break;
            }
        }
        match example.expected {
            Expected::Complete(values) => {
                assert_eq!(rejected, None, "{}", example.name);
                assert_eq!(checker.finish(), Ok(()), "{}", example.name);
                assert_eq!(oracle.finish(), Ok(()));
                assert_eq!(oracle.machine.output, values, "{}", example.name);
            }
            Expected::Reject(at, error) => {
                assert_eq!(rejected, Some((at, error)), "{}", example.name);
            }
            Expected::Exit(error) => {
                assert_eq!(rejected, None, "{}", example.name);
                assert_eq!(checker.finish(), Err(error));
                assert_eq!(oracle.finish(), Err(error));
            }
        }
        println!("{}: {:?}", example.name, example.expected);
    }
}

fn alphabet() -> Vec<Op> {
    let mut ops = Vec::new();
    for reg in 0..BINDINGS {
        ops.extend([
            Op::Read(reg),
            Op::Store(reg, 9),
            Op::Take(reg),
            Op::Free(reg),
        ]);
        for src in 0..BINDINGS {
            ops.push(Op::Rebind(reg, src));
            // Entry owner names cannot be redeclared as locals. Rebinding an
            // existing owner is nevertheless included as a negative boundary.
            if reg >= OBJECTS {
                ops.push(Op::Alias(reg, src));
            }
        }
    }
    ops
}

#[test]
fn all_reachable_local_resource_states_agree() {
    let (checker, oracle) = entry();
    assert_eq!(checked_state(&checker), concrete_state(&oracle));
    let mut seen = BTreeSet::from([concrete_state(&oracle)]);
    let mut queue = VecDeque::from([(checker, oracle, Vec::<Op>::new())]);
    let instructions = alphabet();
    let mut admitted = 0;
    let mut rejected = 0;
    let mut complete_exits = 0;
    while let Some((checker, oracle, path)) = queue.pop_front() {
        assert_eq!(checker.finish(), oracle.finish(), "exit after {path:?}");
        complete_exits += usize::from(oracle.finish().is_ok());
        for op in &instructions {
            let mut next_checker = checker.clone();
            let mut next_oracle = oracle.clone();
            let mut next_path = path.clone();
            next_path.push(op.clone());
            // Compare rejected transitions too. Otherwise an always-rejecting
            // checker would appear safe without checking any useful program.
            match compare_step(&mut next_checker, &mut next_oracle, op) {
                Err(_) => rejected += 1,
                Ok(()) => {
                    admitted += 1;
                    // Scalar values, output history, accumulated effects and
                    // query counters cannot select a local rule. The state
                    // key is their resource-relation quotient, not a machine
                    // snapshot or a claim about arbitrary value properties.
                    if seen.insert(concrete_state(&next_oracle)) {
                        next_oracle.machine.output.clear();
                        next_checker.accesses.clear();
                        next_checker.queries = 0;
                        queue.push_back((next_checker, next_oracle, next_path));
                    }
                }
            }
        }
    }
    assert!(admitted > 0 && rejected > 0 && complete_exits > 0);
    assert_eq!(admitted + rejected, seen.len() * instructions.len());
    println!(
        "local closure: {} states, {} instructions, {admitted} admitted transitions, \
         {rejected} rejected transitions, {complete_exits} complete exits",
        seen.len(),
        instructions.len(),
    );
}

#[test]
fn state_comparison_detects_corrupted_facts() {
    let (mut checker, mut oracle) = entry();
    shared_locators(&mut checker, &mut oracle);
    compare_step(&mut checker, &mut oracle, &Op::Rebind(2, 1)).unwrap();
    checker.bindings.get_mut(&3).unwrap().target = 1;
    assert_ne!(checked_state(&checker), concrete_state(&oracle));

    let (mut checker, mut oracle) = entry();
    shared_locators(&mut checker, &mut oracle);
    compare_step(&mut checker, &mut oracle, &Op::Take(2)).unwrap();
    checker.cells[0].init = Init::Yes;
    assert_ne!(checked_state(&checker), concrete_state(&oracle));
    assert_ne!(checker.step(&Op::Read(3), &[]), oracle.step(&Op::Read(3)));

    let (mut checker, mut oracle) = entry();
    let old_owner = checker.binding(0).unwrap();
    compare_step(&mut checker, &mut oracle, &Op::Free(0)).unwrap();
    compare_step(&mut checker, &mut oracle, &Op::Free(1)).unwrap();
    checker.bindings.insert(0, old_owner);
    assert_ne!(checked_state(&checker), concrete_state(&oracle));
    assert_ne!(checker.finish(), oracle.finish());
    println!("corrupted alias, initialization and owner records: all detected");
}

#[test]
fn trace_rebinding_and_restoration() {
    let (mut checker, mut oracle) = entry();
    shared_locators(&mut checker, &mut oracle);
    // a/r0 owns A/0, b/r1 owns B/1, p/r2 and q/r3 initially designate A.
    let trace = [
        ("p = ref(b)", Op::Rebind(2, 1)),
        ("take(q)", Op::Take(3)),
        ("put(p, 9)", Op::Store(2, 9)),
        ("put(q, 7)", Op::Store(3, 7)),
        ("read(q)", Op::Read(3)),
        ("release(a)", Op::Free(0)),
        ("release(b)", Op::Free(1)),
    ];
    println!("entry: {:?}", checked_state(&checker));
    for (source, op) in trace {
        compare_step(&mut checker, &mut oracle, &op).unwrap();
        println!("{source}: {:?}", checked_state(&checker));
    }
    assert_eq!(checker.finish(), Ok(()));
    assert_eq!(oracle.machine.output, [10, 7]);
}
