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
- Protected conformance/compliance evidence: NO changes. Zero corpus
  cases added or modified (the rule changes no acceptance and no verdict;
  rationale in DESIGN.md §6). Gates and wiring untouched.
- Repository root: no new entries.
- Rulings requested at merge (not blocking branch work): (a) permission
  attribution is not "undeclared parallelism" under the recorded
  auto-parallelism constraint (it changes no acceptance; declaration-shaped
  surface, the `pal` marker, is the named next packet); (b) runtime worker
  threads are TCB implementation below the language (no construct exists to
  carry a row; DESIGN.md §5).

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

## Outcome

(Filled at closure: landed commits, verification results, measurements,
audit dispositions.)
