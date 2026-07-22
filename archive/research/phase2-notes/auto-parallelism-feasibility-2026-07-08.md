# Can XL effect+region give "write sequential, get safe automatic parallelism" that beats Rust? (2026-07-08)

**Research + adversarial feasibility (4 prior-art survey clusters → verdict). Verdict: NO on the
strong claim; narrow PARTIAL yes on a scoped feature; do NOT bet the project on auto-parallelism.**

## The findings
- **Regions + effect rows prove TASK-level independence** (Bernstein's conditions over regions,
  from signatures alone) — real, better than C's may-alias fog, same as DPJ/Legion had.
- **But region granularity is too coarse for the volume case.** A data-parallel loop
  `for i: out[perm[i]] = f(in[i])` is "writes('out)" every iteration → one region → the effect
  system says *conflict*. The fact that makes it parallel (`perm` injective ⇒ element footprints
  disjoint) is **not expressible in XL's effect vocabulary** — it needs index-parameterized
  regions / injectivity refinement, i.e. exactly the fine machinery that broke DPJ's annotation
  budget. **XL's own lone measured structural win (disjoint scatter, 1.1–1.5×) is NOT provable
  from regions+effects alone.**
- **Missing beyond the type system:** a cost model (absent — independence ≠ profitability; no type
  system solves granularity; HELIX shows even a perfect disjointness oracle exposes only modest
  parallelism in general code because it's genuinely sequential); a work-stealing runtime
  scheduler; element-level injectivity; reduction-commutativity (DPJ's was unchecked/trust-based).

## The four traps vs XL
1. **Annotation:** XL escapes DPJ's *coarse* tax (regions are already mandatory for D1 safety, so
   task-level proof rides free) but **inherits DPJ's fine tax** the moment it wants loop-level
   parallelism (where the volume is). Survival lever: effect rows inferred intraprocedurally,
   written only at module boundaries — else it's DPJ's effect summaries = death.
2. **Granularity/cost model:** kills the strong claim outright; the type system contributes nothing.
3. **Determinism:** cheap in the data-parallel domain, not a differentiator; fights peak only at
   the fast-reduction edge (needs a checked commutativity story XL doesn't have).
4. **Adoption:** brutal & consistent — low-ceremony + runtime scheduler won (Cilk→TBB/OpenMP/rayon);
   sound region+effect determinism (DPJ, sound polyhedral) is off-by-default or dead. DPJ is XL's
   near-twin (sound, deterministic, inference-assisted) and got ~zero uptake; creator (Bocchino,
   WoDet 2013) disavowed the effect-annotation layer as "the wrong layer." rayon `.par_iter()` =
   one token, already TIES XL's ceiling (prior study).

## Strong vs weak
The honest capability is the **weak** version: XL automates the **safety** of programmer-exposed
parallelism (removes Send/Sync/Arc/Mutex/`unsafe` ceremony), NOT the **discovery** of parallelism.
Real Rust win only on the **injective disjoint-scatter band** safe Rust's borrow checker can't
prove without `unsafe` — ~1.1–1.5× memory-bound + the no-ceremony ergonomic delta. The ISPC/Futhark
win shape; durable but narrow.

## Recommendation
Do not re-aim the project around auto-parallelism (betting on the space's longest failure track
record). KEEP the scoped feature, framed honestly: *"XL makes the naive data-parallel shape safe,
deterministic, and parallel without `unsafe`/Send-Sync ceremony,"* positioned vs rayon on the
injective-scatter band — not vs a mythical auto-parallelizer.

**Cheap decisive falsifier (1–2 days, no code):** hand-simulate a Bernstein-over-effect-rows
auto-par pass on ONE real mid-size sequential XL-shaped program (parser / interpreter loop / small
sim); count (1) parallelizable sites effect-rows-alone expose, (2) profitable ones, (3) ones needing
injectivity beyond current rows. Prediction: ~0 profitable human-non-obvious sites in straight-line
code; everything valuable in (3). If so, the strong thesis is dead on paper for two days' cost.
