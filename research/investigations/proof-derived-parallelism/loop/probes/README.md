# Loop-permission design probes

Every verdict below is what the in-tree compiler reports for that file at
the commit that landed this directory; the DESIGN.md beside it states what
each probe decides.

- `p4_split_equiv.wf` — a 2^20-term `+wrap` left fold and the same terms as
  a recursive halving tree publish one byte sequence at every worker count:
  the wrap family recombines value-exactly. Positive control for the
  reduction law. Remove it when [PAR-2]'s normatively enumerated combine set
  is superseded, or when a conformance case carries the same value-exactness
  and this probe is no longer the only place it is written down.
- `p5_float_split.wf` — the identical shape under `fadd.strict`: the fold
  and the tree publish two different byte sequences (each internally stable
  at every worker count, because [PAR-1] never regroups a source-written
  tree). The reason the admitted operation set must be normative and
  float-free. Remove it when the float question is reopened — when any
  inexact operation is admitted to the combine set, at which point this file
  is measuring a rule the language no longer has.
- `p9_facts.wf` — an accumulator provably below a buffer's length still
  carries no entailment fact past the loop (the counted recurrence
  subtracts facts supported by continuing writes), so regrouping can
  falsify no surviving proof. [OP-4] still demands the claim; that denial
  is the probe's point. Remove it when [OP-4]'s obligation on a counted
  recurrence changes, or when the surviving-fact question is settled by a
  compiler test rather than by this observation.
- `x1_same_buffer.wf` — sibling element writes into one buffer through two
  `&uniq` borrows: legal source, denied by condition 2 — a resolved place
  carries no index segment, so `dst[i]` and `dst[j]` are one place. The
  granularity fact behind the map deferral. Remove it when the map
  deferral's re-entry condition is met and a resolved place carries an index
  segment: that is exactly when the fact this file records stops holding.
- `m1_pair_in_for.wf` — the 20-line reproducer for the two-world phi-label
  defect (invalid LLVM when an actualized pair sits in a loop body), fixed
  at `eabefcc8`; kept as the historical witness beside its regression test.
  Remove it when that regression test is retired, since it is kept only as
  the witness beside it.
- `r1_mandelbrot_for.wf` — `tests/programs/mandelbrot_grid.wf` with its two
  hand-counted `loop`s written as counted `for`s, plus one claim-free
  `to_float` helper the rewrite needs because the binder is `own u64`
  [TYPE-5] and `cvt<u64, f64>` is inexact and therefore affine. It exits 0,
  so its `ieq(escaped_points, 2437_u32)` claim holds exactly as the
  original's does, and the loop judgment permits **both** of its loops under
  `+wrap`. This is the demonstration that [PAR-2] reaches a program the
  project actually wrote, once that program is written in the form the
  language says is the default one; the corpus census in the 0078 record is
  the other half of the picture, and says the rule reaches none of the
  counted loops already written. Remove it when a loop-form mandelbrot lands
  in `tests/programs`, or when batch B's split measurement against the
  recursive oracle is closed.

- `r2_grid_loop_d21_w256.wf` — the bench family's `grid_d21_w256` with its
  recursive `tile` written as the counted `for` a writer reaches for, and
  everything else the same text. It is batch B's measurement subject: the
  loop form against the hand-written recursive twin it was rewritten from,
  and against `rayon`, on the standing oracle grid. All three publish
  `000000000033517d`, and at every worker count the loop form's ratio against
  the twin lies inside the measurement protocol's 0.83x-1.20x unresolved band
  (u) — the table is in the 0078 record, with the machine load it was taken
  at. "Reaches the twin's numbers" is that band and not an equality. The one thing it adds over
  the twin is a claim-free `narrow`, because the binder is `own u64` [TYPE-5]
  while `point_escapes` takes the `u32` the twin's recursion carries; that is
  one compare and one select against a 256-round orbit, and the residual it
  could explain was measured to be grain instead. Remove it when a loop-form
  program of this shape lands in `tests/programs`, or when the bench
  generator learns to emit the loop form directly.

The value-falsifier measurement table (map splits on memory-shaped vs
compute-heavy bodies) is in `VALUE.md`.
