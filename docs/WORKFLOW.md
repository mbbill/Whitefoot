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
- The active specification at `spec/kernel-spec.md`, named by
  `docs/roadmap.md`, is the sole language authority. Flat
  `spec/kernel-spec-vN.md` files are immutable archives, not parallel active
  authorities.
- `docs/roadmap.md` is the living Direction Outline: the canonical map of
  directions, current facts, gaps, and candidate projects. It does not sequence
  current work.
- `docs/current-plan.md` is the sole current execution proposal or approved
  plan and the sole source of plan-derived authority and sequencing. It is
  derived from one outline revision and cannot add an unselected direction.
- `docs/planned/` contains numbered, non-authorizing task decompositions of
  the `ACTIVE` plan awaiting claim; `docs/ongoing/` contains the same class of
  record for each substantial in-flight task; `docs/done/` retains the same
  numbered records as concise terminal history.
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

Inside one selected project, that governance loop advances the product this
way:

```text
freeze the next real slice + correctness oracle + cost obligation
  -> attempt it with the current language and compiler
  -> semantic or implementation blocker: resolve one general capability
  -> correctness-green but materially slow: attribute and resolve one cause
  -> rerun the same frozen slice
  -> widen the project only after correctness and performance gates pass
```

Performance is not a cleanup phase after a broad port. A known structurally
slow path is not a foundation on which to accumulate more project code.

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
4. the smallest next slice, its credible fast shape or explicit performance
   viability question, verification, acceptance, and stop condition;
5. expected blockers and explicitly excluded work; and
6. optional bounded research that could proceed independently if approved.

The owner may approve, revise, park, or reject it. Approval changes the one file
to `ACTIVE`; rejection or parking changes it to `NO ACTIVE PLAN`. Normal
project selection does not enter `governance/APPROVALS.md`; protected evidence,
project law, trust boundaries, and exact specification bytes still require
their existing explicit approval records.

An active plan names one milestone, one current step, and normally at most one
direct blocker. A step either attempts one project slice or resolves one
semantic, implementation, or attributed performance blocker; it does not hide
both a broad port and an open-ended feature sequence. Each step says `Why`,
`Do`, `Verify`, and `Accept`. Approval covers that written `Do`, not every next
action exposed by it. When a completed step reveals another slice outside that
boundary, replace the plan with a proposal for owner selection before expanding
execution.

The step freezes one consumer-visible slice: pinned upstream identity and input
class, the exact Whitefoot/host ownership boundary, observable behavior, oracle,
and any preregistered comparator envelope. Project source may change to use an
approved general capability; the slice may not be replaced with an easier
project, input class, behavior, boundary, oracle, or comparator to escape a
blocker.

## Task coordination and closure history

`docs/ongoing/` makes concurrent execution visible without creating another
planning authority. Create one Markdown file for each substantial independently
integrable task, or task with a distinct integration or handoff boundary. A
task may be one independently executable part of an active plan's written `Do`,
or work covered by a separate owner approval. Agents contributing to one
deliverable share its record; read-only reviewers do not create another. Make
the record the task's first small integration commit so other workspaces can
see it before substantial work begins. It cannot broaden or resequence the
authority it cites.

A task record moves through at most three stages: `docs/planned/` holds tasks
decomposed from an `ACTIVE` plan's written `Do` but not yet claimed,
`docs/ongoing/` holds claimed in-flight tasks, and `docs/done/` holds terminal
history. The planned stage is optional; work that starts immediately registers
directly in `docs/ongoing/`.

`docs/planned/` is the claimable decomposition of an `ACTIVE` plan, not a
second roadmap or a self-growing backlog. Each planned file carries the
ongoing schema minus live state: `Authority` naming the exact `ACTIVE` plan
item it implements, `Goal`, `Direction and invariants`, `Method`, `Scope and
expected touch set`, `Dependencies and integration order` linking the `NNNN`
records it waits on, `Validation`, and `Done-when`. A planned task must be
independently integrable, small enough for one executor context, and free of
unwritten decisions; a task that still needs a design choice is not plannable.
Registering a batch of planned tasks is one integration commit by the planning
agent, and that commit allocates their numbers.

