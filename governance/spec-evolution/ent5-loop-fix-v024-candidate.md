# ENT-5 loop-rule fix — specification-change candidate

Status: **EXACT-APPROVED AND ACTIVATED 2026-08-09 AS v0.24**. The owner approved
the complete stable-file digest and named protected-corpus changes recorded in
`governance/APPROVALS.md`; the installed `spec/kernel-spec.md` has SHA-256
`53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.
The 2026-08-07 owner approval covered bytes anchored to v0.22 and did **not**
carry over; this document records the re-cut delta that received the later
exact approval against `spec/kernel-spec-v0.23.md` (installed a01bc70; SHA-256
`e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5`, read from
the `ACTIVE-SPEC:` chain in `governance/APPROVALS.md`; roadmap revision 19).
Approval covers these exact bytes; a changed byte returns to review.

**The rule text in §4 is unchanged from the approved version.** What changed is
everything that named the superseded baseline — the version number, the prior
version and its digest, the roadmap revision, §7's tense, and §8's measurements
— plus §2's status word, which mandatory amendment 8 of
`stable-spec-filename-proposal.md` now places inside the approved bytes.

This candidate was split out of `semantics-v024-candidate.md` (drafted and
withdrawn 2026-08-07) on the lead's ruling. Its companion — the
subject-position taint gate — is `provenance-gate-candidate.md`, HELD FOR
MEASUREMENT. The two share no rule text; nothing here depends on that one.

**Numbering, settled.** The provisional reservation resolved: FLOOR-5 activated
first and took v0.23 (a01bc70), so this batch is **v0.24**. §7 records the
ordering interaction with FLOOR-5 as it actually turned out.

**This activation also carries the stable-filename switchover**, per §5 of
`stable-spec-filename-proposal.md`, which routes it onto the first activation
with no EBNF change. That is a change to the specification's file model, not to
this batch's rule text, and it is the activation task's work rather than this
document's. The one place it reaches these bytes is mandatory amendment 8: the
status line in §2 must read `Status: ACTIVE v0.24` before approval, because
under the stable model the status line is part of the bytes the owner approves
and the file is never edited afterwards.

**O11 does not ride this activation** (ruled 2026-08-09). The ENT boolean-
composition correction was queued "drafted alongside the approved ENT-5 loop
fix" before the switchover was sequenced onto the same activation; pairing a
second semantic correction with the vehicle chosen for being small undercuts
the reason it was chosen. Nothing here depends on it.

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

> Status: ACTIVE v0.24 (2026-08-09; the [ENT-5] loop-rule scope-exit
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
> superset of the one v0.23 computes and discharge only widens.
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
> with the `D1h`/`D1i` witness pinning the cause to this rule alone. The review
> candidate carries the `Status: ACTIVE v0.24` token as an input to exact
> approval, but that token names language authority only after the grammar
> check, derived-material review, full-document hash, exact owner approval,
> chained `ACTIVE-SPEC:` record, and active-target installation are all
> complete. Before then the branch remains non-authoritative; after
> installation those approved bytes remain the active integration bytes until
> a later exact-approved activation replaces them.

**Header assembly is exact, not inferred.** In the complete v0.24 document,
replace the first line with `# Kernel Specification v0.24`. Replace the one
current v0.23 `Status:` paragraph with the paragraph quoted above, one blank
line, and the exact former v0.23 paragraph after changing only its leading
`Status:` token to `Prior:`. Every byte after that prefix remains the installed
v0.23 byte. This is the same history-header transformation used by the prior
version steps; inserting the new paragraph while leaving a second `Status:`,
or omitting the v0.23 history paragraph, is not this candidate.

## 3. Grammar delta

None. This batch adds, removes, and reshapes no production, terminal
predicate, token form, operation-table row, or source construct. Its one
modification site is prose at line 1053 of the active spec, outside every
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
3. It composes with control forms the version it was drafted against did not
   have — and that prediction has now been tested rather than left standing.
   FLOOR-5 added `if_stmt` and `value_if` in v0.23. An else-free `if`'s false
   edge stays inside the body and is continuing; `value_if`'s delivery edges
   route to the same three targets `value_match`'s do [FN-1]. The enumerated
   form would have needed re-enumeration at v0.23; this text needed no edit,
   which is the ground itself coming true. §5.2 records the check.

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
**v0.23**: `break_stmt` reaching `normal_successor` of its resolved target loop,
`return_stmt` and `propagate_let_rhs`'s `Err` edge reaching the function-return
sink, and the delivery edges of `value_match` and — added by FLOOR-5 —
`if_stmt` and `value_if`, which [FN-1] routes to exactly those same targets.
There is no `continue` form.

The FLOOR-5 control forms were checked against this list rather than assumed to
fit it. [ENT-5]'s join paragraph in the installed v0.23 states that an
`if_stmt` or `value_if` branch "every path of which leaves by `return`, `break`
to an enclosing loop, or `propagate`'s error edge contributes nothing" at the
continuation — the same three targets, named in the same terms as the
`match_stmt` clause beside it. An else-free `if_stmt`'s false edge is the one
new edge, and it reaches the continuation **inside** the body, so it is
continuing and is scanned. The list therefore grew by two forms and by no new
target, which is why §4.1's third ground holds: the reachability form needed no
edit to absorb them, where an enumerated form would have needed re-enumeration.

