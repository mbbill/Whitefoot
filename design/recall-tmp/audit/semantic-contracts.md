# Audit: semantic contracts, requires, ensures, and proof commits against the design tree

Module: `compiler/src/semantic/check/requires.rs`, `check/ensures.rs`, `check/contracts.rs`,
`compiler/src/semantic/goal.rs`, `postcondition.rs`,
`compiler/src/semantic/check/control/proofs.rs`, `check/control/commit.rs`,
`check/control/results.rs` (7,059 lines total). Requirement/ensures clause
checking, the goal/relation intermediate representations FN-8 and FN-9 build,
source contracts and conformance-law discharge, local `invariant`/`proof_use`
certificates, and the `set`-statement commit and multi-result-return/
destructuring statements those clauses' obligations interact with. Direction:
code to tree — finding choices the code embodies that `design/` does not
record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/contracts.md`, `design/language/checks-and-proofs.md`,
`design/language/checks-and-proofs/certificate-fold.md`,
`design/language/checks-and-proofs/obligation-discharge.md` and its two
children `goal-decomposition.md`/`loop-fact-retention.md`,
`design/language/checks-and-proofs/requires-entry-contract.md`, and
`design/language/effects.md`. Also read for grounding, since the module's own
comments cite them constantly: the relevant sections of `spec/kernel-spec.md`
(FN-1 through FN-9, MSR-1 through MSR-6, CALL-4, CALL-6, INV-1, PRF-1, ENT-2
through ENT-5, TYPE-5, TYPE-7, LIV-2, VIEW-4, STOR-1, OWN-1, OWN-7, ERR-3,
EFF-1, EFF-2, and DIAG-1's semantic-ordering paragraphs). The entailment
engine (`compiler/src/semantic/entailment*`) is treated as a black box per the
task's own instruction; where a function in this module hands data to it
(`entailment.rs`'s `verified_postconditions: &[Vec<&CheckedPostcondition>]`,
the `AffineCheckState`/`normalize_bounded_less_equal` calls in `proofs.rs`,
`check/publication.rs`'s `relations_are_contradictory`), only the boundary is
described, never its internals. `check.rs` and `check/publication.rs` are
likewise outside the assigned file set; they are read only far enough to see
how they call into or consume the eight assigned files (call-site goal
instantiation, `Uninhabited` disposition, and CALL-6 contradiction detection
all live there, not here).

One scope note: `check/control/commit.rs` (the `set`-statement/LIV-2 checker)
and `check/control/results.rs` (result-list return, destructuring, and
`propagate`) cite ownership/liveness rules (LIV-2, VIEW-4, STOR-1, OWN-1,
OWN-7, ERR-3, TYPE-7) rather than the contract rules proper. They belong to
this family because they are the other half of a real coupling: a `set`
commit's LIV-2 read-out is what makes a binding's `local.live` flag change
mid-statement, and `control/proofs.rs::check_affine_atom`/`check_local_invariant`
read that exact flag to decide whether a value is an admitted affine-proof
operand; and `CALL-4`'s two multi-result "S12 destinations" — a destructuring
`let`'s binders and a `set` target list's targets — are exactly the statement
shapes `results.rs::check_destructuring_let` and `commit.rs::check_commit`
build. `design/language/ownership.md` was not one of the assigned nodes and
was not read; LIV-2/VIEW-4/STOR-1/OWN-1/OWN-7 are checked against
`spec/kernel-spec.md` directly rather than against a design node.

Two of the assigned children, `checks-and-proofs/obligation-discharge/goal-decomposition.md`
and `.../loop-fact-retention.md`, govern how the (black-boxed) entailment
engine decomposes Boolean goals and retains loop-carried facts; nothing in
the eight assigned files performs that decomposition or retention itself —
`control/proofs.rs` only hands the entailment engine finite premise lists
(`CheckedProofUse`) and affine relations for it to fold in. They are read for
grounding and are not cited as covered below for that reason, matching how
the lexer-and-syntax audit excluded grammar decisions with no code in its own
two modules.

Git history note: the repository is the full, unshallowed history back to
the true root `7c1d7641` (2026-07-07); `git log -S`/`-G` and `git blame`
below run against that complete history, not a truncated view.

## 1. Covered by the tree

- `design/compiler.md` (decision 1: one safe-Rust crate, unsafe forbidden) —
  none of the eight files contains an `unsafe` block; the crate-wide
  `#![forbid(unsafe_code)]` (`compiler/src/lib.rs:1`) covers all of them.
