# E0.1 Traceability into the Capability-Floor Program

Status: G0-Core reconciliation, 2026-07-14. E0.1 remains suspended. This file
prevents the dense-family program from silently losing, restarting, or treating
as approved any arm, control, measurement, or review obligation from the
historical fixed-record/owning-sequence work.

## 1. Precedence and interpretation

D11 and D12 supersede E0.1's next-step authorization. The historical files are
evidence and protocol input, not active preregistrations. G0-Core cannot restart
them. A separately owner-authorized dense Family Lock A must explicitly retain,
revise, or supersede every row below before candidate work.

The exact hostile-reviewed ownership-route protocol is a historical Git object,
not the later supersession edits in the working-tree file:

- commit: `1c7b980e5eec19042839711ca54ad015a96bf0a2`;
- path: `experiments/data-layout-owning-sequence/OWNERSHIP_ROUTE_PROTOCOL.md`;
- byte count: 46,320; and
- SHA-256: `88d70083f9cf0219d558675b34a42f54c851793125fccebc07c3f48f4aa1b003`.

The detached candidate is durably archived at
`experiments/data-layout-owning-sequence/DETACHED_CANDIDATE.patch`, against base
commit `58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1`, with 57,547 bytes and SHA-256
`bed070414f9552ea105857404d6d1296b98542a28cc65fa6899a197830e6774e`.
These identities make both reviewed inputs recoverable; neither is active or
authorized.

E0.1 mixed three questions that the capability census now separates:

1. whether fixed contiguous record/AoS storage is expressible;
2. which records are semantically duplicable; and
3. how an owner represents and transitions partially live affine storage.

Layout eligibility, Copy, and live-set state are orthogonal. Evidence for one
cannot select another. The general dense-family requirement also includes
unknown-length construction, pop, remove, growth, relocation, compaction,
sorting, exact destruction, failure, and iteration, which neither old fixed
route closes.

## 2. Ownership and initialization routes

| Historical item | What it tested | G0-Core disposition | Mandatory later treatment |
|---|---|---|---|
| `CURRENT` | Unchanged language and primitive-buffer identity | Retain as B-FIX authority, not a competitor on new contracts. | Every family runs exact unchanged-source/layout/IR/assembly controls. |
| `DECLARATIVE_COPY` | Positive fail-closed duplicability declaration for a closed record domain, reusing full repeated fill | Retain as one fixed-AoS ownership candidate. It can address `ST-AOS` for deliberately Copy records but cannot establish affine storage, unknown-length append, move-out, or exact affine drop. | A future lock must decide whether this arm remains, revise its record domain, or supersede it with a justified alternative; it must measure hidden record materialization and weak-writer misuse. |
| `AFFINE_FIXED_BUILDER` | Nominally affine no-resource records, exact-capacity per-slot construction, finish only at full capacity | Retain as a historical exact-fill control, not as a general dense sequence solution. Its confined builder cannot cross calls, expose a live prefix, grow, pop, remove, or contain arbitrary drop-bearing T. | Any retained form must be priced against the general `ST-DENSE`/`ST-HOLE`/`EX-ABANDON` obligations. A general affine builder must dispose exactly its live prefix and cannot rely on a later `finish`. |
| Automatic structural Copy | Inferred Copy for every eligible all-Copy-leaf record | Remains an unselected alternative. The old protocol excluded it from one paired run but did not reject it in production. | If revived, confront nominal protocol-token counterexamples, field-evolution behavior, copy provenance, and the design-tree declarative-copy path. |
| Copy by default plus negative `affine` marker | Omission grants duplicability | Preserve the old fail-closed objection. It is not an admissible default without new evidence. | Any revival must prove that omission cannot silently accept unwanted duplication and must reopen R3 scope. |
| Affine fixed storage predicate | Storage eligibility without semantic Copy | Preserve the separation as a capability requirement, not the old exact predicate. | Family Lock A may compare checked routes for `ST-AOS`, `OW-*`, and `OW-DROP`; it may not infer Copy from layout or no-drop representation. |
| Recursive recipe | Fresh value generation for repeated full initialization | Preserve as an unpriced alternative. GRAM-9 prevents treating nested construction as an atom. | A future lock must freeze grammar/evaluation/effect semantics and compare its surface/checker cost; no recipe is assumed free. |
| Single-level initializer | Limited fresh construction | Preserve only as a possible bounded contract; it cannot silently stand for nested or generic records. | Either justify the exact admitted domain and blocked use cases or remove it from the family comparison. |
| Explicit Repeat/Clone | Semantic duplication rather than storage relocation | Preserve as a separate `OW-CLONE` behavior contract. | Require explicit callable behavior, partial-failure cleanup, and cost accounting; never use it to simulate affine relocation. |
| Per-slot builder / initialized-prefix owner | Partial initialization | Split. An initialized-prefix owner is the mandatory `ST-DENSE` steady state; a transient exact-fill builder is only one construction protocol. | Compare complete safe transitions, not names. Every normal exit, underfill, overfill, and abandoned protocol state needs a proof. |
| Public `MaybeUninit`, raw tail, unchecked `set_len`, or split occupancy/payload mutation | Rust-like raw implementation privilege | Retain rejection as ordinary writer/library surface. | A candidate may use only checked atomic transitions or unforgeable proof state; any trusted path is separately ledgered and hostile reviewed. |

