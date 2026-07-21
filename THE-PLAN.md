# THE PLAN

Status: CANONICAL ROADMAP, updated 2026-07-21. Phase 1's audited foundation
handoff and Phase 2's standalone grammar-change evidence package are complete.
Phase 3 is the next conditional gate; no normative edit is pre-authorized.

## Objective

Build one production-quality Whitefoot compiler in safe Rust. It must accept
and reject every program according to one exact, owner-approved numbered
specification, preserve every required safety check unless a machine-verified
proof authorizes its removal, and lower all accepted programs through one
general pipeline.

The active language authority remains
`spec/kernel-spec-v0.8.md`, SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.
That file is immutable. Its recorded contradictions block a production parser
and later semantic boundaries; compiler code may not invent resolutions.
Exact v0.8 remains the baseline for evidence until the owner approves a new
numbered specification and every live authority is switched to its exact
bytes and hash.

The durable products are:

1. immutable numbered specifications, derivation evidence, and governance;
2. compiler-independent conformance, grammar, semantic, backend, and
   real-project evidence;
3. one permanent safe-Rust production compiler with one trusted semantic
   kernel;
4. canonical artifacts with mandatory same-kernel replay before lowering;
5. one generic facts-off lowerer and runtime path;
6. independently verified optional optimizer facts; and
7. complete production-project compatibility evidence.

Progress is an end-to-end semantic capability with its authority boundary and
evidence. Function counts, corpus counts, facet counts, source shapes, and
passing examples are not work selectors or completeness measures.

## Authority and current authorization

`CONSTITUTION.md` is project law. The active numbered specification defines
the language. `PATTERNS.md` defines writer forms. This file alone defines
implementation order, phase gates, and execution authorization. The compiler
architecture dossier records the selected design in detail; it cannot bypass
this roadmap or a specification gate.

The owner replaced the self-host-first route on 2026-07-20:

- wfc and democ are inert historical artifacts under `archive/`;
- the production compiler is one permanent safe-Rust implementation;
- there is no disposable or parallel compiler ladder;
- compiler-independent tests and real-project evidence are durable products;
- self-hosting is a later product decision, not a prerequisite; and
- archived behavior never defines language or lowering semantics.

The 2026-07-21 architecture handoff selects:

- one typed postorder grammar-derivation tree, one linear topology finalizer,
  no copied second syntax tree, and a tree-driven exact-source audit;
- one trusted semantic kernel plus mandatory complete artifact-only replay
  through that same kernel before the originating invocation may construct
  lowering authority;
- no second production semantic certificate verifier;
- a separately runnable grammar-change verifier as the first implementation
  tranche;
- independent evidence at every assigned grammar, source/tree, semantic,
  target, backend, guard, publication, and optional-fact boundary; and
- the A-01 direction that all top-level function signatures are visible
  throughout the closed compilation unit, while locals, regions, labels, and
  named constants remain declaration-before-use. This rule has no compiler
  authority until exact successor-specification bytes encode it.

This roadmap supersedes D22 only where D22 selected an independent production
semantic verifier and authorized implementation against an internally
contradictory exact-v0.8 completion path. It preserves D22's safe-Rust,
single-implementation, archive, conformance, and specification-governance
decisions. It preserves D21's bans on source-shaped dispatch and census
clearing, its one-general-pipeline requirement, and its separation of checked
acceptance from optimizer proof.

Current execution authority is deliberately narrow:

1. Phase 1's exact audit-selected foundation migration is complete.
2. Phase 2's standalone grammar-change verifier has exited with reproducible
   exact-v0.8 and non-authoritative successor evidence.
3. Phase 3 is next, but remains conditional on exact owner review and advance
   approval. Phases 3 onward define mandatory order and entrance gates. They do not
   authorize a specification edit, protected-surface change, active-target
   switch, production parser, semantic schema, artifact schema, backend
   profile, migration, or release merely by appearing here.
