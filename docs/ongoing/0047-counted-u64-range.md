# 0047 — counted `u64` range loop

- **Status:** `IN PROGRESS`
- **Authority:** the ACTIVE stage-6 step in `docs/current-plan.md`, derived from
  Direction Outline revision 25
- **Owner / workspace:** Codex lead /
  `/Users/bytedance/do_not_scan/wf-0045-final-activation`, branch
  `codex/0045-ent5-activation`
- **Base revision:** `5683c855f796c2163c36f5e9ca4bcb566c13a6f7`

## Goal

Activate one ascending, unit-stride, half-open counted range over captured
`own u64` endpoints and implement it through the ordinary frontend, semantic,
entailment, lowering, backend, conformance, and real-program paths. The real
SHA-256 program must replace exactly three counted index loops, remove exactly
four claims, discharge all 9/9 index obligations without S2/S3, become `pure`,
and preserve its exact digest and emitted no-trap shape.

## Direction and invariants

- The sole new source form is canonical
  `for @label i in lower..upper { ... }`. `for` and `in` are fixed lowercase
  words; `..` is one attached compound terminal. Endpoints are term-or-constant
  `own u64` atoms evaluated once, left to right, into compiler-owned immutable
  captures. The range is ascending, half-open, and zero-trip when
  `lower >= upper`; there is no step, reverse, iterator protocol, or `continue`.
- The binder is an immutable compiler-updated `own u64` body local and is never
  a writable root. Source cannot set it, uniquely borrow it, or pass its
  storage to a callee write. Counted bodies inherit both [OWN-11] repeated-body
  restrictions. Ordinary `loop @label` remains unchanged.
- Normal body fallthrough performs body teardown and reverse cleanup, then the
  hidden increment and backedge. Matching/enclosing break, return, and
  propagated error clean up exactly once and never increment. Guard-false
  exhaustion does not repeat body cleanup. The `u64::MAX` boundary introduces
  no overflow and no writer-visible trap.
- Compiler-owned endpoint terms have stable private identity from the counted
  node and endpoint side. New source S11 (never the retired S8)
  establishes capture equalities at the preheader and
  `lower_capture <= i < upper_capture` at each body entry. Existing closure,
  support, kills, and S7 perform all further discharge; no general induction,
  accumulator relation, or ordinary-loop fact is added.
- Counted continuation is not ordinary `loop_stmt`'s empty-break join. It joins
  the real guard-false exhaustion edge and breaks naming this range after
  cleanup and scope kills. Enclosing breaks and function exits do not join.
  Binder and captures are out of scope before the join, and no exhaustion or
  postcondition fact escapes. Header state carries outer bindings, captures,
  and binder; continuation/local-break state carries outer bindings only.
- The active specification stays at `spec/kernel-spec.md`. Activation creates
  the immutable outgoing archive `spec/kernel-spec-v0.24.md` and installs
  v0.25 at the stable path; no `spec/kernel-spec-v0.25.md` is created. Exact
  bytes, complete digest, active pins, approval/activation chain, derived
  material, and implementation land as one coherent branch activation.
- Protected conformance changes are limited to rederiving the stale source-doc
  and manifest reason/doc for `gram6-pos-no-operators`; its id, GRAM-6 rule,
  source behavior, and Run verdict remain fixed. All other protected verdicts,
  rules, and sources remain byte-stable unless a separately reviewed impact
  stops the task.

## Method

1. Freeze the v0.24 identity, grammar inventory, SHA source/oracle, protected
   corpus, and exact declaration/use census for the new fixed words. Prove or
   stop on any unplanned `for`/`in` identifier collision.
2. Draft the smallest complete v0.25 stable-file delta, including lexical
   partition, canonical grammar, node/control graph, label/binding scope,
   endpoint snapshots, ownership, cleanup, structural entailment source,
   counted continuation, and unchanged ordinary-loop boundary. Verify both
   native grammar paths and hostile seam mutants, then independently review
   the complete stable-file diff and digest under the owner's delegated
   branch-revision authority before implementation. Freeze that reviewed
   candidate identity; any later spec-byte change returns through this step.
3. Implement one checked counted-range representation from parser through
   resolution, semantic checking, facts and obligation discharge. Keep endpoint
   capture identity private and finite; reject source writes/unique aliases to
   the binder and extend [OWN-11] by construct class rather than source shape.
4. Lower a dedicated preheader/header/body/exit CFG with distinct header and
   exit carried sets. Reuse existing cleanup and labelled-exit machinery, but
   do not desugar to a pre-tested ordinary loop or teach the wide-probe
   recognizer this form.
5. Add focused positive, negative, near-miss, invalidation, cleanup, lowering,
   backend, canonical, and conformance evidence. Migrate only the three SHA
   index loops and directly verify `sha256_abc_word_zero() == 3128432319_u32`
   in addition to the sustained aggregate oracle.
6. Recompute and compare the final spec identity with the pre-implementation
   reviewed bytes, and independently review the protected prose delta and
   compiler behavior. Activate atomically, rerun all gates plus the ignored
   adapter, record exact results in canonical owners, update the design tree
   through its skill, and close this record before advancing to stage 7.

## Progress

- **Completed:** task 0046 closed at `5683c85`; revision 25 selected this slice.
  The proof and surface-form design nodes and the rejected retained-check
  alternative were consulted. Independent compiler/source walks confirmed the
  SHA target is three loops, four claims, and nine obligations, and identified
  the required dedicated exhaustion edge and split carried sets.
