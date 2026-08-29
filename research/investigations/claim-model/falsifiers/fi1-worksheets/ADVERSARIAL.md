# The three adversarial constructions

## (a) greedy-accepted but no certificate  ->  NOT FOUND; the claim holds
The drafted greedy rule eliminated each term, in term order, against "the first
UNUSED hypothesis whose coefficient `b` on `t` satisfies `a*b > 0`". "Unused"
makes the selection injective; the traversal is the same term order the
certificate uses; a term greedy could not match is a term outside sigma's
domain. So every greedy run IS a certificate with the same final `p` and `s`,
and "the certificate form accepts a superset of the drafted greedy rule" is
correct. The inclusion is strict in the useful direction: greedy's choice is
mandatory, so a hypothesis that ruins the residual is forced on it where a
certificate simply omits the term. I could construct no counterexample and I do
not believe one exists.

## (b) a certificate that accepts a FALSE bound  ->  FOUND, twice

### B1. The base can prove itself  (the sharper of the two)
`[IND-5]`: "the base obligation is the statement polynomial checked by
`[IND-7]` in the closed state at `@l`'s preheader."
`[IND-7]`: "The **hypothesis list** `H` is, in this order: the statement
polynomials of that loop's `bound_stmt`s as written `[IND-6]`; ..."
Nothing restricts group 1 to the step. So the statement under check is a
hypothesis of its own base.
```whitefoot
command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(1_u64, 0_u8);
  let idx = 9_u64;
  let n = 4_u64;
  for @spin t in 0_u64..n {
    bound @spin lie: ile(idx, 0_u64);
  }
  set out[idx] = 1_u8;
  return exit_status(code: 0_u8);
}
```
P = `idx <= 0`.  Honest base: p0 = `idx`, RELAX = cu(idx) = 9 > 0, refused.
With group 1: sigma(idx) = H1 = `idx`; a = +1, b = +1, a*b > 0;
p := 1*p0 - 1*H1 = 0; s = 1; RELAX(0) = 0; floor(0/1) = 0 <= 0  ->  VERIFIED.
Step: no commit touches `idx`, so p0 = `idx` = H1 and the step verifies honestly.
`[IND-8]`: t = idx, a = +1, r empty, m = 0 -> publishes `idx - Z <= 0` on both
header edges. At the continuation `set out[idx] = 1_u8` needs `idx < len(out)`
= `idx < 1`, discharged. At run time idx = 9 and out is one byte:
**memory corruption in an accepted program.**
Compiled arbitration (already in the tree): `j3b_ind6_consumer.wf` ACCEPTS with
`requires ile(x, 0_u64)` and `ieq(len(out), 1_u64)` - the fact is consumed
exactly as assumed; `j3_ind6_checkpoint_break.wf` REJECTS
`[OP-4] residual: x < len(out)` without it.
A charitable reading exists ("as written `[IND-6]`" = only where `[IND-6]`
supplies them), so at minimum this is a determinism defect; read as drafted it
is a soundness break. `[IND-10]` is worse: it has "no base and no step", so if
its own polynomial is in group 1 the local statement is true by construction.

### B2. The division witness is unsound for a signed dividend
`[IND-4]` clause (d): "`a / k` for a literal `k >= 1`: a fresh opaque term `q`
together with the two hypotheses `k*q - a <= 0` and `a - k*q <= k - 1`."
Nothing restricts `a`'s sign, and `[IND-3]`'s typing note admits signed terms
explicitly. But kernel-spec 845: "Exact division is truncating toward zero".
For `a = -5`, `k = 2`, the language's `q` is -2, and `k*q - a = -4 + 5 = 1 > 0`:
**the first witness hypothesis is false.** `[IND-7]`'s soundness argument
requires `h <= 0` for every hypothesis used.
```whitefoot
fn f(a: own i64) -> out: own i64 pure contract {
  requires ile(a, -5_i64);
  requires ige(a, -5_i64);
} {
  let h = a / 2_i64;
  bound low: ige(h, -5_i64);     // true
  bound tight: ile(h, -3_i64);   // FALSE: h = -2
  let dv = h + 2_i64;            // [-3,-1] under the published facts
  let r = 100_i64 / dv;          // divisor proved nonzero
  return r;
}
```
`tight`: p = `h + 3`; clause (d) gives `h := q` with `2q - a <= 0`,
`a - 2q <= 1`; p0 = `q + 3`; elimination terms { q }.
  sigma(q) = `2q - a <= 0` : a_coef = +1, b = +2, a*b > 0
  p := 2*(q+3) - 1*(2q - a) = `a + 6` ; s = 2
  RELAX(a + 6) = cu(a) + 6 = -5 + 6 = 1 ; floor(1/2) = 0 <= 0 -> VERIFIED.