### 5.3 Monotonicity under [ENT-1]

The change only removes kills from one scan, so at every loop head the fact
state is a superset of the state v0.23 computes. [ENT-4]'s closure is monotone
and derivability is upward-closed in the state, so:

- every obligation v0.23 discharges is still discharged, and more are;
- no [OP-4] rejection is newly created on discharge grounds;
- a claim v0.23 accepts as non-redundant may become redundant, which [CLM-2]
  makes a non-rejecting advisory precisely so this direction stays monotone.

The one edge that is not monotone is the one [ENT-1] already enumerates.
[CLM-2] rejects a claim whose exact negation the non-contradictory state
derives. A pre-loop fact that now survives to the loop head can supply that
negation, so a program accepted under v0.23 can be newly rejected as a refuted
claim. This is the lifecycle's single deliberate non-monotone edge, already law
in v0.23 ("Refutation is the lifecycle's one deliberate non-monotone edge"),
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

**FLOOR-5 activated first, and the predicted disjointness held — re-measured
against v0.23, not carried forward.** FLOOR-5's single [ENT-5] site was an
*insertion* into the join paragraph; this batch *replaces* the rule's final
paragraph. In the installed v0.23 those are two distinct lines: the joins
paragraph is line 1051, the loop paragraph this batch replaces is line 1053.
No overlap, and no re-take was needed.

Both cautions this section raised for the activation task were discharged
mechanically rather than by reading, 2026-08-09:

1. **Re-verified, not assumed.** The §4 anchor was re-matched as a fixed
   string against the active `spec/kernel-spec-v0.23.md`: **exactly one
   occurrence**, at line 1053. §8 records the command and its result.
2. **The inserted sentence was read, and its irrelevance checked rather than
   asserted.** FLOOR-5's insertion defines the `if_stmt` / `value_if`
   continuation join and adds an empty-join clause. It adds no way for control
   to leave a loop body, so §5.2's exhaustiveness argument stands unchanged,
   and §4.1's third ground is what makes the reachability form need no edit for
   the new control forms — the property it was written to have. Separately
   confirmed by search: this batch's replacement text contains **no** token
   FLOOR-5 respells — no `match`, no written type argument, no annotated `let`
   — so nothing in it was silently left in a superseded spelling.

The anchors that genuinely do need re-taking against FLOOR-5 belong to the
companion candidate, not to this one: `provenance-gate-candidate.md`'s [ENT-6]
second site is exactly FLOOR-5's [ENT-6] site. That is recorded there.

## 8. Verification record

All checks re-run 2026-08-09 against the active spec `spec/kernel-spec-v0.23.md`
at a01bc70. The v0.22 figures this section previously carried are superseded and
are not retained: an anchor checked against superseded text is worth nothing.

1. **Anchor exactness.** The §4 anchor spans eight wrapped lines in this
   document and one line in the specification, so it is unwrapped to a single
   547-byte string before comparison, never matched line by line. Against
   `spec/kernel-spec-v0.23.md` it matches **exactly one line, line 1053**, and
   the match is **whole-line exact** (`grep -x -F`): the anchor is that
   paragraph in its entirety, not a fragment of it, so the replacement in §4
   cannot silently leave a tail of the old rule behind. It is quoted verbatim,
   not paraphrased.

   The whole-line form of the test is deliberate. A substring match proves only
   that the anchor occurs somewhere; it would pass just as happily if the
   specification's paragraph had grown a clause this document does not know
   about. Whole-line equality is what rules that out.
2. **Grammar containment.** The active spec's fenced blocks span lines 101–129,
   133–142, 146–171, 175–193, 671–673, 717–751, 777–837, 841–853, and
   1061–1096. Line 1053 lies outside all of them, so the site touches no
   grammar production, terminal, operation-table row, or worked example.
3. **Native grammar verifier, baseline** (the two-path form required by
   mandatory amendment 5 — both paths read at runtime, so the comparison is not
   `X != X`):

```sh
cargo run -q --manifest-path compiler/Cargo.toml --bin whitefoot-grammar -- \
  spec/kernel-spec-v0.23.md spec/kernel-spec-v0.23.md
# -> grammar-preserving candidate verified by the active compiler:
#    69 productions, 84 decisions, 93 terminal predicates
# exit code 0, read from $? and not through a pipe
```

   This batch introduces no new bytes outside the one prose paragraph, so the
   assembled document must reproduce those three counts and exit code exactly.
   Any other result is a drafting defect in this file, not a language change.
   The full-document assembly and its byte comparison are the activation task's,
   and are authoritative over this record.

## 9. Activation disposition

The exact delta was assembled into the complete v0.24 stable-file bytes and
activated with the continuing-kill implementation, the approved rewrite of
`ent5-neg-loop-rule-drops-preloop-fact`, and the additive
`ent5-pos-return-does-not-kill-loop-head-fact` case. There was no existing
verdict, cited rule, or runnable-status change. The candidate document digest
`8c520d868b54ff40332ac2c2475a8e4e32fe217b4ab513279420a0a67818c656`
identifies the historical review record at commit `7e47130`; this disposition
changes the document and therefore does not reuse that digest as its current
identity. The complete active specification digest above is the approval and
activation identity.

As a delta record, this document itself directly modified no file under
`spec/`, `docs/`, `tests/`, or `compiler/`; task 0045 assembled and installed
the approved complete tree.
