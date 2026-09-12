# Audit: semantic types and data model against the design tree

Module: `compiler/src/semantic/{model.rs, kernel.rs, tree.rs, places.rs, mod.rs}`
(module structure only for `mod.rs`) and
`compiler/src/semantic/check/{types.rs, nominals.rs, nominal_instances.rs,
generics.rs, floats.rs, expressions/places.rs, expressions/flat_storage.rs}`
plus `check/expressions/flat_storage/{borrowed.rs, slices.rs}` — about 18,150
lines total. This is the checked-program data model (`CheckedType`,
`CheckedValue`, `CheckedExpression`, `CheckedNominal*`, the [BLK-0] kernel
signature table, the syntax-tree query facade, and place/borrow-holder
resolution) plus the checkers that build it for types, nominals and their
generic instances, generic parameters, floating-point literals, explicit-deref
places, and flat storage (array/buffer/slice/run) expressions and places.
Direction: code to tree — finding choices the code embodies that `design/`
does not record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/data-model.md` and its child
`design/language/data-model/tag-only-equality.md`,
`design/language/ownership/copy-classification.md`,
`design/language/surface-form.md` and its children `construction-form.md` and
`operation-spelling.md`, `design/compiler/tag-only-lowering.md`. Also read for
grounding, since this module's own comments cite them constantly:
`design/language/ownership.md` and its children `affine-replacement.md`,
`no-reborrow.md`, `slice-result-provenance.md`; and the relevant sections of
`spec/kernel-spec.md` (§4 data model and constants, §5 ownership/OWN-1, [BLK-0]
through [BLK-4], [MSR-1]/[MSR-2], [VIEW-1]/[VIEW-2], [CONST-1]/[CONST-2],
[TYPE-2]/[TYPE-5]/[TYPE-6]/[TYPE-7], [FORM-5]/[FORM-7]/[FORM-8], [FN-2]/[FN-6]).
`compiler/src/semantic/tests.rs` and `tests/` were read only as evidence of
intended behavior, not audited.

Two scope notes, parallel to the lexer-and-syntax audit's own:

- `mod.rs` is 1,467 lines but is assigned to this audit only for its module
  structure (the `mod`/`pub use` declarations and the `V031_CANDIDATE_SEMANTICS`
  gate). Its `SemanticRule`/`SemanticIssueKind`/`SemanticIssue` machinery is
  shared diagnostic infrastructure for every semantic rule family — ownership,
  contracts, control flow, effects, and so on, not only types and data — so it
  is read here for context and cited only where a decision is specifically
  about data representation, not exhaustively catalogued rule by rule.
- Two rule families this family's own code visibly feeds but does not itself
  decide: `design/compiler/tag-only-lowering.md`'s bit-width choice is backend
  code (`compiler/src/backend/`), outside this family — this family supplies
  only the front-end classification (`CheckedNominal::is_copy`,
  `CheckedFlatElement::TagOnlyNominal`, `CheckedVariant::tag: u32`) the
  lowering reads. Similarly, `design/language/data-model/tag-only-equality.md`'s
  operator-selection judgment (which operator syntax at a call site produces a
  `CheckedExpression::EnumEquality`) is decided in
  `compiler/src/semantic/check/expressions/calls.rs`, outside this family —
  this family owns only the resulting data shape.

Git history note: the repository clone available for this audit is already
unshallowed (2,475 commits, real root `7c1d7641`, 2026-07-07), so every
"Reason" line below is checked directly against the full history with
`git log -S`/`-G`, `git show`, and reading the introducing commit's body; no
shallow-clone correction was needed. "No reason recorded" means the
introducing commit and its successors say nothing.

## 1. Covered by the tree

- `design/compiler.md` (decision 1: one safe-Rust crate, simple
  implementations over ordinary collections) — the whole family: no `unsafe`
  block anywhere in it; every collection is `Vec`/`HashMap`/`HashSet`;
  `check/floats.rs`'s canonical-float search is a small self-contained,
  provably-bounded routine rather than an external shortest-round-trip crate.
- `design/compiler.md` (decision 2: admits and lowers by the specification's
  rules alone) — `compiler/src/semantic/kernel.rs`'s `KERNEL_SIGNATURES` table
  is a direct transcription of [BLK-0]'s twelve-row signature table (own
  module doc: "This table carries what checking needs, in a closed shape
  language that is exactly as wide as the twelve rows of the inventory"), held
  to that transcription by its own tests
  `every_resolved_row_has_the_record_it_resolves_to`,
  `every_row_is_complete_over_every_measure_it_writes`, and
  `no_row_publishes_a_contradictory_relation_set`.
- `design/compiler.md` (decision 3: valid specified source the compiler has
  not implemented stops as an explicit unsupported capability, never a
  rejection) — `compiler/src/semantic/mod.rs::UnsupportedSemanticFeature`
  (thirteen variants, each commented with exactly why it is a compiler gap:
  `RecursiveNominalLayout`, `BoxReferentMove`, `ExclusiveViewOverArray`,
  `ArenaRuntime`, `ContainerRuntime`, and so on);
  `check/nominals.rs::reject_recursive_nominal_layouts` (an iterative
  three-color DFS over the nominal field graph that stops a genuine value
  cycle here rather than overflowing a representation);
  `check/generics.rs::generic_cycle_components` (a call cycle whose const
  arguments would mint an unbounded instance set stops here rather than
  looping forever or being silently truncated).
- `design/compiler.md` (decision 4: a rejection names the numbered rule and
  location; no acceptance path depends on a timeout, budget, or iteration
  order) — `check/generics.rs::call_repeats_caller_generic_arguments`/
  `targ_names_type_parameter`/`targ_names_const_parameter` (FN-6's cycle
  admissibility is one exact syntactic test, never a size heuristic);
  `mod.rs::SemanticRule::definition_rank` (a closed, exhaustive-by-construction
  rule ordering for [DIAG-1] same-node simultaneity, machine-checked against
  the active specification by a dedicated test named in its own doc comment).
- `design/language/ownership.md` (decision 1: explicit borrow modes at
  mode-bearing positions) — `model.rs::CheckedMode` (`Own`,
  `Shared(DeclarationId)`, `Unique(DeclarationId)`), the one mode value every
  parameter, result, and match binder in the checked tree carries.
- `design/language/ownership/copy-classification.md` (decision 1: primitives,
  shared borrows, shared slice views, and tag-only enums copy; owned
  composites and exclusive views affine; every enum affine regardless of
  payload explicitly rejected) — `model.rs::CheckedNominal::is_copy` (exactly
  "every variant has no fields"); `check/nominals.rs::is_copy_type` (`Bool`
  and primitives copy inline, every `CheckedNominalKind::Struct` affine with
  no exception for an all-copy-field struct, `Slice{Shared}` copy and
  `Slice{Exclusive}` affine, matching the node's own reasoning about the
  shared view costing nothing to duplicate).
- `design/language/ownership/copy-classification.md` (decision 2: a generic
  body's consuming spelling is checked once against its declared bound, never
  re-checked when a concrete argument turns out to be copy) —
  `check/generics.rs::GenericBound` (closed `{Int, Float,
  Class(LinearityClass)}`, "selects no behavior and admits no contract
  member"); cross-checked against `spec/kernel-spec.md:661` ("That spelling
  judgment is made once per written body: at a concrete instance of a generic
  template it is not re-made").
- `design/language/data-model/tag-only-equality.md` (tag-only equality as a
  distinct enum-domain operation family comparing declared-variant identity
  directly) — `model.rs::CheckedExpression::EnumEquality`'s data shape
  (`equal: bool, operand_type: CheckedType, arguments`); the operator-selection
  judgment itself is outside this family (see scope note above).
- `design/compiler/tag-only-lowering.md` — front-end data only (see scope note
  above): `model.rs::CheckedNominal::is_copy`, `CheckedFlatElement::
  TagOnlyNominal`, `CheckedVariant::tag: u32`.
- `design/language/surface-form.md` (decision 4: a body binder's mode and type
  are derived, never written, while signature modes and types stay written) —
  `model.rs::CheckedStatement::Let` carries no mode/type field at all (only
  `node_path, binding, value`; the type is always `value.ty()`), against
  `check/types.rs::parse_parameters_with`/`parse_rtype_with`, which always
  parse a written mode-and-type node for a parameter or result.
- `design/language/surface-form.md` (decision 5: two iteration forms, an
  ordinary loop and an ascending half-open counted loop with a
  compiler-updated, source-immutable binder) —
  `model.rs::CheckedStatement::{Loop, CountedRange}` is the complete,
  exhaustive set of loop-shaped statements; `CountedRange` carries `binder,
  lower, upper` with no source-writable increment.
- `design/language/surface-form/construction-form.md` (every field named in
  declared order, including a single-field variant) —
  `check/types.rs::parse_const_construction` (the [CONST-2]/[GRAM-8] struct
  constant path: label count, label spelling, and label order are each
  checked against the declared field list before any value is read).
- `design/language/surface-form/operation-spelling.md` (decision 2: explicit
  `wrap`/`defined`/`checked`/`sat`/`strict` modes, no bare trapping operator) —
  `model.rs::CheckedIntegerOperation`/`CheckedFloatOperation` and their
  `spelling()` tables (`AddWrap`/`AddExact`/`AddDefined`/`AddChecked`/
  `AddSaturating`, `*Strict` float rows), which the module's own comment says
  are "locked against the specification table by
  `semantic::tests::operation_table`" in both directions.
- `design/language/ownership/slice-result-provenance.md` (a finite,
  closed-world origin set formed once at formation, preserved by every
  operation, and computed for a call's result from the callee's signature
  alone, never its body) — `model.rs::{CheckedSliceOrigin, CheckedStateOrigins,
  CheckedResultStateOrigin}`; `check/expressions/flat_storage/slices.rs::
  check_slice_of` (the one shared [VIEW-2] formation judgment for both
  strengths); `check/types.rs::state_origins_of_value`/`finish_state_origins`
  (a call's contribution is read back from the callee's already-derived
  `CheckedResultStateOrigin::Finite { formals }`, never re-derived from the
  callee's body).
- `design/language/data-model.md` (decision 4: full fixed arrays,
  initialized-prefix/circular-window runs, and placement-versus-identity are
  distinct states and separate axes, not one universal dynamic-array
  representation) — `model.rs::CheckedType::{Array, FixedVector, Vector}` are
  three separate constructors, not one array type with a run-time
  discriminant; `MeasuredKind`/`CheckedMeasure::cell` gives `Array`/`Buffer`/
  `Slice` a constant-zero `Room`/`Head` (no window at all) while the two runs
  get real, runtime `Room`/`Head` cells; `kernel.rs`'s `placement_relations`/
  `removal_relations`/`head_bounded` implement the literal circular-window
  (`head`/`room` modulo `cap`) bookkeeping only for the two runs. Placement
  (frame-resident `FixedVector` versus store-resident `Vector`) is visibly a
  separate axis from the run abstraction itself: both share the same measure
  table and the same [BLK-0] relation shapes, differing only in where their
  storage lives.

## 2. No decision needed

- `tree.rs::TreeView` is a small read-only query facade over the finalized
  syntax topology (`children`/`production`/`first_child_with`/
  `descendants_with`/`path`), precomputing every node's `NodePath` once by a
  parent-pointer walk at construction — ordinary caching, not a policy choice.
- `check/expressions.rs`+`expressions/`, `check.rs`+`check/`, and this
  family's own `expressions/flat_storage.rs`+`flat_storage/` all use the
  file-plus-sibling-directory module convention with no `mod.rs` boilerplate —
  ordinary modern Rust layout.
- Numeric width/signedness tables (`IntegerType`/`FloatType::width`/`signed`,
  `CheckedNumericType::converts_totally_to`/`reinterprets_to`) are small
  closed match tables transcribing [OP-6]'s total-conversion and
  reinterpretation pairs — mechanical case analysis.
- `kernel.rs`'s `KernelShape`/`KernelOperand`/`KernelTerm`/`KernelRelation`/
  `KernelBound` form one small closed "shape language" rather than a general
  expression AST, by the module's own stated reason: it need be "exactly as
  wide as the twelve rows of the inventory."
- `kernel.rs`'s three self-checking tests over its own hand-transcribed table
  (every resolved row has its record; every row is measure-complete on every
  declared exit; no row's declared set is self-contradictory, checked at
  three chosen `advance` values) are ordinary data-consistency test
  engineering, not a policy choice.
- `MeasureCell`/`CheckedMeasure::cell` is explicitly framed by its own code
  comment as "data, not a rule," transcribing [MSR-1]'s own measure table.
- `CheckedElement`'s one-level run-in-run lift is bounded by an explicit
  [BLK-1] citation and by `CheckedType`'s own `Copy` requirement (stated in
  the type's doc comment), not an independently chosen depth limit.
- `nominals.rs::reject_recursive_nominal_layouts` is an ordinary iterative
  three-color-marked DFS over the nominal field graph; excluding a `Box`-typed
  field from the "dependency" edge is the same finite-size-aggregate necessity
  every compiler with value-typed recursive structs has (Rust's own included),
  not a bespoke rule.
- `floats.rs`'s canonical-float search is a small, self-contained,
  provably-bounded brute-force search — the file's own comment proves the
  fixed `CANDIDATE_RADIUS` covers every shortest candidate — rather than an
  external shortest-round-trip crate, and is locked by exact-value tests
  (`f32::MAX`, `MIN_POSITIVE`, the smallest denormal, and their `f64`
  counterparts).
- `types.rs::parse_integer` mechanically implements [FORM-7]'s exact-range,
  no-leading-zero, no-negative-zero literal grammar through a `u128`
  intermediate — ordinary bounds-checked parsing.
- Generic-template bodies are validated once by replaying the ordinary
  concrete-checking path over a checkpointed, later-rolled-back suffix of the
  shared nominal table (`generics.rs`), bridging facts that must outlive the
  rollback through a parallel `Stable*` structural type family — the direct
  cost of running generic validation through this project's one general
  checking path instead of writing a second, bespoke symbolic checker.
- `places.rs::PlaceOffset`/`PlaceStep` admit exactly `{Literal, Binding,
  Const, Opaque}` and decide sameness/distinctness by one small match — the
  minimal realization [OWN-7]'s own text requires ("two written literals ...
  and every other offset is opaque").
- Ordered multi-result function returns are desugared at interning time into
  an ordinary compiler-synthesized struct nominal, one field per result
  ordinal (`nominals.rs::intern_result_list_nominal`), reusing the one normal
  struct/ownership/lowering path instead of adding a dedicated result-list
  shape to the checked IR — the code's own comment gives the reason ("nothing
  else in the language changes"), and it is the obvious way to honor one
  general implementation path.
- The half-dozen compiler-owned synthetic-nominal interning functions (box,
  store-box, arena, the arena allocation-list, five prelude types, system
  nominals) all follow one repeated idiom — check the memo table, else push
  and register once — rather than being five independent designs.
- `CheckedStateOrigins`'s small algebra (`union`/`projected`/`enum_payload`/
  `replace_path`) is ordinary finite-set-with-path-prefix bookkeeping
  implementing the closed-world origin tracking `slice-result-provenance.md`
  already decided, with no separate policy of its own.
- `check/expressions/flat_storage/borrowed.rs`'s one function handles "an
  indexable place reached through a borrow holder" by an exhaustive match on
  the dereferenced type (buffer/slice/run/extent), falling through to
  `UnsupportedSemanticFeature::RegionsAndBorrows` for anything else — an
  ordinary exhaustive dispatch, not a policy choice.

## 3. Choices without a node

**1. A compile-time switch kept alive after its candidate spec fully merged
holds two dead code paths open, in direct tension with a rejection the tree
already records for exactly this shape.**
`V031_CANDIDATE_SEMANTICS: bool = true` (`mod.rs:65`) gates struct-typed named
consts (the [CONST-2] candidate) and a contract-conditional [OWN-1] repair
wording, at five call sites: `mod.rs:65` (definition), `check/types.rs:1704,
1815, 1922` (in this family), plus `check/requires.rs:273` and
`check.rs:1606` (outside it). It was introduced `false` in `1fc2aebe`
(2026-08-17, "semantic: gated v0.31 candidate surface — struct consts and
clause repair") and flipped to `true` the same day in `fdacd71f` ("compiler:
flip the three v0.31 switches and activate O11 decomposition"). The active
specification is now v0.53 — 22 versions past v0.31 — and both passages it
once conditionally gated are unconditional, non-candidate text there:
[CONST-2]'s struct-typed const rule (`spec/kernel-spec.md:641-651`) and
[OWN-1]'s position-conditional bare-affine repair (`spec/kernel-spec.md:662`).
No test in `compiler/src/semantic/tests/` exercises the `false` branch.
Alternative: delete the switch and its five `!V031_CANDIDATE_SEMANTICS` arms
once the candidate became the sole authority, the way a two-commit feature
landing ordinarily finishes.
Where: `compiler/src/semantic/mod.rs:65`; `compiler/src/semantic/check/
types.rs:1704, 1815, 1922`.
Reason (quoted, `fdacd71f`, 2026-08-17): "The candidate at spec/kernel-spec.md
is now the branch's own authority, so the compiler implements it rather than
v0.30. Switches: ... V031_CANDIDATE_SEMANTICS (semantic/mod.rs) all become
`true` ... Each workstream had already implemented and tested its complete
judgment behind its switch; no other line of those judgments changes." Neither
this commit nor any later one explains why the dead `false` arms were kept
rather than deleted in the same or a following change.
Effect: none on acceptance today (the switch is a compile-time `true`); the
five sites and their unreachable `false` arms are maintenance weight with no
consumer and no test.
Reads as an unfinished cleanup rather than a deliberate choice, and it is
drift against `design/compiler.md`'s own already-recorded rejection:
"Superseded system-inventory states kept reachable behind compile-time
switches so that a differential test can show an earlier program's module
unchanged: rejected because the compiler implements exactly one specification,
the active one, so a switch that reconstructs a superseded inventory has no
consumer and only adds code paths to maintain." That sentence names this
switch's shape exactly, even though it was written about a different
subsystem (resolution's [SYS-2] inventory switches).
Decision: Remove `V031_CANDIDATE_SEMANTICS` and its five `false`-branch call
sites, because the v0.31 candidate it gated has been the sole active
specification since 2026-08-17 (now v0.53, 22 versions later) and
`design/compiler.md` already rejects a switch retained past its own
supersession for the identical reason, instead of leaving a permanently-`true`
switch threaded through five files across two module families.

**2. A nominal's region parameters are a real, undocumented axis of the data
model: two instances differing only by region are two checked types
reconciled to one lowered representation by a second, post-hoc pass.**
`GenericSubstitution.regions` (`check/generics.rs:69-79`) gives a source
nominal's `region_params` [S20] their own axis beside its type and const
arguments, so `Vector<'a, T>` and `Vector<'b, T>` — or any nominal built over
either — are minted as two distinct `NominalId`s and two distinct
`CheckedType`s. A second, separate pass then decides which earlier instance
each one actually lowers as: `check/nominal_instances.rs::
nominal_lowering_aliases`/`nominals_differ_only_in_region` (an extensively
commented coinductive walk that compares two instances' content "with every
region erased and every region-derived datum kept"), consumed into
`CheckedProgramData.nominal_lowering_alias` (`model.rs:2727`, "the first such
instance is the one they all lower as").
Alternative: never mint a second `NominalId` for two instances whose type and
const arguments already agree and whose regions differ — treat the region
argument as proof-only metadata at the point a nominal instance is looked up,
so no later reconciliation pass is needed, at the cost of monomorphizing every
region-parameterized nominal (and the functions that use it) once per region
actually reached.
Where: `compiler/src/semantic/check/generics.rs:69-79` (`GenericSubstitution.
regions`), `1821-1826` (the symbolic region-identity instance); `compiler/src/
semantic/check/nominal_instances.rs:1029-1038` (`nominal_lowering_aliases`'s
doc, "[S20] where each nominal instance's region axis leaves the program"),
`1071-1097` (`substitute_type_regions`), `1325-1420`
(`nominals_differ_only_in_region`/`nominals_are_region_blind_equal`);
`compiler/src/semantic/model.rs:2718-2727` (`nominal_lowering_alias`'s field
doc).
Reason (quoted, commit `552f6e52`, 2026-09-05, "B8a: a nominal instance is
keyed on its region, and the region is erased at lowering"): "A region names a
store for the proof and nothing at run time ... so two instances of one
declaration that differ only in their region arguments are two checked types
and **one IR nominal**. That is what lets a callee's own formal-region
instance and a caller's actual-region instance meet at the boundary between
them without monomorphizing every function over its regions, and it is
checked rather than assumed[.]" The same commit body separately states the
research ground it built on: "[S20] gave a nominal `region_params` in B7a and
nothing ever instantiated them." No record weighs "two checked types, one
representation, reconciled after checking" against "one checked type from the
start" as the alternative this audit names; `research/investigations/
containers-and-resources/DESIGN.md`'s own S20 entry ("ADOPTED") states only
that a store's identity belongs in the type, not why the checked-type/
lowered-representation split should be two-then-one rather than one throughout.
Effect: architecture and compiler-internal bookkeeping, not acceptance — a
caller sees ordinary [TYPE-5] exact-identity checking either way, and the
alias table is read only by a lowering stage outside this family.
Reads as deliberate and carefully executed (the region-blind-equality walk is
extensively commented and tested against release-class and content
differences), but the decision — and the alternative it implicitly
refused — has never reached `design/language/data-model.md` or any other live
tree node; the only record is a research investigation's numbered item, which
`CLAUDE.md`'s own authority rules do not treat as current guidance ("A
historical plan or research proposal is not an implied requirement").
Decision: A nominal's region parameters are their own axis of type identity at
check time, with two instances of one declaration differing only by region
reconciled to one representation by a region-erased structural-equality pass
before lowering, because a region is a proof-time identity with no run-time
representation and this lets a callee's formal-region instance and a caller's
actual-region instance meet as one representation without monomorphizing every
region-parameterized function and nominal over each region it is reached at,
instead of erasing the region at instance lookup so only one `NominalId` is
ever minted per representation.

**3. Two independent representations of "a resolved storage place" coexist,
split by pipeline stage, with no tree record that either exists relative to
the other.**
`semantic/places.rs::{PlaceRoot, ResolvedPlace}` (root:
`Binding(BindingId) | Constant(CheckedConstantId)`) is used only by the two
whole-function, post-hoc passes named in its own module doc: [ENT-5] kill
projection and the sibling-call permission judgment ("Neither may grow a
private copy of the overlap relation"). Every consumer actually in this same
family that runs *during* live, flow-sensitive expression checking instead
imports a second, separately defined `check/borrows.rs::ResolvedPlace` (root:
a bare `DeclarationId`, no `Constant` case) —
`check/expressions/places.rs:11` and `check/expressions/flat_storage.rs:21-24`
both do this, not `semantic/places.rs`'s own type. The two share only the
lower-level `PlaceStep`/`PlaceOffset`/`paths_diverge` primitives
(`check/borrows.rs:14` imports exactly those three names from
`semantic/places.rs`); the wrapping struct, its root representation, and its
holder-chain resolution walk are each implemented twice
(`semantic/places.rs::PlaceMap::resolve`/`resolve_deref` versus
`check/borrows.rs`'s own resolution reachable through
`resolve_dereference_holder`, which `check/expressions/places.rs` and
`check/expressions/flat_storage/borrowed.rs` call).
Alternative: one `ResolvedPlace` type (and one root representation) used by
both the live per-statement checker and the later whole-function passes,
translating `DeclarationId` to `BindingId` at whichever boundary needs it, so
"a resolved place" is one type in the data model rather than two.
Where: `compiler/src/semantic/places.rs:26-30, 150-190` (`PlaceRoot`,
`ResolvedPlace`, `overlaps`/`is_prefix_of`); `compiler/src/semantic/check/
borrows.rs:128-158` (its own `ResolvedPlace`, `BorrowInfo`); consuming imports
at `compiler/src/semantic/check/expressions/places.rs:11` and
`compiler/src/semantic/check/expressions/flat_storage.rs:21-24`.
Reason: no reason recorded for the non-unification specifically.
`semantic/places.rs` was added later (`623660da`, 2026-08-21, "compiler: judge
which sibling call pairs may be overlapped") with a stated scope limited to
exactly its two consumers ("Two consumers share this module ... Neither may
grow a private copy of the overlap relation"); that commit's body explains why
the *permission judgment and ENT-5* should share one implementation, but
neither it nor any later commit discusses why the pre-existing
`check/borrows.rs` representation, used for the bulk of ordinary [OWN-1]/
[OWN-5]/[OWN-6]/[OWN-10] checking, was left as a separate type rather than
being unified with or migrated to the new one.
Effect: structure only — nothing in this audit found a case where the two
walks would resolve one written place two different ways, but no test or
type boundary forces them to keep agreeing as either one's holder-chain logic
changes independently of the other.
Reads as a defensible split rather than an oversight (the live checker
resolves against a `bindings: HashMap<DeclarationId, LocalBinding>` that is
still being built statement by statement; the later passes see one already
complete, already `BindingId`-keyed function body and a denser identity space
suits them better), but it is exactly the kind of unrecorded structural fact —
that "resolved place" is two types, not one — that a reader of either
module's own doc comment would not learn from that comment alone.
Decision: Keep two `ResolvedPlace` representations, a `DeclarationId`-rooted
one for live per-statement ownership/borrow checking and a `BindingId`-rooted
one for whole-function post-hoc passes (kills and the permission judgment),
sharing only the place-step and offset primitives, because [owner to state the
reason — the live checker's available context is a growing declaration-keyed
binding map while the post-hoc passes see one complete, densely `BindingId`-
keyed function], instead of one `ResolvedPlace` translated at the boundary
between the two pipeline stages.

**4. Three separately chosen, undocumented recursion-depth ceilings guard
structural walks over checked places and nominals, with no shared constant and
no stated rationale for any of the three numbers.**
`places.rs::resolve_deref`/`resolve_deref_with_holders` cut a borrow-holder
chain walk off at `depth > 32`, returning the un-followed binding itself as a
conservative anchor. `nominal_instances.rs::nominals_are_region_blind_equal`
cuts its top-level coinductive nominal-pair walk off at `depth > 64` — after
which the cycle-breaking `assumed`-pair memo (added specifically because, per
its own comment, "the depth cap used to cut off" answered wrongly for a
genuine `Box`-mediated cycle) already makes the depth check mostly a backstop
rather than the primary termination argument. `nominal_instances.rs::
nominal_content_is_region_blind_equal`, a nested per-field walk inside that
same comparison, cuts off at a third, different number, `depth > 16`. None of
the three constants is named, shared, or explained at its site.
Alternative: one named, documented constant reused by every bounded
structural walk over a checked type or place in this family, with a comment
stating what real program shape could reach it and why that number rather
than another — or, following this compiler's own convention elsewhere for a
ceiling that could matter (e.g., the lexer's and parser's explicit,
caller-supplied `*Limits` structs), a counted budget that raises a
`SemanticCompilerFailure` past its limit instead of silently returning a
conservative answer.
Where: `compiler/src/semantic/places.rs:466, 495`;
`compiler/src/semantic/check/nominal_instances.rs:1373, 1517`.
Reason: no reason recorded at any of the three sites. `git log -S "depth > 32"`
(`places.rs`), `-S "depth > 64"` and `-S "depth > 16"`
(`nominal_instances.rs`) each show the guard introduced together with its
owning function and never revisited.
Effect: none currently observable on acceptance — recursive nominal layouts
are already excluded upstream except through `Box` (see §2 above and
`reject_recursive_nominal_layouts` in §1/§2), and a borrow-holder chain is
bounded by the small number of admitted reborrow forms
(`design/language/ownership/no-reborrow.md`), so no known accepted program is
believed to reach any of the three. That belief rests on the three numbers
themselves rather than on a proof any of them documents, and design/compiler.md
rule 4 already states why an acceptance-relevant ceiling should never be
silent or ad hoc: "no acceptance path depends on ... a work budget."
Reads as three independent ad hoc defensive backstops added at the moment
each function was written, not a considered ceiling — the mismatch between 16,
32, and 64 for structurally similar walks is itself evidence no single
reasoning process produced all three, though none is known to be a live
defect.
Decision: Replace the three unrelated depth ceilings in `places.rs` and
`nominal_instances.rs` with one named, documented constant (or with a counted
budget that fails closed to a `SemanticCompilerFailure` rather than a silent
conservative answer), because three different, unexplained numbers guarding
structurally similar bounded walks is accidental rather than chosen, instead
of leaving each site to invent and justify its own limit independently.

**5. `design/language/data-model.md`'s own stated reason for keeping
array-of-structs describes a "copy-struct tier" that exists in neither the
active specification nor the compiler — a tree-currency defect, not a code
defect.**
`data-model.md`'s decision 1 reasons that "array-of-structs stays a genuine
need with a named path through a copy-struct tier, because a struct of copy
fields can be declared copy and stored in a buffer without importing the
affine-element problems." In the active specification and in this family's
own code, every struct is unconditionally affine, with no declared-copy
mechanism of any kind: [OWN-1]'s copy list (`spec/kernel-spec.md:657`) names
only "primitives ... shared borrows, `Slice<'r, T>` ... and tag-only enums,"
and states plainly that "all other values (owned composites, ...) are
affine" — no exception for a struct whose fields are themselves all copy.
`model.rs::CheckedNominal::is_copy` (lines 844-852) and
`check/nominals.rs::is_copy_type` (lines 106-135) have no arm that ever
returns `true` for a `CheckedNominalKind::Struct`, regardless of its fields'
types. `check/types.rs::buffer_element` does admit a "region-free affine
nominal stored by value" as a buffer element — i.e., array-of-structs is
already reachable today, but only through the ordinary *affine* path the node
calls "the affine-element problems," never through a "declared copy" one.
This is not a code defect: the code correctly implements [OWN-1] exactly as
the active specification states it, which has no copy-struct declaration at
all. The defect is the tree node's own premise, which was true only as a
2026-07-10 proposal: `mcts_mem/whitefoot/data-model.md`'s 2026-07-10 entry
records the tier as "carded as smaller than the blocked affine-element
cluster" — i.e., explicitly deferred, not adopted — and the introducing commit
(`fa3a4093`, 2026-07-10, "gates: totality economics (owner flag), copy-struct
tier for AoS, port-study evidence leg") is a bodiless gate-tracking commit with
no spec or compiler change attached. No later record shows the tier shipping
in the many spec versions since (the active specification is now v0.53).
Where (evidence that the described capability does not exist, not a defect's
location): `spec/kernel-spec.md:657`; `compiler/src/semantic/model.rs:844-852`;
`compiler/src/semantic/check/nominals.rs:106-135`;
`compiler/src/semantic/check/types.rs::buffer_element` (the [TYPE-2] buffer
element domain).
Effect: a reader of the live tree is told a capability is currently available
("a struct of copy fields can be declared copy") that no writer can actually
reach; this is worth the owner's correction independent of any code change.
Decision: [for the owner to choose, not this audit] either retire the
"copy-struct tier" clause from `data-model.md`'s decision 1 and state
array-of-structs as an accepted, currently-unserved gap instead of one with "a
named path," or, if the tier is still wanted, promote it from the frozen
`mcts_mem` record into an actual specification and compiler task before the
node cites it as already available.

## Summary

- Covered by the tree: 15
- No decision needed: 16
- Choices without a node: 5 (3 architecture choices, 1 code-hygiene finding,
  1 tree-currency defect)
