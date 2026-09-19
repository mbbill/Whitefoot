#![forbid(unsafe_code)]

//! A first-order research model, not a Whitefoot implementation. The checker
//! uses symbolic allocations. Execution uses a finite, reusable physical store.
//! The generation-bearing pointer is an oracle; the other pointer is an address.

use std::collections::{BTreeMap, BTreeSet};

type Reg = usize;
type Target = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Part {
    Data,
    Link,
    Whole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Access {
    target: Target,
    part: Part,
    write: bool,
}

#[derive(Clone, Debug)]
enum Op {
    New(Reg, i32),
    Alias(Reg, Reg),
    Rebind(Reg, Reg),
    Move(Reg, Reg),
    Read(Reg),
    Store(Reg, i32),
    Take(Reg),
    SetLink(Reg, Reg),
    GetLink(Reg, Reg),
    Free(Reg),
    Call(usize, Vec<Reg>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Life {
    Live,
    Dead,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Init {
    Yes,
    No,
    Entry(Target),
    Unknown,
}

#[derive(Clone, Debug)]
struct CellFacts {
    life: Life,
    init: Init,
    link: Option<Target>,
}

#[derive(Clone, Copy, Debug)]
struct Binding {
    target: Target,
    owner: bool,
    loaded_from: Option<Target>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    None,
    ReuseIdentity,
    RetargetSaved,
    SeparateFormals,
}

#[derive(Clone, Debug)]
struct Checker {
    bindings: BTreeMap<Reg, Binding>,
    cells: Vec<CellFacts>,
    separated: BTreeSet<(Target, Target)>,
    accesses: BTreeSet<Access>,
    queries: usize,
    fault: Fault,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Error {
    MissingBinding,
    OccupiedBinding,
    OwnerOverwrite,
    MissingOwner,
    RemainingOwners,
    NotLive,
    NotInitialized,
    UnknownLink,
    NeedSeparation,
    InvalidSummary,
    Capacity,
    Stale,
}

type Checked<T> = Result<T, Error>;

fn pair(a: Target, b: Target) -> (Target, Target) {
    (a.min(b), a.max(b))
}

impl Checker {
    fn new(fault: Fault) -> Self {
        Self {
            bindings: BTreeMap::new(),
            cells: Vec::new(),
            separated: BTreeSet::new(),
            accesses: BTreeSet::new(),
            queries: 0,
            fault,
        }
    }

    fn binding(&self, reg: Reg) -> Checked<Binding> {
        self.bindings
            .get(&reg)
            .copied()
            .ok_or(Error::MissingBinding)
    }

    fn bind(&mut self, reg: Reg, binding: Binding) -> Checked<()> {
        if self.bindings.contains_key(&reg) {
            return Err(Error::OccupiedBinding);
        }
        self.bindings.insert(reg, binding);
        Ok(())
    }

    fn may_alias(&mut self, a: Target, b: Target) -> bool {
        self.queries += 1;
        a == b || !self.separated.contains(&pair(a, b))
    }

    fn live(&self, target: Target) -> Checked<()> {
        if self.cells[target].life == Life::Live {
            Ok(())
        } else {
            Err(Error::NotLive)
        }
    }

    fn initialized(&self, target: Target) -> Checked<()> {
        self.live(target)?;
        if self.cells[target].init == Init::Yes {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn access(&mut self, target: Target, part: Part, write: bool) {
        self.accesses.insert(Access {
            target,
            part,
            write,
        });
    }

    fn new_target(&mut self) -> Target {
        // Only the negative control identifies a new allocation with a dead one.
        if self.fault == Fault::ReuseIdentity
            && let Some(target) = self.cells.iter().position(|c| c.life == Life::Dead)
        {
            self.cells[target] = CellFacts {
                life: Life::Live,
                init: Init::Yes,
                link: None,
            };
            return target;
        }
        let target = self.cells.len();
        for old in 0..target {
            // The model allocates independent roots. It does not infer spatial
            // separation from spelling or model an overlapping parent owner.
            if self.cells[old].life == Life::Live {
                self.separated.insert(pair(target, old));
            }
        }
        self.cells.push(CellFacts {
            life: Life::Live,
            init: Init::Yes,
            link: None,
        });
        target
    }

    fn update_init(&mut self, target: Target, next: Init) {
        for other in 0..self.cells.len() {
            if other == target {
                self.cells[other].init = next;
            } else if self.may_alias(target, other) && self.cells[other].init != next {
                // A possible alias may or may not have changed. No alias-case
                // enumeration and no equality test is inserted into the program.
                self.cells[other].init = Init::Unknown;
            }
        }
    }

    fn end_target(&mut self, target: Target) {
        for other in 0..self.cells.len() {
            if self.cells[other].life != Life::Dead && self.may_alias(target, other) {
                self.cells[other].life = if other == target {
                    Life::Dead
                } else {
                    Life::Unknown
                };
                self.cells[other].init = Init::Unknown;
                self.cells[other].link = None;
            }
        }
    }

    fn step(&mut self, op: &Op, library: &[Function]) -> Checked<()> {
        match op {
            Op::New(dst, _) => {
                let target = self.new_target();
                self.bind(
                    *dst,
                    Binding {
                        target,
                        owner: true,
                        loaded_from: None,
                    },
                )?;
            }
            Op::Alias(dst, src) => {
                let mut binding = self.binding(*src)?;
                binding.owner = false;
                self.bind(*dst, binding)?;
            }
            Op::Rebind(dst, src) => {
                let mut binding = self.binding(*src)?;
                if self.binding(*dst)?.owner {
                    return Err(Error::OwnerOverwrite);
                }
                binding.owner = false;
                self.bindings.insert(*dst, binding);
            }
            Op::Move(dst, src) => {
                let binding = self.binding(*src)?;
                if !binding.owner {
                    return Err(Error::MissingOwner);
                }
                self.bind(*dst, binding)?;
                self.bindings.remove(src);
            }
            Op::Read(reg) => {
                let target = self.binding(*reg)?.target;
                self.initialized(target)?;
                self.access(target, Part::Data, false);
            }
            Op::Store(reg, _) => {
                let target = self.binding(*reg)?.target;
                self.live(target)?;
                self.update_init(target, Init::Yes);
                self.access(target, Part::Data, true);
            }
            Op::Take(reg) => {
                let target = self.binding(*reg)?.target;
                self.initialized(target)?;
                self.update_init(target, Init::No);
                self.access(target, Part::Data, false);
                self.access(target, Part::Data, true);
            }
            Op::SetLink(at, to) => {
                let parent = self.binding(*at)?.target;
                let target = self.binding(*to)?.target;
                self.live(parent)?;
                for other in 0..self.cells.len() {
                    if self.may_alias(parent, other) {
                        self.cells[other].link = None;
                    }
                }
                self.cells[parent].link = Some(target);
                if self.fault == Fault::RetargetSaved {
                    for binding in self.bindings.values_mut() {
                        if binding.loaded_from == Some(parent) {
                            binding.target = target;
                        }
                    }
                }
                self.access(parent, Part::Link, true);
            }
            Op::GetLink(dst, at) => {
                let parent = self.binding(*at)?.target;
                self.live(parent)?;
                let target = self.cells[parent].link.ok_or(Error::UnknownLink)?;
                self.bind(
                    *dst,
                    Binding {
                        target,
                        owner: false,
                        loaded_from: Some(parent),
                    },
                )?;
                self.access(parent, Part::Link, false);
            }
            Op::Free(reg) => {
                let binding = self.binding(*reg)?;
                if !binding.owner {
                    return Err(Error::MissingOwner);
                }
                self.live(binding.target)?;
                self.end_target(binding.target);
                self.bindings.remove(reg);
                self.access(binding.target, Part::Whole, true);
            }
            Op::Call(index, args) => self.call(&library[*index].summary, args)?,
        }
        Ok(())
    }

    fn run(&mut self, ops: &[Op], library: &[Function]) -> Checked<()> {
        for op in ops {
            self.step(op, library)?;
        }
        Ok(())
    }

    fn finish(&self) -> Checked<()> {
        if self.bindings.values().any(|binding| binding.owner) {
            Err(Error::RemainingOwners)
        } else {
            Ok(())
        }
    }

    fn independent(&mut self, a: &BTreeSet<Access>, b: &BTreeSet<Access>) -> bool {
        for left in a {
            for right in b {
                if (left.write || right.write)
                    && (left.part == right.part
                        || left.part == Part::Whole
                        || right.part == Part::Whole)
                    && self.may_alias(left.target, right.target)
                {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Clone, Copy, Debug)]
enum Subject {
    Arg(usize),
    Link(usize), // The entry target of the preceding subject's reference field.
}

#[derive(Clone, Copy, Debug)]
enum Post {
    Preserve,
    Initialized,
    Empty,
    Dead,
}

#[derive(Clone, Debug)]
struct Summary {
    owners: Vec<bool>,
    subjects: Vec<Subject>,
    needs_init: BTreeSet<usize>,
    separate: Vec<(usize, usize)>,
    effects: BTreeSet<Access>, // Target indexes name subjects, not physical slots.
    post: Vec<Post>,
    link_updates: BTreeMap<usize, usize>,
}

#[derive(Clone, Debug)]
struct Function {
    summary: Summary,
    body: Vec<Op>,
}

impl Checker {
    fn call(&mut self, summary: &Summary, args: &[Reg]) -> Checked<()> {
        if args.len() != summary.owners.len() {
            return Err(Error::InvalidSummary);
        }
        let mut targets: Vec<Target> = Vec::new();
        for subject in &summary.subjects {
            let target = match *subject {
                Subject::Arg(arg) => self.binding(args[arg])?.target,
                Subject::Link(parent) => {
                    self.cells[targets[parent]].link.ok_or(Error::UnknownLink)?
                }
            };
            self.live(target)?;
            targets.push(target);
        }
        for &index in &summary.needs_init {
            self.initialized(targets[index])?;
        }
        for &(a, b) in &summary.separate {
            if self.may_alias(targets[a], targets[b]) {
                return Err(Error::NeedSeparation);
            }
        }
        let mut transferred = BTreeSet::new();
        for (arg, owner) in summary.owners.iter().enumerate() {
            if *owner {
                let binding = self.binding(args[arg])?;
                if !binding.owner || !transferred.insert(binding.target) {
                    return Err(Error::MissingOwner);
                }
            }
        }
        // A checked summary exports a state transformer. This model's bodies
        // have no result values: every owned argument must be consumed.
        for effect in &summary.effects {
            self.access(targets[effect.target], effect.part, effect.write);
        }
        // Destructive updates first forget possibly aliased caller facts. The
        // declared exit assertions are then established together. Verification
        // under symbolic aliases must already prove those assertions compatible.
        for (index, post) in summary.post.iter().enumerate() {
            match post {
                Post::Initialized => self.update_init(targets[index], Init::Yes),
                Post::Empty => self.update_init(targets[index], Init::No),
                Post::Dead => self.end_target(targets[index]),
                Post::Preserve => {}
            }
        }
        for (index, post) in summary.post.iter().enumerate() {
            match post {
                Post::Initialized => self.cells[targets[index]].init = Init::Yes,
                Post::Empty => self.cells[targets[index]].init = Init::No,
                Post::Dead | Post::Preserve => {}
            }
        }
        for &index in summary.link_updates.keys() {
            let target = targets[index];
            for other in 0..self.cells.len() {
                if self.may_alias(target, other) {
                    self.cells[other].link = None;
                }
            }
        }
        for (&index, &target) in &summary.link_updates {
            self.cells[targets[index]].link = Some(targets[target]);
        }
        for (arg, owner) in summary.owners.iter().enumerate() {
            if *owner {
                self.bindings.remove(&args[arg]);
            }
        }
        Ok(())
    }
}

fn verify(function: &Function, fault: Fault) -> Checked<usize> {
    let summary = &function.summary;
    if summary.subjects.len() != summary.post.len() {
        return Err(Error::InvalidSummary);
    }
    let mut checker = Checker::new(fault);
    for (index, subject) in summary.subjects.iter().enumerate() {
        checker.cells.push(CellFacts {
            life: Life::Live,
            init: if summary.needs_init.contains(&index) {
                Init::Yes
            } else {
                Init::Entry(index)
            },
            link: None,
        });
        match *subject {
            Subject::Arg(arg) => checker.bind(
                arg,
                Binding {
                    target: index,
                    owner: summary.owners[arg],
                    loaded_from: None,
                },
            )?,
            Subject::Link(parent) => checker.cells[parent].link = Some(index),
        }
    }
    for &(a, b) in &summary.separate {
        checker.separated.insert(pair(a, b));
    }
    if fault == Fault::SeparateFormals {
        for a in 0..checker.cells.len() {
            for b in 0..a {
                checker.separated.insert(pair(a, b));
            }
        }
    }
    let entry = checker.cells.clone();
    checker.run(&function.body, &[])?;
    if checker.accesses != summary.effects {
        return Err(Error::InvalidSummary);
    }
    for (index, post) in summary.post.iter().enumerate() {
        let after = &checker.cells[index];
        match post {
            Post::Dead if after.life == Life::Dead => {}
            Post::Initialized if after.life == Life::Live && after.init == Init::Yes => {}
            Post::Empty if after.life == Life::Live && after.init == Init::No => {}
            Post::Preserve
                if after.life == entry[index].life && after.init == entry[index].init => {}
            _ => return Err(Error::InvalidSummary),
        }
        let required_link = summary
            .link_updates
            .get(&index)
            .copied()
            .or(entry[index].link);
        if !matches!(post, Post::Dead) && after.link != required_link {
            return Err(Error::InvalidSummary);
        }
    }
    if checker.finish().is_err() {
        return Err(Error::InvalidSummary);
    }
    Ok(checker.queries)
}

trait Pointer: Clone + Copy + std::fmt::Debug {
    type Stamp: Clone + Copy + Default + std::fmt::Debug;
    fn next(stamp: &mut Self::Stamp);
    fn make(slot: usize, stamp: Self::Stamp) -> Self;
    fn slot(self) -> usize;
    fn matches(self, stamp: Self::Stamp) -> bool;
}

#[derive(Clone, Copy, Debug)]
struct ObservedPointer {
    slot: usize,
    generation: usize,
}

impl Pointer for ObservedPointer {
    type Stamp = usize;
    fn next(stamp: &mut usize) {
        *stamp += 1;
    }
    fn make(slot: usize, generation: usize) -> Self {
        Self { slot, generation }
    }
    fn slot(self) -> usize {
        self.slot
    }
    fn matches(self, stamp: usize) -> bool {
        self.generation == stamp
    }
}

#[derive(Clone, Copy, Debug)]
struct Address(usize);

impl Pointer for Address {
    type Stamp = ();
    fn next(_: &mut ()) {}
    fn make(slot: usize, _: ()) -> Self {
        Self(slot)
    }
    fn slot(self) -> usize {
        self.0
    }
    fn matches(self, _: ()) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
struct Node<P: Pointer> {
    stamp: P::Stamp,
    data: Option<i32>,
    link: Option<P>,
}

#[derive(Clone, Debug)]
struct Machine<P: Pointer> {
    heap: Vec<Option<Node<P>>>,
    bindings: BTreeMap<Reg, P>,
    stamp: P::Stamp,
    output: Vec<i32>,
    allocations: Vec<usize>,
}

impl<P: Pointer> Machine<P> {
    fn new(capacity: usize) -> Self {
        Self {
            heap: vec![None; capacity],
            bindings: BTreeMap::new(),
            stamp: P::Stamp::default(),
            output: Vec::new(),
            allocations: Vec::new(),
        }
    }

    fn pointer(&self, reg: Reg) -> Checked<P> {
        self.bindings
            .get(&reg)
            .copied()
            .ok_or(Error::MissingBinding)
    }

    fn node(&self, pointer: P) -> Checked<&Node<P>> {
        let node = self.heap[pointer.slot()].as_ref().ok_or(Error::NotLive)?;
        if !pointer.matches(node.stamp) {
            return Err(Error::Stale);
        }
        Ok(node)
    }

    fn step(&mut self, op: &Op, library: &[Function]) -> Checked<()> {
        match op {
            Op::New(dst, value) => {
                let slot = self
                    .heap
                    .iter()
                    .position(Option::is_none)
                    .ok_or(Error::Capacity)?;
                P::next(&mut self.stamp);
                let pointer = P::make(slot, self.stamp);
                self.heap[slot] = Some(Node {
                    stamp: self.stamp,
                    data: Some(*value),
                    link: None,
                });
                self.bindings.insert(*dst, pointer);
                self.allocations.push(slot);
            }
            Op::Alias(dst, src) | Op::Rebind(dst, src) => {
                self.bindings.insert(*dst, self.pointer(*src)?);
            }
            Op::Move(dst, src) => {
                let pointer = self.pointer(*src)?;
                self.bindings.remove(src);
                self.bindings.insert(*dst, pointer);
            }
            Op::Read(reg) => {
                let node = self.node(self.pointer(*reg)?)?;
                self.output.push(node.data.ok_or(Error::NotInitialized)?);
            }
            Op::Store(reg, value) => {
                let pointer = self.pointer(*reg)?;
                self.node(pointer)?;
                self.heap[pointer.slot()].as_mut().unwrap().data = Some(*value);
            }
            Op::Take(reg) => {
                let pointer = self.pointer(*reg)?;
                let value = self.node(pointer)?.data.ok_or(Error::NotInitialized)?;
                self.output.push(value);
                self.heap[pointer.slot()].as_mut().unwrap().data = None;
            }
            Op::SetLink(at, to) => {
                let pointer = self.pointer(*at)?;
                let target = self.pointer(*to)?;
                self.node(pointer)?;
                self.heap[pointer.slot()].as_mut().unwrap().link = Some(target);
            }
            Op::GetLink(dst, at) => {
                let target = self
                    .node(self.pointer(*at)?)?
                    .link
                    .ok_or(Error::UnknownLink)?;
                self.bindings.insert(*dst, target);
            }
            Op::Free(reg) => {
                let pointer = self.pointer(*reg)?;
                self.node(pointer)?;
                self.heap[pointer.slot()] = None;
                self.bindings.remove(reg);
            }
            Op::Call(index, args) => {
                let function = &library[*index];
                let mut local = BTreeMap::new();
                for (index, arg) in args.iter().enumerate() {
                    local.insert(index, self.pointer(*arg)?);
                }
                let mut caller = std::mem::replace(&mut self.bindings, local);
                self.run(&function.body, library)?;
                for (index, owner) in function.summary.owners.iter().enumerate() {
                    if *owner {
                        caller.remove(&args[index]);
                    }
                }
                self.bindings = caller;
            }
        }
        Ok(())
    }

    fn run(&mut self, ops: &[Op], library: &[Function]) -> Checked<()> {
        for op in ops {
            self.step(op, library)?;
        }
        Ok(())
    }
}

fn effects(entries: &[(usize, Part, bool)]) -> BTreeSet<Access> {
    entries
        .iter()
        .map(|&(target, part, write)| Access {
            target,
            part,
            write,
        })
        .collect()
}

fn write_pair(count: usize) -> Function {
    Function {
        summary: Summary {
            owners: vec![false; count],
            subjects: (0..count).map(Subject::Arg).collect(),
            needs_init: (0..count).collect(),
            separate: vec![],
            effects: effects(
                &(0..count)
                    .map(|p| (p, Part::Data, true))
                    .collect::<Vec<_>>(),
            ),
            post: vec![Post::Initialized; count],
            link_updates: BTreeMap::new(),
        },
        body: (0..count).map(|p| Op::Store(p, p as i32)).collect(),
    }
}

fn library() -> Vec<Function> {
    let release = Function {
        summary: Summary {
            owners: vec![true],
            subjects: vec![Subject::Arg(0), Subject::Link(0)],
            needs_init: BTreeSet::new(),
            separate: vec![(0, 1)],
            effects: effects(&[
                (0, Part::Link, false),
                (1, Part::Data, true),
                (0, Part::Whole, true),
            ]),
            post: vec![Post::Dead, Post::Initialized],
            link_updates: BTreeMap::new(),
        },
        body: vec![Op::GetLink(1, 0), Op::Store(1, 1), Op::Free(0)],
    };
    let read = Function {
        summary: Summary {
            owners: vec![false],
            subjects: vec![Subject::Arg(0)],
            needs_init: BTreeSet::from([0]),
            separate: vec![],
            effects: effects(&[(0, Part::Data, false)]),
            post: vec![Post::Preserve],
            link_updates: BTreeMap::new(),
        },
        body: vec![Op::Read(0)],
    };
    let control_write = Function {
        summary: Summary {
            owners: vec![false],
            subjects: vec![Subject::Arg(0), Subject::Link(0)],
            needs_init: BTreeSet::new(),
            separate: vec![(0, 1)],
            effects: effects(&[(0, Part::Link, false), (1, Part::Data, true)]),
            post: vec![Post::Preserve, Post::Initialized],
            link_updates: BTreeMap::new(),
        },
        body: vec![Op::GetLink(1, 0), Op::Store(1, 7)],
    };
    let take = Function {
        summary: Summary {
            owners: vec![false],
            subjects: vec![Subject::Arg(0)],
            needs_init: BTreeSet::from([0]),
            separate: vec![],
            effects: effects(&[(0, Part::Data, false), (0, Part::Data, true)]),
            post: vec![Post::Empty],
            link_updates: BTreeMap::new(),
        },
        body: vec![Op::Take(0)],
    };
    let retarget = Function {
        summary: Summary {
            owners: vec![false, false],
            subjects: vec![Subject::Arg(0), Subject::Arg(1)],
            needs_init: BTreeSet::new(),
            separate: vec![],
            effects: effects(&[(0, Part::Link, true)]),
            post: vec![Post::Preserve; 2],
            link_updates: BTreeMap::from([(0, 1)]),
        },
        body: vec![Op::SetLink(0, 1)],
    };
    vec![write_pair(2), release, read, control_write, take, retarget]
}

fn agree(ops: &[Op], library: &[Function]) -> Checked<Machine<Address>> {
    Checker::new(Fault::None).run(ops, library)?;
    let mut oracle = Machine::<ObservedPointer>::new(ops.len() + 1);
    let mut erased = Machine::<Address>::new(ops.len() + 1);
    oracle.run(ops, library)?;
    erased.run(ops, library)?;
    assert_eq!(oracle.output, erased.output);
    assert_eq!(oracle.allocations, erased.allocations);
    Ok(erased)
}

fn reuse_program() -> Vec<Op> {
    vec![
        Op::New(0, 0), // Shared control state, not a resource-specific rule.
        Op::New(1, 11),
        Op::New(2, 22),
        Op::New(3, 33),
        Op::SetLink(1, 0),
        Op::SetLink(2, 0),
        Op::SetLink(3, 0),
        Op::Alias(4, 2),
        Op::Move(5, 2),
        Op::Call(1, vec![5]),
        Op::New(6, 44),
        Op::SetLink(6, 0),
        Op::Read(1),
        Op::Read(3),
        Op::Read(6),
        Op::Call(1, vec![1]),
        Op::Call(1, vec![3]),
        Op::Call(1, vec![6]),
        Op::Free(0),
    ]
}

#[derive(Default, Debug)]
struct Counts {
    attempted: usize,
    accepted: usize,
    reused: usize,
}

fn enumerate(depth: usize, fault: Fault, library: &[Function]) -> (Counts, Option<Vec<Op>>) {
    fn visit(
        left: usize,
        program: &mut Vec<Op>,
        checker: &Checker,
        oracle: &Machine<ObservedPointer>,
        erased: &Machine<Address>,
        counts: &mut Counts,
        library: &[Function],
    ) -> Option<Vec<Op>> {
        if left == 0 {
            return None;
        }
        let regs: Vec<_> = checker.bindings.keys().copied().collect();
        let free = (0..3).find(|reg| !checker.bindings.contains_key(reg));
        let mut next = Vec::new();
        if let Some(dst) = free {
            next.push(Op::New(dst, program.len() as i32));
            for &src in &regs {
                next.push(Op::Alias(dst, src));
                next.push(Op::Move(dst, src));
                next.push(Op::GetLink(dst, src));
            }
        }
        for &at in &regs {
            next.extend([Op::Read(at), Op::Store(at, 9), Op::Take(at), Op::Free(at)]);
            for &to in &regs {
                next.push(Op::SetLink(at, to));
                next.push(Op::Call(0, vec![at, to]));
            }
        }
        for op in next {
            counts.attempted += 1;
            let mut checked = checker.clone();
            if checked.step(&op, library).is_err() {
                continue;
            }
            counts.accepted += 1;
            let mut observed = oracle.clone();
            let mut address = erased.clone();
            program.push(op.clone());
            if observed.step(&op, library).is_err()
                || address.step(&op, library).is_err()
                || observed.output != address.output
            {
                return Some(program.clone());
            }
            if matches!(op, Op::New(..)) {
                let slots = &observed.allocations;
                if slots[..slots.len() - 1].contains(slots.last().unwrap()) {
                    counts.reused += 1;
                }
            }
            if let Some(counterexample) = visit(
                left - 1,
                program,
                &checked,
                &observed,
                &address,
                counts,
                library,
            ) {
                return Some(counterexample);
            }
            program.pop();
        }
        None
    }
    let mut counts = Counts::default();
    let found = visit(
        depth,
        &mut Vec::new(),
        &Checker::new(fault),
        &Machine::new(depth),
        &Machine::new(depth),
        &mut counts,
        library,
    );
    (counts, found)
}

fn main() {
    let library = library();
    for function in &library {
        verify(function, Fault::None).unwrap();
    }
    let reused = agree(&reuse_program(), &library).unwrap();
    assert_eq!(reused.allocations, [0, 1, 2, 3, 2]);
    assert_eq!(reused.output, [11, 33, 44]);
    println!(
        "reuse: slots {:?}, values {:?}",
        reused.allocations, reused.output
    );
    let mut access_state = Checker::new(Fault::None);
    access_state
        .run(
            &[
                Op::New(0, 1),
                Op::New(1, 2),
                Op::Alias(2, 0),
                Op::Rebind(2, 1),
            ],
            &[],
        )
        .unwrap();
    assert_eq!(access_state.binding(2).unwrap().target, 1);
    assert!(access_state.independent(
        &effects(&[(0, Part::Data, true)]),
        &effects(&[(1, Part::Data, true)]),
    ));
    let (counts, found) = enumerate(6, Fault::None, &library);
    assert!(found.is_none(), "{found:?}");
    println!("depth 6, three registers: {counts:?}; no counterexample");
    let (_, broken) = enumerate(6, Fault::ReuseIdentity, &library);
    assert!(broken.is_some());
    println!("reused-identity negative control: {:?}", broken.unwrap());
    for count in [8, 32, 128, 512] {
        let queries = verify(&write_pair(count), Fault::None).unwrap();
        assert_eq!(queries, count * (count - 1));
        println!("symbolic parameters {count}: {queries} may-alias queries");
    }
}

#[cfg(test)]
mod local;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_control_reuse_and_owner_move() {
        let result = agree(&reuse_program(), &library()).unwrap();
        assert_eq!(result.allocations, [0, 1, 2, 3, 2]);
        assert_eq!(result.output, [11, 33, 44]);
        assert!(result.heap.iter().all(Option::is_none));
        Machine::<ObservedPointer>::new(4)
            .run(&reuse_program(), &library())
            .unwrap();
        Machine::<Address>::new(4)
            .run(&reuse_program(), &library())
            .unwrap();
    }

    #[test]
    fn old_alias_does_not_revive_when_address_is_reused() {
        let program = vec![
            Op::New(0, 1),
            Op::Alias(1, 0),
            Op::Free(0),
            Op::New(0, 2),
            Op::Read(1),
        ];
        assert_eq!(
            Checker::new(Fault::None).run(&program, &[]),
            Err(Error::NotLive)
        );
        let mut faulty = Checker::new(Fault::ReuseIdentity);
        faulty.run(&program, &[]).unwrap();
        assert_eq!(
            Machine::<ObservedPointer>::new(1).run(&program, &[]),
            Err(Error::Stale)
        );
        let mut erased = Machine::<Address>::new(1);
        erased.run(&program, &[]).unwrap();
        assert_eq!(erased.output, [2]);
    }

    #[test]
    fn saved_link_keeps_its_original_target() {
        let prefix = vec![
            Op::New(0, 0),
            Op::New(1, 11),
            Op::New(2, 22),
            Op::SetLink(0, 1),
            Op::GetLink(3, 0),
            Op::SetLink(0, 2),
        ];
        let mut read = prefix.clone();
        read.push(Op::Read(3));
        assert_eq!(agree(&read, &[]).unwrap().output, [11]);
        let mut stale = prefix;
        stale.extend([Op::Free(1), Op::Read(3)]);
        assert_eq!(
            Checker::new(Fault::None).run(&stale, &[]),
            Err(Error::NotLive)
        );
        Checker::new(Fault::RetargetSaved).run(&stale, &[]).unwrap();
        assert_eq!(
            Machine::<ObservedPointer>::new(3).run(&stale, &[]),
            Err(Error::NotLive)
        );
    }

    #[test]
    fn cyclic_edges_and_dead_locators_can_remain_in_live_storage() {
        let program = vec![
            Op::New(0, 10),
            Op::New(1, 20),
            Op::SetLink(0, 1),
            Op::SetLink(1, 0),
            Op::Alias(2, 1),
            Op::Free(1),
            Op::GetLink(3, 0),
            Op::SetLink(0, 2),
            Op::Read(0),
            Op::Free(0),
        ];
        assert_eq!(agree(&program, &[]).unwrap().output, [10]);
    }

    #[test]
    fn aliases_do_not_own_and_moves_do_not_relocate() {
        let prefix = vec![Op::New(0, 4), Op::Alias(1, 0), Op::Move(2, 0)];
        let mut use_alias = prefix.clone();
        use_alias.extend([Op::Store(1, 9), Op::Read(2), Op::Free(2)]);
        let result = agree(&use_alias, &[]).unwrap();
        assert_eq!(result.allocations, [0]);
        assert_eq!(result.output, [9]);
        let mut wrong = prefix;
        wrong.push(Op::Free(1));
        assert_eq!(
            Checker::new(Fault::None).run(&wrong, &[]),
            Err(Error::MissingOwner)
        );
    }

    #[test]
    fn generic_call_admits_equal_and_separate_targets_with_one_body() {
        let library = library();
        verify(&library[0], Fault::None).unwrap();
        let same = vec![Op::New(0, 9), Op::Call(0, vec![0, 0]), Op::Read(0)];
        assert_eq!(agree(&same, &library).unwrap().output, [1]);
        let separate = vec![
            Op::New(0, 9),
            Op::New(1, 9),
            Op::Call(0, vec![0, 1]),
            Op::Read(0),
            Op::Read(1),
        ];
        assert_eq!(agree(&separate, &library).unwrap().output, [0, 1]);
    }

    #[test]
    fn unknown_aliases_cannot_keep_initialization_after_take() {
        let mut function = Function {
            summary: Summary {
                owners: vec![false, false],
                subjects: vec![Subject::Arg(0), Subject::Arg(1)],
                needs_init: BTreeSet::from([0, 1]),
                separate: vec![],
                effects: effects(&[
                    (1, Part::Data, false),
                    (1, Part::Data, true),
                    (0, Part::Data, false),
                ]),
                post: vec![Post::Preserve, Post::Empty],
                link_updates: BTreeMap::new(),
            },
            body: vec![Op::Take(1), Op::Read(0)],
        };
        assert_eq!(verify(&function, Fault::None), Err(Error::NotInitialized));
        verify(&function, Fault::SeparateFormals).unwrap();
        let library = vec![function.clone()];
        let same = vec![Op::New(0, 7), Op::Call(0, vec![0, 0])];
        Checker::new(Fault::None).run(&same, &library).unwrap();
        assert_eq!(
            Machine::<ObservedPointer>::new(1).run(&same, &library),
            Err(Error::NotInitialized)
        );
        function.summary.separate.push((0, 1));
        verify(&function, Fault::None).unwrap();
        let library = vec![function];
        assert_eq!(
            Checker::new(Fault::None).run(&same, &library),
            Err(Error::NeedSeparation)
        );
        let valid = vec![Op::New(0, 7), Op::New(1, 8), Op::Call(0, vec![0, 1])];
        assert_eq!(agree(&valid, &library).unwrap().output, [8, 7]);
    }

    #[test]
    fn nested_read_rejects_a_temporarily_uninitialized_alias() {
        let mut program = vec![Op::New(0, 7), Op::Alias(1, 0), Op::Take(0)];
        let mut bad = program.clone();
        bad.push(Op::Call(2, vec![1]));
        assert_eq!(
            Checker::new(Fault::None).run(&bad, &library()),
            Err(Error::NotInitialized)
        );
        program.extend([Op::Store(0, 8), Op::Call(2, vec![1])]);
        assert_eq!(agree(&program, &library()).unwrap().output, [7, 8]);
    }

    #[test]
    fn selected_effects_preserve_payload_separation_and_control_aliases() {
        let prefix = vec![
            Op::New(0, 0),
            Op::New(1, 1),
            Op::New(2, 2),
            Op::SetLink(1, 0),
            Op::SetLink(2, 0),
        ];
        let library = library();
        let mut state = Checker::new(Fault::None);
        state.run(&prefix, &library).unwrap();
        let row = |op: Op| {
            let mut next = state.clone();
            next.accesses.clear();
            next.step(&op, &library).unwrap();
            next.accesses
        };
        let data1 = row(Op::Store(1, 3));
        let data2 = row(Op::Store(2, 4));
        let control1 = row(Op::Call(3, vec![1]));
        let control2 = row(Op::Call(3, vec![2]));
        let release1 = row(Op::Call(1, vec![1]));
        let release2 = row(Op::Call(1, vec![2]));
        assert!(state.independent(&data1, &data2));
        assert!(!state.independent(&control1, &control2));
        assert!(!state.independent(&release1, &release2));
        assert!(state.independent(&data2, &release1));
        assert!(!state.independent(&data1, &release1));
        assert!(state.independent(&data1, &control2));
    }

    #[test]
    fn checked_summary_rejects_an_unreported_write() {
        let mut function = library().remove(2);
        function.body.push(Op::Store(0, 4));
        assert_eq!(verify(&function, Fault::None), Err(Error::InvalidSummary));
    }

    #[test]
    fn a_call_exports_retargeting_without_retargeting_saved_aliases() {
        let library = library();
        verify(&library[5], Fault::None).unwrap();
        let mut program = vec![
            Op::New(0, 0),
            Op::New(1, 11),
            Op::New(2, 22),
            Op::SetLink(0, 1),
            Op::GetLink(3, 0),
            Op::Call(5, vec![0, 2]),
            Op::GetLink(4, 0),
            Op::Read(3),
            Op::Read(4),
        ];
        assert_eq!(agree(&program, &library).unwrap().output, [11, 22]);
        program.extend([Op::Free(1), Op::Read(3)]);
        assert_eq!(
            Checker::new(Fault::None).run(&program, &library),
            Err(Error::NotLive)
        );
    }

    #[test]
    fn changing_a_checked_body_keeps_caller_acceptance_modular() {
        let first = write_pair(2);
        let mut second = first.clone();
        second.body.reverse();
        verify(&first, Fault::None).unwrap();
        verify(&second, Fault::None).unwrap();
        let program = vec![Op::New(0, 9), Op::Call(0, vec![0, 0]), Op::Read(0)];
        // The contract specifies initialized state and the accesses, not the
        // resulting integer. Each body keeps its own sequential behavior.
        assert_eq!(agree(&program, &[first]).unwrap().output, [1]);
        assert_eq!(agree(&program, &[second]).unwrap().output, [0]);
    }

    #[test]
    fn checker_and_observational_machine_agree_on_bounded_programs() {
        let library = library();
        let (counts, result) = enumerate(5, Fault::None, &library);
        assert!(result.is_none(), "{result:?}");
        assert!(counts.accepted > 1000);
        assert!(counts.reused > 0);
        assert!(enumerate(6, Fault::ReuseIdentity, &library).1.is_some());
    }
}
