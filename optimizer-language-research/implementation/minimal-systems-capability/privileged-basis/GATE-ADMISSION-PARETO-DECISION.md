# Privileged Gate Admission Pareto Decision

Date: 2026-07-15

Status: repaired conditional D14 research recommendation for owner review. No
production gate is selected. The stateless signed-frame grant is the preferred
candidate if the owner requires a new instance of an already defined external-
frame template to be usable without an authorization-release update. If that
property is not required, immutable release entries are smaller. The prior
stateful successor-snapshot design remains historical evidence and is not
superseded until the owner rules. This document authorizes no language,
specification, compiler, verifier, cryptographic deployment, capability entry,
standard library, container, benchmark, or production change.

## 1. Result

Three admission-state designs survive the earlier elimination of source
keywords, attributes, package paths, build flags, public unchecked operations,
private unchecked standard libraries, and in-process plugins:

- **F: immutable release entries.** Every privileged external frame is part of
  an immutable authorization release.
- **C: stateless signed-frame grants.** A fixed verifier accepts an immutable,
  signed instance of a root-defined external-frame template as an explicit
  build input.
- **S: stateful successor snapshots.** C is augmented with protected current
  state, successor epochs, tombstones, and local rollback resistance.

S has one unique benefit: after an installation observes a valid successor, it
can reject a revoked grant under the same approval-policy lineage. No owner
ruling requires that property. S therefore loses to C whenever C satisfies the
required extension contract.

F and C differ on one owner choice:

> Must a human-approved instance of an already policy-defined external-frame
> template be usable without publishing a new authorization release?

If yes, C is the smallest surviving design. If no, F is smaller because it
needs no extension-signature verification or distributed grant input. The
matrix records both branches rather than converting this choice into a hidden
assumption.

## 2. Scope of the comparison

This is a state-placement comparison inside the already researched sealed-gate
architecture. It is not a proof over every imaginable security architecture.
The earlier D14 dossier supplies the elimination evidence for writer-emittable
trust, name/path/package admission, compiler flags, private arbitrary unsafe
cores, public raw memory, and plugin-defined semantics.

The remaining alternatives are exhaustive under the following model. A
privileged decision can depend only on:

1. immutable authority already selected by the installed release;
2. an authenticated candidate input; or
3. protected ambient authorization state.

Those locations yield F, C, and S respectively. An interactive approval prompt
is ambient state and is not hermetic. An unsigned manifest, path, package name,
or operating-system ACL supplied with the build is forgeable input. A symmetric
MAC verifier distributed to every compiler would also distribute the forging
secret unless a protected service or device is added, which returns to S. A
public-key signature is therefore the minimal stateless witness for out-of-band
human approval; it is not semantic proof.

Proof-carrying code is deliberately outside this privilege comparison. If an
artifact proves its complete behavior against the existing public policy, it
is an ordinary proof-bearing module and every producer may submit it without a
signature. If it needs a new proof axiom, fact/effect kind, machine semantics,
or frame template, the semantic policy must change. A signature cannot promote
an ordinary helper or incomplete proof into privilege.

## 3. Why the extension choice matters

The case for C follows from D1, D4, D11, D12, and D14 but is returned for an
explicit owner ruling because it changes the release boundary.

A general systems language must reach project-specific kernels, devices,
vendor binaries, callbacks, register maps, and provider identities. A single
generic unapproved native-call operation cannot safely and efficiently cover
that domain:

- an ordinary writer-authored claim that foreign code does not retain, race,
  overrun, free, unwind through, or reenter xlang memory is forgeable;
- runtime isolation, serialization, or copying can enforce a conservative
  boundary, but imposes forbidden structural cost on zero-copy I/O, mappings,
  DMA, MMIO, callbacks, and direct provider calls; and
- returning no facts does not repair ownership, cleanup, retention, or race
  obligations that must be known before the call is allowed.

