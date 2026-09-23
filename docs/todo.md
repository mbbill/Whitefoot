# Defects and follow-up work

Known defects, capability gaps, unresolved costs, and improvement opportunities
found during design or implementation, including unverified ones. An unverified
opportunity is a validation task: state its expected benefit, uncertainty, and
criterion for deciding whether to pursue it. Entries do not select a design.
Remove an item when its implementation and checks land, or its validation
concludes with a recorded disposition; retain any selected follow-up work here.

- **Generic struct constants are not implemented.** CONST-2's `cvalue`
  grammar admits written type arguments, but
  `compiler/src/semantic/check/types.rs::parse_const_construction` returns
  `UnsupportedSemanticFeature::CompositeValues` whenever they are present.
  This is an implementation gap, not a source-language rejection. Reopen when
  extending static aggregate initialization: instantiate the exact named
  const-eligible struct, preserve declared field order and field types, and
  validate a generic struct constant's field reads against wrong-type,
  wrong-order and non-const-eligible controls. Remove this item when those
  cases pass through the ordinary compiler and conformance paths.

- **Audit numeric conversion coverage and unnecessary fallible interfaces.**
  The known starting case is integer low-bit narrowing: `cvt::<u32, u8>(x)`
  preserves the numeric value and returns `Result`, while `reinterpret` only
  admits its listed equal-width pairs. Masking with `iand(x, 255_u32)` before
  `cvt` expresses the low-byte result but still exposes `Result`; there is no
  direct total truncating conversion. Survey similar gaps across integer widths
  and signedness, bit reinterpretation, saturation, and floating-point rounding
  or narrowing, distinguishing existing compositions from missing operations.
  Use small source examples and boundary controls to define each desired
  behavior, including negative values, range edges, and relevant NaN/infinity
  cases. Assess whether a clearer total operation or proved-domain form removes
  unnecessary source branching without weakening exact conversion or proof
  requirements; inspect ordinary emitted code before claiming a runtime cost
  or improvement. Additional gaps and performance costs are unverified. Defer
  operation selection and implementation to the requested conversion review;
  reopen when that review starts or a real numeric workload needs a workaround.

