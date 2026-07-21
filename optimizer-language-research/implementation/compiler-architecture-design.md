# Production Compiler Architecture

Status: OWNER-APPROVED ARCHITECTURE WITH BLOCKERS. This is the design record;
`THE-PLAN.md` remains the sole roadmap and implementation authority. The owner
directed repository handoff to this architecture on 2026-07-21. No numbered
specification, protected semantic surface, profile, schema, or entrance gate is
approved merely by adopting the architecture. Production parser, semantic,
artifact-replay, and lowering work remains paused wherever this dossier names a
blocker.

Exact source under review: `spec/kernel-spec-v0.8.md`, SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.

This dossier answers the eighteen decisions required by
`compiler-architecture-preflight.md`. It is deliberately honest about places
where v0.8 does not define one implementable language. A detailed compiler
architecture cannot turn a missing language rule into an implementation
choice.

## Dossier map

This index and the following four family files form one architecture review
bundle. Each numbered decision has exactly one owning file; the index owns the
cross-stage authority chain, blockers, execution order, prohibited routes,
owner decisions, and exit checklist.

- [Frontend and Source](compiler-architecture-frontend.md): Decisions 2–6 and
  the proposed successor-specification compilation-unit rule.
- [Semantic Kernel](compiler-architecture-semantics.md): Decisions 7–10.
- [Artifacts and Backend](compiler-architecture-artifacts-backend.md):
  Decisions 11–14 and the conservative LLVM contract.
- [Systems and Evidence](compiler-architecture-systems.md): Decisions 15–18.

The files are reviewed and revised as one bundle. A cross-file contradiction is
a stop condition; the index does not override a family decision silently.

## Executive decision

The production architecture is **GO WITH BLOCKERS**. Implementation of the
disputed boundaries is **NO-GO**.

The decisions that are ready are:

1. Keep exact source bytes and the shape-only lossless lexer.
2. After the grammar is repaired in a successor numbered specification, use a
   typed, iterative, predictive production parser. Before approval, run a
   separately runnable grammar-change verifier over the exact current
   specification bytes and exact non-authoritative proposal bytes; it combines
   a complete static
   nullable/`FIRST_2`/`FOLLOW_2`/predictive-relation audit with a separately
   extracting bounded generalized-parser oracle. The engines share no derived
   grammar table, and neither is a production dependency.
3. Build one immutable grammar derivation tree. Use typed postorder construction,
   one linear topology finalizer, and a tree-driven canonical-source audit over
   that same tree; there is no copied, vaguely "validated AST." The concrete
   program-root extent, diagnostic location, node-kind mapping, multi-file
   contract, and `CanonicalSyntaxUnit` factory remain blocked on their separate
   language and entrance gates.
4. Symbolically check every function declaration, run the finite template-call
   graph/FN-6 legality gate before monomorphization, then concretely recheck a
   closed, explicitly seeded instance set before constructing concrete CFGs;
   target-independent template CFG/semantic coverage for every source function
   separately protects unused and cross-kind callees.
5. Put ownership states, every normal and trap edge, checks, drops, releases,
   effects, substitutions, and instantiation closure in the checked semantic
   representation. The backend may not rediscover those decisions.
6. Bind language acceptance to an explicit `CompilationTargetProfile`, because
   target layout and the frame limit can cause a required compile-time
   rejection.
7. Derive an ephemeral `CodegenPlan` only from `FinalizedCompilation`, which
   already owns its bound target/backend profiles. It may schedule and select
   target instructions; it may not add semantic facts or change source
   acceptance.
8. Keep optional optimizer propositions in a separate, independently verified
   overlay. An empty overlay preserves all core semantic information and all
   mandatory checks.
9. Give each stage a closed outcome family, explicit resource profile,
   deterministic order, and failure-atomic publication boundary.
10. Keep production authority independent of the facet catalog, capability
    overlay, corpus identities, project names, and archived implementations.

The design rejects a separate production semantic verifier. Whitefoot's
explicit types, regions, substitutions, control-flow states, effects, and
cleanup make checking deliberately deterministic; independently validating all
of them still requires nearly the same visibility, substitution, ownership,
join, cleanup, effect, layout, and closure predicates. A proposed
production-schema experiment would have to implement those families twice
before choosing the architecture, recreating the two-compiler dead end.

The selected route is one trusted semantic kernel plus mandatory artifact-only
replay through that same kernel. Replay makes DIAG-2 acceptance decidable from
the canonical artifact and detects projection, omission, corruption, and codec
defects. It is explicitly shared trusted authority, not independent semantic
evidence, and it can never construct a lowering capability for later untrusted
bytes. Independent verifiers remain mandatory for every optional optimizer fact
because those facts grant new authority-increasing consequences. Rejecting a
second production *semantic certificate* verifier does not remove the other
independent evidence lanes: grammar analysis and its separately extracting
oracle, source/tree models, conformance, target/backend differentials, hostile
mutants, policy guards, and publication tests remain mandatory where assigned.

This selection requires explicit owner approval as the proposed replacement for
the independent-semantic-verifier part of D22. A narrow architecture ruling
does not itself revise `THE-PLAN.md`; the existing roadmap remains authority and
production semantic implementation remains paused until the later, separately
reviewed roadmap redecision lands.

## Architecture at a glance

The selected production path is:

```text
ordered SourceBundle + SpecificationIdentity + CompilationTargetProfile
  -> lossless shape lexer
  -> version-bound terminal classifier                 [currently blocked]
  -> typed predictive parser                           [currently blocked]
  -> finalized DerivationTree
  -> tree-driven canonical-source audit
  -> CanonicalSyntaxUnit
  -> declaration inventory and use resolution
  -> SymbolicallyTypedUnit(all function declarations and template calls)
  -> finite TemplateCallGraph + FN-6 SCC gate
  -> RecursionQualifiedTemplates
  -> all-source-function template CFG/ownership/provenance/effect coverage
  -> TemplateSemanticCoverage
  -> finite concrete-instance closure and mandatory concrete rechecking
  -> ConcreteTypedUnit + ConcreteCallGraph
  -> explicit ConcreteControlFlowUnit
  -> owner-approved provenance qualification
  -> region/ownership/cleanup/effect checking
  -> SemanticallyCheckedDraft
```