- `design/language/contracts.md` (decision 1: contracts and conformances are
  compile-time signature-and-law metadata with no runtime value, ABI
  component, dispatch path, or lowering operation) —
  `compiler/src/semantic/check/contracts.rs`'s `CheckedContract`,
  `CheckedConformance`, `CheckedContractLaw`, and `CheckedLawDerivation`
  construction (`collect_contracts`, `check_conformances_and_laws`) builds
  pure identity/name/signature/law metadata with no lowering call anywhere in
  the file.
- `design/language/contracts.md` (decision 2: member compatibility is exact
  after positional region renaming, with normalized-equal read/write/allocate
  effect components and no subtyping) —
  `compiler/src/semantic/check/contracts.rs::signatures_equal` (line 562),
  `alpha_equivalent_type`/`region_ordinal` (lines 606, 632, the positional
  region renaming), `normalize_mode` (639), and
  `normalize_effects`/`normalize_state_paths`/`normalize_regions` (658, 673,
  694 — order-independent via `sort_unstable`, matching contracts.md's own
  "Rejected: raw source-row equality... breaks signature regularity across
  irrelevant occurrence order, repetition, and region spelling").
- `design/language/contracts.md` (decision 3: a conformance with laws must
  discharge every law, only where a rule names it) —
  `compiler/src/semantic/check/contracts.rs::check_conformances_and_laws`'s
  `undischarged_law` rejection (line 532) when `discharge_domain` (711)
  returns `None`; `discharge_domain`/`law_identity`/`identity_is_zero`
  transcribe FN-4's one-row closed discharge shape exactly (`spec/kernel-spec.md:1708-1725`:
  the bound function's body must be exactly `return p0 +sat p1;`, and the law
  table has exactly one operation row), so there is no second, optimizer-side
  or trust-based proof path.
- `design/language/contracts.md` (decision 4: generic bounds over contracts
  stay absent until a consumer justifies them) —
  `compiler/src/semantic/check/contracts.rs::collect_contracts` (lines
  62-68) rejects a `contract` declaration that carries a `Generics` child,
  and the file contains no bound-checking machinery of any kind.
- `design/language/effects.md` (decision 3: contracts and invariants are
  erased proof syntax and introduce no effect) —
  `compiler/src/semantic/check/control/proofs.rs::check_local_invariant`
  (line 46) returns its `CheckedStatement::Proof` with an explicit
  `EffectSet::NONE` (line 128) regardless of what its premises reference; and
  `check/requires.rs::CheckedRequires`/`check/ensures.rs`'s
  `RelationTemplate` output carry no effects field at all, so a
  `contract_block` cannot contribute to a function's effect row by
  construction, not by a runtime check.
- `design/language/checks-and-proofs.md` (decision 1: automatic derivation is
  fixed and terminating, and a harder proof is a finite explicit certificate
  the checker verifies without rediscovering it) —
  `compiler/src/semantic/check/control/proofs.rs::check_local_invariant`
  builds one finite, ordered `uses: Vec<CheckedProofUse>` (lines 74-118)
  directly from the written `proof_use` nodes; nothing here searches for a
  certificate or rediscovers one.
- `design/language/checks-and-proofs/obligation-discharge.md` (decision 2:
  every certificate step reads the same entering context and publishes
  nothing; one flat weighted combination, no search over step order) —
  `control/proofs.rs::check_local_invariant`'s premise loop (lines 74-118)
  passes the identical `bindings`/`allowed_values`/`loop_depth` to
  `check_ordered_affine_relation` for every `use` premise; no premise's
  admission depends on an earlier premise's own result in the same list.
- `design/language/checks-and-proofs/obligation-discharge.md` (decision 3: an
  integer-typed named const is an affine atom folded to the one value it
  declares) — `control/proofs.rs::affine_named_const` (line 535) and its use
  in `check_affine_atom`'s `DeclarationClass::NamedConst` branch (~759-768):
  the constant is read once and folded to its mathematical value rather than
  excluded from the affine fragment.
