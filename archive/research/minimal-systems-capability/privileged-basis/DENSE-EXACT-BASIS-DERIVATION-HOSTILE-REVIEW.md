# Dense Exact Basis Derivation Hostile Review

Status: **PASS for fail-closed reporting and verifier rejection behavior;
exact D-2 and exact P-1 remain PENDING**, 2026-07-15.

This review authenticates the exact dense authority and its refusal to grant
credit while source or schedule obligations remain unresolved. It does not
authenticate a D-2 or P-1 PASS, a production checker, a language change, a
candidate implementation, or performance.

## 1. Reviewed bytes

| Artifact | SHA-256 |
|---|---|
| `DENSE-EXACT-BASIS-DERIVATION-REPORT.md` | `dab4e72f4b115f76eb7f1ff8e7981f8545bade96d68041385ed423a396de8164` |
| `DENSE-OUTCOME-ROUTE-AUTHORITY.tsv` | `e952e286f7ffc5ee0bd115a32d962dc1fb12f172050769d09540070fca277889` |
| `DENSE-OUTCOME-ROUTE-PREDICATE-AUTHORITY.tsv` | `6c23914cdf58e3a5659acd37ad0d509b5b86eedd821474790a9571fd1472988e` |
| `DENSE-EXACT-ROUTE-EVIDENCE-AUTHORITY.tsv` | `fbeb735a942a0a19e4869e1ba3fb75bd7446c5a1d4502c5ad51d6ea0e767d393` |
| `DENSE-CHOICE-RESOLUTION-AUTHORITY.tsv` | `b0fb9b3818f5037d5cad858eb66d1f1b5329e04539e5c328fc755a9bca339a4f` |
| `DENSE-EXACT-ROUTE-MULTIPLICITY-AUTHORITY.tsv` | `d40cde6c75682c0750f94f951c78294ba618d38aaccb48ade339287acb0cf081` |
| `build_dense_outcome_route_authority.py` | `bb9150f5dbd8b75492a3566378110a342516089f746e480af0289fa8b42346c3` |
| `exact_basis_derivation.py` | `7d4ee91b81f565c1abc4b1d5f210e8071e3d8785473bd073676b460970500471` |
| `BASIS-DERIVATION-MANIFEST.tsv` | `5a8e697e4c1df54b2362cbf08883a20d9daebdd53ee4915867a152542d757aa0` |

## 2. Revoked apparent PASS

The first exact ledger was not reviewable evidence. Its expected relation
omitted 4,198 of the 4,242 outcome/route cells. The incomplete expected side
then allowed 15,314 positive credits to appear without an independent
obligation. A file hash could authenticate those bytes but could not make the
relation complete.

The apparent D-2 PASS was revoked before this review. The repaired authority is
the full Cartesian complement of 303 outcomes and 14 critical routes. The
derivation generator cannot author or infer the expected disposition that it
later checks.

## 3. Hostile semantic attacks and repairs

### 3.1 Empty and no-op traces

The review enumerated legal zero-length, identical-place, empty-suffix,
no-growth, and identity-permutation traces. It found false unconditional Copy,
partition-borrow, TakePut, Swap, and ExactFocus credit. The final authority uses
independent input or state predicates and forbids each route on the no-action
branch.

Confirmed boundary cases include:

- Drop on an empty owner still requires terminal ExactFocus;
- Clear `EMPTY_NO_CHANGE` requires no open focus;
- FillClone can abort while disposing an unused seed at length zero and forms
  no target/source child loan; and
- root-changing reserve, shrink, and boxed relocation retain ExactFocus even
  when the logical payload has zero physical bytes.

### 3.2 Copy, move, replace, and owner destination

The review rejected source-preserving Copy as relocation, removed false outer
Replace credit from CloneFrom and FillClone-abort, and repaired compacting
operations that return or retain owners at new logical places. The final
authority distinguishes payload-byte traffic from affine-owner traffic and
does not use disappearance from BASE as evidence that an owner was destroyed.

### 3.3 Borrow and callable routes

The source-derived InitClone, InitCopy, ResizeClone, and FillClone loan routes
are explicit. ResizeClone behavior-abort distinguishes a source-loaning Clone
from a nonloaning disposer. Cached-key behavior-abort distinguishes extraction
from later comparison. No zero-call outcome receives unconditional child-loan
credit.

### 3.4 Circular conditional predicates

Predicates defined by the action being credited were rejected. Independent
runtime guards name input values or selected semantic state. Seven remaining
algorithm-trace predicates are explicitly marked `TRACE_CLASSIFIER_ONLY`.
They receive no D-2 action credit, no P-1 structural credit, and no cost credit.

### 3.5 Rotate and sorting

The all-Swap Rotate choice was rejected because it neither represented the
selected algorithm nor proved optimal dispatch. The final D-2 witness is a
generic GCD one-temporary TakePut route, conditional on a nonidentity rotation.
Stack-buffer, block-swap, and data-dependent dispatch remain P-1 evidence debt.

Stable sorting retains only an expressible paper merge route. Cached-key sorting
does not inherit an unproved stable-sort scratch schedule. Exact event traces,
thresholds, allocation multiplicity, comparator order, and same-contract cost
remain pending.

### 3.6 ZST, fullness, and Convert

The review rejected implicit reuse of positive-size full-storage reasoning for
zero-sized logical-capacity changes. IntoOwner and IntoBoxed remain blocked
until their positive-size and ZST subcontracts are split. Coarse Convert
outcomes remain blocked until direction and callable contracts are split.

### 3.7 Carrier multiplicity

A Boolean route cell cannot distinguish one acquisition from two. EagerExtract
and EagerSplice success outcomes now have five exact multiplicity authority
rows. The two growing Splice outcomes require at least two carriers, and that
lower bound propagates into structural-cost contexts without measured credit.

## 4. Final exact status

The verifier reconstructs:

- 8,075 activated required obligations with none missing;
- 35,021 forbidden obligations with no violation;
- 340 unresolved required obligations across 150 contexts;
- 1,253 conditional availability obligations with no observed action credit;
- zero semantic route gaps and zero basis contradictions; and
- zero mechanized trace, mechanized cost, or measurement results.

The 340 unresolved obligations are 168 coarse Convert routes, 24 Convert
callable routes, 136 allocator-applicability obligations, six IntoOwner
ZST/capacity-reshape obligations, and six IntoBoxed fullness/ZST obligations.
This exact partition is a PENDING result, not a classified PASS.

## 5. Independent reproduction and mutation results

`build_dense_outcome_route_authority.py verify` reproduces the route,
predicate, evidence, and empty-choice authorities byte-for-byte without
importing the derivation generator.

Two deterministic `exact_basis_derivation.py verify` runs agree on manifest
SHA-256
`5a8e697e4c1df54b2362cbf08883a20d9daebdd53ee4915867a152542d757aa0`.
The hostile command rejects all thirteen mutations, including deletion of a
multiplicity row and a hash-recomputed lowering from two carriers to one.

## 6. Verdict

The exact packet now says only what its evidence supports. The proposed basis
has no demonstrated contradiction in the closed portion of the dense ledger,
but exact D-2 and P-1 remain pending on named source and schedule obligations.
No unresolved or trace-classifier row contributes positive action or cost
credit. The combined D14 completion lock remains open.
