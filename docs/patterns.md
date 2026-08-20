# Whitefoot Pattern Doctrine (D6)

Status: seeded, non-normative writer guidance since 2026-07-09. Only the active
specification defines accepted source. The selected D6 direction is to test
whether a closed architecture-level vocabulary can stay COMPLETE (every task
modelable — a gap is a finding) and EFFICIENT (each pattern names the fact
channel or machine property that makes it fast) before any owner ratification.
Writers may be taught this catalog during validation; hitting a wall is a
catalog finding, not authority to invent a language rule.

Capability boundary: the current backend emits no effect-derived attributes or
alias metadata, performs no proof-driven check elision, has no termination
checker or `willreturn` derivation, and does not implement arenas. The speed
rationales in P1–P4 and P7–P9 therefore include historical measurements or
future hypotheses; each entry labels the current boundary. P6 and P10 already
state their exact v0.17 status.

Each entry: problem shape -> candidate or validated pattern -> current or
historical speed rationale -> what it would replace in mainstream languages.

## P1. Command buffer (write intents)

Problem: deep code needs to mutate shared long-lived state (pool, arena,
world), and no clean exclusive window exists at depth.
Pattern: deep functions are `pure` or `reads('p)`; they compute and RETURN
write intents as plain values. Exactly one shallow function holds the single
`&uniq` and applies the intents. Effect rows make the architecture checkable:
grep the signatures — one `writes('p)` in the system.
Current value: exact effect rows make scattered writes visible and reject a
false architectural summary. Potential speed: the retired channel-2 experiment
mapped read-only/pure code to memory attributes for hoisting, CSE, and call
reordering; the current backend does not emit those attributes. Read-only deep
code is also a prerequisite for a future verified parallel fan-out.
Replaces: `Rc<RefCell>` interior mutability, observer mutation, scattered
in-place writes. Those are unrepresentable here BY DESIGN.

## P2. Struct-of-arrays pool (append-only, index-linked)

Problem: many homogeneous-ish nodes with cross-references (AST, graph, ECS).
Pattern: one struct of parallel `buffer<T>` columns plus a count; a node is a
`u64` index; construction appends through `&uniq`; indices never recycle; the
whole pool drops at once. Current v0.17 executable reference for the
fixed-capacity append-only shape:
`tests/conformance/cases/x-borrowed-pool-tree-run.wf`.
Current value: contiguous per-field columns improve locality and avoid
per-node allocation, headers, and refcounts. Potential speed: the retired
channel-1 experiment also emitted scoped-alias facts for this borrowed SoA
shape; the current backend does not.
Replaces: `Rc<RefCell<Node>>` graphs, pointer-linked heap nodes, and Rust's
Vec-index arena WITH free-lists (STOR-1 rejects recycling: stale indices are
well-typed UAF).

## P3. Region staircase + static nursery (lifetime shape)

Problem: interleaved lifetimes vs bulk free — arenas leak if everything lives
in one region.
Pattern status: DEFERRED for current compiler use. v0.17 defines arena-related
language vocabulary, but the compiler reports arena operations as unsupported.
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
Current value: borrow-holder singleton provenance keeps the checker simple; a
suspended parent yields no usable alias under the checked relation.
Direct slices are the separate v0.17 case: they carry a finite static origin
set, and every alias and effect judgment checks the whole set even though one
runtime descriptor points to one root. A future alias-metadata consumer would
also need the holder-singleton and finite-origin coverage proofs; none ships in
the current backend.
Replaces: Rust's unbounded implicit `&mut` reborrow chains and aliased mutable
captures; Whitefoot's reborrow is bounded to one statement and cannot escape.

## P5. Env-struct behavior parameterization (FN-5)

Problem: callbacks / strategy objects / closures.
Pattern status: DEFERRED in v0.17. The active specification defines static
contract, complete-conformance, and checked-law validation, but it rejects
source-contract generic bounds and defines no member-call operation that could
select a conformance binding. Therefore contract-driven env-struct behavior is
not currently a writable Whitefoot pattern. For a closed behavior set, use an
enum and exhaustive `match`; otherwise use explicitly named direct functions
and thread the environment struct by value or borrow.
Candidate direction: an eventual approved form would keep the environment
explicit and monomorphize a checked member call to a direct call, but v0.17
does not claim that mechanism or its performance.
Would replace: closures capturing mutable environments, trait objects, and
function pointers.

