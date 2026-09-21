use std::fmt;

const NONE: usize = usize::MAX;
const SENTINEL: usize = 0;
const UNINIT: u64 = 0xd1d1_d1d1_d1d1_d1d1;
const FREED: u64 = 0xf3f3_f3f3_f3f3_f3f3;
const S_WORDS: usize = 2;
const RING_HEADER_WORDS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    RootS,
    Ring,
}

#[derive(Debug)]
struct Allocation {
    base: usize,
    words: usize,
    kind: Kind,
    freed: bool,
}

struct Heap {
    words: Vec<u64>,
    next: usize,
    allocations: Vec<Allocation>,
    allocation_at: Vec<usize>,
    frees: Vec<usize>,
    free_len: usize,
    sealed: bool,
}

impl Heap {
    fn new(word_capacity: usize, allocation_capacity: usize) -> Self {
        let mut words = vec![UNINIT; word_capacity];
        words[SENTINEL] = 0;
        Self {
            words,
            next: 8,
            allocations: Vec::with_capacity(allocation_capacity),
            allocation_at: vec![NONE; word_capacity],
            frees: Vec::new(),
            free_len: 0,
            sealed: false,
        }
    }

    fn alloc(&mut self, words: usize, kind: Kind) -> usize {
        assert!(!self.sealed);
        assert!(self.next + words <= self.words.len());
        let base = self.next;
        self.next += words;
        let index = self.allocations.len();
        self.allocation_at[base] = index;
        self.allocations.push(Allocation {
            base,
            words,
            kind,
            freed: false,
        });
        base
    }

    fn read(&self, at: usize) -> u64 {
        assert!(at < self.words.len(), "out-of-range read {at}");
        let value = self.words[at];
        assert_ne!(value, UNINIT, "read of vacant slot {at}");
        assert_ne!(value, FREED, "read after free {at}");
        value
    }

    fn write(&mut self, at: usize, value: u64) {
        assert!(at < self.words.len(), "out-of-range write {at}");
        assert_ne!(self.words[at], FREED, "write after free {at}");
        self.words[at] = value;
    }

    fn seal(&mut self) {
        assert!(!self.sealed);
        self.frees = vec![NONE; self.allocations.len()];
        self.sealed = true;
    }

    fn free_ring(&mut self, base: usize) {
        assert!(self.sealed);
        let index = self.allocation_at[base];
        assert_ne!(index, NONE, "free of non-allocation {base}");
        let allocation = &mut self.allocations[index];
        assert_eq!(allocation.kind, Kind::Ring);
        assert!(!allocation.freed, "double free {base}");
        allocation.freed = true;
        assert!(self.free_len < self.frees.len());
        self.frees[self.free_len] = base;
        self.free_len += 1;
        for word in &mut self.words[base..base + allocation.words] {
            *word = FREED;
        }
    }

    fn observed(&self) -> &[usize] {
        &self.frees[..self.free_len]
    }

    fn assert_all_rings_freed(&self) {
        for allocation in &self.allocations {
            match allocation.kind {
                Kind::RootS => assert!(!allocation.freed),
                Kind::Ring => assert!(allocation.freed, "leaked ring at {}", allocation.base),
            }
        }
    }
}

#[derive(Clone)]
struct SShape {
    left: RingShape,
    right: RingShape,
}

#[derive(Clone)]
struct RingShape {
    cap: usize,
    head: usize,
    live: Vec<SShape>,
}

fn empty_ring() -> RingShape {
    RingShape {
        cap: 0,
        head: 0,
        live: vec![],
    }
}

fn build_s(heap: &mut Heap, base: usize, shape: &SShape) {
    let left = build_ring(heap, &shape.left);
    let right = build_ring(heap, &shape.right);
    heap.write(base, left as u64);
    heap.write(base + 1, right as u64);
}

fn build_ring(heap: &mut Heap, shape: &RingShape) -> usize {
    assert!(shape.live.len() <= shape.cap);
    assert!(shape.cap == 0 && shape.head == 0 || shape.head < shape.cap);
    let base = heap.alloc(RING_HEADER_WORDS + shape.cap * S_WORDS, Kind::Ring);
    heap.write(base, shape.live.len() as u64);
    heap.write(base + 1, shape.cap as u64);
    heap.write(base + 2, shape.head as u64);
    for (logical, item) in shape.live.iter().enumerate() {
        let physical = (shape.head + logical) % shape.cap;
        build_s(heap, base + RING_HEADER_WORDS + physical * S_WORDS, item);
    }
    base
}

