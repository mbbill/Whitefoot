# N5: the widened [IND-3] reopens row 7, through clause (i)'s dropped condition

## The collision

Three sentences of the fourth repair, all new or newly load-bearing:

1. `[IND-3]`:1909-1913 - the limits cover "every **hypothesis shape carried into
   the head frame** ... and the `[IND-3]` polynomial of each `[IND-6]` clause (i)
   path condition, each after the same backward substitution", each violation a
   **hard error**.
2. `[IND-6]`(i):2178-2186 - "a path condition whose substitution refuses - **by
   clause (b)'s `set` refusal** or by clause (e)'s, the two are treated alike -
   is **dropped**", its slot present and **empty**.
3. Sweep row 7 / part (i) / 3.8.4 - clause (b)'s `set` refusal is the one
   prover-flippable decision in `[IND-4]`, and it is safe "because a
   strengthening moves it only from refusing to **admitting**, the direction
   `[ENT-1]` permits".

(3) was true while admitting could only *add a hypothesis*. After (1),
admitting also **submits a new polynomial to a hard-error limit**. Clause (b)'s
refusal is prover-driven; so, now, is whether a degree-8 path condition is
measured. Nothing in the file says whether a *dropped* condition's polynomial is
still normalized and limit-checked.

## N5, hand-executed. An ordinary program.

```whitefoot
fn tally(x: own u64, n: own u64) contract { requires ile(x, 1_u64); } {
  let hits = 0_u64;
  let t = 9_u64;
  for @l i in 0_u64..n {
    bound @l counted: ile(hits, i);
    let w = imin(t, 1_u64);                 // C0 clause (e), let binder: witness
    set t = t -wrap w;                      // C1 clause (b), SET destination
    let m1 = x * x;                         // C2 clause (a)   (x <= 1)
    let m2 = m1 * m1;                       // C3
    let m3 = m2 * m2;                       // C4   m3 = x^8
    let c = ilt(m3, t);                     // C5
    if c { set hits = hits +wrap 1_u64; }   // C6 clause (b)
  }
}
```

Legal: `x <= 1` discharges every exact `*`; the two `wrap`s have no domain
obligation; `ilt` compares u64 with u64. `imin` is spec 774.

**True-path step.** Binder shift `p = hits - i - 1`. Backward pass, end to entry:

- C6 clause (b): `hits <- o`, `p0 = o - i - 1`; two constant bounds on `hits`,
  pair `E1 = o - hits - 1 <= 0`, `E2 = hits + 1 - o <= 0`, **filled** exactly as
  I4's are, through the published `hits - i <= 0` with `i < n <= max(u64)`.
- the branch: clause (i) takes `ilt(m3, t)` -> `m3 - t + 1 <= 0`; `{m3, t}`
  enter the term set.
- C4, C3, C2 clause (a), unconditional: `m3 -> m2*m2 -> m1^4 -> x^8`. The
  condition is now `x^8 - t + 1 <= 0`.
- C1: `t` is in the term set, so the commit is visited. `-wrap`'s side condition
  is `b - a <= 0`, i.e. **`w - t <= 0`**.

Elimination terms of `p0`: `o`, `i` - two. Slots: 1 (H1) + 1 (path condition)
+ 4 (C6) + 4 (C1) + 2 (C0) + 2 (ordered pairs) = **fourteen**. No cap fires, at
either version.

**Weak checker** (no `imin` row image, exactly `N3`'s device): `w - t <= 0` is
not derivable, the destination is a `set` destination, so **clause (b) refuses**;
the refusal is met while substituting a *path condition*, so by `[IND-6]`(i) the
condition is **dropped**, its slot present and empty. The statement's own chain
is untouched: `sigma(o) = E1` gives `p := p0 - E1 = hits - i`, `sigma(i) = H1`
gives `0`, `RELAX(0) = 0`. **VERIFIED.**

**Strong checker** (`imin` row image gives `w - t <= 0`): no refusal, the
condition is **kept**, and it is carried into the head frame as
`x^8 - o_t + 1 <= 0` - **degree 8**. Rows 2-4 now measure it. **HARD ERROR at
`[IND-3]`**, naming the statement and the degree limit.

**A strengthening turns a verified statement into a hard error.** That is
`[ENT-1]` monotonicity, broken by this round's own repair, on the one decision
the file had already identified as prover-flippable and cleared under the old
scope.

## The other reading, and why the file does not choose it

Read instead "every polynomial `[IND-4]`'s shape rule **produces**" (`[IND-3]`'s
lead sentence, and the `N4` block's "crosses row 3 **at the moment the shape rule
produces it**"): then the dropped condition is measured too, both versions hard-
error, and there is no break. Both readings are in the same paragraph. A third
is available - treat an over-limit shape as row 11's discard - and gives a third
verdict. Nothing in the file picks one.

The repair is one sentence: say that the limits are measured on every shape the
pass produces, **whether or not the condition is later dropped and whether or
not the slot fills**. That also matches the `N4` block, which already argues the
class-(a) case from production rather than from filling. Note `[IND-7]`'s
*resource* premise survives either way, since a dropped condition never fills a
slot - the damage is confined to monotonicity.
