# Whitefoot compiler

This directory contains the safe-Rust Whitefoot compiler. It is one evolving
compiler crate, not a collection of stable libraries. Module boundaries are
private implementation choices; the active language is defined by
[`spec/kernel-spec.md`](../spec/kernel-spec.md), not by the compiler source or
this README.

The frontend targets the exact bytes at `../spec/kernel-spec.md`. Their
version and SHA-256 are derived from those bytes by `build.rs` on every build
that touches them, and every other identity constant in the crate reads that
generated module. Nothing is committed, so nothing can go stale: amending the
specification changes the identity in the same build.
`whitefoot-spec` checks the selected identity, activation chain,
rule inventory, and generated syntax identity as one compiler gate.

## Compilation path

The normal path is:

```text
ordered .wf source bundle
  -> lossless lexing and fixed terminal classification
  -> generated strong-LL(2) parser
  -> source-bound syntax finalization and exact FORM-2 validation
  -> lexical resolution
  -> typing, ownership, effects, and source proof checking
  -> private checked program
  -> proof erasure
  -> target-independent typed control-flow IR
  -> selected-target layout, address, and system qualification
  -> conservative textual LLVM
  -> host executable
```

There is one semantic compilation path. Valid specified source that this
compiler has not implemented stops as an explicit unsupported capability; it
is not reported as invalid Whitefoot. A disagreement between compiler stages is
a compiler defect to fix in code and tests, not another source obligation.

## Source proof checking

Whitefoot source is the only writer-controlled proof input. The compiler checks
three kinds of evidence in the ordinary semantic walk:

- `requires` states what every caller must prove before argument transfer. A
  successfully checked call makes the instantiated facts available at callee
  entry without an executable prologue.
- `ensures` is proved at every selected normal return. Verified summaries are
  published atomically by call-graph component and then instantiated at later
  callers; a recursive component cannot bootstrap itself from an unpublished
  summary.
- A loop-header `invariant` is an induction contract. In a counted loop the
  binding is the first header item and every later item is an invariant; in an
  ordinary loop every header item is an invariant. The compiler checks all
  base obligations before activating the header batch, then checks every
  arbitrary reachable backedge against the simultaneous next-header batch.
  Normal exact exhaustion of a counted loop may export the separately
  justified binder-free consequence; `break` does not receive it. Header
  invariant names exist only in the body and header entries cannot have `use`
  blocks.
- A local `invariant` states a relation at one program point. With no block it
  is submitted to AUTO. With `{ use ... }`, each written premise is proved from
  the same entering snapshot and the checker follows the explicit weighted
  combination. A `use` premise never publishes a new fact; only the checked
  outer invariant is published for its remaining dominance region.

The canonical counted shape has no trailing comma:

```wf
for (
  i in 0_u64..count,
  invariant per_byte: sum <= 255_u32 * i
) {
  let w = deref(weights)[i];
  let wide = cvt::<u8, u32>(w);
  set sum = sum + wide;
}
```

AUTO subtracts the one published affine premise `per_byte`; DIRECT then proves
the residual from the `u8` type interval of `wide`. An explicit use block at
this point would be redundant and is therefore invalid.

`loop { ... }` remains the zero-invariant ordinary form. An ordinary loop with
induction contracts uses `loop (` followed only by invariant items, `)` and the
body. Labels occur after `for` or `loop` and before `(`.

At each program point the semantic checker has one current `ProofContext`.
Selected control-flow edges, type and declaration facts, checked requirements,
verified postconditions, and proved invariants update that context. Numeric and
logical consumers submit one normalized goal to the shared proof entry. AUTO's
complete affine boundary is exact and source-visible: the zero-premise direct route, every
available coefficient-one single premise, every unordered coefficient-one
premise pair including the same premise twice, and the final fixed L0-image
route. Every family is exhausted in specification order for an unproved goal.
Combinations that need three or more published affine premises outside the
final fixed L0-image route, special elimination routes, and future named
nonlinear rules require explicit `use` steps rather than compiler guesswork.
Ownership, initialization, effects, layout, target, and parallel permission
remain separate deterministic domains tied to the same checked source flow;
this is not a universal solver.

Within a `use` block, a bare decimal factor from two upward scales one premise;
factor one must be omitted, and the same normalized premise cannot be repeated.
The final target may be a direct weakening of the checked weighted sum. A
nonempty block is a source error if AUTO proves the target without it. This
redundancy rule is tied to the exact specification version, so an author can
decide from the language rules whether the block is required instead of
probing compiler behavior.

