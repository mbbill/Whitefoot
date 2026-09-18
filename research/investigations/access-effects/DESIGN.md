# Ownership design combinations for Whitefoot

This is a whole-language proposal, not a change to the active specification.
The [research record](RESEARCH.md) owns the requirements, alternatives, source
assessment, and affected current decisions. The
[executable experiment](../../experiments/access-state/RESULTS.md) tests a
bounded fragment of the first candidate. Maintain this proposal while that
comparison remains active; supersede it when a subsequent design replaces it.
Notation in this document is explanatory, not final source grammar.

The [incremental case catalog](CASES.md) records the continuing local design
discussion and its expected positive/negative examples. Its permissive
locator-release, branch, and loop cases extend beyond the earlier executable probe;
they are not implemented or measured results. Use those cases to challenge the
whole-language hypotheses with focused examples without narrowing the critical
capabilities a complete candidate must cover.

## Current evaluation method (owner discussion, 2026-09-17)

The owner defined three evaluation criteria:

1. Safety: types and checked proofs must preserve WF's current memory-safety
   and absence-of-unproved-runtime-failure guarantees, under its existing scope
   assumptions. A speed or capability benefit does not compensate for a safety
   failure. Existing deterministic, terminating, SMT-free acceptance constraints
   continue to apply.
2. Performance: evaluate both sequential execution and parallel computation;
   IO performance is outside this round. Parallel permission alone is not a
   runtime measurement, and sequential executability does not preserve a lost
   parallel capability.
3. Expressiveness: support the structures and algorithms needed for substantial
   compilers, browsers and OS kernels. Preserve at least current WF capabilities,
   but do not use its currently limited corpus as the ceiling or sufficient
   evidence of this criterion. IO and FFI remain deferred in this comparison.

Holes, storable references, validity loans, implicit resource contexts and tokens
are possible mechanisms, not independent ends. In particular, accepting the
literal `take` spelling in an earlier candidate is not an evaluation requirement.
Compare ways to implement the same task, including their representation,
algorithm, proof and runtime costs; record any loss rather than silently changing
the task. The three criteria do not have an invented numeric weighting.

The capability/choice inventory to cover before selecting a complete combination
is below. Alternatives are starting options, not an exhaustive enumeration;
several may coexist. A row is a group of choices, not a single Boolean feature.

| Area | Features to account for | Choices to make explicit |
|---|---|---|
| Values and composite types | Scalars, structs, enums, fixed arrays, nested and recursive data. | Inline versus indirect members; where ownership lives in recursive/shared representations; which properties compose from fields. |
| Value use | Copy/affine/linear, transfer, assignment, replacement and cleanup. | Separate ownership transfer from content relocation or combine them in defined forms; allow holes, require exchange, or represent vacancy as data; implicit versus explicit disposal. |
| Storage | Local/heap/store allocation, independent release, scope exit, reuse and stable locations. | Stable versus relocatable allocations; allocation ownership representation; individual versus grouped reclamation; old-reference behavior after end/reuse. |
| Addressing and parts | Direct/indirect access, aliases, fields, dynamic indices, ranges, saving/reloading associations. | Scoped names, storable locators, container-plus-index forms, or combinations; copying/rebinding rules; target identity and validity guarantees. |
| Control flow | Branches, matches, loops, nesting, normal/early exits and typed error propagation. | Structural state agreement, retained conditional relations, explicit state handoffs or a mixture; loop entry/backedge/exit obligations and cleanup. |
| Functions and abstraction | Parameters/results, generics, recursion, function arguments and resource/target relations. | Written versus checked-derived summaries; vocabulary for access, ownership, lifetime and result relations; definition-side checking and call-side substitution. |
| Contracts and proofs | Preconditions/postconditions, object/loop invariants, numeric facts and invalidation. | Fixed automatic fact families versus explicit finite steps; dependencies shared between numeric and resource facts; establishing and opening library invariants. |
| Parallel computation | Calls, iterations, range tasks, reductions and lifetime-changing operations. | Exclusivity carried by values, operation-scoped access demands, separate permissions, or combinations; how conflicts and legal recombination are proved. |
| Representation and execution | Contiguity, alignment/addressing, proof erasure and optimization facts. | Runtime components of each value; stable layout contracts; exact origin of backend assumptions; extra allocation, copying, tags and indirection. |

