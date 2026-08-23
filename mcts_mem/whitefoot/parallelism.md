- Parallel execution is admitted by a compiler permission judgment derived from the proofs acceptance already computes (resolved places, effect rows, the overlap relation, the call graph); no source construct declares, requests, or gates it.
- Permission is never an obligation: actualization is a runtime choice, invisible in source, and published bytes are identical under every schedule and worker count.
- The permission judgment never consults optimizer fact state; facts-on and facts-off builds produce one permission table.
- Permission suffices to actualize; the schedule-unobservability guarantee is conditional on contract compliance, and an erroneous execution (a false executed claim) yields exactly one well-formed trap record whose identity a schedule may select.
- Overlap-capable compilation is a compile-time opt-in (`--par`); the default build's output is byte-identical to a build with no parallel machinery at all.
- A developer diagnostic channel reports a verdict for every judged site, the denial's judging condition, and a split hint for counted loops reducing under exactly-associative integer operations.

## Facts

- 2026-07-27 rationale: automatic discovery of parallelism was rejected as a direction before this node existed — the language removes the soundness half of auto-parallelization and none of the decision half, and granularity, not legality, was the binding constraint in 26 years of prior systems. (sourced)
- 2026-08-20 rationale: the owner's doctrine set the frame this node implements — permission before actualization, resources never language concepts, optional keywords never gating legality (the claim analogy), profitability decided at runtime and invisible in code. (sourced)
- 2026-08-21 measurement: with eligibility limited to claim-free closures, the trap-arbitration question dissolves — a fully reviewed program cannot trap, so no arbitration machinery, parked lane, or coordinator is built. (sourced)
- 2026-08-22 measurement: end state of the first optimization campaign on the paired oracle (N=18, 3276 runs, byte-identical within and across languages): rayon wins zero cells; matched worker counts 14 Whitefoot wins / 25 parity / 0 losses; shipped defaults 11 / 2 / 0; per-fork excess 5.39 ns against rayon's 5.13 ns in the same pass. Recorded in the batch 0075/0076 records and `research/investigations/proof-derived-parallelism/bench/baseline-20260822/`. (sourced)
- 2026-08-22 measurement: the machine ceiling measured by N independent sequential processes is ~1.9x at 2, ~3.7-3.9x at 4 and only ~4.7-5.7x at 8 on the 4P+6E development machine; 16 of 30 parity cells sit at or above 92% of that ceiling, and the fifth-worker turnover afflicts every scheduler measured, rayon worst. (sourced)
