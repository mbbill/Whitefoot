# Post-FORM-2 case-intent hostile review

Date: 2026-07-21

Verdict: **GO for owner-approval presentation**

Authority: proposal-only review evidence. This is not specification approval,
protected-surface approval, implementation authorization, optimizer authority,
or a release claim.

Reviewed candidate:

- path: `grammar-verifier/proposal/kernel-spec-successor-candidate.md`
- byte count: `98044`
- SHA-256: `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`

Reviewed patches, in mandatory order:

1. `grammar-verifier/evidence/form2-structural-migration.patch`, SHA-256
   `4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2`
2. `grammar-verifier/evidence/v0.9-post-form2-case-intent.patch`, SHA-256
   `62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a`
3. `grammar-verifier/evidence/v0.9-manifest-metadata.patch`, SHA-256
   `ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6`

This verdict is valid only for those exact bytes and that exact order.

## Ordinary composition

The three patches were checked and applied with ordinary `git apply`, in order,
to an isolated copy of the current protected conformance tree. No three-way
merge, reject recovery, patch editing, or manual repair was needed. Patch A has
274 paths, patch B has six paths, and patch C has one path. Patch A does not
edit the manifest.

| stage | manifest byte count | manifest SHA-256 |
|---|---:|---|
| current protected tree | `99776` | `20bb50032c112150c3d9a7387a17bde708922e426550b47b64f2214cd7341d69` |
| after A | `99776` | `20bb50032c112150c3d9a7387a17bde708922e426550b47b64f2214cd7341d69` |
| after A then B | `99869` | `e0e3138869c337c47f2c527bda359fef1108ca1483b8a3e3f22cb86140581c3f` |
| after A then B then C | `99869` | `0eff27bfb87ca14086f31f4b171d72c9eb1a49072aa4563a3f7c937d0b8bb90c` |

No live protected file was modified during this review.

## Five source postimages and outcomes

Both the generalized candidate parser and the separately implemented FORM-2
parser find one derivation for each final source and render it back to the exact
same bytes. The case-intent patch changes only documentation in four sources;
its one executable-shape change is the FN-8 hostile case.

| case | SHA-256 after A | final SHA-256 after A+B+C | reviewed outcome |
|---|---|---|---|
| `fn4-pos-law-discharged` | `264ec02be1e00a8682a8792b950b63d78364d924d7398e3a0c55b5579a28489e` | `9cd070cd331b163f0f230c8c57ee7c38f0d7aa23a6807987981bc29ee13c0418` | accept; exact unsigned `iadd.sat<u64>` wrapper discharges all three stated laws |
| `fn4-pos-law-in-contract` | `58bdefbfa7bd6b76019df7dac090bc07f5e2a02d1543aed87f002fdddb78fb4b` | `66f30c62380f95a332a00bd468ae9505307c87ca77db3c62dcb13f1e767b7d0d` | accept; a declaration-only law creates no accepted-law base record |
| `fn8-neg-requires-control` | `a0e5c4f6844f295515325070c21e29ec05b5be1c9c847e5ac785992c3ef7c1a0` | `00a2b65bbfd272897a2b0596123c32e0069306c680cb77c4f7f229337c25202f` | reject `FN-8` at the nonfinal `return` |
| `gram1-pos-lookahead` | `e536ee58cac178d0cd5a1b06ec91ab25d73782ff171db6b6947d9ff055bf0af5` | `3b146c7ac6185b12e5e703a4643cf0afd3c8b4f05ccc56fdd6ef5d6a07b71b18` | run to exit 0; documentation now describes the actual grammar forks |
| `gram7-pos-two-productions` | `7daf418c6a21a7a08dd6605469c44db43f8a22c270d4ae479077639427649353` | `a1c1986fedbbc00c0756986582dccebe07b7aad013258ccb4936a40dc5d6e43e` | run to exit 0; documentation now records two distinct node kinds |

### FN-8 first-violation audit

The final direct `requires_entry` children are, from left to right:

