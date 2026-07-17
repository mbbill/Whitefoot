# Gate Authentication Lock

Status: selected D14 research architecture, 2026-07-15. This document locks
the authentication and authority shape of the single privileged admission
gate. It selects no language syntax, compiler implementation, cryptographic
library, operating-system integration, capability entry, standard library,
container, or production deployment.

## 1. Decision

Relative to one fixed authenticated semantic toolchain root, xlang has exactly
one stateful sealed admission verifier for privileged semantic edges.

The compiler installation embeds:

1. one immutable semantic policy root;
2. one initial threshold-key policy;
3. one genesis approval snapshot S0; and
4. the exact verifier, canonical decoder, and fixed backend identities needed
   to enforce that root.

Compiler-shipped capabilities are ordinary authorization records in S0. Later
capability capsules enter only through signed successor snapshots. Every use of
a privileged edge, regardless of origin, passes the same current-membership,
evidence, scope, final-resolution, and dependency-cone predicate.

Ordinary source has no privilege-admission keyword, attribute, module, import,
file location, package identity, flag, environment variable, build-script
output, plugin, linker symbol, or cache record. A readable logical name is
never authority. Replacing the semantic root or verifier creates a distinct
toolchain installation; the gate cannot authorize a redefinition of itself.

The exact claim is:

> One privileged admission gate per fixed authenticated semantic toolchain
> root, with toolchain installation and root replacement named as the external
> TCB boundary.

## 2. Threat boundary

The gate trusts:

- the exact installed compiler, fixed backends, semantic-policy root,
  canonical decoder, signature verifier, and certificate checker;
- the installation's protected key, snapshot, and monotonic-version state;
- atomic persistence of that state;
- the configured approval-key threshold;
- collision resistance and signature unforgeability of the root-selected
  algorithms; and
- every exact residual assumption recorded by a TRUSTED_FOREIGN entry.

It does not trust:

- ordinary source, dependencies, package manifests, build scripts,
  environment variables, search paths, network repositories, or readable
  names;
- capsule, certificate, proof, object, cache, LTO, linker, and binary-library
  producers;
- producer-supplied dependency lists;
- stale but formerly valid artifacts or snapshots;
- dynamically resolved helpers or providers merely because their symbols
  match; or
- compiler plugins, alternative backends, and transformations outside the
  installed toolchain identity.

The gate does not protect against replacement of the trusted compiler or
protected state, compromise of the approval threshold, cryptographic failure,
or an already deployed static executable continuing to run after a later
revocation. It supplies local protected-state freshness, not global knowledge
that another installation has seen a newer snapshot.

Admission and invocation authority are separate. The gate decides whether a
semantic edge may exist in a checked artifact. Restricting which ordinary
dependency may invoke an intentionally public file, network, device, or other
capability requires explicit runtime capability distribution or another
ordinary effect policy; admission alone is not a dependency sandbox.

## 3. Immutable semantic root

The semantic root binds one canonical identity for:

~~~
canonical schemas and encodings
typed hash and signature domains
reference machine, ownership, resource, effect, and fact semantics
certificate kernel and verifier
closed proof-axiom, effect-kind, fact-kind, and authority-kind sets
fixed generic ABI and fixed lowering-instance schemas
backend identity policy
snapshot transition and key rotation rules
initial key state K0
genesis approval snapshot S0
~~~

The root's encoding is injective and self-delimiting. Every record type has a
domain-separated type identifier, one schema version, fields in schema order,
fixed integer widths, length-delimited variable data, canonical list ordering,
and fixed resource limits. Decoding rejects unknown, duplicate, missing,
out-of-order, overlong, nonminimal, and out-of-range values. Acceptance
requires encode(decode(bytes)) == bytes.

The root selects exact hash and signature algorithms. A signed record
authenticates its domain-separated payload type and exact canonical bytes.
key_id is only a lookup hint; the protected key policy decides which keys and
algorithms count.

A capsule may instantiate only schemas and semantic kinds already present in
the root. It cannot install:

~~~
a proof axiom or verifier
a fact or effect kind
a memory-model rule
a backend or compiler plugin
a new lowering algorithm
a new capsule or authority kind
~~~

Changing any item above creates a new semantic-root and toolchain identity. A
capsule may bind an implementation to an already fixed helper, ABI-action,
opcode, or lowering-template schema; it may not supply arbitrary transformation
code.

## 4. Typed canonical identities

For every schema type tau, identity is conceptually:

~~~
id_tau(value) =
  Hash(
    "xlang-gate-v1" ||
    length(tau) || tau ||
    length(CanonicalEncode_tau(value)) || CanonicalEncode_tau(value)
  )
~~~

At minimum, the gate keeps distinct identities for:

