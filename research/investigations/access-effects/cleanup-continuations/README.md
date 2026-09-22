# Cleanup continuation models

These two bounded Rust programs test whether compiler-derived cleanup can walk
selected recursive owner layouts without allocating a worklist and without
using the host call stack. They are investigation artifacts, not compiler
code, specification text, conformance evidence, or a selected general
lowering.

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
