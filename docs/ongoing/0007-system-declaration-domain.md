# 0007 — System-declaration domain and opaque types

Live coordination record. It reports how authorized work is being carried
out; it is not authority and cannot expand or resequence work.

- **Status:** IN PROGRESS
- **Owner / workspace:** executor agent / isolated worktree
  (`worktree-agent-af3d05349151067ca`), lead-reviewed
- **Base revision:** `615bbae`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, first bullet
  ("system-declaration domain, opaque types"). Implements dossier §11.1's
  Route C selection as codified in `spec/kernel-spec-v0.18.md` §16:
  `SYS-1` (the declaration-domain rule), `SYS-2` (the system inventory —
  fourteen nominal types, thirty-nine constructors, eleven operation
  signatures, one hundred sixty-seven records total), `SYS-3` (the syntactic
  kind-declaring admission predicate), the concrete opaque-type contracts
  `HOST-1`/`HOST-2`/`HOST-3` and `PATH-1`/`PATH-2`, and `GATE-2` (system
  domain is not the gated FFI family). Claimed under Work item 3's executor
  fan-out while `docs/current-plan.md` remains `ACTIVE`.

## Goal

Register the compiler-owned Route C declaration domain: the seven fixed
opaque types (`Args`, `HostString`, `DirectoryRead`, `RelativePath`,
`ReadFile`, `Output`, `ExitStatus`), the outcome/error nominal inventory
(`ArgError`, `Utf8Error`, `CopyError`, `Utf8CopyError`, `PathError`,
`ReadOutcome`, the 30-class `IoError`, and `ExitStatus`'s total
`exit_status` constructor), and the eleven operation names — admitted only
when task 0006's predicate marks a unit kind-declaring, and resolved as
ordinary undeclared/user names otherwise. `SYS-2` gives the exact signature
of every operation, including region parameters and each one's fixed
effect-category classification; use it as the concrete catalog content.

## Direction and invariants

- Names are visible only when task 0006's syntactic predicate is true for
  the unit. In a non-kind-declaring unit every system spelling is an
  ordinary undeclared name, and a user top-level function reusing one of
  these spellings there is an ordinary declaration colliding with
  nothing — this is §12.2's "primitive lookalike" rejection gate.
- Inside a kind-declaring unit, a same-spelling source declaration is a
  deterministic rejection under the new `DIAG-1` rank (inserted at position
  5, before the existing duplicate/shadow ranks) — neither name resolves;
  this is not shadowing and not a silent user override.
- Exactly three existing declaration-domain rows gain a system source:
  nominal-type TYPEID, constructor TYPEID, and the lexical-IDENT row for a
  `callee` only — **not** `fn_bind`. `SYS-2`'s own paragraph states this
  exclusion directly: a system operation is not a contract member, is not
  the right IDENT of an FN-3 `fn_bind`, and never satisfies FN-4's
  bound-function premise. Implement the exclusion as stated; do not extend
  admission to `fn_bind` on this task's own initiative.
