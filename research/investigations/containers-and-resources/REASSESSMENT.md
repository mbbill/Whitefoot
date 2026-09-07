# Container architecture reassessment

This is the container architecture selection from the independent workload,
critical-case, and semantic reviews of 2026-09-06, their executable experiments,
and adversarial cross-review, supplemented by the pinned external workload traces.
It selects the architectural direction and first implementation scope below; it
does not amend the active specification. The foundation review of 2026-09-07
reopens the precise IR and phase boundary before further call-adapter work. The
current implementation is a candidate, not the definition of that foundation.
Its checkpoint is distinct from the retained pre-implementation measurements and gate.
Keep this decision and its linked evidence current as implementation replaces the
old container paths. `DESIGN.md` points here for the superseding container choice;
its separate resource research is outside this selection.

The implementation examined was `ea97222adc0aff481df320f4c39624eb6813e488`, rebased as
`eff095c701b473a0108822a16cd1e1274c43f621` onto main
`8a5ad14b1d9093117dff1bd437d4de9d7aa564c4`. The rebase adds exactly main's intervening
tree changes: container implementation, active and archived specification bytes,
scheduler, and completion code are unchanged from the examined container revision.

## Decision

**Start with compiler-checked container states over general typed owned storage
places. Preserve value ownership semantics; stop representing every owned update
as reconstruction of a whole aggregate. Distinguish full fixed arrays, initialized
prefixes, and circular windows. Use explicit storage identity, initialization, and
range/loan relationships internally. Do not open general source-library
representation privileges in this implementation.**

The comparison concerns architecture, not the final spelling of every future
operation. Existing source spellings and semantics remain in force until their
corresponding specification amendments. The initial programs need no new public
container syntax. Their implementation now supplies evidence for reviewing the
foundation's concrete form, rather than committing to every mechanism it introduced.

### Empirical ground

The maintained [experiment bundle](../../experiments/container-representation/README.md)
contains all sources, native controls, a finite checker model, exact current
compiler outcomes, and the retained 168 timing samples.

These are bounded capability and cost witnesses, not a sample of production
workload prevalence. Whitefoot does not yet have a substantial real-application
corpus. The first storage implementation is justified by the witnessed ownership
and machine-work requirements; this evidence does not rank broader container
demand or establish that the selected families cover production use.

| Evidence | Observation | Selection consequence |
| --- | --- | --- |
| [Dense storage baseline](../../experiments/container-representation/dense/RESULTS.md) | With four updates, 16/256/4096 `u64` elements cost about 408 ns/176 us/43.9 ms at the examined compiler baseline, versus 65 ns/1.50 us/29.7 us in the native local-build/value-return control. Whole-payload transfers remain in that baseline's element loops. | Final storage placement and in-place element work are an immediate architecture requirement. The gap is for this workload and host, not a language-wide speed claim or a measurement of the new implementation. |
| Same-layout native controls | Value return and explicit destination are close; forcing whole-value transfer at every append is also expensive in C. A separate retained-call control uses a caller-provided `sret` destination without an aggregate copy. | Keep value semantics, but make storage reuse/result destinations an explicit lowering responsibility. Do not hope an optimizer repairs every aggregate chain. |
| [Lifecycle source](../../experiments/container-representation/lifecycle/RESULTS.md) | Five complete programs execute. Pool count contracts, direct result contracts, and several local field/variant paths work. Ten probes identify precise limitations, invalid programs, or two compiler defects. | Preserve working contracts and fix the exact missing relationships. A broad claim that all container facts are lost is false. |
| Static-block alternative | A normal struct containing `FixedVector<u8,4>` retains capacity through checkout, use, and return. The neighboring initialized-length claim still fails. | Put persistent fixed extent and full initialization in the relevant type/state. This pool does not yet justify general quantified element refinements. |
| Linear cleanup | Individual linear values are discharged on success and failure, but an emptied run remains unconsumable. | Add checked empty-owner discharge without weakening element or outer nominal obligations. |
| [Finite range model](../../experiments/container-representation/authority/RESULTS.md) | All 510 live sets at capacities 1..8 can be accounted for by checked interval tokens; 184 fit one circular window. Two-span loans and four-slot sparse reuse pass independent occupancy/conservation checks. | One window is not a universal live-set representation. Finite ranges are a useful internal basis; this is not evidence that a public symbolic representation-proof system is ready. |

The dense comparison keeps the same data and descriptor layout in the controls.
It does **not** attribute the measured gap to ring metadata. The reason to
distinguish full arrays from prefix/ring states is their different initialization
guarantees and representation needs, also exposed by the static-block negative.

The [external sample](EXTERNAL-WORKLOADS.md) adds six source-level traces from
ripgrep 14.1.1, DuckDB v1.2.0, and Kubernetes v1.32.0 at pinned commits. It recovers
application contracts and candidate needs across Rust, C++, and Go; it includes no
upstream timing, allocation profile, or prevalence measurement. Its consequences
and remaining causal uncertainties are recorded under workload evidence below.

### Storage and lowering boundary

- An owned value has a typed storage place, an initialization/ownership state, and
  a lifetime. Fields, indices, and cell dereferences are projections of that place.
  Semantic resolution and lowering must walk type structure generally, rather
  than recognizing a run of one particular element shape or rebuilding each
  enclosing aggregate after an element update.
- Callee results may be constructed in a caller-provided destination. A consumed
  owner's place may be reused when the checked lifetime and alias relationships
  permit it. This is a storage decision, not a second source mutation API. Do not
  mark two potentially identical input/output pointers as independent `noalias`
  destinations or duplicate their ownership to obtain this representation.
