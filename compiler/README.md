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

The frontend targets the exact v0.39 bytes at `../spec/kernel-spec.md`,
SHA-256 `4be4830fa87a534879de17524599b0919aef4dfab072dad823bf2f9b54d32d58`.
v0.39 narrows [CLM-1]'s claim-authority control dependence and supersedes v0.38
at `5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d`, whose
outgoing bytes are archived at `../spec/kernel-spec-v0.38.md`.
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

The system-interface surface is in semantic migration. It continues to parse,
resolve, and check through the normal path: FN-7 admits exactly one uncallable
`command fn main`, and system operations type against the system catalog. The
v0.37 direction replaces lifetime effect subjects and the temporary
resource-only operand path with one formal-rooted static state-path form.
Lifetimes state loan duration only. EFF-2 derives the incoming state actually
read or written by the body and compiler-derived release, then checks the row
in both directions. `external` and `blocks` are ordinary identifiers, not
effects. Target suspension and completion milestones remain compiler-owned
contracts beneath the source call; no root, family, fragment, or relation
metadata may grant permission.

Target-independent lowering then carries those facts into the typed IR. Each
opaque system type becomes an ordinary IR nominal holding its complete
SYS-5/HOST-3 contract: the target-independent semantic identity QUAL-1 owns,
its release action and state row, and whether the value is an inline lease
over command-lifetime argument backing. The HOST-3 lease fact is retained for
auditing and lowering and refuses no program. A system operation call
lowers to its SYS-2 inventory identity, never a source spelling. Every
compiler-derived release is an explicit IR record on the normal edge that
carries it — a `Jump` or `Return` terminator, or a straight-line `Drop` —
holding the released value's own action and the union of the rows it may run
over owned content, in the checked program's reverse declaration order and in
the position EFF-5 requires relative to surrounding calls; a failing `claim`
has no edge that can carry one (TRAP-1). The IR also records the FN-7 command
entry and its selected standard-input ordinals. Standard output and standard
error arrive as distinct ordinary owned values even when host redirection
makes them contact one physical sink. An alias introduced outside the mapped
program does not merge those owners inside the language proof.

A semantically accepted system program then compiles, links, and runs. The
QUAL-1 target-qualification table is fixed Rust data mapping a specification
identity, semantic operation, target, and program kind to an approved
implementation and private ABI symbol. It is consulted once after target
selection and before layout. An absent or incompatible entry, or an unmet
QUAL-2 guarantee, is a target-qualification failure that cites no language
rule.

Operations which survive the rebuild emit as private wrappers with one direct
call per site. Directory-relative open resolves against the ordinary supplied
directory value, never by prefix concatenation (PATH-2). Each transfer takes a
half-open `start, end` range; the caller proves `start <= end` and
`end <= len(buffer)` before target work. Empty ranges complete with
`next = start` without a transfer. A successful nonempty operation returns the
absolute next endpoint and performs no second progress-producing attempt.
`read_at` uses an explicit file offset and native positioned I/O. No-progress
interruption and readiness refusal stay inside target progress. A host
zero-length write maps to `WriteZero`, not to a successful unchanged endpoint.
One cold shared mapper turns native error codes into the portable typed error
set and preserves the inline native detail when the API promises it.

File opening consumes an ordinary proof-only `FilePermit` produced by total
inline `reserve_file(&uniq FileFactory)`. The open wrapper reads a shared
`DirectoryRead` selector and burns the permit on every outcome. Qualification
erases the permit before the native open ABI, so no descriptor, dispatch, or
extra native argument is added; host exhaustion remains the open operation's
typed `ResourceExhausted` result.

Resource release uses the same ordinary state contract as an explicit call.
A resource with a meaningful finish result uses a consuming finish operation;
compiler-derived release performs only the weaker action its type declares.
The old shared advancing directory source and shared mutable Output shapes are
not v0.39 authority. Advancing sources and outputs use `own` or `&uniq`.

The macOS/Linux command bootstrap owns the process before entry: it establishes
the QUAL-2 argument backing from the native vector, installs the selected
write-to-closed-pipe disposition once, opens `command.cwd`, supplies the two
ordinary `Output` owners and the proof-only `command.files` factory, invokes the entry once, and maps the returned
`ExitStatus` onto the process status. It evaluates no entry contract because
main cannot carry one. QUAL-3 verifies the emitted wrappers and direct target
shape on the optimized module. The completion path may add qualified
submission and drain work, but source transfer still performs no hidden data
copy or writer callback.

