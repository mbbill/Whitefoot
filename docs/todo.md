# Known compiler defects and open costs

Defects, capability gaps, and unresolved costs of the current compiler. None
of them is a decision. Remove an item when its fix and test land.

- **A contract fact stated over `room` does not survive the window write it
  precedes.** [MSR-1]'s table fixes a window's `room` cell as `cap - len`, so
  the affine image of `r.room` is the difference of the other two images and a
  relation over it is an [ENT-4] fact about the `room` *term*. That term has
  no difference bound relating it to `len` -- `room + len = cap` is a
  three-term relation no difference-bound domain holds -- so the consequence
  the relation carries about `len` exists only as an ephemeral affine image at
  query time and dies with the term at the first descriptor write. A callee
  whose own `requires deref(window).room > 1_u64` should admit two
  `place_back` calls therefore stops at the second
  ([`call6-pos-a-row-relation-establishes-at-a-caller`](../tests/conformance/cases/call6-pos-a-row-relation-establishes-at-a-caller.wf)),
  while the same program with the requirement restated as a local `invariant`
  is accepted, because an invariant conclusion is published over the immutable
  atoms. Decide whether such a relation should also be published over its
  images, which extends [ENT-6]'s automatic premise sequence, or whether the
  translation belongs in the fact establishment itself.

- **An accepted [OP-9] site's proved count ceiling does not reach the
  allocation it authorizes.** The obligation is now submitted and refused at
  each runtime-capacity construction and at `grow`, but the numeric ceiling
  the discharge retains is not carried into the compiler-owned row whose
  lowered body performs the allocation: that row is out of line and one
  instance's body serves every call site. Target qualification therefore keeps
  the `has_call_site_bound` gate at
  [`backend/target.rs`](../compiler/src/backend/target.rs), comparing the byte
  ceiling only where the retained bound is the allocation site's own, and a
  proved 5e18-element `u16` construction is still lowered instead of being
  reported unrepresentable. Either the weakest ceiling over an instance's call
  sites must reach the row, or the row must be specialized per site.

- **A field projected after dereferencing a runtime-indexed composite element
  stops as unsupported.** The specification admits ordinary chained element,
  dereference, and field selection, but
  `deref(owners.storage[index]).id` stops in semantic checking as
  `Unsupported(CompositeValues)` with no rule or source diagnostic. The
  growable-vector executable currently borrows `owners.storage[index]` into a
  helper and performs `deref(deref(item)).id` there. Complete the general
  checked-place and lowering path for a subscript followed by dereference and
  field projection, add owning and copy-element tests, then remove that helper.

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
  [W8 mixed-input trial](../research/investigations/compute-model/DESIGN.md#stable-scatter-result-2026-09-14)
  gives roughly 1.78 average occupied CPUs for Whitefoot and 4.04 for oneTBB,
  from process-CPU/wall-time medians; this includes runtime work and does not
  identify the cause. First attribute wall time, CPU time, runnable work and
  worker activity to block partitioning, count tally, packing and final copy.
  Hold the algorithm and representation fixed for scheduling controls, and
  distinguish insufficient parallel work or a long serial critical path from
  available work not reaching workers. Both implementations have a packing
  chain and final two-way copy. Padded storage, owned take/restore and copying
  remain separate costs; use the attribution to choose between task expansion,
  scheduling, critical-path reduction and a balanced destination representation.
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
  Close this item when the source of both observations and the resulting
  measurement/detection tradeoff are established.

- **A runtime-sized `buffer_new` fails with no rule and no location.** At an
  unproved runtime capacity the driver stops four stages after semantic
  checking with `TargetLayout(Unrepresentable(RuntimeSizedAllocation))` and no
  rule id or source coordinate; the real defect is an undischarged size
  obligation. The store surface already answers it (`heap_vector` hands back
  an `Option` and [OP-9] refuses at the source with a rule and a line). The
  item is removed with `buffer_new` and `buffer_vacant`, not repaired
  separately.
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
- **Pre-kill L0 closure has an unresolved compilation cost.** Before an
  [ENT-5] invalidation batch, `materialize_before_event_kill` in
  [`semantic/entailment/flow.rs`](../compiler/src/semantic/entailment/flow.rs)
  calls `materialize_closure_before_kill` in
  [`semantic/entailment/state.rs`](../compiler/src/semantic/entailment/state.rs).
  A non-closed state with explicit relations takes the complete closure;
  already-closed, contradictory, and empty-relation states have fast paths.
  The [build/test investigation](../research/investigations/test-economy/build-and-test.md#a-compiler-hotspot-not-native-execution)
  measures 58.29 s for one optimized read-heavy LLVM-only compilation; a
  three-second semantic-checking sample puts 61.2% of leaf samples in
  `close_with_excluded_term`. Whole-run attribution and a controlled algorithm
  comparison remain necessary. A narrower projection must preserve every
  surviving consequence,
  including implicit type edges and disequality strengthening; filtering
  explicit edges alone is insufficient. No speedup is established.
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

- **Iterative descent of owned links by reference.** A path has a static
  shape, so `loop { p = &deref(p.next) }` over a Box-linked list is refused
  and the walk is recursion or a pool with an index. Revisit if the
  pool-plus-index form proves too slow or inexpressible on a real workload;
  the candidate mechanism on record is a wildcard path form (`x.**`) with
  conservative overlap and invalidation against everything under `x`.
- **Totality and recursion-depth proofs.** Stack depth is an implementation
  obligation, not a language promise: the compiler-derived release of an
  owned chain must run in bounded stack (using the freed cells as its
  worklist) and self tail calls must be eliminated. Domains that need
  determinism about resource use will need proved totality (termination) and
  proved recursion depth as obligation families; the atomic in-place update
  deliberately requires only a function that returns the place's type with no
  failure exit.
- **Facts beyond affine comparisons in contracts.** A `requires` stating a
  variant refinement (`p is Some`) and an `ensures` naming a single indexed
  path (`deref(p.slots)[h.idx].gen == h.gen`). The second decides whether a
  guarded pool access pays one load, compare, and branch per call. Take up
  when the fact language is next revised.
- **Open-addressing tables with non-Copy payloads.** One null check per hit
  versus hashbrown, because occupancy that is decided by data is stored as
  data. Measure on a real table before deciding whether any mechanism is
  worth it.
- **Channel primitive.** An ownership-transfer queue in the trusted base for
  producer/consumer pipelines and work stealing; lock-free rings are not
  expressible without it and batched fork-join is the available form. Research
  when the future concurrency primitives are designed.
- **Header-plus-tail heap block.** One allocation holding a fixed header and a
  runtime-length tail (LLVM `User` with its operand list, `sk_buff`). Costs
  one extra dependent memory access per hop today.
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
- **Exact sum contracts for `append` and `split_off`.** Their rows publish
  only `source.len == 0` (or `== index`) and
  `destination.len >= entry(destination).len`, because a contract side
  admits one datum and the exact sum needs two. A caller that needs the
  exact length reads `destination.len` after the call. Revisit with the
  contract-fact widening item above.
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
- **Generic linearity bound spelling.** `T: linear` accepts every class and
  `T: copy` only copy types; the bound names how the body treats `T`, which
  reads backwards. The current spelling never had an explicit owner ruling;
  find one that reads in the intuitive direction.
- **Performance floor after the port.** Re-measure the existing kernels and
  the eight engineering tasks of the matrix rounds once the compiler
  implements v0.60, so that the recorded costs (data-determined index
  compare, refused scatter, re-descent on find-then-mutate, one element move
  into the append slot) have numbers.