1. an ordinary-let statement,
2. a return statement,
3. a check statement.

FN-8 requires every entry before the final position to be an ordinary let and
the final entry to be a check. It also requires the structural pass to run
before recursive child checking and to report the first invalid direct entry.
Entry 1 passes. Entry 2 is therefore the first violation and is rejected at its
`return_stmt` node. The later final check prevents a missing-final-check result,
but it cannot make entry 2 valid or outrank the earlier violation. No child
semantic diagnostic can win before FN-8.

## Complete FN-4 protected set

The final composed tree preserves all five protected FN-4 outcomes:

| case | final source SHA-256 | preserved outcome |
|---|---|---|
| `fn4-pos-law-in-contract` | `66f30c62380f95a332a00bd468ae9505307c87ca77db3c62dcb13f1e767b7d0d` | accept; no concrete conformance, hence no accepted-law base record |
| `fn4-neg-bad-lawname` | `4506d9e3838e1684b0c29201a2bbfda29957ce22db2561609a5cb6dca7407655` | reject `FN-4`; `distributive` is not a row in the closed declaration table |
| `fn4-pos-law-discharged` | `9cd070cd331b163f0f230c8c57ee7c38f0d7aa23a6807987981bc29ee13c0418` | accept; the exact wrapper and unsigned table cells emit exactly three base records |
| `fn4-neg-law-refuted-signedness` | `9ff762227200a1c32c12ade99c271664bac63a0613494543eff606aa62f5b5c1` | reject `FN-4`; signed saturating addition is not associative |
| `fn4-neg-law-undischarged` | `faaea94e907a233d6079fe5fadf3ca4409c7e94991f80ee39dcbba95981a47f4` | reject `FN-4`; the two-step body is outside the exact admitted wrapper relation |

The two positive-source documentation edits do not change contracts, laws,
signatures, conform bindings, function bodies, or identities. The other three
FN-4 sources receive only the FORM-2 structural migration.

## Manifest audit

Patch B changes prose in exactly nine manifest records:

| record | changed prose field or fields |
|---|---|
| `form2-neg-noncanonical-ws` | `doc` |
| `gram1-pos-lookahead` | `doc` |
| `gram2-pos-items` | `reason` |
| `gram3-pos-modes` | `reason` |
| `gram4-pos-stmts` | `reason` |
| `gram7-pos-two-productions` | `doc`, `reason` |
| `fn4-pos-law-in-contract` | `doc` |
| `fn4-neg-bad-lawname` | `doc` |
| `fn4-pos-law-discharged` | `doc` |

Patch C changes only the `reason` prose of the `META-1`, `META-3`, and
`META-4` coverage annotations, replacing stale v0.8 references with v0.9
references. Across A, B, and C, a structural comparison finds no change to any
record's `rules`, `expect`, `status`, or `covered_by` projection. IDs and record
order are also unchanged. Thus the patches update stated intent and metadata;
they do not change a protected verdict, rule citation, execution status, or
coverage assignment.

## Optimizer-authority audit

None of these patches creates optimizer authority. Source `doc` text and
manifest `doc` or `reason` text are not fact channels. The FN-4 executable
shapes, closed table, mandatory source-admission relation, and base-record
cardinality remain exactly those reviewed in the candidate. A same-kernel base
record still has no lowering consequence. Any optimizer use still requires the
candidate's separately approved independent verifier, exact proposition and
consumer binding, and accepted-artifact, target, and backend binding. Failure
or absence of that optional path leaves the canonical empty-overlay lowering
unchanged.

## Scope conclusion

No composition failure, case-intent reversal, FN-8 diagnostic-order ambiguity,
FN-4 protected-outcome regression, manifest-verdict change, or optimizer-fact
forgery path remains in the reviewed bytes. The exact candidate and exact
A-then-B-then-C patch sequence are a **GO for owner-approval presentation**.
Applying any patch to protected material, installing the successor
specification, or authorizing compiler work remains a separate owner-gated
action under `THE-PLAN.md`.
