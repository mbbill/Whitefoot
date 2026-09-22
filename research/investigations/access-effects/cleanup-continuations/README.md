# Cleanup continuation models

This investigation combines a native compiler reproduction, an abstract
continuation-cost comparison, and the two earlier Rust models of selected
recursive owner layouts. The models test traversal without a cleanup-time
worklist allocation or depth-dependent host stack. They are research artifacts,
not compiler code, conformance evidence, or a selected general lowering.

## Current-compiler investigation, 2026-09-22

The resumed question is whether merged self-tail lowering also removed the
compiler-generated release stack, and, if it did not, what a general replacement
would require. The baseline is main `f3cf41d4` (PR #75), specification v0.62.
Mutual source tail calls and termination proofs are outside this investigation.

After code inspection and an initial automatic-chain ledger probe, the criteria
for the controlled comparisons and replacement study are:

- Compare an ordinary scope release of a heap-linked chain with the maintained
  `tests/programs/tail_list.wf` consuming traversal. Inspect both unoptimized
  emitted LLVM and the ordinary optimized host assembly/stack ledger. A remaining
  nonzero-cost cleanup cycle establishes a depth-dependent stack cost even if a
  selected shallow execution succeeds. A native depth control must distinguish
  construction from cleanup and must not turn an exhaustion boundary into a
  language rejection.
- Any proposed general replacement must account for the continuation of every
  suspended struct field, enum variant, fixed array and runtime window, including
  recursive layouts without an enum tag. It must preserve the specification's
  exact release order, visit only live elements, release each allocation once,
  and neither allocate nor grow traversal state with value depth during cleanup.
  Additional persistent object storage and target pointer assumptions are costs
  to state explicitly, not facts to hide in the model.
- A model must compare complete free traces with a separate structural oracle,
  use deep and mixed-layout cases, and count traversal-state and allocation
  requirements. Passing another closed type grammar is useful evidence only for
  that grammar; it does not establish a general layout theorem. A native compiler
  implementation and its performance remain separate qualification work.

This study compares implementation alternatives; it does not select a new
language rule or replace the live `compiler/cleanup-traversal` decision.

### Native result and the two different cleanup paths

The compiler at `f3cf41d4` was rebuilt with `make -C compiler build`. On
arm64 macOS 26.6.2, Apple clang 21.0.0 with the ordinary `-O2` arguments,
[scope-drop.wf](scope-drop.wf) constructs 100,000 links iteratively and leaves
the root for automatic scope release. Its emitted `wf.drop.t0` contains:

```llvm
%drop.1 = load %wf.t0, ptr %drop.0
call void @wf.drop.t0(%wf.t0 %drop.1)
call void @free(ptr %drop.0)
```

The free after the call is real remaining work. The optimized stack ledger
reports `wf.drop.t0` as a **32 B/level cycle**. The maintained
[`tail_list.wf`](../../../../tests/programs/tail_list.wf) instead has a
48-byte `wf_drain` frame and no recursive machine-call cycle. Its owned content
extraction releases the current Box before transferring ownership of the next
Box to another iteration. That explicit consumption has a different free
order from automatic content-before-Box release; it is a control for the
self-tail implementation, not a semantics-preserving drop replacement.

For a bounded native depth experiment, only the floor runtime's reservation
constant was changed in a scratch copy, from 1 GiB to 1 MiB, following the
existing stack-ledger boundary test's method. All other runtime units were
built through `compiler/runtime.mk`; the emitted module and `-O2` were unchanged.
Core dumps were disabled for the expected exhaustion run.

| Source / links | Result with the diagnostic 1 MiB stack |
|---|---|
| Automatic release / 1,000 | Exit 0, empty stderr |
| Automatic release / 100,000 | SIGABRT, shell status 134, `{"resource":"stack"}` |
| Explicit consuming self-tail traversal / 100,000 | Exit 0, empty stderr; expected sum verified |

Thus construction at the deep size succeeds in the control, while an automatic
release retains a measured recursive frame. This is not a claim that 100,000
links exhaust the shipped 1 GiB reservation: the ledger's nominal
`33554432 levels` is a division of that reservation by 32, not a measured safe
depth including all entry/runtime overhead. The bytes per frame are specific
to this compiler, host target and module.

[PR #75](https://github.com/mbbill/Whitefoot/pull/75) changes source self-call
selection and entry-jump lowering; it does not
replace `backend/emitter/cleanup.rs`. [PR #70](https://github.com/mbbill/Whitefoot/pull/70)'s final description explicitly
defers general bounded-stack cleanup (its review item #47). Commit `7044db24`
adds the two layout models below; `ea141997` records their limitation and the
retained recursive compiler. This explains the earlier research without
interpreting a successful model as a shipped compiler capability.

### What the missing continuation contains

A suspension must retain which allocation owns it, where to continue within
that allocation, and how to resume its parent. An enum also needs the selected
variant; an array/window needs an element position. Retaining these in native
activation records is what the current emitter does. Eliminating the calls
still requires somewhere to put this information.

Whitefoot supplies useful conditions: no writer finalizers or unwinding,
compiler-known release graphs and layouts, and a finite concretely instantiated
program. A generated release dispatcher can therefore represent its own calls
as state transitions. This would not need a source mutual-tail-call feature or
an inter-function tail-call ABI. What is unestablished is the storage and
transition scheme for all admitted layouts, not the existence of a loop syntax.

The live rule requires declaration order inside aggregates, logical order
inside windows, and content release before the enclosing Box free
[STOR-3, PROV-6]. A parent cannot simply be freed on descent and then be used as
a continuation. Reusable storage means **consumed fields in a still-allocated
object**, never reading allocator-freed memory. Missing heap tag bits, padding,
or spare window capacity must not be assumed.

### Alternatives and a continuation-cost control

[continuation-cost.rs](continuation-cost.rs) separates continuation control from
the still-open typed-layout problem. Its input is an ordered ownership tree;
child lists stand for a correct typed field iterator, not Whitefoot object
bytes. It compares two nonrecursive controllers:

- Restart at the root after each free, marking a consumed child slot empty.
  This adds no continuation fields but repeatedly discovers the remaining path.
- Reserve a parent and a next-child cursor for each abstract node before
  traversal, then resume directly. These two machine words are persistent
  storage in the model, not a cleanup-time allocation. They exclude concrete
  type dispatch, alignment and the typed iterator's own state.

The comparison counts slot reads, not elapsed time. Both controllers allocate
nothing in their traversal functions; their tree, header array and fixed-size
observer are prepared beforehand. The observer verifies the entire free order,
missing/duplicate releases and reads through freed nodes. Shallow mixed trees
use an independent recursive oracle; deep chains use the analytically known
leaf-to-root sequence. Both candidate controllers use fixed local state.

Rust 1.98.1, `rustc --edition=2021 -C opt-level=2 -D warnings`, produced:

| Shape | Nodes | Restart slot reads | Parent/cursor slot reads |
|---|---:|---:|---:|
| Chain, 1,000 edges | 1,001 | 501,500 | 1,000 |
| Chain, 2,000 edges | 2,001 | 2,003,000 | 2,000 |
| Chain, 4,000 edges | 4,001 | 8,006,000 | 4,000 |
| Wide root, 4,000 children | 4,001 | 8,006,000 | 4,000 |
| Mixed fields, seed 1 | 1,480 | 19,894 | 1,479 |
| Mixed fields, seed 7 | 1,159 | 15,922 | 1,158 |
| Mixed fields, seed 41 | 989 | 13,264 | 988 |
| Mixed fields, seed 127 | 136 | 1,544 | 135 |
| Chain, 100,000 edges | 100,001 | Not run | 100,000 |

All order/state assertions passed, including the empty root. For both the
chain and wide root with n edges the restart implementation makes exactly
`n*(n+3)/2` slot reads, asserted at 1,000, 2,000 and 4,000. Direct resumption
reads each edge once. Its two-word headers consume 16 bytes per abstract node
on this host: 1,600,016 bytes for the deep case, excluding observer and input.
This is neither a Whitefoot allocation measurement nor a claim that two words
suffice for all its concrete continuation states.

| Alternative | Benefit | Unresolved cost / disposition |
|---|---|---|
| Keep recursive helpers | Current simple lowering, no new object state | O(depth) machine stack remains; larger reservations only move the boundary |
| Allocate a worklist while releasing | General explicit continuation storage | O(depth) auxiliary storage and a new allocation/failure point during release; the current tree already refuses this design |
| Repeatedly rescan from the root | Fixed local state without added continuation fields in the model | Quadratic work even for chains and wide roots; unsuitable as the ordinary large-container path |
| Reserve continuation storage with each participating allocation | Direct resumption, no cleanup-time allocation | Persistent heap/layout cost; the model isolates the controller but does not establish the WF encoding |
| Reuse consumed object fields and tags | Potentially direct resumption with unchanged normal layout | Needs a storage proof for every continuation state and target; the two existing models cover only their stated grammars |

The preferred next experiment is a **type-directed continuation layout
calculation**, not another handwritten linked-list specialization. Derive a
continuation state at each recursive release edge, list its still-live fields,
parent link, resume identity and dynamic cursor, then show exactly where these
fit. Begin with the existing tagged and tagless models, and challenge the
calculation with nested inline arrays, differently sized mutually recursive
contents, full/empty/wrapped runtime windows and first-element suspension.
A failed fit is a concrete representation question; it is not an impossibility
proof. If some states need reserved bytes, compare a selective reservation
against a uniform reservation before proposing a production layout.

Any later candidate must additionally qualify native pointer provenance and
alignment, complete allocation-size accounting under OP-9/STOR-6, partial moves
and residual releases, and sequential/parallel use of the same release path.
No source language refusal, pointer-bit convention, release-order amendment or
production representation is selected by this study.

### Relation to the cited work

The [2019 paper, sections 2–3](https://guillaume.munch.name/files/efficient-drops-mlworkshop.pdf)
derives continuation reuse for tagged algebraic representations and explicitly
identifies tag-space constraints for less uniform layouts. The
[2024 paper, sections 2 and 5](https://guillaume.munch.name/files/modular_deconstruction.pdf)
extends the approach to arrays and sketches unboxed subfields using offsets and
tagged continuation pointers; its concluding proof/layout questions remain
relevant. These support investigating a systematic transformation, not assuming
that Whitefoot already has the required bits or that the examples implement
its exact window and Box release order. The correspondence to Whitefoot above
is this investigation's analysis, not a result supplied by either paper.

### Design suitability and artifact lifetime

The existing checked release graph is the semantic owner, and one eventual
continuation lowering should consume that graph for both ordinary and parallel
emission. A second source checker or container-specific compiler path would
duplicate that responsibility. The concrete opportunity is to make automatic
cleanup independent of value depth without changing writer programs. The
remaining layout and cost questions stay in `docs/todo.md`; the research models
do not change the current lowering decision.

`scope-drop.wf` serves the native stack reproduction; `continuation-cost.rs`
serves the storage-versus-repeated-work comparison. They belong with these
existing cleanup models and are explicitly run by the commands below. Replace
or remove them once a general lowering and maintained regressions supersede
those questions. No correctness gate consumes research.

## Existing layout models

The compiler still emits recursive release actions. Generalizing these
models is the [open bounded-stack cleanup item](../../../../docs/todo.md),
not behavior supplied by these research programs.

The programs use a preallocated `Vec<u64>` as an aligned word-addressed heap.
All graph construction and observer allocation finish before traversal. The
traversals use a fixed-size program counter and machine-register state, mutate
only words whose source values have been consumed, and compare the complete
free sequence with an independent oracle. The observer rejects invalid,
wrong-kind, repeated, and missing frees and poisons freed storage.

## `tagged.rs`

The first grammar is deliberately closed:

```text
E = Leaf | ViaS(Box<S>) | ViaT(Box<T>) | ViaRing(Box<Ring<E>>)
S = { first: Box<E>, second: Box<E> }
T = { first: Box<E>, second: Box<E> }
```

`E` has one eight-byte tag/padding word followed by one eight-byte payload slot
for every payload-bearing variant. An active `E` uses only its tag word and the
selected variant slot: the tag holds the caller continuation, and the slot
holds the predecessor-slot address. `S` and `T` reverse their consumed field
links. A ring replaces `{len, cap, head}` with `{outer predecessor, remaining,
one-past-capacity}`; the completed element address is its cursor. The cursor
advances by the fixed `E` stride and wraps at the saved end, so vacant capacity
is neither normalized nor scanned.

The fixtures cover leaf roots, alternating `S`/`T`, empty rings including
capacity zero, full and partial wrap, nested rings, four fixed generated
shapes, and an alternating chain of depth 100,000. The deep case performs
300,000 frees with traversal state independent of value depth.

## `anchorless.rs`

The second executable removes the enum anchor:

```text
S = { left: Box<Ring<S>>, right: Box<Ring<S>> }
```

For this exact grammar, a right-field frame can be represented by
`right -> left -> predecessor`. The return path recognizes it by
`heap[right] == right - 1`. This marker is derived from the modeled layout:
the root predecessor is zero with a root base of at least eight, while an
inline `S` begins at `ring_base + 3 + 2*i` and its predecessor is `ring_base`.
The predecessor therefore cannot equal `left - 1`.

The model checks empty leaves, wrapped nested rings, and a depth-100,000 chain
with 200,002 frees against exact expected order. The address marker's proof
depends on this exact two-field layout. Generalizing it requires a concrete
encoding of each suspended caller's return site, including arbitrary field
positions, mutual types, and nested inline arrays and windows. These tests
supply neither that encoding nor a counterexample proving it impossible.

## Reproduction

Run from the repository root while the shared verification guard is free:

```sh
make -C compiler build
cleanup_root=$PWD
cleanup_study=research/investigations/access-effects/cleanup-continuations
cleanup_scratch=$(mktemp -d "${TMPDIR:-/tmp}/whitefoot-cleanup.XXXXXX")
perl .github/run-check.pl cleanup-auto-ledger compiler/target/gate/whitefootc \
  --emit-llvm --stack-ledger "$cleanup_study/scope-drop.wf" \
  -o "$cleanup_scratch/automatic.ll"
perl .github/run-check.pl cleanup-tail-ledger compiler/target/gate/whitefootc \
  --emit-llvm --stack-ledger tests/programs/tail_list.wf \
  -o "$cleanup_scratch/explicit.ll"
sed 's/100000_u64/1000_u64/' "$cleanup_study/scope-drop.wf" \
  > "$cleanup_scratch/shallow.wf"
perl .github/run-check.pl cleanup-shallow compiler/target/gate/whitefootc \
  --emit-llvm "$cleanup_scratch/shallow.wf" -o "$cleanup_scratch/shallow.ll"

# Same runtime, only a smaller diagnostic reservation.
sed 's/^#define WF_FLOOR_STACK_BYTES .*/#define WF_FLOOR_STACK_BYTES ((size_t)1024u * 1024u)/' \
  compiler/src/backend/wf_floor.c > "$cleanup_scratch/floor-small.c"
cat > "$cleanup_scratch/native.mk" <<'MAKE'
include $(ROOT)/compiler/runtime.mk
.PHONY: all
all: $(filter-out $(BUILD)/native/wf_floor.o,$(NATIVE_OBJECTS))
MAKE
perl .github/run-check.pl cleanup-runtime make -f "$cleanup_scratch/native.mk" \
  all ROOT="$cleanup_root" BUILD="$cleanup_scratch" CLANG=clang
perl .github/run-check.pl cleanup-small-floor clang -std=c11 -O2 -pthread \
  -c "$cleanup_scratch/floor-small.c" -o "$cleanup_scratch/floor-small.o"
for cleanup_mode in automatic explicit shallow; do
  perl .github/run-check.pl cleanup-link clang -O2 -Wno-override-module \
    "$cleanup_scratch/$cleanup_mode.ll" "$cleanup_scratch/floor-small.o" \
    "$cleanup_scratch"/native/*.o "$cleanup_scratch"/native/sched/*.o \
    "$cleanup_scratch"/native/completion/*.o -pthread -lm \
    -o "$cleanup_scratch/$cleanup_mode-small"
done
ulimit -c 0
perl .github/run-check.pl cleanup-shallow-run "$cleanup_scratch/shallow-small"
# This diagnostic control is expected to return 134 and the stack record.
perl .github/run-check.pl cleanup-auto-run "$cleanup_scratch/automatic-small"
perl .github/run-check.pl cleanup-tail-run "$cleanup_scratch/explicit-small"

perl .github/run-check.pl cleanup-controller-build rustc --edition=2021 \
  -C opt-level=2 -D warnings "$cleanup_study/continuation-cost.rs" \
  -o "$cleanup_scratch/continuation-cost"
perl .github/run-check.pl cleanup-controller-run "$cleanup_scratch/continuation-cost"
```

These native commands describe the POSIX host used above, not a Windows
qualification. Keep the scratch output while inspecting the result, then remove
that explicitly created directory. The controller is an abstract work/storage
comparison; native throughput and concrete layout costs have not been measured.

The two earlier layout models have their separate reproduction commands:

```sh
perl .github/run-check.pl cleanup-continuations-tagged-compile \
  rustc --edition=2021 -C opt-level=2 \
  research/investigations/access-effects/cleanup-continuations/tagged.rs \
  -o /tmp/whitefoot-cleanup-tagged
perl .github/run-check.pl cleanup-continuations-tagged-run \
  /tmp/whitefoot-cleanup-tagged
perl .github/run-check.pl cleanup-continuations-anchorless-compile \
  rustc --edition=2021 -C opt-level=2 \
  research/investigations/access-effects/cleanup-continuations/anchorless.rs \
  -o /tmp/whitefoot-cleanup-anchorless
perl .github/run-check.pl cleanup-continuations-anchorless-run \
  /tmp/whitefoot-cleanup-anchorless
rm /tmp/whitefoot-cleanup-tagged /tmp/whitefoot-cleanup-anchorless
```

On 2026-09-20 both compilations and runs exited zero. The tagged run ended
with `all cleanup-model assertions passed`; the anchorless run reported its
two ordinary fixtures, the 100,000-deep chain, and
`all anchorless-model assertions passed`.

## Interpretation and limits

The experiments are related to destructive traversal and efficient-drop work
described in [Munch's 2019 ML Workshop paper](https://guillaume.munch.name/files/efficient-drops-mlworkshop.pdf)
and the [2024 JFLA paper](https://inria.hal.science/hal-04406342/file/jfla2024-paper-13.pdf).
Their observations apply only under the representations stated above.

They do not cover arbitrary anchor-gap graphs, shared or cyclic ownership,
failed construction, unwinding, allocator reentrancy, arbitrary aggregate and
enum nesting, real-backend provenance, or types without suitable consumed
storage. Neither this pointer-reversal proposal nor pointer tagging has been
established for the whole Whitefoot type system. No compiler, specification,
or live design-tree conclusion follows from these runs alone.

These models are invoked explicitly during this investigation. They have no
CI or `make check` dependency. Remove them when a general implementation and
maintained regression tests supersede the questions they isolate.
