# Whitefoot Pattern Doctrine (D6)

Status: seeded, non-normative writer guidance since 2026-07-09. Only the active
specification defines accepted source. The selected D6 direction is to test
whether a closed architecture-level vocabulary can stay COMPLETE (every task
modelable — a gap is a finding) and EFFICIENT (each pattern names the fact
channel or machine property that makes it fast) before normative adoption.
Writers may be taught this catalog during validation; hitting a wall is a
catalog finding, not authority to invent a language rule.

This document carries active v0.38 guidance, including the unified-state
completion-I/O forms activated by v0.37, the per-iteration scratch form
[PAR-3] admits (P15), and the three forms the 2026-08-28 blind-writer trial
found a writer lacking: the inline factory reserve inside P15, the hoisted
length fact (P16), and the subtotal-returning walk (P17).

Implementation boundary: the current backend emits no effect-derived
attributes or alias metadata, performs no proof-driven check elision, has no
termination checker or `willreturn` derivation, and does not implement arenas.
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
grep the signatures and find one `writes(state)` in the system. The superseded v0.36 wrote those
subjects as lifetimes; active v0.38 names the formal state directly.
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
Candidate direction: a possible future specification form would keep the
environment explicit and monomorphize a checked member call to a direct call, but v0.17
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
serial shape. Shipping that transform requires an independently verified fact
family that rederives the law and names its exact permitted consequence; until
then facts-off lowering is unchanged.
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
exists. It remains valid ordinary design advice to establish a current-function
invariant at one local control or state boundary and keep its auditable claim
outside the hot loop when the same fact dominates every use. A fact about a
callee result belongs in that callee's verified `ensures` and reaches the caller
through S12; moving a caller claim away from the call does not make that fact
local. Use `.wrap` only where modular behavior is the intended semantics; it
must never evade an exact operation's static domain obligation.
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
the declaration's error rather than the caller's. Pattern status: active v0.38
guidance, introduced before v0.36 and preserved since.

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
superseded v0.36 spelled that effect `reads('r)`; under active v0.38 `'r`
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

## P14. Claim only the proof residual

Problem: a partial operation needs a theorem that is true for every execution,
but the normative checker intentionally does not derive it—for example, an
ordinary loop invariant or a relation maintained by the current function's
local state machine. Write a claim only when the proposition has a complete
offline derivation, every runtime value component it reads is local to the
current function, and a later source-admission root would fail without that
exact occurrence. The five `because` fields state the available premises,
every inference step, the exact conclusion, the exact checker limitation, and
the exact terminal consumers.

Never use a claim to supply an omitted postcondition for a user or system call.
A returned scalar, tag, payload, aggregate, length, element, or borrow remains a
boundary result through copy, conversion, operation, construction, projection,
control selection, join, storage, and dereference. Put an expressible
cross-function relation in the callee's verified `ensures` and consume S12
directly; otherwise use a typed outcome or ordinary control. An `ensures` does
not make the returned value claim-local, so a caller cannot restate or
strengthen it with another claim.

Do not use a claim for a condition that can legitimately be false, an output
comparison, an impossible-arm sentinel, a test oracle, a deliberate abort, or
a fact the checker already knows. Use `if`, `match`, a typed outcome, return or
exit status for ordinary decisions and failures. Use a total operation row
when the operation's domain is not guaranteed. If removing the claim changes no
admission root, remove it; if two claims cover for one another, remove or
restructure both until each surviving occurrence is independently necessary.
Human, AI, SMT, or certificate review may approve the prose proof, but it never
changes compiler acceptance and never removes the retained runtime check.

Current value: CLM-1 checks the proof-predicate shape, exact five-field record,
canonical contribution formation, and current-function authority; CLM-2 then
rejects proved, refuted, overlapping, vacuous, and non-residual occurrences
before publishing a checked program. Accepted claims execute through the
ordinary IR/backend path in every build mode.

