# Gate Authentication Lock: Hostile Review

Status: PASS for the research-level semantic architecture, 2026-07-15. This
review authorizes no implementation, syntax, compiler, specification,
cryptographic deployment, capability entry, standard library, or production
change.

## 1. Reviewed artifacts

| Artifact | SHA-256 |
|---|---|
| `GATE-AUTHENTICATION-LOCK.md` | `6c104f0cba7a3fe2c3e354be37d4351bd4f9413ab38cdd4bfa35d4cbd6eb4e13` |
| `PRIVILEGED-BASIS-COMPLETION-LOCK.md` | `395279c549aa37137d122282eed5c5e4a732521697f41ae406632bd4e2141e4d` |

The review independently reconstructed the threat boundary, identity layers,
snapshot transitions, key continuity, use predicate, cache revalidation,
ordinary-proof boundary, and componentwise removal witnesses.

## 2. P0 and P1 attacks repaired before PASS

### 2.1 Dependency substitution

Attack: an approved helper can resolve a changed object, provider, backend
component, or linker dependency even when its readable name and approval
snapshot are unchanged.

Disposition: implementation and consumer dependency cones are both mandatory.
The compiler reconstructs them from canonical artifacts, and final resolution
must match the approved implementation subject. Snapshot membership never
substitutes for final-code binding.

### 2.2 Identity conflation

Attack: using one digest for semantic policy, mutable authorization state, and
object caching either permits semantic substitution or invalidates unrelated
artifacts after every approval update.

Disposition: the lock separates the immutable semantic-root identity, mutable
authorization-state identity, and exact per-entry semantic/cache identity. The
complete active snapshot belongs in build attestation; an object may be reused
only after its exact used entries pass current revalidation.

### 2.3 Forgeable program scope

Attack: a source path, package name, UUID, or project label can be copied, while
a whole-program digest creates unnecessary approval churn.

Disposition: the default use scope is the compiler-derived consumer-use cone.
A broader public scope must be an explicit protected root-defined value rather
than an omitted field or writer string.

### 2.4 Root and key replacement

Attack: an in-band key identifier, plugin, alternate backend, or capsule-
supplied verifier can authorize its own replacement.

Disposition: the semantic root fixes the verifier, schemas, authority kinds,
proof/fact/effect vocabulary, backend policy, and initial threshold-key state.
Old-plus-new threshold authorization is required for sequential key rotation.
Replacing the verifier or root creates a new toolchain installation and is
outside the in-language gate.

### 2.5 Base-entry bypass

Attack: compiler-shipped intrinsics can form a second unreviewed branch even if
extensions use the signed gate.

Disposition: every shipped entry is an authorization record in the embedded
genesis snapshot `S0`. Base and later entries pass the same use predicate.

### 2.6 Rollback and overclaimed freshness

Attack: local monotonic state does not tell a fresh installation that a newer
revocation exists and cannot detect a split view never presented locally.

Disposition: the claim is explicitly local to protected installation state.
Global freshness or cross-installation equivocation detection would require a
separately selected freshness or transparency service.

### 2.7 Signature-as-proof confusion

Attack: a valid signature can be misread as proof that an implementation or
foreign provider satisfies its contract.

Disposition: every entry has exactly one disposition: `CERTIFIED`, with a
complete proof against the fixed policy, or `TRUSTED_FOREIGN`, with every
residual assumption named. Approval authenticates the exact reviewed tuple; it
does not establish semantic truth.

### 2.8 Admission-as-sandbox confusion

Attack: the gate may prevent a dependency from minting an operation but still
allow it to invoke a deliberately public operation.

Disposition: the lock limits its claim to semantic admission. Runtime
invocation policy requires explicit capability-value distribution or another
ordinary effect policy and does not become a second privilege-admission path.

### 2.9 Ordinary abstraction misclassified as privilege

Attack: prohibiting every opaque representation rule or new public semantic
identity would force an ordinary nominal type and hidden checked
representation through the privilege gate, defeating ordinary-library
generativity.

Disposition: only a new root-governed primitive identity or an unproved
primitive/machine representation axiom requires admission. A nominal type,
contract, or hidden representation is ordinary when every constructor and
operation is proved through existing public semantics.

### 2.10 Opaque provider bytes

Attack: exact-byte binding cannot be claimed for a kernel, loader, device, or
other dynamically supplied provider whose implementation is unavailable to the
verifier.

Disposition: `CERTIFIED` requires the exact implementation subject and complete
byte/semantic cone. Only `TRUSTED_FOREIGN` may instead bind one exact root-
defined provider-validation policy, and it must name the unavailable provider
bytes as a residual assumption and attenuate exported facts.

### 2.11 Premature general-proof requirement

Attack: requiring a producer certificate for every unseen shape would
preselect a general logic even when bounded static proof, existing runtime
metadata, or another efficient representation already supplies safety.

Disposition: the completion contract accepts exactly those three explicit
safety-authority routes and forbids topology-specific privilege. Selecting a
general proof layer still requires an independent expressibility or dominance
argument.

## 3. Componentwise lower bound

The review found an independent failure witness for removing each retained
component:

1. no immutable policy root permits an artifact-supplied verifier;
2. no typed canonical identity permits payload or record-type confusion;
3. no authenticated out-of-band approval permits ordinary inputs to mint
   trusted authority;
4. no protected monotonic state permits post-revocation replay;
5. no exact target, frame, and use scope permits cross-context replay;
6. no mandatory implementation cone permits dependency substitution;
7. no proof/trust disposition confuses approval with correctness;
8. no use-time revalidation lets stale caches survive revocation; and
9. a capsule able to alter the verifier creates a self-authorizing gate.

One canonical verifier and state machine account for all nine. Separate
lang-item, intrinsic, foreign-function, runtime-helper, standard-library, and
plugin admission paths are unnecessary.

## 4. Research boundary

The architecture is selected at the semantic level:

> Exactly one sealed privileged admission verifier exists per fixed
> authenticated semantic toolchain root. Compiler-shipped and later entries
> are records in one authenticated successor-snapshot state machine, and every
> use is bound to its exact contract, evidence, target, frame, implementation,
> consumer cone, and current membership.

The following remain implementation-lock choices rather than semantic
alternatives: wire encoding, cryptographic suite, threshold size and custody,
protected-state provider, dynamic-provider validation, resource limits,
conformance vectors, and optional transparency. None may weaken the selected
identity, evidence, scope, revalidation, or fail-closed rules.

The gate lock does not select the public resource basis or establish systems-
domain derivability. Those remain subject to the completion contract.
