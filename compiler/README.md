# Whitefoot compiler

This directory contains the safe-Rust Whitefoot compiler. It is one evolving
compiler crate, not a collection of stable libraries. Module boundaries are
private implementation choices; the active language is defined by
[`spec/kernel-spec.md`](../spec/kernel-spec.md), not by the compiler source or
this README.

The frontend targets the exact bytes at `../spec/kernel-spec.md`. Their
version and SHA-256 are derived from those bytes by `build.rs` on every build
that touches them, and every other identity constant in the crate reads that
generated module. The generated identity is not committed; amending the
specification changes it in the same build. `whitefoot-spec` checks identity
consistency, rule references and inventory, derivation-row coverage, and
generated syntax identity. The root `make check` also checks that released
specification archives have not changed. There is no approval-ledger chain.

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

Within a `use` block, `use (a <= b);` writes a relation premise and
`use bound;` cites a named invariant. The optional `N times` prefix scales
that premise by a bare decimal from two upward or an admitted unsigned value
name. An explicit decimal one is omitted, and the same normalized premise
cannot be repeated. PRF-1 defines the exact admissible forms and the bounded
product-folding rule for named multiplicities.
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

For a `--par` module with reachable staged I/O hand-outs, `WF_WORKERS=1`
keeps cooperative I/O overlap on the entry worker. A compute-only module
still takes its sequential clone at one worker. `WF_WORKERS=0` explicitly
selects sequential execution in both cases. The bootstrap passes this minimum
worker count through an internal runtime query; source function signatures
do not change. This separates CPU parallelism from I/O concurrency but does
not guarantee progress when the configured window or stack capacity is too
small for a peer protocol.

The work-branch experiment `--par --sched-quantum N` inserts a cooperative
checkpoint after N natural-loop backedges per function activation. It keeps
source signatures and proof checking unchanged, progresses completions, and
gives an already-ready stack a turn. It does not guarantee admission, preempt
arbitrary recursion or long host calls, or establish a wall-clock service
bound. The default emits no checkpoint counter or calls. Measurement and
selection live in `research/investigations/io-model/SCHEDULER-EXPERIMENT.md`.
The alternative `--par --sched-chunks N` keeps recognized unsigned unit-stride
loops intact within chunks of at most N iterations and checkpoints between
chunks. Their upper bound must be invariant and the bound test must leave the
loop. Empty ranges, early exits and integer limits retain their source
behavior; other loops use the existing counter fallback. This also remains
an experimental scheduling policy without a source progress guarantee.

`--continuations --emit-llvm` is a separate work-branch representation
experiment qualified with Linux LLVM 20 and local Apple clang 21. Existing derived
`may_suspend` effects select switched-resume coroutine frames, including
nested and recursive calls; pure functions keep their ordinary representation.
Source signatures and proof checking are unchanged. Without `--par`, the
experiment keeps the serial source schedule and awaits each mapped direct
completion operation before continuing. Adding `--par` actualizes existing
checked staged-loop permissions with independently suspended task roots;
pure compute outlining stays off. One research-host thread resumes all roots
and their nested callees. The issuer's frame owns task descriptors and results
until the original drain joins and destroys each child. The network extension
also awaits `tcp_listen`, `tcp_accept` and `tcp_connect`, using the same
typed completion mappers and retirement as their ordinary wrappers.
Compute checkpoints, asynchronous cleanup, Windows and normal executable
linking are not integrated, so incompatible command-line modes fail explicitly.
Other system wrappers and cleanup may still block the host. The new host ABI
is experimental and does not alter the existing staged runtime or container
storage contract. `compiler-continuation-check` in the I/O completion experiment
qualifies actual WF-generated code against native byte oracles and sanitizers.

`--continuation-compute --emit-llvm` adds an opt-in pure-call scheduling
experiment. A checked empty effect row and an IR cost estimate select calls;
an unchanged bounded scalar entry prefix can identify a cheap return before
CPU admission. Long calls use the existing CPU pool while their caller's
coroutine remains suspended. Arguments and results stay in its typed frame,
and the host retires the worker slot only after core completion, before
resuming the caller. Submitted, executing and completed-unretired work is
bounded by twice the actual CPU worker count; admission waiters retain their
source activation storage. With `--par`, workers keep ordinary compute
outlining and the I/O owner calls sequential helper clones. Without actual
CPU workers, calls run sequentially. Direct owner-side range split operations
remain an explicit experimental capability gap. This changes no source
signature or acceptance rule and establishes no source progress guarantee.
`compiler-continuation-compute-check`, reached by canonical `make check`,
checks the generated mixed protocol, held-worker light progress, frame reuse
and failed/partial CPU startup under sanitizers. Performance qualification,
general cancellation and effectful compute offload remain open.

