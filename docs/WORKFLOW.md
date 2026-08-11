# Whitefoot workflow

This is the sole operational guide for advancing Whitefoot. Ordinary delivery
is the default. Specification and protected-compliance changes use guarded
branches with explicit owner approval.

## Authorities

- `docs/constitution.md` is project law.
- `spec/kernel-spec.md`, as named by `docs/roadmap.md`, is the sole active
  language authority. Flat `spec/kernel-spec-vN.md` files are immutable
  archives.
- `docs/roadmap.md` is the living Direction Outline: it records directions,
  current facts, gaps, and candidate projects, but does not sequence work.
- `docs/current-plan.md` is the sole high-level execution plan. It is derived
  from one outline revision and cannot select a direction absent from that
  outline.
- `docs/planned/`, `docs/ongoing/`, and `docs/done/` contain numbered task
  records for work under the active plan. They coordinate execution and never
  create authority by themselves.
- `research/**/RESULTS.md` and equivalent accepted evidence records own
  measurements; `mcts_mem/` owns durable design choices and rejected
  alternatives; `governance/APPROVALS.md` records protected owner approvals.
- `docs/WORKFLOW.md` defines process but selects no work.

Compiler behavior, tests, plans, task records, design prose, and archives do
not define language semantics. Active source, builds, tests, and tools may not
depend on `archive/`.

## Approval boundaries

Owner approval is required in exactly these situations:

1. **High-level direction plan.** A new `docs/current-plan.md`, or a material
   revision to its objective, overall strategy, semantic direction, principal
   consumer boundary, acceptance criteria, risk posture, or stop conditions,
   starts as `PROPOSED`. It becomes `ACTIVE` only after an owner-facing
   explanation and explicit owner approval.
2. **Specification.** Any batch that will land different bytes at
   `spec/kernel-spec.md` requires the exact specification workflow below.
3. **Protected compliance evidence.** Any addition, modification, deletion, or
   rename involving conformance case source, manifest row or annotation,
   declared verdict, status, rule assignment, or coverage requires the protected
   evidence workflow below. So does any change to a canonical compliance
   runner, adapter, oracle, baseline, gate, collection or invocation wiring, or
   gate-integrity test that can alter collection, interpretation, verdict,
   coverage, baseline identity, or whether the gate runs. Ordinary compiler
   unit, integration, and regression tests are not protected by this gate.

Before asking for either protected approval, present the explanation and exact
candidate boundary, then stop and wait. Approval covers only the named bytes
and changes. Any changed candidate byte or scope returns to review.

No separate owner approval is required for task decomposition, claiming,
implementation, ordinary tests, documentation, bounded probes, integration,
or closure inside an `ACTIVE` plan. The lead may also create and execute a
subordinate task discovered during execution when it supports the approved
plan and does not alter its direction or protected boundaries.

A high-level plan approval does not preapprove later specification or
conformance changes. Conversely, task autonomy does not permit an agent to
turn a subordinate discovery into a new direction.

## High-level plan lifecycle

The Current Plan describes the whole approved undertaking, not one temporary
task. It may contain several stages, workstreams, blockers, and task families.
It states:

- why the work matters and which Direction Outline items it advances;
- the principal consumer or experiment and its authentic boundary;
- the major workstreams and their dependency order;
- invariants and protected boundaries that must not drift;
- expected specification or conformance decision points;
- verification, acceptance, and stop conditions; and
- explicitly excluded work.

The lifecycle is:

```text
Direction Outline + evidence
  -> AI drafts PROPOSED high-level plan
  -> independent review
  -> owner explanation and selection
  -> ACTIVE high-level plan
  -> autonomous planned/ongoing/done task execution
  -> terminal evidence and plan closure or replacement
```

`PROPOSED` authorizes no execution. Owner approval changes it to `ACTIVE`;
rejection or parking changes it to `NO ACTIVE PLAN` or replaces it with another
proposal. The approval is recorded in the plan's status and authority text;
ordinary plan selection does not need a separate approval-ledger entry.

