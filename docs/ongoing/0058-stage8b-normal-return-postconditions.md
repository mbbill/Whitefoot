# 0058 — Stage 8b verified-postcondition semantic freeze

- **Status:** `IN PROGRESS` (resumed 2026-08-14 after owner selection of the
  Stage 8b Current Plan correction)
- **Owner / workspace:** Codex lead /
  `/Users/bytedance/do_not_scan/whitefoot-0058-stage8b-normal-return-postconditions`,
  branch `codex/0058-stage8b-normal-return-postconditions`
- **Base revision:**
  `fa689611162452614eac79cfcf35fed85a9a16eb`
- **Authority:** the `ACTIVE` Current Plan revision selected 2026-08-14,
  Workstream 8b `verified normal-return postconditions`, derived from Direction
  Outline revision 36 item `PROOF-8`

## Goal

Freeze one complete, hostile-reviewed v0.28 semantic candidate and decompose it
into decision-complete, single-context executor handoffs. This lead-owned task
settles spelling, judgments, ordering, provenance, derivation, diagnostics,
consumer mapping, and the later approval boundary. It does not implement the
compiler, migrate a real consumer, modify protected conformance, activate a
specification, or ask the owner to approve incomplete bytes.

## Fixed premise and refreshed baseline

- Active v0.27 specification SHA-256:
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.
- Frozen real sources, in compilation order:
  - `raw_deflate.wf`:
    `5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585`;
  - `raw_deflate_dynamic.wf`:
    `2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5`;
  - `raw_deflate_dynamic_decode.wf`:
    `72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f`;
  - `raw_deflate_boundary.wf`:
    `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`;
  - `wfgrep.wf`:
    `a1e49bcb9ffd353e707d4bbafe1eb4a2b634b9177c6916b2ff7b503ec5dff0bd`.
- Installed acceptance: UTF-8 `33/22/11/0`, SHA-256 `9/9/0/0`, complete
  DEFLATE `29/24/5/0`, dynamic DEFLATE `24/19/5/0`; twelve DEFLATE claims are
  seven retained, five redundant, zero refuted.
- Current post-DIAG baseline: provenance `41/41`, raw-DEFLATE `3/3`, compiler
  library `718/718`, real programs `30/30`, and rule coverage `131/131`. The
  Current Plan's original pre-DIAG compiler baseline was `698/698`.
- Protected baseline: 423 cases, 30 coverage annotations, and adapter
  `Pass=409 Fail=1 Skip=13`; the only failure is the existing OWN-3 unsupported
  boundary.
- Research premises:
  - task 0051 bit-source witness section
    `0e1c9336b2b15d9a7c2d84d067514019ae8c5878b0b05183ba3f2c6be18cfc65`;
  - task 0052 counted-append witness section
    `42736be221f5bb60ba30cac07b04f8f4e5b3e17ac88b85baf48dd57d0be90e1a`;
  - task 0053 complete 14/20 caller map section
    `c23f92e1b835e6269358e2a1b0fcdc84db7b15c2e6f93995031a45e4a5c985d2`;
  - task 0056 DIAG-2 trust/cost section
    `7d3c6827d92c45a2916298b6d1c347fec7726117afc50293015fc5452844cc5e`.

The frozen output, error, cleanup, status, claim, accepted-set, adapter, and
protected identities are validation oracles, not values this task may rewrite.

## Design constraints already in force

The live design memory and active plan rule out five tempting shortcuts:

- ordinary calls prove complete requirements before transfer; no callee
  prologue or hidden runtime fallback replaces that proof;
- recognizers and implicit retained checks do not regain acceptance authority;
- one selected accepted ordinary-call `Ok` payload's verified normal-result
  relation may cross its arm-local scope only through the plan's
  result-specific bare-payload-to-bare-outer receiver event; that event
  establishes no assignment equality or arbitrary RHS fact transfer;
- A10's independent conditional delivery remains the existing `give`-based
  `value_if` form only; no Stage 8b read caller relies on a `value_match`
  delivery rule; and
- every new fact extends the same function-local DIAG-2 derivation DAG, with no
  second closure, semantic walk, proof authority, identity system, or lowering
  path.

## Exact semantic questions to freeze

### 1. Surface and declaration identity

Define one canonical normal-return postcondition spelling, its declaration
placement and rule identifiers, result/outcome selectors for every admitted
return type, alpha/concrete identity, and exclusions. Resolve duplicate,
missing, ill-typed, non-pure, effectful, trapping, non-total, unsupported, and
false clauses with deterministic diagnostics. No syntax may trust a writer or
silence an existing check.

### 2. Callee proof and provenance

Define which normal exits must prove the complete instantiated clause, how
early returns and nested value forms contribute, and how `Err`/other outcomes,
divergence, cleanup, consumes, writes, scope exits, joins, recursion, generic
instances, and contradiction behave.

