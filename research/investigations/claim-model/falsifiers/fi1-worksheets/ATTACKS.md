# Every prior attack re-run against ee730567

Each must land as one verdict, same at weak and strong provers, same at any
conforming implementation.

| attack | verdict under the final text | uniform? |
| --- | --- | --- |
| B1 vacuous base | refused: `[IND-5]` supplies no statement hypothesis, group 1 empty, group 3 empty (one term), every certificate is the empty one, `RELAX(idx) = 9`, `floor(9/1) = 9 > 0` | yes - no fact consulted |
| B2 signed division | d2 refuses `ile(h,-3)` at `floor(2/2) = 1`; companion `ige(h,-5)` verifies at `floor(-4/2) = -2` | yes; the d1/d2 flip is content, see SEAM.md |
| N1 visit set -> clause (e) | pair in shape puts `acc` in the term set, `set acc = load(...)` visited, clause (e) refuses at a `set` destination while substituting a **witness hypothesis carried into the head frame** -> ends at `[IND-1]` | yes |
| N2 visit set -> slot cap | nine visited clause (b) commits, `1 + 9*4 = 37 > 32`, row 17 | yes |
| N3 step arithmetic | verifies at both. `sigma'` drops the two empty-to-filled terms, leaving `sigma(acc) = H1`, `p := 0`. `K^2 = 2.25e38 > 2^127 = 1.70e38` is an ordinary intermediate; row 18 carries no limit | yes |
| N4 degree-8 pair | row 10 fires at the third clause (a) step, `E1 -> o - hits - x^8` | yes - pair is in shape |
| N5 degree-8 dropped condition | row 10 fires when the pass **produces** `x^8 - t + 1`, three clause (a) steps before C1's `-wrap` side condition is consulted. Hand-executed below | yes |
| N6 unbounded intermediate | refused within four substitution steps: `z -> u40 - w40` (deg 1), `-> w39^2` (2), `-> w38^4` (4), `-> w37^8` (8) = row 10. Nothing near `10^12` materialized | yes |
| FF2 (A2's exhaustive space) | `[IND-6]` removes the substituted polynomial from group 1 entirely, so the only hypothesis that would have worked is not in the space; residual `255*745 = 189975` | yes |
| FATAL-1 / A16 frame | head state, not exit state: `p = cursor`, `RELAX = 7`, refused | yes |
| A2 frame | group 1 is the statement **as written**: `255*p0 - 255*H1 = 189975`, refused | yes |
| A17 (one pass, never revisits) | `[IND-4]`'s sentence stands; term set vs elimination terms worked at 3.9.1:2961-2973 (`i*factor` is a term, not an elimination term) | yes |
| F-I1c #1: destination reached only through an empty slot | visited at both - pair terms enter the term set unconditionally | yes |
| F-I1c #2: clause (i) route with a witness-introducing path condition | worked at 3.9.1:2975-3011: `t - 4 <= 0` dropped by clause (e)'s refusal, slots present and empty, statement verifies | yes |
| F-I1c #3: destination only in a degree-2 monomial | clause (e) refuses; the visit rule reads the **term set** | yes |

## N5, hand-executed under L1

Backward over the true path: C6 `set hits = hits +wrap 1` (clause b) -> witness
`o`, `p0 = o - i - 1`, pair `E1 = o - hits - 1`, `E2 = hits + 1 - o`, `hits`
re-enters the term set. Branch: clause (i) normalizes `ilt(m3, t)` to
`m3 - t + 1 <= 0`; `m3`, `t` enter. C4 -> `m2^2 - t + 1` (deg 2, measured, passes);
C3 -> `m1^4 - t + 1` (deg 4, measured, passes); C2 -> `x^8 - t + 1` (**deg 8**,
measured, **row 10 hard error**). C1 is never reached, so `w - t <= 0` is never
asked and the weak/strong split never arises. Two elimination terms, fourteen
slots (1 H1 + 1 condition + 4 C6 + 4 C1 + 2 C0 + 2 pairs) - no cap fires first.
Same at both checkers. The file's account is exact.