- A known place is not proof of target uniqueness or initialized payload. In the
  external grouping trace, different input rows can map to one aggregate state;
  a slot reserved during construction is not yet a usable row reference. Keep
  those relations separate from address formation and result placement.
- Track which fields/elements own initialized values throughout construction and
  replacement, including failure and variant returns. A location alone is not
  enough information to generate correct cleanup. Scalar/Copy values may remain
  ordinary SSA values; owned aggregate representation must support real places.
- Preserve evaluation, allocation, and release order. In the current flat source
  form, `let value = construct(...);` followed by
  `heap_box(store: &uniq heap, value: move value)` may not become allocate-first
  merely to avoid a copy: refusal could skip observable construction effects.
  A program needing
  allocation-first construction must acquire backing first in its source, then
  construct and seal it. Typed failure returns the actual initialized prefix,
  unconsumed inputs, and remaining resource responsibilities.
- Moving a descriptor does not move or prolong its backing. Moving inline payload
  can change its address. Form loans at a location whose lifetime covers their
  use, and forbid relocating/reusing that payload while the loan remains live.
  Staged storage additionally lasts through join return, result consumption, and
  its retirement. This requirement belongs in storage identity/lifetime handling
  now, even though staged-window optimization is not the first slice.

### Foundation review: authority, representation, and placement

When construction, replacement, borrowing, helper transfer, cleanup, or deferred
execution exposes a problem, first identify the missing state or transition and
its owning layer. Repeated adapters are evidence to examine that boundary; they
are not proof that the language needs another operation. Conversely, an isolated
incorrect use of a sufficient representation is an implementation bug, not a
reason to redesign the whole compiler.

| Layer | Responsibility | Boundary |
| --- | --- | --- |
| Source specification and semantic checker | Admit operations using ownership, initialized domains, loan origins, effects, and verified callable contracts. | A backend destination grants no new source authority. Missing source permissions require specified rules and their checker, not an emitter exception. |
| Typed lowering and normalized storage/call representation | Preserve value identity separately from storage identity, typed projections, construction/commit order, ownership transfer, cleanup, and address-use lifetimes. | Lowering may choose a physical construction destination for an already admitted value. It must preserve every observable source behavior and responsibility. |
| Target storage, ABI, and optimization | Select layouts and frame objects, implement calls, and reuse storage when contents and derived addresses no longer need it. | This is compiler correctness analysis, not another source acceptance path. STOR-6 checks target materializations before optional optimization; optimizer facts cannot supply missing source proof. |

The current [storage planner](../../../compiler/src/backend/storage.rs) analyzes
immutable IR contents and their CFG interference. It does not infer source
ownership: a failed coalescing opportunity keeps separate backing. Loads remain
snapshots, exposed backing is protected, and coalescing is disabled for functions
with overlap or completion pipelines. These are legitimate conservative compiler
choices. That conservatism does not yet model the complete lifetime of an address
carried across asynchronous execution.

The more consequential gap is the phase boundary. An
[`IrOperation::Call`](../../../compiler/src/lowering.rs) names value arguments;
the aggregate result-pointer ABI is introduced later by the
[emitter bridge](../../../compiler/src/backend/emitter/places.rs).
[`promote_binding_if_needed`](../../../compiler/src/lowering/builder/storage.rs)
can then introduce separate addressable owner storage. The dense measurement
retains a whole result-to-binding copy, and ordinary, system, and parallel emission
routes have each needed to honor that late bridge. The system-operand correction
at `bb8eb30f` and remaining parallel failure are concrete evidence that the shared
representation deserves review; they do not establish that every ABI conversion
is avoidable or unsound.

The preferred candidate for that review is **one typed storage/call normalization
before emission**, with explicit aggregate destinations and a shared call
representation consumed by ordinary, system, and parallel paths. Runtime-specific
marshalling still exists, but must not independently reconstruct whether an
operand is a value, a borrowed place, or an aggregate destination. This candidate
is not implemented or validated. Keeping the late bridge is cheaper locally but
must demonstrate equally complete propagation and placement; adding public output
parameters does not by itself solve either obligation.

The representation must distinguish these transitions:

- **Fresh construction:** an ordinary `let` receiving a completed value may use
  its final backing as the physical result destination. Source value semantics
  do not require a temporary copy. Direct placement still needs valid extent,
  alignment, aliasing, lifetime, and cleanup on every relevant exit.
- **Commit to an existing place:** SET-1 captures the target before RHS evaluation;
  SET-2 reads the displaced owner after that evaluation, then exchanges owners.
  The RHS may observe or mutate the old value. The target is not a fresh result
  slot, and the displaced owner may remain independently live after commit.
- **Partial construction and failure:** record exactly which values have acquired
  ownership on each edge and how they are returned or discharged. A normal error
  result is part of the program. Moving an allocation before construction can
  suppress effects or change refusal and returned ownership; destination choice
  alone does not justify that reorder.
- **Borrow and retirement:** a descriptor's last read does not imply its backing
  is dead. Derived addresses remain valid for their complete uses, including
  join return, result consumption, and retirement. Cross-worker execution and
  suspension must not change that requirement. Reuse follows that boundary,
  not a DONE observation or an assumed worker assignment.
- **Control-flow transfer:** snapshot simultaneously transferred values, perform
  the required predecessor cleanup, then write reusable destinations. A storage
  slot is neither a second owning value nor evidence that an old snapshot may
  be overwritten.

These are requirements on the representation, not a mandate for a runtime
per-field bitmap, a new allocation, a universal storage wrapper, or a second
general theorem prover. Statically known construction paths can have statically
selected cleanup. Container window state, genuinely dynamic occupancy, and
source-level linear discharge remain their own semantic responsibilities.

