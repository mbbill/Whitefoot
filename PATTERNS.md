# Whitefoot Pattern Doctrine (D6)

Status: seeded 2026-07-09; normative once ratified. The language forces a
closed pattern vocabulary at the architecture level, exactly as the kernel
forces one loop form and one conditional at the statement level. The catalog
must stay COMPLETE (every task modelable — a gap is a finding) and EFFICIENT
(each pattern names the fact channel or machine property that makes it fast).
Writers are TAUGHT this catalog up front (teaching pack / writer's excerpt);
hitting a wall because a familiar architecture is unrepresentable is a
documentation defect, not a writer error.

Each entry: problem shape -> blessed pattern -> why it is fast here -> what it
replaces from mainstream languages.

## P1. Command buffer (write intents)

Problem: deep code needs to mutate shared long-lived state (pool, arena,
world), and no clean exclusive window exists at depth.
Pattern: deep functions are `pure` or `reads('p)`; they compute and RETURN
write intents as plain values. Exactly one shallow function holds the single
`&uniq` and applies the intents. Effect rows make the architecture checkable:
grep the signatures — one `writes('p)` in the system.
Fast because: deep code carries `memory(read)`/`memory(none)` attributes
(channel 2: hoisting, CSE, reordering across calls), and read-only deep code
is the precondition for the parallel fan-out story (D1).
Replaces: `Rc<RefCell>` interior mutability, observer mutation, scattered
in-place writes. Those are unrepresentable here BY DESIGN.

## P2. Struct-of-arrays pool (append-only, index-linked)

Problem: many homogeneous-ish nodes with cross-references (AST, graph, ECS).
Pattern: one struct of parallel `buffer<T>` columns plus a count; a node is a
`u64` index; construction appends (`push` through `&uniq`); indices never
recycle; the whole pool drops at once. Reference:
`archive/m3/submissions/reference/whitefoot/arena_ast_builder.wf`.
Fast because: contiguous per-field columns (cache, vectorization), and the
borrowed-SoA shape is exactly what channel 1's scoped-alias facts optimize;
no per-node allocation, headers, or refcounts.
Replaces: `Rc<RefCell<Node>>` graphs, pointer-linked heap nodes, and Rust's
Vec-index arena WITH free-lists (STOR-1 rejects recycling: stale indices are
well-typed UAF).

## P3. Region staircase + static nursery (lifetime shape)

Problem: interleaved lifetimes vs bulk free — arenas leak if everything lives
in one region.
Pattern: nest regions by phase (request -> pass -> sub-pass); allocate into
the innermost region; anything that survives a phase is EXPLICITLY moved out
(`move`) to the outer owner — escape is visible and checker-verified; truly
interleaved individual lifetimes use `box`. Effect rows (`allocates('r)` vs
`allocates(heap)`) make the split auditable per signature.
Fast because: bulk free is O(1) per phase; affine moves make promotion a
header copy; no GC, no refcount traffic.
Replaces: GC nurseries (same insight, zero runtime), `Rc` lifetime webs.

## P4. Linear threading (exclusive access through a call chain)

Problem: a callee chain must transform exclusive state and hand it back.
Pattern: pass the affine value (or `&uniq`) in, return it (or the derived
state) out — possession flows like a token. v0 admits only bounded
statement-scoped reborrowing (OWN-6): a child borrow of a holder is a transient,
non-escaping call argument that suspends its parent for one statement, so the
token never silently forks or escapes.
Fast because: singleton provenance keeps the checker simple; the noalias facts
hold for usable borrows (a suspended parent yields no usable alias), so channel
1's soundness rests on T-A plus statement-scoped suspension.
Replaces: Rust's unbounded implicit `&mut` reborrow chains and aliased mutable
captures; Whitefoot's reborrow is bounded to one statement and cannot escape.

## P5. Env-struct behavior parameterization (FN-5)

Problem: callbacks / strategy objects / closures.
Pattern: behavior is a generic over a contract-conforming type; the
environment is an explicit struct threaded by value or borrow; closed-set
dispatch is `match`. No function pointers, no dynamic dispatch in the kernel.
Fast because: every call is direct post-monomorphization — inlining and the
channel-2 attributes survive; no vtable opacity.
Replaces: closures capturing mutable environments, trait objects, fn pointers.

## P6. Checked-law reduction (FN-4)

Problem: custom folds/reductions that a compiler cannot legally reorder.
Pattern: state the algebra (`law associative/commutative/identity`) in a
contract; conform the op; write the OBVIOUS sequential fold. The compiler
discharges the law and reassociates for you; a false law is refuted at
compile time.
Fast because: channel 3 — 3.3x measured over the serial shape; the transform
is licensed by a checked fact, not writer folklore.
Replaces: hand-written multi-accumulator loops resting on unchecked human
algebra (the signed-sat-add trap).

## P7. Branchless classifier (i1 dataflow)

Problem: byte/token classification with loop-carried state (word boundaries,
token starts, run detection) — the shape inside every scanner and utility.
Pattern: keep ALL state and predicates in `Bool` (copy, i1): predicates via
comparisons, combination via `band`/`bor`/`bnot`, transitions via
`set state = predicate;`, counters bumped through a give-match select
(`match p { True() => { give 1_u64; } False() => { give 0_u64; } }`).
NEVER route state through integer flags or match-arm control flow.
Fast because: the state stays an i1 recurrence, which the vectorizer widens
to full-width byte vectors (measured: width 16 vs width 2x4 for the integer
form — the difference between C parity and a 1.6-1.8x loss on wc-class
kernels). 2-variant tag-only user enums lower identically to Bool, so a
domain-named state enum costs nothing.
Replaces: integer state flags, branchy per-byte match chains.

## P8. Traps to the boundary

Problem: one trapping op in a hot leaf strips derived totality (willreturn)
from the whole call tower and blocks vectorization of reductions.
Pattern: validate at the edge (check/trap where cold), keep hot interiors
trap-free — bounded counters use `.wrap` ONLY where the bound is structural
(counter <= buffer length); everything else keeps `.trap`/`.checked` and
waits for proof-elision (OP-4 tier) rather than weakening semantics. Use
`--totality` to see exactly which trap poisons which tower.
Fast because: trap-freedom is what admits willreturn (hoisting/CSE of calls)
and single-exit loops (vectorization). Measured: one trap-per-increment
counter = zero vector ops; the wrap form = full SIMD, 2x on wc -l.
Replaces: sprinkling checks uniformly and paying for them in the one loop
that matters.

## P9. Exact capacity contract or recoverable shortage

Problem: an encoder/decoder writes caller-owned output, but the amount may be
fixed-ratio, cheaply preflightable, or genuinely data-dependent.  A
worst-case entry contract can make the inner loop look perfect by forcing
ordinary callers to overallocate or trap.
Pattern: use `requires` only when a false predicate means the caller has
violated the actual API contract.  For a fixed-ratio transform, state the
weakest overflow-safe capacity relation that covers the body.  If insufficient
capacity is an expected runtime outcome, test the next token/burst before any
of its effects and return a value such as `NeedMoreOutput`; do not turn that
outcome into a contract trap.  A preflight/exact-allocation API is appropriate
only when its validated size remains bound to the input it describes.  Never
put a merely common-case size or a rare worst-case allocation in `requires`.
Fast because: a checked exact relation can discharge repeated implicit bounds
checks, while the body-derived obligation report names a missing or mismatched
fact.  Recoverable boundary control preserves the useful small-buffer domain
and provides the explicit slow path needed by future guarded fast-region
proofs without weakening OP-4 safety.
Replaces: per-store bounds checks in fixed-ratio kernels, unconditional
maximum-size caller allocation, retry-after-partial-token mutation, and using
`requires` as an optimizer hint.

## Known gaps (findings, not yet patterns)

- In-place mutation interleaved with traversal of the same structure (graph
  rewriting while walking). Restructure via P1/P2 or reject (OWN-8 posture);
  relief valves carded: split_uniq disjoint views, checked Cell-for-copy.
- Shared memo/cache written during logically-read traversals: model as
  explicit `&uniq` cache parameter (the write is signature-visible) — needs a
  worked exemplar before it earns a P-number.
- Long-lived borrows stored in data (self-referential structs): structurally
  unrepresentable in v0 (structs store values, not borrows); the index pool
  (P2) is the blessed encoding.
