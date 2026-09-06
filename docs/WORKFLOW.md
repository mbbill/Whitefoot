# Whitefoot branch and main boundary

This document contains the complete live approval and workflow policy. No
historical plan, batch record, approval entry, design note, or tool convention
adds another process requirement.

## The four rules

1. Any change may be made on a work branch without approval. This includes
   plans, repository layout, specifications, conformance evidence, gate wiring,
   code, tests, and documentation.
2. Every change merged into `main` requires owner approval of the exact
   revision to be merged.
3. The exact revision merged into `main` must pass all repository tests before
   the merge. The canonical command is `make check`.
4. If the merge changes `spec/kernel-spec.md` or conformance evidence,
   the pull request states what changed and its selection ground, answered against
   the merge.

These four conditions are sufficient. No plan status, branch charter, batch
record, worktree arrangement, audit, packet, rebase method, commit shape, or
other workflow step is an additional approval or merge precondition.

## Exact meanings

- **Work branch** means any branch other than `main`. Branch work never pauses
  for approval, including when it edits a specification, conformance evidence,
  or this policy.
- **Exact revision** means the complete tree that will enter `main`. If that
  tree changes after approval or after its successful test run, rules 2 and 3
  apply to the new revision.
- **All repository tests** means the root `make check` target. It includes the
  compiler build, formatting and lint checks, every maintained executable test
  target in the compiler and active research experiments, the specification
  and archive checks, conformance structure and coverage, and the full native
  conformance adapter, including its test otherwise marked ignored for ordinary
  Cargo runs. A file explicitly retained as a deferred or historical artifact
  that cannot run against the current toolchain is evidence, not a test target.
- **Conformance evidence** means `tests/conformance` case source and manifest
  content, its runner and adapter, and collection or invocation wiring that can
  change which cases run or how their results are interpreted.
- **Recorded approval content** identifies what was approved, not a larger
  process packet. For a specification change it includes the exact
  `spec/kernel-spec.md` identity. For a conformance change it includes the
  exact added, modified, deleted, or renamed content and its before/after
  boundary. An ordinary merge that changes neither needs no ledger entry.

Technical language, safety, archive-immutability, and evidence-integrity rules
still determine whether a change is correct. They do not introduce another
approval point or workflow prerequisite.

Imperative process text retained in historical plans, completed records,
design memory, or research evidence is superseded by these four rules. Terms
such as *validation*, *ratification*, and *approved implementation* in language
or design artifacts describe technical evidence or trust state, not permission
to work and not another merge condition.

## Engineering guidance (not workflow)

The following questions improve implementation quality but are not approval or
merge prerequisites:

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

`mcts_mem/` can preserve durable design choices and rejected alternatives. It
does not authorize work or add a workflow step.

## Technical failure categories (not workflow)

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

## Evidence guidance (not workflow)

Evidence can improve confidence, but none of the practices in this section is
an additional approval or merge condition beyond the four rules above.

- State exact commands, inputs, outputs, counts, and exit codes. Read an exit
  code directly, not through a pipe.
- Prefer differential reproduction on the same source before and after the
  change.
- Resolve every commit id, digest, path, and count with the relevant tool before
  writing it. Do not copy old measurements forward.
- When adding tests, verify the collected count increased as expected and that
  a deliberate negative control makes the check fail.
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

## Specification identity (not a separate workflow)

`spec/kernel-spec.md` always declares `Status: ACTIVE vN`, and its bytes are
that version's identity. An amendment lands as one change on a work branch:
the amended active file titled and declared vN+1; the outgoing vN bytes
archived as `spec/kernel-spec-vN.md`; and `compiler/src/spec_identity.rs`
regenerated with
`cargo run --bin whitefoot-spec -- --emit-identity src/spec_identity.rs`.
`make spec-append-only` checks that no released archive changed, and `make -C
compiler static` that the generated identity names the installed bytes. There is no separate candidate state: a branch carrying an
amended specification is merge-ready the moment its gate is green, and the
owner's merge approval of that exact revision is the activation. No live
document quotes the version or digest as the active authority; they live in
the chain and the generated identity only (`make spec-prose-integrity`).
Released versioned archives and existing approval records are append-only.
These are artifact-integrity properties, not additional approval or workflow
stages.

## Conformance integrity (not a separate workflow)

Never change a verdict, status, rule citation, coverage row, or baseline merely
to make a test green. A compiler limitation, internal error, timeout, or
unsupported feature cannot rewrite normative expectations. `make check` runs
the complete conformance adapter; rule 4 makes the exact changed conformance
content part of the recorded merge approval.

## Test boundary

`make check` is the single all-tests command required by rule 3. A green run
states only that the exact tested revision passed the repository's current test
inventory; it does not say the inventory is complete. Language safety remains
substantive: every source proof is checked in its current control-flow context
and erased before lowering, every partial operation is lowered only after
machine proof of its domain, and optional optimization facts may not change
acceptance, cleanup, or output.
