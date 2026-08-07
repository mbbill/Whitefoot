# 0033 — ENT remaining fact sources

This is a temporary live coordination record, not execution authority.

- **Status:** `ONGOING` (claimed 2026-08-07)
- **Authority:** `ACTIVE` docs/current-plan.md selected slice; batch-1/v0.22 candidates and rulings in governance/
- **Owner / workspace:** exec-0033 / worktree `/Users/bytedance/do_not_scan/wf-0033`, branch `task/0033-ent-fact-sources`
- **Base revision:** 482609d39f82d4170dff297334d6cee601876256
- **Dependency:** 0032 (terminal, `docs/done/0032-ent-core-engine.md`)

## Goal

Extend 0032's engine with the remaining ruled sources: FN-8 requires substitution, check/claim facts, buffer_new/array_new length equality, const-array element ranges, literal/constant propagation with constant-offset arithmetic, S10 boundary count bounds (QUAL trust class). Unit tests per source incl. kill discipline. Still dark. Dependency: 0032 terminal.

## Progress

- Completed: every ruled source except S3 lands in
  `compiler/src/semantic/entailment/flow/sources.rs`, a child module of the
  walker so the graph, kills, joins, and obligation judgment stay in `flow.rs`
  and no visibility widens. S2 check facts; S4 requires substitution to
  fixpoint with the parameter/const operand restriction; S5 literal, copy, and
  total-[OP-6]-conversion equalities; S6 `buffer_new` and `len<T>(P)`
  equalities and `slice_of`'s source-length equality; S7 wrapping (range-
  guarded), trapping, and checked constant offsets; S9 const-array element
  ranges; S10 boundary count bounds for all four [SYS-2] transfers, resolved
  by catalog parameter name and observing-variant name. The [ENT-2] implicit
  `array<T, N>` length equality now registers wherever a length term is
  interned rather than only at obligation sites. S7-checked and S10 share one
  pending arm fact on the state under the comparison origins' no-kill,
  no-`set` discipline; kills, joins, scope exits, and the loop rule prune it
  alongside them. Landed on branch at b78cf38.
- Validation: 46 entailment unit tests (was 25); `make -C compiler check` and
  `make check` both exit 0 before and after; lib tests 488 -> 507 with no
  acceptance change. Mutation checks confirm the new tests fail when the S10
  variant/parameter names, the S4 repeated substitution, the S7 wrap range
  guard, or the S9 declared range are broken.
- Current: reported to lead for review; branch tip rebased on main.
- Next: lead review and integration; OP-4/CLM behavior and claim semantics
  are task 0034.

## Findings (reported to lead)

- **S3 has no checked representation.** A `claim_stmt` stops as
  `UnsupportedSemanticFeature::ClaimStatement` in `check/control.rs` before a
  checked tree exists, so no S3 clause can fire. Giving claim statements
  semantics changes acceptance (today's unsupported stop becomes an accepted
  program) and is explicitly 0034's scope. The clause is written where S2's
  is, so 0034 adds one call, not a design.
- **The card's S6 kill shape is unwritable.** "The allocation equality dies
  when the buffer binding is reassigned" cannot be shown: `set b = ...` on a
  buffer place is rejected by [STOR-1] `AffineSetTarget` before the engine
  runs. The two kill routes the language does admit are tested instead — a
  write to the term the equality is held against, and a consuming use of the
  buffer root killing a length binding taken from it.
- **S10's path condition on the direct scrutinee form.** [ENT-3] S10 admits
  the bound "where no [ENT-5] kill event applies to a fact supported by k on
  the path to the match". For a bare-IDENT scrutinee that path is explicit;
  for a direct call scrutinee the call's own boundary writes are on it. The
  engine takes the conservative reading and tests the call's own kills against
  k. The permissive reading would establish a bound over a place the same call
  just wrote, so only the conservative one is sound — recorded as a
  precision note, not a two-way ambiguity.
- No change to 0032's reported ENT-5 loop reading, per lead guidance.
