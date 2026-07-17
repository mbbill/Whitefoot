# Privileged Gate Admission Pareto Hostile Review

Date: 2026-07-15

Status: PASS for the conditional research-level authentication architecture and
F/C/S Pareto comparison. This PASS does not select F or C, prove external-frame
template coverage, close public-basis derivability, approve the required V2
completion lock, or authorize any language, specification, compiler, verifier,
cryptographic deployment, capability entry, standard library, container,
benchmark, or production change.

## 1. Reviewed artifacts

| Artifact | SHA-256 |
|---|---|
| `GATE-ADMISSION-PARETO-DECISION.md` | `004318ed7acee04e77dece09c4fc361fa4b19c0c447a52ac5636b553e2aaa74c` |
| `GATE-ADMISSION-PARETO-DIMENSIONS.tsv` | `492444e5d2421159019dcb2a370e6902bc9cc175b7b490b9a26f7387507834b7` |
| `GATE-ADMISSION-PARETO-MATRIX.tsv` | `7e43cc84106f4b35197adf57b35941b5f6fb8891aa052c22d449c60b48e2922b` |
| `verify_gate_admission_pareto.py` | `6da14a8fd5d403fd194ce6b804235434ced3eb5e3650120e9017dda88baa0a1e` |
| `test_verify_gate_admission_pareto.py` | `38b30be2c97ff93490a4307b42b0b17e4e3ac88c57bc48e31b2bdbb1180a0580` |

Three independent attacks reconstructed the authentication boundary, public-
basis privilege cut, repository-law consequences, and matrix selection rather
than accepting the report's conclusion. The first draft failed. This PASS
applies only to the repaired bytes above.

## 2. Blocking attacks repaired before PASS

### 2.1 Circular identities

Attack: the original root contained grants that bound the root, the entry
identity depended on the grant that signed it, and the signature appeared
inside the payload it authenticated.

Repair: the final DAG separates semantic policy, signer policy, approval
policy, state-transition policy, admission mode, toolchain implementation,
frame implementation commitment, frame entry, approval and activation evidence,
consumer cone, derived use, authorization body, signed request, release entry,
grant envelope, snapshot, protected current state, and toolchain release.
Every edge points forward. A semantic-policy change starts a new approval
lineage.

### 2.2 Unfair fixed-release candidate

Attack: F was initially charged the extension-signature verifier while the
matrix credited it with no such TCB.

Repair: both F and C/S derive the same `EntryApprovalBodyID`. F authorizes it by
exact `BaseEntrySetID` membership. C/S may additionally authorize it through an
exact signed envelope. `AdmissionModeID` fixes which typed witness is legal
inside one `PermitUse` relation.

### 2.3 Missing state discriminator

Attack: the original predicate could not distinguish F, C, and S, so F accepted
signed extensions and S could not enforce current membership.

Repair: the toolchain release binds `FixedRelease`, `StatelessExtensions`, or
`StatefulExtensions`. Stateful use binds a valid successor snapshot, current
signer policy, protected epoch, active grant membership, and cumulative
tombstones. Release-entry-only artifacts do not depend on unrelated snapshot
state.

### 2.4 Key rotation and migration

Attack: S claimed current-key rotation while every grant was permanently bound
to initial approval keys. C migration could also have inherited transitive or
cross-semantic trust.

Repair: each envelope binds an exact `GrantSignerPolicyID`; S snapshots bind
and validly transition the current signer. C migration names exact envelopes
from an explicit strict-ancestor chain with the identical `SemanticPolicyID`.
There is no wildcard or implicit transitive inheritance. Compromise recovery
does not inherit arbitrary old signatures.

### 2.5 Stage/final-image contradiction

Attack: one predicate required final provider resolution during source
compilation, making separate compilation impossible or treating a pre-link
commitment as final.

Repair: the single relation is stage-indexed. Compile/import produce
non-executable pending uses, link binds static resolution, and load/JIT alone
activate exact loaded bytes or actual policy-validated providers. Address-
sensitive mappings bind actual address, permissions, and lease state without
claiming a reproducible activation record.

### 2.6 Public-scope and implementation-cone gaps

Attack: the signed request omitted the approved provider cone, and a public
grant was required to match the original use-specific consumer cone.