## P6. Checked-law reduction (FN-4)

Problem: custom folds/reductions that a compiler cannot legally reorder.
Pattern status: validation-only in v0.17. State the admitted algebra
(`law associative/commutative/identity`) in a contract and conform its
ordinary top-level function. The compiler must discharge the law for source
acceptance and refutes an invalid or unavailable law at compile time. The
stored base derivation is not optimizer authority, so v0.17 does not
reassociate the sequential fold from that record.
Potential speed: the archived channel-3 experiment measured 3.3x over the
serial shape. Shipping that transform requires a separately approved fact
family that independently rederives the law and names its exact authorized
consequence; until then facts-off lowering is unchanged.
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
Historical speed evidence: in the retired wc-class experiment, the `i1`
recurrence vectorized at width 16 while the integer form used width 2x4, a
1.6–1.8x gap. The current compiler supports Boolean and two-variant tag-only
forms, but this floor result has not been revalidated on a selected project.
Replaces: integer state flags, branchy per-byte match chains.

## P8. Claims to the boundary

Potential problem for a future totality consumer: one retained claim in a hot
leaf may block a `willreturn` proof for the call tower and inhibit
transformations. Pattern status: DEFERRED as a totality/optimization pattern.
The current language has no termination checker, `pure` does not promise
return, the compiler never emits `willreturn`, and no `--totality` report
exists. It remains valid ordinary design advice to establish an invariant at a
boundary and keep its auditable claim outside the hot loop when the same fact
dominates every use. Use `.wrap` only where modular behavior is the intended
semantics; it must never evade an exact operation's static domain obligation.
Historical speed evidence: the retired wc line-count experiment found that a
trap-per-increment form produced no vector operations while the semantically
valid wrapping-counter form reached full SIMD and roughly 2x throughput. A
future totality consumer needs its own selected project and proof boundary.
Replaces: sprinkling invariant claims uniformly and paying for them in the one
loop that matters.

## P9. Exact capacity contract or recoverable shortage

Problem: an encoder/decoder writes caller-owned output, but the amount may be
fixed-ratio, cheaply preflightable, or genuinely data-dependent.  A
worst-case requirement can make the inner loop look perfect by forcing ordinary
callers to overallocate or making legitimate calls impossible to prove.
Pattern: use a `requires` clause only when a false predicate means the caller has
violated the actual API contract.  For a fixed-ratio transform, state the
weakest overflow-safe capacity relation that covers the body.  If insufficient
capacity is an expected runtime outcome, test the next token/burst before any
of its effects and return a value such as `NeedMoreOutput`; do not turn that
outcome into a requirement or claim. A preflight/exact-allocation API is
appropriate only when its validated size remains bound to the input it
describes. Never
put a merely common-case size or a rare worst-case allocation in `requires`.
Current value: one `contract` block may state several independent `requires`
goals. Every ordinary caller establishes every goal in the same pre-transfer
state; the callee body receives them as static facts and executes no prologue.
The sole `command fn main` has no contract, so there is no entry exception or
wrapper check. Recoverable boundary control preserves the useful small-buffer
domain. The current compiler can use these facts to discharge existing finite
obligations but provides no general Boolean theorem prover. Any future guarded
fast region must re-establish its authority without weakening OP-4 safety.
Replaces: per-store bounds checks in fixed-ratio kernels, unconditional
maximum-size caller allocation, retry-after-partial-token mutation, and using
`requires` as an optimizer hint.

## P10. Direct returned view

Problem: a helper must pass through or select a read-only slice without moving
the backing owner or hiding where the result may point.
Pattern: return `own slice<'r, T>` directly. Every possible parameter supplier
is also written as exactly `own slice<'r, T>` under the same formal region and
element type. A function with several such parameters may return any of them,
but the caller conservatively treats all of them as possible origins. If a
helper always selects one source and that precision matters, give that source
the result region and put unrelated slices under distinct formal regions.
Named constants are also legal suppliers. Do not return a fresh view of local,
raw-borrowed, or arena storage, and do not return `& slice` or `&uniq slice`;
those forms need provenance or cleanup semantics that v0.17 deliberately does
not claim.
Fast because: the written signature is the complete interprocedural summary.
Calls substitute finite origin sets and check aliases and effects against the
whole union without opening bodies, computing recursive fixed points, changing
the two-word slice descriptor, or adding a runtime tag.
Replaces: hidden body-derived return-borrow summaries and caller guesses about
which same-region argument a returned view references.

