use std::fmt;

const NONE: usize = usize::MAX;
const SENTINEL: usize = 0;
const UNINIT: u64 = 0xd1d1_d1d1_d1d1_d1d1;
const FREED: u64 = 0xf3f3_f3f3_f3f3_f3f3;

const TAG_LEAF: u64 = 0;
const TAG_S: u64 = 1;
const TAG_T: u64 = 2;
const TAG_RING: u64 = 3;

// E is four words in this model: one tag/padding word and one payload word
// for each payload-bearing variant, in declaration order.
const E_WORDS: usize = 4;
const S_SLOT: usize = 1;
const T_SLOT: usize = 2;
const RING_SLOT: usize = 3;
const LEAF_LINK_SLOT: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocKind {
    RootE,
    BoxE,
    S,
    T,
    Ring,
}

#[derive(Debug)]
struct Allocation {
    base: usize,
    words: usize,
    kind: AllocKind,
    freed: bool,
}

struct Heap {
    words: Vec<u64>,
    next: usize,
    allocations: Vec<Allocation>,
    allocation_at: Vec<usize>,
    observed_frees: Vec<usize>,
    observed_len: usize,
    sealed: bool,
}

impl Heap {
    fn new(word_capacity: usize, allocation_capacity: usize) -> Self {
        let mut words = vec![UNINIT; word_capacity];
        words[SENTINEL] = 0;
        Self {
            words,
            // Keep zero as a distinguished null predecessor. Every word index
            // is an 8-byte-aligned address in the modeled target.
            next: 8,
            allocations: Vec::with_capacity(allocation_capacity),
            allocation_at: vec![NONE; word_capacity],
            observed_frees: Vec::new(),
            observed_len: 0,
            sealed: false,
        }
    }

    fn alloc(&mut self, words: usize, kind: AllocKind) -> usize {
        assert!(!self.sealed, "allocation after traversal was prepared");
        assert!(words > 0);
        assert!(self.next + words <= self.words.len(), "fixture heap exhausted");
        let base = self.next;
        self.next += words;
        let index = self.allocations.len();
        assert_eq!(self.allocation_at[base], NONE);
        self.allocation_at[base] = index;
        self.allocations.push(Allocation {
            base,
            words,
            kind,
            freed: false,
        });
        base
    }

    fn set_kind(&mut self, base: usize, kind: AllocKind) {
        let index = self.allocation_at[base];
        assert_ne!(index, NONE);
        self.allocations[index].kind = kind;
    }

    fn seal(&mut self) {
        assert!(!self.sealed);
        // A fixed-size observer buffer is allocated before traversal. Recording
        // a free only writes an index and advances observed_len.
        self.observed_frees = vec![NONE; self.allocations.len()];
        self.sealed = true;
    }

    fn read(&self, at: usize) -> u64 {
        assert!(at < self.words.len(), "out-of-range heap read at {at}");
        let value = self.words[at];
        assert_ne!(value, UNINIT, "read of vacant/uninitialized word at {at}");
        assert_ne!(value, FREED, "read after free at {at}");
        value
    }

    fn write(&mut self, at: usize, value: u64) {
        assert!(at < self.words.len(), "out-of-range heap write at {at}");
        assert_ne!(self.words[at], FREED, "write after free at {at}");
        self.words[at] = value;
    }

    fn free(&mut self, base: usize, expected_kind: AllocKind) {
        assert!(self.sealed, "observer used before fixture was sealed");
        assert!(base < self.allocation_at.len(), "invalid free address {base}");
        let index = self.allocation_at[base];
        assert_ne!(index, NONE, "free of non-allocation base {base}");
        let allocation = &mut self.allocations[index];
        assert_eq!(allocation.base, base);
        assert_eq!(allocation.kind, expected_kind, "wrong allocation kind at {base}");
        assert!(!allocation.freed, "double free at {base}");
        assert!(self.observed_len < self.observed_frees.len());
        allocation.freed = true;
        self.observed_frees[self.observed_len] = base;
        self.observed_len += 1;
        for word in &mut self.words[base..base + allocation.words] {
            *word = FREED;
        }
    }

