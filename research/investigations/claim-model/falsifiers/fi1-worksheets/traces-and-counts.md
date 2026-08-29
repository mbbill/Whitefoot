# Every trace re-executed under the shape rule, and every count recomputed

## Traces

| trace | visit set under the shape rule | slots | certificate | verdict |
| --- | --- | --- | --- | --- |
| I1 midpoint (3.9.4) | `mid` (a), `half` (d, witness q + pair naming q,k,span), `span` (a) - reached because clause (d)'s pair shape names the dividend unconditionally | 0 + 4 + 6 = **10** | `sigma(q)=H1` -> `lo-hi+2`, `s=2`; `sigma(hi)=H3` -> `1` | `floor(1/2)=0` VERIFIED |
| I2 base (3.9.3) | none | 0 | empty | `C=0` VERIFIED |
| I2 step | `sum` (a, puts `sum` back), `wide` (c), `w` (e, witness o) | 1 + 2 + 6 = **9** | `sigma(i)=H1` -> `255*o - 65025`, `s=255` | `cu(o)=255`, `C=0` VERIFIED |
| I3 base | none | 0 | empty | `C=0` VERIFIED |
| I3 step | `acc` (a) | 1 | `sigma(acc)=H1` -> `0` | VERIFIED |
| I4 base (3.9.5) | none | 0 | empty | `C=0` VERIFIED |
| I4 step, matching | `hits` (b, `P=hits+1` puts `hits` back, no earlier commit); path condition puts `value` in, `let value` (e) | 1 + 1x2 + 4 + 2 + 2 = **11** | `sigma(o)=E1` -> `hits-i`; `sigma(i)=H1` -> `0` | VERIFIED |
| I4 step, non-matching | disequality's terms put `value` in, `let value` (e) | 1 + 1 + 2 + 2 = **6** | `sigma(hits)=H1` -> `-1` | `floor(-1/1)=-1` VERIFIED |
| T7 (A16/FATAL-1) | clause (c) only, no witness, no branch | - | every certificate empty | `cu(cursor)=7`, `7>0` REFUSED |
| T8 (A2/FF2) | clause (a) only | - | four, best is `255*p0-255*H1` | `189975 > 0` REFUSED |

The shape rule moves **no** trace: I1 has no clause (b) and clause (d)'s pair was
already unconditional in shape; I2/I3 have no clause (b) and no branch; T7/T8
have no (b), (d) or (e) commit at all; I4 gains nothing it did not already
visit. The file's own annotations to that effect are correct.

The two counts the repair changed are both right: I4's matching path is
**eleven** slots, not ten, because `[IND-3]` gives an `ieq` condition two
polynomials; and the non-matching path's six is `1 + 1 + 2 + 2`.

## Counts, recomputed independently

| figure | check |
| --- | --- |
| `sum_k C(4,k)P(32,k)` | `1 + 128 + 5952 + 119040 + 863040` = **988,161** OK |
| `sum_k C(4,k)P(16,k)` | `1 + 64 + 1440 + 13440 + 43680` = **58,625** OK |
| `16*15*14*13` | **43,680** OK |
| I1: 3 terms, 10 slots | `1 + 30 + 270 + 720` = **1,021** OK |
| I2 step: 3 terms, 9 slots | `1 + 27 + 216 + 504` = **748** OK |
| I4 matching: 2 terms, 11 slots | `1 + 22 + 110` = **133** OK |
| ordered pairs at 4/3/2/1 terms | `12 / 6 / 2 / 0` OK |
| "twelve + one + three commits at four" | `12 + 1 + 12` = **25** OK |
| "sixteen could not hold even one such commit beside the pair slots" | `12 + 1 + 4 = 17 > 16` OK |
| N2 | `1 + 9*4` = **37 > 32** OK |
| clause (b) chain cap | `1 + 4*8 = 33 > 32`; `1 + 4*7 = 29` OK |
| A2 residual | `255*1000 - 255*255 = 255*745` = **189,975** OK (the `190125` of both earlier revisions is wrong; both positive, refusal unaffected) |
| I2 publication | `255*999` = **254,745**; false edge `255*1000` = **255,000**; consumer `254745 + 255 = 255000` OK |
| I3 with the added `requires` | `999*1000` = **999,000** OK |
| 4.4/11.5 `halved` publication | `p = 2*half - length`; `length` solitary with `a=-1`, `r = 2*half`, corner minimum `0`, so `Z - length <= 0` - vacuous, correct; `half` has coefficient `+2`, outside `{+1,-1}`, and the difference-bound clause needs `b=-a=+1` against `+2`, so nothing else OK |

