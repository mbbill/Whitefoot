# THE PLAN

Status: CANONICAL ROADMAP, reoriented to Phase 10 performance evidence
2026-07-24; Phase 10 completion set fixed and Phase 11 (declared parallelism)
added by the owner 2026-07-27.

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

The active language authority is `spec/kernel-spec-v0.17.md`, SHA-256
`19642ffb0ad9c7146a84762ada192ed2a25dc446a93c4d060aa29d9a99f69c93`.
Those bytes are immutable and byte-identical to the owner-approved candidate.
Exact v0.8 through v0.16 remain immutable historical evidence. v0.12 added the
SET-1 copy-place assignment judgment, target-before-RHS ordering, post-RHS
writability revalidation, and ultimate-storage-origin read/write effects.
v0.13 makes a direct bare affine own-rooted `Result` place a consuming
`propagate` operand, matching the already-approved writer form while retaining
explicit `move` as a valid spelling and leaving every other ownership rule
unchanged. v0.14 closes the already-listed integer-negation rows: wrapping
minimum remains minimum, trapping minimum emits OP-2's exact mandatory record,
and checked minimum returns `Err(Overflow())`. v0.15 removes the undefined
array “frame limit” and defines the selected-target layout boundary: complete
static objects must fit the selected target, while runtime allocation and
element-address arithmetic must be proved exact or guarded before use. These
target failures are not source-language rejections or language traps.
v0.16 closes the static source-contract family: contracts are nongeneric,
conformances bind one exact concrete subject to one source contract with a
complete declared-order binding vector, callable signatures require exact
mode, type, region, and normalized effect-capability equality, and every
applicable law receives FN-4's mandatory closed discharge. This metadata has
no runtime or lowering representation and its base law evidence grants no
optimizer authority. Source-contract generic bounds receive an explicit FN-3
rejection, and contract member calls remain absent from v0.16.
v0.17 closes direct returned-slice provenance without changing the slice
descriptor or ABI. Every direct slice carries a finite static set of possible
storage origins; an `own slice` result gets its finite supplier ceiling from
the written signature, calls substitute actual origin sets without inspecting
the body, and ownership overlap and effects quantify over the whole set.
Region-bearing function and nominal generic arguments now receive FN-2
rejections, and region-bearing box or arena content receives STOR-5
rejections. Borrow-mode direct-slice results receive FN-1 rejections because
their descriptor-place provenance belongs to the separate returned-borrow
design. Direct nested-slice type formation remains valid but compiler-
unsupported.
The compiler keeps that exact version, path, byte content, and digest at one
active-specification identity boundary. Its stage, syntax, rule, and pipeline
symbols are stable implementation names rather than `V0_xx` APIs, so a
grammar-preserving specification bump changes identity data and real semantics
without mechanically renaming the compiler.

The Rust compiler now has one ordinary path from ordered source transport
through the lossless frontend and direct resolver into semantic checking, a
private checked program, target-independent typed control-flow IR,
conservative textual LLVM, and a runnable host executable. The scalar family
supports exact integer and unit values, `Bool`, integer and unit constants,
nongeneric own-mode functions, locals, direct named calls, explicit returns,
pure/traps effects, wrapping and trapping add/subtract/multiply, integer
division/remainder, negation, bitwise operations, shifts, rotates, bit counts,
byte swap, high multiply, saturating arithmetic, min/max, integer comparisons,
Boolean operations, exact integer conversion, and nominal tag equality.

The first Phase 8 slice adds nongeneric own-mode acyclic structs and enums,
construction, nested struct projection, statement and value matches, exact
field/order and exhaustiveness checks, `give` delivery, and whole-binding
affine moves. The same typed CFG and LLVM path handles cross-function
aggregates, tag-only enums, and payload enums. Reverse-order affine cleanup is
explicit on checked return, give, and match-fallthrough edges before lowering;
an affine field move records the untouched sibling subtrees to drop at the
consuming projection, including nested paths. Struct fields may now contain
buffers: the checker expands each whole or partial owner drop into exact
reverse-order projected buffer drops before lowering, while resource-free
nominal drops need no runtime action. Source enums and concrete PRE-1
`Option`/`Result` instances may also own buffers, directly or through nested
payloads. Their checked cleanup remains one enum-owner drop; LLVM dispatches
on the active tag and recursively drops only that variant's resource-bearing
fields. A consuming match transfers its active payload to the arm binder
instead, so the root is not dropped twice. Required checks remain explicit
through lowering and emit the exact DIAG-3 record before abort.

The v0.12 activation adds one general SET-1 path for the place families the
compiler already represents: live own-mode scalar/tag-only-enum locals and
nested copy fields inside acyclic structs. Semantic checking forms the target
before the RHS, rejects constants and affine final places under their owning
rules, checks the exact RHS type, and revalidates root liveness afterward. The
checked program retains the root and field path; lowering performs an SSA
rebinding or rebuilds the required aggregate layers with LLVM `insertvalue`.
Focused host tests execute root and nested-field updates and preserve siblings.

Structured `loop` and resolved labelled `break` now use that same checked and
lowered control-flow path. Loop-entry and break-exit blocks carry the current
binding values as typed parameters, so arbitrary copy-local and tag-only-enum
updates become ordinary LLVM phi nodes. Nested labels route to their resolved
loop identities; FN-1 rejects unreachable suffixes; OWN-11 rejects consuming
an affine binding declared outside the loop; and checked break/backedge edges
retain their derived cleanup. Existing compiler-independent accumulator and
loop-carried-enum programs execute through the host backend.

Concrete PRE-1 `Result<T, E>` values now reuse the same nominal enum,
construction, matching, call, return, and aggregate-lowering path for arbitrary
currently supported T and E. `Ok` and `Err` still resolve by their unique
context-free constructor identities; only their concrete generic instance is
recovered from a written consuming type in a let, return, give, or propagation
site, and a context-free uninstantiated Result constructor remains a TYPE-5
error. Checked add, subtract, and multiply construct `Result<T, Overflow>` from
LLVM's defined overflow intrinsics without trapping. ERR-3 propagation records
its `(function, node_path)` context and lowers to an explicit Ok continuation
and Err return edge with exact same-E checking and derived cleanup. Independent
Result value-match, checked-overflow, loop, and custom propagation programs run
through the host backend, including a Result whose Ok payload is a struct.
Checked division and remainder now use the same Result, match, propagation,
checked-program, and typed-IR path for all eight integer types. The backend
tests divisor zero and signed minimum/-1 before entering a block containing
`sdiv`, `udiv`, `srem`, or `urem`; error edges construct the exact
`DivideByZero` or `DivOverflow` payload without executing a partial LLVM
instruction.
All three `iabs` modes now share one unary integer path for every signed width.
The backend calls `llvm.abs` with `is_int_min_poison = false`, so the signed
minimum edge is defined before the selected mode retains it, emits the exact
OP-2 trap record, or constructs `Err(Overflow())`.
All three `ineg` modes use the ordinary integer arithmetic path. Wrapping
negation is a plain modular subtraction from zero without overflow flags;
trapping and checked negation reuse defined signed-subtraction overflow
detection. Executable tests cover every signed width, including the minimum
edge and exact trap record.

This is not a completeness claim. Cyclic generic forms,
generic `requires`, general borrow referents and borrowed affine match
payloads, returned borrows, bound/result-carrying/grandchild reborrows, affine
moves out through owning indirection, arenas, non-flat slice elements, slice
formation through borrow holders, inline
recursive nominal layouts, branch-dependent ownership/loan joins, projected
array targets reached through borrow holders, and remaining effect-table
operations are explicit unsupported compiler capabilities rather than
source-language rejections.
Direct own returned slices use v0.17's signature-derived finite-origin path
through semantic checking, lowering, and the unchanged slice descriptor.
Region-bearing function and nominal generic arguments, region-bearing box or
arena content, and borrow-mode direct-slice results receive v0.17's specified
FN-2, STOR-5, and FN-1 rejections. Generic source contracts and source-contract
bounds retain FN-3's specified rejection; contract-member calls have no v0.17
grammar or semantic operation.
Repeated exhaustive match arms also stop as
unsupported because v0.17 defines neither duplicate-arm meaning nor a
duplicate-arm rejection rule.

The exact approved v0.17 candidate is installed as the language authority.
Compiler and conformance identities, the direct returned-slice semantic path,
and additive evidence are synchronized to that authority. The resolver
implementation completes Phase 6, the first executable scalar slice completes
Phase 7, and nominal data, the current SET-1 place family, structured loops,
and the first Result family advance Phase 8.

The compiler implements the v0.13 consuming context through one general
expression judgment shared by `match` and `propagate`. A direct bare affine
own-rooted operand consumes its whole storage root exactly once; an explicit
`move` remains valid; copy operands remain ordinary reads; and a later reuse is
rejected under OWN-1. The approved ERR-3 source repairs preserve every existing
conformance verdict and status while restoring required affine returns, exact
effect rows, complete programs, and fresh match binders. Checked
division/remainder, all three `iabs` modes, and all three `ineg` modes are
complete.

The remaining non-floating integer operation family defined by OP-1 and OP-8
is complete through one shared semantic and lowering path: trapping
division/remainder, bitwise operations, shifts, rotates, bit counts, byte swap,
high multiply, saturating arithmetic, and min/max cover their exact domains.
The backend preserves every trap edge, uses defined LLVM operations, emits no
unearned overflow flags, and widens saturating multiplication rather than
using the rejected partial intrinsic. A compiler-independent checksum-style
mix and focused host regressions exercise the family.

