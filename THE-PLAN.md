# THE PLAN

Status: CANONICAL ROADMAP, updated 2026-07-20.

## Objective

Build one production-quality Whitefoot compiler in safe Rust for the exact
normative content of `spec/kernel-spec-v0.8.md`, SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.

The implementation must generalize over every program defined by that exact
specification. Progress is measured by closed specification obligations and
end-to-end guarantees, never by how many functions from a particular corpus
compile.

The durable products are:

1. the immutable specification and its derivation record;
2. compiler-independent conformance and verification assets;
3. a deterministic Rust frontend, checker, artifact verifier, and generic
   backend;
4. independently verified optimizer facts; and
5. evidence from real projects that informs later language and library
   evolution.

Implementing v0.8 does not silently change its `DRAFT`, `PROVISIONAL`, or
`DEFERRED` markings. Any specification change remains separately owner-gated
and creates a new immutable numbered version. Self-hosting is not a prerequisite
or a current objective. It is a later product decision after the language and
libraries are demonstrably suitable for compiler implementation.

## Authority and authorization

`CONSTITUTION.md` is project law. `spec/kernel-spec-v0.8.md` is the exact first
implementation target. `PATTERNS.md` defines writer forms. This file alone
defines current execution order, phase gates, and authorization.

On 2026-07-20 the owner replaced the self-host-first route:

- wfc and democ are archived historical implementations;
- the production compiler starts fresh in Rust;
- there is no disposable Rust implementation;
- v0.8 is the exact first implementation target;
- compiler-independent tests and real-project evidence are first-class
  products; and
- implementation proceeds directly from the clean production architecture.

This supersedes D20's stage-0/self-host ordering and D21's wfc recovery cursor.
Their lessons about one general semantic pipeline, facts-off correctness,
artifact verification, and the rejection of source-shaped profiles remain
binding.

Phases 1 through 5 are authorized in order through the exact-v0.8 Rust compiler
baseline. Phase 6 research, additive verification, and open dogfood are
authorized, but no finding authorizes a specification change by itself. Every
numbered-specification change and every protected semantic-expectation change
still requires its exact owner approval and governance record. Phase 7 product
qualification protocols and any later self-hosting work require a new explicit
owner decision before execution.

## Permanent architecture

```text
canonical source
  -> lexer/parser/validated AST
  -> declarations/resolution
  -> syntax-directed semantic checker
  -> candidate CheckedUnit
  -> independent CheckedUnit verifier
  -> VerifiedCheckedUnit
  -> generic facts-off lowering
  -> LLVM text/runtime/linker

candidate FactOverlay
  -> independent fact verifier
  -> verified optional overlay
  -> the same generic lowerer
```

The checker constructs proofs. The verifier validates a closed proof language
without parsing raw source, resolving names, inferring types, searching for
proofs, optimizing, lowering, executing programs, or reproducing frontend
diagnostics. The artifact binds exact source bytes, canonical tree, node paths,
specification hash, facet-catalog hash, and proof-schema hash. The verifier uses
an independently implemented canonical renderer to confirm the source/tree
binding. The lowerer cannot consume raw ASTs or unverified artifacts.

The Rust workspace follows coherent responsibility boundaries rather than a
fixed crate or file count. Production Rust code forbids `unsafe`. Any unavoidable
foreign boundary is outside the semantic core, minimized, audited, and declared
in the trusted computing base. Toolchain and dependencies are pinned. Semantic
output contains no timestamps, absolute host paths, random identifiers,
hash-iteration dependence, or scheduling dependence.

## Verification contract

The conformance corpus tests language behavior, not compiler internals.
Specification-versioned source and expected verdicts are immutable authority.
An adapter reports separately audited implementation capability; it neither owns
that state nor changes a normative expectation. The adapter protocol
distinguishes semantic acceptance, rule-cited rejection, run, trap, exact
pending facet, internal error, timeout, artifact-verifier failure, backend
failure, and toolchain failure. No exception or crash is translated into
`Unsupported`.

A specification-hash-bound source index accounts mechanically for every rule,
syntax production, operation-table row, report row, and byte-exact normative
block. An authored semantic decomposition maps stable facets to exact source
atoms, one abstract owning stage, required pipeline lanes, and required evidence
classes. It contains no compiler symbol, implementation state, concrete witness,
expected verdict, semantic fallback, or replacement normative prose. A generated
static catalog binds those two inputs.