Current source authority is narrower than arbitrary construction into a chosen
raw slot. A constructor builds a complete value; `fixed_vector()` builds a valid
empty run; `place_back` consumes a completed element and proved room. `&uniq T`
authorizes access to an already valid `T`, while `writes(dst)` is a write footprint,
not a guarantee of initialization on every exit. A `MutSlice` covers initialized
length, not raw capacity. Borrowing an empty run's first slot fails OP-4, and BLK-4
also excludes source `&uniq` run parameters through nested fields and generics.
Thus a source-written raw-slot helper cannot currently be obtained by changing a
parameter spelling. Whether a real workload needs that additional authority is
a separate source-design question under D17; fresh value-result placement does
not require it.

Use existing complete witnesses to judge the candidate, with their actual limits:

| Witness | Required distinction and current evidence |
| --- | --- |
| [Dense scalar, wide record, and inline view](../../experiments/container-representation/dense/RESULTS.md) | The retained matrix executes and element loops no longer copy whole payloads. Remaining result-to-binding copying and frame cost show that final placement is unfinished. These are cost/capability probes, not production prevalence data. |
| [Owned-place execution tests](../../../compiler/src/backend/tests/owned_places.rs): `replace_reads_the_displaced_value_after_rhs_mutation` and `replaced_aggregate_snapshots_survive_writes_and_helper_returns` | Passed with normal and retained helper calls. RHS effects occur before old-owner readout, and the old aggregate remains a snapshot after the new target changes. A fresh-destination rewrite must not erase either behavior. |
| Same suite: `partial_construction_refusal_preserves_effects_values_and_release_order` | Passed with retained calls and refusal at each of three allocations, observing exact allocation/release order and returned values. The source constructs two complete cells before making the pair; this does **not** demonstrate source-visible partially initialized struct authority. |
| Same suite: `returned_element_borrows_and_inline_views_reach_the_owners_storage`; [semantic neighbors](../../../compiler/src/semantic/tests/owned_places.rs) | Passed owner-storage execution and bounds/loan checks, including refusal of raw-slot access and moving a borrowed owner. Root value liveness alone cannot replace loan/storage identity. |
| [Linear lifecycle programs](../../experiments/container-representation/lifecycle/RESULTS.md) | Actual linear values are discharged on success and failure; the leak neighbor is rejected. A proved-empty run of linear values still cannot be discharged. That missing language capability is not solved by an aggregate ABI. |
| [Parallel corpus execution](../../../compiler/tests/programs/parallel.rs) using [generic nominals](../../../tests/programs/generic_nominals.wf) | At `bb8eb30f`, the canonical program stage reports 72 passes and this one failure: abnormal exit with four workers. Existing green sampling predominantly returns scalars. Aggregate-result and staged-cleanup test additions remain paused, uncompiled, and unexecuted; no complete parallel aggregate lifetime result is claimed. |

Select the concrete normalization using these witnesses and the frozen external
contracts already recorded below. Its design must account for successful handout,
refused handout, deferred execution, join, result consumption, and cleanup through
one consistent set of value/storage relationships. Avoid enlarging runtime slots
or adding scheduling edges to make an incomplete storage model appear sound.
This review precedes more adapter implementation; it does not expand the first
slice into a public raw-storage framework or six application ports.

### Selected initialization states and public authority

The selected direction distinguishes these semantic state families. The current
specification still has the two window runs of BLK-1 and transitional legacy
arrays; this table is not a claim that three new source families already exist:

| State | Inherent guarantee | Representation requirement |
| --- | --- | --- |
| Full fixed array | Exactly N initialized `T` values; length/extent are properties of the type | Dense payload; no universal variable-length or ring-head state |
| Prefix run | Exactly `[0,len)` is initialized within its capacity | Dense backing plus the needed runtime length/capacity state |
| Circular run | One logical sequence maps to one or two physical initialized intervals | Head/order information only for a workload that needs it |

Backing placement is a separate axis: inline, a provider allocation, or an owning
cell/store. This is not a demand for a runtime tagged union of every representation
or a new wrapper/indirection on every array. A fully initialized fixed block held
at a stable allocation should retain its full-state type when the owning reference
moves through a free list. At the examined baseline, the boxed source probes
were blocked by incorrect region inference and owned-cell measure resolution;
those compiler defects do not refute that representation.

The proposed full-state construction proceeds through a checked partial state.
This is a follow-on source capability, distinct from constructing an already
admitted complete value in its physical result destination. Sealing into a full fixed
array requires full initialization, matching extent, and the correct contiguous
layout; it transfers ownership rather than treating raw slots as values. For an
already acquired provider allocation, a compatible seal may retain that allocation
and its identity. It must account for exact backing extent/alignment and release
authority, not silently substitute a second allocation. Dynamic full extents and
arbitrary sparse source containers are later selection questions, not additional
metadata required by the fixed-array implementation.

These families are not an exhaustive account of application state. DuckDB's
nullable list producer writes payload only for valid cells while its logical
extent includes NULL positions; its dictionary selection is a separate mapping.
An `Option<T>` element representation is a candidate, while a separate bitmap and
payload layout requires its own checked initialization relation. Neither the
need for that layout nor its cost advantage is established by source occurrence.

Internal range/loan relationships must be explicit enough to justify disjoint
views and initialized accesses, using finite specified checks. This does not mean
copying the concrete model's per-slot tokens into the compiler or runtime, or
unrolling every source loop. Native state transitions can establish the required
symbolic range relations from the existing fixed proof families.

