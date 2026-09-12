# Audit: the entailment engine against the design tree

Module: `compiler/src/semantic/entailment.rs` (1,479 lines), `entailment/flow.rs`
(15,147 lines) with its own submodules `entailment/flow/sources.rs` (1,825
lines, the [ENT-3] fact sources) and `entailment/flow/kernel.rs` (681 lines,
[BLK-0] kernel-row [CALL-6] publication), `entailment/state.rs` (4,778 lines,
the [ENT-4] closed fact state and derivation ledger), `entailment/term.rs`
(305 lines, the [ENT-2] term vocabulary), and `entailment/polynomial.rs` (282
lines, the [PRF-1] degree-two certificate accumulator) — about 22,600 lines
excluding tests. `entailment/affine.rs` (1,354 lines, the affine-check core
`AUTO`/`DIRECT` sit on top of) is out of scope, per the ownership audit's
claim on it. Direction: code to tree — finding choices the code embodies
that `design/` does not record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/checks-and-proofs.md` and all of
`design/language/checks-and-proofs/` (`certificate-fold.md`,
`obligation-discharge.md` with its children `goal-decomposition.md` and
`loop-fact-retention.md`, `requires-entry-contract.md`). Also read:
`design/recall-tmp/README.md` and `sources.md`, and the two finished audits in
this directory as the format. Checked against `spec/kernel-spec.md`'s complete
entailment fragment — [ENT-1] through [ENT-6], [MSR-1] through [MSR-4],
[CALL-1] through [CALL-6], [INV-1], [PRF-1] (lines 3375-4223) — wherever the
code cites a rule id; no `[FCT-*]` family exists in this specification.

Git history is complete back to the true root `7c1d7641` (2026-07-07); every
"Reason" and "no reason recorded" line below is checked against the full
history with `git log -S`, `git blame`, and reading introducing-commit bodies,
not a shallow view.

## 1. Covered by the tree

- `design/compiler.md` (decision 1: one safe-Rust crate, no writer-accessible
  unsafe) — the whole module family: only `Vec`/`HashMap`/`HashSet`/`BTreeMap`,
  no `unsafe` block anywhere (confirmed by search).
- `design/compiler.md` (decision 4: a rejection names the numbered rule and
  location; no acceptance path depends on a timeout, fuel, a work budget,
  heuristic early failure, or hash-iteration order) —
  `compiler/src/semantic/entailment/state.rs::InternHashBuilder`/`InternHasher`/`DerivationLedger::probe_intern`
  (lines 884-1074: the index is private, never iterated, reached only by exact
  key lookup, and a *new* node's `DerivationId` is fixed by `self.nodes.len()`
  at insertion, never by its hash — the doc comment states this directly:
  "The index is private to this module and is never iterated, so its hash
  function reaches no compiler output and no traversal order"); the pervasive
  `.keys().copied().collect::<Vec<_>>(); ....sort_unstable()` pattern before
  every derivation-interning loop over a `HashMap`/`HashSet` snapshot
  (`state.rs::materialize_closure_at` lines 3891-3913,
  `join_at_once` lines 4081-4082, `close_with_excluded_term`'s
  `distinct_pairs.sort_unstable()` at line 3477).
- `design/language.md` (decision 2: proof checking never requires work that
  grows exponentially with program or written-proof size) —
  `state.rs::close_with_excluded_term`'s semi-naive, freshness-tracked dense
  fixed point over the difference-bound matrix (lines 3357-3504), which
  terminates on `if !changed { break; }` under a strictly monotone
  bound-decreasing lattice rather than a round cap; `flow.rs`'s `AUTO`
  orchestration explicitly enumerating only the zero-, one-, and
  unordered-two-premise families with two integer tightenings each
  (`flow.rs:12211-12269`, comment: "There is no greedy state, backtracking
  cutoff, or cumulative work budget: fact order changes only which successful
  derivation is retained, never acceptance").
- `design/language/checks-and-proofs.md` (decision 1: automatic derivation is
  a fixed set of terminating families; a harder proof is a finite explicit
  certificate the checker verifies without rediscovering it) — the same
  `AUTO`/certificate machinery above, together with
  `flow.rs::source_proof_sum`/`extend_certificate_sum` (lines 11253-11361),
  which forms exactly the writer's own `use` list in source order and proves
  nothing the writer did not name.
- `design/language/checks-and-proofs.md` (decision 2: required facts come only
  from admitted types/declarations, selected control-flow edges, verified
  contracts, and checked invariants) —
  `compiler/src/semantic/entailment/flow/sources.rs`'s exhaustive, additive S1/
  S4-S14 shape recognizers ("Sources are additive and never reject: a shape
  outside a source's stated form contributes nothing, which only
  under-derives", lines 7-9) and `flow/kernel.rs::establish_kernel_relations`/
  `establish_kernel_exit`, which publishes only a [BLK-0] row's own declared,
  compiler-owned relation list.
- `design/language/checks-and-proofs.md` (decision 3: a nonempty explicit
  certificate is rejected when the automatic families already prove its
  target) — `flow.rs` lines 13090-13103 (`auto_proved`/
  `redundant = !proof.uses.is_empty() && auto_proved`, computed from `AUTO`
  before the certificate sum or premises are judged) and
  `entailment.rs::SourceProofCheck::discharged`/`failure_use_index`
  (lines 748-777), which fail a nonempty, otherwise-provable block on
  `redundant` alone.
- `design/language/checks-and-proofs/certificate-fold.md` (all four decisions:
  degree-at-most-two folding by admitted-multiplication identity with a
  surviving nonlinear monomial rejecting; a folded binding is an opaque handle
  only between the fold and the residual; multiplicities are unsigned; a
  product operand may itself be a product, each fold removing one finite
  monomial) — `polynomial.rs`'s whole `Monomial`/`CertificatePolynomial`
  design (`fold_products` lines 180-196, `unfold_handles` lines 206-233,
  `is_affine`/`into_inequality` lines 249-281) plus
  `flow.rs::folded_certificate_sum` (lines 11433-11489), whose tie-break —
  "the one the target itself names is chosen, and otherwise the least" — is
  implemented essentially verbatim at lines 11444-11466.
- `design/language/checks-and-proofs/obligation-discharge.md` (decision 1:
  postcondition summaries publish in call-graph component order and are
  unavailable while their own component is checked) —
  `entailment.rs::EntailmentContext::verified_postconditions`'s doc comment
  ("Same-component entries remain absent until the component's atomic
  publication boundary", lines 225-227) and `postcondition_schedule`'s Tarjan
  SCC plus Kahn ordering (lines 1139-1290), which fixes one deterministic
  callee-before-caller component order.
- `design/language/checks-and-proofs/obligation-discharge.md` (decision 2:
  every certificate step reads the same entering context and publishes
  nothing; only the checked outer invariant becomes a fact, one flat weighted
  combination with no search over step order) —
  `flow.rs::source_proof_premise_results`'s comment, "Every written use is
  proved against the same pre-proof program point. No premise established by
  this statement can help another premise in the same statement" (lines
  13113-13115), and the fact that only the invariant's own `target` — never
  the sum, a scaled premise, or an intermediate residual — is pushed to
  `state.affine.facts`/`published_invariants` on success (lines 13174-13189).
- `design/language/checks-and-proofs/obligation-discharge.md` (decision 3: an
  integer-typed named const is an affine atom folded at formation to the one
  value it declares) — `term.rs::TermKind::Constant`/`ConstParameter`'s split
  (a named const's mathematical value vs. a symbolic const-generic parameter,
  lines 33-38), matching [ENT-2] clauses (c) and (g) exactly.
- `design/language/checks-and-proofs/obligation-discharge/goal-decomposition.md`
  (both decisions: a Boolean goal decomposes at establishment through its
  and/or/not structure with the same fixed rules reconstructing parent goals;
  composition of arbitrary formulas stays excluded) —
  `flow.rs::collect_decomposition_members`/`signed_boolean_decomposition`
  (lines 5309-5395), which recurses only into an already-written `band`/`bor`/
  `bnot` tree and never synthesizes a new formula, and
  `intern_goal_expression` (lines 5278-5297), which interns only written
  subexpressions.
- `design/language/checks-and-proofs/obligation-discharge/loop-fact-retention.md`
  (a pre-loop fact survives a loop head unless a kill inside the body can
  reach a later head without leaving the body; a return, propagated error, or
  break kills nothing at the head) — `flow.rs::loop_statement_reaches`/
  `collect_continuing_loop_kills` (lines 13932-14167: `Return => false`,
  `Break => reachability.break_reaches(target)`, a reverse per-statement walk
  with no fixed-point loop needed since it is a single tree pass), and the
  exact call-site ordering at an ordinary loop (lines 13389-13404:
  `prove_loop_invariant_bases` → `collect_continuing_loop_kills` →
  `apply_loop_kills` → `activate_loop_invariant_batch`) and a counted loop
  (lines 13491-13590, with `establish_counted_body_entry`'s S11 bounds coming
  last), both matching the specification's fixed order verbatim.
- `design/language/checks-and-proofs/requires-entry-contract.md` (decision 4:
  each requires clause is proved on its own at the call site in the caller's
  state, with no callee prologue; two clauses are the same goal when they
  normalize identically) — `sources.rs::establish_requires_facts` ([ENT-3]
  S4, lines 275-302) is the *only* place a requirement enters a callee body,
  and `state.rs::GoalTable::intern`'s exact-typed-tree dedup means no
  callee-entry re-check code exists anywhere in `flow.rs`.

## 2. No decision needed

- Semi-naive, freshness-tracked dense-matrix (Floyd-Warshall-shaped) fixed
  point for the [ENT-4] difference-bound closure, replacing a hashed
  tuple-keyed map read on every candidate — ordinary technique for a finite
  closure once termination and exactness are preserved.
- Content-addressed structural interning of derivation nodes behind a
  private, never-iterated deterministic word hasher, with a *new* node's
  identity fixed by insertion order rather than its hash — ordinary
  performance engineering once determinism is preserved (see item above in
  section 1); collision handling walks the free key by `+1`, which the
  code's own comment explains is safe because the index is never removed
  from, only rebuilt whole.
- Checked/saturating `i128` arithmetic throughout proof and certificate
  formation, every overflow surfacing as a typed `SourceProofCertificateFailure`/
  `PolynomialError`/`AffineCheckError` rather than silent wraparound —
  ordinary overflow-safety practice matching the rest of the compiler.
- `.expect()`/`assert!` guarding internal arena invariants (a derivation's
  parents precede it, a dense id fits `u32`, a [BLK-0] row's declared
  relations cannot make a caller's state contradictory —
  `flow/kernel.rs:383-391`) rather than a typed `CompilerFailure` — none of
  these is reachable from source shape alone, matching [ENT-1]'s framing that
  "a wrong derivation is a compiler defect... not a second runtime validation
  layer," not the frontend's separate zero-panic discipline.
- The dual "ordinary" (non-postcondition-dependent) closure/join computed
  alongside the primary one whenever the winning derivation depends on an S12
  candidate (`state.rs::retain_non_postcondition_candidates`,
  `materialize_closure_at`'s and `join_at`'s `needs_ordinary_fallback`,
  lines 3889 and 4018) — a mechanical consequence of [ENT-3.S12]'s own
  stated failure-atomic candidate scratch, not an independent policy.
- `CertificatePolynomial`'s canonical `BTreeMap<Monomial, i128>`
  representation, a quadratic pair's two atoms stored smaller-first — ordinary
  canonicalization so equal polynomials compare equal, which [ENT-2]'s exact
  typed-tree identity rule requires anyway.
- Tarjan SCC plus Kahn's algorithm with `BTreeSet`-based tie-breaking by
  smallest dense `FunctionId` for the FN-9 concrete-call component schedule
  (`entailment.rs:1139-1290`) — ordinary deterministic-graph-algorithm
  engineering satisfying [ENT-1]'s "same concrete-SCC order" requirement.
- `TermTable`/`GoalTable` as flat `Vec` plus `HashMap<_, DenseId>` registries
  queried only by exact key and never enumerated for an acceptance decision —
  ordinary dense-identity bookkeeping matching the rest of the compiler's
  dense-id convention.
- `flow.rs::judge_system_ranges` deriving each system-operation row's
  `start`/`end`/range-bearing-buffer operand ordinals from the declared
  parameter table by name and type (lines 8358-8384) rather than a
  per-operation positional constant — table-driven and automatically correct
  for every member [SYS-8] names, consistent with [CALL-5]'s "the declared
  shape... and nothing else" reading.
- `TermTable::intern`'s merge of the written constant zero into the
  distinguished Z term — the one deliberate exception to term-identity-by-
  spelling, and the doc comment states the [ENT-2] reason directly (an
  unmerged zero would leave `d != 0_i32` unable to reach a disequality against
  Z at a signed type, making [OP-2]'s own mechanical fix unwritable).

## 3. Choices without a node

**1. [ENT-3] S10's boundary-endpoint fact source covers three system
operations the specification's own S10 rule text does not name — reads as a
stale specification, not a compiler defect, but is unresolved as of the
active v0.53 text.**
`BOUNDARY_ENDPOINTS` lists eight `(spelling, observing-variant)` pairs —
`read_at`, `read_next`, `receive_next` (all `"ReadBytes"`); `write_once`,
`send_once`, `host_copy_bytes`, `host_copy_utf8` (all `"Ok"`); `directory_next`
(`"ListBytes"`) — and `boundary_endpoint_outcome` establishes the S10 endpoint
bound `s <= w <= e` on the observing arm of every one of them. The live
`spec/kernel-spec.md`'s own [ENT-3.S10] rule text names only five: "a call to
`read_at`, `write_once`, `directory_next`, `host_copy_bytes`, or
`host_copy_utf8`," with the `Ok` arm belonging to "the other three" — no
mention of `read_next`, `receive_next`, or `send_once` at all.
Alternative: restrict `BOUNDARY_ENDPOINTS` to literally the five operations
[ENT-3.S10]'s text names, under-deriving for the three TCP/stream operations
(the safe, version-monotone direction [ENT-1] explicitly allows) until the
specification is amended to name them.
Where: `compiler/src/semantic/entailment/flow/sources.rs:41-50`
(`BOUNDARY_ENDPOINTS`) and `:1628-1665` (`boundary_endpoint_outcome`);
`spec/kernel-spec.md:3721-3727` (the current [ENT-3.S10] text).
Reason: the three extra operations were added in `8cd4baed` (2026-09-05,
"Batch 3 slice 1: specification v0.46, streams and TCP under T4"), which
introduced `read_next`/`receive_next`/`send_once` project-wide as the
[SYS-15]-[SYS-18] streams/TCP feature; that commit's body describes the
lowering and lists the new declarations but says nothing about extending S10.
It is well grounded elsewhere in the specification even so: [SYS-8] itself
states, unprompted, that these three are full behavioral siblings of
`read_at`/`write_once` — "`read_at`, `read_next`, and `receive_next` share
[the three `ReadBytes`/`ReadEnd`/`ReadFailed` outcomes] because each reports
the same three things about one transfer" (line 3147), the identical
empty-range and nonempty-range host-transfer rules naming all three or all
five together throughout §14 (lines 3187, 3191, 3196, 3208, 3211), and
`judge_system_ranges`'s own [SYS-8] obligation (see §2 above) already applies
uniformly to every member of that ten-operation family by table lookup, not
by name. But `git log -S` on the [ENT-3.S10] paragraph's own text finds
exactly one commit ever touching it, `55a75434` ("spec: compose the
claim-only v0.33 candidate"), which predates the streams/TCP feature by
several weeks and has not been revisited since — no commit anywhere states a
reason for leaving the S10 rule's named list at five once [SYS-8] grew to
ten.
Effect: acceptance. Since v0.46, the shipped compiler derives more S10 facts
than the current specification's own S10 sentence authorizes for three named
operations, which is real over-derivation relative to the rule's literal
text even though it is consistent with what [SYS-8] elsewhere already says
about those same operations. No test in
`compiler/src/semantic/tests/entailment.rs` exercises S10 on `read_next`,
`receive_next`, or `send_once` (only `read_at` is tested for this rule); the
only test present, `a_read_at_endpoint_is_observed_on_its_own_outcome_variant`,
predates the extension in substance. The likely correct repair is amending
[ENT-3.S10]'s named list to match [SYS-8]'s existing ten-operation family
rather than narrowing the code, but that is a specification judgment for the
owner, not one this audit makes.

**2. The FN-9 call-graph SCC scheduler recurses natively over the concrete
call graph with no depth ceiling — a likely defect, joining an already-
recorded hazard of the same shape.**
`postcondition_schedule` builds the ordinary caller→callee graph, restricted
to functions with at least one `ensures` clause reachable in the unit's call
graph (an empty check short-circuits to `PostconditionSchedule::default()`),
then calls `strongly_connected_components`, whose `Tarjan::visit` recurses
one native Rust stack frame per unvisited successor
(`self.visit(*successor)`, entailment.rs:1425-1428) with no iterative
worklist and no caller-side ceiling on call-graph depth or function count.
Alternative: an explicit iterative task-stack Tarjan (the same shape
`compiler/src/syntax/parser/engine.rs` and its diagnostic/finalizer siblings
already use for grammar derivation, per the lexer-and-syntax audit's item 3),
bounded by a `ParseLimits`-style ceiling; or a driver-level cap on the number
of functions per compilation unit that would incidentally bound recursion
depth here too.
Where: `compiler/src/semantic/entailment.rs:1139-1149` (the postconditions
guard) and `:1405-1470` (`strongly_connected_components`, `Tarjan::visit`).
Reason: no reason recorded. `strongly_connected_components` and its plain
recursive `visit` are present, unchanged in this respect, from the function's
introduction (`491446af`, "feat: retain complete counted-range derivations",
part of the v0.28 verified-postconditions activation window) through to
today; no later commit revisits its recursion shape or adds a ceiling.
Effect: robustness, not acceptance for any program this scheduler actually
finishes checking. A concrete call chain with no mutual recursion — a long
sequence of small helper functions, each with an `ensures` clause, each
calling the next, of a size a machine-generated program could plausibly
reach — recurses `Tarjan::visit` one native stack frame per function in the
chain and can exhaust the process stack with no diagnostic, an unguarded
crash rather than the source rejection or `ResourceFailure` the specification
requires for a compiler-side limit [ENT-1, DIAG-1]. `docs/todo.md` already
records exactly this hazard shape for a different trigger — "Unguarded affine
expression nesting depth... aborts the driver with a stack overflow and no
diagnostic (exit 134)," reachable through `affine.rs` (out of this audit's
scope) via a `use` premise or an `invariant` target, with the fix "still
unresolved which side owns it." This is a second, independent instance in a
mechanism squarely inside this audit's scope, gated behind ordinary [FN-9]
`ensures` usage rather than a proof-only construct, and it is not yet named
alongside the recorded one.

**3. The [ENT-4] closure and the derivation ledger's interning index are a
hand-rolled, measured performance rewrite, not decided in the design tree.**
The plain, obvious implementation of both — hash the whole `DerivationNode`
with the default `SipHash`-backed `HashMap` for interning, and answer every
closure candidate by reading and writing `HashMap<(TermId, TermId), _>` pairs
directly — was replaced by a private word hasher with content-addressed
linear probing (`InternHasher`/`InternHashBuilder`/`probe_intern`,
state.rs:884-1074), a dense `dimension × dimension` matrix with per-cell
freshness rounds that lets the transitivity fixed point skip an unchanged
triple (`DenseClosureBounds`, state.rs:3663-3802), and a dense boolean
membership index for which terms can serve as a transitivity middle
(`ActiveMiddles`/`closure_middle_terms`, state.rs:3575-3654).
Alternative: keep the ordinary `HashMap<DerivationNode, DerivationId>` with
the standard hasher, and answer every `(TermId, TermId)` closure query
through the state's own hashed maps directly, at whatever their raw
random-access cost is.
Where: `state.rs::InternIndex`/`InternHashBuilder`/`InternHasher` (884-961),
`DerivationLedger::probe_intern` (1064-1074), `DenseClosureBounds` and its
use inside `close_with_excluded_term`'s fixed point (3310-3444, 3656-3802),
`ActiveMiddles`/`closure_middle_terms` (3575-3654).
Reason (quoted, commit `727194fe`, 2026-08-27, "entailment: stop rerunning
identical counterfactual analyses"): "Counters over the closure engine locate
the cost precisely: the term table is small (91 terms at most)... but 82 of
the 102 whole-function analyses are bit-identical repeats... and carry 83% of
the 2.46 billion transitivity triples the check evaluates... Inside the
closure, the transitivity fixed point skips a triple whose premises have not
changed since it was last offered, the live bound and proof maps are rebuilt
once from the settled matrix instead of on every accepted candidate,
middle-term membership becomes a dense index, and the private interning map
gets a deterministic word hasher." The same commit also introduced a
per-function `CounterfactualReuse` cache for the claim-blinding reruns that
motivated the whole session; that mechanism no longer exists anywhere in this
audit's five files (it is not found by search) and evidently now lives with
the claim/affine machinery this audit excludes, so it is mentioned here only
for provenance and is not itself part of this finding.
Effect: performance only, and unusually well verified for an unrecorded
choice — the same commit states the complete observable output (stdout,
stderr, exit status, and emitted LLVM IR under `--par --par-ledger
--stack-ledger`) is byte-identical to the pre-change compiler across all 623
sources in `tests/programs`, `tests/codegen`, and `tests/conformance/cases`,
and adds three regression guards naming the mechanisms rather than a
wall-clock number (present today as `interning_into_a_cloned_ledger_keeps_the_original_identity`,
`a_strengthened_bound_still_propagates_in_a_later_closure_round`, and
`settling_a_ledger_does_not_move_the_finished_byte_metric` in
`state.rs`'s own test module). No design/compiler node records that this is
how the entailment engine's own internals should be built for performance,
unlike the parallel case in `compiler/src/syntax/`, whose iterative-task-
stack and flat-postorder-tree choices are grounded in
`compiler-architecture-frontend.md`.

## Summary

- Covered by the tree: 13
- No decision needed: 10
- Choices without a node: 3 (1 spec/code discrepancy, 1 likely defect, 1
  deliberate measured performance choice)
