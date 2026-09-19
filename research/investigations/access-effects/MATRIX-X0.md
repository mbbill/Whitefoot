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

- Rows 1-5 (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758): 110 cells.
- Rows 6-12 (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758): 112 cells.
- Rows 13-24 (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758): 78 cells.

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

## Follow-up discussion and parked drafts

The [gap map](GAPS.md) covers all 67 U and 12 C cells, with code discriminators,
secondary dependencies and witness-precision corrections to investigate. The
E1 target-package note (`TARGET-PACKAGES.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) and
E2 range-family note (`RANGE-FAMILIES.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) are parked, unselected drafts. They
leave this frozen rule sheet, cell verdicts and totals unchanged. Their local
results are not a completed successor matrix or capability-floor audit. The
owner's subsequent discussion is recorded in
[next-candidate preparation](DESIGN.md#next-candidate-preparation-temporary-references-and-effect-derived-calls).
The next pass has not started; do not patch x0 or import E1/E2 rules while
preparing it.

## Complete upper triangle

These verdicts are not a language-wide pass rate. D is derived only within the shown premises; C records a restriction/cost; U records a missing rule or derivation; X records a contradiction. The mirrored half is omitted.

| Axis | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1. Storage identity | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 2. Owned storage representation | — | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 3. Value and ownership transfer | — | — | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 4. Copy, affine and linear | — | — | — | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 5. Holes | — | — | — | — | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/01-05.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 6. Reading and replacement | — | — | — | — | — | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 7. Locator copying and aliasing | — | — | — | — | — | — | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 8. Stored locators | — | — | — | — | — | — | — | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 9. Validity and scope | — | — | — | — | — | — | — | — | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 10. Disposal authority | — | — | — | — | — | — | — | — | — | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 11. Automatic cleanup | — | — | — | — | — | — | — | — | — | — | C (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 12. Fields and active layout | — | — | — | — | — | — | — | — | — | — | — | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/06-12.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 13. Arrays and ranges | — | — | — | — | — | — | — | — | — | — | — | — | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 14. Providers and shared management | — | — | — | — | — | — | — | — | — | — | — | — | — | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 15. Containers and library invariants | — | — | — | — | — | — | — | — | — | — | — | — | — | — | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 16. Branch joins | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 17. Loops and exits | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 18. Function contracts | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 19. Generics, recursion and function arguments | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | C (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 20. Proof mechanism | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 21. Access and state effects | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 22. Call overlap | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 23. Loop overlap and reductions | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | C (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) | U (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
| 24. Lowering and optimization facts | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | D (`matrix-x0/13-24.md`, removed in the cleanup, recoverable at commit 4ca62f8db758) |