Freeze three distinct analyses of every candidate postcondition: complete,
unasserted (S2/S3 disabled), and S4-blinded. A proposition proved only by a
callee `check`, `claim`, requirement seed, or blinded requirement bridge must
not become unconditional caller U/B evidence. Specify direct, two-hop,
self-recursive, mutually recursive, seedless-cycle, command-entry, and
constrained-subject bridge behavior under PRV-1/PRV-2/PRV-3, including which
diagnostic owns a rejected publication. Postcondition summaries must retain
the exact result projection and assertion-dependence needed to make this gate
fail closed.

### 3. Ordinary call ordering and the narrow direct-result receiver rule

Keep requirement and provenance proof in the caller's pre-transfer state.
Freeze this exact order: resolution and concrete type/const instantiation;
argument/type checks and borrow feasibility; actual-expression obligations;
exact formal-datum and projection substitution; complete FN-8 requirement
proof; the PRV-2 call-argument gate against that same pre-transfer state; and,
only for a call with no rejection event, borrow/consume commit, callee-effect
kills, target `set` kill, normal-result substitution, and result-fact
publication. FN-8 or PRV-2 rejection performs no transfer and publishes no
postcondition fact. Complete, unasserted, and S4-blinded caller analyses may
receive only the correspondingly admissible instantiated summary relation.

The twenty mapped append rows exercise only this general semantic shape: when
an ordinary call's result is assigned directly to the same typed binding that
supplied the corresponding formal, apply all ordinary call/write/consume and
target-write kills, substitute the normal-result atom with the post-write
receiving binding, retain only surviving exact support, and publish that
instantiated relation with its DIAG-2 parent. Formal datums, types, consts, and
projections remain the exact pre-call actual identities: if the target also
supplied an actual, its overwritten pre-call value is killed and must never be
conflated with the post-call receiver introduced only by result substitution.
Define aliasing, projected targets, distinct receivers, non-atom results,
discarded results, nested expressions, FN-8 or PRV-2 rejection, and
contradictory states. This is a direct call-result rule, not general `set`
equality or arbitrary RHS fact transfer.

### 4. Selected accepted-call payload receiver and A10-only delivery

For the fourteen retained direct-call match shapes, freeze one general but
shape-bounded selected-outcome receiver event. The match scrutinee must be one
ordinary user call that has passed the complete resolution, instantiation,
argument, borrow, actual-obligation, FN-8, and PRV-2 sequence above. At the
selected direct `Ok(value: payload)` arm, the entry state must hold the exact
instantiated planned-S12 verified normal-result relation. A direct
non-consuming `set outer = payload;` may carry that relation only when
`payload` is the bare own fragment binder, `outer` is a previously live bare
outer own fragment binding of the exact same type, and no intervening event
occurs.

Evaluate the RHS and apply the ordinary target commit and kill first. Then
replace only the result-payload occurrences in that relation with the
post-write receiver and re-establish it in the same complete, U, or B view only
when every non-payload support remains live. `outer` may not be an actual of
the call; within the instantiated relation it may not occur as a non-result
term or support, nor overlap another substituted support. Unrelated
pre-existing facts on `outer` neither authorize nor block the event and die by
the ordinary target kill. A projected, computed, or consuming RHS; a wrong or
unselected binder; a differently typed receiver; a named or pending outcome;
an extra reaching write; aliasing; a missing planned-S12 relation; or killed
support establishes nothing. Payload scope exit and the ordinary match join
then apply normally. The event establishes neither `outer = payload` nor any
unrelated fact. The mapped `read_bits` calls exercise this general semantic
shape; source, function, corpus, or test identity does not select it.

Retain bounded `value_if` delivery; A10 is its only Stage 8b consumer. On each
reaching `give` edge, for each eligible typed L0 relation replace every
occurrence of the delivered non-consuming bare copy atom with the receiving
binding, apply ordinary scope-exit and event kills to every other support, and
pass the surviving edge relations through the ordinary L0 join. Establish only
each joined typed relation on the fresh receiver, supported by it plus the exact
surviving support. Perform this independently in the complete, U, and B views;
no edge evidence crosses views. A non-atom or consuming delivery, a missing or
ill-typed edge relation, or a join with no common relation establishes nothing.
This is delivery-specific substitution, not assignment equality, and no
Stage 8b consumer relies on a corresponding `value_match` rule.

### 5. Measured local facts and real declaration contracts

Freeze only the two unsigned sources established by task 0051:
`iand` result bounded by each admitted operand, and nonzero `ishl.wrap(one,
count)` only for a checked unsigned mathematical-one constant. Preserve every
signed, operation, operand, boundary, support, and kill near miss.