~~~
semantic_policy_root
toolchain_release
key_state
approval_snapshot
entry_contract
implementation_subject
implementation_dependency_cone
capsule
consumer_use_cone
approval_record
review
certificate or foreign-frame assumption
target
frame
~~~

Three identity layers must not be conflated:

1. the immutable semantic toolchain identity;
2. the mutable authorization-state identity; and
3. the exact per-entry semantic and cache identity.

An unrelated snapshot update changes build attestation but need not change
generated code or invalidate an object whose exact used entries remain active.

## 5. Capability capsule

A capsule has one closed root-defined implementation kind, such as:

~~~
ROOT_TABLE_ENTRY
FIXED_HELPER
FIXED_LOWERING_INSTANCE
FOREIGN_FRAME
~~~

COMPILER_PLUGIN, arbitrary native transformation code, and a producer-defined
verifier are not capsule kinds.

Each capsule binds:

~~~
semantic policy root
public checked contract and signature
ownership and resource preconditions
every normal, recoverable-failure, and abort postcondition
effect contract
fact production, transfer, attenuation, and invalidation contract
representation, layout, provenance, and address-stability contract
cleanup, abandonment, and release obligations
implementation kind and exact implementation subject
target and backend identity
frame and exact provider identity, or a root-defined provider-validation policy
  only under TRUSTED_FOREIGN
mandatory implementation dependency cone
one disposition and its evidence
~~~

Disposition is exactly one of:

- CERTIFIED(certificate): the fixed verifier proves the complete exported
  contract for the exact implementation subject and complete byte/semantic
  dependency cone, target, and frame; or
- TRUSTED_FOREIGN(frame_assumption, optional_partial_certificate): every
  residual machine, runtime, OS, loader, vendor, device, or other opaque
  assumption is named and the exported contract attenuates facts accordingly.
  An opaque dynamically supplied provider may bind an exact root-defined
  provider-validation policy only in this disposition; the unavailable
  provider bytes remain an explicit residual assumption.

A signature proves approval and integrity, not semantic correctness. CERTIFIED
is legal only when no residual trusted semantic assumption remains.

## 6. Scope and dependency cones

An approval record binds the exact tuple:

~~~
(
  semantic_policy_root,
  entry_contract,
  capsule,
  disposition,
  target,
  frame,
  implementation_dependency_cone,
  consumer_use_cone,
  review
)
~~~

There is no tuple, snapshot, or dependency-cone alternative. Current snapshot
membership authenticates approval; it never substitutes for binding the final
code that will execute.

The target identity binds at least architecture, ISA features, endianness,
pointer width, data layout, atomic and memory-model profile, calling
convention, object and relocation model, backend identity, and any target facts
consumed by the contract.

The frame identity binds at least ABI, loader and symbol-resolution model,
allocator/runtime/OS/device assumptions, provider identity or provider-
validation policy, and exact normal, failure, abort, cleanup, and reentry
behavior.

The compiler reconstructs both dependency cones from canonical artifacts. A
producer-supplied input list is audit material only.

The implementation cone closes over every byte and semantic dependency that may
affect the resolved helper, lowering instance, provider, or fixed runtime entry.
The consumer-use cone is reverse-reachable from the privileged use and includes
at least:

~~~
type, layout, constant, and representation dependencies
data and control dependencies
ownership and resource dependencies
effect and fact producers
generic instantiations and transitive callees
exact call-node path
requested entry identity
~~~

Traversal may stop only at immutable semantic-root nodes or another already
authenticated checked capability boundary. Final backend, linker, loader, and
helper resolution is rechecked against the authorized implementation subject
and implementation cone.

A whole-program digest is not required and would recreate approval churn. A
source-supplied project name, path, package name, or UUID is forgeable. The
consumer-use cone is the default program/use scope. A broad public approval is
an explicit protected root-defined scope value, never an omitted field or
wildcard string.

## 7. Genesis and successor snapshots

S0 is embedded in the toolchain release and contains authorization records for
every compiler-shipped capability. Every successor is a full canonical state:

~~~
S_n = {
  semantic_policy_root,
  key_state,
  epoch: n,
  previous_snapshot,
  active_approval_records,
  cumulative_revocation_tombstones
}
~~~

Transition S_n to S_(n+1) succeeds only when:

~~~
the semantic root is unchanged
the epoch is exactly n + 1
the predecessor identity is exactly id(S_n)
the current protected key threshold validates the signature
all records and tombstones are canonical and unique
every prior active record is carried unchanged or explicitly tombstoned
every prior tombstone remains present
no tombstoned approval identity is reactivated
every new record references a present canonical capsule and review
the successor is atomically persisted before it becomes active
~~~

