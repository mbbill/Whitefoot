# Current Plan

Status: PROPOSED (AI proposal 2026-08-10): measure two local
proof-feasibility prerequisites for future normal-return postconditions and
audit, without closing, their real caller prerequisites as Stage 8a of the
selected obligation-discharge direction.

This proposal authorizes no execution, research run, task registration,
specification change, compiler change, source migration, or protected-material
change. Owner selection must change this file to `ACTIVE` in a separate plan
selection commit before any work below begins. Stage 8b, stages 9a/9b, and
further wfgrep work remain unauthorized.

Derived from: [Direction Outline revision 31](roadmap.md), item `PROOF-8`
(primary), with `VERIFY-1` and `VERIFY-2` as safety and evidence constraints.
`CAND-8` remains the selected flagship but stays parked until the complete
obligation-discharge direction reaches its completion boundary.

## Proposed milestone — bounded postcondition proof feasibility

### Why

Future Stage 8b may expose verified normal-return facts to callers, but two
real helpers first need local facts the active v0.27 entailment fragment does
not establish. `read_bits` needs a bit-mask bound on its successful result.
`append_slice` needs a truthful bound on its returned filled length. Stage 8a
does not claim that those two local facts suffice for either real caller
sequence: it separately audits every mapped call prerequisite and records any
remaining caller-side gap before Stage 8b can be proposed.

This stage measures whether two deliberately small structural additions and an
existing counted-range form suffice. It does not design or implement
`ensures`, select syntax, or leave a production fact source behind. A negative
result is useful evidence: it stops Stage 8b instead of hiding general
induction, arithmetic entailment, a solver, or a source-shape recognizer inside
the postcondition work.

The historical unconditional target `append_slice result <= capacity` is
false: when `filled > len(deref(destination))` and `len(text) = 0`, the current
body returns `filled`. This proposal therefore selects the truthful future
boundary `filled <= len(deref(destination))` and measures the result fact only
inside that admitted domain. The proof-only scratch helper must express that
boundary with this exact existing-language clause and trap payload:

```whitefoot
requires {
  let capacity = len(deref(destination));
  let admitted = ile(filled, capacity);
  check admitted else trap "append filled exceeds destination";
}
```

It does not select a conditional postcondition or change invalid-domain
runtime behavior.

### Do

1. Freeze the installed authority and real witnesses before probing:

   - active v0.27 at `spec/kernel-spec.md`, SHA-256
     `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`;
   - the four-source raw-DEFLATE unit, in compilation order:
     `raw_deflate.wf`
     `5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585`,
     `raw_deflate_dynamic.wf`
     `2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5`,
     `raw_deflate_dynamic_decode.wf`
     `72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f`,
     and `raw_deflate_boundary.wf`
     `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`;
   - `wfgrep.wf`, SHA-256
     `a1e49bcb9ffd353e707d4bbafe1eb4a2b634b9177c6916b2ff7b503ec5dff0bd`;
   - the fourteen real calls to `read_bits`, the twelve wfgrep calls and eight
     raw-DEFLATE boundary calls to `append_slice`, their output/error oracles,
     the current 423 conformance cases, all existing verdicts and rows, and all
     30 coverage annotations.