FN-7 entry validation reads finalized syntax and admits exactly one
`command fn main`: it is nongeneric, source-uncallable, contract-free, returns
one writer-named `ExitStatus`, and may select zero through five standard inputs
in table order. SYS-3 reserves the complete system declaration domain in every
unit, independently of entry validity. System nominals, variants, and
operations enter resolution through one fixed declaration source beside source
declarations and the prelude. A source declaration colliding with a system
entry is the deterministic DIAG-1 rank-5 rejection at the source event, top
level and nested scopes alike, with a
`(System, system_declaration_ordinal)` origin; there is no shadowing in either
direction.

The system catalog records parameter names, modes, loan lifetimes, result
types, exact formal state paths, and completion contracts. Ordinary place and
loan overlap decides permission. The temporary root/family/fragment and
Free/Ordered/Exclusive tables are gone; no capability root, family relation,
authority fragment, or `Ordered` relation authorizes accepted source or
lowering.

The resolver covers every active-specification declaration, lexical-use, and deferred
owner/member role through one grammar-driven path, including exact scopes,
visibility, reservations, collisions, and deterministic diagnostics.

The implemented scalar families support exact fixed-width integers, strict
`f32` and `f64`, `unit`, `Bool`, and unit, integer, and finite floating-point
constants. The function path supports nongeneric functions and the bounded
explicit source-generic subset described below, with locals, direct calls,
returns, and exact source effect-row checking. Wrapping, checked, and saturating
integer rows are total value operations. Bare add/subtract/multiply/divide/
remainder and dotless `ineg`, `iabs`, `ishl`, and `ishr` are exact operations:
each carries one canonical `IntegerDomain` obligation and reaches IR only after
that obligation is discharged. The matching total `.defined` operations
produce the exact Bool goal that a branch, requirement, or retained claim can
establish. Integer comparisons, Boolean operations, bitwise operations,
rotates, bit counts, byte swap, high multiply, min/max, and nominal tag
equality remain total. Every distinct integer-to-integer `cvt` pair uses one exact
conversion path. The 18 value-preserving widening pairs return the destination
directly; the other 38 pairs return `Result<Dst, NarrowError>` after an exact
representability check, never a visible truncation. Checked division and remainder guard
divisor zero and signed minimum/-1 before the partial LLVM instruction and
produce the exact `Result<T, DivError>` variant. Exact division/remainder never
reach a partial LLVM instruction unless nonzero and the signed minimum/-1 pair
are proved impossible. Absolute value uses defined-edge `llvm.abs` for every
signed width: wrapping retains the minimum value, exact excludes it statically,
and checked returns `Err(Overflow())`. Negation uses modular `sub 0, x` for
wrapping, the same static minimum exclusion for exact, and overflow detection
for checked mode, with no unearned `nsw`/`nuw` promises.

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
retains discharged obligations, checked set paths, and cleanup.
Runtime-length buffers use a `{data pointer, u64 length}` value. Every
constructor first discharges its canonical `buffer_fits` goal; target
qualification then verifies actual layout against the language ceiling, so
the emitted byte-size multiplication needs no overflow branch. A separate
selected-target representability guard remains a TCB/resource boundary before
allocation, followed by complete initialization, discharged OP-4 reads and
target-before-RHS writes, cross-function affine transfer, and
compiler-derived `free` on normal owner exits.

Subscripts follow OP-4 discharge-or-reject: the L0 entailment engine
(ENT-1..6) derives per-function difference-bound fact states over the
conservative structural graph, an accepted subscript compiles with no runtime
bounds branch in any build mode, and an undischarged obligation is a
compile-time OP-4 rejection carrying the exact ENT-6 residual. `claim` is the
CLM-1 named runtime check and the sole writer-reachable language runtime trap
carrier. Its predicate is a side-effect-free proof predicate and its exact
five-field `because` record names premises, derivation, conclusion, checker
gap, and terminal consumers. Before its S3 event, the checker rejects a claim
whose direct, support-canonical, or fully structural lifecycle image is proved,
refuted, contradictory, ambiguous, or outside the finite contribution basis.
Every canonical contribution component must be checker-unknown. With one fixed
eligible occurrence set, fresh Full-minus analyses must then show that removing
each component and removing the whole occurrence changes at least one
non-explosive source-admission terminal root while leaving provenance invariant.
A component or occurrence that is independently reconstructed is a hard CLM-2
non-residual rejection, not an advisory. Thus an accepted claim is an
independently true, checker-unknown, load-bearing residual theorem—not an
assertion, test oracle, intentional abort, or substitute for control flow.