A separate implementation capability overlay outside the compiler build
workspace binds the catalog hash and names explicit generic-handler
responsibilities plus exact evidence-receipt identities. Closed class-specific
runners independently replay receipts; a reference alone grants nothing.
Capability is derived: a facet remains pending when any required handler lane is
unexercised or any required evidence class is absent, and there is no editable
completeness flag. Production code neither dispatches on facet IDs nor reads the
overlay to decide acceptance or lowering. Open specification or
protected-surface discrepancies live in a separate machine-checked sidecar,
cannot waive an obligation, prevent affected facets from closing, and block a
release. Coherent tranches may close multiple dependent facets; catalog counts
are integrity facts, not a work selector or progress measure.

Verification combines:

- canonical and malformed source cases;
- exact accept/reject/rule-diagnostic conformance;
- hand-authored valid and hostile artifact vectors;
- verifier mutation tests;
- focused independent models for ownership, loans, effects, drops, numeric
  edges, and call graphs;
- property and metamorphic generation over names, list lengths, nesting,
  control flow, and graph shape;
- source and artifact fuzzing;
- facts-on/facts-off differentials;
- runtime, ABI, trap, cleanup, determinism, code-shape, and performance tests;
  and
- complete external compatibility protocols for production projects.

Passing examples do not define completeness. A v0.8 production claim requires
every normative facet to close and no parseable v0.8 program to end in
`pending`, `xfail`, skip, broad fallback rejection, or `Unsupported`.

## Phase 1: archive the former implementations and reset authority

Archive exact snapshots of wfc and democ with their source commit, Git tree
hashes, limitations, and replay instructions. They are historical evidence and
cannot remain semantic, lowering, build, or release authorities. The Python
reference checker remains temporarily active only as a focused independent
model and source of additive regressions; it is not a compiler or language
authority.

Preserve the constitution, v0.8 specification, patterns, conformance corpus,
codegen corpus, governance records, decision log, design tree, and measured
evidence. Archive other material only when it is clearly superseded and no
active tool depends on it. No active build or source import may cross from
`archive/`.

Classify former implementation tests as protected semantic expectation,
reusable implementation-independent regression, implementation-specific
regression, historical profile/census test, or generated debris. Historical
tests move with their implementation. Useful general regressions are later
re-derived additively; old compiler architecture is never transplanted.

Rewrite the roadmap, instructions, README, archive index, root gate, owner
directive, governance record, decision log, and design tree. The transition
gate must state honestly that no active compiler exists; it must not claim
compiler correctness merely because the old implementations no longer run.

**Exit:** repository roles are unambiguous; exact replay is documented;
protected relocations are owner-approved and logged; archived paths are inert;
the repository foundation gate is green; and the transition is durable.

Status: completed 2026-07-20. Phase 2 is active.

## Phase 2: establish the Rust and verification foundations

Create the active Rust workspace and its first real build gate. Pin the Rust
toolchain and dependency graph. Add formatting, lint, unit, dependency-policy,
license, and deterministic-build checks. Prefer the standard library and
justify every semantic-core dependency.

Freeze and implement:

- the ordered multi-file `SourceBundle` contract;
- stable source, token, node, declaration, type, function, instantiation,
  check, and proof identities;
- the exact-v0.8 structural source index, authored semantic decomposition, and
  generated static facet-catalog checker;
- the separately validated compiler capability overlay and open-discrepancy
  sidecar;
- the canonical artifact envelope and codec;
- selectable conformance adapters with capability state separate from
  normative expectations;
- implementation-independent test support; and
- the independent verifier dependency boundary.

The compiler may report exact stage-qualified incompleteness during
development. Completed facets may not fall through to a broad rejection or
`Unsupported` result. No production claim is made in this phase.

**Exit:** the workspace architecture and dependency direction are executable;
specification drift fails closed; artifacts and diagnostics are deterministic;
the conformance runner targets named adapters without changing expected
verdicts; and root plus compiler development gates are green.

## Phase 3: build the complete v0.8 semantic pipeline

Land the first permanent end-to-end path, then close v0.8 in
dependency-coherent tranches:

1. canonical lexing, parsing, structural validation, declarations, types,
   constants, and resolution;
2. expressions, operation domains, statements, control flow, and effects;
3. ownership, place overlap, regions, loans, moves, joins, drops, and failure
   edges;
4. calls, explicit substitutions, generics, conformances, instantiations, SCC
   rules, and whole-unit closure; and
5. canonical diagnostics, proof records, artifact completeness, and reports.

Each tranche updates compiler-independent tests, checker producer, independent
verifier, artifact projection, lowering implementation, and the capability
overlay's concrete evidence. It does not rewrite the static catalog to report
progress; an index or decomposition correction is a separately reviewed catalog
change. Handlers operate over arbitrary legal names, list lengths, nesting,
source order, and graph shape. Whole-unit publication is failure-atomic. A
tranche cannot leave a parallel semantic path or temporary acceptance authority.

