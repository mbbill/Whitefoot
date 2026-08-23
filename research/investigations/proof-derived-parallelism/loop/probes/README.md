# Loop-permission design probes

Every verdict below is what the in-tree compiler reports for that file at
the commit that landed this directory; the DESIGN.md beside it states what
each probe decides.

- `p4_split_equiv.wf` — a 2^20-term `+wrap` left fold and the same terms as
  a recursive halving tree publish one byte sequence at every worker count:
  the wrap family recombines value-exactly. Positive control for the
  reduction law.
- `p5_float_split.wf` — the identical shape under `fadd.strict`: the fold
  and the tree publish two different byte sequences (each internally stable
  at every worker count, because [PAR-1] never regroups a source-written
  tree). The reason the admitted operation set must be normative and
  float-free.
- `p9_facts.wf` — an accumulator provably below a buffer's length still
  carries no entailment fact past the loop (the counted recurrence
  subtracts facts supported by continuing writes), so regrouping can
  falsify no surviving proof. [OP-4] still demands the claim; that denial
  is the probe's point.
- `x1_same_buffer.wf` — sibling element writes into one buffer through two
  `&uniq` borrows: legal source, denied by condition 2 — a resolved place
  carries no index segment, so `dst[i]` and `dst[j]` are one place. The
  granularity fact behind the map deferral.
- `m1_pair_in_for.wf` — the 20-line reproducer for the two-world phi-label
  defect (invalid LLVM when an actualized pair sits in a loop body), fixed
  at `eabefcc8`; kept as the historical witness beside its regression test.

The value-falsifier measurement table (map splits on memory-shaped vs
compute-heavy bodies) is in `VALUE.md`.
