# Audit: resolution against the design tree

Module: `compiler/src/resolution/` — `mod.rs`, `kernel.rs`, `scopes.rs`,
`catalog.rs`, `engine.rs` and `engine/{admission,inventory,lookup,roles}.rs`,
plus `tests.rs` (about 10,700 lines total). Name resolution, declaration
inventory, scope construction, and the resolution engine. Direction: code to
tree — finding choices the code embodies that `design/` does not record, not
the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/name-resolution.md`,
`design/language/system-interface.md` and
`design/language/system-interface/declaration-home.md`. Also read for
grounding, since this module's own comments name them constantly:
`design/language/ownership.md`, `design/language/effects.md`,
`design/language/contracts.md`,
`design/language/checks-and-proofs/requires-entry-contract.md`,
`design/language/surface-form/operation-spelling.md`, and the relevant
sections of `spec/kernel-spec.md` (name/scope visibility, [DIAG-1]'s ranks and
canonical event key, [SYS-1]-[SYS-3], [BLK-0], [FORM-3], [OWN-3], [LIV-2],
[EFF-1], [FN-3]/[FN-4]/[FN-8]/[FN-9]).

Git history note, relevant to "no reason recorded" below: the audit first ran on a shallow clone whose history stopped at `65b3d24` (2026-08-28); the repository has since been unshallowed to its real root `7c1d7641` (2026-07-07), and every reason field below was re-checked against the full history with `git log -S`, `git blame`, and the introducing commits' bodies. Where a commit speaks to the reason it is quoted with its hash and date; "no reason recorded" means the introducing commit and its successors say nothing.

## 1. Covered by the tree

- `design/compiler.md` (a rejection names the numbered rule it violates and
  the location; no acceptance path depends on hash-iteration order) —
  `compiler/src/resolution/mod.rs::{ResolutionRule, ResolutionIssue}` (every
  issue carries a rule and an origin); `compiler/src/resolution/engine/lookup.rs::resolve_uses_deferred`
  and `compiler/src/resolution/engine/inventory.rs::sort_conflicts` (every
  `HashSet`/`HashMap`-derived collection is sorted by the canonical event key
  before it reaches a diagnostic or a decision).
- `design/compiler.md` (an unimplemented capability stops as an explicit gap,
  never a rejection) — `compiler/src/resolution/engine.rs`'s handling of
  `RawRoleKind::TableChecked` (the `program_kind`/`input_label` IDENTs: "its
  FN-7 kind-table judgment is an unimplemented compiler capability, so
  classification produces no retained record yet").
- `design/compiler.md` (a compiler-internal failure is never a
  source-language rejection) — `compiler/src/resolution/mod.rs::{ResolutionCompilerFailure, ResolutionOutcome}`,
  which keeps `SourceIssue` and `CompilerFailure` as distinct outcome
  variants throughout the module.
- `design/compiler.md` (one safe-Rust crate, simple implementations over
  ordinary collections) — the whole module: only `Vec`/`HashMap`/`HashSet`,
  no `unsafe`.
- `design/language/name-resolution.md` (decision 1: every top-level function
  signature is visible throughout the unit; every other declaration keeps its
  lexical visibility point) — `compiler/src/resolution/engine.rs::declaration_visibility`
  (`Visibility::Always` for `DeclarationRole::Function`, `Visibility::After`
  computed per role otherwise) and `is_visible`.
- `design/language/name-resolution.md` (decision 2: resolution starts from
  one complete declaration inventory, then resolves each use role in its own
  domain and scope; inventory knowledge never grants visibility outside the
  rule for that class) — `compiler/src/resolution/engine.rs::build_tables`
  (build the complete inventory first, only then check it and resolve uses)
  and `compiler/src/resolution/engine/lookup.rs::{admissible_classes, universe_classes, resolve_uses_deferred}`
  (per-role admissible/universe classes plus `is_visible`, never blanket
  inventory-wide visibility).
- `design/language/system-interface/declaration-home.md` (system types and
  operations resolve from a distinct compiler-owned domain, visible in every
  unit, neither the prelude nor the foreign-boundary family) —
  `compiler/src/resolution/catalog.rs::system_declarations` (the [SYS-2]
  table, built and looked up independently of `PRELUDE_DECLARATIONS`);
  `compiler/src/resolution/engine/lookup.rs::resolve_uses_deferred` (the
  system-entry contribution block, "the third admitted declaration source
  [SYS-1]") and `compiler/src/resolution/engine/inventory.rs::collision_issue`
  (the system-collision block, ranked ahead of ordinary redeclaration).
- `design/language/ownership.md` (decision 1: lexical named or unnamed
  regions) — `compiler/src/resolution/scopes.rs::ScopeBuild::build` (the
  `RegionStmt`/`LoopStmt`/`ForStmt`/`Mode`-ampersand cases mint a scope per
  region) and `compiler/src/resolution/engine.rs::region_scope_owner`.
- `design/language/checks-and-proofs/requires-entry-contract.md` (decision 1:
  a present contract block holds at least one clause) —
  `compiler/src/resolution/engine/admission.rs::check_clause_blocks`.
- `design/language/checks-and-proofs/requires-entry-contract.md` (decision 3:
  every function names its result so `ensures` can refer to it; a Result
  success binds `Ok(value: name)`) — `compiler/src/resolution/mod.rs::PostconditionResolutionRecord`
  and `compiler/src/resolution/engine.rs::{build_postcondition_records, build_postcondition_candidate}`.
- `design/language/surface-form/operation-spelling.md` (an operation's
  spelling is fixed by its grammar class and never by its use site) —
  `compiler/src/resolution/catalog.rs::{OPERATION_FAMILIES, reserved_name, operation_id}`
  (a bare spelling is looked up and reserved independently of any call-site
  context).

## 2. No decision needed

- Scope tree built once by an explicit iterative worklist over the canonical
  syntax tree (no recursion); each node's scope follows from its production
  kind alone, per the specification's scope-construction matrix.
- Declarations sit in a flat `Vec` plus a `HashMap<String, Vec<usize>>` name
  index (`DeclarationIndex`); membership within one spelling is always
  filtered by scope/visibility at query time, never precomputed per scope.
- Every dense identity (`ScopeId`, `DeclarationId`, `SystemDeclarationId`,
  `OperationFamilyId`) is a checked `u32`/`u16` newtype; construction goes
  through `checked_add`/`try_from` and fails closed to
  `ResolutionCompilerFailure::CounterOverflow` rather than wrapping.
- Strict two-phase pipeline: build the complete scope tree and role
  classification once, then run declaration-inventory checking and
  lexical-use lookup as separate passes over the same tables, matching the
  specification's stated stage order (FN-8 admission, then inventory, then
  lexical resolution).
- The prelude (PRE-1), system (SYS-2/SYS-3) and the two compiler-owned
  domains (TYPE-2 container/provider nominals, BLK-0 kernel operations) are
  each one `const` table transcribed in normative table order and reached by
  direct indexing or linear scan, never by name-based special-casing; several
  are round-tripped against the specification's own rendered text in tests
  (`catalog.rs`'s `render_operation`/`render_type`).
- Reserved-name (FORM-3), region-uniqueness (OWN-3), match-binder-freshness
  (GRAM-10), contract-shape (FN-8) and postcondition-selector (FN-9) checks
  are each one small function named for its rule, called once from the
  shared per-role inventory loop.
- Elided/unnamed regions (a bare `loop`/`for`/`region { }`, an elided `&`)
  are minted as ordinary declarations at their introducing token under a
  `'0_<source>_<offset>` spelling no source token can produce, so nothing
  downstream needs a separate "anonymous declaration" case.
