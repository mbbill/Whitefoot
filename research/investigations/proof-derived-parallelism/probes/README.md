# Deciding probes for proof-derived parallelism

Three groups live here. The first nine sources decided the design questions
behind `../DESIGN.md` during the three research rounds; the nine in the second
group decided the findings in `../gap-hunt-findings.md`; the four in the third
carry the band/derived-index discharge asymmetry that Dig 9 of batch 0076
closed. All were written and run outside the repository, so all were dying
artifacts; they are landed here
because the design's, the gap hunt's, and that dig's load-bearing claims cite
them, and a claim whose evidence has evaporated is an assertion. They are **archived
evidence, not a gated corpus**: no build, test, or tool reads this directory,
they are not held to the canonical-form or must-compile rules that
`tests/programs/` enforces, and each is deleted when the decision or finding it
settles is superseded.

Every verdict quoted below is what the in-tree compiler's
`whitefootc --par-ledger` reports for that file today, except where a file is a
template that must be instantiated first — those say so and quote the
instantiated source. **Refreshed again 2026-08-23 after the claim redirect of
batch 0078-C**, which withdrew claim-free eligibility: the `not-actualizable`
verdict class no longer exists, and the seven probes that quoted it —
`a2_bubble.wf`, `a2r_layout_two.wf`, `d1_two_traps.wf`,
`d2_band_window_claim.wf`, `p1a.wf`, `p1b.wf`, `p7_dyn.wf` — now report
`eligible` with a two-member chain. Each paragraph below states today's verdict
and then what it used to say.

**Several of these probes were landed to record a decision that is now
reversed. Disposition, 2026-08-23: all seven are kept, under this file's own
standing rule that a probe is deleted when the decision or finding it settles
is superseded.** What the redirect superseded in each of the seven is the
`not-actualizable` *verdict line* the paragraph quoted, not the finding the
probe settles: `a2_bubble.wf` and `a2r_layout_two.wf` still carry the
uniq-tree fold and the realistic layout body the investigation was built on and
now demonstrate that nothing divides them from their claim-free twins;
`d1_two_traps.wf` is the only program in the repository that reaches two live
traps, which is the shape `backend/tests/trap_latch.rs` needs; `p1a.wf` and
`p1b.wf` are kept for Dig 8's adjacency fix, which the redirect did not touch;
`d2_band_window_claim.wf` is kept with its guarded twin until the
band/derived-index finding is superseded, which it is not. The one whose
*conclusion* was reversed is `p7_dyn.wf` — the design boundary it named is gone
— and it is kept because it remains the only carrier of the F6 shape and, now,
of the reversal. Each paragraph below states today's verdict and then what it
used to say, which is that record. Previously **refreshed 2026-08-22 after
the 0076/0077 batch audit**,
which found the warranty unmet on four entries: batch 0076's Dig 8 replaced the
adjacency rule and rewrote the denial wording, and Dig 9 widened the discharge,
so several probes now report verdicts their paragraphs described as impossible.
Where an entry changed, the paragraph states today's verdict and then what it
used to say, because the superseded reading is often the finding the probe was
landed to record.

## Retired by v0.34, and deliberately not rewritten

**Rebased onto `main` 2026-08-23.** `main` activated v0.34, which admits a
`claim` only as a current-function-local, non-derivable, load-bearing residual
carrying a five-field review record. Seventeen of the twenty-eight sources here
carry claims written before that rule and no longer compile:

`a2_bubble`, `a2r_layout`, `a2r_layout_two`, `d1_closure_div`, `d1_two_traps`,
`d2_band2_loop`, `d2_band_window_claim`, `d2_band_window_guard`,
`d2_branch_loop`, `d2_tree_zeroclaim`, `g2_propagate`, `g3_dep`, `p1a`, `p1b`,
`p7_dyn`, `x2_spin`, and `loop/probes/r1_mandelbrot_for`. Every one reports
`[CLM-1] InvalidClaimJustification` first; behind that, most carry a claim over
a user-call result, which v0.34 rejects as `NonLocalClaim`, and several are
result-verification claims — "did the fold produce 7?" — which v0.34 retired as
a genre in favour of ordinary control flow.

**They are not rewritten, and that is the disposition, not an omission.** These
are archived evidence: each was landed because a design decision or a finding
cites it, and the verdict quoted in its paragraph is the verdict *that source*
produced. Editing the source to satisfy a later rule would leave a file that no
longer produces the recorded verdict, which is the one thing an evidence
artifact must not do. Each is deleted when the decision it settles is
superseded — that is this directory's own rule, above — not migrated.

