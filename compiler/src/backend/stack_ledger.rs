//! The stack ledger: what a program's frames cost, which of its functions can
//! reach themselves, and how many levels of each the runtime's stack holds.
//!
//! Non-normative developer output on a separately selected channel, exactly
//! like [`crate::semantic::permission_ledger`], and for the same reason: it
//! participates in no acceptance judgment, changes no lowering, and cannot
//! alter, prefix, suffix, or replace a mandatory record. It reports a fact the
//! writer otherwise has no way to see.
//!
//! **Why it is measured after codegen and nowhere earlier.** The compiler does
//! accumulate a frame size during target qualification, but that number counts
//! the slots the emitter asked for. Every frame the corpus actually dies on is
//! made of things that do not exist until the register allocator has run: the
//! ABI frame record a non-leaf function is forced to keep, and the
//! callee-saved registers the allocator chose to spill. Measured on the
//! deepest recursion in the corpus, a pre-codegen ledger reports zero bytes
//! for the function that ends the program. So the numbers here come from
//! `-fstack-usage` and the call graph from the assembly of that same
//! compilation — the post-inline graph, which is the graph the frame numbers
//! belong to.
//!
//! **What it is not.** It is a build fact, not a source fact: it is per target,
//! per optimization level, and per host compiler version. It covers the
//! emitted module's own machine functions, so the runtime translation units
//! linked beside it — the floor and, when a module hands work out, the
//! parallel runtime — are outside it. And an inlined callee has no row of its
//! own; its bytes are inside its caller's.

use std::collections::HashMap;
use std::fmt::Write;

use super::emitter::overlapped_clone_symbol;

/// One machine function's frame, as the host compiler reported it.
struct Frame {
    name: String,
    bytes: u64,
    /// `-fstack-usage`'s own classification: `static` for a frame fixed at
    /// compile time, `dynamic` or `bounded` for one that is not. Every frame
    /// this compiler has ever emitted is `static`, which is what makes the
    /// arithmetic below exact rather than an estimate; the qualifier is
    /// carried so that stops being an assumption.
    qualifier: String,
    /// Which of a `--par` build's two worlds this row belongs to, when the
    /// module carries both and this function is one of the pair.
    world: World,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum World {
    /// The one lowering, run in every schedule.
    Both,
    /// The lowering that reaches a hand-out, run when a pool was asked for.
    Overlapped,
    /// The clone the bootstrap selects when no pool was asked for.
    Sequential,
}

impl World {
    fn suffix(self) -> &'static str {
        match self {
            Self::Both => "",
            Self::Overlapped => "  overlapped clone",
            Self::Sequential => "  sequential clone",
        }
    }
}

