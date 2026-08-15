# Whitefoot compiler

This directory is one safe-Rust crate containing the active compiler. It is an
implementation, not a collection of stable libraries: module boundaries are
private design choices and should change when the next compiler capability
needs them.

The implemented path is currently:

```text
ordered source bundle
  -> lossless lexer
  -> terminal classification
  -> strong-LL(2) parser
  -> finalized source-bound syntax tree
  -> exact FORM-2 validation
  -> direct active-specification lexical name resolution
  -> semantic and ownership checking
  -> private checked program
  -> target-independent typed control-flow IR
  -> selected-host layout and target-domain qualification
  -> conservative textual LLVM
  -> host executable
```

The frontend targets the exact v0.28 bytes of `../spec/kernel-spec.md`, SHA-256
`08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`.
`cargo run --bin whitefoot-spec` checks the embedded bytes against the recorded
activation chain and checks that the terminal and grammar data name the same
specification identity. The committed grammar tables are ordinary compiler
data. The exact specification identity is versioned data; compiler stage, type,
and API names remain stable across grammar-preserving specification bumps
instead of acquiring a `V0_xx` suffix. For a specification proposal, run the
native verifier through this compiler:

```sh
cargo run --bin whitefoot-grammar -- \
  ../spec/kernel-spec-vPREVIOUS.md \
  ../spec/kernel-spec.md
```

It verifies that a grammar-preserving proposal keeps the baseline
specification's complete canonical-format, lexer, and grammar contract
byte-for-byte, checks the committed terminal inventory and every strong-LL(2)
decision, and runs the real lexer and parser over the active tables. Both
specifications are arguments read at run time: comparing against a compiled-in
copy of the active bytes said nothing once the candidate became that copy. A
proposal that changes that frontend contract fails closed: a structural change
must first extend this same native path rather than reviving an independent
grammar engine.

The v0.19 system-interface surface parses, resolves, and checks through the
normal semantic path: FN-7 admits both entry forms, system operation calls
type against the SYS-2 catalog signatures, and EFF-2 checks the
`external`/`blocks` categories exactly — the exhibited row is the union of
the syntactic contribution and the release contribution, the SYS-5 release
rows of every compiler-derived release recorded on a normal control-flow
edge, with `buffer`/`box`/arena/`const` reclamation contributing nothing
(STOR-3).

Target-independent lowering then carries those facts into the typed IR. Each
of the seven opaque types becomes one IR nominal holding its complete
SYS-5/HOST-3 contract: the target-independent semantic identity QUAL-1 owns,
the one release action (logical consume, native close attempt, or `Output`'s
source detach), that action's row, and whether the value is an inline lease
over command-lifetime argument backing — the HOST-3 lease fact is retained
for auditing and lowering and refuses no program. A system operation call
lowers to its SYS-2 inventory identity, never a source spelling. Every
compiler-derived release is an explicit IR record on the normal edge that
carries it — a `Jump` or `Return` terminator, or a straight-line `Drop` —
holding the released value's own action and the union of the rows it may run
over owned content, in the checked program's reverse declaration order and in
the position EFF-5 requires relative to surrounding calls; a trapping `check`
has no edge that can carry one (TRAP-1). The IR also records the FN-7 entry
form, its selected standard-input ordinals, and the SYS-12 stdout/stderr
may-alias link, which nothing yet reads.

A semantically accepted system program then compiles, links, and runs. The
QUAL-1 target-qualification table — fixed Rust data mapping `(specification
version, semantic ID, target, program kind)` to one approved implementation
version and one private ABI symbol, plus per-type representation and release
rows — is consulted once after target selection and before layout; an absent
or incompatible row, or an unmet QUAL-2 guarantee, is a target-qualification
failure that cites no language rule. All eleven SYS-2 operations emit as
`alwaysinline` private wrappers with one direct call per site: the argument
and host-string cluster, `relative_path`, `exit_status`, and the I/O cluster.
`open_read` resolves against the capability's own descriptor through the
target's directory-relative facility, never a prefix concatenated onto a path
(PATH-2). `read_once` and `write_once` are SYS-8 one-attempt transfers whose
range validation traps before any host action, report a count of zero for an
empty range without issuing a transfer, return the host's reported progress
without a second attempt, and map a host zero-length write to `WriteZero`
rather than `Ok(0)`. One cold shared mapper turns a native error code into
exactly one of SYS-7's thirty portable classes, carrying the two-field inline
detail (`code`, `origin`); a native error with no portable distinction in that
set is `Other`. Releases emit per SYS-5: a logical consume and `Output`'s
source detach emit no code, while `DirectoryRead` and `ReadFile` emit one
direct close whose diagnostic is discarded and never retried. The macOS/Linux
command bootstrap owns the process before entry: it establishes the QUAL-2
argument backing from the native vector (refusing startup otherwise), installs
the ignored write-to-closed-pipe disposition once, opens `command.cwd`,
supplies the two `Output` owners, invokes the entry once, and maps the
returned `ExitStatus` onto the process status exactly. QUAL-3's emitted shape
is verified on the optimized module: the wrappers inline, one source transfer
is one direct host call, and the transfer path carries no allocation, data
copy, dispatch, lock, or per-call signal operation.