(The tightest constant it proves is exactly `h <= -3`, one below the truth -
the floor-versus-truncate off-by-one, and `floor(C/s)` is what converts it into
an accepted bound.)
`low` verifies honestly (sigma(q) = `a - 2q - 1 <= 0`, p := `-a - 9`,
RELAX = -4, floor(-4/2) = -2 <= 0).
`[IND-8]` publishes `h - Z <= -3` and `Z - h <= 5`; `h + 2` is then proved in
range and nonzero. At run time h = -2, dv = 0: **division by zero in an accepted
program**, and exact division's obligation "excludes zero divisor".
Compiled arbitration, written for this experiment:
  `probes/f2_sdiv_consumer.wf`   ACCEPT  - the false bound buys the divisor
  `probes/f3_sdiv_false_bound.wf` REJECT `[FN-8]` instantiated_goal
                                  `ile(h, -3_i64)` - today's checker refuses
                                  exactly the bound clause (d) would prove
  `probes/f4_sdiv_interval.wf`   REJECT `[OP-2] residual: h +defined 2_i64`
                                  - today's `/` image supplies nothing here,
                                  so the witness pair is the only source.
The break needs `k >= 2` and a negative dividend; with `k = 1` the pair is
`q - a <= 0`, `a - q <= 0`, which is true for either sign.

### The judges' two shapes, re-tested against the final text
FF2 (self-cancelling substitution) and FATAL-1 (head-vs-exit value) are both
REFUSED - worksheet `T7-T8_refusals.md`. `[IND-6]`'s two frame sentences hold,
and the certificate search opens no new route to either. That half of the
synthesis stands.

## (c) is the certificate space finite and spec-fixed?
**Finite: yes.** Midpoint trace: 3 elimination terms, |H| = 3 (two division
witnesses plus the one derivable `(lo,hi)` slot), so
sum_k C(3,k)*(3)_k = 1 + 9 + 18 + 6 = **34** certificates, enumerated in
`T4_i1_midpoint.md`. Worst case 4 terms into 16 hypotheses:
1 + 64 + 1440 + 13440 + 43680 = **58,625** partial injections. The file's "at
most `16*15*14*13`" counts only the full-domain injections (43,680) and
understates the space by a third - harmless for the affordability argument,
wrong as a count.
**Spec-fixed: no.** The caps are fixed; the CONTENT is not. Group 3 is "the
tightest difference bound ... derivable at the check point `[ENT-4]`, if any"
and RELAX's intervals are "derivable at the check point", so both are functions
of prover strength. The file knows this and relies on it for monotonicity, but
the sentence "There is no implementation choice: two conforming implementations
decide the same predicate on the same inputs" is false as written; what is true
is that two implementations WITH THE SAME DERIVABILITY decide the same
predicate, plus the monotonicity below.
**And the caps break the monotonicity theorem they were added to protect.**
 - F7a, the `|H| <= 16` hard error. Four elimination terms give group 3 twelve
   ordered-pair slots, each present only "if any". A strengthening that derives
   one more pair bound takes |H| from 16 to 17, and "`H` has at most **16**
   members, and more is a hard error naming the statement". A program that
   compiled under v0.40 does not compile under v0.41.
 - F7b, the `<= 4` elimination-term hard error. Clause (b) is prover-dependent:
   `let y = a +wrap b;` whose side condition is not derivable falls to clause
   (e) and contributes ONE witness term; when a tightened row image makes the
   side condition derivable, clause (b) substitutes `a + b` and contributes
   TWO. At four terms that is a fifth term and a hard error.
 - F7c, a hazard I could not close. Because clause (a)/(b) provisos are
   prover-dependent, a strengthening changes `p` itself. Clause (e)'s witness
   bound is read AT THE COMMIT, where path conditions and everything the body
   has learned are in the state; RELAX evaluates the exact polynomial's terms
   AT THE CHECK POINT, where path conditions are only hypotheses. So the
   witness route can be strictly stronger than the exact route, and
   strengthening moves a commit from the first to the second. I built no
   concrete regression - the natural candidates need a tight image on the
   destination while the operands stay loose - so this is recorded as a named
   mechanism, not a confirmed break.
So `[ENT-1]`'s "No fact-source or closure strengthening can refuse a statement
an earlier conforming checker verified" is FALSE as written, for two independent
and fully concrete reasons. A4 is not closed; it moved from the elimination
order to the two caps.