Acceptance uses no SMT solver, random seed, heuristic proof search, timeout, or
cumulative proof-work budget. Rule families, traversal, normalization, and
structural source ceilings are fixed by the specification, and every admitted
family runs to completion. A successful query may stop at its first witness in
the fixed order because later candidates cannot revoke success; an unproved
query is reported only after the required family is exhausted. Internal
derivations explain the originating decision but grant no independent
authority. An inconsistency among compiler data structures is a compiler bug,
not a reason to export or replay compiler-generated proof objects.
The current compiler therefore emits no `.wfproof`, external certificate,
proof-cache entry, or self-verification payload. Incremental and cross-module
proof reuse remain future build-system questions, outside this source-proof
implementation.

Contracts and invariants have no runtime behavior.
Lowering drops their syntax and diagnostic derivations. Later consumers see
only semantic decisions already justified by the checker: an admitted
operation, a verified callable summary, a target obligation, or a parallel
permission.

## Partial operations and safety

Every supported partial operation is admitted only after its exact domain goal
has been proved. This includes the implemented exact integer arithmetic,
division and remainder, shifts, subscripts, buffer-allocation fit, counted-loop
hidden updates, callable requirements, selected return postconditions, and
system buffer ranges. Failure to prove the goal is a compile-time rejection;
the compiler does not insert a hidden runtime check or fallback.

The same semantic path checks affine ownership, moves, borrows, resolved-place
overlap, initialization, exact effect rows, cleanup, fixed arrays, runtime
buffers, structs, enums, concrete generic instances, and the supported system
interfaces. Checked, wrapping, and saturating integer operations are total
value operations. Recoverable language and system failures use typed
`Result`/`Option` values rather than a proof-failure path.

After source checking, selected-target qualification proves concrete object
layout, element stride, allocation byte ceilings, frame materialization, and
address-index representability before emission. A source proof of
`i < len_of(buffer)` does not by itself prove that the selected target can
represent `base + stride*i`; the target stage checks that separate obligation.
An unrepresentable target is a target compilation failure and emits no partial
operation.

The only boundary temporarily left outside the source outcome model is
external resource availability: heap exhaustion, stack exhaustion,
operating-system quotas, and runtime-start resources may stop execution at the
host boundary without a Whitefoot value or cleanup guarantee. This does not
defer layout, address, allocation-ceiling, target-domain, target qualification,
parallel independence, or bounded queue/completion proof, and resource failure
never establishes a source fact or licenses an unproved operation.

## Parallel and completion lowering

Parallel permission is derived from the same checked program. The compiler
uses ordinary data dependencies, ownership and loan overlap, exact effect
footprints, control exits, and already-discharged operation goals. It does not
repeat a bounds proof to authorize an index map. The counted-loop path
currently supports a fixed single-binder affine map `a*i+b`, with one identical
map required at every read or write to the same root. This includes
same-index read-modify-write and an output reached through a live usable
`&uniq` holder, as well as the enumerated exactly-associative reductions.
Sibling-call and staged-I/O judgments use their own fixed, fail-closed shape
rules.

Permission and actualization are separate. The default lowering actualizes
eligible finite completion operations while leaving compute-call outlining
off. `--par` additionally actualizes eligible compute groups, maps, and
reductions. A denied permission leaves the program sequential; it does not
change source acceptance. Proof-only statements introduce no runtime branch,
lock, dependency, scheduling event, or task edge.

The first multi-operation loop path is deliberately specific: one
source-derived fixed two-slot bounded batch for the direct staged counted-loop
shape. On native POSIX completion targets the runtime window is bounded to
`1..2`. A qualified target without native completion uses the same generated
CFG with a deterministic window of one and direct calls. The driver issues up
to that window, drains the complete batch in source order, and only then reuses
slot zero. Backend evidence covers dynamic per-iteration paths, an odd final
batch, the ordinary result/error arm, LLVM emission, linking, and execution.
When one function contains two staged loops, both deliberately remain ordinary.
Wider control flow, operation families, and multi-loop selection are possible
future extensions, not v0.40 activation gaps or permission to infer a broader
path from this one.

The completion runtime uses bounded, generation-checked operation storage and
separate exactly-once result-ready, loan-released, and terminal milestones.
Native queues, helper lanes, wakeups, and completion ports are target-private
protocol state, never Whitefoot shared storage. The macOS and Linux paths are
qualified for the implemented operations, including Linux io_uring where its
route is available.