## P11. Counted half-open range

Problem: a fixed ascending index walk needs the current index bound inside the
body without hand-written termination tests, increments, or claims.
Pattern: write `for @label i in lower..upper { ... }` when both endpoints are
`own u64` terms or constants. They are evaluated once from left to right; `i`
is a read-only body binding, the upper endpoint is excluded, and
`lower >= upper` is zero-trip. A normal fallthrough advances by one; `break`,
`return`, and propagated errors do not. Use ordinary `loop` when progress is
not exactly this counted shape. Do not write a claim for `i < upper`; the
compiler supplies that structural fact, while derived offsets such as `i-k`
still require the real lower-bound relation.
Current value: the SHA-256 reference uses this one form for its three index
walks, removes four claims, and proves all nine schedule accesses. The form
adds no general induction, iterator protocol, reverse range, step, or
post-loop equality.
Replaces: `let i`, `loop`, equality break, index-bound claim, and wrapping
increment boilerplate for an exact half-open u64 walk.

## P12. External constrained subject takes a value path

Problem: a protected storage access uses an offset derived from process or
system input, so valid hostile input may falsify its bound. Test the relation
with a real branch and return the domain's normal error value on the false
edge. A `claim` or an ordinary callee requirement is not a repair: each turns
expected external failure into a trap or an uncallable path. Main has no
contract and no process-entry wrapper check.

Place the branch where the protected relation belongs. For a local protected
access, branch in the function that owns that access. For a call rejection,
branch in the rejecting caller before the call so its unasserted state proves
the complete bridged goal; alternatively restructure the dataflow so the
external value no longer reaches the callee's constrained subject. An internal
constrained subject may still use an honest invariant `claim` under its
ordinary lifecycle. External values used only as a bound, storage base,
write-address choice, or unrelated goal operand do not taint the constrained
subject and need no repair merely for being external. This does not exempt a
write address's own protected offset obligation when that offset is itself the
constrained subject.

Replaces: assertion-backed bounds on malformed input and moving the same
failure behind a helper contract.

## P13. Return the decision, not the access

Problem: a helper must choose between two borrowed sources and hand the chosen
one back, but the callable boundary cannot say which one it chose.
`fn pick['r](a: &uniq 'r Node, b: &uniq 'r Node) -> selected: &uniq 'r Node` is rejected
at its own `rtype` [FN-1]: two parameters share the result's region and kind,
so no caller can root the returned claim, and a result no caller can bind is
the declaration's error rather than the caller's. Pattern status: active v0.32 guidance.

Decide which fix applies by asking why there are two sources. If the sources
are structurally distinct — a node and its scratch buffer, a subject and its
dictionary — give the non-source its own formal region:
`fn pick['r, 's](a: &uniq 'r Node, b: &uniq 's Node) -> selected: &uniq 'r Node` is
accepted, and its result is an ordinary holder over `a`'s storage that the
caller binds, writes through, and reborrows from. If instead the choice is
data-dependent, no signature can name the source, and the access belongs to
the caller: return the decision as an owned value — a two-variant enum, or an
index into a pool (P2) — and let the caller re-borrow from the place the
decision names.

The worked shape for the data-dependent case is three parts. The callee
`fn heavier(a: &'r Node, b: &'r Node) -> side: own Side reads('r)` reads both weights
through its shared borrows and returns `Left()` or `Right()`; it takes shared
borrows, so both sources may name one region and nothing is ambiguous — a
returned owned value has no provenance. The caller binds
`let side = heavier(a: &'a left, b: &'a right);`, and then `match side` takes
the exclusive borrow it actually wants inside the taken arm, from `left` or
from `right` by name. The result is longer than the rejected one-liner and
that is the whole trade: the borrow is created where its source is a written
place, so the checker sees one root per holder, and the caller keeps both
sources usable until it commits.

Fast because: the decision is a scalar. The read pass takes shared borrows
that constrain nothing, and the write pass takes exactly one exclusive borrow
at the place it names, so no facts are lost to a conservative merge.

Replaces: an ambiguous-provenance borrow-returning signature, and the
caller-side workaround of binding a result the language cannot root.

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
