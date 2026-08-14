# Current Plan

Status: **ACTIVE** (owner selection 2026-08-14): preserve the owner-selected
obligation-discharge sequence while correcting Stage 8b's fourteen-call
result-delivery mechanism after hostile source replay falsified the former
existing-language value-match strategy.

The owner selected the predecessor plan on 2026-08-12 and explicitly selected
the narrow Stage 8b receiver correction described below on 2026-08-14. Task
0058 proved that OWN-6 prevents a value-producing match result from surviving
the mandatory statement-scoped child-reborrow region at all fourteen mapped
`read_bits` calls. This revision changes only that Stage 8b delivery mechanism;
it carries forward the objective, workstream order, consumers, verification
boundary, and later exact specification/protected-conformance waits. Its
`ACTIVE` status authorizes resumed task execution inside the exact written
boundary, but does not approve any future specification or protected
conformance bytes.

Derived from [Direction Outline revision 36](roadmap.md), primarily `PROOF-8`,
with `PROOF-1`, `VERIFY-1`, and `VERIFY-2` as proof, safety, and evidence
constraints. `CAND-8` remains the selected flagship pressure source but stays
parked until this plan reaches its terminal boundary.

## Objective

Finish the already selected obligation-discharge sequence without repeatedly
returning for task-level approval:

1. establish or falsify the two local facts needed by the real `read_bits` and
   `append_slice` consumers, and inventory every caller-side gap;
2. independently repair the checked program's already-required exact
   derivation retention before adding another normative fact family;
3. if those prerequisites close, design and activate the smallest verified
   normal-return postcondition mechanism that closes the real call sequences;
4. expose a deterministic, complete review ledger for the resulting claims;
   and
5. only after that ledger exists, add an opt-in strict partition in which
   obligations must be proved or handled by value control flow rather than by
   claims.

The authentic consumers are the four-source raw-DEFLATE unit and `wfgrep`.
The plan ends by making the obligation-discharge direction terminal and
unparking the existing wfgrep checkpoint. It does not begin a new wfgrep
optimization or performance project.

## Installed baseline and frozen boundaries

- Active language authority: v0.27 at `spec/kernel-spec.md`, SHA-256
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.
- Raw-DEFLATE sources, in compilation order:
  - `raw_deflate.wf`:
    `5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585`;
  - `raw_deflate_dynamic.wf`:
    `2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5`;
  - `raw_deflate_dynamic_decode.wf`:
    `72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f`;
  - `raw_deflate_boundary.wf`:
    `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`.
- `wfgrep.wf`:
  `a1e49bcb9ffd353e707d4bbafe1eb4a2b634b9177c6916b2ff7b503ec5dff0bd`.
- Installed acceptance: UTF-8 `33/22/11/0`, SHA-256 `9/9/0/0`, complete
  DEFLATE `29/24/5/0`, dynamic DEFLATE `24/19/5/0`; twelve DEFLATE
  claims are seven retained, five redundant, zero refuted.
- Installed tests: provenance `41/41`, raw-DEFLATE `3/3`, complete compiler
  library `718/718`, real programs `30/30`, rule coverage `131/131`. The
  predecessor plan recorded its earlier pre-DIAG compiler baseline as
  `698/698`.
- Protected conformance boundary: 423 cases, 30 coverage annotations, and the
  installed adapter result `Pass=409 Fail=1 Skip=13`; the sole failure remains
  the existing OWN-3 unsupported boundary.

Every task must resolve these identities before using them. A changed real
consumer is permitted only when its task states the exact behavior-preserving
reason and replays the frozen output, error, cleanup, and status oracles.
Protected conformance and canonical compliance machinery use their separate
approval gate.

## Workstream 8a — bounded proof-feasibility evidence

Stage 8a is a removable experiment, not a language or compiler installation.
It first freezes the baseline, then may split into parallel bit-bound and
counted-append probes, followed by one synthesis task.

### Local facts

For `read_bits`, use a temporary in-crate harness over production checked and
entailment representations to test exactly these two unsigned fact sources:

- `result = iand(left, right)` establishes `result <= left` and
  `result <= right` only for the corresponding admitted term or checked
  constant; and
- `high = ishl.wrap(one, count)` establishes `high != 0` only when `one` is a
  checked unsigned constant with mathematical value one.

