//! Abstract continuation-space comparison, not a Whitefoot memory-layout model.
//! Child lists stand for an already-correct, ordered typed field iterator.
//! All model and observer allocation finishes before either traversal starts.
#![forbid(unsafe_code)]

const NONE: usize = usize::MAX;

#[derive(Clone)]
struct Tree {
    children: Vec<Vec<usize>>,
    root: usize,
}

#[derive(Clone, Copy)]
struct Header {
    parent: usize,
    next: usize,
}

struct Observer {
    live: Vec<bool>,
    trace: Vec<usize>,
    used: usize,
    slot_reads: u64,
}

impl Observer {
    fn new(count: usize) -> Self {
        Self {
            live: vec![true; count],
            trace: vec![NONE; count],
            used: 0,
            slot_reads: 0,
        }
    }

    fn read(&mut self, tree: &Tree, node: usize, slot: usize) -> usize {
        assert!(self.live[node], "read after release");
        self.slot_reads += 1;
        tree.children[node][slot]
    }

    fn free(&mut self, node: usize) {
        assert!(self.live[node], "duplicate release");
        self.live[node] = false;
        self.trace[self.used] = node;
        self.used += 1;
    }

    fn check(&self, expected: &[usize]) {
        assert_eq!(self.used, self.trace.len(), "missing release");
        assert!(self.live.iter().all(|live| !live));
        assert_eq!(&self.trace[..self.used], expected, "release order");
    }
}

// No continuation fields. Consumed owning-pointer slots become NONE; remaining
// fields retain their identities. Restarting at the root reconstructs the path.
// The caller allocates the tree and observer before this function is entered.
fn restart_from_root(tree: &mut Tree, seen: &mut Observer) {
    loop {
        let mut current = tree.root;
        let mut parent = NONE;
        let mut parent_slot = 0;
        loop {
            let mut next = None;
            for slot in 0..tree.children[current].len() {
                let child = seen.read(tree, current, slot);
                if child != NONE {
                    next = Some((slot, child));
                    break;
                }
            }
            match next {
                Some((slot, child)) => {
                    parent = current;
                    parent_slot = slot;
                    current = child;
                }
                None => {
                    if parent != NONE {
                        tree.children[parent][parent_slot] = NONE;
                    }
                    seen.free(current);
                    if parent == NONE {
                        return;
                    }
                    break;
                }
            }
        }
    }
}

// Two pre-existing control words per abstract node. They model persistent
// storage, not a worklist allocated on entry to cleanup. A concrete WF encoding
// also needs typed dispatch and field/window iteration, which this model omits.
fn parent_and_cursor(tree: &Tree, headers: &mut [Header], seen: &mut Observer) {
    let mut current = tree.root;
    headers[current] = Header {
        parent: NONE,
        next: 0,
    };
    loop {
        assert!(seen.live[current]);
        let slot = headers[current].next;
        if slot < tree.children[current].len() {
            let child = seen.read(tree, current, slot);
            assert_ne!(child, NONE);
            headers[current].next += 1;
            headers[child] = Header {
                parent: current,
                next: 0,
            };
            current = child;
        } else {
            let parent = headers[current].parent;
            seen.free(current);
            if parent == NONE {
                return;
            }
            current = parent;
        }
    }
}

// An independent ordinary recursive specification, used only for shallow
// branching fixtures before the traversal. Deep chain expectations are closed
// sequences, so the oracle itself does not require a deep native call stack.
fn oracle(tree: &Tree, node: usize, trace: &mut Vec<usize>) {
    for child in &tree.children[node] {
        oracle(tree, *child, trace);
    }
    trace.push(node);
}

fn chain(edges: usize) -> Tree {
    let mut children = Vec::with_capacity(edges + 1);
    children.push(vec![]);
    for index in 1..=edges {
        children.push(vec![index - 1]);
    }
    Tree {
        children,
        root: edges,
    }
}

fn wide(edges: usize) -> Tree {
    let mut children = vec![vec![]; edges + 1];
    children[edges] = (0..edges).collect();
    Tree {
        children,
        root: edges,
    }
}

fn branching(depth: usize, seed: &mut u64, children: &mut Vec<Vec<usize>>) -> usize {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let width = if depth == 0 {
        0
    } else {
        1 + ((*seed >> 32) as usize % 4)
    };
    let mut fields = Vec::with_capacity(width);
    for _ in 0..width {
        fields.push(branching(depth - 1, seed, children));
    }
    // Allocation identity deliberately differs from field order.
    if *seed & 1 != 0 {
        fields.reverse();
    }
    let root = children.len();
    children.push(fields);
    root
}

fn compare(name: &str, tree: Tree, expected: Vec<usize>, run_restart: bool) -> (u64, u64) {
    let count = tree.children.len();
    let mut headers = vec![
        Header {
            parent: NONE,
            next: 0
        };
        count
    ];
    let mut header_seen = Observer::new(count);
    parent_and_cursor(&tree, &mut headers, &mut header_seen);
    header_seen.check(&expected);
    assert_eq!(header_seen.slot_reads, (count - 1) as u64);

    let restart_reads = if run_restart {
        let mut restart_tree = tree.clone();
        let mut restart_seen = Observer::new(count);
        restart_from_root(&mut restart_tree, &mut restart_seen);
        restart_seen.check(&expected);
        Some(restart_seen.slot_reads)
    } else {
        None
    };
    println!(
        "{name}\t{count}\t{}\t{}\t{}",
        restart_reads.map_or_else(|| "not-run".to_owned(), |value| value.to_string()),
        header_seen.slot_reads,
        count * std::mem::size_of::<Header>(),
    );
    (restart_reads.unwrap_or(0), header_seen.slot_reads)
}

fn main() {
    println!("case\tnodes\trestart-slot-reads\theader-slot-reads\theader-bytes");
    compare("empty", chain(0), vec![0], true);
    for edges in [1000, 2000, 4000] {
        let expected = (0..=edges).collect::<Vec<_>>();
        let (reads, _) = compare(
            &format!("chain-{edges}"),
            chain(edges),
            expected.clone(),
            true,
        );
        // For n edges: n(n+1)/2 live reads plus n consumed-slot reads.
        assert_eq!(reads, (edges * (edges + 3) / 2) as u64);
        let (reads, _) = compare(&format!("wide-{edges}"), wide(edges), expected, true);
        assert_eq!(reads, (edges * (edges + 3) / 2) as u64);
    }
    for initial_seed in [1, 7, 41, 127] {
        let mut children = Vec::new();
        let mut seed = initial_seed;
        let root = branching(7, &mut seed, &mut children);
        let tree = Tree { children, root };
        let mut expected = Vec::new();
        oracle(&tree, root, &mut expected);
        compare(&format!("branch-{initial_seed}"), tree, expected, true);
    }
    compare("deep-header", chain(100000), (0..=100000).collect(), false);
    println!("all continuation-controller assertions passed");
}