**None of this is rebase damage, and it is checked rather than asserted.** All
twenty-eight sources were compiled by the rebased compiler and by `main`'s own
compiler at `18d332e7`, and every verdict is identical between them: the same
seventeen `[CLM-1]`, the same nine clean accepts (`bt`, `m1_pair_in_for`,
`p4_split_equiv`, `p5_float_split`, `p6`, `q4`, `r2_grid_loop_d21_w256`,
`x1_same_buffer`, `zero_elig`), and the same two pre-existing conditions
(`min_stack` is a `sed` template and reports `[FORM-1]`; `p9_facts` is the
deliberate undischarged-obligation demonstration and reports `[OP-4]`). The
rebase moved no probe's verdict.

**Where Dig 9's detection lives now.** The four band/derived-index probes were
this dig's detector, and a detector that cannot be run is not one. That
coverage is a compiler test rather than a file here:
`semantic::tests::boolean_composition::band_conjunct_over_a_derived_binding_discharges_like_the_single_bound_pair`
uses branch guards instead of claims, so it needs no claim at all, and it was
checked against `main`'s compiler — which carries v0.34 but not the Dig 9 fix —
where the conjoined half fails `[OP-4]` on `next < len(deref(input))` while the
nested single-bound half still discharges. That is the asymmetry the probes
recorded, kept where it runs on every gate.

## The design-round probes

