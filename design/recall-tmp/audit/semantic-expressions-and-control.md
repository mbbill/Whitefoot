# Audit: semantic expressions, calls, control flow, and the check driver against the design tree

Module: `compiler/src/semantic/check.rs` (the check driver, ~3,917 lines);
`compiler/src/semantic/check/expressions.rs` (~2,314 lines); `check/expressions/calls.rs`
(~904 lines) and `check/expressions/calls/{conversions,floating,kernel,reinterpret,system,user}.rs`
(~72/106/1,105/60/414/1,589 lines); `check/control.rs` (~1,097 lines) and
`check/control/{matches,loops}.rs` (~816/651 lines); `check/entry_form.rs`
(~462 lines). About 13,500 non-test lines total. Direction: code to tree —
finding choices the code embodies that `design/` does not record, not the
reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/surface-form.md` and its five children (`borrow-lexicon`,
`construction-form`, `match-form`, `operation-spelling`, `result-propagation`),
`design/language/effects.md`, `design/language/pattern-doctrine.md`,
`design/language/checks-and-proofs/requires-entry-contract.md`,
`design/language/parallelism.md` and `parallelism/permission-judgment.md`,
`design/recall-tmp/README.md` and `sources.md`, and the finished audits
(`resolution.md`, `lexer-and-syntax.md`) as the format. Also read for grounding
since the code cites them constantly: the relevant sections of
`spec/kernel-spec.md` — [DIAG-1]'s stage order and attribution rows, [FORM-8],
[GRAM-6]/[GRAM-8]/[GRAM-9]/[GRAM-11], [OP-1]/[OP-2], [EFF-1]/[EFF-2], [FN-1]
through [FN-9], [OWN-5]/[OWN-6]/[OWN-11]/[OWN-12]/[OWN-14], [GIVE-1],
[LIV-1]/[LIV-2], [ENT-2], and [STOR-2] through [STOR-5].

Two scope notes:

- `design/language/parallelism.md` and `parallelism/permission-judgment.md` have
  no implementing code in these eleven files at all: a counted loop's [PAR-2]
  permission verdict is computed in `compiler/src/semantic/loop_permission.rs`
  and `permission.rs`, both outside this module. A whole-family grep for
  `PAR-` or `parallel` under `compiler/src/semantic/check/` returns nothing.
  They are read here only for orientation; the design/compiler.md audit for
  those files is the one that can actually check them against the code.
- `design/language/checks-and-proofs/requires-entry-contract.md`'s grammar and
  clause-checking substance (the `contract_block` shape, `define` expansion,
  per-clause proof) lives in `check/requires.rs`, `ensures.rs`, and
  `contracts.rs`, all explicitly excluded from this audit. Only the driver's
  own orchestration of that material — the order `check_function_signature_body`
  calls into it, and the "a contradictory instance is still fully checked"
  guarantee — is visible in `check.rs` and cited below.

Git history note: this audit ran directly against the already-unshallowed
repository (2,376 commits back to the true root `7c1d7641`, 2026-07-07; see
`design/recall-tmp/sources.md` for the correction history). Every "Reason"
line below is sourced with `git log -S`, `git blame`, and the introducing and
successor commits' own bodies read in full; "no reason recorded" means those
bodies say nothing more. Rule citations quoted from the code were spot-checked
against the exact current spec text they name (ENT-2, OWN-5, OWN-12, GRAM-6,
GIVE-1, FN-7 among them); none of the checks performed here turned up a
citation mismatch like the FN-3/FN-4 one the resolution audit found — the
citations sampled all matched the spec's own wording, in several cases
verbatim down to the restructuring text.

## 1. Covered by the tree

- `design/compiler.md` (decision 1: safe-Rust research instrument, ordinary
  collections, no unsafe) — the whole module: not one `unsafe` block across
  the ~13,500 lines; every collection is `Vec`/`HashMap`/`HashSet`, every
  side channel a `RefCell`/`Cell`.
- `design/compiler.md` (decision 2: admits and lowers by the specification's
  rules alone, never by recognizing a name, signature, or shape) —
  `compiler/src/semantic/check/expressions/calls/kernel.rs`'s own module doc
  ("Nothing here is keyed on a spelling. The record's own shapes decide which
  parameter supplies which of the row's type, const and region parameters")
  and its `KernelShape`-driven `kernel_parameter_type`/`kernel_shape_type`;
  `check/entry_form.rs`'s closed, ordinal-addressed `COMMAND_INPUTS` table,
  read by declared position and never by a writer's chosen order.
- `design/compiler.md` (decision 3: an unimplemented capability stops as an
  explicit gap, never a rejection) — the `self.unsupported(UnsupportedSemanticFeature::_, node)`
  calls threaded through every file in this family: e.g.
  `check.rs::check_function_signature_body`'s arena-parameter stop,
  `expressions.rs::resolve_struct_path`'s subscript-through-composite gap and
  `check_owned_content_set_target`'s box/arena mutation gap,
  `calls/kernel.rs::check_kernel_call`'s `ContainerRuntime` stop on an
  inexpressible requirement, and `control/matches.rs::join_states` /
  `control/loops.rs::check_loop`'s `OwnershipJoin` stop when a control-flow
  join's bindings disagree beyond liveness.
- `design/compiler.md` (decision 4: a rejection names the numbered rule and
  location; no acceptance path depends on iteration order; which of several
  violations is reported first is not part of the language) —
  `check.rs::entailment_rejection`'s `ProofPosition`/`min_by` selection
  ("`min_by` keeps the first of several equal minima, so the selection
  depends only on this order and on collection order, never on a hash") and
  its multi-function rejection sort (`rejections.sort_by` on `NodePath`
  components, `concrete_instance_rank`, then `rule.definition_rank()`).
- `design/language/surface-form.md` (decision 3: flat three-address form, one
  operation per expression) — `expressions.rs::check_infix` ("[GRAM-9] admits
  exactly one operation per expression, so there is no precedence to apply")
  and `check_written_operand`'s exhaustive three-way dispatch (`atom` /
  `call` / `construct`, nothing else).
- `design/language/surface-form.md` (decision 4: a body binder's mode and
  type are derived from its right-hand side, never written) —
  `control.rs::check_let` ("An `ordinary_let_rhs` is always self-typed
  [TYPE-5], so it is checked with no expectation and the binder takes what
  it produces") and its `ValueMatch`/`ValueIf`/`PropagateLetRhs`/`ReplaceLetRhs`
  arms, none of which reads a written mode or type off the binder.
- `design/language/surface-form.md` (decision 5: two iteration forms, an
  ordinary loop with break and an ascending half-open counted loop) —
  `control.rs`'s `Production::LoopStmt` / `Production::ForStmt` dispatch to
  exactly `control/loops.rs::check_loop` / `check_counted_range`, the complete
  iteration surface this driver admits.
- `design/language/surface-form/construction-form.md` (decisions 1 and 2:
  every field named in declared order, no positional form, naming applies
  even to a single-payload variant) — `expressions.rs::check_construct` /
  `check_regional_construct` (a field-count mismatch and a name/order
  mismatch both cite GRAM-8; there is no positional path at all, and a
  nullary constructor is checked through the same field-count judgment).
- `design/language/surface-form/match-form.md` (decision 1: the conditional
  form is fixed by the scrutinee's type; a Bool takes `if`, an enum takes
  `match`) — `control/matches.rs::match_descriptor` (a Bool scrutinee is a
  hard GRAM-6 rejection of `match`, "spell the Bool conditional `if`") and
  `check_if`'s reuse of the exact two-tag `bool_descriptor()` a Bool `match`
  would use.
- `design/language/surface-form/match-form.md` (decision 2: a conditional
  value is a `let` initializer whose every arm or branch ends in exactly one
  `give` or diverges) — `control.rs::check_let`'s value-initializer arm
  (the [GIVE-1] `all_paths_deliver`/`delivered` machinery), shared verbatim
  by `check_match` and `check_if` through one `MatchResult`.
- `design/language/surface-form/match-form.md` (decision 3: the scrutinee is
  a full expression, not restricted to a place) —
  `control/matches.rs::check_match` / `check_conditional` both call
  `check_match_expression`, the ordinary `check_consuming_expression`, with
  no place-only restriction anywhere in the path.
- `design/language/surface-form/match-form.md` (decision 4: the else-free
  form is the one spelling of the empty alternative; an empty `else` block is
  rejected; an `else` holding exactly one `if` flattens to `else if`) —
  `control/matches.rs::reject_unspellable_else`, which rejects exactly those
  two spellings and no others.
- `design/language/surface-form/operation-spelling.md` (decisions 1 and 2:
  arithmetic/comparison spelled infix, every other table operation a named
  call; explicit `wrap`/`checked`/`sat`/`defined` modes, no implicit
  bare-trapping default) — `expressions.rs::infix_operation`'s closed token
  table (`+`, `+defined`, `+wrap`, `+checked`, `+sat`, …) and
  `calls.rs::check_integer_operation_operands`'s `checked_error` handling,
  which always produces an explicit `Result<T, Overflow/DivError>` rather
  than a silent trap.
- `design/language/effects.md` (decision 1: an effect row is checked exact in
  both directions; padding is forbidden even where a later proof could
  remove the effect) — `check.rs::check_function_signature_body`'s [EFF-2]
  comparison (`exhibited.written_row() != signature.declared_effects.written_row()`),
  reporting both the `missing` and the `extra` categories via
  `effect_row_difference`.
- `design/language/effects.md` (decision 2: effects describe ordinary state
  including opaque system resources; reads/writes name parameter paths; host
  scheduling belongs to target lowering) —
  `calls/system.rs::project_system_call_effects` and
  `calls/kernel.rs::project_kernel_call_effects` fold system- and
  kernel-operation state access into the very same `EffectSet::add_read`/
  `add_write` an ordinary user call uses; `CheckedExpression::SystemCall`'s
  `target_action` field carries dispatch metadata entirely outside the
  effect row, which itself (`check.rs::EffectSet`) has only
  reads/writes/allocates fields — no external, blocks, or traps category.
- `design/language/effects.md` (decision 3: contracts and invariants are
  erased proof syntax and introduce no effect) —
  `check.rs::check_function_signature_body` folds only `checked.effects`
  (the body block) into the exhibited row; the separately checked
  `requirements` and postcondition relations never enter it.
  `control/loops.rs::form_loop_invariants` returns `CheckedLoopInvariant`
  values that `check_loop`/`check_counted_range` carry beside, never inside,
  their effects union.
- `design/language/checks-and-proofs/requires-entry-contract.md` (decision 5:
  a function instance whose requirements contradict each other is legal and
  uncallable, and the compiler still checks its structure, ownership,
  effects, and return shape) — `check.rs::check_function_signature_body` runs
  the complete ordinary structural/ownership/effect/return-shape judgment
  unconditionally for every signature; whether its requirements are
  satisfiable is decided later, in entailment, never gating this pass.
- `design/language/pattern-doctrine.md` (decision 1: a closed catalog of
  program architectures, not an open-ended one) — `entry_form.rs` admits
  exactly one entry shape ([FN-7]'s `command` kind) against one closed,
  ordinal-addressed `COMMAND_INPUTS` table; a second `program_kind`, a
  foreign `input_label`, or a direct call to the entry are each a hard
  rejection with no second admitted shape anywhere in the file.

## 2. No decision needed

- `HashMap<DeclarationId, LocalBinding>` cloned once per branch (an `if`/
  `match` arm, a loop body, a region block) and reconciled afterwards by
  `join_states`/`LocalBinding::same_except_region_loans` — the ordinary
  clone-and-join technique for a branching dataflow checker, matching the
  compiler's own "simple implementations over ordinary collections" charter.
- Interior mutability (`RefCell`/`Cell`) on `Checker` for `pending_nominals`,
  `derived_consts`, `statement_loans`, `commit_read_outs`,
  `elided_store_brand`, `general_store_reachable`,
  `template_spelling_authority`, `deriving_result_state_origin`,
  `active_postcondition`, and `active_result_datums` — lets every expression-
  and statement-checking function stay `&self` while still recording narrow
  side-channel discoveries; ordinary Rust technique, not a language rule.
- Checked/saturating arithmetic and `try_from`/`checked_add` on every dense
  counter (`BindingId`, `CheckedLoopId`, `NominalId`, `DerivedConstId`), each
  failing closed to `SemanticCompilerFailure::CounterOverflow` rather than
  wrapping — the same overflow-safety convention the lexer/syntax and
  resolution audits already found compiler-wide.
- Every internal invariant but one (flagged in §3) is threaded through
  `Result<_, CheckStop>` and a handful of `SemanticCompilerFailure`/
  `UnsupportedSemanticFeature` variants rather than a panic; confirmed by
  grepping all eleven files for `.unwrap()`/`.expect(`/`panic!`/
  `unreachable!`/`todo!` in non-test code.
- Long string-match dispatch tables (`infix_operation`, `check_operation`'s
  spelling chain, `float_operation`, `measure_former`, `view_former`)
  translate one of the language's own closed OPNAME/keyword spellings to a
  checked-operation enum; this matches against the language's fixed
  vocabulary, not a source declaration's name, so it is not the name-based
  special-casing decision 2 forbids.
- Table-driven [BLK-0]/[SYS-2] call typing (`KERNEL_OPERATIONS`,
  `SYSTEM_OPERATIONS`, the `KernelShape`/`KernelGenericKind` matches in
  `calls/kernel.rs`) is reached by declared ordinal and shape, never by
  re-deriving the operation from source syntax.
- `Checker::check_program`'s stage order (entry form and system-call
  arguments first; then nominal/constant/function-signature collection; then
  per-function phase-A checking; then contract/law collection; then
  entailment) follows the driver's own DIAG-1-cited comments and needs no
  separate decision — a mechanical consequence of "an unsupported capability
  establishes no source violation and must never mask one."
- `EffectSet`, `ResultSignature`, `FunctionSignature`, and their kin are
  ordinary owned-`Vec`-based value types with no arena or interning scheme —
  matching the research-compiler simplicity charter, not a distinct choice.
- Deterministic tie-break keys (`ProofPosition`, `concrete_instance_rank`,
  `SemanticRule::definition_rank`) are ordinary `Ord` values compared with
  `min_by`/`sort_by`; the general requirement is a tree decision (§1), but
  the specific key shape is ordinary engineering under that requirement.
- `check_integer_operation_row` (checks written atoms) and
  `check_integer_operation_operands` (checks already-typed operands) are
  split because a `clause_expr`'s affine operands are not all plain `atom`
  nodes [MSR-5] — a direct grammar consequence, not an independent policy.
- The `operation_atoms`/`reject_named_operation_arguments`/
  `reject_written_operation_type_argument` trio of small, one-purpose helper
  functions, each named for the rule it enforces and called once per
  table-operation family, mirrors the "one small function per rule" pattern
  the resolution audit already found compiler-wide.
- `CheckedFunctionInventory`'s separation of the checked `CheckedFunction`
  from `binding_names` — kept only to render an owner in a release-attributed
  EFF-2 diagnostic — is an ordinary diagnostic-only sidecar, not a semantic-
  model choice.
- `check_reservation_placement`/`reserving_occurrence_names`
  (`calls/kernel.rs`) walk the syntax tree with a parent-pointer scan up and a
  descendant scan down rather than a precomputed index — a one-off, whole-
  function scan, matching the "filter at query time, no separate index" style
  the resolution audit found for a comparable structure.
- Extensive fixture-driven coverage in `tests.rs`/`tests/*.rs` (one file per
  rule family, positive and negative cases, several exercising every
  `UnsupportedSemanticFeature` variant) — ordinary test engineering.
- `Checker::new`'s flat `Vec::new()`/`HashMap::new()`/`RefCell::new()`
  initializer list, one field per concern and no builder — ordinary struct
  construction.
- `SemanticLocation::SourceNode`/`BundleRoot` values are read straight from
  the caller's own `TreeView` (`self.tree.path(node)?`, never a fabricated
  path) — a direct consequence of [DIAG-1]'s closed location sum, not a
  separate compiler policy.

## 3. Choices without a node

**1. A permanently-true reborrow-extension switch still gates two live call
sites, unlike its sibling switches, which were deleted the same week.**
`REBORROW_EXTENSION_ACTIVE: bool = true` is threaded as `Checker.reborrow_extension`
and read at exactly two sites: `control.rs`'s `let`-binding capability-stop
guard and `calls/user.rs::result_borrow_candidate`. Every production and
test entry point — `check_semantics`, `check_semantics_dark`,
`check_semantics_arithmetic_obligations`, `check_semantics_division_obligations`,
and even the dedicated `check_semantics_reborrow_extension` test helper —
passes `true`, directly or via the constant; nothing anywhere in the
repository constructs a `Checker` with `reborrow_extension: false`, so each
read's negative arm (`user.rs`'s `return None`, and the left side of
`control.rs`'s `&&` failing to short-circuit) is dead code, unexercised even
by a test.
Alternative: delete the field, the constant, and the two guards, folding
each into the code its `true` arm already always takes — the repair the
project applied to its own sibling switches within 48 hours of this one.
Where: `compiler/src/semantic/check.rs:673` (`REBORROW_EXTENSION_ACTIVE`),
`:682,1248,1253` (the `Checker.reborrow_extension` field and its two
constructor sites), `:825-829` (`check_semantics_reborrow_extension`, whose
own doc comment already calls it redundant); `compiler/src/semantic/check/control.rs:666`;
`compiler/src/semantic/check/expressions/calls/user.rs:103-105`.
Reason (quoted): introduced `false` by `d0dd08df` (2026-08-17, "compiler:
implement the reborrow extension behind its v0.31 switch": "default false,
v0.30 semantics byte-identical; the activation change flips one constant …
Tests cover both switch directions"), flipped permanently `true` the same
day by `fdacd71fe` ("compiler: flip the three v0.31 switches and activate
O11 decomposition": "The candidate at spec/kernel-spec.md is now the
branch's own authority, so the compiler implements it rather than v0.30 …
Each workstream had already implemented and tested its complete judgment
behind its switch; no other line of those judgments changes." — the same
commit states the contrasting norm for a feature that never had a switch:
"O11 had no switch by design — the recorded one-path doctrine forbids a
second acceptance mode.") The very next day, two commits deleted a sibling
switch (the declaration-provenance ambiguity rejection) for exactly this
reason: `baf18897` ("compiler: delete OWN-6's unreachable binding-side
ambiguity rejection": "The arm, the issue kind, and the switch that guarded
it are deleted rather than kept as a second, unreachable rejection path,
exactly as the CHECK_DISSOLUTION residue was. With the switch gone the
test-only forced-on entry says nothing the default entry does not, so its
callers move to the ordinary one …") and `4d41d6bf` ("compiler: migrate body
checks to claims and retire the dissolution switch": "… are all superseded
and deleted rather than kept as a second, unreachable rejection path.") No
commit since `fdacd71fe` revisits `REBORROW_EXTENSION_ACTIVE` itself across
the roughly twenty spec versions between v0.31 and the current v0.53; the
current spec text (`kernel-spec.md:683-780`, OWN-6/OWN-14) states the
reborrow-extension forms unconditionally, with no candidate framing left.
The identical switch shape recurs once more in this same file family, one
step outside it: `V031_CANDIDATE_SEMANTICS` (defined `compiler/src/semantic/mod.rs:65`,
also flipped permanently `true` by the same `fdacd71fe`) is read at
`check.rs:1606` inside `constant_declaration_is_deferred`, where its
negative arm is equally dead; its other two use sites are in `check/types.rs`
and `check/requires.rs`, both outside this audit's files.
Deliberate or defect: reads as an oversight, not a deliberate choice — it is
the same residue the project explicitly named and removed twice in the same
window for its sibling switches, and nothing in the record explains why
this one was spared.
Effect: no acceptance difference (the shipped compiler already always
admits the extension); a real, small structural cost — two live conditionals
and a `bool` field carry no information, and the `check_semantics_reborrow_extension`/
`check_semantics_dark` doc comments exist only to explain a distinction that
no longer holds.

**2. Phase-A function checking discovers a derived nominal type by retrying
the same function from scratch to a bounded fixpoint, rather than a separate
type-discovery pre-pass.**
`check_function_interning_nominals` wraps the `&self` call into
`check_function_signature`/`check_function_inventory` in a `loop`: whenever
checking a function body first names a derived type not yet interned — a
locally formed `box<T>`, a `store_box`, a compiler-owned result list, an
`arena<'r, T>`, the arena-storage list nominal, a prelude instance, or a
source-nominal instance at a new region — the relevant `check_*` site pushes
a `PendingNominal` entry into `self.pending_nominals` (a `RefCell<Vec<_>>`)
and returns the private `CheckStop::DeferredNominal`; the driver drains and
interns every pending entry, then re-invokes `check_function_inventory` for
the *same* function index from the beginning, repeating until an attempt
interns nothing new, at which point it fails closed to
`SemanticCompilerFailure::InvalidResolution`.
Alternative: a dedicated pre-pass enumerating each function's derived types
once — mirroring how `collect_function_signatures`/`declare_nominals`
already make a first, separate pass over *written* types — before the
ordinary `&self` body check ever runs, so no function is checked twice.
Where: `compiler/src/semantic/check.rs:372-398` (`PendingNominal`), `:706`
(`pending_nominals`), `:1813-1857` (`check_function_interning_nominals`);
the pushing sites in `compiler/src/semantic/check/expressions/calls.rs`
(`check_arena_new`, `check_box_new`) and
`compiler/src/semantic/check/expressions/calls/kernel.rs`
(`kernel_result_type`).
Reason (quoted, commit `3a409d7c0c`, 2026-08-08, "semantic: intern the box
nominal a box_new derives" — the mechanism's origin, generalized from boxes
alone to the current seven `PendingNominal` variants by `27316895` the same
day and by later per-feature commits, none of which revisits the retry-vs-
pre-pass choice itself): "The repair is the ruled one: the checker interns
what it derives, and no normative byte moves. `check_box_new` records the
missed referent and returns a private `CheckStop::DeferredBoxNominal`; the
`&mut self` driver drains the pending referents, interns them, and checks
that one function again. Each attempt must intern at least one new nominal,
which bounds the loop by the finitely many referent types a function can
name … The guard that function checking interns nothing becomes a guard
that it interns only boxes, which is the invariant it was really
protecting." The stated reason is for keeping function-body checking itself
`&self`, not for choosing retry-to-fixpoint over a pre-pass; no later commit
weighs that specific alternative.
Deliberate or defect: a deliberate, load-bearing architecture choice — it is
what lets every expression-checking function stay `&self`, which the
interior-mutability fields throughout `Checker` (§2) all depend on — not a
defect.
Effect: performance and structure only, not acceptance: the same programs
are accepted either way, but a function naming several distinct
not-yet-interned derived types is checked once per distinct type it
introduces, in exchange for no separate type-discovery pass and no
`&mut self` plumbing through the whole expression-checking surface.

**3. A missing-`main` rejection is deliberately held back behind a full
second phase-A pass so a more specific per-declaration violation can
preempt it.**
`reject_missing_main_last` fires only when `check_entry_form` failed
specifically with `SemanticRule::Fn7`/`MissingMain`; on that one failure it
re-runs almost the entire phase-A pipeline a second time (system-call
arguments, nominal declaration, constants, function-signature collection,
postcondition-selector admission, generic-template validation,
result-state-origin derivation, then every function's own signature check)
and, if that salvage run meets an established source or resolution issue
first, reports that instead; any other outcome (success, an unsupported
capability, an internal failure) falls back to the original `MissingMain`.
Alternative: report `MissingMain` the moment `check_entry_form` fails, with
no salvage pass — simpler, cheaper, and equally compliant with the tree's
own "which of several violations is reported first is not part of the
language."
Where: `compiler/src/semantic/check.rs:1752-1793`.
Reason (quoted, commit `c44fdfebed`, 2026-08-17, "compiler: execute
region-confined arenas and judge STOR-4 confinement"): "Missing `main` is
now reported after per-declaration source rejections. DIAG-1 leaves the
order among rejection events at distinct nodes implementation-defined, and
a declaration's own established violation is the more useful report.
Verified against the base compiler: none of the eleven main-less
conformance cases changes its reported rule."
Deliberate or defect: deliberate, and regression-checked at introduction
against the existing main-less conformance corpus.
Effect: diagnostics only, and only on the failing path — a main-bearing
unit never runs the salvage pass — but every unit that is missing `main`
now pays a second traversal of most of phase A to decide which of two rules
to report, in exchange for naming a writer's own declaration defect instead
of the coarser whole-unit one when both are true.

**4. One `.expect()` in `check_float_operation` is the module's sole
exception to an otherwise universal no-panic discipline.**
`check_float_operation` re-derives `float_operation(spelling)` and calls
`.expect("caller dispatches only closed floating-point operation names")`
on it, even though its one call site already tested
`floating::is_float_operation(spelling)` — the same computation —
immediately beforehand. Grepping all eleven files in this family for
`.unwrap()`, `.expect(`, `panic!`, `unreachable!`, and `todo!` in non-test
code finds exactly this one hit; every other comparable "this should
already be guaranteed" situation in the same files instead returns
`Err(SemanticCompilerFailure::InvalidResolution.into())` or an `Option`.
Alternative: `float_operation(spelling).ok_or(SemanticCompilerFailure::InvalidResolution)?`,
matching every sibling site in the same module family; or have the one
caller pass the already-computed `CheckedFloatOperation` down instead of
re-deriving it from the spelling a second time.
Where: `compiler/src/semantic/check/expressions/calls/floating.rs:56-57`;
the guarding call site is `compiler/src/semantic/check/expressions/calls.rs:90-92`.
Reason: no reason recorded. The line is unchanged since the function's
introduction in `d38eb4ccd` (2026-07-23, "Implement strict scalar floating
point"), a bodiless commit message; `git blame` finds no later touch.
Deliberate or defect: reads as an overlooked outlier rather than a
considered exception — nothing distinguishes this one dispatch-consistency
invariant from the many others in the same files that fail closed instead,
and the panic is unreachable today only by construction of the one caller.
Effect: none today (the panic cannot currently fire); the risk is only that
a future second call site, or a refactor of the existing one, could
reintroduce a reachable panic where every sibling function in the module
fails closed instead.

## Summary

- Covered by the tree: 18
- No decision needed: 16
- Choices without a node: 4 (2 deliberate architecture/diagnostics choices,
  2 likely oversights)
