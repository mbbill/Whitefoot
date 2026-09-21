# Known compiler defects and open costs

Defects, capability gaps, and unresolved costs of the current compiler. None
of them is a decision. Remove an item when its fix and test land.

- **Parallel grain policy needs a dedicated study.** Captured extents are a
  provisional scheduling input, not an established broadly suitable policy.
  The [first same-source trial](../research/investigations/compute-model/DESIGN.md#runtime-extent-trial-result)
  improves prefix, histogram and stencil, but makes chain-pull 51 percent
  slower at two workers and incurs substantial CPU costs in some faster
  cases. Those measurements precede the continuation-accounting correction;
  its performance has not been remeasured. Study whether a robust common
  policy exists or workload, input shape, worker count and hardware require
  different choices, comparing wall time, CPU and scheduling/profile overhead.
  [Runtime profiles and PGO](ideas.md#parallel-grain-policies-and-runtime-profiles)
  are candidate inputs to that later study. The trial's failures remain
  evidence, not proof that no broadly useful strategy exists. Close this item
  when a policy meets explicit representative criteria or its accepted
  tradeoffs are recorded.

- **Stable scatter has low parallel utilization and unresolved costs.** The
  [reference-model trial](../research/investigations/compute-model/DESIGN.md#reference-model-scatter-result-2026-09-20)
  removes the owned tally/packing transfers and verifies nonempty helper work
  in both input partitioning and output packing. At eight workers its mixed
  input uses roughly 2.63 occupied CPUs for Whitefoot and 3.96 for oneTBB chain,
  from process-CPU/wall-time medians; this includes runtime work and does not
  identify the remaining cause. Attribute wall time, CPU time, runnable work and
  worker activity to block partitioning, count tally, packing and final copy.
  Hold the algorithm and representation fixed for scheduling controls, and
  distinguish insufficient parallel work or a long serial critical path from
  available work not reaching workers. Both implementations have a packing
  chain and final two-way copy, with different recursive-budget realizations.
  Padded initialization, partition payload construction, linear packing span
  and final copying remain costs; use the attribution to choose between task expansion,
  scheduling, critical-path reduction and a balanced destination representation.
  Reestablish the baseline after the reference-model merge using the
  [post-port attribution boundary](../research/investigations/compute-model/DESIGN.md#post-port-attribution-boundary-2026-09-21).
  The earlier indexed-aggregate copy candidate was not performance-qualified;
  revisit it only if current emitted code still exposes that cost on identical
  source. Its old phase injector targets a retired ABI and is not a current tool.
  Preserve stable order and machine-checked bounds. This local investigation
  precedes the separate general grain/profile/PGO study. Remove this item when
  the cause is established and the trial's work, space and measured-cost
  criteria are met, or its remaining tradeoffs are accepted.

- **The formal compute comparison has unresolved attribution and measurement costs.**
  [Hosted observations](../research/investigations/test-economy/redesign.md#identical-image-host-control-failure)
  include an identical-image stencil control failing the unchanged three-percent
  band, and a separate actual records comparison failing at two widths while
  later runs retain a one-width suspect. A null failure supplies no compiler
  regression verdict, and a later pass does not explain an earlier failure.
  Attribute host/sample variability separately from emitted code, linked layout
  and runtime changes before changing a policy or declaring the suspect noise.
  The [PR 70 comparison at `7044db24`](https://github.com/mbbill/Whitefoot/actions/runs/35539977014)
  still fails for `records`: baseline/candidate wall-time ratios are 0.938915
  at two workers and 0.882544 at four, adverse in all five pairs at both
  widths; the other four kernels pass. Identical-image and intentional-slowdown
  qualification steps pass. The subsequent
  [bounded capture repair](../research/investigations/access-effects/parallel-array-captures.md)
  passed the unchanged formal comparison at every width: `records` ratios were
  1.087361, 1.004834 and 1.060012 at W1, W2 and W4, and all five kernels passed.
  Its identical-image control nevertheless retained a `records` W4 suspect at
  0.962815708 with four adverse pairs. The concrete PR 70 regression is repaired,
  while its cause and the earlier and remaining control variation are not
  attributed. Keep this item until those observations and the resulting
  measurement/detection tradeoff are explained.

- **Recursive cleanup has no general bounded-stack lowering.** The current
  emitter recursively calls release actions, so machine-stack use can grow
  with owned value depth; its stack ledger reports the release cycle. The
  [continuation models](../research/investigations/access-effects/cleanup-continuations/README.md)
  demonstrate fixed-stack, nonallocating walks only for their selected layouts.
  They establish neither an encoding for all WF types without extra object
  fields nor its impossibility. Retain the existing lowering while researching
  how every suspended aggregate, enum, array and window traversal records its
  continuation. Preserve reverse binding order, declaration order within
  aggregates, logical window order, and content-before-Box-free order. Close
  this item when a general implementation and native regressions establish
  those properties, or a different resource tradeoff is selected explicitly.
- **Retired implicit empty-window release leaves unused proof scaffolding.**
  No source operation constructs the checked `EmptyRun` release mode, but its
  release-graph branch, obligation family and derivation plumbing remain.
  This is maintenance debt, not a promise to restore implicit dropping of
  linear windows. Remove the unused paths when next changing cleanup or its
  proof inventory, retaining `free_empty` and its active OP-14 requirement
  diagnostic; the similarly named diagnostic is not the retired mechanism.
  The current semantic fixes take precedence over this deletion. Close the
  item with the normal release and explicit-empty-release regressions intact.
- **Box/window representation costs remain unqualified.** The current runtime-
  capacity Box is one pointer to one header-first allocation; `grow` uses
  allocation, memmove and free. A one-word owner, one allocation and header
  placement are distinct choices: a fat descriptor can also own one element
  allocation and make measure reads direct, while widening transport and
  capture storage. Neither alternative is established as generally faster.
  Keep the current implementation while separating owner width, measure loads,
  allocation count, copying and linked layout in representative single-thread
  and parallel comparisons. The successful bounded capture repair above is
  evidence about the synthesized task ABI; it neither attributes the earlier
  `records` failure nor proves that any one general layout choice caused it.
  Keep the deferred general representation study separate, and close this item
  only when the relevant costs and chosen tradeoffs have discriminating evidence.
- **Loop reference abstraction needs practical precision and cost evidence.**
  Current loop headers keep possible roots and static path shapes, give
  potentially rebound endpoints finite opaque capture identities, and solve
  owner-tagged validity dependencies over entry and executable backedges.
  This prevents a current iteration's facts from authorizing a previous
  iteration's reference. Its precision and checking cost on larger real loops,
  nested loops and joined targets remain unqualified. Investigate useful facts
  lost at headers and the evidence needed to recover them without merging
  distinct evaluations, dropping possible targets or imposing an acceptance
  budget. Close this item with representative positive and hostile cases,
  cost measurements, and any required precision repair or explicit limitation.
- **Runtime-capacity Array element suffixes retain a flat-buffer limitation.**
  A valid field selection such as `values.inner[i].field` on a
  `Box<Array<CopyStruct>>` can still reach `CompositeValues` instead of the
  general storage-place path. The checker resolves the suffix before reporting
  this capability gap; it is not a source-language rejection. Whole-element
  reads into a copy local and whole-element replacements avoid this path,
  while range-reference element suffixes already use the general path. Unify
  the remaining flat-buffer projections with it and cover field reads, writes,
  and borrows before removing this item.
- **Pair-scoped parallel proofs need scaling and coverage work.** The current
  PAR-1 planner constructs questions for every ordered source pair in a segment
  and retains range separation only for that pair's first-statement state;
  repeated visits meet with logical AND. A segment of n members has n(n-1)/2
  pairs, but that logical requirement does not mandate quadratic repeated
  proof work. General index mapping through the first member's `ensures` is
  still unavailable; missing evidence keeps sequential lowering. Investigate
  indexing and reuse without losing statement identity, captured endpoints,
  flow context or all-pairs composition. Close this item when larger segments
  have measured costs and the intended proof coverage, retaining guarded,
  nonadjacent and stale-capture negative controls.

- **Large entering proof contexts still have substantial checking cost.**
  In the [pinned row-summary comparison](../research/investigations/proof-certificate-architecture/CHECKING-COST.md#row-summary-selection-2026-09-15),
  256 independent inequality pairs with 256 uses still take a median 5.50 s;
  the same context with only three uses takes 0.626 s. Query-preparation reuse
  and conservative closure-product pruning remove repeated and non-improving
  work, but complete matrix/index construction and long-target AUTO traversal
  remain. This is not certificate-length cost alone: a fixed three-pair
  context admits all 4096 uses in 295 ms. Larger growing contexts remain
  unmeasured; these results establish neither linear total cost nor a
  universal cost for the full use ceiling.
  Preserve the complete [ENT-6]/[PRF-1] rules when investigating that cost.
- **Ordinary-fallback views still copy a fact state per materialization.**
  After [incremental closure](../research/investigations/proof-certificate-architecture/INCREMENTAL-CLOSURE.md#selection),
  the [retained-proof follow-up](../research/investigations/proof-certificate-architecture/INCREMENTAL-CLOSURE.md#retained-proof-follow-up-results)
  checks `tests/programs/fixed_run_library.wf` in 1.21 s and
  `tests/programs/wfgrep.wf` in 0.94 s. The previously attributed largest
  fixed-run cost is `materialize_closure_at` in
  [`semantic/entailment/state.rs`](../compiler/src/semantic/entailment/state.rs):
  whenever a selected proof depends on a postcondition call, it clones the
  state, removes the call-dependent candidates and closes that view again.
  Kill-time edge insertion and derivation interning for recreated cells are
  the next costs.
- **Connection-level concurrency is not supplied by ordinary source order.**
  A loop that accepts and serves connections in source order
  completes the current handler before entering the next, so a handler waiting
  on a silent peer holds up every later connection, and 1024 open connections
  are not 1024 independently resumable handlers. The source is accepted and
  compiled through ordinary calls. The retained multi-client TCP protocol can
  wait forever when the first handler awaits EOF while clients close only
  after every peer has finished; the
  [C2 measurements](../research/experiments/io-completion-bench/C2-RESULTS.md)
  record that noncompletion without a throughput result. No replacement
  interface has been chosen. `WF_STACKS` is inert: the runtime has no
  switchable-stack pool for it to size, so it is neither read nor validated.
- **Acyclic generic instantiation has no established practical bound.**
  D7's unchanged-argument cycle rule establishes termination while acyclic
  fan-out may still require exponentially many instances relative to written
  source. The owner deferred this question in D7, whereas the current language
  design rules out exponential checking work. The
  [behavior investigation](../research/investigations/containers-and-resources/BEHAVIOR.md#shared-semantic-boundary-and-exact-deltas)
  records the accepted 1343-byte / 2047-instance witness, same-instance controls,
  stage measurements and unresolved correspondence finding. No budget, timeout, new
  source refusal, or measured asymptotic guarantee has been selected.
- **At most eight peers may wait at once on a host without a native ring.**
  On Darwin, and under `WF_IO_NO_NATIVE_RING`, a peer wait beyond the eighth
  concurrent one has no helper and queues with no timeout. The readiness-
  driven adapter that would lift this, one poll over every queued descriptor
  from inside the park, was never built.
- **A `propagate` statement cannot be a [PAR-1] window member.** The rule
  admits only `let`-bound and scrutinee calls, so `let a = f(); let b =
  propagate g();` never overlaps. Allowing a `propagate` second member would
  need the lowering to join the hand-out before the `Err` return; a future
  investigation, taken up when a real program shows the gap.

## Open language questions

Questions the owner has left open on purpose. None of them is a decision;
each is resolved by a discussion and a tree change.

- **Sparse containers over must-consume linear elements need ownership-visible
  slot state.** The behavior-map growth witness previously wrote
  `formal Key<K: linear, ...>` while replacing `progress.held` and disposing
  the returned `Slot<K>`. That depended on a compiler defect which failed to
  apply PROV-6 to a symbolic linear bound. The current checker correctly
  rejects `dispose previous_held`: a numeric phase does not prove that the
  returned enum is `Vacant`, and an `Occupied` value contains a `K` that must
  be consumed. The maintained witness is narrowed to `K: affine`, which still
  covers its scalar and store-branded owning instances. Investigate a state
  encoding or checked variant-state relation that lets rehash move every
  must-consume key without an impossible cleanup branch; do not add a discard
  behavior merely to satisfy the checker.

- **Local region introduction and explicit region blocks.** Revisit whether
  an ordinary function body should introduce a local region, and which
  borrows need a writer-spelled `region` block. In the
  [buffer checksum case](../tests/conformance/cases/x-buffer-mutable-checksum-run.wf),
  a region encloses allocation and the `place_back` calls that fill the vector.
  The temporary loans already end at their statement boundaries under OWN-6;
  their region's formation and storage-validity extent is a different matter
  under OWN-3 and OWN-10. Compare explicit blocks, function-body regions and
  implicit regions for non-escaping temporaries without conflating those two
  boundaries. Cover locals declared partway through a body, bound holders,
  surviving views, returned borrows, loops and control headers; preserve
  storage validity and exclusivity with deterministic checking. The owner
  requested this investigation during PR #30 review; no alternative is selected.
  Defer bulk cleanup of the repeated per-call region wrappers in migrated
  tests until this question is settled, preserving each case's intended
  behavior or rejection reason when the selected spelling is applied.
- **Last-use endpoints for ordinary borrow holders.** Investigate ending a
  `let`-bound shared or unique borrow after its last required use instead of
  retaining it to region-block exit under [OWN-4]. Keep loan liveness separate
  from region selection and type validity: this need not introduce inference
  of region arguments from expected result types or later uses. Shared
  `Slice` values already have last-use endpoints under [OWN-5]/[VIEW-1] in the
  [current specification](../spec/kernel-spec.md). Cover reference copies,
  returned borrows, surviving child loans and unique-parent suspension,
  branches, loops, and statement-scoped temporaries. Compare the current
  lexical endpoints with deterministic, terminating last-use analysis while
  preserving storage validity, exclusivity, and signature-only call checking.
  No change to the ordinary borrow rules is selected.
- **A view-valued match or if.** [OWN-5] rejects a `match` or `if` expression
  whose value is a view, rather than joining the arms' origin sets, which the
  origin machinery could represent. If the join can be admitted it should be;
  until then the rejection stands without a recorded reason.
- **The automatic-fact menu is a leftover.** [ENT-3] admits a narrow and
  asymmetric set of arithmetic idioms as automatic facts, each added for one
  proof pattern, with no general criterion and no counterpart for rows it
  omits, such as a lower bound from `ior`. The owner wants it made principled;
  nobody has had the time.
- **The two-premise cutoff of automatic affine derivation.** [ENT-6] tries
  zero, one, and two premises and no more without a written certificate. Why
  the line sits at two, against one or three, is not remembered and needs a
  study before it is recorded.

## Ownership redesign (candidate x1) follow-ups

Items the owner asked to be kept on this list during the redesign recorded in
`research/investigations/access-effects/CANDIDATE-X1.md` and adopted into
`design/language` on 2026-09-19. None of them is a decision; each names the
condition under which it is taken up.

- **Iterative descent of owned links by reference (wildcard path).** A path
  has a static shape, so `loop { set p = &deref(p).next.Some.value.inner; }`
  over a Box-linked list is refused and the walk is a tail recursion or a
  pool with an index. Owner's direction (2026-09-20): add the wildcard path
  after PR 70 merges, because it is purely additive and costs the compiler
  almost nothing. Design on record, needing no new syntax because a
  reference's path is never written: when a loop-carried rebinding extends
  the reference's loop-entry path through itself, the checker widens the
  path to `R.**` ("somewhere under R") and rechecks the loop body once to
  its fixed point. Rules: (1) `R.**` overlaps every path at or under R, one
  prefix test; (2) while `p` is valid, a write, move or free of a place
  under R that does not go through `p` invalidates `p`, except a write of a
  primitive leaf field, which is a prefix of nothing; reads are free; (3) a
  write through `p` of a non-leaf place invalidates every other reference
  under R and leaves `p` valid. Runtime cost none (a reference stays a bare
  pointer). Checked against: tree descent through either child, a cursor
  reset to the root, node removal through a single cursor on the link slot.
  Known price: two live cursors under one root invalidate each other on a
  link write, and a live cursor is the whole subtree's footprint for the
  parallel judgments.
  INCOMPLETE as recorded (independent study, 2026-09-20, branch
  `research/x1-wildcard-path`, `research/investigations/wildcard-path/`):
  the basic loop is still refused, because the rebinding goes through the
  payload step `.Some.value` and [ENT-3.S15] ends the refinement fact at the
  arm's exit, which [REF-2] makes an invalidation, so the rebound reference
  is invalid in the next iteration. The design needs a rule that a payload
  place already selected keeps existing until the enum is written; a
  widened path is a may-alias cover and never one term of the fact system;
  ancestor moves and window removals must still invalidate; "recheck once"
  must become a fixed point over a finite domain. Start from that study.
- **`musttail` at the call.** Owner's ruling (2026-09-20): a call-site marker
  named `musttail`, rejected with the failing condition named when the call
  is not a guaranteed tail call. Conditions for a self call: it is the
  operand of `return`; every reference argument's path is rooted at a
  reference parameter and never at a local of the current activation; no
  local with a non-empty release is live across the call (a local nothing
  refers to may be released before the call, release order being
  unobservable). Lower a self tail call in the compiler's own lowering as
  parameter reassignment plus a branch to the entry, so it holds on every
  target; mutual recursion needs LLVM `musttail` with a matching
  convention and is a later step. Without the marker the stack bound rests
  on an implementation obligation the writer cannot check, and a pending
  release silently breaks tail position. Implement after PR 70 merges.
- **Totality and recursion-depth proofs.** Domains that need determinism about
  resource use will need proved totality (termination) and proved recursion
  depth as obligation families; the atomic in-place update deliberately
  requires only a function that returns the place's type with no failure exit.
  The current recursive-cleanup stack cost is a separate compiler limitation
  recorded above, and the call-site `musttail` mechanism remains a separate
  follow-up. Neither is an implemented source-level recursion-depth proof.
- **Facts a contract can carry (after PR 70 merges; owner, 2026-09-20).**
  Three additive widenings, taken up together, each measured:
  (1) Affine `ensures`. A `requires` may already be an affine relation and
  enters the body as affine premises, but an `ensures` must fit the
  difference-bound template, one datum a side, so `append` and `split_off`
  cannot publish their exact sum. The affine layer [ENT-6] already holds
  arbitrary affine inequalities over immutable value atoms and proves with
  the fixed AUTO families, so publishing an `ensures` as affine premises in
  the caller changes neither determinism nor termination. Costs to
  measure first: AUTO tries every pair of premises, so checking time grows
  with the square of the premises a body accumulates; and a proof chaining
  more than two published facts needs written `use` steps. When it lands,
  restore the exact-sum contracts of `append` and `split_off`.
  (2) A `requires` stating a variant refinement (`p is Some`).
  (3) An `ensures` naming a single indexed path
  (`deref(p.slots)[h.idx].gen == h.gen`), which decides whether a guarded
  pool access pays one load, compare and branch per call.
- **Open-addressing tables with non-Copy payloads.** One null check per hit
  versus hashbrown, because occupancy that is decided by data is stored as
  data. Measure on a real table before deciding whether any mechanism is
  worth it.
- **Channel primitive.** An ownership-transfer queue in the trusted base for
  producer/consumer pipelines and work stealing; lock-free rings are not
  expressible without it and batched fork-join is the available form. Research
  when the future concurrency primitives are designed.
- **Header-plus-tail heap block.** One allocation holding a fixed header and a
  runtime-length tail (LLVM `User` with its operand list, `sk_buff`). Today a
  struct with a `Box<Slots<T>>` field costs a second allocation and one extra
  dependent memory access per hop. Additive, after PR 70 merges. Two shapes
  under discussion: (a) a struct whose last field is a runtime-capacity shape
  becomes itself Box-only content, laid out `[header fields | len | cap |
  elements]`, which needs a construction route that knows the capacity, a
  `grow` that moves the whole block, and the no-move-out rule extended to
  it; (b) one more prelude storage shape carrying a header value beside its
  window, built by a construction function taking the header and the
  capacity, which needs no new struct rule.
  Undecided (owner, 2026-09-20: revisit later). Notes for that discussion:
  the tail is always the last field and always one of the runtime-capacity
  shapes; a `Slots` or `Ring` tail starts empty, an `Array` tail does not
  (it needs a fill value and a count); a construction sketch is
  `box_new_tail::<Message>(value: Message(kind: 1_u8, flags: 0_u8, body: _),
  capacity: n)`, where the capacity is an argument of the boxing function,
  the expression with the hole is admitted only as that argument because
  such a struct is never a local value, and `_` would be a new token
  (`..` exists already as the destructuring rest marker).
- **Bitmask fact.** `x & (c - 1) < c` for a power-of-two `c`, which would
  remove the per-probe bounds compare in hash tables.
- **Handing checker facts to the backend.** Emitted since the v0.60 port:
  `noalias` (not on `swap`), `nonnull`, `dereferenceable`,
  `captures(none)` or `nocapture` by a build-time probe, `inbounds`, and
  `nuw`/`nsw` on the exact family. Not emitted: `memory(argmem: ...)` (the
  IR carries neither the declared row nor the allocation fact), scoped
  alias metadata and `llvm.loop.parallel_accesses` (the emitter has no
  metadata table). Build the metadata subsystem as its own step with a
  before/after benchmark.
- **Subscripted integer places as terms.** Today a place with subscripts is
  a term only when its last step is a readonly field. The kill machinery
  (offset support, overlapping element writes) already serves measure terms
  and whole-expression goals, so generalizing to every integer place is
  cheap in mechanism; measure its effect on closure size and checking time
  first.
- **Vocabulary no declaration can state.** The `len` of a range reference
  (`&[T]` is a kind, not a type) and the four effect-row part names `next`,
  `last`, `filled`, `free` remain specification vocabulary after the
  measures became declared readonly fields. Find a better home for them.
- **The storage shape declarations are inelegant.** `Array`, `Slots` and
  `Ring` are prelude opaque structs with readonly fields, but the
  omitted-capacity form, element storage and placement still live in the
  type rules, and a constant-capacity `cap` is a field whose value is a
  type constant.
- **Retire the class names copy, affine and linear from the specification's
  prose.** The keywords are the two capabilities `copy` and `drop` and the
  modifiers `nocopy` and `nodrop`; the three class names survive only as
  prose terms defined once in OWN-1 (copy: copyable; affine: droppable but
  not copyable; linear: neither). Rewrite the several hundred prose uses in
  capability words when a specification pass can afford the review.
- **Performance floor after the port.** Re-measure the existing kernels and
  the eight engineering tasks of the matrix rounds once the compiler
  implements v0.60, so that the recorded costs (data-determined index
  compare, refused scatter, re-descent on find-then-mutate, one element move
  into the append slot) have numbers.