FN-7's kind-declaring judgment has one home, `syntax::entry_form`, which reads
it from finalized syntax alone: a unit is kind-declaring exactly when a
`program_kind` node exists, independent of names, types, and effect rows. The
resolver takes the SYS-3 system-admission decision from it in DIAG-1's stage
order, after complete unit-wide FN-8 requires-block admission and before
declaration inventory: a kind-declaring unit admits the complete SYS-2
inventory — fourteen nominal types, thirty-nine enum-variant constructors, and
eleven operation signatures, one hundred sixty-seven records in normative
preorder — as a third declaration source beside source declarations and the
prelude, while every other unit sees none of it and a system spelling there is
an ordinary name. A source declaration colliding with an admitted system entry
in its domain is the deterministic DIAG-1 rank-5 rejection at the source event,
root and nested scopes alike, with a `(System, system_declaration_ordinal)`
origin; there is no shadowing in either direction. The registered signature
data (parameter names, modes, region parameters, result types, and the fixed
`external`/`blocks`/`traps` classifications, with `reads`/`writes` derived
mechanically from parameter modes) lives in the resolution catalog for the
system semantic-admission and effect-attribution tasks to consume.

The resolver covers every active-specification declaration, lexical-use, and deferred
owner/member role through one grammar-driven path, including exact scopes,
visibility, reservations, collisions, and deterministic diagnostics.

The implemented scalar families support exact fixed-width integers, strict
`f32` and `f64`, `unit`, `Bool`, and unit, integer, and finite floating-point
constants. The function path supports nongeneric functions and the bounded
explicit source-generic subset described below, with locals, direct calls,
returns, and exact source effect-row checking. The integer operation family
includes wrapping and trapping add/subtract/multiply, checked
add/subtract/multiply/divide/remainder, integer
absolute value and negation in all three modes, integer comparisons, Boolean
operations, the remaining OP-8 integer family, and nominal tag equality.
That integer family includes trapping division/remainder, bitwise operations,
shifts, rotates, bit counts, byte swap, high multiply, saturating arithmetic,
and min/max. Every distinct integer-to-integer `cvt` pair uses one exact
conversion path. The 18 value-preserving widening pairs return the destination
directly; the other 38 pairs return `Result<Dst, NarrowError>` after an exact
representability check, never a visible truncation. Checked division and remainder guard
divisor zero and signed minimum/-1 before the partial LLVM instruction and
produce the exact `Result<T, DivError>` variant. Absolute value uses
defined-edge `llvm.abs` for every signed width: wrapping retains the minimum
value, trapping emits OP-2, and checked returns `Err(Overflow())`. Negation uses
modular `sub 0, x` for wrapping and signed-subtraction overflow detection for
trapping and checked modes, with no `nsw`/`nuw` promises.

Scalar `f32` and `f64` run end to end through semantic checking, typed IR,
LLVM emission, and host execution. Source literals use the unique canonical
finite spelling and retain their exact IEEE bits. All 24 direct floating-point
operations execute for both widths without emitted fast-math flags, including
the specified NaN and signed-zero cases. With the integer rows above, `cvt`
covers all 90 ordered pairs of distinct concrete numeric primitives: 29 are
total and return the destination directly; 61 return
`Result<Dst, NarrowError>` under OP-6's exact-conversion rules. The 16 specified
equal-width numeric `reinterpret` pairs preserve source bits. Floats compose
with calls, loop-carried locals, structs, constants, fixed arrays, primitive
runtime buffers, checked indexing, and SET-1.

The counted-range path implements one ascending half-open source form,
`for @label i in lower..upper { ... }`, alongside the unchanged ordinary loop.
It captures both `own u64` endpoints once from left to right, gives the
read-only body binder a stable checked identity, carries cleanup on every
labelled or function exit, and lowers a dedicated preheader/header/body/update
graph whose representable hidden increment adds no trap. ENT-3 S11 contributes
only the structural body-entry relation
`lower_capture <= i < upper_capture`; ordinary loops gain no induction and no
counted fact escapes the exhaustion/break continuation.