4. Additive implementation-independent tests and read-only evidence may
   continue when they do not cross a protected or earlier authority boundary.

## Specification and protected-surface protocol

A compiler/specification discrepancy stops implementation at that boundary.
Compiler behavior, an archived implementation, a corpus majority, or a design
document cannot define the language.

Every numbered-specification change follows this order:

```text
exact non-authoritative proposal bytes
  -> independent evidence and protected-surface impact census
  -> exact owner-visible delta and full-byte hash
  -> explicit advance owner approval
  -> new immutable numbered specification with identical reviewed bytes
  -> every live reference and lock updated
  -> exact numbered-byte hash check and required verifier rerun
  -> make approve-spec REASON="..."
     (regenerates the guard baseline and appends the governance entry)
  -> ordinary gates
```

Any change to protected conformance source or verdicts, frozen oracle digests,
approved baselines, or active reference-semantics tests requires its own exact
advance approval. Additive tests are free. Modifying, deleting, weakening, or
regenerating protected material is not. A red specification guard is a stop;
it is never permission to regenerate a baseline.

A grammar report is evidence, not approval. Proposal bytes remain outside the
numbered `spec/` surface until the protocol completes. Exact-v0.8 catalogs,
discrepancy records, fixtures, and locks remain versioned historical authority;
a successor receives new version-bound assets rather than rewritten v0.8
assets.

## Permanent production architecture

The production path is:

```text
ordered SourceBundle transport + SpecificationIdentity + CompilationTargetProfile
  -> lossless shape lexer
  -> specification-versioned terminal classifier
  -> typed iterative predictive parser
  -> typed postorder DerivationTree
  -> linear topology finalizer
  -> tree-driven exact-source audit
  -> CanonicalSyntaxUnit
  -> declaration inventory and resolution
  -> SymbolicallyTypedUnit for every source function
  -> finite template call graph and pre-instantiation FN-6 gate
  -> all-source template semantic coverage
  -> finite concrete-instance closure and mandatory concrete rechecking
  -> explicit concrete CFG and call graph
  -> provenance, region, ownership, cleanup, and effect checking
  -> private SemanticallyCheckedDraft
  -> target qualification
  -> canonical base artifact projection
  -> mandatory artifact-only replay through the same semantic kernel
  -> AcceptedCompilation built only from replay-decoded state
  -> exact backend invocation + empty or verified optimization overlay
  -> canonical final-envelope projection and validation
  -> FinalizedCompilation
  -> derived target-bound CodegenPlan
  -> conservative LLVM/runtime/link/publication pipeline
```

The current ordered `SourceBundle` is a transport envelope, not authority for
normative multi-file composition. File order, zero-source behavior, program-root
extent, and declaration order acquire language meaning only after A-10 is
encoded in an approved numbered specification.

The semantic kernel is trusted production code. Same-kernel replay checks that
canonical artifact bytes completely and consistently encode the accepted
invocation; it detects projection, omission, reference, codec, and corruption
defects. Replay is not independent semantic evidence and does not reduce the
trusted computing base.

Only the originating invocation's mandatory replay may construct
`AcceptedCompilation`. Later, cached, third-party, hand-authored, or
independently loaded artifact bytes may be audited, but they can never
reconstruct a lowering capability. Lowering accepts only
`FinalizedCompilation`; raw syntax, a producer draft, decoded bytes, an audit
result, or an authority-sounding Rust type name is insufficient.

Optional optimizer facts are a separate path:

```text
candidate proposition overlay bound to accepted artifact, target, and backend
  -> proposition-family independent verifier
  -> exact-byte-backed verified overlay
  -> the same final envelope and generic lowerer
```

The canonical empty overlay is always valid and preserves every required
semantic fact and runtime check. An optional fact may improve an already
accepted program; it cannot change acceptance, semantic identity, explicit
checks, or select a second lowerer.

## Verification and trust contract

Conformance source and expected verdicts are compiler-independent authority.
Compiler capability, resource failure, internal failure, timeout, replay
failure, backend failure, and toolchain failure remain adapter observations;
they never rewrite normative expectations or become `Unsupported`.