Concrete fixed arrays and immutable const tables now run through the normal
compiler path. Decimal and explicitly earlier integer constants determine
exact array lengths; primitive const arrays become immutable LLVM globals;
`array_new` initializes every element; `len` retains the static length; and
every direct local or const-table `index` read branches through its retained
OP-4 bounds check before the backend forms an inbounds element address.
Arrays remain affine and use the ordinary cleanup and cross-function aggregate
paths. A compiler-independent loop checksum reads a static table through a
runtime cursor and executes through host LLVM.

Indexed SET-1 for direct local fixed-array roots now uses the same array layout
and OP-4 machinery. The checked program retains the evaluated offset and trap
site; lowering creates a guarded-index value before lowering the right-hand
side, then performs one copy-element store and rebinds the rebuilt array after
the right-hand side. A failing target never evaluates the right-hand side.
A compiler-independent two-loop program fills and folds a mutable array through
that path.

Direct own-root runtime-length primitive buffers now run through
the normal compiler path. `buffer_new` computes `n * sizeof(T)` with retained u64 overflow
before allocation, checks the selected target's allocator/address domain
before the allocator call, aborts target-domain or allocator failure as
non-language TCB/resource edges, fills every element, and produces the
specified `{data pointer, u64 length}` owner.
`len`, OP-4 reads, and target-before-RHS indexed SET-1 use the runtime length;
buffers cross function boundaries as affine values; and every normal checked
owner exit emits one `free`. The effect checker now tracks `allocates(heap)`
and `traps` independently and checks both directions. A compiler-independent
two-loop program allocates, fills, folds, and releases a buffer.

Resource-bearing struct ownership now extends that path. A projected buffer
root carries one binding plus its exact field path through `len`, OP-4, and
SET-1; lowering projects it once before the offset, retained guard, and RHS.
Whole and partial struct moves publish structural reverse-order cleanup,
skipping exactly a transferred subtree. A compiler-independent two-column
structure-of-arrays checksum executes through those paths and frees both
columns.

The first lexical buffer-borrowing slice is complete. Region parameters and
local region blocks use resolved declaration identities; borrow holders retain
their mode, resolved owner/field path, and ultimate caller origin; OWN-5/7
checks prefix overlap; OWN-10 prevents local owners from escaping into caller
regions; OWN-12 substitutes explicit call regions and checks overlapping
arguments; and EFF-2 projects callee reads/writes back through the actual
storage origin. Shared and usable `&uniq` buffer holders reach `len`, OP-4, and
SET-1 only through explicit `deref`. The backend passes the existing buffer
descriptor by value and never frees a borrow. A compiler-independent
structure-of-arrays program uniquely fills two distinct borrowed columns, then
shared-borrows and folds them, while the sole owner frees both columns. Forms
that need returned-reference provenance, bound/result-carrying/grandchild
reborrows, or branch-dependent loan joins remain explicit unsupported
capability stops.

Concrete FN-8 `requires` prologues are complete. Resolution performs the
specified unit-wide structural admission before name classification. Semantic
checking then admits only own copy lets initialized by pure, total table
operations followed by the one final Bool check. The checked function retains
that prologue separately from its body, combines both effect sets, and lowering
executes the explicit OP-5 check once after parameter binding and before the
body. It creates no caller proof obligation or optimizer assumption, and no
downstream check is elided. An independent output-capacity program reads a
uniquely borrowed output length in the prologue, copies an owned input buffer
through the normal checked loop, and executes successfully.

The complete integer-to-integer OP-6 `cvt` family is implemented through one
pair judgment and one checked-IR operation. All 56 distinct ordered pairs use
that path: the 18 spec-defined widening pairs return the destination directly,
and the other 38 construct `Result<Dst, NarrowError>` after an exact signed
range judgment. LLVM uses only defined extension, truncation, comparison, and
fully initialized aggregate operations; a truncated candidate is never exposed
on an error edge. An exhaustive host matrix executes one representable and one
unrepresentable edge for every checked pair. A compiler-independent CRC32
program computes the standard `123456789` vector through checked buffer access
and the same general `u8`-to-`u32` conversion path.

Concrete PRE-1 `Option<T>` now reuses the ordinary nominal path for every
payload type the compiler can already represent. Explicitly
written Option instances are interned structurally; `None` and `Some` use their
declared variants and fields; and calls, returns, nested Options, construction,
and exhaustive matches need no Option-specific IR or backend representation.
The existing combined Result/Option program and a compiler-independent
shared-borrow byte scanner execute both `Some(value: offset)` and `None()`
edges. A context-free generic constructor still has no inferred instance.

Variant-dependent cleanup for resource-bearing enum payloads is complete
through the ordinary source-enum and concrete PRE-1 `Option`/`Result` path. A
compiler-independent fixed-size byte transform returns
`Result<buffer<u8>, DecodeError>` and executes success transfer, error cleanup,
matching, and abandonment of both active variants. Enum construction still
zero-initializes the whole inactive representation; cleanup switches on the
active tag, drops its fields in reverse declaration order, and aborts
defensively on an invalid tag. It introduces no user destructors, source
generics, replacement storage, or container growth.

The borrowed-struct slice is complete through semantic checking, checked IR,
lowering, LLVM, and execution. Helpers receive `&'r Pool` or `&uniq 'r Pool`,
read and index projected buffer fields such as `deref(pool).left`, and update
copy state such as `deref(pool).count` through a usable unique holder. One
resolved place path retains the borrowed root, field prefix, ultimate caller
origin, loan checks, and exact EFF-2 reads/writes. The implementation does not
move affine fields out of a borrow, return references, admit bound or
result-carrying child reborrows, or add arenas. Read-only slices over direct
owned and constant storage use the separate path described below; forming one
through a borrow holder remains unsupported.

The owner-approved protected corrections to five inherited runnable
conformance entries are applied. `pending-op9-buffer-new` and
`op4-trap-index-oob` now declare the `allocates(heap)` effect exhibited by
`buffer_new`; `type2-pos-buffer-tagonly` tests the legal type without calling
the primitive-only allocation row; `own1-pos-match-projected-copy` retains its
projected-copy judgment through a primitive buffer; and the FN-8 non-Bool
condition now expects its specification-selected OP-5 rejection. Their
statuses and expectation kinds are unchanged, and the corrected runnable
programs execute through the normal compiler path.

The exact v0.15 target-layout candidate above is owner-approved, installed, and
implemented. The checked program records target-domain obligations for every
implemented runtime-sized allocation and emitted element address, including
array/buffer initialization, reads, and SET-1 writes. Before emitting
target-dependent LLVM, the backend selects one exact executable-fixed host
triple and DataLayout, computes concrete aggregate/enum/array layouts, static
objects, source-call ABI objects, actual emitted stack slots and complete
frames with checked arithmetic, and rejects an unrepresentable materialization
as a target failure with no source rule. Runtime buffer lowering preserves
OP-9's u64 overflow trap first, then checks the allocator/address domain before
`malloc`; failure aborts without a DIAG-3 record. Bounds plus the established
complete layout or successful allocation invariant discharge the corresponding
address obligation. No numeric source-language cap, hidden heap fallback, new
effect, optimizer fact, or alternate lowering path was added.

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
minimal, record its durable rationale in `mcts_mem/`, and record protected owner
approval in `governance/APPROVALS.md`.

Before proposing a spec change, verify the new grammar with the grammar
verifier: a proposed specification must pass the main compiler's own lexer and
parser and satisfy the grammar constraints (parses, strong-LL(2), clean
terminal partition, no conflicts).

When the specification changes, everything derived from it is brought to the
newest version in the same work: conformance cases and verdicts, the
lexer/parser and generated syntax data, tests, and docs. This consistency is
the responsibility of whoever changes the spec; it is not machine-enforced,
and derived material is never silently weakened to make a check pass.

**Grammar proposal check.** The native `whitefoot-grammar` tool verifies an
unchanged frontend contract against the active compiler, checks the complete
FORM-1-through-GRAM contract plus the CONST-1 and EFF-1 grammar fragments,
checks every compiler SELECT_2 decision, and runs the real lexer and parser. It
fails closed on a grammar change. A future structural proposal must extend this
same native path; it may not revive the archived independent grammar engines.
This tool is run for specification proposals, not routine compilation.

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
workspace, specification governance, and compiler-independent conformance data
were established. The historical Python reference model was later archived;
it consumed its own toy AST and did not exercise or compare with the Rust
compiler.

## Phase 2: grammar evidence

Status: complete.

The historical independent evidence established the terminal partition,
grammar conflicts, and lookahead needed for the frontend. The active native
proposal check now reuses the compiler as described above and is not part of
normal compilation.

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

Status: complete.

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

The resolver covers D01-D14, X01-X03, U01-U18, and X04-X09 through one general
path. Its unit, property, mutation, selected existing-conformance, and
hostile-review cases are green. The owner reconciled the protected
`fn7-neg-two-mains` expectation with exact v0.10: the later duplicate `main`
declaration receives TYPE-6, while a missing or unique wrong-signature `main`
remains FN-7.

## Phase 7: first executable semantic slice

Status: complete.