```text
trusted semantic kernel + checker + target qualifier
  -> private target-qualified draft
  -> canonical DIAG-2 artifact projection
  -> mandatory same-kernel replay of the projected artifact
  -> AcceptedCompilation(replay-reconstructed payload, exact base artifact bytes)
  -> ValidatedBackendInvocation(exact BackendInvocationProfile bytes)
  -> canonical empty or exact-byte-backed verified optimization overlay
  -> CompilationArtifactBytes(base + backend + overlay + report projection)
  -> mandatory final-envelope validation from those exact bytes
  -> FinalizedCompilation(reconstructed composite views)
  -> derived target-bound CodegenPlan
  -> exact-byte-backed LlvmEmissionOutput from the conservative emitter
  -> exact-byte-backed GuardAuditedLlvmModule
  -> pinned LLVM runner + exact-byte-backed LlvmRunOutput
  -> exact-byte-backed GuardPresenceAuditedObject
  -> pinned linker + exact-byte-backed LinkRunOutput
  -> exact-byte-backed LinkBoundOutputSet
  -> downstream publication final validation + internal runtime metadata
  -> crate-private PublicationReadyOutputSet
  -> atomic directory commit
  -> CommittedOutputSet

later replay of serialized artifact
  -> audit result only (cannot reconstruct a lowering capability)

later validation of serialized compilation envelope
  -> final-envelope audit result only (cannot reconstruct FinalizedCompilation)
```

The optional optimization path is deliberately separate from source
acceptance:

```text
candidate OptimizationOverlay bound to the checked artifact, target, and backend
  -> canonical candidate bytes
  -> proposition-family verifier
  -> exact-byte-backed VerifiedOptimizationOverlay
  -> final-envelope projection and decoded-overlay revalidation
  -> the same lowerer and CodegenPlan builder
```

## Authority, trust, and threat model

The v0.8 guarantee already places the compiler, checker, runtime, allocator,
and OS in the trusted computing base. Same-kernel artifact replay does not
remove or reduce that TCB. Its narrower purpose is to make the published
artifact self-contained and artifact-decidable, and to catch projection,
omission, codec, reference, coverage, and corruption defects before the
originating invocation acquires lowering authority. Semantic-kernel logic bugs
are attacked by the independent evidence lanes in Decision 16, not by calling
the same judgments twice.

The following inputs are untrusted at their boundaries:

- source paths and bytes;
- serialized source, syntax, checked-artifact, backend-profile, final-envelope,
  and fact-overlay bytes;
- all producer-supplied indices, lengths, counts, states, proofs, and digests;
- compiler caches and outputs from alternative producers;
- LLVM and linker diagnostics and generated objects when used as test
  observations.

The following never establish authority by themselves:

- a digest match;
- a Rust type name containing `Canonical`, `Checked`, or `Verified`;
- a facet or capability claim;
- a producer-supplied completeness bit;
- a successful decode;
- a passing corpus slice; or
- a report emitted by the component whose claim it is meant to verify.

Authority is granted only by a private constructor after the complete boundary
check succeeds. Lowering accepts only `FinalizedCompilation`. Serialized
artifacts never re-enter production lowering.

## Entrance gates and unresolved language decisions

The current discrepancy registry remains the machine-readable source of truth.
Its fifteen records already block complete v0.8 release: affine-deref storage
lifecycle; the pre-tree DIAG-1 node path; retained-check `proof_ref`; EFF-1 row
canonicality; body-local region effects; contract-member semantics; law
admission; `main` return spelling; `requires` rule attribution; protected
FORM-2 spacing; the FORM-4 cross-reference; FORM-5/FORM-7 float spelling; the
terminal/`IDENT` partition; the GRAM-1/GRAM-7 node-kind conflict; and dotless
operation reservation.

Architecture review found additional questions that must be registered with
the same exact-evidence discipline before their affected implementation lands:

| ID | Question | Why architecture cannot choose silently |
|---|---|---|
| A-01 | TYPE-6 declaration-before-use versus FN-1/FN-6 mutual recursion | Owner-selected 2026-07-21: all top-level function signatures are visible throughout the closed compilation unit; locals, regions, labels, and named constants remain declaration-before-use. Exact successor-specification encoding remains guarded. |
| A-02 | Disposition of an affine value overwritten by `set` | Dropping, leaking, rejecting, or requiring a prior move produce different ownership and storage semantics. |
| A-03 | Evaluation order of multiple atoms, fields, and call arguments | Bounds checks, traps, borrows, and moves can make order observable. LLVM lowering cannot choose it. |
| A-04 | Ordinary lexical-scope, match-arm, and loop-iteration cleanup | STOR-3 names region-exit edges but does not completely state all owner-scope exit behavior needed for exact drop plans. |
| A-05 | Finite layout for recursive nominal types | Direct recursive values need a rejection rule; indirection-breaking rules are not stated. |
| A-06 | Frame-limit value and target binding | OP-9 requires a compile-time rejection but gives neither a limit nor its target/profile contract. |
| A-07 | TYPEID collision between nominal types and globally unique variant constructors | The grammar uses the same lexical class while TYPE-6 states only variant-to-variant uniqueness. |
| A-08 | Affine liveness at continuing control-flow joins | The ownership state lattice and any conservative rejection rule must be explicit before proofs or cleanup are frozen. |
| A-09 | Semantic checking of source statements after a diverging terminator | EFF-2 scans the complete declaration and artifact coverage cannot omit source nodes, but v0.8 does not define the ownership entry state or rejection behavior of an unreachable suffix. |
| A-10 | Normative multi-file compilation-unit formation | v0.8 defines one `program` but not how an ordered source bundle becomes that program; file order changes TYPE-6/FN-1 acceptance, while boundaries/root extent affect identities and diagnostics. This requires a numbered successor-specification rule, not an architecture-only "toolchain contract." |
| A-11 | Mutability and alias claim of `slice_of` results | `slice_of` takes a shared borrow and returns `own slice<'r, T>`, while OP-4 calls `index` on slices a read/write place. The rules do not say whether writes are forbidden, require unique provenance, or how the slice keeps its source loan live. |
| A-12 | Cross-function provenance of returned borrows and slices | FN-1 says signatures contain everything callers need, but a return region does not identify which of several same-region parameters a returned view may reference. Caller alias checking needs an owner-approved signature restriction/annotation or exact finite-origin rule. |
| A-13 | Recursive enforcement of STOR-5 region-free storage | The prose forbids region-carrying stored values, but generic/nested instantiation can form types such as `box<slice<'r, T>>` or a generic field instantiated with `slice<'r, T>` unless a recursive well-formedness judgment is normative. |
| A-14 | DIAG-3 lifetime report for path-dependent destruction | The schema has one singular `drop node_path` per binding, while a binding may be dropped on several normal edges, moved on other paths, or have no dynamic drop site. Report composition cannot invent the mapping. |
| A-15 | DIAG-3 `stack_attribution` order and frame-node meaning | The schema says only “frame list: function, node_path”; it does not say inner/outer order or whether a frame path names the active trap, caller call site, or function declaration. Lowering cannot make native unwind order normative by accident. |
| A-16 | Holder/loan identity after borrow copy, move, and projection | OWN-1 makes shared borrows copy values, while OWN-6 defines a holder only as the binding initialized by a `borrow_expr`; moved unique borrows, parameters, return/`give` transfer, and OWN-13 borrow-mode match binders create the same missing binding-to-claim rule. The successor specification must say whether a copy creates a new holder/claim linked to one loan, shares a claim, or uses another exact model; how every transfer updates it; and how a match binder projects a claim while remaining ineligible as an OWN-6 child-reborrow parent. |
| A-17 | Region-bearing generic type arguments and portable instance identity | A type argument may contain `slice<'r, T>` or `arena<'r, T>`. Embedding the caller's `RegionRef` in an instance key makes identity owner-dependent/non-portable and duplicates equivalent instantiations; erasing it without a rule can lose acceptance-relevant region facts. The successor specification must define which equality/outlives facts a concrete generic body may use. Architecture then alpha-normalizes nested regions to finite slots for semantic instantiations and keeps baseline code identity one-to-one; any later region-erased body sharing needs verified whole-class equivalence. |
| A-18 | FN-6 after const-generic reintroduction | FN-6 quantifies over cycles among “generic functions” but requires each call to use the caller's own “type parameters.” Grammar `generics` now also contains const parameters. A mixed cycle such as type-generic `f<T>` through const-only `h<const N>` has no unambiguous kind/arity comparison. The successor specification must define the cycle domain and exact type/const argument-vector relation before the pre-instantiation gate or diagnostic schema lands. |