The static source index, semantic decomposition, generated facet catalog,
capability overlay, discrepancy sidecar, corpus identities, and project names
are audit inputs only. Production code does not read them to choose a handler,
acceptance result, semantic path, or lowering path. A digest match proves
identity, not truth, completeness, or capability.

Mandatory independent evidence includes, where assigned:

- a static grammar auditor and a separately extracting bounded generalized
  parser oracle;
- source/tree reconstruction models and hostile tree mutations;
- exact conformance and deterministic diagnostic cases;
- focused models for types, ownership, loans, effects, cleanup, numerics,
  recursion, and instance closure;
- property, metamorphic, fuzz, and mutation testing over arbitrary names,
  lengths, nesting, source order, and graph shape;
- target, ABI, runtime, guard, linker, and publication differentials;
- reproducibility, resource, failure-injection, and policy checks; and
- complete compatibility protocols for real projects.

Sharing input bytes does not make two engines dependent. Sharing an extracted
grammar, predictive table, semantic judgment, proof algorithm, or expected
result does. Every authority-increasing optimizer proposition receives hostile
adversarial review before shipment.

All production stages have closed source-rejection, resource-failure, and
compiler-invariant outcome families; explicit caller-selected resource
profiles; deterministic ordering; checked arithmetic; fallible growth; and
failure-atomic publication. A resource limit never proves a source invalid or
an ambiguity absent.

## Active blockers

The exact-v0.8 discrepancy registry is the machine-readable source of truth.
Its fifteen open records remain: affine-dereference backing-storage cleanup,
pre-tree DIAG-1 location, retained-check `proof_ref`, EFF-1 row canonicality,
body-local region effects, contract-member semantics, optimizer-law admission,
`main` return spelling, `requires` rejection attribution, protected FORM-2
spacing, the FORM-4 cross-reference, FORM-5/FORM-7 float spelling, the
fixed-terminal/`IDENT` partition, the GRAM-1/GRAM-7 node-kind conflict, and
dotless-operation reservation.

A-01 has an owner-selected direction but remains blocked on exact successor
encoding and the ordinary protected/specification protocol.

All other architecture questions remain open:

- A-02 through A-09: affine overwrite disposition, evaluation order, ordinary
  scope cleanup, recursive nominal layout, frame-limit target binding, TYPEID
  collisions, continuing-join affine liveness, and unreachable suffix checking;
- A-10: normative multi-file compilation-unit formation, zero-item files,
  zero-source behavior, declaration order, and program-root extent;
- A-11 through A-13: `slice_of` authority, returned-view provenance, and
  recursive region-free storage well-formedness;
- A-14 and A-15: lifetime-report drop sites and logical-stack attribution;
- A-16: holder and loan identity across copy, move, call, return, `give`, and
  match projection;
- A-17: region-bearing generic arguments and portable semantic-instance
  identity; and
- A-18: FN-6's type/const-generic cycle domain and argument-vector test.

The recursive-effect rule, exact target/frame behavior, concrete resource,
backend, and host profiles, artifact records, and portable identity schemas
also remain gated. A roadmap description is not a resolution. Each affected
implementation waits for exact normative or owner-approved profile authority.

## Phase 1: complete the audited foundation handoff

Status: complete. The read-only audit receipt and D25-authorized migration are
durable in the repository.

The receipt is
`optimizer-language-research/implementation/compiler-foundation-audit-2026-07-21.md`.
It binds HEAD `c1975d5d30f29a95647ff21d5e1895cad40adf0d`, compiler tree
`bead7377dc6b7c880d630d873143da79fadf5852`, and status digest
`2aa3438ea56678b36f81a863b8e6e69aa0edcf81d274660655c855894b36e2d4`.

The retained foundation is:

- inert archived wfc and democ snapshots with no active dependency;
- the pinned safe-Rust workspace, dependency/source policies, formatting,
  linting, testing, rustdoc, and cross-copy reproducibility gates;