Together with existing closure and the existing `mask = high -wrap 1` fact,
measure whether every `Ok(value:)` return of `read_bits` establishes
`value < high`; `Err` establishes no result fact. Cover unsigned widths,
signed and operation near misses, boundary counts, operand order, and
per-support mutation kills. In particular, the real later mutation of
`state.hold` must preserve the bound through `mask`, while mutation of a fact's
own supported term kills only that fact.

For `append_slice`, use the truthful admitted domain expressed by this exact
existing-language requirement in the proof-only scratch helper:

```whitefoot
requires {
  let capacity = len(deref(destination));
  let admitted = ile(filled, capacity);
  check admitted else trap "append filled exceeds destination";
}
```

Rewrite only the scratch/runtime variant to the existing counted form
`for @append at in filled..capacity`. An early return uses the counted body
fact `at < capacity`; exhaustion returns `capacity` directly. Do not invent a
post-loop binder equality or a variable-subtraction fact. Exhaustively compare
result and every written byte with the current body for capacities and text
lengths `0..=8`, all admitted filled values, two destination fills, and zero,
maximum, and ascending text patterns. Preserve the wfgrep `9/9` and
raw-DEFLATE `3/3` program oracles.

### Caller audit

Before hypothetical facts, reproduce that v0.27 proves neither local result
goal and that unconditional `append_slice result <= capacity` is false for
`filled > capacity && len(text) = 0`.

Inventory all fourteen real `read_bits` calls and all twenty real
`append_slice` calls. With only the hypothetical local result fact, classify
each future caller requirement as `discharged` or `unproved`; an unexpected
refutation is a blocker. The audit must reproduce at least these known gaps:

- all fourteen `read_bits` `Ok` payloads are assigned with `set` into an outer
  binding, while current ENT-3 transfers no fact through that assignment and
  the inner support expires; and
- in wfgrep, `length <= len(deref(report))` survives the host-copy element
  write, but `set length = length +wrap copied` kills the scalar relation and
  current S7 creates no variable-offset replacement.

Stage 8a leaves no tracked compiler, source, specification, conformance,
generated, or MCTS change. It appends measured evidence to the existing
acceptance record only after temporary files are removed and host hashes are
restored. Local success is not a claim that either complete caller sequence is
feasible.

### Stage 8a stop

Stop with the smallest reproducer if either local goal needs general loop
induction or a loop fixed point, arithmetic-expression terms, Boolean
decomposition, a solver, source/function recognition, invariant or
postcondition syntax, an unproved premise, a third fact-source family,
invalid-domain behavior changes, or an incomplete or ill-typed caller map.
Do not start Stage 8b after a stopped result.

## Trust prerequisite — bounded existing-DIAG-2 repair

Independently of the Stage 8a outcome, repair a present implementation gap
against active DIAG-2. The active specification already requires the checked
program to retain the exact ENT-4 derivation for every accepted subscript,
every discharged ordinary-call goal, and every S11 fact of every counted
statement, including S11 facts not used by a later query. This work may run in
parallel with Stage 8a, remains required if Stage 8a stops, and must be terminal
before Stage 8b.

Fold in only task 0049's bounded Stage 1 result. While the canonical entailment
engine performs its existing closure and flow work, record canonical parents
and exact roots for that complete required set. Cover source identity,
invalidation, joins, contradiction, ordinary and counted loops, substitutions,
generics, recursion, and root completeness. Record parents during the existing
pass; do not rerun closure to reconstruct a proof. Measure parent nodes, bytes,
proof depth, compile time, and peak memory on the frozen programs.

This is an implementation of already-active semantics, not a new proof system.
It changes neither accepted source nor runtime behavior and grants no new
lowering authority. It creates no certificate format, portable identity,
serialized artifact, ProofFlow, shadow verifier, or second semantic walk. If
the complete required derivations cannot be represented compactly and exactly
without one of those expansions, stop with the smallest missing case. Ordinary
compiler tests are autonomous; any protected conformance change enters the
separate approval gate.

## Workstream 8b — verified normal-return postconditions

