# Whitefoot Pattern Doctrine (D6)

Status: seeded, non-normative writer guidance since 2026-07-09. Only the active
specification defines accepted source. The selected D6 direction is to test
whether a closed architecture-level vocabulary can stay COMPLETE (every task
modelable — a gap is a finding) and EFFICIENT (each pattern names the fact
channel or machine property that makes it fast) before normative adoption.
Writers may be taught this catalog during validation; hitting a wall is a
catalog finding, not authority to invent a language rule.

This document carries guidance for the active specification at
`spec/kernel-spec.md`, including the readable byte stream, the socket address,
and the two-field connection introduced by v0.46 (P34), the ordered result list
and its destructuring `let` and `set` binder forms introduced by v0.45 — a
transforming operation now
hands back the value it was given beside what it computed, `-> (rest: own
Vector<u8>, written: own u64)`, instead of a two-field struct per operation, and
its caller writes `let (rest, written) = collect(...);` — and the four measure
terms and their readers `len_of`, `cap_of`, `room_of` and `head_of`, also
introduced by v0.45 (P16): a measured value carries the standing facts
`len_of(P) <= cap_of(P)`, `head_of(P) <= cap_of(P)` and
`len_of(P) + room_of(P) = cap_of(P)` with no writer statement, a write to a
sibling field kills no measure, and it is those four `_of` spellings, not the
four bare words a writer wants for a binding, that are reserved against every
writer declaration — and the four call transports v0.45 also introduces (P16),
which fix what a call kills from the callee's declared parameter and never from
the argument's spelling or the callee's body, so a helper that fills a caller's
storage takes a view and a call through a `&uniq` run costs its caller that
run's measures — the contract-clause
measure operands and the
call datum introduced by v0.44 (P16, P21), the loop-body
region block and the associative [ENT-6] join introduced by v0.43, the one canonical
region spelling introduced by [FORM-8] in v0.42, the comparison symbols and
call-site `::` delimiter introduced by v0.41, the source-proof forms introduced
by v0.40, the unified-state
completion-I/O forms introduced by v0.37, the
per-iteration scratch form [PAR-3] admits (P15), and the three forms the
2026-08-28 blind-writer trial found a writer lacking: the inline factory reserve
inside P15, the hoisted length fact (P16), and the accumulator fold (P17),
whose rejection [LIV-2] removed by admitting the commit that reads its target
out. P18 is the explicit buffer a loop holds in place of the output resource
it may not hold. P19 is the join-image rule for a loop binding advanced under a
condition, the one place the 2026-09-03 scenario sweep found the catalog
misleading a writer. References to earlier versions describe historical evidence,
not a second writable proof surface.

Implementation boundary: the work-branch compiler is being aligned to prove
supported partial operations before lowering and erase source proof syntax. It
is not described as complete or activated before whole-repository verification.
The backend
still emits no effect-derived attributes or alias metadata, has no termination
checker or `willreturn` derivation. Local region-confined arenas now lower and
execute, with selected-target layout and address checks before emission and
release on the implemented region exits. Arena parameters and slices over arena
content remain explicit capability limits. The [PAR-3] judgment is implemented;
its first multi-operation actualization is the narrow direct counted-loop form
described in P15, not every loop that receives a permitted judgment.
The speed rationales in P1–P4 and P7–P9 therefore include historical
measurements or future hypotheses; each entry labels the current boundary. P6
and P10 already state their exact v0.17 status.

Each entry: problem shape -> candidate or validated pattern -> current or
historical speed rationale -> what it would replace in mainstream languages.

## P1. Command buffer (write intents)

Problem: deep code needs to mutate shared long-lived state such as a pool,
arena, or process resource, and no clean exclusive window exists at depth.
Pattern: deep functions are `pure` or `reads(state)`; they compute and RETURN
write intents as plain values. Exactly one shallow function holds the single
`&uniq` and applies the intents. Effect rows make the architecture checkable:
grep the signatures and find one `writes(state)` in the system. The superseded
v0.36 wrote those subjects as lifetimes; v0.39 introduced formal state paths and
active v0.40 retains them.
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
Pattern: nest regions by phase (request -> pass -> sub-pass); allocate into
the innermost suitable region with `arena_new::<'r, T>(value)`. The returned
`arena<'r, T>` stays inside that region, and `deref` reads its content. The
compiler rejects an arena value that leaves the region rather than promoting
it implicitly. A value that truly needs a different lifetime therefore belongs
in that region from the start or uses another owned storage form such as `box`.
A single value at a store is `Box<'s, T>` [S39], formed by `heap_box(store: h,
value: e)` or `arena_box(store: a, value: e)`: the call writes no type or region
argument, its outcome is `Result<Box<'s, T>, T>` whose `Err` arm hands the value
back, `deref(cell)` reads the referent, and `let Box(value: v) = move cell;`
takes the value out and releases the cell in one statement. A cell carries no
measure, so a read of one owes no proof; a recursive shape writes the region on
its own nominal — `enum Tree['s] { Leaf(); Branch(left: Box<'s, Tree<'s>>,
right: Box<'s, Tree<'s>>); }` — and its release is the recursive walk.

Effect rows keep the allocation site visible in a signature. Since [S23] the
`allocates` entry takes the same formal-rooted paths `reads` and `writes` take,
so an allocation from a store whose provider is a value names that provider:
`allocates(store)` for a helper taking `&uniq Arena<'s, ...>` or `&uniq Heap`,
and `allocates(heap)` for a `main` whose `command.heap` binder is spelled
`heap`. The ambient heap of `box<T>` and `buffer<T>` has no provider value and
therefore no path, so it writes no entry at all; `allocates(arena 'r)` is the
one remaining region-keyed entry and retires with `arena<'r, T>`.

Current value: each region owns a compiler-generated allocation list. An
allocation pushes one `{ next, content }` node. Normal fallthrough, loop
re-entry, `break`, `return`, and nested-region exits walk the correct list and
release every node; an allocation naming an enclosing region survives an inner
region's exit. Before emission, selected-target checking computes the complete
node layout, including padding and alignment, and verifies the allocator and
address bounds. Backend tests execute content addressing and every exit shape
above. The load-bearing cases are
`selected_target_validates_the_complete_padded_arena_node`,
`arena_node_emission_uses_the_validated_complete_llvm_type`,
`a_confined_arena_allocation_reads_and_releases_with_its_region`, and the two
`arena_release_covers_*` cases. Release currently walks all nodes in the region,
so it is one region-exit operation with O(number of allocations) work, not an
O(1) reset.

Boundary: local confined allocation, `deref`, inside-region value delivery, and
release are implemented. Arena-typed function parameters and slices over arena
content still stop as unsupported compiler capabilities. This pattern must not
be read as evidence for those wider forms.
Replaces: tracing or reference-counted lifetime management for values that
naturally die with one lexical phase. The current implementation still pays one
list insertion per allocation and one list walk at region exit.

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
Candidate direction: a possible future specification form would keep the
environment explicit and monomorphize a checked member call to a direct call, but v0.17
does not provide that mechanism or its performance evidence.
Would replace: closures capturing mutable environments, trait objects, and
function pointers.

## P6. Checked-law reduction (FN-4)

Problem: custom folds/reductions that a compiler cannot legally reorder.
Pattern status: validation-only in v0.17. State the admitted algebra
(`law associative/commutative/identity`) in a contract and conform its
ordinary top-level function. The compiler must discharge the law for source
acceptance and refutes an invalid or unavailable law at compile time. The
checked law is not yet optimizer authority, so v0.17 does not reassociate the
sequential fold from that fact.
Potential speed: the archived channel-3 experiment measured 3.3x over the
serial shape. Shipping that transform requires one specification-fixed consumer
of the originating checked fact and an exact permitted consequence. The
originating semantic decision remains the only proof authority. Until then
facts-off lowering is unchanged.
Replaces: hand-written multi-accumulator loops resting on unchecked human
algebra.

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

## P8. State a proof at the boundary that maintains it

Problem: several partial operations need one relation maintained by a loop or
local state machine. Repeating a runtime check at every use would add branches
and would still leave the compiler unable to use the relation after the loop.
Pattern: put the maintained relation in the loop's parenthesized header. The
checker proves the base case and every arbitrary reachable backedge, then
exports only the separately proved normal-exhaustion consequence. The binding
is the first item of a counted `for`; every later item is a header invariant,
and the final item has no trailing comma:

```whitefoot
for (
  i in 0_u64..count,
  invariant per_byte: sum <= 255_u32 * i
) {
  let w = deref(weights)[i];
  let wide = cvt::<u8, u32>(w);
  set sum = sum + wide;
}
```

Header entries cannot carry `use` blocks, and their names exist only inside the
body. Here AUTO subtracts the one published affine premise `per_byte`; DIRECT
then proves the residual from the `u8` type interval of `wide`. An explicit use
block would be redundant and invalid. The bound is stated relative to the
counter, so it carries no overflow conclusion of its own until the counter is
itself bounded: with `count` an unbounded `len_of(deref(weights))` the byte-for-byte
identical loop is undischarged at [INV-1], and one `requires n <= 65536_u64`
over that length in the function's own contract compiles it. Put a
cross-function fact in the callee's verified `ensures`; the caller receives it
only after the callee's return proof succeeds. Use a local
`invariant { use ... }` when three or more published affine premises outside
the final fixed L0-image route, a special elimination route, or an explicit
factor needs written guidance.

This is the default decision rule, not merely a performance suggestion. If the
false edge would contradict the function's contract or the algorithm's stated
invariant, do not add an early return or another observable branch just to make
a partial operation compile. State the missing proof, or improve the checker
when it cannot verify the stated proof. Use executed control flow only when the
false edge is a real result the program is meant to handle, as in P9 and P12.

