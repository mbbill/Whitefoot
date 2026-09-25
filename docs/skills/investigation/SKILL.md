---
name: investigation
description: Plan, run and report a Whitefoot investigation or experiment - a measurement, benchmark, performance attribution or agent writer trial - so its result can select or reject a design. Use when starting or extending research/investigations/<name>/ or research/experiments/, or when a claim needs a new measurement. Not for routine test runs, reading existing research or reporting a CI result.
---

# Investigation

An investigation exists to decide something. Its design, measurements and
rejected alternatives live in `research/investigations/<name>/`, reproducible
experiments in `research/experiments/`, and the decision that survives goes to
the design tree through the `design-tree` skill. AGENTS.md's verification
rules apply throughout; these add what experiments need.

1. **Question first.** Before running anything, state the question, the live
   alternatives, the result that would distinguish them and when you will
   stop; then run the cheapest probe that can produce that result. Keep
   behavior, contracts, workloads and comparison conditions fixed where they
   define the question; a probe that makes the task easier answers a
   different question.
2. **Performance.** Attribute a loss with a same-source causal comparison,
   before and after the change, and a falsifier. Profile proof cost by
   formation, automatic derivation, certificate checking and fact propagation
   before attributing it to `use`. When diagnostic order or cited rules may
   move, compare every affected case's result and rule across both binaries;
   an unchanged failure set is not enough.
3. **Agent writer trials.** Keep four observations apart: whether the program
   and its proofs can be expressed; whether the tested agent writes them with
   the supplied interfaces, context, tools and repair help; whether separately
   written parts meet independent expectations when composed; and whether the
   result meets its runtime cost goal, and why. Attribute a failure to
   contracts, proof vocabulary, feedback, model limits or architecture before
   selecting a language change. Model identity and assistance are experimental
   conditions, not language ceilings, and a restriction can be worth extra
   writing effort: measure both sides.
4. **Report.** Give exact commands, inputs, outputs, counts and exit codes.
   Label what is unverified; if a probe did not isolate the hypothesis, write
   `not measured`. Keep conclusions conditional: name the compiler and workload
   a result holds for, and what would reopen a rejected alternative. A result
   on a retired compiler remains evidence about those conditions until
   reproduced; better agents can change an authoring-cost result, never a
   soundness counterexample.
5. **Boundary.** Research stays outside `make check` and daily CI. Extract a
   useful case, with its inputs and oracle, into formal tests; formal checks
   never import research, and `make static` rejects literal references to it.