While a plan is `ACTIVE`, the lead may autonomously refine task decomposition,
ordering, progress, evidence links, implementation choices, and bounded
supporting research. A newly discovered compiler defect, prerequisite, or
diagnostic investigation may become a subordinate task without changing the
plan.

Return to `PROPOSED` and owner review when a change would alter the approved
objective, overall strategy, semantic direction, principal boundary, acceptance
or stop conditions, or materially expand the risk being accepted. Do not use a
large number of side tasks to conceal such a change.

Replacing the Current Plan is a coordination barrier. Unclaimed planned tasks
from the old plan are deleted in the same change unless the replacement
explicitly carries them.
An ongoing task continues only when the new `ACTIVE` plan carries its exact
scope. A `PROPOSED` plan cannot carry execution authority.

## Task lifecycle

Tasks are independently integrable pieces of an `ACTIVE` plan:

- `docs/planned/` holds decision-complete tasks not yet claimed;
- `docs/ongoing/` holds claimed or immediately started tasks; and
- `docs/done/` holds concise terminal history for work that actually ran.

The planned stage is optional. Immediate work may register directly in
`docs/ongoing/`, but its registration commit must land before substantive work.
Several agents contributing to one deliverable share one record; read-only
reviewers do not create records.

### Register

After refreshing the integration branch, allocate consecutive numbers beginning
at `max(existing numbers) + 1` across all three task directories. Names are
`NNNN-short-slug.md`. The first registration commit assigns each number
permanently; a concurrent later registration refreshes and renumbers. Assigned
or deleted numbers are never reused.

A planned record contains:

- `Authority`: the exact `ACTIVE` plan item;
- `Goal`, direction, scope, invariants, method, and expected touch set;
- dependencies and integration order;
- validation and done criteria; and
- a stop condition.

It must be small enough for one executor context, independently integrable, and
free of unwritten design choices. Planned records authorize nothing beyond the
active plan.

### Claim or start

Claiming moves a record from `planned` to `ongoing` without renumbering and
adds `Status` (`IN PROGRESS`, `WAITING`, or `BLOCKED`), owner, workspace or
branch, base revision, and current progress. Refresh immediately before
claiming; the claim commit must land before substantive work begins. The first
claim to land wins. Work registered directly in `ongoing` supplies the same
fields in its registration commit. `ACTIVE` is reserved for the Current Plan.

Claim only when each listed premise is terminal, its exact required premise
commit or canonical result has landed, or cross-linked records explicitly
permit concurrent execution and state the integration order. A parent task may
create a subordinate task that runs alongside it; link the parent and child
with an explicit integration order rather than falsely requiring the parent to
finish first.

### Dependencies and side tasks

Ordinary textual overlap is a rebase warning. Semantic or authority overlap—
the same language rule, ABI, effect or resource model, proof contract, durable
decision, correctness oracle, plan, or protected evidence—requires cross-linked
records and one stated integration order.

The lead may register a subordinate side task without owner approval when it:

- directly advances or unblocks the exact active-plan item and is
  proportionate to it;
- has a bounded goal and stop condition;
- does not change the high-level direction or acceptance boundary; and
- does not cross the specification or protected-compliance gates.

If any condition fails, stop and route the discovery back to the relevant
approval boundary instead of disguising it as a side task.

### Execute and integrate

An executor reads this workflow, the full task record, and its cited
authorities; works in an isolated worktree; implements exactly the written
scope; runs the required gates; and submits the result for lead review. A
blocker, plan defect, or out-of-scope discovery is a successful stopped outcome
when reported with a reproduction. Never hack around it, weaken evidence, or
quietly narrow the deliverable.

One live worktree has one writer. Do not commit, reset, rebase, or edit inside
another executor's worktree. Integration and conflict resolution happen in a
worktree owned by the integrator.

Before resuming, rebasing, or integrating, refresh the integration branch,
reread relevant task records and changed authorities, rebase, and rerun the
gates owed by those changes. Review challenges relevance, proportionality,
sequencing, and correctness.

