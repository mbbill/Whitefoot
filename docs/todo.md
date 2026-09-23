# Defects and follow-up work

Known defects, capability gaps, unresolved costs, and improvement opportunities
found during design or implementation, including unverified ones. An unverified
opportunity is a validation task: state its expected benefit, uncertainty, and
criterion for deciding whether to pursue it. Entries do not select a design.
Remove an item when its implementation and checks land, or its validation
concludes with a recorded disposition; retain any selected follow-up work here.

- **Modular incremental checking and optimized code generation.** The compiler
  currently rechecks one source bundle and emits one LLVM module. The proposed
  [architecture](../research/investigations/modular-compilation/DESIGN.md)
  targets self-contained public `.wfm` interfaces, reusable source proofs and
  generic instances, dependency-tracked cross-module optimization and ordinary
  full final linking. Complete public declarations are checked against ordinary
  implementations; direct `.wf` files share one local namespace and each
  directory's fixed `module.wfm` owns its path-derived module. One project-root graph file is the
  proposed sole dependency authority, with ordered module rows listing exact
  earlier targets; `.wfm` and `.wf` do not duplicate dependency lists.
  File-local alias headers may abbreviate canonical module or declaration names
  without granting graph edges, creating new identities or exporting aliases;
  `.wfm` carries all aliases needed to read its own API. This remains an
  amendment, not implemented capability. Specify the complete `modules.wfg`,
  interface/source and alias grammars with qualified-name factoring; naive
  segmented paths make value/call alternatives share `IDENT ::` prefixes.
  Validate the implicit primary root at the sole graph's directory and the
  fixed `pkg::` qualifier in graph/interface/implementation contexts. A
  library's internal qualifier must bind to its selected owning root, not the
  consuming application's root; it must not require a user-chosen app/lib name
  or activate the library's own graph. Qualify explicit external names such
  as std independently from the compiler-owned prelude. Complete the external
  binding format and dependency-name environments before claiming reusable
  cross-package source: test import-name changes, multiple selected versions,
  references to the primary root from another root and cache separation when
  equal textual paths resolve to distinct source identities. These are open
  design/validation tasks, deferred because the current demo has one source
  package; reopen when external-library composition is explicitly selected.
  External-library binding and library-to-library dependencies are deferred
  beyond the next single-package implementation; no binding syntax, version
  resolver or graph-import tooling is selected. Do not infer edges from
  dependency-name bindings or imported graph files.
  Validate canonical roots, row uniqueness, earlier-target
  checks, graph closure and public/implementation lookup against the same row.
  Reject undeclared references even to earlier or transitively reachable
  modules. Qualify parent/child edges in either permitted order and acyclic
  cross-branch interleaving without namespace moves. Check graph-only edits,
  used-edge deletion and order-only changes without global-position churn or a
  whole-file digest in every body key. Count moved graph rows and concurrent
  editing conflicts separately from source changes required by actual API use;
  one file does not promise one-line repairs or conflict-free collaboration.
  These costs remain unmeasured; compare matched dependency edits before
  claiming an advantage over distributed ranks or subtree ordering.
  Validate one graph with multiple named entry targets and target-scoped
  no-heap requirements. Separate architecture formation, declared module
  composition and conservative concrete execution closure; preserve normal
  checks for unused definitions in selected modules. Build a no-heap kernel
  and a heap-using tool sharing a library, including a module with an unused
  allocating helper. Qualify entry arguments, generic actuals, private layouts,
  derived release and required native/runtime supplies; optimizer removal
  cannot excuse a reachable heap requirement. Ensure object selection does not
  require heap infrastructure solely for unused helpers. This revises STOR-8's
  unit-wide spelling ban and needs a complete deterministic closure judgment.
  Exercise check-only/all-module checking, stable namespace permissions across
  targets, working-directory independence and distinct target/entry/requirement
  invalidation without duplicating shared source proofs. Keep unused implicit
  prelude availability under its existing rules. Exercise
  alias file isolation, domain/case/collision checks,
  wrong targets, private access, chains and attempted re-export. Compare
  same-target renaming with retargeting and body moves into a different alias
  environment; retain ordinary scope invalidation when an unused alias collides.
  Validate complete interface/qualified grammars, normalized declaration
  correspondence, public semantic closure, order-independent top-level
  formation and checked abstract nominal capabilities/representation. Qualify
  the sole directory/module.wfm layout, rejection of the old sibling and
  repeated-name layouts as interface lookup alternatives, direct directory
  membership without child collection, optional registered root modules, path/case/alias
  ambiguity, graph-registered namespace/declaration collisions, exclusion of
  unregistered modules and cross-file private calls. Reject executable function
  bodies in .wfm, including getter bodies. Qualify public records with direct
  component moves/borrows and abstract public structs with complete declared
  capabilities and one private representation; retain existing opaque/readonly
  rules, reject private field access and do not invent reference-returning
  getters under REF-3. Compare private-layout and public-schema invalidation
  and prove useful by-value no-heap use without forced handles. Exact abstract
  capability syntax and logical getter admission remain unqualified.
  A type with both public fields and private representation reopens the
  whole-record visibility question; assume one agent per module and leave
  coordination inside the module outside this discussion. The current
  candidate still selects only
  full public schemas or fully hidden fields. Evaluate the
  [mixed-representation questions](../research/investigations/modular-compilation/DESIGN.md#questions-for-a-mixed-public-and-private-representation)
  before selecting field visibility: compare a checked partial interface with
  one complete representation against composition with an abstract member.
  Require useful direct access where needed, complete hidden-field ownership
  and initialization, explicit capability correspondence and justified
  invariant/effect handling. Use both independent public data and a visible
  length tied to hidden storage; distinguish inspection from mutation. The
  expected benefit is direct public data access without exposing the complete
  representation, but declaration complexity and performance remain unverified.
  The [field-operation candidate](../research/investigations/modular-compilation/DESIGN.md#field-visibility-and-structural-operations-discussion-candidate)
  proposes an explicit complete/hidden schema distinction, one complete private
  definition, ordinary visible-field access and function-mediated construction
  and destructive extraction for hidden representations. These are unselected
  options, not admitted syntax. Compare against explicit residual-ownership
  permissions and composition with an abstract member; an unchanged interface
  must not acquire different extraction rights solely from hidden linear fields.
  Preserve current readonly and opaque meanings. This design
  and validation work precedes adopting a mixed form; remove it when the
  comparison selects or declines that extension with its required evidence.
  Public declaration duplication and useful module sizes remain unmeasured.
  Preserve ordinary privacy without transitive access or parent/child
  privileges. Subtree-private separately compiled modules remain unselected:
  reopen for a concrete consumer that cannot use one module's private files.
  The prior private-contract composition gap is unresolved:
  accessor facts in a body cannot express a private requirement in a wrapper
  or function-kind formal while FN-8 forbids ordinary calls in contracts.
  The declared-getter direction keeps the public callable in .wfm and its body
  in .wf, with proposed logical use of the same callable rather than a parallel
  getter API. It still needs a typed total logical interpretation, finite
  checking, state/support/write-kill rules, precise public effect correspondence
  and independently checked realization; EFF-3 pure or read-only alone does
  not supply these. Validate a GrowVector wrapper and function-kind formal
  preserving requirements, runtime/logical getter agreement and useful
  footprints without runtime proof work; name aliases do not supply that mechanism.
  Copying hidden paths into a public interface does not satisfy self-containment.
  The [complete queue demo](../research/investigations/modular-compilation/demo/README.md)
  provides a smaller source witness with provisional notation, not validation.
  Its constructor also requires abstract aggregate-result observations,
  proof-only result views and transport through construction/return/binding;
  merely admitting getter calls does not supply those FN-8/FN-9 extensions.
  Qualify both entries and the documented rejection/edit probes through the
  real module compiler when available, including getter realization and
  runtime-result agreement. Retain the broader GrowVector and precise-effect
  criteria; the small FIFO does not resolve them.
  Establish component-proof composition, exercise graph-edge deletion and
  SCC changes despite acyclic module imports, and qualify complete LLVM
  planning/cache dependencies. Compare one indivisible LLVM unit per module
  with file-independent backend partitions, separating frontend/proof reuse from
  LLVM/object rebuild cost, runtime quality, peak memory and final linking.
  Require cold/incremental agreement, no unchanged-body proof work on a no-op
  build, precise isolated-edit invalidation, and causal investigation of every
  repeatable runtime loss against matched optimized baselines. Fragment sizes,
  persistence cost and optimizer integration remain unmeasured. Implementation
  is deferred because this work delivers the design; reopen when the owner
  selects its revision for implementation, and remove this entry when the
  stated end-to-end evidence lands or the direction is explicitly superseded.

- **Ordered Vector consumption still makes avoidable transfers.** The ordinary
  prefix-window library reverses a removed suffix before consuming it in
  original order. It is O(n), but the
  [matched native comparison](../research/experiments/container-representation/vector-library/RESULTS.md#lowering-attribution)
  retains three whole-record transfers per reversed pair that a direct
  consumer does not need. With retained helpers, the 4096-element 256-byte
  reuse chain is 18.4 percent slower than direct C; ordinary optimization
  also exposes a separate WF/reverse-C gap and short-vector overhead. The
  Slots wrap-arithmetic repair does not remove either source-required movement
  or every lowering cost. Retain the tested composition as the current
  implementation, without claiming minimum-transfer or general native parity.
  Reopen before relying on ordered consumption in a performance-critical
  container: compare a representation or operation that avoids reversal under
  the same original-order, disjoint-callback, nodrop-ownership contract, and
  separately attribute alignment/alias facts and ordinary inlining against
  the retained-helper controls. No new language operation is selected yet.

- **Measure placement stops at Box content.** Destructuring an owner with a
  `Box<Slots<T>>` field loses established facts about its `.inner.len`;
  `free_empty` on the resulting binding then fails OP-14. The exact
  [Vector example](../research/investigations/containers-and-resources/X1-LIBRARY.md#vector-source-obligations)
  is a naming event covered by MSR-3, whose implementation's `measured_paths`
  currently traverses inline nominal fields but stops at a Box. The Vector
  can consume its sole storage field directly, so this does not block its
  cleanup. Repair the general placement path when a consumer needs the
  destructured or rebound owner; account for recursive nominal types without
  enumerating infinitely many content paths and test kills as well as fact
  retention.

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
- **Descendant references retain precision opportunities.** A write through a
  widened range can discard its previously established length facts, and
  independent cursors within one descendant cover cannot use suffix spelling
  alone to establish separation. The
  [cursor investigation](../research/investigations/wildcard-path/DESIGN.md)
  records these limits and the current checking-cost qualification. Preserving
  unaffected extent facts or proving a relation between independently selected
  targets could reduce repeated bound proofs and admit more range-edit programs;
  the benefit and a sound representation remain unverified. Defer this work
  because the maintained list/tree/cursor program needs neither extension.
  Reopen when a concrete program needs that precision. Validate the proposed
  gain with positive editing cases, ancestor/window/stale-capture negative
  controls and the investigation's checking-cost criterion; do not equate
  targets merely because their covers agree. Close this item when the gain is
  implemented and qualified or the measured tradeoff supports declining it.
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
  In the [post-x1 comparison](../research/investigations/proof-certificate-architecture/CHECKING-COST.md#post-x1-selection),
  256 independent inequality pairs with 256 uses still take a median 2.337 s;
  the same context with only three uses takes 0.264 s. Reusing the ordered
  affine index within a certificate removes repeated premise preparation,
  but complete matrix/index construction and long-target AUTO traversal
  remain. This is not certificate-length cost alone: a fixed three-pair
  context admits all 4096 uses in 377 ms. The 512-pair context was accepted
  in exploratory runs; these results establish neither linear total cost
  nor a universal cost for the full use ceiling.
  Preserve the complete [ENT-6]/[PRF-1] rules when investigating that cost.
- **Ordinary-fallback views still copy a fact state per materialization.**
  The [current comparison](../research/investigations/proof-certificate-architecture/CHECKING-COST.md#post-x1-selection)
  checks `tests/programs/fixed_run_library.wf` in 134 ms and
  `tests/programs/wfgrep.wf` in 834 ms. In `materialize_closure_at` in
  [`semantic/entailment/state.rs`](../compiler/src/semantic/entailment/state.rs):
  whenever a selected proof depends on a postcondition call, it clones the
  state, removes the call-dependent candidates and closes that view again.
  A [query-only ordinary projection](../research/investigations/proof-certificate-architecture/CHECKING-COST.md#ordinary-fallback-attribution-and-candidate)
  passed the transition checks but improved fixed-run only 1.03x and left
  wfgrep unchanged, so it was not retained. Revisit the representation when
  a current workload attributes a substantial share to this path. Kill-time
  edge insertion and derivation interning for recreated cells also remain.
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
  slot state.** The maintained
  [owning-map witness](../tests/programs/containers/owning-behavior.wf)
  uses `interface Key<K: drop, E>`, so it covers copyable and droppable keys,
  including Box owners, but excludes keys with a must-consume obligation.
  An unconstrained `K` also admits those linear values under OWN-1 and
  PROV-6. A numeric phase alone cannot prove that a returned enum slot is
  vacant; an occupied variant still contains a key that must be consumed.
  Investigate an ordinary state encoding or checked variant-state relation
  that lets rehash move every must-consume key without an impossible cleanup
  branch. Retain occupancy as program data, and do not add an implicit
  discard merely to satisfy the checker.
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

- **Mutual tail transfers.** [FN-10](../spec/kernel-spec.md) admits direct
  self calls through `return musttail f(...);`. Extending the guarantee to
  a different function needs a matching tail-call ABI and target evidence;
  the current parameter reassignment and entry jump cannot cross a function
  boundary. Reopen when a real mutually recursive program needs that bound.
- **Totality and recursion-depth proofs.** Domains that need determinism about
  resource use will need proved totality (termination) and proved recursion
  depth as obligation families; the atomic in-place update deliberately
  requires only a function that returns the place's type with no failure exit.
  The current recursive-cleanup stack cost is a separate compiler limitation
  recorded above; the call-site `musttail` guarantee covers only retained
  activations at marked self transfers. Neither is an implemented
  source-level recursion-depth proof.
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