The safe alternatives for an opaque direct edge are an exact preapproved frame
instance or a runtime-enforced isolated/copying contract. F requires every new
preapproved instance to enter through a new authorization release. C allows the
same instance to travel as a signed immutable build input. Neither allows a
grant to define a new semantic kind.

A project-specific immutable root-manifest release is the strongest form of F;
it need not be a full compiler binary release. If the owner accepts that update
for every new frame instance, F remains viable and smaller. Calling such a
manifest an ordinary input while allowing the existing verifier to authenticate
it is C under another name.

## 4. The authority ceiling

Only instances of closed external-frame templates may use the privileged gate.
The semantic policy fixes the template language and generates the complete
checked contract from canonical bounded parameters. A grant may bind values
such as:

```text
template identity
provider or provider-validation-policy identity
target and ABI values
raw memory footprints and access modes
ownership-transfer and retention modes
handle, callback, reentry, blocking, cancellation, and concurrency modes
closed effect and external-fact codes
exact residual provider assumptions permitted by the template
```

A grant may not carry an arbitrary proposition, proof axiom, contract formula,
fact or effect kind, compiler pass, lowering algorithm, public semantic
operation, container invariant, topology, cleanup algorithm, allocator policy,
retry policy, parser, serializer, buffer implementation, or high-level helper.

The root-defined template, not the grant author, computes all preconditions,
normal/failure/abort postconditions, resource transitions, fact attenuation,
callback obligations, and cleanup obligations. Unknown fields, template values,
or residual assumptions fail closed. A template-language extension changes the
semantic policy.

This ceiling separates two routes:

- **ordinary proof route:** any artifact class accepted by the fixed public
  verifier whose entire refinement is proved through existing public semantics;
  native objects or lowering certificates are ordinary only if their artifact
  format and public verifier have been independently selected; and
- **privileged frame route:** the smallest opaque environmental leaf whose
  behavior cannot be proved because its provider is outside the checked cone.

Only the second route may receive a privileged authorization witness, either a
release entry or an extension grant. Wrappers, policies, buffering,
containers, strings, parsing, retry, and composite cleanup remain ordinary even
when an opaque leaf is used underneath them. Every derivability witness must
list its exact privilege cut. A privileged high-level operation receives no
D-2 or P-1 credit regardless of its authorization-witness kind.

Whether D4 permits a particular opaque provider is human-reviewed ledger
evidence, not a machine-decidable claim about source availability. The machine
check enforces only the closed template and exact signed disposition.

## 5. Acyclic identity graph

The prior draft incorrectly made the semantic root contain grants that bound
that root, and made an entry identity depend on the grant that signed it. The
repaired identities form this acyclic graph:

```text
SemanticPolicyID = H_policy(
  canonical reference semantics,
  proof policy and verifier semantics,
  closed authority/effect/fact vocabulary,
  canonical schemas and decoders,
  closed external-frame templates,
  stage-indexed PermitUse semantics)

GrantSignerPolicyID = H_grant_signer_policy(
  public keys, algorithms, threshold)

ApprovalPolicyID = H_approval_policy(
  SemanticPolicyID,
  initial GrantSignerPolicyID,
  predecessor ApprovalPolicyID with the identical SemanticPolicyID if any,
  immutable migration and exact-revocation policy over strict-ancestor grants)

StateTransitionPolicyID = H_state_transition_policy(
  SemanticPolicyID,
  ApprovalPolicyID,
  canonical snapshot, successor, key-continuity, epoch, and tombstone rules)

AdmissionModeID = H_admission_mode(
  FixedRelease
    | StatelessExtensions(ApprovalPolicyID)
    | StatefulExtensions(ApprovalPolicyID, StateTransitionPolicyID))

ImplementationConeID = H_implementation_cone(
  canonical toolchain component identities)

FrameImplementationCommitmentID = H_frame_implementation(
  SemanticPolicyID,
  exact provider subject and complete provider dependency cone
    | exact provider-validation policy and allowed evidence schema,
  target, frame, and unresolved stage commitments)

FrameEntryID = H_frame_entry(
  SemanticPolicyID,
  template identity and canonical instance parameters,
  generated contract,
  FrameImplementationCommitmentID,
  residual assumptions and fact attenuation)

ApprovalEvidenceID = H_approval_evidence(
  SemanticPolicyID,
  FrameEntryID,
  canonical D4 review and template/policy qualification evidence,
  exact-provider evidence available at approval if applicable)

ActivationProviderEvidenceID = H_activation_provider_evidence(
  SemanticPolicyID,
  FrameEntryID,
  actual policy-validated provider identity and dependency closure,
  actual provider evidence available only at activation)

ConsumerConeID = H_consumer_cone(
  canonical semantic program inputs and use dependencies,
  excluding grant, signature, and approval bytes)

DerivedUseID = H_derived_use(
  SemanticPolicyID,
  FrameEntryID,
  exact use path and compiler-derived ConsumerConeID,
  target, frame, artifact stage,
  exact implementation and provider resolution available at that stage)

EntryApprovalBodyID = H_entry_approval_body(
  SemanticPolicyID,
  FrameEntryID,
  ApprovalEvidenceID,
  Scope(ExactCone(ConsumerConeID) | Public),
  review identity and D4 disposition)

GrantApprovalRequestID = H_grant_approval_request(
  ApprovalPolicyID,
  EntryApprovalBodyID)

GrantSignatureSet = CanonicalThresholdSignatures(
  private keys selected by GrantSignerPolicyID,
  "xlang-frame-grant-v2" || ApprovalPolicyID ||
    GrantSignerPolicyID || GrantApprovalRequestID)

GrantEnvelopeID = H_grant_envelope(
  GrantApprovalRequestID,
  GrantSignerPolicyID,
  canonical GrantSignatureSet)

ReleaseEntryID = H_release_entry(
  SemanticPolicyID,
  EntryApprovalBodyID)

BaseEntrySetID = H_base_entries(
  sorted exact ReleaseEntryID values)

SnapshotID = H_snapshot(
  StateTransitionPolicyID,
  epoch, predecessor SnapshotEnvelopeID,
  current GrantSignerPolicyID,
  sorted active GrantEnvelopeID values,
  sorted cumulative tombstones)

SnapshotEnvelopeID = H_snapshot_envelope(
  SnapshotID,
  canonical transition signatures required by StateTransitionPolicyID)

ProtectedCurrentStateID = H_protected_current_state(
  StateTransitionPolicyID,
  exact installed SnapshotEnvelopeID,
  protected monotonic epoch and installation identity)

ToolchainReleaseID = H_toolchain_release(
  SemanticPolicyID,
  AdmissionModeID,
  ImplementationConeID,
  BaseEntrySetID)
```

Signatures are outside the payload they authenticate. `FrameEntryID` is
independent of approval, scope, review, and signatures. Release entries belong
to an exact release manifest outside `SemanticPolicyID`; they bind the policy
rather than being hashed into it. Compiler component hashes are leaves and
never contain `ToolchainReleaseID`.

`EntryApprovalBodyID` is ordered before both authorization witnesses. A
`ReleaseEntryID` proves exact membership in `BaseEntrySetID`; it needs no
extension signature verifier. A `GrantEnvelopeID` proves an out-of-release
approval under `ApprovalPolicyID`. C supports release entries and stateless
grants. S supports release entries and only grants active in the exact protected
current snapshot. F fixes `FixedRelease` and supports only release entries.
This is one admission predicate with a release-selected mode and typed
witnesses, not independently configurable gates.

An approval policy may name only strict-ancestor-policy grant identities in a
migration or revocation rule; it may not name a `GrantApprovalRequestID` that binds the
same `ApprovalPolicyID`. This preserves the DAG. An exact-provider entry signs
the complete provider dependency commitment. A provider-policy entry signs the
policy and allowed evidence schema; the actual provider identity is necessarily
later and belongs in the activation receipt.

