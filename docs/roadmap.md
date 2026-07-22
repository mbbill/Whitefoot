# THE PLAN

Status: CANONICAL ROADMAP, corrected 2026-07-22.

## Goal

The target is a serious research compiler: general enough to implement the
real language, clean enough to evolve, and capable of compiling nontrivial
programs so we can test semantics and performance ideas quickly. It is not an
untrusted-input service or a stable LLVM-scale product.

The compiler must be more than democ: it uses general language rules rather
than source-shaped exceptions, has independent correctness tests, produces
useful diagnostics, emits executable programs, and remains maintainable as the
specification changes. But it does not need release engineering for millions
of users, stable external protocols, adversarial resource guarantees,
transactional publication, or exhaustive operational failure handling.

The practical destination is:

```text
source programs
  -> complete frontend
  -> name resolution and semantic checking
  -> simple checked IR
  -> LLVM and runtime
  -> executable programs
  -> language and performance experiments
```

Specifications, tests, design notes, evidence, and tools serve this path. They
are not parallel products.

## Priority rule

When work competes, choose in this order:

1. the next meaningful end-to-end language or performance experiment;
2. semantic correctness and Whitefoot's required safety checks;
3. code that is understandable and easy to change;
4. enough independent evidence to trust the current result; and
5. robustness or polish only when a real experiment needs it.

Before doing supporting work, name the concrete compiler capability or
experiment it unlocks. If the supporting system becomes larger or more complex
than that capability, stop and choose a smaller route.

Function counts, issue counts, facet counts, protocol completeness, document
counts, and receipt counts do not measure progress. A useful compiled program,
a general semantic capability, a caught correctness bug, or a meaningful
measurement does.

## Current state

The active language authority is `spec/kernel-spec-v0.10.md`, SHA-256
`71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9`.
Those bytes are immutable. Exact v0.8 and v0.9 remain immutable historical
evidence.

The Rust compiler currently has source transport, a lossless lexer, terminal
classification, a strong-LL(2) parser, one finalized syntax tree, and exact
FORM-2 source validation. It ends at `CanonicalSyntaxUnit`. It has no resolver,
semantic checker, IR, LLVM backend, compiler executable, or runnable program.

The owner approved the exact successor proposal SHA-256
`7fc48cc30f94d25be5be1106e3265d92c1b0cdf2bfea5a7a17759a12f3cf092d` and
the exact generated v0.10 candidate SHA-256
`71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9`.
The exact approved candidate is installed and the existing frontend is
reproduced against its identity in one safe-Rust crate. The current goal is one
direct general name resolver, followed by the first semantically checked
program through a simple LLVM backend.

## Authority and specification changes

`docs/constitution.md` is project law. The active numbered specification defines the
language. `docs/patterns.md` defines writer forms. This file alone defines current
implementation order. Architecture and research documents are explanations and
inputs, not additional entrance gates.

A compiler/specification discrepancy stops the affected behavior for
investigation. Compiler behavior, tests, archived implementations, and design
prose cannot silently define the language.

The numbered specification is append-only: a released `spec/kernel-spec-v*.md`
is never edited, renamed, or deleted, and a pre-commit hook enforces it (install
once with `make install-hooks`). Amending the language is allowed, with care — a
change batch goes into a new numbered version. State the exact change, keep it
minimal, and record material changes in the decision log.

Before proposing a spec change, verify the new grammar with the grammar
verifier: a proposed specification must pass the main compiler's own lexer and
parser and satisfy the grammar constraints (parses, strong-LL(2), clean
terminal partition, no conflicts).

When the specification changes, everything derived from it is brought to the
newest version in the same work: conformance cases and verdicts, the reference
model, the lexer/parser and generated syntax data, tests, and docs. This
consistency is the responsibility of whoever changes the spec; it is not
machine-enforced, and derived material is never silently weakened to make a
check pass.

**Build task — a correct grammar verifier.** The archived
`archive/retired-gate/grammar-verifier` reimplemented grammar analysis in two
independent engines instead of reusing the compiler. Replace it with a small
tool that runs a proposed specification through the main compiler's own lexer
and parser and checks the grammar constraints above. Build it, and use it,
before the next numbered-specification change; it is not needed for routine
compiler work.

## What “good enough” requires

The research compiler must:

- implement each supported language capability by grammar and semantic rule,
  never by function name, source shape, project, or corpus membership;