**The line that no longer divides them.** `a2_bubble.wf` is the two-child
`uniq` tree fold that started the whole investigation. Its recursive child pair
reports `pair(bubble, bubble)  eligible` with a two-member chain; **until
2026-08-23 it reported `not-actualizable: 1 claim site via bubble`**, and that
line was the demonstration the whole first group was built around.
`d2_tree_zeroclaim.wf` is the same fold with its two overflow claims rewritten
as ordinary branches, and it reported `eligible` then and reports `eligible`
now — so the pair, which was the cleanest demonstration that a claim was what
stood between a permitted pair and an actualized one, is now the cleanest
demonstration that nothing does. `a2r_layout.wf` and `a2r_layout_two.wf` carried
that contrast into a realistic body (a box-tree layout with a per-node float
cascade and a word loop): both are `eligible` today, and the two-table version,
whose second buffer forces a derived index and therefore two claims,
**previously reported `not-actualizable: 2 claim sites via measure_words`**. `a2r_layout.wf` is the direct ancestor of
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
second child argument computed from the first child's result. Today the ledger
judges that child pair and denies it —
`g3_dep.wf:29 pair(bubble, bubble_shift) condition 4: the trap edge of
interposed statement 2 skips s2` — and it reports the two pairs in `main`:
`pair(boxed_leaf, boxed_leaf) eligible` with a two-member chain, and
`pair(boxed_leaf, boxed_branch) condition 1: the operands of s2 read what s1
defines`. That fixes what a genuine def-use denial reads like. *What this probe
originally settled, and what changed:* until Dig 8 of batch 0076 (`974d5513`)
the child pair produced **no line at all**, because the judgment grew a
candidate group only from consecutive `let x = f(...)` statements, and the probe
was the evidence that a dependence broken across statements removed the
candidate rather than denying it. Dig 8 replaced that with a window: an
interposed statement is now judged rather than ending the enumeration, so the
pair gets a verdict and a reason. The denial wording changed in the same
commit. (A third file, `g3_base.wf`,
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

**The question this probe posed, now answered.** `d1_two_traps.wf` is two
sibling calls whose claims are both false on the literal inputs `main` passes,
so both traps are live. Overlapping them raises the question the v1 design
refused to answer — which trap's `[DIAG-3]` record is the program's — and the
ledger's answer was to decline: `not-actualizable: 2 claim sites via left`.
**Since 2026-08-23 the ledger reports `pair(left, right)  eligible` and the
question is answered rather than declined**: the record is *a* claim whose
predicate evaluated false, which one may depend on the schedule, and exactly
one record is written because `wf_trap` latches. The shape this file carries is
the shape `compiler/src/backend/tests/trap_latch.rs` now exercises forty times
a run at four workers.

That file is **a deliberately unreviewed program**, and the batch audit was
right to flag it. Under the claim doctrine this design rests on (`../DESIGN.md`
section 0, ruling 1) a claim is an always-true lemma that cannot fail on an
admissible input, so a fully reviewed program cannot trap and this program
could not have passed review. It is kept because naming the deferred
arbitration question needs a program that actually reaches two live traps, and
because the trap-selection question is exactly the one the doctrine sets aside
rather than one it answers. It is not a model of ordinary usage, and its
`because` strings assert what the program falsifies two lines later. The ledger
line it produced — the decline — needed no false claim at all: `a2_bubble.wf`
produced the same `not-actualizable` verdict from claims that are true. With
the decline withdrawn, what this file uniquely carries is a program that
actually reaches two live traps, which is what the latch's own control needs.

(`x2_spin.wf`'s `claim never: ieq(v, 7_u64) because "unreachable when spin
diverges"` is vacuously true because the claim is never reached. It reads as a
claim justified by unreachability rather than by a lemma, which is the same
shape of defect at a smaller scale; the file is kept only for the divergence
observation above.)

## The gap-hunt probes (2026-08-21)

These nine settle the findings recorded in `../gap-hunt-findings.md`. The
verdicts quoted are what the in-tree compiler's `whitefootc --par-ledger`
reports for each file today. Two of them are templates, not compilable sources:
instantiate the placeholder first.

**F1, the hand-out frame's stack tax.** `min_stack.wf` is the minimal carrier:
one deep `spine` recursion whose single eligible pair is `pair(spine, leafval)`.
It is a **template and does not compile as it stands** — the bare `DEPTH`
placeholder is a `FORM-1 UnexpectedByte` — so substitute first
(`sed 's/DEPTH/140000/' min_stack.wf > ms.wf`), and the instantiated source
reports `pair(spine, leafval) eligible` with a two-member chain. It settles the
finding in its purest form: the sequential module contains no alloca at all,
`--par` adds one hand-out frame per function with a pair, and the arm64
prologue grows from 16 to 64 bytes plus two FP spills. *The fourfold ceiling
loss this paragraph reported with the pool off was fixed by Dig 1 of batch
0076*, which moved the frame to the lane; the pool-*off* ceiling now matches the
sequential build's. The probe still sweeps a ceiling, but the ceiling it finds
today is the pool-**on** one, which at an ordinary 8 MB stack is roughly a
third of the sequential build's — see the flagged default-behavior entry in
`docs/ongoing/0077-night-par-ceiling.md`. `bt_skew.tmpl` carries the
same finding on a realistic left-spine `BNode` fold (placeholder `SKEWDEPTH`);
it is what shows the loss is 23%, not 4x, on a real tree, and it produces the
case that crashes pool-off and survives with a lane.

**F3, adjacency brittleness and its cost.** `p1a.wf` and `p1b.wf` are the pair
that makes the finding undeniable, because they are the same program twice.
`p1a.wf` puts one `fmul.strict` builtin between the two recursive `layout`
calls; `p1b.wf` puts the identical arithmetic there wrapped in a `pure` user
function. **Dig 8 of batch 0076 (`974d5513`) closed the gap these two were
landed to demonstrate, and their ledgers now agree.** Today `p1a.wf` reports
`p1a.wf:116 pair(layout, layout) eligible` with a two-member chain through line
118, and `p1b.wf` reports `pair(layout, scale_up) eligible`,
`pair(scale_up, layout) eligible`, and a three-member chain. Dig 8 measured them
at parity (0.3984 s against 0.4049 s). *What the finding was:* until that
commit `p1a.wf` reported **no `layout` pair at all** — the candidate was not
denied, it ceased to exist, because `analyze_block` grew a group only from
consecutive `let x = f(...)` statements — while `p1b.wf` reported both pairs, so
the same program was 1.41x apart at four lanes on nothing but where the writer
put one line. That is the brittleness the judgment's window rule removed; the
pair is kept because the fix is only legible beside the shape that motivated it.
Both files also carry the `layout_banded` fold, which reports
`eligible` today and **reported `not-actualizable: 1 claim site via
measure_band` until 2026-08-23** — the tie back to the claim line the first
group established.

**F2, grain.** `q4.wf` is the quad tree that produced the fine-grain
catastrophe: six eligible pairs, three adjacent in `build4` and three in
`qfold`, and about 65.5M offers. As landed it fell to **48.6x slower** than one
lane at 64 workers, non-monotone in between. *Dig 2 of batch 0076 dissolved
that*: the same cell now runs **1.99x faster** than one lane (0.2530 s against
0.5026 s), so the catastrophe is history and the probe is kept as the carrier of
the shape, not of the number. It is also the N-ary carrier — a run of four
adjacent calls handing out N−1 — and therefore the file behind F4a, the ledger
printing three unrelated-looking pairs where the checker built one run of four.

**Nested hand-out actually works.** `bt.wf` is the one probe here that carries a
positive result, and it is the reason the whole scheduler line of digs is worth
pursuing rather than abandoning. A depth-16 binary `BNode` fold, three eligible
pairs (`pair(build_bt, build_bt)`, `pair(build_skew, boxed_bleaf)`,
`pair(bfold, bfold)`), it offers a hand-out at all sixteen nested levels. At
eight workers it burns about 1.75 s of CPU in 0.41 s of wall time — roughly 4.3
concurrent streams, which a binary fold cannot reach unless the workers
themselves fork and join. So lanes recursing and handing out further is
demonstrated, sixteen levels deep, byte-identical at every worker count tried up
to 64, with no deadlock and no livelock. It is also the fine-grain half of F2's
discriminator: the same structure as `base.wf` but roughly 24 flops per node
instead of an 8192-word scan, which is what isolates grain rather than thread
count as the cause. Keep it: it is the sole source of the nested-hand-out
evidence, and rebuilding it from prose would be rebuilding the argument.

**Denial accuracy.** `p6.wf` is the denial exhaustive: it reaches all five
refusal conditions in one file, including all four condition-2 sub-kinds, and
the caller-side operand read that is the subtle one. Reaching conditions 3 and 4
required unmasking earlier refusals first, which is why the file is shaped the
way it is. Its three `eligible` lines exist to prove the denials are selective
rather than a blanket refusal.

**F6, the design limit on dynamic allocation.** `p7_dyn.wf` is four lines of
consequence: a buffer sized by a parameter raises an undischarged `buffer_fits`
obligation, and the only discharge is a `claim`. **Until 2026-08-23 that claim
in the closure made the pair `not-actualizable: 1 claim site via mkbuf_dyn`;
today it reports `eligible`.** Dropping the claim does not compile. What it
settled — that the natural shape of a per-node style-resolve phase is
permitted and permanently out of reach in v1 — **is reversed**: the shape is
permitted and reachable, and the design boundary it named is gone.

**Linkage.** `zero_elig.wf` is `min_stack.wf` with its one pair broken by an
interposed builtin. **It no longer has zero eligible sites, and the linkage
result it settled has to be read from a different file now.** Today it reports
`zero_elig.wf:12 pair(spine, leafval) eligible` with a two-member chain, and
`--par --emit-llvm` on it contains 16 `wf__par` references — Dig 8's window rule
judges the interposed builtin instead of ending the enumeration, so breaking a
pair by interposition is no longer something a source can do. *What it
settled:* that zero eligible sites really means zero cost — no `wf__par` symbol
at all in the IR (the check is on that reserved prefix rather than on any one
entry point's spelling), no undefined `pthread` symbol in the binary, and a byte
size identical to the sequential build — and it was the control for F4c, since
an empty ledger exiting 0 is indistinguishable from a flag that silently did
nothing. That control is now vacant: no probe here produces an empty ledger,
and re-establishing it needs a source with no adjacent call pair at all rather
than one whose pair is interposed.

## The band/derived-index probes

**The asymmetry itself.** `d2_band2_loop.wf` is the file the Dig 9 queue entry
cited, and at the time it was cited it did not compile: a `band` of two bounds
is claimed, `i` discharges, and `j = i +wrap 1` comes back
`UndischargedBoundsObligation { residual: "j < len(deref(input))" }` — while
the same two bounds claimed *separately* both discharge. `d2_branch_loop.wf`
is the same body with the claim replaced by an `if both { .. } else { break }`
guard, which failed identically, and which matters more than the claim form
because a function with no claim site is the only kind a caller's pair can
actualize. Both compile now. Keep them: they are the reproduction, and the
loop they are written in turned out **not** to be the trigger — the fault is
over let-bound derived terms and reproduces in straight-line code — so a
reader who only has the fix will not otherwise see the shape that was
reported.

**What the fix buys, in two files.** `d2_band_window_guard.wf` and
`d2_band_window_claim.wf` are the same two adjacent reads with the same
output, differing only in how the pair of bounds is admitted. The guarded
version holds no claim site and the ledger reports
`pair(window, window)  eligible` with a two-member chain; **the claimed version
now reports the same, where until 2026-08-23 it reported
`not-actualizable: 1 claim site via window`** — so the two files differ in what
they had to prove and no longer in what they get.
Before Dig 9 **neither compiled**, which is the point worth keeping: the
eligible shape was not merely unrewarded, it was unwritable, and closing a
checker gap made it reachable without a spec byte or a source edit. They are
deleted when the band/derived-index finding is superseded.
