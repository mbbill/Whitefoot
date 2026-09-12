# Audit: parallel permissions against the design tree

Module: `compiler/src/semantic/permission.rs`, `staged_permission.rs`,
`loop_permission.rs`, and `permission_ledger.rs` (about 6,150 lines total).
The three parallel-permission judgments — [PAR-1]'s sibling-call window,
[PAR-2]'s counted-loop reduction/map, [PAR-3]'s staged I/O-loop pipeline — plus
the non-normative developer-channel ledger that renders their verdicts as
text. Direction: code to tree — finding choices the code embodies that
`design/` does not record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/parallelism.md`, `design/language/parallelism/permission-judgment.md`,
`design/language/effects.md`, `design/language/checks-and-proofs.md`,
`design/compiler/parallel-lowering.md`, `design/compiler/parallel-lowering/two-worlds.md`,
`design/compiler/parallel-lowering/parallel-runtime.md`, and
`design/recall-tmp/README.md`/`sources.md`. Also read for grounding, since the
module's own comments cite them constantly: `spec/kernel-spec.md` §13
("Execution overlap": [CAP-1], [PAR-1], [PAR-2], [PAR-3] in full, lines
2572-2667), its [GRAM-4] statement grammar and `let_stmt` production
(lines 261-276), and the [OWN-1], [OWN-5], [OWN-7] text the permission
judgments cite by name.

Scope note: `design/compiler/parallel-lowering.md` and its two children govern
what a `--par` build *emits and runs* — the two-worlds clone split, the
recursive-frontier budget, the work-stealing runtime — none of which any of
these four files touches. This module family stops at the read-only
`PermissionMetadata`/`LoopPermission`/`StagedPermission` tables; nothing here
selects a lowering, a grain, or a runtime parameter. That boundary is itself
`design/language/parallelism.md`'s decision 2 ("permission and actualization
are separate judgments"), so the near-total absence of parallel-lowering
citations below is the expected shape of this audit, not a gap in it.

Git history note: this session's clone is the unshallowed repository
`design/recall-tmp/sources.md` describes — 2,475 commits reachable from
`HEAD`, real parentless root `7c1d7641` (2026-07-07). Every "Reason" line
below was found with `git log -S`/`git log --follow`/`git blame` against that
full history; "no reason recorded" means the introducing commit and its
successors say nothing, not that the search stopped early.

## 1. Covered by the tree

- `design/language/parallelism.md` (decision 1: permission derives from
  checked proofs and a failed permission leaves the program sequential rather
  than rejecting it) — the identical invariant sentence opens all three
  judgments: `permission.rs:4-7` ("P is a compiler-internal legality
  judgment. It refuses nothing, changes no acceptance, and grants no lowering
  by itself"), `loop_permission.rs:7-10`, `staged_permission.rs:5-10`; no
  `Denial`/`LoopDenial`/`StagedDenial` variant anywhere in the three files
  reaches a `SourceIssue` or any other rejection path.
- `design/language/parallelism.md` (decision 2: permission and actualization
  are separate judgments) — `PermissionVerdict`/`LoopVerdict`/`StagedVerdict`
  (`permission.rs:367-372`, `loop_permission.rs:199-206`,
  `staged_permission.rs:205-210`) each carry exactly `{Permitted(Eligible),
  Denied}`, with no actualizability variant; `LoopActualization`'s own doc
  (`loop_permission.rs:135-141`, "The judgment does not decide that anything
  is emitted: lowering reads this... The verdict above is the same either
  way"); and commit `d7098ffe` (2026-08-23, "compiler: overlap a correct
  program, latch a defective one's trap") deleting the earlier combined
  `PermittedNotActualizable` verdict outright.
- `design/language/parallelism.md` (decision 3: a counted loop is a
  parallel-permission site in its own right, not an amendment to the
  sibling-call rule) — `loop_permission.rs` exists as a wholly separate
  judgment (module doc lines 5-10: "The window judgment next door reads a
  pair of *statements*... This judgment reads the loop itself"), invoked as
  its own pass in `permission.rs:857` (`judge_loops`) after the pair/run
  tables are finished, reading only their finished eligible-pairs list for
  advice (`permission.rs:851-856`) and never regrouping a loop's iterations
  into sibling-call windows.
- `design/language/parallelism/permission-judgment.md` (decision 1: judge a
  window of statements, not adjacent pairs; rejected: adjacent-pair
  enumeration measured at 1.9x) — `permission.rs`'s module doc (lines 9-21)
  restates the same finding; `BlockWindows`/`Interposed`/`interposed_of`
  (`permission.rs:773-810`, `1172-1307`) classify every statement of a block
  once, and `judge` (`1028-1163`) walks `windows.statements.get(window_start..second.index)`
  rather than only the two candidates themselves.
- `design/language/parallelism/permission-judgment.md` (decision 2: the loan
  judgment reuses the borrow checker's own overlap vocabulary lifted from one
  call's arguments to a window's statements) — `Loan`/`LoanStrength`/`excludes_use`/`excludes_loan`
  (`permission.rs:189-228`) transcribe [OWN-5]'s own matrix (`spec/kernel-spec.md:2584-2585`)
  once, and `loan_conflict` (`permission.rs:1685-1759`) is the only place it
  is applied; `loop_permission.rs` (`call_loans`, `440-444`, `987-992`) and
  `staged_permission.rs` (`record`'s loan handling, `1282-1295`) both import
  and reuse the same `Loan`/`LoanStrength` types unchanged rather than
  growing their own.
- `design/language/parallelism/permission-judgment.md` (decision 3:
  unresolved overlap or an unsupported interposed form denies, and a missing
  classification never contributes an empty footprint) — every one of the
  three judgments' statement-handling matches is written as an *exhaustive*
  match ending in an explicit refusal rather than a wildcard default:
  `permission.rs::interposed_of` (`1172-1307`), `loop_permission.rs::Survey::statement`
  (`495-604`, plus the module's own "# The one-sided reading" section,
  `97-104`), `staged_permission.rs::StagedSurvey::statement` (`1108-1191`,
  plus condition 7's statement, `90-95`).
- `design/language/effects.md` (decision: contracts and invariants are erased
  proof syntax and introduce no effect) — every proof statement is a no-op to
  every judgment: `CheckedStatement::Proof(_) => Ok(Interposed { footprint:
  Footprint::default(), .. })` (`permission.rs:1179-1183`), `CheckedStatement::Proof(_)
  => {}` (`loop_permission.rs:568`, `staged_permission.rs:1126`), matching
  each module's own "proof statements do not add a fifth/further condition"
  section (`permission.rs:87-95`, `loop_permission.rs:42-47`).
- `design/language/effects.md` (decision: effects describe ordinary state
  including opaque system resources, and host scheduling belongs to target
  lowering) — `system_call_footprint` (`permission.rs:1448-1541`) projects a
  system operation through the identical `Access`/`Footprint`/`Loan`
  machinery `user_call_footprint` uses for a declared function, via
  `operation_state_effects`; `target_action`/`may_suspend` is carried on
  every `PermissionSite` purely as metadata (`permission.rs:403-406`, "The
  permission judgment does not use it as an alias fact") and never gates a
  `verdict` in any of the three modules — only `actualization`/`LoopActualization`
  (a scheduling/lowering-facing field, not the permission itself) reads it.
- `design/language/checks-and-proofs.md` (decision: required domains are
  established only by the specification's deterministic proof system, whose
  facts come from admitted types, declarations, control-flow, verified
  contracts, and checked invariants, not writer assertions) —
  `loop_permission.rs::proven_affine_map_at` (`698-714`) consumes an
  already-discharged `ObligationOutcome`/`ProvedAffineIndexMap` by source-node
  identity and its own doc states the boundary directly (`673-676`):
  "Permission neither evaluates the source expression nor reruns proof:
  absence of this checked evidence fails closed."
- `design/compiler.md` (decision: valid specified source the compiler has not
  implemented stops as an explicit unsupported capability, never a rejection)
  — `Access::Arena`'s own doc (`permission.rs:161-174`) names the gap by name:
  "The other half of the arena boundary is **not** covered and must be
  [closed] before any arena program compiles... every arena program stops
  today at `UnsupportedSemanticFeature::ArenaRuntime`, so nothing reaches this
  gap."
- `design/compiler.md` (decision: a rejection names the numbered rule and
  location; no acceptance path depends on... hash-iteration order) — every
  denial carries a fixed, hand-numbered `.condition() -> u8`
  (`permission.rs:353-362`, `loop_permission.rs:258-269`,
  `staged_permission.rs:383-393`) matching the specification's own numbered
  clauses; `permission_ledger.rs::collapse` (`312-338`) sorts every rendered
  line by `(path, line, kind, ordinal, text)` before dedup, and
  `Program::analyze_function` (`permission.rs:829-846`) explicitly sorts
  `pairs`/`runs`/`completion_steps` by source position — table order never
  depends on `HashMap` or monomorphization iteration order.
- `design/compiler.md` (decision: one safe-Rust crate, simple implementations
  over ordinary collections) — the whole family: only `Vec`, plain
  structs/enums, no `unsafe`, no dependency beyond the compiler's own
  `model`/`places`/`entailment` modules.
- `design/language.md` (decision: every required source fact is
  machine-checked and erased before lowering, with no writer-accessible
  unsafe, trusted theorem, or runtime trap of any kind) — the current module
  family carries no `Claim`/`ClaimTrap`/trap-latch concept anywhere; the only
  proof-shaped statement form any of the three judgments ever sees is
  `CheckedStatement::Proof(_)`, treated as wholly inert in all three. An
  earlier version of `permission.rs` still modeled a writer-reachable
  `claim` trap edge (`ExitKind::ClaimTrap`, present as of commit `d7098ffe`,
  2026-08-23); it was removed by `cf69135a` ("Implement source-carried proof
  compiler"), and neither `ClaimTrap` nor `CLM-1` appears anywhere in the
  current `spec/kernel-spec.md` or these four files today.

## 2. No decision needed

- `BlockWindows` classifies every statement and projects every candidate's
  footprint once per block rather than once per judged window
  (`permission.rs:791-810`) — the ordinary algorithmic fix for the O(m²·n)
  blow-up the already-decided window design would otherwise cause; the
  introducing commit (`ae7be029`) measured the wider candidate groups as
  costing nothing ("the suite runs 167.80s against 167.92s before"), i.e.
  this is the same window decision's implementation detail, not a second
  choice.
- `staged_permission.rs`'s `Flow`/`FlowBuilder` is a dense array-based
  control-flow graph over exactly the loop body's statements
  (`646-925`), with `dominators`/`post_dominators` as classic iterative
  worklist fixpoints — the ordinary technique for the "real dominator and
  post-dominator query... never a statement-index heuristic" condition 1
  itself demands (module doc, `43-44`, and `spec/kernel-spec.md:2644`).
- `Access`/`Footprint`/`Loan`/`call_projection`/`kernel_projection` are
  defined once in `permission.rs` (`151-268`, `551-635`) and imported
  unchanged by both `loop_permission.rs` and `staged_permission.rs`, matching
  the modules' own stated discipline ("both judgments read one place relation
  and neither grows a private copy of it," `permission.rs:551-557`) —
  ordinary code-sharing, not an independent policy.
- `collect_introduced`/`nested_bodies`/`borrows_only_iteration_own` are
  defined once in `loop_permission.rs` (`1253-1338`) and reused verbatim by
  `staged_permission.rs` (imported at `168`, called at `514`, `1209`): "The
  staged judgment next door asks the same question of the same body, so both
  read this one walk rather than growing two drifting copies"
  (`loop_permission.rs:1251-1252`).
- Dense checked identities (`BindingId`, `CheckedLoopId`, `FunctionId`) are
  used as plain indices into caller-owned slices throughout, with no
  wrapping arithmetic — ordinary type-safety practice, the same pattern the
  resolution-module audit records for its own dense identities.
- The fixed order each `denial()` checks its numbered conditions in (a form
  refusal always checked first) is a dependency requirement, not a free
  choice: each module states directly that a statement whose footprint is
  unclassified has no condition-1/2/3 answer to give
  (`permission.rs:1018-1027`, `loop_permission.rs:1018-1023`,
  `staged_permission.rs:1527-1533`); `design/compiler.md`'s own rule already
  leaves "which of several violations is reported first" outside the
  language, so any deterministic order these dependencies allow is licensed.
- `permission_ledger.rs::Entry`/`collapse` (`280-338`) — the specific
  dedup-by-`(kind, ordinal, text)` mechanism that lets one generic's several
  monomorphized instances collapse to one reported site while OR-ing their
  `notice` flags is the mechanical enactment of the determinism rule already
  cited in section 1, applied to this table's specific shape, not a further
  choice.
- Small index conversions saturate rather than panic or wrap
  (`u32::try_from(ordinal).unwrap_or(u32::MAX)`, `permission_ledger.rs:244`)
  — ordinary overflow-safety practice, consequential only past four billion
  rows of one disposition table.
- `PermissionMetadata::of`/`named` (`permission.rs:469-485`) — a dense lookup
  by `FunctionId` plus a name-based convenience explicitly marked
  `#[allow(dead_code)]` for ledger/test callers; ordinary API surface, not a
  policy.
