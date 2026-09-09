# Engineering and evidence practice

Use concrete compiler questions, independent behavior evidence, and measured
costs to guide engineering choices. The four branch-and-main rules are in
[AGENTS.md](../AGENTS.md#branch-and-main-boundary); completion checks are
collected in the [review checklist](review-checklist.md).

## Engineering guidance

These questions improve implementation quality:

1. What concrete compiler capability, real program, or experiment does this
   unlock?
2. What is the smallest general implementation?
3. Does it exercise the normal compiler path rather than a project, function,
   source-shape, corpus, or test special case?
4. Has supporting machinery become larger than the capability it serves?

The strongest implementation work has a concrete consumer, oracle, and cost
obligation. A project supplies pressure rather than language semantics, so
compiler changes remain general and project-independent. Performance work is
most useful when the loss is attributed with a same-source causal comparison
and a falsifier.

## Decision work

Use four occasions: start, choose, update, finish. A material choice changes
accepted behavior, a safety or trust condition, a shared interface or
representation, a significant performance commitment, or a standing project
rule. Restoring already specified behavior or editing prose without changing
its meaning is normally routine. If the work reveals a new design question,
use the choice step then; task size and file count do not determine this.
Scale the reasoning to uncertainty, impact, error cost, and reversibility.

Identify the delivery entry point and observable outcome from the user's
request at start or resumption. For a compiler, runtime, library, or performance
change, state how an ordinary user reaches the changed behavior. Keep this
brief in the existing PR or investigation; do not create a separate plan.
An explicitly requested experiment may deliver evidence alone. Otherwise,
an experimental link override, private entry point, or manual setup does not
establish delivery through the normal path.

An experimental deliverable does not waive the
[research implementation boundary](../AGENTS.md#repository-structure-and-hygiene).
Use maintained compiler components for compiler experiments; a separate
research implementation requires the specific owner permission defined there.

Check experimental validity and delivery completeness separately. Once an
experiment supports a candidate, carry the required integration forward as
unfinished work. If integration requires a directional design choice, surface
that choice rather than silently deferring integration or narrowing the goal.
Additional experiments, passing tests, and merging code do not substitute for
the requested observable outcome. Reassess the remaining work from the original
request at handoff, including necessary changes absent from the diff.

| Occasion | Action | Observable result / completion check |
|---|---|---|
| Start or resume | Read the requested outcome and scope, then the affected current owner: the specification for language behavior, compiler guide and code for implementation, or document role for prose. Newly found issues do not expand the task's scope. For a material choice, read the relevant constitutional clauses and follow the rule index to its reasons; walk the relevant memory branch and alternatives with the skill. On resumption, verify the actual working tree and PR state. | The work follows the relevant requirements and accounts for prior objections. No reading log or task document. A1, D3, R1. |
| Choose | State required properties, facts, assumptions, actual alternatives, the selection reason, and what could change it. Use deduction only for conclusions the stated premises entail; otherwise state the empirical or provisional ground. Before an experiment intended to select a design, record what result would distinguish the candidates; keep later exploration identifiable. | A concise reason in the existing investigation, or the PR for a small choice; experimental criteria and results at their source. R1, R2. |
| Update | When a choice is settled or its grounds change, update the standing owner and memory; update index rows for affected language rules. Follow references and material dependencies into consumers, reconsidering each affected choice. Continue if its conclusion or grounds change; stop at an unaffected dependency. Keep unresolved grounds explicit, with a concrete question and affected rows marked `revisit`. Do this when the conclusion is reached, including during long tasks. | Current guidance, recorded reasons, and the index agree. State the affected set and any unresolved reason in the existing explanation. R3, R4, M1–M3. |
| Finish | Run applicable mechanical checks, give another agent the task constraints, full diff and actual results, and use the completion checklist. Fix findings and recheck affected items, then publish the reviewed changes and compact report to the existing PR. | Check results, findings and limitations at the review surface. No separate review file or additional approval stage. V0–V4. |

For a choice without an existing rule or memory node, use the nearest relevant
owner and memory branch; do not require an index entry merely to begin.
A recorded reason names its material premises and dependent rules or interfaces
where known. Follow these links and search changed rule IDs or concepts to
find consumers; a search supplements reading and cannot prove completeness.
Choose where to write using the [document roles](review-checklist.md#document-roles).
The skill determines whether a memory node is warranted; the index covers
language rules, not every task or engineering choice.

The constitution supplies purpose, objectives, tradeoffs, and conditional
principles. It does not supply a unique solution. The active specification
defines the chosen language. MCTS-Mem owns concrete decisions and their
reasons, evidence, and replacement history. The
[rule-to-ground index](../spec/derivation/derivation-ledger.md#current-index)
connects active rules to those reasons and their direct technical sources.
This method governs their use; the checklist checks the resulting work.

**Compare against useful alternatives.** Assess major design directions
against effective existing approaches, including Rust where relevant. State
the task, baseline, expected benefit, and uncertainty. Compare performance,
resistance to unchecked shortcuts, and ordinary implementation quality where
they bear on the question. A local win does not establish an ecosystem-wide
advantage, and each reused construct need not separately outperform Rust.

**Distinguish the grounds.** A conditional deduction names its premises
and the conclusion they actually entail. Empirical support names what was
observed and under which conditions. A provisional choice names its reason,
uncertainty, and reopening condition. One decision can use all three. Explain
which claim each supports; a measured instance or a constitutional citation
does not prove a uniquely necessary mechanism or checker soundness.

Keep an unresolved question unresolved. When using an assumption to proceed,
name it as an assumption and state how it will be checked. Do not record or
cite a discussion proposal or an agent's default as a settled project decision.
When grounds change, reconsider the dependent choice; keep it only on stated
grounds that still hold, which may differ from its original reason.

### Maintaining the rule index

Keep one row per active rule in the existing ledger's current index. Its
four columns are the rule ID, basis kinds, review state, and source links with
their scope. Several rules can link to one shared decision. The linked reason
owns the detailed argument, relevant constitutional aims, assumptions,
alternatives, and reopening condition; the index is not a second decision
record. Use `deduction`, `empirical`, or `provisional`, joined with `+` when
needed. `current` means the ground has been assessed for the present question,
not that the design is proved optimal or implemented correctly.

During migration, use `unassessed` with `revisit` for legacy grounds that have
not been reassessed. Preserve their actual source and open conditions; never
translate `derived` or `derived_existence_only` mechanically into a claim of
logical or empirical support. On the next material change to that rule or its
reason, read the linked evidence, classify the supported claims, and replace
the marker with an assessed ground or an explicit unresolved question. New
rules need stated grounds; `unassessed` is not a shortcut for documenting a
new choice. An ordinary implementation fix need not clear unrelated legacy
markers.

When a rule is added, amended, or retired, add, update, or remove its current
row. Retain useful dated evidence and skill-managed history. When a reason
changes or moves, check rows that cite it and the directly affected standing
guidance; changing an index row alone cannot repair a false source. Follow
actual premise dependencies rather than treating every related link as an
implication. Explain the affected set in the existing PR or investigation,
including any unresolved `revisit` entries. There is no calendar sweep or
requirement to load every rule for every task.

Run `make -C compiler spec` after index changes. It checks unique active-rule
coverage, recognized basis/state fields, and a source reference in each row.
It does not assess the truth or sufficiency of the reason. Check reference
targets and meaning in the affected set at completion. After memory edits,
follow the current `mcts-mem-use` skill's verification, provenance, and history
instructions. Checker setup and invocation belong to the skill.
Do not use an index status as a source acceptance rule or an extra approval
condition; an unresolved safety objection still requires substantive resolution.

Reconsider the method itself when a task exposes a missed dependency,
unsupported conclusion, repeated owner correction, or upkeep that displaces
useful compiler work. Repair the specific trigger, owner, or check that failed
and record a changed decision in the workflow memory. This is also a material
choice; adding more process without a demonstrated use is not the remedy.

## Documentation and local context

Use the [document roles and citation boundaries](review-checklist.md#document-roles)
when choosing where to write. Keep a summary short enough to point to its
owner instead of copying the owner's changing details.

For a change, identify the concepts it changes and follow their rule IDs,
interfaces, and references into the affected documents. Read those sections
with their surrounding qualifications. Update the standing text when the
decision is settled, then record the reason and replaced alternatives in the
owning memory node. A new dated fact beneath an old instruction leaves two
conflicting instructions; it is not an update to the old one.

At task completion, use the [review checklist](review-checklist.md) to check
content placement, references, examples and consistency in this affected set.
Mechanical checks establish paths and executable behavior, not consistency
of the prose or suitability for its reader.

Invalid legacy memory formatting needs a documented repair, not a lint waiver.
Identify the original Git revision and account for each changed entry; preserve
its claims, dates, experimental limits, and actual alternatives. Correct only
supported metadata or classification errors; append substantive corrections.
Review the repair against that original revision before committing it, then run
lint on the committed tree. Its HEAD-based append-only check is not evidence
that the historical repair preserved meaning.

Historical essays and experimental records keep their original conditions;
do not append a second implementation inventory to keep an old essay
apparently current. The roadmap remains outside the working loop. A retained
investigation need not be moved when implementation lands: its design and
measurements remain useful evidence, while the implementation README changes.

## Feedback and implementation boundaries

Use the compiler README's focused development commands for the part being
changed; run the complete root `make check` for the exact merge revision.
Choose additional checks for a concrete uncertainty, not merely to repeat a
successful run. A documentation edit needs relevant reference and example
checks; it does not need a test that mirrors its prose.

When improving a diagnostic, expose the operation, required fact, relevant
location, and missing or invalidated evidence. Keep source rejection,
unsupported capability, target failure, and internal compiler failure distinct.
Measure the writer's repair loop as well as the compiler's individual stages.
Proof checking should be profiled by formation, automatic derivation,
certificate checking, and fact propagation before attributing a cost to `use`.

Keep code organized around the invariants it owns. When a change exposes an
ambiguous state flag or duplicated rule, prefer a precise representation and
one owner of that decision. Split a large file when that clarifies a real
responsibility; a line-count split or forwarding layer alone does not improve
local reasoning.

Writer trials can measure completion cost, repair attempts, required context,
and cross-module rework. Performance comparisons also need a defined workload
and attribution. These are separate observations: a program can be easy to
write without being fast, and can be fast without the proof system causing
the improvement. Agent collaboration models remain experiments rather than a
mandatory project workflow.

## Technical failure categories

- **Compiler defect:** implemented behavior contradicts the active spec. Add
  the smallest regression and fix the normal path without changing normative
  expectations.
- **Unsupported specified capability:** report it as unsupported rather than
  invalid source.
- **Conformance-evidence issue:** keep the active spec authoritative and do not
  disguise a compiler gap as a normative verdict.
- **Research or performance question:** run the cheapest bounded probe with
  a hypothesis, observable, and stop condition.
- **Language gap:** distinguish the minimal semantic witness from the compiler
  implementation that exposed it.
- **Project-local issue:** adapt the project when the frozen contract is
  preserved, rather than generalizing the language or compiler.

A soundness defect is a correctness issue regardless of planning status.

## Evidence guidance

Use the constitution to identify objectives and candidate directions, then
use technical arguments and experiments to decide between them. Keep four
things distinct: the desired property, the assumptions behind a deduction,
the chosen mechanism, and the observations supporting it. More than one
mechanism may meet the objective. A minimality choice can remain provisional
without an invented experiment or a claim of unique necessity.

Before real projects use Whitefoot, exclude the migration cost of existing
language designs from language-selection arguments. Updating the implementation,
tests, examples, or design documents is not evidence against a broad language
change. Internal test and example counts or spelling distributions do not
establish adoption, familiarity, or frequency in real use. Tests check the
specification and implementation; when a language rule changes, update its
tests to preserve their verification purpose under the amended rule.

Choose a probe that could distinguish the live alternatives. Keep behavior,
contracts, workloads, and comparison conditions fixed where they define the
question. If an agent makes the task easier by weakening a requirement, the
new green result does not answer the original question. A representative
writer trial can expose that failure and test a possible constraint or
diagnostic; it cannot establish that the language knows unstated requirements.

When evaluating implementation delegated to agents, distinguish at least these
observations:

- whether the required implementation and proof can be expressed;
- whether the tested agent can produce them with the supplied interfaces,
  context, tools, and repair assistance;
- whether separately implemented components meet independent behavior
  expectations when composed; and
- whether the resulting program meets its runtime cost goal, and why.

A failed trial can reveal inadequate contracts, missing proof vocabulary,
poor feedback, model limitations, or a bad architecture. Attribute the cause
before selecting a language change. Model identity and assistance are
experimental conditions, not permanent language ceilings. A restriction can
still be worthwhile when it increases writing effort; measure the benefit
and cost rather than treating brevity as success.

Keep conclusions conditional. Record what would make a rejected alternative
worth reopening and preserve the failure it must address. Improved agents can
change an authoring-cost result; they do not invalidate a counterexample to
soundness. A result on a retired compiler or a different workload remains
evidence about those conditions until reproduced on the new ones. The
relevant [decision memory](../mcts_mem/) records choices and their reasons;
the experiment or design remains the source of the technical evidence.

- State exact commands, inputs, outputs, counts, and exit codes. Read an exit
  code directly, not through a pipe.
- Prefer differential reproduction on the same source before and after the
  change.
- Resolve every commit id, digest, path, and count with the relevant tool before
  writing it. Do not copy old measurements forward.
- If diagnostic ordering, precedence, or rule citation may move, compare every
  affected case's result and cited rule across both binaries; an unchanged
  failure set is insufficient.
- A peer report is a lead rather than independent evidence. Label any
  unverified part; if a probe did not isolate the hypothesis, write `not measured`.
- Every new check states what a green run does and does not establish.
- A test earns its runtime with its purpose, never with its duration: a slow
  test is not thereby a thorough test. Shared setup avoids unjustified
  repetition unless isolation or repetition is itself the property under test.

### The failures that look like success

Most defects announce themselves. A handful do not, and every one of them was
found here by a deliberate question rather than by a gate, because **their
failure mode is success**: a conformance case that passes while testing
nothing, a check that cannot fail, a transform verified against its own
output, an operation performed against a baseline that no longer describes
reality. Nothing that watches for failure sees any of them.

The one habit that reaches all of them is to **prefer the observation that
separates two hypotheses over one consistent with the hypothesis you already
hold**. Before running a check, ask what result would make you believe the
other thing; if no result would, the check is decorative. Worked instances:

- A clean working tree *and* HEAD containing the fix — either alone is equally
  consistent with the fix having been destroyed.
- A test that MOVED to a different error versus one that STAYED PUT: moving
  means the fix worked and a second cause is underneath; staying means it did
  not work. The pass count is identical either way.
- Breaking a check in each direction it can fail, not once. A wrong value and
  a missing entry should fail differently; proving both is what separates a
  real check from a decorative one.

Three corollaries: run a transform against the input it should have handled,
never against its own output — a migrator or renderer checked on what it
produced is a fixed point and always agrees with itself. A mask's fix is
itself a probe — read the run immediately after removing one instead of
treating it as confirmation; a mask means the number of hidden problems is
unknown, never one. When a migrated case behaves oddly, read the migration
diff before the compiler — the program may have stopped being the program the
case was written about.

When writing rules like these, state the **property** that produces the
failure, not the causes you happen to have met; a cause list is wrong in both
directions at once.

## Test boundary

`make check` is the single all-tests command rule 3 names. A green run states
only that the exact tested revision passed the repository's current test
inventory; it does not say the inventory is complete. Language safety remains
substantive: every source proof is checked in its current control-flow context
and erased before lowering, every partial operation is lowered only after
machine proof of its domain, and optional optimization facts may not change
acceptance, cleanup, or output.
