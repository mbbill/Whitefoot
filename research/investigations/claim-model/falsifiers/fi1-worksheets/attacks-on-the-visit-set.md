# Three attacks on the repaired visit-set rule - all three FAIL to break it

## A1 - a destination reached only through an EMPTY slot's terms

```whitefoot
let a0_seed = 0_u8;
for @l i in 0_u64..n {
  bound @l s: ile(x, 255_u16);        // x is u16, so a real fact
  let a0 = load(buf, i);              // C0  clause (e), LET binder -> no refusal
  let y  = a0 +wrap 7_u8;             // C1  clause (b), LET binder
  let z  = cvt<u8, u16>(y);           // C2  clause (c)
  set x  = z;                         // C3  clause (c)
}
```

Weak prover: `a0`'s only bound is u8's 255, `255 + 7 = 262 > 255`, so C1's pair
is **empty**. Its SHAPE `o - a0 - 7 <= 0` still puts `a0` in the term set, so C0
is visited anyway. Strong prover deriving `a0 <= 100`: `107 <= 255`, pair
**filled**, `a0` in the term set by the same sentence, C0 visited.

| | weak | strong |
| --- | --- | --- |
| visited | C3, C2, C1, C0 | C3, C2, C1, C0 |
| slots | 1 stmt + 4 (C1) + 2 (C0) = **7** | **7** |
| `p0` | `o - 255` | `o - 255` |
| elimination terms | `{o}`, one, no ordered pair | same |
| certificate | `sigma(o) = (o - 255 <= 0)` -> `p := 0` | same |
| verdict | VERIFIED | VERIFIED |

Only the *contents* moved (`oa - 255 <= 0` becomes `oa - 100 <= 0`, a
tightening; the pair fills). This is (ii) and (iii) working. **No break.**

## A2 - the `[IND-6]` clause (i) route, witness-introducing path condition

```whitefoot
let acc = 0_u8;
for @l i in 0_u64..n {
  bound @l s: ile(x, 255_u16);
  set acc = acc +wrap 3_u8;        // C0 clause (b) on a SET destination
  let c   = ilt(acc, 200_u8);
  if c { set x = 0_u16; } else { set x = 1_u16; }
}
```

Weak prover: at C0 the side condition needs `acc <= c1`, `3 <= c2`,
`c1 + c2 <= 255`; `acc`'s only bound is 255, so it does not derive, the
destination is a `set` destination, and the substitution **refuses**. Because
this commit is reached only through a path condition, `[IND-6]` (i) **drops** the
path condition instead of refusing the statement.
Strong prover deriving `acc <= 100`: `103 <= 255`, admits, witness `o_acc`, the
path condition substitutes and its slot **fills**.

The route the round-one report flagged as the unflagged fourth instance is the
one that decides this: does the pass reach PAST a refusing commit? Two repaired
sentences answer, and they agree:

- `[IND-4]` clause (b): "`P`'s terms enter the term set, **at every clause (b)
  commit the pass visits**" - unconditional, refusal included.
- `[IND-6]` (i): "every commit it thereby reaches is visited and contributes its
  slots in shape, **whether or not the substitution refuses and whether or not
  the condition is dropped**; a dropped path condition's slots are present and
  empty."

So `acc` enters the term set at C0 at both versions, any earlier commit to `acc`
is visited at both, C0 contributes its four slots at both, and the path
condition's one slot is present at both - empty at the weak version, filled at
the strong. Empty -> filled is (iii). **No break.** Residual: `[IND-7]` concedes
that a refusal "left no witness term to bound", which makes the pair's shape
`o - P <= 0` name an `o` that does not exist. Harmless today (the slot is empty
either way and `P`'s terms enter by clause (b)'s own sentence), but the two
sentences do not agree on whether a refusing commit produces a witness.

## A3 - a destination occurring only in a degree-2 monomial

```whitefoot
for @step i in 0_u64..n {
  bound @step running: ile(acc, i * factor);
  set factor = load(cfg, i);       // destination occurs in p ONLY inside i*factor
  set acc = acc + factor;
}
```

This does not move with the prover - it moves with the READING, and the file
never fixes it. `[IND-4]`'s term set "starts as the **terms** of the polynomial";
`[IND-7]`'s elimination terms are "the **degree-1 monomials** of `p`". The file
uses both words and never says they are different sets, though I3's own trace
turns on their being different ("one elimination term - `i*factor` is degree 2").

- Reading A (term = `[ENT-2]` term, i.e. an atom of a monomial; this is what
  `[IND-3]` means by "over `[ENT-2]` terms"): `factor` is in the term set,
  `set factor = load(cfg,i)` is visited, clause (e) fires on a `set` destination,
  the statement is REFUSED.
- Reading B (term = elimination term): `factor` is not in the term set, the
  commit is not visited, `p` keeps a `factor` that the body has overwritten, and
  the statement can VERIFY on a head-frame claim that is false. **Unsound.**

Two readings accept different programs and one is unsound, which is A17's defect
class exactly. Reading A is the right one and `[IND-3]`'s wording supports it,
but the shape rule's whole force rests on it and no sentence states it. **Not a
monotonicity break; a determinism/soundness under-specification.**

## What did break: N3

See `BREAK-magnitude.md`. The visit set is closed. The break is one level
further in: **which certificate STEPS execute is still the prover's**, and
`[IND-7]` imports `[IND-3]`'s per-step magnitude hard error into that space.