- keep unsupported implementation capability distinct from language rejection;
- exercise all supported capability through one normal compiler pipeline;
- preserve every required runtime safety check unless a verified fact removes
  it;
- produce deterministic results where tests and measurements depend on them;
- use safe Rust without `unsafe` escape;
- keep modules cohesive and internal boundaries easy to revise;
- test semantic rules independently of the compiler where that materially
  increases confidence; and
- compile nontrivial dogfood programs that expose missing language and compiler
  capabilities.

It does not currently require:

- hard service-level limits for hostile input;
- a versioned whole-compiler `ResourceProfile`;
- evidence-selected numerical maxima before implementation;
- exact allocation-failure coverage for every Rust dependency;
- process sandboxes, transactional publication, crash recovery, or stable
  artifact interchange;
- a second semantic verifier or mandatory artifact replay;
- portable identities for private compiler records;
- stable internal APIs or compatibility with unknown external consumers;
- exhaustive failure taxonomies for paths that only compiler developers use;
  or
- release qualification for multiple hosts and targets.

Use normal Rust collections and allocation. Keep obvious size arithmetic
checked, avoid accidental unbounded recursion, and fix observed resource or
performance failures. Existing local limits may remain when they are simple
and tested, but do not expand them into a separate resource product. Resource
exhaustion is a compiler/development failure, not a source-language verdict.

## Implementation approach

Work in vertical language-capability slices once the shared resolver exists.
Each slice must implement a coherent family across semantic checking, checked
IR, lowering, runtime behavior, diagnostics, and tests. A slice may temporarily
leave other valid Whitefoot programs reported as not yet implemented; it may
not misclassify them as invalid.

This is not the old function-by-function route. A capability such as integer
operations, direct calls, structs, or loans must work for arbitrary legal
names, function counts, source order, nesting, and program shape. Dogfood
projects reveal which capability should come next, but production code never
special-cases a dogfood project.

For the next slice only, write down:

1. the exact active rules it implements;
2. its input and output;
3. what source is accepted, rejected, or explicitly not yet implemented;
4. the data required by its immediate downstream consumer; and
5. the smallest independent tests likely to expose a wrong implementation.

Then code it. Private structures may change freely while learning. Do not
design stable schemas, generalized frameworks, artifact protocols, or future
backend abstractions before a real consumer exists.

Resolve specification questions just in time. If the next capability is
blocked, present the exact behavior alternatives and evidence. Do not fill the
pause with unrelated infrastructure.

## Phase 1: repository and Rust foundation

Status: complete.

Obsolete wfc and democ implementations were archived. The continuing safe-Rust
workspace, specification governance, compiler-independent conformance data,
and focused reference models were established.

## Phase 2: independent grammar evidence

Status: complete.

The separate grammar verifier established the terminal partition, grammar
conflicts, and lookahead evidence needed for the frontend. It remains a spec
development tool and is not part of normal compilation.

## Phase 3: exact v0.9 installation

Status: complete.

Exact v0.9 was installed through the protected versioning procedure. Its bytes
and version-bound evidence remain immutable.

## Phase 4: canonical frontend

Status: complete, except ordinary bug fixes.

The lexer, classifier, parser, topology/source finalizer, and FORM-2 check
produce one `CanonicalSyntaxUnit`. A reproducible bug receives a focused
regression and direct fix; it does not justify a new support framework.

## Phase 5: activate v0.10

Status: complete.

Install the exact approved v0.10 candidate without editing it. Update its live
identity references and reproduce the current grammar and frontend evidence.
Do not add semantic implementation, resource measurement machinery, or new
frontend architecture during the version switch.

**Exit:** v0.10 is the active immutable target and the existing canonical
frontend passes against it.

## Phase 6: direct name resolver

Status: next.

Implement the exact v0.10 declaration inventory and lexical resolution rules
over `CanonicalSyntaxUnit`. Use straightforward owned records and deterministic
lookup structures.

The resolver must cover every grammar-defined declaration and use role, all
specified scopes and visibility, reservations, duplicates, shadowing,
declaration-before-use, top-level function visibility, operation families, and
deterministic diagnostics. Owner/member relations that require types remain
explicit deferred records for the type checker.

Do not implement the abandoned measurement routes, replay protocols, receipt
identities, or a versioned 33-field resource schema. Use the ordinary compiler
data structures the algorithm needs.

