# N3: the shape rule closes the visit set; the certificate ARITHMETIC is still the prover's

Re-verification of 6e06911b on `batch/0106-claim-model-design`.
Verdict: **REFUTED**. `[ENT-1]` monotonicity is false at 6e06911b by a fourth
mechanism, in a place the four-part *Monotonicity* paragraph never looks.

## The two sentences that collide

DESIGN.md 2288, inside `[IND-7]`, immediately after the certificate step rule:

> Every quantity is a mathematical integer and `[IND-3]`'s magnitude limit
> applies at every step.

DESIGN.md 1845-1847, `[IND-3]`:

> Three spec-fixed limits apply at every step and each violation is **a hard
> error naming the statement**: coefficient or constant magnitude at most
> `2^127`, degree at most 4, and at most 256 monomials.

And DESIGN.md 2278-2280, the step rule itself:

> If `sigma(t)`'s slot is **empty**, or `a` is zero, or `a*b <= 0`, the term is
> **skipped** and `p` and `s` are unchanged.

So **which steps are executed is a function of which slots the ambient prover
filled**, and a step's arithmetic is `p := |b|*p - |a|*h`, which multiplies `p`'s
coefficients by `|b|`. `|b|` is a hypothesis coefficient, and a clause (b) pair
on a `wrap` by a literal carries that literal as a coefficient. A strengthening
that fills such a pair therefore turns a *skipped* step into an *executed* one
whose coefficients can cross `2^127`, and `[IND-3]` makes that a hard error
naming the statement.

The theorem's own closing sentence (2346-2349) is the one that dies:

> by (i) and (ii) the certificate space is *literally the same set* at both
> versions, and neither cap is a count the prover contributes to, so **no hard
> error appears that was not already there**

The 32-slot and 4-term caps are indeed not counts the prover contributes to -
the shape rule closed that, and `N1`/`N2` confirm it. `[IND-3]`'s magnitude
limit is not a count at all. It is a hard error on the *value* of an
intermediate the prover decides whether to compute.

`[IND-8]` shows the author knew the shape of this hazard and gave the analogous
rule the guard `[IND-7]` lacks (2492-2494): "A published constant whose
magnitude exceeds `[IND-3]`'s limit is **not published** ... publication never
raises a hard error."

## Attack N3, hand-executed

```whitefoot
// K = 15000000000000000000_u64, a legal u64 literal (max u64 = 18446744073709551615)
let w1 = imin(k1, 1_u64);
let w2 = imin(k2, 1_u64);
let acc = 0_u64;
for @l i in 0_u64..n {
  bound @l s: ile(acc, i + w1 + w2);
  let g1 = w1 *wrap 15000000000000000000_u64;   // C1  clause (b), let binder
  let g2 = w2 *wrap 15000000000000000000_u64;   // C2  clause (b), let binder
  if ilt(g1, g2) { }                            // puts g1, g2 in the term set
  set acc = acc + 1;                            // C3  clause (a)
}
```

The strengthening is exactly 2.4's own example: **v0.40 lacks `imin`'s row
image, v0.41 has it**, so v0.41 derives `w1 - Z <= 1` and `w2 - Z <= 1` in the
body and v0.40 has only `[ENT-2]` 2905's implicit `u64` bound. Strictly more
facts, none fewer.

*Base* `[IND-5]`: post-capture `acc = 0`, `binder = 0`, group 1 empty (`s` is
the only `bound_stmt` and `[IND-5]` excludes it). `p0 = acc - i - w1 - w2`;
`RELAX` = 0 + 0 + 0 + 0 = 0; `floor(0/1) = 0 <= 0`. Verified at both versions.

*Step* `[IND-6]`, one body path taken. **The visit set and the slot list are
identical at both versions** - the shape rule works:

```
term set: {acc,i,w1,w2} -> C3 clause (a) -> {acc,i,w1,w2}
          branch condition ilt(g1,g2) adds {g1,g2}
          C2 clause (b): g2 <- o2 ; P = K*w2 puts w2 back (already in)
          C1 clause (b): g1 <- o1 ; P = K*w1 puts w1 back (already in)
p0 = acc - i - w1 - w2
elimination terms: acc, i, w1, w2      -- FOUR, at the cap, no witness in p0
H slots: (1) 1   H1 = acc - i - w1 - w2
         (2) 1   path condition, substituted to  o1 - o2 + 1 <= 0
             4   C1: two constant bounds on g1, and the pair
                   Q1a = o1 - K*w1 <= 0 ,  Q1b = K*w1 - o1 <= 0
             4   C2: two constant bounds on g2, and the pair
                   Q2a = o2 - K*w2 <= 0 ,  Q2b = K*w2 - o2 <= 0
         (3) 12  ordered pairs over four terms
         --- 22 slots, <= 32, IDENTICAL at both versions
```

