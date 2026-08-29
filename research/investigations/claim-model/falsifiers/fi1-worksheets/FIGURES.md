# Every figure re-derived, and every count re-run

## The pass (row 12) - the new figure, derived independently

Substituted right-hand sides: clause (a) `a+b`, `a-b`, `a*b` and their Ok arms
(<= 2 monomials, degree <= 2, literal coefficients < 2^64); clause (c)'s atom;
clauses (b)/(d)/(e)'s single witness. So "at most 2 monomials of degree at most 2
with coefficients under 2^64" is right.

Replacing degree-1 `t` in a polynomial inside the limits (<= 256 monomials,
degree <= 4, coefficients <= M = 2^127): a monomial carries `t^e`, `e <= 4`;
`R^e` has <= `2^e <= 16` monomials of degree <= `2e`, total degree
<= `(4-e) + 2e = 4 + e <= 8`; its coefficient <= `C(4,2)*(2^64)^4 = 6*2^256 < 2^259`.
So <= `256*16 = 4096` monomials, degree <= 8, coefficients
<= `2^127 * 2^259 = 2^386`, and after collecting <= `2^12` of them, `< 2^398`.
**All four numbers confirm.** Step count = commits x shapes carried
(1 + 2 per branch condition + 4 per visited commit) - a count of the path's text.

## The certificate (row 18)

`k <= 4`; degree never rises (both operands <= 4); monomials
`<= 256*(k+1) = 1280 = 5*256`; a step replaces `C` by `|b|C + |a|M <= 2MC`, so
`(2M)^k * M <= (2M)^(k+1) = 2^640`; `s <= M^4 = 2^508`. **Confirms.**

## RELAX (row 20) and the corner minimum (row 23)

Type set closed at `i8..u64`, so every endpoint `<= max(u64) < M`; slot-borne
endpoints `<= M`. Degree-4 corner product `<= 2^640 * M^4 = 2^1148`; `1280 < 2^11`
of them `=> < 2^1159`. **Confirms.** Corner minimum from the statement as written
(coefficients `<= M` by row 7): `>= -(M * M^4) = -2^635`, 256 of them `> -2^643`.
**Confirms, and this closes the round-4 asymmetry finding.**

## Counts

- space at the caps: `1 + 128 + 5952 + 119040 + 863040 = 988,161`
- 16-slot space: `1 + 64 + 1440 + 13440 + 43680 = 58,625`; `16*15*14*13 = 43,680`
- I1 midpoint: 3 terms, 10 slots -> `1 + 30 + 270 + 720 = 1,021`
- I2 step: 3 terms, 9 slots -> `1 + 27 + 216 + 504 = 748`
- I4 matching: 2 terms, 11 slots -> `1 + 22 + 110 = 133`; non-matching 6 slots
- N2: `1 + 9*4 = 37 > 32`; cap binds at 8 links (`1 + 32 = 33`), 7 fit (29)
- N3: `1 + 1 + 4 + 4 + 12 = 22`; `K^2 = 2.25e38 > 2^127 = 1.70e38`
- N4: 2 terms, `1 + 4 + 2 = 7` slots; `C(19,3) = 969 > 256`
- N5: `1 + 1 + 4 + 4 + 2 + 2 = 14`
- A2: `255*(1000 - 255) = 189,975`

## Traces

I1 midpoint: `2*p0 - H1 = (2lo+2q-2hi+2) - (2q-hi+lo) = lo - hi + 2`, `s = 2`;
`- H3 = 1`; `floor(1/2) = 0`. VERIFIED.
I2 step: `255*p0 - 255*H1 = 255o - 65025`; `255*255 - 65025 = 0`; `floor(0/255) = 0`.
I4 matching: `p0 - E1 = hits - i`; `- H1 = 0`; `floor(0/1) = 0`.
I4 non-matching: `p0 - H1 = -1`; `floor(-1/1) = -1`.
**Every number in the file confirms.**

## L3 completeness - is every computation in a row?

path enumeration (row 8), pass + intermediates (12), normalization (9-11),
fact-normalizability (19), certificate space (21, figure at 3.9.1), one
certificate's steps (18), RELAX (20), projection (22-23). The only cost with no
figure is the **number of certificates actually evaluated**, which the file
itself names as the one unmeasured claim left (11.5, Q3), and it is a pruning
argument over a space whose size is published. **L3 holds.**
