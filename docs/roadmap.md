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

The active language authority is `spec/kernel-spec-v0.14.md`, SHA-256
`31c09313363304f405c8db1191d1982e3625b86788bf953ec3bb169648466e9f`.
Those bytes are immutable and byte-identical to the owner-approved candidate.
Exact v0.8 through v0.13 remain immutable historical evidence. v0.12 added the
SET-1 copy-place assignment judgment, target-before-RHS ordering, post-RHS
writability revalidation, and ultimate-storage-origin read/write effects.
v0.13 makes a direct bare affine own-rooted `Result` place a consuming
`propagate` operand, matching the already-approved writer form while retaining
explicit `move` as a valid spelling and leaving every other ownership rule
unchanged. v0.14 closes the already-listed integer-negation rows: wrapping
minimum remains minimum, trapping minimum emits OP-2's exact mandatory record,
and checked minimum returns `Err(Overflow())`.

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

This is not a completeness claim. Generics and contracts, borrow referents
outside buffers and acyclic structs, returned borrows, bound/result-carrying/
grandchild reborrows, floats, boxes, arenas, slices, recursive nominal layouts,
branch-dependent ownership/loan joins, projected array targets, and
floating-point and remaining effect-table operations are explicit unsupported
compiler capabilities rather than source-language rejections.
Repeated exhaustive match arms also stop as
unsupported because v0.14 defines neither duplicate-arm meaning nor a
duplicate-arm rejection rule.

The exact approved v0.14 candidate is installed and every live identity names
it. The resolver implementation completes Phase 6, the first executable scalar
slice completes Phase 7, and nominal data, the current SET-1 place family,
structured loops, and the first Result family advance Phase 8.

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

Direct own-root runtime-length non-floating primitive buffers now run through
the normal compiler path. `buffer_new` computes `n * sizeof(T)` with retained u64 overflow
before allocation, aborts on allocator failure as a TCB edge, fills every
element, and produces the specified `{data pointer, u64 length}` owner.
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
result-carrying child reborrows, or add slices, boxes, or arenas.

Five inherited runnable conformance entries need protected-evidence correction
before the compiler adapter can promote their current families. `pending-op9-buffer-new`
and `op4-trap-index-oob` allocate but omit `allocates(heap)` from their function
rows. `type2-pos-buffer-tagonly` and `own1-pos-match-projected-copy` construct
`buffer<Bool>` even though TYPE-2 admits that type while the OP-1
`buffer_new` row remains primitive-only. In addition,
`fn8-neg-requires-non-bool-check` expects TYPE-5, while OP-5 explicitly owns
every non-borrow exact-mode/type failure of a checked condition and FN-8 says
its final statement has exactly OP-5 semantics. The compiler follows the
active specification and none of those protected sources, verdicts, or
statuses has been changed; exact owner approval under `WORKFLOW.md` is required
first.
The specification's array frame-limit value is still not defined; the compiler
does not invent one, and full all-N completeness remains blocked on that owner
rule rather than on ordinary representable arrays.

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

Status: in progress.

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
indexed fixed-array SET-1, and direct or struct-projected non-floating
primitive runtime-length buffers are implemented. Resource-bearing struct
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

The next decoder milestone is fixed-Huffman decoding through one borrowed
state object, a retained bit accumulator, canonical literal/length and distance
tables, ordinary malformed/truncated/output-shortage results, and overlapping
history copies. Implement only the first general compiler capability that this
complete-decoder path actually exposes. The selection does not authorize CLI
or streaming-wrapper infrastructure, benchmark reconstruction, the archived
target-specific optimizer prototypes, or proof-based check removal.

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
  capability is blocked.
- No silent specification reinterpretation, protected-test weakening, or
  baseline regeneration merely to make a gate green.
- No optional optimizer fact changing acceptance or removing an unproved
  required check.
- No active source, build, test, or tool dependency on `archive/`.
