# Deciding probes for proof-derived parallelism

These nine Whitefoot sources are the probes that actually decided something
during the three research rounds behind `../DESIGN.md`. They were written and
run outside the repository, so they were dying artifacts; they are landed here
because the design's load-bearing claims cite them and a claim whose evidence
has evaporated is an assertion. They are **archived evidence, not a gated
corpus**: no build, test, or tool reads this directory, they are not held to
the canonical-form or must-compile rules that `tests/programs/` enforces, and
they are deleted when the design decision each one settles is superseded.

Every verdict quoted below is what the in-tree compiler's
`whitefootc --par-ledger` reports for that file today.

**The permitted-but-not-actualizable line.** `a2_bubble.wf` is the two-child
`uniq` tree fold that started the whole investigation: its recursive child pair
is permitted, and the ledger reports `not-actualizable: 1 claim site via
bubble`. `d2_tree_zeroclaim.wf` is the same fold with its two overflow claims
rewritten as ordinary branches, and the identical pair comes back `eligible` —
together they are the cleanest demonstration that a claim, and only a claim, is
what stands between a permitted pair and an actualized one. `a2r_layout.wf` and
`a2r_layout_two.wf` carry that contrast into a realistic body (a box-tree
layout with a per-node float cascade and a word loop): the single-table version
is `eligible`, and the two-table version, whose second buffer forces a derived
index and therefore two claims, is `not-actualizable: 2 claim sites via
measure_words`. `a2r_layout.wf` is the direct ancestor of
`tests/programs/par_layout.wf`, the measured demo in `../RESULTS.md`.

**The condition-4 requirement.** `g2_propagate.wf` is why the judgment's fourth
condition exists at all. Its `probe` function propagates an `Err` edge out of
the function *before* a following write executes, so under any overlap that
ignored exit edges the write would run when the sequential execution would have
skipped it — a compiling, terminating program that breaks the equivalence law.
The ledger is silent on this file, and that silence is the point: the judgment
analyzes ordered pairs of adjacent `let x = f(...)` statements, and the hazard
here is a `let` followed by a `set`, which is never a candidate. The probe
established the requirement; the per-condition denial is pinned by the compiler
tests, not by this file.

**The dependence conditions.** `g3_dep.wf` is `a2_bubble.wf` with the fold's
second child argument computed from the first child's result. The ledger goes
silent on that child pair — the intervening statements mean the two calls are
no longer adjacent, so no pair exists to judge — while it still reports the two
pairs in `main`: `pair(boxed_leaf, boxed_leaf) eligible` and
`pair(boxed_leaf, boxed_branch) condition 1: an argument of s2 uses the result
of s1`. Together those fix both halves of the dataflow question: what a genuine
def-use denial reads like, and that a dependence broken across statements
simply removes the candidate rather than denying it. (A third file, `g3_base.wf`,
was landed alongside these and was a byte-identical copy of `a2_bubble.wf`; the
batch audit found it and it was deleted rather than kept under a second name.)

**Divergence.** `x2_spin.wf` is the minimal non-terminating program — a loop
whose exit test can never hold — and it confirmed that the compiler emits a real
infinite loop rather than optimizing the divergence away.
`d1_closure_div.wf` is the shape that matters: a claim-free recursive fold whose
leaf helper diverges on the values the program actually passes. Its child pair
is `eligible`, so a lane is handed out and diverges. That is the design's
"divergence dissolves" argument made concrete — the join waits for all its
lanes, nothing observable is emitted mid-lane, and the overlapped execution
hangs exactly where the sequential one hangs. **Do not run this file, or
`x2_spin.wf`; both are non-terminating by construction.**

**Why claim-bearing regions are deferred.** `d1_two_traps.wf` is two sibling
calls whose claims are both false on the literal inputs `main` passes, so both
traps are live. Overlapping them raises the question the v1 design refuses to
answer — which trap's `[DIAG-3]` record is the program's — and the ledger's
answer is to decline: `not-actualizable: 2 claim sites via left`.

That file is **a deliberately unreviewed program**, and the batch audit was
right to flag it. Under the claim doctrine this design rests on (`../DESIGN.md`
section 0, ruling 1) a claim is an always-true lemma that cannot fail on an
admissible input, so a fully reviewed program cannot trap and this program
could not have passed review. It is kept because naming the deferred
arbitration question needs a program that actually reaches two live traps, and
because the trap-selection question is exactly the one the doctrine sets aside
rather than one it answers. It is not a model of ordinary usage, and its
`because` strings assert what the program falsifies two lines later. The ledger
line it produces — the decline — needs no false claim at all: `a2_bubble.wf`
produces the same `not-actualizable` verdict from claims that are true.

(`x2_spin.wf`'s `claim never: ieq(v, 7_u64) because "unreachable when spin
diverges"` is vacuously true because the claim is never reached. It reads as a
claim justified by unreachability rather than by a lemma, which is the same
shape of defect at a smaller scale; the file is kept only for the divergence
observation above.)