fn oracle_s(heap: &Heap, base: usize, frees: &mut Vec<usize>) {
    let left = heap.read(base) as usize;
    oracle_ring(heap, left, frees);
    let right = heap.read(base + 1) as usize;
    oracle_ring(heap, right, frees);
}

fn oracle_ring(heap: &Heap, base: usize, frees: &mut Vec<usize>) {
    let len = heap.read(base) as usize;
    let cap = heap.read(base + 1) as usize;
    let head = heap.read(base + 2) as usize;
    assert!(len <= cap);
    assert!(cap == 0 && head == 0 || head < cap);
    for logical in 0..len {
        let physical = (head + logical) % cap;
        oracle_s(
            heap,
            base + RING_HEADER_WORDS + physical * S_WORDS,
            frees,
        );
    }
    frees.push(base);
}

#[derive(Clone, Copy, Debug)]
enum Pc {
    EnterS {
        base: usize,
        outer_top: usize,
    },
    EnterRing {
        base: usize,
        caller_slot: usize,
    },
    RingNext {
        ring_base: usize,
        completed_s: usize,
    },
    RingDone {
        caller_slot: usize,
    },
    Done,
}

fn finish_ring(heap: &mut Heap, ring_base: usize) -> Pc {
    // Load the sole caller link before poisoning all three header words.
    let caller_slot = heap.read(ring_base) as usize;
    heap.free_ring(ring_base);
    Pc::RingDone { caller_slot }
}

fn traverse_root_s(heap: &mut Heap, root: usize) {
    let mut pc = Pc::EnterS {
        base: root,
        outer_top: SENTINEL,
    };
    loop {
        pc = match pc {
            Pc::Done => return,
            Pc::EnterS { base, outer_top } => {
                // Derived termination invariant for this grammar:
                // root outer_top=0 and root base>=8; an inline S starts at
                // ring_base+3+2*i while outer_top=ring_base. Therefore the
                // predecessor can never equal left_slot-1.
                assert_ne!(outer_top, base - 1);
                let left_ring = heap.read(base) as usize;
                heap.write(base, outer_top as u64);
                Pc::EnterRing {
                    base: left_ring,
                    caller_slot: base,
                }
            }
            Pc::EnterRing { base, caller_slot } => {
                let len = heap.read(base) as usize;
                let cap = heap.read(base + 1) as usize;
                let head = heap.read(base + 2) as usize;
                assert!(len <= cap);
                assert!(cap == 0 && head == 0 || head < cap);
                let end = base + RING_HEADER_WORDS + cap * S_WORDS;
                heap.write(base, caller_slot as u64);
                heap.write(base + 1, len as u64);
                heap.write(base + 2, end as u64);
                if len == 0 {
                    finish_ring(heap, base)
                } else {
                    Pc::EnterS {
                        base: base + RING_HEADER_WORDS + head * S_WORDS,
                        outer_top: base,
                    }
                }
            }
            Pc::RingNext {
                ring_base,
                completed_s,
            } => {
                let remaining = heap.read(ring_base + 1) as usize;
                assert!(remaining > 0);
                let remaining = remaining - 1;
                if remaining == 0 {
                    finish_ring(heap, ring_base)
                } else {
                    heap.write(ring_base + 1, remaining as u64);
                    let end = heap.read(ring_base + 2) as usize;
                    let mut next = completed_s + S_WORDS;
                    if next == end {
                        next = ring_base + RING_HEADER_WORDS;
                    }
                    assert!(next < end);
                    Pc::EnterS {
                        base: next,
                        outer_top: ring_base,
                    }
                }
            }
            Pc::RingDone { caller_slot } => {
                // The right-field frame is a two-link chain:
                // right -> left -> outer predecessor. The equality is a marker
                // derived above, not spare pointer bits or allocator metadata.
                if heap.read(caller_slot) as usize == caller_slot - 1 {
                    let s_base = caller_slot - 1;
                    let outer_top = heap.read(s_base) as usize;
                    if outer_top == SENTINEL {
                        Pc::Done
                    } else {
                        Pc::RingNext {
                            ring_base: outer_top,
                            completed_s: s_base,
                        }
                    }
                } else {
                    // A left ring completed. Preserve left's backlink, consume
                    // right, and make right point to the dead left slot.
                    let s_base = caller_slot;
                    let right_ring = heap.read(s_base + 1) as usize;
                    heap.write(s_base + 1, s_base as u64);
                    Pc::EnterRing {
                        base: right_ring,
                        caller_slot: s_base + 1,
                    }
                }
            }
        };
    }
}

