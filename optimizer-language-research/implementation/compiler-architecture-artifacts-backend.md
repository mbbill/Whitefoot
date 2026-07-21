# Production Compiler Architecture — Artifacts and Backend

Status: OWNER-APPROVED ARCHITECTURE WITH BLOCKERS. This file is part of the
single [Production Compiler Architecture](compiler-architecture-design.md)
design record. `THE-PLAN.md` is execution authority. This file does not amend a
numbered specification, protected semantic surface, profile, or entrance gate.

This file uniquely owns Decisions 11 through 14 and the conservative backend contract. Cross-stage authority, entrance gates,
execution order, owner decisions, and exit status remain in the parent index.

## Decision records

### Decision 11: checked artifact and derivation language

**Decision:** Define one complete, canonical, target-qualified checked artifact
contract before production checker, replay, or lowering interfaces. Keep its exact wire
schema deferred until the grammar/tree and semantic contracts close. Use a separate
canonical compilation envelope for the selected empty or verified optimization
overlay.

**Problem being solved:** DIAG-2 requires artifact-decidable acceptance and
explicit derived operations. A dump of checker internals is neither a stable
proof language nor a complete contract. Optional facts also must not mutate the
accepted semantic artifact.

**Specification and project constraints:** DIAG-2 requires explicit derived
operations and proof objects; DIAG-3 binds reports to the artifact; the checker
must reject invalid source itself; optional facts cannot affect acceptance.

**Selected design:** The base checked artifact contains, in canonical order:

- envelope magic/version; exact specification, syntax-schema, checked-schema,
  and derivation-schema identities;
- a non-authorizing provenance record containing the static-catalog identity;
  it is release/accounting binding only and is never read to choose a semantic
  judgment or lowering action;
- complete canonical `CompilationTargetProfile` bytes and identity;
- the complete canonical source binding and finalized derivation tree;
- scopes, declarations, uses, and exact resolutions;
- types, modes, constants, canonical region trees/classes/outlives judgments,
  owner-discriminated holders, loans, places, values, CFG/check/proof references,
  alias-origin sets, approved operation/function return-provenance summaries,
  per-call provenance records, validated staged `TypedCallBoundary` and
  `CallProvenanceRecord` families, normalized nested-region slots and
  owner-scoped instance-slot references, tagged lexical/slot region actuals,
  bounded call-environment composition records, finite A-17 region-fact
  profiles, distinct semantic/code-instance types and their one-to-one baseline
  mapping,
  and complete substitutions; binding-to-claim transitions and alias-view
  OWN-10 records remain schema-blocked on A-16 and A-11 respectively;
- every exact STOR-5 obligation site—field, variant payload, array/buffer
  element, and box/arena content—its fully type/const-substituted stored type,
  positive recursive `StorableType` judgment and derivation, plus exact coverage
  tying that site to the proved type; transient non-storage values require no
  positive storage judgment; this
  record family remains schema-blocked until A-13 supplies the normative
  recursive rule;
- every symbolically checked function/template, immutable template typed-call
  record and typed boundary, finite template call edge/SCC, A-18-resolved FN-6
  accepted-edge judgment, and all-source-function template semantic/effect coverage
  record; rejecting `Fn6CycleWitness` records belong only to the deterministic
  stage-failure outcome and never appear in accepted base bytes;
- every concrete semantic function instantiation, its checked signature/body
  coverage,
  immutable concrete typed-call record, finite concrete call edge/SCC, and
  closed seed/reachability derivation, plus its one-to-one baseline
  `CodeInstanceRef`; any region-erased `EmissionShapeKey` is non-authorizing and
  cannot merge emitted bodies;
- complete concrete type layouts, sizes, alignments, fields, discriminants,
  ABI classifications, checked size/alignment derivations, per-function frame
  totals, frame ceiling, OP-9 allocation-size premises, target rejection
  premises, and target-family coverage;
- structured CFG blocks, operations, terminators, edges, origins, and
  reachability;
- ownership entry states and every validated transition;
- declared and exhibited symbolic/concrete effects, distinct template/concrete
  call graphs/SCCs, provenance-equation dependencies, and FN-6 evidence;
- every semantic check site with a stable base `CheckSiteId`; every optional
  elimination candidate has a stable checker `DerivationId` even while it has
  no authority under the empty overlay;
- retained semantic checks, potential trap edges, explicit normal-edge drops,
  frees, arena releases, and ERR-3 context records;
- DIAG-3 report source records, complete check-site inventory, and—after
  A-14/A-15 close—the full canonical keys from which collision-free dense
  function/caller-site/trap-site report tables are derived;
- same-kernel replay derivation records in the closed families established by
  Decision 10; and
- exact coverage tables tying every semantic tree node, declaration, template,
  typed call, typed boundary, provenance boundary, graph edge/SCC, body,
  semantic/code-instance mapping, instance seed/reachability edge,
  storage-obligation site, region, holder, loan, provenance summary/call, block, edge,
  check, cleanup, and derivation to its required record. Every template or
  concrete semantic record/reference repeats its Decision 5
  `SemanticOwnerRef`. Typed/provenance boundaries satisfy only their own exact
  owner-pair coverage and can never substitute for either side's owner-local
  record or coverage; every raw cross-owner reference is invalid.

The base artifact records every path-specific drop occurrence by binding,
source/derived origin, and exact CFG edge regardless of the eventual DIAG-3
projection. The singular lifetime `drop node_path` projection is not frozen:
A-14 must decide whether the successor report carries all occurrences, a
binding/owner site, or another stable identity. Likewise A-15 must fix logical
stack order/node meaning. No final report schema, golden report bytes, or
report-producing backend code lands before those rulings.

Identity construction is acyclic and domain separated:

```text
TargetProfileId = SHA256("whitefoot-target-profile-v1" ||
                         CompilationTargetProfileBytes)

BaseId = SHA256("whitefoot-base-artifact-v1" || BaseArtifactBytes)

BackendProfileId = SHA256("whitefoot-backend-profile-v1" ||
                          BackendInvocationProfileBytes)

OverlayBytes embeds BaseId, TargetProfileId, and BackendProfileId
OverlayId = SHA256("whitefoot-optimization-overlay-v1" || OverlayBytes)

ReportProjectionBytes contains final check/lifetime/density records and trap
templates, but excludes every artifact_hash value

CompilationArtifactBytes = canonical(BaseArtifactBytes,
                                     BackendInvocationProfileBytes,
                                     OverlayBytes,
                                     ReportProjectionBytes)
CompilationId = SHA256("whitefoot-compilation-artifact-v1" ||
                       CompilationArtifactBytes)
```

