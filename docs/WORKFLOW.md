# Whitefoot workflow

This is the sole operational guide for advancing Whitefoot. It contains two
related but distinct workflows:

1. the **project-delivery workflow**, used for every selected milestone,
   including project adaptation, implementation of already-specified
   capabilities, compiler defect repair, performance work, and bounded
   research; and
2. the **specification-change workflow**, entered only when the active language
   is ambiguous, incomplete, or must deliberately change.

The specification workflow is a conditional branch of project delivery, not a
phase that every change must pass. It carries additional design, evidence,
exact-approval, synchronization, and activation obligations, then returns to
the project that exposed the language gap.

## Authorities

- `docs/constitution.md` is project law.
- The active numbered specification named by `docs/roadmap.md` is the sole
  language authority.
- `docs/roadmap.md` is the living Direction Outline: the canonical map of
  directions, current facts, gaps, and candidate projects. It does not sequence
  current work.
- `docs/current-plan.md` is the only current execution proposal or approved
  plan. It is derived from one outline revision and cannot add an unselected
  direction.
- `docs/WORKFLOW.md` defines process but selects no work.
- Bare `WORKFLOW.md` references in immutable or protected artifacts name this
  sole guide by basename; they do not imply a second copy at the repository
  root.
- `compiler/README.md` and tests describe implementation reality;
  `research/experiments/*/RESULTS.md` owns measurements; `mcts_mem/` owns
  durable design choices and rejected alternatives; and
  `governance/APPROVALS.md` records protected owner approvals.

Compiler behavior, tests, candidates, plans, design prose, and archive material
never define language semantics. Active source, builds, tests, and tools may not
depend on `archive/`.

## Workflow boundary

Stay in the project-delivery workflow by default. Implementing behavior already
required by the active specification, fixing a compiler bug, adapting project
code, improving lowering without changing semantics, repairing unprotected
documentation or tests, and running approved bounded research do **not** open a
specification change.

Enter the specification-change workflow only after blocker classification
shows that the active language itself is ambiguous, incomplete, or should
deliberately mean something different. Discovering such a gap does not itself
authorize a proposal or implementation: the approved Current Plan or an
explicit owner reordering must name the gap and the project capability it
unlocks.

## Project-delivery workflow

The main loop is:

```text
Direction Outline + candidate evidence
                 -> AI next-stage proposal
                 -> proposed Current Plan
                 -> owner selection
                 -> active Current Plan
                 -> execute and verify one cohesive slice
                 -> commit evidence and implementation
                 -> update Outline and replace Current Plan
```

A selected umbrella project supplies the long-lived external pressure source
and honest end-state claim until the owner explicitly reevaluates it. It does
not authorize a monolithic implementation or make every capability on the way a
prerequisite. The rolling Current Plan still names only the next independently
reviewable vertical slice. A different project may run as separately approved
research, but it does not become a hidden phase in front of the selected project.

An AI proposal starts as a review packet in the conversation. When it is ready
for an owner decision, it may replace `docs/current-plan.md` with the sole
`PROPOSED` plan; a proposed plan authorizes no execution. It states:

1. the candidate project and bounded milestone;
2. the outline items it tests and why they matter now;
3. the authentic Whitefoot boundary, correctness oracle, and any honest
   comparator;
4. the smallest next slice, its verification, acceptance, and stop condition;
5. expected blockers and explicitly excluded work; and
6. optional bounded research that could proceed independently if approved.

The owner may approve, revise, park, or reject it. Approval changes the one file
to `ACTIVE`; rejection or parking changes it to `NO ACTIVE PLAN`. Normal
project selection does not enter `governance/APPROVALS.md`; protected evidence,
project law, trust boundaries, and exact specification bytes still require
their existing explicit approval records.

An active plan names one milestone, one current step, and normally at most one
direct blocker. Each step says `Why`, `Do`, `Verify`, and `Accept`. Do not
predict a long feature sequence before the first real port exposes its blockers.
Approval covers that written `Do`, not every next action exposed by it. When a
completed step reveals another slice outside that boundary, replace the plan
with a proposal for owner selection before expanding execution.

The step freezes one consumer-visible slice: pinned upstream identity and input
class, the exact Whitefoot/host ownership boundary, observable behavior, oracle,
and any preregistered comparator envelope. Project source may change to use an
approved general capability; the slice may not be replaced with an easier
project, input class, behavior, boundary, oracle, or comparator to escape a
blocker.

## Project gates

Every selected project milestone passes four gates:

1. **Frame.** Pin the project/version/license, claim, included and excluded
   scope, authenticity boundary, oracle, optional comparator, canonical
   evidence home, and stop condition.
2. **First faithful attempt.** Express the smallest real path with current
   Whitefoot before changing the language or compiler. Produce an execution
   result or one reproducible blocker; getting to green through a materially
   distorted workaround is not required.
3. **Resolve one blocker.** Classify it, make the smallest general change, add
   project-independent evidence, and immediately rerun the same frozen slice.
   Before a nontrivial compiler-design change, consult the relevant live MCTS
   node and rejected alternatives. Repeat only along the milestone's critical
   path.