The exact `x86_64-pc-windows-msvc` row is native-qualified for the
compiler-owned UTF-16 command bootstrap and the direct, bounded blocking, and
IOCP positioned-I/O routes. An IOCP-eligible request cannot silently use the
direct or blocking route: handle association or submission failure stops at
the host boundary. At full bounded storage the emitter retires the oldest
addressable source-owned generation; when no one-slot owner is addressable it
waits for core progress, then retries that same request. Native probes require
zero eligible fallback. Synchronous-success operations publish inline only
after the runtime has disabled their completion packets; pending operations
publish through the IOCP worker.

Every emitted Windows `--par` module requires the compiler-owned compute pool
through hard external ABI obligations. A missing runtime fails to link, and an
invalid worker configuration or partial startup fails at the host boundary
instead of selecting sequential execution. The native gate requires a
non-owner worker to execute and steal source work while preserving the
sequential build's exact bytes. A fixed-host paired gate qualifies compute,
warm IOCP, and mixed compute-plus-IOCP execution against matched controls on
the same revision.

`--par-ledger` prints the permission and actualization explanation for compiler
development. `--stack-ledger` reports selected-host frame costs. Neither report
participates in source acceptance or lowering authority.

## Implemented language surface

The compiler currently carries the following families through semantic
checking, typed IR, LLVM, linking, and execution where the selected host
supports them:

- fixed-width integer, strict `f32`/`f64`, Bool, unit, comparisons, conversions,
  bit operations, and the specified exact/total arithmetic modes;
- ordinary and counted control flow, `match`, `if`, `give`, `propagate`,
  `set`, and affine replacement;
- acyclic structs and enums, `Option`, `Result`, fixed arrays, runtime buffers,
  boxes, the two direct views `Slice<'r, T>` and `MutSlice<'r, T>`, and a
  finite monomorphizing generic subset;
- shared and unique borrows over the implemented storage forms, exact
  caller-visible state effects, compiler-derived cleanup, and verified
  contracts; and
- the current command entry, owned system resources, positioned I/O,
  directory enumeration, typed host errors, and completion lowering.