The overlay binds `BaseId` and both execution profiles, never a later final
identity. CodegenPlan consumes
only `FinalizedCompilation`, which owns `CompilationArtifactBytes` and
`CompilationId`. Runtime and accompanying reports insert `CompilationId` into
DIAG-3 `artifact_hash` after hashing; those filled report bytes are not part of
the hashed projection. Empty and nonempty verified overlays have different
canonical tags/bytes and therefore different `OverlayId` and `CompilationId`.
An overlay with no authorized fact canonicalizes to the one empty form for its
exact base/target/backend tuple.
Base accepted semantics are identical in every case.

`BaseId` therefore identifies accepted source semantics for one exact target;
it deliberately does not identify emitted code. `CompilationId` is the complete
output/publication identity because its embedded backend profile binds the exact
Whitefoot compiler and lowering build, plan schema, lowering dossier, requested
output set and metadata/publication contract, LLVM pipeline/tools, runtime, and
the explicit zero-postprocessor baseline. Any legitimate change to one of those byte-affecting inputs
produces a different
`BackendProfileId`, `CompilationId`, and publication directory.

**Input contract:** Complete private target-qualified draft from the trusted
checker, exact source/tree binding, target profile and derivations, exact
schemas, and explicit resource profile for the base projection. Final
composition additionally receives `AcceptedCompilation`, the exact
`ValidatedBackendInvocation`, and canonical empty or verified overlay bytes.

**Output contract and established invariants:** Canonical length-delimited
bytes; no unknown, duplicate, missing, cyclic, unjustified orphan, out-of-range,
or cross-container record; explicitly unreachable source blocks remain covered
and classified; no editable status/completeness bit; every reference validated;
full bytes available whenever a digest is trusted.

The original in-memory checking/target-qualification path first projects the
canonical bytes and replays them through the same kernel. Only that same
invocation may then construct `AcceptedCompilation`; replaying arbitrary
serialized bytes later can return an audit result but never reconstruct the
lowering capability. `AcceptedCompilation` owns the private replay-decoded
payload, not the projector's draft, so a coherent encoder substitution cannot
make replay accept one program/layout while lowering another.

Artifact replay is mandatory and complete, not a best-effort inspection. Its
semantic input is the canonical artifact bytes alone; versioned decoder and
rule implementations are the audit program, not ambient producer state. It:

1. reconstructs and validates the embedded source binding, specification,
   schemas, target profile, and all canonical tables;
2. validates complete source/tree correspondence without reparsing source;
3. replays every resolution, symbolic type/call, template graph/SCC/FN-6,
   template semantic/effect coverage, concrete seed/instantiation/recheck,
   concrete call graph, type, constant, substitution, CFG, region/outlives,
   provenance, ownership, recursive storage-admissibility, join, loop, effect,
   target-layout/ABI/frame, check, cleanup, report, derivation, and coverage
   judgment in the same legality-before-expansion order; and
4. returns `ArtifactDecision::Accepted` or
   `ArtifactDecision::Rejected(ArtifactReason)` for every artifact within the
   supported schema and hard profile. Resource inability is a separate
   `ArtifactResourceFailure`, never acceptance.

The compiler may seal `AcceptedCompilation` or make `BaseArtifactBytes`
eligible for later staging only after this replay returns `Accepted`. A rejection of bytes just projected by
the checker is a compiler invariant failure. The same public audit procedure
may classify later untrusted bytes, but its return type has no path to the
private lowering capability. This satisfies artifact-decidable acceptance
while making no false claim of semantic independence.

There is a second mandatory byte-derived gate after overlay/report composition.
The originating-invocation final-envelope validator receives
`AcceptedCompilation`, the exact `ValidatedBackendInvocation`, the selected
overlay's
exact expected bytes and `OverlayId`, and candidate
`CompilationArtifactBytes`. It then:

1. canonical-decodes the candidate composite and recomputes `CompilationId`;
2. requires the embedded `BaseArtifactBytes` and `BaseId` to equal the bytes
   owned by `AcceptedCompilation`, not merely to decode to equivalent data;
3. requires the decoded `BackendInvocationProfileBytes` and
   `BackendProfileId` to equal the exact `ValidatedBackendInvocation` values and
   to be compatible with the accepted `CompilationTargetProfile`;
4. requires the decoded overlay bytes and `OverlayId` to equal the selected
   expected bytes/identity; validates the one canonical empty form or reruns
   the independent fact verifier's compile-authority API on those exact decoded
   bytes through a newly borrowed `CompilationFactContext`; any non-`Verified`
   outcome aborts this finalization without changing source acceptance;
5. rederives `ReportProjectionBytes` from the accepted replay payload and the
   newly decoded/validated overlay, then requires exact byte equality;
6. materializes every static report and runtime report template with the new
   `CompilationId`, canonical-decodes them, and requires every `artifact_hash`
   field and embedded report-table identity to equal that ID; and
7. constructs `FinalizedCompilation` only from the decoded composite views,
   exact composite bytes, recomputed identities, and sealed report bytes/tables.
   The pre-envelope overlay capability and report objects are discarded
   before CodegenPlan construction; no later formatter may substitute them.

This gate prevents the final encoder from making empty/different overlay or
report bytes staging-eligible while lowering an in-memory selection with more authority. A
public final-envelope audit performs the same byte-only base replay, overlay
validation, report reprojection, reference/identity checks, and returns only
`CompilationArtifactDecision`. `whitefoot-compilation` orchestrates it by
pairing the replay-produced `ArtifactAuditContext` with the decoded compatible
backend-profile view as a lifetime-scoped `FactEnvelopeAuditContext`, then
borrowing that context into `whitefoot-fact-verifier`'s audit-only API and the
report auditor. The nested result is `FactAuditOutcome`; its fact rejection,
resource inability, and auditor invariant failure remain distinct in the outer
audit outcome. `whitefoot-semantics` never depends on either later component.
The audit context cannot enter the compile-authority verifier API or escape
into an authority factory, so this audit cannot construct
`VerifiedOptimizationOverlay` or `FinalizedCompilation` for later untrusted
bytes. Code and the composite artifact become eligible for later output staging
only after the originating-invocation final gate accepts; Decision 15's
crate-private `PublicationReadyOutputSet` reaching the atomic-directory commit
inside `finalize_and_commit` is the sole publication.

**Explicit non-responsibilities:** The codec performs no semantic judgment. The
artifact is not an optimization overlay. Rejection diagnostics are not proof
records. Catalog provenance can affect release identity/mismatch reporting but
cannot affect semantic replay or lowering behavior.

