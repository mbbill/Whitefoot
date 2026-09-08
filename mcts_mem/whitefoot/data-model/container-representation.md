- Represent owned aggregates through general typed storage places, with explicit
  initialization/ownership state and lifetimes. Use caller-provided result
  destinations and consumed-place reuse when sound; source value transfer is not
  permission to duplicate owners or a requirement to copy an entire payload at
  every element operation. Fields, indices, and cell dereferences share the normal
  place path.
- Source-function signatures retain their checked ownership/access modes
  independently of parameter and result representation. Compiler-synthesized
  functions carry no invented source signature. These modes alone supply no loan
  origin, lifetime, or input/result aliasing permission.
- User-call arguments retain their checked occurrence uses independently of
  their actual value identities. A direct borrow result retains its candidate
  actual argument; this relation may cover a wider place than the returned
  suffix and supplies neither exact disjointness nor a complete loan lifetime.
  Absence of a candidate record does not imply that an owned view has no borrowed
  backing.
- Distinguish full fixed arrays, initialized prefixes, and circular windows.
  Persistent fixed extent and full initialization belong to the relevant type or
  state; variable length and head are not universal array metadata. Placement and
  stable identity are separate axes, with no mandatory handle/store indirection
  on dense values. Current specification bytes still define source behavior.
- Keep initial public storage transitions compiler-checked. Make finite ranges,
  loans, storage identity, and initialized responsibility explicit internally, but
  do not mistake a concrete checker model for a verified symbolic library or a
  production erasure path. General source-library representation authority remains
  the D17 direction, deferred until an exact checked implementation and a real
  workload establish its soundness and usefulness.
- Prefer inherent fixed-block type facts and existing verified count contracts
  for the measured pool. Do not infer general element-refinement requirements
  from a fact that a fixed-capacity type already preserves. Do not infer that
  dynamic capacities or graph relations can always be reduced to constants.
- Treat current Whitefoot tests and small programs as capability/cost witnesses,
  not evidence of production workload distribution. Use an explicit cross-domain
  source sample of established applications to recover candidate needs; prevalence
  and hot-path priorities require representative runtime evidence. Distinguish
  static use sites from measured execution, sizes, allocations, and lifetimes, and
  retain corpus and language biases. Translate external requirements into
  Whitefoot probes without treating a library API, a qualitative source sample,
  or a successful probe as a representative demand distribution.
- Infer the need behind an external representation: separate semantic and
  measured cost constraints from language/library workarounds, compatibility,
  and history. Ask what survives without those expression limits. Preserve actual
  identity, lifetime, failure, and resource contracts in a Whitefoot alternative;
  validate its advantage rather than assuming static proof removes every cost.
  Mixed or unknown causes stay explicit, and cross-language frequency is not
  proof that a representation is necessary.
- Keep physical capacity, initialized payload, logical membership, NULL validity,
  selection mappings, and reserved/ready states distinct. A known address or
  distinct input index is not permission to read a payload or proof of distinct
  mutable targets. Nullable columns and many-to-one aggregation supply concrete
  pressure beyond a universal prefix/window model; they do not by themselves
  select bitmap storage, packed pointers, or a new public proof framework.
- Preserve composed relations through library operations: map/array correspondence
  during heap repair, reservation accounting before row publication, and the
  oldest remaining reader's reclamation boundary. Stable keys, stable addresses,
  and collection positions are different choices. A source representation's
  convenience under GC or unchecked pointers is not evidence that every
  Whitefoot container needs handles or stable backing.
- A checked empty run may discharge its own backing responsibility after every
  element obligation is gone, with the provider and loan rules still satisfied.
  Emptiness of a field does not discharge an independently declared linear wrapper.
  This is a selected amendment, not a claim that today's empty-run rejection is
  already fixed.
- Finite projected result contracts use the normal entry snapshots, publication,
  and write invalidation. Two-span results must carry each physical range and
  origin through helper boundaries; a second descriptor without checked sibling
  disjointness and lifetime is insufficient. Do not require nontrivial published
  relations for every internal representation coordinate.
- Destination selection preserves source evaluation, allocation failure, and
  release order. Acquire backing first in source when allocation-first construction
  is required; do not hoist an allocation across observable initialization just to
  obtain destination passing. Retain every partial-construction responsibility.