- **Joined reference proofs lose useful target-relative information.** A
  reference selecting either of two freshly empty Slots cannot establish the
  append precondition from both constructors' facts; captured disjoint ranges
  formed in separate branches also lose their branch-local endpoint images
  at the join. These safe examples are rejected under the current fixed proof
  routes, rather than demonstrating an implementation violation. The
  [bounded query experiment](../research/investigations/consistency-followups/DESIGN.md#reference-joins-and-bounded-proof-precision)
  supports substituting both operands for the same selected alternative, but
  does not yet establish a complete family: current target authority differs
  from the function-wide origin inventory; Boolean and integer-domain consumers
  need uniform normalization; failed-query term registration needs inertness
  evidence; and polynomial work in an explicit target set is not a bound in
  source size. Keep the current rules until those obligations are resolved and
  matching full-origin, stale-capture, query-order and growth controls pass.
  Branch-local range images additionally need target-presence and capture-
  generation information; a plain union of branch images is insufficient.

- **Establish whether the reference-summary depth fallback is source-reachable.**
  `PlaceMap::resolve_root` returns wholly unresolved beyond 32 recursive summary
  expansions. Any unresolved child discards the whole alternative set; inspected
  proof and parallel consumers fail closed, so no partial-origin omission or
  incorrect acceptance is established. Ordinary aliases are flattened when
  recorded, and a long source alias chain is not itself a reproducer. Trace
  checked-source summary construction and test the internal boundary with a
  shallow sibling; if reachable, replace the depth-dependent precision boundary
  with source-bounded traversal and explicit cycle handling. Deferred until
  reference-summary work provides a discriminating source witness or proves the
  cap redundant; reopen before reusing this resolver for a new proof family.

- **Validate reuse of selected-target element layouts during emission.**
  [Zero-stride addressing](../compiler/src/backend/target.rs) currently queries
  the ordinary layout calculator afresh for each element-address step. Repeated
  accesses to a deeply nested nominal element may recompute the same layout.
  Compare checking/emission cost on repeated nested-element accesses before
  introducing shared layout storage; require identical qualification and emitted
  addresses. The benefit and material cost are unmeasured, so keep the simple
  query for now and reopen when measuring target-emission cost or extending its
  layout consumers.

- **Validate a shared Ring wrap calculation independent of layout bounds.**
  The corrected front predecessor handles every admitted capacity. Remaining
  address-only modular additions are justified by the positive-stride target
  bound or the zero-stride address operand; head advancement separately uses
  the safe offset one. An overflow-free common formulation could simplify
  those grounds across indexed access, shifts, transfers and cleanup, at the
  cost of more emitted arithmetic. Compare exact coordinates at u64 boundaries
  and representative native cost before selecting it. No remaining observable
  defect is established; defer beyond the predecessor repair and reopen when
  changing Ring layout or coordinate consumers.

- **Ordered Vector consumption still relocates rear elements.** The take-first
  composition exchanges an owned local with each first-half suffix slot, then
  consumes the reversed remainder. It preserves the prefix and callback order
  with O(removed) work and constant auxiliary storage, but still relocates
  `floor(removed / 2)` rear elements beyond a direct consumer's handoffs. The
  [matched native comparison](../research/experiments/container-representation/vector-library/RESULTS.md)
  separates that source cost from redundant compiler snapshots; qualified
  independent stack slots and descriptor-before-transfer takes remove the
  latter in the local Clang 21 retained-record witness. That result establishes
  neither a guarantee across optimizers nor universal native parity. Keep the
  current ordinary composition while measuring any
  concrete workload that makes its remaining movement significant; introducing
  a more general operation without that evidence is deferred. Reopen before
  relying on ordered consumption in a performance-critical container. Compare
  an alternative under the same original-order, disjoint-callback,
  nodrop-ownership, constant-auxiliary-space and O(removed) contract, including
  nearly complete retention; require an attributable measured improvement
  against direct C and the current WF implementation. No new language operation
  is selected yet.

- **Deque scalar costs remain after payload-address qualification.** The
  [paired comparison](../research/experiments/container-representation/deque-library/RESULTS.md)
  isolates the qualified index fact and reduces normal scalar forward churn
  from about 2.3x C to 1.20–1.26x, leaving that residual gap unattributed.
  Retained scalar reverse churn is about seven percent slower in the new
  production layout despite identical relevant instructions and dependencies.
  A controlled 32-byte padding experiment restores the endpoint addresses
  without reliably removing the difference, so neither endpoint placement
  nor an intrinsic assumption cost is established as its cause. The owner
  selected provisional retention of the fact with both results preserved;
  no measured application mix makes the forward gain cancel the reverse loss.
  Compare the remaining scalar work with the same source, independent oracle
  and C controls, preserving native code/data placement and recording
  execution-state variation before attributing a cost to the interface or
  choosing a production alignment policy. Require repeatable improvement in
  both measurement orders and account for other affected paths. Defer broader
  tuning while this causal question is open; reopen for a workload dominated
  by retained reverse calls, a native-toolchain change or another material
  regression under the matched comparison.

- **Upstream LLVM on Darwin does not yet support the selected stack-probe
  spelling.** The [Deque comparison](../research/experiments/container-representation/deque-library/RESULTS.md)
  records LLVM 22.1.8 rejecting native construction of the unchanged baseline
  with `Unsupported stack probing method`; the emitted
  `"probe-stack"="__chkstk_darwin"` remains present. Parsing and optimization
  succeed, and the native builder's Apple Clang path works, so this does not
  establish a failure of the new address fact. Before offering upstream LLVM
  as a native Darwin consumer, determine the supported probe form and link
  requirements and validate large-frame and recursive exhaustion through the
  existing floor tests. Disabling probes is not an acceptable workaround.
  Defer this separate toolchain extension while the current native path is
  supported; reopen when another native Darwin consumer is required.

- **Slab aggregate results retain extra transfers and layout overhead.**
  The [Slab comparison](../research/experiments/container-representation/slab-library/RESULTS.md)
  separates the one-slot cell's extra word from its helper boundary: retained
  wide removal and consumption has three 256-byte transfers in WF versus one
  in C even though both `Option<Record>` results occupy 264 bytes. The separate
  insertion `Result<SlabHandle, Record>` occupies 280 bytes in WF's product
  layout versus 264 in C's union ABI. Keep these distinctions when interpreting
  timing; a cell-layout change alone cannot remove these costs. Validate
  forwarding or result placement with
  the same owning return paths, failed insertion returning the offered owner,
  partial cleanup and alias controls, checking optimized transfers and
  same-source timings on supported toolchains. Defer general enum layout and
  call ABI changes until that experiment establishes which transfer can be
  removed without changing ownership; reopen with the owning-map library or
  a workload dominated by wide Slab removal.
  The [map's exhaustive returned-owner protocol](../research/investigations/containers-and-resources/X1-LIBRARY.md#generic-owning-map-trial-after-the-ring-comparison)
  also supplies an ordinary enum alternative to reassess for Slab's extra
  cell word. Its fit and cost for stable slots, generation retirement,
  exhaustion and removal are unverified; compare that full Slab contract
  before replacing the maintained one-slot form. Defer that distinct
  consumer experiment rather than infer a Slab improvement from map timings.

- **Short Vector cycles retain unresolved lowering costs.** The paired
  consumption experiment improves the large-record paths but slows the
  16-element scalar reuse chain in both source orders. Ordinary optimization
  also leaves a large WF/direct-C gap in the one-element suffix cycle, where
  neither composition relocates a rear element. Fewer aggregate transfers do
  not explain either cost. Keep this attribution separate from the operation
  choice above: compare the emitted loop, callback and argument code under
  ordinary and retained helpers, preserving the same source contract and
  accounting for the in-binary C controls' variation. A general lowering
  improvement is worthwhile if the short-cycle reduction is reproducible
  without losing the established large-record gain. Defer further tuning until
  that cause is established; reopen for a workload dominated by these cycles.
  The [paired samples and limits](../research/experiments/container-representation/vector-library/RESULTS.md)
  are the starting evidence, not a claim of uniform improvement.

- **Empty owning slots initialize inactive payload bytes.** In the owning-map
  trial's optimized wide code, constructing each vacant enum slot zeros the
  inactive 264-byte Pair region. A one-slot window similarly zeros that region
  together with its length, although the native controls initialize only
  occupancy. This is shared lowering work, not a necessary cost of one
  sparse representation. The [map comparison](../research/experiments/container-representation/map-library/RESULTS.md)
  separates these stores from later per-live-owner transfers. Investigate
  leaving inactive storage uninitialized without allowing an active value,
  discriminant or length to become undefined; check consuming projections,
  all variants, empty windows, must-consume owners and ordinary call boundaries.
  Measure an unchanged-source compiler comparison before claiming a runtime
  gain. Defer the compiler change during representation selection; reopen
  when the library's construction/rebuild trace supplies the measured consumer.

- **Consumed aggregate locals can retain an argument snapshot.** An exposed
  mutable local is loaded into an immutable argument snapshot before a consuming
  call. Clang 21 forwards that snapshot in the large-record regression, while
  Apple Clang 15 retains an extra whole-record copy. General forwarding could
  remove that copy independently of the optimizer, but needs a liveness and
  interference argument across the complete argument list and result/input
  reuse. Existing call-result coalescing does not cover a consumer returning
  unit. Defer broadening that path while the frame and descriptor changes are
  qualified; reopen when the retained snapshot materially affects a measured
  workload. Require a before/after transfer and timing comparison plus the
  existing exposed-place, later-argument-write, reentered-block and owned-result
  snapshot controls. The
  [transfer evidence](../research/experiments/container-representation/vector-library/RESULTS.md#v061-copy-and-consumption-trial)
  separates this opportunity from the library's remaining element relocation.

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

- **Array-helper pricing beyond original read-only references remains conservative.**
  The pending [typed Box-array extent proposal](../research/investigations/compute-model/DESIGN.md#read-only-box-array-helper-work-pricing)
  keeps static estimates for local owners, write-capable formals and references
  changed away from the original formal. Some unchanged forwarded references
  also lose the exact capture identity and fall back. Retaining those runtime extents could
  expose useful work, but their measured workload impact is unknown and a
  captured owner may already be consumed. Reopen when an affected helper's
  static price demonstrably withholds useful splitting and an existing checked
  validity fact or captured scalar measure can authorize the observation at
  every split site, including zero-trip loops. Defer broader transport until
  that case supplies both the benefit and the availability evidence; pricing
  must not infer a separate source lifetime.

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
  Reopen when generic container/behavior composition makes instance count or
  checking cost material. Recheck the distinct-instance and repeated-instance
  controls on that composition, separating semantic checking, lowering and
  emitted-code size; faster duplicate lookup alone cannot close the bound.
  The broader admission or sharing question remains deferred to an explicit
  choice supported by those controls and a complexity argument.
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

- **Remaining value-evidence boundaries.** The
  [investigation](../research/investigations/result-proof-transport/DESIGN.md)
  leaves three related extensions to assess together: borrowed Result
  selection and aggregate/indexed storage, multiple Result destinations from
  one call, and general scalar `give` expressions beyond the existing bare
  atom. These can remove remaining naming/projection workarounds, but storage
  invalidation, cross-result guard identity and evaluated-expression images
  need their own acceptance rules and cost evidence. Reopen when an ordinary
  library example needs one of these boundaries. Validate matched direct/local/
  projected programs, alias and descriptor writes, joins, loop iterations and
  stronger-contract negatives before choosing an extension; do not infer a
  general refinement system from the local-result implementation. A separate
  FN-9 result-selector limit remains: a nominal Slab result cannot publish
  `ensures result.cells.inner.len == 0_u64;`, whereas the direct boxed Ring
  carrier can publish its measure. The
  [exact rejected forms](../research/investigations/containers-and-resources/X1-LIBRARY.md#exact-unavailable-source-forms)
  distinguish this wrapper boundary from indexed postcondition targets and
  from storing an already-related Result. Reopen it when a library wrapper
  needs the relation, with direct-carrier, nested-field and stale-write controls.
  Also assess
  sharing or projecting per-local conditional fact matrices when many outcomes
  remain live: 32 outcome additions measured 585 ms versus 23 ms at the
  baseline, and 32 chained joins measured 721 ms and 214 MiB peak RSS. These
  are whole compilations of small sources; the benefit and precision tradeoff
  of sharing/projection remain unverified by that observation. Compare checking
  time, retained evidence and peak memory
  on the investigation's scaled sources before selecting that representation
  improvement. These extensions are deferred because the selected ordinary
  local composition rule can be validated without widening the storage or
  predicate vocabulary.
- **Declaration and call-boundary syntax after the ownership redesign.**
  Reassess mandatory `own` on value parameters and results, mandatory names
  for every result including `unit`, and the named-argument/construction-field
  discipline together. References now have only the `&` form and cannot be
  returned; result names serve contracts rather than runtime storage. These
  changes may leave declarations repeating information without improving the
  callable boundary. Named arguments and fields have a separate transposition
  rationale and must not be removed merely because they are verbose. Compare
  complete alternative signature and contract forms on scalar, generic,
  multi-result and resource APIs. A candidate must preserve explicit boundary
  types, unambiguous result references, useful mismatch diagnostics and one
  grammar-defined spelling, without site-dependent inference relief. The
  benefit and final spelling are unverified; defer selection until the next
  syntax-design discussion.
- **Ownership transfer and reference-access forms.** Audit unnecessary
  owner-in/owner-out APIs now expressible with reference parameters and exact
  effect rows, the differing consumption spellings of calls, returns, matches
  and `propagate`, and repeated `deref`/`&deref` paths. `move` still marks a
  consumption boundary; `deref` distinguishes a reference holder from its
  referent and from owned `Box.inner`, so neither is redundant solely because
  `own` may be. Compare the same container and owned-link operations under
  proposed forms, preserving copy/drop capabilities, whole-owner consumption,
  atomic replacement, reference rebinding, invalidation and effect separation.
  Require the ordinary positive and invalid-use examples to remain explainable
  by one rule per operation, with no additional runtime checks or transfers.
  Reduced ceremony is an opportunity, not an established gain. Defer these
  interface and syntax choices to a dedicated discussion; reopen with those
  same-operation comparisons.
- **Expression composition and canonical source policy.** Reassess mandatory
  three-address computation and intermediate names together with the ban on
  comments and rejection of noncanonical formatting. Compare authoring,
  local refactoring and diagnostic locality on unchanged algorithms and proof
  obligations; assess each restriction's concrete purpose rather than treating
  explicitness or brevity as sufficient grounds. Expression alternatives must
  specify evaluation order, temporary ownership and cleanup, proof invalidation
  and parallel-permission granularity while retaining deterministic parsing.
  Documentation and formatting alternatives must distinguish canonical output
  from the accepted-input boundary and must grant no proof authority to prose.
  No relaxation or authoring-cost improvement is established. Defer selection
  until the syntax review reaches this group; close it only with an explicit
  disposition supported by these comparisons.

- **Retained membership beyond the single-object composite is unestablished.**
  The [Slab membership caller](../tests/programs/containers/slab-membership-program.wf)
  verifies two indexes over one object: deleting one membership preserves the
  other reader, and the composite refuses object deletion until both retire. Weak
  indexes instead expire after deletion. The
  [analysis](../research/investigations/containers-and-resources/X1-LIBRARY.md#slab-reuse-addresses-and-retained-membership)
  does not establish a multi-object protocol or protection from independently
  authored bookkeeping mutations; ordinary handles and nodrop tickets do not
  authenticate a slab or make membership unforgeable. Defer stronger guarantees
  while callers need only the demonstrated composite or weak-index contract.
  Reopen for a real multi-index consumer that must retain objects across
  independent removals. First validate an ordinary composite with multiple
  objects, wrong-store/stale handles, removal ordering, final cleanup and a
  matched native retention contract, including its validation/storage cost.
  Do not infer that failure of an unrestricted static theorem rules out a
  correct protocol with ordinary checked data.
- **Deque still lacks zero-copy two-span access over Ring.** REF-4 rejects
  every Ring range, even empty and proved non-wrapping ones. The current
  library's slot visitor is not a substitute for a native consumer accepting
  two contiguous extents. A fully initialized Array works for copy elements
  but adds spare-capacity initialization and does not provide arbitrary T.
  The [source analysis](../research/investigations/containers-and-resources/X1-LIBRARY.md#ring-range-correspondence)
  identifies the missing contiguous-span interface. An extension needs a
  concrete span consumer, precise empty/non-wrap formation and invalidation
  rules, native-cost comparison and negative wrap/stale-reference cases.
  Defer extension while this library tests endpoint and rebase costs; reopen
  before using it for scatter/gather or another required bulk span consumer.
- **Deque rebase is an explicit new-owner conversion.** Reference-based
  replacement currently loses the exchanged owners' measures; append's
  lower-bound-only contract also lacks the exact sum needed by the library's
  return contract. The current counted take/place conversion supports nodrop
  T without an impossible cleanup branch, but its cost must be separated
  from a two-extent native transfer. The
  [source limits](../research/investigations/containers-and-resources/X1-LIBRARY.md#exact-unavailable-source-forms)
  and [cost comparison](../research/experiments/container-representation/deque-library/RESULTS.md)
  distinguish interface precision from lowering. Keep the explicit conversion
  for this slice; reopen if a real caller needs automatic reference-based
  growth or rebase dominates its work. Evaluate the already-open affine
  contract question below before choosing a new storage operation; require
  exact length, emptied-old-owner and unchanged element-order evidence.
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

- **Fixed-resource execution with proved completion — deferred.** Resume from
  the [research checkpoint](../research/investigations/fixed-resource-execution/README.md#deferred-work-and-resumption),
  which preserves the stack, recursion, loop, allocation/runtime and cleanup
  findings, proposals, probes and remaining validation. The goal is no heap,
  proved completion and peak storage within supplied byte capacities; a depth
  cap or `program no_heap;` alone does not establish it. Automatic qualification
  is unimplemented, the diagnostic stack ledger has coverage/geometry gaps,
  and general recursive release can still grow with value depth. Work is
  deferred until this topic is explicitly resumed. Start by rechecking the
  recorded compiler/target assumptions, then the complete acyclic stack-byte
  inventory; preserve unknown-call/alignment controls and exact budget-boundary
  cases. Progress proofs and full resource closure follow separately. The
  checkpoint also retains the consumer conditions for mutual tail transfers,
  general cleanup lowering and total-work estimation; none is scheduled now.
- **Facts a contract can carry (owner, 2026-09-20).**
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
  Reopen with a library operation that needs one of these facts; retain the
  exact refused clause and its best ordinary implementation. Validate support
  invalidation, aliasing, entry/exit and index changes, ownership outcomes and
  checking cost as well as the runtime check or source work saved. The Slab
  and Deque [source limits](../research/investigations/containers-and-resources/X1-LIBRARY.md#exact-unavailable-source-forms)
  remain examples, not an amendment or a claim that runtime state is lost.
- **Reference exit fields and custom outcome contracts remain restricted.**
  FN-9 gives exit-state denotation to a written reference parameter's storage
  measures, not its ordinary mutable integer fields. An owning sparse map
  cannot publish `deref(map).length == deref(entry(map)).length` for its own
  scalar occupancy counter; the caller can still read that counter normally.
  FN-9 routes only the integer success payload of the prelude Result, so a
  custom `Inserted / Replaced / Full` outcome cannot directly publish a
  different length relation for each variant. An unconditional insertion
  interval alone does not prove that a failed first attempt leaves length
  unchanged before a retry. Keep exact source refusals and the best ordinary
  interfaces in the [map trial](../research/investigations/containers-and-resources/X1-LIBRARY.md#generic-owning-map-trial-after-the-ring-comparison).
  Reopen when a caller needs those facts: compare ordinary re-reads or a
  separate numeric result with richer publication, measuring remaining checks,
  result transfers and checking cost. Validate entry/exit substitution, writes
  through aliases and incorrect variant claims before selecting any extension.
  Defer language changes while the complete map can use ordinary checked
  accesses; no runtime data or ownership state is missing.
- **Affine equality in call requirements has a narrower route than invariants.**
  The [map proof reduction](../research/investigations/containers-and-resources/X1-LIBRARY.md#generic-owning-map-trial-after-the-ring-comparison)
  verifies a local equality after a counted loop but rejects the identical
  equality as a callee requirement. INV-1 splits equality into two affine
  inequalities; ENT-6's signed FN-8 normalization lists ordering leaves only.
  Splitting that one requirement into `<=` and `>=` admits the unchanged
  algorithm without runtime checks. This is a specified proof-shape limit,
  not an observed compiler violation. Reopen with contract-proof work to
  assess consistent equality decomposition, including false equalities,
  negative goals, alias invalidation and deterministic checking cost. Defer
  a rule change while the exact paired-bound interface supplies the needed
  proof without runtime or ownership cost.
- **Whole-owner swap does not publish exchanged descriptor facts.** PRE-1's
  `swap` declares writes to both referents and no postcondition; MSR-3's
  placement rules do not include swap. Exchanging two boxed windows therefore
  kills their supported length/capacity facts without connecting the incoming
  descriptor to the new place. Actual values and unique ownership still move
  correctly. The direct map-migration trial can re-read bounds, but cannot
  derive its promised nondecreasing extent merely from the pre-swap facts.
  Reopen with the contract-publication work: retain scalar-only and nested
  owner controls, alias invalidation and input/output swaps, and compare the
  source/proof benefit with checking cost before selecting a general relation.
  Defer a language extension while ordinary reads suffice for the map;
  do not manufacture an impossible branch to satisfy a postcondition.
- **Conditional measure preservation needs a precise remaining diagnosis.**
  A counted-loop control calling a length/capacity-preserving helper in only
  one arm rejects its backedge facts. Capturing both measures before the
  branch and restating their equality afterward admits the small control;
  this is not a blanket inability to preserve conditional measures. The
  full sparse-map loop still rejects its extent invariant when its
  length-preserving wrapper is inlined with an explicit extent bridge. Its
  normative classification is unresolved. The [exact controls](../research/investigations/containers-and-resources/X1-LIBRARY.md#generic-owning-map-trial-after-the-ring-comparison)
  retain both outcomes. Reopen with contract-proof work: reduce the remaining
  refusal, compare it with ENT-5/ENT-6, and distinguish a compiler defect from
  a proposed rule change before implementation. Keep the admitted wrapper
  while it supplies the needed proof; validate aliases and false preservation
  claims as well as checking cost for any improvement.
- **Owning HashMap has a remaining large-value performance gap.** The
  [matched comparison](../research/experiments/container-representation/map-library/RESULTS.md)
  exercises the actual generic library, including must-consume pairs, without
  requiring one Box per payload. Its compact result and direct enum migration
  improve the original planned sparse source, but the measured 256-byte-value
  growth trace still costs about 1.64–1.69 times direct C with the same
  migration direction, and replacement about 2.09–2.35 times. These are
  complete checked traces, not isolated copy costs.
  Optimized migration still initializes inactive payload bytes, stages live
  pairs and reads the displaced payload before its tag is used. Public owning
  results retain transfers. Compare unchanged-source initialization/forwarding
  improvements against the same contract and verify all owner-return paths;
  copying counts alone do not establish their runtime contribution.
  Dense storage remains faster for wide growth but adds reserved metadata and
  dependent lookup, while a fresh sparse rehash retains two complete backings.
  Reopen for a workload dominated by these costs, preserving full backing and
  peak bytes, hash/load policy, retained helpers and exact cleanup. Defer a
  second maintained representation and compiler changes until that consumer or
  a discriminating unchanged-source improvement supplies their grounds.
  SIMD-group probing and a general projected-storage benefit remain untested;
  the earlier [native hash-slot study](https://github.com/mbbill/Whitefoot/blob/38c28403a2defd0b65b8a2ab2b5e4794315e9940/research/experiments/hash-slot-occupancy/RESULTS.md)
  did not establish a recurring tag-check tax. A working library does not
  close either question or imply a universal native-performance ceiling.
- **Channel primitive.** An ownership-transfer queue in the trusted base for
  producer/consumer pipelines and work stealing; lock-free rings are not
  expressible without it and batched fork-join is the available form. Research
  when the future concurrency primitives are designed.
- **Header-plus-tail heap block.** One allocation holding a fixed header and a
  runtime-length tail (LLVM `User` with its operand list, `sk_buff`). Today a
  boxed struct with a `Box<Slots<T>>` field costs two allocations and an extra
  dependent access; an inline struct plus a boxed tail uses one allocation
  but a wider handle and separated header. An encoded byte block is another
  available representation with codec costs. TYPE-9 still excludes a typed
  runtime-capacity tail inside a source struct; the
  [layout comparison](../research/investigations/containers-and-resources/X1-LIBRARY.md#ceiling-challenges-connected-to-real-source-contracts)
  separates these contracts. Two shapes
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
  Reopen when a concrete typed-header/tail consumer needs the compact owning
  handle or adjacent layout. Compare full allocation bytes, dependent accesses,
  initialization, movement/growth and cleanup against both ordinary alternatives;
  no new layout or construction spelling is selected without that evidence.
- **Bitmask fact.** `x & (c - 1) < c` for a power-of-two `c`, which would
  remove the per-probe bounds compare in hash tables.
- **Handing checker facts to the backend.** Emitted since the v0.60 port:
  `noalias` (not on `swap`), `nonnull`, `dereferenceable`,
  `captures(none)` or `nocapture` by a build-time probe, `inbounds`, and
  `nuw`/`nsw` on the exact family. The later qualified Ring payload-address
  `llvm.assume` is measured in the [Deque comparison](../research/experiments/container-representation/deque-library/RESULTS.md);
  its remaining costs are tracked above. Not emitted: `memory(argmem: ...)` (the
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
- **Unmeasured performance claims from the ownership-redesign matrix.**
  The v0.60 port has landed; the current Vector, Slab and Deque comparisons
  establish only their stated operation contracts and toolchains. The earlier
  matrix's costs for data-determined index checks, refused scatter,
  find-then-mutate re-descent and construction into an append slot still need
  current source and native controls where those comparisons do not cover them.
  Reopen the affected claim when a container or systems workload exercises it,
  preserving its ownership, order, overlap and allocation contract and separating
  required source work from removable lowering cost. Defer a broad repeat of all
  eight engineering tasks until it answers a concrete selection question;
  a passing new library does not dispose of the remaining matrix claims.