    fn observed(&self) -> &[usize] {
        &self.observed_frees[..self.observed_len]
    }

    fn assert_freed_exactly_except_root_e(&self) {
        for allocation in &self.allocations {
            match allocation.kind {
                AllocKind::RootE => assert!(!allocation.freed, "root E was freed"),
                _ => assert!(
                    allocation.freed,
                    "allocation {:?} at {} was leaked",
                    allocation.kind,
                    allocation.base
                ),
            }
        }
    }
}

#[derive(Clone)]
enum Shape {
    Leaf,
    ViaS(Box<PairShape>),
    ViaT(Box<PairShape>),
    ViaRing(Box<RingShape>),
}

#[derive(Clone)]
struct PairShape {
    first: Box<Shape>,
    second: Box<Shape>,
}

#[derive(Clone)]
struct RingShape {
    cap: usize,
    head: usize,
    live: Vec<Shape>,
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Leaf => f.write_str("Leaf"),
            Shape::ViaS(_) => f.write_str("ViaS(..)"),
            Shape::ViaT(_) => f.write_str("ViaT(..)"),
            Shape::ViaRing(r) => f
                .debug_struct("ViaRing")
                .field("cap", &r.cap)
                .field("head", &r.head)
                .field("len", &r.live.len())
                .finish(),
        }
    }
}

fn pair(first: Shape, second: Shape) -> PairShape {
    PairShape {
        first: Box::new(first),
        second: Box::new(second),
    }
}

fn build_owned_e(heap: &mut Heap, shape: &Shape, kind: AllocKind) -> usize {
    let base = heap.alloc(E_WORDS, kind);
    build_inline_e(heap, base, shape);
    base
}

fn build_inline_e(heap: &mut Heap, base: usize, shape: &Shape) {
    assert!(base + E_WORDS <= heap.words.len());
    match shape {
        Shape::Leaf => heap.write(base, TAG_LEAF),
        Shape::ViaS(pair) => {
            let pair_base = build_pair(heap, pair, AllocKind::S);
            heap.write(base, TAG_S);
            heap.write(base + S_SLOT, pair_base as u64);
        }
        Shape::ViaT(pair) => {
            let pair_base = build_pair(heap, pair, AllocKind::T);
            heap.write(base, TAG_T);
            heap.write(base + T_SLOT, pair_base as u64);
        }
        Shape::ViaRing(ring) => {
            let ring_base = build_ring(heap, ring);
            heap.write(base, TAG_RING);
            heap.write(base + RING_SLOT, ring_base as u64);
        }
    }
}

fn build_pair(heap: &mut Heap, shape: &PairShape, kind: AllocKind) -> usize {
    let base = heap.alloc(2, kind);
    let first = build_owned_e(heap, &shape.first, AllocKind::BoxE);
    let second = build_owned_e(heap, &shape.second, AllocKind::BoxE);
    heap.write(base, first as u64);
    heap.write(base + 1, second as u64);
    base
}

fn build_ring(heap: &mut Heap, shape: &RingShape) -> usize {
    assert!(shape.live.len() <= shape.cap);
    assert!(shape.cap == 0 && shape.head == 0 || shape.head < shape.cap);
    let base = heap.alloc(3 + shape.cap * E_WORDS, AllocKind::Ring);
    heap.write(base, shape.live.len() as u64);
    heap.write(base + 1, shape.cap as u64);
    heap.write(base + 2, shape.head as u64);
    for (logical, item) in shape.live.iter().enumerate() {
        let physical = if shape.cap == 0 {
            unreachable!()
        } else {
            (shape.head + logical) % shape.cap
        };
        build_inline_e(heap, base + 3 + physical * E_WORDS, item);
    }
    base
}

fn oracle_e(heap: &Heap, base: usize, frees: &mut Vec<usize>) {
    match heap.read(base) {
        TAG_LEAF => {}
        TAG_S => {
            let pair_base = heap.read(base + S_SLOT) as usize;
            oracle_pair(heap, pair_base, frees);
            frees.push(pair_base);
        }
        TAG_T => {
            let pair_base = heap.read(base + T_SLOT) as usize;
            oracle_pair(heap, pair_base, frees);
            frees.push(pair_base);
        }
        TAG_RING => {
            let ring_base = heap.read(base + RING_SLOT) as usize;
            oracle_ring(heap, ring_base, frees);
        }
        tag => panic!("oracle saw invalid E tag {tag} at {base}"),
    }
}

