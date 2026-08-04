# Whitefoot workflow

This is the sole operational guide for advancing Whitefoot. Development has
one project-driven main loop, a bounded parallel-research lane, and one guarded
branch for changing the language.

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
- `WORKFLOW.md` defines process but selects no work.
- `compiler/README.md` and tests describe implementation reality;
  `research/experiments/*/RESULTS.md` owns measurements; `mcts_mem/` owns
  durable design choices and rejected alternatives; and
  `governance/APPROVALS.md` records protected owner approvals.

Compiler behavior, tests, candidates, plans, design prose, and archive material
never define language semantics. Active source, builds, tests, and tools may not
depend on `archive/`.

## From outline to execution

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

## Project gates

Every selected project milestone passes four gates:

1. **Frame.** Pin the project/version/license, claim, included and excluded
   scope, authenticity boundary, oracle, optional comparator, and stop
   condition.
2. **First runnable port.** Try the smallest real path with current Whitefoot
   before changing the language or compiler. Produce an execution result or
   one reproducible blocker.
3. **Resolve one blocker.** Classify it, make the smallest general change, add
   project-independent evidence, and immediately rerun the project. Repeat only
   along the milestone's critical path.
4. **Validate and close.** Run the real integration and oracle; measure only
   claims the project makes; record limitations. Success, a useful negative
   result, or a triggered stop condition are all honest closures.

A project selects pressure, not semantics. A project-local inconvenience is
fixed in the project. Compiler changes may not dispatch on project, function,
source shape, corpus, or test identity.

## Route a blocker

Start with the active specification:

- **Compiler defect:** the specification determines behavior and the compiler
  disagrees. Add the smallest practical regression, keep the spec and protected
  expectations unchanged, and fix the normal path.
- **Unsupported specified capability:** the specification determines behavior
  but the compiler does not implement it. Report unsupported rather than source
  rejection; implement only when the current plan needs it.
- **Protected-evidence mismatch:** an existing conformance verdict or status is
  wrong. Stop and obtain owner approval before changing, removing, or weakening
  it. Never change the language to preserve a bad test.
- **Research or performance question:** evidence is insufficient to choose.
  Run the cheapest bounded probe with a hypothesis, observable, and stop
  condition. Attribution precedes magnitude; parity and negative results are
  retained.
- **Language gap:** the active specification is ambiguous, incomplete, or must
  deliberately change. Enter the guarded branch below.
- **Project-local inconvenience:** adapt the project rather than generalizing
  the language or compiler.

A soundness defect may preempt the current milestone. Fix it with hostile
regression evidence, then return to the selected project.

## Parallel research

Independent research may run beside the milestone only when an active Current
Plan lists it or the owner separately approves it. A gap in the outline alone
is not authorization. Each probe must state one question, the evidence that
would change the outline, and the condition that stops the work.

Research records facts in the appropriate note or self-contained experiment
bundle. After review, update the outline's `Current`, `Missing / next`, and fact
links and increment its revision. Research does not authorize compiler work,
change project law, alter protected evidence, or activate specification bytes.

## Guarded language-change branch

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
   or deleted.
3. **Prepare evidence.** Derive positive, negative, and near-miss expectations
   before implementation. For grammar or syntax changes, run the production
   compiler's native verifier:

   ```sh
   cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
     governance/spec-evolution/kernel-spec-vN-candidate.md
   ```

4. **Obtain exact approval.** Present the complete candidate SHA-256, semantic
   delta, impact inventory, verifier results, requested protected changes, and
   limitations. Owner approval covers only those exact bytes and named changes;
   record it in `governance/APPROVALS.md`. A changed byte returns to review.
5. **Activate atomically.** Copy the approved candidate byte-for-byte to the new
   numbered spec and, in the same cohesive change, update every active spec
   identity, compiler rule and generated datum, conformance expectation or
   status explicitly approved, test, writer form, live doc, outline item, and
   current plan affected by the delta. Valid but still unsupported behavior
   remains unsupported, never rejection.
6. **Verify and close.** Compare candidate and installed bytes, run focused
   checks plus the complete gate, inspect every impact row, commit the state
   transition, and return to the project that required it.

Use MCTS only for a durable re-decision where a real alternative existed; keep
approval bookkeeping and progress out of the tree.

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
fact consumer carries focused negative canaries and facts-off comparison.

At closure, put semantics in the spec, implementation facts in code/tests and
the compiler README, measurements in RESULTS, durable decisions in MCTS, and
protected approvals in the ledger. Update the living outline and replace the
rolling plan; do not copy the same status into supporting prose. Keep commits
cohesive, and never weaken evidence merely to make a gate green.