/// The ledger's lines for one compiled module, given the host compiler's
/// stack-usage report, the assembly of the same compilation, and the number of
/// bytes of stack the runtime gives every thread.
///
/// Both inputs come from one clang invocation on purpose. Reading frames from
/// one compilation and the call graph from another would let an unrelated flag
/// put a function in one and not the other, and the ledger would report a
/// chain through a function whose bytes it took from somewhere else.
pub fn stack_ledger(stack_usage: &str, assembly: &str, stack_bytes: u64) -> Vec<String> {
    let frames = parse_frames(stack_usage);
    let index: HashMap<&str, usize> = frames
        .iter()
        .enumerate()
        .map(|(position, frame)| (frame.name.as_str(), position))
        .collect();
    let edges = parse_call_graph(assembly, &index);
    let components = components(&edges);

    let mut lines = Vec::new();
    lines.push(format!(
        "STACK stack     {stack_bytes} B  the entry thread and every worker lane"
    ));
    for frame in &frames {
        lines.push(format!(
            "STACK frame     {:<40}  {:>9} B  {}{}",
            frame.name,
            frame.bytes,
            frame.qualifier,
            frame.world.suffix()
        ));
    }
    for component in &components {
        if component.len() == 1 && !edges[component[0]].contains(&component[0]) {
            continue;
        }
        // One turn of the cycle is the unit: a recursion through three
        // functions spends all three frames per level, so summing the members
        // is the per-level cost whether the cycle is a self-call or not.
        let per_level: u64 = component.iter().map(|node| frames[*node].bytes).sum();
        let mut members = component
            .iter()
            .map(|node| frames[*node].name.as_str())
            .collect::<Vec<_>>();
        members.sort_unstable();
        let world = component_world(component, &frames);
        let mut line = format!(
            "STACK cycle     {:<40}  {per_level:>9} B/level",
            members.join(" + ")
        );
        // A cycle that spends nothing per level cannot exhaust anything, and
        // dividing by it would be the ledger inventing a number.
        match stack_bytes.checked_div(per_level) {
            Some(levels) => {
                let _ = write!(line, "  {levels} levels");
            }
            None => line.push_str("  no stack per level"),
        }
        line.push_str(world.suffix());
        lines.push(line);
    }
    for (root, chain, bytes) in chains(&frames, &edges, &components) {
        lines.push(format!(
            "STACK chain     {:<40}  {bytes:>9} B  {}",
            frames[root].name,
            chain.join(" -> ")
        ));
    }
    lines
}