General source-library representation authority (the B direction) is **deferred,
not disproved**. The model does not verify symbolic bodies, arbitrary abstract
invariants, multi-store provenance, typed payload identity, or proof erasure.
Opening such privileges needs that complete checked path on a real library and
workload. The current fixed-array/pool experiment can advance without it. D17's
long-term commitment remains intact.

### Callable and resource contracts

- Preserve existing verified pool count and direct measured-result contracts.
  Inherent type/state facts survive storage and helper transfer. Incidental facts
  about a particular former element do not become facts about whichever element
  a later operation returns.
- Extend result relations over finite typed field projections, with the same
  entry snapshots, result publication point, and write invalidation as direct
  results. A wrapper must be able to state a true relationship about its returned
  field. Admit only relations proved for every relevant return; do not replace
  missing premises with a runtime guard or a new trusted assertion.
- Full-state types solve the fixed block's persistent initialization requirement;
  they do not solve arbitrary dynamic capacities, graph membership, or
  cross-collection invariants. Those remain valid future demand sources rather
  than being excluded because this pool has a simpler representation.
- Consume a kernel run proved empty without inventing any element release. The
  existing `dispose` surface can carry this rule in its amendment: a linear-element
  run needs proved zero length, every required provider, and ended loans. It does
  not permit discarding live elements or an independently declared `linear`
  wrapper merely because one field is empty. Scope-relative provider obligations
  continue to apply even when no element is live.
- Two-span views require more than two descriptors. Each result must retain its
  owning allocation, physical interval, access mode, and lifetime through helper
  transfer. One checked split establishes coverage/order and disjointness; ending
  one loan does not end its sibling. Lift the blanket same-region/result-type
  restriction only with these explicit result-origin relationships.
- Do not adopt B6's proposed requirement for a nontrivial relation on every
  representation measure. Check the promised abstraction and operation domains;
  an otherwise correct front removal must not be rejected for hiding its head
  coordinate. Do not weaken a written promise to compensate.

### Rejected defaults and reconsideration conditions

| Alternative | Disposition and ground |
| --- | --- |
| Keep aggregate reconstruction and rely on LLVM | Rejected for dense owner operations by retained copies, stack growth, and the measured scaling gap. |
| One ring representation for every sequence | Rejected as the universal semantic/representation requirement. Full initialization is a different persistent guarantee; sparse states are not all single windows. No claim that removing head alone repairs the measured performance. |
| Make every container a stable handle/store entry | Rejected as a default: the dense workload needs no identity lookup. Preserve a separate stable-allocation route for actual retained-address requirements. |
| Open generalized library representation proofs first | Deferred: no symbolic implementation/contract/erasure evidence yet, and the immediate programs have a smaller path. Revisit when a real library cannot meet its frozen contract with the selected states, or a checked alternative supplies better complete evidence. |
| Add general element refinements to recover fixed capacity | Not selected for this witness: the executable static-block alternative already retains capacity. Revisit for a real dynamic or relational requirement that inherent state and callable contracts cannot express. |
| Replace lost facts with checks, tags, or another backing | Not an equivalent solution when it violates that workload's behavior, resource, or cost contract. Necessary dynamic algorithm state remains legitimate. |

## First implementation scope and completion evidence

Begin with **general owned-place and result-destination support**, including the
semantic projection/borrow support needed by the existing probes. This includes
semantic work: the wide-record and inline-view programs stopped before the
relevant backend path at the examined baseline. Preserve source semantics during
this first slice.

Its concrete acceptance criteria are:

1. The frozen dense construction/update programs execute correctly for the full
   runtime-seed/round matrix. Element loops perform element-sized work, not a
   whole-payload transfer per operation. Construction and updates have linear
   work and bounded backing; repeat the same recorded measurements and attribute
   remaining differences against comparable native code.
2. The existing wide-record program and inline exclusive-view probe **compile and
   execute correctly**. Merely retaining today's known unsupported diagnostic does
   not satisfy this criterion. Field/index/cell paths use the general mechanism.
3. Owned results cross real retained helper calls without mandatory intermediate
   aggregate copies where the lifetime/alias conditions permit destination reuse.
   Partial-construction and failure cleanup preserve every responsibility and the
   source's effect order. Negative ownership, borrow, and raw-read cases remain
   rejected for their rules.
4. Storage lifetime is represented independently of descriptor/binding lifetime,
   so subsequent staged-window integration can preserve the supplied retirement
   boundary without replacing this foundation. No new I/O/runtime protocol is
   introduced, and private runtime slot layouts are not assumed.

The in-progress implementation now executes the frozen scalar correctness matrix
and the mandatory wide-record and inline exclusive-view programs. Its seven
[owned-place execution tests](../../../compiler/src/backend/tests/owned_places.rs)
observe value snapshots, simultaneous assignment, returned element borrows,
retained helper calls, allocation refusal, cleanup order, and mutation evaluation
order. In particular, SET-1 captures target components before the RHS, while
SET-2 reads the displaced owner after the RHS. A phi edge snapshots its inputs,
performs predecessor cleanup, then writes the destination: liveness may allow the
destination to reuse storage that cleanup still needs before that point.

The old borrowed-buffer descriptor path exposed an existing unsafe capability:
replacing a descriptor passed by value can free backing without updating its
caller. Replacing an owning borrowed wrapper can likewise retire backing held by
an earlier prepared target. These forms now stop explicitly as
`BorrowedBufferDescriptorMutation`, after ordinary source judgments; no normative
verdict was changed to call them invalid. Borrowed element writes and direct
owned replacements remain supported. Current BLK-4 excludes mutating a new run
through a source helper, including nested run owners. Any later lifting of that
restriction must prove captured target storage identity and lifetime, not merely
that the root binding is live after the call.

