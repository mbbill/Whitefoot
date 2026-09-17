# x0 interaction matrix

This is the current hand-derivation record for the 24-axis candidate in
[DESIGN.md](DESIGN.md#candidate-x0-current-state-access-with-separate-resource-accounting),
frozen at research revision `ace725c8`. The owner authorized matrix work with
GPT-5.6 Sol on 2026-09-17. No new language specification, implementation,
runtime measurement or correctness theorem is produced here. Maintain this
record with its three cell batches while x0 is compared; supersede the current
index when the candidate changes, retaining cited counterexamples as evidence.

## Frozen evaluation protocol

Every cell uses all of x0. Evaluate one upper triangle including the diagonal,
300 cells total. Symmetry removes duplicate assignments, not ordered execution
variants. A cell contains pseudocode, a state/rule argument, a dangerous variant
or capability boundary, costs and relevant higher-order dependencies. Simple
cells get short explanations rather than disappearing. Do not evaluate a menu
of alternative choices inside a cell.

Each batch is authored by GPT-5.6 Sol. A worker may identify a rule gap or
propose an axis split, but may not silently patch the frozen candidate. An
unavailable proof mechanism is a gap, not an axiom supplied by a helper name.
Independent challenge checks use the same model and the same rule sheet.

Statuses classify the demonstrated interaction, not the whole language:

- **D — derived:** the shown safe case and its boundary follow from the rules.
- **C — costed restriction:** the task has a stated rewrite/restriction under
  x0; include the loss/cost. This does not waive the current-capability floor.
- **U — unresolved:** a necessary rule, derivation or runtime representation is
  missing. Name it. Prefer U to an invented proof.
- **X — contradiction:** the frozen rules derive an unsafe operation or
  incompatible outcomes. Show both grounds. Merely rejecting an unsafe variant
  is D, not X.

All cost statements are qualitative predictions. Distinguish sequential runtime,
parallel opportunity, representation and checker/writer cost. Unknown overlap
denies permission to overlap; it does not automatically reject safe sequential
source. No compiler-inserted locks, alias tests, occupancy flags or generation
maps may be treated as erased proof information.

## Shared rule sheet, version 1

This sheet fixes the directly usable fragment. Where x0 promises a more general
facility without supplying its proof rules, the explicit gaps below force U.
This is a deliberately honest first matrix, not a claim of a complete calculus.

- **R1 — bindings and slots.** A local `slot` has lexical storage and a typed
  initialization state. `alloc(v)` creates a separate allocation A, initializes
  it with v, and introduces exactly one disposal duty for A. Distinct fresh live
  allocations are disjoint; names alone are not. Fields/elements do not acquire
  independent allocation-disposal duties merely by being addressed.
- **R2 — targets.** `loc(place)` requires live storage and current layout and
  captures the target; it need not require full contents. `loc(*a)` targets an
  owning descriptor's referent. Copying/loading a locator preserves its captured
  target and supplies no liveness, disjointness or disposal evidence. Previously
  loaded copies do not follow later field rebinding. Forming/reading the
  descriptor itself still requires its containing storage to be readable.
- **R3 — read and write.** `read(p)` needs live, layout-compatible, full target
  and Copy contents. `write(p,v)` is Copy-scalar initialization/overwrite of a
  live compatible slot. Both record actual target accesses. Observing non-Copy
  contents requires a checked function contract, not owner duplication.
- **R4 — take, put, replace.** `take(p)` needs a live full target, transfers the
  content and its duties, and leaves the same slot empty. `put(p,v)` needs a
  live empty target and transfers v into it. `replace(p,v)` needs a live full
  target, returns the old content/duties and installs the new ones. Arguments
  evaluate in source order; the final exchange is not hardware atomic.
- **R5 — move and classes.** `move(x)` transfers its full value and duties to
  the destination and empties x. Copy values may duplicate; affine/linear duties
  cannot. Moving an owner descriptor does not move its referent. Moving inline
  content does not retarget locators to the source slot. Whole aggregate read
  or move requires the relevant full layout; field transfer need not empty
  unrelated fields. Linear duties require explicit discharge/transfer.
- **R6 — end and duties.** `release(p)` needs the independently disposable
  target's available unique duty, Live, and a verified disposition of content
  duties. It consumes that duty and ends the target. All aliases lose access;
  any old owner descriptor also loses authority to consume that duty again.
  An empty allocation can end without consuming duties already moved into a
  separate value. Field addressability alone never authorizes releasing the
  containing allocation. Scope exit ends local storage after cleanup.
- **R7 — cleanup.** Derive only a cleanup sequence valid unconditionally on
  the represented exit states, or within branches the source already wrote.
  Full affine values may use their verified cleanup; empty slots have no
  content cleanup. Remaining linear duties reject exit. If full/empty/dead
  alternatives need different runtime actions, the writer must express the
  discriminating control; do not invent a drop flag or test. Availability of
  general aggregate cleanup is subject to the gaps below.
- **R8 — frame and invalidation.** Update the selected target, not every member
  of its possible-target set. Forget old mutable facts whose support may be
  affected; retain facts only under proved separation or explicit valid
  postconditions. End is permanent for that allocation identity. Retire enum
  payload identity on a variant switch. Do not infer Live/Full from old bytes,
  a length assignment, a type parameter or a numeric address.
- **R9 — joins.** For finite local examples, represent a branch as the guarded
  union of its joint states, keyed to the evaluated condition value. Check each
  primitive in every remaining alternative. Recognize identical captured atoms,
  negation and constants; split finite Boolean guards structurally, without SMT.
  A later assignment changes the binding's value, not the old atom. Keep duties
  per alternative; do not intersect them away. Numeric contradiction/separation
  needs the existing WF derivation or an explicit supported proof. Guarded
  enumeration can grow exponentially; no cost claim conceals that.
- **R10 — loops.** A written head invariant must follow from entry, justify the
  body and hold after each backedge substitution. Every exit keeps its actual
  facts and duties; include zero iterations. Check one symbolic iteration;
  do not unroll to an unknown runtime count. Fixed-object invariants may use
  R1-R9. General dynamic families require proof rules not yet supplied here.
- **R11 — calls.** Verify a function body for every input allowed by its
  contract, including aliases unless separation is required. Substitute actual
  targets at calls; validate entry facts and resource transfers, invalidate by
  complete effects, then establish verified exit facts. Recursive calls use
  their declared contract, not unfolding. Function arguments require compatible
  checked contracts; simple identical contracts need no new subtyping rule.
- **R12 — effects.** An effect summary covers the whole execution, including
  argument evaluation and indirect management-state accesses. Restoring contents
  does not erase an intermediate write/hole. Transfers and storage ending must
  be visible in addition to ordinary read/write information. An invariant or
  effect declaration is not evidence that its body meets it.
- **R13 — parts and bounds.** Fixed fields of a live compatible layout are
  separate parts; whole-object access includes affected parts. Captured indices
  and ranges select locations in a captured backing identity. Use proved bounds
  and structural/checked numeric separation. A view descriptor is not fresh
  element storage. Parameter roots may alias unless explicitly separated.
- **R14 — overlap.** Preserve sequential observables and data dependencies.
  Permit call overlap only with proven noninterference of whole-call accesses,
  argument evaluation, transferred duties and retained storage. Read/read is
  compatible; possible read/write, write/write or end/access conflicts deny
  overlap. Shared provider metadata must be included. No new scheduling edge or
  lock is inserted to rescue an otherwise unsupported source operation.
- **R15 — counted parallelism.** Retain v0.59's admitted single-index affine
  maps, adjacent-range helpers and fixed reduction laws. Require bounds and
  cross-iteration separation. Same-index read/modify/write is not an iteration
  conflict; an arbitrary shifted read/write map is not automatically independent.
  Wrapping sum is an admitted example; checked sum and floating addition do not
  acquire that law. State the source-order result being preserved.
- **R16 — representation.** Ghost identities, fact tables and duties are not
  runtime flags. Owned descriptors and locators have their declared runtime
  fields; indexing/views may carry ordinary length/offset data. Never emit
  pointer-wide uniqueness from locator type. A machine alias fact requires its
  own scope and proof. A logical identity inequality alone does not prove
  simultaneously disjoint bytes of ancestor/descendant storage.

Known rule gaps, not permissions to invent mechanisms: general predicate
definition/fold/unfold and quantified resource-family rules; existential target
packaging and unpacking across mutable heterogeneous collections; cleanup of
conditionally partial arbitrary aggregates; generic contract refinement beyond
direct substitution; complete physical lowering and optimizer metadata
semantics; allocation/provider implementation beyond the inherited primitive
contracts. Show local concrete witnesses where R1-R16 suffice, but classify the
general interaction U when it depends on one of these gaps. Preserving current
parallel permission must be demonstrated, not inferred from the word "effects".

`Int` is an ordinary Copy integer; arithmetic must have its stated domain or
use wrapping operations. An example may declare an incoming full linear value
as an entry obligation; it may not invent a magical safe resource constructor.
`request_overlap(f(p), g(q))` requests a verdict on overlapping two source-ordered
calls. It is test notation, not a thread construct. Helpers used for D must
have a small body or a direct R1-R16 derivation. For U, a missing helper contract
may be shown as the exact obligation that still needs a rule.

## Coverage and batch records

- [Rows 1-5](matrix-x0/01-05.md): 110 cells.
- [Rows 6-12](matrix-x0/06-12.md): 112 cells.
- [Rows 13-24](matrix-x0/13-24.md): 78 cells.

All 300 cells now have a code witness and a local verdict: 221 D,
12 C, 67 U and 0 X. This is coverage of
pair assignments, not completeness of programs, proof rules or capability-floor
preservation. The three batches also record selected higher-order witnesses.
No model vote selects a language rule.

## First-round findings

**x0 is not yet a self-consistent, implementable design.** The record establishes
small fragments and exposes obligations that the candidate has not specified.
A D concerns its displayed premises and operations; it must not be promoted to
"this entire feature works." An absence of X is not a soundness theorem. Costs
are qualitative; neither sequential nor parallel performance has been measured.

The most useful results divide into five areas:

| Area | Concrete result or missing rule | Witnesses |
|---|---|---|
| Local state and aliasing | Captured targets, take/put, replacement and guarded joint states support many fixed-object examples without exclusive locator lifetimes. This relies on checking every alternative, not updating every possible target at once. | 1-5, 6-7, 16-16, 16-22 |
| Conditional cleanup | Knowing each path's state does not supply a single valid cleanup at their shared exit. Moving an owner in only one branch may need an explicit source rewrite under R7; a hidden drop flag is not proof erasure. | 3-11, 5-11, 11-11, 11-16 |
| Dynamic structures | General stored-target packages, representation predicates and quantified resource families need introduction, elimination, framing and duty-accounting rules. A named invariant or helper does not supply those rules. | 1-8, 15-15, 15-18, 17-20, 20-20 |
| Providers and physical storage | Payload separation does not separate shared management writes. Suballocation identity must distinguish sibling separation from ancestor overlap, and connect backing lifetime to child duties. R1-R16 do not yet define the necessary formation transition. | 1-14, 13-14, 14-14, 14-22; rows 13-24 higher-order witness 5 |
| Parallelism and lowering | Per-iteration safety alone does not establish current PAR-2 permission. Direct set forms, adjacent-range helpers and fixed reduction laws have distinct boundaries. Source-level separation also needs a defined bridge to physical lowering and scoped optimizer metadata. | 5-23, 6-23, 18-23, 21-23, 23-23, 23-24 |

Several U cells share one missing rule. Count that as one design problem with
multiple dependents, not multiple independent failures. Conversely, closing a
rule gap does not automatically change all of its cells to D: their bodies and
boundaries must be derived again. The current v0.59 capability floor and the
engineering programs in PROGRAMS.md still require an explicit end-to-end
coverage check; a pair matrix does not replace it.

## Axis refinements supported by this round

Keep the current 24 coordinates fixed for this record. These are proposed
refinements for a later candidate, not amendments to x0:

- Axis 8: storing a known target versus opening/repacking a hidden target.
- Axis 11: definite affine cleanup versus conditional or partial cleanup.
- Axis 14: provider/child lifetime duties versus management access conflicts.
- Axis 17: loops over fixed objects versus loops carrying dynamic resource families.
- Axis 20: fixed automatic derivation versus written predicate/family proof steps.
- Axis 24: executable data representation versus scoped optimization facts.

Each refined axis must still select a concrete decision. It must not become a
question with alternative choices embedded in every matrix cell. Other proposed
splits and their local evidence remain in the batch records.

## Challenge record and limits

GPT-5.6 Sol authored the three batches and performed focused cross-batch
challenges under the frozen sheet. Follow-up derivations corrected overly broad
parallel claims, monomorphic stored-target examples mislabeled as existential
problems, a take/restore loop mislabeled as requiring a dynamic resource family,
and branch cleanup claims that lacked an unconditional sequence at the actual
exit. Pseudocode helpers and entry premises were made explicit where needed.

These were content challenges, not DCR, a compiler test run or a formal proof.
The matrix is hand-derived explanatory pseudocode. Structural checks cover the
300 expected unique IDs, code and required fields, verdict/index consistency,
and local navigation. They do not prove the verdicts. Remaining rule gaps stay
U; ordinary unsafe boundary variants stay rejected examples, not X. No current
specification or compiler rule is changed by this record.

## Follow-up refinement

The [gap map](GAPS.md) covers all 67 U and 12 C cells, with code discriminators,
secondary dependencies and witness-precision corrections to investigate. The
[E1 target-package note](TARGET-PACKAGES.md) is the next bounded rule experiment;
it leaves this frozen rule sheet, cell verdicts and totals unchanged. Its local
results must not be read as a completed new 300-cell matrix or a capability-floor
audit.

## Complete upper triangle

These verdicts are not a language-wide pass rate. D is derived only within the shown premises; C records a restriction/cost; U records a missing rule or derivation; X records a contradiction. The mirrored half is omitted.

| Axis | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1. Storage identity | [D](matrix-x0/01-05.md#1-1-fresh-and-permanent-allocation-identities) | [D](matrix-x0/01-05.md#1-2-descriptor-storage-differs-from-referent-identity) | [D](matrix-x0/01-05.md#1-3-heap-transfer-preserves-target-identity) | [D](matrix-x0/01-05.md#1-4-class-does-not-manufacture-identity-separation) | [D](matrix-x0/01-05.md#1-5-a-hole-belongs-to-the-captured-target) | [D](matrix-x0/01-05.md#1-6-replacement-keeps-storage-identity) | [D](matrix-x0/01-05.md#1-7-copied-locators-preserve-one-identity) | [U](matrix-x0/01-05.md#1-8-stored-target-identity-needs-packaging-rules) | [D](matrix-x0/01-05.md#1-9-ended-identity-invalidates-every-old-path) | [D](matrix-x0/01-05.md#1-10-disposal-consumes-the-identity-indexed-duty) | [C](matrix-x0/01-05.md#1-11-cleanup-cannot-guess-identity-state) | [D](matrix-x0/01-05.md#1-12-subobject-identity-is-layout-scoped) | [D](matrix-x0/01-05.md#1-13-indices-select-parts-of-one-backing-identity) | [U](matrix-x0/01-05.md#1-14-provider-and-payload-identities-are-separate) | [U](matrix-x0/01-05.md#1-15-container-identity-abstraction-lacks-general-rules) | [D](matrix-x0/01-05.md#1-16-branch-retains-identity-correlation) | [U](matrix-x0/01-05.md#1-17-dynamic-identity-families-lack-proof-rules) | [D](matrix-x0/01-05.md#1-18-call-substitution-may-unify-identities) | [D](matrix-x0/01-05.md#1-19-recursive-identity-abstraction-is-contractual) | [D](matrix-x0/01-05.md#1-20-identity-facts-have-bounded-automatic-strength) | [D](matrix-x0/01-05.md#1-21-effects-name-actual-identities) | [D](matrix-x0/01-05.md#1-22-logical-separation-permits-local-overlap) | [D](matrix-x0/01-05.md#1-23-iteration-identity-needs-admitted-maps) | [D](matrix-x0/01-05.md#1-24-ghost-identity-is-not-noalias-metadata) |
| 2. Owned storage representation | — | [D](matrix-x0/01-05.md#2-2-inline-and-allocated-ownership-representations) | [D](matrix-x0/01-05.md#2-3-moving-inline-content-and-descriptors-differs) | [D](matrix-x0/01-05.md#2-4-representation-follows-contained-obligations) | [D](matrix-x0/01-05.md#2-5-holes-do-not-remove-storage-representation) | [D](matrix-x0/01-05.md#2-6-allocated-replacement-preserves-descriptor-ownership) | [D](matrix-x0/01-05.md#2-7-locator-copies-never-become-owners) | [D](matrix-x0/01-05.md#2-8-storing-an-owner-differs-from-storing-a-locator) | [D](matrix-x0/01-05.md#2-9-local-lifetime-is-not-extended-by-ownership-movement) | [D](matrix-x0/01-05.md#2-10-allocation-duty-is-separate-from-content) | [U](matrix-x0/01-05.md#2-11-representation-does-not-supply-conditional-cleanup) | [D](matrix-x0/01-05.md#2-12-whole-representation-requires-full-parts) | [D](matrix-x0/01-05.md#2-13-views-do-not-own-backing-storage) | [U](matrix-x0/01-05.md#2-14-payload-owners-may-share-provider-representation) | [U](matrix-x0/01-05.md#2-15-container-representation-must-expose-ownership-invariant) | [D](matrix-x0/01-05.md#2-16-guarded-descriptors-preserve-alternative-duties) | [U](matrix-x0/01-05.md#2-17-loop-owned-descriptor-families-are-unresolved) | [D](matrix-x0/01-05.md#2-18-contracts-distinguish-owner-and-locator-parameters) | [D](matrix-x0/01-05.md#2-19-generic-representation-cannot-imply-ownership) | [D](matrix-x0/01-05.md#2-20-proof-accounting-stays-separate-from-machine-layout) | [D](matrix-x0/01-05.md#2-21-descriptor-and-referent-effects-are-distinct) | [D](matrix-x0/01-05.md#2-22-disjoint-payloads-can-still-conflict-on-owner-slots) | [D](matrix-x0/01-05.md#2-23-parallel-element-work-cannot-move-one-owner-twice) | [D](matrix-x0/01-05.md#2-24-equal-pointer-shapes-do-not-justify-optimizer-attributes) |
| 3. Value and ownership transfer | — | — | [D](matrix-x0/01-05.md#3-3-repeated-handoff-preserves-one-duty) | [D](matrix-x0/01-05.md#3-4-transfer-follows-value-class) | [D](matrix-x0/01-05.md#3-5-take-is-content-transfer-not-storage-transfer) | [D](matrix-x0/01-05.md#3-6-replacement-transfers-old-obligations) | [D](matrix-x0/01-05.md#3-7-aliases-do-not-follow-moved-inline-values) | [D](matrix-x0/01-05.md#3-8-moving-a-stored-locator-copies-no-owner) | [D](matrix-x0/01-05.md#3-9-transfer-cannot-extend-inline-storage-lifetime) | [D](matrix-x0/01-05.md#3-10-owner-transfer-carries-disposal-authority) | [C](matrix-x0/01-05.md#3-11-moved-resources-constrain-cleanup) | [D](matrix-x0/01-05.md#3-12-partial-field-transfer-blocks-whole-move) | [D](matrix-x0/01-05.md#3-13-moving-a-view-preserves-backing-association) | [U](matrix-x0/01-05.md#3-14-provider-backed-owner-transfer-has-an-open-dependency-model) | [U](matrix-x0/01-05.md#3-15-transfer-through-dynamic-containers-is-unresolved) | [D](matrix-x0/01-05.md#3-16-branch-transfer-preserves-duty-correlation) | [D](matrix-x0/01-05.md#3-17-loop-carried-fixed-ownership-can-rotate) | [D](matrix-x0/01-05.md#3-18-call-transfer-is-explicit-in-contracts) | [D](matrix-x0/01-05.md#3-19-recursive-ownership-needs-its-declared-measure-and-contract) | [D](matrix-x0/01-05.md#3-20-transfer-proofs-cannot-be-asserted) | [D](matrix-x0/01-05.md#3-21-transfer-is-distinct-from-access) | [D](matrix-x0/01-05.md#3-22-two-calls-cannot-consume-one-ownership-duty) | [D](matrix-x0/01-05.md#3-23-per-iteration-moves-require-separated-sources) | [U](matrix-x0/01-05.md#3-24-erased-movement-must-preserve-operational-copies) |
| 4. Copy, affine and linear | — | — | — | [D](matrix-x0/01-05.md#4-4-class-lattice-follows-obligations) | [D](matrix-x0/01-05.md#4-5-holes-retain-transferred-class-obligations) | [D](matrix-x0/01-05.md#4-6-non-copy-observation-cannot-duplicate-ownership) | [D](matrix-x0/01-05.md#4-7-copying-a-locator-ignores-pointee-class) | [D](matrix-x0/01-05.md#4-8-stored-locators-stay-copy-while-stored-owners-do-not) | [D](matrix-x0/01-05.md#4-9-ended-targets-do-not-change-locator-copyability) | [D](matrix-x0/01-05.md#4-10-disposal-duties-are-never-copy) | [D](matrix-x0/01-05.md#4-11-affine-cleanup-and-linear-rejection-are-explicit) | [D](matrix-x0/01-05.md#4-12-composite-class-follows-field-contents) | [D](matrix-x0/01-05.md#4-13-copying-a-view-does-not-copy-element-duties) | [U](matrix-x0/01-05.md#4-14-provider-dependency-class-is-unspecified) | [U](matrix-x0/01-05.md#4-15-container-element-classes-need-resource-predicates) | [D](matrix-x0/01-05.md#4-16-branch-alternatives-cannot-merge-linear-duties-by-intersection) | [D](matrix-x0/01-05.md#4-17-loop-invariants-must-enumerate-fixed-linear-ownership) | [D](matrix-x0/01-05.md#4-18-contracts-combine-access-demands-but-not-resources) | [D](matrix-x0/01-05.md#4-19-passed-functions-carry-class-sensitive-contracts) | [D](matrix-x0/01-05.md#4-20-proof-steps-cannot-weaken-value-class) | [D](matrix-x0/01-05.md#4-21-effects-expose-obligation-transfers) | [D](matrix-x0/01-05.md#4-22-affine-cleanup-calls-can-conflict) | [U](matrix-x0/01-05.md#4-23-reduction-laws-do-not-dispose-linear-values) | [U](matrix-x0/01-05.md#4-24-erased-classes-cannot-become-runtime-flags) |
| 5. Holes | — | — | — | — | [D](matrix-x0/01-05.md#5-5-take-and-put-preserve-live-storage) | [D](matrix-x0/01-05.md#5-6-replacement-cannot-commit-over-a-hole) | [D](matrix-x0/01-05.md#5-7-aliases-may-restore-one-shared-hole) | [D](matrix-x0/01-05.md#5-8-storing-a-locator-does-not-snapshot-fullness) | [D](matrix-x0/01-05.md#5-9-holes-remain-addressable-but-not-readable) | [D](matrix-x0/01-05.md#5-10-empty-allocation-may-be-released) | [C](matrix-x0/01-05.md#5-11-branch-dependent-holes-require-source-cleanup-control) | [D](matrix-x0/01-05.md#5-12-one-field-hole-blocks-whole-object-operations) | [D](matrix-x0/01-05.md#5-13-element-holes-require-captured-indices) | [U](matrix-x0/01-05.md#5-14-holes-do-not-erase-provider-effects) | [U](matrix-x0/01-05.md#5-15-container-holes-need-quantified-invariants) | [D](matrix-x0/01-05.md#5-16-branch-joins-preserve-correlated-holes) | [D](matrix-x0/01-05.md#5-17-fixed-object-loop-holes-can-be-invariant-controlled) | [D](matrix-x0/01-05.md#5-18-contracts-must-expose-temporary-holes) | [D](matrix-x0/01-05.md#5-19-higher-order-contracts-retain-hole-effects) | [D](matrix-x0/01-05.md#5-20-a-proof-step-cannot-conjure-fullness) | [D](matrix-x0/01-05.md#5-21-hole-effects-survive-restoration) | [D](matrix-x0/01-05.md#5-22-temporary-holes-conflict-with-concurrent-access) | [C](matrix-x0/01-05.md#5-23-per-iteration-holes-need-separated-maps) | [D](matrix-x0/01-05.md#5-24-holes-are-proof-state-not-runtime-tags) |
| 6. Reading and replacement | — | — | — | — | — | [D](matrix-x0/06-12.md#6-6-reading-and-replacement--reading-and-replacement) | [D](matrix-x0/06-12.md#6-7-reading-and-replacement--locator-copying-and-aliasing) | [D](matrix-x0/06-12.md#6-8-reading-and-replacement--stored-locators) | [D](matrix-x0/06-12.md#6-9-reading-and-replacement--validity-and-scope) | [D](matrix-x0/06-12.md#6-10-reading-and-replacement--disposal-authority) | [D](matrix-x0/06-12.md#6-11-reading-and-replacement--automatic-cleanup) | [D](matrix-x0/06-12.md#6-12-reading-and-replacement--fields-and-active-layout) | [D](matrix-x0/06-12.md#6-13-reading-and-replacement--arrays-and-ranges) | [U](matrix-x0/06-12.md#6-14-reading-and-replacement--providers-and-shared-management) | [U](matrix-x0/06-12.md#6-15-reading-and-replacement--containers-and-library-invariants) | [D](matrix-x0/06-12.md#6-16-reading-and-replacement--branch-joins) | [D](matrix-x0/06-12.md#6-17-reading-and-replacement--loops-and-exits) | [D](matrix-x0/06-12.md#6-18-reading-and-replacement--function-contracts) | [D](matrix-x0/06-12.md#6-19-reading-and-replacement--generics-recursion-and-function-arguments) | [U](matrix-x0/06-12.md#6-20-reading-and-replacement--proof-mechanism) | [D](matrix-x0/06-12.md#6-21-reading-and-replacement--access-and-state-effects) | [D](matrix-x0/06-12.md#6-22-reading-and-replacement--call-overlap) | [C](matrix-x0/06-12.md#6-23-reading-and-replacement--loop-overlap-and-reductions) | [D](matrix-x0/06-12.md#6-24-reading-and-replacement--lowering-and-optimization-facts) |
| 7. Locator copying and aliasing | — | — | — | — | — | — | [D](matrix-x0/06-12.md#7-7-locator-copying-and-aliasing--locator-copying-and-aliasing) | [D](matrix-x0/06-12.md#7-8-locator-copying-and-aliasing--stored-locators) | [D](matrix-x0/06-12.md#7-9-locator-copying-and-aliasing--validity-and-scope) | [D](matrix-x0/06-12.md#7-10-locator-copying-and-aliasing--disposal-authority) | [D](matrix-x0/06-12.md#7-11-locator-copying-and-aliasing--automatic-cleanup) | [D](matrix-x0/06-12.md#7-12-locator-copying-and-aliasing--fields-and-active-layout) | [D](matrix-x0/06-12.md#7-13-locator-copying-and-aliasing--arrays-and-ranges) | [U](matrix-x0/06-12.md#7-14-locator-copying-and-aliasing--providers-and-shared-management) | [U](matrix-x0/06-12.md#7-15-locator-copying-and-aliasing--containers-and-library-invariants) | [D](matrix-x0/06-12.md#7-16-locator-copying-and-aliasing--branch-joins) | [D](matrix-x0/06-12.md#7-17-locator-copying-and-aliasing--loops-and-exits) | [D](matrix-x0/06-12.md#7-18-locator-copying-and-aliasing--function-contracts) | [D](matrix-x0/06-12.md#7-19-locator-copying-and-aliasing--generics-recursion-and-function-arguments) | [D](matrix-x0/06-12.md#7-20-locator-copying-and-aliasing--proof-mechanism) | [D](matrix-x0/06-12.md#7-21-locator-copying-and-aliasing--access-and-state-effects) | [D](matrix-x0/06-12.md#7-22-locator-copying-and-aliasing--call-overlap) | [D](matrix-x0/06-12.md#7-23-locator-copying-and-aliasing--loop-overlap-and-reductions) | [D](matrix-x0/06-12.md#7-24-locator-copying-and-aliasing--lowering-and-optimization-facts) |
| 8. Stored locators | — | — | — | — | — | — | — | [D](matrix-x0/06-12.md#8-8-stored-locators--stored-locators) | [D](matrix-x0/06-12.md#8-9-stored-locators--validity-and-scope) | [D](matrix-x0/06-12.md#8-10-stored-locators--disposal-authority) | [D](matrix-x0/06-12.md#8-11-stored-locators--automatic-cleanup) | [D](matrix-x0/06-12.md#8-12-stored-locators--fields-and-active-layout) | [D](matrix-x0/06-12.md#8-13-stored-locators--arrays-and-ranges) | [U](matrix-x0/06-12.md#8-14-stored-locators--providers-and-shared-management) | [U](matrix-x0/06-12.md#8-15-stored-locators--containers-and-library-invariants) | [D](matrix-x0/06-12.md#8-16-stored-locators--branch-joins) | [U](matrix-x0/06-12.md#8-17-stored-locators--loops-and-exits) | [D](matrix-x0/06-12.md#8-18-stored-locators--function-contracts) | [U](matrix-x0/06-12.md#8-19-stored-locators--generics-recursion-and-function-arguments) | [U](matrix-x0/06-12.md#8-20-stored-locators--proof-mechanism) | [U](matrix-x0/06-12.md#8-21-stored-locators--access-and-state-effects) | [U](matrix-x0/06-12.md#8-22-stored-locators--call-overlap) | [U](matrix-x0/06-12.md#8-23-stored-locators--loop-overlap-and-reductions) | [D](matrix-x0/06-12.md#8-24-stored-locators--lowering-and-optimization-facts) |
| 9. Validity and scope | — | — | — | — | — | — | — | — | [D](matrix-x0/06-12.md#9-9-validity-and-scope--validity-and-scope) | [D](matrix-x0/06-12.md#9-10-validity-and-scope--disposal-authority) | [D](matrix-x0/06-12.md#9-11-validity-and-scope--automatic-cleanup) | [D](matrix-x0/06-12.md#9-12-validity-and-scope--fields-and-active-layout) | [D](matrix-x0/06-12.md#9-13-validity-and-scope--arrays-and-ranges) | [D](matrix-x0/06-12.md#9-14-validity-and-scope--providers-and-shared-management) | [U](matrix-x0/06-12.md#9-15-validity-and-scope--containers-and-library-invariants) | [D](matrix-x0/06-12.md#9-16-validity-and-scope--branch-joins) | [D](matrix-x0/06-12.md#9-17-validity-and-scope--loops-and-exits) | [D](matrix-x0/06-12.md#9-18-validity-and-scope--function-contracts) | [D](matrix-x0/06-12.md#9-19-validity-and-scope--generics-recursion-and-function-arguments) | [D](matrix-x0/06-12.md#9-20-validity-and-scope--proof-mechanism) | [D](matrix-x0/06-12.md#9-21-validity-and-scope--access-and-state-effects) | [D](matrix-x0/06-12.md#9-22-validity-and-scope--call-overlap) | [C](matrix-x0/06-12.md#9-23-validity-and-scope--loop-overlap-and-reductions) | [D](matrix-x0/06-12.md#9-24-validity-and-scope--lowering-and-optimization-facts) |
| 10. Disposal authority | — | — | — | — | — | — | — | — | — | [D](matrix-x0/06-12.md#10-10-disposal-authority--disposal-authority) | [D](matrix-x0/06-12.md#10-11-disposal-authority--automatic-cleanup) | [D](matrix-x0/06-12.md#10-12-disposal-authority--fields-and-active-layout) | [D](matrix-x0/06-12.md#10-13-disposal-authority--arrays-and-ranges) | [U](matrix-x0/06-12.md#10-14-disposal-authority--providers-and-shared-management) | [U](matrix-x0/06-12.md#10-15-disposal-authority--containers-and-library-invariants) | [D](matrix-x0/06-12.md#10-16-disposal-authority--branch-joins) | [U](matrix-x0/06-12.md#10-17-disposal-authority--loops-and-exits) | [D](matrix-x0/06-12.md#10-18-disposal-authority--function-contracts) | [D](matrix-x0/06-12.md#10-19-disposal-authority--generics-recursion-and-function-arguments) | [D](matrix-x0/06-12.md#10-20-disposal-authority--proof-mechanism) | [D](matrix-x0/06-12.md#10-21-disposal-authority--access-and-state-effects) | [D](matrix-x0/06-12.md#10-22-disposal-authority--call-overlap) | [C](matrix-x0/06-12.md#10-23-disposal-authority--loop-overlap-and-reductions) | [D](matrix-x0/06-12.md#10-24-disposal-authority--lowering-and-optimization-facts) |
| 11. Automatic cleanup | — | — | — | — | — | — | — | — | — | — | [C](matrix-x0/06-12.md#11-11-automatic-cleanup--automatic-cleanup) | [D](matrix-x0/06-12.md#11-12-automatic-cleanup--fields-and-active-layout) | [U](matrix-x0/06-12.md#11-13-automatic-cleanup--arrays-and-ranges) | [U](matrix-x0/06-12.md#11-14-automatic-cleanup--providers-and-shared-management) | [U](matrix-x0/06-12.md#11-15-automatic-cleanup--containers-and-library-invariants) | [D](matrix-x0/06-12.md#11-16-automatic-cleanup--branch-joins) | [U](matrix-x0/06-12.md#11-17-automatic-cleanup--loops-and-exits) | [D](matrix-x0/06-12.md#11-18-automatic-cleanup--function-contracts) | [D](matrix-x0/06-12.md#11-19-automatic-cleanup--generics-recursion-and-function-arguments) | [U](matrix-x0/06-12.md#11-20-automatic-cleanup--proof-mechanism) | [D](matrix-x0/06-12.md#11-21-automatic-cleanup--access-and-state-effects) | [D](matrix-x0/06-12.md#11-22-automatic-cleanup--call-overlap) | [C](matrix-x0/06-12.md#11-23-automatic-cleanup--loop-overlap-and-reductions) | [D](matrix-x0/06-12.md#11-24-automatic-cleanup--lowering-and-optimization-facts) |
| 12. Fields and active layout | — | — | — | — | — | — | — | — | — | — | — | [D](matrix-x0/06-12.md#12-12-fields-and-active-layout--fields-and-active-layout) | [D](matrix-x0/06-12.md#12-13-fields-and-active-layout--arrays-and-ranges) | [U](matrix-x0/06-12.md#12-14-fields-and-active-layout--providers-and-shared-management) | [U](matrix-x0/06-12.md#12-15-fields-and-active-layout--containers-and-library-invariants) | [D](matrix-x0/06-12.md#12-16-fields-and-active-layout--branch-joins) | [D](matrix-x0/06-12.md#12-17-fields-and-active-layout--loops-and-exits) | [D](matrix-x0/06-12.md#12-18-fields-and-active-layout--function-contracts) | [U](matrix-x0/06-12.md#12-19-fields-and-active-layout--generics-recursion-and-function-arguments) | [D](matrix-x0/06-12.md#12-20-fields-and-active-layout--proof-mechanism) | [D](matrix-x0/06-12.md#12-21-fields-and-active-layout--access-and-state-effects) | [D](matrix-x0/06-12.md#12-22-fields-and-active-layout--call-overlap) | [C](matrix-x0/06-12.md#12-23-fields-and-active-layout--loop-overlap-and-reductions) | [D](matrix-x0/06-12.md#12-24-fields-and-active-layout--lowering-and-optimization-facts) |
| 13. Arrays and ranges | — | — | — | — | — | — | — | — | — | — | — | — | [D](matrix-x0/13-24.md#13-13-captured-array-elements-remain-parts-of-one-backing) | [U](matrix-x0/13-24.md#13-14-disjoint-payload-ranges-may-share-provider-metadata) | [U](matrix-x0/13-24.md#13-15-a-range-view-needs-a-checked-container-relation) | [D](matrix-x0/13-24.md#13-16-a-guarded-index-preserves-target-and-state-alternatives) | [D](matrix-x0/13-24.md#13-17-restored-per-iteration-holes-need-no-dynamic-family) | [D](matrix-x0/13-24.md#13-18-a-range-contract-substitutes-the-callers-captured-backing) | [D](matrix-x0/13-24.md#13-19-generic-distinct-roots-do-not-prove-disjoint-element-access) | [D](matrix-x0/13-24.md#13-20-adjacent-range-separation-uses-the-fixed-numeric-family) | [D](matrix-x0/13-24.md#13-21-an-element-hole-is-visible-in-the-whole-range-effect) | [D](matrix-x0/13-24.md#13-22-disjoint-subrange-calls-may-overlap) | [D](matrix-x0/13-24.md#13-23-same-affine-map-preserves-current-loop-overlap) | [U](matrix-x0/13-24.md#13-24-range-proofs-lower-without-uniqueness-claims) |
| 14. Providers and shared management | — | — | — | — | — | — | — | — | — | — | — | — | — | [U](matrix-x0/13-24.md#14-14-provider-release-waits-for-dependent-allocations) | [U](matrix-x0/13-24.md#14-15-container-allocation-exposes-provider-effects) | [D](matrix-x0/13-24.md#14-16-branch-specific-provider-duties-stay-guarded) | [U](matrix-x0/13-24.md#14-17-loop-created-provider-children-need-symbolic-duties) | [D](matrix-x0/13-24.md#14-18-provider-effects-participate-in-contract-substitution) | [D](matrix-x0/13-24.md#14-19-passed-allocators-need-independently-verified-effects) | [U](matrix-x0/13-24.md#14-20-provider-dependency-families-exceed-fixed-propagation) | [U](matrix-x0/13-24.md#14-21-management-state-is-part-of-allocation-effects) | [D](matrix-x0/13-24.md#14-22-shared-provider-writes-deny-call-overlap) | [D](matrix-x0/13-24.md#14-23-provider-traffic-inside-iterations-blocks-counted-overlap) | [U](matrix-x0/13-24.md#14-24-provider-implementation-is-not-erased-proof-metadata) |
| 15. Containers and library invariants | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [U](matrix-x0/13-24.md#15-15-general-container-invariants-lack-fold-and-unfold) | [D](matrix-x0/13-24.md#15-16-finite-variants-can-carry-explicit-container-states) | [U](matrix-x0/13-24.md#15-17-mutating-containers-across-loops-needs-a-quantified-invariant) | [U](matrix-x0/13-24.md#15-18-abstract-container-methods-need-relational-contracts) | [U](matrix-x0/13-24.md#15-19-higher-order-container-traversal-lacks-contract-refinement) | [U](matrix-x0/13-24.md#15-20-an-explicit-invariant-step-needs-a-defined-predicate-calculus) | [D](matrix-x0/13-24.md#15-21-container-effects-must-expose-representation-access) | [D](matrix-x0/13-24.md#15-22-opaque-container-calls-cannot-claim-overlap-by-assertion) | [U](matrix-x0/13-24.md#15-23-disjoint-container-shards-need-more-than-distinct-descriptors) | [U](matrix-x0/13-24.md#15-24-container-proof-erasure-leaves-concrete-descriptors) |
| 16. Branch joins | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [D](matrix-x0/13-24.md#16-16-guarded-union-retains-correlated-duties) | [D](matrix-x0/13-24.md#16-17-a-loop-head-may-carry-a-finite-guarded-state) | [D](matrix-x0/13-24.md#16-18-conditional-actual-targets-substitute-as-alternatives) | [U](matrix-x0/13-24.md#16-19-generic-recursion-preserves-contract-level-joins) | [D](matrix-x0/13-24.md#16-20-boolean-guarded-checking-is-fixed-but-numeric-case-proof-is-bounded) | [D](matrix-x0/13-24.md#16-21-state-effects-publish-a-guarded-exit-relation) | [D](matrix-x0/13-24.md#16-22-conditional-disjointness-must-hold-in-every-alternative) | [D](matrix-x0/13-24.md#16-23-loop-overlap-cannot-depend-on-a-per-iteration-unresolved-branch) | [D](matrix-x0/13-24.md#16-24-erased-branches-retain-ordinary-runtime-control) |
| 17. Loops and exits | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [D](matrix-x0/13-24.md#17-17-one-fixed-resource-can-circulate-through-a-loop) | [D](matrix-x0/13-24.md#17-18-early-exits-carry-their-actual-resource-state) | [U](matrix-x0/13-24.md#17-19-recursive-traversal-needs-a-stable-declared-resource-relation) | [U](matrix-x0/13-24.md#17-20-dynamic-allocation-families-need-explicit-certificate-rules) | [D](matrix-x0/13-24.md#17-21-loop-effects-include-intermediate-holes-and-exits) | [D](matrix-x0/13-24.md#17-22-a-call-pair-across-uncertain-loop-iterations-has-no-par-1-window) | [D](matrix-x0/13-24.md#17-23-counted-loops-preserve-admitted-reductions) | [U](matrix-x0/13-24.md#17-24-loop-proof-state-erases-while-source-exits-remain) |
| 18. Function contracts | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [D](matrix-x0/13-24.md#18-18-equal-actual-targets-combine-access-but-not-duties) | [D](matrix-x0/13-24.md#18-19-identical-passed-function-contracts-are-checkable) | [D](matrix-x0/13-24.md#18-20-a-contract-declaration-is-not-body-evidence) | [D](matrix-x0/13-24.md#18-21-access-and-exit-state-are-independent-contract-dimensions) | [D](matrix-x0/13-24.md#18-22-verified-contract-projection-decides-call-overlap) | [D](matrix-x0/13-24.md#18-23-helper-effects-preserve-adjacent-range-permission) | [U](matrix-x0/13-24.md#18-24-contracts-can-authorize-scoped-facts-not-pointer-wide-attributes) |
| 19. Generics, recursion and function arguments | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [U](matrix-x0/13-24.md#19-19-recursion-uses-declarations-rather-than-expansion) | [D](matrix-x0/13-24.md#19-20-generic-separation-cannot-be-inferred-from-type-names) | [D](matrix-x0/13-24.md#19-21-passed-functions-carry-complete-access-and-transfer-rows) | [D](matrix-x0/13-24.md#19-22-indirect-calls-overlap-only-through-verified-summaries) | [C](matrix-x0/13-24.md#19-23-generic-callbacks-in-counted-loops-require-per-element-projection) | [D](matrix-x0/13-24.md#19-24-erased-target-parameters-do-not-imply-runtime-uniqueness) |
| 20. Proof mechanism | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [U](matrix-x0/13-24.md#20-20-written-steps-cannot-fill-the-family-rule-gap) | [D](matrix-x0/13-24.md#20-21-invalidation-precedes-publication-of-replacement-facts) | [D](matrix-x0/13-24.md#20-22-absent-separation-proof-denies-overlap-deterministically) | [D](matrix-x0/13-24.md#20-23-fixed-affine-derivation-is-sufficient-for-counted-overlap) | [U](matrix-x0/13-24.md#20-24-proof-erasure-needs-a-defined-scoped-metadata-bridge) |
| 21. Access and state effects | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [D](matrix-x0/13-24.md#21-21-restored-state-does-not-erase-execution-effects) | [D](matrix-x0/13-24.md#21-22-intermediate-holes-conflict-with-overlapping-readers) | [D](matrix-x0/13-24.md#21-23-reduction-effects-separate-accumulator-and-element-reads) | [D](matrix-x0/13-24.md#21-24-state-effects-erase-but-runtime-data-changes-remain) |
| 22. Call overlap | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [D](matrix-x0/13-24.md#22-22-overlap-preserves-the-ordered-result) | [D](matrix-x0/13-24.md#22-23-call-and-loop-permissions-share-complete-footprints) | [D](matrix-x0/13-24.md#22-24-overlap-permission-does-not-mandate-parallel-lowering) |
| 23. Loop overlap and reductions | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [C](matrix-x0/13-24.md#23-23-only-fixed-reductions-admit-arbitrary-recombination) | [U](matrix-x0/13-24.md#23-24-admitted-loop-facts-need-scoped-lowering-semantics) |
| 24. Lowering and optimization facts | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | [D](matrix-x0/13-24.md#24-24-proof-erasure-has-a-conservative-executable-floor) |
