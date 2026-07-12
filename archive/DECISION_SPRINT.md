# xlang Decision Sprint

Status: Phase B COMPLETE (2026-07-09): all three un-Rust-able fact channels built
and measured with differentiated deltas (see decision-gates.md). Phase C (model-tier
sprint) is the remaining evidence leg; it needs real weak/middle/strong model runs.

Purpose: decide whether xlang should proceed beyond the research prototype into the
self-hosting compiler track. This sprint treats the current negative performance
audit as evidence, not as a final verdict. The only survivor worth testing is the
distributional AI-codegen claim:

> xlang makes AI-written systems code land closer to the correct, safe, fast shape
> than Rust does, especially for weak models.

If this sprint does not show that advantage, the project should stop or pivot.

## Decision

Run one bounded M3 validation sprint before committing to the compiler bootstrap
milestones in `compiler/PLAN.md`.

Do not start full self-hosting work until the sprint result is known. The current
compiler plan remains useful as the implementation path if the sprint passes, but
it is too expensive to run as an act of faith.

## What Is Being Tested

Primary hypothesis:

- AI writers produce correct and performant xlang solutions more reliably than
  Rust solutions for the same systems-programming tasks.

Secondary hypotheses:

- xlang reduces repair-loop count because rule-cited diagnostics make failures
  easier for weak models to fix.
- xlang reduces performance footguns by making the obvious source shape compile
  into the fast shape.
- xlang's constraints do not create equal or worse footguns, such as excessive
  defensive copying, inability to express normal data structures, or prompt bloat.

Non-goals:

- Proving that xlang beats expert Rust.
- Proving that xlang is a general auto-parallelizing language.
- Building the production compiler.
- Measuring human ergonomics.

## Pre-Sprint Cleanup

These are blocking because otherwise the sprint will measure stale documentation
or known evidence gaps.

1. Update top-level docs to point at `spec/kernel-spec-v0.6.md`, not older specs.
2. Reconcile ownership status:
   - `spec/fr-reconciliation-m0.md` says the section 5 obligations are discharged.
   - `spec/kernel-spec-v0.6.md` still marks section 5 provisional.
   - Pick one status and make the docs consistent.
3. Tighten the codegen evidence package:
   - Add the missing Rust additive noalias comparator for B-add.
   - Record that xlang matches Rust and `restrict` C on the additive collapse.
   - Keep the conclusion as "beats unannotated C, not Rust."
4. Relabel the scatter result precisely:
   - Current region/effect rows do not prove permutation-disjoint scatter.
   - The structural win requires an additional injectivity/permutation fact.
5. Keep `make check` green after each documentation or experiment change.

## Task Set

Each task must have equivalent Rust and xlang prompts, the same functional tests,
and the same performance target where performance applies.

| Task | Why it matters | Rust baseline | xlang requirement |
|---|---|---|---|
| Checked integer parser | Tests byte handling, error values, and no silent overflow | Safe Rust parser over bytes | xlang parser using explicit checks and `Result` |
| Arena AST builder | Tests allocation shape and ownership friction | Safe Rust with typed arena or index pool | xlang region/pool shape without per-node free |
| Buffer/index kernel | Tests bounds checks and proof/report path | Safe Rust slices or Vec | xlang `buffer`, `index`, retained/elided checks |
| Error propagation chain | Tests recoverable error ergonomics | Rust `Result` with `?` | xlang `try`/ERR-3 equivalent |
| Pure pipeline fusion | Tests footgun avoidance | Idiomatic iterator chain and naive Vec stages | xlang obvious staged form and fused form |
| Noalias accumulator | Tests optimizer fact emission | Rust `&mut`/`&`, C naive/restrict reference | xlang `&uniq`/`&` emits noalias |
| Injective scatter | Tests the one residual structural candidate | Best safe Rayon variants plus one unsafe ceiling | xlang must either express injectivity or mark unsupported |

Minimum viable sprint: run the first four tasks plus noalias accumulator. Scatter
is included only if the language grows an explicit injectivity fact or a gated
proxy rule for the experiment.

## Model Tiers

Use at least three tiers because W1 is specifically about weak writers:

- Weak: small/cheap model with limited reasoning.
- Middle: current practical coding model.
- Strong: frontier coding model.

For each tier, run the same prompt budget and the same maximum repair attempts.
Do not tune prompts separately per language after seeing results.

## Prompt Protocol

Each trial has three phases:

1. Generate from the task prompt and the relevant language spec excerpt.
2. Run the checker/test harness.
3. Feed back only machine output: compiler diagnostics, test failures, and perf
   results. Allow a fixed number of repairs.

Record:

- first-shot parse/check success
- first-shot test success
- final test success within repair budget
- number of repair turns
- final source size
- final runtime
- whether the model attempted to bypass the rules
- whether the result used an unintended slow shape

## Scoring

A trial is correct if it passes all functional tests and uses no disallowed escape
hatches.

A trial is fast if it is within the task-specific threshold:

