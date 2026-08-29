# Round-6 check of c14f4793 (wording-only residual pass)

State: worktree clean, HEAD == origin/batch/0106-claim-model-design ==
c14f4793. One file changed, +150/-38. Diff read in full (`diff6.txt`, -U6).

## The ten items, each against the auditor's own statement

| # | auditor's item | site in the file | verdict |
| --- | --- | --- | --- |
| 1 | clause (d) clearance invalid; supply shared-second-member proof, add a row, fix two absolutes | 2003-2027, row 14a (2999), row 14 (2998), 3030-3038 | applied; proof transcribed correctly, see below |
| 2 | part (ii) undercounts (side condition also fills) | 2838-2844, six sites, "no seventh" | applied |
| 3 | L2's opener loose | 1872-1878 | applied |
| 4 | part (i) needs the check-ends carve-out | 2809-2822 | applied |
| 5 | four stale row citations | 4160 (7->14), 4167 (11->19), 4183 (15->23), 4184 (12->20); convention at 4227-4231 | applied |
| 6 | disequality missing from the no-polynomial list | 2046-2050 | applied |
| 7 | false edge undefined | 2346-2355 | applied; matches 3.9.5 |
| 8 | two-literal product exceeds 2^64 | 2635-2644 | applied; figure unchanged |
| 9 | [IND-10] depth vs size | 3628-3634 | applied |
| 10 | [IND-8.V] doubles every figure | 2655-2659 | applied |
| 11 | front-matter run-on | 24-26 | applied |

## Item 1, checked line by line against reverify5/SEAM.md

d1/d2 as [IND-4](d) writes them (2179-2184): d1 = `k*q - a <= 0`,
`a - k*q <= k-1`; d2 = `k*q - a <= k-1`, same second member. Normalizing,
d1 carries constants `C` and `-C-j`, d2 carries `C-j` and `-C-j`, and
`P2 = -P1 - j`. Magnitude: `|C| > M`, `|C-j| <= M`, `|C+j| <= M` gives
`|C| <= M`. Monomials: `N + [C!=0] = 257` with the other two `<= 256` forces
`C = j` and `C = -j`, so `k = 1` and the pairs coincide. Degree identical.
All four steps reproduce.

Cross-checks on the file's own traces: I1 (3670-3673) has `H1 = 2q - hi + lo`
and `H2 = hi - lo - 2q - 1`, and `-H1 - 1` is exactly `H2` (`j = k-1 = 1`).
B2 (3989-3991) prints `2q - a` against `2q - a - 1`, which is the monomial
difference the new paragraph cites.

## Row mechanics

Table is 25 rows, 24 numbered plus 14a, five columns each, no renumbering.
Every `row N` / `rows N to M` citation in the file re-checked against the
current table: all current. 3117's "twenty-five" is a slot count, not a row
count. 5389's "4.3 row 5" and 5857-5910's rows 40-49 are other tables.

## Acceptance

No rule's operative content changed. The one edit with operative reach is
item 7 (the false-edge negation and the `ieq` one-slot rule), which resolves a
point the previous text left undetermined - and it resolves it the way the
file's own worked case already counted: 3.9.5's I4 non-matching path (3849-3857,
unchanged) is six slots on an always-empty disequality slot, and 3.9.1:3131-3132
already said "eleven rather than ten because an `ieq` path condition now takes
one slot per `[IND-3]` polynomial". No program in the file changes verdict.

## Trace re-executed: I1's midpoint (3676-3685)

`p0 = lo + q - hi + 1`; `H1 = 2q - hi + lo`; `H3 = lo - hi + 1`.
t = q: a = +1, b = +2 -> `2*p0 - H1 = (2lo + 2q - 2hi + 2) - (2q - hi + lo)
= lo - hi + 2`, s = 2. t = hi: a = -1, b = -1 -> `(lo - hi + 2) - (lo - hi + 1)
= 1`. t = lo skipped. `floor(1/2) = 0 <= 0` VERIFIED. Ten slots
(2 constant bounds + 2 pair + 6 ordered pairs), 3 terms,
`1 + 30 + 270 + 720 = 1021`. Unchanged.

## Residuals (all wording, none acceptance-bearing)

1. **Row 9 (line 2993) still carries the replaced clearance**: "The one
   prover-selected constant in any shape is clause (d)'s `k - 1` against `0`,
   and both are under `2^64`" - which row 14a, six lines later, calls "invalid
   as reasoned". The head repair did not reach the row it clears.
2. **Line 2823**: "The only `[IND-4]` decision a strengthening can flip is
   clause (b)'s refusal on a `set` destination" - part (ii)'s own list (2840)
   names "clause (d)'s `d2 -> d1` constant", and row 14a says "A strengthening
   moves `d2` to `d1`". A third absolute of the class item 1 named.
3. **Part (i)'s headline and conclusion (2795-2796, 2809-2811)**: "the produced
   shapes ... are unchanged, and so is everything measured" / "the set of
   `[IND-3]` tests performed, and their answers, are identical at both
   versions". A clause (d) flip changes a produced shape's constant (`C-j` vs
   `C`); only the *answers* coincide, and only by the new shared-second-member
   argument, not by L1. The round carved out the check-ends case (item 4) and
   not this one.
4. **L2 (line 1873)** borrowed row 14's *old* form: "the only row ... a prover
   decision moves". Row 14 itself now says "moves **acceptance**"; without that
   word row 14a is a counterexample.
5. **Row 14a's law column reads "(b), L1"** while 3033 says "L1 alone does
   **not** clear row 14a". The column should name [IND-3]'s seam argument too.
6. **"the one/single prover-flippable decision"** survives in historical prose
   at 528, 2292, 2363, 2496, 3143, 6087, 6432, 6643, 6844 - now an undercount
   by one. The round stated a currency convention for row numbers (4227-4231)
   but not for this phrase.
7. Pre-existing asymmetry: clause (b) (2160-2161) names only "the statement's
   own polynomial" as ending the check, clause (e) (2194-2197) and part (i)'s
   new carve-out also name a carried witness hypothesis; clause (b) leans on
   "Clause (e)'s `set` refusal is read by the same two sentences".
