# Iterative reference descent

The target is an executable linked-list walk, a branch-selecting tree walk,
and a link-slot cursor which removes a node without leaving a hole. The
same implementation must handle aliases, joins and nested loops, reject
destruction of a selected place, and retain deterministic terminating
checking without a runtime reference descriptor or check.

The starting evidence is the independent study at
[`dafff174`](https://github.com/mbbill/Whitefoot/tree/dafff174/research/investigations/wildcard-path).
Its decisive cases distinguish an existing selected payload from the lexical
fact which selected it, a possible-location cover from target identity,
ancestor destruction from a content write, and a finite fixed point from
one extra visit. Its examples are design arguments, not executed evidence.

## Selected mechanism

Separate already checked selection from later selection. A reference to an
existing payload keeps naming that place after leaving the selecting match;
a later selection still requires a current variant fact. Enum replacement,
owner movement, release and window removal still invalidate descendants.
Apply this rule uniformly to ordinary and summarized references: giving
only widened references this property would make validity depend on whether
an unrelated loop needs a summary.

A changing loop-header reference retains a finite possible-location cover
for each root. Unchanged entering static shapes retain their opaque captured
offsets. When a contribution leaves those entering shapes, replace them with their common
known prefix followed by a subtree cover, including that prefix itself.
Further contributions can add roots or shorten the prefix, never expand an
unknown tail. A distinct header target identity permits finite projections
relative to one current target; equal covers alone confer no equality.
Straight-line aliases snapshot this identity. Each rebound header holder
has its own identity; no heap-shape or cross-iteration equality is inferred.

The structural checker grows these header summaries monotonically and
restarts its ordinary statement walk whenever a new contribution changes
one. Only the settled walk publishes checked statements and obligations.
The existing owner-tagged validity equations still solve entry/backedge
validity and aliases. Proof closure is not iterated as a loop invariant.
This reuses the typed place and expression path instead of introducing a
second syntax interpreter. The cost of replay is a provisional tradeoff:
measure it on alias chains and nested loops, and replace replay by a shared
transfer graph if those costs prevent practical checking.

There are finitely many roots, headers and written static prefixes. For
each header/root, a shape disagreement introduces a cover once; thereafter
its prefix can only shorten. Rebound offset identities are finite header
identities. Thus a restart strictly grows a finite-height state. Numeric
checking runs only over the resulting checked function. There is no pass
count, fuel, timeout or acceptance budget.

A cover overlaps its anchor, every ancestor and every descendant. Finite
suffixes can prove separation only when both accesses share a captured
target identity. A primitive leaf store cannot destroy a cursor, but still
kills overlapping value facts. Structural writes preserve a target at or
above the written place and invalidate unrelated potentially overlapping
cursors. Moves, releases and window changes keep their stronger consequences.
Call arguments receive no blanket exemption from another actual's write.
Exchange requires equal-or-disjoint targets, excluding possible proper
ancestry. Effects and parallel permission use the same conservative covers.

The executable cursor removal exposed a pre-existing displacement defect:
`set deref(cursor) = without_first(head: move deref(cursor));` was accepted,
but lowering released the already consumed old value a second time. The
checked post-right-hand-side disposition now belongs to the commit for all
target shapes, including references and elements, rather than only a named
binding target. This restores the existing cleanup-traversal decision; the
backend neither rediscovers liveness nor infers it from the target's shape.

Range summaries retain their range kind as well as element type. An indexed
write must run the shared readonly-origin check after resolving its complete
suffix; the new range cursor case exposed that missing check in the existing
indexed-target path. Both single-target and range-target cursors therefore
retain readonly provenance through widening and later projections.

## Alternatives and tradeoffs

Recursion and indexed pools remain usable but do not implement iterative
owned-link cursors. A raw wildcard step without target identity confuses
two independently selected nodes and loses safe sibling-field calls. A
single extra body visit misses multi-holder propagation. Full heap-shape
proofs could recover independent-cursor precision but require a different
proof language. Restricting all structural writes while a cursor lives
would prevent the requested link-slot edits.

Once widening is needed, one prefix per root can lose precision when a holder alternates between
different static shapes. Independent cursors in one cone cannot both
survive a potentially destructive write. These are explicit conservative
limitations, not runtime checks or evidence of a general cost bound.

## Validation criterion

Before using results to qualify this choice: require executable list, tree
and cursor cases; positive unchanged/same-target and separated-root controls;
hostile alias, ancestor, refinement, window, readonly, stale-fact, call,
exchange and parallel-footprint cases; and loop cases propagating through
at least three holders, with nested and joined targets. Retain existing
static-index controls. Measure compiler checking separately from building
the compiler and executing generated programs. Report source sizes, loop
shapes, repetitions and limits; do not infer asymptotics from a few points.
For the replay implementation specifically, the qualification target is a
median source-check time below one second for the complete cursor program
and for a 64-holder propagation chain on this development host, over five
runs after compiler construction. Missing that target reopens replay rather
than changing any acceptance rule; the corresponding cost TODO remains open
until the measured limitation is repaired or explicitly accepted. Larger
points qualify scale but do not create a source-size limit.

The specification and formal tests own admitted behavior. This document
owns the selection grounds and measurements, and is superseded in place if
the mechanism changes. No correctness gate depends on research files.

The explicit cost experiment uses `measure.rs`, retained while this summary
implementation's scaling question remains relevant. From the repository
root, construct the compiler with `make -C compiler build`, then link the
standalone Rust driver against `compiler/target/gate/deps/libwhitefoot-*.rlib`
using `rustc --edition=2024 -O`, `--extern whitefoot=<that library>` and
`-L dependency=compiler/target/gate/deps`. Place the executable outside the
repository and run it through `.github/run-check.pl`. The driver reports
each of five complete source checks, separately from compiler construction
and any target-program execution. `--check <source>...` prints individual
source verdicts for diagnosis; it is not a conformance runner or gate path.

## Measurement on 2026-09-21

Apple M1 Pro, arm64 macOS, Rust 1.98.1; ordinary optimized `gate` compiler
profile, with compiler construction excluded from these times. The build
took 43.48 seconds. Five consecutive complete `whitefoot::check` calls per
case include the prelude, parsing, semantic checking and proof analysis;
the executable program was separately compiled and exited successfully.
The shared check guard excluded other Whitefoot gates during this run.

| Source | Bytes | Five source-check times (ms) | Median (ms) |
| --- | ---: | --- | ---: |
| List, tree and removal cursor program | 4471 | 19.432, 17.589, 18.105, 17.530, 18.892 | 18.105 |
| 1 holder | 428 | 12.163, 11.710, 11.982, 11.713, 11.825 | 11.825 |
| 8 holders | 799 | 12.507, 12.423, 12.583, 12.941, 12.701 | 12.583 |
| 16 holders | 1242 | 14.345, 14.839, 14.632, 14.012, 13.898 | 14.345 |
| 32 holders | 2138 | 19.748, 19.679, 20.517, 20.858, 20.457 | 20.457 |
| 64 holders | 3930 | 43.969, 42.444, 43.851, 43.185, 43.029 | 43.185 |
| 128 holders | 7599 | 144.619, 143.918, 143.871, 145.676, 143.720 | 143.918 |
| 4 holders, 2 nested loops | 647 | 11.985, 12.119, 11.550, 11.577, 11.483 | 11.577 |
| 4 holders, 4 nested loops | 779 | 12.036, 12.169, 12.041, 12.001, 12.237 | 12.041 |
| 4 holders, 8 nested loops | 1091 | 14.150, 14.208, 14.047, 14.015, 14.050 | 14.050 |
| 2 joined roots | 555 | 11.421, 11.558, 11.891, 11.315, 11.368 | 11.421 |
| 8 joined roots | 1035 | 13.083, 13.024, 13.131, 13.020, 13.221 | 13.083 |
| 32 joined roots | 3043 | 48.078, 46.832, 46.618, 46.638, 47.266 | 46.832 |

Both predeclared one-second criteria pass. These observations support
retaining replay for the present experiments, not a linear-time claim or a
cost bound for arbitrary functions. The 64-to-128-holder increase shows a
remaining scale cost; reopen the transfer-graph alternative when a real
program needs that scale or a later measurement misses the criterion.
Precision remains intentionally conservative for independent cursors in
one subtree and for changing captured indices. A write through a widened
range can also discard a prior length bound because its support covers the
subtree; selecting an element reference before the write, or establishing
the bound again afterwards, avoids relying on that lost fact. This study
does not establish precision for all recursive range edits. The formal
cases own the acceptance boundaries. No measurement threshold selects
source acceptance.
