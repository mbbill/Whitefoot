- The judged unit is a window: an ordered pair of let-bound calls in one block together with every statement between them, with all conditions quantified over the interposed statements (`judge`).
- Four conditions, each necessary: no dataflow from the first call's result into the second's operands; disjoint write/read footprints under the acceptance overlap relation, projected through call boundaries and including caller-side operand evaluation in both directions; no external or blocking effect row in either closure; no exit edge of the first call's span that bypasses the second.
- Interposed statement forms are classified by an exhaustive match; an unclassified form denies rather than contributing an empty footprint, and the denial is reported.
- Footprint questions the judgment cannot resolve deny; permission fails closed.
- A permitted window is eligible to actualize; no claim-freedom gate exists, and a claim written between the window's calls still denies as an exit-bearing interposed form.

## Facts

- 2026-08-21 (e06e6da4) pitfall: the caller-side operand reads of the handed-out call participate in the footprint in both directions; omitting them admitted a race the audit's counterexample exposed. (code)
- 2026-08-21 measurement: the g2-propagate shape — an exit edge of the first call bypassing the second — compiles and would change even whether a cell was written; it is the counterexample behind the fourth condition. (sourced)
- 2026-08-22 (0942ee24) statement: a `band` claim whose conjunct subject is a let-bound derived value discharges by reading through the unprojected boolean leaf to its proving binding; before that fix the projection silently dropped the fact and the equivalent pair of single-bound claims diverged from the band form. (code)

## Moves

- 2026-08-21 (974d5513) replaced [[adjacent-pair-enumeration]]: one ordinary statement between two calls ended the candidate group, so permission turned on statement adjacency rather than semantics — two byte-identical-output programs differed 1.9x in wall time; the window judges the pair plus every interposed statement with all four conditions quantified over them. (sourced)
- 2026-08-21 replaced [[claim-count-eligibility]]: a claim is an always-true reviewed lemma, not an assertion — a fully reviewed program cannot trap, so trap-ordering machinery guards a case that indicates a review defect rather than a language obligation; eligibility is claim-freedom of the transitive closure. (sourced)
- 2026-08-23 (f6c55a9d) replaced [[claim-free-eligibility]]: a false executed claim is the sole writer-reachable contract violation, so an execution reaching one is erroneous and the program defective; refusing to overlap correct programs to keep a defective execution's trap record stable is the wrong side of the trade — the schedule guarantee becomes conditional on contract compliance, a process-wide latch keeps the record singular and well-formed, and the sequential world reproduces it deterministically. (sourced)