All three forms are compile-time only. They are erased before lowering and add
no branch to the hot loop. Use `.wrap` only where modular behavior is the
intended semantics; it must never evade an exact operation's static domain
obligation. Historical speed evidence from the retired wc line-count experiment
showed that a per-increment runtime check prevented vectorization while the
semantically valid wrapping counter reached full SIMD and roughly 2x
throughput. That measurement is historical, but the writer rule is current:
state one machine-checked fact at the boundary that actually maintains it.

Replaces: repeated assertions, duplicated guards, and caller restatements of a
callee fact.

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
outcome into a requirement or invariant. A preflight/exact-allocation API
is appropriate only when its validated size remains bound to the input it
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
Pattern: return `own Slice<'r, T>` directly. `'r` is written at the result and
at every supplier because they share it [FORM-8]. Every possible parameter
supplier is also written as exactly `own Slice<'r, T>` under the same region
and element type. A function with several such parameters may return any of them,
but the caller conservatively treats all of them as possible origins. If a
helper always selects one source and that precision matters, give that source
the result region and put unrelated slices under distinct formal regions.
Named constants are also legal suppliers. Do not return a fresh view of local,
raw-borrowed, or arena storage, and do not return `& slice` or `&uniq slice`;
those forms need provenance or cleanup semantics that v0.17 deliberately does
not provide.
Fast because: the written signature is the complete interprocedural summary.
Calls substitute finite origin sets and check aliases and effects against the
whole union without opening bodies, computing recursive fixed points, changing
the two-word slice descriptor, or adding a runtime tag.
Replaces: hidden body-derived return-borrow summaries and caller guesses about
which same-region argument a returned view references.

## P11. Counted half-open range

Problem: a fixed ascending index walk needs the current index bound inside the
body without hand-written termination tests, increments, or assertions.
Pattern: write the closed one-line header below when both endpoints are
`own u64` terms or constants:

```whitefoot
for (i in lower..upper) {
  consume(i);
}
```

The endpoints are evaluated once from left to right; `i` is a
read-only body binding, the upper endpoint is excluded, and
`lower >= upper` is zero-trip. A normal fallthrough advances by one; `break`,
`return`, and propagated errors do not. Use ordinary `loop` when progress is
not exactly this counted shape. Add a loop label only when an explicit
cross-level `break` needs one; an `invariant` is structurally attached to its
direct parent loop and never names the label. Do not write a proof step for
`i < upper`; the compiler supplies that structural fact, while derived offsets
such as `i-k` still require the real lower-bound relation.
Current value: the SHA-256 reference uses this one form for its three index
walks, removes four former runtime assertions, and proves all nine schedule
accesses. The source-proof successor adds explicit header induction but never guesses an
invariant; it still adds no iterator protocol, reverse range, variable step, or
unconditional post-loop equality.
Replaces: `let i`, `loop`, equality break, redundant index proof, and wrapping
increment boilerplate for an exact half-open u64 walk.

## P12. External constrained subject takes a value path

Problem: a storage access uses an offset derived from process or system input,
so valid input may falsify its bound. Test the relation
with a real branch and return the domain's normal error value on the false
edge. An unconditional invariant or an ordinary callee requirement is
not a repair: each turns expected external failure into an uncallable path.
Main has no contract and no process-entry wrapper check.

Place the branch where the protected relation belongs. For a local protected
access, branch in the function that owns that access. For a call rejection,
branch in the rejecting caller before the call so the true edge proves
the complete bridged goal; alternatively restructure the dataflow so the
external value no longer reaches the operation. An internal relation that is
true on every execution may instead be stated as a machine-checked local or
loop-header `invariant`; writing the conclusion never grants it authority—the
checker must still prove it, either through AUTO or its explicit `use` steps.
Every address and offset still has its own exact domain obligation regardless of
where its operands originated.

Replaces: assertion-backed bounds on malformed input and moving the same
failure behind a helper contract.

## P13. Return the decision, not the access

Problem: a helper must choose between two borrowed sources and hand the chosen
one back, but the callable boundary cannot say which one it chose.
`fn pick['r](a: &uniq 'r Node, b: &uniq 'r Node) -> selected: &uniq 'r Node` is rejected
at its own `rtype` [FN-1]: two parameters share the result's region and kind,
so no caller can root the returned borrow, and a result no caller can bind is
the declaration's error rather than the caller's. Pattern status: current
candidate guidance, introduced before v0.36 and preserved since.

Decide which fix applies by asking why there are two sources. If the sources
are structurally distinct — a node and its scratch buffer, a subject and its
dictionary — give the non-source its own formal region:
`fn pick['r](a: &uniq 'r Node, b: &uniq Node) -> selected: &uniq 'r Node` is
accepted — `'r` is written because the result shares it with `a`, and `b`'s own
region relates to nothing and is therefore left unwritten [FORM-8] — and its
result is an ordinary holder over `a`'s storage that the caller binds, writes
through, and reborrows from. If instead the choice is
data-dependent, no signature can name the source, and the access belongs to
the caller: return the decision as an owned value — a two-variant enum, or an
index into a pool (P2) — and let the caller re-borrow from the place the
decision names.

The worked shape for the data-dependent case is three parts. The callee
`fn heavier(a: &Node, b: &Node) -> side: own Side reads(a, b)` reads both
weights through its shared borrows and returns `Left()` or `Right()`. The
superseded v0.36 spelled that effect `reads('r)`; since v0.39 a region
remains only the shared loan lifetime, and since v0.42 neither parameter
writes one because neither relates to another position [FORM-8]. Both forms
take shared borrows, so the returned owned decision has no borrow provenance.
The caller binds
`let side = heavier(a: &left, b: &right);` inside the region block whose
region those borrows take, and then `match side` takes
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

## P14. Guide a larger affine proof with `use`

Problem: the automatic checker knows several affine relations, but the next
relation needs three or more published affine premises outside the final fixed
L0-image route, a special elimination route, or an explicit factor. The
compiler's automatic boundary must not be discovered by
trial and error, and it cannot be read off the diagnostics either: [DIAG-1]
reports one rule and one location per rejection, so a probe `invariant` that
draws no message of its own has not been shown to hold — an earlier failure may
simply be standing in front of it. The boundary is fixed by the language:
zero-premise direct proof, every coefficient-one single premise, every unordered
coefficient-one pair including the same premise twice, then the final fixed
L0-image route. If none applies, write a local `invariant` and direct its
finite calculation with `use`.

```whitefoot
invariant total_limit: first + second + third <= first_limit + second_limit + third_limit {
  use first_bound;
  use second_bound;
  use third_bound;
}
```

The checker snapshots the facts before the outer invariant and proves every
`use` against that same snapshot. A prior use cannot help prove a later one and
no use publishes a fact; only `total_limit` enters the ordinary proof context
after the combination succeeds. A named use resolves the exact live theorem;
a relation-form use is itself discharged by AUTO. A written factor begins at
two—factor one must be omitted—and the same normalized premise cannot appear
twice. The final target may be a direct weakening of the weighted sum.

A nonempty use block is an error when AUTO already proves the outer target.
The pair family includes the self-pair, so a doubling target such as
`x + x <= limit + limit` already follows automatically from the single
premise `x <= limit`, and a use block naming that premise is rejected as a
redundant block rather than accepted as guidance.
This is a canonical-source rule tied to the exact language version, not a
warning about a compiler optimization. Use a header invariant when the relation
is the induction contract; use `ensures` when it must cross a function
boundary; use a typed result or real branch when the condition can legitimately
be false. AI may search while authoring the source, but the compiler performs
no SMT query, heuristic premise selection, timeout-bounded attempt, or runtime
fallback.

Replaces: assertions, intentional aborts, "trust me" comments, and compiler
guessing over proof candidates.

## P15. Per-iteration scratch in an I/O loop

Problem: a loop opens and reads one file per iteration and folds what it read.
The habit imported from every other systems language is to allocate the name
buffer and the destination buffer once above the loop and reuse them, because
allocation is expensive. Under [PAR-3] that habit is exactly what costs the
loop its pipeline: the destination is storage the body writes and the iteration
does not introduce, a `may-suspend` call retains a borrow of it past its own
submission, and the staged permission denies. Reusing one buffer is also what
makes the program genuinely order-dependent — after a short read the bytes
beyond it are the previous iteration's.

Pattern: construct the per-iteration scratch **inside** the loop body.

```whitefoot
for @scan (index in 0_u64..8192_u64) {
  let name = buffer_new(16_u64, 0_u8);
  let data = buffer_new(65536_u64, 0_u8);
  let rendered = name_at(name: &uniq name, index: index);
  region 'f {
    match reserve_handle(factory: &uniq files) {
      Ok(value: permit) => {
        region {
          match open_file(permit: move permit, root: &'f cwd,
                          name: &name, start: 0_u64, end: 10_u64) {
            FileOpened(value: handle) => { /* read, fold, accumulate */ }
            FileOpenFailed(error: problem, permit: refused) => { }
          }
        }
      }
      Err(error: spent) => { break; }   // the factory is out of credits: leave before any submission
    }
  }
}
```

Three companion rules make the rest of that body work, and each is a form to
copy rather than a fact to rediscover:

- **Take every early exit before the first submission.** A `break` or `return`
  written after the submission denies the loop: with later iterations already
  in flight, the decision to leave would be taken after opens the source-order
  execution never performs. Write the guard and its `break` at the top of the
  body, before any I/O. `let written = propagate write_once(…);` is such an
  exit and not an exception to it: the `Err` edge is selected by the
  submission's own outcome, so it leaves from the remainder however early the
  statement is written. Match on the outcome instead and handle the error
  inside the body. An open has no `propagate` form at all: its outcome is
  `FileOpened(value: …)` or `FileOpenFailed(error: …, permit: …)`, and the
  failed arm hands the permit back, so the body decides what to do with the
  error and the credit in the same place.