- A `set` statement's undeclared bare target is recognized by grammar shape
  alone (`Pbase`/`Place`/`SetStmt`, no suffix or deref) before lookup ever
  runs, matching [LIV-2]'s "declares like a `let`" rule.
- Diagnostics carry a numbered `ResolutionRule` plus a structured
  `ResolutionIssueKind`; the small amount of finished wording present (the
  four collision "mechanical fix" sentences) is a plain `&'static str`
  constant, not a template engine.
- A routed `ensures` clause resolves its selector before its own ordinary
  lexical uses are resolved at all, matching the specification's stated stage
  order for that construct.
- System operations carry backend-facing execution metadata (inline vs.
  may-suspend dispatch, completion milestones) in the same table as their
  signature, reflecting what each host call can actually do rather than an
  independent compiler judgment.
- A nominal declaration is its own region-uniqueness scope, exactly like a
  function (`region_scope_owner`) — this is [OWN-3]'s own explicit sentence
  ("A nominal's `region_params` ... are its own in the same sense"), not a
  resolver-invented extension of it.
- Internal `Dnn`/`Unn`/`Xnn` comment tags label each declaration/use/deferred
  role for cross-referencing in comments and tests; a compiler documentation
  convention distinct from the specification's own rule numbers (FORM-3,
  TYPE-6, and so on), which do not use this numbering.