Changing reference semantics or a frame template changes `SemanticPolicyID`.
Under C, changing approval keys or exact revocation policy changes
`ApprovalPolicyID` without changing semantics. Under S, a valid current-signer
rotation changes `SnapshotEnvelopeID` and `ProtectedCurrentStateID` while the
fixed approval and transition policies remain unchanged. An implementation-only compiler fix changes
`ImplementationConeID` and `ToolchainReleaseID` without invalidating a grant
whose semantic entry and final subject are unchanged.

## 6. Scope and the approval workflow

There are exactly two grant scopes:

```text
ExactCone(ConsumerConeID)  accepted only by byte equality
Public                     explicit policy-defined public marker
```

There is no subset, prefix, path, package, producer predicate, wildcard,
omitted scope, or caller-authored identity. Copying a public grant exercises
its exact approved authority; it does not mint new authority. A project-scoped
grant cannot replay into another consumer cone.

Approval and use have separate identities. `GrantApprovalRequestID` is the
canonical prospective `EntryApprovalBodyID` plus the selected
`ApprovalPolicyID`: it contains the frame entry, signed
implementation commitment, approval evidence, selected scope, review, and D4
disposition. It contains no grant envelope bytes and no per-use path.
`DerivedUseID` contains the entry, exact use path, compiler-derived consumer
cone, target, frame, stage, and the resolution available at that stage.

Approval is a deterministic two-pass workflow:

1. The fixed toolchain derives an unsigned `GrantApprovalRequest` from exact
   frame-template parameters, implementation commitments, approval evidence, selected
   scope, review inputs, and D4 disposition. Grant and signature bytes are
   excluded. `ExactCone` includes the one derived `ConsumerConeID`; `Public`
   includes only the explicit public marker.
2. Human review approves the request and signs the resulting
   `GrantApprovalRequestID`.
3. Every build re-derives the grant-level request and requires byte equality
   before accepting the envelope. It separately derives a fresh `DerivedUseID`
   for each use and applies the signed scope rule.

If adding the grant changes name resolution, the signed implementation
commitment, the target, the frame, or an `ExactCone` consumer identity, the
request is stale and fails. A later public use may have a different use path or
consumer cone; it remains valid only because `Public` was explicitly signed.
The build never lets grant bytes influence the semantic cone they authorize.

Admission is not runtime invocation authority. A public grant may make a frame
entry available, but checked file handles, device leases, callback tokens,
memory owners, and other runtime capabilities still control which code may
invoke it.

## 7. One stage-indexed gate

Release entries and extension grants pass one root-defined relation indexed by
artifact stage:

```text
PermitUse(stage, policy, release, admission_mode,
          authorization_witness, protected_current_state_if_stateful_grant,
          entry_approval_body, grant_approval_request_if_any,
          derived_use, artifact)
```

Every stage checks the common conditions:

```text
all inputs are canonical and bounded
all typed identities recompute exactly and form the declared DAG
the release, admission mode, approval lineage, FrameEntry, EntryApprovalBody,
  and DerivedUse all bind the identical current SemanticPolicyID
the authorization witness is exactly one of:
  FixedRelease:
    ReleaseEntry whose identity is a member of the release BaseEntrySetID
  StatelessExtensions:
    ReleaseEntry membership or AuthorizeEnvelope(current policy, exact envelope)
  StatefulExtensions:
    ReleaseEntry membership or exact non-tombstoned active-envelope membership
      in the valid SnapshotEnvelopeID named by ProtectedCurrentStateID
the frame entry instantiates a closed root-defined template
the generated contract equals the template result
every template parameter belongs to its closed root-defined domain
no ordinary predicate, proof identity, formula hash, bytecode, or callback rule
  enters the generated contract
the generated authority delta remains within the template's fixed ceiling
the re-derived EntryApprovalBody matches the release entry or grant request
the approval evidence and D4 disposition match that authorization body
the re-derived GrantApprovalRequest equals the signed request when one is used
the per-use DerivedUse matches the authorized FrameEntry and implementation commitment
the scope is Public or exact-equal to the derived ConsumerConeID
the toolchain implementation belongs to the exact release cone
every transitive privileged leaf independently passes PermitUse
```