Stage 8b begins only after Stage 8a establishes both local witnesses, every
caller gap has the fixed disposition below, and the trust prerequisite
correctly retains the complete existing derivation set. General arithmetic,
induction, solvers, recognizers, trusted writer assertions, general `set`
equality or arbitrary RHS fact transfer, and variable-offset S7 are outside
the plan. The one result-specific selected-outcome receiver event enumerated
below is not general assignment authority. Every new postcondition or
operation-derived fact extends the same exact derivation representation rather
than creating a parallel proof channel.

Prepare the smallest normal-return postcondition design and implementation:

- install only the measured bit-operation fact sources needed by `read_bits`;
- adopt the truthful counted `append_slice` body and admitted-domain
  requirement rather than the refuted unconditional contract;
- define verified normal-return postconditions, including result/outcome
  selection, body proof at every normal return, formal and result
  substitution, support and kills, branch and early-return behavior, cleanup,
  concrete generic instantiation, false-postcondition rejection, unsupported
  boundaries, deterministic diagnostics, and caller fact establishment;
- retain the fourteen existing direct-call match shapes and add one bounded
  selected-outcome receiver event. At the selected direct
  `Ok(value: payload)` arm of a match whose scrutinee is one accepted ordinary
  user call, and whose entry holds exactly the instantiated planned-S12
  verified normal-result relation, a direct non-consuming
  `set outer = payload;` may carry that relation only when `payload` is the bare
  own fragment binder, `outer` is a previously live bare outer own binding of
  the exact same type, and no intervening event occurs. Evaluate the RHS, apply
  the ordinary target commit and kill, then replace only the result-payload
  occurrences in that relation with the post-write receiver and re-establish
  the relation in the same complete/U/B view when every non-payload support
  remains live. The receiver may not be any actual argument of that call;
  within that instantiated relation it may not appear as a non-result term or
  support, nor overlap another substituted support. This check is
  relation-local: unrelated pre-existing receiver facts do not block the event
  and are removed by the ordinary target kill. A projected, computed, or
  consuming RHS; wrong or unselected binder; differently typed receiver;
  named/pending outcome; extra reaching write; alias; missing relation; or
  killed support establishes nothing. Payload scope exit and the ordinary
  match join then apply normally. This event establishes neither
  `outer = payload` nor any unrelated fact and is not general `set` equality;
- retain bounded immutable delivery for `value_if` only. On each reaching
  `give` edge, for each eligible typed L0 relation replace every occurrence of
  the delivered bare copy atom with the receiving binding, apply ordinary
  scope kills to every other support, and pass the surviving edge relations
  through the ordinary L0 join. Establish only each joined typed relation on the
  fresh receiving binding, supported by it plus the exact surviving support.
  Perform this independently within the same complete/U/B view; no relation or
  edge evidence crosses views. A non-atom or consuming delivery, a missing or
  ill-typed edge relation, or a join with no common relation establishes
  nothing. This remains delivery-specific fact substitution, not assignment
  equality, and no Stage 8b consumer relies on a corresponding `value_match`
  rule;
- repair wfgrep's post-copy transition after the child region ends: retain the
  prior length and the copied count in outer scalars, form the candidate next
  length, then take an explicit value-producing branch on
  `candidate_length <= len(deref(report))`. Give the candidate only on the
  established true edge and otherwise give the prior length exactly as the
  existing failed-copy path leaves it unchanged. Use the resulting
  `bounded_length` for every post-selection length use: as both receiver and
  `filled` actual for the separator and all later append calls, and as the final
  length passed to `publish_all`. Do not write it back through the old `length`
  binding, add variable-addition entailment, or add a trusted system-result
  arithmetic premise; and
- exercise the complete raw-DEFLATE and wfgrep call sequences through the one
  ordinary compiler path with unchanged output, errors, cleanup, effects, and
  required runtime checks.

This workstream is expected to change language semantics and protected
conformance. Candidate design, implementation, ordinary tests, real-program
repair, measurement, and independent review may proceed autonomously on task
branches. Before any protected bytes land, present one combined owner packet
with the exact full specification SHA/diff/impact/verifier results and the
exact conformance before/after inventory, then hard-wait. Changed bytes or
scope require renewed explanation and approval. Activation is atomic with the
outgoing archive, digest chain, compiler, approved corpus, docs, and real
consumer migration.

