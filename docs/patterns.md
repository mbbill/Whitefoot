# Whitefoot Pattern Doctrine (D6)

Status: seeded, non-normative writer guidance since 2026-07-09. Only the active
specification defines accepted source. The selected D6 direction is to test
whether a closed architecture-level vocabulary can stay COMPLETE (every task
modelable — a gap is a finding) and EFFICIENT (each pattern names the fact
channel or machine property that makes it fast) before normative adoption.
Writers may be taught this catalog during validation; hitting a wall is a
catalog finding, not authority to invent a language rule.

This document carries active v0.41 guidance, including the comparison symbols
and call-site `::` delimiter v0.41 activates, the source-proof forms introduced
by v0.40, the unified-state
completion-I/O forms introduced by v0.37, the
per-iteration scratch form [PAR-3] admits (P15), and the three forms the
2026-08-28 blind-writer trial found a writer lacking: the inline factory reserve
inside P15, the hoisted length fact (P16), and the subtotal-returning walk
(P17). P18 is the explicit buffer a loop holds in place of the output resource
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
Effect rows (`allocates(arena 'r)` vs `allocates(heap)`) keep the allocation
site visible in a signature.

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
itself bounded: with `count` an unbounded `len(deref(weights))` the byte-for-byte
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
Pattern: return `own slice<'r, T>` directly. Every possible parameter supplier
is also written as exactly `own slice<'r, T>` under the same formal region and
element type. A function with several such parameters may return any of them,
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
`fn pick['r, 's](a: &uniq 'r Node, b: &uniq 's Node) -> selected: &uniq 'r Node` is
accepted, and its result is an ordinary holder over `a`'s storage that the
caller binds, writes through, and reborrows from. If instead the choice is
data-dependent, no signature can name the source, and the access belongs to
the caller: return the decision as an owned value — a two-variant enum, or an
index into a pool (P2) — and let the caller re-borrow from the place the
decision names.

