# Dense Exact Basis Derivation Report

Date: 2026-07-15

Status: fail-closed D14 research evidence pending owner review. Exact dense D-2
and exact P-1 are `PENDING`. Candidate construction, scored or held-out
execution, production implementation, specification or compiler changes,
performance claims, and default teaching remain unauthorized.

## 1. Verdict

The proposed certified-resource architecture has no demonstrated route
contradiction in the frozen dense-family ledger, but the exact ledger is not
closed. Its final machine-checked status is:

`PENDING_EXPLICIT_OUTCOME_AUTHORITY_BLOCKERS_NO_POSITIVE_CREDIT`

The exact population is:

- 303 source outcomes across 93 members;
- 1,773 payload, size, stored-borrow, and policy contexts;
- 44,689 independently checked context-to-route obligations;
- 8,075 required route obligations, all activated;
- 35,021 forbidden route obligations, with zero violations;
- 340 unresolved required obligations across 150 contexts, with no positive
  route or cost credit;
- 1,253 conditional route-availability obligations whose runtime action is
  unobserved, including 908 semantic runtime guards and 345 trace classifiers;
- zero semantic route-gap contexts and zero basis contradictions;
- zero mechanized linear-resource traces;
- zero mechanized cost-algebra proofs; and
- zero measured performance results.

Consequently, the evidence supports continued study of the proposed basis but
does not prove that every exact dense contract derives through it, that no
additional public authority will be needed, or that every protected contract
has same-representation structural parity.

The deterministic manifest SHA-256 is
`5a8e697e4c1df54b2362cbf08883a20d9daebdd53ee4915867a152542d757aa0`.
Its authenticated row SHA-256 is
`fcee68706519d2e4eb1d60522f6f9d1996979629d01b602ba667e9582b325933`.

## 2. Exact unresolved partition

The 340 unresolved obligations have a disjoint explanation:

| Obligation class | Count | Why it is unresolved | Smallest repair |
|---|---:|---|---|
| Coarse Convert route direction | 168 | One outcome combines owned, borrowed, moving, copying, cloning, allocating, and reusing directions. | Split direction-specific subcontracts. |
| Convert callable direction | 24 | The coarse result does not select the exact callable or source-loan contract. | Bind callable and source/result ownership in the same direction split. |
| Outer allocator applicability | 136 | The source does not freeze the exact acquisition point, zero-allocation arm, abort prefix, or failure disposition. | Freeze an outcome-level allocator event schedule. |
| IntoOwner ZST/capacity reshape | 6 | Positive-size full-storage repackaging and zero-size logical-capacity reshape are conflated. | Split positive-size and included-ZST subcontracts. |
| IntoBoxed no-change fullness/ZST | 6 | `SUCCESS_NO_CHANGE` does not prove positive-size fullness and also conflates ZST logical capacity. | Require exact fullness or select relocation, and split the ZST arm. |

Sixteen allocator obligations also belong to Convert outcomes. Therefore 208
obligations are Convert-implicated and 136 are allocator-implicated, but those
overlapping labels must not be added. The disjoint table above sums to 340.

Every exact outcome containing an unresolved authority row now carries
`PENDING_EXACT_DIRECTION_NO_D2_OR_P1_CREDIT` in both route and structural-cost
summary fields. Classification is not treated as satisfaction.

## 3. Frozen exact authorities

The derivation generator consumes five independently authored authorities. It
does not generate its own expected route relation.

| Authority | Rows | SHA-256 | Role |
|---|---:|---|---|
| `DENSE-OUTCOME-ROUTE-AUTHORITY.tsv` | 4,242 | `e952e286f7ffc5ee0bd115a32d962dc1fb12f172050769d09540070fca277889` | Full 303 by 14 route complement: 366 required, 3,633 forbidden, 225 conditional, zero choices, and 18 unresolved cells. |
| `DENSE-OUTCOME-ROUTE-PREDICATE-AUTHORITY.tsv` | 23 | `6c23914cdf58e3a5659acd37ad0d509b5b86eedd821474790a9571fd1472988e` | Exact frozen guard kind, fields, expression, and true/false dispositions. |
| `DENSE-EXACT-ROUTE-EVIDENCE-AUTHORITY.tsv` | 204 | `fbeb735a942a0a19e4869e1ba3fb75bd7446c5a1d4502c5ad51d6ea0e767d393` | Checked-arithmetic and exact-focus obligation sequences. |
| `DENSE-CHOICE-RESOLUTION-AUTHORITY.tsv` | 0 | `b0fb9b3818f5037d5cad858eb66d1f1b5329e04539e5c328fc755a9bca339a4f` | Header-only proof that no unresolved route choice receives activation credit. |
| `DENSE-EXACT-ROUTE-MULTIPLICITY-AUTHORITY.tsv` | 5 | `d40cde6c75682c0750f94f951c78294ba618d38aaccb48ade339287acb0cf081` | Structural lower bounds for distinct returned and replacement carriers. |

The independent authority builder is
`build_dense_outcome_route_authority.py`, SHA-256
`bb9150f5dbd8b75492a3566378110a342516089f746e480af0289fa8b42346c3`.
It reads only the frozen exact outcome registry, does not import the derivation
generator or member classifier, and reproduces the first four authorities
byte-for-byte.

## 4. Multiplicity cannot collapse to a Boolean

