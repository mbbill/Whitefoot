# 0016 — Cost-shape inspection and hostile test gates

**Claimed task.** Decomposed from `docs/current-plan.md` Work item 2 (wave 8
of 8; task 11 of 11 — final task before Work item 4). This record reports
how authorized work is carried out; it authorizes nothing beyond Work items
2 and 4 themselves.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, eighth bullet
  ("the §9.1 cost and §12.2 hostile test gates"), and Work item 4 ("return
  to the `wfgrep` checkpoint"), whose validation this task's evidence
  feeds. Also implements the plan's Verification bullet ("the §9.1 native
  cost shape is inspected on emitted code ..."). Implements dossier §9.1's
  table, the remainder of §12.2's rejection-gate list, and `QUAL-3` as the
  qualification-side counterpart to the cost-shape claim.
- **Owner:** executor agent `exec-0016`
- **Base revision:** `36dbc47` (main, "docs: close task 0014")
- **Workspace:** worktree branch `worktree-agent-a5c67d0970c39e8c1`

## Progress

- Completed: claim.
- Current: the §9.1 cost-shape gates over the optimized `wfgrep` module.
- Next: the `Accept(0)` WriteZero behavioural case; the initialization-cost
  measurement; coverage verification for the already-landed §12.2 items.

## Goal

Close out the remaining inspection-only or fault-injection-only §9.1/§12.2
obligations that task 0014's conformance corpus explicitly excludes (see
task 0014's Direction and invariants): structural inspection of emitted
LLVM/assembly for every hot path (no allocation, copy, dispatch, handle
lookup, or lock; one reusable output buffer; one buffer-initialization on
allocation with reuse across reads; the uninitialized-control
initialization-cost comparison), plus the subset of the OS-level
integration-test lane that genuinely needs task 0013's deterministic test
target rather than a real OS mechanism: close-error/no-fd-retry behavior,
mid-stream `ReadFailed`, a forced short write, and an output sink that
fails only at close or writeback.

**Fault-injection-only premise.** Per the lead's placement ruling, this
task does **not** claim the broken-pipe case or the symlink/changing-file
filesystem arrangements from that same lane — those exercise the real
compiled program against real, portable, deterministic OS mechanisms and
belong to task 0015, not this task. This task's four remaining cases are
scoped exactly to what genuinely cannot be triggered deterministically
without task 0013's fake target; do not pull in a case that a real OS
mechanism can already exercise portably.

## Direction and invariants

- Per §9.1's own design, the output-batching/buffer-init-reuse/
  initialization-cost rows "carry no threshold by design" — these are
  structural inspections and one controlled comparison, not pass/fail
  conformance verdicts with an arbitrary numeric threshold attached.
- The two buffer-cost rows need different controls: an initialized
  control answers "does initialization happen once"; an uninitialized
  control answers "is paying for initialization material at all." Do not
  conflate them or reuse one control for both questions.
- A close-error/no-retry claim is a target-code property (never retry a
  POSIX fd after `EINTR`), verified either by code inspection of the
  emitted close lowering or by exercising task 0013's fake target — not by
  trying to force a real `EINTR` deterministically. The same reasoning
  applies to a forced short write, a forced mid-stream `ReadFailed`, and a
  close/writeback-only failure: use task 0013's fake target rather than
  attempting non-portable real-OS timing.
- A broken pipe (a downstream reader that closes immediately) is a real,
  portable, deterministic OS mechanism and does not need task 0013's fake
  target — it, the symlink-policy witness, and the changing-file witness
  are task 0015's, exercised against the real wfgrep program with its own
  end-to-end harness.
- This task inspects and gates; it must not weaken tasks 0011/0012/0015's
  implementation to make an inspection pass. Any observed structural
  violation is a reproducible blocker per `docs/WORKFLOW.md`'s
  project-gates rule and routes to Work item 4's attribution step — never
  a silent pass.

## Method

Reuse the `emitted_function`-style LLVM-text inspection pattern already
used for other operation families (for example
`compiler/src/backend/tests/effect_attributes.rs`) to assert the absence
of forbidden call/instruction shapes on each hot path §9.1's table names.
Use task 0013's deterministic test target for the close-error,
mid-stream-`ReadFailed`, short-write, and close/writeback-failure cases.
Place the buffer-initialization-cost comparison under `research/`, per
`tests/codegen/README.md`'s existing guidance that "runtime and code-shape
measurements" belong there rather than in an every-commit gate, with a
stated hypothesis, observable, and stop condition per
`docs/WORKFLOW.md`'s performance-attribution rules.

## Scope and expected touch set

- A new file or files under `compiler/src/backend/tests/` (or a sibling
  location) for the codegen-shape inspections
- Tests consuming task 0013's deterministic test target for the two
  fault-injection cases
- `research/` — one bounded, self-contained note or experiment for the
  initialization-cost comparison, per the standing research-lane rules;
  not a permanent script.

## Dependencies and integration order

Depends on task 0011, task 0012 (real lowering to inspect), task 0013
(deterministic target for fault injection), and tasks 0014/0015 (shares
their harness and fixture conventions). Last task before Work item 4's
"return to the `wfgrep` checkpoint" step.

## Validation

Every §9.1 row has either a passing structural inspection or a recorded,
attributed measurement; the close-error/no-retry and close/writeback-failure
cases pass against task 0013's fake target; `make -C compiler check` and
`make check` green. A claimed task lands only through lead review per the
executor lane in `docs/WORKFLOW.md`.

## Done-when

Every §9.1 cost-shape row and every remaining §12.2 rejection gate is
observed with recorded evidence (per the current plan's own Done-when),
closing the last Work-item-2 task before Work item 4's wfgrep-checkpoint
validation.