Acyclic source structs and enums, including reachable concrete generic nominal
instances, flow through the same path with construction, nested projection,
statement/value matching, `give`,
per-site exhaustiveness checking, whole-binding affine moves, and explicit
reverse-order cleanup edges. Struct fields may own buffers; whole and partial
owner cleanup expands to exact projected buffer frees, and consuming field
projections skip only the transferred subtree. Resource-bearing source enums
and concrete `Option`/`Result` instances retain one checked owner drop; the
backend switches on the active tag and recursively cleans only that variant's
resource fields. A consuming match transfers the payload without also dropping
the enum root. SET-1 supports live own-mode copy locals and nested copy
fields, rejects affine replacement under STOR-1, and rechecks target liveness
after the right-hand side. Semantic success produces the only lowering
authority. Concrete fixed arrays support
decimal or earlier-integer lengths, complete `array_new` initialization,
immutable static const tables, `len`, discharged index reads, and
target-before-RHS discharged indexed writes for direct local roots. The IR
retains required checks, source trap sites, checked set paths, and cleanup.
Runtime-length primitive buffers use a `{data pointer, u64 length}` value,
checked OP-9 byte-size multiplication, a separate selected-target domain guard
before allocation, complete fill initialization, discharged OP-4 reads and
target-before-RHS writes, cross-function affine transfer, and
compiler-derived `free` on normal owner exits.

Subscripts follow OP-4 discharge-or-reject: the L0 entailment engine
(ENT-1..6) derives per-function difference-bound fact states over the
conservative structural graph, an accepted subscript compiles with no runtime
bounds branch in any build mode, and an undischarged obligation is a
compile-time OP-4 rejection carrying the exact ENT-6 residual. `claim` is the
CLM-1 named runtime check: always retained, judged by the engine for CLM-2
redundancy (a required non-rejecting advisory, printed to stderr by the
driver) and refutation (a hard CLM-2 rejection), and its passed predicate is
an ENT-3 S3 fact source. A failing claim aborts with a DIAG-3 record citing
CLM-1 and the claim name. Subscripts are not an EFF-2 trap source; `check`,
`claim`, `.trap` operations, and trapping callees are. Buffer fields retain exact
projected roots through length, read, and write operations without
re-evaluating source paths.

Each successful function summary also retains DIAG-2 explanation data in one
function-local derivation DAG. Mandatory roots cover every accepted bounds
obligation, every discharged call goal, and the eight directed S11 atoms for
each counted range. Finalization keeps only their transitive ancestry and
remaps the retained nodes, events, and roots against the same function-local
term and goal inventory. This is observational compiler state: it adds no
second semantic walk or closure, serialized proof artifact, optimizer fact,
lowering input, or independent authority, and it does not change dispositions
or accepted programs. The frozen-program completeness and bounded-cost
measurements live in
`research/investigations/obligation-discharge/ACCEPTANCE.md`.

The source-generic path is a finite monomorphizing subset, not complete generic
support. Functions, structs, and enums support unbounded type parameters, the
built-in `Int` and `Float` bounds, and integer-typed const parameters; every
type and const argument is explicit. Templates are checked symbolically, then
each reachable kind-correct concrete instance is rechecked through the
ordinary semantic path and receives a concrete nominal identity or
collision-free internal function symbol before normal lowering. Acyclic nested
calls and nominal discovery, forwarded const parameters, generic arrays and
primitive buffers, and symbolic and concrete `Option` and `Result` instances
use that same path. There is no argument inference, backend-level generic IR,
or cross-instance body sharing. Generic call cycles, generic functions with
region parameters, and type-dependent generic `cvt` or
`reinterpret` are explicit unsupported capabilities. Generic source contracts,
source-contract bounds, and region-bearing generic arguments instead receive
their v0.18-specified source rejections.

The first lexical borrow family adds caller region parameters, local region
blocks, shared and unique buffer holders, explicit `deref`, resolved
field-prefix overlap, and ultimate-origin `reads`/`writes` effects. Borrowed
buffer descriptors cross ordinary calls by value, but only the original owner
is cleaned up. Distinct struct fields can therefore be uniquely passed to a
fill helper and then shared with a fold helper without transferring either
allocation. The backend remains conservative LLVM without unearned overflow
flags or check elision.