/// `-fstack-usage`'s report: one line per surviving machine function, as
/// `locator\tbytes\tqualifier`, where the locator ends in the function name.
fn parse_frames(stack_usage: &str) -> Vec<Frame> {
    let mut frames: Vec<Frame> = Vec::new();
    for line in stack_usage.lines() {
        let mut fields = line.split('\t');
        let (Some(locator), Some(bytes), Some(qualifier)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let name = locator.rsplit(':').next().unwrap_or(locator);
        let Ok(bytes) = bytes.trim().parse::<u64>() else {
            continue;
        };
        frames.push(Frame {
            name: name.to_owned(),
            bytes,
            qualifier: qualifier.trim().to_owned(),
            world: World::Both,
        });
    }
    assign_worlds(&mut frames);
    frames
}

/// Marks the two lowerings a `--par` module carries.
///
/// The pairing is derived from the module rather than guessed at: a
/// `wf__par_seq_` definition exists only where the emitter made a clone, and
/// its presence is what makes the function it was cloned from the overlapped
/// one. A function with no clone is the same code in both worlds and is left
/// unmarked, which is the honest answer for it.
fn assign_worlds(frames: &mut [Frame]) {
    let mut overlapped: Vec<usize> = Vec::new();
    let mut sequential: Vec<usize> = Vec::new();
    for position in 0..frames.len() {
        let Some(original) = overlapped_clone_symbol(&frames[position].name) else {
            continue;
        };
        sequential.push(position);
        if let Some(paired) = frames.iter().position(|frame| frame.name == original) {
            overlapped.push(paired);
        }
    }
    for position in sequential {
        frames[position].world = World::Sequential;
    }
    for position in overlapped {
        frames[position].world = World::Overlapped;
    }
}

/// The world a whole cycle belongs to, when every member agrees.
fn component_world(component: &[usize], frames: &[Frame]) -> World {
    let first = frames[component[0]].world;
    if component.iter().all(|node| frames[*node].world == first) {
        first
    } else {
        World::Both
    }
}

/// The post-inline call graph, read out of the assembly of the same
/// compilation the frames came from.
///
/// Almost every call is direct and can simply be read: [GRAM-3] has no
/// function type and conformances lower to no vtable, dictionary, or indirect
/// call, so a Whitefoot call site names its callee and this graph is exact
/// where a general one would have to guess. The single exception is the
/// parallel runtime's own protocol, which hands the pool a pointer to a thunk;
/// that edge is not in the assembly, which is why a `wf__par_thunk_` row heads
/// a chain of its own rather than appearing under whatever offered it.
///
/// A tail call is counted as an ordinary edge, which over-counts the caller's
/// frame by what a tail call has already released — the safe direction for a
/// stack figure.
fn parse_call_graph(assembly: &str, index: &HashMap<&str, usize>) -> Vec<Vec<usize>> {
    let mut edges = vec![Vec::new(); index.len()];
    let mut current: Option<usize> = None;
    for line in assembly.lines() {
        let code = strip_comment(line);
        if !code.starts_with([' ', '\t']) {
            // A label at column zero. A function's own label names it; every
            // other label belongs to whatever function is already open.
            if let Some(label) = code.trim_end().strip_suffix(':')
                && let Some(position) = resolve(label, index)
            {
                current = Some(position);
            }
            continue;
        }
        let Some(caller) = current else {
            continue;
        };
        let mut tokens = code.split_whitespace();
        let (Some(mnemonic), Some(operand)) = (tokens.next(), tokens.next()) else {
            continue;
        };
        if !matches!(
            mnemonic,
            "bl" | "b" | "call" | "callq" | "jmp" | "jmpq" | "bl.w" | "b.w"
        ) {
            continue;
        }
        let operand = operand.trim_end_matches(',');
        if let Some(callee) = resolve(operand, index)
            && !edges[caller].contains(&callee)
        {
            edges[caller].push(callee);
        }
    }
    edges
}

/// One assembly line with its trailing comment removed.
///
/// The marker is the assembler's, not the architecture's, and the targets this
/// compiler emits for do not agree on it: Darwin's arm64 assembler comments
/// with `;`, its x86-64 assembler with `##`, and the ELF assemblers with `//`
/// on arm64 and `#` on x86-64. Reading only the Darwin markers cost the whole
/// call graph on x86-64 Linux, where every function label is written
/// `wf_spine:                # @wf_spine`: the line does not end at the colon,
/// so no label resolved, no instruction had an open caller, and the ledger
/// reported a recursion as a chain with no cycle. Found by running the gate on
/// a Linux runner in batch 0090.
///
/// Truncating at `#` is safe on an arm64 ELF operand that spells an immediate
/// `#48`, because the only instructions read past the mnemonic here are calls
/// and branches, whose operand is a symbol.
fn strip_comment(line: &str) -> &str {
    line.find([';', '#'])
        .into_iter()
        .chain(line.find("//"))
        .min()
        .map_or(line, |at| &line[..at])
}

/// The function one assembler symbol names, if it is one of this module's.
///
/// The assembler spelling carries the target's own decoration, and a private
/// definition does not get the same decoration as an exported one — on this
/// host an exported `main` is `_main` while a private helper is `l_wf_trap`.
/// Trying the plain name and each decoration in turn keeps that a property of
/// the assembler rather than something the ledger has to be told per target,
/// and an operand that resolves to nothing is an external call, which this
/// module has no frame for anyway.
fn resolve(symbol: &str, index: &HashMap<&str, usize>) -> Option<usize> {
    for prefix in ["", "l_", "_", ".L"] {
        if let Some(rest) = symbol.strip_prefix(prefix)
            && let Some(position) = index.get(rest)
        {
            return Some(*position);
        }
    }
    None
}

/// Tarjan's strongly connected components, iteratively.
///
/// Iterative because this is the analysis that finds unbounded recursion, and
/// a recursive implementation of it would descend exactly the graph it is
/// looking for.
fn components(edges: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let count = edges.len();
    let mut order = vec![usize::MAX; count];
    let mut low = vec![0_usize; count];
    let mut on_stack = vec![false; count];
    let mut stack: Vec<usize> = Vec::new();
    let mut frames: Vec<(usize, usize)> = Vec::new();
    let mut next_order = 0_usize;
    let mut found: Vec<Vec<usize>> = Vec::new();
    for root in 0..count {
        if order[root] != usize::MAX {
            continue;
        }
        order[root] = next_order;
        low[root] = next_order;
        next_order += 1;
        stack.push(root);
        on_stack[root] = true;
        frames.push((root, 0));
        while let Some((node, cursor)) = frames.last_mut() {
            let node = *node;
            if let Some(target) = edges[node].get(*cursor).copied() {
                *cursor += 1;
                if order[target] == usize::MAX {
                    order[target] = next_order;
                    low[target] = next_order;
                    next_order += 1;
                    stack.push(target);
                    on_stack[target] = true;
                    frames.push((target, 0));
                } else if on_stack[target] {
                    low[node] = low[node].min(order[target]);
                }
                continue;
            }
            frames.pop();
            if let Some((parent, _)) = frames.last() {
                low[*parent] = low[*parent].min(low[node]);
            }
            if low[node] == order[node] {
                let mut component = Vec::new();
                while let Some(member) = stack.pop() {
                    on_stack[member] = false;
                    component.push(member);
                    if member == node {
                        break;
                    }
                }
                found.push(component);
            }
        }
    }
    found
}

/// The worst acyclic chain from each root, with every cycle's own per-level
/// cost excluded — a cycle has no worst case until something bounds its depth,
/// and the ledger reports that separately rather than folding a guess into
/// this number.
fn chains(
    frames: &[Frame],
    edges: &[Vec<usize>],
    components: &[Vec<usize>],
) -> Vec<(usize, Vec<String>, u64)> {
    let count = frames.len();
    let mut component_of = vec![usize::MAX; count];
    for (position, component) in components.iter().enumerate() {
        for node in component {
            component_of[*node] = position;
        }
    }
    // Tarjan emits a component only after every component it can reach, so
    // walking the components in that order visits callees before callers.
    let mut best = vec![0_u64; count];
    let mut next = vec![usize::MAX; count];
    for component in components {
        let cyclic = component.len() > 1 || edges[component[0]].contains(&component[0]);
        for node in component {
            let own = if cyclic { 0 } else { frames[*node].bytes };
            let mut deepest = 0_u64;
            let mut through = usize::MAX;
            for target in &edges[*node] {
                if component_of[*target] == component_of[*node] {
                    continue;
                }
                if best[*target] > deepest {
                    deepest = best[*target];
                    through = *target;
                }
            }
            best[*node] = own.saturating_add(deepest);
            next[*node] = through;
        }
    }
    let mut reached = vec![false; count];
    for (node, targets) in edges.iter().enumerate().take(count) {
        for target in targets {
            if *target != node {
                reached[*target] = true;
            }
        }
    }
    let mut rendered = Vec::new();
    for node in 0..count {
        // A function something else calls is reported inside that caller's
        // chain, and a root that spends nothing is a row with no reader.
        if reached[node] || best[node] == 0 {
            continue;
        }
        let mut chain = Vec::new();
        let mut cursor = node;
        while cursor != usize::MAX && chain.len() < count {
            chain.push(frames[cursor].name.clone());
            cursor = next[cursor];
        }
        rendered.push((node, chain, best[node]));
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::stack_ledger;

    /// One `--par` compilation of a two-million-deep recursion, as the host
    /// compiler reported it: a self-recursive function in each of the two
    /// worlds, a thunk, the entry, and the runtime fallbacks.
    const SPINE_USAGE: &str = "m.ll:wf__par_claim\t0\tstatic
m.ll:wf__par_thunk_0\t32\tstatic
m.ll:wf_spine\t48\tstatic
m.ll:wf__par_seq_spine\t16\tstatic
m.ll:wf__floor_run\t0\tstatic
m.ll:wf__main_body\t16\tstatic
m.ll:main\t0\tstatic
";

    const SPINE_ASSEMBLY: &str = "\t.globl\t_wf__par_claim
_wf__par_claim:                         ; @wf__par_claim
\tret
_wf__par_thunk_0:                       ; @wf__par_thunk_0
\tbl\t_wf_spine
\tret
_wf_spine:                              ; @wf_spine
\tbl\t_wf__par_claim
\tb.eq\tLBB2_3
\tbl\t_wf_spine
\tret
_wf__par_seq_spine:                     ; @wf__par_seq_spine
\tbl\t_wf__par_seq_spine
\tret
_wf__floor_run:                         ; @wf__floor_run
\tb\t_wf__main_body
_wf__main_body:                         ; @wf__main_body
\tbl\t_wf_spine
\tbl\t_wf__par_seq_spine
\tret
_main:                                  ; @main
\tb\t_wf__floor_run
";

    fn ledger() -> Vec<String> {
        stack_ledger(SPINE_USAGE, SPINE_ASSEMBLY, 1_073_741_824)
    }

    fn line(lines: &[String], needle: &str) -> String {
        lines
            .iter()
            .find(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("no ledger line mentions {needle}: {lines:#?}"))
            .clone()
    }

    /// A `--par` module holds two lowerings of the same recursion and the
    /// bootstrap picks between them at startup, so one number for "the depth
    /// this program reaches" would be wrong in one of the two worlds. The
    /// ledger reports both and says which is which — which is exactly the
    /// difference a default-build depth regression hid.
    #[test]
    fn the_two_lowerings_of_one_recursion_get_their_own_rows() {
        let lines = ledger();
        let overlapped = line(&lines, "STACK cycle     wf_spine ");
        assert!(overlapped.contains("48 B/level"), "{overlapped}");
        assert!(overlapped.contains("22369621 levels"), "{overlapped}");
        assert!(overlapped.contains("overlapped clone"), "{overlapped}");

        let sequential = line(&lines, "STACK cycle     wf__par_seq_spine");
        assert!(sequential.contains("16 B/level"), "{sequential}");
        assert!(sequential.contains("67108864 levels"), "{sequential}");
        assert!(sequential.contains("sequential clone"), "{sequential}");
    }

    /// Only a function that can reach itself gets a cycle row. A ledger that
    /// reported a per-level cost for a chain would be telling the writer a
    /// bounded call has an unbounded depth.
    #[test]
    fn a_function_that_cannot_reach_itself_has_no_cycle_row() {
        let lines = ledger();
        for name in ["wf__main_body", "main", "wf__par_thunk_0"] {
            assert!(
                !lines
                    .iter()
                    .any(|line| line.starts_with("STACK cycle") && line.contains(name)),
                "{name} is not recursive but got a cycle row: {lines:#?}"
            );
        }
    }

    /// The acyclic chain excludes the recursion's own cost, because a cycle
    /// has no worst case until something bounds its depth. Reporting the
    /// deepest bounded path and the per-level cost separately is what keeps
    /// the ledger from folding a guess into a number.
    #[test]
    fn the_chain_stops_at_the_recursion_rather_than_guessing_its_depth() {
        let lines = ledger();
        let chain = line(&lines, "STACK chain     main");
        assert!(
            chain.contains("main -> wf__floor_run -> wf__main_body"),
            "{chain}"
        );
        // main 0 + floor_run 0 + main_body 16, and nothing for the recursion.
        assert!(chain.contains("16 B"), "{chain}");
    }

    /// Every surviving machine function gets a row, including the ones no
    /// writer wrote. The invisible recursion this ledger exists to expose is
    /// exactly the kind that has no name in the source.
    #[test]
    fn every_reported_frame_gets_a_row() {
        let lines = ledger();
        for name in [
            "wf__par_claim",
            "wf__par_thunk_0",
            "wf_spine",
            "wf__par_seq_spine",
            "wf__floor_run",
            "wf__main_body",
            "main",
        ] {
            assert!(
                lines
                    .iter()
                    .any(|line| line.starts_with("STACK frame") && line.contains(name)),
                "{name} has no frame row: {lines:#?}"
            );
        }
    }
}
