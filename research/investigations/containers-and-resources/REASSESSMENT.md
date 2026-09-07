# Container architecture reassessment

This is the container architecture selection from the independent workload,
critical-case, and semantic reviews of 2026-09-06, their executable experiments,
and adversarial cross-review. It selects the architecture and first implementation
scope below; it does not amend the active specification or claim implementation.
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
corresponding specification amendments. This decision is sufficient to begin the
first implementation slice, which needs no new public container syntax.

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
| [Dense storage](../../experiments/container-representation/dense/RESULTS.md) | With four updates, 16/256/4096 `u64` elements cost about 408 ns/176 us/43.9 ms in current Whitefoot, versus 65 ns/1.50 us/29.7 us in the native local-build/value-return control. Whole-payload transfers remain in element loops. | Final storage placement and in-place element work are an immediate architecture requirement. The gap is for this workload and host, not a language-wide speed claim. |
| Same-layout native controls | Value return and explicit destination are close; forcing whole-value transfer at every append is also expensive in C. A separate retained-call control uses a caller-provided `sret` destination without an aggregate copy. | Keep value semantics, but make storage reuse/result destinations an explicit lowering responsibility. Do not hope an optimizer repairs every aggregate chain. |
| [Lifecycle source](../../experiments/container-representation/lifecycle/RESULTS.md) | Five complete programs execute. Pool count contracts, direct result contracts, and several local field/variant paths work. Ten probes identify precise limitations, invalid programs, or two compiler defects. | Preserve working contracts and fix the exact missing relationships. A broad claim that all container facts are lost is false. |
| Static-block alternative | A normal struct containing `FixedVector<u8,4>` retains capacity through checkout, use, and return. The neighboring initialized-length claim still fails. | Put persistent fixed extent and full initialization in the relevant type/state. This pool does not yet justify general quantified element refinements. |
| Linear cleanup | Individual linear values are discharged on success and failure, but an emptied run remains unconsumable. | Add checked empty-owner discharge without weakening element or outer nominal obligations. |
| [Finite range model](../../experiments/container-representation/authority/RESULTS.md) | All 510 live sets at capacities 1..8 can be accounted for by checked interval tokens; 184 fit one circular window. Two-span loans and four-slot sparse reuse pass independent occupancy/conservation checks. | One window is not a universal live-set representation. Finite ranges are a useful internal basis; this is not evidence that a public symbolic representation-proof system is ready. |

The dense comparison keeps the same data and descriptor layout in the controls.
It does **not** attribute the measured gap to ring metadata. The reason to
distinguish full arrays from prefix/ring states is their different initialization
guarantees and representation needs, also exposed by the static-block negative.

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
- Track which fields/elements own initialized values throughout construction and
  replacement, including failure and variant returns. A location alone is not
  enough information to generate correct cleanup. Scalar/Copy values may remain
  ordinary SSA values; owned aggregate representation must support real places.
- Preserve evaluation, allocation, and release order. In particular,
  `heap_box(value: construct(...))` may not become allocate-first merely to avoid
  a copy: refusal could skip observable construction effects. A program needing
  allocation-first construction must acquire backing first in its source, then
  construct and seal it. Typed failure returns the actual initialized prefix,
  unconsumed inputs, and remaining resource responsibilities.
- Moving a descriptor does not move or prolong its backing. Moving inline payload
  can change its address. Form loans at a location whose lifetime covers their
  use, and forbid relocating/reusing that payload while the loan remains live.
  Staged storage additionally lasts through join return, result consumption, and
  its retirement. This requirement belongs in storage identity/lifetime handling
  now, even though staged-window optimization is not the first slice.

### Initialization and public authority

The initial public kernel owns these semantic state families:

| State | Inherent guarantee | Representation requirement |
| --- | --- | --- |
| Full fixed array | Exactly N initialized `T` values; length/extent are properties of the type | Dense payload; no universal variable-length or ring-head state |
| Prefix run | Exactly `[0,len)` is initialized within its capacity | Dense backing plus the needed runtime length/capacity state |
| Circular run | One logical sequence maps to one or two physical initialized intervals | Head/order information only for a workload that needs it |

Backing placement is a separate axis: inline, a provider allocation, or an owning
cell/store. This is not a demand for a runtime tagged union of every representation
or a new wrapper/indirection on every array. A fully initialized fixed block held
at a stable allocation should retain its full-state type when the owning reference
moves through a free list. The boxed source probes are currently blocked by
incorrect region inference and owned-cell measure resolution; those compiler
defects do not refute that representation.

Construction proceeds through a checked partial state. Sealing into a full fixed
array requires full initialization, matching extent, and the correct contiguous
layout; it transfers ownership rather than treating raw slots as values. For an
already acquired provider allocation, a compatible seal may retain that allocation
and its identity. It must account for exact backing extent/alignment and release
authority, not silently substitute a second allocation. Dynamic full extents and
arbitrary sparse source containers are later selection questions, not additional
metadata required by the fixed-array implementation.

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
semantic projection/borrow support needed by the existing probes. This is not
lowering-only work: the wide-record and inline-view programs presently stop before
the relevant backend path. Preserve source semantics during this first slice.

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

Then implement the selected bounded semantic capabilities: full-state construction
and sealing, checked empty-run consume, projected result contracts, and two-span
result provenance. These may amend the specification with their derived evidence;
they do not require a general library-proof framework. Broad source migration and
legacy retirement follow only when the replacement capabilities run the relevant
programs, including fixed blocks on the stable-storage route. Do not retire the
full-array guarantee merely because a variable run accepts a constant literal.

No open choice changes the first slice's required storage/ownership architecture.
Remaining exact spellings and the later sparse, dynamic-refinement, destruction,
and device extensions are bounded follow-on questions. This decision is a scoped
implementation starting point, not a claim that every future container API is
designed or that the proposed language rules have already passed a compiler.

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
percentages. No such external distribution study has been performed here.

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

## What the existing implementation establishes

The following distinctions prevent an implementation gap from being mistaken for
a language design result:

1. **Confirmed implementation limitation:** forming an exclusive view of an inline
   run reports `SemanticUnsupported::ExclusiveViewOverInlineRun`. The relevant path
   is `compiler/src/semantic/check/expressions/flat_storage/slices.rs`. This is not a
   proof that the source operation is semantically invalid. A minimal probe is:

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

2. **Confirmed lowering concern:** `compiler/src/backend/emitter/runs.rs` represents
   inline runs as aggregates and stages whole values through storage slots around
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
removed to obtain this result. This completes the experiment and selection goal;
production implementation has not begun, and no main merge is proposed here.