Hostile preflight found that v0.10 deliberately leaves post-resolution
semantic diagnostic validity and determinism boundary for later approval. It
also does not close the ordinary `check` operand type, function fallthrough,
the exact scalar integer wrap/trap/compare behavior needed by the first backend
slice, or the conflict between DIAG-2/DIAG-3 product-scale artifact/report
obligations and the current research-compiler architecture. Do not invent
these behaviors in compiler code or derive them from LLVM.

The owner approved and activated a minimal v0.11 revision that closes only
those semantic boundaries, preserves all required runtime checks, and replaces
product-scale artifact/replay obligations with the smallest checked in-memory
authority and runtime-report contract the research compiler needs. Review rejected both a
whole-language diagnostic-owner census and a normative first-slice support
profile: neither affects language acceptance, and both would couple the
specification to implementation order. The candidate instead requires every
semantic rejection to establish an actual numbered-rule violation, keeps
rule-specific locations exact, makes simultaneous post-resolution first-error
choice deterministic per compiler executable, and publishes checked authority
only after every applicable whole-unit judgment succeeds. The compiler-sharing
grammar verifier and hostile review are complete, and the exact approved bytes
now define compiler behavior.

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

The completed slice uses grammar and resolved identities rather than function
or corpus allowlists. Whole-unit semantic success is the only lowering
authority. Its checked representation records exact scalar types and values,
direct calls, retained OP-2/OP-5 checks, trap attribution, and returns; lowering
then produces one target-independent IR and one conservative host LLVM path.
Wrapping arithmetic carries no LLVM overflow promises, trapping arithmetic
uses explicit signed/unsigned overflow intrinsics and branches, and explicit
checks are never elided. `unit` remains a first-class source value across
locals, parameters, calls, and returns.

Independent positive and negative conformance sources exercise constants,
FORM-7, named calls, TYPE-5, FN-2, EFF-2, wrapping arithmetic, and normal
execution through this same path. Focused host tests cover every implemented
integer width/sign lowering, mandatory OP-2 and OP-5 trap records, check
retention, and the absence of `nsw`, `nuw`, or `llvm.assume` claims.

## Phase 8: expand semantic capability

Status: in progress, and subordinate to Phase 10 since the 2026-07-24
reorientation. Freezing active behavior at v0.17 stops this phase's exit
criterion from receding, so the remaining work is a finite list rather than a
treadmill. Phase 8 no longer selects its own next family: a remaining gap is
worked when a Phase 10 slice or a dogfood port names it as the concrete
blocker, and otherwise waits.

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
and lowering path, and the compiler adapter passes the full conformance suite
after its sources and expected verdicts have been reconciled with the active
specification. The suite is evidence, not language authority: an inherited
case or expectation that contradicts the specification is corrected through
`WORKFLOW.md` and protected owner approval, never implemented merely to make an
old test pass.

The implemented nominal-data subset covers nongeneric own-mode acyclic structs
and enums, including resource-bearing variant payloads. It implements
construction, nested projection,
statement and value matching, `give`, exact GRAM-8/GRAM-10 declared-field
diagnostics, TYPE-5/TYPE-6 typing, per-site ERR-2 exhaustiveness rejection with
the missing variant list, OWN-1/OWN-13 copy-versus-affine consumption, explicit
checked cleanup edges, and tag-only enum equality through the normal
checked-program, typed-CFG, LLVM, and host-execution path. Struct fields may
own buffers; their whole and residual cleanup is expanded structurally in
reverse declaration order before lowering. Enum owners retain one
variant-dependent drop before lowering; the backend dispatches on the active
tag, recursively cleans only that variant's resource fields, and matched
payload transfer remains single-owner. Independent positive and negative cases
cover cross-function aggregate values, mixed-width and multi-field
resource-free enum payloads, every Boolean operation, nested fields, ownership
failures, wrong variants, missing arms, invalid field order, and nested buffer
cleanup.

The implemented SET-1 subset covers direct live own-mode copy locals, nested
copy fields, direct local fixed-array indices, and direct or struct-projected
buffer indices. Buffer indices may also be reached through a live usable
`&uniq` holder with explicit `deref`; the target keeps resolved provenance,
checks live loans, attributes the commit to the ultimate caller region, and
still forms its bounds guard before the RHS. One checked target record carries
the root path, evaluated offset, retained OP-4 check, and copy type across RHS
checking; lowering forms the projected root and guarded index once before the
RHS and commits one store afterward. Constants cite CONST-2, affine final
places cite STOR-1 with the required restructuring, type mismatch cites TYPE-5
at the RHS, and an RHS that moves the root cites OWN-1 at the later commit.
Projected array indices, slices, boxes, arenas, and non-buffer dereference
targets remain unsupported; they are not treated as invalid source.

This is not the complete ERR-2 toolchain contract: a whole-unit
variant-addition query that enumerates every affected match site is still
pending. The compiler adapter also does not yet implement the full independent
conformance manifest. Neither is claimed by the current green gate.

Direct recursive nominal layout and branch-dependent affine state joins remain
explicit implementation limits, as do repeated exhaustive match arms; no
source-language rule has been invented for them. Loops with a structurally
reachable break now run through the same checked CFG and LLVM path. Header and
exit block parameters carry current bindings, nested break targets use resolved
loop identities, OWN-11 blocks outer affine consumption, and normal backedges
and breaks retain explicit cleanup. A loop with no structurally reachable break
remains an explicit lowering limitation rather than a source rejection.

The first closed PRE-1 `Result` slice is implemented through one
nominal/control-flow path: contextual construction, arbitrary currently
supported payload types, calls and returns, exhaustive matching, checked
add/subtract/multiply, and explicit ERR-3 forwarding. It does not special-case
`run-ex2` or another corpus source. The v0.13 propagation-ownership rule and
approved source repairs are synchronized through that same path. Checked
division/remainder now produces `Result<T, DivError>` through this path and
guards both LLVM hazards before the partial instruction. All three `iabs`
modes use one defined-edge unary path. All three `ineg` modes reuse the
ordinary wrapping and overflow-detecting subtraction path.
Direct fixed-array index reads, immutable const-table reads, direct-root
indexed fixed-array SET-1, and direct or struct-projected primitive
runtime-length buffers are implemented. Resource-bearing struct
cleanup supports nested and partial owners. The structure-of-arrays experiment
now runs through separate uniquely borrowed fill and shared-borrowed fold
helpers, with exact loan expiry, call-region substitution, effects, checks, and
owner-only cleanup. Concrete FN-8 `requires` prologues execute before function
bodies without creating assumptions, and the borrowed output-capacity
experiment runs with every bounds check retained. Integer OP-6 conversion is
complete for all signed/unsigned pairs, and the standard CRC32 vector executes
through its general byte-to-word widening path. Concrete `Option<T>` also
executes through the normal nominal path, and a borrowed byte scanner returns
real offsets or absence without sentinel values. Resource-bearing source enums
and concrete Option/Result instances now use the same active-variant cleanup
path.

Whole acyclic struct borrows now use the same resolved-root and field-prefix
loan model as buffer borrows. Shared and unique parameters project copy-field
reads, checked buffer accesses, and copy-field SET-1 writes with the ultimate
caller storage region preserved for exact EFF-2 attribution. Checked IR
distinguishes borrowing the struct owner from copying an aggregate value;
lowering gives every actually borrowed owner one stable address, passes
borrowed struct parameters as addresses, reloads owner values for ordinary
projection and cleanup, and stores reconstructed aggregates after copy-field
updates. Call-scoped loan facts are checked against later argument-place
accesses, so correctness does not depend on putting a borrow last in a
signature. No alias promise or required check is removed.

The compiler-independent `x-borrowed-pool-tree-run` program now builds a
63-node complete binary tree bottom-up in two buffer fields, recursively checks
it through a shared whole-struct borrow, observes unique-borrowed count updates
from the caller, and releases both buffers only from the original owner.
`x-wc-chunk-summary-run` now supplies the text-processing leg: two owned byte
chunks are summarized through unique output structs, then combined through one
unique output and two shared inputs. Its general and empty-identity paths
preserve lines, words, bytes, boundary state, caller-visible writes, retained
bounds checks, and exactly-once input cleanup. It exposed no additional
compiler capability gap.

The compiler-independent `x-base64-rfc-vectors-run` program now executes the
complete scalar encoder shape against `Man`, `M`, and `Ma`. One ordinary
function handles the full three-byte group, one-byte `==` tail, and two-byte
`=` tail through the immutable 64-byte alphabet; its checked prologue relates
caller-visible output capacity to the owned input length; exact widening feeds
the bit operations; every input, table, and output index keeps its OP-4 check;
and the transferred input and caller-owned output are each released by their
actual owner. The experiment exposed no additional compiler capability gap.

The next sustained target is a complete one-shot raw RFC 1951 decoder with
caller-provided input and output storage. Correctness work proceeds through
stored, fixed-Huffman, and dynamic-Huffman streams, but those are milestones
inside one decoder rather than three unrelated fixtures. The evolving
`tests/programs/raw_deflate.wf` now executes multi-block stored streams, checks
LEN/NLEN before copying, reports truncation, invalid length, and output
shortage as ordinary `Result` failures, leaves output untouched on every
pre-copy failure, and releases the transferred input on every return edge.