The recursive-effects corner receives a proposed exact-v0.8 disposition rather
than being hidden. EFF-2 literally counts a call to any function whose declared
row contains an effect. Therefore a mutually recursive SCC can syntactically
exhibit `traps` only through its own declared rows. The checker should implement
that literal declared-row rule if the owner confirms it. A separately computed
grounded-effects view is non-authorizing evidence. Requiring a grounded or least
fixed point is a successor-specification change, not an optimizer decision.

The recommended successor-specification and owner decisions are:

1. Make every fixed lowercase grammar terminal ineligible for `IDENT`; do not
   use ordered choice or a parser-local keyword subset.
2. Resolve whether the two `match` grammar productions have one node kind or
   distinct production node kinds.
3. Give pre-canonical-tree failures a source-coordinate location form instead
   of requiring a nonexistent canonical node path.
4. Encode the owner-selected A-01 rule in the exact successor proposal: make
   top-level function signatures visible throughout the closed unit while
   preserving lexical declaration-before-use for locals, regions, labels, and
   named constants.
5. Add the multi-file closed-unit contract below to the successor numbered
   specification: an ordered nonempty sequence of FORM-2 files, with declaration
   order and one logical program root defined normatively.
6. Confirm literal declared-row effect exhibition, or specify a grounded rule.
7. Define target/frame-profile acceptance and the additional semantic questions
   A-02 through A-09 and A-11 through A-18.
8. Approve the exact A-10 successor-specification delta, including zero-item
   file and zero-source-bundle behavior.
9. Resolve every existing discrepancy through its listed authority; none is
   waived by this design.

Until the terminal partition, node-kind conflict, and pre-tree diagnostic gap
are resolved, a production parser, canonical node path, resolution schema, or
checked-artifact schema would freeze invented semantics and must not be built.
For the terminal partition specifically, resolution also requires the
Decision 2 grammar-change report: exact v0.8 must reproduce `many` for the
registry's exact `deref(p)` witness and the separately registered proposal case
`deref(x)`; the reviewed proposal must produce `one` through the
`expr/atom/place/pbase` fixed-`deref` alternative, remove the named conflict,
and introduce no unreviewed conflict. That verifies the proposed transition,
not whole-grammar readiness. The verifier requires the dossier's pinned v0.8
hash and binds the full proposal hash. The exact approved successor
specification must equal those reviewed proposal bytes, produce a
conflict-free complete static audit, and pass the bounded oracle domain before
parser work. The report is evidence for the separately owner-gated
specification decision; it is not the decision itself.

## Decision records

### Decision 1: representations and authority

**Decision:** Use the representations in the following table and no unnamed
semantic pass-through structure.