2. Establish the negative baseline before adding hypothetical facts:

   - show that active v0.27 cannot derive the exact local successful-result
     bound `value < mask_high` at the `read_bits` return;
   - enumerate the fourteen caller mappings from the returned `Ok(value:)`
     payload and formal `count` to their exact actuals, and show that v0.27 has
     no normal-result carrier that could publish a callee-local term or
     substitute a result relation at those callers; additionally reproduce the
     common source seam at all fourteen calls: the `Ok` payload binder is
     assigned with `set` into a predeclared outer binding, active ENT-3 has no
     fact transfer for that `set` right-hand side, the assignment kills the
     outer binding's prior facts, and arm/region exit kills the inner binder's
     support, so even a hypothetical fact attached to the `Ok` binder does not
     reach the later outer use;
   - show that the ordinary `append_slice` loop carries no accepted fact from
     one mutation of `at` to the next or to its return;
   - audit all twenty mapped `append_slice` calls under the hypothetical result
     fact alone, without installing it, and reproduce the known wfgrep
     prerequisite gap: after the prefix append in `report_failure`, the
     prior `length <= len(deref(report))` relation correctly survives the
     `host_copy_bytes` element write under ENT-5, and the success arm
     establishes only `copied <= 256`; then
     `set length = length +wrap copied` kills the old scalar `length` relation
     and has no variable-offset S7 equality to rebuild it, so the separator
     append cannot prove `length <= len(deref(report))` from the candidate
     summary and active v0.27 facts; and
   - execute the `filled > capacity && len(text) = 0` counterexample and retain
     its exact returned value, proving that no unconditional result bound may
     be proposed.

3. Test exactly two unsigned bit-operation fact sources for `read_bits` in a
   temporary dark-checker harness:

   - after `let result = iand(left, right)`, independently derive
     `result <= left` or `result <= right` only when that corresponding operand
     is an [ENT-2] term or checked constant; and
   - after `let high = ishl.wrap(one, count)` for an unsigned integer `T`,
     derive `high != 0_T` only when `one` is a checked unsigned constant whose
     mathematical value is exactly one. A typed literal one and a named const
     of the same checked value are positive forms; a non-term expression is
     not converted into a term merely because it evaluates to one.

   Use only existing scalar closure and the existing subtraction fact for
   `mask = high -wrap 1_T` to test whether
   `value = iand(hold, mask)` yields `value < high` at the successful return.
   Record that relation as a candidate normal-result fact only for
   `Ok(value:)`; `Err(error:)` carries none. Do not install a caller summary or
   substitution rule, add an arithmetic term, decompose a Boolean, invoke a
   solver, or add a source-, function-, corpus-, or test-shaped recognizer.

4. Challenge the bit candidate with exact negative and near-miss controls.
   Exercise unsigned `u8`, `u16`, `u32`, and `u64`; for each width `W`, test
   counts `{0, 1, W-2, W-1, W, W+1}`. Test the corresponding signed types; a
   left shift operand other than typed one, including zero, two, and the type's
   maximum; `ior` and `ixor`; and an arbitrary shifted value. Swapping the two
   `iand` operands must preserve both unsigned bounds.

   Invalidation is per derived bound, not per operation origin. Mutating the
   operand named by `result <= left` kills that bound but retains
   `result <= right`; mutating the right operand does the converse; mutating
   `result` kills both. The real `read_bits` mutation of `state.hold` after the
   `iand` is a positive control: it kills the unused hold bound while
   `value <= mask` survives to the return. An unrelated mutation also preserves
   both. Mutating `high` kills `high != 0_T`; mutating `count` after `high` was
   bound preserves that disequality because its support is only `high` and
   zero. The `Err` arm publishes neither bound. Every negative candidate fails
   by absence of the hypothetical fact, never by changing source acceptance.

5. Test one existing-structure route for `append_slice`, only in throwaway
   proof and runtime variants:

   - put the proof-only variant in a standalone scratch compilation unit with
     a valid noncalling entry; give the helper the truthful existing-language
     requirement clause fixed above and analyze its body and returns without
     bypassing call checking or treating any real caller as accepted;
   - replace the ordinary mutation loop with the existing counted range
     `for @append at in filled..capacity`;
   - in each iteration compute and bind `taken = at -wrap filled`, without
     requiring or claiming a variable-subtraction fact; return `at` when
     `taken >= len(text)`, and otherwise copy `text[taken]` into
     `destination[at]`; and
   - return `capacity` at range exhaustion.

   Measure whether the counted-range interior fact proves `at < capacity` on
   the early return and the value branch proves `taken < len(text)`. The
   exhaustion path returns `capacity` directly; it neither relies on nor
   invents a binder-equals-upper postcondition after the counted construct.
   Record `result <= len(deref(destination))` as the candidate normal-result
   fact at each proved return. Enumerate the exact result, destination, and filled
   mapping at all twenty real calls, but do not install a result summary,
   substitute the relation, establish a caller fact, or change call acceptance.