## The ten wording items

| # | item | status |
| --- | --- | --- |
| 1 | `190125` -> `189975` (3.8.3 x2, plus the worksheet noted in 12.4 and 3.9.7) | FIXED, arithmetic re-checked |
| 2 | `[IND-5]`'s B1 cross-reference `3.9.5` -> `3.9.7` | FIXED |
| 3 | 2.4 bullet 2's "seven" vs the enumeration | FIXED - the list now names seven, and 12.4 states the counting rule ("one `bound_stmt` obligation is one derivation ... I4's step is one derivation over two body paths"). The list matches the rule. **Residual: the parenthetical still says "reconciles seven with the eight lines above" and there is no longer an eight-item list above** |
| 4 | 4.5.2's Score "four corrected rows" | FIXED - "five rows corrected, covering six of the eleven sites, since the third row carries two"; consistent at 964, 4231-4234 and 4857 |
| 5 | `[IND-7]` group 2: how many slots an `ieq` path condition takes | FIXED - "two for an `ieq` condition, one for any other `rel_term`, and one slot for a branch condition `[IND-3]` gives no polynomial at all"; I4's count moved 10 -> 11 with it |
| 6 | `[IND-7]` group 2: "where that clause applies" | FIXED - "at every such commit governed by clause (b) or clause (d)" |
| 7 | `[IND-6]` closing enumeration omitting clause (d1)'s sign test | FIXED - "a clause (d1) sign test `Z - a <= 0`" is named |
| 8 | 4.4 / 11.5 "publishes nothing" | FIXED - "the vacuous `Z - length <= 0`", derivation checked above |
| 9 | 2.4 bullet 2 and 3.9.1's superset paragraph | FIXED - both now say the superset is over a fixed hypothesis list and that `[IND-5]`/`[IND-10]` lose B1's vacuous bases deliberately |
| 10 | *Monotonicity* (iii)'s false justification | FIXED - it now argues from the omitting certificate and says explicitly that a certificate naming the newly-filled slot may move |

All ten are corrected. **Residual: 3.9.7's F-I1b bullet says "Ten wording items"
and enumerates nine** - item 10 above is missing from that list, which is the
same count-vs-enumeration slip item 3 was.

## Compiled evidence spot-check (gate binary)

`probes/f2_sdiv_consumer.wf` accepts; `f3_sdiv_false_bound.wf` rejects
`[FN-8] instantiated_goal: "ile(h, -3_i64)"`; `f4_sdiv_interval.wf` rejects
`[OP-2] residual: "h +defined 2_i64"`. Exactly what 12.4 records. The diff adds
no new compiled claim (its one new ledger row, `N1`/`N2`, is correctly marked
"hand-derived, not compiled").

## Diff scope, 767c4823..6e06911b

One file, +475/-151, 32 hunks. Every hunk is the refutation or a necessary
consequence: `[IND-4]`'s term set and shape rule and its clause (b)/(d)
rewrites; `[IND-6]` (i)'s dropping sentence; `[IND-7]`'s group 2, both caps and
the *Monotonicity* four parts; `N1`/`N2` in 3.9.1; the I1/I4 trace annotations
and I4's 10 -> 11; 3.8.4's new paragraph and the header list at :51; 3.9.7's
F-I1b entry and the F-I2 seeds; the ten wording corrections; and the downstream
bookkeeping in 2.4, 4.4, 4.5.2, 8 (B0), 9 (D1), 10 (Q3), 11.5, 12.4. **Nothing
out of scope found.** The F-D1 "extend it twice" hunk and the 12.4 counting-rule
paragraph are consequences of items the refutation raised.