The [parallel aggregate ABI patch](parallel-abi-review.patch) is an **inert review
artifact**, not an applied compiler change. Automatic approval review rejected
the remaining cross-worker adapter edit and requested explicit owner permission;
the draft PR carries the exact proposed patch for inspection. Its owner is this
implementation investigation, and it is deleted when that patch is applied or
superseded. It proposes preserving frame layout and publication/join/release order,
copying aggregate results into caller storage before frame release. The owner has
questioned the adaptation strategy; foundation review now precedes further adapter
work. Neither this patch nor another way to perform the rejected edit is assumed
selected or authorized. Parallel integration remains incomplete and needs a
reviewed implementation and its required validation.

The committed implementation at `f5dab70c` now has a separate 168-sample run using
the same scalar kernels and procedure. Four-pass medians at 16/256/4096 elements
are 108.38 ns/1.52 us/25.4 us, versus the baseline's 408 ns/176 us/43.9 ms.
Construction and update loops no longer transfer whole payloads. A one-time
result-to-addressable-binding copy and excess frame storage remain: N=4096 uses
125,872 static entry-frame bytes versus the new native control's 32,832 bytes.
This bounds the result rather than establishing optimal placement or a language
performance ranking. [Both runs, machine shape and limits](../../experiments/container-representation/dense/RESULTS.md)
are retained together. The full regression gate and parallel integration still
prevent treating this checkpoint as completion of the slice.

Then implement the selected bounded semantic capabilities: full-state construction
and sealing, checked empty-run consume, projected result contracts, and two-span
result provenance. These may amend the specification with their derived evidence;
they do not require a general library-proof framework. Broad source migration and
legacy retirement follow only when the replacement capabilities run the relevant
programs, including fixed blocks on the stable-storage route. Do not retire the
full-array guarantee merely because a variable run accepts a constant literal.

The direction of general owned storage remains supported; the exact representation
and phase boundary are open in the foundation review above. That choice can change
the current implementation, including its call adapters. Later sparse,
dynamic-refinement, destruction, and device extensions remain bounded follow-on
questions. Neither the direction nor the measured checkpoint establishes that the
first slice is complete or that proposed source rules have passed a compiler.

## Selection ground

[Project instructions](../../../AGENTS.md), the
[constitution](../../../docs/constitution.md), the
[active specification](../../../spec/kernel-spec.md), and the corrected
[development decisions](../../../mcts_mem/whitefoot/development-workflow.md) supply
the ground. Historical design selections and compiler convenience do not define a
new architecture's requirements. Current language behavior remains the active
specification's, including its proof rules; historical claim/trap language does not
authorize a writer-accessible escape.

- Performance is the reason to exist. Evaluate ordinary accepted source against an
  equivalent reference implementation, including its layout, algorithm, resource
  contract, and generated machine code. A smaller operation table is not evidence
  of a better design. Name the advantage over Rust in performance, checked safety
  without writer-accessible unsafe, or enforced fast source forms.
- Safety and proof authority cannot be traded away. Memory validity, exclusivity,
  initialization, ownership, and partial-operation domains must be established by
  the specified deterministic checker. No proof-search timeout, work budget, or
  runtime fallback selects acceptance.
- The default source form must reliably expose the appropriate fast representation.
  An expert-only rewrite does not establish that ordinary accepted forms are good.
  Necessary dynamic program state is allowed; redundant safety scaffolding is not
  justified merely because it makes a source program compile.
- An `existence-only` decision separates a derived need from a minimality-selected
  form. Read its rationale and reconsideration condition. New workloads can
  disprove the adequacy of the form without disproving the need.
- D17 commits to checked representation authority as a long-term lane. It does not
  require implementing a universal storage proof framework before the next useful
  program. Such machinery earns priority by resolving an immediate experiment.

Frequency determines optimization and investigation priority, not whether a needed
capability may exist. A rare workload can veto a representation for its own hard
contract; it does not thereby get to impose its metadata on every dense array.
Do not average away a hard-contract failure with wins on common cases. Conversely,
do not turn contiguous layout, stable addresses, no allocation, or no runtime tags
into universal requirements without a workload-specific derivation.

## Independent obligations

| Obligation | What must stay distinct |
| --- | --- |
| Store provenance | A store brand identifies the provider; it does not identify an allocation or a recycled object generation. |
| Object identity | Identity, address stability, and current ownership are different properties. |
| Ownership | Every live affine or linear element has exactly one owning responsibility, including failure and early-return paths. |
| Loans | Authorized ranges remain valid through their promised use; copying a descriptor does not extend backing lifetime. |
| Logical shape | Length, order, continuity, keys, and occupancy need not describe the same physical representation. |
| Initialization | Only constructed values may be read or destroyed; temporary holes need checked authority. |
| Abstraction | Useful relations must survive helpers, fields, variants, and elements through verified contracts, with sound invalidation. |
| Resources | Allocation failure, peak storage, backing reuse, and destruction are part of the complete operation. |

Runtime occupancy, a ring's head, and a genuine failure outcome can carry necessary
information. Rechecking a property already guaranteed by a returned capability or
verified contract is a different issue. Proof erasure must not discard semantic
facts needed to select efficient lowering.

## Workload evidence and coverage gaps

These are candidate workload classes, not measured prevalence estimates. Existing
Whitefoot tests and small programs are capability probes; their distribution
reflects what the compiler supports and what its authors chose to test. They cannot
establish production demand, including the relative importance of byte storage,
dense construction, or other classes below. The compiler's own Rust implementation
is one additional demand source, not a representative corpus or a reason to start
full self-hosting.