Effect rows are checked as exact source-level summaries for every admitted
function. `pure` is the empty effect row, not a termination claim. The
implemented executable paths otherwise track `reads('r)`, `writes('r)`,
`allocates(heap)`, `external`, `blocks`, and `traps`, union local expression
effects, propagate callee heap, trap, and payload-free-category effects by
presence, and substitute formal read and write regions onto actual
borrowed-storage and slice origins. The exhibited row additionally unions the
release contribution — the fixed SYS-5 rows of every compiler-derived release
on a normal edge — and a mismatch a release alone explains is reported at the
function's `effects` node, rendering the owning parameter or binding. The
computed row must equal the declared row, so both missing and superfluous
capabilities reject under EFF-2. These facts currently stop at semantic
checking and static-contract compatibility.
The backend emits no effect-derived LLVM function attributes or alias metadata,
licenses no check elision from an effect row, and never emits `willreturn`;
Whitefoot currently has no termination checker.

Target qualification is one private stage immediately before LLVM emission.
The compiler executable fixes an exact aarch64 or x86-64 macOS/Linux triple and
DataLayout, checks concrete representations, statics, source-call ABI objects,
actual emitted stack slots, and complete frames with checked arithmetic, and
reports unrepresentable materialization as a target failure without a source
rule. The checked program and IR retain allocation and element-address
obligations. Fixed-array bounds plus static layout discharge address
representability; buffer bounds plus the successful allocation invariant do
the same; buffer allocation retains an exact non-language guard before
`malloc`. This is not a language array limit, stack-capacity prediction, hidden
heap fallback, or optimizer fact.

An admitted `requires` block is one finite typed atomic goal. The semantic
checker alpha-expands its restricted own-copy, pure-total ANF clause into a
`GoalTemplate`; each ordinary call substitutes concrete pre-transfer actual
values and must prove that exact goal before argument transfer and callee
effects. No ordinary callee executes a requirement prologue or fallback trap.
The proved goal is the body-entry S4 axiom, while the declaration itself adds
no effect to the callee row and never becomes `llvm.assume`.

The two real process wrappers remain checked boundaries: they evaluate an
entry function's pure goal once after setup and before the body, trapping with
the original OP-5 record on false. A borrowed-output capacity program exercises
caller discharge, the body axiom, ordinary loop and buffer operations, exact
effects, and cleanup through this single implementation path.

The v0.27 implementation activates the retained
requirement-to-protected-leaf bridge as one two-stratum provenance judgment.
The first finite fixed point derives explicit-dataflow component pairs for
values, whole storage roots, direct enum payload projections, user-call
results, and writes. Command inputs and only the environment-origin cells in
the closed system result/write table are unconditional external seeds. A place
read joins its root with every explicit subscript offset; `len` remains
internal. After those pairs freeze, a second finite fixed point composes direct
protected demands, S4
bridges, call targets, and one rejection event per call argument. Checked
metadata retains complete target sets and post-convergence deterministic
NodePath witnesses; no witness choice participates in either lattice.

Existing acceptance keeps precedence. A complete-state local failure remains
OP-4, and an unproved or refuted ordinary-call goal remains FN-8. After base
success, PRV-3 rejects only a local protected subscript whose constrained
offset is unconditionally external and whose relation needs a body `check`,
`claim`, or S4 requirement fact; PRV-2 owns the corresponding downstream call
argument. A real branch/value outcome remains visible in the unasserted and
S4-blinded states and is accepted. The command entry is rewalked by the same
rule, so its checked wrapper cannot launder an external protected leaf.

This gate is deliberately narrower than taint or noninterference. Control
choice, write-address choice, path-sensitive storage, recursive payload paths,
and implicit flow add no provenance edge. An external value used only as a
bound, base, write address, or unrelated operand is outside the gate, and an
internal constrained subject may still rely on an ordinary claim. The sole
current protected subject is the offset in `i < len(P)`. Provenance changes no
runtime operation, effect row by itself, optimizer fact, or check-elision
license, and facts-on/facts-off acceptance and required runtime behavior use
the same semantic path.

The v0.28 implementation adds one verified normal-return `ensures` relation
without adding an executable callee check. Each concrete function instance
proves every selected normal exit in complete, assertion-blinded, and
S4-blinded views. Concrete call components are scheduled callee before caller;
same-component summaries remain unavailable, and a component publishes its
verified summary atomically without iteration or a summary fixed point.