This list is an implementation map, not a second language specification. The
compiler deliberately reports remaining active-spec gaps as unsupported and
keeps conservative LLVM when no specification-backed optimization fact exists.
The largest such gap today is the general store. Both runs execute:
`fixed_vector` forms a frame-resident one, `arena_frame` reserves one bump extent
in the reserving activation's own frame and `arena_vector_proved` and `arena_vector`
take a store-resident one from it, [BLK-3]'s four boundary operations move
either run's boundaries, `len_of`, `cap_of`, `room_of` and `head_of` read the
measures of a run and of a store, a subscript reads the window at
`(head_of + i) mod cap_of`, each row's requirement is discharged at the call
under [MSR-4] and each row's declared relations are published at the caller
under [CALL-6]. What is not implemented is `heap_vector`, which stops as an
explicit unsupported capability, now for one reason rather than two: [FN-7]'s
`command.heap` row is DEFERRED, so no program can obtain a `Heap<'s>` value at
all. The second reason is gone — a run's release class is decided from its
store region's declaration alone and travels on `CheckedType::Vector` and
`IrType::Vector`, so a region-erased lowering can select a heap-backed run's
free from an arena-backed run's empty action; nothing spends one yet, and a
unit test pins the four classifications. A source function is generic over a
store now: a parameter type naming a formal region determines that region from
its actual and is substituted with it, so `fn carve['s: affine](store: &uniq
Arena<'s, 256, 16>) -> made: own Option<Vector<'s, u64>>` declares, checks and
runs. Two stops remain. A proved take whose count is not a closed expression
stops, because `advance<T>(count)` is then an opaque term with no source
spelling and its requirement has no difference-bound form a caller could
discharge; the refusing row is the one for that position. And a run whose
element type is itself a run of runs stops, explicitly: the element domain
carries **one** level of lift, so `FixedVector<Vector<'s, u8>, 8>` and
`FixedVector<FixedVector<u8, 4>, 4>` are represented and a third level is not.
That one level is real all the way down: `CheckedElement` and `IrElement` are
the lifted domain, a slot holding a run has that run's own layout in A.1's
ceilings, the element read and the element store move the whole descriptor, and
[PROV-6]'s release walk visits the window in ascending logical order before the
run's own backing is released — a per-run helper the emitter derives, so a run
of heap-owning elements frees each of them once
(`tests/programs/block_pool.wf`, `prov6-pos-a-run-visits-its-window-before-its-backing`).
A formal region a parameter names one level down, in a run's element type, is
determined by its actual exactly as a top-level one is, which is what makes a
helper generic over the store of the runs a run holds. A run's element type is
otherwise every type [BLK-1] states —
every copy element, one region-free affine nominal stored by value, and a type
parameter under any of its three bounds, which [FN-2] resolves at every
concrete instance. Element-position writes into
a run execute: `set v[i] = e;` and `replace v[i] = e;` commit at the window's
logical offset `(head_of + i) mod cap_of`, under [OP-4]'s ordinary subscript
obligation judged at the target place and [MSR-2]'s storage-granular kill, so
the store kills every measure of the element and none of the run's own.
**There are two views now, and the exclusive one writes.** [S35] capitalizes the
view nominals, so v0.44's `slice<'r, T>` is spelled `Slice<'r, T>` and the
lowercase word is an ordinary identifier again; `MutSlice<'r, T>` [VIEW-1] is
the added view, formed by `mut_slice_of(&uniq p)` beside `slice_of(&p)`, and
`set view[i] = e;` through it compiles, links and runs — the write goes through
the view's own data pointer to the storage it views, and the descriptor is
unchanged. [SET-1] admits a target path through a view exactly at the exclusive
strength, so the same statement through a `Slice` is the refusal probe `p7`
measured. **The storage has to be addressable, and an `array<T, N>` is not**: an
array is a value here — an element commit rebuilds it and writes it back to its
binding — so the descriptor a view of one carries points at a snapshot, and a
write through an exclusive view of an array would reach the snapshot. That stops
as the explicit unsupported capability `ExclusiveViewOverArray` rather than
lowering a write nobody can observe; the shared view over an array is unaffected,
because a live shared loan refuses every write to its origin and the snapshot and
the array therefore agree wherever the view is readable; a `FixedVector<T, n>`
is inline storage for the same reason and stops the same way. Exclusivity is not
a clause of its own: the formation takes the borrow
its strength names, so a second `mut_slice_of` over one place meets the first
view's loan and is refused there as an ordinary [OWN-5] conflict, while two
`slice_of` views of one place are admitted.
**The shared view is copy, and its loan ends at its last use.** A `Slice` is
used bare, a `move` of one is [OWN-1]'s `MoveOfCopy`, and the storage it reaches
is writable again after that view's last use rather than at the end of its
region — which is what lets a run be appended to inside the block a view of it
was formed in. A commit at either view type is [VIEW-4]'s refusal, because a
copy target would otherwise be displaced with nothing consumed.
**The two runs are viewable**, and the formation carries the row's own
requirement `head_of(vector) <= room_of(vector)`, submitted at the call and
judged under [MSR-4]: a run whose window wraps is refused citing [BLK-0], and one
drained to empty is accepted. The two formation rows' record data is [BLK-0]'s —
a viewable operand class, a shared-borrow operand mode, the requirement and four
published relations — while their spelling stays an [OP-1] table entry until
`array<T, N>` and `buffer<T>` retire, because two domains may not claim one
spelling.
**A shared view of a place a live exclusive view holds is that view's child
reborrow**: it is admitted, and the parent may not write the elements it views
until the child's last use. **The same child forms through a view holder.**
`slice_of(&'r deref(destination))` at a `&uniq MutSlice<'r, u8>` parameter is
that reborrow with a view as the parent: the child carries the parent's origin
set and range, its region is the one the operand borrow writes and the parent's
own region must outlive it, and the freeze stands both inside the callee — where
the loan sits at the holder's own place, which is what an element write through
that holder resolves its origin to — and at the caller, where a shared loan is
registered on every origin place the returned child reaches that already carries
an exclusive one. `mut_slice_of` over a view holder is refused citing [OWN-5].
The result is legal because [VIEW-6]'s ceiling admits a borrow-mode view
parameter's formal origin for a **shared** view result at the same region and
element type, at either parent strength, and for no exclusive one.
**A helper re-lends the destination it was handed** with
`&uniq deref(destination)`. A view value is already a descriptor, so the child
is that descriptor read once more rather than an addressed reborrow of one;
there is nothing to load and nothing to narrow, which is why the semantic arm
and the lowering arm are one line each.
**The system boundary takes views.** The range-bearing parameter of each of the
seven operations [SYS-8] names is one operand class rather than one type:
`&uniq MutSlice<u8>` where the operation writes the storage, `&Slice<u8>` where
it reads it, and `buffer<u8>` at the same position for as long as `buffer<T>`
lives. A view handed to a call is borrowed and not consumed, so the caller's
facts about it survive the call, and the write a callee performs through one is
an element write over the view's own place [ENT-5, MSR-2], so the view's
measures survive it and its element facts do not. A `&uniq` parameter whose
referent is a view is admitted at a source declaration for the same reason
[BLK-4]. What still stops is the exclusive view over an inline run
(`ExclusiveViewOverInlineRun`), which is why a writable destination is a
`Vector<'s, T>` taken from a store or a bump extent and never a
`FixedVector<T, n>`.
**One judgment does not follow a view yet, and it is the permission one.** The
[PAR] footprint resolver reads a direct `slice_of` expression and a borrow; a
*bound* view value resolves to no place, so an overlap pair or a staged loop
whose call hands one on is denied for the unresolved-footprint condition rather
than for its own reason. `par_layout.wf` therefore hands its metric table on as
`&Vector<f64>`, a shared borrow of the run, and keeps both of its eligible
folds.
**A shared borrow of a run is the ordinary borrow.** `&FixedVector<T, n>` as a
parameter, a `let`-bound holder over either run, and a run reached through a
shared borrow of the nominal that owns it all reach the borrow through one path:
a run's storage lives in its owner, so a borrow of it is the address of that
storage exactly as a borrow of a struct is, and the holder's `deref` resolves to
the same measured place the deref-free path forms. A run holder written where
the run itself is required is [TYPE-7]'s missing dereference, as a `buffer`
holder already was.
**A `const` may name the inline run.** A `FixedVector<T, n>` of const-eligible
flat `T` with exactly `n` literal entries is const-eligible: the item's storage
type is the run of `n` slots itself, because its four measures are the standing
facts `len_of = cap_of = n` and `room_of = head_of = Z` and the checker's array
place already is exactly those four constants over that storage. Nothing else is
added — the subscript discharges from `len_of = n`, the four readers answer from
the type, `slice_of` gives the `immutable-const` origin, and `mut_slice_of` and
a `set` through it are the two [CONST-2] refusals a const already had. The wart
B7c4b-1 recorded here — a diagnostic about such an item spelling its type
`array<T, N>` — is closed, and it was two warts rather than one: the message
also named `index` and `len`, neither of which is a current spelling. It now
names the run, the subscript, the four readers and the shared view, which is
exactly the read set above.
**A `set` target is a published relation's destination.** A single-target
`set x = helper(...)` takes result ordinal zero through the same destination
route a destructuring `let` binder and a `set` target list take, with the
target's own kills as the events every substitution must survive; [FN-9]'s
narrow receiver route is the case of it where the target is also an argument.
**Every acquiring kernel row's allocation-fit obligation is submitted.**
`heap_vector`, `arena_vector` and `arena_vector_proved` each carry [OP-9]'s
allocation-fit obligation over their element type and count — the obligation
[BLK-0]'s record notation spells `fits::<T>(count)` — and it is judged at the
call through the same path `buffer_new`'s is, so an unconstrained count is the
ordinary static [OP-9] rejection. `fixed_vector` carries none, its count being a
type constant, and the two cell rows carry none, taking no count.
**One declaration may carry both a const generic parameter and a region
parameter**, the instance being keyed on the const argument while the region is
substituted positionally from the call's own operands, and **a generic call
cycle that instantiates the callee at exactly the caller's own parameters
monomorphizes** — [FN-6] permits it and the call mints no second instance. A
cycle that derives a *const* argument from the caller's own const parameter has
an unbounded instance set and still stops as an explicit unsupported capability;
[FN-6]'s syntactic rule is written over type parameters and does not refuse it.