Claiming is one commit that moves the file from `docs/planned/` to
`docs/ongoing/` with its number unchanged and fills in `Status`, `Owner`,
workspace, and `Base revision`. The first claim to land on the integration
branch wins; a losing workspace rebases and claims another task. Only a task
whose listed dependencies are all terminal, or whose cross-linked integration
order explicitly permits the overlap, may be claimed.

Planned files authorize nothing and are pruned like any superseded material:
when the plan they cite is replaced, unclaimed planned tasks are deleted in
the same change unless the new `ACTIVE` plan explicitly carries them. A
deleted number is burned, never reused. A planned task that will never start
is deleted rather than moved to `docs/done/`; done history records only
executed work.

Every task record is named `NNNN-short-slug.md`. The four-digit number comes
from one monotonically increasing sequence shared across `docs/planned/`,
`docs/ongoing/`, and `docs/done/`. After refreshing the integration branch, a
new task proposes
`max(existing numbers) + 1` in its first registration commit. That integration
commit assigns the number permanently: moving the task never changes it and a
closed number is never reused. Concurrent branches may propose the same next
number, but the later one must renumber before integration. This intentionally
uses ordinary Git conflict and review rather than a separate allocator.

Keep each record short and operational. It contains:

- `Status`: `IN PROGRESS`, `BLOCKED`, or `WAITING`; `ACTIVE` is reserved for
  the Current Plan;
- `Authority`, `Owner`, `Base revision`, and the workspace or branch;
- `Goal`, `Direction and invariants`, and `Method`;
- `Progress` as completed, current, and next meaningful outcomes;
- `Scope and expected touch set`, covering semantic areas and likely paths;
- `Dependencies and integration order`, linking other ongoing records when
  relevant; and
- `Validation`, `Stop condition`, and `Closure`.

The touch set is a rebase warning, not exclusive ownership. Agents in separate
workspaces may edit the same files. Ordinary textual overlap is settled during
rebase. Semantic overlap—two tasks changing the same language rule, ABI, proof
contract, resource or effect model, durable design decision, correctness
oracle, premise, Current Plan, outline status, approval, or workflow authority—
requires both records to cross-link the dependency and state one integration
order before both changes land. One task provides the premise; the dependent
task refreshes its base, rereads the changed authority and design records,
rebases, and reruns its gates. Incompatible decisions return to the owner
rather than resolving by last-writer-wins.

Before starting or resuming work, refresh the integration branch and fully read
every relevant record there. Before rebasing or integrating, refresh and read
them again, rebase, rescan the rebased directory, and update the record's base
revision and any changed dependency or validation obligation. Update a record
when direction, scope, meaningful progress, blockers, or handoff state changes—
not after every command. Discovery outside its cited authority is a blocker or
candidate for the next plan, not permission to expand the task.

At terminal disposition, first move durable facts, measurements, decisions, and
status to their canonical owners. Then, in the same integration change, move
the task record without renumbering from `docs/ongoing/` to `docs/done/` and set
its final status to `DONE`, `PARKED`, `REPLACED`, or `ABANDONED`. Replace
operational current/next and advisory touch-set detail with a concise outcome,
landed commits, canonical evidence, validation, and any remaining dependency or
follow-up links. If a record has live dependents, the same change replaces
their task link with the landed commit or canonical result and records the
refresh and gates they now owe.

Files in `docs/done/` are frozen coordination history. Do not keep updating
them as the project evolves and do not treat them as authority or as a second
copy of the Direction Outline, Current Plan, RESULTS, MCTS-Mem, approval
ledger, compiler status, or specification. Repair a broken closure link when
necessary, but put new facts and re-decisions in their canonical homes.

Replacing the Current Plan is a coordination barrier. A plan-derived task loses
execution authority when its `ACTIVE` plan is replaced unless the new `ACTIVE`
plan explicitly carries that record and exact scope; a `PROPOSED` plan cannot
carry execution authority. Separately approved work survives only through its
recorded stop condition. Several agents executing one approved `Do` are
ordinary project delivery; only independently authorized decision-changing
investigation uses the parallel-research lane below.