**Why this stage owns the work / why adjacent stages do not:** The checker is
the first component with all derivations. The artifact boundary is the first
place an untrusted producer/cache can be checked and the last place before an
accepted capability. LLVM must not consume or repair it.

**Alternatives considered and rejected:** Serializing all checker maps leaks
algorithm accidents. A hash-only source/tree binding is incomplete. One
mutable artifact whose checks are rewritten by optimization loses base semantic
identity. A verifier-only semantic obligation would let the checker accept bad
source.

**Trusted assumptions and threat model:** Codecs and schema definitions are
judgment-free shared code. All lengths, tags, order, references, derivations,
coverage, and hashes in input bytes are hostile.

**Failure modes:** Checker rejection, target qualification rejection, base or
final encode resource failure, decode/artifact failure, base replay failure,
overlay failure, final-envelope mismatch, and publication failure remain
separate. Rejection of checker/finalizer-produced bytes is a compiler defect,
never a new language rejection. No base artifact is eligible for staging before
base replay; no composite artifact or code is eligible before final-envelope
validation; nothing is published before the final closed output set passes
Decision 15's sole commit boundary.

**Independent evidence required:** Golden canonical vectors; every truncation;
unknown/reordered/duplicate/missing records; reference and count overflow;
cycle and orphan mutants; source/tree mutation; every semantic proof mutant;
omitted-site, wrongly substituted, shallow-only, non-storage-positive, and
orphaned `StorableType` judgments; missing/noncanonical instance seeds, type-growing cycles accepted
past FN-6, template/concrete graph substitution, region-class/parent,
holder/loan, and provenance-summary/call mutations; base substitution,
backend/compiler identity substitution, overlay
substitution, report substitution, expected-versus-decoded overlay mismatch,
round-trip exactness, and reconstruction of DIAG-3 views.

**Resource and determinism bounds:** Precompute encoded length with checked
arithmetic; reserve fallibly; stream where possible; reject trailing bytes;
bound record counts, reference depth, strings, states, proofs, coverage, and
total bytes. Canonical order is defined per table, never inherited from a map.

**Dependencies on unresolved specification questions:** All parser/tree
blockers, all semantic blockers affecting a record family, retained-check
`proof_ref`, path-dependent lifetime projection, logical-stack attribution,
target/frame contract, and artifact/overlay report identity.

**Migration or foundation-audit consequences:** Keep `WFSOURCE` as source plus
specification only. Do not extend it into a checked envelope. Do not add
placeholder node/proof identities.

**Approval status:** The architecture is adopted. The final artifact schema and
any protected report expectation are separately guarded.

### Decision 12: target qualification and semantic-to-lowering boundary

**Decision:** Separate target-neutral semantic construction from
target-qualified acceptance. The first lowerable capability is produced only
after all required layout, ABI, frame, and target legality checks. Derive one
narrow, ephemeral `CodegenPlan`; do not add another semantic IR.

**Problem being solved:** OP-9 makes `sizeof(T)` and a frame ceiling acceptance
relevant. A target-neutral value mislabeled as finally verified and followed by a possibly failing
backend would misclassify a specified compile-time rejection. Conversely,
putting LLVM details in the source checker entangles portable semantics and one
backend.

**Specification and project constraints:** OP-9, DIAG-1/2/3, exact checked
arithmetic, explicit cleanup, one generic lowerer, retained mandatory checks,
and the declared target/runtime/LLVM TCB boundary all constrain this seam.

**Selected design:**

```text
semantic checker
  -> SemanticallyCheckedDraft
  -> target qualifier(CompilationTargetProfile)
  -> TargetQualifiedDraft
  -> BaseArtifactBytes
  -> mandatory same-kernel artifact replay
  -> AcceptedCompilation
  -> ValidatedBackendInvocation(exact BackendInvocationProfile bytes)
  -> canonical empty or exact-byte-backed verified optimization overlay
  -> candidate CompilationArtifactBytes
  -> mandatory decoded final-envelope validation and overlay revalidation
  -> CompilationArtifactBytes + CompilationId
  -> FinalizedCompilation
  -> CodegenPlan
  -> LlvmEmissionOutput
  -> GuardAuditedLlvmModule
  -> pinned LLVM runner + LlvmRunOutput
  -> GuardPresenceAuditedObject
  -> pinned linker + LinkRunOutput
  -> LinkBoundOutputSet
  -> whitefoot-publication::finalize_and_commit
     -> crate-private PublicationReadyOutputSet
     -> atomic directory commit
  -> CommittedOutputSet
```

`CompilationTargetProfile` is acceptance-affecting and has canonical bytes
covering target triple, LLVM DataLayout, endianness, integer/pointer widths and
alignments, aggregate layout rules, enum discriminants, calling convention and
ABI classification, maximum object/frame limits, zero-size policy, stack
alignment/probes, and runtime, allocator, trap, and private logical-frame ABIs.
Target qualification uses checked size/alignment arithmetic and records every
concrete layout and ABI decision.

`BackendInvocationProfile` is non-semantic but byte/output-affecting. Its
canonical bytes reference `TargetProfileId` and fix: target CPU and sorted
feature set; relocation and code models; object format; LLVM build identity and
the exact Whitefoot compiler/driver/emitter executable digest and build
manifest; `CodegenPlan` schema identity; canonical
operation-lowering-dossier/consumer-contract identity; LLVM text/object-audit and
link/final-output-audit contract/implementation identities; exact requested
output-set contract (logical names, formats, file kinds, canonical modes/metadata
policy and optional products; baseline postprocessor count is exactly zero);
approved publication-host/filesystem/sandbox profile identity, non-operational host-evidence schema, and
pinned downstream publication implementation boundary;
hashes of every invoked executable plus its interpreter/dynamic-loader and
library closure, or a reviewed static-link declaration; optimization level and exact versioned pass
pipeline; target-machine, floating-point, section, visibility, stack-probe,
debug-info, and deterministic-codegen options; assembler/linker identities,
closed argument vectors and link order; hashes and ABI versions of
runtime/allocator/startup objects; and the cleared/allowlisted subprocess environment.
It contains no "native," host-default, ambient environment, unspecified feature
setting, or unversioned lowering choice.