4. **Validate and close.** Run the real integration and oracle; measure only
   claims the project makes; record limitations. Success, a useful negative
   result, or a triggered stop condition are all honest closures.

A project selects pressure, not semantics. A project-local inconvenience is
fixed in the project. Compiler changes may not dispatch on project, function,
source shape, corpus, or test identity.

A workaround is material when it changes advertised results or errors, moves a
responsibility outside the frozen ownership boundary, worsens a required
asymptotic or resource property, or leaves a preregistered performance band. A
different algorithm, representation, or copy is not a blocker merely because it
differs from upstream. A material workaround may be used as a bounded diagnostic
control, but it cannot close the slice. Record the intended shape, the runnable
workaround, and the exact semantic or cost delta instead of hiding it behind a
green result.

A reproducible blocker names the exact source identity, invocation, compiler
outcome, controlling active-spec rule, and a control that distinguishes the
claimed cause. A conversation summary is not the canonical evidence record.

## Route a blocker

Start with the active specification:

Failure to reproduce an upstream implementation shape is not by itself a
language gap. Call a required operation operationally inexpressible only when
the frozen consumer contract has no legal construction under the active
specification, a minimal project-independent witness demonstrates the missing
operation, and a bounded review finds no credible Whitefoot-native algorithm,
representation, or boundary that preserves the required behavior, errors,
resource obligations, and asymptotic contract. A missed constant-factor or
performance target is routed separately below; it never makes a legal operation
inexpressible.

- **Compiler defect:** a path the compiler claims to implement accepts,
  rejects, computes, traps, or lowers contrary to the specification. Add the
  smallest practical regression, keep the spec and protected expectations
  unchanged, and fix the normal path.
- **Unsupported specified capability:** the specification determines behavior
  but compilation deliberately stops as unsupported before an implemented
  semantic or lowering path. Report unsupported rather than source rejection;
  implement only when the current plan needs it.
- **Protected-evidence mismatch:** an existing conformance verdict or status is
  wrong. Stop and obtain owner approval before changing, removing, or weakening
  it. Never change the language to preserve a bad test.
- **Research or performance question:** evidence is insufficient to choose.
  Run the cheapest bounded probe with a hypothesis, observable, and stop
  condition. Attribution precedes magnitude; parity and negative results are
  retained.
- **Language gap:** the active specification is ambiguous, incomplete, or must
  deliberately change. An inexpressible result records the gap but grants no
  specification authority; enter the guarded branch below only with exact
  plan or owner authorization.
- **Project-local inconvenience:** adapt the project rather than generalizing
  the language or compiler when a Whitefoot-native substitute preserves the
  frozen consumer contract and cost obligations.

A soundness defect may preempt the current milestone. Fix it with hostile
regression evidence, then return to the selected project.

A correct accepted program outside its performance band remains a research or
performance blocker until the loss is attributed among algorithm and work,
representation, required checks, writer pattern, compiler lowering, LLVM
recovery, target code, allocation/cache/I/O, and measurement noise. Slowness
alone is not a language gap. If another independent blocker lies outside the
active step's `Do`, stop and replace the Current Plan instead of ratcheting the
project through an unreviewed sequence of language changes.

Language and specification gaps accumulate across every Current Plan for one
project milestone until that milestone completes or is parked. A second
independent gap is a presumptive reframe-or-park condition even when the first
was resolved. Continuing requires an explicit owner override; replacing the
Current Plan or completing a smaller slice never resets the blocker list.

## Performance and attribution

Correctness and comparable work precede timing. A performance-bearing plan
freezes revisions; all source and dependencies inside the timed boundary; input
and corpus digests; observable result consumption; target, build, resource,
durability, and initial-state settings; timed phases; and the statistic,
uncertainty, and materiality rules. Compare equivalent work or report unequal
phases separately. Describe a subset only as that subset.

The first correctness-green source on the ordinary compiler path is the
zero-change baseline. Profile it before selecting an optimizer direction. Trace
only the first material divergence, at the layers needed by the hypothesis:

```text
algorithm and work performed
  -> source representation and required checks
  -> checked program and available facts
  -> emitted and optimized LLVM
  -> final binary shape
  -> timing and relevant hardware or I/O counters
```

Classify the first supported cause before changing code. Use only the focused
dumps or experiment-local summaries needed by the hypothesis.

One optimization slice addresses one attributed cause. Preregister its expected
code-shape consequence and a falsifier; keep the frozen work byte-identical; and
vary one target-local fact toggle or compiler patch. If other material code
changes, enumerate them and credit only the whole patch until a narrower
ablation isolates the target. Credit a gain only when correctness is unchanged,
the expected binary delta exists, repeated measurements clear the frozen rules,
and no material work difference remains unexplained. An upstream ratio supports
only the scoped product comparison; a Whitefoot mechanism requires this
same-source causal ablation. Retain parity, loss, and inconclusive outcomes.

Planned proof and optimizer directions are candidates, not obligations. Open
one only when a measured project hotspot exhibits its exact pressure.

## Parallel research