**v0.40.** The `*wrap` side condition is `c1 * c2 <= max(u64)` with `c2 = K`.
The only bound on `w1` is `max(u64)`, and `max(u64) * K` is far past `max(u64)`,
so **Q1a/Q1b and Q2a/Q2b are all four EMPTY** (present in shape, empty in
content - which is the repair working as designed). Certificate
`sigma(acc) = H1`: `a = +1`, `b = +1`, `p := 1*p0 - 1*H1 = 0`, `s = 1`,
`RELAX(0) = 0`, `floor(0/1) = 0 <= 0`. **VERIFIED.** Every other assignment in
the space either skips (empty pair slots, or `b = 0` on the constant-bound and
path-condition slots, whose only terms are `o1`, `o2`, which are not elimination
terms) or uses a slot whose coefficients are `+1`/`-1` (H1, the twelve group-3
difference bounds). **No step's coefficient ever exceeds 1 in magnitude, so no
`[IND-3]` limit is approached.**

**v0.41.** `w1 - Z <= 1` derives, `1 * K = 15000000000000000000 <= max(u64)`, so
all four pair slots FILL. `sigma(acc) = H1` still succeeds, exactly as
*Monotonicity* (iii) promises. But the space also contains
`sigma = {w1 |-> Q1a, w2 |-> Q2a}`, and the rule says the predicate is
"discharged exactly when **some** certificate succeeds", so the space is what is
evaluated:

```
order: acc (outside sigma, skipped), i (outside sigma, skipped),
t = w1 : a = -1 in p, b = -K in Q1a, a*b = K > 0  -> admitted
         p := K*p0 - 1*Q1a
            = K*acc - K*i - K*w1 - K*w2 - (o1 - K*w1)
            = K*acc - K*i - K*w2 - o1                      ; s := K
         largest coefficient K = 1.5e19  <  2^127          OK
t = w2 : a = -K in p, b = -K in Q2a, a*b = K^2 > 0 -> admitted
         p := K*p - K*Q2a
            = K^2*acc - K^2*i - K^2*w2 - K*o1 - K*o2 + K^2*w2
            = K^2*acc - K^2*i - K*o1 - K*o2                ; s := K^2
         K^2 = 225000000000000000000000000000000000000
         2^127 = 170141183460469231731687303715884105728
         K^2 > 2^127   ->  [IND-3] MAGNITUDE VIOLATION
```

`[IND-3]`: "each violation is a hard error naming the statement". `[IND-7]`:
"`[IND-3]`'s magnitude limit applies at every step."

**A program v0.40 compiles does not compile on v0.41.** No cap count moved, no
slot appeared or vanished, no refusal flipped, the visit set is identical, and
the succeeding certificate of v0.40 still succeeds at v0.41. The theorem is
still false.

## The reading that saves it, and why the text does not have it

Three readings of "applies at every step" are available and the file picks none:

1. **Every assignment in the space is evaluated and an overflowing step is a
   hard error.** That is the plain reading of 2288 + 1845-1847, and it is the
   break above.
2. **Evaluation stops at the first succeeding certificate.** Then whether the
   hard error fires depends on the order an implementation enumerates the space
   - an implementation-chosen strategy, which is the `[ENT-1]` determinism half
   the whole certificate form exists to satisfy (2.4 property 4).
3. **An assignment whose step would exceed the limit is discarded, and the
   predicate is unaffected.** Sound, monotone, and the right rule - it is
   `[IND-8]`'s "not published ... never raises a hard error", one rule earlier.
   **The file does not say it.**

Reading 1 refutes the theorem; reading 2 refutes determinism; reading 3 has to
be written down. Two readings accept different programs, which is the same
defect class as A17's "until".

The same sentence carries the same hazard for `[IND-3]`'s **256-monomial** limit
(`p := |b|*p - |a|*h` unions the monomial sets, and whether `h` is added is the
prover's), though 2288 names only the magnitude limit - which is itself a gap,
since `[IND-3]` says all three limits "apply at every step".

## Sentences that must move

- 2288 - needs `[IND-8]` 2492-2494's treatment: an assignment whose step would
  cross a `[IND-3]` limit is discarded, not a hard error; and say which of the
  three limits apply to certificate steps.
- 2346-2349 - "no hard error appears that was not already there" is unsupported
  while any hard error inside `[IND-7]` is reachable through a slot the prover
  filled. The four parts (i)-(iv) cover the visit set, the slot list, filling
  and tightening, and `RELAX`; there is no part covering the *arithmetic* of a
  step that was previously skipped.
- 5137-5144 (Q3) - "after F-I1 and F-I1b every one is also a limit on a
  **syntactic** count". `[IND-3]`'s magnitude limit as imported by `[IND-7]` is
  not a limit on a count at all, and whether it is crossed is the prover's.
- 4968-4977 (D1) - "its bounds are counts of slots and terms that `[IND-4]`'s
  shape rule computes from the program's own text" is true of the caps and false
  of the per-step limit that also lives in the space.