**Exit:** every normative v0.8 facet is owned and independently verified; all
declared bodies and concrete instantiations are checked; valid programs are
accepted and invalid programs receive the required deterministic rule
diagnostic; zero production-relevant pending, xfail, skip, or unsupported states
remain.

## Phase 4: complete generic facts-off lowering and runtime behavior

Lower only `VerifiedCheckedUnit` through one generic path. Cover layouts, ABI,
constants, monomorphization, control flow, aggregate values, every operation
mode, retained arithmetic and bounds checks, allocation, traps and reports,
derived drops/releases, and deterministic module publication.

LLVM, linker, runtime, allocator, OS, and every authorized foreign frame remain
declared TCB components. The compiler emits LLVM text; build orchestration invokes
external tools. No source-specific proof is an acceptance prerequisite and
every unproved required check remains.

Require differential execution, codegen-corpus checks, ABI fixtures, cleanup
and failure injection, malformed-artifact rejection, deterministic rebuilds,
resource ceilings, and target-specific tests for every claimed target.

**Exit:** the exact-v0.8 facts-off compiler passes full conformance and artifact
verification, produces runnable code for every supported normative construct,
retains every unproved check, and passes the correctness, resource, and named
target gates.

## Phase 5: harden and release the v0.8 compiler baseline

Run sustained source and artifact fuzzing, verifier mutation testing, bounded
semantic models, reproducibility checks, dependency and TCB review, adversarial
diagnostics, resource-limit tests, and full named-target CI.

The release manifest names the exact specification and facet hashes, proof
schema, Rust toolchain, dependency graph, LLVM/toolchain versions, runtime,
targets, DataLayout, ABI, resource ceilings, and platform assumptions. Require
byte-identical semantic artifacts and diagnostics for fixed inputs; do not
claim native-object identity across different host toolchains.

**Exit:** the complete release ledger is green with no hidden incompleteness.
Rust is the sole production Whitefoot compiler for the named exact-v0.8 profile.
Independent artifact verification does not certify LLVM, the linker, runtime,
allocator, or final machine code beyond their stated test and TCB boundary.

## Phase 6: facts and post-baseline capability discovery

Add optimizer facts one family at a time only after the facts-off compiler is
complete. Freeze each proposition, proof schema, producer, verifier, consumer,
scope, invalidators, and facts-off control. A fact may optimize an already
accepted program; it cannot affect acceptance, mutate `CheckedUnit`, remove an
explicit or non-elidable check, create another lowerer, or reach code generation
without independent verification. Each family requires hostile review,
verifier mutants, paired near misses, semantic identity, and measured value.

Open development projects may begin as soon as their v0.8 substrate exists.
Every finding is classified as a compiler defect, missing library or blessed
pattern, generic optimizer miss, genuine language gap, or target-specific
algorithm. A dogfood finding never changes the specification automatically. A
genuine language gap follows this order:

```text
nonnormative experiment and evidence
  -> exact owner approval
  -> new immutable numbered specification
  -> normative expectations and proof schema
  -> compiler implementation
```

**Exit:** each enabled fact family is independently justified, and a reviewed
capability ledger explains what real projects require without embedding project
identity in the language or compiler.

## Phase 7: qualify real production projects and reconsider self-hosting

This phase requires a fresh owner authorization. Its portfolio includes
complete utilities with real I/O, an embeddable library with an exact ABI and
lifecycle, and an external compiler-shaped project using parsing, indexed and
keyed state, graphs or worklists, structured diagnostics, and deterministic
serialization.

Development targets are permanently open evidence. Held-out targets use a
preregistered, non-discretionary pool and draw; a target that causes a compiler,
specification, optimizer, library, pattern, diagnostic, source-policy, harness,
or protected-oracle change fails that release and becomes open evidence.

A nontrivial production rewrite is an externally usable, swappable artifact
that passes its complete declared compatibility profile, incumbent tests,
differential fuzzing, unchanged downstream workflows, reproducibility, cleanup
and failure behavior, resource ceilings, and preregistered target-specific
performance gates. Isolated kernels and microbenchmarks do not qualify.

Only after ordinary compiler-useful collections, bytes/text, I/O, diagnostics,
packaging, and stable semantic architecture exist may the owner authorize a
Whitefoot compiler bakeoff. A self-hosting fixpoint is reproducibility evidence,
not proof of compiler correctness. If a Whitefoot implementation ever replaces
Rust, Rust freezes at one atomic authority switch; two production compilers do
not grow together.

## Prohibited routes

- No disposable Rust compiler and no import from the archived implementations.
- No active democ, wfc, and Rust semantic triad.
- No function, signature, name, ordinal, body, AST digest, project, or corpus
  dispatch.