## 3. Historical semantic details that remain live inputs

| Obligation | Historical evidence | Current mapping |
|---|---|---|
| Record layout is target-derived | E0.1 froze target size, alignment, stride, offsets, object-size and pointer-index ceilings, zero-count behavior, and allocator alignment. | `ST-AOS`, `FL-CAPACITY`, target `DataLayout`, and Family Lock A layout fixtures. No target property may change source Copy classification. |
| `Flat != Copy` | The detached prototype and reviews found that storage eligibility must not authorize implicit record duplication. | `ST-AOS` is separate from `OW-CLONE`/Copy classification. |
| Nested marked records fail closed | The declarative arm required every record-typed field to carry its own positive declaration. | Retain if that candidate survives; an outer marker cannot launder affine inner ownership. |
| Every ownership context is closed and auditable | The declarative arm enumerated let/set/call/construct/return/give/fill/replacement/match/try contexts and required copy-provenance artifacts. | Any future Copy-like candidate must regenerate a complete current-language context table and fail Lock A on a newly discovered omitted context. |
| Complete-row copies are separately attributed | Semantic sites were to map to scalarized operations, aggregate loads, `memcpy`, `byval`, `sret`, spills, and eliminated copies. | Retain as structural accounting for every candidate that can duplicate or relocate records. `indeterminate` earns no credit. |
| Fixed-builder state ordering | Push wrote payload before increasing initialized count; finish required exact fill; open-scope normal exits raw-freed once. | Positive state-machine evidence, but only for resource-free exact fill. General dense candidates add exact live-element destruction, recoverable failure, move-out, and abandonment validity. |
| No trap cleanup | E0.1 followed EFF-4. | Retain `EX-ABORT`: abort owes no post-state/drop, but no invalid read may occur before abort. |
| Normal exits are broader than return | Hostile review added fallthrough, return, break, give, and `try` propagation. | Retain `EX-NORMAL` globally and instantiate every applicable control edge. |
| Builder assignment/transfer hazards | Review forbade destructive overwrite, unrestricted moves, aggregation, and escaping signatures for the confined prototype. | These restrictions are evidence that mere affinity is insufficient. General protocols must prove overwrite, transfer, and abandonment rather than hiding them as undocumented bans. |
| Index/type-flow seam | The prototype exposed an indexed-use ownership-checker bypass, repaired independently in current conformance. | Retain negative seam tests for every new place/state transition; production bug status is independent of candidate timing. |
| Recursive construction under ANF | Strict GRAM-9 repair required explicit inner bindings and moves. | Preserve grammar reality in all writer/source-shape comparisons. |

## 4. Layout and capacity arms

The old `PROTOCOL.md` matrix remains a source of representation and policy
controls, not an automatically active experiment.

| Historical arm | Retained question | Current disposition |
|---|---|---|
| `F-SOA` | Current fixed full-initialized SoA | B-FIX/B-P2 production baseline where applicable. |
| `F-SOA-P` | Coallocated column segments | Diagnostic composite only. It changes allocation, locality, alignment, pointer derivation, and alias facts and cannot isolate one cause. |
| `F-AOS` | Fixed full-initialized AoS | Eligible only after an ownership/initialization contract closes. It is a total representation effect, not a pure locality experiment. |
| `R-SOA` / `R-AOS` | Exact reserve then initialized prefix | Move into the dense-family candidate/policy space. Same element-capacity and failure policy are required for layout comparison. |
| `D-SOA` / `D-AOS` | Frozen growth policy | Move into the dense-family candidate/policy space. Capacity trajectory, allocation count, moved bytes, and allocator classes are part of the route. |

A future family lock need not retain these exact seven arms if the capability
census makes one invalid or introduces a required route. It must record the
disposition of each and preserve the causal warnings:

- AoS versus SoA changes check combination, alias facts, and layout as well as
  locality;
- full versus prefix owner changes header/API/lifetime/drop as well as
  initialization;
- reserve versus growth changes capacity trajectory and allocation traffic;
- independently grown SoA columns are not the same logical sequence contract;
  and
- no subtraction between composite arms may be called a one-variable effect.

## 5. Correctness, structural, and performance obligations

The following historical requirements remain mandatory schema inputs to the
applicable Family Lock A:

- full repository and compiler gates before and after each candidate step;
- field-by-field semantic comparison rather than raw record `memcmp` across
  padding;
