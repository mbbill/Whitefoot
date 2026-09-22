# Result proof transport across source forms

This investigation asks why a verified result relation survives a direct call
match but not naming that result or forwarding its success with `propagate`.
The active specification remains authoritative. This study changes no language
rule, conformance expectation or compiler implementation, and selects no live
design-tree revision.

The consumer is ordinary library composition: perform one operation, keep its
outcome while doing unrelated work, then use the established success bound.
Keep this record with its reproducible examples while its comparison informs
that design question; supersede its guidance in place if the candidates change.

## Scope and comparison criteria

The requirements are the current single-owner and reference-validity rules,
verified callable contracts, erased evidence, deterministic terminating
checking without SMT or acceptance budgets, and no added runtime check, data
tag, allocation or scheduling edge. Existing restrictions on which relations
an `ensures` can express are a separate question.

Before running the baseline probes, use these discriminating criteria:

1. Compare a direct call match, an intervening `let`, an ordinary value copy or
   move, and direct/indirect `propagate` for the same verified `Ok` relation.
   Keep the producer, arguments and consuming obligation fixed. Record the
   actual rejection rule, not merely the exit status.
2. Separate publication at the caller from verification of a wrapper's own
   routed postcondition. A transport repair that still requires rebuilding
   the same `Ok` at every boundary must report that remaining limitation.
3. Overwriting a result, changing a supporting value or measure, merging
   unrelated outcomes and crossing loop iterations must not authorize stale
   or mismatched evidence. A candidate must specify capture, transport,
   activation, invalidation and joins together.
4. Compare a direct-`propagate` extension, a finite local result-evidence
   mechanism, and general refinements/dependent result types. State what each
   admits and leaves out; do not call a finite representation a practical
   complexity bound without accounting for joins and loops.

Baseline acceptance checks establish current behavior only. A proposed
extension needs its own implementation and checking-cost evidence before any
claim of performance or complete soundness.