## Execution agents

Fan-out execution separates judgment from throughput. The owner and the lead
agent do the top-down work — direction, plans, decomposition, review, and
integration — and executor agents maximize output inside that frame. An
executor implements; it does not research, explore, redesign, or plan.

An executor's loop is fixed:

1. **Orient.** Refresh the integration branch, then read this workflow, the
   task record being claimed, and every authority and design record it cites.
2. **Claim.** Land the claim commit, then work in an isolated workspace or
   worktree branched from the recorded base revision.
3. **Execute exactly the written scope.** The task's cited plan item is the
   whole authority. An executor does not expand scope, substitute an easier
   interpretation, or improve adjacent code beyond the task.
4. **Escalate instead of forcing.** A blocker, plan defect, specification or
   compiler discrepancy, or discovery outside the cited authority stops the
   task and is reported honestly through the blocker routing below, with exact
   reproduction and classification evidence, so the owner and lead can repair
   the plan. Hacking around a blocker, weakening a check, test, or verdict, or
   quietly narrowing the deliverable is a governance breach; a material
   workaround remains a bounded diagnostic that cannot close a slice. An
   honest blocked report is a successful executor outcome — it converts a plan
   defect into the owner's and lead's next decision at the cheapest moment.
5. **Integrate through review.** Run the task's validation gates, update the
   record, and submit the change for lead review before it lands on the
   integration branch. Review challenges relevance, proportionality, and
   sequencing as well as technical soundness.

### Evidence discipline for executors and reviewers

These are standing rules, learned by paying for each one. They apply to every
report, record, brief, and approval entry, and a brief may cite this section
instead of restating them.

- **Say what you ran, on both sides.** A claim backed by a differential
  reproduction — the same source at the parent revision and at the fix, both
  rebuilt — is evidence. A claim backed by prose is a guess that reads
  identically in a report. Where a probe failed to isolate what you intended,
  write **"not measured"** rather than reasoning to a verdict; that phrase is
  worth more than the paragraph it replaces.
- **A retraction needs measurement too.** Walking a claim back without running
  anything is the same error as making it, and it is harder to catch because it
  sounds careful. Ask of a retraction what you would ask of the original.
- **Differential measurement moves the working tree**, which is how an
  uncommitted, already-verified fix was destroyed. Prefer a throwaway
  `git worktree add --detach` for the other side; it costs one compile and
  touches nothing. When taking the cheaper in-place path, commit the after
  state before checking out the before.
- **Never write an identifier you did not resolve.** Commit ids, short hashes,
  and digests are recomputed with `git rev-parse --short=8` or the hashing tool
  before they are written, in prose as much as in tables. A brief names a
  **branch**, never a commit id, so the reader resolves the tip and cannot
  inherit a stale or invented one. Every count carries the command that
  reproduces it, and no figure is copied forward without re-measuring.
- **A failure-set diff sees a citation move only where a test asserts the
  rule.** The set of passing and failing tests is the regression oracle for
  most changes, and it does catch a moved citation wherever some case asserts
  the cited rule. It is blind wherever nothing asserts it at that site — the
  common condition in a mostly-positive corpus, where cases assert pass/fail
  and the rule is never named. So the trigger is not "a rank changed" but
  **any change to which rule is cited in a population that mostly asserts
  pass/fail**: precedence, ordering, citation selection, and equally anything
  that reorders checks within a function, since hoisting one check above
  another moves every citation the second used to win. Those need a **per-case
  verdict differential** — every source run against both binaries, comparing
  exit code and cited rule — not the set diff. An unchanged failure set is
  exactly where such a change hides.
- **Short identifiers are not greppable as substrings, and never count with
  `-c`.** Four measurements were wrong this way in one day. `ieq|ine|ile|ige`
  matches inside *retained*, *scrutinee*, *multi-line*; `ile|ine` matches inside
  *while* and *line*. Use word boundaries with `-P` — `-E` treats `\b` as a
  literal `b`, silently, and ugrep refuses some bounded-repetition patterns
  outright. For counting, the portable form is `grep -o … | wc -l`, because
  **`-c` means different things in different greps**: the `grep` on this machine
  is ugrep 7.5.0, where `-oc` reports occurrences, while GNU `grep -c` ignores
  `-o` and reports matching lines. Two tools, opposite answers, identical flags,
  no error from either. A scope estimate built on any of this propagates into a
  brief as fact: one such figure was off by more than an order of magnitude and
  nobody could reconstruct its basis afterwards.
