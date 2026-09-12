# Audit: semantic ownership, borrows, and linearity against the design tree

Module: `compiler/src/semantic/check/{borrows,linearity,cleanup,confinement,
publication,result_state_origin,support}.rs`, `compiler/src/semantic/
target_action.rs`, `compiler/src/semantic/entailment/affine.rs` (about 8,450
lines). Ownership/borrow/reborrow admission, region and overlap judgment,
linearity and the release graph, confinement, declared-relation consistency,
compiler-derived state-origin routing, target-action summaries, and the
deterministic affine arithmetic core. Direction: code to tree — finding
choices the code embodies that `design/` does not record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/ownership.md` and all of `design/language/ownership/`
(`affine-replacement.md`, `copy-classification.md`, `no-reborrow.md`,
`slice-result-provenance.md`), `design/language/surface-form/borrow-lexicon.md`,
`design/language/surface-form/result-propagation.md`,
`design/language/system-interface.md`, `design/compiler/cleanup-traversal.md`.
Also read for grounding, since this module's own comments cite them
constantly: `design/language/effects.md`, `design/language/data-model.md`,
`compiler/src/semantic/check.rs`, `model.rs`, and `check/types.rs` (none of
which is in this audit's file list, but each is where this module's output is
consumed or its input is produced), and the relevant sections of
`spec/kernel-spec.md` ([OWN-1] through [OWN-14], [BLK-1] through [BLK-4],
[VIEW-1]/[VIEW-2], [PROV-1] through [PROV-6], [STOR-1] through [STOR-5],
[EFF-1] through [EFF-5], [CALL-6], [S31], [S37]). `compiler/src/semantic/
tests.rs` and `tests/` were read only where a specific finding below needed
evidence of intended behavior, per the README.

Three scope notes that shape section 1, found by tracing where a node's own
subject is actually admitted or emitted:

- `design/compiler/cleanup-traversal.md` — despite the name matching this
  module's own `check/cleanup.rs`, the decision it records (one release
  action per node type that calls itself where a type's release graph
  closes, avoiding an allocating explicit worklist) is implemented in
  `compiler/src/backend/emitter/cleanup.rs` (lowering/code emission), a
  same-named but different module outside this audit's family.
  `check/cleanup.rs` (this module) only collects the release *sites* a
  checked function body carries for [EFF-2] effect attribution; it never
  walks a type's release graph to emit a drop routine, and the cycle
  question the node answers does not arise in it.
- `design/language/ownership/no-reborrow.md`'s decision 4 (a completed
  owned-enum match header or exact Boolean conditional header ends only the
  loans it created, before the selected arm) is implemented in
  `compiler/src/semantic/check/control/matches.rs`
  (`header_loan_base`/`statement_loans.borrow_mut().truncate(...)`), outside
  this module's files.
- `design/language/ownership/affine-replacement.md` and
  `design/language/surface-form/result-propagation.md` are both admitted —
  the `replace`-of-slice/arena refusal and `propagate`'s own type and
  consume-once checks — in `compiler/src/semantic/check/control.rs` and
  `check/expressions.rs`, outside this module. This module's files only
  consume the already-checked `CheckedStatement::Replace` and
  `CheckedStatement::PropagateLet` shapes for state-origin tracking
  (`result_state_origin.rs`) and release/effect bookkeeping (`cleanup.rs`,
  `target_action.rs`), so they are cited below only for that.

Git history note: the clone is complete back to the true root `7c1d7641`
(2026-07-07); `git log -S`, `git blame`, and full commit bodies were read for
every "Reason" below, and a source search across
`design/recall-tmp/sources/commits.md`, `pull-requests.md`,
`research/investigations/io-model/`, and `docs/todo.md` was run for the
section-3 items specifically. "No reason recorded" means the introducing
commit and its successors say nothing.

## 1. Covered by the tree

- `design/compiler.md` (decision 1: one safe-Rust crate, unsafe forbidden) —
  none of the nine files contains an `unsafe` block (confirmed by search);
  only `Vec`/`HashMap`/`HashSet` throughout.
- `design/compiler.md` (decision 2: no SMT, deterministic and terminating,
  never an exponential family relying on a timeout or a budget) —
  `compiler/src/semantic/entailment/affine.rs`'s own module doc comment ("It
  performs no heuristic search and every `i128` operation is checked. Work is
  counted for measurement, but a cumulative compiler budget never changes
  whether a source proposition is accepted") and `AffineCheckState::charge`
  (line 563), which can never itself fail — only the fixed structural
  ceilings (`AffineCheckLimit`) do.
- `design/compiler.md` (decision 3: an unimplemented-but-valid source stops
  as an explicit unsupported capability, never a rejection) —
  `compiler/src/semantic/check/borrows.rs`'s `unsupported(...)` calls
  (`UnsupportedSemanticFeature::RegionsAndBorrows`, `::ArenaRuntime`) in
  `check_borrow`, `check_owned_content_borrow`, and
  `resolve_dereference_holder`, each reached only after every source
  rejection for that position has already been judged and passed.
- `design/compiler.md` (decision 4: a rejection names the numbered rule and
  the location; no acceptance path depends on iteration order) —
  `compiler/src/semantic/check/support.rs::{issue_node, issue_at, issue_value}`
  (every `SemanticIssue` carries a `SemanticRule` and a `SemanticLocation`);
  `borrows.rs:83-87`'s `definition_rank` const-assertions, which pin
  same-node judgment order (a suspended holder before a reborrow admission,
  OWN-10 before OWN-14) to DIAG-1's first-defined-rule rule at compile time.
- `design/language.md` (decision 1: every required fact machine-checked and
  erased before lowering, no runtime trap) — `target_action.rs`'s whole
  design: `derive_target_actions` cannot itself fail or reject, runs only
  after every function body is already accepted, and produces pure
  backend-facing metadata the language itself never observes.
- `design/language/ownership.md` (decision 1: explicit regions, a checker
  that rejects when it cannot establish safety) — `borrows.rs::region_outlives`
  (two distinct region parameters are unconditionally incomparable, never
  assumed related, matching OWN-3's own fail-closed answer) and the
  pervasive pattern of an explicit `unsupported` stop wherever a written form
  falls outside what the checker can currently prove, rather than a guess.
- `design/language/ownership.md` (decision 2: overlap judged conservatively
  over complete resolved paths) — `borrows.rs::places_overlap` (same
  declaration root and non-diverging paths); the literal-index/field
  disjointness judgment itself is `places.rs::paths_diverge`, shared across
  the whole semantic checker rather than reimplemented here.
- `design/language/ownership/affine-replacement.md` — the checked statement
  model every file in this module switches over has no "bare take" variant
  at all, only `CheckedStatement::Replace` with its old-value binder and
  replacement in one operation; `result_state_origin.rs:311-347` and
  `cleanup.rs`'s release-site collection both read that mandatory shape
  directly. (Admission is outside this module; see the scope note above.)
- `design/language/ownership/copy-classification.md` (decisions 1-2) —
  `linearity.rs::{LinearityClass, generic_parameter_class, check_linearity_bound}`
  (the `copy < affine < linear` chain and its `satisfies` ceiling test,
  checked once against a generic body's own declared bound and never
  re-tightened per concrete argument).
- `design/language/ownership/no-reborrow.md` (decisions 1-3) —
  `borrows.rs::{ReborrowPosition, check_child_reborrow, check_borrow}` and the
  `OWN6_ARGUMENT_POSITION`/`OWN6_HOLDER`/`OWN6_STATEMENT_SCOPE`/
  `OWN10_LOCAL_STORAGE`/`OWN14_RESTRUCTURING` constants: exactly the bounded
  family the node names (statement-scoped call-argument child, mode-preserving
  returned reborrow of a parameter or let holder, and the call-result
  single-provenance-candidate position, internally still called "the reborrow
  extension" but unconditional — no compile-time switch gates it). (Decision
  4 is outside this module; see the scope note above.)
- `design/language/ownership/slice-result-provenance.md` (decisions 1-2) —
  `borrows.rs::{SliceInfo, parameter_slice, push_slice_origin}`: a formal
  slice's origin set is exactly one `CheckedSliceOrigin::FormalSlice`,
  computed from the parameter's own signature and never from a callee's body.
- `design/language/surface-form/borrow-lexicon.md` (decision 2: the exclusive
  mode is spelled `&uniq`, not `&mut`) — `borrows.rs::BorrowKind::Unique` is
  read and written only through `crate::FixedTerminal::Uniq` throughout
  `check_borrow`/`check_child_reborrow`; no `Mut` terminal is read anywhere in
  the module.
- `design/language/surface-form/result-propagation.md` — the checked model's
  `CheckedStatement::PropagateLet`, as `result_state_origin.rs:260-287` and
  `cleanup.rs` read it, carries a bare scrutinee and a binder with no
  explicit-move operand, matching the node's "ordinary consuming rule" text
  exactly. (Admission is outside this module; see the scope note above.)
- `design/language/system-interface.md` (decision 1: ordinary owned resources
  under ordinary ownership, no capability category, no interior-mutability
  patch) — `confinement.rs`'s [BLK-4] provider exception ("a provider
  parameter is the one `&uniq` this rule does not refuse") and
  `linearity.rs::scope_holds_store_capability`, which tracks a `Heap<'s>`
  capability as an ordinary live binding of an ordinary parameter type, never
  as a distinguished language feature.