`AuthorizeEnvelope(current_policy, envelope)` accepts exactly either a
canonical envelope signed by the static `GrantSignerPolicyID` selected by the
current approval policy or an exact strict-ancestor-policy envelope named by
the current policy's immutable migration allowlist. The complete predecessor
chain to that ancestor must be explicit and every policy in it must bind the
identical `SemanticPolicyID`, but no intermediate policy grants
transitive authority. An exact revocation rejects the envelope. Names, ranges,
prefixes, and implicit transitive trust are forbidden. In
stateful mode, a snapshot transition may replace the current
`GrantSignerPolicyID` only through the fixed key-continuity rule. Every newly
added envelope must validate under a signer policy authorized by that
transition. `ProtectedCurrentStateID`, snapshot validity, successor continuity,
signer state, epoch monotonicity, active membership, and cumulative tombstones
are all rechecked and bound to the stage receipt.

A semantic-policy change starts a new approval lineage. It cannot migrate an
old release entry, grant request, envelope, or review as authority. Those
artifacts may remain historical evidence, but every entry used under the new
policy requires a new derivation and review.

Stage-specific results are:

```text
Compile -> PendingUse(exact unresolved subject and dependency commitments)
Import  -> PendingUse(revalidated commitments; still non-executable)
Link    -> BoundStaticUse(exact linked subject, bytes, imports, relocations)
Load    -> ActivatedUse(exact loaded image or actual validated provider)
JIT     -> ActivatedUse(exact generated bytes and executable lease)
```

Compile and import cannot claim final resolution. Pending artifacts are not
executable. Link discharges only static commitments. Load or JIT discharges the
remaining provider, relocation, mapping, immutability, and executable-lifetime
obligations before activation.

An exact-provider frame requires identity and complete cone equality. A
provider-policy frame validates the actual resolved provider under the exact
root-defined policy, records its actual identity and evidence in the activation
receipt, attenuates facts, and invalidates reuse when that identity changes.
Substitution inside the policy is allowed but produces a different receipt;
substitution outside it fails.

No provider code, constructor, thread-local callback, or executable mapping may
run before the applicable activation check. If a platform loader cannot provide
that ordering, its preactivation behavior is itself an explicit external-frame
assumption; the design cannot claim fully certified final-image closure.

For a fully certified route, the checked chain covers optimization, code
generation, code and data bytes, imports, relocations, provider identities,
loader mapping, executable immutability, and the load receipt. An uncovered
stage is an explicit TCB frame, never silently upgraded to proof.

## 8. No in-process plugin route

Executable code that can affect parsing, checking, proof verification, grant
resolution, cone derivation, lowering, backend output, final binding, or trusted
artifact publication must be in `ImplementationConeID` and therefore in the
exact `ToolchainReleaseID`.

An outside producer may return only canonical untrusted data rechecked by the
fixed implementation. This includes AI output, proof search, solvers, generated
proofs, machine code, and proposed lowering instances. A grant cannot admit an
executable compiler plugin. A new in-process component requires a new toolchain
release even when its output is later checked.

## 9. Content, receipts, and dynamic providers

Executable semantics and approval attestation have separate identities:

```text
BuildContentID = H_build_content(
  SemanticPolicyID,
  canonical executable or IR bytes,
  target,
  exact FrameEntryID values,
  exact semantic dependency cones)

StageReceiptID = H_stage_receipt(
  BuildContentID,
  ToolchainReleaseID,
  AdmissionModeID,
  exact authorization-witness identities,
  ApprovalPolicyID values for signed grants,
  exact ProtectedCurrentStateID only for StatefulGrant witnesses,
  exact EntryApprovalBodyID values for every witness,
  exact GrantApprovalRequestID values for signed grants,
  exact per-use DerivedUseID values,
  exact ActivationProviderEvidenceID values when applicable,
  exact stage result,
  canonical final-resolution evidence when available)
```

Approval scope, review identity, release-entry or grant-envelope identity, and
signature bytes do not change `BuildContentID`. They do change the authorization
receipt. Signature bytes are immutable pinned inputs and are never regenerated
during a reproduction build.

Byte reproducibility is claimed only for a pinned build whose complete link-
time byte closure is explicit. Historical reproduction under S additionally
requires the archived exact authorization state. For a dynamic provider
accepted by policy, only the build, policy, and grant are reproducible until the
actual provider bytes are pinned. Loaded-image or activation-receipt equality is
not claimed from policy identity alone.

Canonical activation receipts exclude timestamps, random nonces, absolute ASLR
addresses, and incidental load-instance values only when the exact frame proves
address independence. Address-sensitive, MMIO, fixed-address, and code-lease
contracts bind the actual mapping base, permissions, lifetime, and lease in a
non-reproducible activation record while preserving a separate reproducible
content identity. Receipts bind relative image bytes, imports, relocations,
provider identities, and policy evidence. If exact mapped bytes cannot be
canonicalized, the receipt is auditable but not claimed byte-reproducible.

Artifact and receipt publication is one fail-closed commit: an artifact is
non-executable and unavailable to trusted consumers until its matching receipt
is durably published. A torn publication yields neither an authorized cache
entry nor executable code.

Every authorized build emits one canonical audit manifest:

```text
AuditManifestID = H_audit_manifest(
  BuildContentID,
  ToolchainReleaseID and AdmissionModeID,
  sorted EntryApprovalBodyID and authorization-witness identities,
  approval evidence and review identities,
  sorted StageReceiptID values)
```

Release entries, stateless grants, stateful snapshots, and stage receipts use
this same schema and typed identity graph. There is no alternate plugin,
package, helper, or development-mode audit path. The manifest proves exactly
which authority a build consumed; it does not claim a globally complete log of
every grant ever issued. Signer-custody and optional transparency logs are
separate operational evidence, not admission authority.

## 10. Replay, rotation, compromise, and revocation

C intentionally permits indefinite replay of a valid grant within its exact
scope and `ApprovalPolicyID`. Nonforgeability is conditional on the approval
threshold remaining uncompromised for that policy identity.

Under C, routine key rotation publishes a new `ApprovalPolicyID` without
changing `SemanticPolicyID`. Existing grants are either re-signed or admitted
through an immutable migration rule naming exact old `GrantEnvelopeID` values.
Revoking one grant likewise publishes a new approval policy that omits or
explicitly rejects it. Under S, a valid protected successor may rotate current
keys and membership within the fixed state-transition policy. F changes its
release-installation trust policy outside the extension gate. These are
authorization changes, not semantic-policy changes.

After key compromise, no new policy may inherit trust from arbitrary old
signatures. Recovery requires a new approval threshold and either fresh review
and signatures or an exact allowlist of independently re-reviewed old payload
identities. An old toolchain can reproduce old bytes but is no longer a
trustworthy approval verifier. Historical reproduction and renewed trust are
different claims.

S can reject an old grant locally after observing a valid successor, and can
rotate keys in protected state. It still cannot guarantee global freshness,
protect a fresh installation given only an old valid history, or recall an
already emitted binary without an external oracle or deployment system.

## 11. Pareto matrix

`GATE-ADMISSION-PARETO-DIMENSIONS.tsv` freezes the comparison dimensions.
`GATE-ADMISSION-PARETO-MATRIX.tsv` scores F, C, and S.
`verify_gate_admission_pareto.py` fails closed on schema drift, missing or extra
dimensions, shifted columns, conditional-selection errors, and key protected
cost claims.

