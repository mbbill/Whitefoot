# x1 container interface probes

These are explicit research probes for
[X1-LIBRARY.md](../../../investigations/containers-and-resources/X1-LIBRARY.md),
not conformance cases or a library implementation. They distinguish ordinary
owned-value operations from source-interface limits under kernel v0.60 at
`efd6ebc9efc3012735f4ddedf6979617d321d3a6`. No compiler, library or spec fix is
part of the experiment. The comparison criteria were committed in `619a14cb`
before these probes were run.

## Construction and observations

The compiler was built once from that pinned revision's unmodified compiler
with Cargo's optimized `gate` profile, offline and locked, two build jobs,
under the shared verification guard. Construction took 42.24 seconds wall
(70.88 user, 1.19 system). Its SHA-256 is
`7fbdea110e0aa192b37c564ff1d09e226424f722d03b453691bd948a2dcf4bdb`.
The host is arm64 macOS; native linking used Apple clang 21.0.0
(`clang-2100.3.34.2`). Runs used the ordinary default lowering, without overlap
flags, a modified compiler or substituted native implementations.

| Source | Normative expectation | Observation | What the program establishes |
| --- | --- | --- | --- |
| [ownership-swap.wf](ownership-swap.wf) | Accept: OP-11, PROV-6 and FN-2 | Emitted LLVM; native exit 0 | One generic vacancy exchange, instantiated for scalar, Box and nodrop payloads; indexed insert, replacement, extraction and explicit cleanup for the nodrop case. |
| [reverse-drain.wf](reverse-drain.wf) | Accept: OP-11, OP-10, FN-3..5, OP-14 | Emitted LLVM; native exit 0 | A generic reference helper consumes four nodrop elements in their original order after reversal, publishes zero length, and permits free_empty. |
| [copy-ring-spans.wf](copy-ring-spans.wf) | Accept: REF-4 on an Array | Emitted LLVM; native exit 0 | The physical tail and beginning of an initialized copy-element array can be consumed as two ranges in queue order. This is not a complete deque. |
| [bounded-reserve.wf](bounded-reserve.wf) | Accept: OP-9 with a verified count bound | Emitted LLVM; native exit 0 | A bounded u64 reserve publishes capacity/length, then append/take operates over the grown backing. |
| [ring-range.wf](ring-range.wf) | Reject: REF-4 | REF-4, RangeOverRing, at the range in the count call | The refusal also applies to an empty, nonwrapping range. |
| [append-contract.wf](append-contract.wf) | Reject: FN-9 | FN-9, InvalidPostconditionRelation, at the ensures | A sum of two entry measures is outside the published difference-bound fragment. |
| [unbounded-reserve.wf](unbounded-reserve.wf) | Reject: OP-9 | OP-9 at grow; residual `count <= 2305843009213693951_u64` | A safe concrete caller does not discharge missing requirements of the helper body. |
| [linear-ring-publish.wf](linear-ring-publish.wf) | Reject: WIN-3; OP-12 excludes linear targets | WIN-3, LinearAssignmentTarget, at line 18 | Appending into a new Ring does not make the reference target eligible for the affine/copy atomic-update form. |

The initial versions of the first two positive probes incorrectly nested
constructors in argument-atom positions. GRAM-9 rejected them before the
intended question. They were corrected to explicit local bindings; those
authoring errors are not compiler findings or evidence about containers.

Seven-source observation took 0.31 seconds wall after compiler construction;
the final eight-source observation, including linear-ring-publish, took
0.32 seconds wall (0.19 user, 0.05 system). All four rejections were checked
at the intended sites against the retained diagnostics.
A four-program native compile/link/run invocation took 3.53 seconds wall
(1.89 user, 0.60 system) before the scalar/Box extension of ownership-swap;
that extension, including installed-value checks, passed its final focused
rerun in 0.94 seconds wall. These are command durations,
not steady-state operation timings. The shared guard declined starts while
PR #70 owned verification; no concurrent build or bypass was started.

## Interpretation and limits

An empty enum variant is a valid value even when its type can also hold a
nodrop payload. Swapping with it transfers the current occupant without a
hole; consuming the returned enum accounts for its actual payload. This
requires no automatic quantified fact about all vacant slots.

The reverse-drain probe gives a linear element-movement algorithm, not a
native-performance floor. Its full operation count and callback-observation
condition are in the investigation. Generated unoptimized LLVM still contains
aggregate transfers around owned parameters and result helpers. Merely counting
those instructions does not price optimized execution; no claim of retained-
helper timing, C parity, or zero-copy construction follows from these runs.

The native probes cover ordinary execution of the stated small cases, not
all lowering modes, an allocator release ledger, generation wrap, complete
map/deque/slab growth, a complete B-tree or hostile behavior. In particular,
the Box instantiation's successful execution does not prove exact allocator
counts. Those observations belong to the next library implementation trials.
The four rejections agree with the selected rules and are not compiler defects.
The reserve migration finding concerns library code missing the rule's bound.
The Ring publication probe instead exposes a scope question: OP-12 explicitly
limits atomic updates to affine/copy targets, while the candidate and design
tree state their admission without that qualifier. This is recorded as X1-P3
for the owner/#70 agent, without changing either rule or implementation.
No native success or complete nodrop Ring-growth route is claimed from that
negative probe.

## Reproduce

From the repository root, at a checkout with the pinned compiler sources:

```sh
perl .github/run-check.pl x1-probe-build cargo build --manifest-path compiler/Cargo.toml --profile gate --bin whitefootc --locked --offline --jobs 2
perl .github/run-check.pl x1-probe-observe make -C research/experiments/container-representation x1-observe
perl .github/run-check.pl x1-probe-native make -C research/experiments/container-representation/x1 native
```

`observe` records process outcomes, standard output, diagnostics and emitted
LLVM under `OUT` (default `/tmp/whitefoot-x1-container-probes`). It deliberately
does not turn the expected rejections into a failed research command, and its
exit 0 is not a conformance pass. Read the source/exit table and diagnostics.
`native` stops on a compile, link or execution failure; each executable uses
an independent expected result and returns nonzero on a mismatch.
`WHITEFOOTC`, `OUT`, and `NATIVE_SOURCES` may select a pinned compiler, scratch
directory and focused positive subset. Neither target is called by daily CI
or canonical make check. Re-derive expectations against a changed specification
before interpreting a later compiler's outcome.