[MSR-3]'s placements are per placement and not per depth: a measured value keeps
its measures across a `let` binder, a `set` target, a construct's field, a
destructuring binder, an element position and a single-payload enum's arm
binder, and a struct operand carries the measures of every run beneath it and
not only its own, so a run two field levels down arrives at its new path with
what it had. A `requires` or `ensures` side is an affine
expression [GRAM-4, GRAM-5, MSR-5], a parameter's measure named in an
`ensures` is its entry datum [MSR-3], and an in-scope const generic is an
affine atom [MSR-6, INV-1], which together are what let the container
design's own fixed-run library — `vacant`, `filled`, `take_at`, `try_place`,
`try_take` and `rebase` — prove its contracts and execute
(`tests/programs/fixed_run_library.wf`), each capacity-parametric loop stating
its bound as the const generic itself. All six are generic in their element
type again: a type parameter carries exactly one written bound [S37], the
three classes form the chain `copy < affine < linear` whose satisfaction is
that chain read left to right, and the template is the spelling authority, so
one `affine`-bounded body serves `u8` and `Option<u8>` and the program
exercises each at both. `rebase` is what needed [LIV-2]'s declaring `set`
target, which the resolver mints by its own lookup: a bare identifier target
that resolves to no binding becomes an ordinary `let` declaration there.
`tests/programs/arena_workspace.wf` is the store-backed companion: it reserves
an extent, reads the store's own cursor across each take, fills a taken run and
observes that a refused take leaves the cursor where it was.
`tests/programs/block_pool.wf` is 3.L.4's block pool **entire**, its two
nominals included: `struct BlockPool['s]` holds the free list,
`linear struct Lease['s]` holds the leased run, and `pool_new`, `pool_take` and
`pool_release` are all three generic over the store. A source nominal's
`region_params` are components of its type name, so an instance is keyed on its
region arguments beside its type and const arguments and two instances at two
regions are two types; a `type` position writes those arguments as the
leading members of the same `targs` list the two runs and the two providers
already use, and a parameter type naming such a nominal determines its region
from the actual exactly as `Vector<'s, T>` does. A `construct` writes only what
its own operands leave open [FORM-8]: a field whose declared type names one of
the nominal's region parameters determines it from its actual, exactly as a
parameter position determines a callee's formal, so `BlockPool(free: move
free)` writes nothing and `BlockPool<'a>(free: move free)` is refused with
`drop the region argument`, while `struct Ticket['s] { count: u64; }`, no field
of which names `'s`, is still built `Ticket<'a>(count: 7_u64)`. The instance is
therefore formed *after* the operands are checked and not before — the shape a
construct needs beforehand is read off the declaration's own symbolic instance,
whose region arguments are its region parameters — and construction still
consults no expected nominal type. Where the axis leaves the
program is the lowering: a region names a store for the proof and nothing at run
time, so two instances that differ only in their region arguments — and in
nothing a run time can see, a run's release class included — are **one IR
nominal**, which is what lets a callee's own formal-region instance and a
caller's actual-region instance meet at the boundary between them. A store
region is also invariant at a call now: the first parameter position that names
a formal fixes it, so two runs of two extents no longer satisfy one `'s` by
taking the least region.