### Close

First put facts, measurements, decisions, and live status in their canonical
homes. Then move the same record from `ongoing` to `done` in the integration
change and set `DONE`, `PARKED`, `REPLACED`, or `ABANDONED`. Replace live
operational detail with outcome, landed commits, canonical evidence,
validation, and remaining follow-up.

If live dependents exist, the same integration change replaces their task link
with the landed commit or canonical result and records the refresh, rebase, and
gates now owed.

Done records are frozen coordination history, not a second roadmap, plan,
results report, decision tree, approval ledger, compiler status, or
specification. A planned task that never starts is deleted rather than moved to
done; its number remains burned.

## Execution discipline

The owner selects the high-level plan and protected changes. The lead owns task
decomposition, bounded side work, design within the approved boundaries,
review, and integration. Executors maximize throughput inside one written task;
they do not redesign the plan.

Before changing code, answer:

1. What concrete compiler capability, real program, or experiment does this
   unlock?
2. Which `ACTIVE` plan item and task record authorize it?
3. What is the smallest general implementation?
4. Does it exercise the normal compiler path rather than a project, function,
   source-shape, corpus, or test special case?
5. Has supporting machinery become larger than the capability it serves?

Freeze the real consumer boundary, oracle, and cost obligation before changing
the implementation. A project selects pressure, not language semantics. Adapt
project code when a Whitefoot-native form preserves the frozen behavior; make
compiler changes general and project-independent.

For performance work, attribute the loss before optimizing. Use a same-source
causal comparison, record the expected code-shape consequence and a falsifier,
and retain parity or negative results. A workaround that changes behavior,
errors, ownership boundary, required complexity, or a preregistered performance
band cannot close the task.

## Blocker routing

- **Compiler defect:** implemented behavior contradicts the active spec. Add
  the smallest regression and fix the normal path without changing normative
  expectations.
- **Unsupported specified capability:** the spec determines behavior but the
  compiler stops as unsupported. Implement it only when the active plan needs
  it; never report it as invalid source.
- **Protected-evidence issue:** conformance or another protected compliance
  artifact appears wrong or needs expansion. Reproduce it, keep the active spec
  authoritative, and enter the protected evidence workflow before landing a
  change.
- **Research or performance question:** evidence is insufficient. Run the
  cheapest bounded probe with a hypothesis, observable, and stop condition.
  A plan-supporting probe is an autonomous side task.
- **Language gap:** the active spec is ambiguous, incomplete, or should change.
  Record the minimal witness. If the active high-level plan does not already
  contain this direction, return to plan approval; in all cases, exact spec
  bytes still require their own approval.
- **Project-local issue:** adapt the project rather than generalizing the
  language or compiler when the frozen contract is preserved.

A soundness defect may preempt other work. Fix it with hostile regression
evidence, then return to the active plan. A second or unexpected blocker does
not automatically require owner review; the deciding question is whether its
resolution changes the approved high-level plan or a protected boundary.

## Evidence discipline

- State exact commands, inputs, outputs, counts, and exit codes. Read an exit
  code directly, not through a pipe.
- Prefer differential reproduction on the same source before and after the
  change. Use a detached worktree for the other side rather than moving a live
  working tree.
- Resolve every commit id, digest, path, and count with the relevant tool before
  writing it. Do not copy old measurements forward.
- When adding tests, verify the collected count increased as expected and that
  a deliberate negative control makes the check fail.
- If diagnostic ordering, precedence, or rule citation may move, compare every
  affected case's result and cited rule across both binaries; an unchanged
  failure set is insufficient.
- A peer report is a lead, not evidence. Reproduce it or label the unverified
  part. If a probe did not isolate the claim, write `not measured`.
- Every new check states what a green run does and does not establish.

## Specification-change workflow

Use this branch only for a real language gap named by the `ACTIVE` high-level
plan. Candidate preparation is non-authoritative; owner approval is required
before activation.

