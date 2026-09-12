# Audit: lowering against the design tree

Module: `compiler/src/lowering.rs`, `compiler/src/lowering/builder.rs`, every
file under `compiler/src/lowering/builder/` (`buffers.rs`, `loops.rs`,
`probe.rs`, `results.rs`, `runs.rs`, `scalar_grain.rs`, `slices.rs`,
`split.rs`, `storage.rs`, `targets.rs`), and `compiler/src/backend/storage.rs`
(about 10,800 lines total, `lowering/tests.rs` included). Target-independent
lowering from the checked program to the private IR, actualization of
proof-derived parallelism and the wide-probe fast path, and the backend's
aggregate storage-placement plan. Direction: code to tree — finding choices
the code embodies that `design/` does not record, not the reverse. The LLVM
emitter (`compiler/src/backend/emitter*`) is a different audit's subject and
is treated here as a black box; `compiler/src/backend/target.rs` and
`compiler/src/backend/tests/` are touched only where a lowering-side constant
is pinned against them.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md` and all of
`design/compiler/` (`cleanup-traversal.md`, `parallel-lowering.md` with its
children `two-worlds.md` and `parallel-runtime.md`, `resource-exhaustion-floor.md`,
`tag-only-lowering.md`, `wide-probe-lowering.md`), `design/language.md`,
`design/language/data-model.md` and its child `tag-only-equality.md`,
`design/language/parallelism.md` and its child `permission-judgment.md`,
`design/language/ownership.md` and its children (`affine-replacement.md`,
`copy-classification.md`, `no-reborrow.md`, `slice-result-provenance.md`).
Also read for grounding, since the module's own subject touches it directly
(the staged I/O completion pipeline): `design/language/system-interface.md`
decision 1, which assigns "completion and host scheduling" to lowering with
no further language-level constraint — the reason §3.1 below finds no
compiler-tree node picking that assignment up either. Checked against
`spec/kernel-spec.md` wherever the code cites a rule id; a representative,
policy-weighted sample (OWN-2/3/6, EFF-5, STOR-3/4, PROV-6, BLK-2, PAR-2/3)
was read against the code's own claim about it and found consistent, not an
exhaustive line-by-line check of every citation in the module.

Two scope notes that shape section 1, matching the audited code rather than
the node list handed to this audit:

- `design/compiler/cleanup-traversal.md` and `design/compiler/resource-exhaustion-floor.md`
  have no implementing code in this module family at all. Cyclic-release
  code generation (the self-calling drop action cleanup-traversal.md
  decides) and the stack-probing-attribute/abort-record machinery
  resource-exhaustion-floor.md decides both live in the emitter and target
  stage, outside this audit's files; `compiler/src/lowering.rs::type_derives_release`
  (`:427`) is the shared "does this type derive release at all" predicate
  both that emitter code and this module's own `loops.rs::drops_release_nothing`
  (`:1143`) consult, and it decides no cyclic-drop *action*, only whether one
  exists. Neither node is force-fit into section 1 below.
- `design/language/ownership/copy-classification.md`, `no-reborrow.md`, and
  `slice-result-provenance.md` decide what the *checker* admits (which values
  are copy, which reborrow shapes are legal, which slice-origin summaries a
  call may report); lowering never re-derives or branches on any of the
  three; it reads whatever already-checked expression or place the tree
  handed it and lowers it uniformly regardless of which clause of any of
  these three admitted it. Only `ownership.md`'s own root decision (that
  lowering must not invent a borrow lowering never itself decided) has a
  distinguishable realization here, cited below.

## 1. Covered by the tree

- `design/compiler.md` (decision 1: one safe-Rust crate, simple
  implementations over ordinary collections) — the whole module: only
  `Vec`/`HashMap`/`HashSet`/`BTreeSet`, no `unsafe` block anywhere in
  `lowering.rs`, `builder.rs`, `builder/*.rs`, or `backend/storage.rs`.
- `design/compiler.md` (decision 2: admits and lowers by the specification's
  rules alone, never by recognizing a name, signature, or shape) —
  `compiler/src/lowering.rs:1` ("It performs no source admission, label
  lookup, exhaustiveness decision, or ownership judgment"); the graceful
  no-op fallback every shape-recognizing actualization takes on any mismatch:
  `compiler/src/lowering/builder/split.rs::split_counted_range` (`:191`,
  returns `Ok(false)` and "leaves the builder exactly where it found it") and
  `compiler/src/lowering/builder/probe.rs::emit_probe_skip_if_recognized`
  (`:344`, "any recognition or representation mismatch falls back to the
  ordinary lowering with zero change").
- `design/compiler.md` (decision 3: an unimplemented capability stops as an
  explicit gap, never a rejection) — `compiler/src/lowering.rs::LoweringFailure`
  (`:2392`) has exactly two compiler-internal variants and lowering runs only
  on an already-accepted `CheckedProgram`, so neither variant is ever a
  source-language rejection.
- `design/compiler.md` (decision 4: no acceptance path depends on a timeout,
  budget, or iteration order) — `compiler/src/lowering.rs::OverlapLowering`'s
  own doc (`:1517`: "no accepted program changes... this selects only whether
  a permitted compute group or direct completion schedule reaches the IR")
  and `RecursionBudget`'s own doc (`:1569`: "read by the emitter and by no
  acceptance path... it cannot reject a program, and compiling the same
  source with any of these three forms accepts exactly the same programs and
  computes exactly the same values").
- `design/language.md` (decision 1: every required fact is machine-checked
  and erased before lowering; no runtime trap) —
  `compiler/src/lowering/builder.rs`'s handling of `CheckedStatement::Proof`
  (`:1323`, "have no runtime value, effect, branch, or instruction") and of
  `CheckedExpression::PostconditionResultMeasure` (`:2024`, returns
  `InvalidCheckedProgram`: "the checker discards it with the clause's typing,
  so no checked program carries one here").
- `design/language.md` (decision 3: one observable behavior; whatever the
  compiler hands its backend is an implementation detail the language never
  sees) — `compiler/src/lowering.rs::OverlapLowering`'s doc (`:1517-1532`):
  every variant computes the same values and differs only in whether a
  schedule reaches the IR at all.
- `design/language/parallelism.md` (decision 1: parallel permission derives
  from what the checker already proves; a failed permission leaves the
  program sequential rather than rejecting it) — the `overlap: OverlapLowering`
  gate threaded through every builder method that reads `self.permissions`
  (`compiler/src/lowering/builder.rs::{overlaps, completion_steps}`, `:969`,
  `:1016`) and every `Decline` in
  `compiler/src/lowering/builder/split.rs::decline` (`:339`): a declined
  actualization always leaves the ordinary, already-accepted lowering
  untouched.
- `design/language/parallelism.md` (decision 2: permission and actualization
  are separate judgments) — the permission table (`self.permissions`) is
  read-only throughout lowering and never widened; the checker's own
  judgment surfaces unconditionally through `--par-ledger` while
  `compiler/src/lowering.rs::IrProgram::actualization_ledger` (`:2281`,
  "these state what *this* lowering did with a permission, which is a
  different fact") is the separate, actualization-only record.
- `design/language/parallelism.md` (decision 3: a counted loop is a
  parallel-permission site in its own right) —
  `compiler/src/lowering/builder/loops.rs::lower_counted_range` (`:102`)
  and `compiler/src/lowering/builder/split.rs::split_counted_range` (`:191`)
  treat `CheckedStatement::CountedRange` as its own actualization path,
  entirely distinct from the sibling-call `overlaps()` mechanism.
- `design/compiler/parallel-lowering.md` (decision 1: two lowerings of a
  permitted loop, the leaf is the loop and never one iteration) —
  `compiler/src/lowering/builder/split.rs::{build_chunk, build_splitter}`
  (`:400`, `:473`) and `compiler/src/lowering.rs::{IrOperation::LoopSplit, IrSynthesis}`
  (`:1362`, `:2047`): the chunk is "the same block graph
  [`IrBuilder::counted_range_graph`] builds at an unsplit site, built by the
  same code," and the sequential world's call is the loop exactly.
- `design/compiler/parallel-lowering.md` (decision 3:
  `--par-sequential-refusal` enters the callee's existing sequential clone)
  — `compiler/src/lowering.rs::OverlapLowering::{OnWithSequentialRefusal, OnWithRecursionBudget}`'s
  `sequential_refusal` field and its `sequential_compute_refusal` derivation
  in `compiler/src/lowering/builder.rs::lower_checked` (`:33`).
- `design/compiler/parallel-lowering.md` (decision 4: scalar-leaf suppression
  is offer selection after checking, never an acceptance change) —
  `compiler/src/lowering/builder/scalar_grain.rs` in full: its own module
  doc states "no source name selects behavior and no acceptance judgment
  consumes it."
- `design/compiler/parallel-lowering/two-worlds.md` (decision 5: the initial
  recursion budget is the runtime's own answer by default, with an explicit
  pin-or-off control) — `compiler/src/lowering.rs::RecursionBudget{Off, RuntimeDerived, Pinned}`
  (`:1583`) and `OverlapLowering::OnWithRecursionBudget` (`:1546`).
- `design/compiler/tag-only-lowering.md` (word-sized tags cost a measured
  penalty; tag-only enums need a compact, uniform representation) —
  `compiler/src/lowering.rs::IrNominal::is_tag_only_enum` (`:666`), consumed
  by `compiler/src/backend/storage.rs::is_stored_aggregate` (`:22`) to keep a
  tag-only enum out of aggregate/stack storage entirely, uniformly at every
  value, call, and match site (the decision's own "consistently across
  values, calls, equality, and match dispatch"); the specific bit width
  (1-bit vs. 32-bit) is the emitter's downstream choice, out of this audit's
  scope, but the classification it needs is supplied here and nowhere else.
- `design/compiler/wide-probe-lowering.md` (recognize a byte-walk loop and
  give it a header fast path built on one probe operation; any mismatch
  falls back with zero change and acceptance never consulted) —
  `compiler/src/lowering/builder/probe.rs` in full
  (`recognize_byte_walk`, `:46`; `emit_probe_skip_if_recognized`, `:344`)
  and `compiler/src/lowering.rs::IrOperation::BufferProbeSkip` (`:1233`).
- `design/language/data-model/tag-only-equality.md` (a distinct enum-domain
  equality operation family comparing declared-variant identity directly,
  never an integer conversion or per-type mapper) —
  `compiler/src/lowering.rs::IrOperation::EnumEquality` (`:1136`) and its
  lowering at `compiler/src/lowering/builder.rs`'s
  `CheckedExpression::EnumEquality` arm (`:1871`): one dedicated operation
  family, kept separate from `IrIntegerOperation::{Equal,NotEqual}` and from
  ordinary `IrOperation::Boolean`.
- `design/language/data-model.md` (decision 2: append-only stable identity
  pays no generation cost; access is a bare check) —
  `compiler/src/lowering.rs::IrOperation::{ArrayIndex, RunIndex, BufferIndex, SliceIndex}`'s
  own doc (`:1145`: "one discharged source subscript read [OP-4]: the
  checker has already derived the bounds obligation, so no runtime branch is
  emitted in any build mode") and their lowering in `builder.rs`/`buffers.rs`/`slices.rs`/`runs.rs`.
- `design/language/data-model.md` (decision 4: full fixed arrays, initialized
  prefixes, and circular windows are distinct states; placement and stable
  identity are separate axes with no mandatory indirection on dense values)
  — the four-way split of `compiler/src/lowering.rs::IrType` (`:307`):
  `Array` (dense, no window), `FixedVector` (frame-resident inline run plus
  its own `len`/`head` words), `Vector` (a store-taken run reached only
  through its owner's descriptor), `Buffer` (dense, runtime length, no
  window); `compiler/src/lowering/builder/runs.rs`'s module doc (`:1`)
  states the same split in its own words.
- `design/language/data-model.md` (decision 5: source signatures keep their
  checked ownership and access modes independently of representation) —
  `compiler/src/lowering.rs::{IrSourceMode, IrSourceSignature, IrSourceArgument, IrSourceCall}`
  (`:2065`-`:2120`), populated by `compiler/src/lowering/builder.rs::{lower_source_mode, lower_source_argument, note_call_result}`
  (`:457`, `:465`, `:1119`) purely for the backend's later use and never
  consulted by lowering's own judgment; `lowering/tests.rs`'s
  `source_signature_modes_distinguish_identical_descriptor_representations`
  and `source_signature_modes_are_not_invented_for_synthesized_functions`
  (`:202`, `:264`) pin exactly this independence.
- `design/language/ownership.md` (decision 1: explicit borrow modes at
  mode-bearing positions, fixed by the checker) —
  `compiler/src/lowering/builder/storage.rs`'s module doc (`:1`: "Lowering
  never infers a borrow from source shape or type alone") and
  `promote_binding_if_needed` (`:278`), which materializes an address only
  for a binding the checker's own `addressed_bindings` set already named.
- `design/language/ownership/affine-replacement.md` (decision 1: `replace`
  is capture-the-target, evaluate-the-replacement, then one atomic exchange)
  — `compiler/src/lowering/builder.rs::replace` (`:2275`): "capture the
  target, evaluate the RHS, then exchange the old and new owners at one
  commit."

## 2. No decision needed

- Every dense identity (`IrValueId`, `IrBlockId`, `IrNominalId`, `IrConstantId`,
  `IrSystemOperation`) is a checked `u32`/`u8` newtype; construction goes
  through `checked_add`/`try_from` and fails closed to
  `LoweringFailure::CounterOverflow` rather than wrapping — ordinary
  overflow-safety practice, the same pattern both finished audits already
  noted in their own modules.
- `type_derives_release`'s worklist-based reachability walk over the
  compile-time type graph, with a `visited: HashSet` guarding the one place a
  cycle can occur (a `Box`/`Arena` referent chain), is an ordinary bounded
  graph-reachability idiom over a finite, compile-time structure — distinct
  from the runtime cyclic-release *action* `cleanup-traversal.md` decides,
  which this predicate only feeds and does not itself choose (see the scope
  note above).
- One flat `Vec<BuildingBlock>` plus a single "current block" cursor and an
  ordinary `HashMap<BindingId, IrValueId>` scope per builder
  (`compiler/src/lowering/builder.rs::IrBuilder`) is a textbook single-pass
  SSA-construction shape, not an independent design.
- Lazy storage promotion (`collect_addressed_bindings`'s whole-function
  pre-pass plus `promote_binding_if_needed`'s per-binding check) materializes
  a stable address only for a binding some checked expression already
  borrows, instead of allocating storage for every local uniformly —
  ordinary allocation-avoidance engineering built directly on the checker's
  own borrow classification, not a new judgment.
- Every `[BLK-0]`/`[BLK-1]`/`[BLK-2]`/`[BLK-3]` container row in `runs.rs`
  is one small function named for its row (`lower_kernel_call`,
  `lower_store_take`, `lower_store_box`, `lower_boundary_row`), each a
  mechanical transcription of the specification's own operation-table entry;
  none contains a name-, spelling-, or shape-based special case.
- The "prepare the target, evaluate the right-hand side, then commit" shape
  every `set`/`replace`/`set_list` form uses (`builder/targets.rs`) is a
  direct, mechanical reading of `[SET-1]`/`[SET-2]`/`[LIV-2]`'s own
  evaluate-then-commit order, not an independent choice.
- `IrSourceArgument`/`IrSourceCall`/`note_call_result` bookkeeping records
  each call's already-checked argument classification and the block/value
  where its definition landed purely for the out-of-scope backend's later
  alias analysis; lowering itself never inspects or branches on these tags.
- `fixed_measure`'s direct lookup against the checked `MeasureCell` table,
  loading nothing for a compile-time-constant cell, is a mechanical
  consequence of `[MSR-1]`/`[MSR-2]` exactly as the checker already
  discharged them.
- Deterministic, allocation-ordered flat `Vec` construction throughout
  (parameters, blocks, values, nominals, constants, synthesized functions
  filed by reserved ordinal) with no `HashMap`/`HashSet` iteration ever
  reaching emitted order — matches the compiler root's hash-order
  independence rule as ordinary practice, not a new decision.
- `scalar_grain.rs::small_leaf`'s shape test (one block, scalar
  parameters/result, a bounded count of scalar-only operations, empty
  drops) is a narrow, direct reading of the scalar-leaf-suppression policy
  already covered above; the shape test's own boundaries need no separate
  rationale.
- `IrDropSubject::{Value, Place}` (`compiler/src/lowering.rs:1415`) lets a
  release of addressed content reference the existing typed place instead of
  first snapshotting the whole owner into a value
  (`builder.rs::lower_drop_subject`, `:2511`) — an ordinary avoid-the-copy
  optimization once lazy storage promotion already exists, not an
  independent choice about what a release does.
- `backend/storage.rs`'s test suite runs a from-scratch "concrete oracle"
  (`compare_execution`, `:685`) that executes value snapshots and physical
  slots together without reading liveness or interference, beside the
  algorithm under test — the same independent-evidence-lane testing
  discipline both finished audits already noted as ordinary engineering in
  their own modules, not a new decision here.
- Checked/saturating arithmetic (`saturating_add`, `saturating_mul`,
  `saturating_pow`, `checked_add`) throughout `split.rs` and
  `backend/storage.rs` instead of raw arithmetic — ordinary overflow-safety
  practice, unrelated to language semantics.
- `IrCompletionPipeline`'s closed three-state driver enum
  (`Pending`/`OneSlot`/`BoundedBatch`) keeps every schedule-relevant fact
  (which block drains, which value is the delayed result) in the IR's own
  typed topology instead of letting the backend infer it from block shape —
  ordinary "make illegal states unrepresentable" API design once the
  pipeline mechanism itself exists; the mechanism's existence and its
  specific ring depth are a choice without a node (§3.1 below), but the
  enum shape carrying it is not a second decision.
- `IrBuilder::{new, finish}` (`compiler/src/lowering/builder.rs:609`, `:685`)
  refusing an unterminated block or a builder left mid-block is ordinary
  well-formedness bookkeeping for a hand-rolled SSA builder, not a language
  rule.

## 3. Choices without a node

**1. The staged loop I/O-completion pipeline is a complete lowering
architecture with no design-tree node at all, and its ring depth is a
hard-coded 2 with per-slot storage cost never computed.**
`IrCompletionPipeline` carries one of three driver states — no descriptor
yet, a materialized one-slot feeder/drain edge, or a materialized
two-slot bounded-batch issue/drain ring — and
`lower_bounded_completion_range` recognizes an eligible loop shape
(`direct_staged_match`/`direct_staged_tail`) and builds the batch form,
fixing `let (ceiling, slots) = (2, 2);` unconditionally: every staged loop
that qualifies gets exactly two operations in flight, and the window
query's `slot_bytes` argument is always `0`
(`IrCompletionWindow::new(counted_span(lower, upper), 0, ceiling)`), never
computed from the privatized per-iteration storage the loop actually
carries. `lowering/tests.rs:1401` pins the emitted call as
`@wf__completion_window(i64 4, i64 0, i64 2)`, so this is the deliberately
shipped behavior, not code left mid-implementation.
Alternative: `design/language/system-interface.md` (decision 1) explicitly
assigns "completion and host scheduling" to lowering with no further
language-level constraint, so the door was open to derive the window bound
`K` from real per-iteration storage cost and a runtime capacity query, as
`research/investigations/io-model/LOOP-PIPELINE.md` §3.1 designs at length —
"the compiler supplies `slot_bytes` (computed statically from the
privatized places' lengths...) and a `ceiling` from storage cost," with a
worked example putting `K` at 8-32 for a real workload — or, short of that,
naming the ring depth as a constant or CLI knob instead of an inline
literal buried in one function.
Where: `compiler/src/lowering.rs::{IrCompletionPipeline, IrCompletionWindow, IrCompletionOneSlotDriver, IrCompletionBatchDriver}`
(`:1713`-`:2031`); `compiler/src/lowering/builder/loops.rs::lower_bounded_completion_range`
(`:168`-`:830`, the literal at `:810`) and `direct_staged_match`/`direct_staged_tail`
(`:1158`, `:1169`); consumed by
`compiler/src/backend/storage.rs::FunctionStoragePlan::build`'s `pipeline`
parameter (`:60`-`:74`, `:127`-`:129`) to exempt per-slot storage from the
acyclic-reentry coalescing rule.
Reason: no reason recorded for the number 2 itself. The mechanism's
incremental history is all title-only: `98b676ae`/`72aac9f8` (2026-08-28,
"window query, deferred doorbell, retire-and-retry") and `2bed9cc1`
(2026-08-29, "the pipeline descriptor carries a slot count and each block's
slot parameter") carry no body, and the commit whose diff contains the
exact `(2, 2)` literal today, `6816e9bd` ("Restore ordinary-stack compute
execution independently of I/O waits"), does not discuss why two rather
than a computed or larger fixed value. `research/investigations/io-model/LOOP-PIPELINE.md`
is the closest record of intent, but it explicitly designs a
*runtime-computed*, potentially much deeper window (`K` up to 32-64 in its
own worked examples), and its own closing section (§9.7, "What is decided,
and what is not") records that batch 0089 decided "nothing" about the
mechanism proper — so the shipped fixed-2, zero-`slot_bytes` form is a
narrower, later, and unrecorded departure from what the investigation
actually explored, not that investigation's stated conclusion.
Effect: acceptance is unaffected — a loop this recognizer misses lowers
ordinarily. Performance and structure: every staged loop's I/O overlap is
capped at 2 operations in flight regardless of what the loop's own storage
or the host's real capacity could support, and any future change to that
cap, or to whether storage cost is accounted for at all, is currently a
silent one-line change in `loops.rs` with no design-tree ground to check it
against.

**2. The runtime split allowance's static body-weight estimate is a bounded
three-round substitution with a fixed 16x-per-nesting-level multiplier, not
a precise fixed point.**
`assign_weights` estimates each chunk's per-iteration cost by charging every
instruction `LOOP_FACTOR.saturating_pow(depth.min(4))` (`LOOP_FACTOR = 16`),
where `depth` comes from `loop_depths`'s back-edge-reachability test
(deliberately not block order, because the builder emits a loop's exit
block before its body, which would misread most `break` edges as back
edges), then propagates callee costs through exactly three rounds of
substitution — "Recursion is bounded by the same count rather than by a
cycle test: an estimate does not need a fixed point" — rather than to an
actual fixed point over the callee graph's cycles. The result becomes
`IrOperation::LoopSplit`'s `weight` field, which the runtime multiplies by
the span to decide how far a split may recurse.
Alternative: compute the exact fixed point (for instance by
strongly-connected-component decomposition of the callee graph, mirroring
the SCC-based approach `design/compiler/cleanup-traversal.md` chose for
cyclic release graphs), or use a different bounded approximation — another
per-level multiplier, another round count, or a multiplier calibrated
against measured data the way the runtime-side allowance constants in
`design/compiler/parallel-lowering/two-worlds.md` and `parallel-runtime.md`
were.
Where: `compiler/src/lowering/builder/split.rs::{assign_weights, LOOP_FACTOR, cost, loop_depths, reachable_without}`
(`:827`-`:987`).
Reason: no reason recorded for the specific constants. The mechanism was
introduced whole in `f7127c03` (2026-08-23, "compiler: actualize a
permitted counted loop as a range split"), whose body states only that
"the caller's static weight estimate over the emitted IR" feeds the runtime
query and, separately, that "an estimate that is wrong by a factor still
lands on the measured grain plateau, which is flat over four thousandfold"
— a claim about how loose the estimate is allowed to be, not about why
`LOOP_FACTOR = 16`, three rounds, or a depth-4 cap were the way chosen to
build a cheap-enough estimate. `research/investigations/proof-derived-parallelism/loop/DESIGN.md:125`
states only "compiler-estimated body weight," the same level of detail as
the commit.
Effect: performance only, not acceptance. The commit's own four-thousandfold
tolerance claim suggests the exact constants matter little in practice, but
nothing records that tolerance as the reason for choosing them, and a
future change to any of the three numbers has no recorded ground to answer
to.

**3. Aggregate SSA-value storage is placed by a from-scratch
liveness-and-interference-graph coalescing pass, scoped to two narrow
candidate-pair families.**
`FunctionStoragePlan::build` gives every "stored aggregate" value
(`is_stored_aggregate`: every `Array`/`FixedVector` and every non-tag-only
`Struct`/`Enum` nominal) its own storage slot by default, then
`FlowGraph::plan` runs a textbook fixed-point liveness analysis (`live_in`)
and builds a full interference graph (`interference`) before greedily
coalescing only two specific candidate-pair families into shared backing:
an operation's own declared "reuse" operand (`RunBoundary`'s run,
`InsertStruct`'s aggregate) and a block-parameter transfer pair across a
`Jump` edge — "unrelated dead values are deliberately not packed into the
same backing." `select_destinations` then further redirects a single-use,
single-member slot's construction directly into an `AddressOf` destination
when the CFG proves it safe.
Alternative: no coalescing at all — every stored-aggregate value keeps
independent backing, the simplest reading of `design/compiler.md`'s "simple
implementations over ordinary collections"; a coalescing scope wider than
these two candidate families, approaching a general graph-coloring register
allocator; or a coalescing decision driven by an explicit checked-ownership
signal from the semantic layer instead of an interference computation
re-derived here from scratch.
Where: `compiler/src/backend/storage.rs::{FunctionStoragePlan, FlowGraph, is_stored_aggregate, select_destinations, interference, live_in, plan}`
— the file's whole production section, `:1`-`:626`.
Reason: no reason recorded. The mechanism was built across three
consecutive checkpoint-style commits with no design discussion in any body:
`f5dab70c` (2026-09-06, "Implement typed container places and aggregate
destinations" — "Checkpoint the ordinary storage path, value snapshots,
deterministic slot reuse, target evaluation order and executable ownership
evidence"), `d5722f10` (2026-09-07, "Release owned places without
whole-aggregate cleanup snapshots," bodiless), and `c4964ce2` (2026-09-07,
"Construct fresh aggregate bindings in their planned destinations,"
bodiless).
Effect: performance and structure, not acceptance — every stored-aggregate
value is correct however it is placed (the module's own from-scratch
execution oracle, `compare_execution`, checks this independently of the
liveness/interference machinery for every test program), but the scope of
what may share backing, and therefore how much stack traffic a function
generates, is fixed by this one file with no design-tree record of why
coalescing exists at all or why it stops at exactly these two candidate
families.

## Summary

- Covered by the tree: 21
- No decision needed: 15
- Choices without a node: 3