| Representation | Form and authority | Constructor and consumer |
|---|---|---|
| `SourceBundle` | Ordered logical paths plus exact raw bytes; lossless, internal; canonical source-binding encoding is serializable but not a language verdict. | Ingress validates limits and paths; lexer and artifact binding consume it. |
| `LexedBundle` | Complete byte partition into shape tokens and trivia; internal, lossless, source-bound; no keyword or grammar authority. | Shape lexer constructs; classifier and evidence adapters consume. |
| classified token view | Specification-versioned membership of token bytes in fixed terminals and lexical classes; internal. | Grammar layer constructs after the terminal partition is approved. |
| `DerivationTree` | Immutable typed production tree with terminal leaves; internal; punctuation retained, trivia remains in the tape; no recovery nodes. | Parser builder constructs, finalizer validates. |
| `CanonicalSyntaxUnit` | Opaque capability over the same finalized tree, exact sources, spec, and canonical-source audit; no copied AST. | Syntax finalizer constructs; resolver consumes. |
| lexical resolution tables | Scope tree, declarations, lexical uses, and exact lexical targets; internal checker state and serializable artifact data, but not independently authoritative. | Resolver constructs; typed checking consumes. |
| typed member-resolution tables | Field/member/label uses resolved from an already checked base type, constructor, function signature, or contract; internal and later serializable. | Type checker constructs; semantic checking consumes. |
| `SymbolicallyTypedUnit` | Internal, non-accepted coverage of every function declaration under its kind/contract environment, with immutable `TemplateTypedCallRecord`s and no graph, provenance, instance, or target claim. | Symbolic type checker constructs; template graph/FN-6 gate consumes. |
| `TemplateCallGraph` | Finite source-declaration call edges and deterministic SCCs derived only from template typed calls; contains no concrete instance key or provenance solution. | Effect/graph component constructs; FN-6 gate and provenance scheduling consume. |
| `RecursionQualifiedTemplates` | Opaque internal pairing of `SymbolicallyTypedUnit` with complete template graph/SCC evidence after every cyclic generic edge passes FN-6; not whole-semantic acceptance. | Effect/graph component's pre-instantiation gate constructs. |
| `TemplateControlFlowUnit` | Finite target-independent symbolic CFG/value/place/region structure for every source function after FN-6, including zero-parameter functions, with no concrete layout or whole-semantic authority. | Template CFG builder constructs from `RecursionQualifiedTemplates`; template provenance/ownership/effect checks consume. |
| `TemplateSemanticCoverage` | Opaque internal receipt that every source function has symbolic CFG, region, provenance, ownership, cleanup-obligation, body-to-contract, and effect coverage under its declared environment; target and concrete-only judgments remain absent. Nominal types, contracts, and const items have their own kind/member/storage/const-dependency coverage and never enter this function-CFG receipt. | Semantic coordinator constructs from Decision 8/9 template results; instance closure consumes. |
| `ConcreteTypedUnit` | Canonical finite set seeded by every function with zero type/const monomorphization parameters—including region-only functions—and closed over admitted type/const generic calls after both template gates. A `SemanticInstanceRef` uses alpha-normalized nested region slots and the A-17-approved finite fact profile; baseline `CodeInstanceRef` is a one-to-one typed wrapper around it. Every semantic instantiation is rechecked, but has no provenance or CFG authority. | Instance checker constructs; CFG and concrete-call-graph builders consume. |
| `ConcreteCallGraph` | Finite concrete-instance edges/SCCs derived from immutable concrete typed calls; schedules provenance/effect work but does not assert either result. | Effect/graph component constructs; provenance, effects, and replay consume. |
| `ConcreteControlFlowUnit` | Immutable complete structured CFG, places, value links, calls, returns, and edge/source coverage for every concrete instance; no ownership or return-provenance acceptance. | CFG builder constructs; provenance qualification consumes. |
| `ProvenanceQualifiedUnit` | `ConcreteControlFlowUnit` plus immutable complete function summaries and call-provenance records under the one owner-approved A-11/A-12 rule; typed calls are not mutated. | Provenance checker constructs from CFG plus `ConcreteCallGraph`; ownership consumes. |
| `OwnershipCheckedUnit` | Provenance-qualified CFG plus exact region/outlives judgments, ownership states/transitions, joins, and normal-edge cleanup; still not whole-semantic or target acceptance. | Ownership/cleanup checker constructs; semantic coordinator consumes. |
| `SemanticallyCheckedDraft` | Target-neutral template coverage, types, modes, constants, substitutions, resolved places, instances, CFG, provenance, regions, ownership, effects, checks, cleanup, reports, derivations, and coverage; private, transient, and not accepted. | Trusted semantic coordinator constructs only after all named semantic capabilities close; target qualifier consumes. |
| `CompilationTargetProfile` | Opaque validated view over canonical acceptance-affecting target bytes: layouts, ABI, frame/object ceilings, and runtime contracts; its identity is embedded in the base artifact. A decoded candidate alone has no authority. | Target-contract validator constructs; target qualifier and later profile-compatibility checks consume. |
| `TargetQualifiedDraft` | `SemanticallyCheckedDraft` plus complete concrete layouts, ABI, frame decisions, target derivations, and target coverage; private, transient, and not lowerable. | Trusted target qualifier constructs; artifact projector consumes. |
| `BackendInvocationProfile` | Canonical non-semantic identity of every Whitefoot compiler/lowering, LLVM/codegen/link/runtime, requested-output-set/metadata, and publication-host input that can change emitted bytes, executable behavior, or required machine/filesystem features; serializable and publication-bound, but cannot change source acceptance. | Release configuration supplies candidate bytes; `whitefoot-backend-invocation` validates. |
| `ValidatedBackendInvocation` | Opaque originating-invocation view over exact backend-profile bytes after target compatibility and sealed observations of running compiler/tool/runtime bytes, output contract, publication host, and sandbox support validate; it also owns the pinned executable/input capabilities and non-operational host evidence later stages require. It is not source authority. | `whitefoot-backend-invocation` alone constructs; compilation, fact contexts, plan/backend, and publication borrow. |
| `BackendProfileAuditView` | Canonical decoded profile plus target-compatibility audit result; contains no live tool/host capability and cannot become `ValidatedBackendInvocation`. | `whitefoot-backend-invocation` audit API constructs; final-envelope/fact audit borrows. |
| `PinnedLaunchImageSet` / `PinnedLinkInputSet` | Opaque immutable owners of exact bounded tool/loader/library or runtime/startup/link bytes copied into driver-exclusive storage after hash/profile validation; the current compiler instead has OS-backed evidence for its already loaded image. Paths and prior hashes cannot launch or link. | `whitefoot-backend-invocation` constructs inside `ValidatedBackendInvocation`; runners/linker borrow. |
| `PublicationHostEvidence` | Opaque non-operational evidence that the configured host/profile supports the required sandbox, filesystem, rename, and durability contract at backend-validation time. It contains no writable handle and exposes no create/write/rename operation; publication reopens and revalidates the exact root under private handles at commit time. | `whitefoot-backend-invocation` constructs inside `ValidatedBackendInvocation`; downstream publication borrows only for comparison. |
| `BaseArtifactBytes` | Canonical DIAG-2 serialization of the complete target-qualified draft and derivation records; mandatory same-kernel replay input and later audit material, never a lowering input. | Projector constructs only after whole-unit and target success; replay consumes. |
| `ArtifactDecision` | `Accepted` or a closed artifact reason for supported bytes, with resource inability separate; not serializable authority and never lowerable. | Same-kernel replay constructs. |
| `ArtifactAuditContext` | Opaque, non-lowerable read-only facts reconstructed during accepted public replay; sufficient only for audit decisions. It is not source/target/optimizer authority and no authority factory accepts it. | Semantic replay constructs internally and lends only through the audit orchestrator. |
| `ReplayedAcceptedPayload` | Private semantic/target representation decoded and reconstructed from replay-accepted base bytes; all later lowering-visible data comes from it. | Private compilation replay constructs; public audit never exposes it. |
| `AcceptedCompilation` | Opaque pairing of `ReplayedAcceptedPayload` with exact replay-accepted `BaseArtifactBytes` and `BaseId`, after exact invocation source/spec/target binding comparison; sole source/target acceptance capability. | Trusted semantic component constructs only in the originating invocation after mandatory replay; original drafts are discarded. |
| `OptimizationOverlayCandidate` | Serializable optional proposition/proof records bound to exact `BaseId`, target, and backend profile; no authority. | Fact producer constructs; family verifier consumes. |
| `EmptyOptimizationOverlay` | One canonical serializable no-fact form for an exact base/target/backend tuple; grants no check removal or target authority. | Compilation finalizer supplies by default. |
| `FactAuditDecision` | Audit-only `Accepted` or closed fact reason over bytes checked with `FactEnvelopeAuditContext`; never convertible to an overlay capability. | Independent fact verifier's audit API constructs. |
| `CompilationFactContext<'a>` | Non-serializable lifetime-scoped pairing of a narrow fact view borrowed from originating `AcceptedCompilation` with the exact `ValidatedBackendInvocation`; the only fact context that may mint optimizer authority. Fields are private, and its validating constructor requires both unforgeable capability borrows and exact base/target/backend equality. | Fact-contract owner validates; originating compilation forms and fact verifier borrows. |
| `FactEnvelopeAuditContext<'a>` | Non-serializable lifetime-scoped pairing of `ArtifactAuditContext` with the canonical decoded and target-compatible backend-profile audit view; never authority-producing. Private fields and a validating constructor prevent substitution. | Fact-contract owner validates; public final-envelope audit forms and fact verifier borrows. |
| `FactVerificationOutcome` | Closed originating outcome: `Verified`, `Rejected`, `ResourceFailure`, or `InvariantFailure`; only `Verified` carries an overlay capability. | Fact verifier's compile-authority API constructs. |
| `FactAuditOutcome` | Closed audit outcome wrapping `FactAuditDecision` or a distinct `ResourceFailure`/`InvariantFailure`; never carries an overlay capability. | Fact verifier's audit API constructs. |
| `VerifiedOptimizationOverlay` | Opaque exact-byte-backed result of proposition-family verification through `CompilationFactContext`; never source-acceptance-bearing. | Independent fact verifier's compile-authority API constructs; pre-envelope value is discarded after final-envelope validation. |
| `CompilationArtifactBytes` | Canonical acyclic composite of base bytes, backend-profile bytes, chosen overlay bytes, and final report projection; serializable candidate output, never independently lowerable. | Compilation projector constructs; final-envelope validator consumes. |
| `CompilationArtifactDecision` | `Accepted` or a closed final-envelope reason for supported composite bytes, with resource inability separate; audit only and never lowerable. | Final-envelope validator constructs. |
| `FinalizedCompilation` | Opaque exact-byte-backed view reconstructed from validated `CompilationArtifactBytes`, paired with the originating `AcceptedCompilation` only after exact base/backend binding, decoded-overlay revalidation, report reprojection/materialization, and identity checks; owns sealed report tables/templates and is the sole CodegenPlan input. | Compilation component's originating-invocation final-envelope validator constructs. |
| `CodegenPlan` | Ephemeral, target-bound, mechanically derived lowering schedule, layouts, ABI classifications, symbols, and explicit check/trap blocks; not serialized semantic authority. | Plan builder constructs only from `FinalizedCompilation`; LLVM emitter borrows. |
| `LlvmEmissionOutput<'plan>` | Private provenance capability owning the exact complete LLVM text produced by the conservative emitter from one `CodegenPlan`, with borrows binding its compilation and validated backend invocation. It proves trusted-emitter provenance, not retained-guard correctness. | `whitefoot-llvm-emit` constructs only inside `emit(&CodegenPlan)`; guard audit accepts it as its sole candidate input. |
| `GuardAuditedLlvmModule<'plan>` | Private lifetime-scoped capability wrapping the consumed `LlvmEmissionOutput` after the independent retained-guard validator accepts its exact bytes, thereby preserving the originating `CodegenPlan`, `CompilationId`, emitter, and `ValidatedBackendInvocation` binding. | `whitefoot-llvm-audit` constructs; pinned LLVM runner accepts as its sole module input. |
| `LlvmRunOutput<'plan>` | Private provenance capability owning the exact bounded object-candidate bytes imported by the supervised pinned-LLVM invocation itself, bound to its `GuardAuditedLlvmModule`, launch-image/profile identity, plan, compilation, and backend invocation. It proves which invocation produced the bytes, not object correctness. | `whitefoot-llvm-run` constructs only inside its launch/wait/import operation; object audit accepts it as its sole candidate input. |
| `GuardPresenceAuditedObject<'plan>` | Private capability wrapping the consumed `LlvmRunOutput` after its exact object bytes pass the narrower guard-reference/site audit, thereby preserving the originating audited module, pinned LLVM invocation, plan, compilation, and backend binding. | `whitefoot-object-audit` constructs; pinned linker accepts as its sole Whitefoot object input. |
| `LinkRunOutput<'plan>` | Private provenance capability owning the complete exact bounded linker-output candidate set imported by the supervised pinned-linker invocation itself, bound to its `GuardPresenceAuditedObject`, every pinned link input, linker image/profile, plan, compilation, and backend invocation. It proves provenance, not final-output validity. | `whitefoot-link` constructs only inside its launch/wait/import operation; output audit accepts it as its sole linker-output candidate. |
| `LinkBoundOutputSet<'plan>` | Private capability wrapping the consumed `LinkRunOutput` after its exact closed linker outputs validate, thereby preserving the audited Whitefoot object, every pinned runtime/startup/link input, plan, compilation, and backend binding; a reconstructed receipt or staged path has no authority. | `whitefoot-output-audit` constructs only from `LinkRunOutput`; `whitefoot-publication::finalize_and_commit` accepts it as its sole linked-output input. |
| `PublicationReadyOutputSet<'plan>` | Crate-private closed-set owner of every requested output byte plus canonical path/kind/mode/metadata and manifest, retaining the consumed `LinkBoundOutputSet` provenance after final cross-output validation. Baseline has no postprocessors. It never appears in a public signature or crosses a crate boundary. | `whitefoot-publication` constructs only inside `finalize_and_commit` and consumes before that operation returns. |
| `CommittedOutputSet` | Opaque result that one exact `PublicationReadyOutputSet` was installed or matched byte-for-byte at its profile-bound final directory after revalidation and required sync. It adds durable-installation identity, not semantic or lowering authority. | `whitefoot-publication` constructs only at the end of `finalize_and_commit`; consumers still validate the committed directory contract. |
| LLVM/runtime output | Deterministic exact-byte-bound LLVM text, object/link output, runtime metadata, and report templates; compiler output, not proof of source correctness. | Backend validates; downstream publication commits atomically; external tools consume. |