Two source-shape bounds the pool met are worth naming. A loop that allocates
from a `&uniq` store parameter has **one statement per iteration**, because a
child reborrow's region may not extend beyond its own statement [OWN-6] and a
loop body's own region extends over the whole body; `pool_new`'s body is
therefore one `match` over the acquiring row's own call. And every clause naming
a measure over a *result*'s field — `ensures head_of(rest.free) == ...` — is
[CALL-4]'s own first DEFERRED admission, so the pool states none and its caller
reads `room_of(rest.free)` and branches; a `requires` over a **parameter**'s
field measure is an ordinary [MSR-1] place and is what `pool_release` proves
with.

An affine element leaves its slot through one further route: the [LIV-2]
read-out of an element target of the same `set`, so
`set (v[i], v[j]) = move v[j], move v[i];` exchanges two elements in one commit
when the two offsets are literals with unequal values.

A tracked place is a root plus **field selections and subscripts** in written
order, so `len_of(table[i])` is a term and `grid[i][j]` reads and writes. The
offset a subscript carries in a place is a written literal, a live `own`
fragment-integer binding, or an in-scope const generic, because the place's
identity is decided over it: [OWN-7] decides two subscripted places by their
offsets and [ENT-5] takes each offset's own support into every measure term it
occurs in. [OWN-7]'s relation reads the complete path — two places fail to
overlap exactly when some step of their common prefix provably selects two
different storages — so `grid[k]` and `grid[i][j]` are decided at `k` against
`i` and never at their last offsets, which is what [LIV-2]'s second condition
and its element read-out both read. [MSR-2]'s granularity is that relation
rather than a flag: an element write carries the element's own place, so it
kills every measure of `P[i]` and no measure of `P`, and a whole-value write of
`P` kills both.

**Which of the two a projected callee write is comes from the callee's
declaration** [CALL-5]. One `CallTransport` is computed per declared parameter
from its declared mode and type and is the same at every call site: a `&'r`
parameter of any type is [CALL-1]'s shared borrow and the call kills nothing
through it, a parameter of view type own or behind a borrow is [CALL-3]'s viewed
range and the write reaches element storage only, an `own` parameter is
[CALL-2]'s value, and every other `&uniq` selects none and kills the actual's
descriptor storage. A callee with no body is read the same way from its own
record: [SYS-8]'s range-bearing operand class is a viewed range at either of its
members, which is why the I/O corpus keeps its lengths across a `read_at`
whether it supplies a `MutSlice<u8>` or the transitional `buffer<u8>`, and a
kernel row's `&uniq` state operand is a run or a provider whose descriptor the
row changes. The argument expression's shape is read for the *place* a write
reaches and never for how far into it that write goes; deriving the latter from
the former was the unsound accept the sweep of 2026-09-03 recorded, and
`ent5-neg-a-callee-write-through-a-uniq-extent-kills-the-room` is its successor
over the one measured non-view referent a `&uniq` parameter may still name. A subscript inside a measured place owes [OP-4]'s obligation
against the prefix that reaches its base, submitted where the place is formed,
and the lowering projects a measured place step by step — a field selection is
the ordinary struct projection and a subscript is [BLK-1]'s element read — so a
descriptor is read through the slot address that holds it. An [INV-1] affine
factor reaches the same place: its measure place is formed under the enclosing
concrete instance and at the enclosing loop depth, so
`invariant flat: len_of(grid[0_u64]) <= cap_of(grid[0_u64])` is a header
relation, and the subscript inside it owes [OP-4]'s bound where the relation is
written — at the loop header in its entering context, at an `invariant`
statement at that statement — because a measure over a place whose subscripts
are not all discharged is no term there either.

