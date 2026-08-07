# ENT-5 loop-rule fix — specification-change candidate

Status: CANDIDATE, OWNER-APPROVED 2026-08-07 (recorded in
`governance/APPROVALS.md`), ready for activation. This document is the complete
delta against the exact text of the active `spec/kernel-spec-v0.22.md`
(installed 8f91ede; SHA-256
`b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8`; roadmap
revision 18). Approval covers these exact bytes; a changed byte returns to
review.

This candidate was split out of `semantics-v024-candidate.md` (drafted and
withdrawn 2026-08-07) on the lead's ruling. Its companion — the
subject-position taint gate — is `provenance-gate-candidate.md`, HELD FOR
MEASUREMENT. The two share no rule text; nothing here depends on that one.

**Numbering.** The file name reserves v0.24 provisionally. This batch is
approved while the FLOOR-5 spelling batch (`spelling-relief-candidate.md`,
planned task 0036) is still a draft with open questions, so this may well
activate first and take v0.23. Per `docs/WORKFLOW.md` step 2 the activation
task takes the next free number and stops for an owner choice if the canonical
path is occupied; it does not skip a version to avoid the choice. §7 states the
ordering interaction with FLOOR-5 in full: it is nil for this batch.

## 1. What the rule does today, and why it is wrong

[ENT-5]'s loop rule removes, at each iteration head, every fact having a
support member that any kill event (a)–(d) occurring **anywhere** inside the
loop body may kill. Kill event (d) is an edge leaving the lexical scope of any
support binding. A `return` edge leaves the scope of every binding in the
function, so a single `return` anywhere inside a loop body discards every fact
established before that loop — including `requires` axioms and
allocation-length equalities that no execution can invalidate.

`research/investigations/obligation-discharge/ACCEPTANCE.md` (2026-08-07)
isolates this as the dominant cause of the deflate divergence: 5 of 29 sites
proven against 17 of 30 predicted, a drop from 57% to 17%, and 21 claims where
about 8 were expected. The minimal witness moves one statement and changes
nothing else:

- `D1h`: a loop indexing a const table, with one early `return` in the body —
  `ordered_symbol < len(code_lengths)` **undischarged**.
- `D1i`: the identical `return`, hoisted just outside the loop —
  **discharged**.

The reach is every deflate function that has a loop and returns inside it:
`read_bits`, `decode_fixed_symbol`, `decode_fixed`, `inflate`,
`copy_distance`, `build_huffman_table`, `decode_table_symbol`,
`decode_dynamic`. sha256's three loops contain no `return`, which is why
sha256 landed on its predicted buckets. `break` alone does not trigger it
(`D1b`, `D1c` both discharge with a `break` in the loop).

The current behaviour is spec-conformant on a literal reading. It is
avoidable conservatism: a `return`, a `break`, or a `propagate` error edge
never reaches the next iteration head, so counting its scope-exit kills in the
loop-head state removes facts no execution can observe as false there.

## 2. Proposed version-header paragraph

> Status: REVIEW CANDIDATE vNEXT (2026-08-07; the [ENT-5] loop-rule scope-exit
> fix). Restates [ENT-5]'s loop rule so that the kill scan at a loop head
> considers exactly the kill events an execution can carry into a later
> iteration head of the same loop: an event inside the body is scanned when
> some path of the conservative structural normal-control graph [FN-1] leads
> from its edge back to that loop's body entry without leaving the body, and
> the events reachable only through a `break` naming that loop or an enclosing
> one, a `return`, or a `propagate` error edge are not scanned, because no
> later iteration head observes them. Today a single `return` anywhere in a
> body discards every pre-loop fact at the head, including `requires` axioms
> and allocation-length equalities that no execution can invalidate; the
> corrected scan removes only kills, so the fact state at every loop head is a
> superset of the one this version computes and discharge only widens.
> Specification delta: numbered rules +0/-0; one existing rule modified at one
> verbatim-anchored modification site: ENT-5 (the loop rule, its final
> paragraph). Tokens +0/-0; terminal spellings +0/-0; grammar productions
> +0/-0; operation-table rows +0/-0; source constructs +0/-0; sections +0. No
> rule gains an exception clause [META-3]: the scan condition is one total
> positive reachability predicate over the [FN-1] graph. The accepted-program
> set widens by every program whose loop-head facts survive under the
> corrected scan — measured on the deflate unit as the dominant cause of a
> 12-site discharge miss — and narrows on no discharge, redundancy, or [OP-4]
> ground; the sole newly reachable rejection is [CLM-2] refutation of a claim
> a surviving pre-loop fact proves cannot pass, which is [ENT-1]'s already
> enumerated non-monotone edge. Selection ground:
> `research/investigations/obligation-discharge/ACCEPTANCE.md` (2026-08-07),
> with the `D1h`/`D1i` witness pinning the cause to this rule alone. These
> bytes are non-authoritative until the derived-material review,
> full-document hash, exact owner approval, and active-target installation
> complete.