**Problem being solved:** Earlier plans used `validated AST`, `checked unit`, and
`verified` without enough distinction, allowing a digest, decode, or wrapper to
claim more than it proved.

**Specification and project constraints:** FORM-1/GRAM-1 require one canonical
form and tree; DIAG-2 requires explicit derivations; archived compilers and
catalog metadata cannot supply production semantics; lowering may not consume
an unaccepted representation.

**Selected design:** The representation and authority table above is the
selected pipeline contract.

**Input contract:** Each constructor receives the exact previous capability,
the same specification identity, a compatible resource profile, and where
needed the same compilation target profile.

**Output contract and established invariants:** Every capability names exactly
the checks it completed. No representation contains placeholders, poison,
recovery, unknown variants, cross-bundle handles, or partial whole-unit output.

**Explicit non-responsibilities:** Digests locate content; they do not validate
it. The lexical tape does not parse. The tree does not resolve. The artifact
codec does not check semantics. The CodegenPlan does not infer semantics.

**Why this stage owns the work / why adjacent stages do not:** Each boundary is
the first point with all required inputs and the last point before a consumer
would otherwise trust the claim. Earlier stages lack information; later stages
would duplicate or silently assume it.

**Alternatives considered and rejected:** A separate normalized AST duplicates
the exact one-spelling tree without a specified normalization. A parse forest
has no normative selector. A generic untyped IR moves semantic checking into
the backend. Digest-only bindings permit collision or stale-content authority.