Reapproval after revocation requires a new capsule or approval-record identity
and a new review. The active snapshot is selected only from protected
installation state; ordinary inputs cannot supply or override it.

## 8. Key lifecycle

A key state is:

~~~
K_v = {
  semantic_policy_root,
  version: v,
  previous_key_state,
  exact public keys and algorithms,
  threshold
}
~~~

Rotation K_v to K_(v+1) requires exact sequential versioning, the same semantic
root, threshold authorization under both K_v and K_(v+1), and atomic protected
persistence. This old-plus-new continuity prevents an unapproved replacement
key from authorizing itself.

If the current quorum is unavailable or compromised beyond its threshold,
recovery requires installation of a new trusted toolchain/root state. The gate
cannot safely self-authorize recovery from loss of its own trust anchor.

## 9. Single use predicate

For every base or extension entry:

~~~
PermitUse(toolchain, artifact, use, capsule) iff

  toolchain semantic root and protected state are intact
  and artifact and capsule are canonical
  and every typed identity recomputes exactly
  and the active snapshot contains the exact approval record
  and no referenced identity is tombstoned
  and policy, entry, disposition, target, frame,
      implementation cone, use cone, scope, and review match
  and every transitive privileged dependency independently passes PermitUse
  and (
        disposition is CERTIFIED
        and the fixed verifier validates the certificate against the exact
            contract, subject, target, frame, policy, and dependency cone
      or
        disposition is TRUSTED_FOREIGN
        and the active record contains the exact residual assumption set
        and exported facts are no stronger than the reviewed contract
      )
  and final backend, linker, loader, and helper resolution equals the
      authorized implementation subject and implementation cone
~~~

Unknown artifacts, incomplete resolution, unclassified dependencies,
unsupported capsule kinds, and indeterminate verification fail closed.

## 10. Artifact, cache, and link revalidation

Every artifact records the exact set of privileged uses, including:

~~~
semantic_policy_root
entry_contract
capsule
approval_record
implementation_dependency_cone
consumer_use_cone
target
frame
~~~

On every cache read, binary import, LTO import, object import, final link, and
future runtime load:

1. recompute the applicable identities and cones;
2. check exact active membership and absence of tombstones;
3. revalidate certificates or exact foreign assumptions; and
4. recheck final implementation/provider resolution.

The complete snapshot identity belongs in the build attestation, not every
semantic object cache key. An old artifact whose exact entries remain active may
be reused after current revalidation. An artifact containing a revoked or
mismatched entry is rejected.

Current xlang has no runtime loader or JIT. If either is later admitted, it must
apply the same predicate at the load boundary. This document makes no claim of
recalling an already emitted static executable.

## 11. Rollback, replay, and freshness

- A lower snapshot or key version is rejected.
- The same version and same digest is a deterministic no-op.
- The same version and a different digest is rejected as equivocation.
- A higher non-successor version is rejected; intermediate transitions are
  required.
- Cross-policy, target, frame, backend, provider, consumer-cone, review, and
  disposition replay fails by identity mismatch.
- A stale cache with an active unchanged entry may pass current revalidation.
- A stale cache with a revoked entry fails.

Protected local state prevents rollback after that installation has observed a
new state. A fresh or offline installation given an old but valid chain cannot
know that another installation has seen a revocation. Cross-installation
equivocation and global freshness would require a separately selected trusted
freshness source or transparency service. They are not part of this local
nonforgeability claim.

## 12. Ordinary proof versus privilege

A proof-bearing artifact is ordinary when:

1. its executable behavior uses only existing public checked semantics and
   already admitted checked entries;
2. its proof derives only propositions in the fixed reference policy;
3. every derived proof rule is itself proved to imply that policy; and
4. it introduces no opaque machine, backend, foreign, effect, fact, trust, or
   unproved primitive representation edge.

Every producer may submit such an artifact to the same fixed verifier. It needs
no snapshot approval. Solvers, proof search, AI generation, optimizers, and
producer-defined derived rules remain untrusted.

A capsule is required when an artifact binds a new root-governed primitive
semantic identity, unproved primitive or machine representation axiom, fixed-
helper or lowering instance, backend/ABI/OS/device/foreign edge, or residual
trusted assumption. An ordinary nominal type, checked contract, or hidden
representation remains ordinary when its construction and every operation are
proved entirely through existing public semantics. A general compiler pass or
backend changes the toolchain root rather than entering as a capsule.

A future fixed checked-resource IR may allow ordinary proof-carrying modules
without admitting arbitrary native objects or native-code verifiers. Any such
IR, reference semantics, and verifier must be part of the immutable semantic
root before ordinary modules can use it. This gate lock neither selects nor
assumes that public basis; it only fixes the authority boundary it must obey.