The matrix has two terminal rows:

- if extension without an authorization release is required, C is recommended;
- if a release update per new frame instance is acceptable, F is recommended.

The selection algebra treats every `REQUIRED` row and the selected
`OWNER_CHOICE` branch as hard. A `NOT_REQUIRED` property receives no selection
credit. A `DESIRABLE` row can separate candidates only when one fails it; F, C,
and S all preserve approved bodies across their respective key-rotation route.
S therefore has no required or desirable capability unique over C. Its only
unique capability is unrequired same-policy local extension-grant currentness, while its state
machine, protected storage, persistence, verification, and distribution costs
are strict additions. S is eliminated in both branches and remains a valid
later alternative if the owner makes local extension-grant currentness required.

## 12. Primary-source support and limits

- [SPKI authorization certificates](https://www.rfc-editor.org/rfc/rfc2693)
  show that canonical signed authorization records can be held and transported
  by untrusted parties. Xlang rejects SPKI delegation, validity intervals, and
  general tuple reduction; they add authority and currentness semantics not
  required here.
- [KeyNote](https://www.rfc-editor.org/rfc/rfc2704) separates signed credentials
  from a compliance checker. Xlang uses a closed frame-template schema rather
  than a general assertion language.
- [DSSE](https://github.com/secure-systems-lab/dsse) authenticates a separated
  payload type and payload bytes. It does not define xlang key custody,
  canonical schemas, or semantic checking.
- [in-toto](https://github.com/in-toto/docs/blob/master/in-toto-spec.md) provides
  precedent for signed explicit material/product binding. It does not establish
  xlang ownership, effects, facts, or final-code refinement.
- [TUF](https://theupdateframework.github.io/specification/draft/) uses trusted
  version state for rollback protection. This supports the conclusion that S's
  mutable state purchases currentness, not signature nonforgeability.
- [Proof-carrying code](https://people.eecs.berkeley.edu/~necula/Papers/pcc_popl97.pdf)
  supports accepting an untrusted producer's code through fixed-policy proof
  checking. It supports keeping fully proved modules ordinary; it does not
  justify opaque provider assumptions.
- The [Nix deployment model](https://www.usenix.org/legacy/event/hotos07/tech/full_papers/dolstra/dolstra_html/)
  supports explicit content-addressed input closures for reproducible
  realization. It does not make dynamic providers reproducible.

These sources support components and tradeoffs. None proves this complete gate,
the adequacy of the frame-template language, the public resource basis, or an
implementation.

## 13. Remaining falsification obligations

Before the conditional recommendation can become an owner-ready selection:

1. independently hostile-review the repaired identity DAG, stage relation,
   frame authority ceiling, replay boundary, and matrix;
2. constructively show that the closed frame-template language covers the
   required D6 external domains without arbitrary propositions or structural
   copying/isolation tax;
3. prove that every dense and held-out derivation exposes only minimal frame
   leaves and cannot use any privileged high-level shortcut through either a
   release entry or an extension grant;
4. replace the stateful and identity-conflating G-2 through G-5 completion rows
   with an owner-approved V2 lock; and
5. preserve exact D-2 and P-1 as fail-closed until their independent gaps close.

The first item belongs to this gate step. Items two and three connect the gate
to public-basis completion and cannot be assumed from authentication alone.

## 14. Owner decision eventually required

After hostile review, frame-template coverage evidence, and exact minimal-
privilege-cut evidence, the owner must choose exactly one release boundary:

1. **C if extension without a release is required:** permit signed immutable
   instances of existing external-frame templates as explicit build inputs,
   with no same-policy extension-grant revocation; or
2. **F otherwise:** require every new frame instance to arrive in a new
   immutable authorization release.

No decision about syntax, proof language, compiler implementation,
cryptographic suite, standard library, container, or production adoption is
bundled with that choice.