- **A test that never runs and a test that passes are indistinguishable by the
  result line — only the count separates them.** Four tests appended after a
  `__main__` guard were never collected and the suite still printed `OK`; what
  caught it was the count sitting at 18 where 22 was expected. Whenever you add
  tests, check that the reported count moved by the number you added. This
  happened while building a check whose entire purpose was catching that family,
  so being the person looking for it is no protection.
- **Assert a limit rather than documenting it, where the limit is checkable.** A
  documented limit decays silently as the thing around it changes; an asserted
  one fails when it stops being true. When a check cannot see some class by
  construction, write that inability as a test.
- **A check must say what a green run does NOT mean, and name what owns the
  rest.** A reader meeting a new check reasonably assumes it covers the problem
  it was built for. The declared-verdict diff carries all three
  migration-damage classes on the function itself — one already caught by the
  adapter, one reached by nothing verdict-based, one it owns — because without
  that, a green run reads as "the migration broke nothing", and that reading is
  wrong in the direction that has actually cost this project cases.
- **Do not collapse a sequence of corrections into one tidy entry.** Three
  successive corrections to one ruling stayed visible in the approval ledger,
  and the third was found by a reader whose objection attached to the
  *reasoning* of the first. A single clean entry states the conclusion and
  discards what could be checked; the sequence is what makes a ledger auditable
  rather than merely filed.
- **Read exit codes from `$?` directly, never through a pipe.** `make check |
  tail` reports the status of `tail`; a red gate has been committed here that
  way. Write `make check; echo "exit=$?"`.
- **Relaying converts a claim into an authority.** A peer's report is not
  evidence. Forward the reproduced half with its command and mark the rest
  unverified; never restate a peer's finding as a specification, a ruling, or an
  approval-ledger fact.
- **Ask whether a cited rule is the subject or an accident.** When a test or
  case changes which rule rejects it, determine whether the new citation is the
  rule the specification assigns to the violation that remains, or merely the
  one that happened to fire first. Recording the latter deletes coverage while
  leaving a green row, and it can launder a compiler defect into normative
  expectation.

### The failures that look like success

Most defects announce themselves. A handful do not, and every one of them was
found here by a deliberate question rather than by a gate, because **their
failure mode is success**: a conformance case that passes while testing nothing,
a check that cannot fail, a transform verified against its own output, an
operation performed against a baseline that no longer describes reality. Nothing
that watches for failure sees any of them.

The one habit that reaches all of them is to **prefer the observation that
separates two hypotheses over one consistent with the hypothesis you already
hold**. Gathering agreeing evidence feels like verification and is not, because
the same observation usually fits the rival reading too. Before running a check,
ask what result would make you believe the *other* thing; if no result would,
the check is decorative. Worked instances from this project:

- A working tree is clean *and* HEAD contains the fix — either alone is equally
  consistent with the fix having been destroyed.
- A test that MOVED to a different error versus one that STAYED PUT: moving
  means the fix worked and a second cause is underneath; staying means it did
  not work. The pass count is identical either way.
- Two branches moving the *same* binding versus *different* bindings, holding
  everything else fixed — that reclassified a supposed missing capability as a
  masked rejection in one probe.
- Breaking a check in each direction it can fail, not once. A wrong value and a
  missing entry should fail differently; proving both is what separates a real
  check from a decorative one.

Three corollaries worth stating on their own:

- **Run a transform against the input it should have handled, never against its
  own output.** A migrator, renderer, or formatter checked on what it produced
  is a fixed point and always agrees with itself.
- **A mask's fix is itself a probe.** Read the run immediately after removing
  one carefully instead of treating it as confirmation — that is the moment
  previously unreachable code first executes, and it is where a second hidden
  problem surfaces. A mask means the number of hidden problems is unknown, never
  one.