Independent research may run when an active Current Plan lists it or when the
owner separately approves it, including while the plan is `PROPOSED` or `NO
ACTIVE PLAN`. A gap in the outline alone is not authorization. Independent
research uses this lane, not the four project gates, unless it starts a real
port.

Before work, the authorization names one question; the decision or outline
item the answer may change; a hypothesis and observable when empirical;
required evidence; an existing note or, only when reproducibility needs it, one
self-contained experiment bundle; the stop condition; and any outline edits or
status outcomes it permits. A separately approved probe records that packet at
the top of its evidence artifact so authority survives the conversation.

At the stop condition, record a positive, negative, or inconclusive result. An
independent agent or the owner reviews whether the evidence supports the exact
wording. Only dispositions and outline edits already named by the authorization
may then land; anything broader returns to the owner. Update the outline's
`Current`, `Missing / next`, and fact links as applicable and increment its
revision. Reconcile the Current Plan's `Derived from` revision in the same
change: advance it when its premises and item identities remain valid, or
replace it for review when they do not. Research never authorizes compiler
work, changes project law or protected evidence, or activates specification
bytes.

## Specification-change workflow (conditional branch)

A language change is allowed only when an approved current milestone or an
explicit owner reordering names the semantic gap it unlocks.

1. **Bound the delta.** Consult the relevant live MCTS node and rejected
   alternatives with the `mcts-mem-use` skill. Inventory grammar, names, types,
   ownership, effects, runtime behavior, checks, diagnostics, ABI, conformance,
   compiler, examples, and documentation as changed, unchanged, or not
   applicable.
2. **Draft one candidate.** Copy the active spec to
   `governance/spec-evolution/kernel-spec-vN-candidate.md` and apply the smallest
   complete change. A candidate is non-authoritative. Released
   `spec/kernel-spec-v*.md` files are append-only and are never edited, renamed,
   or deleted. If the canonical next-version candidate path is already occupied,
   stop for an owner choice: merge only a coherent delta on the same critical
   path, supersede only after preserving its still-live constraints and fixing
   every active link, or defer the new proposal. Never silently overwrite,
   combine unrelated deltas, or skip a version to avoid the choice.
3. **Prepare evidence.** Derive positive, negative, and near-miss expectations
   before implementation. For grammar or syntax changes, run the production
   compiler's native verifier:

   ```sh
   cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
     governance/spec-evolution/kernel-spec-vN-candidate.md
   ```

   When performance is part of the selection ground, produce the cheapest
   non-authoritative feasibility evidence available before exact approval. If
   an essential claim cannot be tested first, defer activation unless the owner
   explicitly accepts it as an unverified limitation. A later negative result
   closes the project honestly; it does not retroactively invalidate immutable
   approved bytes.

4. **Obtain exact approval.** Present the complete candidate SHA-256, semantic
   delta, impact inventory, verifier results, requested protected changes, and
   limitations. Owner approval covers only those exact bytes and named changes;
   record it in `governance/APPROVALS.md`. A changed byte returns to review.
5. **Activate atomically.** Copy the approved candidate byte-for-byte to the new
   numbered spec and, in the same cohesive change, update every active spec
   identity, compiler rule and generated datum, conformance expectation or
   status explicitly approved, test, writer form, live doc, outline item, and
   current plan affected by the delta, plus the derivation ledger and any MCTS
   Item made false by the change. Record a paired Move only for a genuine
   re-decision. Valid but still unsupported behavior remains unsupported, never
   rejection.
6. **Verify and close.** Compare candidate and installed bytes, run focused
   checks plus the complete gate and MCTS lint when applicable, and inspect
   every impact row. Rerun the same frozen project slice with the same input
   class, boundary, and oracle before the specification branch is closed;
   conformance tests alone do not close the project blocker. Activation may be
   its own cohesive protected commit, but the Current Plan remains on the same
   milestone until the project result and limitations are recorded.

Use MCTS only for a durable re-decision where a real alternative existed; keep
approval bookkeeping and progress out of the tree. If no dedicated node exists,
consult the nearest live decision and its real alternatives; do not fabricate a
node or rival merely to satisfy process.

## Verification and closure

Before closing a repository slice:

```sh
make -C compiler check   # before and after compiler work
make check               # complete repository gate
```

Use the project-mandated scratch root under `/Users/bytedance/do_not_scan` when
tests or experiments need temporary files. A green gate states only what it
exercises.

Required checks remain unless machine proof discharges them. Optional facts may
improve an accepted program but may not change acceptance, select another
semantic path, or alter output, cleanup, or required trap behavior. Every new
fact producer/consumer pair carries bounded project-independent positive,
premise-near-miss, and invalidation or mutation cases; facts-off comparisons
cover success, trap, and cleanup. Keep this evidence local to the fact family.

At closure, put semantics in the spec, implementation facts in code/tests and
the compiler README, measurements in RESULTS, durable decisions in MCTS, and
protected approvals in the ledger. Update the living outline and replace the
rolling plan. The next state may continue the umbrella project, park or deselect
it and reopen candidate selection, or record `NO ACTIVE PLAN`; closure does not
imply another project slice. Do not copy the same status into supporting prose.
Keep commits cohesive, and never weaken evidence merely to make a gate green.