- **Current:** freeze exact v0.24/compiler/corpus baselines and assemble the
  v0.25 normative delta plus regression-first implementation inventory.
- **Next:** land hostile frontend and semantic regressions, then implement the
  general construct without touching the SHA consumer until the mechanism is
  independently green.

## Scope and expected touch set

- Specification/governance: `spec/kernel-spec.md`, new outgoing
  `spec/kernel-spec-v0.24.md`, `spec/derivation/derivation-ledger.md`,
  `governance/APPROVALS.md`, compiler/conformance spec identity pins, and the
  stable-spec workflow records that require a v0.25 activation entry. No new
  governance candidate copy.
- Frontend: `compiler/src/lexer/scanner.rs`, `compiler/src/syntax/terminal.rs`,
  `compiler/src/syntax/grammar.rs`, generated grammar data, parser tree/engine,
  canonical renderer, resolution roles/scopes/labels, and their focused tests.
- Checked semantics: `compiler/src/semantic/model.rs`, control/loop checking,
  expression/set/borrow/effect checks needed for binder immutability and
  [OWN-11], cleanup edges, entailment terms/state/flow/sources, and tests.
- Lowering/backend: `compiler/src/lowering/builder.rs`, dedicated loop lowering,
  storage/results/cleanup integration, typed IR tests, backend behavior/effect
  tests, and program code-shape tests. The ordinary-loop wide-probe recognizer
  is a negative control, not an implementation target.
- Evidence/consumers: additive `tests/conformance/cases/` rows and
  `tests/conformance/manifest.jsonl`; the three approved prose fields for
  `gram6-pos-no-operators`; `tests/programs/sha256_abc.wf` and
  `compiler/tests/programs/hashing.rs`; installed acceptance evidence,
  `docs/patterns.md` as the writer-form canonical owner only when the exact
  candidate activates, `docs/roadmap.md`, `docs/current-plan.md`, this record,
  and the relevant `mcts_mem/` proof/surface-form nodes through the skill
  workflow.

## Dependencies and integration order

- Terminal task 0046 and commit `5683c85` are the premise. No other ongoing
  task may activate a specification or change the proof/loop contract in
  parallel.
- The normative delta and hostile tests precede the implementation; the
  general compiler path precedes SHA migration. Exact identity/approval chain,
  stable bytes, archive, pins, protected prose, generated data, and compiler
  behavior integrate atomically. Post-activation acceptance and canonical
  closure may follow only after that tree is green.
- Stage 7, held provenance activation, ensures, ledger, and deny-claims remain
  outside this task. A positive closure replaces the rolling plan before any
  of them begins.

## Validation

- Lexical/grammar: exact `for`/`in` role census; `..` maximal-munch and numeric
  seams; fixed-word reservation; mandatory label/binder/endpoints; canonical
  nesting; all noncanonical and out-of-scope forms; both native grammar paths;
  generated-data identity and canonical idempotence.
- Semantic/ownership: once-only endpoint capture despite source mutation;
  reject set/unique borrow/callee write of binder; reject outer affine move and
  outer-region borrow while accepting body-local counterparts; label,
  shadowing, scope, type, non-term, and endpoint-self-use failures.
- Control/runtime: empty, reversed, singleton,
  `18446744073709551614_u64..18446744073709551615_u64`, and MAX..MAX;
  matching/enclosing breaks, return, propagate error, nested ranges, body-local
  affine/shared-borrow cleanup, exactly-once release, and no hidden trap.
- Entailment: both structural bounds at each body entry; real support kills;
  safe S7 `i-k` composition; no facts after exit; reachable non-contradictory
  break-free continuation; zero-trip imports no body facts; early break gets no
  exhaustion fact. Carried `j`, access at `i +wrap 1` against the same upper,
  upper-without-`upper<=len`, and insufficient-lower `i-k` remain unproved.
  Empty/reversed break-free range followed by unproved OOB must reject.
- SHA: exactly three loops migrate; exactly four claims disappear; 9/9
  subscripts discharge without S2/S3; function is `pure`; direct result is
  `3128432319_u32`; sustained output is unchanged; no `wf_trap`; rotate and
  schedule-address shapes remain; unrelated ordinary loop remains ordinary.
- Integration: active/archive digests and chain; archive/append-only gates;
  derivation coverage; both grammar paths; focused frontend/semantic/lowering/
  backend suites; `make -C compiler check`; `make check`; complete ignored
  adapter tally and rule attribution; installed acceptance buckets; MCTS lint;
  zero unreviewed corpus verdict or observable-behavior drift.

## Stop condition

Stop with the smallest reproducer if endpoint snapshots cannot have finite
checked identity, the MAX edge requires a hidden runtime trap, the nine SHA
obligations require general induction, fixed-word reservation has an unplanned
accepted-program impact, protected corpus behavior/rule drifts, or the exact
spec/compiler/corpus activation cannot satisfy the stable workflow. Do not
weaken a rule, change a verdict, add a program-shaped special case, or hide the
blocker in an ordinary-loop desugaring.

## Closure

Move this record to `docs/done/` only after durable specification identity,
implementation status, SHA/acceptance results, protected impact, and design
facts are in their canonical homes. A positive closure must replace the ACTIVE
plan with stage 7; a blocker records the exact reproducer for owner disposition.