That first decoder milestone exposed and now uses the general v0.14 OWN-6
statement-scoped child-reborrow path. Buffer and whole acyclic-struct holders
can form an unbound shared or mode-compatible unique child only in a
single-statement local region around an own- or unit-result call. The checked child
retains its resolved parent place and ultimate effect origin; overlapping
unique siblings reject under OWN-12; the parent is excluded while the child is
the call claim and resumes immediately afterward. The same rule works in a
loop only when the child region is introduced inside the current loop body, as
OWN-11 requires. Checked IR distinguishes a struct reborrow from borrowing a
new owner, so lowering reuses the holder's existing address; buffer children
reuse the descriptor path. No alias metadata or check elision follows.

The fixed-Huffman milestone now runs through one retained `InflateState`, one
bit reader, the complete canonical fixed literal/length code, the RFC
length/distance tables, and the same checked byte-emission path used by stored
blocks. Compiler-independent vectors cover literals, an overlapping
distance-one copy, a nonzero distance-extra field, truncation, reserved
literal/length symbols, distance-before-history, and output shortage. Every
const-table, input, history, and output index keeps its normal OP-4 guard.
The decoder and its vectors are separate source records in one closed
compilation unit so the sustained implementation remains readable.

That work exposed one general semantic-prepass gap: a partial `cvt` used
directly as a `match` scrutinee has no written `Result<Dst, NarrowError>` type
node to intern. The existing nominal prepass now derives and interns that
result for every syntactically valid non-total integer conversion before
expression checking; malformed and identity conversions still reach their
normal source judgments. A focused regression covers the unannotated shape,
and the decoder executes it through checked IR, lowering, and LLVM.

The dynamic-Huffman milestone completes the one-shot raw decoder. It reads the
code-length alphabet, expands all three bounded repeat forms, validates and
builds canonical runtime tables, and sends literals and length/distance pairs
through the same checked payload path as fixed blocks. Complete trees,
one-symbol one-bit trees, and RFC 1951's literal-only one-entry zero-length
distance alphabet have explicit representations; oversubscribed, otherwise
incomplete, missing-end, reserved-code, and distance-without-history cases are
ordinary data failures. Executable independent vectors cover literal streams,
dynamic length/distance matches, a nonfinal dynamic block followed by a fixed
block, the literal-only distance edge, malformed trees, truncation, reserved
block types, and output shortage. Every table, input, history, and output
access keeps its OP-4 check, and all allocated tables follow the existing
resource-bearing Result cleanup path. No new source semantic capability was
needed; the only compiler correction makes the LLVM test helper select an
actual function definition rather than an earlier call with the same symbol.

The next Phase 8 target is the source-generic monomorphization core. It is the
next language capability because reusable Whitefoot libraries require explicit
type and const abstraction, while duplicating a concrete helper per type or
size would repeat the rejected corpus-shaped route. This first milestone has
the following closed scope:

1. It implements TYPE-2/3/5/6, CONST-1, FN-1/2, and PROG-2 for explicitly
   instantiated acyclic source-generic structs, enums, and functions, including
   unbounded type parameters, PRE-1 `Int` bounds, and forwarded const
   parameters. Existing ownership, effects, storage, and operation judgments
   are applied again after complete substitution; they are not approximated by
   the template checker.
2. Its input is the resolved generic declarations and explicit kinded
   type/const arguments. Its output is a deterministic finite set of concrete
   nominal and function instances that the existing checked-program and
   lowering path can consume.
3. It accepts only well-kinded, fully explicit, concretely valid reachable
   instances and all zero-monomorphization functions. It reports actual
   numbered-rule violations normally. Cyclic generic calls, region-bearing
   generic arguments, source contracts/conformances/laws, and generic
   `requires` remain explicit unsupported capabilities in this milestone; no
   interpretation of their unresolved rules is introduced.
4. The immediate downstream consumer needs complete normalized substitutions,
   concrete signatures and nominal member tables, source-ordered typed call
   records, and one stable concrete symbol per semantic instance. It does not
   need serialized identities, cross-instance body sharing, or an alternate
   generic IR.
5. Independent tests must cover one declaration instantiated at multiple
   concrete types and const sizes, nested instance discovery, forwarding across
   source records, wrong kind/arity, a concretely invalid body, deterministic
   closure order, and a generic call cycle that stops before instance
   enumeration rather than expanding.

Template checking precedes instance discovery; every admitted instance is then
rechecked through the one semantic path before lowering. Do not implement
generics by source-text expansion, inferred arguments, function or corpus
allowlists, backend substitution, or an instance worklist that runs before the
finite template-call graph has ruled out unsupported cycles.

This bounded source-generic milestone is now in place. Type, `Int`-bounded, and
integer-typed const parameters have distinct kinded terms; const identifiers
remain symbolic through template calls and become u64 values only in concrete
instances. Source-generic struct and enum templates receive symbolic member
coverage, then each reachable kind-correct substitution gets one concrete
nominal identity and fully substituted member table. Symbolic and concrete
`Option`/`Result` instances, checked integer results, and generic arrays and
buffers use those same tables rather than a backend substitution path.

Template-call cycles stop before closure, acyclic nested calls are checked
before reachability, and the deterministic concrete worklist rechecks every
reachable function instance through the ordinary semantic path. Each function
instance has one collision-free internal symbol while diagnostics retain the
source declaration name. Independent semantic and executable tests cover
multiple concrete type and const instances, nested nominal discovery,
cross-record forwarding, exact kind/arity failures, constructor-only
diagnostics, a symbolically valid body that fails its concrete OWN-1 recheck,
deterministic closure, and a generic call cycle that stops before instance
enumeration. `generic_instances.wf` and `generic_nominals.wf` execute the
ordinary checked-IR and LLVM path.

The v0.15 STOR-6 target-layout slice runs through the same checked-program and
LLVM path. The exact v0.16 static-conformance closure now runs as one normal
semantic pass over the resolved contract table. It checks FN-3 whole-
conformance identity, complete declared-order bindings, exact callable
signatures with normalized effects, and mandatory FN-4 discharge before
checked-program publication. The resulting contract, conformance, binding, and
base-law records are semantic metadata only: lowering ignores them, bound
functions remain on the ordinary direct-function path, and no second dispatch
IR, runtime object, inferred conformance, trusted law, check elimination, or
optimizer fact exists. Generic source contracts and source-contract bounds
receive their specified FN-3 rejections; contract member calls remain absent.
This closes the selected Phase 8 static-conformance milestone.

The first Phase 9 text probes are current-spec ports of the preserved
percent-decoder and stateful UTF-8 parser kernels. Both execute through the
public compiler pipeline with their ordinary bounds guards and trap paths
retained, and neither exposed a missing compiler capability. Repository-owned
program sources remain in `tests/programs/`; Cargo exercises them from the
`compiler/tests/programs.rs` integration boundary, while private backend tests
remain responsible only for lowering and emission invariants.

The first graph-shaped Phase 9 target is a naturally recursive binary tree,
not an index-pool substitute. It selected the general `box<T>` owner path:
structural box type formation, `box_new`, explicit dereference, shared
borrowing, box payloads in recursive enums, finite pointer-based layout,
heap allocation, and compiler-derived recursive cleanup now pass through the
ordinary checked program, typed IR, LLVM, and public integration boundary.
The tree builds seven nodes, traverses them recursively through shared box
borrows, and releases the complete graph from one root drop. Allocation
failure remains a non-language abort edge. Moving an affine referent out of a
box remains an explicit unsupported capability because v0.16 does not define
the resulting empty owner or allocation cleanup; this implementation does not
invent one.

The owner-approved protected reconciliation corrects the two inherited
positive box cases to declare OP-1's required `allocates(heap)` effect and
makes them runnable now that allocation, dereference, and derived cleanup use
the ordinary compiler path. Their successful-run expectations and rule
assignments are unchanged. The borrowed inline affine-payload case remains
pending and outside this box slice.

The sustained-workload Phase 9 target is a complete SHA-256 compression of the
standard `abc` block, repeated 1,024 times. It builds and mutates the 64-word
message schedule, performs all 64 compression rounds with rotates, shifts,
Boolean bit operations, wrapping arithmetic, constant-table reads, and checked
runtime indexing, and verifies the accumulated known result. It executes
through the public compiler boundary without exposing a new capability gap,
which is evidence that the existing integer, array, loop, call, and SET-1 paths
compose under sustained use rather than a reason to add SHA-specific compiler
machinery.

The next Phase 9 domain probe is a 4,096-step feedback controller. It selected
the concrete scalar floating-point family rather than controller-shaped
machinery. `f32` and `f64` types and finite literals now retain exact IEEE bits;
FORM-5/FORM-7 canonicalization searches the bounded round-trip interval and
applies the specification's shortest-byte and lexicographic rules rather than
accepting the host formatter's preferred decimal. Every direct OP-1 float
operation uses one checked operation enum and one typed-IR path for both
widths. LLVM emission uses strict arithmetic without fast-math flags, the exact
OP-8 intrinsics, ordered comparisons except unordered `fne`, canonical quiet
NaNs, and signed-zero-preserving minimum and maximum. Floats compose through
calls, loop-carried mutation, structs, const arrays, buffers, checked indexing,
and SET-1. Executable edge tests cover both widths, NaN propagation, infinities,
and signed zero. `Float`-bounded templates now reuse this concrete path after
monomorphization as described below.

A 64-by-48 Mandelbrot grid then selected OP-6's complete total-conversion
family with a floating-point endpoint. One numeric-conversion judgment now
classifies all integer pairs and the eleven exact total float-related pairs;
the checked program retains concrete source, destination, and result types,
and the typed IR carries that judgment unchanged. LLVM uses signed or unsigned
integer-to-float conversion only for the exact-width rows and `fpext` for
`f32` to `f64`. The grid executes nested loops, calls, mutation, integer
control, strict float arithmetic, and `u32`-to-`f64` conversion through the
public compiler boundary.