- capacities/lengths at 0, 1, cap-1, cap, cap+1, repeated growth, overflow,
  object-size, layout, alignment, poison, and allocation-failure boundaries;
- rejection of borrow-then-grow, borrow-bearing payload outside its explicit
  scope, incompatible live borrows, and overlapping relocation;
- construction/move/replacement/drop/raw-free counters for affine payloads;
- unchanged primitive-buffer and SoA source, verdict, diagnostics, raw IR,
  optimized hot body, traps, alias metadata, vectorization, and call set;
- no whole-record traffic from field-only source;
- a proved no-grow append reduced to direct store plus length/state update and
  no hidden allocator slow path;
- requested/live/peak/transient bytes, allocation/free counts, initialized and
  moved bytes, final capacity/utilization, checks, branches, code size, stack,
  vector width, cache/TLB counters where reliable, and lifecycle timing;
- balanced randomized measurement, immutable raw samples, 99% confidence
  bounds, registered exclusion rules, and no discretionary outlier removal; and
- exact source/build/toolchain/target/allocator/corpus hashes.

The old numeric selection thresholds (`<= 0.90` time benefit, `<= 0.85` primary
memory or registered-maintenance benefit, plus non-inferiority guardrails) are
historical inputs, not G0-Core global law. The future Family Lock A must retain
or replace them before any candidate exists and explain the statistical and
product consequence.

## 6. Three decisions remain separate

1. **Capability adoption:** whether a checked ownership/storage transition
   deserves production existence for its frozen general contract.
2. **xlc migration:** whether Token/Ast tapes should change representation.
   Capability existence never implies migration. The historical migration gate
   required cold and retained compiler non-inferiority plus a material time,
   memory, or registered-edit gain.
3. **Default teaching:** which shape a benchmark-blind low-tier writer should
   choose. Expert feasibility and xlc migration do not establish the default.

Any later lock that merges these decisions reopens this reconciliation.

## 7. Explicitly withdrawn old narrowing

The historical protocol deferred arbitrary drop-bearing affine elements,
general move-out/take, broad partial initialization, and compositional builders
to keep one experiment small. Those deferrals cannot define the general dense
family. To close the dense contract, the new program must cover:

- arbitrary region-free, borrow-free affine payloads or state a narrower claim
  that does not pretend to be general;
- every selected dense coverage cluster's payload-scope classification and
  `scope_owner_contract_ids`, with each applicable overlay branch rebound to an
  exact Family Lock `member_contract_id` and `outcome_id`, its unit-specific
  base capability list, conditional delta, and effective ordered union;
- active extract/splice/filter `BR-STORED` state for live `RangeBounds` input
  where present and retained callable/replacement/cursor state; exact
  `BuildHasher`/caller-`Hasher` roles for set relations/algebra and trait
  branches; zero-or-more, zero-or-one, and exactly-once callable partitions;
  and the rule that only `VIEW-SORT-01` may retain a cached-key array;
- zero fields, metadata bytes, checks, branches, allocations, generated-code
  paths, payload traffic, or new fact dependencies from every excluded branch
  or unresolved delegated/boundary/frame state on the protected
  region-free/borrow-free shape, with all such states continuing to block
  unrestricted `E` or `P`;
- unknown-length append, pop, ordered/unordered remove, growth, relocation,
  truncate/clear, stable compaction, sorting, clone, and owning iteration;
- exact element destruction and partial-construction cleanup;
- recoverable failure semantics where the family freezes them;
- dynamic disjoint mutation and result reborrows; and
- ordinary-library derivation and H-FLATSET.

H-FLATSET is the dense-family held-out because its exact budget exercises
public `ST-AOS`, `ST-DENSE`, and dense ownership transitions without importing
`FAM-DENSE`. H-STORE requires public `ST-SPARSE`; it is explicitly deferred to
sparse-family closure after dense adoption. A dense lock must neither import
the unclosed sparse family nor use H-STORE as its G-14 closure witness.

An unguarded public `take(index)` remains rejected because it would leave a
writer-readable hole. A complete checked pop/remove/replace/Full-to-Vacant
transition is mandatory and is not the same operation.

## 8. Future lock checklist

Before dense Family Lock A may close, it must attach one disposition to each
section above and list:

- every retained arm under its new contract ID;
- every revised semantic or benchmark term;
- every superseded arm and why the new registry makes it unnecessary;
- every new mandatory operation absent from E0.1;
- the unchanged B-FIX/B-P2 controls;
- the exact relationship between fixed AoS ownership and general dense storage;
- the applicable W/H dependencies;
- H-FLATSET as the dense held-out and H-STORE as a later sparse-family held-out;
- the incremental META-5 ledger; and
- a fresh independent hostile review of the exact lock bytes.

Until then, E0.1 remains research history and no candidate route is authorized.