- Fresh binding destinations are selected before physical frame layout. A
  single-use value with independent backing can initialize its same-block
  binding directly in an acyclic activation or a selected pipeline's retired
  per-slot storage. Other cases retain separate storage; source consume modes
  grant no input/result aliasing permission.
- Capture mutation targets before the RHS and revalidate their writability under
  the complete post-RHS loan state. Read the displaced old owner at the admitted
  replace commit. An address's storage must survive RHS effects;
  a live root binding alone is insufficient if a descriptor replacement can
  retire its backing. The legacy borrowed-buffer path explicitly lacks that
  capability; do not turn this implementation limit into a source rejection.
- On phi edges, snapshot incoming values before cleanup and write coalesced
  destinations after cleanup. The destination can share a predecessor owner's
  storage through another edge without owning it before that owner's final read.
- A published/borrowed address lasts through its actual use, including staged join
  return, result consumption, and corresponding retirement. DONE is not reuse
  authority. The runtime interface stays opaque; no private slot size or worker
  placement policy selects container correctness.
- The bounded lane driver carries issue-local values and addressed owners per
  pipeline slot. An addressed owner retains its address over distinct backing
  until its remainder and checked releases finish; its mutable contents are not
  snapshotted before the outstanding callee completes.
- Cleanup subjects distinguish saved values from initialized content at typed
  places. A checked release group captures the content its actions need before
  its first release. A no-op owner node requires no whole-value materialization;
  release responsibility does not create another aggregate owner.

## Facts

- 2026-09-06 owner correction: Whitefoot has no substantial real-application
  corpus yet. This selection's small executable programs establish particular
  requirements and implementation failures, not how often workloads occur. No
  external C++/Rust/Go distribution study was performed; production prevalence
  remains unmeasured. The general storage foundation has witness-based support,
  while the subsequent qualitative external sample supplies additional needs
  without measuring their prevalence or runtime importance.
  External forms are themselves constrained by their languages; the owner requires
  recovering underlying needs rather than treating observed forms as requirements.
- 2026-09-06 selection ground: main's `existence-only` correction separates a
  derived need from a minimality-selected form. The independent review and
  executable experiments compare forms against performance, checked authority,
  default source shape, and concrete resource contracts. Workload frequency
  selects priority, not whether a necessary capability exists. The complete
  selection and first implementation criteria are in
  [the container assessment](../../../research/investigations/containers-and-resources/REASSESSMENT.md).
- 2026-09-06 measurement: on one arm64 macOS/Clang 21 run, the dense construction
  plus four-update trace at 16/256/4096 `u64` elements measured approximately
  408 ns/176 us/43.9 ms in the examined compiler baseline versus
  65 ns/1.50 us/29.7 us in the same-layout native local-build/value-return control.
  Retained whole-payload transfers explain an adverse work shape; at 4096 elements the Whitefoot entry
  and its constructor have simultaneous static frames totaling 229,648 bytes,
  excluding platform helpers. This is a workload result, not a general language
  ranking. [Sources, 168 raw samples, controls, and limits](../../../research/experiments/container-representation/dense/RESULTS.md).
- 2026-09-06 external evidence: six complete source traces cover ripgrep 14.1.1
  (Rust/text search, `4649aa9700619f94cf9c66876e9549d83420e16c`), DuckDB v1.2.0
  (C++/analytical execution, `5f5512b827df6397afd31daedb4bbdee76520019`), and
  Kubernetes v1.32.0 (Go/scheduler state,
  `70d3cc986aa8221cd1dfb1121852688902d3bf53`). The sample recovers retained windows,
  nullable nested payloads and selections, stable/inline strings, sparse aggregate
  reservations, indexed heap correspondence, and out-of-order reader retirement.
  Multiple traces from one application are correlated evidence. No upstream
  timings, allocation profiles, or prevalence measurements were performed.
  [Pinned sources, traces, alternative hypotheses, and unknowns](../../../research/investigations/containers-and-resources/EXTERNAL-WORKLOADS.md).