Every accepted claim is retained and executes at every dynamic reach in every
build mode. Its passed contribution is an ENT-3 S3 fact source; a violated
approved theorem aborts with one DIAG-3 record citing CLM-1 and the claim name.
Subscripts, exact arithmetic, allocation fit, system ranges, and contracts are
static, while a callee's `traps` effect can only summarize a reachable retained
claim. Buffer fields retain exact
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

The successfully checked program also exposes one private read-only
`ClaimLedger` in stable source/instance order. Each retained claim carries its
source identity and coordinate, exact source spelling and five-field
justification, the direct/support-canonical/fully-structural predicate images,
ordered structured contribution facts, component and parent-reconstruction
proofs, and separately labelled component/whole Full-minus witnesses. Published
uses are only complete bounds/domain obligations, FN-8 call goals, and complete
FN-9 aggregates whose finalized non-explosive proof ancestry reaches an exact
S3 component. Bounds and call uses additionally join the existing protected-
leaf, direct-demand, structural/subject bridge, call-argument, and bridge-call
provenance inventories exactly; a missing or duplicate required mapping is an
internal compiler failure. Claim-free units take an empty-ledger fast path.
The ledger is neither serialized nor read by semantic acceptance, lowering, or
optimization, and it performs no second semantic walk or closure. Installed
real-source populations and the bounded-cost result live in the same acceptance
record. `DerivationMetrics` separately counts claim-lifecycle roots so the
existing retained-size accounting remains complete; no prior metric class or
consumer is repurposed.

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
field-prefix overlap, and ultimate-origin `reads`/`writes` effects. The
superseded v0.36 spelled those effect subjects as lifetimes; active v0.39
preserves the borrow behavior while naming formal state paths instead. Borrowed
buffer descriptors cross ordinary calls by value, but only the original owner
is cleaned up. Distinct struct fields can therefore be uniquely passed to a
fill helper and then shared with a fold helper without transferring either
allocation. The backend remains conservative LLVM without unearned overflow
flags or check elision.

Effect rows are checked as exact source-level summaries for every admitted
function. `pure` is the empty effect row, not a termination claim. v0.39 gives `reads`
and `writes` one operand kind: a formal parameter or static field path naming
logical state. Lifetimes remain in borrow and slice
types only, where they state loan duration and outlives facts. A write requires
an ordinary `own` or `&uniq` route; a read may use `own`, `&uniq`, or `&`.
Reads and writes remain separate exact sets. A transition that observes the
old state and changes it declares the same path in both; `write_once` therefore
uses `reads(output, source), writes(output)`.
Calls resolve the callee path against the actual place, then project that place
onto the current function's formals. A field path refines actual behavior but
does not shrink the borrow that granted permission.

Owned-state provenance preserves ordinary identity through move, aggregate,
enum payload, result, replacement, and compiler-derived release. It is stored
per affine leaf and exists only to attribute a renamed value back to the formal
state it contains. A fresh factory result is a new ordinary owner; later child
operations do not project back to a hidden factory ancestor. This provenance
has no runtime identity and grants no access, overlap, or ordering permission.
The exhibited row additionally unions every compiler-derived release on a
normal edge, and a mismatch a release alone explains is reported at the
function's `effects` node, rendering the owning parameter or binding. The
computed row must equal the declared row, so both missing and superfluous
state paths reject under EFF-2. Extending this provenance across every
remaining owned aggregate and result is ordinary compiler work under the
activated rules; the rules themselves are settled.
The backend emits no effect-derived LLVM function attributes or alias metadata,
licenses no check elision from an effect row, and never emits `willreturn`;
Whitefoot currently has no termination checker.

The completion path is compiler-owned and selective. Completion is the only
source-level I/O model; direct or inline execution at depth one is a lowering
specialization of that model. Compute overlap remains controlled separately by
`--par`, and a pure compute module names and links no completion symbol.

The retained completion core allocates stable bounded operation storage before
target handoff, transfers the complete resource and payload owner/loan bundle,
maps the target result through the same qualified outcome mapper as a direct
path, and returns each loan only after the target's last permitted access. The
rebuild is deleting fixed free/ordered group sizes, shared Output scheduling,
and family relation tables. Independent calls use disjoint ordinary places;
two writes to one Output use one `&uniq` place and therefore cannot be in
flight together. Dependency-driven activation must make a later use ready as
soon as its own loan returns, without waiting for an unrelated operation.