An RGB-to-grayscale image kernel then selected OP-6's partial conversions with
a floating-point endpoint. The same numeric-conversion judgment now covers all
90 ordered pairs of distinct concrete numeric primitives: 29 total pairs
return the destination directly, and the other 61 return
`Result<Dst, NarrowError>`. For integer-to-float conversion, LLVM's ordinary
cast produces the candidate and a defined saturating reverse cast plus an
integer-maximum collision guard proves exactness. Float-to-integer conversion
uses the saturating cast before testing an exact round trip, so NaN, infinity,
fractions, and out-of-range values never reach a poisoning LLVM conversion.
Float narrowing uses `fptrunc` plus an exact widening check; infinity succeeds,
and either direction maps NaN to the destination's canonical quiet NaN.
Executable tests cover every float-endpoint pair and the range boundaries.
The image program converts RGB bytes to strict `f32`, rounds an eight-pixel
grayscale result, converts it exactly back to `u8`, and writes it through a
uniquely borrowed output buffer.

A big-endian telemetry packet then selected OP-1/OP-8 bit-preserving
`reinterpret`. One semantic judgment admits exactly the 16 listed equal-width
primitive pairs and retains the concrete source and destination types.
Lowering carries that judgment without consulting source spelling. LLVM uses
`bitcast` for integer/float pairs and an integer identity operation for
same-width signed/unsigned relabelling, whose LLVM storage type is already the
same. Family-wide executable tests preserve every integer bit, including
noncanonical NaN payloads, in both directions. The public packet program
serializes normal `f32`, negative zero, and NaN values to network-order bytes,
loads them again through checked buffer accesses, and verifies their value,
sign, and NaN behavior.

A finite-impulse-response audio filter then selected fixed arrays projected
through direct own-rooted nested structs. The checked array root now retains
one binding plus its complete field path for `len`, guarded `index` reads, and
indexed SET-1 targets. A projected write is checked against overlapping loans,
forms and guards its target before the right-hand side, and revalidates the
whole owner afterward. Lowering projects the current array value, performs one
guarded element replacement, and rebuilds every enclosing struct layer while
preserving siblings. Focused tests cover nested paths, shared-loan conflicts,
RHS owner consumption, update ordering, and sibling preservation. The public
eight-tap filter executes a 64-frame impulse response through nested array
state and strict `f64` arithmetic. Array places reached through borrow holders
remain part of the explicitly unsupported general-borrow-referent family.

An IPv4 header checksum then selected the read-only slice family. `slice_of`
now forms `slice<'r, T>` over a direct owned array or primitive buffer and over
an immutable const array, with exact OWN-10/11 lifetime checks and one shared
loan over the resolved source place. Moving and passing the descriptor
preserves its source provenance; live slices block source writes, moves, and
unique borrows under OWN-5. `len` and guarded `index` use one checked slice
root, incoming slice reads contribute `reads('r)` under EFF-2, and SET-1
continues to reject slice-rooted targets because the view is read-only.
Lowering uses a distinct non-owning `{pointer, u64 length}` IR type: array
sources receive stable stack or static storage, buffer sources reuse their
owner's allocation, OP-4 guards every element address, and dropping a slice
never frees its source. The public checksum executes the same consumer over a
20-byte const header and a runtime buffer and verifies `0xb890` in both cases.
Direct own returned slices now use v0.17's signature-derived finite-origin
semantics through the normal compiler path. Region-scoped source claims are
independent of descriptor bindings, union conservatively across control joins,
and end only with their named data region. Non-flat slice elements and slice
formation through `deref` borrow holders remain valid but explicitly
unsupported compiler capabilities, while branch-dependent slice-value
provenance joins are specified OWN-5 rejections. Region-bearing function and
nominal generic arguments are specified FN-2 rejections rather than
unsupported compiler capabilities.

A precision-polymorphic 3D vector kernel then selected the built-in `Float`
generic bound. Symbolic checking now distinguishes a `Float`-bounded type
parameter from an unbounded parameter and an `Int`-bounded parameter, admits
exactly the OP-1 floating rows for it, and treats it as copy and primitive-flat
because every admitted substitution is `f32` or `f64`. FORM-5 `0_T` and `1_T`
remain symbolic only while the template is checked and become exact integer or
IEEE identity bits during each concrete recheck. Generic float struct fields,
arrays, buffers, indexing, and calls therefore reuse the existing substitution
and concrete semantic path; checked IR and LLVM see only ordinary concrete
types. The public vector dot product instantiates one source definition at both
float widths and executes both bodies. Type-dependent generic `cvt` and
`reinterpret` remain explicit unsupported capabilities rather than selecting a
result from an expected type.

A recursive prefix-expression parser then exercised compiler construction as a
dogfood domain. It reads a runtime byte buffer through a shared region, builds
a naturally recursive boxed AST, returns resource-bearing parse results,
propagates truncation after partially building the tree, evaluates the accepted
tree recursively through shared box borrows, and releases both successful and
abandoned ownership paths through compiler-derived cleanup. Valid and
truncated inputs execute through the public compiler boundary. This probe
exposed no new semantic capability gap; it provides composition evidence for
the existing buffer, Result, box, recursion, borrow, effect, and cleanup paths.

The owner reoriented this roadmap on 2026-07-24. The constitution's P0 states
that machine performance is the reason this project exists, yet every recorded
fact-channel measurement predates the current compiler, the active compiler
links its executables without optimization, and Phase 10 had no starts. The
v0.17 language is broad enough to compile the performance workloads Phase 10
needs, so language amendment now yields to performance evidence: no further
specification amendment starts without naming the measured experiment it
unblocks. The exact next work is the first Phase 10 slice defined below.

The owner extended that reorientation on 2026-07-27 with three decisions.

First, Phase 10's remaining work is now a finite completion set, enumerated in
the Phase 10 section below. Its green state plus an explicit owner go decision
is the sole entry gate for Phase 11. Phase 10 no longer accretes new slices
beyond that set.

Second, declared parallelism is the next major direction after Phase 10 and is
defined as Phase 11 below. The strong automatic claim stays dead; the pursued
claim is writer-declared, compiler-verified parallelism. Parallelism research
and implementation are one program, not two stages: its experiments need the
Phase 10 instruments and leave durable language and compiler changes behind, so
a research-only side track is a fiction. The sequencing consequence is this
roadmap's shape — finish the work parallelism is unlikely to affect, then enter
Phase 11 wholly instead of context-switching.

Third, the W1 reading is reframed. The durable property is the floor stated as
language behavior — default shape is optimal shape: an accepted program has
been forced onto a fast shape, and the writer's only alternative is a program
that does not compile. Measured model capability is no longer a gate anywhere
in this roadmap, because model behavior is unpredictable and improves
independently of this project. Models appear below only as generators of
realistic mistakes, in the floor audit. The constitution's W1 wording and R0
three-leg reading still carry the old weak-model framing; amending them is
protected owner-authored work and is pending. Until that amendment lands, R0
deltas cited in this roadmap use the floor property in place of measured
weak-model success.

Deferred until the Phase 11 result is in, or until the owner reorders, each
keeping its existing re-entry bar of a measured experiment or port naming it as
the concrete blocker: the headline artifact ladder (crc32 preload, LZ4 decode,
zlib inflate) and with it the take/replace-versus-sealed-kernel storage
decision it would force; contract member calls (P5); and the constant-time
`secret` effect, which stays carded in
`research/notes/headline-artifact-brainstorm.md` with its honest blocker — a
backend constant-time-preservation contract, since optimized LLVM lowering may
legally reintroduce secret-dependent branches.

The v0.18 loan-lifetime and scope candidate is parked, not abandoned. Its
exact bytes remain in `governance/spec-evolution/kernel-spec-v0.18-candidate.md`
as a non-authoritative design record; active behavior stays v0.17. It re-enters
review only when a measured experiment or dogfood port names the loan-lifetime
wall as its concrete blocker, the way v0.7's bounded reborrow was driven by
measured blocked sites. When it re-enters, the previously stated package bar is
unchanged: complete impact inventory, validation evidence comparable to the
reborrow investigation, and multiple exact-byte hostile-review rounds before
owner review, without pulling arena cleanup, stored slice leaves, or
borrow-producing branch joins into the amendment.

Phase 9 dogfood selection now serves channel pressure: recorded evidence says
facts are neutral on single-buffer byte kernels, so the next targets are chosen
for aliasing, effect, or law pressure. Cyclic generic calls, generic
`requires`, and type-dependent generic `cvt`/`reinterpret` remain explicit
unsupported compiler capabilities. Region-bearing function and nominal generic
arguments are v0.17 FN-2 source rejections, not members of that unsupported
set.

## Phase 9: dogfood and language iteration

Continuously use production-shaped but manageable projects to reveal missing
features and bad design. zlib remains a useful example, not a privileged
target.

Selection is by channel pressure since the 2026-07-24 reorientation, not by
domain coverage. Recorded evidence puts facts at neutral on single-accumulator
byte kernels, because those are latency-bound and out-of-order execution hides
the eliminated loads, so a new target earns its place by exercising aliasing,
effect, or law pressure. The earlier coverage list — binary data/compression,
text and command-line processing, collections or graph-shaped work, and one
sustained workload — is satisfied by the current corpus and is now a breadth
check on that corpus rather than the criterion for the next target.