Before compilation, `whitefoot-backend-invocation` constructs a sealed
`InvocationObservationSet` itself. It first length-checks and decodes candidate
profile bytes under `BackendProfileResourceProfile`, accounting for every
record/string/path, argument/environment byte, input/output/postprocessor entry,
held handle, and validation step; baseline postprocessor count must be zero.
For the already running Whitefoot compiler,
the approved host must provide a launch-time `LoadedCompilerImageEvidence`
capability binding the actually mapped executable and loader/library closure;
reopening the launch pathname is forbidden, and a host without enforceable
loaded-image evidence is rejected. For every later tool executable,
loader/library dependency, and runtime/allocator/startup object, the validator opens
the source no-follow, requires stable regular single-link metadata, copies and
hashes the bounded bytes into a new exclusive driver-owned private byte blob,
rechecks source metadata, closes the writable destination, and seals an
immutable `PinnedLaunchImageSet`/`PinnedLinkInputSet` capability. The sandboxed
runner/linker may materialize only those owned bytes into new exclusive private
inputs; source paths and prior hashes are never execution/link inputs.
Caller-provided hashes or decoded observation structs cannot satisfy this
operation. It also validates the embedded build manifest, plan schema,
lowering-dossier identity, and output-set/publication-host contract against the
candidate profile and retains the exact pinned capabilities later runners use.
The target
compatibility validator
rejects any backend profile whose triple/DataLayout/ABI-affecting feature
premise disagrees with the accepted target profile. These identities and all
runtime/tool/output-contract hashes are embedded in the composite artifact and
publication manifest. Only this complete check constructs
`ValidatedBackendInvocation`; a successful canonical decode alone yields at
most `BackendProfileAuditView` and cannot be upgraded.
Consequently a legitimate emitter or lowering revision
changes `BackendProfileId` and `CompilationId` rather than colliding at the
existing publication directory.

Postprocessors are absent from the conservative baseline. Adding one later
requires a new backend-profile schema and an exact-byte capability stage that
owns its input and output, binds the prior stage/plan/compilation/profile, and is
the sole input to the following validator; a command named in a profile is not
enough.

The first production backend is safe Rust emitting deterministic textual LLVM
IR and invoking the pinned LLVM/assembler/linker toolchain as bounded
subprocesses with direct argument arrays, never a shell. It uses no in-process
LLVM FFI and therefore introduces no hidden `unsafe` Rust boundary. The emitted
module triple and DataLayout, every
subprocess-executable/hash/argument/environment tuple, and every linked object are checked against the two profiles; no
host discovery or default may fill a field.

`CodegenPlan` contains only already-accepted concrete functions, layouts, ABI
locations, deterministic symbols, explicit blocks/edges, selected target
instructions, retained checks and trap blocks, explicit cleanup, constants,
logical-report frames, sealed report-table/`CompilationId` references, and
independently authorized optional consequences. A
field-by-field audit must show that every field is target-specific or materially
simplifies emission.

DIAG-3 stack attribution uses an explicit logical frame chain, never a native
backtrace or debug-info heuristic. Every private Whitefoot ABI call carries a
hidden parent-frame pointer. Each invocation creates one fixed-layout stack
record containing only a target-width parent pointer, a `FunctionReportId(u32)`,
and a `CallerSiteReportId(u32)`. `main` uses a null parent and the reserved
`NO_CALLER = 0xffff_ffff`; no variable-length path or string is present in a
frame. A trap call supplies a fixed-width `TrapSiteReportId(u32)` plus the
current frame.

The finalized static report tables assign collision-free dense IDs in canonical
full-key order. The function table maps every admitted `FunctionReportId` to
one exact concrete function instance. The site table maps every admitted site
ID to one exact `{site kind, owning FunctionReportId, canonical NodePath}`;
`CallerSiteReportId` admits only call sites and `TrapSiteReportId` admits only
trap/check sites. Counts must be below `u32::MAX`, the reserved value has no
table entry, and canonical decode rejects gaps, duplicates, wrong owners,
wrong kinds, out-of-range IDs, or noncanonical order. These are dense indices,
not truncated hashes, so collision handling is unnecessary; the full identity
remains in the sealed table bytes.

The runtime emits the current `{function, trap node}` first, then walks outward;
for each child record it resolves that record's caller-site ID, requires its
owner to equal the parent frame's function ID, and emits `{parent function,
child's caller-call-site node}`. This proposed innermost-to-outermost order and
node meaning are blocked on A-15's normative report ruling.

The two ID widths, sentinel, logical-frame layout, hidden parameter,
report-table ABI, and record bytes are part of the target/runtime profile.
Target qualification includes the fixed logical-frame record and any
alignment/padding in every function's OP-9 frame total. Recursion naturally creates
another logical record; dynamic stack exhaustion remains an OS/runtime TCB
process failure because v0.8 defines no dynamic recursion limit or
stack-overflow trap. On a reachable trap, the runtime streams the complete finite
chain and resolves IDs against read-only sealed tables without heap allocation;
an invalid ID/table relation is a runtime/compiler-integrity failure, never a
changed language verdict.

The baseline backend emits no `tail`/`musttail`, sets the reviewed target option
that disables tail-call elimination, and uses a pass pipeline with no inliner.
Logical-frame stores/links remain observable to every trap/report call. Any
later inlining or tail-call transformation must preserve exactly the same
logical frame sequence and report bytes under its own closed verified
proposition, CodegenPlan consumer, and differential/mutant gate; native frame
shape is never normative.

Every accepted check site is classified before planning as either retained or
omitted by one exact verified-overlay consequence; an explicit source `check`
is always retained. Each retained site produces exactly one `GuardPlan` keyed
by its base `CheckSiteId` and fixed report-site ID. In the conservative backend,
each `GuardPlan` lowers through a versioned, class-specific external runtime
guard ABI. The optimizer sees only a declaration with observable effects—never
`readnone`, `memory(none)`, `speculatable`, or a runtime body—and the exact
runtime object is linked only after LLVM optimization with LTO disabled. The
guard traps with the sealed report identity on failure and otherwise returns
the validated operand or value that the protected partial LLVM operation must
consume. Where a safe LLVM intrinsic computes both value and failure flag, the
guard consumes that flag and the result remains defined independently. A
hazardous divide, remainder, shift, conversion, address formation, load, or
store may neither precede the guard nor use the original unvalidated operand;
its dossier must name the guard-result data dependency. An arbitrary explicit
source check with no protected operation is preserved by the observable guard
call itself.

The reviewed LLVM pipeline receives no runtime bitcode, summaries, or guard
implementation and contains no LTO or whole-program pass. `whitefoot-llvm-emit`
alone accepts `CodegenPlan`; its private `emit` operation moves the exact
complete emitted text into `LlvmEmissionOutput<'plan>`, bound to the plan,
`CompilationId`, and exact `ValidatedBackendInvocation`. Its temporary buffer
is never returned, and no public constructor accepts alternate text, bytes,
paths, digests, or receipts.