Repair: `FrameImplementationCommitmentID` signs an exact provider cone or exact
provider-validation policy/evidence schema. `GrantApprovalRequestID` is
grant-level; `DerivedUseID` is per use. `ExactCone` signs byte equality and
`Public` signs one explicit public marker.

### 2.7 Approval/activation evidence conflation

Attack: a provider-policy grant purported to sign evidence that does not exist
until dynamic activation.

Repair: `ApprovalEvidenceID` contains review-time evidence; actual provider
identity, closure, and validation reside in `ActivationProviderEvidenceID` and
the stage receipt.

### 2.8 High-level privilege laundering

Attack: the first draft allowed a certified high-level helper, container,
allocator policy, or serializer to become privileged, recreating a hidden
standard library.

Repair: every privileged witness instantiates a root-defined, bounded,
no-formula external-frame template. Parameters cannot contain predicates,
proof identities, formula hashes, bytecode, semantic callbacks, or arbitrary
contracts. Any artifact completely proved through public semantics is ordinary.
Any privileged high-level operation receives zero D-2 and P-1 credit,
regardless of whether its witness is a release entry or extension grant.

### 2.9 Content/authorization and dynamic-provider conflation

Attack: review, scope, signature, snapshot, and provider-policy changes altered
the claimed executable content identity, while dynamic providers were
incorrectly called byte-reproducible.

Repair: `BuildContentID` binds semantic and byte closure. `StageReceiptID` binds
authorization witnesses and the exact state actually consumed. Content may be
reused across an authorization change, but a new receipt is required when its
release, policy, grant, or relevant snapshot authority changes. Provider-policy
builds claim no loaded-provider reproducibility until actual provider bytes are
pinned.

### 2.10 Incomplete and mutable matrix

Attack: the first table had shifted TSV columns, omitted key, cache, migration,
supply-chain, stage, loader, TCB, and invocation dimensions, and could accept
arbitrary cell mutations or multiple recommended candidates.

Repair: the dimension manifest and matrix contain 42 exact rows. The verifier
pins the complete raw-byte SHA-256 of both tables, requires exact schema and
coverage, and enforces exactly one recommendation in each conditional branch.
The hostile test proves rejection of four mutations: arbitrary required-cell
change, second recommended candidate, coordinated dimension replacement, and
shifted columns.

## 3. Pareto verdict

The hard set consists of every `REQUIRED` row plus the owner-selected branch of
`OWNER_CHOICE`. `NOT_REQUIRED` properties receive no selection credit. Every
candidate preserves approved authorization bodies across its key-rotation
route, so the `DESIRABLE` row does not separate C and S.

S's unique capability is local same-policy extension-grant currentness after a
valid successor is observed. That property is not required. S otherwise adds a
state-transition verifier, protected mutable state, atomic persistence, update
distribution, and stateful grant-use verification. S is therefore eliminated
under the current hard set without denying that its unique property is real.

The remaining choice is exact:

- if a new approved instance of an existing frame template must be usable
  without an authorization-release update, C is recommended; or
- if an immutable authorization-release update per instance is acceptable, F
  is recommended because it omits the extension-signature path.

This is a conditional recommendation, not an owner decision.

## 4. Deliberately open gates

Authentication PASS cannot establish that the gate is sufficient. The
following remain fail-closed:

1. a concrete bounded external-frame template registry and constructive D6
   coverage;
2. an exact minimal privilege-cut registry for every dense and held-out
   derivation;
3. provider-side allocation, copy, indirection, metadata, call-count, and
   comparator cost accounting;
4. exact D-2 and P-1 closure;
5. an owner-approved completion-lock V2 replacing the identity-conflating and
   stateful G-2 through G-5 rows; and
6. the owner choice between C and F.

No signed or release-embedded high-level shortcut may close any of these rows.

## 5. Final review boundary

The repaired result establishes only this statement:

> One source-inaccessible, stage-indexed `PermitUse` relation can uniformly
> authenticate exact root-defined external-frame instances through immutable
> release membership, stateless signed grants, or an optional stateful current-
> membership mode without granting authority to ordinary source. Under current
> requirements, S is unnecessary; C is preferred exactly when extension without
> a release is required, and F otherwise.

Whether the public capability basis can use that gate to cover all required
systems contracts efficiently remains a separate, still-open proof obligation.