- 2026-09-06 external interpretation: the sample does not falsify owned places or
  value semantics. It requires distinguishing location from initialization and
  access authority, and it gives concrete relational workloads for reconsidering
  the initial state families and eventual checked-library authority. Whether a
  separate NULL bitmap, pointer/salt packing, stable list, or particular growth
  factor is necessary for a frozen cost contract remains unmeasured. Safe enum,
  ordinary collection, or indexed-log alternatives are hypotheses until checked
  against the same behavior, failure, lifetime, and resource obligations.
- 2026-09-06 control: native whole-value append also retains copies, whereas a
  separate noinline aggregate-return control writes directly to a caller's `sret`
  destination. Value semantics and physical aggregate copies are distinct; the
  result does not show that LLVM eliminates all move chains or that every ABI has
  identical cost. The native controls are not checked Whitefoot implementations.
- 2026-09-06 source evidence: five complete lifecycle programs run, including pool
  count conservation and static-capacity block checkout/mutation/return. Capacity
  survives through the field type; the neighboring full-initialization assertion
  does not. Boundary-element incidental measures, projected result contracts, and
  empty linear-owner discharge are separate source limitations. Two boxed-block
  cases expose incorrect compiler rejections for nested region inference and owned
  cell measures; they do not refute stable fixed storage. [Exact 15 outcomes and
  source](../../../research/experiments/container-representation/lifecycle/RESULTS.md).
- 2026-09-06 finite-model evidence: checked interval tokens account for all 510
  live sets at capacities 1..8, versus 184 single-window sets, and pass independent
  occupancy/conservation checks through cleanup. Two-span loans and middle-slot
  reuse are checked. The model has concrete endpoints, anonymous linear counts,
  and one storage context; it establishes neither public symbolic proof authority
  nor runtime token overhead. [Model and limits](../../../research/experiments/container-representation/authority/RESULTS.md).
- 2026-09-06 correction: removing the middle element of a capacity-three full run
  is not a sparse-window falsifier: remaining slots 0 and 2 form a circular window.
  Capacity four with initial live slots 0,1,2 and raw slot 3 supplies the actual
  `0101` separating state. An executable near-neighbor check corrected the paper
  claim. Cross-review also strengthened the model's independent oracle to retain
  the obligation count from trace entry instead of recomputing it before cleanup.
