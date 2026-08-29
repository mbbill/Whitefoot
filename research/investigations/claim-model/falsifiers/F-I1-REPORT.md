# F-I1 - hand-executing `[IND-7]`'s certificate check

Target: `research/investigations/claim-model/DESIGN.md` at 236b837f,
branch `batch/0106-claim-model-design`. Nothing committed; nothing pushed.
Compiler: the existing gate build `wf-0107-audit/target/gate/whitefootc`.
Worksheets: `worksheets/`. New probes: `probes/`.

## Verdict: FAIL

The falsifier's own refutation criterion fires, and two further breaks turned up
on the way. Stated plainly, because the failure is the useful part:

1. **Every derivation the file actually drafts reproduces, digit for digit** -
   the six traces of 3.9 (four families) plus the two refusal traces of 3.8.3.
   I2's base and step, I3's step, I1's midpoint and I4's base and two body paths
   all execute exactly as written, including `255*o - 65025`, `floor(0/255)`,
   the `p := 0` cancellation, and I1's `floor(1/2) = 0`. The certificate form
   does the arithmetic the file says it does.
2. **Two of the six traces 2.4 names do not exist**, and the one F-I1 names by
   section - "the counted ipv4 restructure of 4.4" - is refuted when
   constructed: it needs `2*half - length <= 0`, a two-term relation with a
   coefficient of 2 that no clause of `[IND-4]`, `[IND-6]`, `[IND-7]` or
   `[IND-8]` can put at a check point. F-I1: "*Refuted if* any needs a
   hypothesis the rule does not name."
3. **Two soundness breaks.** The base obligation can discharge itself from
   `[IND-7]`'s own hypothesis list, which admits a false invariant and a
   one-byte out-of-bounds write; and `[IND-4]`'s division witness is false for a
   negative dividend, because the language's exact division truncates toward
   zero, which admits a division by zero. Both have compiled consumer probes.
4. **`[ENT-1]` monotonicity is still not restored.** The certificate form does
   remove the elimination-order break (A4 as diagnosed), but the two caps that
   make the search affordable are hard errors on counts that grow under prover
   strengthening, so a v0.40 program can still fail to compile on v0.41.
5. **Three determinism findings**, one of them on I1's midpoint - the file's one
   surviving customer for `[IND-10]` - where the two readings of `[IND-4]`'s
   backward-pass sentence accept different programs.
6. **What holds:** the superset claim over the greedy rule is correct; FF2 and
   FATAL-1 are both genuinely closed by `[IND-6]`'s frame sentences and the
   certificate search reopens neither; and 2.4's rejected "syntactically total
   hypothesis list" repair is indeed dissolved - with the `(q,hi)` slot present
   the midpoint certificate still exists.

## 1. The traces the file presents, and their outcome

F-I1 names "the traces of 3.9.3, 3.9.4 and 3.9.5, plus the counted ipv4
restructure of 4.4"; 2.4 names six - "I1's midpoint, I2's accumulator, I3's
product, I4's counter, the four bucket-B statements, the counted ipv4
restructure". Every worked `[IND-7]`/`[IND-4]` derivation in the file:

| # | trace | where | outcome |
| --- | --- | --- | --- |
| T1 | I2 base | 3.9.3 | reproduces; exposed to B1 |
| T2 | I2 step | 3.9.3 | reproduces; depends on F4 and F5 |
| T3 | I3 step | 3.9.3 | reproduces; depends on F5 |
| T4 | I1 midpoint (local) | 3.9.4 | reproduces under the file's reading; **dies under the drafted sentence read literally** (F2), and again under F3 |
| T5 | I4 base + 2 body paths | 3.9.5 | reproduces; F5 in its plainest form, stated by the file itself |
| T6 | counted ipv4 restructure | "4.4" | **no trace is drafted**; constructed, it is refused four ways |
| - | the four bucket-B statements | 2.4 | no trace is drafted; 2.8 routes all four to guard rewrites or restructure |
| T7 | A16 / FATAL-1 refusal | 3.8.3 | refused, as claimed |
| T8 | A2 / FF2 refusal | 3.8.3 | refused, as claimed |

The file names the set twice and names two different sets. Section 11 (line
3990) says "the six traces of 3.9", which is exactly T1-T5 counting I2's base,
I2's step, I3's step, I1's midpoint, I4's base and I4's step - all six executed
here, all six reproduce. Section 3.9.7's F-I1 (line 2329) instead says "plus the
counted ipv4 restructure of 4.4", and 2.4 adds "the four bucket-B statements".
Those last two have no derivation anywhere in the file.