- **Write the accumulator as an ordinary source-order `set`.** `set sum = sum
  +wrap digest;` needs no associativity, no identity element, and no
  combination tree, because [PAR-3] commits the remainder's writes to storage
  rooted outside the body in iteration order. This is strictly more general
  than [PAR-2]'s admitted operation set: a non-associative fold, a float fold,
  and a `Result` route are all admitted here.
- **Reserve the handle factory in the prologue, inline.** `reserve_handle` takes
  and returns a short unique `&uniq HandleFactory` loan inline [SYS-10], and
  prologues run in index order without overlapping, so one enclosing factory
  serves every iteration with no replication and no [OWN-5] relaxation. Its
  `Err(ResourceExhausted)` edge is the program's own source-order outcome (the
  factory's capacity is real: one credit per descriptor the target provides),
  so match on it and take the exit there, before the open: that is an early
  exit before the first submission, which the first rule admits. A program
  that reuses its capacity closes explicitly (`close_read`,
  `close_directory`, `close_directory_source`, `close_listener` and
  `close_connection` return the permit); derived release closes but returns
  nothing. One factory serves every handle the target counts, so a listener
  and a connection draw from the same capacity a file open draws from
  [SYS-10, SYS-17, SYS-18]. Write
  the reserve and the open in the loop body itself. Factoring the pair into a
  helper — `fn open_source_from(factory: &uniq HandleFactory, …)` — costs the
  loop its pipeline, because the callee's own retained loan is what
  the staged judgment then sees. Two programs identical except for that
  factoring (‹loop› stands for the writer's own file and line; the verdict text
  after it is byte-exact):

  ```text
  inline  PAR stage  ‹loop›             for  permitted  staged at open_file(…); 5 places classified
  helper  PAR stage  ‹loop›             for  denied     condition 3: a may-suspend call retains a borrow
                     past its own submission on storage the body writes and the iteration does not
                     introduce; instead, give each iteration its own resource; or, where the body only
                     publishes to that storage — an output stream is the pointed case — hoist the
                     per-iteration write out of the loop, folding a total in the body and writing it
                     once after the loop; or leave this loop sequential, because storage that carries
                     one position cannot be held by two iterations at once, at &uniq files
  ```

  When the factory is itself a borrow — which it is in any recursive walker —
  [OWN-6] pushes the other way and admits no inline `region { let
  permit = …; match open_… }`, because that region holds two statements. The
  two rules genuinely conflict there, and the resolution is that only one of
  the two forms is a program at all. Which form to write is decided by how the
  loop holds its factory, and the three measured outcomes are (‹loop› again
  stands for the writer's own file and line):

  ```text
  owned factory, inline    PAR stage  ‹loop›                   for  permitted  staged at
                           open_file(permit: move permit, root: &'f cwd, name: &name,
                           start: 0_u64, end: 4_u64); 4 places classified
  borrowed factory, inline [OWN-6] InvalidChildReborrow — the program does not compile
  borrowed factory, helper PAR stage  ‹loop›                   for  denied     condition 3: a
                           may-suspend call retains a borrow past its own submission on storage the
                           body writes and the iteration does not introduce; … at
                           &uniq deref(factory)
  ```

  So: **in a loop whose factory is an owned entry parameter — every top-level
  I/O loop — write the reserve and the open inline, and the staged permission
  is granted.** **In a recursive walker, whose factory is a `&uniq` borrow,
  write the helper factoring, and the pipeline is the price.** There is no
  third form: the inline shape does not compile there, so the choice is between
  a denied loop and no program. The helper is the whole of the idiom only when
  its two companions come with it — the region's single statement is the
  `match` on the helper's call, and every statement that uses the opened value
  lives inside that `match` arm, because the opened value dies with the region.
  `tests/programs/dir_walk.wf` is that form written out, and [OWN-6]'s own
  rejection now states all three parts. The blind-writer trial that found this
  also proposed that the helper boundary should not cost the pipeline; that is
  a compiler change and is still open, so the price above is today's price and
  not a fixed one.

Read the verdict rather than guessing it. An ordinary `whitefootc` compile
prints a denied staged verdict to stderr, prefixed `whitefootc: note:`, with
every denied row of that loop's disposition table under it; the compilation
succeeded and the note is not a rejection. A loop whose staged verdict is
granted says nothing at all — including when its counted [PAR-2] verdict is
denied, which is the ordinary case for the form above: the counted rule refuses
the short factory loan the staged rule exists to admit, and that denial is
deliberately withheld from the default channel rather than telling a writer
their granted loop was denied. It is in the full report. A `--no-overlap`
build prints none of these notes: that flag has already said this build takes
no overlap at all, so a denied loop is the build the writer asked for rather
than news about the program they wrote.

`whitefootc --par-ledger` is that full report: one `PAR stage` line per loop
that performs I/O, and one `PAR place` line for every place the judgment
classified, with its disposition and the reason, plus the `PAR pair`, `PAR
chain`, and `PAR loop` lines of the other judgments. A denial names the
offending place, the numbered condition, and the admitted form, and the flag
prints it under every lowering. Every notice is one of those lines, byte for
byte.

One remedy the report can print is not one a writer can take, and it says so:
where a loop's exit is selected by the may-suspend call's own outcome — the
`ReadEnd` break of a read-to-EOF loop over one file — the condition-2 line
states that [PAR-3] cannot stage that loop as written. The shapes staged today
by the permission judgment include a fixed-trip bounded loop and a per-file loop
over names; one file's chunk loop stays sequential. Permission does not by
itself promise that the backend has a multi-operation schedule for the whole
permitted set.

The current multi-operation actualizer covers one smaller form: one direct
counted loop, one straight-line prologue, and a final `match` whose scrutinee is
the selected may-suspend system call. It derives a fixed two-slot driver from
the checked source. A native POSIX completion target asks once for a window in
`1..2`; a qualified target without native completion keeps the same generated
control flow, fixes the window at one, and issues direct calls. The driver
issues one batch, drains every issued result in source order, then reuses slot
zero. Executable tests cover a changing path slice on each iteration, an odd
five-iteration final batch, the ordinary success and error arms, twelve issued
iterations, and helper counts zero, one, and four.

That evidence does not cover the full P15 example above. In particular, wider
open/read/fold bodies, a remainder that needs the counted binder or a prologue
local, additional branch or cleanup shapes, other operation families, and more
than one staged loop in a function remain on the ordinary path. If a function
contains two staged loops, the compiler currently transforms neither. The
permission judgment and its ledger still apply to those loops; only the
multi-operation schedule is narrower.

`dir_walk.wf`, `wfgrep.wf`, and `byte_string.wf` remain useful negative
boundaries. Their walker loops use the helper factoring because the inline
borrow form does not compile, and their chunk loops leave on a read result, so
condition 2 keeps them sequential. They are not evidence for the new direct
counted-loop driver. The driver evidence currently lives in lowering and
backend tests rather than in a selected real program.

Replaces: hoisting scratch buffers out of loops for allocation cost, and every
writer-visible depth, window, batch, or `par for` marker a language would
otherwise need to express I/O overlap. There is no source spelling for how many
operations stay outstanding. The compiler fixes the available storage at two
slots for the implemented form, and the selected target chooses a window no
larger than those two slots.

## P16. One length fact above the writes

Problem: a program fills storage through callees and then hands a prefix of it
to a call whose `requires` bounds that prefix by the storage's length. The
habit — and the reading of [ENT-5] an unguided writer forms in twenty minutes —
is that the callee's write killed the length fact, so `let room =
len_of(line);` has to be re-bound after every call that wrote through the
borrow. Whether it did is decided by **the callee's declared parameter**, and by
nothing else [CALL-5].

Within one body the support of a measure term over `P` is `P`'s **descriptor
storage** [MSR-2] — the measure words the value carries — together with every
holder a prefix of `P` reads through and the support of every offset in `P`. An
element write overlaps the descriptor storage of the written element and none of
`P`'s own, so it kills no measure of `P`. Only a write to `P`'s own descriptor
storage or to a prefix of it — a fresh `buffer_new`, a `set` of the whole
binding, a `replace` of `P` — kills it.

At a call the transports decide which of those two a projected callee write is:

- a **shared borrow** `&'r T` of any type kills nothing at all [CALL-1];
- a **view** parameter — `&uniq MutSlice<'r, T>`, or the range-bearing operand
  of an [SYS-8] operation — writes the viewed range's element storage, so every
  measure of the origin place and of the view survives it [CALL-3];
- an **`own`** parameter consumes an affine or linear actual and duplicates a
  copy one: a `Slice` handed at an `own` parameter leaves the caller's place and
  its facts standing, which is what lets a view-taking helper be called in a
  loop [CALL-2];
- **every other `&uniq` parameter** — a `&uniq buffer<T>` among them — selects
  no transport and kills the actual's descriptor storage, whatever the callee's
  body does [CALL-5].

So the helper that fills a caller's storage without costing it the length takes
`destination: &uniq MutSlice<u8>`, and the caller forms the view:

```whitefoot
region {
  let window = mut_slice_of(&uniq output);
  region {
    let written = fill(destination: &uniq window, value: 9_u8);
  }
  let still_known = len_of(window);
}
```

**Correction, v0.45.** The pattern used to say the support was `P`'s *root
binding*, and the compiler used to read it that way. That is a strictly larger
support than [MSR-2] states, and it cost a real fact: a write to a **sibling
field** of the same struct killed the measure of a field beside it, so
`set frame.flags = 1_u64;` killed `len_of(frame.tail)`. Descriptor storage is the
place itself, so a sibling-field write now kills neither, and within one body
the length fact survives a write to anything but the run's own descriptor.

**Correction, v0.45 (B3).** This pattern used to say that a callee's write never
killed a caller's length and that the compiler honoured that across a callee
boundary. It honoured it by reading the *argument's spelling*, which is
precisely the selector [CALL-5] removes: a callee that replaced the whole
referent of its `&uniq buffer<u8>` parameter left its caller holding the length
the buffer had before, which is the out-of-bounds heap read
`ent5-neg-callee-uniq-buffer-replace-kills-length` records and which is no
longer an `xfail`. The transport list above is what replaces the claim.

Pattern: bind the length once, above the loop and above every write, and
discharge every later requirement from that one binding.

```whitefoot
let spare = len_of(line);
let fits = end <= spare;
```

**Correction, v0.45.** The binding above was spelled `room` for part of v0.45's
drafting, when the readers were spelled `len`, `cap`, `room` and `head` and all
four were in `ReservedLowerNames`. The readers are spelled `len_of`, `cap_of`,
`room_of` and `head_of` [S36, MSR-1], and the four bare words are ordinary
identifiers again: a reader is a call-shaped measure *of* its operand rather
than a method of a sequence, and `len`, `cap`, `room` and `head` are words a
writer wants for bindings of their own. `let room = ...;` is therefore a legal
declaration once more; what a writer may not declare is `room_of`.

Under v0.45 a measure other than the length is available in the same
positions: `cap_of(P)`, `room_of(P)` and `head_of(P)` are [OP-1] readers and [ENT-2]
terms exactly where `len_of(P)` is [MSR-1], and every measured value carries the
standing facts `len_of(P) <= cap_of(P)`, `head_of(P) <= cap_of(P)` and
`len_of(P) + room_of(P) = cap_of(P)` with no writer statement. A `requires` written
over `cap_of` therefore discharges a subscript stated over `len_of` with nothing
in between.

A measure former is also an **affine factor**, so the relation a clause states
across a call is statable at a loop header or an `invariant_stmt` in the same
spelling [INV-1, GRAM-4]:

```whitefoot
for (
  at in 0_u64..count,
  invariant reserved: written + 4_u64 <= len_of(destination)
) {
```

That is the form a filling loop needs: without it the operation that consumes
the room has no premise on the backedge, and the loop is unwritable rather than
unproved. The factor is an ordinary proof obligation — the checker proves it at
the base and at every backedge — and its image is retargeted by exactly the
writes that kill the measure [MSR-2], so a header conclusion never survives the
statement that refutes it. Only the four measure formers are admitted there;
every other call in an affine position is an [INV-1] rejection naming them.

**Addition, v0.45: a contract clause side is that same affine expression.**
The clause and the invariant now share one production [GRAM-4, GRAM-5, MSR-5],
so what a loop header may state a contract may state too:

```whitefoot
requires at + 2_u64 <= len_of(vector);
ensures len_of(rest) + 1_u64 == len_of(vector);
```

Before this version the `+` was a [GRAM-2] parse rejection at the operator, and
the relation a boundary operation publishes — `len_of(result) = len_of(vector) + 1`
— was unwritable in any source declaration, so no source helper could republish
it and every library function over a run was unstatable. The arithmetic in a
clause side performs no [OP-1] operation and creates no [OP-2] obligation
[MSR-5]: it is a relation over mathematical values, not a computation. What a
*published* relation is narrowed to is smaller than what a `requires` may
carry — one difference bound between two operands displaced by a constant
[FN-9] — so `ensures room_of(rebased) + len_of(vector) >= n;` is refused at the
declaration while the same expression in a `requires` is admitted; write the
two-datum fact as two clauses, or let [MSR-2]'s standing identity supply it.

**An affine atom is one bare local, one literal, or one const generic.** A
capacity-parametric loop states its bound as the parameter itself
[INV-1, MSR-6]:

```whitefoot
for @fill (
  at in 0_u64..n,
  invariant spare: room_of(built) + at >= n
) {
```

The const generic is the constant [ENT-2] clause (c) already fixes, so it needs
no liveness and no support and nothing kills it. Until v0.45 the position was
missing and the loop bound it first with `let limit = n;`; that binding is now
redundant and the direct spelling is the canonical one. A *named* const is
still not an affine atom — it is a tracked place rather than a constant — so a
`const CAP: u64 = 8;` read inside an invariant still needs the binding.

Under v0.44 the same fact is stated directly in the contract
that consumes it, with no binding and no `contract_define` at all: a
`requires` and an `ensures` operand may be a measure of a place [MSR-5], so a
callee writes `requires end <= len_of(destination);` where it used to write
`define room = len_of(destination); requires end <= room;`. The define spelling
of one measure is what v0.44 removes; the hoisted binding above remains the
right form for a *body* fact a loop reads many times, because a body is not a
contract.

The first line sits above the loop and above every `put_text` that writes
through `&uniq line`. The second sits inside the loop after all of them,
and it still discharges `emit_all`'s `requires length <= capacity`, because
nothing between the two killed `len_of(line)`.

Evidence that it compiles as written:
`research/experiments/blind-writer/2026-08-28/probes/probe_e_hoisted_length.wf`
is a whole program in that shape — both length bindings above the loop, above
every `put_text` and every `put_decimal` — and it is accepted.

Current value: the fact is load-bearing, not ceremony. It is the re-bind that
is redundant, and the compiler accepts the re-bind, which is why the belief
survives a whole program: 34 of the 41 length bindings in the five programs of
the 2026-08-28 blind-writer trial existed only to re-establish a fact that had
never died. Without that live fact, the call receives an [FN-8] rejection
because nothing proves the callee's complete requirement. The repair is a
dominating real branch, an already verified contract fact, or a local
`invariant` whose optional written premises the checker can discharge. The
compiler never inserts a callee-side fallback check.

Replaces: defensive re-measurement of a container after every call that wrote
into it, which in a language without a length fact is the only way to be safe.

## P17. Commit the transformed value back into the place it came from

Problem: a recursive walk accumulates counts into a record. Every other
language writes `totals = walk(dir, totals)`. Here that record is affine —
[OWN-1] makes every owned composite affine regardless of its field types, so
three `u64`s in a struct need `move` at every use — and the assignment writes
an affine place, which the language refused outright before [LIV-2].

Pattern: write the assignment. `set p = f(x: move p);` is one commit: the
`move` of the target place is that target's read-out, the target is dead
through the right-hand side, and the same statement reinitializes it, so the
binding is live again afterwards and nothing is duplicated or dropped twice.

```whitefoot
set totals = walk(factory: &uniq deref(factory), directory: dir);
```

The same statement works at a field, at a `deref` of a live usable `&uniq`
holder, and at a subscript, which is what makes it more than sugar for a
rebind: `move p.f` and `move deref(h)` are the two places a two-statement
rebind cannot reach, the first because a partial move kills the root and the
second because content reached through a borrow may not be moved at all.

```whitefoot
set kept.bytes = collect(out: move kept.bytes, source: line);
```

Two targets are one commit when they do not overlap, and the value list is the
swap the language has instead of an exchange operation:

```whitefoot
set (pair.low, pair.high) = split(bound: 4_u64);
set (p, q) = move q, move p;
```

Two subscripts of one run are refused at [LIV-2]'s second condition, because
the commit order would decide the result; write them as two statements.

The subtotal return is still the right shape when the callee does not consume
the value being committed, and the per-field fold is still ordinary: the fields
are `u64`, [OWN-1] copies primitives, and the accumulation never touches the
record as a value.

```whitefoot
let sub = walk(factory: &uniq deref(factory), directory: dir);
set totals.lines = totals.lines +wrap sub.lines;
set totals.bytes = totals.bytes +wrap sub.bytes;
```

`replace` is the commit for the other case, and only in it: when the value
being committed does not consume the target's previous value, `replace` writes
the new owner in and binds the previous one out.

```whitefoot
let stale = replace totals = fresh(lines: 3_u64);
```

Current value: the rejection this pattern used to route around is now exactly
one rule and one sentence. A live affine target whose previous value the
right-hand side does not read out is still [STOR-1]'s error, and its
restructuring is `replace`:

```text
whitefootc: Semantics/Source [STOR-1]: SemanticIssue { rule: Stor1, …, kind: AffineSetTarget
{ target_type: "Cell", mechanical_fix: "use replace: let old = replace p = e; binds the
previous owner" } } at cells.wf:8:7 in line "  set left = move right;"
```

**Addition, v0.45: the same commit over an `own` parameter keeps that
parameter's contract meaning.** A measure of an `own` parameter named in an
`ensures` denotes that parameter's **entry datum** [MSR-3] — a compiler-owned
term with no place in it, which no write and no consume kills — so a body may
write its own parameter back and every clause naming it still means what it
read as at entry:

```whitefoot
fn try_place(vector: own FixedVector<u8, 4>, value: own u8)
    -> (rest: own FixedVector<u8, 4>, unplaced: own Option<u8>)
    reads(vector), writes(vector) contract {
  ensures len_of(rest) <= len_of(vector) + 1_u64;
} {
  set vector = place_back(vector: move vector, value: value);
```

Before this version the `set` killed `len_of(vector)` and the clause was
unproved, so the shape this pattern recommends could not be used in any
function with a contract over the value it commits.

**And a `let` that only renames a measured value keeps its measures.** The same
former stands at the rebind, so `let built = move spare;` carries every measure
`spare` had onto `built`; a rename is not a place a proof falls out of. The
other four naming events — a `construct`, a [LIV-2] `set` target, a `match`
payload binder and a destructuring binder — do not carry them yet, so a
measured value that reaches its new name through one of those arrives with no
measures and the fact has to be re-established from the operation that
published it.

Replaces: a mutable accumulator parameter threaded by reference, `+=` into a
caller's struct, and the three-line temporary-variable swap.

## P18. The loop's own buffer, published once

Problem: a loop reads a file per iteration and writes one line to an output —
`command.stdout`, or another positioned write target — from inside the body.
[PAR-3] denies it: the resource carries one position, so no iteration can be
given its own. (‹loop› is the writer's file and line; verdicts are byte-exact.)

```whitefoot
for @scan (index in 0_u64..8_u64) {
  /* P15's reserve, open, and read; then */
  region {
    let written = write_once(output: &uniq out, source: &data, start: 0_u64, end: 2_u64);
  }
}
```

```text
PAR stage  ‹loop›  for  denied  condition 3: a may-suspend call retains a borrow past its own submission
           on storage the body writes and the iteration does not introduce; instead, give each iteration
           its own resource; or, where the body only publishes to that storage — an output stream is the
           pointed case — hoist the per-iteration write out of the loop, folding a total in the body and
           writing it once after the loop; or leave this loop sequential, because storage that carries
           one position cannot be held by two iterations at once, at &uniq out
```

Pattern: **hold no `&uniq` resource inside a loop body; hold a buffer instead,
and publish it once after the loop.** Each iteration folds its line into the
loop's own buffer with an element `set` under one length fact above the loop
(P16); that element is `serialized-E`, the remainder's writes to storage
outside the loop being taken in index order.

```whitefoot
let page = buffer_new(8_u64, 0_u8);
let spare = len_of(page);
for @scan (index in 0_u64..8_u64) {
  /* reserve, open, and read into the iteration's own data buffer */
  let writable = index < spare;
  if writable {
    set page[index] = data[0_u64];
  }
}
region {
  let written = write_once(output: &uniq out, source: &page, start: 0_u64, end: 8_u64);
}
```

```text
PAR stage  ‹loop›  for  permitted  staged at open_file(permit: move permit, root: &cwd,
           name: &name, start: 0_u64, end: 2_u64); 6 places classified
```

Write into the page by element. A helper taking `&uniq` of it costs the loop
its pipeline under condition 4 instead: `denied  &uniq page  a call of
the remainder holds an exclusive loan on it, and two remainders coexist`.

Current value: this is the explicit writer form. The [PAR-3] judgment can grant
it, but the current fixed two-slot actualizer does not cover the shown
remainder because it reads the counted binder and per-iteration data while
writing the enclosing page. The loop therefore keeps ordinary lowering today.
The implicit alternative, a per-iteration output write committed by a final
ordered stage, is also not implemented. Output accumulated through this pattern
appears only when the one write after the loop runs.

Replaces: writing each line where it is produced, as every other language does.

## P19. Advance a tracked binding the same way on every arm

Problem: a loop header states an invariant over a binding the body updates only
under a condition — a cursor advanced while input remains, an accumulator folded
on the matching half, two cursors merged in one loop. The relation is true on
every execution, and the loop is still rejected at [INV-1].

The rule, in one line: every arm of a body join must leave a tracked binding
with the same affine image, or with images differing only by a constant;
otherwise the guard fact dies at the join and the header invariant cannot be
re-established. [ENT-6]'s value-image join keeps an identical image; otherwise
it normalizes each input by folding every delta atom an earlier join minted
back into the constant interval it stands for, and where the inputs then share one
nonconstant form it joins them to that form plus a fresh delta atom over the
hull of their constant intervals. Every other combination gives the binding one
fresh full-type atom. Because of that normalization the join is associative:
one branch set written as nested `if`/`else` reaches exactly the image the same
set written as one flat `match` reaches, so nesting the arms never costs a
binding its image and the shapes below may be nested freely. [ENT-5]'s
all-predecessor join then keeps only the bounds held on every input, so the
correlation the writer is reasoning with — the delta is one exactly where
`i < n` held — is precisely what the join discards, and [INV-1] proves the next
header target over what is left. No source spelling recovers it: a per-arm or
tail `invariant` restating the target is not canonically identical across the
arms, so none of those conclusions survives the join either.

Three body shapes are accepted. The first applies the identical update on every
arm; only the image is compared, so the condition may be entirely
data-dependent:

```whitefoot
    if is_odd {
      set even_sum = even_sum + wide;
    } else {
      set even_sum = even_sum + wide;
    }
```

The second adds a different constant on each arm. The images share their
coefficient vector, so they join to that vector plus a delta atom over the
incoming constants:

```whitefoot
    if is_odd {
      set even_sum = even_sum + 7_u32;
    } else {
      set even_sum = even_sum + 200_u32;
    }
```

The third lifts the choice out of the control join and into the addend. The
`value_if` delivers one owned value, the body applies one unconditional update,
and a local invariant bounds the addend, so the header target is re-established
over that single image:

```whitefoot
    let addend = if is_odd {
      give zero;
    } else {
      give wide;
    }
    invariant addend_bound: addend <= 255_u32;
    set sum = sum + addend;
```

Where the update itself must stay conditional, re-expose the fact after the
join with a dominating guard. The guarded write may lose the relation; one real
branch on the joined value restores it for the next header, and the false edge
is an ordinary result (P12):

```whitefoot
    if shrink {
      set hi = candidate;
    }
    if hi <= spare {
    } else {
      return 0_u64;
    }
```

That one guard is enough for a whole binary search: `hi` narrows on one arm of
the three-way comparison and `lo` on another, re-testing `hi <= spare` at the
end of the body restores the header invariant on every path, and the midpoint's
own two-premise `mid < hi` certificate (P14) then carries that bound down to
the subscript, which needs no guard of its own. The
other repair is to remove the join: where the non-advancing arm is a real result
— `break`, `return`, a typed error — there is no join to weaken and the guarded
increment keeps its fact (P8, P12).

Evidence that both verdicts are the language's:
`tests/conformance/cases/ent6-neg-join-one-arm-advances-accumulator.wf` and
`tests/conformance/cases/ent6-pos-join-value-if-lifted-addend.wf` are the same
fold before and after the lift, rejected at [INV-1] and run to exit 0.

Current value: the rejections are mandatory, not a checker weakness, so no
amount of restating helps and the repair is always structural. One shape has no
route today — two cursors merged in one loop, each advanced on its own arm,
where neither the lift nor a dominating guard applies because the header
invariant is on the cursor the other arm did not move. That is a specification
question, not a ticket: admitting it needs a way to keep a per-edge published
conclusion attached to the delta-atom join.

Replaces: the reflex of restating the invariant inside each arm, and the belief
that a relation true on every execution is therefore provable at a join the
language deliberately does not make path-sensitive.

## P20. The loop body is already the region

Problem: a borrow taken inside a loop body must die with the iteration, and the
reflex — carried over from every earlier version — is to wrap the body in a
`region` block so that it does.

Pattern: write the borrow bare. Under v0.43 every `loop_stmt` and `for_stmt`
body is itself a region block: it introduces one unnamed region whose
block is that body, so a borrow written directly in the body takes that region,
dies with the iteration, and lets the outer binding be written again before the
next one [OWN-11]. That is exactly the guarantee the wrapper used to buy, and it
now costs nothing to write.

```whitefoot
for @concat (at in 0_u64..count) {
  match bs_byte(s: &deref(source), index: at) {
    Some(value: byte) => {
      region {
        bs_push(s: &uniq deref(destination), value: byte);
      }
    }
    None() => {
    }
  }
}
```

The inner block stays: it is not the loop body's only statement, it is the
statement scope [OWN-6] needs for the `&uniq deref(destination)` child
reborrow, and only the outer wrapper the body no longer needs is gone.

Because that region exists, a `region` block that is the loop body's only
statement is now a hard error citing [FORM-8]: its block is the body, so it is a
second spelling of one region. Delete it and keep its statements as the body.

A block the body writes another statement beside is a different region — it ends
strictly earlier than the iteration — and stays legal, and there are two reasons
to write one. The first is [OWN-6]: a statement-scoped child reborrow needs a
region whose block does not extend beyond the enclosing statement, which a
one-statement block inside a longer body gives and the body's own region does
not. The second is a borrow that must be dead before a later statement of the
same iteration writes the place it borrowed.

```whitefoot
for @append (i in 0_u64..count) {
  let byte = deref(src)[i];
  region {
    let pushed = propagate vec_push(v: &uniq deref(dst), x: byte);
  }
}
```

Decide by reading the loop body alone: if the body writes nothing beside the
block, the block is the body's own region under a second name and must go.

Current value: mechanical. The corpus rewrite for v0.43 removed four
such blocks across `tests/` and touched nothing else, because most existing
blocks were already narrower than their bodies.

Replaces: the habit of opening a `region` block as the first line of every loop
body.

## P21. Hand the measure back by value, and never through a `&uniq`

Status: active in v0.44. Two of its rules decide one writer choice together.

Problem: a helper receives a run, does something with it, and the caller
afterwards needs to know a measure of what it got back. The reflex is to lend
the run — `fn fill(destination: &uniq buffer<u8>, ...)` — and publish the
measure from the callee: `ensures written <= len_of(deref(destination));`. That
clause is a claim about a caller's object at a point the callee cannot name,
because the callee may have replaced the very thing the measure describes, and
[MSR-3] refuses it at the clause.

Pattern: take the value by value and relate the *result*; state the fact the
caller must supply as a `requires` instead of an `ensures`.

```whitefoot
fn size_of(taken: own buffer<u8>) -> measured: own u64 reads(taken) contract {
  ensures measured == len_of(taken);
} { ... }

let run = buffer_new(8_u64, 0_u8);
let measured = size_of(taken: move run);
```

The relation reads at the caller as it is written, and it survives the `move`
in the very same statement, because an `own` operand of a published relation
denotes that call's **call datum** [MSR-3] — the value at transfer, a term
with no place in it, which no consume and no later write can kill. That is the
whole reason the value-in form is not merely tolerated but preferred: a
borrowed run's measure cannot be published at all, and a consumed run's can.

The second half is a discipline on the contract as a set. Everything a
contract publishes is closed together at the caller [ENT-4], so two clauses
that cannot both hold do not make one caller goal wrong — they make every
caller goal discharge. [CALL-6] refuses such a contract at its declaration, so
write the clauses that hold and check that they hold together; a clause added
"to be safe" that contradicts an earlier one is not conservative, it is the
end of every proof downstream of the call.

Current value: measured on the v0.44 batch branch. The `ensures` over a consumed
operand's measure is a real fact at the caller, where before v0.44 the
consume in the same statement deleted it and the caller was left proving the
measure again from its own allocation.

Replaces: publishing a caller's post-state through a `&uniq` parameter, and
re-measuring a run the caller just handed away.

## P22. Write `linear` for a logical obligation, and never for a storage one

Status: active in v0.45. [PROV-6] states the criterion, the modifier and the
two forms that discharge it.

Problem: the writer wants a value that cannot be dropped by accident. The
reflex is to reach for a marker on every owning type, and the reflex is wrong
twice over: the storage obligation is already derived, and marking it costs a
written statement at every scope exit of every value of that type, including
in code the writer does not own.

Pattern: mark nothing whose only cost of being dropped is memory. A run backed
by a store is reclaimed by the compiler-derived release at every leaving edge
[STOR-3, LIV-1], a view owns nothing, and any type that owns a marked value is
linear by ownership without being marked itself. **Marking a store-derived
type is always redundant and is a sign the criterion has been misread.** The
modifier is for a *logical* obligation, and the whole test is one question:
**would silently dropping this value be a bug?** The shapes that pass it are a
lease from a pool, a transaction that must commit or roll back, a request that
must be answered, a counted permit or ticket, and a builder that must be
finished.

```whitefoot
linear struct Lease {
  slot: u8;
}

fn hand_back(lease: own Lease) -> returned: own Lease pure {
  return move lease;
}
```

What the modifier buys is one sentence: it makes a discard **visible and
deliberate**. The value must be moved out whole or destructured whole, and a
destructuring is a legal consume that can throw the contents away — so a
*directional* obligation, where the value must reach a specific holder, is
bought by proving the return, not by the marker. Write the library's return
operation as the proved spelling — total, under a `requires` the caller
discharges from the take's own published relation — and the value has exactly
one route on every path; the modifier is the visibility insurance beside that
proof rather than a substitute for it.

The admission condition is [PROV-6]'s: an affine nominal only. `linear` on a
tag-only enum is a hard error, because such an enum is copy [OWN-1] and the
marker would name a value the language duplicates on every use.

Replaces: a marker on every owning type, and a comment asking the next writer
to remember to use a value.

## P23. Take the whole value apart in one statement

Status: active in v0.45. [PROV-6] adds the form and states the refusal it
repairs.

Problem: the writer needs one field out of a value that must not be silently
dropped. `let page = move chunk.page;` is a partial consume: [OWN-1] kills the
whole root, and the residual — every other field — is abandoned in a scope
that has no derived release to reclaim it. [PROV-6] refuses it and names the
residual.

Pattern: consume the whole value in one statement and bind every field.

```whitefoot
let Chunk(page: page, spare: spare) = move chunk;
dispose page;
```

Every declared field is written exactly once, in declared order, as
`field: binder`, exactly as a `match` arm writes its payload binders; each
binder receives that field's declared type and `own` mode, and the binders are
ordinary own bindings of the enclosing block. No residual survives the
statement, so it derives no release of the consumed value's own storage.

The one shape that is *not* a partial consume is the commit that puts a value
back: `set chunk.page = exchange(taken: move chunk.page);` reads the target
out and reinitialises it at the same statement's one commit [LIV-2], so it
leaves no residual and the refusal does not reach it. That is the difference
between transforming a component in place and abandoning the rest of the
value.

Replaces: a field-by-field sequence of moves out of a value whose first move
already killed the root.

## P24. `dispose` is the early release, not a free

Status: active in v0.45. [PROV-6] states the statement and its admission.

Problem: a value backed by a store stays alive to the end of its scope, and
the scope is sometimes the whole program. Reserving a second run while the
first is still live doubles the peak; a loop whose scope is the entry function
holds every run it ever built.

Pattern: run the release where the value stops being needed.

```whitefoot
let run = buffer_new(4_u64, 0_u8);
let first = run[0_u64];
dispose run;
```

`dispose p;` runs at the point it is written exactly the walk the scope exit
would have run for `p`, and it names no capability: the store is determined by
the value's own type and is never written. It is one consuming use of `p`'s
root, so `p` must be rooted in a live own-mode binding of this function —
content reached through a borrow may never be moved and this statement is no
exception — and it exhibits one write of `p`'s ultimate storage origin, so the
release a writer chooses appears in the effect row where the derived one does
not.

Three shapes it refuses, each for its own reason. A value whose release graph
reaches no capability-released leaf has nothing to reclaim early; let the
scope exit run it. A view owns nothing and has no release action of its own;
release the value it views. And a value one of whose release-graph nodes
carries the `linear` modifier must be taken apart with P23 first, so the
marked component reaches a written statement rather than a silent walk.

Do not reach for it by default. The derived release is correct and free; this
is the statement for the one place where the peak is the point.

Replaces: holding a value to the end of its scope because there was no way to
say otherwise.

## P25. Write a store's region only where it relates two positions

Problem: the two runs and the two providers carry the store that backs them in
their own types [PROV-1], so every one of them names a region. Writing that
region everywhere would put a region parameter on every hosted nominal and
every hosted signature, and the design counted fifteen brand occurrences and
twelve call-site brand arguments in one byte-string program before it stopped.

Pattern: write the region exactly where it relates two positions, and elide it
everywhere else. An elided store brand at a field, an enum payload, a run
element, or a written type argument denotes the enclosing nominal's sole region
parameter when it declares one, and the entry heap's store region otherwise; at
a parameter or a result it denotes the entry heap's store region. So a nominal
over the one general store declares no region and writes none:

```whitefoot
struct Bytes {
  v: Vector<u8>;
}
```

and a nominal over a bump extent declares exactly one and writes it at the one
field that must be branded to it:

```whitefoot
struct Chunk['s] {
  page: Vector<'s, u8>;
}
```

The rule is the ordinary [FORM-8] discipline read over stores: a region a
reader cannot check and a transposition cannot catch is deleted, and a region
the caller must choose is written. A bump extent's own region is always the
second kind, so an `Arena` writes it at every position: `Arena<4096, 16>` is a
[FORM-8] rejection and `Arena<'s, 4096, 16>` is the form.

Replaces: putting a region parameter on every declaration that touches a run.

## P26. Reserve the extent in the outer block and take inside an inner one

Problem: a bump extent is reserved by naming its own region —
`arena_frame::<4096, 16, 'a>()` — so the binding that holds the provider is
declared *inside* `'a`'s block. A take borrows that provider, and the borrow's
elided region is the innermost enclosing one, which is `'a` itself. [OWN-10]
refuses it: `'a` is introduced outside the binding it would borrow, so a loan
living for `'a` could outlive the storage.

Pattern: reserve in the named block and take inside a nested unnamed one, with
the run's uses inside it too:

```whitefoot
region 'a {
  let workspace = arena_frame::<256, 8, 'a>();
  region {
    let page = arena_vector_proved::<u64>(store: &uniq workspace, count: 4_u64);
    let one = place_back(vector: move page, value: 11_u64);
  }
}
```

This is the same `region { call(&uniq local) }` shape a `&uniq` argument
already takes anywhere else; what is particular to a store is that the runs it
hands out are used inside the inner block as well, because the binding that
holds one dies at that block's exit. That costs nothing: an arena-backed run's
release action is empty, its storage being the extent's [PROV-6].

Two takes share one inner block. A second reservation for the same region is a
[PROV-1] rejection — one region names one store — so a program that wants two
extents opens two region blocks.

Replaces: keeping every take at the reservation. A helper generic over its
store is spellable now — a parameter type naming a formal region determines
that region from its actual — so `fn carve['s: affine](store: &uniq Arena<'s,
256, 16>) -> made: own Option<Vector<'s, u64>>` is a legal declaration and the
`region { ... }` above is what the caller writes around the call.

## P27. Choose a type parameter's bound from what the body does with the value

A type parameter carries exactly one bound, always written, never inferred,
with no default. Read it off the body, not off the types you expect to
instantiate at:

| the body ...                                    | write     |
|-------------------------------------------------|-----------|
| uses the value bare, or more than once           | `T: copy` |
| writes `move value` and may let it reach an exit | `T: affine` |
| must hand the value on, and may never drop it    | `T: linear` |
| does integer arithmetic on it                    | `T: Int`  |
| does float arithmetic on it                      | `T: Float`|

The three linearity classes form the chain `copy < affine < linear`, and
satisfaction is that chain read left to right: an argument of class C
instantiates a bound B exactly when `C <= B`. So the bound is a **ceiling on
what the body assumes**, not a claim about the argument. `T: linear` accepts
every type; `T: affine` accepts copy and affine; `T: copy` accepts copy alone.
Writing a tighter bound than the body needs is the mistake this table exists to
prevent — `filled` writes `T: copy` because it uses `value` bare in a loop, and
`try_place` writes `T: affine` because it writes `move value`, and neither
would gain anything from a tighter one.

The bound is also what the body is *checked* under, once, and the concrete
instances do not re-judge its spelling. That is why one `affine`-bounded body
serves `u8` and `Option<u8>`: at `u8` the `move` denotes a copy. So write
`move` wherever the body needs the affine discipline and do not split the
function per element class.

`Int` and `Float` are the two prelude markers and each implies `copy`, so a
numeric body needs no second bound; a source contract is not a bound at all
[FN-3]. A `const` parameter carries none.

A region parameter's bound is optional and means something else: `'s: affine`
declares that `'s` names a bump extent and `'s: linear` that it names a general
store, while an unbounded `['s]` is any region at all, a loan region included,
and a body that assumes no store. A region argument that names no store — a
loan region, or a `region { }` region no reserving occurrence names — satisfies
neither bound, so write the bound only on a region a store is actually reserved
in.

Replaces: two functions with two signatures where one body would do, and
`let limit = ...;`-style workarounds for a bound a declaration could not state.

## P28. Take a run out of a run before you read it

A run's element type may itself be a run — `FixedVector<Vector<'s, u8>, 8>` is
a free list of eight store-backed blocks, and `FixedVector<FixedVector<u8, 4>,
4>` is a fixed grid — and the slot holds the element run's complete
representation, its descriptor words included.

An element that is a run is **affine**, and that decides how you read one:

```whitefoot
let (rest, block) = take_back(vector: move free);   // the element comes out
let width = cap_of(block);                          // and is read there
let back = place_back(vector: move rest, value: move block);
```

A bare `free[0_u64]` is [OWN-1]'s ordinary refusal at an affine element, exactly
as it is for any other affine element type, so the two routes are the boundary
rows [BLK-3] and the element-position exchange `let old = replace free[i] = e;`
[SET-2]. Reach for `take_back` and `place_back` first: a free list is used at
one end, and the pair is total under `room_of > 0` and `len_of > 0`.

A helper over such a run is generic over the store the *elements* live in, and
[FORM-8] writes that region nowhere at the call: the parameter type names it one
level down, in the element position, and the actual determines it.

```whitefoot
fn pool_take['s: affine](free: own FixedVector<Vector<'s, u8>, 8>)
    -> (rest: own FixedVector<Vector<'s, u8>, 8>, leased: own Option<Vector<'s, u8>>)
```

A measure of an element is an ordinary term — `len_of(free[i])`, `cap_of(free[i])`
— so a figure about one slot is read in place and the element does not have to
come out for it:

```whitefoot
let rows = len_of(grid);
if rows > 0_u64 {
  let width = len_of(grid[0_u64]);          // the element's own descriptor
  let cell = grid[0_u64][0_u64];            // and, for a copy element, its slot
}
```

Three things follow from that and are worth knowing before you design around
them. The subscript inside the place owes the same `i < len_of(base)` every
written subscript owes, so the branch above is what pays for it. Its offset must
be one the rules can name — a written literal, a live `own u64` binding, or a
const generic — because two such places are told apart by their offsets; an
offset a call computes is not a place this version represents. And a write at
one slot kills that slot's measures and none of the run's own, so a
`replace grid[i] = e;` costs you `len_of(grid[i])` and leaves `len_of(grid)`
standing.

A measured value **written at a slot keeps its figure there, and the value the
same slot hands back inherits it** — that is one of [MSR-3]'s placements, and it
reaches exactly the two routes that name the position:

```whitefoot
let spare = replace free[0_u64] = move fresh;   // free[0]'s measures are fresh's
let held = replace free[0_u64] = move other;    // held's measures are fresh's
let filled = place_back(vector: move held, value: 3_u8);   // and this discharges
```

The boundary rows are the exception, and they are the common case, so plan for
it: `place_back` puts its value at position `len_of(vector)` and `take_back`
takes one from position `len_of(rest)`, and neither is an offset the place rules
can name. **A block pushed onto a free list with `place_back` and leased off it
with `take_back` therefore comes back with no measures of its own**, and a
caller that needs its room reads `room_of` once and branches:

```whitefoot
let (rest, block) = take_back(vector: move free);
let spare = room_of(block);
if spare > 0_u64 {
  let filled = place_back(vector: move block, value: 7_u8);
}
```

That branch is not a workaround for a missing check; it is the honest price of a
capacity that lives in a descriptor rather than in a type. Reach for
`replace free[i] = e` when the position is written and you want the figure to
travel, and for the boundary rows when you want the end of the run.

One limit remains: one level of *element* is what exists, so a run of runs of
runs is an unsupported capability.

Replaces: an `Option<T>` slot array standing in for a run of runs, a parallel
array of lengths beside a run of buffers, and a hand-written `cap` field beside
every leased block.

## P29. Give a nominal the store its contents live in

A struct or enum that holds a store-backed value must name that store, and it
names it the way every other type does: as a region parameter that becomes a
component of its type name.

```whitefoot
linear struct Lease['s] {
  run: Vector<'s, u8>;
}

struct BlockPool['s] {
  free: FixedVector<Vector<'s, u8>, 8>;
}
```

Write the region argument at every `type` position, as the leading member of
the same `<...>` list a type argument goes in: `BlockPool<'a>` names the type
and `Some<BlockPool<'a>>(value: move pool)` carries one, because `Option`'s own
argument is a type position.

At a `construct` you write only the region parameters **no field determines**.
A field determines one exactly when its declared type names it, which is the
same relation a parameter position bears at a call, so `BlockPool`'s `free :
FixedVector<Vector<'s, u8>, 8>` fixes `'s` from its operand and the construct
writes nothing:

```whitefoot
let pool = BlockPool(free: move free);
let ticket = Lease(run: move one);
```

Writing it anyway — `BlockPool<'a>(free: move free)` — is a [FORM-8] rejection
whose fix is to drop the argument. A nominal none of whose fields names its
region has nothing to determine it, so the construct writes it after all:

```whitefoot
struct Ticket['s] {
  count: u64;
}

let one = Ticket<'a>(count: 7_u64);
```

Construction still consults no expected type: the field operands and the
written members are the only supply there is, and never a destination.

At a call you write **nothing**: a parameter whose type names the nominal's
region determines it from the actual, exactly as `Vector<'s, T>` does.

```whitefoot
fn pool_take['s: affine](pool: own BlockPool<'s>)
    -> (rest: own BlockPool<'s>, leased: own Option<Lease<'s>>)
```

Two instances at two regions are two types, and a store region is invariant: a
formal region occupying two parameter positions is fixed by the first actual, so
one function cannot be handed a pool of one arena and a lease of another.

`linear` on such a nominal is what buys must-return: a path that neither returns
the lease nor takes it apart with `let Lease(run: back) = move lease;` is
refused, which is how a pool gets its blocks back.

**A measured field keeps its figure across the wrapper, in both directions.**
The construct carries what the operand had into the field, and the destructuring
consume carries it back out to the binder that names the field, so a block does
not lose its room by being put in a `Lease` and taken out again:

```whitefoot
let ticket = Lease(run: move block);          // lease.run has block's measures
let Lease(run: back) = move ticket;           // and back has lease.run's
let filled = place_back(vector: move back, value: 7_u8);
```

The same holds through an enum whose nominal has **one** payload-carrying
variant — `Option` is one — so `Some<Vector<'a, u8>>(value: move block)` and the
`Some(value: back)` arm that consumes it are the same pair. `Result` is not one:
its `Ok(value)` and `Err(error)` are two storages one field path cannot tell
apart, so a payload of a `Result` arrives with no measures and a caller that
needs one reads it and branches.

Two shapes to design around. A loop that allocates from a `&uniq` store
parameter has **one statement per iteration** — a child reborrow's region cannot
extend beyond its own statement, so write the loop body as one `match` over the
acquiring call itself rather than a `let` and then a `match` on its binding. And
a contract clause may name a measure of a **parameter**'s field
(`requires room_of(pool.free) > 0_u64;`) but not of a *result*'s, so a caller
that needs a figure about a returned nominal reads it and branches.

Replaces: threading the bare container the struct would have held through every
signature, and a store-blind wrapper that cannot say which arena its contents
came from.

## P30. Swap two elements in one commit

Two elements of one run exchange slots in a single `set`, and the read-out is
what makes it legal:

```whitefoot
set (v[0_u64], v[1_u64]) = move v[1_u64], move v[0_u64];
```

Each `move` is the read-out of the target whose offset it names, so the affine
element leaves its slot and the same statement's one commit fills it again; no
program point between them sees a slot empty.

The offsets must be **written literals with unequal values**. That is the whole
of what the rule can decide: two targets whose offsets it cannot tell apart
overlap and are refused, and a `move v[i]` whose offset does not provably match
its target's reads nothing out, which leaves the live affine target its ordinary
refusal. For an offset a loop computes, take the elements out with `take_back`
and put them back (P28), or exchange one at a time with
`let old = replace v[i] = e;`.

Two targets of a run of runs are compared over their **complete paths**, first
step first, so `grid[0][1]` and `grid[1][1]` are two storages even though their
last offsets agree:

```whitefoot
set (grid[0_u64][1_u64], grid[1_u64][1_u64]) = 9_u8, 8_u8;
```

Write the offset that distinguishes them as early in the path as you can: two
targets that agree at every decidable step overlap, however their later steps
read.

Replaces: `take_back` / `replace` / `place_back` for a swap of two known
positions, and an `Option<T>` slot standing in for a temporarily empty one.

## P31. Write through a view with `mut_slice_of`, read with `slice_of`

There are two views, and the one you form says what you may do through it:

```whitefoot
let window = mut_slice_of(&uniq buffer);
set window[0_u64] = 9_u8;
let seen = window[0_u64];
```

`slice_of` hands back `Slice<'r, T>`, which reads its range; `mut_slice_of`
hands back `MutSlice<'r, T>`, which reads it and writes its elements. A
writable target path traverses a view exactly at the exclusive strength
[SET-1], so the same `set` through a `Slice` is a SET-1 rejection whose
diagnostic names the shared view — the fix is to form the view with the other
row, not to borrow the descriptor uniquely, which grants nothing over the
viewed storage.

**Only one exclusive view of a place may be live.** The formation itself takes
the borrow its strength names, so a second `mut_slice_of` over one place meets
the first view's loan and is refused at the second formation as an ordinary
[OWN-5] conflict; two `slice_of` views of one place are admitted without limit
and read the same elements. The refusal is reported where the second view is
formed, so a program that wants two writable windows wants one view and two
offsets, not two views.

A named const is a legal `slice_of` source and never a `mut_slice_of` source:
its storage is permanently read-only [CONST-2], and the rejection is at the
operand.

**A `Slice` is copy: use it bare, and its loan ends at its last use.** Write
`total(window: window)` and not `total(window: move window)` — a `move` of one
is the ordinary [OWN-1] `MoveOfCopy` — and the same view may be handed to two
calls and read afterwards. What the classification buys back is the run: the
storage a shared view reaches is writable again after that view's **last use**,
inside the same region block, so

```whitefoot
let window = slice_of(&run);
let sum = total(window: window);
let longer = place_back(vector: move run, value: 9_u8);
```

compiles, while moving the `place_back` above the `total` call does not. A
`MutSlice` stays affine and is moved as any other affine value is.

**A view is bound once and never committed at.** `set view = other;` and
`let old = replace view = other;` are both refused: the displaced view's loan
would outlive the descriptor whose place it was held from [VIEW-4]. Bind a new
view under a new `let` instead.

**Read a run through a view, and drain it first if its window wrapped.** The two
runs are viewable, and the formation carries the requirement that the window
does not wrap: `head_of(vector) <= room_of(vector)`. A run only ever appended to
and taken from the back satisfies it, and so does one drained to empty; a run
that has had a front removal and been refilled does not, and the formation is
refused citing [BLK-0] with the goal it could not discharge. The repair is
3.L.8's drain — take the window front-to-back into a fresh run — and not a
second view.

**A shared view of a place an exclusive view already views is that view's
child.** It is admitted, it reads the same range, and it freezes its parent
while it lives: an element write through the `MutSlice` is refused until the
child's last use, and admitted after it. That is how a reader and a writer of
one buffer are spelled without two exclusive views.

**View a `buffer<T>` or a `Vector<'s, T>`, not an `array<T, N>` or a
`FixedVector<T, n>`, when you mean to write.** Inline storage is a value in this
compiler — an element commit rebuilds it and writes it back to its binding — so
a view of one carries a snapshot, and an exclusive view over inline storage
stops as an explicit unsupported capability rather than writing where nobody can
see it. A shared view is unaffected, because a live shared loan refuses every
write to that storage while the view can be read.

Replaces: taking a run or a buffer by value in order to write it, passing a
`&uniq buffer<T>` where the callee only needs a window, and the `Option<T>`
slot that stood in for a writable view.

## P32. Pass the destination on, and hand its reader back as the child

Status: active in v0.45 (B7c4b-1). Two forms a helper handed a writable view
could not write until this batch.

Problem: a decoder's output destination is threaded three frames deep, and a
helper that fills a destination usually also wants to publish what it filled.
Both were refused: a helper handed `&uniq MutSlice<'r, u8>` could not pass that
destination to a second helper, and could not form the shared view a reader —
or a `write_once` source — needs.

Pattern: re-lend with `&uniq deref(destination)` and publish with
`slice_of(&'r deref(destination))`.

```whitefoot
fn outer(destination: &uniq MutSlice<u8>, value: own u8) -> written: own u64
    writes(destination) contract {
  requires 2_u64 <= len_of(deref(destination));
} {
  region {
    let count = inner(destination: &uniq deref(destination), value: value);
  }
  return 2_u64;
}

fn fill_and_publish['r](destination: &uniq MutSlice<'r, u8>, value: own u8)
    -> filled: own Slice<'r, u8> writes(destination) contract {
  requires 2_u64 <= len_of(deref(destination));
} {
  set deref(destination)[0_u64] = value;
  set deref(destination)[1_u64] = value;
  return slice_of(&'r deref(destination));
}
```

The re-lend is [OWN-6]'s ordinary child reborrow: it lives for its statement,
the holder is suspended while it does, and the inner callee's write is
classified over the viewed range [CALL-3], so the outer helper's own
requirement still stands after the call. The publish is [OWN-6]'s *shared*
child of an exclusive loan applied to a view [VIEW-2]: the child carries the
parent's range and origin set, its region is the one the operand borrow writes
and the parent's own region must outlive it, and while the child lives the
parent may not write the elements it views — at the caller too, which is what
makes the returned child safe to read.

Two things this does not buy. The child a borrowed view holder can form is
**shared** and nothing else, so a helper cannot hand out a second writable
window; and the ceiling half that admits the result is a shared result only, so
`-> own MutSlice<'r, u8>` from a borrowed holder is still refused [VIEW-6].

Replaces: the hand-the-length-back workaround two diagnostics helpers took in
B3, and the `&uniq buffer<u8>` spelling a chained output destination kept.

## P33. A full fixed run of literals is a `const`

Status: active in v0.45 (B7c4b-1).

Problem: a lookup table, a test vector, a message — a run of `n` literal
elements the program only ever reads — was written either as a `const` of the
retiring `array<T, N>` or built at run time with `n` appends and the invariants
that proof needs.

Pattern: write it as a `const` of the inline run.

```whitefoot
const digit_glyphs: FixedVector<u8, 10> =[48_u8, 49_u8, 50_u8, 51_u8, 52_u8,
  53_u8, 54_u8, 55_u8, 56_u8, 57_u8];
```

The entry count is the type's own `n`, and `len_of = cap_of = n`,
`room_of = head_of = Z` are standing facts of the type rather than stored
words: the item lowers to element storage only, a subscript's bound discharges
from `len_of = n` with no invariant to write, and all four readers answer from
the type. `slice_of(&table)` gives the `immutable-const` origin, so the const
travels into any consumer that takes a `Slice`; `mut_slice_of` over it and a
`set` through it are the two refusals a const has always had [CONST-2].

Replaces: a `const` of `array<T, N>`, and the counted `place_back` fill of a
run whose contents are literals.
## P34. Read a stream to its end, and publish through a helper

The stream operations `read_next` and `receive_next` have no offset: the
position they advance is the stream's own [SYS-15, SYS-18]. A read to end is
therefore an uncounted loop whose only exit is the `ReadEnd` arm, and the run
it reads into is the loop's own — one store-resident `Vector` filled once
before the loop, whose length the loop reads back as `held`.

```whitefoot
let held = len_of(chunk);
loop @chunks {
  let available = 0_u64;
  let ended = 0_u8;
  region {
    let sink = mut_slice_of(&uniq chunk);
    region {
      match read_next(input: &uniq input, destination: &uniq sink,
                      start: 0_u64, end: held) {
        ReadBytes(next: endpoint) => { set available = endpoint; }
        ReadEnd() => { set ended = 1_u8; }
        ReadFailed(error: problem) => { set outcome = 3_u8; set ended = 2_u8; }
      }
    }
  }
  if ended == 0_u8 {
    region {
      let payload = slice_of(&chunk);
      region {
        match publish_all(output: &uniq out, source: &payload, length: available) {
          Ok(value: published) => { }
          Err(error: problem) => { set outcome = 4_u8; set ended = 2_u8; }
        }
      }
    }
  }
  if ended == 0_u8 { } else { break @chunks; }
}
```

Three rules make it work, and all three are forms to copy:

- **One run, two views, one at a time.** The read needs the exclusive view and
  the publish needs the shared one, and a run is viewed one way at a time
  [VIEW-2, OWN-5]: a shared child of a live `MutSlice` freezes the parent for
  as long as it lives, and a second exclusive view of a live one is refused
  outright. Giving each view a region of its own — sibling regions, not nested
  — ends each loan at the brace before the other view forms, so the loop body
  reads and publishes the same storage without either view ever seeing the
  other.
- **Publish through the helper of P16's shape, not through a second inner
  loop.** `write_once` over the same run the read filled needs
  `available <= len_of(payload)` at its call site. The read's own [SYS-8]
  relation gives `available <= held` on the `ReadBytes` edge, and both views
  carry their origin's length, so `len_of(sink) == len_of(payload) == held`;
  the facts are live immediately after the read region and neither survives a
  second loop header. A helper whose contract states
  `requires length <= len_of(deref(source))` moves the obligation to that one
  live point, and the writer never restates the bound. Reading `held` once,
  before the loop, is what lets the bound be the run's own length rather than a
  literal the writer must keep in step with the allocation.
- **Leave through one flag, at the bottom.** The `ReadEnd` break is selected
  by the submission's own outcome, so it can never be taken in a staged
  prologue and the loop stays sequential ([PAR-3] says so in as many words).
  Writing the exit as a flag the body sets and one `break` at the end keeps the
  failure and the end on the same edge and keeps the body's shape readable.

Current value: measured on the v0.46 batch branch, on `tests/programs/stdin_echo.wf`
against a pipe and against a redirected file, on both runtime routes; carried
onto the view forms with the same two shapes and the same two routes.

Replaces: a positioned read with a writer-tracked offset, which is the wrong
operation for a stream, and an inner publish loop, which loses the length
fact the read left behind.

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
