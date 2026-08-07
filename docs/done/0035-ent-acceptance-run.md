# 0035 — ENT acceptance run

This is a temporary live coordination record, not execution authority.

- **Status:** `DONE` (2026-08-07: acceptance measured — utf8parse and sha256 held, deflate diverged 5/29 vs 17/30 predicted; ENT-5 loop rule isolated as dominant cause; 20 conformance cases added, annotations 36->30)
- **Authority:** `ACTIVE` docs/current-plan.md selected slice; batch-1/v0.22 candidates and rulings in governance/
- **Owner / workspace:** exec-0035 / /Users/bytedance/do_not_scan/wf-0035 (branch task/0035-ent-acceptance-run)
- **Base revision:** 9fa3d6d
- **Dependency:** 0034

## Goal

The preregistered acceptance criterion: the real checker reproduces SIMULATION.md's proven/claim/branch buckets on utf8parse, the deflate-dynamic unit, and sha256_abc, with one-line residuals; add conformance cases covering CLM-1/2 and each ENT rule (positive and negative); full gates green. Divergence from the simulation stops the task with evidence — the buckets are the falsifier, not a target to tune toward. Dependency: 0034 terminal.

## Progress

- Completed: the acceptance measurement, recorded in
  `research/investigations/obligation-discharge/ACCEPTANCE.md`. utf8parse and
  sha256 held (sha256 one claim over, because a two-sided bound is not
  expressible as one claim); the deflate-dynamic unit diverged — 5 of 29 sites
  proven against 17 of 30 predicted, 21 claims against ~8. Measured through
  the compiler's retained `FunctionEntailment` summaries via the test-only
  dark checker, with claim facts blinded by a `band` rewrite that preserves
  the statement, its check, and its effect row. All four drivers 0034
  attributed were re-checked against concrete sites and confirmed, one with a
  correction; a fifth and dominant driver was found and isolated.
- Completed: twenty conformance cases covering CLM-1 (negative), CLM-2
  (redundancy accept, refutation reject), and ENT-1 through ENT-5 positive and
  negative; the six `compiler-entailment-tests` annotations are deleted.
  Coverage holds at 128/128 with 30 annotation-covered rules, down from 36.
  Adapter lane 386/1/14 — the twenty new cases pass; the single failure is the
  pre-existing `own1-neg-match-move-through-borrow`.
- Current: reported to lead. Both gates exit 0.
- Next: lead review. Two findings need an owner or lead decision and are not
  acted on here: the [ENT-5] loop rule's treatment of scope-leaving edges, and
  the three canonical-Huffman sites the migration made claims rather than the
  `Err` branches the design predicted.

## Findings that need a decision, not an executor

- **The loop rule discards every pre-loop fact in any loop containing a
  `return`.** [ENT-5](d) makes a scope-leaving edge a kill event, and the loop
  rule removes any fact a kill event anywhere in the body may kill; a `return`
  leaves the scope of every binding. Isolated by moving one statement:
  identical programs discharge with the `return` outside the loop and reject
  with it inside. This is spec-conformant on a literal reading and accounts
  for most of the deflate divergence. A `return`, `break`, or `propagate`
  error edge never reaches the next iteration head, so excluding those edges
  from the loop rule's kill scan looks like an [ENT-1]-monotone strengthening.
  Not proposed here; the conformance corpus pins only the uncontroversial
  `set`-inside-loop form.
- **Claims silently replaced the predicted `Err` branches.** SIMULATION's
  three permanent L3 branch regions all exist and all three are claims, so
  they abort where the design promised a recoverable `InvalidHuffmanCode`
  path. The 0034 migration added no branch restructuring anywhere.