Full arithmetic in `worksheets/T1-T2_i2.md`, `T3_i3.md`, `T4_i1_midpoint.md`,
`T5_i4.md`, `T6_ipv4.md`, `T7-T8_refusals.md`.

The acceptance claim of section 2.4 - "It accepts a superset of the greedy rule.
Every greedy elimination sequence *is* a certificate" - is CORRECT, and I could
build no counterexample: "unused" makes greedy's selection injective and the
traversal is the same term order, so every greedy run is a certificate with the
same final `p` and `s`. What is wrong with the sentence is its scope: it is used
to conclude that "none of the drafted traces ... has to be re-derived", and two
of the six were never derived at all.

## 2. Compiled arbitration

Existing probes, re-run here; every verdict matches what the design claims:

```
REJECT  s22_accum_const.wf          [OP-2] residual: sum +defined wide
REJECT  s23_accum_param.wf          [OP-2] residual: acc +defined factor
REJECT  L08_i2_consumer.wf          [OP-2] residual: sum +defined wide
ACCEPT  L05_i4_step.wf              ACCEPT  L06_i4_step_skip.wf
REJECT  L24_matchcount_correct.wf   [OP-4] residual: hits < len(out)
REJECT  L26_ipv4_counted.wf         [OP-4] residual: offset < len(deref(header))
REJECT  j3_ind6_checkpoint_break.wf [OP-4] residual: x < len(out)
ACCEPT  j3b_ind6_consumer.wf        ACCEPT  L11_bsearch_ifelse_price.wf
```

Written for this experiment, in `probes/`:

```
REJECT  f3_sdiv_false_bound.wf   [FN-8] instantiated_goal: "ile(h, -3_i64)"
ACCEPT  f2_sdiv_consumer.wf
REJECT  f4_sdiv_interval.wf      [OP-2] residual: h +defined 2_i64
```

`f3` is the arbitration for B2: today's checker refuses precisely the bound
`[IND-4]` clause (d)'s witness pair would prove for `-5 / 2`, and `f2` shows
that bound is exactly what buys the nonzero divisor. `f4` shows today's `/`
image supplies nothing about `h` here, so the witness pair is the only source.

## 3. The findings

Severity ordered. `worksheets/ADVERSARIAL.md` carries the full constructions.

### B1 (soundness) - the base obligation can discharge itself
`[IND-5]` sends the base through `[IND-7]`, and `[IND-7]`'s hypothesis list
begins with "the statement polynomials of that loop's `bound_stmt`s as written",
unrestricted. `sigma(t) = H1` where `H1` is the statement itself gives
`p := |b|*p - |a|*h = 0` for the leading term of any statement, so **every
labelled `bound_stmt` has a vacuous base**. Witness: `bound @spin lie:
ile(idx, 0_u64);` with `idx = 9`; the step is honestly true, `[IND-8]` publishes
`idx <= 0`, and `set out[idx] = 1_u8` on a one-byte buffer discharges.
`j3b_ind6_consumer.wf` (compiled) accepts exactly that consumer.
`[IND-10]` is the worse case: it has "no base and no step", so if its own
polynomial is in group 1 the local statement is true by construction.

### B2 (soundness) - the division witness is false for a signed dividend
`[IND-4]` clause (d) asserts `k*q - a <= 0` and `a - k*q <= k - 1` for
"`a / k` for a literal `k >= 1`", with no restriction on `a`'s sign, while
kernel-spec 845 fixes exact division as "truncating toward zero". For
`a = -5, k = 2` the language's `q` is -2 and the first hypothesis is false.
The certificate then proves `h <= -3` where the truth is `h <= -2`, one integer
of falsehood, and `floor(C/s)` is what converts it into an accepted bound - the
same integer tightening I1's midpoint depends on. Consumer: `h + 2` is proved
nonzero and is 0, so an accepted program divides by zero.