- nominal specification and catalog identities;
- ordered source transport, source spans, source-binding bytes, and exact
  source-equality audit;
- the v0.8 source index, static semantic catalog, discrepancy registry, and
  empty non-authorizing capability overlay;
- the permanent lossless shape lexer;
- the binary-only lexical observer and independent byte model; and
- compiler-independent conformance, reference models, and evidence corpora.

This foundation proves only its named contracts. It contains no production
parser, canonical syntax authority, semantic acceptance path, checked
artifact, lowerer, executable compiler, or release capability.

The completed migration applied the fixed audit disposition:

- kept the source contract, lossless lexer, lexical observer, and
  source-equality audit;
- renamed `whitefoot-frontend` to `whitefoot-lexer` and
  `whitefoot-verifier` to `whitefoot-source-audit`, with their responsibilities
  narrowed to match those names;
- made owned-source and source-binding construction audibly fallible under
  exact limits before whole-compiler boundedness is claimed;
- preserved the current source-binding wire contract;
- retained the catalog, discrepancies, and capability metadata outside
  production dispatch;
- deleted nothing; and
- created no parser, semantic, identity, artifact, or crate placeholder.

The migration may perform only the receipt's exact renames, API/allocation
repairs, dependency-policy updates, documentation alignment, and tests.
Anything beyond that inventory requires a new owner decision.

**Exit:** names and APIs claim exactly what they check; owned-source and
binding growth is fallible and bounded; dependency direction remains acyclic;
the source-binding wire contract is unchanged; all focused, compiler, and root
gates are green; and the handoff is durable in one cohesive commit and
append-only decision entry.

## Phase 2: grammar-change verifier and evidence package

Status: complete. The separately runnable tool, both independent engines,
canonical evidence package, exact proposal bytes, and protected-surface census
have passed their written exit gate.