6. Prove the counted-range body behaviorally equivalent to the current body on
   its admitted domain. The runtime variant uses that counted body but retains
   the current signature without a requirement, so this stage does not need a
   caller-summary bypass to compile either real program. Exhaustively test
   capacity and text length in `0..=8` and every filled value in
   `0..=capacity`. For each tuple, compare returned length and every destination
   byte with destination fills `0x00` and `0xa5`, and text patterns all-`0x00`,
   all-`0xff`, and ascending byte ordinals. Then run exactly
   `cargo test --manifest-path compiler/Cargo.toml --test programs wfgrep`
   (9/9) and
   `cargo test --manifest-path compiler/Cargo.toml --test programs raw_deflate`
   (3/3), preserving every existing status and byte oracle. Inputs violating
   the selected requirement are caller rejections in a future design and
   receive no equivalence claim.

7. Keep the experiment bounded and removable. A temporary in-crate probe may
   use the production resolver, type/effect checker, checked representations,
   and entailment machinery, but it must be deleted after the run and every
   host-file hash restored. No tracked specification, compiler, source,
   conformance, MCTS, or generated byte may remain changed. Append the exact
   measurements and minimal reproducers to the existing obligation-discharge
   acceptance record only after the run; do not create a new framework or
   evidence bundle.

8. Report proof dispositions and deterministic witnesses for the two local
   normal-return goals and every negative canary. Separately report the exact
   fourteen and twenty caller mapping inventories as non-deriving Stage 8b
   inputs, including each mapped caller's future requirement disposition when
   only the candidate result fact is assumed. Classify each mapped requirement
   as `discharged` or `unproved`; an unexpected refutation is a blocker. The
   audit must identify both pre-registered seams: every raw-DEFLATE
   `Ok`-binder-to-outer-binding `set`, and wfgrep's `length +wrap copied`
   transition as at least one `unproved` append premise. It may report more. Do
   not call any mapping a proof, use one for acceptance, claim either real
   consumer sequence feasible, or treat a positive local result as authority
   to begin Stage 8b. Record
   compile-time delta as a bounded measurement only; do not infer or build a
   certificate architecture from it. Task 0049's proof-certificate research
   supplies no production authority to this plan.

### Verify and accept

- The frozen hashes, call counts, conformance identities, and runtime oracles
  match before and after the temporary run.
- The negative baseline reproduces: neither exact result goal is available in
  active v0.27, and the unconditional `append_slice` bound has the stated
  concrete counterexample.
- The `read_bits` candidate proves the exact local goal on every `Ok(value:)`
  return; it publishes nothing on `Err`, the per-bound kill controls behave as
  specified, and every hostile or near-miss control remains unproved. The
  fourteen caller mappings are complete and well typed but derive no fact, and
  their common result-through-`set` seam is reproduced rather than hidden.
- The `append_slice` candidate proves `result <= len(deref(destination))` on every
  normal return under the selected requirement, preserves result and bytes on
  its complete tested admitted domain, and records all twenty caller mappings
  without injecting the result fact at any call. Its caller audit reproduces
  the wfgrep post-copy premise gap and reports every other unproved premise;
  local proof success is not real-sequence feasibility. Both real program
  oracle suites remain byte-for-byte and status-for-status unchanged at
  wfgrep 9/9 and raw-DEFLATE 3/3.
- A complete per-disposition differential reports any new proof, redundancy,
  refutation, rejection, or unsupported result. No existing case, verdict,
  status, source byte, or acceptance result changes.
- Focused checks and `make check` are green, the temporary probe is absent, the
  worktree and index are clean, and no production or normative file changed.