When dogfood reveals a language problem, change the specification through the
numbered process and update the compiler. When it reveals a compiler problem,
add a minimized independent regression. When it reveals performance behavior,
measure before redesigning.

The compiler is successful when it can support these experiments reliably and
can be changed without repeatedly rebuilding unrelated infrastructure.

## Phase 10: optimizer experiments

Status: in progress; opened 2026-07-24 by the owner reorientation above.

The first slice is an honest facts-off baseline. It changes no language
behavior and needs no specification work:

1. Repair conservative array lowering first. Dynamic indexing of an
   `array<T,N>` value emits a fresh `alloca` plus a whole-array store at the
   point of use, inside loop bodies, so stack use grows per iteration and a
   long loop dies with no DIAG record: `fir_filter` exits 0 at 5,000 frames
   and 139 at 10,000, at both no-flag and `-O2`. That is a compiler defect,
   not a language gap, and it is a prerequisite for the workload drivers
   rather than an optimization. Hoisting the slot to the entry block was
   validated by IR surgery: the crash is gone at 100,000 frames and `sha256`
   loses 18% of its `-O2` instructions.
2. Make optimized compilation the driver's single behavior over the same
   emitted LLVM. There is no writer-facing optimization level: the default
   shape must be the optimal shape, and where semantics are identical no
   writer decision exists, so offering an unoptimized alternative would only
   offer a worse one. Facts-off semantics, every retained check, and the exact
   DIAG-3 trap record stay intact, and the level itself must be one shared
   definition consumed by all three sites that invoke `clang`: the driver at
   `compiler/src/bin/whitefootc.rs:51` and the two test helpers that duplicate
   the invocation with no `-O` flag, `compiler/src/backend/tests.rs:135` and
   `compiler/tests/programs/support.rs:44`. Editing only the driver leaves
   every test compiling at `-O0`, so the existing assertions hold unchanged
   trivially and prove nothing. The proof must also be stated honestly: at
   `-O2` all 12 DIAG-3 record kernels constant-fold to the same
   28-instruction `write(2, <constant>, len); abort()` with every check and
   both `malloc` and `free` deleted, so they prove that the record plumbing
   survives optimization and nothing about check survival. The step therefore
   additionally requires at least one regression that traps out of a
   non-foldable optimized loop. Four settings are excluded, not merely
   unchosen: fast-math, because v0.17 makes floats IEEE-754 with no
   reassociation or contraction and states that a relaxed float op would
   arrive as a distinct OPNAME, so a build flag would settle an open
   source-language question and would also displace FN-4's proof-licensed
   reassociation; `-march=native`, because R6 forbids marrying one ISA, v0.15
   already fixes one exact target triple and DataLayout, and machine-dependent
   output destroys cross-machine comparability; and profile-guided
   optimization, because the founding static-versus-profile decision makes
   verified facts the thesis and sampled profiles its alternative. Link-time
   optimization is deferred rather than excluded: it is moot while every user
   function is `internal` in one module, and it is the recorded adversary for
   the effect-attribute channel, whose claim is LTO-class results without LTO,
   so enabling it by default would erase the comparison it must win.
   `-O2` versus `-O3` has no runtime evidence yet: every measured difference
   is a size change — `fir_filter` 120 to 209 instructions,
   `prefix_expression` 221 to 248, `raw_deflate` 2452 to 2672, `sha256` 1234
   to 1232 — plus `ipv4_checksum` folding to 2 instructions at `-O3`. `-O2` is
   provisionally selected, and the choice is revisited when the workload
   drivers land.
3. Give each measured kernel a scaled workload driver — input size and
   repetition count stated in its experiment directory — so wall-clock exceeds
   process-startup noise. This is a prerequisite for the attribution harness,
   not a peer task. The current `tests/programs/` corpus is correctness tests
   over fixed tiny inputs and every member finishes inside startup, so no
   member is timeable as it stands. A driver adds no language surface and must
   not weaken the correctness assertion it wraps. Each driver is gated on one
   thing only: a two-size scaling check at the measured optimization level,
   timed interleaved as a minimum of repeated runs, because this machine drifts
   by around 60 percent across non-interleaved runs. The presence of the
   kernel's own call in the optimized assembly is explicitly *not* a valid
   gate: a measured counter-example keeps `bl _wf_*` and 891 instructions of
   kernel in the binary while `licm` hoists the loop-invariant call out and
   loop deletion removes the loop, so it executes once and both workload sizes
   report 0.00s. A driver that fails the scaling gate yields a void number, and
   the attribution harness cannot catch it, because a deleted kernel looks
   exactly like an inert fact.
   The SHA-256 repetition driver already fails that way: it is deleted
   outright at `-O2`, user time flat at 0.00s across 65536, 262144 and 1048576
   repetitions while `-O0` scales 0.67s, 2.58s and 10.26s. SHA-256 is
   therefore reclassified as a port — a message-length parameter and the full
   8-word state, roughly 80 to 120 lines — and not a driver. `raw-deflate`
   cannot grow its input inside Whitefoot at all, because the repo has only a
   decoder, so a larger stream is generated out of band and embedded as a
   `const array<u8, N>`; that route is verified at 214,011 bytes. CRC32,
   base64 and FIR scale fine, at roughly 19 to 63 driver lines each. Timing
   uses user time or discards a warm-up run, because a freshly linked binary
   costs about 0.2s real on its first execution, and there is no clock in the
   language, so every measurement is taken by the harness.
4. Build the attribution harness, because a fact channel that reports no gain
   must be distinguishable from a fact the backend never consumed, and the
   second must fail loudly instead of reading as a negative result. It has
   four parts, and every invocation in it pins `/usr/bin/clang` by absolute
   path, because a bare `clang` here resolves to a wasi-sdk build that
   silently retargets the module. A codegen diff between facts-on and
   facts-off output at one optimization level is the gate, taken on IR rather
   than on assembly: identical optimized IR proves the fact was inert, and no
   timing number from that pair may then be cited for or against the channel;
   differing IR proves consumption and localizes it per function. An assembly
   gate is weak in both directions, scoring scheduler and register-allocation
   noise over identical optimized IR as a regression, and a bare store reorder
   as consumption. Captured LLVM optimization remarks
   (`-Rpass`, `-Rpass-missed`, `-fsave-optimization-record`) supply the
   diagnosis, turning a null into a named pass and reason; debug-info emission
   is a prerequisite for any remark-based prediction, because the compiler
   emits none and every remark therefore lands at `<unknown>:0:0`, which makes
   a remark prediction unattributable in principle. A positive canary per
   channel — a kernel where the fact is the only thing that can unlock one
   named transform, asserted on the resulting assembly — guards the instrument
   itself, so a canary that stops flipping indicts the harness rather than the
   thesis; the recorded `accumulate` reduction, which collapses to one `madd`
   under `noalias` and stays an alias-guarded loop without it, is the model.
   Pipeline confirmation that the consuming pass runs at the selected level at
   all is the cheap precondition for the other three, and is also how the
   provisional `-O2` choice is revisited per kernel instead of by judgment.
5. Measure the kernel set against C or Rust equivalents at matched
   optimization and at both `-O2` and `-O3`, since the recorded Rust adversary
   ran `opt-level=3` and a weaker baseline flatters our own facts; report both
   arms so the numbers stay comparable with the recorded democ `-O2` results.
   Use the `research/experiments/` discipline: one directory
   per experiment with sources, a run script, and a RESULTS.md carrying
   measured numbers and honest caveats, now bound to the Rust compiler. The
   set is at minimum the raw-deflate decoder, SHA-256, and the FIR filter,
   which are already `tests/programs/` members, plus base64 and CRC32, which
   are not: they exist only as `tests/conformance/cases/`
   `x-base64-rfc-vectors-run.wf` and `x-crc32-standard-vector-run.wf` and as
   democ-era sources under `research/experiments/port-study/base64/` and
   `research/experiments/crc32-swap-in/`. Porting those two into
   `tests/programs/` is part of this slice.
6. Record, per kernel and per level, the retained-check tax and the
   conservative-lowering tax, and rank the candidate channels by measured
   opportunity. The measured ranking is already in hand. Conservative array
   lowering leads it and is not a channel at all: 472 of 1015 signal remarks
   (`sha256` 264, `utf8parse` 140, `percent_decode` 48, `fir_filter` 20) and
   60% of `sha256`'s `-O2` instruction stream, with no aliasing, effect, law or
   bounds fact touching any of it, which is why it is step 1. The step-1 entry
   slot hoist is only the shallow half of that repair; the whole-aggregate
   copy-modify-reload remains. The first measurement this slice owes is
   therefore a re-run of the fold-resistant SHA-256 feedback pattern on the
   post-hoist compiler. Its one recorded reading — Whitefoot 1.16s against C
   0.07s and Rust 0.08s at 500,000 blocks, beside exact parity at 0.14s on a
   register-only recurrence — was taken while the hoist was landing and is
   therefore unattributable to either compiler. If any large multiple survives
   re-measurement, array lowering is the dominant P0 gap and outranks every
   fact channel by an order of magnitude; until it is re-run, that reading is
   cited nowhere. Bounds-proof
   elision is next: LLVM already discharges 89% unaided, 313 emitted safety
   checks down to 34 surviving at `-O2` (`raw_deflate` 16, `utf8parse` 10,
   `percent_decode` 3, `fir_filter` 2, `recursive_tree` 2,
   `prefix_expression` 1), so the channel is judged on the residual 34 and
   never on 313; it needs no LLVM attribute and no ABI change, and it carries
   the R0 argument. Effect attributes come third: they are consumable
   in-module — 0.54s to 0.00s measured with `nounwind memory(argmem: read)` on
   a real corpus function, which refutes the separate-compilation
   hypothesis — but they are soundness-gated, as the second slice below
   records. Scoped alias comes fourth: cheap via per-access metadata, but with
   zero sites in the corpus, because LLVM inlines every internal kernel into
   `main`, where buffers come from `noalias` `malloc` and it already has the
   fact; corpus-wide there are zero vectorized loops and zero surviving user
   functions with two or more buffer parameters. Checked law is dropped from
   this slice: zero measured opportunity, with no reassociation-refusal remark
   anywhere.