Stage 8b is terminal only when every mapped real caller sequence derives its
intended fact. An unresolved or refuted mapping, a hidden runtime fallback, or
a changed error, cleanup, effect, or output contract is a stop.

## Workstream 9a — deterministic claim ledger

Run this workstream after Stage 8b so it describes the final fact sources,
postcondition edges, and remaining claim population rather than a transitional
compiler.

Expose a deterministic read-only claim ledger from the checked program. Reuse
the trust prerequisite's exact derivation parents and retain only the additional
observational used-premise links actually needed to connect a named claim to a
reported obligation and provenance. For every remaining named claim the report
includes stable source identity, name and predicate, justification, lifecycle
disposition, the obligation or obligations it supports, the used derivation,
and provenance. Ordering and counts must be identical across clean builds and
complete for UTF-8, raw-DEFLATE, and wfgrep. Synthetic retained, redundant,
refuted, kill, join, call, and loop controls challenge the mapping.

This is ordinary compiler/tooling work and changes neither language acceptance
nor lowering authority. It creates no serialized artifact, portable identity,
replay protocol, second semantic engine, optimizer fact, or compliance
baseline. If exact completeness requires re-analysis per claim, a second
closure, guessed support, or a durable identity framework, stop and report the
smallest gap. Any protected compliance change still enters its own owner gate.

## Workstream 9b — opt-in strict no-claim partition

Stage 9b begins only after Stage 9a deterministically enumerates the actual
remaining claims and their transitive support.

Design and implement the smallest opt-in `deny-claims` partition semantics.
The marker is carried by a function declaration; its exact spelling is left to
the later specification candidate. Each marked concrete function instance is
one strict root. Its outgoing transitive closure contains every concrete user
function instance reached by an ordinary call, each recursive strongly
connected component, and the generated entry wrapper or adapter when the root
has one. The closure does not flow upward into unrelated callers.

One function may therefore be used by both ordinary and strict callers. It has
one runtime body and ordinary acceptance remains unchanged; strict membership
is an additional unasserted semantic rewalk and finite summary, not a second
lowering or specialization. A call from outside into a marked root must prove
that root's requirement in the caller's unasserted state. Calls within the
strict closure are likewise judged unasserted, and their callees must have a
successful strict summary.

The closure's complete transitive claim ledger must be empty. Every checked
direct claim counts, including one in a structurally unreachable arm. A finite
may-claim fixed point propagates through ordinary calls, recursion, concrete
generic instances, and generated boundaries. A direct claim rejects at its
claim node; the first source-ordered call that imports a may-claim summary
rejects at that call node. The exact rule id and payload are fixed by the later
specification candidate, but those two ownership locations may not change.

All protected obligations and ordinary required calls inside the partition are
judged in the unasserted state: a body `check` may remain as an executed check
for an unrelated invariant, but neither it nor a claim grants authorization.
A real branch, proved requirement, verified normal-return fact, structural fact,
or other non-asserted source may still discharge. Outside the opt-in partition,
the existing claim lifecycle and runtime checks remain unchanged. This is not
a global claim ban, a universal sole-trap-source law, or permission to erase
explicit runtime checks.

Prepare direct and transitive positive, negative, near-miss, recursion,
adapter, generic, and bypass evidence, plus claim-free acceptance,
claim-bearing rejection, and value-branch repair on a real path. This
workstream is expected to require a second exact specification and protected
conformance packet. Candidate work is autonomous; landing waits for the
explanation, exact candidate SHA/diff/impact/verifier results, exact corpus
before/after audit, and explicit owner approval.

Stop if the partition cannot be finite and deterministic without a generalized
effect/proof framework, if a transitive bypass remains, or if ordinary code's
existing acceptance or required runtime behavior changes.

## Cross-workstream invariants

- The active specification remains sole language authority; tests and plans do
  not select semantics.
- There is one normal compiler path with no project-, function-, source-,
  corpus-, or test-shaped exception.
- No writer statement becomes trusted unchecked. A required runtime check is
  removed only by the exact machine proof authorized by the active spec.
- Expected external failure remains a value path. Claims remain named executed
  runtime backstops for internal invariants outside an opt-in strict partition.
- Real-program success, error, cleanup, ownership, effect, and byte oracles are
  preserved unless a later material plan revision explicitly says otherwise.