## 3. Choices without a node

**1. Historical [SYS-2] inventory states are kept reachable behind
compile-time switches for differential testing.**
The complete superseded-and-current system-declaration surface lives in one
set of tables; three `bool` consts (`TRAVERSAL_SURFACE`, `OPEN_BY_NAME`,
`STREAMS_AND_TCP`) and an `Inventory` enum select a strictly nested
table-length prefix ("the three states are strictly nested prefixes... so a
state is a length rather than a set of independent features"), and
`Inventory::ACTIVE`, a compile-time const, fixes what the shipped compiler
resolves against.
Alternative: delete a superseded surface once its successor ships and rely on
git history / the archived `spec/kernel-spec-vN.md` files for regression
evidence; or keep independent, unshared tables per historical version; or make
each surface an independently selectable flag instead of a strictly nested
prefix.
Where: `compiler/src/resolution/catalog.rs` — `Inventory`, `Inventory::ACTIVE`,
the three switches, `system_nominals`/`system_constructors`/`system_operations`,
and the ordinal-arithmetic helpers (`system_nominal_index`,
`system_constructor_index`, `system_operation_index`, `system_entity`) that
all thread an `Inventory` parameter.
Reason (quoted, commit `22534349`, 2026-08-18, "compiler: admit open_file
behind the OPEN_BY_NAME inventory switch," which introduced the `Inventory`
enum itself by generalizing the single `traversal_surface` bool that
`bfb8ce89` had added the same day): "The two inventory switches are now
carried as one `Inventory` value ... threaded where the single
`traversal_surface` bool was threaded before. The states are strictly nested
prefixes of the [SYS-2] tables, so `OPEN_BY_NAME = false` leaves every
declaration ordinal and the whole 192-record active inventory unchanged." The
current code comment ("That is what lets one differential test show that
switching a candidate off leaves every earlier program's emitted module
byte-identical") restates this. Neither commit's body says why a superseded
state is kept reachable at all rather than deleted once its successor ships.
Effect: no acceptance difference in the shipped compiler (`ACTIVE` is fixed at
compile time, consistent with `design/language.md`'s "one observable
behavior" rule — this is not a build-mode flag). It is a real structural cost
instead: six-plus functions carry an `Inventory` parameter and prefix
arithmetic, and no compiler-tree node decides that this is how spec-version
regression testing should be built.

**2. `set`-target-declares is resolved by rebuilding the whole pass to a
fixpoint, not by a targeted check.**
Every shape-eligible bare `set` target is collected once
(`declaring_set_target_candidates`); the complete declaration-inventory-and-
lookup pass then reruns from scratch, promoting exactly the one candidate
that pass's own `UnresolvedUse` failure names to an ordinary `let`-like
declaration, until a pass leaves none unresolved.
Alternative: a single dedicated pre-pass that, for each shape-eligible
candidate, walks enclosing scopes directly to decide whether a prior binding
exists, without re-running the general resolver.
Where: `compiler/src/resolution/engine.rs::build_tables` (the `loop { ... }`),
`declaring_set_target_candidates`, `promotable_candidate`.
Reason (quoted, commit `5a9250b9`, 2026-09-05, "B7a5: the bound, the declaring
set target, and a run's release class"): "B7a4 measured that this is not
checker work: the target identifier is an unresolved use before the checker
runs. The promotion is one pass of the resolver's own lookup ... repeated
until every use resolves, so a declared target is judged by exactly the rules
a `let` binder is." The current code comment repeats this near-verbatim, and
the per-pass promotion order is separately pinned to source order "and never
a search," matching the compiler's general determinism rule. The commit body
gives the reason for putting this in the resolver at all (an earlier batch,
B7a4, measured it as resolver work rather than checker work) but does not
weigh the rebuild-to-fixpoint shape against a single targeted pre-pass.
Effect: performance and structure, not acceptance — the same programs are
accepted either way, but the worst case reruns the complete inventory-and-
lookup pass once per distinct declaring `set` target in one unit, in exchange
for adding no bespoke scope-walking code.

**3. Diagnostic wording lives inside the resolution payload itself, not a
separate renderer.**
`ResolutionIssueKind::DeclarationCollision` carries a finished English
sentence (`mechanical_fix: &'static str`, one of five constants
`COLLIDES_WITH_PRELUDE`/`COLLIDES_WITH_SYSTEM`/`COLLIDES_WITH_CONTAINER`/
`COLLIDES_IN_ONE_SCOPE`/`COLLIDES_WITH_LIVE_OUTER`), and every
`ResolutionIssueKind` currently reaches the compiler's caller as the derived
`Debug` of that structured value with only a location and source line wrapped
around it.
Alternative: keep the payload purely structured (e.g. a `CollisionRepair`
enum) and defer all wording to a dedicated presentation layer.
Where: `compiler/src/resolution/mod.rs::ResolutionIssueKind::DeclarationCollision`
(the `mechanical_fix` field); `compiler/src/resolution/engine/inventory.rs`
(the five constants and the `collision` helper); consumed as-is by
`compiler/src/driver/rejection.rs::Located`.
Reason: three commits, all 2026-08-28. The five collision sentences were
written in `dfb0a30d` ("diagnostics: teach the four remaining bad defaults the
verification writer met" — four situations at the time; `COLLIDES_WITH_LIVE_OUTER`
is the one that motivated the item). Its batch record, added the same day in
`fd65823f` ("docs: record batch 0100"), states the reason directly in its
"Judgment calls" section: "The four situations the rule selects between admit
genuinely different repairs — rename, rename, delete-or-rename, and
rename-or-close-the-block — and a single sentence would have to hedge across
all four." Separately, `a91b9825` ("diagnostics: print the expected spellings
and the offending line") added `driver/rejection.rs`'s `Located` wrapper; its
own comment (quoted) gives the reason for using `Debug` rather than a
renderer: "Nothing here is a rendering redesign. The detail text is still one
stage value's `Debug`; this only wraps that value with the location and the
line, so a reader gets a sentence instead of an offset." None of the three
commit bodies adds beyond what its code or record comment already says.
Effect: diagnostics only. Every future wording change for a resolution
rejection is currently a resolution-module code change, since nothing
downstream reinterprets the payload; no design/compiler node decides that
diagnostic prose belongs in the data model rather than in a renderer.

**4. A contract's `define` binder shares the `let` reserved-name carrier
role, rather than the specification's own distinct one — reads as an
oversight, not a deliberate choice.**
`Production::ContractDefine` classifies as `DeclarationRole::Let`
(`engine/roles.rs`), so a FORM-3 reserved-name violation on a `define` binder
is reported with `ReservedDeclarationRole::Let`; the enum has no
`ContractDefinition` variant at all.
Alternative: a distinct `ReservedDeclarationRole::ContractDefinition`.
Where: `compiler/src/resolution/mod.rs::{DeclarationRole, ReservedDeclarationRole}`;
`compiler/src/resolution/engine/roles.rs` (the `Production::ContractDefine`
arm); `compiler/src/resolution/engine/inventory.rs::reserved_role`.
Reason: no reason recorded, and the full history does not change this. The
`Production::ContractDefine` grammar production and its resolver
classification as `DeclarationRole::Let` were introduced together in one
commit, `b8ccecbb` (2026-08-19, "compiler: implement claim-only static
contracts"), which has no body beyond its title; `ReservedDeclarationRole`
itself predates it by almost a month (`72f4ac18`, 2026-07-22, the original
name resolver, also bodiless on this point). No later commit revisits the
mapping, and no test in `tests.rs` exercises a reserved-name violation on a
`define` binder. Not deliberate as far as the record shows: nothing indicates
the missing `ContractDefinition` role was ever considered and rejected, only
that `contract_define` was wired to the nearest existing declaration role
when it was first added.
Effect: diagnostics only. `spec/kernel-spec.md` (line 2307) states FORM-3's
closed carrier-role list as fourteen roles, naming "contract-definition"
separately from "let"; the code's `ReservedDeclarationRole` has thirteen
variants and folds the two together. Acceptance is unaffected — the reserved
name is still rejected — but the reported carrier role would be wrong for
this one case. Worth the owner's attention as a defect (`docs/todo.md`-shaped)
independently of any tree decision.

**5. `fn_bind` right-IDENT lookup failure cites FN-4 instead of the
specification's FN-3 — reads as a defect, not a deliberate choice.**
`use_rule` maps `LexicalUseRole::FunctionBinding` (the bound-function IDENT of
a conformance's `fn_bind`) to `ResolutionRule::Fn4`.
Alternative: `ResolutionRule::Fn3`. FN-3's own text is what states this exact
lookup ("its right IDENT resolves under [TYPE-6] to one top-level source
function," `spec/kernel-spec.md` line 1660), and the specification's own
closed "rule cited by rank 1 or rank 3" table assigns "`fn_bind` right IDENT"
to FN-3 (line 2361); FN-4 is a later, unrelated judgment about arithmetic-law
discharge.
Where: `compiler/src/resolution/engine/lookup.rs::use_rule`.
Reason: no reason recorded, and the full history strengthens rather than
weakens that. `LexicalUseRole::FunctionBinding => Fn4` is not a recent slip:
it is present, unchanged in substance, in the very first commit that
implemented the resolver at all, `72f4ac18` (2026-07-22, "Implement the exact
v0.10 name resolver," itself bodiless), and every one of the version-
activation commits between v0.10 and v0.16 that subsequently touched this
mapping (`d5c95b72`, `c6e41137`, `c8f3a2b0`, `67f0227b`, `e1bfdffd`,
`ecb34335`) only renamed the versioned `ResolutionRuleV0_N` wrapper type to
match, never the `Fn3`/`Fn4` choice itself; none of their bodies mentions it.
`spec/kernel-spec-v0.10.md` already gave FN-3 the conformance/binding
declaration and FN-4 only the law discharge, the same split as today, so this
is not a case of the two rules' scope drifting apart over time — the citation
reads wrong from the first commit onward. The module's own test
`a_system_operation_never_satisfies_a_conformance_binding`
(`compiler/src/resolution/tests.rs:995`) states in a comment "a system
operation is not the right IDENT of an FN-3 `fn_bind`," matching the
specification, but the test never asserts `issue.rule()`, so nothing catches
the mismatch with the code that runs. Not deliberate as far as the record
shows: no commit, across more than forty version bumps, ever discusses this
specific rule choice.
Effect: diagnostics only — a rejected conformance binding currently names the
wrong specification rule to the writer. Acceptance is unaffected: the binding
is still correctly rejected as unresolved or invisible either way.

## Summary

- Covered by the tree: 11
- No decision needed: 13
- Choices without a node: 5 (3 architecture choices, 2 likely defects)
