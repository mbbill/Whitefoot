# The seam, enumerated. Clause (d) is a second class-(b) decision.

The round-4 charter: enumerate every place a class-(b)/content decision sits
next to a class-(a) hard error, and check the hard error's inputs are
production-time in every one.

| # | class-(b) decision | nearest class-(a) error | verdict |
| --- | --- | --- | --- |
| 1 | clause (b) `set` refusal (row 14) | rows 9-11 via row 15's drop | **closed by L1** - measured at production; a refusal never truncates the pass; clause (e) is the only clause that ends one, on grammar + binder kind |
| 2 | clause (b)/(d)/(e) constant-bound fill | none - slots exist regardless | closed |
| 3 | clause (b) no-wrap side condition **filling the pair** | none - pair shape unconditional | closed, but **not in part (ii)'s five-item list** |
| 4 | group 3 difference-bound fill | none | closed |
| 5 | `RELAX` interval | row 20 has no limit | closed |
| 6 | **clause (d)'s `Z - a <= 0` sign test selecting d1 over d2** | **rows 9-11, directly** | **real seam, harmless - but the file's clearance is invalid and it has no sweep row** |
| 7 | `[IND-8]` publication (row 22) feeding the head state | rows 9-11 via 6 | closed, given 6 |

## The finding on 6

`[IND-3]`:1997-2002 clears it: "The one constant in any shape that a prover
decision selects is clause (d)'s `k - 1` against `0`; `k` is a literal of a
fragment integer type, so `|k - 1| < 2^64 < 2^127` and the magnitude test answers
identically whichever member applies. No ambient prover contributes a
coefficient, a degree or a **monomial** to any polynomial in this scope."

Both sentences are wrong as reasoning.

Write `C` for the constant of the normalized `k*q - a`, `j = k - 1`:

- d1 members: `P1 = k*q - a` (constant `C`), `P2 = a - k*q - j` (constant `-C-j`)
- d2 members: `P1' = P1 - j` (constant `C - j`), `P2` unchanged

`|j| < 2^64` does **not** make the magnitude test answer identically: `C` sits
where the pass's own arithmetic puts it, and shifting it by `j` can cross
`2^127`. And the monomial count differs outright whenever exactly one of
`C`, `C - j` is zero - the file's own B2 trace prints it: d1's `2q - a`
(2 monomials) against d2's `2q - a - 1` (3).

## Why it is nonetheless harmless: the shared second member

`P2` is the **same polynomial at both versions**, and `P2 = -P1 - j`.

*Magnitude.* A break needs d2 to pass and d1 to fail: `|C| > M`,
`|C - j| <= M` (d2's P1'), `|C + j| <= M` (the shared P2). The last two give
`|C| = |((C-j)+(C+j))/2| <= M`. Contradiction.

*Monomials.* Let `N` be the non-constant count, shared by all members. A break
needs `N + [C != 0] = 257` and `N + [C - j != 0] <= 256` and
`N + [C + j != 0] <= 256`. The first gives `N = 256, C != 0`; the second gives
`C = j`; the third gives `C = -j`. Hence `j = 0`, i.e. `k = 1`, where d1 and d2
coincide. Contradiction.

*Degree.* Identical in all members.

So no limit can flip - but the file proves none of this.