- Specification and protected-conformance changes always use their separate
  explanation-first approval workflows; this plan never preapproves their
  bytes.
- Bounded subordinate compiler defects, diagnostics, probes, ordinary tests,
  docs, integration, and closure may proceed autonomously when they preserve
  this strategy and these boundaries.

## Approval checkpoints

The revised successful path has four owner decisions, not one decision per
task:

1. **complete (2026-08-12):** approve this high-level plan once;
2. **complete (2026-08-14):** approve this one material Stage 8b delivery
   correction;
3. after task 0058 re-freezes and independently reviews the exact candidate,
   approve Stage 8b's exact specification plus protected-conformance
   activation packet; and
4. after Stage 9a, approve Stage 9b's exact specification plus
   protected-conformance activation packet.

Stage 8a, the existing-DIAG-2 repair, Stage 9a, ordinary implementation and
tests, real-source repair, documentation, integration, task lifecycle, and
closure require no further owner approval while they stay inside this plan.
Splitting an exact protected batch creates another protected checkpoint; a
material strategy change creates a revised high-level-plan checkpoint.

## Explicit exclusions

Task 0049's complete shadow authorization inventory,
`EntailmentApprovedProgram` lowering capability, ProofFlow extraction, shadow
or independent verifier, capability-issuer replacement, and closure
performance work are separate architectural directions and are not required
by this plan; only its bounded Stage 1 parent-retention repair is included.
Also excluded are serialized proof artifacts, cache/replay,
portable proof identities, SMT authority, general theorem or arithmetic
frameworks, general loop induction, O11 Boolean composition, language-wide
claim removal, Stage 9 performance work, and new wfgrep optimization.

## Verification and terminal acceptance

Each task runs focused checks and the complete gates owed by its touch set.
Each installed language stage additionally verifies archive identity, chained
activation digests, grammar and generated data, exact protected differentials,
independent adapter totals, and the same frozen real-program oracles.

The successful terminal boundary requires:

- Stage 8a's two local witnesses, hostile controls, and complete 14/20 caller
  inventories;
- the trust prerequisite's complete exact derivations for every accepted subscript,
  discharged call goal, and S11 fact, with bounded measured cost and no second
  authority;
- Stage 8b's verified postconditions working through every mapped real caller,
  with no unresolved prerequisite or behavior drift;
- Stage 9a's complete deterministic claim ledger, with bounded overhead and no
  second authority;
- Stage 9b's finite transitive strict-partition semantics with no bypass and no
  change to ordinary claim behavior;
- positive, negative, near-miss, invalidation, recursion, generic, entry, and
  facts-on/off evidence appropriate to each installed capability;
- the complete repository gate and independent conformance report green or
  carrying only explicitly recorded unchanged unsupported boundaries; and
- roadmap, compiler documentation, canonical acceptance evidence, durable
  design memory where a real decision changed, and all task records reconciled
  once at closure.

Remaining ordinary claims are reported honestly; terminal does not mean every
claim has disappeared or that claim is the language's sole possible trap.
After closure, `PROOF-8` is terminal and the existing `CAND-8` wfgrep checkpoint
is unparked. Beginning the next wfgrep undertaking requires a new high-level
plan rather than extending this one silently.

An honest Stage 8a prerequisite failure is an earlier terminal stopped result
after the independent DIAG-2 trust repair has completed. Any later blocker that
requires a material change to this objective, strategy, principal consumer
boundary, acceptance, risk, or stop conditions has the same disposition. The
lead records the evidence and returns a revised `PROPOSED` plan instead of
weakening the gate or accumulating side tasks around it. A stopped outcome
does not mark `PROOF-8` complete or unpark `CAND-8`.

## Active authority

The owner approved this plan's predecessor on 2026-08-12 and explicitly
selected the narrow receiver strategy written above on 2026-08-14. This
`ACTIVE` status is execution authority only inside the exact written boundary.
The lead first refreshes task 0058's authority, exact semantic questions, and
scope to this plan, after which 0058 may resume within the exact narrow receiver
boundary above and ordinary decomposition proceeds autonomously. A further
material strategy change returns the plan to `PROPOSED`. Exact specification
and protected-conformance bytes still stop later at their own independent
approval boundary.