- `LoopCombine` and `integer_combine`/`boolean_combine`
  (`loop_permission.rs:159-197`, `1216-1244`) are one closed enum
  transcribing [OP-1]'s exactly-associative operation list
  (`spec/kernel-spec.md:2609`, `2628-2629`) verbatim, including the reasons
  each near-miss (`+`, `+defined`, `+checked`, `+sat`, every float operation)
  is absent — a direct table transcription, the same pattern the
  resolution- and lexer-module audits record for their own closed spec
  tables.

## 3. Choices without a node

**1. A sibling-call window's second member may be a `let x = propagate
f(...);`, though [PAR-1]'s text restricts both members to `ordinary_let_rhs`
— reads as an unexamined gap, not a deliberate choice.**
`candidate_of` (`permission.rs:1901-1936`) admits `CheckedStatement::PropagateLet`
as an ordinary `Candidate` (with `exit: Some(ExitKind::PropagateError)`) on
exactly the same terms as `CheckedStatement::Let`. `judge`
(`1028-1163`) checks `first.exit` at condition 4 (`1153-1158`) but never
`second.exit`, so a pair whose *second* member is a `propagate` statement can
reach `PermissionVerdict::PermittedEligible` whenever conditions 1-3 hold —
for example `let a = some_call(); let b = propagate other_call();` with
disjoint footprints and no dataflow between them.
Alternative: exclude `PropagateLet` from `candidate_of` entirely (as
`Return`/`Give`/`Break`/`Evaluate` already are), leaving `interposed_of`'s
existing `Err(InterposedRefusal::Exit(ExitKind::PropagateError))` arm
(`1276-1278`) as the only place a `propagate` statement is ever classified —
which already correctly denies it whenever it would need to be a first
member or an interposed one.
Where: `permission.rs::candidate_of` (`1901-1936`, the `PropagateLet` arm at
`1908-1919`); `permission.rs::judge` (`1153-1158`, the condition-4 check that
reads only `first.exit`).
Reason: none recorded for the second-member case specifically. `PropagateLet`
has been an admitted `Candidate` since the module's first commit, `623660da4`
(2026-08-21, "compiler: judge which sibling call pairs may be overlapped"),
whose contemporaneous module doc already stated "it is never a first member"
and stopped there. The generalizing commit `d085133b5` (2026-08-27, "io:
judge and lower a call written in match-scrutinee position") preserved the
same asymmetry while extending `candidate_of` for `Match`/`ValueMatchLet`.
The current doc (`permission.rs:70-73`) still states the invariant only for
"a first member and... an interposed one," naming no second-member case
either way. `compiler/src/semantic/tests/permission.rs` carries exactly two
tests on this sentence, `a_propagating_first_statement_is_denied_by_condition_four`
(`:726`) and `an_interposed_propagate_is_denied_by_condition_four` (`:1257`)
— both pinning the first-member and interposed readings the doc names;
neither exercises a `propagate` as a pair's second member, so nothing in the
suite would catch a regression on that shape either way.
Reads as a likely defect: the doc's own careful, twice-repeated wording reads
as an examined boundary, and a second-member case sits just outside it with
no sign the omission was itself considered. It is also, independently, a
discrepancy with `spec/kernel-spec.md:2579`'s literal PAR-1 text, which names
"a `let_stmt` whose selected `ordinary_let_rhs` is one call" for *both* s1
and s2 — a production distinct from `propagate_let_rhs`
(`spec/kernel-spec.md:267-276`). A pair with a `propagate` second member is
therefore not a shape [PAR-1] defines permission for at all, so granting it
eligibility falls outside the rule rather than inside a permissive reading of
it, independent of whether an actualization built on that claim (out of this
audit's scope) happens always to insert the join that would make it safe in
practice.
Effect: optimization/diagnostics scope from this module's own vantage — the
table and the ledger report legality for a window shape the specification's
text does not define; whether any lowering that trusts `PermittedEligible`
here is actually safe depends on an obligation (joining every outstanding
hand-out before any exit reachable from the window, including the second
member's own) this audit did not read.
Decision: `PropagateLet` is removed from `candidate_of` and reaches only
`interposed_of`'s existing exit-bearing classification, because [PAR-1]
defines permission only for a pair whose selected right-hand side is
`ordinary_let_rhs` on both sides and a `propagate` second member is
therefore outside the rule regardless of what lowering might safely do with
it, instead of leaving a `propagate` statement eligible to stand unchecked as
a pair's second member.

**2. A `match`/`value_match` scrutinee call is judged as a [PAR-1] window
member by analogy, though the rule's text names only a `let_stmt` selecting
`ordinary_let_rhs`.**
`candidate_of` and the `Candidate` struct's own doc (`permission.rs:513-523`,
`1892-1936`) admit a bare `match_stmt` scrutinee and a `value_match`/`value_if`
(`ValueMatchLet`) scrutinee as ordinary window members, "the same call as a
`let` right-hand side," judged by the same four conditions.
`spec/kernel-spec.md:2579` restricts s1 and s2 to "a `let_stmt` whose
selected `ordinary_let_rhs` is one call": a bare `match_stmt` is not a
`let_stmt` at all ([GRAM-4], `spec/kernel-spec.md:264-266` lists `match_stmt`
as its own alternative of `stmt`), and `value_match`/`value_if` is a distinct
`let_stmt` alternative from `ordinary_let_rhs`
(`spec/kernel-spec.md:267-269`). Neither shape is the one PAR-1's words
define permission over.
Alternative: confine window membership to `Let`, reporting no verdict at all
for a call written in scrutinee position — the earlier behavior the
introducing commit's own message describes and rejects.
Where: `permission.rs::candidate_of` (`1920-1924`, the `Match`/`ValueMatchLet`
arm); `Candidate::result_read_by_own_statement` and its consumer in `judge`
(`window_start` at `1042`).
Reason (quoted, commit `d085133b5`, 2026-08-27, "io: judge and lower a call
written in match-scrutinee position"): "[PAR-1] judges calls, and a `match`
scrutinee is a call... The judgment reached one only through
`CheckedStatement::Let` and `PropagateLet`, and `IrBuilder::lower_statements`
recorded a call's landing site only from the `Let` arm, so the same two
operations got different work depending on how the second one was spelled...
A candidate is now built from every written call position." The commit gives
a real, considered engineering reason (consistent codegen and a non-silent
ledger for two spellings of the same operation) but does not address, and no
later commit revisits, that the generalization reaches beyond what PAR-1's
text names as an admissible member.
Deliberate choice, not a defect, and still a discrepancy with the
specification's text: unlike item 1, the extension here is fully reasoned in
its introducing commit, internally consistent (a scrutinee "can only ever be
a run's last member," which the code enforces structurally since arm bodies
are separate blocks with no successor statement in the scrutinee's own
block), and its side conditions are worked through in the same change. It is
nonetheless a discrepancy with `spec/kernel-spec.md`'s literal PAR-1 text,
and no design-tree node records the extension or its reason;
`design/language/parallelism/permission-judgment.md` records the
window-vs-adjacent-pair and loan-reuse decisions but says nothing about which
statement forms may be a member at all.
Effect: optimization scope only — a program with a call written in
match-scrutinee position gets a permission verdict, and if eligible an actual
overlap, for a construct PAR-1's text does not mention, by a judgment the
module's own doc argues is sound by analogy to the `let`-bound case.
Decision: [PAR-1]'s window judgment treats a `match_stmt` or
`value_match`/`value_if` scrutinee call as an ordinary member, identical in
every condition to a `let`-bound call, because the two reach the same
[EFF-2] projection and treating them differently produced inconsistent
codegen and a silently incomplete ledger for one of the two spellings,
instead of confining PAR-1's admitted member shape to the exact
`let_stmt`/`ordinary_let_rhs` pair its current text names.

**3. [PAR-3]'s replicated disposition is implemented only for storage the
loop body itself constructs; the general enclosing-storage case the rule's
own text defines is not implemented at all.**
`spec/kernel-spec.md:2648` gives condition 5 exactly three admissible
dispositions for a place *rooted outside the loop*: read-only, serialized to
one segment, or "this rule replicates it," with replication itself
(`spec/kernel-spec.md:2655-2658`) requiring a copy element type and a
full-byte-coverage proof. `disposition_of` (`staged_permission.rs:1655-1680`)
is the only function that computes a disposition for such a place, and it
never returns `Disposition::Replicated` — its arms are `Denied`/`ReadOnly`/`Serialized(Prologue)`/`Serialized(Remainder)`/`Denied`.
Every `Disposition::Replicated` row actually emitted instead comes from the
unrelated `construction()`/`self.replicated` path (`1415-1466`,
`1552-1574`), which only ever records a value the loop body *constructs
fresh each iteration* — storage `is_iteration_own` already excludes from
`touched`/`Class`/condition 5 entirely, and which is condition 6's fact (an
implementation's freedom to reuse a construction's storage across
iterations), not condition 5's replication of *enclosing* storage. The
module's own doc names the gap directly: "# This version replicates only
iteration-own storage... An *enclosing* scratch buffer that the body writes
and reads is denied... Admitting one needs a derived byte-range analysis...
that analysis consumes the entailment fact state and is a later batch"
(`150-160`); `is_replicable_shape`'s doc (`320`) and `Touched::replicable_shape`'s
doc (`949`) both say "once/later a coverage proof... establishes" — present
tense, not yet true.
Alternative: implement the byte-range coverage analysis
(`research/investigations/io-model/LOOP-PIPELINE.md`'s design "A") in the
same change, so a qualifying enclosing buffer is actually replicated, rather
than shipping only design "B"'s chassis with "A" deferred.
Where: `staged_permission.rs::disposition_of` (`1655-1680`);
`StagedSurvey::construction` (`1415-1466`); `StagedSurvey::finish`'s
`self.replicated` loop (`1552-1574`); module doc (`150-160`).
Reason (quoted, commit `2094da6b`, 2026-08-27, "par: judge the staged loop
permission as [PAR-3]", the module's introducing commit): "Stage one
replicates only storage the body itself constructs, so the hoisted
`many_files_narrow.wf` is denied and the new byte-identical
`many_files_loop.wf` is granted." The cited investigation states the
intended sequel directly (`research/investigations/io-model/LOOP-PIPELINE.md:29-40`):
"The winner is B's chassis with A's proof grafted on top, delivered in that
order. B supplies the schedule, the permission shape, the storage
discipline... A supplies the one thing B declines: a derived byte-range
analysis that makes the *byte-unchanged* `many_files_narrow.wf` fast." No
commit reachable from `HEAD` adds that analysis, and `compiler/src/semantic/`
has no `access_range.rs` or equivalent.
Deliberate and recorded, in a commit and an investigation rather than a tree
node: this is exactly the kind of decision `design/recall-tmp/` exists to
recover — a real scope choice with a stated reason and a genuine falsifier
program, that fails closed (every enclosing-storage case this version cannot
prove is denied, matching condition 7's one-sided reading, never over-granted)
and was a sequencing of the general case rather than a rejection of it. No
node under `design/language/parallelism/` or `design/compiler/parallel-lowering/`
records either half.
Effect: optimization scope only, and conservative — a program whose
enclosing scratch buffer genuinely satisfies the coverage condition still
stages sequentially rather than gaining the pipeline PAR-3 would license for
it; no unsound grant results from the gap.
Decision: PAR-3's replicated disposition ships first for iteration-own
construction reuse alone, with the general enclosing-storage case deferred,
because the byte-range coverage analysis that case needs consumes the
entailment fact state and was sequenced after the schedule, permission
shape, and storage discipline the first stage needed to ship and measure
against its own falsifier program, instead of implementing the coverage
proof in the same change.

**4. `StagedDenial` carries finished English advice on the enum itself; its
two sibling denial types keep all wording in the ledger renderer.**
`StagedDenial::writer_form()` (`staged_permission.rs:399-455`) returns a
complete, sometimes multi-sentence, admitted-writer-form string for every
variant, built from three module-level constants (`EXIT_IN_REMAINDER`,
`EXIT_SELECTED_BY_SUBMISSION`, `ONE_POSITION`, `369-378`) with per-field
conditionals of its own (e.g. `RetainedBorrow { overlapping: true, .. }` vs.
the general case, `414-432`). `permission_ledger.rs::staged_denied_detail`
(`502-594`) calls this method directly — the one place the ledger defers to
the judgment module for wording rather than rendering it. Its two
structurally parallel siblings do the opposite: `Denial` (`permission.rs:303-348`)
and `LoopDenial` (`loop_permission.rs:226-253`) carry only `.condition() ->
u8`, and every sentence for either — `permission_ledger.rs::denied_detail`/`loop_denied_detail`
(`351-422`, `442-480`) — is built entirely inside the ledger from the
denial's structured fields, with no wording method on either type.
Alternative: keep `StagedDenial` structured-only, as `Denial` and
`LoopDenial` are, and move `writer_form`'s per-variant text and its three
constants into `permission_ledger.rs` beside `denied_detail` and
`loop_denied_detail`.
Where: `staged_permission.rs::StagedDenial::writer_form` (`399-455`) and the
three constants (`369-378`); contrast `permission_ledger.rs::denied_detail`/`loop_denied_detail`.
Reason: none recorded for the placement. `writer_form` and its constants are
present, in substance, from `staged_permission.rs`'s first commit,
`2094da6b` (2026-08-27). The constants' own comments explain a real,
separately-motivated constraint on the wording's *content* — "A remedy a
writer cannot take is worse than no remedy: the blind-writer verification of
2026-08-28 met the condition-2 sentence on a read-to-EOF loop..." — but
nothing argues for that content living on the enum rather than in the
renderer that already holds the other two judgments' wording; `writer_form`
has one caller in the whole tree, the ledger itself.
Reads as an inconsistency rather than a deliberate architectural split:
nothing in the module, the ledger, or the introducing commit argues that
staged denials specifically need renderer-independent wording. The practical
effect is that a wording change to a staged denial is a
`staged_permission.rs` change today while the same kind of change to a
window or loop denial is a `permission_ledger.rs` change — the same
diagnostic-wording-placement asymmetry the resolution-module audit's item 3
names for a different pair of modules.
Effect: diagnostics maintainability only; no acceptance or lowering
consequence.
Decision: staged-loop denial wording moves out of `StagedDenial::writer_form()`
into `permission_ledger.rs`, matching `Denial` and `LoopDenial`, because
keeping every judgment's prose in one place is what the other two sibling
modules already do and no stated reason singles out the staged judgment for
the opposite structure, instead of the current split where two of three
denial types are pure data and the third carries its own finished sentences.

**5. Which ledger facts print on the default channel without `--par-ledger`
is a targeted usability policy that treats [PAR-2] and [PAR-3] denials
asymmetrically.**
`render_ledger` (`permission_ledger.rs:108-278`) sets a line's `notice` field
— the one thing deciding whether an ordinary compile prints it at all — by a
different rule per line kind: a `pair`/`chain` line is never a notice
(`144-183`); a `loop` ([PAR-2]) line is a notice only when denied *and* that
same loop also has a denied [PAR-3] verdict, i.e. an I/O loop whose pipeline
was independently lost too (`138-143`, `199`); a `stage` ([PAR-3]) line is a
notice whenever its own verdict is denied, with no dependence on [PAR-2]
(`241`); and a `place` line is a notice exactly when its own staged loop is a
notice and that row is itself denied (`265`). Of three permission families,
only [PAR-3] denials are unconditionally surfaced; a [PAR-2] loss is
surfaced only when [PAR-3] also lost on the same loop, and a [PAR-1] window
loss is never surfaced without the flag.
Alternative: make every denial a notice, or make none a notice (require
`--par-ledger` uniformly, as `pair`/`chain` lines already do).
Where: `permission_ledger.rs::render_ledger` (`137-143`, `199`, `241`,
`265`); the policy's own statement (`60-68`).
Reason (quoted, `permission_ledger.rs:60-68`): "A `notice` line also belongs
on the default channel, because it is a verdict about a loop the writer
wrote for I/O that the completion model could not stage — the one class of
ledger fact that is a missed optimization on the program in front of them
rather than a reading of the judgment. The blind-writer trial of 2026-08-28
found every I/O loop in five ordinary utilities denied, and the writer heard
nothing: the flag they would have had to know about is the flag they had no
reason to run." This grounds why *I/O loops specifically* needed an
unprompted signal; it does not separately argue why an ordinary
(non-I/O) counted-loop loss, or a sibling-call loss, should stay silent by
default beyond the same trial not having found that case.
Deliberate and evidence-backed, yet unrecorded in a node: the choice reads
as considered — it responds to a specific, cited usability finding rather
than an arbitrary default — but the resulting three-way asymmetry (always /
only-if-also-staged-denied / never) is a diagnostics-surface policy no
`design/` node states, and a later ledger change could reintroduce silence
for the case the 2026-08-28 trial found, or extend "always notice"
somewhere the trial never tested, without contradicting anything written
down.
Effect: diagnostics/usability only.
Decision: a denied [PAR-3] verdict is always a default-channel notice, a
denied [PAR-2] verdict is a notice only when the same loop's [PAR-3]
judgment also denied it, and [PAR-1] pair/chain lines are never a notice,
because the 2026-08-28 blind-writer trial found I/O-loop denials going
unnoticed while every other loss is either expected background
sequentialization or already visible some other way, instead of surfacing
every denial, or none, uniformly.

## Summary

- Covered by the tree: 13
- No decision needed: 10
- Choices without a node: 5 (2 likely defects, 2 deliberate architecture
  choices, 1 deliberate diagnostics policy)