A structural LLVM-text validator, implemented separately from emission,
consumes only `LlvmEmissionOutput`. It requires a bijection between retained
`CheckSiteId`s and guard calls, validates report-site IDs and guard-result
dependencies, and rejects any unguarded partial operation before invoking LLVM.
On success it wraps the consumed emission result in
`GuardAuditedLlvmModule<'plan>`; thus the guard audit adds its narrower claim
without inventing complete CodegenPlan provenance. `whitefoot-llvm-run` accepts
only that capability and materializes exactly its owned bytes as the module
input. The same private supervised launch/wait/import
operation waits for the complete process tree, imports the named object through
the bounded no-follow protocol, discards the tool path, and constructs
`LlvmRunOutput<'plan>`. That capability owns the imported candidate bytes and
binds them to the exact audited module, pinned LLVM image and closure, profile,
plan, and compilation. No separate public constructor accepts bytes, a path, a
digest, or receipt.

The post-LLVM object audit consumes only `LlvmRunOutput`, requires the expected
guard references and site map, and on success wraps that exact run result in
`GuardPresenceAuditedObject<'plan>`. The linker runner accepts only that
capability as its Whitefoot object input together with the exact
`PinnedLinkInputSet`. Its private supervised launch/wait/import operation alone
can construct `LinkRunOutput<'plan>`, which owns the complete imported
linker-output candidate set and binds it to the audited object, pinned linker image and
closure, all pinned link inputs, profile, plan, and compilation. The output
audit consumes only `LinkRunOutput` and, after validating its closed output
set, wraps that exact run result in `LinkBoundOutputSet<'plan>`. A caller-supplied byte
slice, mutable path, digest, or reconstructed receipt cannot enter any of these
factories.

The object audit does not claim to reconstruct machine-code dominance or prove
guard-result dataflow. The exact pinned LLVM build and pass pipeline are
therefore an explicit semantic-preservation TCB boundary: they
must preserve the observable external call and data dependency already
validated in textual IR. The pinned linker and exact runtime object are TCB
components as well. The final output validator compares the retained/omitted
partition to the decoded verified overlay and consumes the exact
provenance/audit chain through `LinkBoundOutputSet`. On success it constructs
crate-private `PublicationReadyOutputSet` inside `finalize_and_commit`; LLVM's
own analysis is never Whitefoot proof
authority to omit a guard: only the pre-CodegenPlan verified-overlay path may
remove one from input IR. Differential execution, pass-erasure mutants, and the
limited object presence/site audit attack the LLVM TCB without pretending to
replace it. Later removal of this TCB assumption requires a target-specific
machine-code dominance/dataflow verifier or another closed machine-verified
check-preservation contract, plus a new backend-profile identity.

No operation-table row enters CodegenPlan/backend code until a version-bound
lowering dossier covers every admitted type/mode instance. Each dossier row
names operand evaluation order, guards, trap or `Result` construction, exact
LLVM instruction/intrinsic/libcall and version, poison/undefined-behavior
preconditions, NaN/signed-zero behavior, target-feature fallback, optional
facts that may remove a guard, and interpreter/code-shape/runtime mutants. The
matrix is generated from neither the production lowering dispatch nor the
facet catalog, and completeness is checked against the normative operation
inventory.

In particular, integer division **and remainder** guard divisor zero and the
signed minimum/-1 case before `sdiv`/`udiv`/`srem`/`urem`. OP-6 total conversion
pairs use only their exact total lowering; every partial integer-to-integer,
integer-to-float, float-to-integer, and float-to-float pair constructs
`Result` after an exact representability test. No potentially poison
float-to-integer instruction executes before finite/integral/range guards. Narrowing
float checks exact round-trip/value preservation with the specified infinity,
canonical-NaN, and signed-zero behavior. A conversion may use a reviewed helper
instead of LLVM IR, but that helper's exact object/hash/ABI and exhaustive
boundary evidence then belong to the backend profile and dossier.

After emission, the sole public terminal operation is
`whitefoot-publication::finalize_and_commit`, taking `LinkBoundOutputSet` and
`&FinalizedCompilation`. It canonical-decodes all exact linked bytes and
internally reprojects the static reports, runtime metadata, trap templates, and
serialized artifacts; no caller-supplied output byte slice is accepted. It
requires their `CompilationId`, report-table identity, target/backend profiles,
module triple, and DataLayout to match
`FinalizedCompilation`; and checks that every CodegenPlan trap site indexes the
sealed table entry for its accepted `CheckSiteId`/node. It also requires the
compiler/build, lowering-dossier, plan-schema, output-validator,
requested-output-set/publication-host, guard-ABI/runtime-object, LLVM pipeline, and
external-tool identities
to equal the decoded `BackendInvocationProfile`. Baseline postprocessor count
must be zero. Success moves the complete requested byte/metadata set and
canonical manifest into crate-private `PublicationReadyOutputSet`; failure
constructs no such value, and no staging directory is eligible for commit.
Without returning that intermediate, the same operation opens the configured
root without following links, revalidates the current root, same-filesystem and
host properties against the profile and non-operational
`PublicationHostEvidence`, materializes only the set's owned bytes and metadata
through crate-private handles, revalidates them, and performs Decision 15's
atomic-directory commit. No filesystem primitive, writable handle, or ready-set
accessor is a public input or output.

**Input contract:** The CodegenPlan builder receives only
`FinalizedCompilation`. That opaque value owns read-only accepted views of the
complete instances, exact target profile and layouts, chosen allowed overlay,
exact backend invocation profile, sealed reports/tables,
`CompilationArtifactBytes`, and `CompilationId`. No raw semantic/target draft,
tree, artifact decode result, or separately supplied target/backend profile is
an input.

**Output contract and established invariants:** Every target layout is finite,
in bounds, and ABI-valid; every semantic operation and edge has one lowering
plan; every retained check has exactly one non-elidable guard plan unless the
decoded verified overlay carries its exact authorized omission; every protected
partial LLVM operation consumes the guard-validated result; every symbol and
output order is deterministic; no semantic query remains for the emitter. The
emitter alone can construct `LlvmEmissionOutput` from `CodegenPlan`; guard audit
consumes only that result and can return only `GuardAuditedLlvmModule`. LLVM
optimization receives only the audited module's owned exact bytes and its
runner can return only `LlvmRunOutput`; object audit accepts only that result.
Linking receives only `GuardPresenceAuditedObject` plus pinned link inputs and
its runner can return only `LinkRunOutput`; output audit accepts only that
result. Every capability is bound to one plan, compilation, backend invocation,
and exact preceding inputs; raw drafts and tool paths are destroyed before the
next authority boundary. Final validation and commit are one downstream public
operation receiving only `LinkBoundOutputSet` and the exactly matching opaque
`FinalizedCompilation`; its `PublicationReadyOutputSet` is an internal state,
not a cross-crate capability. No baseline postprocessor, reconstructed receipt,
ready-set accessor, or externally held publication handle interrupts this
chain.