The first multi-operation loop path is deliberately specific: one
source-derived fixed two-slot bounded batch for the direct staged counted-loop
shape. On native POSIX completion targets the runtime window is bounded to
`1..2`. A qualified target without native completion uses the same generated
CFG with a deterministic window of one and direct calls. The driver issues up
to that window, drains the complete batch in source order, and only then reuses
slot zero. Backend evidence covers dynamic per-iteration paths, an odd final
batch, the ordinary result/error arm, LLVM emission, linking, and execution.
When one function contains two staged loops, both deliberately remain ordinary.
Wider control flow, operation families, and multi-loop selection remain
possible future extensions; this path does not imply those capabilities.

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

Suspended staged callees currently retain separate scheduler stacks; direct
completion submissions retain records in their caller frames. The earlier
selective stackless emitter has been removed. The `--continuations` work-branch
experiment described above re-evaluates continuation representation through a
new lowering path; the default remains stackful.

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

Contracts currently support the FN-8 requirement vocabulary and FN-9's
restricted integer-result relations, including the selected `Ok` payload
route. They are not a general specification language for aggregate results
or mutable data-structure invariants. A contract-member `fn_sig` cannot carry
a function `contract_block`. Verification is over the closed source bundle;
independent module checking remains future work.

## Finding the implementation

| Responsibility | Entry point |
|---|---|
| Stage boundaries and failure categories | [driver.rs](src/driver.rs) |
| Parsing and canonical source form | [syntax](src/syntax/mod.rs) |
| Names and declaration identity | [resolution](src/resolution/mod.rs) |
| Types, ownership, effects, and checked statements | [semantic checker](src/semantic/check.rs) |
| Proof facts, kills, joins, and obligation consumers | [entailment flow](src/semantic/entailment/flow.rs) and [fact state](src/semantic/entailment/state.rs) |
| Affine arithmetic and written sums | [affine core](src/semantic/entailment/affine.rs) |
| Typed control-flow lowering | [lowering builder](src/lowering/builder.rs) |
| Target qualification and LLVM emission | [qualification](src/backend/qualification.rs) and [emitter](src/backend/emitter.rs) |

Read the owning rule and nearby tests for the change in hand. The map is a
navigation aid, not another definition of language or proof authority.

## Known limitations

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

The source-proof path includes target AUTO for redundancy and separate proof
queries for relation-form premises; named premises check published theorem
availability. Automatic queries can rebuild fact closures and enumerate
premise combinations. Weighted sums also merge growing coefficient vectors.
The recorded measurements do not isolate these costs, so they do not establish
that certificate accumulation alone is the bottleneck. Profile the stages
before changing the implementation or the accepted proof rules.

## Running and checking

From `compiler/`:

```sh
cargo run --bin whitefootc -- source.wf -o program
cargo run --bin whitefootc -- --emit-llvm source.wf
cargo run --bin whitefootc -- --par source.wf -o program
cargo run --bin whitefootc -- --par-ledger source.wf -o program
cargo run --bin whitefootc -- --stack-ledger source.wf -o program
```

`whitefootc` accepts an ordered bundle of multiple source files. `--no-overlap`
selects the exact sequential reference lowering and cannot be combined with
`--par`. When a report and emitted LLVM would otherwise share stdout, name the
LLVM output with `-o`.

For focused development checks, run these from the repository root:

```sh
make static
make -C compiler format lint
cargo test --manifest-path compiler/Cargo.toml --profile gate --locked --offline --lib semantic::tests::source_proofs
make -C compiler test-unit
```

Use a test filter matching the responsibility changed; `source_proofs` above
is one example. The `gate` profile retains debug assertions and overflow
checks while optimizing the compiler's analysis work. `test-sampling` owns
repeated runtime schedules and `test-corpus` owns integration targets;
`test-partition` checks the test split. The [Makefile](Makefile) owns the exact
inventory.

The root `make check` is the canonical complete gate. `make -C compiler check`
runs only its compiler stages. The root gate also runs research tests,
conformance structure and coverage, the full native conformance adapter, and
the snapshot corpus. `make spec-append-only` checks released archives against
local `main`; it does not replace the complete gate or approve a merge.
