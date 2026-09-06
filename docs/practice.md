# Engineering and evidence practice

None of this is workflow. The four branch-and-main rules, and what they mean
exactly, are in `CLAUDE.md`; nothing here adds an approval point or a merge
condition. This is how to do the work well, and most of it was learned by
getting it wrong.

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

`mcts_mem/` can preserve durable design choices and rejected alternatives. It
does not authorize work or add a workflow step.

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

## Test boundary

`make check` is the single all-tests command rule 3 names. A green run states
only that the exact tested revision passed the repository's current test
inventory; it does not say the inventory is complete. Language safety remains
substantive: every source proof is checked in its current control-flow context
and erased before lowering, every partial operation is lowered only after
machine proof of its domain, and optional optimization facts may not change
acceptance, cleanup, or output.