## 3. Grammar delta

None. This batch adds, removes, and reshapes no production, terminal
predicate, token form, operation-table row, or source construct. Its one
modification site is prose at line 1042 of the active spec, outside every
fenced block. §8 records the mechanical confirmation.

## 4. The modification (complete replacement delta, verbatim anchor)

**[ENT-5]** One site: the rule's final paragraph. Verbatim anchor —

> Loops carry no induction in this version: the fact state at the head of each
> iteration of `loop @l { … }` is the state before the loop minus every fact
> having a support member that any kill event (a)–(d) occurring anywhere inside
> the loop body, at any nesting depth, may kill. The surviving facts hold at
> every iteration head; establishment and kills then proceed ordinarily within
> the iteration, and no fact established inside an iteration survives to the
> next iteration's head. Loop induction is a later version's
> [ENT-1]-monotone extension.

becomes —

> Loops carry no induction in this version: the fact state at the head of each
> iteration of `loop @l { … }` is the state before the loop minus every fact
> having a support member that a continuing kill event of `@l` may kill. A kill
> event (a)–(d) placed inside `@l`'s body, at any nesting depth, is continuing
> for `@l` exactly when some path of the conservative structural normal-control
> graph [FN-1] leads from the edge carrying that event to `@l`'s body entry
> without leaving `@l`'s body — that is, exactly when an execution taking that
> edge can reach a later iteration head of the same loop. Every other kill
> event inside the body is not continuing and is not scanned: an event on or
> reachable only through a `break` edge naming `@l` or any enclosing loop, a
> `return` edge, or a `propagate` error edge leaves `@l` for the loop's
> continuation or the function-return sink [FN-1, ERR-3], and no iteration head
> of `@l` is reached from it without first re-entering `@l` from outside, where
> the enclosing flow supplies the state. A kill inside a nested `loop @m` whose
> continuation lies inside `@l`'s body is continuing for `@l`, including the
> kills carried on `@m`'s own `break` edges, because `@l`'s body entry is
> reached from `@m`'s continuation without leaving `@l`. The surviving facts
> hold at every iteration head; establishment and kills then proceed ordinarily
> within the iteration, and no fact established inside an iteration survives to
> the next iteration's head. A fact a non-continuing edge kills is still
> removed on that edge: the continuation join above takes each `break` edge
> after that edge's scope-exit kills, and an edge to the function-return sink
> reaches no queried program point, so narrowing this scan opens no path on
> which a dead fact is read. Loop induction is a later version's
> [ENT-1]-monotone extension.

No other clause changes. The three other places in [ENT-3] and [ENT-5] that
mention kill events — the comparison-origin clause (b), S7's checked-arithmetic
origin, and S10's boundary-count origin — each quantify over *paths from an
initializer to a use*. A `return` or `propagate` error edge lies on no such
path, because nothing follows it, so none of the three is affected by this
change and none is restated.

### 4.1 Why the reachability form, and not the enumerated one

The rule could instead have been written as an enumeration: "the kill events on
`return`, `break`, and `propagate`-error edges are excluded." Both are sound.
The reachability form was drafted and is **lead-accepted (2026-08-07), not
adopted silently**, on three grounds:

1. It states the actual reason — an event is scanned exactly when an execution
   can carry it to a later head — rather than a list of the statement kinds
   that happen to realize that reason today.
2. It is strictly stronger in the right direction. In
   `loop @l { set p = e; return v; }` the `set`'s kill (a) sits on an ordinary
   statement edge, so the enumerated reading scans it, while the reachability
   reading does not, because `@l`'s body entry is unreachable from that edge
   without leaving the body. That execution returns; it observes no later head.
3. It composes with control forms this version does not have. When the FLOOR-5
   batch adds `if_stmt` and `value_if`, an else-free `if`'s false edge stays
   inside the body and is continuing, and `value_if`'s delivery edges route to
   the same three targets `value_match`'s do [FN-1]; the enumerated form would
   need re-enumeration, the reachability form needs no edit.