**Exit:** arbitrary v0.10 programs receive either a complete resolved unit, a
spec-defined resolution error, or an explicit later-stage/not-yet-implemented
result. Resolver unit, property, mutation, and conformance cases are green.

## Phase 7: first executable semantic slice

Choose the smallest coherent language family that can compile and run a real
program while exercising the actual semantic architecture. The expected first
slice includes primitive values, constants, function signatures and direct
calls, local bindings, basic control flow, required arithmetic modes and
checks, and the minimum ownership/effect behavior those forms require.

Implement the family end to end:

```text
resolved syntax
  -> typed checked representation
  -> simple target-independent IR
  -> LLVM for one host target
  -> runtime checks
  -> executable
```

The backend may be simple and inefficient. Correct facts-off behavior matters;
backend abstraction, stable IR serialization, caching, and optimization do not.

**Exit:** at least one nontrivial compiler-independent program and the complete
tests for the supported family compile and run through the normal pipeline.

## Phase 8: expand semantic capability

Add coherent language families in dependency and experimental-value order,
each end to end through execution. The likely families are:

1. aggregates, enums, construction, projection, and pattern matching;
2. generic types, constants, functions, instance closure, and contracts;
3. regions, borrows, loans, moves, joins, and cleanup;
4. slices, arenas, storage roots, and provenance-sensitive operations;
5. effects, recursive call graphs, remaining control flow, and whole-program
   checks; and
6. target/ABI behavior required by dogfood programs.

This order may change when real dependency or dogfood evidence says it should.
Changing order must name the experiment unlocked; it may not be justified by
which issue list is easiest to clear.

**Exit:** every construct in the active specification has one general semantic
and lowering path, and the full compiler-independent conformance suite is
implemented by the compiler adapter.

## Phase 9: dogfood and language iteration

Continuously use production-shaped but manageable projects to reveal missing
features and bad design. Cover at least binary data/compression, text and
command-line processing, collections or graph-shaped work, and one sustained
workload. zlib remains a useful example, not a privileged target.

When dogfood reveals a language problem, change the specification through the
numbered process and update the compiler. When it reveals a compiler problem,
add a minimized independent regression. When it reveals performance behavior,
measure before redesigning.

The compiler is successful when it can support these experiments reliably and
can be changed without repeatedly rebuilding unrelated infrastructure.

## Phase 10: optimizer experiments

Keep facts-off compilation correct. Add proof-based check removal and other
optimizations one proposition family at a time, with focused independent
verification and facts-on/facts-off comparisons. Optimize measured problems,
not hypothetical workloads.

An optimizer fact may improve an accepted program but may not change source
acceptance or remove a required check without proof.

## Phase 11: optional hardening

Only if later use justifies it, consider stable artifacts, caching, broader
targets, stronger resource controls, transactional publication, distribution,
self-hosting, or a product release. None blocks the research compiler or the
current experiments.

## Verification and durability

Run `make -C compiler check` before and after compiler changes and `make check`
for each completed repository slice. A green gate states only what it tests;
it is not a claim that the language or compiler is complete.

Every reproducible defect receives the smallest practical regression before
its fix. Each cohesive completed step gets one commit and one short append-only
entry in `governance/decision-log.md`.

Keep files cohesive and reviewable. Split by invariant-bearing responsibility,
not arbitrary line counts or corpus functions. New and modified repository
content uses English. `AGENTS.md` and `CLAUDE.md` remain byte-identical.

## Prohibited routes

- No function-by-function, signature-by-signature, corpus-by-corpus, or
  issue-count-clearing implementation strategy.
- No source-shaped dispatch, function allowlist, project special case, or test
  identity in compiler semantics or lowering.
- No disposable compiler, parallel semantic implementation, or premature
  self-hosting detour.
- No product-scale resource profile, replay system, receipt/identity scheme,
  publication protocol, sandbox, or failure taxonomy without a current
  experimental need.
- No placeholder artifact, schema, proof record, backend abstraction, or
  generalized framework before its real producer and consumer.
- No later-phase infrastructure used as filler while the next real compiler
  capability is blocked.
- No silent specification reinterpretation, protected-test weakening, or
  baseline regeneration merely to make a gate green.
- No optional optimizer fact changing acceptance or removing an unproved
  required check.
- No active source, build, test, or tool dependency on `archive/`.