The worked shape for the data-dependent case is three parts. The callee
`fn heavier(a: &'r Node, b: &'r Node) -> side: own Side reads(a, b)` reads both
weights through its shared borrows and returns `Left()` or `Right()`. The
superseded v0.36 spelled that effect `reads('r)`; since v0.39, `'r`
remains only the shared loan lifetime. Both forms take shared borrows, so the returned owned decision has no
borrow provenance. The caller binds
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
  region 'name {
    let rendered = name_at::<'name>(name: &uniq 'name name, index: index);
  }
  region 'f {
    match reserve_file::<'f>(factory: &uniq 'f files) {
      Ok(value: permit) => {
        region 'n {
          match open_file::<'f, 'n>(permit: move permit, root: &'f cwd,
                                  name: &'n name, start: 0_u64, end: 10_u64) {
            Ok(value: handle) => { /* read, fold, accumulate */ }
            Err(error: problem) => { }
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
  body, before any I/O. `let handle = propagate open_file(…);` is such an exit
  and not an exception to it: the `Err` edge is selected by the submission's own
  outcome, so it leaves from the remainder however early the statement is
  written. Match on the result instead and handle the error inside the body.
- **Write the accumulator as an ordinary source-order `set`.** `set sum = sum
  +wrap digest;` needs no associativity, no identity element, and no
  combination tree, because [PAR-3] commits the remainder's writes to storage
  rooted outside the body in iteration order. This is strictly more general
  than [PAR-2]'s admitted operation set: a non-associative fold, a float fold,
  and a `Result` route are all admitted here.
- **Reserve the file factory in the prologue, inline.** `reserve_file` takes
  and returns a short unique `&uniq FileFactory` loan inline [SYS-10], and
  prologues run in index order without overlapping, so one enclosing factory
  serves every iteration with no replication and no [OWN-5] relaxation. Its
  `Err(ResourceExhausted)` edge is the program's own source-order outcome (the
  factory's capacity is real: one credit per descriptor the target provides),
  so match on it and take the exit there, before the open: that is an early
  exit before the first submission, which the first rule admits. A program
  that reuses its capacity closes explicitly (`close_read`,
  `close_directory`, `close_directory_source` return the permit); derived
  release closes but returns nothing. Write
  the reserve and the open in the loop body itself. Factoring the pair into a
  helper — `fn open_source_from['f, 'd](factory: &uniq 'f FileFactory, …)` —
  costs the loop its pipeline, because the callee's own retained loan is what
  the staged judgment then sees. Two programs identical except for that
  factoring (‹loop› stands for the writer's own file and line; the verdict text
  after it is byte-exact):

  ```text
  inline  PAR stage  ‹loop›             for  permitted  staged at open_file::<'f, 'f>(…); 5 places classified
  helper  PAR stage  ‹loop›             for  denied     condition 3: a may-suspend call retains a borrow
                     past its own submission on storage the body writes and the iteration does not
                     introduce; instead, give each iteration its own resource; or, where the body only
                     publishes to that storage — an output stream is the pointed case — hoist the
                     per-iteration write out of the loop, folding a total in the body and writing it
                     once after the loop; or leave this loop sequential, because storage that carries
                     one position cannot be held by two iterations at once, at &uniq 'f files
  ```

  When the factory is itself a borrow — which it is in any recursive walker —
  [OWN-6] pushes the other way and admits no inline `region 'source { let
  permit = …; match open_… }`, because that region holds two statements. The
  two rules genuinely conflict there, and the resolution is that only one of
  the two forms is a program at all. Which form to write is decided by how the
  loop holds its factory, and the three measured outcomes are (‹loop› again
  stands for the writer's own file and line):

  ```text
  owned factory, inline    PAR stage  ‹loop›                   for  permitted  staged at
                           open_file::<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name,
                           start: 0_u64, end: 4_u64); 4 places classified
  borrowed factory, inline [OWN-6] InvalidChildReborrow — the program does not compile
  borrowed factory, helper PAR stage  ‹loop›                   for  denied     condition 3: a
                           may-suspend call retains a borrow past its own submission on storage the
                           body writes and the iteration does not introduce; … at
                           &uniq 'open deref(factory)
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
  rejection now states all three parts. The recorded finding is
  `docs/done/0098-blind-writer.md` D2; this is its writer-facing resolution,
  recorded in `docs/done/0100-writer-defaults-2.md`. D2's own proposal — that
  the helper boundary should not cost the pipeline — is a compiler change and
  is still open, so the price above is today's price and not a fixed one.

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

Problem: a program fills a buffer through `&uniq` callees and then hands a
prefix of it to a call whose `requires` bounds that prefix by the buffer's
length. The habit — and the reading of [ENT-5] an unguided writer forms in
twenty minutes — is that the callee's write killed the length fact, so `let
room = len(line);` has to be re-bound after every call that wrote through the
borrow. The write did not kill it. [ENT-5] kills, for each length term
`len(P)`, the root binding of `P` but not `P`'s element storage: an element
write never kills a length fact, and the compiler honours that across a callee
boundary. Only re-binding the root itself — a fresh `buffer_new`, a `set` of
the whole binding — kills it.

Pattern: bind the length once, above the loop and above every write, and
discharge every later requirement from that one binding.

```whitefoot
let room = len(line);
let fits = end <= room;
```

The first line sits above the loop and above every `put_text` that writes
through `&uniq 'put line`. The second sits inside the loop after all of them,
and it still discharges `emit_all`'s `requires length <= capacity`, because
nothing between the two killed `len(line)`.

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

## P17. Subtotal return instead of a threaded accumulator

Problem: a recursive walk accumulates counts into a record. Every other
language writes `totals = walk(dir, totals)`. Here that record is affine —
[OWN-1] makes every owned composite affine regardless of its field types, so
three `u64`s in a struct need `move` at every use — and the assignment is a
`set` on an affine place, which [STOR-1] refuses outright. Reaching for
[STOR-1]'s `replace` does not save it either when the call consumed the target
to compute the value: there is then no live owner to bind out, and [OWN-1]
rejects the reuse.

Pattern: the walk returns its own subtotal, the caller binds it under a fresh
`let`, and the fold is one `set` per field. `move` stays for the places where
the record is used as a value — passed, returned, or rebound.

```whitefoot
let sub = walk::<'recurse, 'c>(factory: &uniq 'recurse deref(factory), directory: dir);
set totals.lines = totals.lines +wrap sub.lines;
set totals.bytes = totals.bytes +wrap sub.bytes;
```

The fields are `u64`, and [OWN-1] copies primitives, so the fold is an ordinary
`set` per field even though the record holding them is affine. That is the
whole trick: the affinity is on the record, and the accumulation never touches
the record as a value.

`replace` is the right commit in the other case, and only in it: when the value
being committed leaves the target's root alive, `replace` writes the new owner
in and binds the previous one out, which is the only way to give an affine
binding a new owner in place.

```whitefoot
let stale = replace totals = fresh(lines: 3_u64);
```

Current value: this is a design the language pushes you to rather than a
ceremony it charges you. A walk that returns its subtotal has no accumulator
parameter to alias, so [OWN-6] never enters, and the caller's fold is the
ordinary source-order `set` P15 wants on `lines` and `bytes` separately.

Without the form, the two rejections a writer meets, in the order they meet
them:

```text
whitefootc: Semantics/Source [OWN-1]: SemanticIssue { rule: Own1, …, kind: BareAffineUse
{ mechanical_fix: "write `move p` for the affine place" } } at counts.wf:7:16 in line
"  let totals = running;"

whitefootc: Semantics/Source [STOR-1]: SemanticIssue { rule: Stor1, …, kind: AffineSetTarget
{ target_type: "Counts", mechanical_fix: "the right-hand side consumes the target root, so
replace cannot commit into it: bind the result under a new let, and combine it with the old
value field by field" } } at counts.wf:14:7 in line
"  set totals = walk(running: move totals);"
```

Replaces: a mutable accumulator parameter threaded by reference, and `+=` into
a caller's struct.

## P18. The loop's own buffer, published once

Problem: a loop reads a file per iteration and writes one line to an output —
`command.stdout`, or another positioned write target — from inside the body.
[PAR-3] denies it: the resource carries one position, so no iteration can be
given its own. (‹loop› is the writer's file and line; verdicts are byte-exact.)

```whitefoot
for @scan (index in 0_u64..8_u64) {
  /* P15's reserve, open, and read; then */
  region 'say {
    let written = write_once::<'say, 'say>(output: &uniq 'say out, source: &'say data, start: 0_u64, end: 2_u64);
  }
}
```

```text
PAR stage  ‹loop›  for  denied  condition 3: a may-suspend call retains a borrow past its own submission
           on storage the body writes and the iteration does not introduce; instead, give each iteration
           its own resource; or, where the body only publishes to that storage — an output stream is the
           pointed case — hoist the per-iteration write out of the loop, folding a total in the body and
           writing it once after the loop; or leave this loop sequential, because storage that carries
           one position cannot be held by two iterations at once, at &uniq 'say out
```

Pattern: **hold no `&uniq` resource inside a loop body; hold a buffer instead,
and publish it once after the loop.** Each iteration folds its line into the
loop's own buffer with an element `set` under one length fact above the loop
(P16); that element is `serialized-E`, the remainder's writes to storage
outside the loop being taken in index order.

```whitefoot
let page = buffer_new(8_u64, 0_u8);
let room = len(page);
for @scan (index in 0_u64..8_u64) {
  /* reserve, open, and read into the iteration's own data buffer */
  let writable = index < room;
  if writable {
    set page[index] = data[0_u64];
  }
}
region 'say {
  let written = write_once::<'say, 'say>(output: &uniq 'say out, source: &'say page, start: 0_u64, end: 8_u64);
}
```

```text
PAR stage  ‹loop›  for  permitted  staged at open_file::<'f, 'n>(permit: move permit, root: &'f cwd,
           name: &'n name, start: 0_u64, end: 2_u64); 6 places classified
```

Write into the page by element. A helper taking `&uniq` of it costs the loop
its pipeline under condition 4 instead: `denied  &uniq 'page page  a call of
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
re-established. [ENT-6]'s value-image join keeps an identical image, and joins
images that share one nonconstant coefficient vector and differ only in their
constant to that form plus a fresh delta atom over the incoming constant range.
Every other combination gives the binding one fresh full-type atom. [ENT-5]'s
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
    if hi <= room {
    } else {
      return 0_u64;
    }
```

That one guard is enough for a whole binary search: `hi` narrows on one arm of
the three-way comparison and `lo` on another, re-testing `hi <= room` at the
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