**Explicit non-responsibilities:** CodegenPlan does not resolve, type-check,
instantiate, compute effects, discover CFG, choose ownership joins, insert
semantic cleanup, prove checks, or accept source. LLVM emission does not repair
the plan.

**Why this stage owns the work / why adjacent stages do not:** Target
qualification is the first point with concrete types plus exact ABI. CodegenPlan
is the first point with accepted semantics plus a final overlay. Earlier stages
lack target facts; the emitter is too late to reject source or invent them.

**Alternatives considered and rejected:** Target-neutral final acceptance is
incoherent under OP-9. A generic optimization IR invites a second semantic path.
Direct ad hoc LLVM emission mixes planning, resource failure, and publication.
If CodegenPlan merely duplicates `FinalizedCompilation`, the separate plan
representation must be deleted and planning folded into the emitter without
weakening the emitter's sole input capability.

**Trusted assumptions and threat model:** Target/backend profiles, compatibility
validator, qualifier, plan builder, LLVM emitter/toolchain, runtime, allocator,
linker, filesystem, and OS remain named TCB components. A false layout, guard,
cleanup, attribute, ABI, CPU feature, pass, linked object, report frame, or
ambient tool option can cause miscompilation, undefined behavior, or a false
normative report.

**Failure modes:** Target-defined compile-time rejection occurs before accepted
capability. Unsupported/mismatched target profile is a toolchain failure.
Impossible plan state is a compiler failure. LLVM/linker failure is backend
failure. No candidate module/object/link/output is published; only a complete
crate-private `PublicationReadyOutputSet` inside `finalize_and_commit` can reach
the commit boundary.

**Independent evidence required:** Independent layout calculator; ABI fixtures
compiled from another language/toolchain; frame and aggregate boundaries;
logical-stack ID range/owner/kind/table and recursion/call-order mutants;
target/backend/compiler-build/plan-schema/lowering-dossier/output-set
substitution and ambient-environment perturbation; loaded-image/pathname,
tool-source mutation, hardlink/writable-fd, loader/library closure, arbitrary
guard-shaped LLVM/object/link candidate substitution, forged emission/runner
capability or receipt, mutable-staging, and postprocessor mutants; complete operation-lowering
dossier and inventory coverage; CodegenPlan interpreter; missing, duplicated,
rewired, hoisted, and pass-erased guard mutants; an independent LLVM-text
structure/dataflow audit; a deliberately narrower object
guard-reference/site-presence audit; LLVM verifier; differential execution; exact
`LlvmEmissionOutput`/`LlvmRunOutput`/`LinkRunOutput` provenance and
`LinkBoundOutputSet`/`PublicationReadyOutputSet` identity audits; and
poison/undefined-behavior hostile review.

**Resource and determinism bounds:** Explicit limits on backend-profile encoded
bytes, records, strings/paths, arguments/environment, linked inputs, requested
outputs, zero postprocessors, held handles and validation work; loaded/pinned
image bytes and hash work; type/layout depth, aggregate/frame bytes, ABI records,
functions, blocks, instructions, constants, report-table entries/bytes, retained
guards, symbols, LLVM/object/link/final-output bytes, external-tool output, and
time. Planning and guard validation are linear/log-linear in accepted artifact
plus output size.

**Dependencies on unresolved specification questions:** Frame limit, recursive
layout, evaluation order, cleanup, target inventory, artifact/overlay identity,
path-dependent lifetime reporting, logical-stack attribution, and report
composition.

**Migration or foundation-audit consequences:** Add a narrow target contract
before layout code. Do not call a target-neutral value `Accepted` or `Verified`.

**Approval status:** The architecture is adopted. The concrete frame, target,
and acceptance contracts remain separately gated.

### Decision 13: required semantics versus optional optimizer facts

**Decision:** Put all information required for correct execution in
`AcceptedCompilation`. Put only authority-increasing propositions in a separately
verified `OptimizationOverlay`. Define the baseline as the canonical empty
overlay, never as "facts off."

**Problem being solved:** The old term `facts-off` can sound as if types,
ownership, effects, or checks are disabled. Combining optional facts with the
`AcceptedCompilation` also lets optimizer bugs affect source acceptance.

**Specification and project constraints:** Checks remain on unless a
machine-verified proof authorizes removal; explicit `check` is never elided;
fact channels require hostile review; optional facts cannot select another
lowerer.

**Selected design:** The base unit always carries required semantics and check
sites; only an exact-unit-bound opaque verified overlay can grant one closed
optional lowering consequence.

For check elimination, a candidate may reference only a stable checker
`DerivationId` already present in `BaseArtifactBytes` for that `CheckSiteId`.
The independent fact verifier validates that candidate and the final DIAG-3
eliminated-check `proof_ref` is exactly the base `DerivationId`; an overlay
cannot invent or renumber it. If no such base derivation exists, the check
remains. The meaning/value of `proof_ref` for a retained check is still a
recorded specification gap, so no final report schema or elimination family may
land until the successor rule resolves it. If the owner instead chooses an
overlay-local proof reference, that is a specification/design revision, not an
implementation shortcut.

**Input contract:** The verifier has two non-interchangeable APIs. The
compile-authority API receives `CompilationFactContext<'_>`, formed only
through the fact-contract validator from a narrow fact view borrowed from
originating `AcceptedCompilation` plus that invocation's exact
`ValidatedBackendInvocation`. The audit-only API receives
`FactEnvelopeAuditContext<'_>`, which pairs `ArtifactAuditContext` with
the canonical decoded, target-compatible backend-profile view from the
envelope under audit. Both have private fields and no unchecked constructor;
their validating constructors require exact base/target/backend equality and
the appropriate unforgeable authority or audit capability borrows. They are
not serializable, storable, or interchangeable. Both APIs also receive a
fact-specific resource profile and canonical candidate bytes containing the
exact `BaseId`,
a closed proposition tag, exact scope and operands, optional base
`DerivationId`, assumptions, proof bytes, producer/schema identities,
invalidators, exact `TargetProfileId`/`BackendProfileId`, and requested closed
consumer consequence. Candidate-carried IDs are compared with the context;
they never supply the expected profile authority.