**Exit:** RESULTS.md numbers for the kernel set at both levels, a named
delta-or-gap versus Rust/C for each under the constitution's R0 test, a ranked
channel list that selects the second slice, and — per candidate channel — the
specific missed-transform remarks from the baseline log that the channel is
predicted to flip. Those predictions are registered before the channel is
built, so its result is falsifiable in advance rather than interpreted
afterwards. The strongest available pre-registration is bounds-proof elision:
it must flip the `loop-vectorize` `MissedDetails` remarks `Cannot vectorize
potentially faulting early exit loop` — 6 of them in `raw_deflate` `main` — and
`cannot identify array bounds` — 6 remaining, `raw_deflate` 5 and `utf8parse`
1 — and must drop surviving safety-check trap sites below 34, while
changing neither `gvn`/`LoadClobbered` nor the 106 surviving user `check`
assertions. That bounds baseline is measured after the array-lowering repair in
step 1: before it the count was 10, of which `sha256` carried 3, and the repair
took `sha256` to zero, so `sha256` can no longer be named a carrier. Any
pre-registration measured before a lowering change is void for the compiler that
then exists. Any check counter must classify the 148 user-written `check`
assertions separately from buffer, array, slice and overflow safety checks,
because the two are indistinguishable to a naive `wf_trap` counter and would
produce a false win. The earlier `grayscale` alias pre-registration is
withdrawn: its number was wrong (21 `gvn` remarks at `-O2`, not seven, and 42
under the wasi clang), its shape was wrong (clearing all 21 leaves the assembly
byte-identical, which this slice's own gate defines as proof the fact was
inert), and it is unattributable in principle while no debug info is emitted.
`grayscale` compiles at `-O2` to `mov w0, #0; ret`.

The expected second slice is the first fact channel on the active compiler, and
the baseline reranks which one that is: proof-based OP-4 bounds elision goes
first on measured opportunity, carrying the facts toggle, facts-off identity,
independent attribution, and hostile negative canaries that the design memory
already requires of any fact family. The frequency study's recorded conclusion
makes relational bounds proofs in a real workload the next bounded bet, so the
proof lane and the first channel are now one slice rather than two. It also
carries the largest share of the R0 question: rustc already emits alias and
read-only attributes from its own type system, so attribute channels chase
parity plus boundary cases, while a checked entry contract that discharges
downstream bounds checks has no Rust-source equivalent at all.

Effect attributes move to third, and their entry work is a design decision
rather than an emission, for three measured reasons. First, a soundness hazard:
`nounwind willreturn memory(argmem: read)` on a trapping function deletes its
required trap — exit 0 and empty stderr — while every other attribute
combination correctly aborts, and v0.17 EFF-3 already forbids that outcome, so
the emission rule is that v0.17 never emits `willreturn` on any row. Second,
`memory(argmem: ...)` is unsound on the `{ptr,i64}` buffer and slice ABI: a
store through a pointer extracted from a by-value struct argument is deleted,
so `writes` rows need that ABI split into separate `ptr` and `i64` parameters
before anything can be emitted, while `reads`-only rows can use `memory(read)`
today. Third, no non-degenerate magnitude has been demonstrated. Scoped alias
is deferred until a non-inlinable or opaque-input kernel exists.

One Phase 8 gap is already known to sit under the alias channel. Borrows of
scalar referents are unimplemented: `&'r i32` and `&uniq 'r i32` report
unsupported `RegionsAndBorrows`. Compiling every conformance case through the
public driver reports that gap for 35 cases, and 39 report some unsupported
capability. Those are compiler-behavior counts, not manifest statuses: the
manifest records only 14 `pending`, so it understates the gap by 21 cases
because `make conformance-run` has no adapter and nothing reconciles the two.
Borrows of aggregate referents work, including unbound `&uniq`
argument temporaries and two sequential temporaries over one source. The
recorded 22x reduction result used scalar `&uniq i64` and `&'r i64`, so
reproducing that kernel verbatim needs scalar-referent borrows first. The alias
channel may instead be opened over aggregate referents, as the recorded
scoped-alias experiment did; whichever route is taken must be stated in the
slice, because only the first requires a named Phase 8 prerequisite. The route
is now constrained from a second direction, independently of scalar referents:
LLVM parameter attributes — `noalias`, `readonly`, `nocapture`, `readnone`,
`align`, `dereferenceable`, `nonnull` — are verifier errors on a `{ptr,i64}`
by-value struct parameter, so the alias channel is a choice between per-access
metadata and an ABI split whichever referent kind it opens over.

The deepest question open under both slices is whether a fold-resistant channel
workload can exist at all without an opaque input. The language has no input
facility — no stdin, no argv, no getenv, no clock, and FN-7 forbids parameters
on `main` — so every input is a compile-time constant, `-O2` already folds 4 to
5 of the 15 corpus groups to 2 instructions, and a purpose-built driven kernel
folded as well. Every number in the first and second slices depends on the
answer. That question is now settled affirmatively by measurement, so no input
path and no harness escape is required to open the slice, and the routing
decision it would otherwise have forced does not arise.

Fold resistance is achievable by construction, and needs two independent
ingredients. The driver's loop-carried recurrence must have no SCEV closed
form: LLVM's exit-value folding rests on add-recurrences, so a bitwise
xor-shift or an integer multiply-add recurrence resists it structurally while
an affine accumulator such as `total += i` always folds. The kernel's own
result must then re-enter its input through a non-convergent mixer — feeding a
raw checksum back reached a fixed point in one step — and the final value must
be observed through the only observable channel the language has, a `check`
trap record, so the chain cannot be discharged without running it. The expected
value comes from an independent reference implementation, which doubles as a
cross-implementation correctness check. Separately, the kernel must be
non-specializable: its inner trip count must exceed the full-unroll budget and
its input buffer must be large enough that constant-propagating it is
unprofitable. Measured boundary: SHA-256's 48-iteration schedule and 64-round
loops are safe, and a 2048-iteration checksum over 4096 bytes is safe, while a
10-iteration checksum over 20 bytes was fully unrolled with its 18 constant
bytes folded into five immediate adds and its allocation removed entirely.
Verified patterns scale at 2.00x for 2x work at the measured level.

Two honest limits on that result. The feedback wire itself costs nothing
measurable on a real kernel — the same SHA-256 kernel with 15 of 16 input words
constant versus all 16 opaque timed 0.61s against 0.60s — but feedback does not
prevent input specialization on a small kernel, which is a constant-input
problem rather than a constant-trip-count one and is the one thing a real input
path would still fix. And the residual case measured instruction-for-instruction
identical to what the host C compiler produces from the same source, so the
distortion is shared by the comparison rather than unique to Whitefoot.

Phase 10 states its falsifiable branch in advance.
`research/notes/regions-effects-vs-safe-rust-2026-07-08.md` already records
three of four probes at parity-with-effort against safe Rust, with one 1.1-1.5x
structural residual on a niche kernel, and `docs/why-whitefoot.md` concedes
the frequency question as the biggest honest unknown. If the channels and the
proof lane land and the kernel set still sits at Rust parity on machine
performance, that is an experimental result, not an execution failure: the
constitution's affirmed R0 reading names the surviving deltas — W3
cheat-proofness and W1 weak-writer robustness — and the project's thesis
narrows to them. A parity or negative result enters RESULTS.md under the same
discipline as a win. This branch is recorded before the measurements so a
parity outcome remains a measured conclusion rather than a rationalization.

Keep facts-off compilation correct. Add proof-based check removal and other
optimizations one proposition family at a time, with focused independent
verification and facts-on/facts-off comparisons. Optimize measured problems,
not hypothetical workloads.

The preserved `tests/codegen/` sources are a pool of historical experiments,
not a completion checklist or an active gate. Do not revive the democ runner or
assume its manifests and expected code shapes are correct. Promote a selected
case into a small regression owned by the current Rust compiler only after
reconciling its source, semantic expectation, and code-generation hypothesis
with the active specification and the experiment that needs it; explicitly
retire obsolete hypotheses instead of preserving them as accidental compiler
requirements.

An optimizer fact may improve an accepted program but may not change source
acceptance or remove a required check without proof.

### The Phase 10 completion set

Fixed by the owner 2026-07-27. Phase 10's remaining work is exactly this list;
its green state plus an explicit owner go decision recorded in this file is the
sole entry gate for Phase 11. The slice definitions above remain authoritative
for every item they cover; nothing here relaxes them. Items 9 and 10 may be
explicitly waived by the owner at the gate; the others may not.