- **When a migrated case behaves oddly, read the migration diff before the
  compiler.** The program may simply have stopped being the program the case was
  written about, in which case a correct diagnosis of the compiler is an answer
  to the wrong question.

And a note on writing rules like these: state the **property** that produces the
failure, not the causes you happen to have met. The verdict-differential rule
above was first written as "ordering and precedence changes need a differential"
and was wrong in both directions at once — too strong because it generalized
from one instance, too weak because it enumerated known causes instead of the
condition (a population that asserts pass/fail and never names the rule) that
makes any of them bite.

### One writer per worktree

An executor's worktree has exactly one writer: that executor. Nobody commits
into it, rebases the branch checked out in it, or resets it while the executor
is live — not to rescue uncommitted work, not to integrate, not to fix a
message.

The reason is not tidiness. Every diagnostic an executor uses to check its own
state means something different when a second writer exists, and it cannot tell
the two apart: a `grep` returning pre-fix source, a `UU` in `git status`, and
its own commit ids going unreachable read exactly like "my work was reverted and
a merge is stuck" when they are in fact a concurrent rebase in progress. The
natural repair from that reading — re-applying a fix on top of itself, or
reverting the other writer's commit — destroys real work to rescue work that
was never lost. Both practices above assume single-writer, and telling the
executor afterwards does not close the window, because the race lives between
the action and the message.

If an idle executor is holding verified-but-uncommitted work, either **stop the
agent first and then operate on the tree**, or **take the diff out and land it
on your own branch**, leaving theirs untouched. Integration follows the same
rule: rebase and merge from a worktree you own, never from one an executor is
sitting in.

## Project gates

Every selected project milestone passes four gates:

1. **Frame.** Pin the project/version/license, claim, included and excluded
   scope, authenticity boundary, oracle, optional comparator, canonical
   evidence home, and stop condition.
2. **First faithful attempt and floor check.** Express only enough of the
   smallest real path to establish its correctness oracle and first
   performance-bearing shape. Use the fastest credible current-language shape
   supported by project-independent evidence. If every legal shape preserves
   behavior but violates the frozen asymptotic, resource, code-shape, or
   material performance obligation, stop with one reproducible performance
   blocker. A known structurally slow workaround is a diagnostic control, not a
   foundation for more project code.
3. **Resolve one blocker.** Classify it, make the smallest general change, add
   project-independent evidence, and immediately rerun the same frozen slice.
   Before a nontrivial compiler-design change, consult the relevant live MCTS
   node and rejected alternatives. Repeat only along the milestone's critical
   path.
4. **Validate and close.** Run the same real integration and oracle, then its
   scoped performance or cost-shape gate. Widen the project only when both pass.
   Measure only claims the slice makes and record limitations. Success, a useful
   negative result, or a triggered stop condition are all honest closures.

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

Before a correctness-green product path exists, a performance risk stops work
only when a structural cost argument identifies an unavoidable bad scaling unit
or architecture: for example an extra complete pass or copy, per-byte boundary
calls, whole-input materialization where bounded streaming is required, or
forced serialization that defeats the frozen slice. Otherwise record the risk
and reach the earliest comparable measurement point; intuition alone does not
authorize a language or optimizer feature.

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
recovery, target code, allocation/cache/I/O, and measurement noise. Once that
loss is material and attributed, stop widening the project, resolve the owning
layer generally, and rerun the same frozen slice. Slowness alone is not a
language gap. If another independent blocker lies outside the active step's
`Do`, stop and replace the Current Plan instead of ratcheting the project
through an unreviewed sequence of language changes.

Language and specification gaps accumulate across every Current Plan for one
project milestone until that milestone completes or is parked. A second
independent gap is a presumptive reframe-or-park condition even when the first
was resolved. Continuing requires an explicit owner override; replacing the
Current Plan or completing a smaller slice never resets the blocker list.

## Performance and attribution

