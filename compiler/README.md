# Whitefoot compiler

This directory contains the safe-Rust Whitefoot compiler. It is one evolving
compiler crate, not a collection of stable libraries. Module boundaries are
private implementation choices; the active language is defined by
[`spec/kernel-spec.md`](../spec/kernel-spec.md), not by the compiler source or
this README.

The frontend targets the exact v0.40 bytes at `../spec/kernel-spec.md`,
SHA-256
`5079ef2efa7862184f06ccf7dc273ae97eda791679a44f66c86e75afbc46c6e0`.
The outgoing exact v0.39 bytes are archived at
`../spec/kernel-spec-v0.39.md`. `whitefoot-spec` checks the selected
specification identity, activation chain, rule inventory, and generated syntax
identity as one compiler gate.

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
four source forms in the ordinary semantic walk:

- `requires` states what every caller must prove before argument transfer. A
  successfully checked call makes the instantiated facts available at callee
  entry without an executable prologue.
- `ensures` is proved at every selected normal return. Verified summaries are
  published atomically by call-graph component and then instantiated at later
  callers; a recursive component cannot bootstrap itself from an unpublished
  summary.
- A counted-loop `invariant` is a direct prefix statement of that loop body.
  The compiler proves its base case and its preservation from an arbitrary
  reachable header through one body fallthrough and the hidden unit update.
  Normal exact exhaustion may export the separately justified binder-free
  consequence; a `break` does not receive that consequence.
- `prove`/`use` supplies an affine target and ordered premises. Every premise is
  independently proved in the same pre-statement context, then the checker
  verifies the exact coefficient-one sum in source order. It does not choose a
  premise, coefficient, intermediate relation, branch split, or rewrite.

At each program point the semantic checker has one current `ProofContext`.
Selected control-flow edges, type and declaration facts, checked requirements,
verified postconditions, proved invariants, and successful source proof steps
update that context. Numeric and logical consumers submit one normalized goal
to the shared proof entry. The current implementation runs the fixed ordinary
ground/difference-bound closure first and the fixed affine rule where that same
goal has an affine form. Ownership, initialization, effects, layout, target,
and parallel permission remain separate deterministic domains tied to the same
checked source flow; this is not a universal solver.

Acceptance uses no SMT solver, random seed, heuristic proof search, or timeout.
Every closure order, source traversal, arithmetic operation, and work ceiling is
fixed. Internal derivations are produced with the originating decision for
diagnostics; they grant no independent authority.

Contracts, invariants, and source proof statements have no runtime behavior.
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
currently supports a fixed write-only single-binder affine map `a*i+b`, with
one identical map required at every write to the same owned root, and the
enumerated exactly-associative reductions. Sibling-call and staged-I/O
judgments use their own fixed, fail-closed shape rules.

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
route is available. The Windows IOCP core remains execution evidence rather
than a qualified compiler target. Selective stackless continuation lowering is
deliberately narrow; unsupported control-flow shapes retain the synchronous
ABI instead of weakening ownership or completion bounds.

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