- `design/language/checks-and-proofs/certificate-fold.md` (decision 3:
  multiplicities are unsigned, because a signed one would need its own
  nonnegativity obligation) — `control/proofs.rs::invariant_use_multiplicity`
  (142) rejects a `0`/non-canonical literal and `invariant_use_value_multiplicity`
  (199) rejects a signed named-const or a signed/negative local
  ("a use multiplicity is a signed integer, which may scale a premise by a
  negative number", line 255) before the value ever reaches the certificate.
- `design/language/checks-and-proofs/requires-entry-contract.md` (decision
  1's "compiled away entirely" clause) — `check/requires.rs::check_requires`
  and `check/ensures.rs::check_postcondition_clause` never place a
  `contract_block`'s checked statements into `CheckedFunction::body` or any
  other lowering-visible structure; their only outputs (`CheckedRequires`,
  `RelationTemplate`) are proof metadata consumed by the entailment engine,
  never executable code.
- `design/language/checks-and-proofs/requires-entry-contract.md` (decision 2:
  a `define` is a proof-only abbreviation, substituted textually, never
  evaluated, snapshotted, or stored) —
  `check/requires.rs::check_requires`'s `ContractDefine` loop (lines 199-229)
  and `check/ensures.rs::check_postcondition_clause`'s equivalent loop
  (519-562): each definition is checked once through the ordinary `Let`
  path only to extract its binding/value identity into `expanded_bindings`
  (`requires.rs:228`), and the checked `Let` statement itself is discarded —
  every later reference substitutes the recorded `ExpandedClauseExpression`
  rather than reading a stored value.
- `design/language/checks-and-proofs/requires-entry-contract.md` (decision 3's
  "a postcondition about a Result's success value writes `Ok(value: name)`"
  clause) — `check/ensures.rs::validate_postcondition_selector` (1888) requires
  a routed clause's field spelling to be exactly `"value"` (1932) and its
  route target to be the prelude `Ok` identity (1919), matching FN-9's fixed
  `Ok`/`value` PRE-1 identities (`spec/kernel-spec.md:1857`) rather than an
  inferred or writer-chosen field name.
- `design/language/checks-and-proofs/requires-entry-contract.md` (decision 4:
  two requirement clauses are the same goal exactly when they name the same
  operations, types, constants, parameters, and literals after definitions
  expand; no callee prologue) — `compiler/src/semantic/goal.rs`'s derived
  structural `Eq`/`Hash` on `GoalTemplate`/`GoalExpression` (lines 24, 40, 63,
  100, 174) is precisely that criterion — plain recursive structural
  comparison with no commutation, folding, or reassociation, matching
  `spec/kernel-spec.md:1821` ("exact typed-tree equality: there is no
  commutation, folding, reassociation, inversion, or De Morgan rewrite")
  verbatim — and `CheckedRequires`/`GoalTemplate` carry no statement or effect
  of their own for a later caller to execute, so nothing in this module
  inserts a callee-side check.

## 2. No decision needed

- Hand-written recursive-descent walkers over the grammar's own shared
  affine/clause productions (`build_clause_expression`/`build_clause_root`/
  `build_clause_affine`/`build_clause_operand` in `requires.rs`;
  `check_affine_expression`/`check_affine_term`/`check_affine_factor` in
  `control/proofs.rs`), one function per production, run in lockstep with the
  already-typed `CheckedExpression` the ordinary expression checker already
  built — the shared-grammar split between clause and body is MSR-5's own
  rule, and reusing the ordinary checker's row/type selection rather than
  re-deriving it is what the code's own comments give as the reason.
- FN-8's closed rejected/admitted operation lists
  (`validate_clause_operation`'s five rejected spellings —
  `ineg`/`iabs`/`ishl`/`ishr`/`buffer_new`/`box_new`/`arena_new` — and
  `clause_affine_operation`'s three admitted arithmetic rows) transcribe the
  specification's own enumerated sets verbatim, leaving no discretion.
- FN-4's law-discharge shape (`discharge_domain`/`law_identity`/
  `identity_is_zero`, the one-row `+sat` law table) is a literal
  transcription of FN-4's own closed discharge relation: the specification
  states the exact required body shape and the exact closed domain/law table,
  not a family of shapes the implementation chose among.
- INV-1's affine-atom/factor admission (`check_affine_atom`,
  `check_affine_factor`) — a bare IDENT place or one integer literal only, no
  field/deref/subscript, `*` requires one direct literal operand, `i128`
  arithmetic — transcribes INV-1's own enumerated admission list and its
  explicitly stated numeric domain and ceilings (`spec/kernel-spec.md:4104-4112`).
- MSR-1/MSR-5's four measure formers, reached by name through a shared
  out-of-module helper and admitted only as a one-operand call with no type
  argument or `fieldinit_list` (`check_affine_measure`,
  `build_clause_expression`'s `ArrayMeasure`/`BufferMeasure`/`SliceMeasure`/
  `ContainerMeasure` arm) — MSR-1/MSR-5's own EBNF-level constraint, not a
  checker-invented restriction.
- FN-9/ENT-4's difference-bound relation-term shape
  (`RelationTemplate`/`RelationDatum`/`NormalizedRelation` in
  `postcondition.rs`; `postcondition_relation`/`postcondition_relation_term`/
  `postcondition_relation_operand` in `ensures.rs`) is FN-9's own stated "one
  datum displaced by a written constant" term shape
  (`spec/kernel-spec.md:1865-1867`), transcribed rather than designed.
- CALL-4's result-ordinal and route bookkeeping
  (`postcondition_route_ordinal`, `postcondition_route_carrier`,
  `admit_postcondition_selector` in `ensures.rs`) implements CALL-4's own
  closed route-admission sentences (one carrier ordinal, ambiguous when two,
  `Ok`/`value` identities fixed by PRE-1) with no free parameter of the
  checker's own.
- LIV-2/VIEW-4/STOR-1's three-condition `set` commit and its stated ordering
  (`control/commit.rs::judge_commit_admission` checking VIEW-4's loan
  displacement before the copy/read-out disjunct, lines 496-519) — the
  code's own comment gives exactly the reason the rules themselves imply: a
  copy-mode early return would silently skip the loan check for a
  loan-bearing copy type such as `Slice`.
- OWN-7's overlap judgment for two element-commit targets
  (`control/commit.rs::commit_targets_overlap`, line 621) delegates to shared
  `places.rs`/`borrows.rs` helpers (`paths_diverge`, `places_overlap`) and
  states no rule of its own beyond OWN-7's text.
- ERR-3/CALL-4/TYPE-5's result-list return, destructuring-let, and
  destructuring-consume handling (`control/results.rs::check_result_list_return`,
  `check_destructuring_let`, `check_destructuring_consume`,
  `check_propagate_let`) — one function per statement form, each a direct
  transcription of its own rule's stated shape (ordinal-by-ordinal typing,
  same-error-type requirement, declared-field-order destructuring).
- TYPE-7's implicit-read-at-return check
  (`control/results.rs::check_return_implicit_read`, line 495) is paired with
  a compile-time `const _: () = assert!(SemanticRule::Type7.definition_rank()
  < SemanticRule::Own1.definition_rank(), ...)` (lines 17-20) pinning DIAG-1's
  stated rule-rank order — an ordinary defensive compile-time check, not a
  policy choice.
- Two `unreachable!()` calls in `goal.rs`
  (`GoalDatum::projections_mut`/`set_ty`, lines 155, 165) guard a private
  helper's `Literal` arm that every call site already excludes by an earlier
  match on the same enum — the ordinary "statically impossible arm" idiom,
  not a departure from the crate's `Result`-based failure style elsewhere in
  these eight files.
- Checked/saturating arithmetic and `u32::try_from(..).map_err(|_| ...CounterOverflow)`
  throughout instead of raw arithmetic — ordinary overflow-safety practice
  used crate-wide, unrelated to language semantics.
- Only `Vec`/`HashMap`/`HashSet` collections across all eight files, no
  custom allocator or `unsafe` — ordinary technique matching
  `design/compiler.md`'s "simple implementations over ordinary collections."
- `ExpandedClauseExpression`/`ExpandedClauseDatum` (`requires.rs`), one shared
  alpha-expansion walker reused by both FN-8 requirement templates and FN-9
  relation templates before each downcasts to its own narrower shape
  (`into_goal_expression` for FN-8, `postcondition_relation_datum` for FN-9),
  rather than two near-duplicate walkers — explained in the type's own doc
  comment as reuse, not an architecture fork with real alternatives worth a
  node.
- `ensures.rs::postcondition_result_placeholder`'s technique of handing the
  ordinary expression typer a synthetic zero-valued placeholder for a
  selector spelling, then discarding it and rebuilding identity from the
  resolver-owned selector-use record — a contained, self-explained trick for
  reusing the ordinary typer rather than writing a selector-aware one.

## 3. Choices without a node

**1. `V031_CANDIDATE_SEMANTICS` is a permanently-`true`, never-toggled switch
whose disabled branches are dead code — the same anti-pattern
`design/compiler.md` already names and rejects for a sibling module.**
`compiler/src/semantic/mod.rs:65` declares
`pub(crate) const V031_CANDIDATE_SEMANTICS: bool = true;`, doc-commented as
the "master switch for the v0.31 candidate's gated semantic surface." Five
call sites branch on it: `check/requires.rs:273` (the FN-8-vs-OWN-1
mechanical-fix rewrite), `check/types.rs:1704` (rejects struct-typed
`cvalue` construction outright when false), `:1815` and `:1922` (two more
CONST-2 eligibility branches), and `check.rs:1606`
(`constant_declaration_is_deferred`). None of the five `if !V031...`/match
guards is reachable: nothing in the crate ever builds with a different
value, and no test toggles it (confirmed by search of
`compiler/src/semantic/tests.rs` and `tests/*.rs`) — unlike the resolution
module's analogous `Inventory` switches, which at least back a stated
differential test. The active specification is v0.53, twenty-two versions
past the "v0.31 candidate" the flag's name and doc comment still describe.
Alternative: delete the constant and its five dead branches now that its
semantics are unconditionally shipped, the way a superseded switch is
retired once its successor is the only implemented behavior.
Where: `compiler/src/semantic/mod.rs:57-65`; `check/requires.rs:271-287`
(`clause_conditional_repair`); `check/types.rs:1694-1710, 1814-1829, 1920-1924`;
`check.rs:1605-1617` (`constant_declaration_is_deferred`).
Reason (quoted): the flag started at `false` in commit `1fc2aebe` ("semantic:
gated v0.31 candidate surface — struct consts and clause repair") and was
flipped by `fdacd71f` ("compiler: flip the three v0.31 switches and activate
O11 decomposition"), which states: "The candidate at spec/kernel-spec.md is
now the branch's own authority, so the compiler implements it rather than
v0.30... V031_CANDIDATE_SEMANTICS (semantic/mod.rs) all become `true`, with
their doc comments restated in the candidate's terms." The same commit
explains why a third candidate flipped in the same batch, O11 decomposition,
got no switch at all: "O11 had no switch by design — the recorded one-path
doctrine forbids a second acceptance mode." No commit before or after
explains why V031_CANDIDATE_SEMANTICS, unlike O11, was kept reachable after
activation instead of deleted along with its now-dead branches.
Judgment: drift. `design/compiler.md`'s Rejected list already refuses exactly
this shape of choice, found in the resolution module's `Inventory` switches:
"Superseded system-inventory states kept reachable behind compile-time
switches so that a differential test can show an earlier program's module
unchanged: rejected because the compiler implements exactly one
specification, the active one, so a switch that reconstructs a superseded
inventory has no consumer and only adds code paths to maintain."
`V031_CANDIDATE_SEMANTICS` is the same anti-pattern one layer over — a
superseded-candidate-vs-active switch hardcoded to the active side — and is
in fact weaker than the case the tree already rejected, since it backs no
stated differential test at all. It is also a plain violation of this
project's own repository-hygiene rule: "Supersede in place... Do not
accumulate parallel versions, stale dossiers, or abandoned experiments beside
their replacements."
Draft Decision: Retire `V031_CANDIDATE_SEMANTICS` and its five dead branches
now that the v0.31 candidate is the specification's sole active behavior,
because the compiler implements exactly one specification and a switch whose
disabled side has no build, test, or spec consumer only adds code paths to
maintain, instead of leaving an activated candidate's gate reachable in the
source.
Effect: none on acceptance today (the constant is unconditionally `true`, so
every program compiles exactly as if the dead branches were already
deleted). Purely a hygiene and legibility cost, compounded by the name and
doc comment now actively misdescribing the current specification version.

**2. `check_affine_expression`/`check_affine_term`/`check_affine_factor`
recurse natively with no depth ceiling of their own — very likely the still-open
"unguarded affine expression nesting depth" crash `docs/todo.md` records, now
newly attributable to this module rather than the parser.**
`compiler/src/semantic/check/control/proofs.rs::check_affine_expression`
(line 407), `check_affine_term` (467), and `check_affine_factor` (556) are
three ordinary, mutually-recursive Rust functions walking a
`header_invariant`/`invariant_stmt`/`use_premise`'s parsed
`affine_expr`/`affine_term`/`affine_factor` subtree; a parenthesized factor
(`check_affine_factor`'s final branch, ~609-620) recurses back into
`check_affine_expression` with no counter, no explicit work-list, and no
`Checker`-level resource ceiling — nesting depth in the source is bounded
only by the native call stack. This is the checker's own walk, distinct from
and prior to two separately-bounded stages: `checked_affine_expression`
(same file, line 933) converts the already-built tree using an explicit
`Vec`-based work list (`pending: Vec<AffineConversion>`), and the entailment
engine's `normalize_bounded_less_equal` enforces INV-1's stated ceilings
(4096 scheduled nodes/input terms/result terms, `spec/kernel-spec.md:4112`)
— but neither runs until after `check_affine_expression` has already
returned, i.e. after the native recursion has already gone exactly as deep
as the source did.
Alternative it implicitly refuses: the same iterative task-stack technique
the parser already uses for this identical threat (per the finished
lexer-and-syntax audit: `ParseLimits::max_tasks`/`max_frames`-bounded
explicit stacks in `parser/engine.rs`, `parser/diagnostic.rs::probe`,
`parser/finalize/shape.rs::verify`), or the technique this same file already
applies one function later, to the same data, for the entailment handoff.
Where: `compiler/src/semantic/check/control/proofs.rs:407-465`
(`check_affine_expression`), `:467-528` (`check_affine_term`), `:556-622`
(`check_affine_factor`, recursive branch ~609-620).
Reason: no reason recorded for choosing native recursion here. Git blame
traces these three functions to the same feature work that first added
`header_invariant`/`invariant_stmt` checking, with no comment weighing an
iterative alternative — unlike the parser's own architecture record
(`4ecc14dd`, "Record production compiler architecture"), which states the
identical threat model ("hostile tokens try to exhaust lookahead, nesting,
and list storage") and its iterative-stack response explicitly, for the
frontend only.
Judgment: likely defect, and — read together with `docs/todo.md` — a live,
previously-unattributed one. `docs/todo.md`'s "Unguarded affine expression
nesting depth" entry: "A proof-domain affine expression nesting parentheses
about 1400 deep aborts the driver with a stack overflow and no diagnostic
(exit 134)... It reaches this from both a `use` premise and an `invariant`
target, so it is in the shared affine-expression handling." That is exactly
this file's shared `check_affine_expression`/`check_affine_term`/
`check_affine_factor`, reached identically from `check_local_invariant`'s
own target (`AffineProofOwner::InvariantTarget`) and its `use` premises
(`AffineProofOwner::ProofUse`) — the todo item's own two reported entry
points. The finished lexer-and-syntax audit examined this same recorded
defect from the parser side and explicitly could not attribute it there,
leaving the question open ("this audit did not find the crash's cause inside
lexer/syntax, but the project's own record does not yet rule either side
out"); the todo item itself says the fix "wants a bisect between the parser
and the semantic former." This audit's reading of the actual recursion shape
in this module resolves that open bisect in the semantic checker's favor:
unbounded native recursion sits exactly where the todo item's two reported
call paths converge, one module handoff downstream of the parser's own
already-bounded machinery.
Draft Decision: Bound `check_affine_expression`/`check_affine_term`/
`check_affine_factor`'s recursion the same way `checked_affine_expression`
already bounds its own walk of the same data, with an explicit work-list and
a `Checker`-level ceiling that fails closed to a resource failure rather
than the native stack, because a deeply but legally nested proof-domain
expression must fail as a resource limit and not as an unguarded
native-stack crash, instead of leaving nesting depth bounded only by the OS
thread stack.
Effect: safety, not acceptance for any program that does not crash — a
well-formed, non-adversarial affine expression is checked identically
either way. As recorded it is a real, reachable defect: past some
source-controlled nesting depth the compiler aborts with no diagnostic
(exit 134) instead of a clean `[INV-1]`/resource-ceiling rejection, against
both `design/compiler.md`'s "a rejection names the numbered rule it violates
and the location" and the parser's own stated threat model for exactly this
input shape.

**3. Postcondition-selector admission runs a complete throwaway duplicate of
nominal/constant/signature construction before the real check, to satisfy
DIAG-1's resolution-interleaved subjudgment.**
`check/ensures.rs::preflight_postcondition_selectors` (line 169) does real,
complete semantic work that is then discarded entirely:
`prepare_postcondition_selector_preflight` (234) calls
`declare_nominals_for_postconditions`, `collect_constants_for_postconditions`,
`collect_function_templates_for_postconditions`, and
`collect_concrete_function_signatures_for_postconditions` — a second,
parallel pass over nominal declaration, constant evaluation, and
function-signature construction, distinct from the tables the ordinary
checker later builds for real. Its own doc comment states the reason and
discipline directly (163-168): "Performs the one semantic subjudgment that
DIAG-1 interleaves into resolution. This checker is throwaway: it reuses the
ordinary nominal, constant, generic-cycle, signature, and FN-2
implementations, but none of the scratch identities or tables are published
to the real checker." The real pass, `admit_postcondition_selectors` (398),
repeats the same `eligible_postcondition_functions`/`admit_postcondition_selector`
judgment over the crate's real tables, and its own comment records the
intended invariant rather than testing it: "The verdict-bearing form of the
same validation already ran in the throwaway preflight; any divergence here
is a compiler invariant failure" (396-397).
Alternative it implicitly refuses: compute the semantic facts
postcondition-selector admission needs once, in a form both the
resolution-stage diagnostic ordering and the real checker can read, rather
than building and discarding a complete scratch universe of `NominalId`/
`FunctionId`/signature tables first.
Where: `compiler/src/semantic/check/ensures.rs:169-263`
(`preflight_postcondition_selectors`, `prepare_postcondition_selector_preflight`,
`forward_delayed_postcondition_issue`), `:304-392`
(`eligible_postcondition_functions`, shared verbatim by both passes),
`:398-445` (`admit_postcondition_selectors`/`admit_postcondition_selectors_including`,
the real pass).
Reason: the ordering requirement this satisfies is spec text, not an
implementation choice — `spec/kernel-spec.md:2268`: "Within an admitted
routed `ensures_clause`, the route's leading lookup and [FN-9]
route-admission subjudgment occur before lexical resolution of that clause
expression"; line 2405: "Apart from FN-9's explicitly interleaved
selector-admission subjudgment above, after complete lexical resolution
succeeds, source semantic checking covers the complete closed unit." No
commit message or design node explains why satisfying that ordering
constraint should take the shape of a complete, discarded duplicate pass
rather than, for instance, caching the facts the first pass computes for
reuse by the second; `design/recall-tmp/` and `mcts_mem/` were searched for
"preflight"/"postcondition"/"selector" and returned no design-tree hits
beyond this audit and the two `checks-and-proofs` nodes already read. A test
(`compiler/src/semantic/tests/postconditions.rs:1985`,
`selector_preflight_precedes_unrelated_entry_form_semantics`) confirms the
*ordering* is deliberately covered, but exercises no case where the two
passes could actually diverge from one another.
Judgment: a genuine architecture choice without a node, not drift — nothing
in the tree forbids this shape, and the doc comments show it was chosen
deliberately and is well understood by its author, not stumbled into. It is
the same general shape as the finished lexer-and-syntax audit's own item #4
(a second full pass re-deriving what an earlier stage could supply), where
at least a design record states the "independent evidence lanes" principle
for why; no equivalent record exists here, and this pass's own comment
frames it as ordering-driven rather than evidence-driven, which is a
different justification than that record gives.
Draft Decision: Satisfy FN-9's DIAG-1-mandated interleaved route-admission
subjudgment with a throwaway duplicate of nominal/constant/signature
construction that publishes nothing to the real checker, because the two
passes must reach the same verdict from independently built tables so a
compiler-invariant failure catches any future divergence between them,
instead of caching and sharing the first pass's facts with the second.
Effect: performance and structure, not acceptance — every compiled unit that
declares even one `ensures_clause` builds its nominal/constant/signature
universe twice. The two passes are asserted by comment, not enforced by any
test this audit found, to always agree; a real divergence between them would
surface only as `SemanticCompilerFailure::InvalidResolution`/a
compiler-invariant failure rather than as a diagnostic naming which pass was
wrong.

## Summary

- Covered by the tree: 14
- No decision needed: 16
- Choices without a node: 3 (1 drift/hygiene, 1 likely defect, 1 architecture
  choice)
