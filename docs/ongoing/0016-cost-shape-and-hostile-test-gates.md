# 0016 — Cost-shape inspection and hostile test gates

**Claimed task.** Decomposed from `docs/current-plan.md` Work item 2 (wave 8
of 8; task 11 of 11 — final task before Work item 4). This record reports
how authorized work is carried out; it authorizes nothing beyond Work items
2 and 4 themselves.

- **Status:** `IN PROGRESS` — implementation complete and both gates green;
  awaiting lead review
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

- Completed: claim; the §9.1 gate module over the real `wfgrep` program
  (`compiler/src/backend/tests/cost_shape.rs`, 10 tests); the `Accept(0)`
  WriteZero behavioural case with its control; the preregistered
  initialization-cost measurement and its result; §12.2 coverage verified
  rather than duplicated. `make -C compiler check` and `make check` green by
  unpiped exit codes; lib tests 427 → 438.
- Current: awaiting lead review.
- Next: none in this task. Its evidence feeds Work item 4's `wfgrep`
  checkpoint.

### Gate inventory

Machine-checked, anchored on `tests/programs/wfgrep.wf`'s own emitted and
optimized module — ten §9.1 rows: target selection (one link-time table
decision, no dispatch, no indirect call); the argument lease; the raw byte
route and the absence of any Unicode gate; `RelativePath` retyping the
consumed lease; `open_read` as one `openat` on the capability's own
descriptor; the transfer path (one host call per source operation, no wrapper
residue, the [SYS-7] mapper outlined entirely out of `main`); close as one
discarded attempt; the value and `Output` releases reaching no host facility;
and one initialization per buffer at allocation. Machine-checked behavioural,
against task 0013's deterministic host: the output-batching row (3,000 matches
cost 2 host writes) and, from the same run, the release rows and §12.2's
per-byte-call rejection (6,000 bytes in and out for 9 host calls in total).

Measured, not gated: the initialization-cost row, in
`research/experiments/buffer-initialization-cost/`.

Verified as already covered, not duplicated: §12.2's effect
omission/addition canonical case (`reject-syseff-return-unit-pure.wf`,
`reject-syseff-declared-unexhibited.wf`,
`accept-sysrelease-return-unit-declared.wf`) and the primitive-lookalike items
(`accept-sysname-lookalike-outside-kind.wf`,
`reject-sysname-callee-outside-kind.wf`, `reject-systype-outside-kind.wf`);
and the four task 0013 injection cases, which 0013 landed green with controls
and this task consumes rather than rewrites.

Not claimed: the §9.1 UTF-8 row's Windows conversion column, which has no
first-slice implementation and therefore nothing to inspect. The `Output`
release row's second half is a recording obligation, discharged by 0013's
close/writeback observation rather than by a threshold.

### Measurement result

The dossier §11 stop condition did **not** fire. Whitefoot's drain over a
language-initialized reused buffer measures at practical parity with the
uninitialized native control §9.1 requires (1.0014, 95% interval
[0.9982, 1.0083], half-width 0.51%), and the same-source `calloc`/`malloc`
ablation is likewise parity (0.9985 [0.9935, 1.0071]). Because a one-page fill
is far below what whole-process timing resolves, the preregistered decisive
observable measured the cost directly: initializing one 4096-byte page costs
28.76 ns, which is 10,612x below 1% of the 256 MiB drain and 612x below 1% of
the program's 1.76 ms empty-input process floor — so no input size makes it
material. Full numbers and limits in that bundle's `RESULTS.md`.

### Correction carried out of task 0015

0015's closure recorded, informally, that `wfgrep`'s newline scan is
"recognized as memchr". It is not: the one `@memchr` call is
`relative_path`'s embedded-NUL check, and the scan is a scalar byte-at-a-time
loop that retains its bounds trap. No §9.1 row requires a `memchr` and §12.2's
per-byte-call rejection is satisfied, so this is a corrected note rather than
a defect; it is recorded in `cost_shape.rs` so the next reader does not
inherit the wrong shape. 0015's other four emitted-shape findings — four
allocations, one syscall site per operation, no `memcpy` libcall, nothing per
file, line, or match — are confirmed and are now standing gates.

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
