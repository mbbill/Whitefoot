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
for each root. An unchanged static shape retains its opaque captured
offsets. When shapes for one root differ, replace them with their common
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

## Alternatives and tradeoffs

Recursion and indexed pools remain usable but do not implement iterative
owned-link cursors. A raw wildcard step without target identity confuses
two independently selected nodes and loses safe sibling-field calls. A
single extra body visit misses multi-holder propagation. Full heap-shape
proofs could recover independent-cursor precision but require a different
proof language. Restricting all structural writes while a cursor lives
would prevent the requested link-slot edits.

One prefix per root can lose precision when a holder alternates between
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

The specification and formal tests own admitted behavior. This document
owns the selection grounds and measurements, and is superseded in place if
the mechanism changes. No correctness gate depends on research files.