- 2026-09-06 implementation checkpoint: commit `f5dab70c` passes the frozen
  scalar correctness matrix and mandatory wide-record/inline-view execution
  probes. This closes their specific baseline capability stops, not the whole
  implementation slice. A new 168-sample run of the same scalar kernels measures
  four-pass medians of 108 ns/1.52 us/25.4 us at 16/256/4096 elements. Element loops
  no longer move whole payloads; one-time result-to-binding copies and excess
  physical storage remain. The N=4096 static entry frame is 125,872 bytes, with
  its constructor now inlined. Keep the baseline and new run distinct, and do not
  infer optimal storage or a language ranking from them. Parallel integration and
  the full repository gate remain incomplete.
  [Both measurements and limits](../../../research/experiments/container-representation/dense/RESULTS.md),
  [current implementation boundary](../../../research/investigations/containers-and-resources/REASSESSMENT.md#first-implementation-scope-and-completion-evidence).
- 2026-09-07 (bb8eb30f) pitfall: the checked binding occurrence records whether
  it consumes its owner, but ordinary binding-expression lowering discards that
  field. Formal own and borrow modes can also lower to the same descriptor or
  opaque-handle type, and user-call lowering does not retain checked result
  provenance. Type shape alone therefore cannot supply the source permissions
  needed by an ownership-directed placement pass. The current conservative
  content-liveness planner does not infer those permissions.
  [Checked occurrences](../../../compiler/src/semantic/model.rs),
  [parameter and expression lowering](../../../compiler/src/lowering/builder.rs),
  [content storage planning](../../../compiler/src/backend/storage.rs). (code)
- 2026-09-07 (f5dab70c) mechanism: the dense helper result is first stored as an
  immutable aggregate, then copied to the addressable owner; cleanup obtains
  another aggregate snapshot. The retained three-field frame distinguishes
  these physical objects, not three live source owners. Direct fresh-result
  placement and cleanup of the actual owner are separate opportunities from
  deleting per-element reconstruction. A cleanup subject still needs its old
  contents until its checked release runs, including through phi transfers.
  [Measured frame and remaining copy](../../../research/experiments/container-representation/dense/RESULTS.md),
  [binding storage](../../../compiler/src/lowering/builder/storage.rs),
  [edge delivery](../../../compiler/src/backend/emitter/places.rs). (code)
- 2026-09-07 (bb8eb30f) trace: the allocation-refusal witness owns one complete
  cell after the first acquisition. Refusal of the second returns its error and
  releases the first cell; no pair owner exists on that edge. Success transfers
  the two complete cells into the pair. The checker supplies ordered releases
  for each edge and lowering preserves them; the witness does not require a
  source-visible partially initialized pair or a runtime drop-needed flag.
  [Execution witness](../../../compiler/src/backend/tests/owned_places.rs),
  [checked release records](../../../compiler/src/semantic/model.rs),
  [release lowering](../../../compiler/src/lowering/builder.rs). (code)
- 2026-09-07 (bb8eb30f) pitfall: consuming a call argument does not end the
  callee's reads of it. Aggregate swap retains both old field values before
  writing, and replacement can mutate the existing target during its RHS before
  reading the displaced owner. A move annotation alone cannot justify aliasing
  a result destination with an input or treating the replacement target as fresh
  storage. [Swap, retained helper, and replacement witnesses](../../../compiler/src/backend/tests/owned_places.rs). (code)
- 2026-09-07 rationale: the foundation review selects preserving checked use
  distinctions and extending the existing typed IR before one storage/call
  normalization. Reusing its scalar SSA, CFG, projections, and explicit cleanup
  avoids turning emitter adapters into the place where source authority is
  reconstructed. This is the next implementation direction, not a statement
  that the normalized representation or parallel integration already exists.
  [Comparison, operation distinctions, and dynamic storage instances](../../../research/investigations/containers-and-resources/REASSESSMENT.md#foundation-review-authority-representation-and-placement). (sourced)
- 2026-09-07 correction: the signature portion of the preceding erasure pitfall
  is now addressed. Own, shared, and unique buffer parameters retain the same
  descriptor representation while keeping distinct source roles and release
  responsibilities. Shared and unique scalar-borrow results likewise keep their
  roles despite an identical address representation. Generated reduction
  functions carry no source signature. Binding-occurrence consume information
  and borrow-result provenance still need their own retained representation.
  [Signature lowering and source boundary](../../../compiler/src/lowering/builder.rs),
  [IR distinctions and focused tests](../../../compiler/src/lowering/tests.rs). (code)

- 2026-09-07 correction: user-call lowering now retains argument-occurrence
  consumption and the direct borrow result's candidate argument ordinal. The
  controls distinguish borrowing and consuming the same descriptor value,
  consuming a unique holder without owning its referent, projected-root
  consumption, and an indexed borrow passed as the second argument. Referencing
  the actual lowered argument avoids reevaluating its index or remapping a
  separate source-root path. This addresses call-boundary information loss,
  not standalone uses, owned-view origin sets, or activation lifetimes.
  [Retained checked candidate](../../../compiler/src/semantic/check/expressions/calls/user.rs),
  [Call lowering and distinctions](../../../compiler/src/lowering/builder.rs),
  [Executable IR controls](../../../compiler/src/lowering/tests.rs). (code)

- 2026-09-07 mechanism: a frame containing an address does not keep repeated
  executions of one static address definition independent. The earlier lane
  driver conservatively declined issue-local addressed owners and remainder
  reads. The current typed frame plan gives issue-stage places per-slot backing,
  while the drain carries their addresses and reads completed mutations after
  join. A native control defers every granted publication until its join, requires
  multiple frames held at once, and checks both per-iteration inputs and updated
  inline content. This is evidence for activation storage and actual-use
  boundaries, not a reason to change source ownership permissions.
  [Lane carry lowering](../../../compiler/src/lowering/builder/loops.rs),
  [Typed frame plan](../../../compiler/src/backend/emitter.rs),
  [Execution and cleanup controls](../../../compiler/src/backend/tests/parallel.rs). (code)

- 2026-09-07 mechanism: the dense cleanup snapshot at `f5dab70c` exists even
  though its scalar run has no element release. Naming the existing owner
  place removes that cleanup-only value while preserving saved-value subjects
  where prior content matters. Capturing a complete release group before its
  first effect and writing phi destinations only after that group preserves
  the distinction between release responsibility and content lifetime.
  [Cleanup subjects and groups](../../../compiler/src/lowering.rs),
  [Capture and release](../../../compiler/src/backend/emitter.rs),
  [Value, failure, and ordering controls](../../../compiler/src/backend/tests/owned_places.rs). (code)

- 2026-09-07 (c4964ce2) pitfall: one static use does not exclude a pointer
  retained from a previous dynamic iteration; the single-use condition alone
  does not establish a fresh dynamic destination.
  [Placement and lifetime guard](../../../compiler/src/backend/storage.rs). (code)

- 2026-09-07 (c4964ce2) measurement: fresh destination placement removes the
  scalar dense result-to-owner transfer and leaves one aggregate frame field.
  At 4096 elements the static entry frame is 32,848 bytes, versus 93,040 at
  the preceding checkpoint and 32,832 in the current native control. The same
  arm64 macOS experiment still retains one-time payload zeroing and wrapped
  element addressing. Removing an intermediate owner destination does not
  establish optimal loop code or a universal storage-reuse analysis.
  [Raw samples, generated-code analysis, and limits](../../../research/experiments/container-representation/dense/RESULTS.md#fresh-destination-checkpoint). (code)

- 2026-09-07 correction: the preceding bb8eb30f replacement witness was accepted
  by a checker missing post-RHS temporary loans. Its RHS borrowed a field of the
  selected old owner, so OWN-5 and OWN-6 forbid the subsequent replacement while
  that loan lives. The legal ordering witness changes a separate index after
  capturing the target; the same-target form is rejection evidence. Preserve
  evaluation order and loan admission together when selecting destinations.
  [Mutation and temporary-loan controls](../../../compiler/src/semantic/tests/owned_places.rs),
  [Captured-index execution](../../../compiler/src/backend/tests/owned_places.rs). (code)

- 2026-09-07 rationale: sequential typed acquisition through a retained unique
  provider uses the selected completed-control-header endpoint. A future storage
  planner may reuse the provider in the selected arm only after all temporary
  children created by that header end; it must preserve longer bound, view, result,
  and borrowed-match loans and the ordinary statement endpoint elsewhere.
  [Selected boundary and alternatives](../../../research/investigations/containers-and-resources/REASSESSMENT.md#selected-control-header-temporary-loan-boundary),
  [ownership decision](../ownership/no-reborrow/control-header-temporary-loans.md). (sourced)

## Moves

- Selected general place/result-destination support as the first implementation,
  including the semantic field/index/cell/borrow support needed to turn the frozen
  wide-record and inline-view probes into executed positives. Require element-sized
  loop work, correct failure/cleanup, and the same-workload measurements; keeping a
  known unsupported verdict does not complete that slice. Broad migration and
  retirement wait for replacement capability.
- Superseded the earlier integrated container dossier's universal-window and
  aggregate-reconstruction defaults. Retained providers, checked ownership, and
  working contract transport. Did not replace the separate I/O/resource research
  or amend the active specification in this decision.
- Deferred generalized representation privileges, arbitrary sparse stores, global
  stable handles, and quantified element refinements as prerequisites. Reconsider
  when a real frozen workload cannot meet its behavior/resource/cost contract with
  the selected states, or a complete checked alternative demonstrates a better
  form. Minimal operation count, a successful workaround that changes the contract,
  and majority agreement among reviewers are not selection evidence.
- Made the reconsideration questions concrete with the external traces: compare
  ordinary nullable elements against separate validity/payload storage, require
  helper-preserved index/reservation relations, and compare stable nodes with a
  segmented history while preserving oldest-reader reclamation. These select
  later bounded experiments; they do not require six application ports or change
  the first slice into a universal storage proof framework. Reopen the owning
  layer if the checked ordinary form breaks the frozen contract, rather than
  hiding the gap with a guard, hard cap, extra scan, or extra allocation.