- Calls use named arguments exactly like a user function call (`GRAM-11`).
  A system-operation call also writes its region arguments as explicit
  `targs` in declared region-parameter order, exactly like a call to any
  other region-parameterized function — this reuses the existing
  region-argument call machinery (`compiler/README.md`'s "caller region
  parameters"); no new call-argument grammar is needed beyond what task
  0006 and `GRAM-11` already cover.
- Out of scope: this task registers names and signatures only. It does not
  implement ownership/effect checking of calls into these operations (task
  0009), checked-IR resource tracking (task 0010), or native lowering
  (tasks 0011/0012). A resolved call to a system operation whose semantic
  path is not yet implemented must report the project's standard explicit
  "unsupported capability" diagnostic at semantic-check time — never a
  resolution failure, and never silent miscompilation.

## Method

Add a system-declaration catalog beside the existing `PRELUDE_DECLARATIONS`
table in `compiler/src/resolution/catalog.rs`, covering `SYS-2`'s exact
inventory — fourteen nominal types (seven opaque plus seven enum types),
thirty-nine enum-variant constructors, and eleven complete operation
signatures with their region parameters and modes — in `SYS-2`'s stated
preorder (each nominal type in table order; then each constructor and its
fields in declared order; then each operation and its parameters in
declared order). Each operation's declared `reads('r)`/`writes('r)` region
entries follow mechanically from its parameter modes (a borrow of region
`'r` contributes `reads('r)`; a `&uniq 'r` parameter the operation actually
mutates additionally contributes `writes('r)`) — the same derivation rule
the existing user-function effect-row inference already applies, so this
task does not hand-curate a region-entry table, only registers each
operation's parameter list and its fixed `external`/`blocks`/`traps`/`pure`
classification from `SYS-2`. Add a third `DeclarationOrigin`/
`ResolvedTarget` variant (`System`) in `compiler/src/resolution/mod.rs`; add
the third `resolve_uses` branch in
`compiler/src/resolution/engine/lookup.rs`, gated by task 0006's
kind-declaring flag, parallel to the existing prelude branch; add the new
collision rank and system origin handling in
`compiler/src/resolution/engine/inventory.rs` (where `DeclarationOrigin`
already lives); update the ordinal assertions in
`compiler/src/resolution/tests.rs`. Every existing match over
`DeclarationOrigin`/`ResolvedTarget` elsewhere in the crate (diagnostic
rendering, any `semantic/check.rs` consumer) needs the new arm — treat an
unhandled match as a build error to find them all, not a silent default.
Add the new explicit "system operation body unsupported" diagnostic surface
in `compiler/src/semantic/mod.rs`/`semantic/check.rs` for calls that
resolve but have no implemented semantic/lowering path yet.

## Scope and expected touch set

- `compiler/src/resolution/catalog.rs` (new system catalog table)
- `compiler/src/resolution/mod.rs` (`DeclarationOrigin`, `ResolvedTarget`
  third variant)
- `compiler/src/resolution/engine/inventory.rs` (new collision rank, system
  origin)
- `compiler/src/resolution/engine/lookup.rs` (third `resolve_uses` branch)
- `compiler/src/resolution/engine/admission.rs`, `roles.rs`
  (declaration-class wiring as needed)
- `compiler/src/resolution/tests.rs` (ordinal and collision-rank
  assertions)
- `compiler/src/semantic/mod.rs`, `compiler/src/semantic/check.rs` (new
  issue kind for the explicit unsupported-capability diagnostic; match-arm
  updates the new origin variant forces)
- Read-only: `spec/kernel-spec-v0.18.md` §16; dossier §9, §6.7, §6.8.

## Dependencies and integration order

Depends on task 0006 (the kind-declaring predicate must exist and be
stable). Tasks 0008 and 0009 depend on this task.

Overlap cross-link (lead-granted): this task was claimed while 0006 was
still live, under the explicit integration order that 0006 lands first and
this task rebases onto it before landing, adopting its kind-declaring
accessor as the [SYS-3] admission trigger. Resolved: 0006 landed at
`5cd1eef` (closure `615bbae`); this task's base includes it and consumes
`crate::syntax::unit_program_kind` (`compiler/src/syntax/entry_form.rs`) as
the admission trigger, keeping the [SYS-3] decision after
`check_requires_blocks` per DIAG-1's stage order.

## Progress

- Done: complete implementation at `928b050` on this worktree branch,
  awaiting lead review. The SYS-2 catalog (167 records, preorder ordinals,
  full signature data, mechanical reads/writes derivation) sits beside
  `PRELUDE_DECLARATIONS` in `compiler/src/resolution/catalog.rs`; the SYS-3
  admission decision reads `syntax::unit_program_kind` after FN-8 admission
  and replaces the activation gate; `DeclarationOrigin::System` /
  `ResolvedTarget::System` carry `system_declaration_ordinal`; the DIAG-1
  rank-5 collision fires at the source event in both directions, root and
  nested alike, with the system conflict bucket ordered between PRE-1 and
  source; `fn_bind` right IDENTs never admit a system operation; the
  unsupported boundary moved to semantic checking
  (`SystemDeclarationUse` at the first resolved system use,
  `KindDeclaringEntry` for a system-name-free kind-declaring unit), and
  resolution no longer has an unsupported outcome.
- Validation: `make -C compiler check` and `make check` green (381 tests).
  New tests: catalog counts/properties/entity recovery plus a byte-exact
  extraction of both SYS-2 blocks from the active spec text (signature
  rendering compared string-for-string); kind-declaring resolution of all
  14 types, 11 callees, construct and arm constructors with exact ordinals;
  unadmitted-unit undeclared-name behavior; lookalike acceptance; rank-5
  collisions in all three domains, both textual directions, and a nested
  scope; rank-4-before-5; conformance-binding exclusion; repeated-run and
  path-independence determinism; semantic and driver gate tests.
- Findings for lead routing (not absorbed here):
  1. v0.18 internal contradiction: [SYS-2] fixes `arg_get`'s second
     parameter name as `index`, [GRAM-11]+[SYS-2] require every system call
     to write each parameter name as a `fieldinit` IDENT, but `index` is a
     fixed [GRAM-5] atom excluded from IDENT by [FORM-3], so no complete
     legal call to `arg_get` exists. Reproduction: a canonical unit calling
     `helper(index: x)` rejects at the `:` citing GRAM-5; renaming the
     parameter to `position` compiles the identical shape (probe pair under
     `/Users/bytedance/do_not_scan/wf-0007/`). Blocks task 0008's
     named-argument admission for `arg_get` and any argv-reading program
     (the wfgrep slice); a specification-change batch must rename the
     parameter or amend the grammar. The catalog keeps the normative
     spelling unchanged.
  2. Pre-existing (v0.17-era) discrepancy: the resolver cites
     `ResolutionRule::Fn4` for a failed `fn_bind` right-IDENT lookup, while
     DIAG-1's role table (v0.17 and v0.18 alike) says FN-3. Not touched by
     this task.

## Validation

`make -C compiler check`; new resolution unit tests for: a kind-declaring
unit resolving all seven types and eleven operations by name; a
non-kind-declaring unit where the same spellings are ordinary undeclared
names; the lookalike-acceptance case; the same-spelling collision rejection
inside a kind-declaring unit citing the new `DIAG-1` rank. A claimed task
lands only through lead review per the executor lane in
`docs/WORKFLOW.md`.

## Done-when

All eleven operation names and seven opaque types resolve correctly under
both admission states; the new `DIAG-1` rank is exercised; a call into a
resolved-but-unimplemented system operation reports "unsupported," never a
resolution failure or silent acceptance; `make -C compiler check` green.