## 5. Soundness, the join, and monotonicity

### 5.1 Soundness

**Claim.** If a fact `F` holds in the state before `loop @l` and no continuing
kill event of `@l` may kill a support member of `F`, then `F` holds at every
iteration head of `@l`.

**Proof.** By induction on the iteration index.

*Base.* Iteration 1's head state is the state before the loop, in which `F`
holds by hypothesis.

*Step.* Suppose `F` holds at iteration `k`'s head. Take any execution that
reaches iteration `k+1`'s head. On the conservative structural graph its
control traverses a path `π` from `@l`'s body entry to the loop-body normal
exit and thence to the body entry again, and every edge of `π` lies inside
`@l`'s body — a path that left the body would reach `@l`'s continuation or the
function-return sink, and neither reaches `@l`'s body entry without a fresh
entry to the whole `loop_stmt` from the enclosing flow, which is a different
program point whose state the enclosing flow (and, when `@l` is nested, the
enclosing loop's own instance of this rule) supplies. Every edge `e` of `π`
therefore has `@l`'s body entry reachable from it along the remainder of `π`
without leaving the body, so every kill event on `π` is continuing for `@l` by
the definition above. By hypothesis no such event kills a support member of
`F`, and establishment along `π` only adds facts ([ENT-3] sources are additive
and [ENT-4] closure is monotone), so `F` survives `π` and holds at iteration
`k+1`'s head. ∎

The contrapositive is the property that matters: the scan omits an event
exactly when no execution can both take that event's edge and be observed at a
later head of the same loop. Nothing survives that could be false where it is
read. The omission is not "returns are ignored" — a `return` inside a loop
still kills on its own edge, and that edge reaches the function-return sink,
which is not a queried point.

### 5.2 The break edge opens no hole

The concern the change must answer: `break @l` carries scope-exit kills, and if
the loop-head scan stops looking at them, some fact might survive into the
loop's continuation after the binding it depends on has died.

It does not, because the continuation state is computed by a different clause
that this delta does not touch. [ENT-5]'s join paragraph already says: "The
continuation of a `loop_stmt` is the join over the states on its `break` edges,
each likewise taken after its scope-exit kills and closed." Every `break` edge
contributes its own post-kill state, the join keeps only what all of them hold,
and a loop with no `break` naming its label yields the contradictory
all-derivable state. So:

- a fact killed on a `break` edge is absent from that edge's contribution and
  therefore absent from the join;
- a fact killed on a `return` or `propagate` error edge is never read, because
  those edges reach the function-return sink and no queried point follows;
- a fact killed on an edge inside the body that continues is still scanned by
  the modified loop rule, so the iteration head never sees it.

The three exits are exhaustive over the ways control leaves a loop body in
v0.22: `break_stmt` reaching `normal_successor` of its resolved target loop,
`return_stmt` and `propagate_let_rhs`'s `Err` edge reaching the function-return
sink, and `value_match`'s `give`/return/break edges, which [FN-1] routes to
exactly those same targets. There is no `continue` form.

### 5.3 Monotonicity under [ENT-1]

The change only removes kills from one scan, so at every loop head the fact
state is a superset of the state v0.22 computes. [ENT-4]'s closure is monotone
and derivability is upward-closed in the state, so:

- every obligation v0.22 discharges is still discharged, and more are;
- no [OP-4] rejection is newly created on discharge grounds;
- a claim v0.22 accepts as non-redundant may become redundant, which [CLM-2]
  makes a non-rejecting advisory precisely so this direction stays monotone.

The one edge that is not monotone is the one [ENT-1] already enumerates.
[CLM-2] rejects a claim whose exact negation the non-contradictory state
derives. A pre-loop fact that now survives to the loop head can supply that
negation, so a program accepted under v0.22 can be newly rejected as a refuted
claim. This is the lifecycle's single deliberate non-monotone edge, already law
in v0.22 ("Refutation is the lifecycle's one deliberate non-monotone edge"),
and it fires only on a claim proven to trap on every execution reaching it — a
defect found at compile time, which is the outcome the rule exists to produce.
The candidate therefore states the property as: **no program that compiles
today loses acceptance on discharge, redundancy, or any [OP-4] ground; the sole
newly reachable rejection is [CLM-2] refutation of a claim that cannot pass.**
The looser sentence "no program that compiles today can break" is not accurate
and is not used.

Note also that a larger surviving state can be contradictory, and [ENT-4]
already fixes that case: at a contradictory point every obligation discharges
and no claim is refuted, so the refutation edge cannot fire there.

## 6. Expected effect and acceptance criterion

This candidate asserts **no** recovered site count. The honest number is
produced by the corrected checker, not by reading source, and producing it is
compiler work belonging to the activation task.

Activation is closed by re-running the `ACCEPTANCE.md` probe — the test-only
dark checker (`check_semantics_dark`) retaining each function's complete
`FunctionEntailment` summary, with the claim-blinding transform — on the same
three programs at the same denominators, and recording:

1. the new proven / claim-supported split on the deflate unit against the
   5-proven-of-29 baseline (24 sites and 5 proven on the dynamic-path-only
   subset), and on utf8parse (33/22) and sha256 (9/0), which must not regress;
2. the `D1h` / `D1i` pair as the pinned witness, `D1h` now discharging.

Falsifiers: `D1h` still failing to discharge, or any site anywhere regressing
from proven to claim-supported. Either outcome means the drafted rule is not
the rule that fixes the measured cause, and the batch returns to review rather
than being closed green.

Derived material to bring to the new version in the same change: the
compiler's loop-rule implementation and its regression cases, any conformance
case pinning loop-head fact survival, the derivation ledger entry, and
`docs/patterns.md` writer forms that work around the current behaviour by
hoisting a `return` out of a loop. Claims the corrected checker now proves
become [CLM-2] advisories, which is the designed cleanup path, not a corpus
edit obligation; no conformance verdict changes meaning and no protected
expectation is weakened.

## 7. Ordering against the FLOOR-5 spelling batch

**This batch's one anchor is disjoint from FLOOR-5's, verified textually
(2026-08-07).** FLOOR-5's single [ENT-5] site is an *insertion* before the
join paragraph's sentence "The continuation of a `loop_stmt` is the join over
the states on its `break` edges"; this batch *replaces* the rule's final
paragraph. The two do not overlap, and this batch's replacement text contains
no token FLOOR-5 respells — no `match`, no written type argument, no annotated
`let`. Either order therefore works with no re-take.

Two things the activation task must still do if FLOOR-5 activates first:

1. **Re-verify, do not assume.** Re-run the anchor check of §8 against the
   then-active spec before applying the delta. The anchor is one fixed string
   and the check is one command; a candidate whose anchor no longer matches
   exactly one line stops for review rather than being fuzzy-matched.
2. **Read the inserted sentence.** FLOOR-5's insertion defines the `if_stmt` /
   `value_if` continuation join and adds an empty-join clause. It does not
   change which edges leave a loop body, so §5.2's exhaustiveness argument
   still holds, and §4.1's third ground explains why the reachability form
   needs no edit for the new control forms. Confirm both when re-reading rather
   than carrying this sentence forward untested.

The anchors that genuinely do need re-taking against FLOOR-5 belong to the
companion candidate, not to this one: `provenance-gate-candidate.md`'s [ENT-6]
second site is exactly FLOOR-5's [ENT-6] site. That is recorded there.

## 8. Verification record

All checks run 2026-08-07 against the active spec at 8f91ede.

1. **Anchor exactness.** The §4 anchor was matched as a fixed string against
   `spec/kernel-spec-v0.22.md`: it matches exactly one line (line 1042). It is
   quoted verbatim, not paraphrased.
2. **Grammar containment.** The active spec's fenced blocks span lines 98–126,
   130–139, 143–165, 169–182, 660–662, 706–740, 766–826, 830–842, and
   1050–1093. Line 1042 lies outside all of them, so the site touches no
   grammar production, terminal, operation-table row, or worked example.
3. **Native grammar verifier, baseline:**

```sh
cargo run -q --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
  spec/kernel-spec-v0.22.md
# -> grammar-preserving candidate verified by the active compiler:
#    65 productions, 75 decisions, 76 terminal predicates
# exit code 0
```

   This batch introduces no new bytes outside the one prose paragraph, so the
   assembled document must reproduce those three counts and exit code exactly.
   Any other result is a drafting defect in this file, not a language change.
   The full-document assembly and its byte comparison are the activation task's,
   and are authoritative over this record.

No file under `spec/`, `docs/`, `tests/`, or `compiler/` is modified by this
candidate.
