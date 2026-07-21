# FN-4 hostile fact-channel review

Date: 2026-07-21

Verdict: **GO for owner-approval presentation**

Authority: proposal-only review evidence. This is not specification approval,
protected-surface approval, implementation authorization, optimizer-fact
authorization, or a release claim.

Reviewed candidate:

- path: `grammar-verifier/proposal/kernel-spec-successor-candidate.md`
- byte count: `98044`
- SHA-256: `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`

This verdict is valid only for those exact bytes.

## Soundness result

The closed `iadd.sat` table is mathematically sound. For unsigned T with
`M = 2^K - 1`, `f(x, y) = min(M, x + y)`. Both associations equal
`min(M, x + y + z)`, integer addition is commutative, zero is a two-sided
identity, and every result remains in T. For signed T, clamping mathematical
integer addition to `[MIN, MAX]` is total, commutative, and has zero identity.
It is not associative: `(MAX sat+ 1) sat+ -1` is `MAX - 1`, while
`MAX sat+ (1 sat+ -1)` is `MAX`, for every listed signed width. The table's
same-integer-value equivalence avoids representation, float, NaN, and
signed-zero ambiguity.

The admission calculus is deterministic across compilers. It requires one
exact contract, member, binding, top-level function, concrete integer domain,
ordinal signature match, and one exact total primitive-wrapper body. Contract
and function generics, region parameters, `requires`, hidden effects, altered
argument order, extra statements, and semantically equivalent but unadmitted
bodies cannot enter the relation. A literal identity is exact FORM-5 zero; a
named identity is an earlier same-typed CONST-2 value whose closed substitution
is zero. Compiler-specific prover strength cannot enlarge source acceptance.

The rule is a capability-family judgment, not function-specific dispatch: it
applies to arbitrary legal declaration, member, binding, function, and
parameter names; all eight integer widths; all three law kinds; and direct or
named-constant zero identities. Other operation rows and proof calculi remain
future specification changes.

## Authority separation

Each discharged `(contract-law node, concrete-conformance node)` pair has one
base derivation record. Same-kernel artifact replay recomputes source
acceptance but grants no lowering consequence. Optional optimizer use remains
behind a separately approved verifier that independently rederives the complete
source relation from the accepted artifact, or validates an exact gated-ledger
entry and scope, and binds the exact proposition and consumer to the accepted
artifact, target, and backend. A producer record or replay verdict is not
optimizer authority. Missing, rejected, or resource-limited optional evidence
cannot change source acceptance, semantic identity, explicit checks, or the
canonical empty-overlay result.

FN-4 decides only one referenced law obligation. It does not decide whole
FN-3 conformance acceptance, uncovered members or bindings, generic contract
substitution, or behavior-parameterized calls.

## Protected-outcome preservation

No protected file was modified. The separately proposed FORM-2 migration
changes only layout for these cases: the independent report gives identical
terminal projections before and after each migration. The frozen rule preserves
all five outcomes:

| case | current protected source SHA-256 | proposed canonical source SHA-256 | preserved outcome |
|---|---|---|---|
| `fn4-pos-law-in-contract` | `9c217084c9fad2d5c6152c8fb200c756f98c5a54a87f3c066fcc3ba0b1091d53` | `58bdefbfa7bd6b76019df7dac090bc07f5e2a02d1543aed87f002fdddb78fb4b` | accept: declaration-only obligation emits no accepted-law evidence |
| `fn4-neg-bad-lawname` | `2aa2af6ff1050334433b546210e0c687dca223328239428f213d34765b8dcfbe` | `4506d9e3838e1684b0c29201a2bbfda29957ce22db2561609a5cb6dca7407655` | reject `FN-4`: unknown declaration-table row |
| `fn4-pos-law-discharged` | `872b891f73e23575f3cf9bfc00f07e54305a2468072b7a0be4068725257853b8` | `264ec02be1e00a8682a8792b950b63d78364d924d7398e3a0c55b5579a28489e` | accept: unsigned single `iadd.sat` discharges associative, commutative, and zero identity |
| `fn4-neg-law-refuted-signedness` | `e8877dff9e645d2ed56aa23d1c0bde2fa018c0249689ea65ffcd55a5847b10a1` | `9ff762227200a1c32c12ade99c271664bac63a0613494543eff606aa62f5b5c1` | reject `FN-4`: signed associativity is refuted |
| `fn4-neg-law-undischarged` | `5959185a75c17ac79c5fb336cb728873b6db8bd83b7174ae05bfd05460fb851a` | `faaea94e907a233d6079fe5fadf3ca4409c7e94991f80ee39dcbba95981a47f4` | reject `FN-4`: two-step body has no admitted derivation, even though a stronger prover could establish its equation |

## Scope conclusion

No fact-forgery path, optional-proof-dependent source verdict, generic escape,
whole-conformance overclaim, or false law-table cell remains in the reviewed
bytes. Installing these specification bytes would still not authorize the
semantic kernel, artifact schema, replay implementation, optimizer verifier,
consumer transformation, or backend. Those remain separate `THE-PLAN.md`
gates and owner approvals.