## 13. Attack disposition

| Attack | Disposition |
|---|---|
| Dependency supplies a fake capsule or snapshot | Ordinary inputs cannot select protected state; signature and predecessor chains fail. |
| Dependency copies a capability or helper name | Names carry no authority. |
| Package inserts a new privileged call | Its reconstructed consumer-use cone has no matching active approval record. |
| Relevant package code changes | The consumer-use cone changes and the approval no longer matches. |
| Unrelated package code changes | The existing use cone may remain reusable after current membership revalidation. |
| Approved helper resolves a changed transitive object | Mandatory final implementation-cone binding rejects it. |
| Linker symbol aliases another helper | Exact final subject and cone mismatch. |
| Plugin adds an intrinsic or lowering | Plugin is outside the fixed toolchain identity and cannot be a capsule. |
| Old snapshot is replayed after a local update | Protected sequential state rejects it. |
| Two same-epoch snapshots are presented locally | Different digest is rejected. |
| Cross-installation split view | Outside the local-freshness claim unless transparency is later selected. |
| Capsule or certificate is replayed across target/frame/provider | Exact scope mismatch. |
| Certificate is reused for changed contract or implementation | Typed identities and cones change. |
| Stale cache imports a revoked entry | Current-membership revalidation rejects it. |
| Approval threshold is compromised | Outside the guarantee and explicitly part of the TCB. |
| Trusted compiler/root is replaced | Different or compromised toolchain installation, not an in-language gate bypass. |

## 14. Componentwise minimality

A global mathematical minimum across all possible cryptographic systems is not
claimed. The architecture is componentwise minimal under the D14 hard
constraints. Each retained component has an independent removal witness:

1. Without an immutable semantic root, an artifact can supply its own verifier
   or authority schema.
2. Without typed canonical identities, record and payload-type confusion is
   possible.
3. Without external authenticated approval, ordinary inputs can mint
   privilege.
4. Without protected monotonic state, a formerly valid authorization can be
   replayed after revocation.
5. Without exact target, frame, and use scope, an approval replays in another
   semantic context.
6. Without mandatory implementation-cone binding, reviewed code can execute
   substituted dependencies.
7. Without distinct proof and trusted-foreign dispositions, a signature is
   mistaken for a semantic proof.
8. Without use-time revalidation, stale caches bypass revocation.
9. If capsules may change the verifier or policy, the gate can authorize a
   rule that authorizes itself.

One canonical verifier and one authenticated snapshot state machine satisfy all
nine. Separate lang-item, intrinsic, FFI, runtime-helper, and standard-library
admission paths are unnecessary. Independent effects and assumptions remain
independent records inside this one gate.

## 15. Primary-source support and limits

- Rust lang items, intrinsics, and compiler-special Box semantics show that
  identity, lowering, and representation are distinct authorities, but Rust
  offers multiple hooks and no authenticated proof-checked admission system.
- GHC primitive operations and Swift's compiler-created Builtin module show
  that compiler-private namespaces are practical; GHC exposes its primitive
  namespace and Swift's standard-library parsing mode is a flag, so neither is
  the required nonforgeable boundary.
- Proof-carrying code separates a fixed consumer safety policy from an
  untrusted producer's proof. PCC with untrusted proof rules shows that a
  producer can prove derived high-level rules imply a fixed low-level
  memory-safety policy rather than installing those rules in the TCB.
- TUF supplies the relevant attack model and state contracts: an out-of-band
  root, threshold verification, sequential versions, nonvolatile state,
  rollback checks, snapshot consistency, revocation, and old-plus-new key
  rotation. Xlang uses one local approval registry rather than copying TUF's
  distribution roles.
- DSSE supports domain-separated authentication of payload type and exact
  bytes while treating key IDs as hints. In-toto supports exact material and
  product binding but does not justify trusting a producer's declared
  dependency list; xlang reconstructs its semantic cones.

These precedents establish feasibility of the selected components, not the
security of an unimplemented xlang gate. Exact encoding test vectors,
cryptographic suite, threshold size and custody, protected-state provider,
dynamic-provider rules, verifier limits, and conformance corpus remain future
implementation-lock work. They cannot weaken the semantics fixed here.

## 16. Research disposition

The Stage-A gate architecture is selected at the semantic level after repairing
the prior optional dependency-cone binding and identity conflation. It proves
one nonforgeable admission path relative to a fixed trusted installation. It
does not prove the public resource basis, ordinary-library derivability,
runtime authority distribution, global freshness, or implementation
correctness.

No production action follows from this lock. The next research requirement is
to select the checked public basis whose ordinary proof-bearing modules obey
Section 12 and then prove the 26-domain derivability partition without adding a
second admission path.