**Output contract and established invariants:** The independent family verifier
uses the same closed proposition judgments behind two output boundaries:

- `verify_for_compilation(CompilationFactContext<'_>, bytes,
  FactResourceProfile) -> FactVerificationOutcome` returns exactly one of
  `Verified(VerifiedOptimizationOverlay)`, `Rejected(FactReason)`,
  `ResourceFailure(FactResourceFailure)`, or
  `InvariantFailure(FactInvariantFailure)`. Only `Verified` contains an opaque
  exact-byte-backed overlay bound to the complete base, target/backend profiles,
  fact schema, and consumer matrix.
- `audit_fact_bytes(FactEnvelopeAuditContext<'_>, bytes,
  FactResourceProfile) -> FactAuditOutcome` returns
  `Decision(FactAuditDecision::{Accepted, Rejected(FactReason)})`,
  `ResourceFailure(FactResourceFailure)`, or
  `InvariantFailure(FactInvariantFailure)`. It has no conversion to
  `VerifiedOptimizationOverlay` and cannot be consumed by finalization or
  CodegenPlan.

Every record has exactly one proposition and authorized consequence. Unknown
facts or consumers reject. Originating final-envelope validation reruns the
compile-authority API over the exact decoded overlay and discards the
pre-envelope capability. Public envelope audit uses only the audit API, so neither
lowering authority nor published bytes can be derived from public artifacts.

The initial allowed proposition families may include proved bounds, no
overflow, resolved-place disjointness, and separately approved laws. Each is
introduced only after its exact schema, verifier, invalidation, LLVM
consequence, mutants, and measured value are owner-reviewed.

**Explicit non-responsibilities:** An overlay cannot change parsing, source
acceptance, types, CFG, effects, drops, ABI, explicit checks, or runtime-visible
results. It cannot contain arbitrary LLVM attributes, metadata, or IR snippets.

**Why this stage owns the work / why adjacent stages do not:**
Proposition-specific verifiers are small enough to guard the exact new authority. The
source checker need not prove optional performance facts, and the backend must
not trust producer assertions.

**Alternatives considered and rejected:** Embedding facts in
`AcceptedCompilation` makes acceptance depend on optimization. A boolean
optimization level gives no
proposition-level authority. Separate optimized lowering paths drift
semantically. Trusting checker facts without independent verification violates
the fact-channel rule.

**Trusted assumptions and threat model:** Each fact verifier and closed consumer
mapping joins the TCB for that family. False facts may remove traps or introduce
LLVM poison, so near misses and scope mutations are first-class attacks.

**Failure modes:** Before envelope composition, a missing proof, producer
failure, `Rejected`, or `FactResourceFailure` discards the optional candidate
and selects the canonical empty overlay; none rejects source. A
`FactInvariantFailure` is a compiler failure and stops the build rather than
being hidden by fallback. Once a verified overlay has been selected and encoded,
any rejection, resource inability, or invariant failure during mandatory
decoded final-envelope revalidation aborts that finalization without staging
eligibility;
an optional driver retry must restart composition explicitly with the canonical
empty overlay. Public audit returns rejection, resource inability, or auditor
invariant failure as those separate families; resource inability never becomes
`FactAuditDecision::Rejected`. Both empty and verified successful builds produce
the same observable program behavior.

**Independent evidence required:** Proposition mutants, scope/target/artifact
and backend-context rebinding, invalidation tests, exact/one-over resource
classification, verifier-invariant injection,
initial-fallback/final-revalidation/public-audit outcome mutants, consumer code-shape inspection,
facts-present versus empty-overlay differential execution, and performance
measurement.

**Resource and determinism bounds:** `FactResourceProfile` has per-family limits
on facts, proof bytes, dependency edges, verifier work, and authorized output
changes. Exact/one-over exhaustion returns `FactResourceFailure` in both APIs;
canonical fact order is fixed; no solver is admitted without a separately
bounded and verified result contract.

**Dependencies on unresolved specification questions:** FN-4 law admission,
retained `proof_ref`, final artifact composition, and later Phase-6 owner gates.

**Migration or foundation-audit consequences:** Keep the current capability
overlay outside production dispatch and empty. Rename user-facing modes to
`empty optimization overlay` and `verified optimization overlay`.

**Approval status:** The separation is adopted as part of this architecture;
each fact family requires its own later hostile review and approval.

### Decision 14: diagnostic and failure authority

**Decision:** Use closed, stage-qualified outcome families. Only a stage that
has a canonical tree may issue a normative language rejection requiring a
`NodePath`. Artifact, backend, and resource failures never masquerade as source
rejection.

| Outcome | Location | Rule attribution | Serialization/authority |
|---|---|---|---|
| source-envelope failure | input record/path/byte offset | none | deterministic tool/input result |
| lexical observation | source ordinal and byte span | none before spec repair | internal/evidence only |
| parse issue / syntax rejection | source/token coordinate and expected terminal set | no invented verdict before repair; exact approved syntax rule afterward | toolchain/specification-blocked observation before the pre-tree DIAG repair; normative syntax rejection afterward |
| specification blocker | discrepancy IDs and exact evidence | none | deterministic toolchain stop |
| canonical-source rejection | canonical node path plus source span | exactly one FORM/GRAM rule | normative after tree/location rules close |
| semantic rejection | node path, span, optional nested-place segment | exactly one rule and required fix/restructuring | normative, byte-stable |
| target qualification rejection | node path and target-profile field | exact approved rule | normative only after frame/target rule closes |
| resource failure | stage, limit, maximum, actual, nearest available coordinate | none | non-normative, retryable under a larger allowed profile |
| compiler invariant failure | stage and stable internal failure code | none | compiler defect; no source verdict |
| artifact/replay/fact-verifier failure | artifact or fact record/path/index and failure code | none | untrusted artifact/fact or compiler defect |
| backend/toolchain failure | target, stage, tool status, bounded output | none | build failure |
| runtime trap | DIAG-3 primary node plus explicit logical-frame chain (conditional on A-15) | exact originating rule/node/canonical logical call attribution | normative runtime report |

**Problem being solved:** A parse failure has no canonical node path; artifact
replay does not reproduce the producer's helpful source diagnostic; a backend
failure says nothing about language validity.

**Specification and project constraints:** DIAG-1 requires one rule and stable
node path; DIAG-3 fixes report fields; pre-tree attribution is currently a
recorded gap.

**Selected design:** The closed outcome table above defines the owner, location,
rule authority, and serialization status of every failure family.