- `design/language/system-interface.md` (decision 2: resource contracts state
  outcomes, ownership, cleanup, and capacity; completion and host scheduling
  are lowering concerns) — the clean split between `cleanup.rs::{release_of_type,
  release_row_of_type}` (the `SystemRelease{action, row}` ownership/cleanup
  fact, built from the compiler-owned `system_resource_contract`/
  `system_release_row` tables) and `target_action.rs::derive_target_actions`
  (the separately-computed, non-source-visible scheduling/`may-suspend`
  summary built from those same release records' `target_action` field) —
  the two concerns the node distinguishes are computed by two different
  functions in two different files in this same module, never conflated.

## 2. No decision needed

Borrows and places:
- `places_overlap` is same-root-plus-non-diverging-path, one line, delegating
  the real field/index judgment to shared code (`places.rs::paths_diverge`).
- `region_outlives` is a direct lexical-scope-containment check; a local
  region never outlives a caller-supplied parameter and two distinct
  parameters are always incomparable, both the conservative reading OWN-3
  already requires.
- A whole statement is treated as one liveness program point
  (`Simultaneity::OneStatementIsOneMoment`, `declaration_is_used_at_or_after`)
  — a direct consequence of the flat, one-operation-per-statement surface
  form the parser already enforces, not an invented refinement.
- Elided/default region and mode resolution (`parse_mode`, `borrow_expr_region`,
  `region_declared_at`, `enclosing_region`, `loop_body_region_owner`,
  `append_elided_formal_regions`) mechanically reads what [FORM-8] already
  fixes syntactically; nothing here infers a region the grammar leaves open.
- `borrowable_type`/`borrow_addresses_storage` are closed dispatches over the
  checked type inventory, each arm one type shape; array content and an
  unsubstituted generic are explicit `unsupported` stops (the function's own
  doc comment says so) rather than silently wrong answers.
- `check_declaration_region_spelling` is a direct transcription of [FORM-8]'s
  own written-order, first-occurrence rule; no heuristic naming is involved.
- `enclosing_loops`/`borrow_region_is_inside_current_loops` walk the tree's
  own ancestor chain once per query; ordinary tree-shaped bookkeeping.

Reachability closures:
- The reachability worklists (`confinement.rs::{reaches_confined_surface,
  reaches_the_entry_heap}`, `linearity.rs::{release_graph_nodes,
  owned_components}`, `cleanup.rs::{release_row_of_type, drop_paths,
  residual_drop_paths}`) all repeat the same "fields, enum variant payloads,
  box referent, arena content" postorder walk — one traversal shape reused
  per concern, not reinvented four times.
- `confinement.rs` as a whole is a close-to-line-by-line transcription of
  [BLK-4]'s own exhaustive spec text: the two named container nominals, the
  provider exception, and both refusal messages with their mechanical fixes
  are dictated by the rule itself, leaving no real implementation choice.

Release, effects, and linearity:
- `cleanup.rs::{collect_release_sites, collect_expression_release_sites,
  collect_drop_release_sites}` are one-arm-per-constructor walks over the
  checked statement/expression tree with no judgment beyond dispatch.
- `linearity.rs::LinearityClass`'s three-value chain and
  `capability_released_stores`/`vector_release_class`'s fail-closed
  classification (misclassifying an extent as a general store would drop a
  free, so the two extent cases are the ones positively identified) are
  direct transcriptions of [PROV-6, S37]'s own stated rule.
- The "the ambient heap is the sole provider in this version" simplification
  threaded through `linearity.rs` reflects the current specification's own
  scope — one general-store type as of this version — not a compiler-invented
  limitation; the code says so itself at each site.
- `publication.rs`'s declared-relation contradiction closure reuses [ENT-4]'s
  own transitive difference-bound composition rather than a second algorithm
  (the file's own comment says as much); its [BLK-0] kernel-row form is
  checked once by this repository's own unit test rather than at every
  compilation, because a fixed compiler-owned table's consistency is a
  one-time fact about the compiler and not a per-program question.

Call-graph summaries and arithmetic:
- `target_action.rs::derive_target_actions` is an ordinary monotone-union
  fixed point over the call graph — a standard, textbook dataflow technique
  with no real alternative worth debating — computed only after every
  function body is already accepted and consumed only by lowering.
- `affine.rs`'s canonical-form merge and insert (`merge_scaled`,
  `insert_coefficient`) are an ordinary sorted-vector merge; its four
  `AffineCheckLimits` ceilings are the same fixed-capacity pattern the rest
  of the compiler already uses, which `docs/todo.md`'s own proposed repair
  for a related gap (below) explicitly names as the pattern to extend.
- Checked/saturating `i128` arithmetic and closed enums throughout
  (`AffineCheckError`, `AffineTermId`) — ordinary overflow- and type-safety
  practice, unrelated to language semantics.

Shared support:
- `support.rs` is pure boilerplate: one job per accessor (`declaration_at`,
  `use_at`, `issue_node`, `unsupported`, ...), each a direct, obvious reading
  of the resolved program's own tables.

## 3. Choices without a node

**1. Compiler-derived release attribution routes a value's opaque state
identity through a whole-program, body-derived fixed point over the call
graph — the same shape two sibling ownership decisions in this exact area
explicitly reject for closely analogous problems, and the specification's
own EFF-2 text says should not be needed at all.**

`result_state_origin.rs::derive_result_state_origins` walks every concrete
function's *checked body* (`OriginAnalyzer::{analyze, scan_statement,
scan_match, scan_loop, expression}`) to build one `CheckedResultStateOrigin`
per function (`model.rs:1685-1703`) — for a `CheckedExpression::UserCall`,
the analyzer looks up the *callee's own* summary
(`result_state_origin.rs:558-590`, `self.summaries.get(function.0 as usize)`)
and iterates the whole function set to a fixed point (`result_state_origin.rs:705-723`,
"`next == summaries` ... break"), with `OriginSet::Absent` standing in as
"the recursive fixed-point bottom only for a user call whose callee has not
published a route yet" (comment at :643-648) — i.e. the mechanism is built to
survive a recursive or mutually recursive call group, not just an acyclic
one. This runs before ordinary per-function acceptance checking
(`check.rs:1320`, right after `validate_generic_templates` and before the
comment "Phase A completes every reachable concrete function before any
acceptance-bearing entailment judgment runs"), gated by a
`deriving_result_state_origin` flag that also *suppresses* the declared/
exhibited effect-row exactness check during that preliminary pass
(`check.rs:2003, 2015`). Once computed, the result is read back at
`check/types.rs::expression_state_origins` (:1289-1330, again keyed by
`self.result_state_origins.borrow().get(function.0 as usize)`) to decide a
returned value's `CheckedStateOrigins`, which `cleanup.rs::effects_of_row`
(:369-386) and `borrows.rs::effect_paths_for_place` (:382-416, in this
module) both consume to decide which of the *current* function's own formal
paths a compiler-derived release's `writes(path)` gets attributed to for the
*real*, acceptance-bearing effect-row exactness check (`check.rs:2015-2028`).

Alternative: the two closely analogous provenance problems already solved
elsewhere in this exact design area both reject exactly this shape.
`design/language/ownership/slice-result-provenance.md`: "A call computes its
result's origin set from signatures alone... because a body-derived exact
summary would make callable contracts depend on implementation bodies and
need fixed points for recursive call groups, which conflicts with the
signature-complete caller boundary, instead of body-derived origin
summaries," with a `Rejected:` line naming the identical alternative this
module implements almost verbatim: "A body-derived exact origin summary:
rejected because callable contracts would depend on implementation bodies
and recursive call groups would need fixed-point handling."
`design/language/ownership/no-reborrow.md` decision 3 makes the same choice
for call-result borrow provenance: "because a summary derived from the
callee's body would make callable contracts depend on implementations and
need fixed points over recursive call groups, instead of body-derived
provenance." Both existing decisions instead make the analogous fact
signature-derivable (a slice's origin set from the signature, a reborrow's
provenance from "exactly one parameter of the result's kind naming the
result's region").

Where: `compiler/src/semantic/check/result_state_origin.rs` (whole file: the
`OriginSet`/`OriginEnvironment`/`OriginAnalyzer`/`OriginFlow` machinery and
`derive_result_state_origins`); this module's own consumers,
`compiler/src/semantic/check/cleanup.rs:369-386` (`effects_of_row`) and
`compiler/src/semantic/check/borrows.rs:382-416` (`effect_paths_for_place`);
supporting context outside this module, `compiler/src/semantic/model.rs:1685-1703`
(`CheckedResultStateOrigin`/`CheckedResultStatePath`),
`compiler/src/semantic/check.rs:756, 1282, 1320, 1783, 2003-2028, 2122-2127`,
and `compiler/src/semantic/check/types.rs:1160-1330`
(`state_origins_of_value`/`expression_state_origins`).

Reason: no reason is recorded for choosing this shape specifically. The
introducing commit, `ea40acd1` (2026-08-26, "io: rebuild completion model
from first principles"), is bodiless. The governing investigation,
`research/investigations/io-model/FIRST-PRINCIPLES.md`, describes the
*general* call-boundary rule as ordinary signature substitution — "§18.2.3:
At a call, substitute each callee formal path with the resolved actual place
and then project it to the current function's formals" — matching
`spec/kernel-spec.md:1996-2028`'s [EFF-2] text exactly ("each callee effect
path selects its root formal's actual argument and appends its static field
suffix to that actual's resolved place"), and its own remaining-work list
still marks the harder case unsettled as of that document: "§23.6. Complete
direct-call result-state routing and per-path completion release." No later
commit or PR body (searched via `commits.md`/`pull-requests.md`) revisits the
fixed-point shape itself or weighs it against a signature-level alternative.

Effect: this is not diagnostics-only — it changes what a function's own
*acceptance-bearing* exhibited effect row can be, because
`result_state_origins[callee]` is read while computing a *caller*'s exhibited
row (via `effects_of_row`/`effect_paths_for_place`), so a function whose
signature, declared effect row, and body are all unchanged can flip between
accepted and rejected purely because some other function it calls changed
its *body* in a way that changes that callee's own `CheckedResultStateOrigin`
without changing that callee's own declared row. This reads as a discrepancy
with `spec/kernel-spec.md:2013` ("Binding, moving, passing, returning,
borrowing, reborrowing, and slicing preserve the existing resolved place
identity. This is the same identity tracking already required by ownership
and move checking; EFF-2 adds no parent link, result ancestry, resource
root, or second provenance system") and with `spec/kernel-spec.md:2060`
("The returned owner of a successful resource-producing operation is a fresh
ordinary value. It carries no hidden ancestry to the parameter that produced
it") — both state a categorical, signature/table-level fact for the cases the
specification actually details (a *system* operation's result, and identity
preserved by ordinary moves), and neither authorizes the additional,
separate, whole-program summary this module builds for an *ordinary user
function's* result. The underlying reason such a summary was reached for at
all is real: unlike a slice or a reborrow, an ordinary owned nominal type
(a wrapped system resource, for instance) carries no region or other
signature-visible annotation a writer could use to state "this result's
state comes from parameter N," so nothing today lets a writer make the fact
this module derives checkable from the signature alone the way [BLK-4]'s
[STOR-5] confinement or [VIEW-1]'s loan-bearing types already are. Whether
the owner's resolution is to constrain this mechanism to a signature-derivable
rule (as the two sibling decisions above chose for their own domains), to
add a source-level annotation for opaque result-state provenance, or to
amend EFF-2's own text to license exactly this compiler-internal technique,
the tension between the code, the two sibling decisions, and the
specification's own words is real and currently unrecorded anywhere.

**2. `AffineExpression`'s recursively boxed tree has no bounded, iterative
`Drop`, so a deeply nested proof-domain expression can still overflow the
stack when it is finally dropped, even though every forward walk this module
performs over the same type is already iterative.**

`AffineExpression` (`affine.rs:29-39`) is `Add`/`Subtract`/
`MultiplyByConstant`, each holding one or two `Box<AffineExpression>`
children, with no custom `Drop` impl (confirmed by search) — Rust's
compiler-generated drop glue for such a type recurses one native stack frame
per nesting level. Every *traversal* the module performs over the same type
is deliberately iterative instead: `normalize_expression` (:616-676) uses an
explicit `pending: Vec<NormalizeExpression>` task stack specifically so that
forming a value from a written expression never recurses.

Alternative: an iterative `Drop` for the owning value (the same fix pattern
already used everywhere else in this file — an explicit task stack in place
of native recursion).

Reason: none recorded; the file's history shows no attention to `Drop` at
all, and the builder of `AffineExpression` from parsed syntax
(`compiler/src/semantic/check/control/proofs.rs`, outside this module) was
not audited for the same property.

Effect: this squarely matches, and adds a specific, previously unidentified
half to, the already-recorded defect in `docs/todo.md`: "Unguarded affine
expression nesting depth. A proof-domain affine expression nesting
parentheses about 1400 deep aborts the driver with a stack overflow and no
diagnostic (exit 134)... it is in the shared affine-expression handling. The
repair is the pattern already used for structural limits, the 4096-entry
`proof_use` capacity and `AffineCheckError::LimitExceeded`, applied to
nesting depth in whichever of the parser and the semantic former overflows."
The sibling `lexer-and-syntax` audit already traced the *construction* side
and found the parser's own derivation, its diagnostic re-walk, and its shape
finalizer are all iterative task-stack machines with no native recursion,
leaving the question of where exactly the overflow comes from explicitly
open ("this audit did not find the crash's cause inside lexer/syntax, but
the project's own record does not yet rule either side out"). This audit's
reading of `AffineExpression`'s own definition supplies a concrete candidate
for the other side of that open question: however the tree is built, dropping
it — at the end of one statement's checking, or at the end of compilation —
recurses through ordinary derive-based drop glue regardless of how carefully
every *forward* walk over the same value is kept iterative elsewhere in this
file. `docs/todo.md` already tracks the user-visible defect and names the
fix pattern to extend; this is not a new tree decision but a likely defect
worth the owner's attention, and worth attaching to that existing item rather
than treating as newly discovered.

**3. Two rejection kinds carry no mechanical-fix text at any of their call
sites — [OWN-5]'s borrow-conflict, the single most-cited outcome of this
module's exclusivity checker, and [PROV-6]'s linearity-bound mismatch on
both its axes — while every sibling rule in the same two files carries one.**

`SemanticIssueKind::BorrowConflict` is a bare, field-less variant, used six
times in this module (`borrows.rs:1069` for OWN-11, and `:1747, 1928, 2004,
2016, 2094` for OWN-5) plus once more in `check/expressions.rs:1756` (outside
this module, confirming the same bare form is the standard way every part of
the checker reports an exclusivity conflict) — none of the seven call sites
attaches any restructuring guidance, and the variant has no field to carry
one. `SemanticIssueKind::LinearityBoundMismatch` does carry named fields
(`parameter`, `bound`, `argument`, `actual`) but no `mechanical_fix` at
either of its two call sites in `linearity.rs`: `check_linearity_bound`
(:585-594, the type axis) and `check_region_linearity_bound` (:622-636, the
region axis) — the only two `SemanticIssueKind` sites in `linearity.rs`
without one. By contrast every other rejection either file raises carries a
`mechanical_fix`: OWN-6's own is three separate constants
(`OWN6_ARGUMENT_POSITION`, `OWN6_HOLDER`, and `OWN6_STATEMENT_SCOPE`, the
last a full worked example naming a specific test program,
`tests/programs/dir_walk.wf`, and a named three-part idiom); `borrows.rs`'s
own `InvalidRegionBound` (:441) has one; and `linearity.rs`'s other eight
sites (`LinearValueNotConsumed` twice, `LinearValuePartiallyConsumed`,
`LinearModifierOnCopyNominal`, `DisposeOfLoanBearingOperand`,
`DisposeOfLinearNode`, `DisposeWithoutCapabilityLeaf`, `DisposeHasNoProvider`)
all have one.

Alternative: give both kinds a mechanical fix too (even a generic one for
OWN-5 — "the existing loan/borrow this conflicts with must end first, or
take a `&` where `&uniq` is not required" — and, for `LinearityBoundMismatch`,
something like "instantiate this parameter with an argument of the bound's
own class, or loosen the written bound" — would match the coverage of every
sibling rule in the same two files), or state explicitly why these two are
deliberately left to the exposed place/coordinate and named classes alone.

Where: `compiler/src/semantic/check/borrows.rs:1069, 1747, 1928, 2004, 2016,
2094`; `compiler/src/semantic/check/linearity.rs:588, 625`.

Reason: none recorded for either. `git log -S BorrowConflict` reaches back
to `0f02b9f4` ("Implement lexical buffer borrowing"), one of this checker's
earliest commits, and every commit since that touches the variant (through
`d0dd08df`'s reborrow extension and `ea700f7f`'s io/state-effects rebuild)
changes only which call sites raise it, never whether it carries a fix. A
search of `commits.md` for "BorrowConflict" finds only discussions of when
the rule is *correctly* or *incorrectly* raised (e.g. `75b39de6`'s
already-resolved v0.20 gap), never a discussion of its diagnostic wording.
`git log -S LinearityBoundMismatch` similarly reaches back to its
introducing commit, `0288b650` ("B5: linearity, the release graph, and the
destructuring forms"), with no body and no later commit discussing the gap;
`commits.md` has no mention of the kind name at all.

Effect: diagnostics quality only, no effect on acceptance or performance —
but the asymmetry is stark for OWN-5 given it is, by construction, the
rejection every one of the six sites in this module exists specifically to
raise, and it is the rule a writer meets first when moving from "no
aliasing rules" languages to this one's exclusivity discipline. This reads
as a likely gap rather than a deliberate choice for both kinds: the same
curated, encountered-not-planned pattern the `lexer-and-syntax` audit found
for `SyntaxIssue::mechanical_fix` (added "as found" when an AI writer
process hit a bad default) appears to have simply never been pointed at
these two rejections, one of them the most common in the whole module.

## Summary

- Covered by the tree: 15
- No decision needed: 17 (7 borrows/places, 2 reachability closures, 4
  release/effects/linearity, 3 call-graph/arithmetic, 1 shared support)
- Choices without a node: 3 (1 significant architecture choice reading as a
  discrepancy with the specification and two sibling decisions, 2 likely
  defects)