When prevalence or application-scale composition is needed to select priorities,
use a bounded, explicitly sampled corpus of established C++, Rust, and Go
applications across relevant domains. Trace actual container use through helper
boundaries and complete lifetimes, retaining the source revision and workload
context. Separate static use-site counts from execution frequency, element-size
and collection-size distributions, allocation behavior, and live-storage costs;
the latter require representative runtime measurements. Language/library defaults
and corpus selection can bias the observed forms, so infer requirements from the
program's behavior and resource contract rather than copying its container API.
Report conclusions within the sampled corpus instead of claiming industry-wide
percentages. The external source sample below is now available; no external
distribution study or upstream runtime measurement has been performed here.

For each consequential representation, ask the counterfactual question: which
constraints remain if the source language's ownership, borrowing, proof, layout,
or library limitations are removed? Recover the required behavior, ordering,
identity/address lifetime, failure behavior, and resource/cost contract before
classifying the observed container. Distinguish evidence for an algorithmic
necessity or measured performance choice from language/library workarounds,
compatibility constraints, and historical accidents. Causes can be mixed or
unknown; occurrence counts alone do not establish them. Comments, design records,
profiles, and relevant alternatives help test the interpretation.

For example, an observed handle table does not by itself establish a requirement
for handle lookup: the required property might be stable identity, stable addresses,
or simply a way to express otherwise awkward references. Conversely, replacing
that table with dense ownership is invalid if the application really needs its
identity and lifetime behavior. A Whitefoot alternative must preserve the recovered
contract and demonstrate its claimed representation or proof advantage. Label it
as a hypothesis until tested; cross-language agreement can still reflect shared
constraints and is not independent proof of necessity.

Use those external traces to select and ground subsequent Whitefoot experiments.
Keep the existing probes for the narrower semantic and cost questions they answer;
their successful execution does not substitute for external demand evidence.

### External sample and the decisions it can change

[EXTERNAL-WORKLOADS.md](EXTERNAL-WORKLOADS.md) carries the six complete operation
traces, precise revisions, source links, and failure/lifetime boundaries. The
sample was selected for discriminating contracts, not weighted by prevalence.
Its three domains are text search, analytical query execution, and scheduler
state. Multiple traces from one application are related evidence, not additional
independent samples.

| External trace | Recovered need | Unknown cause or cost | Next implementation decision |
| --- | --- | --- | --- |
| EW1, ripgrep search window | Retain context and incomplete lines, compact/reuse backing, preserve configured growth refusal and contiguous matcher input | Whether copying tails or initializing new capacity materially costs time; whether a ring wins after boundary handling | Compare prefix compaction with two-span/coalescing only under the same matcher and memory contract; do not select a ring from byte-buffer occurrence alone |
| EW2, DuckDB nullable nested columns and selection | Distinguish logical extent, valid payload, nested intervals, and selected row mappings | Separate bitmap versus per-cell enum layout, copying, and vectorization costs; reused NULL bytes are not an initialization contract | Ground a later nullable-storage/relational-contract probe; first test a checked ordinary element form before requiring public library layout authority |
| EW3, DuckDB string construction | Reserve, populate, then publish; inline short values coexist with stable long-string backing and retained references | Optimal inline threshold/chunk policy; cost of replacing pointers with handles | Keep destinations and stable backing separate; compare optional store forms while preserving construction and refusal order |
| EW4, DuckDB grouped aggregation | Distinguish empty, reserved, and usable slots; preserve row references during pointer-table rehash; admit many-to-one group mappings | Need for pointer/salt packing versus a typed state sum; collision-dependent costs and complete failure recovery | Require reservation and target-alias contracts in a bounded map/scatter experiment; no automatic disjointness from distinct input indices |
| EW5, Kubernetes indexed heap | Preserve map-domain/key-array correspondence and positions while heap order changes | Best map representation and the role of Go pointer/runtime conventions | Exercise compositional contracts through swap/update/remove helpers; neither stable array addresses nor a universal handle layer follows from key identity |
| EW6, Kubernetes event history | Retire interior reader markers out of order and reclaim only history no remaining reader needs | List versus segmented-log costs; maximum retained history under slow readers | Compare stable-node and indexed-log ownership with the same ordering, repeated-finish, and reclamation contract; this is container evidence, not an I/O/runtime protocol change |

None of these traces falsifies owned places as the current lowering foundation.
They do prevent its address knowledge, the initial state-family list, or the
finite range model from being treated as a complete library proof system. They
also make the reconsideration condition concrete: if an ordinary checked form
cannot preserve one of these frozen contracts, fix the owning semantic layer or
reopen representation authority rather than adding a guard, hard cap, extra scan,
or allocation that changes the contract. A more compact upstream layout alone is
insufficient evidence that the alternative is required. These follow-up questions
do not expand the current slice into six application ports.

### Local capability witnesses and remaining gaps