The completion receipt binds exact v0.8 SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`,
candidate SHA-256
`cfd76a2bf9293519623c2448280f4d6f76f32be26cc1b2dadc487415e063f166`,
and a byte-identical 134,019-byte common extraction ledger with SHA-256
`2014897a6d2a4599957bad140f0de73c0d42c559ec629a3fdc20fe0b4d238b27`.
Both registered dereference cases change from two derivations to one with one
removed, one retained, and zero introduced trace; the exact `deref(x)` static
transition changes from one matching predictive conflict to zero. The complete
48-stream generated domain agrees byte for byte. The final hostile runner
review returned GO after source mutants, malformed-ledger probes, and 2,000
deterministic field mutations. These are evidence results, not language
authority.

Build one separately runnable verification entry point outside the production
compiler dependency graph. Normal compilation never links or invokes it. It
accepts the exact full bytes of the current numbered specification and exact
full bytes of a non-authoritative proposal, binds both hashes, and has two
deliberately independent engines:

1. A static auditor independently extracts every normative grammar production,
   lexical class, and fixed terminal; rejects unclassified grammar-shaped
   content; computes nullable, `FIRST_2`, `FOLLOW_2`, terminal-category
   intersections, and the exact strong-LL(2) predictive relation; and emits
   concrete witnesses for every collision.
2. A bounded generalized-parser oracle independently extracts the grammar and
   token-membership rules into its own representation and returns `zero`,
   `one`, or `many` complete source-level derivations for registered and
   generated streams.

The engines share only exact input bytes and the declared resource profile.
They do not share extraction code, grammar tables, terminal classification,
predictive relations, parser tables, or derivation expectations. Extraction or
source-coverage disagreement fails. Resource exhaustion is an explicit
inconclusive result.

The exact transition evidence must:

- require the current input hash to equal the pinned v0.8 hash;
- reproduce the registry's exact `deref(p)` ambiguity;
- register exact `deref(x)` as the transition witness and report two complete
  v0.8 derivations;
- show a candidate terminal/`IDENT` repair removing only the
  call-through-`IDENT` path for that witness while retaining the fixed
  `deref` place derivation;
- audit the complete mechanically extracted set of 47 fixed lowercase
  terminals, not a parser-local subset;
- report every removed, retained, and introduced intersection or predictive
  conflict;
- bind all authored cases, generated domains, engine revisions, limits, and
  canonical report bytes; and
- include mutants that independently break source binding, extraction,
  terminal membership, lookahead relations, and derivation counting.

Prepare, but do not install, exact non-authoritative successor bytes for the
terminal partition and owner-selected A-01 rule. Prepare the complete
protected-surface impact census. A-02 through A-18 remain later blockers,
outside this tranche; do not turn design questions into machine facts by
assertion.

**Exit:** both grammar engines and their hostile tests are green; exact v0.8
baseline and proposal reports are reproducible and full-byte bound; the
`deref(x)` transition is explicit; the protected-surface census and exact
proposal bytes are ready for owner review. No numbered specification,
protected expectation, active target, production parser, compiler schema, or
capability claim has changed.

**Stop:** any hash mismatch, incomplete extraction, engine disagreement,
unexpected conflict, inconclusive required case, protected-surface uncertainty,
or red repository gate.

## Phase 3: approve and install a successor specification

Status: conditional; no normative edit is pre-authorized.

Present the exact proposal bytes and hash, grammar reports, every relevant
semantic discrepancy record, protected-surface impact census, and exact
owner-visible delta. Only explicit owner approval may create a new numbered
specification and any separately approved protected changes.

After approval:

1. create the new numbered file byte-for-byte from the reviewed proposal;
2. apply only the separately approved protected changes;
3. update every live specification reference and lock, including the roadmap,
   design memory, instructions, compiler locks, and public documentation;
4. create new version-bound indexes, catalogs, discrepancies, fixtures, and
   capability metadata without rewriting v0.8 history;
5. rerun both grammar engines over the exact numbered bytes and require the
   numbered hash to equal the reviewed proposal hash; and
6. run `make approve-spec REASON="..."`; it regenerates the guard baseline
   and appends the approval entry with the resulting baseline hash; and
7. run all ordinary gates.

Parser work additionally requires the active successor to resolve the complete
terminal partition, GRAM-1/GRAM-7 node-kind conflict, pre-tree diagnostic
location, FORM-2 interaction, and A-10 program-root/compilation-unit contract,
with a conflict-free complete static grammar audit and every required bounded
oracle result.

Semantic work additionally requires owner-selected A-01 to be encoded exactly
and each later A-question or discrepancy to be resolved before its affected
stage.

**Exit:** one exact successor is active everywhere; all guards and ordinary
gates are green; v0.8 remains immutable; and the next phase's exact entrance
blockers are closed.

**Stop:** any byte difference from the reviewed proposal, stale live reference,
unapproved protected delta, red guard, unresolved required blocker, or
non-passing grammar gate.

## Phase 4: build the canonical frontend

Status: conditional on Phases 1 through 3 and the complete parser entrance
gate.

Keep the existing shape lexer unchanged in authority. Add a
specification-versioned terminal classifier whose accepted spelling sets form
the exact approved disjoint partition. Build an iterative typed LL(2) parser
with no backtracking, ordered-choice priority, recovery, synthetic tokens, or
semantic disambiguation.

Construct one typed postorder `DerivationTree`. A single linear finalizer
checks root extent, source ownership, parent/child topology, production shape,
complete token coverage, and resource invariants. A tree-driven audit
reconstructs expected source bytes from that same tree and compares them with
the input. It does not normalize source and there is no copied
`ValidatedAst`.

Only after the approved node-kind, location, A-10, and canonical-source rules
exist may the stage publish `CanonicalSyntaxUnit` and portable syntax
identities. Runtime traversal handles are not artifact identities.

Independent evidence includes the complete static grammar audit, generalized
oracle, tree/source model, hostile topology and token mutations, generated
shared-prefix cases, fuzzing, deterministic diagnostics, and exact resource
edges.

**Exit:** every accepted source has exactly one complete derivation and one
finalized source-bound tree; every malformed source receives the approved
deterministic syntax outcome; no partial tree escapes; and no semantic
acceptance claim exists.

## Phase 5: build the semantic kernel

Status: conditional on exact normative authority for every affected rule.

Implement one syntax-directed semantic kernel in this order:

1. complete declaration inventory and visibility-governed resolution using the
   exact successor encoding of A-01;
2. modes, types, constants, operations, substitutions, typed labels, and typed
   calls for every function declaration;
3. the finite template call graph and pre-instantiation FN-6 SCC gate;
4. target-independent template CFG/semantic coverage for every source
   function, including unused and zero-type/const-parameter functions;
5. explicitly seeded finite concrete-instance closure and mandatory concrete
   rechecking;
6. the concrete call graph and explicit CFG with every normal, trap, cleanup,
   drop, release, and check edge;
7. approved provenance qualification; and
8. region, loan, ownership, join, cleanup, effect, and whole-unit closure.

A required rejection may not be deferred behind a potentially expanding
worklist. Every handler accepts arbitrary legal names, list lengths, nesting,
source order, and graph shape. Function families and corpus slices may group
delivery and tests but never survive as admission or dispatch architecture.

The kernel publishes only a private `SemanticallyCheckedDraft`. It is not an
accepted artifact and cannot reach lowering. All semantic records retain exact
source origin and complete coverage. Deterministic error selection follows the
approved canonical order.

Before each substage, close its applicable discrepancy and A-question,
including A-02 through A-09, A-11 through A-18, recursive effects, and
diagnostic authority. No implementation convenience chooses the rule.

**Exit:** every normative construct for the active target has one general
semantic path; every declared body and required semantic instance is checked;
all required checks and cleanup are explicit; and independent models, mutants,
fuzzing, and conformance evidence are green.

## Phase 6: seal acceptance through artifact replay

Status: conditional on complete semantic records and approved target, resource,
diagnostic, artifact, backend, and host profiles.

Target-qualify the private semantic draft. Project one canonical base artifact
containing every fact needed for source acceptance, diagnostics, target
acceptance, and lowering. Decode and replay those bytes through the same
semantic kernel with no producer draft or hidden side channel. Compare exact
source, specification, target, schema, and invocation bindings.

Discard the producer draft. Only successful originating-invocation replay may
privately construct `AcceptedCompilation`, and its lowering payload is the
replay-decoded state. A later replay returns an audit result only.

Validate exact backend-invocation bytes, compose the canonical empty overlay or
an exact-byte-backed independently verified optional overlay, project the
complete final envelope, decode and revalidate it, and construct
`FinalizedCompilation` only from those exact bytes.

Require hostile omission, duplication, reordering, reference, coverage,
canonicality, resource, and codec mutations. Prove that no public constructor,
decode result, cache, third-party artifact, or audit result can reach the
lowerer.

**Exit:** acceptance is artifact-decidable; producer state and replay state
agree exactly; only replay-decoded state carries lowering authority; the empty
overlay path is complete; and final-envelope bytes are deterministic.

## Phase 7: complete conservative facts-off lowering and publication

Status: conditional on `FinalizedCompilation` and exact named profiles.

Derive one ephemeral target-bound `CodegenPlan` from
`FinalizedCompilation`. The plan may schedule and choose target instructions;
it may not resolve names, infer types, discover instances, choose ownership or
cleanup, add semantic facts, change acceptance, or omit a required check.

Lower every accepted construct through one generic path. Retain every unproved
overflow, bounds, lifetime, allocation, and trap check. Use the exact guard ABI
and prove guard structure and dataflow in emitted LLVM; audit required guard
sites in objects. Bind pinned LLVM, runtime, linker, target, ABI, layout, and
tool outputs through exact-byte-backed capabilities. The downstream
publication component alone owns final validation and atomic directory commit.

Require facts-off conformance, differential execution, codegen-corpus checks,
ABI fixtures, target tests, malformed-artifact rejection, trap and cleanup
injection, deterministic rebuilds, resource ceilings, runner/linker failure
tests, guard mutants, and publication atomicity.

**Exit:** the facts-off compiler produces correct runnable outputs for every
normative construct in the named target profile, retains every unproved check,
and passes complete semantic, runtime, resource, reproducibility, and
publication evidence.

## Phase 8: dogfood and harden the production baseline

Status: conditional on the complete facts-off path.

Exercise complete, nontrivial projects rather than isolated kernels:

- utilities with real byte/text and I/O behavior;
- an embeddable library with exact ABI and lifecycle requirements;
- allocation, cleanup, error, and resource-pressure workloads; and
- a compiler-shaped project using parsing, indexed/keyed state, graphs or
  worklists, structured diagnostics, and deterministic serialization.

Each claimed rewrite must pass its complete incumbent tests, differential
fuzzing, unchanged downstream workflows, reproducibility, cleanup and failure
behavior, resource ceilings, packaging, and preregistered target-specific
performance gates. Development projects are open evidence. Held-out release
targets use a preregistered non-discretionary pool and draw.

Classify every finding as a compiler defect, missing library or blessed
pattern, generic optimizer miss, genuine language gap, or target-specific
algorithm. A genuine language gap returns to the full proposal and
specification protocol. It never creates a project-specific builtin, compiler
branch, or silent language change.

Run sustained source/artifact fuzzing, semantic and codec mutation, bounded
models, adversarial diagnostics, dependency and trusted-base review, resource
testing, and full named-target CI.

**Exit:** the real-project compatibility ledger is complete, every discovered
authority defect is closed through the proper earlier phase, and no project
identity has entered the compiler.

## Phase 9: qualify and release

Status: conditional; a release claim requires the separate release gate and
exact owner-approved release profile.

The release profile and manifest name the exact specification, source index,
facet catalog, discrepancy state, artifact schemas, Rust toolchain, dependency
graph, LLVM and linker versions, runtime, targets, DataLayout, ABI, resource
ceilings, host/filesystem assumptions, test identities, and real-project
qualification results.

A release requires:

- zero unresolved specification or protected-surface blocker affecting the
  target;
- zero production-relevant pending, skip, xfail, unsupported, broad fallback,
  or unexplained outcome;
- complete exact conformance and deterministic diagnostics;
- complete same-kernel artifact replay and empty-overlay lowering;
- every required independent evidence lane and hostile review;
- all named target, runtime, guard, linker, publication, resource, and
  reproducibility gates; and
- a green release ledger distinct from the development gate.

Native object identity is required only within the exact declared host/tool
profile; no cross-toolchain identity claim is implied.

**Exit:** safe Rust is the sole production Whitefoot compiler for the exact
named specification and profile, with no hidden incompleteness or alternative
semantic authority.

## Work after the facts-off release

Optional optimizer propositions may be added one closed family at a time only
after the complete facts-off path exists. Freeze each proposition, producer,
schema, independent verifier, consumer, scope, invalidators, facts-off control,
mutants, differential, and measured benefit. Facts never change source
acceptance or create an alternative lowerer.

Any product expansion beyond the released profile and self-hosting require
fresh owner authorization.
Self-hosting is dogfood and reproducibility evidence, not proof of compiler
correctness. If a Whitefoot implementation ever replaces Rust, Rust freezes at
one atomic authority switch; two production compilers do not grow together.

## Prohibited routes

- No exact-v0.8 parser workaround using keyword priority, ordered choice,
  invented reserved words, semantic disambiguation, or archived behavior.
- No generalized parser in production; it is a bounded evidence oracle only.
- No grammar-change tool linked into or invoked by normal compilation.
- No grammar report, corpus, compiler behavior, or historical intent acting as
  specification authority.
- No parser, node, path, identity, semantic wrapper, proof record, artifact
  envelope, or crate placeholder before its real normative contract and
  consumer exist.
- No copied second syntax tree, vague `ValidatedAst`, poison declaration,
  synthetic recovery token, placeholder type, or partial checked unit.
- No second production semantic checker or semantic certificate verifier.
- No function name, signature, ordinal, body shape, AST digest, project,
  corpus, facet, capability, or target profile selecting semantic admission or
  lowering.
- No function count, corpus count, target count, facet count, deadline, or line
  count selecting implementation work or proving progress.
- No raw syntax, producer draft, decoded artifact, audit result, cached
  artifact, or third-party bytes reaching lowering.
- No backend name resolution, type inference, instance discovery, ownership
  join, effect closure, cleanup decision, source acceptance, or proof search.
- No source-specific proof as a prerequisite for facts-off compilation; every
  unproved required check remains.
- No optional fact affecting acceptance, semantic identity, explicit checks,
  or selecting a second lowerer.
- No LLVM poison, undefined behavior, check removal, or body sharing from an
  unverified assertion or nominal identity.
- No capability overlay, catalog, discrepancy metadata, project identity, or
  evidence receipt consumed as production semantic input.
- No active import, build, semantic, lowering, or release dependency from
  `archive/`.
- No compiler-generated normative expectation, protected-test weakening,
  silent specification reinterpretation, or unapproved baseline regeneration.
- No infallible or unbounded growth, output, process, or publication path
  inside a claimed resource-bounded authority boundary.
- No later phase used as filler while an earlier authority boundary is blocked.
- No arbitrary file-size rule, forwarding-only module, broad utility drawer,
  duplicate walker, or file split by corpus function. Split by one cohesive
  invariant-bearing responsibility.

## Process and execution cursor

Run `make -C compiler check` before and after compiler work and `make check`
before and after every completed repository slice. A green development gate is
an exact current-capability statement, never a release claim.

Every reproducible defect receives the smallest practical regression before
its fix. Every completed step receives one cohesive commit and one append-only
`decision-gates.md` entry. A red gate, specification conflict, protected
guard, unexplained verdict, unverified authority path, or resource-bound breach
stops the affected work. Never regenerate authority to make a gate green.

Production Rust forbids `unsafe`. Dependencies and tools are pinned.
Semantic outputs contain no timestamps, absolute host paths, random
identifiers, hash-iteration dependence, or scheduling dependence. Files remain
cohesive and reviewable. Every new or modified repository artifact, identifier,
comment, diagnostic, fixture, and test name uses English. `AGENTS.md` and
`CLAUDE.md` remain byte-identical.

The completed current foundation contains three permanent library crates and
one binary-only lexical observer, seven reproducible Cargo artifacts, the
static catalog SHA-256
`2fa586a8a1d9a49f344d64ad2b5f450a2ae2e8362bc187c70267097b9b427e1d`,
the fifteen-record discrepancy sidecar, an empty capability overlay, the
lossless lexer, and the 942-case lexical differential with request-input
manifest SHA-256
`5f84cc0982cd74c46fc9350da4ee6611ec5a513f0c3ede1d2f76dceeeab39ff9`.
These close no semantic facet and grant no parser, artifact, or release
authority.

Phases 1 and 2 are complete. The next roadmap action is the Phase-3 owner review
of the exact proposal bytes, hash, evidence, and protected-surface census. No
successor installation, active-target switch, or production parser work is
authorized until that exact review receives advance approval and the guarded
installation procedure succeeds.

Until Phase 3 exits and every later gate is separately satisfied:

- do not edit a numbered specification or protected expectation;
- do not switch the active target;
- do not build a production classifier, parser, syntax tree schema, portable
  identity, resolver, semantic record, artifact envelope, replay API, target
  profile, lowerer, or publication path; and
- do not link or invoke the grammar-change verifier from normal compilation or
  treat its reports as specification or compiler authority.