The inventory distinguishes required capabilities from alternative mechanisms;
it does not require every conceivable addressing form or general threads.
[PROGRAMS.md](PROGRAMS.md#current-capability-floor) anchors the initial current
floor; the broader coverage audit remains unfinished.

Each iteration starts with explicit choices for this complete critical scope.
Freeze them while deriving the interaction matrix. Record an absent rule as a
gap; do not repair the candidate during that pass. Correct a mistaken derivation
without changing its premises, and distinguish that correction from a new
design choice. At the end of the pass, discuss the defects and alternatives with
the owner. Only after that discussion selects a revised combination may the next
pass begin. The owner explicitly corrected the previous agent-led E1/E2 repair
sequence: a missing rule is not authorization to select a replacement mechanism.
A promising local fragment is not a completed iteration. Internal coherence
also does not establish adequate expressiveness or acceptable performance.

Keep engineering witnesses when a capability is prohibited. Record separately
whether the rules correctly reject the old source form and whether the same
task has a safe implementation at acceptable cost. A forbidden construct must
not disappear from coverage and become a vacuous success. Retain the broader
PROGRAMS tasks to catch interactions involving more than two axes.

An iteration record contains only: the complete compact choice/coverage table;
changed rules and affected combinations; small positive/negative examples with
their derivations; the safety/performance/expressiveness assessment; and the
ranked concrete defects motivating the next change. Link unchanged derivations
instead of repeating them. Keep this current proposal and the case catalogs as
the reading path; a later iteration document earns a separate file only when it
preserves a distinct coherent result and the evidence for changing it.

Coherence in a discussion record means no unresolved core semantic rule is
being used as if it were established. It is not a soundness theorem, a measured
performance claim, or proof of implementability. Label hand derivations,
mechanical checks and measurements separately. Implementation remains outside
this design-discussion task; no DCR is requested for these records.

The owner agreed to refine this method before executing a candidate iteration.
Keep engineering tasks and allowed costs fixed while changing mechanisms. A
retained candidate must close operation, boundary and control-flow rules, make
them mutually consistent, provide derivable grounds for every used fact, and
specify implementable acceptance. Do not hide a missing mechanism behind "the
library proves it". Accumulate defects throughout; a decisive hard-constraint
counterexample can eliminate a candidate before full elaboration. Preserve the
best complete incumbent and allow coupled choices to change together. Where
performance and expressiveness trade off, retain explicit alternatives rather
than inventing a single score. Coverage statuses are derived, refuted, and
unresolved; none means measured without measurement evidence.

Before choosing the next vector, use the choice table above together with
[the engineering-task table](PROGRAMS.md#representative-engineering-tasks).
This establishes the questions and comparison surface, not a selected vector.

## Candidate x0: current-state access with separate resource accounting

Status: proposed concrete choice vector, 2026-09-17. This defines a starting
candidate, not an adopted language or a completed consistency result. It is
chosen because the local cases already discriminate its rules and because it
connects sequential aliases, resource state and existing parallel computation
through the same target information. No evidence establishes it as the best
combination. The earlier A/B sections are background hypotheses, not implicit
permission for a matrix worker to fill a gap with a different rule.

The matrix is evaluated at fixed x0. Its axes below have fixed values; cells
do not enumerate alternate designs. Include the diagonal and one triangle:
24 axes give 300 cells. A cell respects all of x0, not only its two named axes.
Alternative choices belong to a new candidate revision. Additional axes require
a recorded interaction witness and a coordinated inventory revision.

Ordinary scalar/arithmetic domain checks, typed outcome semantics, target layout
qualification and allocation-exhaustion scope start from the v0.59 baseline
linked in PROGRAMS.md. IO/FFI and general thread constructs are outside this
round. Existing call/iteration overlap and reductions are inside it. The changes
below concern ownership, storage and the contracts and proofs needed to connect
them; an omitted ordinary language rule is not silently removed.

### Fixed choices and matrix axes

| Axis | x0 choice |
|---|---|
| 1. Storage identity | Each allocation or suballocation has a fresh logical identity. A locator captures one target, possibly described by conditional or symbolic relations. Different symbols alone do not prove disjointness. Reusing bytes never revives an ended identity. |
| 2. Owned storage representation | Scalars, ordinary fields and fixed arrays are inline; separately allocated cells/runs have owning descriptors. Owning and merely locating an allocation are distinct even if each uses a pointer at runtime. Store identity and target identity are separate proof parameters. |
| 3. Value and ownership transfer | `move(x)` transfers x's value and contained obligations to a destination, leaving its source slot empty. Moving a heap-owning descriptor preserves its backing allocation. Physically transferring an inline aggregate does not retarget old locators to its new location. |
| 4. Copy, affine and linear | Copy duplicates no resource obligation; affine values may use verified automatic cleanup; linear obligations require explicit discharge or transfer. Composite classes follow their owned contents, with explicit linear strengthening permitted. Repeated ownership handoff is allowed; duplicate handoff is not. |
| 5. Holes | `take` may empty a live slot through its name or a locator, including a selected field/element. `put` fills an empty live slot. No compiler-inserted occupancy tag. Unknown concrete target is not itself a rejection if the required relational proof is available. |
| 6. Reading and replacement | Copy reads duplicate content; observing non-copy content returns copy observations or accesses it under a contract, not a duplicate owner. Scalar copy writes may fill/overwrite. Replacing non-copy content exchanges old and new values without losing obligations. |
| 7. Locator copying and aliasing | A locator is copyable, does not own its target and does not grant a lifetime-long exclusion. Sequential aliased reads/writes are allowed when each operation's premises hold. No address-to-locator fabrication. |
| 8. Stored locators | Locators may be fields, elements, arguments and results. Loading one captures that stored target; updating the field later does not retarget an earlier copy. Hidden target identities may be packaged with checked relations; packaging creates no liveness or disposal right. |
| 9. Validity and scope | Access requires live enclosing storage, current layout and sufficient initialization at the operation. A locator may remain after its target ends but cannot access it. Lexical local storage ends on scope exit; ownership transfer does not extend an inline local's storage lifetime. |
| 10. Disposal authority | `release(p)` may identify storage through an owner or locator but must consume that target's single available disposal obligation. A locator does not supply the obligation. Content obligations must be dealt with first or by verified cleanup. Allocation and contained-value duties are distinct. |
| 11. Automatic cleanup | Affine cleanup is derived only where a definite valid cleanup sequence is proved for that source exit. No hidden conditional drop flags or inserted state-testing branches. Otherwise the writer must express the cleanup control flow. Linear duties cannot be silently dropped. No new unchecked user finalizer. |
| 12. Fields and active layout | Fields have separate locations beneath a containing allocation; whole-object operations account for affected parts. Partially initialized aggregates cannot be read or transferred as full values. Switching enum variants ends the old payload's identity; returning to that variant does not revive its old locators. |
| 13. Arrays and ranges | Elements use captured index values and views use captured ranges over a particular backing allocation. Bounds and separation need proofs. Hole facts may describe selected elements or ranges. Reallocation changes backing identity; copying a view never makes new element storage. |
| 14. Providers and shared management | Blocks may share a locator for provider metadata while owning disjoint payloads. Allocation/release declare actual metadata and payload effects. A provider cannot end while dependent allocations remain. No hidden synchronization is introduced to make conflicting operations overlap. |
| 15. Containers and library invariants | Containers use ordinary owned storage, locators and checked representation invariants over the established allocation primitives. Opening/closing an invariant requires resource/fact evidence. No container-specific exemption and no writer assertion accepted as an unproved invariant. |
| 16. Branch joins | Preserve joint conditional target/state/obligation descriptions tied to captured condition values. A use must work in all represented alternatives, or in the subset established by a source guard/proof. Binding reassignment does not rewrite the captured condition. |
| 17. Loops and exits | Resource relations at a loop head are written invariants, proved initially and on every backedge. Break/return/error exits carry their actual state and duties. Dynamic allocation instances use symbolic families with explicit finite proof steps, not one identity reused for all iterations. |
| 18. Function contracts | Definitions are checked against declared entry/exit relations, resource transfers and whole-call accesses. Calls substitute actual associations and use the verified signature. Equal actual targets are allowed unless the contract requires separation; access demands may combine, consumable obligations may not duplicate. |
| 19. Generics, recursion and function arguments | Preserve WF type/const/function abstraction and recursion. Passed functions carry checked contracts, including effects and transfers; recursive calls use the declared contract. No automatic whole-call-tree expansion for ownership checking. |
| 20. Proof mechanism | Extend the checked fact/resource context, retaining existing numeric derivations. Automatic work is structural propagation, direct target equality/separation, guarded case checking and the fixed numeric families. Loop summaries, general resource families and invariant steps are explicit finite certificates. No SMT, timeout-dependent acceptance or unbounded inferred heap invariant. |
| 21. Access and state effects | Contracts separately describe access during execution and exit state. Reads, writes, initialization changes, allocation and storage ending identify their actual targets and management state. Changing or ending a target invalidates affected state facts before new facts are published. |
| 22. Call overlap | Retain source-order observables. Prove independence of full accesses, argument evaluation, consumed resources and retained storage; read/read aliasing is allowed. Restoring a value at return does not remove an intermediate write/hole from conflict checking. No proof, no overlap permission. |
| 23. Loop overlap and reductions | Retain the current admitted element maps, adjacent-range helper calls and fixed reduction operations. Use proven cross-iteration separation and resource accounting; do not introduce overlap through runtime alias guards or silently generalize numeric laws. |
| 24. Lowering and optimization facts | Erase proof identities and resource bookkeeping. Locators do not receive unconditional uniqueness/noalias attributes. Emit access-scoped facts only from established proofs. Any extra runtime descriptor field, map, flag, allocation or copy must be stated and costed; it cannot be hidden as proof metadata. |

### Shared pseudocode for x0

This is explanatory syntax with fixed meanings, not a new WF grammar. `slot`
creates addressable local storage; `alloc` creates a separate initialized
allocation and returns its owner. `loc(place)` captures an addressable place;
`loc(*owner)` captures its referent, not the owner's descriptor slot. Forming a
locator requires live storage and current layout, but does not promise full
content or future validity. Ordinary type and domain premises still apply.

```text
slot a: Int = 10
let p = loc(a)
let q = p
let v = take(p)               // a remains live, empty; q still locates a
put(q, v)                    // a is full again
let old = replace(p, 20)      // same slot; old receives 10
```

`read(p)` returns a copy value and is not a generic copy of an affine/linear
payload. `write(p, v)` in these examples is copy-scalar initialization/overwrite.
For non-copy payloads use transfer, `put`, or `replace`. `put` requires empty;
`replace` requires full. `take` transfers any contained obligations to its result.

```text
let a = alloc(10)             // owns allocation A
let p = loc(*a)
let b = move(a)               // descriptor transfer; A stays in place
let n = read(p)
release(b)                   // consumes A's duty; A ends
read(p)                      // invalid: A ended
```

Replacing `release(b)` by `release(p)` requires the same available A duty;
it retires b's ownership authority as well. It does not leave b entitled to a
second cleanup. A separately taken linear value retains its own obligation
after the empty container allocation is released. A locator without access to
the disposal obligation cannot perform that release.

Physical content relocation is spelled as a transfer between slots, for
example `put(dst, take(src))`; neither the locators for src nor those for dst
change their target. Expression evaluation is in source order. `replace` is
one checked exchange after its operands have been evaluated, not a claim of
hardware atomicity. Calls overlapping it still need the ordinary effect check.

Types may use erased target parameters, for example `Loc<P, T>`, and a separate
provider parameter S. P denotes a captured target, not a duration or a runtime
address known at compilation. A symbolic target may have several possible
origins. Storing hidden identities requires packaging the relations needed by
the abstraction; a type parameter alone is not a proof that a target exists.

```text
struct Cursor<P> { at: Loc<P, Int> }

fn observe<P>(p: Loc<P, Int>) -> Int
    requires Live(P), Full(P)
    accesses reads(P)
    ensures Live(P), Full(P)
{
    return read(p)
}
```

`requires`, `ensures` and `accesses` are checked contract clauses, not trusted
assumptions about an unverified body. Resource transfers are separately named
in the contract; ordinary Boolean conjunction does not copy a resource. Generic
P and Q may alias. A caller cannot infer separation from their different names.
Contract effects name the abstract state actually accessed through a stored
association, not just the containing descriptor. Private representations need
checked abstraction relations connecting those names to their implementation.

Branches use `if/else`; matches and typed outcomes use ordinary tagged values.
Loops use `while` or counted `for`, with `invariant { ... }` for carried resource
relations. `use` denotes an explicit checked proof step, never a runtime test.
For a matrix case, `request_overlap(call1, call2)` is a test-harness request to
judge whether the two source-ordered calls may overlap, not a proposed thread
construct. Baseline source evaluation order remains the reference behavior.

### Definition readiness and matrix discipline

This draft fixes choices but is not yet a complete formal acceptance calculus.
Before independent workers may mark cells derived, supply one shared rule
sheet for primitive state/resource transitions, conditional joins, scope cleanup,
contract substitution, explicit invariant/family proof steps and parallel
composition. In particular, "guarded case checking" is not a license to invent
a different Boolean solver per cell; its traversal/rewrite rules must be fixed,
and branching cost remains an open performance question. Predicates need
checked introduction/elimination rules; writing an invariant does not establish
its implementability. A missing rule produces an unresolved cell or a proposed
coordinated candidate revision, not an ad hoc local acceptance.

The [first matrix](MATRIX-X0.md) uses a frozen shared rule sheet and GPT-5.6 Sol
derivations as requested by the owner. Its statuses and limitations live with
the cell records; coverage is not a correctness theorem. Existing CASES
expectations and the old bounded executable model are evidence to revisit,
not x0 validation.
The full task/capability table remains in scope, including copy/affine/linear,
stored relations, generic calls, dynamic storage and parallel computation.

## Next-candidate preparation: temporary references and effect-derived calls

Status: owner discussion recorded on 2026-09-17; not a frozen x1 rule sheet and
not a language amendment. No second matrix pass has started. The seven broad
first-round problem areas have not all been discussed. This preparation grows
out of the first area, stored target relationships, rather than silently closing
the others. The frozen x0 still has 221 D, 12 C, 67 U and 0 X cells.

### Discussion conclusions and candidate direction

- Static checking describes a sound overapproximation of possible joint states,
  with retained correlations. It need not recover one concrete target after a
  runtime choice. Storing references can make such relationships harder to
  represent, update and expose in contracts. Neither unavoidable exponential
  cost for every representation nor a claimed percentage reduction has been
  established.
- The owner requires the non-owning-reference storage restriction to apply
  transitively: structs, enums, tuples, arrays, slice elements and generic
  wrappers may not hide a reference. Allowing one wrapper would reopen struct
  storage through that wrapper. A non-owning slice/view is itself reference-like,
  not an exception that may be stored in another value. Keep temporary local
  references and direct call inputs, and do not return references. This keeps
  library and parallel range operations from needing to transfer or copy their
  payloads merely to access them. Return indices or
  offsets where a caller can reconstruct a reference with the necessary proof;
  this is a possible rewrite, not a claim that every reference result has a
  free equivalent.
- Use one reference form without a `uniq`/shared access-mode distinction.
  Function contracts identify actual read/write targets. Within one call,
  overlapping read/read demands from different formals are compatible;
  read/write or write/write demands require proved separation. This is the
  owner's chosen direction, replacing x0 R11's general admission of aliased
  formals whose ordered body happened to be safe.
- The owner proposes whole-referent effect names for reference formals:
  `writes(data)` rather than `writes(object.data)`, with the caller passing
  `&object.data` directly. Non-reference parameters arrive as complete values
  by copy or ownership transfer; member effect paths remain a possible form for
  those parameters. This is a proposed signature restriction, not evidence that
  every member-path interface has an equal-cost rewrite. Caller-side target
  resolution must still retain field, backing and range identities.
- Operations through one formal may both read and write; the callee checks
  their order and state transitions. A row records exhibited possible effects,
  including conditional operations, not only the effects taken on one runtime
  execution. This does not choose arbitrary padded effect upper bounds in place
  of WF's existing exact-row discipline.
- Distinguish within-call parameter compatibility from overlap of two calls.
  Sequential calls may access the same target in write/read order. Their
  conflicting complete effects deny concurrent execution, not sequential source.
- A reference supplies no disposal duty. Retain current-state, initialization,
  layout, bounds and resource checks. The candidate aims to require no explicit
  borrow-lifetime parameters; removing surface annotations does not remove those
  checks. No new blanket ban on release through a reference has been
  selected when the corresponding duty is otherwise available.
- A Box is an ordinary owning descriptor plus a relation to separately managed
  storage and its nonduplicable disposal obligation. The owner reopened the
  earlier mandatory-linear proposal in favor of considering affine cleanup for
  convenience. Cleanup must still be valid for the contents and provider and
  expose its accesses and storage ending. This does not authorize silently
  discarding linear content. The exact release timing and partial-state rules
  remain to be fixed before a freeze.
- The earlier proposal that every Box move invalidates all payload references
  was reopened: the latest direction permits a temporary reference to continue
  naming unmoved backing after the descriptor moves, subject to current validity.
  Nested owner transfers still need a precise rule. Do not record either a
  universal move-invalidates rule or a complete move-preservation theorem.
- For this comparison, awkward source is not a failure by itself. Under the
  safety constraint, charge extra copying, allocation, lookup, checking, retained
  memory and lost parallelism. Expressibility and practical proof/checking cost
  still constrain whether a large real program can be implemented.

### Code anchors for the next rule sheet

These are discussion expectations and open boundaries, not a new derivation
batch. `Ref`, `Box`, effects and transfer forms remain explanatory notation.

```text
struct Saved { p: Ref<Int> }  // proposed rejection: stored non-owning reference
refs: Array<Ref<Int>>         // proposed rejection for the same reason
wrapped: Option<Ref<Int>>     // also rejected: enum wrapping is no exception
pair: (Int, Ref<Int>)         // also rejected: tuple wrapping is no exception
fn choose(c, p: Ref<Int>, q: Ref<Int>) -> Ref<Int> // proposed result restriction

fn inspect(p: Ref<Int>) reads(p) { return read(p) }
fn set_one(p: Ref<Int>) writes(p) { write(p, 1) }
inspect(&a); inspect(&a)      // compatible shared reads
set_one(&a); inspect(&a)      // valid order; conflicting calls cannot overlap
```

```text
fn update_then_read(p, q) writes(p), reads(q) {
    write(p, 10)
    return read(q)
}
update_then_read(&a, &a)      // reject under cross-formal conflict rule

fn update_one(p) writes(p), reads(p) {
    write(p, 10)
    return read(p)
}                           // same-formal ordered access is not that conflict

fn process(p: Ref<Int>, owner: Box<Int>)
    reads(target(p)), writes(payload(owner))
    consumes(owner), ends(payload(owner))
{
    release(owner)
    read(p)
}
process(&*a, move(a))        // same target: reject at caller compatibility check
```

`consumes`/`ends` above name resource/post-state information, not a decision to
add new effect-row categories. The write footprint must cover ended storage,
not just the descriptor. A contract's entry facts do not excuse invalid local
operations inside its body.

```text
a = box(10)
p = &*a
b = move(a)
read(p)                     // latest direction: preserve unmoved backing
inspect(p)                  // callee binding ends on return
read(p)                     // caller's p did not expire merely on call return
release(b)
read(p)                     // reject after backing ends
```

```text
struct Node { value: Int, next: Option<Box<Node>> }
struct Link { next: NodeId }
struct Token { offset: Index, length: Index }
```

Owning recursion is not prohibited by the non-owning-reference restriction.
NodeId/offset forms do not establish bounds, stable identity or permission by
their spelling. Deletion, reuse and association with the correct owner remain
proof obligations or explicit, costed program behavior.

```text
fn set_one(data: Ref<Int>) writes(data) { write(data, 1) }
set_one(&object.data)        // resolve formal data to this actual field
inspect(&object.other)      // different fields may still prove separate
```

Whole-formal spelling need not imply whole-owner conflict: the actual argument
may already be a field, element or range projection. Forming the projection
must itself be checked and included in argument-evaluation effects. For such
directly available targets, replacing a member-path effect by a projected formal
requires no payload copy. If a callee must first discover the selected member,
an index/path-return rewrite can require reconstruction or a second traversal;
whether it does is a concrete workload question. No inevitable slowdown or
universal zero-cost translation is claimed. No function body is inspected by
the caller merely to narrow an overly broad declared footprint.

### Boundaries to settle before freezing x1

1. Formalize the now-transitive storage prohibition and direct input/reborrow
   and forwarding rules. Non-owning views are temporary reference forms, not
   storable wrapper exceptions. Captures must not reopen aggregate storage.
   Preserve the current direct range-helper comparison; local branch/loop
   references were not restricted to one statement.
2. Decide the proposed root-only reference-formal spelling and define its exact
   granularity before evaluating it. Fix effect projection for owner payloads,
   nested fields, dynamic ranges, argument evaluation, release and shared
   provider metadata. Whole-root effects may conservatively cover untouched
   parts; specify how the existing exact-row check operates at that chosen
   granularity rather than silently changing it to arbitrary upper bounds.
   A declaration must be verified, and no plain reference grants ambient
   authority. Same-formal and cross-formal access must remain distinguishable.
3. Fix which storage survives owner moves/take/replacement, which references
   lose access, and the Box discharge policy. The owner may itself be a field
   or array element; it is not necessarily a named local binding.

ID reuse, resource families, container proofs, partial cleanup, providers and
lowering remain visible gaps for the next matrix; this summary selects no new
mechanism for them. E1/E2 must not fill those gaps by default.

Prepare the revised vector as explicit deltas to all 24 x0 axes, including
dependencies on axes not directly changed. Begin the next full upper triangle
only after the rule sheet is frozen through owner discussion. Use GPT-5.6 Sol
for the derivations, retain positive/negative code and task outcomes, and stop
at gaps rather than modifying rules. Current parallel capability and the
PROGRAMS engineering tasks remain required comparisons.

The discriminators are: owning-tree traversal without payload copies; token
ranges over one or many backings; cyclic graph deletion/reuse; shared indexes;
resource-container growth/removal; adjacent-range helper parallelism; and large
owned aggregates crossing function boundaries. Ordinary move-in/move-out has
no general zero-copy guarantee. A possible in-place ABI is an implementation
alternative to test, not a selected replacement for temporary call references.

## Parked drafts: E1 target packages and E2 range families

The [gap map](GAPS.md) preserves the first-round U/C classifications and
witness-precision follow-ups. [E1](TARGET-PACKAGES.md) and
[E2](RANGE-FAMILIES.md) preserve bounded exploratory mechanisms and their code
for comparison. The agent pursued them before the required owner discussion;
they are not selected repairs or a successor candidate. E1's stored-reference
packaging is not a premise of the new temporary-reference direction. Dynamic
owned resource families remain a problem, but E2's interval ledger has not been
selected to solve it. Neither draft changes x0 verdicts or supplies a complete
capability-floor audit, runtime measurement or soundness proof.

## Earlier working candidates: A and B

The earlier proposal developed **A: access and current-state checking**, with
**B: validity loans and access effects** as a more restrictive
alternative. Both replace persistent exclusive write loans. The difference is
what a retained reference promises about future storage validity.

- A reference in A is a locator. It says which storage an operation would
  access. Whether that access is legal is proved in the current state.
- A reference in B additionally holds a validity obligation over a declared
  extent. Destruction or a layout change that would violate the obligation is
  forbidden until that obligation ends. Ordinary compatible writes remain
  possible through other aliases.

A fits WF's existing emphasis on explicit, erased proofs and source-order
semantics particularly well: memory validity, initialization, indexing domains,
and object invariants become instances of the same current-state reasoning.
It also admits reclaiming a graph node while unused locator values remain in
other nodes. Its cost is richer state contracts and more proof work for stored
graphs. B makes more references usable by construction and rejects more
programs at destruction rather than at subsequent access. This is an
expressiveness and interface tradeoff, not an established performance ranking.

That earlier preference is a hypothesis to reevaluate under the method above,
not the selected starting vector or a requirement to retain holes. Neither candidate is adopted,
implemented by the WF compiler, or proved sound as a complete language here.

The main semantic questions now have explicit proposed answers rather than
unspecified mechanisms:

| Question | Proposed answer |
|---|---|
| Must each alias hold an exclusive permission? | No. Access demands query one checked sequential context; disposal obligations remain affine/linear. |
| How does a caller recover hidden relationships? | Ordinary stored links and explicit abstract association/result contracts; no body-derived ancestry. |
| Does reuse need a runtime generation check? | Fresh logical allocation identities and current-use premises distinguish reuse; the bounded physical model exercises this. |
| Must the compiler enumerate runtime objects or alias cases? | No. First-order substitution, conservative possible-alias updates, and explicit family/invariant witnesses. |
| Can a stored mutable pointer keep changing its static target? | Its slot can change association; a previously loaded locator retains its captured target. |
| Can effects completely replace safety checking? | No. They describe accesses; type, initialization, liveness, ownership and domain premises justify them. |

Choosing A or B remains an owner decision. Proving and measuring the complete
calculus remains research/implementation work; those limits are stated at the
end and are not passed off as consequences of the bounded model.

## Local straight-line baseline

The whole-language mechanisms below are hypotheses to validate incrementally.
This section describes the earlier executable baseline, including its stricter
owner-only release and explicit-cleanup choices. The subsequent
[discussion cases](CASES.md) keep those choices open while exploring release
through locators and conditional state. The executable baseline fixes a much
smaller language: one procedure, two
independent scalar objects created at entry, and local locator bindings.
Their origins are known. There are no branches, loops, calls, fields, arrays,
owner moves, later allocations, or physical storage reuse in this baseline.
The scalar payload is copy data; arithmetic and payload-dependent control are
absent. Object creation and release are abstract operations, not an allocator
implementation.

Each state records exact locator targets, live/initialized storage, and the
fixed owners' remaining disposal obligations. Locator creation copies a
target without copying ownership. Rebinding a locator changes that binding
only, including when its source is another locator. Rebinding cannot overwrite
an owner. Taking data empties a still-live slot, putting data initializes it,
and release consumes the owner and ends the slot's lifetime. The local probe
uses explicit linear cleanup: procedure exit requires every entry owner to
have been consumed. This is an experiment choice, not WF's general affine
cleanup rule.

With known entry relations and deterministic target transformations, each
straight-line prefix has one exact resource state. The target need not be a
known physical address. An unknown function parameter is outside this scope;
introducing one would not refute exactness under these premises.

The minimum witnesses are locator-copy stability after rebinding, sequential
writes through aliases, a read after take, restoration through another alias,
a read or write after release, disposal through a non-owner, repeated disposal,
and an unconsumed exit obligation. Their expected outcomes follow the
operation rules before the checker is run. The
[local experiment](../../experiments/access-state/RESULTS.md#local-baseline-criteria)
compares acceptance and retained resource state, so a checker that needlessly
forgets a known relation is distinguishable from an exact one.

This baseline deliberately has no target-set join. A later branch experiment
must first define the exact set of reachable resource states and then compare
more compact approximations against it. Both reference/reference and
reference/initialization correlations matter. These later requirements do not
add branches or general resource predicates to the present probe.

## Common foundation: separate three responsibilities

| Information | Question it answers | Can it be copied? |
|---|---|---|
| Locator and association | Which storage or abstract state does this value designate? | Yes, subject to ordinary value/visibility rules. |
| Ownership obligation | Who must consume, return, or release this resource? | An affine or linear obligation cannot be duplicated. |
| Current access evidence | Is this target live, suitably typed and initialized, and is this operation authorized now? | Facts can be reused in one sequential context; they are not permission to duplicate that context into racing executions. |

Effects describe actual reads and writes. They neither manufacture an owned
resource nor prove that storage exists. For example, `store` and `free` both
write their target, but only one preserves it. Ordinary state pre/postconditions
express that difference; no additional scheduling effect category is needed.
Ending a storage lifetime counts as a write to that storage even when the
implementation physically changes only allocator metadata.

An owner is not automatically the only pointer to its content. A locator is
not automatically permission to dispose of its content. A read-only locator
restricts operations through that path; it does not promise that other paths
never write. Frozen data, when required, needs an actual invariant or access
contract that excludes the conflicting writes.

An object's association is a compile-time parameter describing runtime values,
not a runtime parameter pretending to be a type. A pointer still contains the
ordinary address. The static parameter connects its type to contracts and
proofs; it need not encode that address or select different machine code.

## Candidate A: locators plus a current resource context

### Values, storage, and the typing judgment

Use three kinds of static names: ordinary types `T`, storage/association names
`P`, and footprint/family expressions `F`. A footprint can name fields or
captured ranges within storage, or explicitly related abstract state. It is
not the entire graph reachable through every pointer.

```text
Loc<P, T>                 // locator for a T-shaped slot in P
Owner<P, T>               // ordinary owner descriptor plus disposal obligation
exists P. Owner<P, T>     // package an allocation whose identity is hidden
```

Read/write authorization can be restricted by the locator's interface and
ordinary visibility rules. A writable locator is aliasable. It grants no
exclusive interval and cannot promote a read-only path to writable. The exact
surface spelling of these access restrictions is independent of the mechanism.

The proposed checking judgment is:

```text
names; values; resources; facts |- statement
    => values'; resources'; facts' ! reads(R), writes(W)
```

- `values` describes bindings, types, captured target identities, and owned
  values. This is where an owner move is checked.
- `resources` describes current live storage, layout, initialized fields,
  outstanding disposal obligations, and explicitly folded family predicates.
- `facts` describes equalities, separation, bounds, stored links, and current
  invariants. Every fact about mutable state has its supporting footprint.
- `R` and `W` describe runtime access. Proof-only operations do not add effects.

The resource context is compiler state, not a heap table or a runtime token.
At an ordinary sequential call it is checked against the signature and then
updated by the verified exit contract. No user has to pass this context as a
runtime argument. Opaque code has the same explicit contractual obligations as
ordinary code; declaring a contract does not establish its implementation.

The essential distinction from explicit capability threading is that two
sequential demands for `write(P)` are demands on the same context. Substitution
of `P = Q` in `{write(P), write(Q)}` combines the footprint; it does not demand
two consumable ownership tokens. A call consuming two owners still needs two
legitimate owned values and cannot receive the same linear value twice.

This distinction has a precise precedent and a caution. [Alias Types](https://www.cs.cornell.edu/talc/papers/alias.pdf)
uses freely copied pointers and a separate store description. Its linear
store join forces distinct entries to be disjoint; simply reusing that join
for WF's alias-permitting access demands would reject useful calls. The
[SL/implicit-dynamic-frames comparison](https://lmcs.episciences.org/802/pdf)
also distinguishes ordinary conjunction from permission addition. The proposed
access-demand union is not a new way to duplicate linear resources.

### Primitive rules

These rules define the proposed first-order core. They are not trusted
writer-supplied assertions. A library contract must be derived from the rules
and from checked representation invariants.

| Operation | Required evidence | State after the operation |
|---|---|---|
| Create storage | The allocating operation's capacity, layout, alignment and representation conditions, or its success branch | Fresh logical allocation identity, live storage and one accounted-for owner; initialization follows the actual operation. |
| Copy a locator | A well-formed locator value and permission to read its containing slot if loaded from memory | Same target; no new owner, liveness fact, or separation fact. |
| Read data | Live enclosing storage, active layout, initialized selected data, readable path, and all operation-domain proofs | Preserved storage; copying is allowed only for copy values. |
| Write copy data | Live compatible slot, writable path, well-typed value and operation-domain proofs | Selected slot initialized; overlapping current-value facts forgotten before exit facts are added. |
| Replace affine data | Checked transfer of old and new ownership, plus compatible live writable storage | Old value becomes the returned owner; new value occupies the same slot. No release obligation disappears. |
| Take data out | Initialized live writable slot and permission to transfer its content | Value ownership moves out; selected slot is uninitialized. Other aliases cannot read it until initialization is established again. |
| Move an owner descriptor | The source owns the value, and destination formation is valid | Source binding is consumed; backing identity is unchanged if backing storage does not relocate. |
| Reclaim storage | Its disposal authority, current liveness, and discharge of contained/dependent resource obligations required by its representation | Allocation identity permanently ceases to be live; every possibly affected current-state fact is removed. Writes include the reclaimed storage and actual management state. |
| End a stack scope | Ordinary cleanup/linearity obligations | Local storage ceases to be live before function exit postconditions are checked. |

An operation can lose proof information through possible aliasing without
proving that every possible target changed. If writing Q might change P,
forget P's old value fact; do not assert that P definitely has Q's new value.
Only must-alias information permits that strong conclusion. Liveness and
initialization use the same distinction.

Structural state matters as much as byte values. Changing an enum variant
removes the former active-layout facts. Truncating a vector can remove element
initialization; replacing its backing allocation ends the old allocation's
validity. A reference into an old field or buffer needs the corresponding
current layout and live-storage evidence before another read.

There is a deliberate difference between reinitializing a still-live slot and
reallocating storage. After `take(p); put(p, new_value)`, an alias of that slot
may be usable again. It designates a slot, not an immutable value identity.
After `free(P)`, allocating another object at the same physical address does
not restore P. Suballocation release similarly ends that reservation's logical
identity even if its enclosing physical allocation remains.

Existing deterministic affine cleanup can be preserved, with effects and
state transitions made explicit to checking. Linear values still require the
language's explicit consumption discipline. This proposal adds no hidden
user-defined finalizer. A parent owner cannot silently erase separately held
linear child obligations: a bulk-release contract must account for them, or
require an empty outstanding-child family.

### What a reference means after reclamation

An inert locator can remain in a register, field, collection, or capture. It
may be copied or discarded without dereferencing its target. Reading the slot
that contains it still requires that containing slot to be live and initialized.
Using it to access the former target requires evidence that cannot be obtained
from a new allocation's liveness.

The core should permit no pointer-to-integer operation that reconstructs access
authority, and no assertion that distinct logical generations have distinct
observable addresses. For the initial candidate, pointer identity comparisons
require current valid operands and a defined comparison domain. A later raw
address-observation API would need its own explicit semantics; it could not
turn address equality into ownership or resurrect a dead allocation.

This is a concrete policy choice, not an unresolved need for runtime generation
checks. [LLVM's object-lifetime rules](https://llvm.org/docs/LangRef.html#object-lifetime)
distinguish dereferencing from non-dereferencing pointer operations, so inert
pointer values are not inherently incompatible with a low-level backend.
That observation does not prove a particular WF lowering correct.

If an API promises that a result is immediately usable, its exit contract
includes the relevant live/initialized state. Returning a locator to a local
variable cannot satisfy that contract: scope exit destroys the local first.
An interface deliberately returning only an inert locator makes a weaker
promise. This distinction belongs in the interface, not an optimizer heuristic.

### Function boundaries and generics

```text
fn write_pair<P, Q>(x: writable Loc<P, u8>, y: writable Loc<Q, u8>)
    requires live(P), live(Q), initialized(P), initialized(Q)
    writes(P, Q)
    ensures live(P), live(Q), initialized(P), initialized(Q)
{
    store(x, 1)
    store(y, 2)
}
```

P and Q may be equal. The definition is checked once under precisely that
possibility. At `write_pair(r, r)`, the caller substitutes one actual target
for both. The signature requests one writable footprint twice; the second
store wins in source order. No runtime alias dispatch or body cloning is
necessary.

By contrast, `take(y); read(x)` cannot be verified for arbitrary P and Q.
Taking from Q may uninitialize P. A contract requiring `disjoint(P, Q)` makes
the body valid, and the caller must prove that condition. Another valid body
can restore initialization before reading. Caller knowledge does not justify
an undeclared assumption retroactively.

The call algorithm is fixed:

1. Resolve actual values and their captured associations. Infer association
   arguments by first-order matching where an argument position determines
   them. Otherwise require explicit arguments or witnesses. Do not search
   over alias partitions, higher-order qualifier solutions, or callee bodies.
2. Substitute those associations throughout types, preconditions, effects,
   state transitions, and result relationships. Preserve DAG sharing and
   normalize equal effect subjects idempotently.
3. Check required facts and ownership transfers. A pure liveness requirement
   and a consumable owner requirement remain different judgments.
4. Apply the declared changes conservatively through possible aliases, then
   establish verified exit facts. Preserve the unaffected frame.
5. Open result packages. A returned existing target retains its association;
   only a contract for actual creation introduces freshness/separation.

Ordinary type/const code specialization can remain a separate implementation
choice. An association parameter is still a genuine generic parameter even
when it erases and produces no additional code instance.

Higher-order function arguments or captures carry latent effects, target
relationships and state contracts. Capturing a locator does not capture an
unconditional promise that it will remain live. Capturing an owner follows
ordinary affine/linear function-value rules. A callback that clears, moves,
reclaims or retargets state exposes the same transitions as a direct call.

The expanded signatures above expose every premise for clarity. A concrete
surface language can derive ordinary shallow type/readiness obligations from
parameter forms and selected accesses by fixed rules. It must still export
state-changing contracts and additional domain conditions. Inferring a simple
target argument is not permission to infer arbitrary graph invariants or
strengthen a public precondition by inspecting callers.

### Stored associations and mutable fields

The identity attached to a locator is an immutable value image. The heap fact
that a field currently contains that locator is mutable and has support.

```text
holder.next = p
saved = holder.next       // saved targets P
set_next(holder, q)       // writes holder.next; exit contract says target Q
read(saved)               // still accesses P, never Q
```

Writing the field kills its former link fact through every possible alias of
the field. It cannot rewrite the already captured identity of `saved`.
The call contract of `set_next` names the entry holder and new value and
publishes the replacement link. Effect paths are resolved against their
declared entry or result images, never silently re-evaluated later.

For a fixed `Cell<Loc<P, T>>`, P is invariant: storing Q requires P = Q.
For a retargetable cell, use an existential target or a declared possible-target
family. Loading opens a target witness; an update changes the cell's current
association, not every outstanding witness. This handles ordinary structs,
opaque objects and containers through the same rule. No hidden host-origin
table is involved.

At an unknown branch result, use a finite target envelope such as `{P, Q}`
and a captured result identity X. X is not fresh storage. A write through X
can establish a fact about X while conservatively forgetting facts about
possibly aliased P or Q. Joins intersect justified state and retain sound
target envelopes. Correlations not retained automatically need explicit proof;
the checker does not enumerate all branch combinations.

Moving a box or vector descriptor can preserve its heap target. Moving the
inline storage of a self-referential struct cannot simply rename that target.
It must end the old storage identity and establish the new representation,
including any internal links. An invariant requiring intact self-links then
forces backing stability or a checked relocation procedure. Pointer fixups
are program behavior, not an unannounced runtime repair by the checker.

### Dynamic collections, graphs, and loops

A checker must not make one source allocation site stand for one live object,
nor enumerate all future runtime allocations. Use an abstract family F with
a checked ownership/representation predicate. For example:

```text
OwnedFamily(F) = separating collection of the owned resources named by F
LiveFamily(F)  = every member currently has the required live storage
```

These are defined predicates over resources. They are not automatically true
because a programmer wrote their names. Positive inductive definitions,
introduction/elimination, and named finite fold/unfold witnesses supply the
proposed proof mechanism. This extends WF's proof kernel; the current integer
fact system alone does not already implement it.

The generic resource transformations are:

```text
create:  Family(F) -> exists D. Family(F union {D})
         with a fresh live D and checked separation from current members

extract: OwnedFamily(F), D in F
         -> Owner(D) * OwnedFamily(F minus {D})

insert:  Owner(D) * OwnedFamily(F), D not in F, required separation
         -> OwnedFamily(F union {D})

dispose: Owner(D), release conditions -> consumes D's disposal obligation
```

The star separates consumable resource obligations; the union in an access
demand does not. Extracting ownership does not itself deallocate or relocate
storage. It changes where the accounting is held. An actual container
operation has ordinary runtime effects in addition to this proof accounting.

A loop's invariant describes F, the owned collection and its remaining
obligations. Check the body once under that invariant, prove the next invariant,
and check the exit state. Recursive functions use declared contracts in the
same way. A repeated allocation introduces a fresh witness per logical
execution; the static rule is quantified, not a runtime counter.

Two independently opened existential packages are not automatically distinct.
They might hide the same target. Only checked allocation, membership and
separation evidence establishes distinct live storage. Returning a member of
F is not fresh allocation merely because its result uses `exists D`.

For a graph, pointer edges and ownership edges need not coincide. A registry
can own a family of nodes while node fields contain freely copied locators,
including cycles. Removing D consumes the relevant owner and changes the
family to `F minus {D}`. A saved locator P is usable only if current evidence
still establishes its required state; `P in F` does not alone imply
`P in F minus {D}`.

An abstraction promising that every edge remains traversable must restore that
invariant when deleting a node, for example by repairing incoming edges. A
weaker graph abstraction can retain inert locators but cannot dereference them
without proof. This is the graph's chosen contract, not a graph-specific
ownership escape. Arbitrary graph algorithms are not automatically proved by
the compiler; their finite invariants and certificates carry the hard facts.

### Fields, slices, and parallel execution

Under A, several views and the original container locator can coexist.
Sequential access through any of them is allowed when its current domain holds.
There is no intermediate state in which a nominally usable parent is prevented
from touching elements solely because a view exists.

```text
left  = view(data, 0, k)
right = view(data, k, n)
read(data[i])              // sequentially legal if i is in bounds
fill(left); fill(right)   // potentially independent if their ranges are disjoint
```

Range endpoints are captured values. A later assignment to k does not retarget
a view. Unknown indices are not automatically distinct. Non-overlap is proved
from fields, current representation invariants, or captured ranges using the
fixed arithmetic/proof rules. A whole-object write includes every affected
subrange.

Parallel permission requires both complete read/write noninterference and
compatible resource transitions, operation domains, control/data flow, and
observables. Each operation's required current-state facts must remain valid
under the other operation. This is a proof about a proposed overlap of the
sequential program, not runtime permission acquisition.

If independence is unproved, keep source order. If a later operation lacks
memory-safety evidence even in source order, reject it. Neither failure inserts
a lock, generation test, alias test, dependency or scheduling protocol.
Read/read sharing needs no writer-visible fractional-token bookkeeping in
this sequential-source model. Independent overlapping writes still require
disjoint actual footprints, not two copies of the same context.

A call's aggregate row remains conservative over that call. Two long-running
calls that each briefly write the same metadata do not automatically become
independent. Smaller source operations or optional body-based optimization can
expose more opportunities; signature-only checking does not know an internal
access schedule. The proposal removes the reference-lifetime monopoly without
claiming that an unordered effect row captures every possible interleaving.

For two blocks with payloads D1/D2 and shared metadata M:

| Calls | Required result |
|---|---|
| Write D1; write D2 | Can overlap with proved separation. |
| Write D2; release D1, touching M and D1 | Can overlap if the full contracts preserve D2 and no other dependency interferes. |
| Release D1; release D2, both touching M | Source order remains. |
| Release D1; read a saved locator to D1 | Rejected for missing current liveness. |

For IO, distinguish the state operations actually share: handle fields,
open-description offset, file contents, directory namespace, device state,
provider metadata. An ordinary interface exports these associations. Two
wrappers do not prove different files; external aliasing that cannot be
disproved must use a conservative shared footprint. The language's declared
external-environment model still bounds any guarantee about external agents.

### Automatic checking and explicit proofs

The automatic fragment should contain these fixed operations:

1. First-order kind/type checking and argument matching; no impredicative
   qualifier inference or alias-partition enumeration.
2. Shared DAGs for paths, immutable value images, sets and instantiated
   contracts. Equality and supplied relation closure are finite graph tasks.
3. Structural field/range overlap queries plus the existing bounded arithmetic
   derivations. Unknown overlap remains possible overlap.
4. A forward body walk with support-based forgetting and declared join/loop
   invariants. No unbounded whole-program points-to fixpoint is required.
5. Checking written resource introduction/elimination, fold/unfold and family
   witnesses. Search does not recursively unfold arbitrary predicates looking
   for a missing proof. Witnesses name the predicate instance and substitution.

For a straightforward implementation with K retained facts and U declared
write footprints at a call, at most K times U support comparisons suffice
before adding its verified postconditions. This bounds that operation, not
all proof normalization. Each comparison's arithmetic/proof work must obey
its own specification-fixed completion rule. Certificate size and shared term
size must be counted; compressed notation is not permission for exponential
expansion hidden in one checking step.

The experiment's all-pairs symbolic case performs N(N-1) may-alias queries.
That supports a simple non-exponential implementation of this fragment. It
does not establish the complexity of recursive predicates, nested generics,
the complete WF proof system, or a production implementation.

## Candidate B: validity loans, with mutation governed by effects

B uses A's associations, ownership accounting, effect projection, alias-aware
current-value facts, signatures and parallel checks. It changes the reference
formation and destruction rules:

```text
Ref<'a, P, T> = a locator for P plus a validity obligation until 'a ends
```

`'a` means only the validity extent. P names the storage association. Neither
uses the other's syntax or kind. Overlapping writable references are legal;
they do not suspend each other for ordinary compatible accesses.

A borrow formation records a validity obligation against the selected
storage/layout. Copying the reference preserves that obligation. A returned or
stored reference retains it through the type's lifetime parameter. An explicit
end or the declared region boundary discharges it only after all dependent
holders have ended. Destruction, backing replacement and layout changes must
prove that they preserve every still-active obligation, or wait until its
declared endpoint. This does not require inference of last-use endpoints.

This is a general type rule for all objects. Blocks may hold ordinary aliased
references to shared control state and release independently after payload
view obligations end. Graphs can contain cycles within a common validity
scope. A node cannot be individually freed while references to it remain
valid under outstanding scopes; remove those references, shorten the explicit
scope, or retain the node until the scope ends.

The guarantee is storage and layout validity, not the truth of arbitrary
mutable-value facts. Clearing a possibly aliased vector still invalidates a
nonempty proof. If an operation temporarily uninitializes or changes layout
that a valid reference promises, B rejects it while that reference remains
active. This makes B meaningfully simpler and more restrictive than A,
rather than silently falling back to checked-use semantics for the hard cases.

The former `&uniq` exclusion rule is still removed. Whole-container and slice
locators can perform sequential operations normally; effects decide whether
those operations can overlap. B therefore fixes the long-lived write monopoly
without admitting inert references to reclaimed storage.

## A third option and why it is not the recommendation

Explicit location capabilities, as in [L3](https://www.cs.cornell.edu/people/fluet/research/lin-loc/TLCA05/tlca05.pdf),
can also form a whole-language discipline. Aliasable locators travel freely,
while a separate linear capability supplies access to current contents.
Functions explicitly consume and return capabilities; an effect system can
help split disjoint capability fragments for overlap.

This is a useful alternative if explicit authority flow is preferred over an
implicit resource context. It has a clear accounting model, but demanding the
capability through every helper recreates the interface problem motivating
this work. If the compiler infers that flow at each operation and abstracts it
in contracts, the proposal converges toward A; it is not an independent fourth
breakthrough. A family-wide token alone also loses per-payload independence.

Mezzo's [permission system](https://cambium.inria.fr/~fpottier/publis/pottier-protzenko-mezzo.pdf)
is another useful comparison, including implicit permission flow. Its dynamic
adoption/abandon route is not a substitute for the requested erased static
solution. None of these distinctions argues for adding a runtime borrow check
as the default fallback.

## Comparison on the same programs

| Program or property | A: current-state access | B: validity loans | Explicit capabilities |
|---|---|---|---|
| Several long-lived writable aliases, sequential writes | Accepted with current access evidence. | Accepted while validity obligations hold. | Accepted while the separate capability is supplied. |
| Two identical arguments to a sequential writer | Access demand is combined; accepted. | Same. | Needs an alias-permitting capability interface, not two disjoint tokens. |
| Delete graph node while unused locators remain | Allowed; later invalid access fails. | Delete fails until validity obligations end. | Allowed if aliases do not retain the consumed capability. |
| Take a field, do unrelated work, then put it back | Allowed; intervening reads of the hole fail. | Rejected if an outstanding reference promises the affected validity. | Requires a state-changing capability interface. |
| Return/store a reference | Association plus exported current-state contract. | Association plus retained lifetime obligation. | Pointer plus appropriate packaged or separately supplied capability. |
| A growing runtime collection | Inductive resource/family contracts. | Also needs family/ownership accounting; lifetime grouping can simplify validity. | Inductive capability packages; explicit splitting/recombination. |
| Independent views of a common backing object | Proved range effects. | Same, plus validity constraints on structural mutation. | Split capability fragments plus range effects. |
| Shared hidden IO state | Ordinary association and effect contract. | Same. | Shared association plus explicit capability flow. |
| Default reference mental model | Address relationship; each use must be justified now. | Address relationship with a declared validity promise. | Address relationship; bring its separate authority to use it. |

The old WF system remains the baseline for implementation comparison, but it
does not admit these aliasable writable references. Neither A nor B can be
implemented by merely renaming the current store brand or adding another
generic parameter while preserving OWN-5.

## Safety argument and what remains to prove

The proposed invariant relates a checked symbolic state to a physical heap:

1. Every retained current-state assertion holds in every physical state allowed
   by the target-relation environment. Unknown aliases remain included.
2. Every usable storage identity maps to its actual live allocation/range and
   layout. Proved separation implies disjoint relevant physical bytes in the
   same state. Parent/child containment is represented explicitly.
3. Every disposal obligation is accounted for once. Splitting or combining a
   family preserves that accounting; aliasing a locator cannot create an owner.
4. Every runtime access belongs to the instantiated declared footprint and
   has the required live, initialized, typed and domain evidence.

Preservation follows the intended primitive cases: creation establishes a
fresh logical identity for valid storage; writes forget overlapping facts;
takes remove initialization; reclaim removes validity; copies preserve target
identity; owner transfer preserves the backing map; field replacement changes
only the stored link and its dependent facts. Call substitution needs a frame
lemma and universally checked contracts. Loops need invariant induction;
resource packaging needs introduction/elimination and substitution lemmas.

For reuse, retain a historical interpretation of locator values while removing
the reclaimed identity from the live-storage map. A new identity can map to
the former physical range when that range is available. An old locator cannot
pass the live-state premise. No runtime generation check is needed by this
argument. Allocation capacity, overlap, layout, provenance and all actual
allocator actions still require a verified representation refinement.

For parallelism, the proof must combine footprint noninterference with
resource/state stability and the existing observational-equivalence
requirements. Separately, lowering must erase proof/association arguments,
preserve logical-to-physical storage relations, and avoid blanket `noalias`
or invariant-load attributes on aliasable mutable references.

These are proof obligations with proposed rules and invariants, not completed
theorems. The experiment tests the first-order alias/state cases, real slot
reuse, ordinary shared links, and signature substitution. It does not verify
inductive resource predicates, arbitrary higher-order callbacks, full array
layout, native lowering, or a WF-wide soundness theorem. Source-level and
backend performance are unmeasured.

The remaining work is therefore distinguishable from a missing mechanism:
formalize and prove this calculus; implement/check the family certificate
rules and recursive/generic cases; and measure representative graph,
container, IO and parallel workloads through a real backend. A counterexample
or impractical certificate/checking growth would reopen the recommendation.
Neither a green bounded model nor this design authorizes adoption or a claim
that the existing borrow checker can already be deleted.