| Class | Complete trace to preserve | Local demand source |
| --- | --- | --- |
| Tiny temporary sequences | Empty, a few inserts, occasional spill, helper transfer, clear and reuse | `compiler/src/lowering/builder.rs`: block successors and builder collections |
| Fully initialized fixed objects | Construct, store in a record, return through a helper, repeatedly update | `tests/programs/raw_deflate_dynamic.wf`, `fir_filter.wf` |
| Growing nested owned values | Grow or fail, preserve existing elements and unconsumed inputs, clean up | `tests/programs/growable_vec.wf`; byte-only examples underexercise affine payloads |
| Bulk transformations | In-place shrinking, overlapping copy, and LZ back-reference expansion | `tests/programs/percent_decode.wf`, `utf8parse.wf`, `raw_deflate_dynamic.wf` |
| Retained tails and rings | Append, consume a prefix, wrap, inspect two spans, reuse backing | `tests/programs/wfgrep.wf`, `run_queue.wf` |
| Variable records and names | Many short or empty values, rare long values, nesting, retained references | `tests/programs/byte_string.wf`, `wfgrep.wf` |
| Graphs and coordinated tables | Adjacency, membership, discovery state, worklists, deterministic readiness | `compiler/src/semantic/entailment.rs`: SCC and postcondition scheduling |
| Sparse keyed collections | Insert, find, delete, reuse, maintain index relationships | `compiler/src/lowering/builder.rs`; `tests/programs/option_slots.wf` |
| Partial construction | Fail after several owned values, return every responsibility exactly once | `tests/programs/prefix_expression.wf` and ownership rules; broader failure trace needed |
| Fixed pools | Exhaust, lease, return in a different order, reuse without extra backing | `tests/programs/block_pool.wf` |
| Borrowed stable storage | Publish, suspend, resume, join, consume result, retire, then reuse | Runtime interface supplied by the I/O/runtime agent; container side only |
| Destruction | Early exit with a deep owned structure, bounded or explicit cleanup storage | `PROV-6` release graph; a measured deep-chain case is still needed |

Bulk operations need their own semantics: disjoint copying, overlap-preserving
movement, and LZ expansion reading newly written output are not interchangeable.
Likewise, an owned tree does not cover shared graphs or sparse stable identities.

DMA mapping/coherence, general cancellation, arbitrary strided splitting, and
concurrent reclamation remain boundary questions. Their possible future value is
not a reason to add pinning, atomic metadata, or a registry to every container now.

## Baseline defects and current implementation boundary

The following distinctions prevent an implementation gap from being mistaken for
a language design result:

1. **Baseline implementation limitation, now a required positive:** forming an
   exclusive view of an inline run reported
   `SemanticUnsupported::ExclusiveViewOverInlineRun`. It was not a source-language
   rejection. The in-progress implementation now compiles and executes the
   mandatory inline-view probe; that closes this particular capability stop, not
   the complete storage/lifetime work. The minimal source form is:

   ```whitefoot
   command fn main() -> status: own ExitStatus pure {
     let empty = fixed_vector::<u8, 4>();
     let built = place_back(vector: move empty, value: 7_u8);
     region {
       let view = mut_slice_of(&uniq built);
       set view[0_u64] = 9_u8;
     }
     let byte = built[0_u64];
     return exit_status(code: byte);
   }
   ```

2. **Confirmed baseline lowering concern:** inline-run emission represented
   runs as aggregates and staged whole values through storage slots around
   element operations. A 4096-byte fill probe retained whole-aggregate work inside
   its loop after local Apple Clang 21 `-O2` lowering. This establishes a problematic
   machine shape for that probe, not a portable timing result. By-value source
   ownership does not itself require aggregate copying; destination construction
   and in-place updates must be evaluated separately from the source API.

3. **Specified composition gap:** `MSR-3` does not carry incidental inner element
   measures across the run boundary. `CALL-4`/`FN-9` also limit projected result
   contracts. The source experiments distinguish these from working count
   contracts, local field/variant transport, and inherent fixed-capacity facts.
   Improving aggregate lowering alone cannot supply a missing callable contract.

4. **Specified view restriction:** `VIEW-2` supplies a contiguous view of a
   nonwrapping window, while `VIEW-6` restricts repeated result view types/regions.
   A wrapped ring's two-span consumption needs an actual checked expression; a
   second full backing allocation is not an equivalent solution for a one-backing
   workload contract.

5. **Unimplemented proposal to reconsider:** `DESIGN.md`'s CALL-7/B6 proposal asks
   for nontrivial relations for every measure. Its own examples exclude useful
   front operations and measured generic results. Contract adequacy should follow
   the abstraction and the operation's required domain, not a demand to expose
   every representation coordinate. This is a design issue, not a claimed defect
   in an implemented B6 checker.

6. **Evidence boundary:** the lifecycle suite now confirms the empty linear-owner
   gap, and the finite model covers concrete sparse reuse. Full-capacity generic
   permutation, general typed sparse storage, and measured deep destruction still
   require their own executable traces. Do not generalize the bounded results.

Do not generalize one bad fill loop into a ban on all initialization. The existing
`research/experiments/buffer-initialization-cost/RESULTS.md` measures a case where
one-time initialization is negligible for long-lived reusable storage. Repetition,
element width, lifetime, and the surrounding operation matter.

## Architecture alternatives after cross-review

The first review identified three families. Cross-review showed that they are not
three mutually exclusive complete solutions:

| Direction | Question answered | Strongest challenge |
| --- | --- | --- |
| Compiler-owned state machines | Does the compiler directly own full-array, prefix, ring, and any justified additional initialization states? | A new required layout must not need another trusted family merely to avoid extra storage or redundant work. |
| Checked library representation | Can libraries obtain precise backing and initialized-range authority through checked invariants and finite evidence? | Helpers and representation abstraction must compose without replaying long proofs, exposing private layouts everywhere, or requiring unbounded automatic reasoning. |
| Stable objects and owning stores | Where do stable identity and storage live when values are deleted, recycled, or borrowed across suspension? | Identity must not become mandatory indirection or allocation for dense scans; recycled IDs must not silently regain authority. |