**Trusted assumptions and threat model:** Safe Rust memory safety, the trusted
semantic kernel and replay path, exact declarative language tables, and the declared
runtime/LLVM boundary remain trusted as stated. Every serialized byte is
hostile.

**Failure modes:** Input, source-form rejection, language rejection, resource
failure, invariant failure, artifact failure, and backend failure remain
distinct. Construction publishes nothing on failure.

**Independent evidence required:** Codec mutants, cross-source handle mutants,
tree/tape mutations, missing/duplicate semantic records, capability-constructor
API tests, and an attempted lowering call from every non-authoritative type.

**Resource and determinism bounds:** All tables have explicit ceilings; all
portable order is canonical; no hash iteration, address, thread schedule, or
ambient path enters output.

**Dependencies on unresolved specification questions:** Terminal partition,
GRAM-1/GRAM-7, DIAG-1, multi-file composition, and target/frame binding.

**Migration or foundation-audit consequences:** Keep source and lexer types;
narrow names that imply whole-program verification; freeze no new portable
identity yet.

**Approval status:** Adopted by owner directive D25. Concrete syntax, semantic,
artifact, target, resource-profile, and publication contracts remain subject to
the entrance gates named below.

## Handoff execution order

Owner directive D25 adopts this dossier and authorizes the repository handoff.
It does not approve a numbered-specification or protected-surface edit. Work
must proceed in this order:

1. **Durable architecture and foundation record.** Preserve the lexical
   observer checkpoint, land this dossier and the exact Decision 18 audit, and
   reconcile the roadmap, design memory, instructions, and repository status.
2. **Foundation preparation.** Keep the working source and lossless-lexer
   foundations, narrow names and claims that overstate their authority, and
   make owned source and source-binding allocation failures explicit. Create no
   placeholder parser, semantic, artifact, backend, or publication layer.
3. **Grammar verifier, evidence registration, and language proposals.** Build
   the separately runnable Decision 2 verifier without adding a production
   parser dependency.
   First reproduce the exact-v0.8 baseline. Then register
   A-02 through A-18 questions only after exact source/protected evidence
   review, encode the owner-selected A-01 rule in exact non-authoritative
   successor-specification proposal bytes, and run the relevant transition
   reports. Each numbered-spec edit retains the normal advance approval and
   version-bump protocol.
4. **Successor-specification approval.** Present exact version-bumped candidate
   bytes, evidence, protected-surface census, owner-visible delta, and baseline
   procedure. Do not implement a parser against an unapproved interpretation.
5. **Production pipeline.** Only after each named entrance gate closes, land
   the syntax, semantic, replay, lowering, and publication tranches in
   `THE-PLAN.md` order, with their independent evidence in the same changes.
5. **Frontend evidence before parser.** Rerun both independent grammar engines
   over the exact approved successor-specification bytes and require the
   complete static and bounded-oracle parser gate. Only then build the typed
   predictive parser, finalizer, renderer, and canonical syntax capability.
6. **Semantic kernel in ordered vertical capabilities.** Build
   declaration/resolution, then symbolic template types/calls, the finite template call
   graph and pre-instantiation FN-6 gate, all-source-function template semantic coverage,
   finite seeded concrete-instance closure/rechecking, concrete CFG/call graph,
   provenance, region/ownership/cleanup, and effects in that order. Each tranche
   handles arbitrary legal names, list lengths, nesting, source order, and graph
   shape and lands with independent evidence. A required language rejection may
   not be postponed behind a potentially expanding worklist. No corpus/function
   dispatch.
7. **Target qualification, artifact projection, and replay as one acceptance
   seam.** Close exact target/layout/frame records, project the complete base
   artifact, replay it from bytes through the same kernel, compare invocation
   bindings, discard the producer draft, and seal `AcceptedCompilation` only
   from the replay-decoded payload.
8. **Conservative baseline lowering and publication.** Compose the canonical
   empty overlay, close the complete version-bound operation-lowering dossier,
   project and validate the final byte envelope, derive one CodegenPlan, emit
   LLVM/runtime/report outputs, validate staged identity/report bytes, and prove
   semantic/runtime differential behavior and atomic-directory publication end
   to end.
9. **Optional facts only after the complete baseline.** Add one closed
   proposition family at a time with its own verifier, consumer, mutants,
   differential, and measured benefit.

No later step may be used to fill time while an earlier authority boundary is
blocked. Architecture-independent evidence may continue; placeholder semantic
code may not.

## Explicitly prohibited routes

- No exact-v0.8 parser using keyword priority, ordered choice, invented reserved
  words, semantic disambiguation, or archived behavior.
- No production ambiguity forest without a normative selector. The generalized
  parser is a bounded evidence oracle only.
- No grammar-change report is specification authority. Its two evidence
  engines may share exact specification/proposal bytes, but not an extracted
  grammar, terminal-classification result, predictive table, or production
  parser table while claiming independence.
- No canonical node, path, declaration, proof, or artifact ID before the real
  representation and its unresolved rules exist.
- No `ValidatedAst`, `VerifiedArtifact`, `Complete`, or similar wrapper whose
  constructor has not checked the full named claim.
- No parser recovery nodes, poison declarations, placeholder types, partial
  checked units, or fallthrough `Unsupported` over an implemented rule.