1. **Bound the delta.** Consult the relevant live MCTS node and rejected
   alternatives with the `mcts-mem-use` skill. Inventory grammar, names, types,
   ownership, effects, runtime behavior, checks, diagnostics, ABI, conformance,
   compiler, examples, and documentation as changed, unchanged, or not
   applicable.
2. **Draft one exact candidate.** Edit `spec/kernel-spec.md` on the task branch,
   bump the version, and include its final status wording. Prepare
   `spec/kernel-spec-vPREVIOUS.md` from the previously active bytes in the same
   reviewable change; fail if that immutable archive path already exists.
   Concurrent spec branches rebase onto the selected predecessor and recompute
   the complete digest. Never resolve the stable spec with `ours` or `theirs`.
3. **Prepare evidence.** Derive positive, negative, and near-miss expectations.
   For grammar or syntax changes, run the production verifier:

   ```sh
   cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
     spec/kernel-spec-vPREVIOUS.md spec/kernel-spec.md
   ```

   Review the exact diff, impact inventory, protected changes, verifier output,
   real-program effect, accepted-set risk, limitations, archive action, and the
   SHA-256 of the complete candidate.
4. **Explain and wait.** Present the owner-facing explanation before asking for
   approval. Then stop and wait for the owner's explicit response; do not
   continue activation in the same turn. Approval covers only the exact bytes
   and named changes. Record it in `governance/APPROVALS.md`. Any byte or scope
   change, including a rebase resolution, requires a new explanation and
   approval.
5. **Activate atomically.** Land the approved stable bytes, outgoing immutable
   archive, one exact chained record
   `ACTIVE-SPEC: vN <new-sha256> <previous-sha256>`, compiler and generated
   changes, approved conformance changes, docs, plan/outline updates, and other
   derived material as one coherent activation. Valid but unsupported behavior
   remains unsupported, never rejection.
6. **Verify and close.** Recompute the installed digest, check archive identity
   and the activation chain, run focused and complete gates, inspect every
   impact row, and rerun the same frozen real consumer and oracle. Update MCTS
   only for a real durable decision or rejected alternative.

Released versioned specification archives are append-only and are never
edited, renamed, or deleted.

## Protected compliance workflow

This branch covers every protected compliance addition or change defined in
the approval boundary, even when the active specification does not change.

1. Reproduce the need against the active spec and distinguish a bad expectation
   from a compiler defect, unsupported capability, or tool failure.
2. Prepare the smallest exact candidate and a before/after audit listing every
   path or case id, source or manifest change, verdict, status, rule, coverage,
   actual behavior, collection count, and active-spec basis.
3. Explain the change, its compliance and accepted-set implications, and the
   exact candidate boundary to the owner; then stop and wait for explicit
   approval. Record the approval in `governance/APPROVALS.md`. A changed byte or
   scope requires renewed approval.
4. After approval, land the named changes, run the per-case differential,
   adapter or canonical runner, coverage checks, and the complete repository
   gate. Report all before/after totals.

When a protected evidence change follows a spec change, combine it with that
specification approval packet so the owner sees the language and corpus impact
together.

Never change a verdict, status, rule citation, coverage row, or baseline merely
to make a gate green. A compiler limitation, internal error, timeout, or
unsupported feature cannot rewrite normative expectations.

## Verification and closure

Use `/Users/bytedance/do_not_scan` for scratch files and test artifacts.

For compiler work, run before and after:

```sh
make -C compiler check
```

Before committing a completed repository slice, run:

```sh
make check
```

A green gate states only what it exercises. Keep required runtime checks unless
machine proof discharges them; optional facts may not change acceptance,
cleanup, output, or required trap behavior. Every new fact producer/consumer
pair needs bounded positive, near-miss, invalidation, and facts-off evidence.

At closure, put semantics in the spec, implementation facts in code/tests and
the compiler README, measurements in the canonical results record, durable
decisions in MCTS, and protected approvals in the ledger. Update the Direction
Outline and the high-level Current Plan once, rather than copying status into
supporting prose. Keep commits cohesive and never weaken evidence to turn a
gate green.