Acceptance is evidence-only. A positive result supplies two decision-ready
local fact prerequisites, one counted-range source rewrite, and complete
caller-prerequisite inventories for a later owner decision; it does not prove
either real sequence feasible, authorize `ensures`, define result-summary
substitution, select how a result fact crosses the fourteen payload-to-outer
`set` transitions, select the source repair or additional fact needed after
`length +wrap copied`, or make either hypothetical fact normative. Stage 8b
does not begin automatically after any Stage 8a outcome. A later proposal must
dispose of every recorded caller gap explicitly, and if either local witness
cannot close within this exact boundary Stage 8a returns the smallest
reproducer.

### Stop condition

Stop and return evidence to the owner if either goal requires general loop
induction or a loop fixed point, arithmetic-expression terms, Boolean
decomposition, a solver, a source/function recognizer, invariant syntax, a
postcondition language, an unproved trusted premise, or any fact source beyond
the exactly two candidate operation-semantic sources above; if the
counted-range variant is not behaviorally equivalent in the admitted domain;
if either caller mapping inventory is incomplete or ill typed; if
invalid-domain behavior would need to change; or if any
protected source, verdict, row, annotation, runtime oracle, active-spec byte,
or production compiler byte must change. Do not broaden the experiment or
silently proceed to Stage 8b.

### Explicit exclusions

This proposal does not select `ensures` spelling, grammar, effects,
diagnostics, generic substitution, early-exit semantics, or any other Stage 8b
language rule. It does not authorize a proof certificate, ProofFlow or DIAG-2
architecture, general induction, arithmetic entailment, O11 Boolean
composition, claim-ledger work, `deny-claims`, optimizer facts, or wfgrep
performance work.

### Authority and task boundary

This file is an AI proposal and grants zero execution authority. If the owner
selects it, a separate commit changes only its status and stale proposal
wording to `ACTIVE` without expanding the written scope. Only after that commit
may the integration branch be refreshed and the next free task number be
registered in another lifecycle commit. This proposal does not reserve a task
number, create a planned or ongoing record, or authorize substantive work.

## Later dependency map — not execution authority

### Stage 8b — normal-return postconditions

Only after positive local Stage 8a evidence, owner disposition of every caller
gap, and a separately owner-selected ACTIVE plan may Whitefoot propose the
smallest `ensures` language that exposes verified normal-return facts to
callers. That proposal must first dispose of every Stage 8a caller-prerequisite
gap, including wfgrep's post-copy
`length +wrap copied` transition and raw-DEFLATE's fourteen
payload-to-outer-`set` transitions, by explicitly selecting a real source
repair or a separately justified finite fact source for each class; local
Stage 8a success alone is insufficient. Stage 8b owns result-summary identity,
outcome selection, formal/result substitution, support, kills, and call-site
establishment; none is selected by the Stage 8a mapping inventory. Any exact
specification bytes still require the complete explanation, hard wait, digest
approval, archive, and atomic activation workflow.

### Stages 9a and 9b — claim ledger and strict partition

After Stage 8b is terminal, generate a deterministic read-only claim ledger
before proposing an opt-in `deny-claims` partition. Neither stage is authorized
here.

## Stable specification and cross-stage invariants

- The active specification remains `spec/kernel-spec.md`; v0.27 has no
  versioned sibling while active. Released archives are immutable.
- One normal compiler path; no project-, function-, source-, or test-shaped
  behavior.
- Facts widen discharge only when normative entailment derives them. A
  temporary hypothetical fact never becomes acceptance authority.
- Expected failure is a value path; claims remain executed runtime backstops
  for broken program invariants.
- Protected material never changes without exact owner approval, and
  unsupported capability never becomes source rejection.
- Durable design decisions and rejected alternatives follow the
  `mcts-mem-use` workflow only when a later selected plan changes them.

## Direction completion boundary

Wfgrep remains parked until Stages 8a, 8b, 9a, and 9b are implemented end to
end, covered by positive, negative, near-miss, and invalidation evidence,
exercised by their named real programs, and recorded in the Direction Outline;
the complete repository gate is green; and remaining claims and unsupported
gaps are reported honestly. A reproduced Stage 8a stop condition returned for
owner disposition is the only earlier terminal outcome.