Replaces: `assert`, debug-only checks, `unreachable`, intentional aborts,
"trust me" comments, and claims written merely to silence a partial-operation
diagnostic.

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
for @scan index in 0_u64..8192_u64 {
  let name = buffer_new(16_u64, 0_u8);
  let data = buffer_new(65536_u64, 0_u8);
  region 'name {
    let rendered = name_at<'name>(name: &uniq 'name name, index: index);
  }
  region 'f {
    let permit = reserve_file<'f>(factory: &uniq 'f files);
    region 'n {
      match open_file<'f, 'n>(permit: move permit, root: &'f cwd,
                              name: &'n name, start: 0_u64, end: 10_u64) {
        Ok(value: handle) => { /* read, fold, accumulate */ }
        Err(error: problem) => { }
      }
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
  serves every iteration with no replication and no [OWN-5] relaxation. Write
  the reserve and the open in the loop body itself. Factoring the pair into a
  helper — `fn open_source_from['f, 'd](factory: &uniq 'f FileFactory, …)` —
  costs the loop its pipeline, because the callee's own retained loan is what
  the staged judgment then sees. Two programs identical except for that
  factoring:

  ```text
  inline  PAR stage  probes/inline.wf:17  for  permitted  staged at open_file<'f, 'f>(…); 5 places classified
  helper  PAR stage  probes/helper.wf:26  for  denied     condition 3: a may-suspend call retains a borrow
                     past its own submission on storage the body writes and the iteration does not
                     introduce; instead, give each iteration its own resource, or leave this loop
                     sequential: storage that carries one position cannot be held by two iterations at
                     once, at &uniq 'f files
  ```

  When the factory is itself a borrow — which it is in any recursive walker —
  [OWN-6] pushes the other way and admits no inline `region 'source { let
  permit = …; match open_… }`, because that region holds two statements. The
  two rules genuinely conflict there, and the resolution is that only one of
  the two forms is a program at all. Which form to write is decided by how the
  loop holds its factory, and the three measured outcomes are:

  ```text
  owned factory, inline    PAR stage  probes/inline_owned.wf:3   for  permitted  staged at
                           open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name,
                           start: 0_u64, end: 4_u64); 4 places classified
  borrowed factory, inline [OWN-6] InvalidChildReborrow — the program does not compile
  borrowed factory, helper PAR stage  probes/walk_helper.wf:14   for  denied     condition 3: a
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
  `docs/done/0098-blind-writer.md` D2 and its resolution is
  `docs/done/0100-writer-defaults-2.md`.

Read the verdict rather than guessing it. An ordinary `whitefootc` compile
prints a denied staged verdict to stderr, prefixed `whitefootc: note:`, with
every denied row of that loop's disposition table under it; the compilation
succeeded and the note is not a rejection. A loop whose staged verdict is
granted says nothing at all — including when its counted [PAR-2] verdict is
denied, which is the ordinary case for the form above: the counted rule refuses
the short factory loan the staged rule exists to admit, and that denial is
deliberately withheld from the default channel rather than telling a writer
their granted loop was denied. It is in the full report.

`whitefootc --par-ledger` is that full report: one `PAR stage` line per loop
that performs I/O, and one `PAR place` line for every place the judgment
classified, with its disposition and the reason, plus the `PAR pair`, `PAR
chain`, and `PAR loop` lines of the other judgments. A denial names the
offending place, the numbered condition, and the admitted form. Every notice is
one of those lines, byte for byte.

One remedy the report can print is not one a writer can take, and it says so:
where a loop's exit is selected by the may-suspend call's own outcome — the
`ReadEnd` break of a read-to-EOF loop over one file — the condition-2 line
states that [PAR-3] cannot stage that loop as written. The shapes staged today
are a fixed-trip bounded loop and a per-file loop over names; one file's chunk
loop stays sequential.

No worked example in `tests/programs/` currently holds this permission.
`dir_walk.wf`, `wfgrep.wf`, and `byte_string.wf` all compile to the module a
compiler with no overlap lowering at all emits. Two different facts are mixed
together in that sentence, and the resolution above separates them: their
*walker* loops carry the helper factoring because nothing else compiles, and
their denial is the price of the only admitted form; their *chunk* loops over
one file are denied by condition 2 because a read-to-EOF break cannot be
hoisted, which no rewrite fixes either. What a writer must not copy from them
is the hoisted scratch buffer — the form above is the one to copy for a
top-level per-file loop, and it is the one this pattern is about.

Current value: the judgment is landed and reported; the lowering that turns a
granted verdict into overlapped execution is not, so today this form costs a
per-iteration `malloc` and fill and buys a granted verdict rather than speed.
An implementation may allocate the body's constructions once at loop entry and
restore them per iteration, because the storage it reuses across iterations for
a construction whose value the body releases without observing it is not
observable [PAR-3] — so writing the natural form now is what makes the program
fast later, with no source change.

Replaces: hoisting scratch buffers out of loops for allocation cost, and every
writer-visible depth, window, batch, or `par for` marker a language would
otherwise need to express I/O overlap. There is no source spelling for how many
operations stay outstanding; the runtime chooses it.

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
let fits = ile(end, room);
```

The first line sits above the loop and above every `put_text` that writes
through `&uniq 'put line`. The second sits inside the loop after all of them,
and it still discharges `emit_all`'s `requires ile(length, capacity)`, because
nothing between the two killed `len(line)`.

Evidence that it compiles as written:
`research/experiments/blind-writer/2026-08-28/probes/probe_e_hoisted_length.wf`
is a whole program in that shape — both length bindings above the loop, above
every `put_text` and every `put_decimal` — and it is accepted.

Current value: the fact is load-bearing, not ceremony. It is the re-bind that
is redundant, and the compiler accepts the re-bind, which is why the belief
survives a whole program: 34 of the 41 length bindings in the five programs of
the 2026-08-28 blind-writer trial existed only to re-establish a fact that had
never died. What the hoisted fact holds off is the rejection you get with no
live length fact at all:

```text
whitefootc: Semantics/Source [FN-8]: SemanticIssue { rule: Fn8, …, kind:
UndischargedCallRequirement(UndischargedCallRequirementDetail { concrete_callee: "emit",
…, disposition: Unproved, mechanical_fix: "establish the complete callee requirement with
one dominating branch before the call, or, only when it is an independently true theorem
outside checker rules, add a CLM-2-admissible residual claim with a complete exact
`because` record" }) } at line.wf:33:16 in line "    let sent = emit<'e>(source: &'e line, length: end);"
```

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
let sub = walk<'recurse, 'c>(factory: &uniq 'recurse deref(factory), directory: dir);
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