Two soundness repairs landed with that path. A call's region arguments are
substituted into **every** position of the callee's signature, results included
and at any depth — through `Option`, into a nominal instance's own region
arguments, into a run's element position and into every ordinal of a declared
result list — so two calls of one declaration at two extents hand back two
types; a result used to keep the declaration's own formal region, which let a
run of one store be typed later as a run of another. And a run's element
**read** now owes [OP-4]'s bounds obligation, which only its element-position
target did: `let run = fixed_vector::<u8, 4>(); let seen = run[0_u64];`
compiled, linked and ran, reading a slot outside an empty run's window.

A third one is what [BLK-0]'s consistency sentence is for. A kernel row's
operand denotation is decided by its position [MSR-3] *before* the call-datum
table is consulted, because that table is keyed on the call, the ordinal, the
projections and the measure and on nothing that separates a `&uniq` state
operand's post-state from that call's `at the call` datum; reading the datum for
both made `arena_vector_proved`'s own `len_of(store) = len_of(store at the call)
+ advance<T>(count)` the pair of bounds `advance<T>(count) <= 0` and
`>= 0` over one term. And a row's declared effect row is a callee effect like
any other, so the place its `writes` names is written by the call and every fact
whose support that place reaches dies there; without that the post-state term is
still pinned by the caller's pre-call `len_of(store) = 0` and the same
contradiction arrives one statement later. Either way [ENT-4]'s least closure
made the caller's whole fact state universally discharging, so a function that
called the row discharged *every* [OP-4] obligation it contained and a nine-slot
read of a four-slot array compiled, linked and ran. The class is now closed at
both ends: a unit test closes each row's own requirement and relation lists per
declared exit under the same difference-bound closure [CALL-6] uses, and the
establishment path asserts at every call of every row that the caller's fact
state did not turn contradictory across the half of its set every exit carries.
Across the **routed** half no such assert is made, because a routed relation is
available only on the arm its route names and a contradiction there is the
ordinary statement that the arm is not reached: an acquisition asked for more
bytes than the extent holds publishes `len_of(store) = len_of(store at the call)
+ advance<T>(count)` on a `Some` arm the caller can refute, and the arm it makes
underivable is the arm that never runs. What is asserted on every exit instead is
the denotation itself — a measure a row names both `at the call` and in its
post-state is two terms at every instantiation — which is the position the defect
above actually occupied.

A row's routed relations reach the arm the caller matches. The destination list a
binder or target list gives an unrouted result and one arm of a `match` over a
routed one are one publication path over one filter: an unrouted relation is a
member of every exit's set and a routed one only of the arm its route names, and
the payload place a routed clause names is the arm's own binder. A caller of
`arena_vector` therefore holds the four measures of the run on its `Some` arm and
`room_of(store) < advance<T>(count)` on its `None` arm, where before it held
neither.

A measured value keeps its measures across seven naming events [MSR-3], each one
minting an immutable datum before the statement's own kills and reading it back
after them: body entry, a call's pre-transfer point, a `let` or [LIV-2] `set`
rebind, an element-position commit, the value a [SET-2] `replace` displaces, a
`construct`'s field operand, a destructuring consume's binder, and a `match` arm's
payload binder. Two of them carry a boundary the place representation fixes. An
element position is a place only at an offset the place relations can name, so
the boundary rows carry nothing through the slot they write — `place_back` stores
at `len_of(vector)` and `take_back` takes from `len_of(rest)`, and a measure term
is not such an offset — and a run pushed onto a free list and leased back off it
still arrives with no measures of its own. And a place's path names no variant, so
the payload placement is stated over an enum exactly one of whose variants carries
fields; `Option` is one and `Result` is not.

A fourth repair came with the element placement. [ENT-5]'s element-position
carve-out is removed rather than narrowed, and the L0 measure-term path landed
that removal while the **goal** path kept the old clause: an element write killed
no measure goal at all, so a signed goal over `len_of(P[i])` survived the write
that replaced `P[i]`. Nothing stood on it while nothing re-established that
measure, but the element placement does, and the two together made the fact state
contradictory and a nine-slot read of a four-slot array compile. A measure goal
now dies at an element write over a place the written place is a prefix of, and no
other, which is the sentence the L0 path already read.

It has no termination checker and emits no `willreturn` or effect-derived alias
attributes.

A `header_invariant` and an `invariant_stmt` do carry a measure former as an
affine factor [INV-1, GRAM-4], which is what lets a loop state the relation the
operation it calls needs on the backedge. The factor's affine atom is
retargeted by exactly the [ENT-5] events that kill its term, so a header
conclusion never outlives the write that refutes it. The derived release of a
type whose release graph has a cycle is one release action per node type
calling itself [PROV-6]; the depth is the value's, which is the ordinary
stack-availability question [SCOPE-3] defers for a program that is not
`resource_closed`, and the stack ledger reports it as a `STACK cycle` row.

### Known defect: unguarded affine expression nesting depth

A proof-domain affine expression nesting parentheses about 1400 deep aborts
the driver with a stack overflow and no diagnostic at all (exit 134,
`fatal runtime error: stack overflow`); 1200 rejects normally in a third of a
second, and about 20000 does not even abort within twenty seconds, so a
superlinear cost sits on top of the recursion. It reaches this from both a
`use` premise and an `invariant` target, so it is in the shared `affine_expr`
handling rather than in either position. Measured against a build from before
the v0.48 `use` amendment, it reproduces identically, so it is not that
amendment's.

An internal error is not a source rejection, and a crash with no diagnostic
gives a writer nothing to act on. The repair is the pattern this compiler
already uses for structural limits — the 4096-entry `proof_use` capacity and
`AffineCheckError::LimitExceeded` — applied to nesting depth, in whichever of
the parser and the semantic former actually overflows. Removed when that limit
exists and a test pins it.

### Known defect: a runtime-sized allocation fails with no rule and no location

```
fn make(capacity: own u64) -> result: own buffer<u8> allocates(heap) {
  let backing = buffer_new(capacity, 0_u8);
  return move backing;
}
```

stops with `TargetLayout/TargetLayout:
TargetLayout(Unrepresentable(RuntimeSizedAllocation))` — no rule id, no source
coordinate, no line, no mechanical fix. The program's actual defect is an
undischarged size obligation, and `requires capacity <= 1000_u64;` fixes it,
but nothing in the output says so. It passes semantic checking silently and
stops four stages later. A sweep of 22 struct-owning-buffer programs hit it in
all 22, which is why the corpus carries `1000_u64` ceilings that read as style
and are compensation for this.

The stage taxonomy is right — this is not a `Compiler` channel failure — but
for a writer it is indistinguishable from one. The obligation belongs in the
semantic walk with the shape `[OP-4]` and `[OP-2]` already use: a rule, a
residual, a mechanical fix. Pre-existing; reproduces on a pre-v0.48 build.
Removed when the rejection carries a rule and a location.

### Known cost: a large `proof_use` block is impractical well below its ceiling

[PRF-1] admits 4096 `proof_use` entries in one block and calls that "a source
structural ceiling, not a work or time budget". Measured, the checker costs
389 ms at 64 entries, 3.0 s at 128, and 26.8 s at 256 — about eight times per
doubling, which puts the admitted ceiling many hours away. Pre-existing and
not specific to any one entry shape; a pre-v0.48 build measures the same at
128. Nothing in the corpus writes a block anywhere near this size, so this is
recorded rather than fixed. Removed when the ceiling is reachable, or when the
specification says what the real limit is.

## Running and checking

From `compiler/`:

```sh
cargo run --bin whitefootc -- source.wf -o program
cargo run --bin whitefootc -- --emit-llvm source.wf
cargo run --bin whitefootc -- --par source.wf -o program
cargo run --bin whitefootc -- --par-ledger source.wf -o program
cargo run --bin whitefootc -- --stack-ledger source.wf -o program
make check
```

`whitefootc` accepts an ordered bundle of multiple source files. `--no-overlap`
selects the exact sequential reference lowering and cannot be combined with
`--par`. When a report and emitted LLVM would otherwise share stdout, name the
LLVM output with `-o`.

From the repository root, `make check` is the canonical complete gate;
`make static` alone runs the stages that read the tree without running a
compiled program, `make spec-append-only` among them, which checks that no
released `spec/kernel-spec-vN.md` archive differs from main's.
