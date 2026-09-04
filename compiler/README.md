# Whitefoot compiler

This directory contains the safe-Rust Whitefoot compiler. It is one evolving
compiler crate, not a collection of stable libraries. Module boundaries are
private implementation choices; the active language is defined by
[`spec/kernel-spec.md`](../spec/kernel-spec.md), not by the compiler source or
this README.

The frontend targets the exact v0.43 bytes at `../spec/kernel-spec.md`,
SHA-256 `037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`.
v0.43 makes every loop body a region block, so a borrow written in the body
takes that body's own region bare, and repairs [ENT-6]'s control-flow join so
nested joins reach the flat join's image; it supersedes v0.42 at
`6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26`, whose
outgoing bytes are archived at `../spec/kernel-spec-v0.42.md`.
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
`i < len(buffer)` does not by itself prove that the selected target can
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

Selective stackless continuation lowering is deliberately narrow; unsupported
control-flow shapes retain the synchronous ABI instead of weakening ownership
or completion bounds.

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
  boxes, direct slices, and a finite monomorphizing generic subset;
- shared and unique borrows over the implemented storage forms, exact
  caller-visible state effects, compiler-derived cleanup, and verified
  contracts; and
- the current command entry, owned system resources, positioned I/O,
  directory enumeration, typed host errors, and completion lowering.

This list is an implementation map, not a second language specification. The
compiler deliberately reports remaining active-spec gaps as unsupported and
keeps conservative LLVM when no specification-backed optimization fact exists.
It has no termination checker and emits no `willreturn` or effect-derived alias
attributes.

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

From the repository root, `make check` is the canonical complete gate.
Candidate specification work can run `make spec-candidate-integrity` before
activation; this is an integrity check, not a separate branch permission.