**Input contract:** The exact capability available at the failing stage and a
closed deterministic failure code. No later-stage identity is fabricated.

**Output contract and established invariants:** One primary outcome family per
invocation. When multiple normative defects are known, the compiler selects one
by fixed stage, source, node-path, rule, and access-segment order. Diagnostic
bytes contain no absolute host paths, timestamps, addresses, random IDs, map
order, or unbounded tool output.

For a whole-unit rule with no offending source declaration, such as a missing
`main`, the canonical location is root `NodePath` plus
`Location::BundleRoot(BundleRootExtent)` and no byte span. A duplicate or
conflicting declaration instead cites the deterministically selected source
declaration node. No child inherits a synthetic cross-source span.

**Explicit non-responsibilities:** Artifact replay does not duplicate syntax or
semantic producer diagnostics. Resource failure does not mean `Unsupported`. A
compiler bug does not cite a language rule. Lexical observation is not
acceptance authority.

**Why this stage owns the work / why adjacent stages do not:** The detecting
stage has the narrowest truthful location and cause. Translation by a later
stage loses or invents information.

**Alternatives considered and rejected:** Fabricated root paths violate DIAG-1.
Reparsing during artifact replay duplicates syntax work and violates the
artifact-only contract. Broad catch-all rejection hides incompleteness. Raw
panics and stderr are nondeterministic.

**Trusted assumptions and threat model:** Diagnostic formatting cannot grant
semantic authority, but misleading classification can weaken gates. External
tool output is hostile and bounded.

**Failure modes:** Failure while serializing a diagnostic becomes a controlled
tool/resource failure with a small fixed fallback code, never a panic or partial
normative record.

**Independent evidence required:** Golden bytes, insertion-order permutations,
pre-tree cases, nested-place segments, every failure family, multi-defect
ordering, huge external stderr, and report/artifact-hash binding.

**Resource and determinism bounds:** Limits on messages, expected terminals,
paths, frames, related records, fixes, and external output. Ordering is explicit
and serialization is fallible and atomic.

**Dependencies on unresolved specification questions:** DIAG-1 pre-tree path,
DIAG-3 retained `proof_ref`, path-dependent lifetime projection,
`stack_attribution` order/node meaning, target/frame rejection, and artifact
composition.

**Migration or foundation-audit consequences:** Keep existing lexical outcome
separation. Do not expose it through a normative conformance adapter as a
language verdict.

**Approval status:** The architecture is adopted; DIAG-1 and DIAG-3 changes
remain separately specification/protected-surface gated.

## Conservative backend contract

The baseline lowerer emits no authority-increasing LLVM flag, attribute,
metadata, or unreachable assumption unless it follows from a required accepted
semantic/target fact or an independently verified optional proposition whose
consumer contract names that exact consequence.

Retained safety and explicit checks additionally follow Decision 12's external
guard contract. Merely placing a branch in pre-optimization LLVM is insufficient:
the exact guard call, fixed report-site identity, and guard-result dependency
must pass the reviewed textual-IR audit; the narrower object audit confirms only
the expected guard reference and site presence. The pinned LLVM pipeline is
trusted to preserve the audited call/dataflow semantics. No `llvm.assume`,
poison, undefined behavior, runtime bitcode visibility, or optimizer-derived
range fact may stand in for the input guard. An omitted input guard must be
accounted for by the exact decoded verified-overlay consequence before
CodegenPlan construction.

| False or over-broad fact | LLVM hazard | Baseline rule |
|---|---|---|
| signed/unsigned operation cannot overflow | `nsw`, `nuw`, or poison-dependent transforms | Use defined wrapping operations or explicit overflow intrinsics/guards; add flags only through a verified proposition. |
| divisor is nonzero or signed division/remainder cannot overflow | `sdiv`/`udiv`/`srem`/`urem` poison or undefined behavior | The class-specific guard validates zero and signed-minimum with minus one and returns the operands consumed by division/remainder; exact `Result` modes use a defined lowering that never executes the partial instruction on the error path. |
| shift amount is in range | poison from out-of-range shift | Mask for `.wrap`; for `.trap`, consume the amount returned by the retained range guard before emitting the shift. |
| index is in bounds | out-of-object pointer formation, early `inbounds`, or bad load/store | Consume the index/offset returned by the bounds guard, then form/dereference the target pointer; avoid `inbounds` without an exact proved allocation proposition. |
| any partial OP-6 conversion is exactly representable | poison, rounding, truncation, lost signed zero, or wrong NaN from integer/float conversion | Use the complete conversion dossier: check integer ranges and exact round trips, guard float-to-int before conversion, and implement float narrowing/NaN/infinity/signed-zero rules exactly before constructing `Ok`; otherwise construct `Err(NarrowError())`. |
| two pointers are disjoint | invalid `noalias`/alias scopes and reordered memory effects | Emit no alias authority in the empty overlay; only a verified resolved-place proposition may authorize a closed consumer. |
| pointer is nonnull/dereferenceable/aligned beyond ABI | invalid parameter attributes or speculative access | Emit only target-ABI-required properties proved by construction; optional stronger properties require verified scope-specific facts. |
| value/padding is initialized | `noundef`, widened loads, or reading padding | Initialize and move fields explicitly; never read padding; do not emit `noundef` without a proved representation contract. |
| enum discriminant is valid | `unreachable` default and impossible-case transforms | Construct valid internal values; validate gated inputs; keep a trapping/defensive default unless the exact provenance proves validity. |
| function is pure | incorrect `memory(none)`, speculation, or elimination | Whitefoot `pure` alone grants none of these. Preserve calls absent the exact EFF-3 conditions and a termination proof for unused-call elimination. |
| function terminates or makes progress | `willreturn`, `mustprogress`, speculative execution | Emit none without a separately verified termination/progress contract. |
| FP reassociation is harmless | fast-math changes NaN, signed zero, rounding, and traps | Emit strict FP with no fast-math flags; use the exact required intrinsics and operations. |
| cleanup is unnecessary | leaks, double frees, or use after free | Lower the accepted edge's explicit `ExitPlan` exactly once; trap edges perform none. |

The baseline also forbids unearned `exact`, `assume`, `range`, alias-scope,
`nonnull`, `dereferenceable`, `nocapture`, `nofree`, `nosync`, `speculatable`,
lifetime, invariant, and similar authority. ABI-mandated attributes such as a
reviewed `sret`, `byval`, or alignment record come only from target
qualification, not source effects. An audited noreturn trap routine may end in
`unreachable` only after the call under its exact runtime ABI.
