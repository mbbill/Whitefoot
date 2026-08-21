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
one annotation line in
`tests/conformance/manifest.jsonl`, which is protected evidence this batch is
forbidden to touch; adding it silently would also be exactly the "regenerate
evidence to go green" move the constitution names. It is therefore presented
here as the protected addition the merge approval must name, with its exact
proposed bytes:

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
  (c) `probes/README.md` records that two probes the design cites,
  `g2_propagate.wf` and `g3_dep.wf`, produce no ledger line at all, because the
  judgment analyzes ordered pairs of *adjacent* `let x = f(...)` statements and
  neither hazard is such a pair. They established requirements; the denials
  themselves are pinned by the compiler tests. The README states that rather
  than letting the files read as denial witnesses they are not.

  Also landed: `research/investigations/proof-derived-parallelism/RESULTS.md`
  (protocol, machine facts, the tables above, and eight stated limitations),
  ten deciding probes in `probes/` with a README naming what each settles, and
  the two roadmap Facts links E4 could not add because RESULTS.md did not exist
  yet. The roadmap `Revision` is deliberately left at 46: adding an evidence
  link changes no item's goal, current state, next gate, or disposition, and the
  PROPOSED plan's derivation from revision 46 stays true.

## Outcome

(Filled at closure: landed commits, verification results, measurements,
audit dispositions.)
