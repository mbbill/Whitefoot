# N3, N1, N2 and every trace re-executed under 7f600c06; part (v) attacked

## N3, the full hand execution the charter asks for

Statement `ile(acc, i + w1 + w2)` -> `p_stmt = acc - i - w1 - w2` (degree 1,
four monomials, coefficients +/-1: inside `[IND-3]`).

**Base `[IND-5]`.** Post-capture `acc = 0`, `binder = 0`. No substitution. Group
1 empty (`s` is the loop's only statement and `[IND-5]` excludes it). Elimination
terms `acc, i, w1, w2` - four, at the cap. Slots: 0 + 0 + 12 ordered pairs = 12.
Empty certificate: `RELAX = max(acc) + max(-i) + max(-w1) + max(-w2) = 0`,
`floor(0/1) = 0 <= 0`. **VERIFIED at both versions.**

**Step `[IND-6]`.** Binder shift `p = acc - i - w1 - w2 - 1`. Backward pass:

```
C3  set acc = acc + 1        clause (a)  acc <- acc + 1   p = acc - i - w1 - w2
    branch ilt(g1, g2)       clause (i)  {g1, g2} enter;  cond g1 - g2 + 1 <= 0
C2  let g2 = w2 *wrap K      clause (b), LET binder, witness o2
        cond -> g1 - o2 + 1 <= 0 ; slots: 2 constant bounds on g2,
        Q2a = o2 - K*w2 <= 0 , Q2b = K*w2 - o2 <= 0 ; {w2} enters (already in)
C1  let g1 = w1 *wrap K      clause (b), witness o1
        cond -> o1 - o2 + 1 <= 0 ; slots: 2 constant bounds, Q1a, Q1b
p0 = acc - i - w1 - w2      elimination terms acc, i, w1, w2 - four
slots = 1 (H1) + 1 (path cond) + 4 (C1) + 4 (C2) + 12 (ordered pairs) = 22
```

Identical at both versions; the shape rule holds.

**Weak prover (v0.40, no `imin` row image).** `*wrap`'s side condition needs
`c1*c2 <= max(u64)`; `w1`'s only bound is `max(u64)` and `max(u64)*K` is far past
it, so **Q1a, Q1b, Q2a, Q2b are all four present and empty**. The two constant-
bound slots at each of C1 and C2 fill trivially from `u64` (`Z - g <= 0`,
`g - Z <= max(u64)`). Certificate `sigma(acc) = H1`: `a = +1`, `b = +1`,
`p := 1*p0 - 1*H1 = 0`, `s := 1`; `i, w1, w2` outside the domain, skipped;
`RELAX(0) = 0`, `floor(0/1) = 0 <= 0`. **VERIFIED.**

**Strong prover (v0.41).** `w1 - Z <= 1` and `w2 - Z <= 1` derive; `1*K =
15000000000000000000 <= max(u64) = 18446744073709551615`, so all four pairs
**FILL**.

*The step that discards.* `sigma = {w1 |-> Q1a, w2 |-> Q2a}`, in elimination-term
order:

```
acc : outside sigma  -> skipped
i   : outside sigma  -> skipped
w1  : a = -1 in p, b = -K in Q1a, a*b = K > 0  -> admitted
      p := K*p0 - 1*Q1a
         = K*acc - K*i - K*w1 - K*w2 - o1 + K*w1
         = K*acc - K*i - K*w2 - o1                       s := K
w2  : a = -K, b = -K, a*b = K^2 > 0            -> admitted
      p := K*p - K*Q2a
         = K^2*acc - K^2*i - K^2*w2 - K*o1 - K*o2 + K^2*w2
         = K^2*acc - K^2*i - K*o1 - K*o2                 s := K^2
      K^2 = 225000000000000000000000000000000000000
      2^127 = 170141183460469231731687303715884105728        K^2 > 2^127
```

Under the repaired text `[IND-3]`'s limits do not reach here (scope sentence at
1878-1882; `[IND-7]`'s *no hard error* paragraph; sweep row 10). **No error is
raised.** The certificate is finished: `RELAX(p) = K^2*cu(acc) + 0 + 0 + 0`,
positive, `floor(RELAX/K^2) > 0`, so it **fails**, and that failure is a discard
confined to it (row 13).

*The surviving certificate.* `sigma' = sigma(acc) = H1` - which is what part (v)
names, the omitting certificate obtained by dropping every term whose slot went
empty-to-filled (`w1` and `w2`, both of them). It runs `p := 1*p0 - 1*H1 = 0`
with `s = 1`, exactly as at v0.40. `RELAX(0) = 0`, `floor(0/1) = 0 <= 0`.
**VERIFIED at v0.41.**

**N3 verifies at both versions and no enumeration order changes that.** The
file's account of its own witness is correct, digit for digit.

## N1 and N2, re-run

**N1.** `p = x - 255`; `set x = z` and `let z = cvt(y)` are clause (c);
`let y = acc +wrap 7_u8` is clause (b) on a `let` binder, so witness `o`,
`p = o - 255`, and the pair's shape `o - acc - 7 <= 0` / `acc + 7 - o <= 0` puts
`acc` in the term set unconditionally. `set acc = load(buf, i)` is therefore
visited at both versions: right-hand side a call, destination a `set` destination,
**clause (e) refuses** (sweep row 6, class (a)). Hard error at `[IND-1]` on both.
IDENTICAL. Unchanged by this round - the sweep touches neither clause.

**N2.** Nine chained `let a_j = a_{j-1} +wrap 7_u8`, all visited by the same
sentence. Slots `1 + 9*4 = 37`; `p = o9 - 255` has one degree-1 monomial so group
3 contributes 0. `37 > 32`, the same hard error at both versions (row 9, class
(a)). Cap arithmetic re-checked: `1 + 4*8 = 33 > 32`, `1 + 4*7 = 29`. IDENTICAL.

## Every drafted trace, under the repaired term-set / monomial definitions

| trace | terms / slots | certificate | verdict |
| --- | --- | --- | --- |
| I1 midpoint | elim `q, hi, lo`; 0 + 4 + 6 = **10** | `sigma(q)=H1`: `2*p0 - H1 = lo - hi + 2`, `s=2`; `sigma(hi)=H3`: `- (lo-hi+1) = 1`; `lo` skipped | `floor(1/2)=0` VERIFIED |
| I2 base | elim `sum, i`; empty | `RELAX = 0` | VERIFIED |
| I2 step | elim `o, i, sum`; 1 + 2 + 6 = **9** | `o` skipped; `sigma(i)=H1`: `255*p0 - 255*H1 = 255*o - 65025`, `s=255`; `sum` coefficient 0, skipped | `255*255 - 65025 = 0`, `floor(0/255)=0` VERIFIED |
| I3 base | term set `{acc, i, factor}`, elim `{acc}` only (`i*factor` is degree 2) | empty | `C=0` VERIFIED |
| I3 step | elim `{acc}`; 1 slot | `sigma(acc)=H1` -> `0` | VERIFIED |
| I3 + `set factor = load(cfg,i)` | `factor` **is** an `[ENT-2]` term of `p`, so it is in the term set, the commit is visited, clause (e) on a `set` destination | - | **REFUSED**, and the file now says so |
| I4 base | elim `hits, i`; empty | `RELAX = 0` | VERIFIED |
| I4 step, matching | elim `o, i`; 1 + (2 `ieq` + 4 + 2) + 2 = **11** | `sigma(o)=E1`: `p0 - E1 = hits - i`; `sigma(i)=H1`: `- H1 = 0` | VERIFIED |
| I4 step, non-matching | elim `hits, i`; 1 + 1 + 2 + 2 = **6** | `sigma(hits)=H1` -> `-1` | `floor(-1/1) = -1` VERIFIED |

Largest intermediate anywhere: `65,025` (I2's step), multiplier `s = 255`; T8's
refusal at `255*745 = 189,975`. Thirty-three orders of magnitude below `2^127`,
exactly as 3.9.7's F-I1c bullet states from execution. The term-set /
elimination-term paragraph moves no trace: it only writes down the reading I3's
own count already used.

## Part (v), restated in my words, and attacked

*The five parts.* (i) the term set, the visit set, `p` and the elimination-term
list are outputs of one syntactic computation, so the space's **shape** is fixed;
(ii) every slot position is an output of the same computation, so the space's
**size** is fixed and neither cap can be crossed - a strengthening can only fill
an empty slot or tighten a filled one; (iii) tightening makes a hypothesis
polynomial larger, hence `-|a|*h` smaller, hence the residual smaller, and
filling is covered by the *omitting* certificate, which lies in the same space;
(iv) `RELAX` is monotone in each factor interval and the test is monotone in
`RELAX`; (v) the **work** inside the fixed space is not fixed - a fill turns a
skipped step into an executing one - but an executing step raises nothing, and
`sigma'` (`sigma` minus every term whose slot went empty-to-filled) is in the
same space, skips exactly where `sigma` skipped, and runs only steps against
slots filled at both, so it reaches a residual no larger than `sigma` reached at
the weaker version.

*My new attack, the one the charter names: a discard that loses the only
succeeding certificate while the omitting certificate ALSO fails.*

Suppose at the weak version `sigma` is the **only** succeeding certificate, and
at the strong version some slot in `sigma`'s domain goes empty-to-filled so
`sigma`'s residual moves and it fails. Then `sigma != sigma'`. But `sigma'` is
`sigma` with terms dropped whose slots were **empty at the weak version**, and a
step against an empty slot is skipped - so at the weak version `sigma` and
`sigma'` execute *the identical step sequence* and reach the identical residual.
So `sigma'` also succeeded at the weak version, contradicting "`sigma` is the
only one". The attack cannot be set up. And if no slot in `sigma`'s domain moved,
`sigma' = sigma` and every step it runs uses a slot that is same-or-tighter, so
it still succeeds by (iii). **The lemma covers it.**

Three ways I tried to break the covering and all fail:

1. *Make a tightening move a coefficient rather than a constant.* Group 2's
   constant-bound slots are `t - cu <= 0` / `cl - t <= 0` and group 3's are
   `t1 - t2 - c <= 0` - coefficients pinned to `+1`/`-1` by their own form; group
   1, the (b)/(d) pairs and the path conditions are fixed polynomials. `d2 -> d1`
   moves one member's constant from `k-1` to `0` with the same coefficients. So
   `sigma'`'s coefficient stream, and therefore every subsequent `a`, every
   `a*b` sign test and `s` itself, is bit-identical at both versions; only the
   constant term of `p` differs, and it differs downward. No attack.
2. *Empty a filled slot.* A constant-bound slot's `cl` is bounded by its term's
   type width (`< 2^64`), a difference bound's `c` likewise, and `[IND-8]` never
   publishes an over-magnitude constant (row 14) - so row 11's discard cannot be
   triggered *by tightening*. Filled stays filled. No attack.
3. *Make `RELAX` worse.* `RELAX` reads the **tightest** bound over the state and
   the filled slots, so adding a filled slot is a source added to a `min`. A
   newly filled non-constant-bound slot (a pair) matches neither `Z - t <= -cl`
   nor `t - Z <= cu` and supplies no interval at all. Monotone. No attack.

Part (v) holds. Where the round is refuted is elsewhere - see BREAK-bounds.md.