### F7 (monotonicity) - the two caps are hard errors on prover-dependent counts
`[IND-7]`'s "`H` has at most **16** members, and more is a hard error" and
"there are at most **4** [elimination terms], and more is a hard error" are both
counts that GROW under prover strengthening: group 3's twelve ordered-pair slots
are each present only "if any", and clause (b)'s route depends on whether the
no-wrap side condition is derivable, so a strengthening can replace one clause
(e) witness term with two operand terms. Either crossing turns a compiling
program into a hard error. The paragraph "**No fact-source or closure
strengthening can refuse a statement an earlier conforming checker verified**"
is therefore false as written. A4 is not closed; it moved from the elimination
order to the caps. A third mechanism (F7c: strengthening changes `p` itself,
and clause (e)'s commit-point witness can be strictly stronger than the exact
route's check-point RELAX) is named in the worksheet without a confirmed
witness.

### F2 (determinism) - the backward pass and the witness hypotheses
"replacing, at each `let` or `set` commit whose destination occurs **in the
polynomial** at the moment the pass reaches that commit, that destination by the
polynomial of the commit's right-hand side". I1's trace requires the pass to
rewrite `span` inside the two DIVISION WITNESS HYPOTHESES, which are not "the
polynomial". Read literally, `span` survives in `H`, is not an elimination term,
cannot be eliminated, and the residual relaxes to `cu(span)` - at a region entry
where `span` is not even in scope. **The midpoint is refused.** Two readings,
different accepted sets, on `[IND-10]`'s one surviving customer.

### F3 (determinism) - clause (a)'s proviso over the checked operations
"(a) `a + b`, `a - b`, `a * b`, and the `Ok`-arm binding of `+checked`,
`-checked` or `*checked`: the exact polynomial, provided the state at that
commit on that path discharges the operation's `[ENT-6]` domain obligation".
A `+checked` has no `[ENT-6]` domain obligation. Vacuously satisfied, I1 works;
"nothing to discharge, so not discharged", `span` and `mid` both fall to clause
(e) and I1 dies. This is A1 / FATAL-1c's vacuity, repaired in clause (b) by
naming the side conditions, and re-created in clause (a).

### F4 / F9 (well-definedness) - RELAX over opaque terms
RELAX's intervals are "the tightest constant bounds ... derivable at the check
point". `o` and `q` are fresh opaque terms, not terms of the head or region-entry
state; `cu(o) = 255` in I2's trace comes from a witness hypothesis, and a
clause (d) `q` has NO constant bound at all, only the two relative ones. The
exhaustive enumeration necessarily evaluates certificates whose residual retains
`q` (the empty certificate on I1's obligation is one). Two readings - treat the
missing bound as unbounded and let that certificate fail, or raise `[IND-3]`'s
hard error - and only the first leaves the accepted set alone.

### F5 (rule text gap) - the substitution reads a state that assumes the statement
Clause (a)/(b) admit a commit only when the operation's obligation or side
condition discharges AT THAT COMMIT, and for I2, I3 and I4 the only source of
that fact is the statement's own `[IND-8]` projection at the head. The file says
so out loud for I4: "which the head state carries through the published
`hits - i <= 0`". 3.14's pipeline resolves it - step 3 applies `[IND-8]`
projections during the walk, step 4 verifies - and the ordering is sound
assume-guarantee, but no sentence of `[IND-4]` or `[IND-6]` says it, and
`[IND-6]`'s "extended by exactly two sets of hypotheses" reads against it.
Compiled: `s22`, `s23` and `L08` all reject without the projection.

### F8 (determinism) - `[IND-8]`'s `m` has no definition
"`m` is the sum over the monomials of `r` of that monomial's **minimum**". No
rule defines a monomial's minimum; `[IND-7]` defines only "the maximum over the
corner products". This is A24's D-2 repaired in `[IND-7]` and left open one rule
later, and the direction is unsound: too large an `m` publishes too tight a
bound.

### F10 (gap) - `[IND-7]`'s group 1 for a local statement
"the statement polynomials of that loop's `bound_stmt`s" - an `[IND-10]`
statement has no loop. Whether the enclosing loop's statements enter, and
whether the local statement's own polynomial does, is unstated; the second
reading is B1 with no base or step to catch it.

### F11 (legibility only) - a zeroed coefficient
"the step is admitted only when `a*b > 0`" does not say whether a term whose
coefficient an earlier step zeroed makes the certificate fail or is skipped.
The accepted set is unchanged either way (the skipping certificate is in the
space), so this is a wording defect, not a break.

### F12 (bookkeeping) - the space count
"at most `16*15*14*13` orderings" (43,680) counts only the full-domain
injections; the space is `sum_k C(4,k)*P(16,k)` = 58,625 partial injections.
And "two conforming implementations decide the same predicate on the same
inputs" is false unqualified: `H` group 3 and the RELAX intervals are whatever
is derivable, so the predicate is fixed only relative to a fixed derivability.

## 4. Design sentences that must move

Line numbers are DESIGN.md at 236b837f.

1. **1910-1911, `[IND-7]` H group 1** - "The **hypothesis list** `H` is, in this
   order: the statement polynomials of that loop's `bound_stmt`s as written
   `[IND-6]`; ..." Must exclude the base: group 1 is available to the STEP
   obligation only. (B1)
2. **1814-1816, `[IND-5]`** - "the base obligation is the statement polynomial
   checked by `[IND-7]` in the closed state at `@l`'s preheader" must name the
   hypothesis groups the base may use, rather than inheriting `[IND-7]`'s list
   whole. (B1)
3. **1794-1796, `[IND-4]` clause (d)** - "`a / k` for a literal `k >= 1`: a
   fresh opaque term `q` together with the two hypotheses `k*q - a <= 0` and
   `a - k*q <= k - 1`" must either restrict the dividend to a provably
   nonnegative term or give the signed case the truncation-correct pair, because
   the language's exact division truncates toward zero. (B2)
4. **1772-1775, `[IND-4]` the backward pass** - "replacing, at each `let` or
   `set` commit whose destination occurs **in the polynomial** at the moment the
   pass reaches that commit ..." must say whether the pass also rewrites the
   witness hypotheses it has introduced. I1's trace needs it; the sentence
   forbids it. (F2)
5. **1779-1782, `[IND-4]` clause (a)** - the proviso "provided the state at that
   commit on that path discharges the operation's `[ENT-6]` domain obligation"
   must be split: the exact rows carry it, the `Ok`-arm bindings of the checked
   operations do not have such an obligation and are exact unconditionally. (F3)
6. **1913-1915, `[IND-7]` the `|H| <= 16` cap** - the count must be syntactic
   (all twelve ordered-pair slots present, filled or empty) so a strengthening
   can never cross it, or the hard error must be replaced by dropping the
   surplus in a fixed order. (F7a)
7. **1906-1908, `[IND-7]` the four-term cap** - same defect: the elimination
   term count depends on clause (b)'s prover-dependent route, so the hard error
   is reachable by strengthening alone. (F7b)
8. **1932-1937, `[IND-7]` RELAX** - "each factor's interval being `[cl, cu]` for
   the tightest constant bounds ... derivable at the check point" must say that
   the check point's state is the one `[IND-6]`/`[IND-10]` extended, and must
   define RELAX on a term with no derivable constant bound - which is every
   clause (d) witness. (F4, F9)
9. **1927-1929, `[IND-7]` determinism** - "There is no implementation choice:
   two conforming implementations decide the same predicate on the same inputs"
   must be qualified to equal derivability, since group 3 and RELAX both read
   the ambient prover. (F12)
10. **1985-1987, `[IND-8]` the projection constant** - "`m` is the sum over the
    monomials of `r` of that monomial's **minimum**" must define the minimum by
    corner products, as `[IND-7]` does for the maximum. (F8)
11. **1836-1841 / 2140-2142, `[IND-6]` and `[IND-10]` check state** - must state
    that the statement's own `[IND-8]` projection is in the state during its own
    base and step check (3.14 step 3 before step 4), because clause (a)/(b)'s
    provisos depend on it for I2, I3 and I4. As drafted, "extended by exactly
    two sets of hypotheses" reads against the pipeline. (F5)
12. **1910, `[IND-7]` group 1 for a local statement** - "that loop's
    `bound_stmt`s" is undefined for an `[IND-10]` statement; say which
    statements enter, and exclude the statement under check. (F10)
13. **1963-1965, the space count** - "at most `16*15*14*13` orderings" should be
    the partial-injection count, 58,625. (F12)
14. **427-432 (2.4), 2328-2331 (F-I1) and 3990 (section 11)** - "none of the drafted traces - I1's
    midpoint, I2's accumulator, I3's product, I4's counter, the four bucket-B
    statements, the counted ipv4 restructure - has to be re-derived" and "plus
    the counted ipv4 restructure of 4.4" both name traces the file does not
    contain, and 2.8 routes the bucket-B four and the ipv4 congruence away from
    the construct. Either draft them or drop them from the list. (T6)
15. **1967-1971 (the superset paragraph)** - the inference is sound but is doing
    work it cannot do: "none of the drafted traces below has to be re-derived"
    covers only the traces that exist. Say so.
16. **1944-1952, the `[IND-7]` monotonicity paragraph** - "**No fact-source or
    closure strengthening can refuse a statement an earlier conforming checker
    verified**" is false while the caps are hard errors; it must be restated
    against the repaired caps, and 2.4's and 3.1.2's `[ENT-1]` headline sentence
    depends on it.
17. **3990-3992 (section 11's F-I1 entry)** - the falsifier says the experiment
    "must be run before the text is fixed". It has now been run and it fires:
    the entry should record the refutation and the two soundness breaks rather
    than a pending experiment.

## 5. What I did not do

I did not commit, push, or edit anything on the branch, and I wrote no
owner-approval text. The two soundness witnesses are hand executions plus
compiled consumer probes; neither is a compiled end-to-end program, because
`bound_stmt` does not exist in the compiler. The F7c hazard is named without a
witness. `probes/f1_sdiv_trunc_break.wf` rejects earlier than intended (on the
`+` obligation) and is kept only as the negative control; `f3` and `f4` are the
arbitration that matters.