- No function-name, signature, ordinal, AST digest, project, corpus, facet, or
  capability-metadata dispatch in production semantics or lowering.
- No source import or active dependency from archived compilers.
- No two production semantic checkers and no checker plus verifier that share
  soundness-bearing algorithms while claiming independence.
- No public decode/audit result, cached artifact, or third-party artifact can
  acquire lowering authority. Only the originating invocation's mandatory
  replay may seal `AcceptedCompilation`, and it lowers the replay-decoded
  payload rather than the producer draft.
- No backend resolution, type inference, ownership join, effect closure,
  instantiation discovery, drop insertion, check proof, or source acceptance.
- No optional fact that affects acceptance, explicit checks, semantic identity,
  or selects a second lowerer.
- No sharing one emitted body across semantic instances in the baseline. A
  region-erased `EmissionShapeKey` is not equivalence; any later sharing must
  verify every lowering-visible field and selected-overlay consequence for the
  complete class.
- No LLVM poison/UB authority from source `pure`, type shape, nominal identity,
  or a producer assertion without the exact accepted or verified proposition.
- No retained check is represented only by an ordinary optimizable branch or
  protected only by pass convention. The conservative backend uses the exact
  guard ABI and guard-result dependency, validates full structure/dataflow in
  textual IR, audits guard reference/site presence in objects, and names pinned
  LLVM semantic preservation as TCB in Decision 12; only a decoded
  verified-overlay consequence may authorize omission from input IR.
- No raw LLVM candidate bytes may reach the guard validator or optimizer. The
  emitter alone creates `LlvmEmissionOutput` from `CodegenPlan`, and guard audit
  can only wrap that exact result; guard-shaped arbitrary IR cannot acquire plan
  provenance. No unaudited object path/bytes may reach the linker. The
  exact-byte-backed audited capabilities are the only runner/linker inputs. Their
  supervised launch/wait/import operations privately create `LlvmRunOutput`
  and `LinkRunOutput`; auditors cannot bind arbitrary imported bytes to an
  expected invocation. No reconstructed receipt or mutable staged path may
  reach final validation/publication. `LinkBoundOutputSet` enters only the
  downstream publication component's `finalize_and_commit` operation;
  `PublicationReadyOutputSet` is a crate-private intermediate that never crosses
  a public API, and every filesystem primitive remains private.
- No baseline postprocessor. A later one requires a newly profiled exact-byte
  input/output capability stage and private validating factory before it may
  affect an output.
- No changing, deleting, weakening, or regenerating protected tests to make the
  compiler pass; no numbered-spec edit without exact advance approval and
  version bump.
- No infallible or unbounded collection/output/process path inside a claimed
  resource-bounded authority boundary.
- No direct publication-root mutation from the driver or any stage other than
  `whitefoot-publication`. Production source/dependency policy rejects raw
  filesystem, descriptor, platform, and FFI access outside named OS-boundary
  components; tool runners receive only root-relative private-work-directory
  capabilities that cannot address the publication root.
- No arbitrary file-size rule, forwarding-only crate, broad utility drawer, or
  duplicate walker. Split files by cohesive invariant and reviewability.
- No progress metric based on function, corpus, target, or closed-facet count.
  Progress is an end-to-end semantic capability with its evidence and authority
  boundary.

## Hostile-review record and preserved dissent

The review used distinct parser/specification, semantic/backend,
verifier-skeptic, proof-verifier-advocate, Rust/resource/testing, and hostile
integration roles.

- The parser review returned **GO WITH BLOCKERS**. It rejected every exact-v0.8
  terminal-priority workaround, rejected a production parse forest, selected a
  bounded generalized oracle, and found the TYPE-6/FN-6 forward-call conflict.
- The semantic/backend review first returned **NO-GO** for an independent
  semantic verifier and selected one trusted kernel. It also exposed target
  qualification, explicit CFG/cleanup, and false-fact-to-LLVM hazards.
- The strongest verifier-advocate round constructed the closed local
  certificate alternative preserved in Decision 10. It judged independent
  validation plausible for SCC and instance closure, credible for resolution
  and types, and unproved for ownership, joins, reborrows, and cleanup. Its
  strongest requested evidence would bind a second producer/verifier pair to
  the complete real schema. The integration review rejected that route because
  obtaining the evidence would already implement the soundness-bearing
  language twice before architecture selection—the shadow-compiler dead end
  this design is meant to avoid.
- The Rust/resource/testing review selected the one-kernel route today, found
  infallible allocation in the existing source/binding foundation, rejected a
  one-file atomic-rename claim for a multi-output compilation, and required
  artifact replay to remain mandatory but non-independent. Decision 15 now
  distinguishes admitted fallible allocation from supervised process failure
  and makes atomic per-invocation directory installation the commit protocol.
- The hostile integration review also selected one kernel today, and found two
  cross-stage defects in earlier pipeline sketches: target-neutral final
  acceptance contradicts OP-9's frame rejection, and runtime `artifact_hash`
  must bind the final empty-or-verified overlay report projection.
- Parser/specification closure rounds found incomplete authority domains in the
  first synthesis: unused and zero-parameter functions lacked template semantic
  coverage; OWN-10 storage-root classes were not total; copied/moved borrow
  holders, nested regions in generic arguments, and FN-6's type/const cycle test
  lacked exact rules. Decisions 7–10 now contain conservative architectures for
  these questions, and A-16 through A-18 keep implementation blocked until the
  successor specification supplies the language rule.
- Semantic/backend and Rust-systems closure rounds repeatedly returned **NO-GO**
  on exact-byte provenance. They found unpinned loaded/tool/runtime inputs,
  unbounded backend-profile decode, raw external-tool output rebinding, raw
  guard-shaped LLVM rebinding, vague caller-supplied static outputs, and a
  cross-crate filesystem-capability/ready-set visibility bypass. Decisions 12,
  15, and 17 now require
  immutable copied inputs, explicit predecode ceilings, the ownership-preserving
  `LlvmEmissionOutput`/runner/output chain, internal static-output reprojection,
  and a downstream publication component whose only public operation consumes
  `LinkBoundOutputSet` plus the exact opaque `FinalizedCompilation`. Its ready
  set and filesystem primitives remain private, while source/API policy rejects
  publication-root mutation elsewhere. The final independent capability and
  dependency reviews returned **GO** on that closed chain and found no remaining
  raw byte/path/receipt route or cycle.