Correctness and comparable work precede timing and performance claims, but not
performance-aware sequencing. Before widening a runnable slice, inspect its
cost shape; an unavoidable structural violation may stop it before full-product
timing. A performance-bearing plan
freezes revisions; all source and dependencies inside the timed boundary; input
and corpus digests; observable result consumption; target, build, resource,
durability, and initial-state settings; timed phases; and the statistic,
uncertainty, and materiality rules. Compare equivalent work or report unequal
phases separately. Describe a subset only as that subset.

At each newly runnable slice, the first correctness-green implementation of its
first material critical-path shape on the ordinary compiler path is the
zero-change baseline. Profile and classify it before widening the slice or
selecting an optimizer direction. Trace only the first material divergence, at
the layers needed by the hypothesis:

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
2. **Draft one candidate.** Edit `spec/kernel-spec.md` directly on the task
   branch and apply the smallest complete change. Bump its first-line version
   and put the final `Status: ACTIVE vN` paragraph inside the bytes before
   approval; neither spelling makes the branch authoritative. Review the
   candidate as its exact `git diff` plus the SHA-256 of the complete stable
   file. Do not create a parallel candidate copy under `governance/`.

   Prepare the outgoing flat archive `spec/kernel-spec-vPREVIOUS.md` from the
   previously active stable bytes in the same reviewable change, and fail if
   that path already exists. The one-time stable-file switchover instead uses
   the already released v0.23 archive and creates no duplicate archive. Every
   released versioned archive is append-only and is never edited, renamed, or
   deleted. Concurrent specification branches rebase onto the selected
   predecessor and recompute their complete digest; never use `-X ours` or
   `-X theirs` on the stable file, combine unrelated deltas, or skip a version
   to avoid a conflict.
3. **Prepare evidence.** Derive positive, negative, and near-miss expectations
   before implementation. For grammar or syntax changes, run the production
   compiler's native verifier:

   ```sh
   cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
     spec/kernel-spec-vPREVIOUS.md \
     spec/kernel-spec.md
   ```

   Both paths are read at run time, and the first is the exact outgoing
   baseline the candidate must preserve. The verifier does not compare against
   a compiled-in copy, so it keeps saying something after the candidate is
   installed. A grammar-changing batch uses the same two paths and reviews the
   intentional derived-table delta rather than assuming preservation.

   When performance is part of the selection ground, produce the cheapest
   non-authoritative feasibility evidence available before exact approval. If
   an essential claim cannot be tested first, defer activation unless the owner
   explicitly accepts it as an unverified limitation. A later negative result
   closes the project honestly; it does not retroactively invalidate immutable
   approved bytes.

4. **Obtain exact approval.** Present the complete stable-file SHA-256, semantic
   delta, `git diff`, impact inventory, verifier results, requested protected
   changes, and limitations. Owner approval covers only those exact bytes and
   named changes. Record that approval in `governance/APPROVALS.md`; a changed
   byte, including a rebase resolution, returns to review.
5. **Activate atomically.** Land the approved stable bytes in one linear
   commit. After the one-time v0.23-to-v0.24 switchover, that commit also
   creates the exact outgoing archive and fails rather than overwriting an
   occupied path. The switchover itself reuses the already released, digest-
   checked `spec/kernel-spec-v0.23.md`; it neither recreates nor overwrites that
   archive. Append exactly one chained record
   `ACTIVE-SPEC: vN <new-sha256> <previous-sha256>` and update every active spec
   identity, compiler rule and generated datum, conformance expectation or
   status explicitly approved, test, writer form, live doc, outline item, and
   current plan affected by the delta, plus the derivation ledger and any MCTS
   Item made false by the change. Record a paired Move only for a genuine
   re-decision. Valid but still unsupported behavior remains unsupported, never
   rejection.
6. **Verify and close.** Recompute the installed stable-file digest and compare
   it with the exact approval and the final chained record. Run the archive
   integrity gate, focused checks, the complete gate, and MCTS lint when
   applicable, then inspect every impact row. Rerun the same frozen project
   slice with the same input class, boundary, and oracle before the
   specification branch is closed; conformance tests alone do not close the
   project blocker. Activation may be its own cohesive protected commit, but
   the Current Plan remains on the same milestone until the project result and
   limitations are recorded.

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
