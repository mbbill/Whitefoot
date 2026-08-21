# Batch 0074 — proof-derived parallelism v1 (permission, ledger, actualization)

Branch: `par/proof-derived-parallelism` (worktree, from main 4f01bab6).
Authority: owner chartering direction, 2026-08-21 (recorded verbatim in
`research/investigations/proof-derived-parallelism/DESIGN.md` §0), under the
merge-boundary process landed in 93aedd79. The plan revision rides this
branch as PROPOSED and activates at merge.

## Scope

Implement DESIGN.md §2 "In" exactly: permission judgment P (four
conditions + claim-free eligibility), non-normative `--par-ledger`,
default-off runtime actualization (`WF_WORKERS`), spec CANDIDATE v0.34 with
the single [PAR-1] rule, compiler tests (grants, per-condition denials,
codegen, in-crate determinism repeat), demo workload + measured RESULTS,
durable landing of deciding research probes, current-plan (PROPOSED) and
roadmap PAR updates.

Exclusions (deferred with triggers, DESIGN.md §2 "Out"): `pal` marker;
Tier A/B loop machinery; I/O concurrency lane (sequenced first at plan
level, own packet); claim-bearing actualization / arbitration; heartbeat
policy; reduce-clause regrouping.

## Approval classes this batch will touch

- `spec/kernel-spec.md` bytes: YES — CANDIDATE v0.34 on the branch;
  activation is the approved merge's activation commit. Packet carries
  SHA-256, exact diff, impact inventory, grammar-verifier output.
- Protected conformance/compliance evidence: NO changes on the branch. Zero
  corpus cases added or modified (the rule changes no acceptance and no
  verdict; rationale in DESIGN.md §6). Gates and wiring untouched. One
  protected addition is nevertheless required before merge and is deliberately
  not landed here — see "Specification candidate v0.34" below.
- Repository root: no new entries.
- Rulings requested at merge (not blocking branch work): (a) permission
  attribution is not "undeclared parallelism" under the recorded
  auto-parallelism constraint (it changes no acceptance; declaration-shaped
  surface, the `pal` marker, is the named next packet); (b) runtime worker
  threads are TCB implementation below the language (no construct exists to
  carry a row; DESIGN.md §5).

## Specification candidate v0.34

Identity. `spec/kernel-spec.md`, titled `# Kernel Specification v0.34`,
declared `Status: CANDIDATE v0.34 supersedes v0.33 fc6b5a10...d08f`, at
SHA-256 `f3e26631c6f168cdcb0add1f1dec6a5e40867d7469150a3854f1878c56eec0f9`,
3,225 lines and 399,265 bytes. The digest is `shasum -a 256` on the installed
bytes; the compiler recomputes it independently in
`computed_identity_is_the_independently_measured_digest`. The activation chain
in `governance/APPROVALS.md` is untouched and still ends at v0.33.

Delta. One added rule, [PAR-1], in section 13; no existing rule's bytes
change. The section-13 heading widens from `Capabilities` to `Capabilities and
execution overlap` so the law is not filed under the capability stub. Rules
135 to 136; grammar productions, tokens, spellings, operations, and exception
clauses all +0/-0. The rule states when an implementation may overlap two
statements — the four permission conditions plus claim-free call closures —
and requires every observable of a permitted overlap to equal the source-order
execution's, with worker count, thread identity, and schedule explicitly
non-observable and the permission explicitly never an obligation.