fn check_shape(name: &str, shape: SShape) {
    let mut heap = Heap::new(500_000, 100_000);
    let root = heap.alloc(S_WORDS, Kind::RootS);
    build_s(&mut heap, root, &shape);
    let mut expected = Vec::with_capacity(heap.allocations.len());
    oracle_s(&heap, root, &mut expected);
    heap.seal();
    traverse_root_s(&mut heap, root);
    assert_eq!(heap.observed(), expected, "free order mismatch in {name}");
    heap.assert_all_rings_freed();
    println!("ok anchorless {name}: {} frees", expected.len());
}

fn allocate_empty_ring(heap: &mut Heap) -> usize {
    let base = heap.alloc(RING_HEADER_WORDS, Kind::Ring);
    heap.write(base, 0);
    heap.write(base + 1, 0);
    heap.write(base + 2, 0);
    base
}

fn check_deep_chain(depth: usize) {
    let mut heap = Heap::new(32 + depth * 8, 8 + depth * 2);
    let bottom_left = allocate_empty_ring(&mut heap);
    let bottom_right = allocate_empty_ring(&mut heap);
    let mut current_left = bottom_left;
    let mut current_right = bottom_right;
    let mut expected = Vec::with_capacity(2 + depth * 2);
    expected.push(bottom_left);
    expected.push(bottom_right);

    for _ in 0..depth {
        let outer_left = heap.alloc(RING_HEADER_WORDS + S_WORDS, Kind::Ring);
        heap.write(outer_left, 1);
        heap.write(outer_left + 1, 1);
        heap.write(outer_left + 2, 0);
        heap.write(outer_left + 3, current_left as u64);
        heap.write(outer_left + 4, current_right as u64);
        let outer_right = allocate_empty_ring(&mut heap);
        current_left = outer_left;
        current_right = outer_right;
        expected.push(outer_left);
        expected.push(outer_right);
    }

    let root = heap.alloc(S_WORDS, Kind::RootS);
    heap.write(root, current_left as u64);
    heap.write(root + 1, current_right as u64);
    heap.seal();
    traverse_root_s(&mut heap, root);
    assert_eq!(heap.observed(), expected, "deep free order mismatch");
    heap.assert_all_rings_freed();
    println!("ok anchorless deep chain depth {depth}: {} frees", expected.len());
}

impl fmt::Debug for SShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SShape(..)")
    }
}

fn main() {
    check_shape(
        "empty leaf",
        SShape {
            left: empty_ring(),
            right: empty_ring(),
        },
    );

    let nested = SShape {
        left: RingShape {
            cap: 4,
            head: 3,
            live: vec![
                SShape {
                    left: empty_ring(),
                    right: RingShape {
                        cap: 2,
                        head: 1,
                        live: vec![SShape {
                            left: empty_ring(),
                            right: empty_ring(),
                        }],
                    },
                },
                SShape {
                    left: RingShape {
                        cap: 1,
                        head: 0,
                        live: vec![SShape {
                            left: empty_ring(),
                            right: empty_ring(),
                        }],
                    },
                    right: empty_ring(),
                },
            ],
        },
        right: RingShape {
            cap: 3,
            head: 2,
            live: vec![SShape {
                left: empty_ring(),
                right: empty_ring(),
            }],
        },
    };
    check_shape("wrapped nested rings", nested);
    check_deep_chain(100_000);
    println!("all anchorless-model assertions passed");
}