- scalar/noalias kernels: within 5 percent of the best Rust/xlang reference
- buffer/index kernels: within 15 percent of reference
- arena AST: within 20 percent of reference end-to-end
- parser/error tasks: no more than 25 percent slower unless diagnostics or code
  size are substantially better
- scatter: report separately as structural candidate, not as general score

Primary score:

```
correct_fast_rate = correct_and_fast_trials / total_trials
```

Secondary scores:

- median repair turns among successful trials
- first-shot correct rate
- first-shot correct-fast rate
- median source size
- median runtime gap to reference
- cheat/bypass attempt rate
- slow-shape rate among correct trials

## Pass Criteria

The sprint passes only if all of these hold:

1. xlang improves correct-fast rate by at least 20 percentage points over Rust on
   the weak tier.
2. xlang improves or ties correct-fast rate on the middle tier.
3. xlang does not lose by more than 10 percentage points on the strong tier.
4. xlang has lower median repair turns on the weak tier.
5. xlang does not require materially larger prompts than Rust for the same task.
6. At least one performance-footgun task shows that xlang makes the obvious shape
   fast while Rust weak-tier output commonly lands on the slow shape.

The sprint fails if any of these occur:

1. Rust plus normal guardrails performs within 10 percentage points of xlang on
   weak-tier correct-fast rate.
2. xlang's syntax/spec tax causes materially worse first-shot success.
3. The required xlang subset cannot express the first four minimum tasks without
   adding major language features.
4. The measured wins come only from expert-written references, not model outputs.
5. Scatter remains the only positive result and still requires an unimplemented
   injectivity fact.

## Implementation Plan

Phase 0: evidence cleanup.

- Fix stale docs and evidence mismatches listed above.
- Add the missing Rust B-add comparator.
- Keep `make check` green.

Phase 1: harness skeleton.

- Add a task manifest format. Current location: `m3/tasks.jsonl`.
- Add per-language prompt templates. Current location: `m3/prompts/`.
- Add adapters for Rust and xlang checking. Current runner: `m3/harness/run.py`.
- Add result capture as JSONL. Use `--out /path/to/results.jsonl`.
- Start with local deterministic reference solutions. Current location:
  `m3/submissions/reference/`.

Phase 2: minimum task execution.

- Implement checked arithmetic loop task. Status: reference Rust/xlang submissions pass.
- Implement value-match/result task. Status: reference Rust/xlang submissions pass.
- Implement noalias accumulator task. Status: reference Rust/xlang submissions pass.
- Implement checked integer parser task.
- Implement arena AST task.
- Implement buffer/index task, or explicitly mark the missing xlang support.
- Implement error propagation task, or explicitly mark the missing xlang support.
- Current status: Rust reference submissions exist for these four tasks; xlang
  entries are manifest-pending because the current democ subset lacks the needed
  byte/buffer/pool/`try` surface.

Phase 3: model runs.

- Run each task across the three model tiers.
- Use fixed prompt budget and repair budget.
- Preserve all generated sources, diagnostics, and timing output.

Reference harness smoke test:

```
python3 m3/harness/run.py --suite reference
```

Decision-readiness scorer:

```
python3 m3/harness/score.py /path/to/results.jsonl
```

Current local result is recorded in `m3/RESULTS.md`: Rust reference submissions
pass all seven current tasks; xlang reference submissions pass the three runnable
tasks and are pending on four minimum-sprint tasks because the current democ
subset lacks byte/buffer/pool/`try` support.

The local blocker audit is recorded in `m3/IMPLEMENTATION_GATES.md`. The four
pending xlang tasks are currently real subset/toolchain gaps, not harness
mistakes. The working recommendation is: do not start self-hosting compiler work
until those minimum M3 gaps are either implemented or the sprint is explicitly
narrowed with the understanding that it can no longer decide the full thesis.

Phase 4: decision report.

- Produce one table per model tier.
- Compare Rust and xlang on correct-fast rate and repair turns.
- List all language blockers that prevented a fair run.
- Decide pass, fail, or rerun with a narrowly justified missing feature.

## If The Sprint Passes

Proceed to the compiler bootstrap track, but keep scope tight:

1. Freeze subset S around the task shapes that produced the measured advantage.
2. Grow `prototype/democ` only enough to compile that subset.
3. Promote pending conformance cases needed by the measured tasks.
4. Start `compiler/src` only after the subset is demonstrably useful.

## If The Sprint Fails

Stop the self-hosting push. Keep the artifacts as a research record and consider
one of these pivots:

- a Rust linting/codegen assistant that enforces the useful xlang lessons
- a narrow DSL for checked numeric kernels or injective scatter
- a verifier/checker experiment without a new general-purpose language
- a documentation-only research report

## Current Working Judgment

The existing evidence does not support building xlang as a faster Rust. It does
support one more bounded experiment: whether weak AI writers get closer to the
safe, fast shape in xlang than in Rust. This sprint is the decision gate for that
claim.
