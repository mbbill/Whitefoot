# Defects and follow-up work

Known defects, capability gaps, unresolved costs, and improvement opportunities
found during design or implementation, including unverified ones. An unverified
opportunity is a validation task: state its expected benefit, uncertainty, and
criterion for deciding whether to pursue it. Entries do not select a design.
Remove an item when its implementation and checks land, or its validation
concludes with a recorded disposition; retain any selected follow-up work here.
Add an item at the end of the section that owns its topic, so parallel branches
rarely insert at the same place.

## Numeric conversions and value evidence

- **Select the modular conversion companion.** The
  [conversion comparison](../research/investigations/numeric-conversions/DESIGN.md#companion-operations-and-explicit-deferrals)
  recommends integer-only `cvt.wrap` for direct low-bit extraction and modular
  signedness conversion. It is deferred from the exact conversion family
  because it selects an additional result policy. Validate all integer
  width/sign classes, especially negative signed inputs widened to unsigned
  destinations, and ensure changed values publish no exact input equality.
  Reopen when the owner selects this companion for implementation; remove
  after its selected rules and ordinary-path evidence land.

- **Select direct rounded/saturated float conversion policies.** The
  [conversion study](../research/investigations/numeric-conversions/DESIGN.md#companion-operations-and-explicit-deferrals)
  identifies missing direct rounded-to-float semantics and cumbersome total
  float-to-integer compositions. A rounded-to-float candidate needs explicit
  ties, overflow, subnormal, signed-zero and NaN rules; saturation needs its own
  NaN and rounding choice, including nonrepresentable i64 maxima. Defer from the
  exact-conversion implementation because these select different results and
  no concrete consumer has selected their complete surface. Reopen for a
  float-heavy program or owner selection; compare source and emitted/native
  behavior before choosing spellings or claiming an improvement.

- **Validate float and domain evidence through saved Results.** Exact
  conversions extend integer value relations only. A checked result
  with a float endpoint does not transport a domain predicate for its old
  input or a float equality; callers can use the payload or branch on
  `.defined` when the predicate is needed. Extending ENT-5/FN-9 could remove
  repeated validation in a real consumer, but requires typed noninteger value
  identities and guarded goal transport beyond the current numeric context.
  Validate input replacement, copied/replaced Results, joins, loops and proof
  costs without combining independent guards. Defer until such a consumer
  demonstrates the need; remove when a selected evidence rule covers it.

- **Qualify broader proved-range transport to the backend.** The
  [bounded conversion control](../research/investigations/numeric-conversions/DESIGN.md#optimized-helpers)
  removes a residual check when its already-verified entry range is supplied
  as an LLVM assumption. Direct lowering under the bare conversion
  proof solves that conversion case without a general transport family.
  Broader transport may benefit operations outside that family, but needs a
  concrete consumer and a complete retained-evidence-to-target mapping, with
  ordinary value support, mutation and call boundaries preserved. Reopen when
  such a consumer retains measurable work despite checked facts; require a
  matched benefit and unchanged acceptance/behavior before choosing a family.
  Defer from the exact-conversion change, and remove after selection and
  qualification or a documented decision that the candidate brings no benefit.

- **Validate further sharing of dense Result evidence when larger consumers need it.**
  The [cost comparison](../research/investigations/result-proof-transport/DESIGN.md#selected-cost-result)
  still places 32 independent outcomes at about 62 ms and 32 joins at about
  265 ms, versus 22 and 26 ms before value-associated proof transport. A dense
  matrix remains per live value and every surviving context participates in a
  join. Sharing more unchanged ordinary cells may reduce this cost, but the
  benefit and representation complexity remain unmeasured. Require matched
  time/RSS improvement on a larger real consumer, identical acceptance and
  valid retained proofs, and candidate/fallback preservation through support
  kills and joins. Defer a broader storage change because numeric-core reuse
  meets the recorded real-program and scale targets; reopen when more live
  Results or wider storage support makes this cost material. The language
  extensions below remain a separate question.

## Checker precision and proof cost

- **Some ENT-3 sources read no measure operand.** S7's constant-offset,
  checked-offset, exact-division, remainder and unsigned `iand` rows read an
  operand the specification calls an admitted term or constant through the
  flow's tracked-place and constant reader, which omits ENT-2 clause (b)
  measure terms; S5/S6 copies, S1 comparisons and S11 counted captures do
  read measures. So `let r = x % deref(src).len;` establishes no
  `r < deref(src).len`, and a following `deref(src)[r]` is rejected under
  OP-4 although binding the length first is accepted. Other flow readers of
  the same shape (subscript offset terms, S13 index captures, allocation
  lengths, range-formation operands, integer-domain operands, the ENT-5 `Ok`
  payload) are unverified; affine images already cover some of them. Repair
  with one complete ENT-2 term reader, and validate it with paired direct and
  let-bound cases for each source, including a write that kills the measure,
  requiring no other verdict change. Deferred from the counted-endpoint
  repair, which changed only S11's reading; reopen with the next entailment
  change or when a program needs the direct form.

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

- **Distinguish resolved formal anchors from holder queries in proof consumers.**
  Some entailment support/overlap consumers pass an already resolved formal
  root back through `PlaceMap::resolve`, which also serves written reference
  holders. After a parameter rebind this can conservatively add its other
  observed targets. Audit these calls before changing their interpretation;
  use entry-anchor/rebound-holder pairs and overlapping controls to establish
  whether separating the APIs recovers useful precision without omitting an
  origin. No incorrect acceptance or measured benefit is established. Defer
  this consumer change to the joined-reference work above; reopen when that
  work establishes point-current target authority or a real proof needs it.

- **Expose a failed callee proof behind an unavailable summary.** The
  [partially concrete reserve probe](../research/investigations/containers-and-resources/X1-LIBRARY.md#partially-concrete-reserve-diagnostic)
  reports INV-1 at `room` after `priority_queue_make_room<ProbeDue, ceiling>`.
  Adding the 32-byte allocation bound only to the caller still fails; literal
  `8192` admits. Read-only diagnosis identifies reserve's missing local OP-9
  bound under ENT-2, not a demonstrated publication defect. First validate
  the bound in both reserve and caller, propagated through intervening helpers,
  then require the intended OP-9 rejection one element above it. Improve the
  diagnostic to identify the failed callee obligation and unavailable summary
  without changing acceptance. Its benefit and exact attribution remain
  unverified; defer this diagnostic work while the admitted generic standalone
  control serves the experiment, and reopen when improving call-proof reports.

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
  still unavailable; missing evidence keeps sequential lowering. For windows
  this means every [WIN-2] part-relative separation is refused when a member
  before the later one writes that window's `len`, which also refuses a read
  of an old slot after an append; the mapping would recover it. A cheaper
  recovery needs no mapping: a place reached through a reference live at the
  first statement's entry is interpreted in that state, and the reference's
  validity gives `i < len` there, so WIN-2's single-state separation still
  holds. That recovers the one pair this rule newly denies in the maintained
  programs, `deque_push_back` against `let first_after_append =
  deref(original_first)` at `tests/programs/containers/deque-program.wf:113`.
  The ledger's denial should also name the length change as its cause; it
  currently reports only the overlapping write and read. Investigate
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

- **Acceptance and check removal are trusted to the whole checker.** Every
  lowering authorization (a subscript without a check, an exact operation, a
  discharged call goal) is issued by the same entailment engine that decides
  acceptance, so the trusted base for "no unproved partial operation" is the
  full front end plus entailment. The
  [certificate packet](../research/investigations/proof-certificate-architecture/PACKET.md)
  (v0.26, before the x1 ownership redesign) selects a staged route: the engine
  records a positive derivation for every discharged obligation, and a small
  verifier over a trusted proof-flow extraction checks them and jointly issues
  the lowering capability, while rejections stay with the engine because a
  missing certificate does not prove non-derivability. The compiler keeps a
  derivation ledger; no verifier, extraction boundary or joint issuer exists.
  Re-derive the packet's Envelope B against the current specification, then
  prototype the verifier on `tests/programs/` and measure its size, proof size
  and added compile time; a corrupted or missing certificate must never
  authorize lowering. Close when a verifier jointly issues the capability, or
  when the packet's stop gates record why the unified engine remains.

- **Write kills do not submit their own OWN-7 separations.** An ENT-5 write
  kill decides an index or range step against a fact's support only from the
  separations already retained on the current edge, which are the EFF-5
  pairwise and REF-2 preservation questions the structural checker submitted,
  plus literal index inequality. OWN-7 makes two ranges disjoint whenever the
  current ProofContext proves one of its four orderings, so a length fact over
  `deref(head)[0_u64]` with `head = &rows[0_u64..1_u64]` should survive a write
  through `rows[1_u64..3_u64]`, bound or formed at the call; today it dies and
  the dependent subscript is rejected, and binding offsets proved distinct
  only by a guard behave the same way. The effect is over-rejection, never an
  unsound acceptance. Submitting one bounded question per written/support step
  pair at each kill would admit these programs at a proof cost per fact per
  write; a literal-endpoint range shortcut beside the literal index one would
  cover constant ranges cheaply. Validate with the bound and inline spellings,
  stale-capture and joined-origin negative controls, and a measured
  checking-cost comparison. Reopen when a real program needs a fact to survive
  a provably disjoint write; close when kill-time separation is implemented and
  qualified or declined on measured cost.

- **Consumers rebuild call-argument referents from expression shape.** The
  structural checker resolves every actual to its REF-1 places (`actual_paths`,
  including a formation's range step) and uses them for EFF-5, REF-2 and the
  EFF-2 projection. The entailment flow (`argument_referents`) and the
  permission judgments (`argument_places`, PAR-2's range recording) instead
  rebuild those places from the checked argument expression. A missing
  expression arm there is silent: inline range actuals once produced no ENT-5
  kill, and so admitted out-of-bounds reads. Retaining the checker's resolved
  paths per argument on the checked call and reading them in every consumer
  would remove the duplicate reconstruction and this defect class, at the cost
  of a checked-model field and its loop-carried and joined-origin handling,
  which the flow must still read point-currently. Validate that each consumer
  reaches its current verdicts on the full corpus with identical kill,
  permission and ledger results, and that a deliberately removed checker arm
  fails in one place. Reopen when another argument form is added or another
  referent omission is found; close when the consumers read one inventory or
  that inventory is shown unsuitable for point-current flow facts.

## Containers and storage lowering

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

- **PriorityQueue has distinct sift and return-boundary costs.** The
  [complete library comparison](../research/experiments/container-representation/priority-library/RESULTS.md)
  measures retained scalar pop/push at 1.510--1.722 times same-algorithm C for
  16/256 elements. In that comparison WF returned push's Result through a
  pointer and cleared its inactive payload; C returns the scalar result in
  registers. The scalar cohort's three-leaf `Result<unit, u64>` now returns in
  registers too ([result-register investigation](../research/investigations/result-registers/DESIGN.md#selection)),
  while the inactive payload is still cleared and the wide cohort keeps its
  destination. The causal shares, and what that change recovered, are
  unmeasured. Normal wide replacement also costs 1.178--1.289 times
  the swap control at those sizes, while retained replacement reverses the
  direction. Separately, wide hole-sift C halves counted movement on large
  complete traces; ordinary WF swaps cannot be credited with that algorithm's
  cost. Validate return placement and initialization with unchanged-source
  compiler variants, the full owning/refusal chains, preserved C controls,
  both cohorts and emitted-code attribution. Investigate the wide replacement
  reversal before choosing an inlining or forwarding change. A general
  improvement must beat control variation without regressing the complete
  matrix; source ownership must remain intact. Defer ABI changes and a new
  storage operation until those discriminators establish their benefit and
  interference obligations; reopen for the indexed heap composition or an
  application dominated by these paths. Do not report universal native parity
  from the large scalar queue results.

- **Small results beyond the per-leaf register budget still use a
  destination.** A stored result returns in registers only when its scalar
  leaves fit the x86-64 budget of three integer-class words and two floating
  leaves ([result-register investigation](../research/investigations/result-registers/DESIGN.md#demotion-probe)).
  A 16-byte result with four 32-bit fields, a small byte array and the 32-byte
  opaque `ExitStatus` still pass through memory. So does every result with
  four to eight integer words, or three to eight floating leaves, on AArch64.
  Packing small integer leaves into shared words, or a per-target budget,
  could carry some of these. Either one adds per-target lowering to emitted
  code and to linked definitions. A third floating leaf on x86-64 cannot join
  them: it returns through the x87 stack, which is not bit-exact for signaling
  NaNs. No maintained program currently shows a surviving call with such a
  result, and the benefit is unmeasured. Reopen when a maintained program
  keeps such a call on a measured path. Validate with unchanged source and
  both lowerings compiled. Require the destination round trip to disappear
  without a new demotion, a lost float bit pattern, or a regression in the
  program's timing, on each target that changes.

- **The records comparison fails at the register-return revision's
  placement.** For the small-result register ABI, the five maintained paired
  comparisons the
  [hosted comparison](../research/investigations/result-registers/DESIGN.md#hosted-compute-regression)
  lists read `records` at 0.78--0.83 at W=2 and 0.68--0.82 at W=4 (baseline
  over candidate) on two hosted AMD runner classes. At W=1 they read 0.93 on
  one class and 1.15--1.16 on the other. The other four kernels pass, and
  their emitted code is unchanged. On a local Intel host, the same images
  show the same wider-row failure. Shifting both loop copies by 16--48 bytes,
  with no instruction changed, reverses the arms' order at W=2 and W=4.
  Moving only the runtime has no effect. On the hosted fixture the
  candidate's kernel executes 1.2% fewer instructions. The register return
  still loses one structure: its single return block lets SimplifyCFG turn
  `validate_record`'s exit test into a `select`, so the threaded inner loop
  over ASCII bytes is not formed. On ASCII records that costs 41% more kernel
  instructions and 0.1--3.4% of local W=1 time. Clang shows the same loss for
  a C transcription returning its two-field struct. The hosted runners have
  no placement control, so the failure is not attributed on them. A lowering
  that keeps the threading is unexamined. Reopen with a bounded placement
  control on a hosted runner, or with a maintained workload whose time
  follows the lost threading beyond its placement range. Validate against
  unchanged source with an identical-image control. Neither a later passing
  run nor a changed threshold closes this entry.

- **Indexed small-payload costs with retained boundaries need attribution.**
  The [native-cost record](../research/experiments/container-representation/indexed-library/RESULTS.md#remaining-native-costs)
  puts 4096-record growth/cleanup at 1.354--1.368 times swap C and
  1.392--1.408 times hole C across policies, cohorts and both series. The
  standalone/shared indexed executables are identical, so this is separate
  from the sharing choice. Wide mixed traces instead favor WF. The position
  reporter retains a 32-byte Due snapshot and a separate 16-byte handle
  snapshot in 48 stack bytes; its native frame is 80 bytes versus C's 32.
  Successful insertion also clears a 40-byte result before writing the active
  fields. Both implementations retain the same handle-validity checks; these
  observed snapshots and stores do not establish their elapsed-time shares.
  Validate which snapshots or result stores general compiler handling can
  avoid using unchanged source, matched public boundaries, emitted code,
  complete ownership/expiry oracles and unchanged controls. Preserve callback
  effects and all validity checks; add no container-specific compiler path.
  Defer optimization selection until that discriminator identifies a benefit;
  reopen for a consumer dominated by retained small-record growth or a measured
  toolchain change. No general interface, storage or compiler mechanism is
  selected by these ratios.

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

- **Inactive-payload omission has measured optimizer regressions.** The
  destination-construction candidate removes the owning map's 264-byte vacant
  payload clear and its local pending-window clear while preserving active
  fields, descriptors, ownership and the ordinary ABI. The unchanged-source
  [comparison](../research/experiments/container-representation/map-library/RESULTS.md#same-source-inactive-storage-lowering-comparison)
  and its reversed replay show wide growth and rehash gains, but normal wide
  replacement regresses by 7.5--9.0 percent after C normalization. Retained
  scalar Slab lookup also regresses. The registered selection criterion is
  not met; fewer stores are not grounds to accept these costs silently.
  In the replacement path, private exchange inlining adds stack temporaries
  and payload transfers; in Slab, a small result becomes separate field
  stores rather than one combined store. Their causal shares remain
  unisolated. The bounded poison-seeded aggregate-store follow-up also failed
  its structural screen: Slab's successful path grows from 23 to 28 native
  instructions, and wide Map put expands arrays into 105 LLVM loads. It was
  stopped before timings. The owner rejected both candidates; production
  retains baseline destination initialization. Reopening needs a distinct
  argument addressing those optimizer losses. Compare the same full matrices,
  null controls and ordinary/retained boundaries; preserve the dirty-storage,
  selected-variant, partial-window, linked-body and parallel cleanup checks.
  SSA construction and general aggregate forwarding are separate paths, not
  improvements established by this candidate.

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

- **Ordered node construction and cleanup retain wide transfers.** The
  [ordered-map attribution](../research/experiments/container-representation/ordered-library/RESULTS.md#transfer-and-generated-code-attribution)
  shows field-expanded node-to-Box construction and a 504/4224-byte copy of
  each exhausted node before only its leading link is consumed. Reducing that
  work could improve split/build and final cleanup without changing the tree.
  Static transfer counts do not isolate its timing contribution. Validate a
  bounded construction/consumption improvement with unchanged ownership
  outcomes, node allocation counts, dirty/quarantined release checks and
  normal/retained scalar and wide comparisons; inspect optimized code to
  establish which transfers disappear. Keep aggregate-result ABI and general
  argument forwarding under the existing Slab and consumed-argument items;
  this task isolates fixed-node construction and consumed-field selection.
  As with the indexed snapshot/result work, validate general lowering rather
  than a container-specific compiler path. Defer a change until these
  construction/consumption paths isolate its benefit; reopen when the transfers
  materially affect a measured consumer or lowering work reaches those paths.

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

## Parallel lowering and runtime

- **Validate reuse of selected-target element layouts during emission.**
  [Zero-stride addressing](../compiler/src/backend/target.rs) currently queries
  the ordinary layout calculator afresh for each element-address step. Repeated
  accesses to a deeply nested nominal element may recompute the same layout.
  Compare checking/emission cost on repeated nested-element accesses before
  introducing shared layout storage; require identical qualification and emitted
  addresses. The benefit and material cost are unmeasured, so keep the simple
  query for now and reopen when measuring target-emission cost or extending its
  layout consumers.

- **Parallel footprints omit ordinary result-list bindings.** The
  [sparse-routing trial](../research/investigations/compute-model/DESIGN.md#sparse-destination-routing-trial-2026-09-21)
  exposes a receiver map denied solely because one statement binds two local
  results. PAR-2 allows iteration-owned writes, but the current PAR-1/PAR-2
  walkers model one result definition per statement and refuse this form.
  A two-field record admits the same receiver map without added allocation or
  traversal; the retained form now qualifies useful helper discovery.
  Defer a general multi-definition footprint implementation; reopen when that
  workaround materially complicates a real consumer. Validate complete effects,
  consumption, exits and lowering for all result ordinals rather than granting
  a tuple-specific exception.

- **Initialized allocation can impose serial span on parallel work.** The
  [private-outbox representation](../research/investigations/compute-model/DESIGN.md#private-outboxes-without-frontier-compaction)
  requires a fresh C-by-D head matrix each level; its element fill is a
  sequential emitted loop before otherwise independent routing. Initialization
  remains linear work but can dominate the full critical path. The sparse
  oracle and useful helper work are now qualified, but the
  [FIFO comparison](../research/investigations/compute-model/DESIGN.md#native-phase-qualification-and-prospective-fifo-comparison-2026-09-22)
  stopped at its identical-image control. The end-to-end cost is not attributed.
  Measure fill/allocation separately from useful routing before choosing a
  general lowering change; preserve initial values,
  cleanup and the unchanged sequential image in any later experiment. Defer
  repair until a qualified cost comparison establishes materiality.

- **Loop capture selection remains conservative beyond forwarding.** The
  [needed-capture change](../research/investigations/compute-model/DESIGN.md#needed-loop-captures)
  retains every ordinary instruction and call argument. Removing an unused
  pure computation or an unused callee formal could shrink further frames,
  but needs independent effect and call-interface reasoning; no blocked
  consumer currently justifies that scope. Broader dead-computation or
  dead-formal analysis remains deferred: reopen
  when an otherwise useful map still exceeds the fixed frame bound, and
  validate smaller emitted frames on the same source without changing calls,
  cleanup or results before selecting that wider scope.

- **Pruning already-fitting loop frames needs a qualified benefit.** The
  [capture investigation](../research/investigations/compute-model/DESIGN.md#needed-loop-captures)
  now selects capture pruning only to rescue an originally oversized frame.
  Pruning fitting tasks changes transport and code placement without admitting
  a new loop, and records repeatedly reports an adverse W4 observation whose
  cause remains unresolved. Smaller frames could still help another consumer,
  but that benefit is unverified. Defer the broader optimization until a real
  fitting-frame consumer exposes a material transport cost; reopen with an
  unchanged-source comparison that retains native results, cleanup and a
  same-image null control, establishes its benefit and clears the protected
  records case before selecting the wider policy.

- **General DAG scheduling and competitiveness remain unqualified.** The
  [runtime-adjacency probe](../research/investigations/compute-model/DESIGN.md#runtime-adjacency-all-predecessor-probe)
  executes runtime-provided forward graphs with at most two predecessors and
  successors per vertex through ordinary WF source. Both owner mappings pass
  the original-edge oracle, including all such graphs through five vertices.
  Recursive owners overlap, but their joined rounds delay a ready task that
  the native readiness executor overlaps with another long task. The loop
  mapping retains zero split budget at the selected owner counts. Under
  topological numbering and contiguous ownership, rounds are bounded by C;
  routing adds O(C*C*R) work and two C*C head matrices. These results qualify
  the bounded expression, not arbitrary labeling/degree, general efficiency,
  elapsed competitiveness or physical peak workspace. Defer a general executor
  until a concrete consumer makes those remaining costs material. Reopen with
  its unchanged original graph, every result and exactly-once counts, charging
  sorting/remapping if needed, construction, initialization, routing, added
  precedences, wall/CPU and peak space against a useful native readiness
  executor. An owner-round limitation does not establish that every ordinary
  source formulation needs the same barrier.
  Separately, the bounded checked-call bridge recovered the selected A/D
  overlap while reducing observed C/D overlap across its fixed masks. The
  [amended identical-image control](../research/investigations/compute-model/DESIGN.md#amended-cost-result-identical-image-control-failure)
  failed before any candidate comparison, so both the bridge and DONE-first
  runtime change are withdrawn for lack of cost qualification, not a measured
  implementation regression. Reopen only for a concrete call-group consumer
  and a bounded comparison with qualified measurement controls, unchanged
  results/edges and prospective wall/CPU protection for the other masks and W1.
  Recovered overlap alone selects neither implementation nor a broader executor.

- **Recursive frontier policy suppresses deep work on a spine with side leaves.**
  The [cutoff-attribution control](../research/investigations/compute-model/DESIGN.md#native-qualification-and-cutoff-attribution)
  shows default W4 budget eight serializing task IDs 16 onward; the existing
  frontier-off mode restores deep overlap at lengths 16 and 32 on costly and
  last-heavy leaves. Preserving those offers may expose useful work, but its
  elapsed benefit, cheap/skew overhead and longer-spine space costs are unknown.
  Disabling the budget retains the runtime's 64 capture slots per lane, held
  until their enclosing joins. The stable-scatter result below also retains
  a 30–37 percent regression when that budget is disabled, so this observation
  does not select a global off policy. Defer policy changes until a concrete
  recursive consumer or a selected scheduling study makes the cutoff material.
  Reopen with unchanged task values, counts and edges; separate offered work
  from successful steals, account for live frames/slots, and qualify cheap,
  costly and skewed inputs across widths with prospective wall/CPU and W1
  criteria before adopting a replacement.

- **Parallel grain policy needs a dedicated study.** Captured extents are a
  provisional scheduling input, not an established broadly suitable policy.
  The [first same-source trial](../research/investigations/compute-model/DESIGN.md#runtime-extent-trial-result)
  improves prefix, histogram and stencil, but makes chain-pull 51 percent
  slower at two workers and incurs substantial CPU costs in some faster
  cases. Those measurements precede the continuation-accounting correction.
  The [frozen `6fdb6768` baseline](../research/investigations/compute-model/DESIGN.md#frozen-compute-baseline-2026-09-21)
  observes useful regular parallel work and substantial wide-stencil CPU cost;
  it does not isolate that correction's effect. Its scalar native controls do
  not establish optimized-native competitiveness. Study whether a robust common
  policy exists or workload, input shape, worker count and hardware require
  different choices, comparing wall time, CPU and scheduling/profile overhead.
  [Runtime profiles and PGO](ideas.md#parallel-grain-policies-and-runtime-profiles)
  are candidate inputs to that later study. The trial's failures remain
  evidence, not proof that no broadly useful strategy exists. Close this item
  when a policy meets explicit representative criteria or its accepted
  tradeoffs are recorded.
  The [first-index probe](../research/investigations/compute-model/DESIGN.md#native-expression-result)
  exposes a concrete input missing from current wave prices: data-dependent
  record lengths and early exit leave the same static estimate for one-byte
  and 65,536-byte records, and the small permitted waves receive no split
  budget. The [adjacent-helper result](../research/investigations/compute-model/DESIGN.md#adjacent-helper-pair-native-result)
  qualifies another source composition: the same two local-return blocks in
  ordinary calls execute nonempty absent/late-hit predicates on the caller
  and a helper, with observed overlapping lifetimes and preserved prefixes.
  The old counted form still has zero budget, and the cheap-first-hit case
  still pays for the other started block. A native two-block reference
  confirms the work contract, not pool competitiveness. At fixed offsets and
  descriptor lengths, payload-dependent early exit still changes actual work;
  bounds/read-only facts alone do not supply a representative price.
  Validate profitability on a representative costly search against a fair
  native reference, charging completed final-wave work, observer perturbation
  and scheduling/profile costs. The functional participation result selects
  no loaded-read, PGO or general grain policy.
  Defer further search pricing work until a concrete consumer requires it;
  reopen with that workload and a criterion that distinguishes useful overlap
  from merely higher worker participation. No lower threshold or new
  cancellation mechanism is selected by the expression result.

- **Array-helper pricing beyond original read-only references remains conservative.**
  The accepted [typed Box-array extent extension](../research/investigations/compute-model/DESIGN.md#read-only-box-array-helper-work-pricing)
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

- **Per-task loaded costs remain unavailable to loop work pricing.** The
  [phased spine](../research/investigations/compute-model/DESIGN.md#phased-spine-permission-and-work-price)
  has PAR-2 permission but emits the same static price 199 for cheap and costly
  leaves. At the current work unit, 754 iterations afford one chunk and 1,508
  first afford a positive split budget; all selected lengths through 32 remain
  unsplit, matching zero observed W4 overlap. Useful independent work may be
  withheld because a recurrence bound loaded inside a chunk is unavailable
  at its split site, even when the source input is read-only. Extent transport
  alone does not provide a representative task price for mixed loads. The
  [captured-scalar control](../research/investigations/compute-model/DESIGN.md#captured-scalar-availability-control)
  transports the actual uniform leaf bound through the same helper chain:
  prices 48 and 655,398 retain sequential cheap/boundary controls and expose
  costly length-32 overlap on four W4 threads. All six cases pass at W1/W4
  in ordinary/traced images. This establishes scalar availability at the
  existing summary depth, but changes the input representation and derives no
  price for heterogeneous loads. Defer a general pricing change until a
  concrete variable-cost consumer supplies representative benefit criteria.
  Reopen with its unchanged IDs, values, exactly-once counts and original
  edges, preserving cheap/zero-trip controls and qualifying wall/CPU and W1
  overhead prospectively. Charge any added observation or aggregation reads,
  work and storage and establish validity at every split site; do not bypass
  the missing input with padding or manual grain. This is separate from
  read-only header extent transport and selects no pricing policy.

- **Zero-budget dispatch needs caller and placement attribution before adoption.** The
  [query-retained one-site control](../research/investigations/compute-model/DESIGN.md#query-retained-control-result)
  meets its W4 criterion, but grows module text by 2,400 bytes and measures a
  2.97 percent W1 paired-median regression despite identical normalized W1
  instructions. It couples dispatch, pixel inlining and alias-check motion;
  it establishes neither standalone entry cost nor the benefit of applying
  the branch at every site. The general cdac candidate grows text by 3,920
  bytes (44.50 percent), including additional top-level inlining, with no new
  local timing. The subsequent
  [null-qualified Intel comparison](../research/investigations/compute-model/DESIGN.md#general-dispatch-reassessment)
  regresses records by 30.72 percent at W1 and 4.01 percent at W2. W1 never
  executes the added dispatch: its unchanged sequential chunk has different
  placement and a differently optimized caller. Neither cause is isolated.
  Withdraw the all-site optimization while retaining the existing query and
  splitter entry; a wrapper, noinline boundary or alignment policy needs its
  own grounds. Defer that broader optimizer/layout investigation behind the
  remaining compute-expression questions. Reopen with a comparable Intel host
  and a bounded control separating caller/stack changes from placement, with
  an identical-image control and full outputs, before selecting a general
  replacement. Require a demonstrated candidate benefit and qualification of
  the affected W1/parallel paths; a passing ARM or different-host run alone
  cannot clear the retained counterexample.

- **Stable scatter retains construction and packing costs.** The dated merged-model
  [joined-phase result](../research/investigations/compute-model/DESIGN.md#joined-phase-result-2026-09-21)
  identifies about 0.596 ms of chunk initialization and 0.569 ms of packing at
  W8, against a 1.930 ms ordinary mixed-input call. That image clears
  and copies a full inactive chunk payload per appended `None`, and expands
  aggregate transfers during input partitioning; borrowed tally/packing reads
  no longer retain that full-copy cost. These observations do not measure
  current main. A separate, unverified opportunity is to construct a single-use
  aggregate directly in its fresh placement destination, independently of
  whether inactive payload bytes are cleared. Feasibility across ordinary call
  boundaries and loop re-entry, and the whole-call benefit, remain unestablished.
  Defer this behind the current pricing, sparse-discovery and baseline work.
  Reopen when current optimized code reproduces material staging traffic;
  compare unchanged source, require that transfer to disappear, qualify whole-call
  wall/CPU results, and preserve enum/affine snapshots, alias behavior, window
  length updates and exact-once cleanup. The shared destination-initialization
  trial above was rejected; its initialized-value argument and measured
  tradeoffs do not establish which scatter paths benefit or their
  whole-call cost. Those source-specific observations remain unmeasured.
  A direct `Array` replacement is not admitted: `Chunk` contains `nocopy`
  slots and the fill constructor requires a copy element. Any alternative
  affine construction interface needs its own language/library grounds.
  Expanding the existing recursion frontier supplies no qualified win at 32;
  disabling it makes mixed input 30–37 percent slower at W2/W4/W8 despite more
  successful steals. Investigate packing span/batching or a balanced output
  representation while preserving stable order and machine-checked bounds.
  Short-phase CPU counter deltas and several small/skew controls remain
  unqualified, so they do not diagnose worker idleness. No general grain policy
  follows. Close this item only after the remaining construction and packing
  costs meet explicit work, space and performance criteria, or their tradeoffs
  are accepted.

- **The formal compute comparison has unresolved attribution and measurement costs.**
  The separate [amended DAG cost control](../research/investigations/compute-model/DESIGN.md#amended-cost-result-identical-image-control-failure)
  completed all forty identical-image cells with correct results and adequate
  interval resolution, but two cells exceeded their fixed symmetric wall/CPU
  criteria. No candidate comparison ran. The variation is unattributed and
  supplies neither a compiler-regression verdict nor evidence of host noise;
  any later attribution needs its own bounded discriminator, not a favorable
  rerun or relaxed threshold. The
  [retained-data diagnosis](../research/investigations/compute-model/DESIGN.md#follow-up-diagnosis-of-retained-identical-image-variation)
  reproduces all reductions but finds a CPU delta exceeding the eight-CPU
  wall-interval capacity, substantial within-process CPU variation, and fixed
  per-family label order in the earlier BFS control. Establish short-interval
  CPU accounting separately from cumulative process totals before using those
  deltas to diagnose worker idleness. Per-thread activity, placement and
  competing-load observations are absent, so neither runtime work nor the
  wall-time variation is attributed. Preserve both stopped controls and defer
  scheduling or threshold changes. The completed
  [two-process baseline diagnostic](../research/investigations/compute-model/DESIGN.md#baseline-cpu-accounting-result-short-interval-attribution-failure)
  found no work-batch violation, but 25 of 65 W4 gap counter deltas exceeded
  physical capacity despite compatible enclosing and terminal CPU totals.
  Short-boundary attribution is therefore contradicted; it is no longer an
  unanswered validity question. The remaining measurement work is to establish
  accuracy at useful longer intervals and the selected comparison scale with
  independent accounting evidence before interpreting CPU cost. A compatible
  lifetime total does not supply that accuracy. No replacement clock is
  selected, no outcome clears the old null, and the separate wall-time
  variation remains unattributed. Reopen on a bounded discriminator for those
  remaining questions, preserving this result if it is uninformative.
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
  attributed. The [PR 78 hosted records inspection](../research/investigations/compute-model/DESIGN.md#records-w4-hosted-comparison-remains-unresolved)
  retains repeated W4 suspects at `30198a19` and `53c68c29`: the latter has
  wall/CPU ratios 0.898002/0.908317 with four adverse pairs, while its records
  null is not suspect. Exact x86 objects show unchanged hot work and runtime
  objects alongside reduced capture transport and changed linked placement;
  they establish no cause or fix. Existing raw data lacks scheduling counters,
  and ARM or emulated results cannot clear this Linux signal. A retained-image
  W4 paired/null counter check is a possible discriminator, not selected or
  run. Defer mechanism changes until evidence distinguishes the possible causes;
  reopen on selection of a bounded Linux attribution experiment and preserve
  the suspect if that experiment is uninformative. Keep this item
  until the observations and measurement/detection tradeoff are explained by
  discriminating evidence, rather than a later pass or changed threshold.

- **A `propagate` statement cannot be a [PAR-1] window member.** The rule
  admits only `let`-bound and scrutinee calls, so `let a = f(); let b =
  propagate g();` never overlaps. Allowing a `propagate` second member would
  need the lowering to join the hand-out before the `Err` return; a future
  investigation, taken up when a real program shows the gap.

- **Alias facts for worker-run loop chunks.** A synthesized loop-split chunk
  or splitter has no source signature, so its range and reference captures
  carry no `noalias`. The sequential world inlines the chunk into its source
  function, which has the facts; a chunk run as a worker lane does not, so a
  vectorizable chunk loop may keep a runtime overlap check. The facts would
  need their own derivation from PAR-2 independence and the enclosing call's
  EFF-5 result, since sibling chunks write other parts of the same captured
  range concurrently. Impact and whether any current kernel pays such a check
  are unmeasured. Validate by inspecting the optimized worker chunks of the
  formal compute kernels for `vector.memcheck` and, where one appears,
  comparing chunk time with and without a hand-added fact. Deferred because
  the range-reference change covers source signatures only; reopen when a
  measured parallel kernel shows the check.

- **Compute-bench private adapters still pass aggregate ranges.**
  `research/experiments/compute-bench/array_reference_host.ll`,
  `first_index_host.ll`, `dag_fanin_host.ll` and the first-index observation
  rewrite in that Makefile spell a range argument as one `{ ptr, i64 }`
  aggregate. The compiler now passes it as pointer and count; the machine code
  is identical on the admitted targets, but LLVM text bound into a WF module
  must use the split form, and the first-index rewrite no longer matches the
  emitted head and refuses. These dated research inputs were left unchanged;
  update them before running those experiments with a compiler that includes
  the split, keeping an older baseline arm on its own adapter.

- **A discarded affine result's call never joins a hand-out group.** Its
  expression statement runs the result's release immediately after the call,
  reading the value between a hand-out and its join, so
  `compiler/src/lowering/builder.rs` leaves that call unrecorded and it ends
  any overlap group through it, although PAR-1 permits it exactly as the
  let-bound call. It could instead be a group's last member, as an addressed
  binding already may. No measured
  program discards an affine result beside an independent call, so the
  benefit is unverified. Reopen when such a program appears; validate by
  emitting the call as the join site with its release after the join and
  comparing published bytes at several worker counts with the sequential
  lowering.

## Platforms and host interfaces

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

- **At most eight peers may wait at once on a host without a native ring.**
  On Darwin, and under `WF_IO_NO_NATIVE_RING`, a peer wait beyond the eighth
  concurrent one has no helper and queues with no timeout. The readiness-
  driven adapter that would lift this, one poll over every queued descriptor
  from inside the park, was never built.

- **There is no source-level foreign-function boundary.** C enters only as a
  trusted linked definition of an ordinary declaration [PRE-1, SCOPE-3], which
  the checker cannot inspect, and a C program cannot call Whitefoot code
  through a stated ABI. A real systems program needs both directions: calling
  an existing C library and exporting a Whitefoot component. The
  [C ABI capsule idea](ideas.md#safe-c-abi-capsules) sketches export through
  opaque validated handles; import needs an explicit contract for ownership,
  layout, callbacks, foreign threads and failure, and a statement of what the
  compiler trusts. Validate on one real dependency in each direction, starting
  with the capsule experiment's misuse tests (stale handles, double drop,
  overlapping buffers, short outputs, allocation failure). This interacts with
  the module design for separate compilation. Close when a specified boundary
  and its conformance cases land, or the owner records why a narrower
  boundary suffices.
- **The driver's clang lookup is a fixed path.** `clang_executable()` in
  `compiler/src/bin/whitefootc.rs` hard-codes `/usr/bin/clang` on Linux/macOS
  (`clang` on PATH on Windows), so a host whose clang lives only elsewhere —
  a versioned-only `clang-18`, a Nix profile, or Homebrew LLVM — cannot run
  the driver even with clang installed. Validate whether to accept an
  explicit override, for example an environment variable, without changing
  which clang CI uses. Close when the owner decides for or against the
  override and, if accepted, its implementation lands.
- **One rejection per compilation.** The pipeline stops at its first
  violation, so an agent with several independent defects — two unproved
  subscripts in different functions, say — meets them one compile at a time.
  [DIAG-1] already leaves the order of violations at distinct nodes open, and
  the [diagnostic record](../research/investigations/readable-diagnostics/DESIGN.md#the-record)
  and its one-object-per-line JSON form can carry several. Reporting more than
  one needs the semantic checker to continue past a `CheckStop` without
  letting a later judgment consume an earlier failed premise, and stays
  deterministic. Unverified benefit: validate with a writer trial counting
  repair rounds on programs with two or more independent defects; reopen when
  such a trial or an agent harness shows the extra rounds dominate.
  One consumer is already promised: [ERR-2] says variant addition "surfaces
  site-enumerated edit lists", yet adding a variant to an enum matched in two
  functions reports only the first non-exhaustive `match` per run. Either
  every ERR-2 site of one enum is listed in a run, or ERR-2's sentence, which
  no other rule defines, is amended to what the toolchain provides.
- **A float constant in a rendered goal prints its internal form.** An FN-8
  `instantiated_goal` over a float constant renders it as
  `Float { ty: F64, bits: 4607182418800017408 }` instead of its source
  spelling `1.0_f64` (pinned in `driver::pinned_sentences` beside the integer
  goals). The goal renderer should print the constant's canonical FORM-5
  spelling, as it does for integers; update that pin with the fix.
- **Validate the default diagnostic rendering.** Text by default is
  provisional. The [readable-diagnostics investigation](../research/investigations/readable-diagnostics/DESIGN.md#default-format-text-with-json-on-request)
  selected it on reading cost for an agent (the lean OP-4 and FN-8 records
  measured there are 13-17% smaller than their JSON objects) and on the
  familiar summary-line shape, not on a measured repair loop. Run a writer
  trial over a fixed set of rejections covering lexical, grammar,
  canonical-form and proof families, comparing text and JSON defaults and a
  caret marker against a quoted span, with compile rounds to a fix as the
  criterion. Reopen the default, and the marker form, when that trial or an
  agent harness shows a difference.
- **Structured fields for stops that are not source rejections: declined.**
  Resource, invocation, internal-invariant, target-layout and backend stops
  print their stage value's `Debug` text as one `payload` field. They have no
  writer repair, and no consumer reads their fields separately. Reopen when a
  harness or experiment acts on one of them, for example a resource ceiling a
  writer can raise.
- **Text lists are ambiguous when an item contains `, `.** A diagnostic list
  such as `relations: [a, b]` prints items unquoted, so an item holding `, `
  cannot be split exactly from text. The JSON form carries each item as its
  own string and covers exact parsing; reopen only if an agent misreads such a
  list in practice.

- **A directory named through a symbolic link cannot be opened.**
  `open_directory` opens one component without following a link, which the
  walk relies on to leave enumerated links alone, and the prelude has no
  directory open over a `RelativePath`; `open_read` follows links but opens
  only regular files. So `wfgrep PATTERN ROOT` reports a root that is, or
  passes through, a link to a directory as `cannot read`, where `grep -r`
  follows a link named on its command line. Lifting it needs a prelude
  addition, a directory open over a `RelativePath` resolved as `open_read`
  resolves it, so it is a specification change deferred from the wfgrep root
  fix. Validate with a wfgrep case whose root and whose middle root component
  are links while an enumerated link stays unfollowed. Reopen when a program
  must walk a user-named linked directory.

- **`tests/programs/dir_walk.wf` truncates silently past its fixture.** It
  collects into constant-capacity frame storage and stops recording after 64
  entries in the whole walk, stops descending at depth 8, and clips a path at
  126 bytes, all while exiting 0, although its doc says it records every
  entry. Its one corpus case walks a three-level tree, so no check depends on
  the bounds. Either report each bound or collect through growable storage as
  `wfgrep.wf` now does; reopen when the program is pointed at a larger tree or
  its constant-capacity form stops being the point of the case.

## Code structure

- **The entailment flow module has outgrown one reader.**
  `compiler/src/semantic/entailment/flow.rs` has 17,271 lines, 15,040 of them
  in one `impl Analyzer` block; it grew from 8,670 lines on 2026-09-01 over 154
  commits. `compiler/src/semantic/entailment/state.rs` (7,755 lines, including
  a 1,729-line inline test module) and the tests in
  `compiler/src/semantic/tests/entailment.rs` (10,996 lines, 155 tests) grew
  with it. An agent reads such a file only in slices, and every
  responsibility's changes land in the same file. The impl already marks eight
  sections: binding prepass, place resolution and support, terms and relations,
  kill collection, obligations, statement walk, loop kill summary and canonical
  rendering. `flow/` already holds `conversions.rs`, `results.rs` and
  `sources.rs`, split out the same way, so moving each section's methods into
  its own `flow/` file is a mechanical first step. `state.rs` can move its test
  module to its own file and its dense-closure algorithms apart from the fact
  state and ledger types; the tests can group by the section they exercise.
  Validate that each move changes no behavior: identical `make check` results
  and a diff of moved items and module declarations only. Split when no open
  branch has large edits in these files, or one section at a time; close when
  every file named here is under 4,000 lines.

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
  Conditional fact representation cost is the separate compiler defect above.
  The conversion tests also retain an affine precision boundary: if `index`
  has only an affine image `first + second`, its checked integer conversion's
  saved Result does not preserve that image after `index` is replaced, even
  when a direct access can prove the bound. Conditional contexts carry L0
  relations, not affine value images. Validate whether a real saved-result
  consumer needs that extra relation, using paired direct/saved cases,
  mutation, joins and independent guards, and measure proof cost before
  extending the context; the existing numeric closure alone does not select
  such an extension.
  These language extensions are deferred because the selected ordinary
  local composition rule can be validated without widening the storage or
  predicate vocabulary.
- **Ownership transfer and reference-access forms.** Audit unnecessary
  owner-in/owner-out APIs now expressible with reference parameters and exact
  effect rows, the differing consumption spellings of calls, returns, matches
  and `propagate`, and repeated `deref`/`&deref` paths. `move` still marks a
  consumption boundary; `deref` distinguishes a reference holder from its
  referent and from owned `Box.inner`, so neither is redundant solely because
  the signature `own` qualifier was. Compare the same container and owned-link
  operations under proposed forms, preserving copy/drop capabilities,
  whole-owner consumption, atomic replacement, reference rebinding,
  invalidation and effect separation.
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

- **Independent retention authority remains unestablished.**
  The [multi-object result](../research/investigations/containers-and-resources/X1-LIBRARY.md#maintained-composite-correctness)
  establishes a coordinated Slab/HashMap/indexed-heap protocol for both weak
  expiry and retained deletion after both memberships retire. Its independent
  dictionary, expiry-order and owner ledgers cover wrong-store/stale handles,
  both removal orders, reuse and final cleanup; sequential/parallel observed
  images each release all 129 allocations exactly once. The
  [matched comparison](../research/experiments/container-representation/indexed-library/RESULTS.md#measured-result)
  grounds the owner's qualified shared-core selection for maintenance, not a
  proven speedup or native parity. The protocol does not protect bookkeeping from
  independently authored mutations: ordinary handles and nodrop tickets do
  not authenticate a Slab or make membership unforgeable, and stable slots do
  not supply surviving references. Defer stronger authority while consumers
  need only the demonstrated coordinated or weak-index contract. Reopen when
  an actual consumer needs independently held tickets or access spanning
  mutations; validate its complete acquisition/release and invalid-use chain,
  ownership cleanup and same-contract native costs before choosing any new
  mechanism.
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
  In that baseline, optimized migration initializes inactive payload bytes,
  stages live pairs and reads the displaced payload before its tag is used.
  The rejected unchanged-source initialization candidate above reduces the
  measured wide growth gap to about 1.19 times direct C with ordinary helpers and
  1.27--1.29 times with retained helpers, but normal replacement worsens to
  2.26--2.29 times and retained replacement remains 2.16--2.23 times. Public
  owning results still retain transfers. A forwarding follow-up must preserve
  every owner-return path and distinguish code-generation changes from their
  measured contribution; copy counts alone do not establish that contribution.
  Dense storage remains faster for wide growth but adds reserved metadata and
  dependent lookup, while a fresh sparse rehash retains two complete backings.
  Reopen for a workload dominated by these costs, preserving full backing and
  peak bytes, hash/load policy, retained helpers and exact cleanup. Defer a
  second maintained representation until that consumer supplies its grounds;
  the rejected compiler trial's optimizer losses and reopening grounds are
  recorded above.
  SIMD-group probing and a general projected-storage benefit remain untested;
  the earlier [native hash-slot study](https://github.com/mbbill/Whitefoot/blob/38c28403a2defd0b65b8a2ab2b5e4794315e9940/research/experiments/hash-slot-occupancy/RESULTS.md)
  did not establish a recurring tag-check tax. A working library does not
  close either question or imply a universal native-performance ceiling.

- **Ordered insertion replacement costs need attribution.** Both the
  [aggregate-result candidate](../research/experiments/container-representation/ordered-library/RESULTS.md#single-descent-insertion-candidate)
  and [borrowed-promotion follow-up](../research/experiments/container-representation/ordered-library/RESULTS.md#borrowed-promotion-follow-up)
  regress on replacement despite insertion gains. The second removes recursive
  aggregate clearing without curing the loss. One promotion-slot initialization
  per put, Pair placement/swap/result transfers, occupancy checks and substantial
  stack frames remain; their elapsed shares are not isolated. Avoiding the
  baseline's duplicate miss search and recursive Pair transport remains useful
  only if replacement cost is preserved. Defer another source variant: no third
  candidate belongs to this completed comparison. Reopen when a concrete
  consumer or controlled source/lowering discriminator isolates a material
  cause and supplies grounds for a new experiment. Validate unchanged owner
  identities, refusal behavior and exact release/allocation counts, then the
  complete normal/retained matrix including replacement cells and independent
  control observations. Keep general aggregate-result ABI and placement work
  under the existing compiler items; a different return form alone no longer
  supplies the reopening ground.

- **Ordered-map occupancy and tree choice remain workload-dependent.** The
  [reserved-storage comparison](../research/experiments/container-representation/ordered-library/RESULTS.md#allocations-and-reserved-storage)
  records 48.6% peak reserved-slot utilization during 4096-pair bundled-tree
  churn, versus 74.6% for direct C; wide peak storage is about 2.37 MB versus
  1.50 MB and native AVL's 1.18 MB. A different repair policy or tree shape
  could reduce vacant wide storage and churn cost. This is one deterministic
  stream, not an occupancy histogram or a measured WF AVL. First attribute
  node growth to split/merge and reinsertion with per-node occupancy evidence;
  then compare one justified alternative under the complete arbitrary-owner
  map contract, including replacement/refusal, range visits, exact cleanup,
  requested/peak bytes and normal/retained timings. Scalar and range tradeoffs
  must remain visible. Defer a second maintained representation until a
  concrete index supplies its governing
  workload; reopen before choosing a default ordered representation or when
  an index is dominated by wide reserved storage or churn.

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
  `nuw`/`nsw` on the exact family. A `&[T]` range parameter crosses calls as
  its element pointer and count, and the pointer carries the same facts
  except `dereferenceable` (`compiler/backend-facts`; the
  [range-reference fact investigation](../research/investigations/range-reference-facts/DESIGN.md)
  records the derivation and the removed vectorizer overlap check). The later
  qualified Ring payload-address
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