Grammar verifier (`whitefoot-grammar` baseline candidate, the two-path native
verifier over the compiler's own lexer and parser): exit 0,
`grammar-preserving candidate verified by the active compiler: 74 productions,
93 decisions, 105 terminal predicates` — identical to v0.33's installed
inventory.

Impact inventory (`whitefoot-spec --index`): 136 rules. [PAR-1] occupies lines
1976 to 1995, 3,269 bytes, and references CAP-1, CLM-1, DIAG-3, EFF-1, EFF-2,
EFF-5, FN-1, OWN-7, and SCOPE-3. No rule references [PAR-1]: it is a leaf, so
the reference graph shows the addition reaching nothing else in the document.

Derived material. `spec/derivation/derivation-ledger.md` gains the [PAR-1] row
(existence-only, with its form debt stated) and a v0.34 candidate amendment
section; totals move to 84 derived, 52 existence-only, 0 underived across 136
rules. `compiler/src/spec.rs` and the generated `spec_identity.rs` name the
candidate. `backend/qualification.rs`'s per-activation review tripwire is
re-reviewed and bumped to v0.34: the review's one substantive finding is that
[PAR-1]'s row gate plus [EFF-2] propagation make all seven `external`/`blocks`
system operations unreachable from an overlapped statement at any call depth,
while the eight pure ones may appear inside one and need no row change.

Conformance delta: zero cases, and the reason is exactly that the rule is not
a source-to-verdict property. It adds no construct, so no program can be
written that this rule accepts or rejects; acceptance and facts-off lowering
are identical with the rule present or absent. This is the same class the
corpus already covers by annotation rather than by case — CAP-1, GATE-1,
LEDGER-1, OWN-9, DIAG-2, DIAG-3.

BLOCKER (E4, honest stop, not worked around). `make check` cannot be green on
this branch. `tests/conformance/runner.py coverage` takes its denominator from
the active specification's rule ids, so a 136th rule with no case and no
annotation is one uncovered rule and the target exits non-zero. Reproduction
from the worktree root: `python3 -B tests/conformance/runner.py coverage -v`
-> exit 1, `coverage (kernel-spec.md): 135/136 rules covered (116 by case
[+115/-55], 30 by annotation); 1 uncovered` and `uncovered: PAR-1`. The fix is
one annotation line in `tests/conformance/manifest.jsonl`.

Corrected at closure, on the batch audit's finding. The governance in force on
this branch does **not** forbid landing that line: CLAUDE.md "Merge-approval
boundary" places conformance and compliance evidence under branch autonomy, and
docs/WORKFLOW.md "The merge boundary" item 3 puts a protected-compliance change
in the merge packet with an exact before/after audit rather than behind a
mid-flight wait. The freeze was the **lead's scope decision** for this batch —
every executor brief said zero `tests/conformance/` changes — and the record
first stated it as project law, which is wrong and which the owner could have
checked and found wrong. The scope decision itself stands: the line is
presented here as the one protected addition the merge must land, with its
exact proposed bytes, and landing it is the merge's first act rather than a
branch commit nobody audited.

`{"rule": "PAR-1", "covered_by": "compiler-parallel-tests", "reason": "Execution overlap is an implementation liberty with no writer-emittable construct (SCOPE-1): the rule adds no syntax, changes no acceptance, and changes no verdict, so no source-to-verdict pair can assert or violate it. The permission judgment and each of its four denial conditions are covered by in-crate semantic tests, the ledger's per-site verdicts, and a backend determinism repeat test that byte-compares full output at several worker counts against the same module linked with no runtime at all."}`

Nothing else on the branch depends on that line: `make -C compiler check` is
green without it, and `make check`'s other targets — repository invariants,
spec append-only, spec archive integrity in candidate mode, spec digest sync,
and conformance structure — are green with the candidate declared.

Rulings requested at merge, in addition to (a) and (b) above: (c) [PAR-1]
states its four conditions as necessary, so every later widening (indexed
loops, buffer views, claim-bearing regions) is a specification amendment
rather than a checker improvement. The alternative — state only the
equivalence law and leave the judgment wholly to the implementation — was
considered and not taken, because DESIGN.md §6 specifies the stated-conditions
form; the cost is recorded in the [PAR-1] ledger row as that row's live debt.
(d) The runtime-as-weak-override decision E3 recorded below.

## Executor log

(One line per stage at completion; blockers recorded here honestly with
reproduction, never worked around.)

- E1 (permission judgment P): landed `semantic/permission.rs` (four
  conditions, claim-free eligibility, per-function pair/chain table on
  `CheckedProgramData`) plus `semantic/places.rs`, which lifts the [OWN-7]
  overlap relation, the holder/place prepass, and the [EFF-2] actual-side
  projection out of `entailment/flow.rs` so P reuses them instead of copying
  them; `expression_children` moved to `model.rs` for the same reason. Nine
  in-crate tests: three grants, one chain-stops control, four per-condition
  denials, one not-actualizable. `make -C compiler check` green (995 lib
  tests, +9, no other count moved). Three deviations from DESIGN §3, all
  fail-closed widenings, none narrowing the deliverable: (a) an
  `allocates(arena 'r)` row contributes the caller region as a written
  footprint element under condition 2, because two overlapped calls would
  both mutate one allocation list — untestable today, the arena runtime is
  an unsupported capability before a checked program exists; (b) an actual
  whose caller place P cannot resolve while its row projects an access denies
  the pair under condition 2; (c) P resolves a direct slice value's source
  place on its own side only, leaving [ENT-5] kill behavior byte-identical.

- E2 (permission ledger): landed `semantic/permission_ledger.rs`, which renders
  the permission table as the DESIGN section 4 developer lines, plus the
  `TreeView::source_line`/`path_spelling` citations it needs, a
  `permission_ledger` field on `CheckedProgramData`, the public
  `compile_with_permission_ledger` entry point (one pipeline: every other entry
  point is a projection of it), and `whitefootc --par-ledger`, which prints the
  lines on stdout and never on the mandatory record channel. The condition
  number in each denial line comes from `Denial::condition`, so the reported
  condition cannot drift from the judging one. Two decisions worth the audit's
  attention: byte-identical lines are collapsed, because the table is dense by
  `FunctionId` and one generic monomorphized twice would otherwise report one
  source site twice; and `--par-ledger` with `--emit-llvm` and no `-o` is
  refused rather than interleaving two streams into one stdout. Six tests:
  three driver-level ledger tests covering an eligible line, a
  not-actualizable line with its claim count, a denial line for each of
  conditions 1 to 4, and the empty-ledger/unchanged-module control; three
  `whitefootc` option tests. `make -C compiler check` green.

- E3 (runtime and lowering): landed `backend/par_runtime.c` (one lazily started
  pthread pool sized by `WF_WORKERS`, lanes claimed by atomic exchange and
  never queued, 8 MB worker stacks, no output and no global a Whitefoot
  construct can name), `IrFunction::overlaps` carrying each permitted and
  eligible chain into the IR by the values its calls define,
  `backend/emitter/parallel.rs` (frame, outlined `@wf_par_thunk_N`, lane offer,
  and the join whose refusal edge calls the same thunk on the same frame), the
  `whitefootc` link path, and seven backend tests. `make -C compiler check`
  green (1004 lib tests, +6 from E2's 998 plus one added after the first green
  run; every other target's count byte-identical). `make conformance-run`
  re-verified at `Pass=498 Skip=1`, unchanged, with zero changes to
  `tests/conformance/` or to the conformance adapter.

  Four things the audit should look at directly.

  (a) **The runtime is a strong override of a weak in-module refusal, not a
  link obligation.** A module that hands work out emits
  `define weak ptr @wf_par_try_fork` / `define weak void @wf_par_join`, both of
  which refuse every lane, and the runtime's own definitions replace them at
  link. DESIGN section 5 says "linked only when the module contains at least
  one eligible site", which whitefootc and both test harnesses still do on
  exactly that condition. The weak pair was added because the alternative —
  plain declarations — makes an overlapping module unlinkable without the
  runtime, and one link path that would then have needed the runtime wired
  into it is the conformance adapter, which is protected invocation wiring
  this batch may not touch. That constraint is what forced the question, but
  the answer stands on its own merits: the permission is never an obligation,
  so a program that merely could overlap must still link and run correctly
  with no runtime present. Recorded here so the owner can rule on it rather
  than discover it. The silent-failure risk it introduces — a link that keeps
  the weak refusal and is therefore sequential forever, which every ordinary
  test would pass — is closed by `the_runtime_replaces_the_modules_weak_refusal`,
  which links the real runtime with an observer that reports the runtime's own
  `wf_par_grants` counter at process exit and fails on zero.

  (b) **Lowering narrows the judgment, never widens it.** A group is a prefix
  of a permitted chain, cut where a member's `let` did not lower to one call
  definition, where a member left the block, or immediately after a member
  whose binding is addressed (promoting it reads the call's value between the
  offer and the join). `a_permitted_pair_whose_first_member_is_borrowed_is_not_handed_out`
  pins both halves of that last rule.

  (c) **`wf_par_grants`** is one exported counter in the runtime, incremented
  by a relaxed atomic on each grant. No Whitefoot construct can name it. It
  exists because an overlap test passes just as well against a pool that
  silently grants nothing, and E4 will want it for the measurement.

  (d) **`cost_shape`'s wfgrep symbol census gained two accounted names**
  (`wf_par_try_fork`, `wf_par_join`) and the thunk prefix. wfgrep has permitted
  eligible pairs, so it now offers lanes; no count assertion moved and no
  section 9.1 row was relaxed. The census's meaning — every call is accounted
  for by a named row — is preserved with one new named row.

  Measured on the E3 fixture (32-leaf tree, one recursive fold): granted lanes
  0 / 5 / 15 / 27 at `WF_WORKERS` 1 / 2 / 4 / 8, with byte-identical stdout at
  every count and against the same module linked with no runtime at all.

  One boundary the negative control does not cross: the repeat test's control
  proves the byte comparison reports an injected difference, not that a broken
  join would be caught. A join-skipping variant was considered and rejected as
  a gate test because its failure is a race and a flaky red gate is its own
  defect; `the_runtime_replaces_the_modules_weak_refusal` covers the one
  silent-success mode that mattered.

- E4 (spec candidate v0.34 and the document set): drafted [PAR-1] into section
  13 and declared the candidate status line; landed the derivation-ledger row
  and v0.34 candidate amendment, the compiler's candidate identity constants
  and regenerated `spec_identity.rs`, the re-reviewed v0.34 target-qualification
  tripwire, a PROPOSED `docs/current-plan.md` for this direction with the I/O
  lane sequenced first among remaining work, and the outline's revision-46 PAR
  updates. Grammar verifier reports no production, decision, or terminal-predicate
  delta; `--index` shows [PAR-1] as a leaf referencing nine rules and referenced
  by none. `make -C compiler check` green (1005 lib tests and every other target
  byte-identical to E3). Two things the next stage and the audit must carry: the
  spec version bump makes the `command_entry_row` review tripwire fire, so a
  `REVIEWED_FOR` bump with a written v0.34 qualification review is mandatory
  before any program compiles at all; and the recorded BLOCKER above — `make
  check` stops at conformance coverage because [PAR-1] is a 136th rule with no
  annotation, and the annotation is protected evidence this batch may not land.

- E5 (demo workload, measurement, durable evidence): landed
  `tests/programs/par_layout.wf`, one box-tree layout pass folded twice over one
  tree — once with a measure bounded by the metric table's own length, whose
  closure is claim-free, and once with a measure bounded by a caller-supplied
  band, whose closure carries one claim. Nothing else differs between the two
  folds, so the program isolates exactly what a claim costs: the ledger reports
  `pair(layout, layout) eligible` and `pair(layout_banded, layout_banded)
  not-actualizable: 1 claim site via measure_band`, and the emitted module shows
  the thunk, offer, and join in `@wf_layout` and no part of the runtime in
  `@wf_layout_banded`. Three integration cases in
  `compiler/tests/programs/parallel.rs` pin that split and byte-compare the
  published output at WF_WORKERS unset, 2, and 4; `CompiledProgram` gained
  `run_with_workers` and the harness gained `program_permission_ledger` so a
  corpus case reads the compiler's own ledger lines rather than re-deriving
  them. `make -C compiler check` green (1005 lib tests unchanged, integration
  programs 48 to 51); full `make check` still stops at the E4 conformance
  coverage BLOCKER above and nowhere else.

  Measured on Apple M4 (4 performance + 6 efficiency cores), interleaved
  rotation across worker counts, N=9, byte-comparing every run:
  whole program 715.5 / 491.4 / 398.8 / 400.5 ms minima at WF_WORKERS
  1 / 2 / 4 / 8, that is 1.46x, 1.79x, 1.79x. The same module linked with no
  runtime at all runs at 715.8 ms, statistically identical to WF_WORKERS=1, so
  the default-off path costs nothing measurable. Phase decomposition: the
  eligible fold alone scales 1.89x / 2.98x / 3.00x while the claim-bearing fold
  alone stays flat within 5% at every count, and the observed Amdahl share
  (66.1% eligible) composes with the eligible phase's own scaling to predict the
  whole-program figure to within 1% at every worker count. Grant counts read
  from the pool's own counter: 0 / 801 / 2529 / 8031 of 50,463 offers, so the
  lane-budget policy refuses 95% or more of all offers and the wins come from
  the few lanes near the root. All 109 runs published identical bytes.

  Three things the audit should look at directly. (a) The grain hazard is
  recorded as a measurement, not a caveat: an earlier draft of this same program
  with a 16-entry table and a depth-12 tree measured 0.08 s sequential and 0.17 s
  at four workers, a 2.1x slowdown, because the fork is offered at every branch
  node regardless of subtree size. That number is in RESULTS.md section 8 as the
  concrete case for the deferred heartbeat policy. (b) The integration case does
  not read the grant counter, so on its own it could pass against a pool that
  granted nothing; that mode is closed by E3's in-crate test, which does read the
  counter, and the doc comment says so rather than leaving the gap implicit.
  (c) `probes/README.md` records that the design cites two probes whose own
  hazard produces no ledger line, because the judgment analyzes ordered pairs
  of *adjacent* `let x = f(...)` statements and neither hazard is such a pair.
  They established requirements; the denials themselves are pinned by the
  compiler tests. The README states that rather than letting the files read as
  denial witnesses they are not. (Corrected at closure: this entry originally
  said both files "produce no ledger line at all". `g2_propagate.wf` is indeed
  silent; `g3_dep.wf` emits two lines, for the two pairs in its `main`. The
  README itself was accurate; this summary of it was not.)

  Also landed: `research/investigations/proof-derived-parallelism/RESULTS.md`
  (protocol, machine facts, the tables above, and eight stated limitations),
  ten deciding probes in `probes/` with a README naming what each settles, and
  the two roadmap Facts links E4 could not add because RESULTS.md did not exist
  yet. The roadmap `Revision` is deliberately left at 46: adding an evidence
  link changes no item's goal, current state, next gate, or disposition, and the
  PROPOSED plan's derivation from revision 46 stays true.

- Audit finder A (plan-vs-actual and governance): re-derived every load-bearing
  figure rather than reading it. Verified clean: zero `tests/conformance/`
  touches, no Makefile, hook, script, or conformance-adapter change anywhere in
  the branch diff, zero machine-local absolute paths in the diff, no new root
  entry, `CLAUDE.md`/`AGENTS.md`/`docs/WORKFLOW.md`/`governance/APPROVALS.md`
  byte-identical to main with the chain still ending at v0.33, and no released
  archive added, edited, renamed, or removed. Recomputed the candidate digest
  (`f3e26631...c0f9`, 3,225 lines, 399,265 bytes) and the v0.33 tail
  (`fc6b5a10...d08f`) from main's own bytes; re-ran the grammar verifier
  (74/93/105, exit 0) and `--index` (136 rules, PAR-1 at 1976-1995, 3,269
  bytes, nine refs, referenced by none) — every record figure exact. Re-ran
  `--par-ledger` over all ten probes and the demo (stdout only, stderr empty),
  re-ran the demo at WF_WORKERS unset/2/4/8 (identical published bytes), and
  confirmed in the emitted module that `@wf_layout` carries the thunk, offer,
  and join while `@wf_layout_banded` names no runtime symbol.
  `make -C compiler check` exits 0 at 1005 lib and 51 integration tests;
  `make check` exits 2, at the recorded conformance coverage stop and nowhere
  else. Fourteen findings, the two high ones being: (1) a new symbol collision
  — a program with a function named `par_try_fork`, `par_join`, or
  `par_thunk_N` is accepted by the checker and then rejected by clang with
  `invalid redefinition of function 'wf_par_try_fork'`, and `--emit-llvm`
  writes that invalid module at exit 0; and (2) the red gate, whose BLOCKER
  attributes the conformance freeze to project law when the governance in
  force (CLAUDE.md "Merge-approval boundary", WORKFLOW.md "The merge boundary"
  item 3) places conformance evidence under branch autonomy with a
  before/after audit in the merge packet — the freeze was the lead's scope
  decision, which the record should say instead. Also: DESIGN section 9's
  band-vs-derived-index residual gap was neither fixed nor recorded anywhere;
  the candidate rule folds claim-freedom into permission while the ledger and
  RESULTS call such pairs permitted; DESIGN section 7's join-skipping negative
  control was replaced by a control of a different mode; `collect_claim_sites`
  is the judgment's one fail-open wildcard; worker stacks are a fixed 8 MB
  rather than DESIGN section 5's "at least the main thread's"; the E5 log's
  claim that `g3_dep.wf` emits no ledger line is false (it emits two);
  `probes/d1_two_traps.wf` and its README present claims failing on the
  program's own inputs as normal usage, against the batch's own claim
  doctrine; and `probes/g3_base.wf` is a byte-identical copy of
  `probes/a2_bubble.wf`. Full findings with reproductions were handed to the
  orchestrator outside the repository.

- E6 (audit refutation, repair, closure): re-derived all twenty-four findings
  against the branch rather than reading them; every one survived, two only in
  part (A-9's README was accurate and its summary was not; B-F5's stated
  rationale did not survive the finder's own measurement). Repaired thirteen in
  three commits — the caller-side operand footprint in both directions, the
  reserved `wf__par_` symbol namespace, the worker stack floor and its refused
  return, the exhaustive claim walk, the conflict-kind ledger wording, the
  non-outlined differential, the join-removal control, the duplicate probe, and
  four evidence and comment corrections. Re-measured the demo against a
  compiler with no overlap lowering (1.64x at four workers, not 1.79x; ~1.2x
  cost with the feature off) and confirmed the grain hazard at 17x on `fib(38)`
  and 2.2x on `wfgrep`. `make -C compiler check` exits 0 at 1012 lib and 51
  integration tests; `make check` exits 2 at the same conformance-coverage stop
  and nowhere else. Two items are left for the owner rather than
  self-approved: the protected conformance annotation, and a three-sentence
  `[PAR-1]` amendment that was drafted, applied, and reverted because landing
  it requires editing the transcribed digest literal that independently checks
  the bytes being edited.

## Outcome

Closed 2026-08-21 after the adversarial batch audit and its repair pass. The
branch is complete as a merge candidate; it is **not** merged, and two items
below need the owner before it can be.

### Landed commits

Thirteen commits on `par/proof-derived-parallelism`, from `main` at `4f01bab6`:

| commit | what |
|--------|------|
| `20014ab0` | open the batch with the design contract |
| `93db3b01` | E1: the permission judgment P |
| `1b5f6b0f` | E2: the `--par-ledger` developer ledger |
| `3ad11eb2` | E3: runtime and lowering of eligible groups |
| `9eb0d013` | E3: the determinism repeat test |
| `787aa570` | E4: the v0.34 candidate `[PAR-1]` |
| `e210b77b` | E4: PROPOSED plan, roadmap revision 46 PAR updates |
| `935f2a81` | E5: the `par_layout` demo and its integration cases |
| `9bb6ec45` | E5: RESULTS.md and the deciding probes |
| `2eb126bc` | audit finder A's dispositions into this record |
| `e06e6da4` | **repair**: caller-side operand reads, reserved symbol prefix, worker stacks, exhaustive claim walk |
| `bfdcd6d1` | **repair**: both directions of the operand footprint |
| `99733604` | **repair**: join-removal negative control, evidence corrections |

Every commit carries the `Co-Authored-By` trailer. `git diff main...HEAD` contains
zero machine-local absolute paths, zero `tests/conformance/` changes, no
Makefile, hook, script, or conformance-adapter change, and no new repository
root entry.

### Gate at branch tip

- `make -C compiler check` -> **exit 0**. 1012 lib, 51 integration, 9 grammar,
  1 canonical, 18 spec, 3 whitefootc, 3 canonical-corpus, 1 ignored. Lib count
  is +7 over the audited tip (`9bb6ec45`, 1005): four permission-judgment
  denials, the runtime-shaped-names link, the non-outlined differential, and
  the join-removal control.
- `make check` -> **exit 2**, at the recorded conformance-coverage stop and
  nowhere else: `coverage (kernel-spec.md): 135/136 rules covered; 1
  uncovered`, `uncovered: PAR-1`. Everything before it — repository
  invariants, spec append-only, spec archive integrity in candidate mode, spec
  digest sync, conformance structure, and the 28 runner tests — is green. The
  BLOCKER above states the one line that closes it and who must land it.

### Specification candidate

Unchanged from E4 and re-verified at closure: `spec/kernel-spec.md`, title
`# Kernel Specification v0.34`, status line `CANDIDATE v0.34 supersedes v0.33
fc6b5a10...d08f`, SHA-256
`f3e26631c6f168cdcb0add1f1dec6a5e40867d7469150a3854f1878c56eec0f9`, 3,225
lines, 399,265 bytes. Grammar verifier exit 0 with no production, decision, or
terminal-predicate delta. `--index`: 136 rules, `[PAR-1]` at 1976 to 1995,
3,269 bytes, referencing nine rules and referenced by none. Conformance delta
zero cases. No released archive added, edited, renamed, or deleted; the
activation chain still ends at v0.33.

**One amendment to `[PAR-1]` is required and was deliberately not landed** —
see "Required before merge", item 2.

### Measurement headlines

From `research/investigations/proof-derived-parallelism/RESULTS.md`, Apple M4
(4 performance + 6 efficiency cores). Two sets, because the audit falsified the
basis of the first.

*As reported by E5*, `WF_WORKERS` 1/2/4/8 against the same module at
`WF_WORKERS=1`: 715.5 / 491.4 / 398.8 / 400.5 ms, that is 1.46x / 1.79x /
1.79x. Eligible phase alone 1.89x / 2.98x / 3.00x; claim-bearing phase flat
within 5%; observed Amdahl share 66.1% composes with the eligible phase's own
scaling to predict the whole-program figure within 1%. Grants 0 / 801 / 2529 /
8031 of 50,463 offers. All 109 runs published identical bytes.

*Corrected at closure* against a compiler with **no overlap lowering at all**
(built from `main`), which is the only honest baseline — best of 5 whole-process
wall times on the same demo: baseline 0.74 s, branch `WF_WORKERS=1` 0.88 s,
`=2` 0.61 s, `=4` 0.45 s, `=8` 0.45 s. So the outlining costs about **1.2x with
the feature off**, and the whole-program win at four workers is **1.64x**, not
1.79x. All five publish identical bytes.

*The grain result, which is the batch's most important number.* On `fib(38)` —
plain two-way recursion, `eligible` child pair, tiny body — the same source
through both compilers: baseline 0.07 s, branch with no runtime linked 0.15 s
(**2.1x slower**), `WF_WORKERS=1` 0.27 s (**3.9x**), `WF_WORKERS=4` 1.20 s
(**~17x**). On `wfgrep e compiler` over this repository: baseline 0.38 s,
branch unset 0.39 s, branch `WF_WORKERS=4` **0.88 s** (**2.2x slower**), with
byte-identical output throughout. `wfgrep` offers a lane per byte comparison of
every directory-entry sort. Nothing in v1 gates a fork on grain, and the
lane-budget policy's recorded rationale — worst case 0.69x — is two orders of
magnitude off on real programs.

### Audit findings and dispositions

Twenty-four numbered findings across two independent finders. Each was
re-derived at closure before disposition; the two marked REFUTED-IN-PART are
the only ones where the finder's claim did not survive unchanged.

| # | finder | severity | finding | disposition |
|---|--------|----------|---------|-------------|
| B-F1 | B | CRITICAL | P models only callee rows, so `let a = bump(slot: &uniq 'r cell); let b = take(v: cell);` is permitted and eligible; the lowering emits s2's operand evaluation after the lane offer, so the read moves across s1's call on every edge including a build with no runtime. Reproduced: branch publishes 1, `main` publishes 15. With lanes granted, a real data race. | **CONFIRMED, repaired** (`e06e6da4`). Condition 2 gains a caller-side operand footprint; the pair is now `condition 2: the write of s1 overlaps the operand read of s2`, and the program publishes 15. Two regression tests plus the subscript form. |
| A-1 | A | HIGH | `fn par_try_fork(...)` is accepted by the checker and then rejected by clang with `invalid redefinition of function 'wf_par_try_fork'`; `--emit-llvm` writes the invalid module at exit 0. | **CONFIRMED, repaired** (`e06e6da4`). Runtime symbols move to the reserved `wf__par_` prefix, unreachable from source because [FORM-3] spells IDENT `[a-z][a-z0-9_]*`. Regression test links a program declaring all three names. |
| B-F2 | B | HIGH | The determinism repeat's "sequential reference" is the same emitted module linked without the runtime, so no lowering defect can be seen. Nothing on the branch compares against a non-outlined build. | **CONFIRMED, repaired** (`e06e6da4`). `emit_without_overlap` lowers the same checked program with the permission table emptied; `the_overlapped_lowering_agrees_with_the_lowering_that_hands_nothing_out` compiles one source both ways and byte-compares at every worker count. |
| B-F3 | B | HIGH | "Default off costs nothing" and "the lane budget is never catastrophic (worst 0.69x)" are both false. | **CONFIRMED, re-measured, recorded** (RESULTS.md §4 correction and §8.8/8.9). Not repairable in this batch: grain gating is the deferred heartbeat policy. This is now the deciding evidence that it is the next required work, not a refinement. See "Required before merge", item 3. |
| A-2 | A | HIGH | The red gate's BLOCKER attributes the conformance freeze to project law; the governance in force puts conformance evidence under branch autonomy with a merge-packet audit. | **CONFIRMED, record corrected** (BLOCKER section above). The scope decision stands as a lead's call, now stated as one. |
| A-3 | A | MED-HIGH | `[PAR-1]` folds claim-freedom into permission while the ledger and RESULTS call such pairs permitted-but-not-actualizable. | **CONFIRMED, not repaired.** Requires spec bytes. Proposed amendment in "Required before merge", item 2. Nothing is unsound: the compiler overlaps strictly less than the rule permits. |
| A-4 | A | MEDIUM | DESIGN §9's "fix if the night allows, else record" residual gap — the checker's band-vs-derived-index discharge asymmetry — was neither fixed nor recorded anywhere. | **CONFIRMED, recorded** in the deferred register below. |
| A-5 / B-F5 | both | MEDIUM | DESIGN §7's join-skipping negative control was dropped for a control of a different mode; the stated "it would be flaky" rationale does not survive measurement (4-in-5 detection per run). | **CONFIRMED, repaired** (`99733604`). `the_repeat_reports_a_lowering_whose_joins_were_removed` strikes both join calls out of the branch's own module, links the real runtime, and requires at least one of twelve runs to disagree. Stable over five consecutive gate runs. |
| A-6 | A | MEDIUM | `collect_claim_sites`'s wildcard arm fails *open*: a future body-bearing statement would hide claims and widen eligibility. | **CONFIRMED, repaired** (`e06e6da4`). Both that walk and `push_nested_blocks` are now exhaustive. The compiler immediately caught `CheckedStatement::Evaluate`, which the finder's own enumeration had missed. |
| A-7 / B-F4 | both | MEDIUM | Worker stacks are a hardcoded 8 MB rather than DESIGN §5's "at least the main thread's", and `pthread_attr_setstacksize`'s return is discarded. | **CONFIRMED, repaired** (`e06e6da4`). The stack is `max(RLIMIT_STACK, 8 MB)` with absurd and infinite limits falling back to the floor, and a refused `setstacksize` now stops the pool instead of silently handing out 512 KB lanes. |
| A-8 | A | MEDIUM | `cost_shape`'s census allowlist claims the lane offer "reaches no host facility", which is true of the module and false of a shipped link. | **CONFIRMED, comment corrected** (`99733604`). No assertion moved; the claim now says which of the two configurations it describes. |
| A-9 | A | MEDIUM | The E5 log says `g3_dep.wf` produces no ledger line; it produces two. | **CONFIRMED, record corrected** above. The README itself was accurate. |
| A-10 | A | MEDIUM | `probes/d1_two_traps.wf` ships claims that are false on the program's own inputs, and the README presents it as ordinary usage, against the batch's own claim doctrine. | **CONFIRMED, evidence corrected** (`99733604`). The README now states plainly that the file is a deliberately unreviewed program, why it is kept, and that the ledger line it demonstrates needs no false claim. The same note covers `x2_spin.wf`'s vacuous claim. |
| A-11 | A | LOW-MED | `probes/g3_base.wf` is byte-identical to `probes/a2_bubble.wf`; the README describes them as settling different questions and miscounts their pairs. | **CONFIRMED, repaired** (`99733604`). `g3_base.wf` deleted; the dependence paragraph rewritten around the verdicts the compiler actually reports. |
| B-F6 | B | MEDIUM | DESIGN §1's "zero claim sites ⇒ zero trap sites" is false as written: the emitter also emits `abort()` edges (box/arena OOM, buffer fill, the defensive discriminant arm) that a claim-free closure reaches. | **CONFIRMED, recorded, no code change.** `[PAR-1]`'s `[SCOPE-3]` carve-out covers the OOM cases; the defensive discriminant arm is not resource exhaustion, and overlapping raises peak live allocation so an overlapped run can abort where the sequential one does not. The premise sentence should say "zero *claim* sites"; the deferred register carries it. |
| B-F7 | B | LOW-MED | An `arena<'r, T>` parameter is a nominal, so `footprint` derives no region from it and a `writes('r)` row projects nothing onto it. | **CONFIRMED, recorded in the source** (`99733604`). Not exploitable: every arena program stops at `UnsupportedSemanticFeature::ArenaRuntime`. The note sits on `Access::Arena` so the arena lane cannot inherit the gap silently. |
| B-F8 | B | LOW | The ledger renders every condition-2 conflict as "writes overlap", including read/write ones. | **CONFIRMED, repaired** (`e06e6da4`). A denial now carries the conflict kind the judging loop produced, and the ledger words all four. |
| B-F9 | B | LOW | `Denial::UnresolvedFootprint`, the `W(s2) ∩ R(s1)` direction, `Access::Arena` rendering, and `PairSide::Second` have no test. | **CONFIRMED, partly repaired** (`e06e6da4`, `bfdcd6d1`). The read/write direction and both operand directions now have per-kind tests. `UnresolvedFootprint` and `Access::Arena` rendering remain untested — the deferred register carries them. |
| B-F10 | B | LOW | Permission analysis is O(n²·(V+E)) per adjacent-call run and runs on every compile. | **CONFIRMED, deferred.** Measured cost on `wfgrep` (~100 functions) is not observable in the gate time. Memoizing the per-function claim closure removes it; the deferred register carries it. |
| A-12 | A | LOW | The implementation judges only *adjacent* pairs; E1's "three deviations" enumeration omits it. | **CONFIRMED, recorded here.** It is a narrowing and the module header documents it; the enumeration was incomplete, not the code. |
| A-13 | A | LOW | One docs-only commit; the spec commit carries no record update; E3's and E5's entries rode following commits. | **CONFIRMED, accepted.** Real drift from "record updates ride the work commits they describe". No repair; noted so the next batch does not repeat it. |
| A-14 | A | LOW | `PAR-1` names both an outline direction and a spec rule. | **CONFIRMED, deferred.** The `outline:` prefix disambiguates today. Renaming the outline id is a roadmap edit for the next planning pass. |
| B (routes probed) | B | — | Parent-vs-match-binder aliasing, transitive `external`, claims through generic instances, both directions of condition 2, chains vs non-adjacent conflicts, nested calls in argument position, field borrows, the pool handshake, and hostile `WF_WORKERS` values. | **No defect found.** Recorded so the next reviewer does not re-spend the time. |
| A (verified clean) | A | — | Conformance untouched, gate wiring untouched, no machine-local paths, no new root entry, governance files byte-identical to `main`, spec identity and archives exact, grammar verifier and `--index` reproduce, ledger on stdout only, demo deterministic at every worker count. | **Reproduced at closure** after the repairs, unchanged. |

### Required before merge (owner)

1. **The conformance annotation.** One line in `tests/conformance/manifest.jsonl`,
   exact bytes in the BLOCKER section above. Until it lands, `make check` exits
   2. This is a protected-evidence addition: it needs the owner's approval and
   an `governance/APPROVALS.md` entry with the merge. Before: 30 rules covered
   by annotation, 135/136 covered, `PAR-1` uncovered. After: 31 by annotation,
   136/136, nothing uncovered. No case is added, modified, deleted, or renamed;
   no verdict changes; no gate, collection path, or invocation wiring changes.
2. **One `[PAR-1]` amendment.** The rule as drafted states its footprints from
   the callee rows only, which is exactly the hole B-F1 fell through: it
   permits the `bump`/`take` pair and then demands an observable identity no
   implementation can honour for it. The compiler is now stricter than the rule,
   so nothing is unsound, but the rule should not teach the defect. The
   proposed change is three sentences and no grammar delta:
   - after the disjoint-footprints sentence, add: *"Evaluating a statement's
     own argument expressions is part of that statement and therefore part of
     the overlap, so each call's written footprint also overlaps no place the
     other statement's argument expressions read; taking the address of a place
     is not reading it, and both directions are required because which
     statement's argument evaluation an overlap moves is the implementation's
     choice."*
   - extend the unresolved-element sentence to *"...and so does a place read by
     an argument expression whose caller place the implementation does not
     resolve..."*
   - append to the `claim_stmt` condition: *"; this last condition is
     claim-freedom, and an implementation that reports the other conditions and
     claim-freedom as two separate outcomes reports exactly this one list"* —
     which also disposes of A-3.

   This was drafted, applied, and then reverted rather than landed, because
   changing the candidate's bytes requires changing the independently
   transcribed digest literal in `compiler/src/spec.rs`, and that literal is
   the external check on the runtime's own SHA-256. Editing it inside the same
   change that edits the bytes it checks is precisely the move that check
   exists to catch, so the amendment is presented for the owner rather than
   self-approved. The candidate on the branch is therefore the E4 bytes at
   `f3e26631...c0f9`, unchanged.
3. **A direction ruling on grain.** DESIGN §2 defers profitability policy to a
   heartbeat successor and DESIGN §5 justifies the lane budget as "never
   catastrophic". §8.8 and §8.9 of RESULTS falsify that: 17x on `fib(38)`, 2.2x
   on `wfgrep`, and an unconditional ~2x on fine-grained recursion with the
   feature off. Merging as-is ships a feature that is a large loss on the
   project's own flagship program whenever it is switched on, and a measurable
   loss on some programs when it is not. The three options are to merge with
   the deferral standing and the numbers recorded, to gate the fork on a grain
   estimate before merging, or to hold the branch. This is a direction call,
   not a defect, and it belongs to the owner.

### Rulings requested at merge (unchanged from above)

(a) permission attribution is not "undeclared parallelism" under the recorded
auto-parallelism constraint; (b) runtime worker threads are TCB implementation
below the language; (c) `[PAR-1]` states its conditions as necessary, so every
later widening is a specification amendment rather than a checker improvement;
(d) the runtime-as-weak-override decision E3 recorded.

### Deferred register

Carried out of this batch with the trigger that reopens each.

| item | trigger |
|------|---------|
| The `pal` marker (grammar, FORM tables, teaching text) | its own packet; PAL §6 is the agreed semantics |
| Indexed-loop permission (Tier A) and buffer-view splitting (Tier B) | recorded hazards OWN-9 granularity and c2-F4 |
| The I/O concurrency lane (completion-based family) | sequenced **first** among remaining work in the PROPOSED plan; w3 measured 2.83x vs 0.15% |
| Claim-bearing actualization and trap arbitration | when a claim-bearing region is worth actualizing; the EFF-4 two-half ruling and elision-rank arbitration remain on file |
| Heartbeat profitability policy | **now the next required work**, not a refinement — RESULTS §8.8/8.9 |
| `reduce`-clause regrouping | never needed while joins are source-fixed |
| The checker's band-vs-derived-index discharge asymmetry (DESIGN §9) | compiler defect candidate, zero spec bytes; it costs `par_layout`'s banded fold its eligibility today |
| DESIGN §1's "zero trap sites" premise | should read "zero *claim* sites"; the emitter's `abort()` edges are reachable from a claim-free closure (B-F6) |
| `Denial::UnresolvedFootprint` and `Access::Arena` ledger rendering have no test | when either path becomes reachable by a shipped program |
| Permission analysis is quadratic per adjacent-call run | when a generated program makes it observable; memoize the per-function claim closure |
| The arena footprint gap on `arena<'r, T>` parameters | before the arena runtime lands (B-F7); the note is on `Access::Arena` |
| `outline:PAR-1` and spec `[PAR-1]` share a name | next planning pass |