The synthesis does not turn those findings into a vote. The verifier advocate's
strongest alternative is preserved rather than caricatured, but the selected
architecture is one kernel because the proposed proof of a complete second
semantic implementation costs the architecture it was supposed to justify.
That decision can be revisited only if the product threat model later admits
third-party or cached artifacts to code generation. All reviewers agree that
current production semantic implementation is NO-GO until the owner rules on
this design and its blockers.

## Recorded owner rulings and remaining gates

The owner adopted the architecture and repository transition through D23-D25.
That adoption is not approval to edit a numbered specification, protected
expectation, guarded baseline, concrete schema/profile, or to cross a named
entrance gate. Those decisions remain deliberately separate.

### Rulings recorded on 2026-07-21

1. **Approved design direction — Decision 3/4 construction strategy only:**
   one typed postorder derivation tree, one linear topology finalizer, no copied
   AST, and a tree-driven canonical-source audit. "No copied AST" means no
   second syntax tree; it does not prohibit later semantic tables, CFGs, or
   checked representations. The audit reconstructs the expected source bytes
   from the tree and compares them with the input; it neither normalizes nor
   rewrites source. This is a binding design direction only. It does not
   authorize implementation or approve an exact-v0.8 parser,
   `BundleRootExtent`, `Location::BundleRoot`, node kinds or paths, multi-file
   behavior, formatting rules, diagnostic locations, portable identities,
   schemas, or construction of `CanonicalSyntaxUnit`.
2. **Approved design direction — D22 redecision:** replace only the
   independent production semantic certificate verifier with one trusted
   semantic kernel plus mandatory complete artifact-only same-kernel replay
   before the originating invocation can construct lowering authority. Replay
   reconstructs the lowering payload from canonical bytes and is shared trusted
   authority, not independent evidence. Later, cached, third-party, and
   hand-authored bytes can never construct lowering authority. Independent
   grammar, source/tree, conformance, model, target, backend, guard,
   publication, and optional-fact evidence remain mandatory. This ruling
   selects only the trust and authority topology. D25 separately authorizes the
   roadmap and design-memory transition. It does not
   approve Decision 10's named types, record families, replay order, schemas,
   APIs, resource profiles, crate placement, migration, or implementation.
3. **Approved preparation only — possible successor specification:** review the
   separately runnable Decision 2 grammar-verifier
   design and prepare discrepancy evidence, a protected-surface impact census,
   and exact non-authoritative candidate bytes. Exact v0.8 remains the active
   target. This approves no verifier implementation, normative bytes, parser,
   expected verdict, oracle digest, successor version, or active-target change.
   The leading evidence candidate is `IDENT` equal to the lowercase-word
   language minus the complete mechanically extracted set of fixed lowercase
   grammar terminals. That means all 47 such terminals, not only `deref` or the
   13 spellings with known complete ambiguity. This does not select normative
   wording or approve that compatibility restriction.
4. **Authorized and completed — Decision 18 read-only audit:** bind it to an exact manifest
   of the current workspace bytes. It inventories and recommends a disposition
   for every crate, module, authority-bearing API, dependency/feature, reachable
   allocation path, and publication path. It performs no edit, rename,
   refactor, deletion, new-crate creation, wire change, A-10 change,
   specification/protected change, capability claim, profile choice, roadmap
   update, or implementation migration. Every later mutation needs its own
   reviewed approval.

### Still separately gated

Exact terminal/`IDENT`, node-kind, pre-tree diagnostic, A-02 through A-18, the
exact guarded encoding of owner-selected A-01, and other discrepancy deltas are
not approved by the rulings above. For each one,
first present complete proposed numbered-specification bytes and hash, the
required evidence report, the protected-surface impact census, and the exact
owner-visible delta. Only after explicit owner approval may the new numbered
version and any guarded bytes be created, `make approve-spec` record the new
baseline, and the governance record and commit land. A-10, recursive effects,
concrete resource/target/backend/host profiles, foundation migration, the final
roadmap, and the design tree likewise remain separate later decisions.

### Scope of the recorded rulings

The construction strategy and trust topology are binding design directions;
the possible successor-specification evidence plan is approved for preparation
only; and the exact-workspace foundation audit is read-only evidence. Every
exclusion in the packet remains in force. Exact v0.8 remains the active target
until a separately approved successor and target switch. D25 authorizes the
roadmap, design-memory, and foundation handoff described above, but no
specification, protected expectation, guarded baseline, concrete production
schema/profile, or implementation beyond an open entrance gate.

## Preflight exit checklist

- [x] All eighteen architecture decisions have a concrete selected design or a
  precise blocking dependency.
- [x] The architecture selects one trusted semantic kernel plus mandatory
  artifact replay and rejects a second production semantic verifier.
- [x] The owner selected the proposed future trust topology; D22 and
  `THE-PLAN.md` remain current until a separate roadmap redecision.
- [x] The owner approved evidence preparation for a possible successor
  specification; exact v0.8 remains active and tool implementation is separate.
- [ ] Multi-file composition, zero-item files, and zero-source behavior (A-10)
  are owner-ratified in a successor numbered specification.
- [ ] Slice authority, return provenance, and recursive storage well-formedness
  (A-11 through A-13) are owner-ratified.
- [ ] DIAG-3 lifetime and logical-stack projection (A-14/A-15) are
  owner-ratified.
- [ ] Borrow holder/loan identity across copy, move, call, return, `give`, and
  match-binder projection (A-16) is owner-ratified before its transition or
  artifact schema lands.
- [ ] Region-bearing generic type semantics and finite portable
  `SemanticInstanceRef` (A-17) are owner-ratified before concrete instance
  closure or its artifact schema lands; baseline code identity stays one-to-one.
- [ ] FN-6's type/const-generic cycle domain and exact argument-vector test
  (A-18) are owner-ratified before `RecursionQualifiedTemplates` can exist.
- [ ] Recursive-effect disposition is owner-confirmed.
- [x] Candidate/accepted artifact, composite optimization artifact, target
  qualification, and lowering boundaries are concrete at the design level.
- [x] Empty and verified optimization-overlay paths have precise authority and
  failure behavior.
- [x] Independent evidence, resources, determinism, and failure atomicity are
  designed with the stages.
- [x] Hostile review findings and dissent are preserved.
- [x] Current Rust foundation has a preliminary
  keep/narrow/refactor/delete/defer disposition.
- [x] Decision 18's path/API/dependency/allocation audit receipt is complete.
- [ ] Exact release resource, target, backend, and host-filesystem profiles are
  evidence-selected.
- [ ] Owner approves the roadmap revision.

Until every unchecked entrance item is complete, the correct project status is
**design ready for owner discussion; production semantic implementation
paused**.