1. The array-lowering repair's second half — the whole-aggregate
   copy-modify-reload on indexed writes — plus the SHA-256 feedback
   re-measurement the first slice already owes.
2. Steps 3 through 6 of the first slice as specified above: scaled workload
   drivers behind the two-size scaling gate, the attribution harness, the
   C/Rust kernel table at both levels, and the recorded check-tax and
   lowering-tax ranking.
3. Two deciding measurements for Phase 11, landed now as ordinary
   `research/experiments/` entries: single-core STREAM bandwidth on the
   development machine, and at least one corpus kernel above roughly 4 ops per
   byte — matrix work, compression encoding, or parsing with validation. Both
   are days of work, and the new kernel doubles as a fold-resistant
   channel-pressure workload.
4. The buffer/slice ABI split from one `{ptr, i64}` by-value struct parameter
   into separate pointer and length parameters, already recorded above as the
   precondition for any attribute channel.
5. Tier-0 fact emission: per-access alias-scope metadata derived from borrow
   modes, `memory(...)` attributes from effect rows only where sound, access
   groups on loops the existing analysis proves independent, and never
   `willreturn` — the tripwire regression at
   `compiler/src/backend/tests/effect_attributes.rs` stands. This improves
   serial code and is independent of any later threading decision.
6. The bounds-elision channel — the second slice above, unchanged: the facts
   toggle, facts-off identity, attribution evidence, hostile negative canaries,
   and the pre-registered remark flips.
7. A differential trust instrument built alongside item 6: generated legal
   programs executed facts-on and facts-off, with identical outputs and
   byte-identical trap records required. A fact family that elides checks does
   not ship without it. The same instrument is Phase 11's determinism tester.
8. Vectorization repairs: bounds checks hoisted into loop preconditions,
   overflow checks sunk out of loop bodies through OR-reduced failure bits with
   the trap after the loop, and countable single-exit loop canonicalization.
   The soundness ground is that traps abort and no global mutable state exists,
   so nothing observable can occur between a faulting operation and its abort.
   This item contains the set's one language amendment — DIAG-3
   first-failing-site attribution under check motion — which follows the
   complete language-change loop in `WORKFLOW.md` and is written as deliberate
   precedent for Phase 11's trap-selection question (owner ruling 2026-07-27).
9. Floor-audit round 1, bounded to the kernels measured in item 2. For each
   task: the measured expert shape is the reference; an AI writes the same task
   from the teaching pack; the result is diffed against the reference; every
   divergence is triaged as checker-rejected, measured-equal, or
   slower-and-ungated. Each slower-and-ungated divergence is recorded as a
   named defect — a rejection-rule candidate, a `docs/patterns.md` card, or a
   lowering repair. The AI is a generator of realistic mistakes; the finish
   line is the checker, never a model score.
10. Conformance adapter wiring: fill the empty adapter slot in
    `tests/conformance/runner.py` so the corpus executes against the compiler,
    and present the manifest reconciliation list to the owner — the recorded
    understatement is roughly 21 cases. Changing any existing expectation or
    status remains protected work under `WORKFLOW.md` and is never applied
    unilaterally.

**Exit:** every non-waived item green, results recorded per the
`research/experiments/` discipline, and the owner's Phase 11 go decision
appended to this file.

## Phase 11: declared parallelism

Status: not opened. Entry requires the Phase 10 completion set above plus an
explicit owner go decision. This section supersedes and replaces the
non-authoritative investigation note `docs/parallelism.md` (2026-07-25), which
is deleted per its own disposition clause; the underlying survey evidence
remains in `research/experiments/auto-parallelism-feasibility/`.

The pursued claim: the writer marks a parallel construct, and the compiler
proves non-interference from the checked effect rows, resolved places, and
slice origin sets it already computes. No `Send`/`Sync`-class bound at any call
site, no runtime uniqueness check, and the irregular-write cases that push Rust
into `unsafe` or dynamic checks are rejected at compile time. The R0 delta is
primarily W3 and the floor, with P0 upside on suitable kernels: across 14
C++-to-Rust benchmark ports, 29% of parallel memory accesses were irregular and
forced `unsafe` or dynamic checks costing up to 2.8x (Abdi et al., SPAA 2024).
The owner's judgment of record: potentially the largest delta the language can
own, and the current language happens to fit it without having been designed
for it, so this phase shapes the language deliberately rather than by
accident.

Findings that bind this phase, from the recorded survey and its 2026-07-25
corrections:

- The strong claim — the compiler discovers parallelism the writer never
  expressed, profitably — is dead and stays dead. Whitefoot removes the
  soundness half of automatic parallelization more completely than any shipped
  language and removes none of the decision half; the soundness half was never
  the binding constraint. Four serious ownership- or effect-driven systems
  across 26 years (Jade, Æminium, Commutativity Analysis, FX-87) extracted
  parallelism correctly and lost to granularity; every surviving system makes
  the writer request and the compiler verify. SISAL's own authors state a human
  chose the parallel algorithm and the compiler recovered the schedule.
- On the current corpus and machine, four of the six Phase 10 kernels are
  capped near 1.5-2x by memory bandwidth and the other two do not parallelize.
  Every ceiling rests on an inferred single-core bandwidth number; completion
  set item 3 replaces the inference with a measurement, and the go decision
  weighs the result.
- PARSYNT (PLDI 2017) is the one machine-checked precedent for turning
  sequential loops parallel, and it works only over exact algebraic domains —
  precisely FN-4's boundary. It supports the reduction-law direction and
  nothing more general.
- Heartbeat scheduling (PLDI 2018) proves a runtime with no static cost model
  and a machine-checked overhead bound exists, but its unconditional tax fails
  P0 on this corpus; it is a fallback design, not the default route.

The phase closes these questions through the experiment loop, one hypothesis
per loop; research and implementation interleave by design:

1. Construct semantics and the non-interference judgment: which effect-row,
   loan, and origin-set facts license which parallel form, and what is
   rejected with which rule.
2. Intra-object partition — the `split_uniq` gap: symbolic index disjointness,
   sub-range identity, and persistence across calls and recursion. Sequentially
   valuable regardless; a divide-and-conquer port such as merge sort names it.
3. The reduction story: exact-domain integer reductions gated on a discharged
   FN-4 law, with trapping mode excluded because the trap point moves; float
   reductions only as named source forms — a pinned tree, and possibly a
   reproducible accumulator as a distinct operation — never a silent change to
   the meaning of an existing operation.
4. Trap selection among concurrently eligible iterations, inheriting the
   check-motion precedent from completion set item 8.
5. Allocation: `allocates(heap)` is a hidden serializer and an arena bump
   pointer is a genuine loop-carried dependence; a per-task allocation
   position is language design, not runtime detail.
6. The determinism posture and its measured bill — a determinate language
   pays on early-exit search (SISAL Loop 16 ran 60% behind Fortran) — accepted
   deliberately or bounded explicitly, not discovered.
7. Runtime architecture: child stealing rather than continuation stealing,
   worker QoS pinning on asymmetric cores, and the TCB policy question the
   owner decides at entry — a Chase-Lev deque cannot be written in safe Rust,
   so the runtime is either a C linked artifact or a scoped, owner-approved
   exception, decided before implementation rather than during it.

Measurement discipline: absolute wall-clock against the sequential build,
never a scaling curve alone — a work-inflating transform can show excellent
speedup while the program got slower. Attribution before magnitude, as
everywhere in Phase 10. A parity or negative result is recorded as durably as
a win.

Non-goals: no automatic discovery of parallelism; no Tapir fork dependency; no
heartbeat-by-default; no writer-facing toggles, build modes, or thread-count
knobs that change what a source program means.

## Phase 12: optional hardening

Only if later use justifies it, consider stable artifacts, caching, broader
targets, stronger resource controls, transactional publication, distribution,
self-hosting, or a product release. None blocks the research compiler or the
current experiments.

## Verification and durability

Run `make -C compiler check` before and after compiler changes and `make check`
for each completed repository slice. A green gate states only what it tests;
it is not a claim that the language or compiler is complete.

Phase 10 work carries three further obligations, none of them machine-enforced
by the existing gate. An optimized build must preserve facts-off semantics,
every retained check, and the exact DIAG-3 trap record, proved by focused
regressions that trap from an optimized binary with byte-identical records.
Every fact family ships with its facts toggle, a facts-off identity check, and
attribution evidence that the backend consumed the fact. The facts toggle is
compiler-development surface, like `--emit-llvm`, and never a writer-facing
option: optimization level and fact emission are not writer decisions, so
neither becomes a documented flag or a build mode that changes what a source
program means. Every measurement lands as a `research/experiments/` RESULTS.md
with its protocol, machine, and caveats, and a parity or negative result lands
under the same discipline as a win — but a result whose fact was never consumed
is not a parity result, it is a broken instrument, and it is reported as one.

Every reproducible defect receives the smallest practical regression before
its fix. Each cohesive completed step gets one commit. Update current phase
status in this file, durable design choices in `mcts_mem/`, and protected owner
approvals in `governance/APPROVALS.md` when those records materially change.

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
  capability is blocked. Phase 10 is current work by the 2026-07-24
  reorientation, not filler: it is the phase that measures P0, and it is
  blocked by nothing.
- No silent specification reinterpretation, protected-test weakening, or
  baseline regeneration merely to make a gate green.
- No optional optimizer fact changing acceptance or removing an unproved
  required check.
- No active source, build, test, or tool dependency on `archive/`.