The first selective stackless slice covers a single-block root with one
suspension point whose zero-state tail-wrapper chain ends in `read_at` or
`write_once`. Completion publishes an opaque frame into a bounded scheduler
queue, and only a normal scheduler lane invokes its resume entry. Branching,
loops, multiple suspension points, non-tail suspended children,
and may-suspend release edges retain the correct synchronous ABI.

The common core keeps captured generation, separate result-ready,
per-formal `loan-released(path)`, and terminal milestones, exactly-one terminal publication,
bounded ready drain, and one compute/completion/capacity wake epoch. Completion
before a scheduler announces sleep causes no host wake. Target helpers accept
only a closed file request; they receive no writer function pointer. macOS
regular files use the bounded typed fallback. Existing directory work must be
requalified under an ordinary uniquely borrowed Source before it can re-enter
the catalog. Linux `read_at` prefers real io_uring and falls back only before
target ownership; its scheduler parks on an epoll set containing the ring fd
and a broadcast eventfd, with no millisecond polling.
The Windows core plus IOCP adapter strict-cross-links and remains fail-closed
pending an actual Windows execution. No operation path reads a trap latch or
carries trap-specific state.

Native rings, IOCP ports, and helper mailboxes are target-private protocol
state. The target side may coordinate them through qualified atomics or typed
channels after the writer transfers the operation bundle. They are never
exposed as ordinary shared Whitefoot storage, and target publication never
executes writer code.

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

An admitted `contract` block contains erased declaration-before-use `define`
abbreviations followed by independent `requires` and `ensures` clauses. The
checker recursively alpha-expands each definition; no definition is evaluated,
snapshotted, stored, or lowered. Each ordinary call substitutes actual value
images into every requirement and proves every resulting `GoalTemplate` in the
same pre-transfer state. Only total success permits transfer. At body entry the
goals become independent S4 sources; no callee executes a prologue, fallback
trap, or `llvm.assume`.

If those entry facts are contradictory, the concrete function instance is
legal but uninhabited. The checker still audits its complete source; checked
metadata retains the contradiction derivation, and lowering emits one
ABI-shaped entry block terminated by `unreachable` without traversing the
body. No postcondition summary is published. The command process wrapper has
no related boundary logic because main cannot carry a contract.

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
offset is unconditionally external and whose relation needs a body `claim` or
S4 requirement fact; PRV-2 owns the corresponding downstream call
argument. A real branch/value outcome remains visible in the unasserted and
S4-blinded states and is accepted. Main has no contract, so no entry wrapper
can launder an external protected leaf.

This gate is deliberately narrower than taint or noninterference. Control
choice, write-address choice, path-sensitive storage, recursive payload paths,
and implicit flow add no provenance edge. An external value used only as a
bound, base, write address, or unrelated operand is outside the gate, and an
internal constrained subject may still rely on an ordinary residual claim,
but only when that predicate is independently true and passes its complete
CLM-2 lifecycle and necessity judgment. The sole
current protected subject is the offset in `i < len(P)`. Provenance changes no
runtime operation, effect row by itself, optimizer fact, or check-elision
license, and facts-on/facts-off acceptance and required runtime behavior use
the same semantic path.

Each `ensures` clause declares one verified normal-return relation without an
executable epilogue. The mandatory header result binder denotes the whole
result symbolically; a routed `when Ok(value: payload):` clause instead names
that clause's selected payload. Every relation is proved independently at
every selected normal exit in complete, claim-blinded, and S4-blinded
views. Concrete call components are scheduled callee before caller;
same-component summaries remain unavailable, and a component publishes all of
its verified summaries atomically without iteration or a summary fixed point.

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
every member. A binding names an ordinary nongeneric, `contract`-free top-level
function; compatibility reuses the complete callable signature, normalizes
read and write paths by parameter and field ordinals, and alpha-renames regions
only in modes, types, and arena allocations. It then compares allocation and
trap facts exactly. Law-bearing conformances pass the closed FN-4
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

`--no-overlap` emits the module a compiler with no overlap lowering at all
emits: every I/O call reaches the host through an ordinary direct call and the
completion runtime does not join the link. It exists so one source can be
compiled two ways and the pair measured, which is what
`research/experiments/io-completion-bench/` does. It is not a build a program
ships in, and it may not be written together with `--par`. It also prints no
denied-I/O-loop note, because the flag has already said this build takes no
overlap; `--par-ledger` still prints the whole permission report under it.

There is deliberately no artifact protocol, replay layer, resource-profile
product, or compatibility boundary in front of this path.

Run the compiler gate with:

```sh
make check
```