An earlier-component summary can establish only the closed result routes in
the active specification: a fresh direct ordinary-let result, a direct
selected `Ok` payload, the narrow same-binding direct result receiver, and the
first-statement selected-payload receiver. `value_if` can additionally deliver
an eligible bare-atom L0 relation across every reaching `give` edge and the
ordinary weakest-bound join; the checked `ValueInitializerKind` keeps the
byte-similar `value_match` path at zero delivered relations. Transfer, effect,
target, holder, scope, loop, and support kills apply before publication.

Complete/U/B proof nodes, the two measured unsigned bit sources (`iand` bounds
and direct-one `ishl.wrap` nonzero), selected exits, summaries, calls,
receivers, and delivery joins share one function-local DIAG-2 ledger and event
stream. Optimistic S12 and delivery facts are finalized with the checked
program only after the existing provenance batch has no rejection event; a
failure discards the whole candidate. This adds no runtime fallback, optimizer
assumption, second semantic pass, foreign derivation identity, or lowering
authority.

The ordinary real-program path exercises fourteen `read_bits` calls and twenty
`append_slice` calls through these rules. The two append declarations retain
their invalid-domain no-write behavior, and wfgrep's sole post-copy repair uses
one `value_if` result for every subsequent length use. Existing output, error,
cleanup, effect, and required-check oracles remain unchanged.

The compiler retains the static contract family introduced in v0.16 and checks
it before checked-program publication.
A nongeneric source contract contributes its source-ordered unique member
signatures and laws. Each source conformance has one exact concrete subject,
one coherent source-contract key, and exactly one declared-order binding for
every member. A binding names an ordinary nongeneric, `requires`-free top-level
function; compatibility reuses the complete callable signature and compares
normalized read, write, allocation, and trap capabilities after positional
region alpha-renaming. Law-bearing conformances then pass the closed FN-4
discharge before semantic success. The checked program retains the contract
table, complete binding vectors, and base law derivations as semantic evidence.

That evidence is deliberately non-executable. Lowering reads the same ordinary
checked functions and operations as before, ignores the contract metadata, and
creates no contract object, dictionary, vtable, indirect call, runtime check,
ABI component, or optimizer fact. A bound function is emitted only through its
normal direct function path. The language has no contract-member call
operation, and
generic source contracts and source-contract generic bounds receive their
specified FN-3 rejections rather than becoming unsupported compiler features.

Concrete PRE-1 `Option<T>` instances reuse the same checked nominal, typed IR,
and LLVM representation as source enums and `Result<T, E>` for every currently
supported payload. `None` and `Some` cross ordinary function, return, and match
boundaries; nested Options are concrete nominal instances, not erased values.
A shared-borrow byte scanner returns `Option<u64>` through this path. A
fallible byte transform exercises owned-buffer Result success, error, matching,
and abandonment cleanup through the same representation.

The implemented borrow family covers buffer owners, whole acyclic struct
owners, copy-field projection, caller-visible read/write effects, and
statement-scoped shared or mode-compatible unique child reborrows around one
call. Boxes, direct own-rooted projected arrays, and direct read-only slices
over arrays, primitive buffers, and immutable const arrays use the same checked
place, cleanup, and backend paths. The compiler deliberately stops before
returned borrows, borrow-producing branch joins, arenas, slices formed through
borrow holders, non-flat slice elements, and non-buffer borrow-backed SET-1
targets. Direct own returned slices use v0.18's finite-origin rules through the
normal semantic, lowering, and unchanged slice-descriptor path. A source
claim belongs to its named data region rather than to one descriptor binding,
so nested scopes preserve it, control joins take its conservative union, and
only leaving that named region releases it.
Non-flat direct slice types likewise retain their language status while the
compiler lacks their value path. Region-bearing function and nominal generic
arguments, region-bearing box or arena content, borrow-mode direct-slice
results, and slice-valued value matches instead receive the specified FN-2,
STOR-5, FN-1, and OWN-5 source rejections. Unimplemented active-specification
families stop as unsupported rather than becoming source-language rejections.
Whole-unit ERR-2 variant-addition edit-list enumeration remains future work.

The ordinary compiler path also executes repository-owned program witnesses for
numeric and generic composition, text and binary transforms, hashing, heap and
borrowed storage, image and signal kernels, network formats, and stored, fixed,
and dynamic raw-DEFLATE blocks under `tests/programs/`. These are regression
evidence for the implemented surface, not external-project validation or
performance claims.

Compile a source file through the normal path with:

```sh
cargo run --bin whitefootc -- source.wf -o program
cargo run --bin whitefootc -- --emit-llvm source.wf
```

There is deliberately no artifact protocol, replay layer, resource-profile
product, or compatibility boundary in front of this path.

Run the compiler gate with:

```sh
make check
```