- No function count, target count, facet count, deadline, or line count as an
  implementation selector.
- No raw AST or unverified artifact reaching lowering.
- No compiler-generated semantic expectation.
- No broad fallback rejection over a completed normative facet.
- No checker or verifier dependence on Rust-specific lifetime, trait, `Drop`,
  panic, overflow, or iteration semantics.
- No proof search inside the verifier and no speculative giant proof schema.
- No optimizer authority before facts-off semantics is complete.
- No special-purpose builtin or lowering added solely to rescue a project.
- No protected-test weakening, silent specification reinterpretation, or
  numbered-spec edit without exact approval.
- No production or generality claim from skipped cases, isolated kernels,
  self-hosting, or passing examples alone.
- No arbitrary file-size rule. Split code when one review can no longer hold its
  responsibility and invariants together; avoid monoliths, duplicate walkers,
  one-use forwarding modules, and junk-drawer utilities.

## Process and execution cursor

Run every currently applicable gate before and after a completed slice. A
development gate may be green with explicitly declared incomplete facets; a
release gate cannot. Every reproducible defect receives the smallest practical
regression before its fix closes. Every completed step receives one commit and
one decision-log entry. Fact channels and new safety judgments require hostile
review before shipment. Keep project artifacts in English and `AGENTS.md` and
`CLAUDE.md` byte-identical.

Phase 2 is active. The completed foundation now consists of:

- a pinned, dependency-policed three-crate safe-Rust workspace with exact build
  and cross-copy reproducibility gates;
- an ordered raw-source bundle, canonical source/specification binding codec,
  nominal specification and static-catalog identities, and an independently
  typed source-binding verifier;
- a fail-closed structural index covering 57 fenced grammar productions plus
  the two EFF-1 inline productions, and an authored static catalog covering all
  91 rules, 598 exact clauses, 587 facets, and 200 source atoms;
- a separately checked fifteen-record discrepancy sidecar and an empty,
  non-authorizing capability overlay outside the Cargo workspace; and
- a permanent lossless lexer that partitions every source byte into a
  shape-only token or retained trivia under explicit limits, plus a small
  compiler-independent byte model and fixture corpus.

The static catalog SHA-256 remains
`2fa586a8a1d9a49f344d64ad2b5f450a2ae2e8362bc187c70267097b9b427e1d`.
The lexer changes no catalog or capability state: `Complete` means only an
exact lexical partition, its token handles are source-bound runtime handles
rather than portable artifact identities, and its pre-tree issues are internal
locations without rule IDs, canonical node paths, or verdict authority. The
independent model is not a Rust differential or evidence receipt. Consequently
the capability overlay remains empty and this foundation closes zero facets.

Canonical tree, node, declaration, type, instantiation, check, proof, and
artifact-envelope identities remain deliberately unfrozen where schemas or
recorded v0.8 discrepancies prevent an honest contract. Traversal ordinals and
temporary syntax handles may not become artifact authority. Parser preflight
exposed and the discrepancy registry now pins an additional exact-v0.8
boundary: all 47 unique quoted lowercase grammar terminals match FORM-3 IDENT,
while the specification supplies no general terminal-versus-IDENT priority or
exclusion. Thirteen spellings have competing complete derivations across type
arguments, constant values, expressions, calls, and places. The record blocks
only its 16 directly affected facets; ordered-choice precedence or an invented
keyword set remains forbidden.

Only after that boundary is owner-resolved in a successor numbered
specification may the frontend publish a single-derivation grammar result. The
intended producer is permanent,
iterative, and resource-bounded over the lossless token stream, covers all 59
grammar productions while distinguishing the 57 fenced productions from
EFF-1's two inline productions, and retains the original lexical partition and
exact derivation structure. It may not choose a canonical/core-tree
representation, publish node IDs or paths, recover with synthetic tokens,
normalize effect rows, issue normative diagnostics, or claim language
acceptance. An independent bounded grammar oracle and hostile observation tests
must accompany the real producer before any parser capability claim.

The next unblocked Phase-2 slice is an executable lexical observation adapter
and independent Rust/model differential over authored and generated hostile
bytes. It must bind the exact specification and source bundle, restrict model
limits to the Rust `u32`/`u64` domain, keep capability metadata absent from all
compiler input channels, and report only the existing non-authorizing lexical
outcomes. It closes no facet until a separately reviewed evidence provider and
receipt replay boundary exist.

Phase 2 remains open until its artifact, adapter, identity, and verifier
boundaries are real and executable. A red gate, specification conflict,
protected-surface guard, unverified authority path, unexplained verdict, or need
for a numbered-spec change stops the affected work at that boundary.