The first two primarily compete over **where representation authority lives**.
The third is a **storage/identity choice** that can accompany either. Full arrays,
prefixes, rings, sparse slots, and stable objects should be compared on their own
required state and conversion costs; no universal layout is assumed.

All three also need compositional abstract contracts. Storage validity alone does
not establish `free + outstanding = total`, graph/worklist membership implications,
or atomic ownership recovery after a two-index insertion fails. This is a separate
language capability question, not something a storage descriptor solves.

A useful adversarial trace has a four-slot backing: construct values in slots 0,
1, and 2 while 3 remains raw, keep 0 and 2 borrowed, discharge the owner in slot 1,
then reuse only slot 1. One circular live window does not express its `0101` state.
The initial three-slot proposal was wrong: `101` at capacity 3 is a wrapped window,
as the model's near-neighbor control demonstrates.
Ordinary tagged slots, independent slot capabilities, or chunked ownership may
solve it; each must demonstrate its complete trace and cost. This is a prospective
non-I/O sparse-lifecycle case. It does **not** require the current staged runtime
to retire source iterations out of order.

## Evidence needed to select a form

Compare candidates with the same behavior and declared resource contract. Each
candidate must show natural source, helper contracts, initialization transitions,
failure cleanup, final storage placement, and generated machine shape. Pair each
positive trace with a nearby invalid trace: raw read, duplicate ownership, aliased
exclusive ranges, stale identity, missing release, or premature backing reuse.

Record allocation count, bytes copied/initialized, peak live storage, stack demand,
steady-state machine work, proof size, diagnostic usefulness, and checker cost.
Measure only the dimensions relevant to that trace. Runtime state inherent to the
algorithm is not a proof overhead. Reference timings must use comparable algorithms
and layouts; source occurrence counts are not execution hotness.

The smallest useful selection experiment couples a candidate ordinary path with
one critical path that distinguishes the candidates, crosses a real helper
boundary, and ends the relevant storage lifetime. A choice becomes actionable when
these programs run, nearby invalid programs are rejected for their rules, and the architecture
tradeoff is visible in source and cost evidence. Remaining limits should be named;
an ever-growing scenario list or a universal framework is not the completion test.

## I/O boundary and validation

Keep the runtime's acquire/publish/join/read-result/release protocol opaque. A
published callee may change workers and suspend. Its storage must remain valid
through join return, result consumption, and the corresponding retirement; DONE
alone is insufficient. Form an address-bearing view only once its backing is in
the required stable location. Current slot sizes and experimental worker policies
are not language premises. This review adds no I/O, pinning, lease, or scheduling
interface and makes no change to the other agent's worktree.

Before rebase, the compiler build and the six ordinary `programs::runs` tests passed.
After rebase, the three changed unreadable-path program tests passed. The rebased
tree was checked against both input revisions. The dense, lifecycle, and authority
experiment checks pass and are wired into the root research stage. A sandboxed root
`make check` passed 1,518 library tests but failed seven existing loopback program
tests because binding a local socket was prohibited; 66 other program tests passed.
The unchanged canonical `make check` then passed with loopback permission, including
the compiler and research checks, the full native conformance adapter (730 passed,
3 skipped), and the snapshot corpus (484 passed, no flips). No test was weakened or
removed to obtain this result. Those counts establish the pre-implementation
experiment and selection revision. Implementation is now in progress; the
implementation checkpoint above has its own bounded performance result, but the
earlier green gate does not validate the modified compiler. No main merge is
proposed here.

At the `f5dab70c` implementation checkpoint, the canonical compiler `test-unit`
stage passes 1,488 tests (67 sampling tests belong to its separate stage). The
root `make check` passes specification archive/prose checks and conformance
structure/coverage (25 runner tests, 161/161 rules), then stops at compiler
formatting. A formatting-only correction in `cost_shape.rs` resolves the one
unrelated diff; remaining format differences and the independent Clippy
`write_with_newline` failure are in `backend/emitter/parallel.rs`, whose remaining
edit is awaiting the explicit permission described above. No check was disabled.

The subsequent canonical partition check confirms 1,555 library cases split into
1,488 unit and 67 sampling cases; all 67 sampling cases pass. The snapshot corpus
passes all 484 cases with no flips. The full native conformance run exposed one
diagnostic-priority regression: a repeated affine element read-out must report
TYPE-2 before its simultaneous OWN-1 violation under DIAG-1. Correcting the
compiler preserves the original case and verdict, and the rerun passes 732 cases
with one declared pending case. The 19 focused storage semantic tests include
the legal swap control and the repeated-read-out negative.

Integration exposed two further ordinary-path observations. System wrapper calls
must render a stored aggregate's materialized operand rather than its former SSA
name; the corrected path passes all 11 network program tests. The FIR observer
now checks direct enclosing-field addresses instead of requiring aggregate
reconstruction, and retains its unchanged runtime result checks. The canonical
integration rerun passes its binary and adapter harnesses and 72 program cases;
the remaining program case fails because `generic_nominals.wf` exits abnormally
under `--par` with four workers. That existing fixture returns stored aggregates
from eligible sibling calls. This is a remaining parallel integration defect,
not a host loopback-permission failure or a reason to narrow the corpus.

The complete gate must run after parallel integration. Existing green sampling
counts do not establish aggregate result/carry retirement: their ordinary frame
boundary case uses array arguments with a scalar result, and the staged user-call
fixture also returns a scalar. Additional aggregate execution evidence belongs
with the pending adapter. LoopSplit currently charges each aggregate capture the
entire 256-byte lane payload during admission; a small record capture can be
permission-eligible yet never select split actualization. A sequential fallback
for that source is not evidence of a parallel aggregate capture.