Freeze the truthful counted `append_slice` body, admitted-domain requirement,
and normal-result relation on both distinct source declaration identities from
task 0052/0053. Invalid-domain behavior must not change. Freeze the one wfgrep
host-copy repair after its child region ends: retain the prior length and copied
count in outer scalars, form `candidate_length`, and use one `value_if` to
select the candidate on `candidate_length <= len(deref(report))` or the
unchanged prior length otherwise. Bind the result as `bounded_length`; use that
same binding as both receiver and `filled` actual for the separator and every
later append, and pass it to `publish_all`. Do not write it back through the old
`length` binding or add variable-plus-variable/variable-offset S7, a trusted
system-result premise, or another consumer repair.

### 6. DIAG-2, diagnostics, and exact impact

Assign every accepted clause proof and every published caller fact an exact
root and parent shape in the existing derivation ledger, including source,
substitution, call publication, assertion/provenance disposition, direct
call-result receiver, selected-outcome payload receiver, `value_if` delivery,
join, contradiction, and kill cases. Define deterministic error precedence and
residual wording. Inventory grammar, name, type, ownership, effect, runtime,
check, diagnostic, ABI, accepted-set, conformance, real-consumer, and
limitation impact before implementation.

## Method and handoff

1. Work only in the declared isolated 0058 branch. Produce a complete v0.28
   specification candidate, the exact outgoing v0.27 archive candidate, and
   impact inventory there; those bytes are non-authoritative and do not enter
   the integration branch.
2. Run the current native grammar/spec verifier as an identity preflight and
   independently audit the new strong-LL(2) decisions and hostile semantics.
   Because that verifier intentionally compares candidate grammar with the
   compiler's embedded active frontend contract, a real grammar addition
   cannot pass it before the first compiler handoff installs the candidate
   lexer/parser/table bytes. That handoff must make the same native verifier
   pass before any owner packet is formed. A prose placeholder or executor
   choice is a failure, not a delegated detail.
3. Freeze exact candidate bytes, rule IDs, expected diagnostics, derivation
   shapes, protected impact categories, and the implementation dependency DAG
   in one reviewed candidate commit.
4. On the integration branch, record that candidate commit and register the
   smallest consecutive cross-linked executor tasks for compiler/derivation,
   real-consumer migration, and the protected corpus/combined packet. The
   packet task may register an activation task only after the exact reviewed
   candidate and approval premise exist. That activation handoff must require
   any live MCTS item or genuine design re-decision to be updated and linted in
   the same activation change, or record an explicit reviewed no-change
   conclusion. Each task must fit one context, state its exact premise and
   integration order, and stop rather than redesign.
5. Move this task to terminal history after those handoffs land. Candidate
   implementation then proceeds under their records; no owner approval is
   requested until the full spec and protected bytes are stable together.

## Scope and expected touch set

- Integration branch: this task record, roadmap status, and later subordinate
  task records only.
- Candidate branch: `spec/kernel-spec.md`, the exact outgoing
  `spec/kernel-spec-v0.27.md` archive candidate, the derivation-ledger prose,
  and the minimum scratch impact inventory necessary to freeze executor
  semantics.
- Temporary: grammar candidates, matrices, review notes, and logs only below
  `/Users/bytedance/do_not_scan/whitefoot-0058-*`.

No compiler, generated, real-program, conformance, installed-archive,
approval-ledger, MCTS, runner, Makefile, gate wiring, backend/lowering, runtime
ABI, or unrelated documentation byte belongs to this semantic-freeze task.
The candidate-only v0.27 archive must not enter the integration branch. A need
for any such integration change stops this task and routes it to the later
named handoff or the applicable plan/owner boundary.

## Validation

- Every numbered semantic question has complete positive, hostile, false,
  support, kill, join, scope, cleanup, generic, recursive, provenance, and
  deterministic-identity expectations suitable for executor tests.
- The current native grammar/spec verifier is recorded rejecting only the
  expected candidate-versus-embedded-v0.27 identity mismatch; independent
  strong-LL(2) review finds no overlap, and the first frontend handoff is
  required to make that verifier pass with candidate tables before the owner
  packet. The outgoing v0.27 authority remains byte-identical on the
  integration branch; its candidate archive is byte-identical to the outgoing
  active specification and supplies the verifier's exact `PREVIOUS` input.
- The complete 34-row caller map replays on paper: the fourteen read rows use
  the selected `Ok` payload receiver, the twenty append rows use direct
  call-result publication, and A10 additionally uses the `value_if` repair.
  No solver, third fact source, general `set` equality, variable-addition S7,
  runtime fallback, or unresolved/refuted relation is admitted.
- Two independent reviews find no unwritten choice, provenance laundering,
  accepted-set ambiguity, proof-authority split, or unbounded implementation
  requirement.
- The registered executor handoffs are decision-complete, single-context,
  cross-linked, and name the one later combined spec/protected hard wait.

## Stop condition

