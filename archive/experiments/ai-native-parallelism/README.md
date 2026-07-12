# AI-Native Parallelism Investigation

Status: investigation started; no verdict yet.

This track replaces the old "compiler discovers parallelism from ordinary
sequential code" claim with a different first-principles thesis:

> If AI writes most source, the unit of authorship can be bigger than source
> code. The AI can emit source, candidate parallel plans, proof obligations,
> runtime guards, tests, and benchmark hooks. The compiler should verify those
> artifacts, reject unproved claims, measure candidates, and keep only safe
> winners.

That is not DPJ. DPJ asked humans to maintain heavy region/effect annotations.
This investigation asks whether an AI can cheaply generate extra artifacts that
humans would not maintain, while the compiler trusts only verified facts.

## Pipeline Hypothesis

Old pipeline:

```
human source -> compiler inference -> binary
```

AI-native pipeline:

```
source bundle -> verifier -> candidate lowering -> measurement -> selected binary
```

The source bundle may include:

- a simple sequential specification,
- one or more optimized candidate implementations,
- claimed facts such as purity, disjointness, injectivity, associativity, bounds,
  layout, and work estimates,
- proof sketches or references to verifier-checkable obligations,
- runtime guards for facts that depend on input values,
- benchmark fixtures and thresholds,
- repair metadata from prior failed attempts.

The trust rule is strict: AI-authored facts are untrusted until verified. If a
fact cannot be statically proved, the compiler may generate a runtime guard and
fall back to the sequential implementation when the guard fails.

## What This Could Unlock

The important shift is that the compiler no longer has to infer the parallel
plan from scratch. The AI proposes a plan; the compiler verifies safety and
measures profitability.

Candidate plan classes:

1. **Disjoint parallel map**: per-iteration writes are to non-overlapping output
   locations.
2. **Injective scatter/gather**: a runtime or static proof establishes that an
   index map is injective or bijective over the active range.
3. **Reductions**: the plan states the reduction tree and the algebraic law that
   makes reassociation valid under the declared numeric mode.
4. **Task graphs**: functions with disjoint region/effect footprints are issued
   as tasks; the compiler/runtime schedules verified non-interfering nodes.
5. **Multi-version kernels**: sequential, SIMD, parallel, tiled, and
   scatter-specialized versions coexist; selection is based on verified guards
   and measured thresholds.

## First Research Questions

1. Can a compact plan-bundle format express the known structural scatter win
   without importing DPJ-scale annotation burden?
2. Can the safety obligations be verified deterministically or guarded at
   runtime without erasing the performance win?
3. Can weak/middle AI models generate valid plans more reliably than they can
   directly write expert Rust/Rayon/unsafe code?
4. Does the bundle make low-performance code harder to ship because the compiler
   rejects unsupported fast claims instead of silently accepting slow shapes?
5. Does the approach generalize beyond injective scatter into reductions,
   stencils, and task graphs, or is it only a narrow DSL opportunity?

## Initial Experiment Set

### E0: Bundle Schema

Define a minimal machine-readable bundle schema. Current draft:
`parallel-plan-bundle.schema.json`.

The schema is intentionally small. It does not need to encode a full proof
language yet; it only needs enough structure to separate trusted compiler checks
from untrusted AI claims.

### E1: Manual Scatter Bundle

Recast the existing parallel-scatter benchmark as:

- sequential spec: `out[perm[i]] = f(input[i])`,
- candidate plan: parallel chunks over `i`,
- fact: `perm` is bijective over `0..n`,
- runtime guard: verify permutation once, then dispatch to direct parallel
  scatter; otherwise fall back,
- baselines: best safe Rust range-ownership, unsafe Rust ceiling, current xlang
  proxy ceiling.

Decision value: this tests whether the strongest existing structural win remains
real after paying for fact verification.

### E2: Candidate-Set Autotuning

For the same task, emit several verified candidates:

- sequential,
- safe range-owned parallel,
- guarded direct scatter,
- tiled or bucketed scatter.

Measure whether a simple local benchmark selector can choose the right candidate
without a sophisticated static cost model.

Decision value: this tests whether the cost-model problem can move out of the
compiler and into measurement without becoming unmanageable.

### E3: AI Plan Generation

Use the M3-style fixed-budget protocol:

- ask each model tier for source plus plan bundle,
- run verifier and harness,
- feed back only machine diagnostics,
- count valid-plan rate, correct-fast rate, repair turns, plan size, and
  bypass attempts.

Decision value: this is the actual AI-native claim. It is not proven by manual
bundles.

### E4: Reduction And Floating-Point Law Probe

Test whether models can correctly state and preserve reduction laws:

- integer associative reduction under wrapping mode,
- integer checked reduction where overflow changes legality,
- floating-point strict mode where reassociation is illegal unless an explicit
  approximate mode exists.

Decision value: reductions are where "parallel by plan" can easily become
unsound. This probe should run before any broad parallelism claim.

## Pass Criteria

This track becomes project-priority only if all of these hold:

1. The manual scatter bundle preserves a meaningful residual after verification
   overhead, ideally at least 20 percent on memory-bound scatter.
2. At least one non-scatter plan class passes with checked obligations and a
   measurable performance win.
3. Model-generated bundles beat Rust/Rayon submissions on correct-fast rate or
   eliminate unsafe requirements on a task where safe Rust has a known ceiling.
4. The plan/proof artifact is small enough that it does not recreate DPJ's
   annotation-burden failure in AI form.
5. The verifier never trusts unchecked AI claims; unsupported claims produce
   rejection or guarded fallback, not silent unsound speedups.

## Stop Conditions

Stop or narrow to a DSL if any of these happen:

- Fact verification costs erase the scatter win.
- Useful plans require unchecked commutativity/injectivity assertions.
- AI-generated bundles are mostly invalid or too large to fit practical prompt
  budgets.
- Rust/Rayon plus a small helper library matches the verified plan results.
- The only surviving case is injective scatter; then this is a narrow numeric
  DSL feature, not a general-purpose language reason.

## Current Priority

This is now the highest-upside branch, but not yet the most proven branch.

Immediate priority:

1. Build E0 and E1 as a local paper+schema experiment.
2. Reuse existing scatter evidence to estimate the runtime-guard budget.
3. Add one M3 task that asks for a source+plan bundle instead of only source.
4. Only then decide whether AI-native parallelism outranks the existing
   "hard to write slow code" and AI-codegen tracks.

