# Traces re-executed under 081043ce, and the bound re-derived

## [IND-7]'s bound, re-derived independently

Premise: every polynomial a step reads is [IND-3]-limited. Filled slots are
exactly: group 1 statement polynomials (shape, limited at their own admission);
group 2 constant bounds (prover fact -> row 11 discard if not normalizable);
group 2 clause (b)/(d) pairs (shape, now limited); group 2 path conditions
(shape, now limited); group 3 difference bounds (fact -> discard). No third
kind. **Premise restored.** (It holds under either reading of the drop question
in BREAK-drop.md, because a dropped condition never fills a slot.)

- k <= 4 (4 elimination terms, sigma injective).                         OK
- degree: |b|p - |a|h, both operands <= 4, so <= 4, never rises.         OK
- monomials: <= 256 + k*256 = 256*(k+1) = 1280 = 5*256.                  OK
- coefficients: |b| <= M, |a| <= C, |h| <= M  =>  C' <= MC + CM = 2MC.
  Each step multiplies by <= 2M = 2^128 (the round-3 "doubles" wording is
  correctly repaired). From C = M: (2M)^k*M <= (2M)^(k+1) = 2^640.       OK
- s = product of <= 4 |b| <= M^4 = 2^508.                                OK
- RELAX: type set is closed at i8..u64 (spec 812), so every endpoint is
  <= max(u64) < M; slot-borne endpoints <= M by [IND-3]/row 11. Corner
  product <= 2^640 * M^4 = 2^1148; 1280 < 2^11 of them  =>  < 2^1159.    OK

## N4 re-run under the final text

`E1 = o - hits - x^8 <= 0` is a clause (b) pair member, present in shape at
every visited clause (b) commit. Degree 8 > 4 crosses row 3. Implementation A
(exact) and implementation B (sized to the published bound) both reject at
[IND-3], naming the statement. **One answer, from the text.** The A/B split is
closed.

## I1's midpoint, hand-executed

p = mid - hi + 1; clause (a) at `mid = lo +checked half` -> lo + half - hi + 1;
clause (d) at `half = span / 2`, d1 (Z - span <= 0 from u64) -> q, pair
`2q - span <= 0`, `span - 2q <= 1`; clause (a) at `span = hi -checked lo`
rewrites both -> H1 = 2q - hi + lo, H2 = hi - lo - 2q - 1.
p0 = lo + q - hi + 1. Terms q, hi, lo (3). Slots 0 + 4 + 6 = 10; space
1 + 3*10 + 3*90 + 720 = 1021. CONFIRMS the file's 1,021.
sigma(q)=H1: a=+1,b=+2 -> p := 2p0 - H1 = (2lo+2q-2hi+2) - (2q-hi+lo)
  = lo - hi + 2; s := 2.
sigma(hi)=H3=lo-hi+1: a=-1,b=-1 -> p := p - H3 = 1; s := 2.  lo skipped.
floor(1/2) = 0 <= 0. VERIFIED. Every shape (p0, H1, H2, H3) is degree 1,
<= 4 monomials, coefficients <= 2; the widened scope is free here.

Aside: d2 would give H1 = 2q - hi + lo - 1 and residual 2, floor(2/2) = 1 > 0,
REFUSED. So the d1/d2 flip is prover-driven and moves refuse -> verify, the
permitted direction; part (iii)'s tightening case covers it.

## I4, both paths, hand-executed

Matching: p0 = o - i - 1. sigma(o)=E1=o-hits-1: p := p0 - E1 = hits - i, s=1.
sigma(i)=H1=hits-i: a=-1,b=-1 -> p := p - H1 = 0. floor(0/1)=0. VERIFIED.
Slots 1 + (2 ieq + 4 + 2) + 2 = 11; space 1 + 2*11 + 110 = 133. CONFIRMS.
Non-matching: p0 = hits - i - 1; sigma(hits)=H1: p := p0 - H1 = -1;
floor(-1/1) = -1 <= 0. VERIFIED. Slots 1 + 1 + 2 + 2 = 6. CONFIRMS.
I2 step re-checked: 255*p0 - 255*H1 = 255o - 65025; 255*255 - 65025 = 0;
floor(0/255) = 0. CONFIRMS.