Stop with the smallest counterexample if the design needs general loop
induction or a fixed point, arithmetic-expression terms, Boolean decomposition,
a solver, recognizer, third fact-source family, trusted writer assertion,
general `set` equality, variable-offset or variable-plus-variable S7, a second
proof authority/closure/semantic walk, serialized proof framework, portable
identity, hidden runtime fallback, backend/lowering/ABI change, runner/gate
wiring change, or material Current Plan expansion. Also stop if a normal exit
cannot prove its clause, `Err` publishes a result fact, assertion-dependent or
S4-only evidence can launder into a caller, append result publication needs a
broader RHS rule, the read-side receiver needs a projected/computed/consuming
RHS, a call-actual receiver, aliasing, non-payload substitution, or any fact
beyond the exact selected planned-S12 relation, A10 needs `value_match`
delivery, general `set` transfer, new arithmetic entailment, or cannot pass
through the ordinary per-view `value_if` L0 join, any of the 34 mappings remains
unresolved/refuted, or the candidate cannot be made executor-complete in
bounded rule text.

## Progress

- **Current:** the owner-selected Current Plan correction carries the exact
  narrow selected-`Ok` receiver boundary above. This semantic-freeze task may
  resume only after its isolated worktree refreshes and rebases onto the plan
  activation commit. The withdrawn scratch v0.28 draft remains withdrawn and
  supplies no candidate or authority; the complete candidate must be
  re-frozen from the active v0.27 baseline.
- **Preserved surface premise:** the first hostile surface pass rejected the
  research shorthand
  `Ok(result) => result < ishl.wrap(1_u64, count)`: the operation tree is not an
  [ENT-2] L0 term, and admitting it would cross this task's arithmetic-term
  stop. The bounded replacement is an explicit `mask: own u64` parameter on
  `read_bits`, a verified `Ok(result) => result <= mask` relation, literal
  masks at twelve fixed-count callers, and two caller-scope `high`/`mask`
  computations at the variable-count callers. The measured unsigned shift,
  existing constant-offset subtraction, unsigned `iand`, and ordinary ENT-4
  closure recover all fourteen intended bounds without a new term or source
  family at the callee/call boundary.
- **Resolved plan premise:** the predecessor plan's fourteen
  value-producing-match rewrites were not expressible under active OWN-6.
  Every call takes a statement-scoped child reborrow whose local region block
  may not extend beyond the enclosing match or proposed value-match `let`
  statement. A value-producing match inside that block binds its result only
  inside the block; extending the block past that `let` to a later consumer is
  an OWN-6 rejection, while ending it with the statement makes the result
  invisible. The revised ACTIVE plan supersedes only that infeasible migration
  with the selected payload receiver above. The broader OWN-6/value-region and
  unverified row-specific code-motion alternatives remain excluded. No
  compiler, real-source, active-spec, or protected-conformance byte changed to
  resolve this plan boundary.
- **Separate A10 premise:** retain the prior length and copied count outside
  the child region, compute `candidate_length`, select it versus the prior
  length through `value_if`, and use only `bounded_length` for all later append
  and publication uses. Compile and behavior validation remain future work.
- **Boundary routed forward:** activating any v0.28 bytes must also update the
  three target-qualification version mappings currently fixed to `v0.27` in
  `compiler/src/backend/qualification.rs`. That is a mechanical activation
  identity update, not a new lowering, runtime ABI, or semantic path. It does
  not belong to this candidate branch and must be named explicitly in the
  later atomic activation handoff; omitting it would make the real system
  operations fail qualification.
- **Verifier preflight:** the active native grammar verifier accepts the exact
  v0.27 previous bytes and rejects the draft v0.28 grammar solely because the
  compiler still embeds the active v0.27 frontend contract. This is the
  expected pre-implementation identity boundary, not a grammar verdict. The
  first frontend executor handoff must install the frozen candidate tables and
  make the same verifier green; no owner packet may be prepared before that.
- **Next:** refresh the integration branch, rebase the isolated 0058 worktree
  onto the activation commit, reread the revised authority, and re-freeze the
  complete v0.28 candidate. Hostile-review the selected receiver over all
  fourteen read rows and near misses, the twenty append rows, A10's `value_if`
  repair, every planned FN-9/S12 complete/U/B and PRV disposition, and every
  DIAG-2 root before registering executor handoffs or forming an owner packet.
- **Approval state:** only the high-level Current Plan correction has been
  selected. No Stage 8b specification or protected-conformance candidate has
  been presented or approved; integration remains on active v0.27, and all
  candidate bytes remain non-authoritative.

## Done-when

One exact, independently reviewed v0.28 semantic candidate and impact inventory
exist on the isolated candidate branch; every implementation choice is frozen;
the decision-complete executor handoffs and integration order are registered;
this task is terminal; and no candidate specification or protected byte has
landed or been presented for premature approval.