A route-presence bit is insufficient when one outcome needs two distinct
carrier acquisitions. The five multiplicity rows freeze:

- EagerExtract `SUCCESS`: at least one removed-output carrier;
- EagerSplice `SUCCESS_NO_GROW`: at least one removed-output carrier under each
  OD-1 policy; and
- EagerSplice `SUCCESS_GROW`: at least two carriers under each OD-1 policy, one
  for replacement BASE storage and one for the removed-output owner.

These bounds propagate into 39 admitted structural-cost contexts. They are
unmeasured structural lower bounds and grant zero P-1 credit. Deleting a row or
changing a `2` to `1`, even while recomputing the row hash, is rejected.

## 5. Hostile-review corrections

The first apparent PASS was invalid. It omitted 4,198 independent
context/route cells and allowed thousands of positive credits to flow from an
incomplete expected relation. The repaired ledger makes the expected side a
complete 303 by 14 authority and then applies the exact context quotient.

Subsequent hostile passes corrected the following semantic classes:

- zero-length and no-op branches no longer claim Copy, swap, partition-borrow,
  take/put, focus, or allocation actions that never execute;
- source-preserving `CoreCopy` is distinct from relocation and is conditional
  on a nonempty target range;
- CloneFrom and FillClone-abort no longer receive a false outer Replace action;
- InitClone, InitCopy, ResizeClone, and FillClone retain their exact source and
  target loan conditions;
- root-changing reserve, shrink, and boxed-relocation outcomes keep ExactFocus
  even when physical payload length is zero;
- retained or returned affine owners moved to a different logical place keep
  TakePut credit even when the source owner disappears from the final BASE;
- Rotate uses a generic GCD one-temporary take/put witness for D-2
  expressibility, while its best dispatch remains a P-1 blocker;
- stable-sort predicates that describe an already selected trace are marked
  `TRACE_CLASSIFIER_ONLY` and receive no action or cost credit;
- cached-key sorting remains pending rather than inheriting a false stable-sort
  scratch schedule;
- IntoOwner and IntoBoxed ZST/full-storage conflations are explicit blockers;
  and
- returned-output carrier multiplicity is preserved rather than Booleanized.

## 6. Generated evidence relations

`exact_basis_derivation.py` deterministically builds and verifies:

| Relation | Rows | Purpose |
|---|---:|---|
| `DENSE-MEMBER-ACTION-CLASS.tsv` | 93 | Generated member repertoire, explicitly not outcome invocation. |
| `DENSE-EXACT-BASIS-DERIVATION.tsv` | 303 | Exact source outcomes, authority summaries, and blockers. |
| `DENSE-BASIS-BRANCH-CONTEXT.tsv` | 1,773 | Exact product over payload, size, borrow, OD-3, and applicable OD-4 context. |
| `DENSE-CONTEXT-ROUTE-OBLIGATION.tsv` | 44,689 | Independent required, forbidden, conditional, and unresolved route expectations. |
| `DENSE-MEMBER-BASIS-ACTION-STEP.tsv` | 39,265 | Per-context route applicability; not a simulated linear trace. |
| `DENSE-BASIS-CAPABILITY-DISCHARGE.tsv` | 59,700 | Source-origin retention, alias expansion, and authority-constrained credit. |
| `DENSE-BASIS-STRUCTURAL-COST-EVIDENCE.tsv` | 1,773 | Source ceilings, route multiplicity, blockers, and unmeasured status. |
| `BASIS-DERIVATION-MANIFEST.tsv` | 1 | Counts, hashes, authorization state, and fail-closed D-2 status. |

Legacy crosswalk diagnostics remain visible: 109 member-repertoire omissions,
4,466 canonical capability-route gaps, 448 legacy required-binding omissions,
and 925 legacy-only capability pairs. They are source-taxonomy diagnostics, not
positive route evidence and not substitutes for the exact outcome authority.

## 7. Verification and hostile mutations

The final verification sequence is:

```text
python3 build_dense_outcome_route_authority.py verify
python3 exact_basis_derivation.py build
python3 exact_basis_derivation.py verify
python3 exact_basis_derivation.py verify
python3 exact_basis_derivation.py hostile
```

The two deterministic verification passes produce the same manifest hash. The
hostile command rejects thirteen in-memory semantic mutations, including:

- deletion and hash-recomputed lowering of route multiplicity;
- missing unknown-length blocker action or obligation;
- reversed reserve-first discipline;
- missing fact establishment, disposition, producer callable, or premise
  authority;
- a false InitClone allocator route;
- activation in an OD-3 rejected context;
- collapsed positive-size and zero-size contexts; and
- positive capability credit in a rejected context.

## 8. Honest boundary and next research action

The verifier establishes exact joins, complete authority coverage,
fail-closed classification, relation consistency, and absence of unauthorized
performance credit. It does not simulate every resource microstep, mechanize
the cost algebra, observe runtime conditional branches, or measure performance.

Exact D-2 and exact P-1 therefore remain pending. The smallest next research
action is a bounded source-normalization and reference-trace pass: split the
coarse Convert and ZST/fullness contracts, freeze exact allocator schedules,
and reconstruct the same-contract Rotate and stable/cached-key sort schedules.
That pass should return only genuine semantic choices for owner review. It does
not require or authorize a language change, basis change, compiler change,
container implementation, or benchmark unless separately approved.