fn oracle_pair(heap: &Heap, base: usize, frees: &mut Vec<usize>) {
    let first = heap.read(base) as usize;
    oracle_e(heap, first, frees);
    frees.push(first);
    let second = heap.read(base + 1) as usize;
    oracle_e(heap, second, frees);
    frees.push(second);
}

fn oracle_ring(heap: &Heap, base: usize, frees: &mut Vec<usize>) {
    let len = heap.read(base) as usize;
    let cap = heap.read(base + 1) as usize;
    let head = heap.read(base + 2) as usize;
    assert!(len <= cap);
    assert!(cap == 0 && head == 0 || head < cap);
    for logical in 0..len {
        let physical = (head + logical) % cap;
        oracle_e(heap, base + 3 + physical * E_WORDS, frees);
    }
    frees.push(base);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
enum Code {
    RootE = 10,
    SFirst = 11,
    SSecond = 12,
    TFirst = 13,
    TSecond = 14,
    EAfterS = 15,
    EAfterT = 16,
    EAfterRing = 17,
    RingNext = 18,
}

impl Code {
    fn from_word(word: u64) -> Self {
        match word {
            10 => Code::RootE,
            11 => Code::SFirst,
            12 => Code::SSecond,
            13 => Code::TFirst,
            14 => Code::TSecond,
            15 => Code::EAfterS,
            16 => Code::EAfterT,
            17 => Code::EAfterRing,
            18 => Code::RingNext,
            _ => panic!("invalid continuation token {word}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Pc {
    EnterE {
        base: usize,
        caller_top: usize,
        caller_code: Code,
    },
    EnterPair {
        base: usize,
        e_top: usize,
        kind: AllocKind,
    },
    EnterRing {
        base: usize,
        outer_top: usize,
    },
    Continue {
        top: usize,
        code: Code,
        completed: usize,
    },
    Done,
}

fn finish_e(heap: &mut Heap, base: usize, link_offset: usize) -> Pc {
    // Pop both saved registers before the caller can free this E box.
    let outer_top = heap.read(base + link_offset) as usize;
    let outer_code = Code::from_word(heap.read(base));
    Pc::Continue {
        top: outer_top,
        code: outer_code,
        completed: base,
    }
}

fn finish_ring(heap: &mut Heap, base: usize) -> Pc {
    // The backlink must be loaded before free poisons the ring backing.
    let outer_top = heap.read(base) as usize;
    heap.free(base, AllocKind::Ring);
    if outer_top == SENTINEL {
        Pc::Done
    } else {
        Pc::Continue {
            top: outer_top,
            code: Code::EAfterRing,
            completed: base,
        }
    }
}

fn traverse_root_e(heap: &mut Heap, root: usize) {
    run(
        heap,
        Pc::EnterE {
            base: root,
            caller_top: SENTINEL,
            caller_code: Code::RootE,
        },
    );
}

fn traverse_root_ring(heap: &mut Heap, root: usize) {
    run(
        heap,
        Pc::EnterRing {
            base: root,
            outer_top: SENTINEL,
        },
    );
}

fn run(heap: &mut Heap, mut pc: Pc) {
    loop {
        pc = match pc {
            Pc::Done => return,
            Pc::EnterE {
                base,
                caller_top,
                caller_code,
            } => {
                let tag = heap.read(base);
                let link_offset = match tag {
                    TAG_LEAF => LEAF_LINK_SLOT,
                    TAG_S => S_SLOT,
                    TAG_T => T_SLOT,
                    TAG_RING => RING_SLOT,
                    _ => panic!("invalid E tag {tag} at {base}"),
                };
                let payload = if tag == TAG_LEAF {
                    0
                } else {
                    heap.read(base + link_offset) as usize
                };

                // This is the only active-anchor storage: tag/padding holds the
                // caller token and exactly one selected payload word holds the
                // caller's predecessor-slot address.
                heap.write(base, caller_code as u64);
                heap.write(base + link_offset, caller_top as u64);

                match tag {
                    TAG_LEAF => finish_e(heap, base, link_offset),
                    TAG_S => Pc::EnterPair {
                        base: payload,
                        e_top: base + link_offset,
                        kind: AllocKind::S,
                    },
                    TAG_T => Pc::EnterPair {
                        base: payload,
                        e_top: base + link_offset,
                        kind: AllocKind::T,
                    },
                    TAG_RING => Pc::EnterRing {
                        base: payload,
                        outer_top: base + link_offset,
                    },
                    _ => unreachable!(),
                }
            }
            Pc::EnterPair { base, e_top, kind } => {
                let first = heap.read(base) as usize;
                // S/T has no tag. Its consumed first slot links directly to the
                // enclosing E anchor; the finite continuation code identifies
                // both the parent type and this field's offset.
                heap.write(base, e_top as u64);
                let code = match kind {
                    AllocKind::S => Code::SFirst,
                    AllocKind::T => Code::TFirst,
                    _ => unreachable!(),
                };
                Pc::EnterE {
                    base: first,
                    caller_top: base,
                    caller_code: code,
                }
            }
            Pc::EnterRing { base, outer_top } => {
                let len = heap.read(base) as usize;
                let cap = heap.read(base + 1) as usize;
                let head = heap.read(base + 2) as usize;
                assert!(len <= cap, "ring len exceeds cap");
                assert!(cap == 0 && head == 0 || head < cap, "invalid ring head");
                let end = base + 3 + cap * E_WORDS;

                // Once the first cursor and end are computed, the original
                // len/cap/head are dead. Header = {outer predecessor, remaining,
                // one-past-capacity}; the completed E address is the cursor.
                heap.write(base, outer_top as u64);
                heap.write(base + 1, len as u64);
                heap.write(base + 2, end as u64);

                if len == 0 {
                    finish_ring(heap, base)
                } else {
                    let first = base + 3 + head * E_WORDS;
                    Pc::EnterE {
                        base: first,
                        caller_top: base,
                        caller_code: Code::RingNext,
                    }
                }
            }
            Pc::Continue {
                top,
                code,
                completed,
            } => match code {
                Code::RootE => {
                    assert_eq!(top, SENTINEL);
                    Pc::Done
                }
                Code::SFirst | Code::TFirst => {
                    let pair_base = top;
                    // Load the reversed predecessor before the observer poisons
                    // the completed child allocation.
                    let e_top = heap.read(pair_base) as usize;
                    heap.free(completed, AllocKind::BoxE);
                    let second = heap.read(pair_base + 1) as usize;
                    heap.write(pair_base + 1, e_top as u64);
                    let second_code = if code == Code::SFirst {
                        Code::SSecond
                    } else {
                        Code::TSecond
                    };
                    Pc::EnterE {
                        base: second,
                        caller_top: pair_base + 1,
                        caller_code: second_code,
                    }
                }
                Code::SSecond | Code::TSecond => {
                    // top is the second field, so subtracting one recovers the
                    // exact S/T base. The link is loaded before freeing child E.
                    let pair_base = top - 1;
                    let e_top = heap.read(top) as usize;
                    heap.free(completed, AllocKind::BoxE);
                    Pc::Continue {
                        top: e_top,
                        code: if code == Code::SSecond {
                            Code::EAfterS
                        } else {
                            Code::EAfterT
                        },
                        completed: pair_base,
                    }
                }
                Code::EAfterS | Code::EAfterT => {
                    let link_offset = if code == Code::EAfterS { S_SLOT } else { T_SLOT };
                    let e_base = top - link_offset;
                    // Pop the anchor completely before freeing its contained
                    // Box<S/T>; this also tolerates an observer that poisons it.
                    let outer_top = heap.read(top) as usize;
                    let outer_code = Code::from_word(heap.read(e_base));
                    heap.free(
                        completed,
                        if code == Code::EAfterS {
                            AllocKind::S
                        } else {
                            AllocKind::T
                        },
                    );
                    Pc::Continue {
                        top: outer_top,
                        code: outer_code,
                        completed: e_base,
                    }
                }
                Code::EAfterRing => {
                    let e_base = top - RING_SLOT;
                    // Ring backing was already freed by finish_ring; pop the E
                    // anchor to report the now-complete E to its own caller.
                    let outer_top = heap.read(top) as usize;
                    let outer_code = Code::from_word(heap.read(e_base));
                    Pc::Continue {
                        top: outer_top,
                        code: outer_code,
                        completed: e_base,
                    }
                }
                Code::RingNext => {
                    let ring_base = top;
                    let remaining = heap.read(ring_base + 1) as usize;
                    assert!(remaining > 0);
                    let remaining = remaining - 1;
                    if remaining == 0 {
                        finish_ring(heap, ring_base)
                    } else {
                        heap.write(ring_base + 1, remaining as u64);
                        let end = heap.read(ring_base + 2) as usize;
                        let mut next = completed + E_WORDS;
                        if next == end {
                            next = ring_base + 3;
                        }
                        assert!(next < end, "ring cursor escaped backing");
                        Pc::EnterE {
                            base: next,
                            caller_top: ring_base,
                            caller_code: Code::RingNext,
                        }
                    }
                }
            },
        };
    }
}

fn check_root_e(name: &str, shape: Shape) {
    let mut heap = Heap::new(200_000, 20_000);
    let root = build_owned_e(&mut heap, &shape, AllocKind::RootE);
    let mut expected = Vec::with_capacity(heap.allocations.len());
    oracle_e(&heap, root, &mut expected);
    heap.seal();
    traverse_root_e(&mut heap, root);
    assert_eq!(heap.observed(), expected, "free-order mismatch in {name}");
    heap.assert_freed_exactly_except_root_e();
    println!("ok {name}: {} frees", expected.len());
}

fn check_root_ring(name: &str, shape: RingShape) {
    let mut heap = Heap::new(200_000, 20_000);
    let root = build_ring(&mut heap, &shape);
    let mut expected = Vec::with_capacity(heap.allocations.len());
    oracle_ring(&heap, root, &mut expected);
    heap.seal();
    traverse_root_ring(&mut heap, root);
    assert_eq!(heap.observed(), expected, "free-order mismatch in {name}");
    heap.assert_freed_exactly_except_root_e();
    println!("ok {name}: {} frees", expected.len());
}

fn generated_shape(seed: u64, depth: usize) -> Shape {
    fn step(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state
    }
    fn go(state: &mut u64, depth: usize) -> Shape {
        if depth == 0 {
            return Shape::Leaf;
        }
        match step(state) % 4 {
            0 => Shape::Leaf,
            1 => Shape::ViaS(Box::new(pair(go(state, depth - 1), go(state, depth - 1)))),
            2 => Shape::ViaT(Box::new(pair(go(state, depth - 1), go(state, depth - 1)))),
            _ => {
                let cap = 1 + (step(state) as usize % 4);
                let len = step(state) as usize % (cap + 1);
                let head = step(state) as usize % cap;
                let mut live = Vec::with_capacity(len);
                for _ in 0..len {
                    live.push(go(state, depth - 1));
                }
                Shape::ViaRing(Box::new(RingShape { cap, head, live }))
            }
        }
    }
    let mut state = seed;
    go(&mut state, depth)
}

fn check_deep_chain(depth: usize) {
    #[derive(Clone, Copy)]
    struct Record {
        first: usize,
        side: usize,
        pair: usize,
        outer: usize,
    }

    let word_capacity = 16 + depth * 10;
    let allocation_capacity = 8 + depth * 3;
    let mut heap = Heap::new(word_capacity, allocation_capacity);
    let bottom = heap.alloc(E_WORDS, AllocKind::BoxE);
    heap.write(bottom, TAG_LEAF);
    let mut current = bottom;
    let mut records = Vec::with_capacity(depth);

    // Entire graph construction is iterative and finishes before traversal.
    for level in 0..depth {
        let kind = if level % 2 == 0 {
            AllocKind::S
        } else {
            AllocKind::T
        };
        let side = heap.alloc(E_WORDS, AllocKind::BoxE);
        heap.write(side, TAG_LEAF);
        let pair_base = heap.alloc(2, kind);
        heap.write(pair_base, current as u64);
        heap.write(pair_base + 1, side as u64);
        let outer = heap.alloc(E_WORDS, AllocKind::BoxE);
        if kind == AllocKind::S {
            heap.write(outer, TAG_S);
            heap.write(outer + S_SLOT, pair_base as u64);
        } else {
            heap.write(outer, TAG_T);
            heap.write(outer + T_SLOT, pair_base as u64);
        }
        records.push(Record {
            first: current,
            side,
            pair: pair_base,
            outer,
        });
        current = outer;
    }
    heap.set_kind(current, AllocKind::RootE);

    // This exact-order oracle is constructed independently from the fixture's
    // ownership records. The ordinary fixtures above use the recursive oracle;
    // this avoids making the host call stack part of the depth experiment.
    let mut expected = Vec::with_capacity(depth * 3);
    for record in &records {
        expected.push(record.first);
        expected.push(record.side);
        expected.push(record.pair);
    }
    assert_eq!(records.last().map(|r| r.outer), Some(current));
    drop(records);

    heap.seal();
    traverse_root_e(&mut heap, current);
    assert_eq!(heap.observed(), expected, "deep-chain free-order mismatch");
    heap.assert_freed_exactly_except_root_e();
    println!("ok deep alternating chain depth {depth}: {} frees", expected.len());
}

fn main() {
    check_root_e("root leaf", Shape::Leaf);

    let alternating = Shape::ViaS(Box::new(pair(
        Shape::ViaT(Box::new(pair(
            Shape::ViaS(Box::new(pair(Shape::Leaf, Shape::Leaf))),
            Shape::Leaf,
        ))),
        Shape::ViaT(Box::new(pair(Shape::Leaf, Shape::Leaf))),
    )));
    check_root_e("alternating S/T", alternating);

    check_root_ring(
        "empty ring cap=0",
        RingShape {
            cap: 0,
            head: 0,
            live: vec![],
        },
    );
    check_root_ring(
        "empty ring cap>0",
        RingShape {
            cap: 5,
            head: 3,
            live: vec![],
        },
    );
    check_root_ring(
        "fully occupied wrapped ring",
        RingShape {
            cap: 4,
            head: 2,
            live: vec![
                Shape::Leaf,
                Shape::ViaS(Box::new(pair(Shape::Leaf, Shape::Leaf))),
                Shape::ViaT(Box::new(pair(Shape::Leaf, Shape::Leaf))),
                Shape::Leaf,
            ],
        },
    );
    check_root_ring(
        "partially occupied wrapped ring",
        RingShape {
            cap: 5,
            head: 4,
            live: vec![
                Shape::ViaT(Box::new(pair(Shape::Leaf, Shape::Leaf))),
                Shape::Leaf,
                Shape::ViaS(Box::new(pair(Shape::Leaf, Shape::Leaf))),
            ],
        },
    );

    let nested_rings = RingShape {
        cap: 3,
        head: 2,
        live: vec![
            Shape::ViaRing(Box::new(RingShape {
                cap: 4,
                head: 3,
                live: vec![
                    Shape::Leaf,
                    Shape::ViaS(Box::new(pair(Shape::Leaf, Shape::Leaf))),
                ],
            })),
            Shape::ViaT(Box::new(pair(
                Shape::ViaRing(Box::new(RingShape {
                    cap: 2,
                    head: 1,
                    live: vec![Shape::Leaf, Shape::Leaf],
                })),
                Shape::Leaf,
            ))),
        ],
    };
    check_root_ring("nested rings", nested_rings);

    for seed in [
        0x0000_0000_0000_0000,
        0x8f3c_19a2_7721_4b06,
        0xffff_ffff_ffff_ffc7,
        0x6a09_e667_f3bc_c908,
    ] {
        check_root_e("fixed-seed generated shape", generated_shape(seed, 6));
    }

    check_deep_chain(100_000);
    println!("all cleanup-model assertions passed");
}
