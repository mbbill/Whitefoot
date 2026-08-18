# Check dissolution — v0.32 candidate delta (#47)

Status: executor delta document for batch 0071 (E3). Input to the lead's
one v0.32 candidate; nothing here edits `spec/kernel-spec.md`. Baseline is
ACTIVE v0.31, SHA-256
`ea4b8ad4a56fbf43f3c98b91fc667da0b693c75b81807250a36454e03a197f1c`.

Owner doctrine (plan W2, #47): `claim` becomes the sole writer-stated trap
construct; the OP-5 `check` statement retires from the language.

## 1. The exact v0.31 check-vs-claim semantic delta

Measured against the active specification bytes, not the batch brief. One
premise in the tasking rationale is false against v0.31 and is corrected
here: **neither construct is ever elided.** A redundant claim still
executes ([CLM-2]: "the check still executes [CLM-1]") and both carry the
always-`retained` disposition ([DIAG-2]: "An explicit body [OP-5] check
and every [CLM-1] claim are always `retained`"). There is no proven-site
claim elision in v0.31, so dissolution removes no runtime branch and adds
none: a migrated program's executed check set is byte-for-byte the same
branch set. The real deltas are lifecycle, accountability, and record
surface:

| Axis | `check e else trap "msg";` (OP-5) | `claim n: e because "txt";` (CLM-1/2) |
| --- | --- | --- |
| Condition judgment | own `Bool` exact value mode, TYPE-7 exclusivity (OP-5) | identical, by reference to the OP-5 judgment (CLM-1) |
| Runtime retention | retained in all build modes, never elided | identical (`retained` disposition, never elided) |
| Trap record [DIAG-3] | `rule_id` `OP-5`, `message` = STRING decoded by FORM-5 | `rule_id` `CLM-1`, `message` = claim IDENT spelling; `because` STRING never reaches runtime |
| Fact establishment [ENT-3] | S2: positive goal-origin set + comparison-origin projection on the normal continuation | S3: "establishment is exactly [ENT-3.S2]'s" — textually identical |
| Signed decomposition | attaches at S2 establishment | attaches at S3 establishment, same rule |
| Kill/join/scope behavior [ENT-5] | ordinary signed-goal support | identical |
| Unasserted U view | S2 blinded | S3 blinded — identical |
| Redundancy (predicate already derivable) | nothing; silently accepted | non-rejecting advisory naming the claim |
| Refutation (exact negation derivable) | nothing; accepted, traps whenever reached | hard error citing CLM-2 — compile-time rejection |
| Identity | anonymous | IDENT name, unique per `fn_decl`, claim-name DIAG-1 carrier |
| Justification | none | mandatory `because` STRING, compile-time review data retained by the checked program |
| Accountability | absent from the ClaimLedger | ClaimLedger entry; deterministic accountability projection |
| CLM-3 `deny_claims` strict closure | a body check inside a demanded closure is **legal** (only its S2 fact is blinded) | a claim inside a demanded closure **rejects** (`MayClaims` nonempty) |
| Effect row [EFF-2] | contributes `traps` | identical — contributes `traps` |
| Delivery/divergence [GIVE-1] | normally continuing; not delivery, not must-divergence (spec line: "A `check`, `claim`, or call that may trap...") | identical |
| `requires`/`ensures` blocks | the **final `check_stmt`** is the contract form, owned by FN-8/FN-9, not by the body statement | a claim is illegal in a `requires` block (FN-8 structural pass) |
| External constrained subject [PRV-3, P12] | not a repair | not a repair — identical |

Answering the four tasked questions directly:

- **Retention.** A fragment-proven (redundant) check and a fragment-proven
  claim are both still executed at runtime in every build mode. No elision
  exists on either side in v0.31; dissolution changes zero executed
  branches for a check that migrates to a claim.
- **Refutation.** A refuted claim is a hard CLM-2 error. A check whose
  exact negation the closed state derives is **accepted today** and traps
  whenever it executes. Dissolution therefore converts that (defective)
  program class from a guaranteed runtime abort into a compile-time
  rejection. For a check that any execution can pass, refutation cannot
  fire: the fragment is sound, so a derivable negation and a passing
  execution are mutually exclusive.
- **Fact strength.** S3 establishment is definitionally S2's: same
  positive sign, same goal-origin set, same comparison-origin projection,
  same signed decomposition attachment, same ENT-5 support/kill/join
  lifetime, same blinding in the unasserted view. There is no S2
  establishment S1/S3 cannot cover; the stop condition is not triggered.
- **Effect rows.** [EFF-2] lists `check` and `claim` in the same
  body-syntactic `traps` clause. A migrated function's exhibited and
  declared rows are unchanged. Retiring check removes the word `check`
  from EFF-2's text; no program's `traps` annotation changes.

## 2. Selected specification delta

### 2.1 Grammar (GRAM-4, GRAM-2 fence)

Remove `check_stmt` from the `stmt` alternation in the GRAM-4 fence:

```
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | for_stmt | break_stmt | region_stmt | claim_stmt
             | if_stmt | match_stmt | give_stmt
```

Keep the `check_stmt` production itself — it remains the mandatory final
form of `requires_block` and `ensures_block` — and admit it directly at
the contract entries in the GRAM-2 fence:

```
requires_entry:= doc | stmt | check_stmt
ensures_entry := doc | stmt | check_stmt
```

Consequences:

- The `check` fixed terminal, the `check_stmt` production, and the
  production count are all retained; the token inventory and terminal
  predicates are unchanged. The accepted byte set only narrows: a body
  `check` no longer parses (GRAM-4 statement-selection parse rejection at
  the former statement position).
- FN-8's and FN-9's early structural passes are textually unchanged
  ("zero or more ordinary lets followed by exactly one final
  `check_stmt`"); their admission set is unchanged because the contract
  entries still admit every `doc | stmt` plus the final check.
- The alternative — retiring `check_stmt` entirely and respelling the
  contract final — was rejected: the `requires { let* check }` spelling
  is a separately tracked R3-provisional surface (constitution audit
  2026-07-05) whose comparison is not this delta's mandate, and a
  respelling would churn every requirement-bearing program for zero
  semantic gain.

### 2.2 OP-5 refit (statement retirement, judgment retention)

OP-5 loses its body-statement clauses ("A conforming check in a function
body is a runtime check in all build modes...", the `False()`/`True()`
execution sentences, and the body-position hard-error clause) and retains
exactly:

1. the condition judgment — exact value mode, own `Bool`, TYPE-7
   implicit-read exclusivity — which CLM-1, FN-8, and FN-9 reference;
2. the contract-final ownership sentences (requires final owned by FN-8;
   ensures final owned by FN-9);
3. the program-start dynamic-boundary trap semantics referenced by
   PROG-3, GATE-1, and CLM-3.

Rule inventory count is unchanged; no rule is deleted. The program-start
trap record keeps `rule_id` `OP-5` and its `message` (the final
`check_stmt` STRING, FORM-5-decoded), so entry-requirement failure
records are byte-identical across the version. The alternative — folding
the judgment into CLM-1 and re-attributing the program-start record to
FN-8 — was rejected because it changes trap-record bytes for unmigrated
behavior and inverts the CLM-1-cites-OP-5 reference direction for no
writer-visible gain.

### 2.3 ENT-3: S2 retirement

- Delete `[ENT-3.S2]`. Re-home its establishment text into `[ENT-3.S3]`
  so S3 is self-contained: "After `claim n: e because "…";` [CLM-1], each
  goal in `e`'s goal-origin set is established with positive sign on the
  normal continuation; when `e` also has comparison origin R, R is
  established there independently."
- The signed-decomposition preamble ("When a source establishes `+G` or
  `-G`...") is source-generic and survives verbatim; decomposition
  attachment at S1 both-sign edges, S3 claims, and S4 requirement
  transfer is unchanged. (Recorded analysis, goal-decomposition node:
  members formerly established through a body check migrate to S1
  two-sided branch edges or S3 claims.)
- The unasserted-state sentence becomes "The unasserted state removes
  exactly S3 claim establishment." and the following sentence drops
  "`check` or"; the S-source enumeration lists lose S2.
- ENT-1's SCOPE-2 recap keeps "an executed runtime check" as a fact
  source description — the claim is that executed runtime check.

### 2.4 CLM rules

- CLM-1: unchanged semantics; its OP-5 condition-judgment reference now
  points at the retained judgment clause. The sentence fragment "exactly
  as today's `check` on the same expression" in CLM-2 is rewritten to
  state the trap behavior directly (there is no `check` to compare with).
- CLM-3: "it neither removes nor changes any [CLM-1] claim or [OP-5]
  check" drops the check clause; both "a body `check` or `claim` is not a
  strict repair" occurrences (OP-4 strict clause, FN-8 strict clause)
  become "a claim is not a strict repair".
- Behavior sharpening, stated: under v0.31 a body check inside a demanded
  `deny_claims` closure is legal; after dissolution every writer-stated
  trap in such a closure is a claim and rejects. `deny_claims` now means
  literally "no writer assertion in the demanded closure" instead of "no
  *named* writer assertion". Migration leg B (below) covers the affected
  sites; the only in-repo strict root (`wfgrep.wf` `report_failure`) has
  zero body checks in its closure, so no ordinary surface changes
  acceptance.

### 2.5 DIAG rules

- DIAG-3 `node_path` clause: drop "the `check_stmt` for [OP-5]," keeping
  "the final `check_stmt` whose complete goal fails at program start
  [FN-8, PROG-3]" under `rule_id` `OP-5`; the claim and operation-call
  clauses are unchanged.
- DIAG-3 message table: drop the "explicit [OP-5] body check" row; the
  program-start row stays.
- DIAG-2: "An explicit body [OP-5] check and every [CLM-1] claim are
  always `retained`" loses its check clause; the contract-final sentences
  are unchanged.
- DIAG-1: no carrier change (the check message STRING was never a
  carrier; the claim-name carrier already exists). The FORM-1/lexical
  surface is unchanged.
- Prose inventory line "STRING appears only in `doc` entries, `check`
  messages, and `claim` justifications" is rewritten to "`doc` entries,
  contract final `check` messages, and `claim` justifications".

### 2.6 EFF-2

The body-syntactic contribution list drops `check` (claim remains):
"... a `.trap` OPNAME — `claim`, or a call to any operation or function
whose effect row includes `traps` ...". Same for the two later
occurrences ("An explicit body `check` or `claim` still contributes
`traps`" → claim only; "no call, no `check`, no `claim`" → "no call, no
`claim`"). No program's exhibited or declared row changes: every removed
`check` occurrence is replaced by a `claim` occurrence with the same
`traps` contribution, and contract finals contribute no effect in either
version.

### 2.7 EX-1 worked example (normative bytes)

The example contains one body check:
`check ieq(v, 42_i32) else trap "arithmetic drift";`. It becomes, per the
recipe: `claim arithmetic_drift: ieq(v, 42_i32) because "arithmetic
drift";` — a normative byte change the candidate must carry, with the
example's prose updated from check to claim.

### 2.8 Inventory arithmetic (for the candidate status line)

Rules +0/-0 (OP-5 refitted, not deleted); tokens +0/-0; fixed terminals
+0/-0; grammar productions +0/-0 (three right-hand sides change: `stmt`,
`requires_entry`, `ensures_entry`); operation-table rows +0/-0. Accepted
byte set: narrows only (body `check` statements stop parsing); no new
spelling is introduced.

## 3. Writer migration recipe

Leg A — ordinary body check (the default):

```
check E else trap "MSG";
=>  claim NAME: E because "MSG";
```

- `NAME` is the MSG slug: ASCII-lowercased, every non-`[a-z0-9]` run
  collapsed to one `_`, trimmed; prefixed `c_` when it would start with a
  digit or underscore; suffixed `_2`, `_3`, ... on collision with any
  existing or generated claim name in the same `fn_decl`; replaced by a
  non-terminal spelling when it collides with a fixed terminal (IDENT
  excludes the fixed lowercase grammar atoms).
- The original message is preserved verbatim as the `because`
  justification; a human or AI writer should later strengthen it into a
  genuine invariant argument. This mechanical form is honest: it records
  exactly what the check recorded.

Leg B — body check inside a `deny_claims` demanded closure: a claim would
reject under CLM-3. Apply CLM-3's mechanical repair instead — a
dominating real branch whose false edge takes the domain outcome
(establishment moves to S1's two-sided edges) — or drop the strict
marker. Never migrate such a check to a claim silently.

Leg C — the final `check_stmt` of a `requires` or `ensures` block: not a
body check; keep unchanged.

### Stated behavior deltas under leg A (never silent)

1. **Failure record bytes.** A failing migrated site emits
   `"rule":"CLM-1","message":"NAME"` instead of
   `"rule":"OP-5","message":"MSG"`, at the `claim_stmt` node path instead
   of the `check_stmt` node path. Passing executions are unobservable
   either way.
2. **Redundancy.** A migrated check whose predicate the closed state
   already derives now draws one non-rejecting advisory. Acceptance is
   unchanged.
3. **Refutation.** A migrated check whose predicate's exact negation the
   closed non-contradictory state derives becomes a compile-time CLM-2
   rejection. Such a check could never pass at runtime, so this converts
   a guaranteed abort into a rejection; no check that can execute
   successfully is affected.
4. **Strict closures.** See leg B: presence inside a demanded
   `deny_claims` closure flips from legal to rejecting.
5. **Accountability.** Every migrated site enters the ClaimLedger and the
   deterministic claim-accountability projection; ledger populations grow
   accordingly (e.g. the frozen real-source populations move from
   2/12/8 to include every former body check in those programs).
6. **Name surface.** The generated IDENT joins the claim-name carrier
   and the per-`fn_decl` uniqueness domain. It is not a declaration and
   collides with nothing else.

Acceptance and discharge preservation argument: S3 establishment is
textually S2's (section 1), decomposition and kills are shared, the U
view blinds both equally, effect rows are identical, and delivery and
divergence judgments treat both constructs alike; therefore every
accepted v0.31 program whose body checks migrate under leg A (outside
deltas 3 and 4, which are enumerated defect classes) is accepted with
identical obligation discharge, identical facts-on/facts-off behavior,
and an identical executed-branch set. Deltas 3 and 4 are the only
acceptance flips, both narrowing, both compile-time, both listed.

## 4. patterns.md P12 replacement text (draft — patterns.md is not edited here)

> Problem: a protected storage access uses an offset derived from process
> or system input, so valid hostile input may falsify its bound. Pattern
> status: active v0.32 guidance. Test the relation with a real branch and
> return the domain's normal error value on the false edge. A `claim`, an
> ordinary callee requirement/prologue, or a process-entry wrapper check
> is not a repair: each turns expected external failure into a trap.
>
> (Second paragraph unchanged except: "An internal constrained subject
> may still use an honest invariant `claim` under its ordinary
> lifecycle." — already claim-only; the two remaining `check` mentions in
> the first paragraph fold into `claim` as above.)

## 5. Implementation state behind the switch

`CHECK_DISSOLUTION` (`compiler/src/semantic/check.rs`), default `false`,
plumbed like the shipped dissolution switches, with the test-only entry
`check_semantics_check_dissolution`:

- **Off (shipped v0.31 path):** byte-identical behavior; the gate stays
  green with no verdict change.
- **On (v0.32 candidate readiness):** a body-position `check_stmt` is
  rejected at its node — the semantic modeling of the grammar removal the
  candidate performs at parse time; contract finals (requires/ensures)
  are untouched because they never pass through the body statement path.
- Coverage added regardless of the switch: claims-only (no body check
  anywhere) programs exercising S3 establishment with signed Boolean
  decomposition and S1 two-sided branch decomposition, proving
  goal-decomposition attachment does not depend on S2's existence.

Not implementable now without changing v0.31 acceptance, deliberately
deferred to the candidate's activation batch: the actual GRAM-4/GRAM-2
table change (generated grammar identity is pinned to the active spec
hash), EX-1 byte refresh, EFF-2/DIAG prose, and the protected conformance
family (lead-owned).

## 6. Ordinary-surface migration and conformance inventory

The ordinary migration (tests/programs and compiler fixtures) and the
list-only protected inventory live beside this document:

- migration results: recorded in the batch report; every migrated file
  compiles and its program tests pass with unchanged output.
- `conformance-inventory.md`: every `tests/conformance/` case and
  manifest row touching `check`, each with a proposed disposition for the
  lead's protected candidate. No conformance byte is edited by this
  batch task.

Compiler-fixture surfaces deliberately kept on `check` (they test the
v0.31 language definition itself and flip only with the candidate's
activation): OP-5/S2/DIAG behavior tests, canonical-formatting and
parser/grammar tests of the check syntax, CLM-3 strict tests that assert
the v0.31 check/claim asymmetry, the v0.22→v0.23 migrate tool's frozen
corpus, and every requires/ensures contract final. These are enumerated
in the batch report.
