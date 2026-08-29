# N1 and N2 re-executed against 6e06911b

Weak prover = v0.40, type bounds only. Strong prover = v0.41, additionally
derives `<= 100` on the loaded byte (dropped `ensures`, or a sharper interval
domain). Both are strengthenings in the class 2.4 and F-I2 define.

## N1

Body: `set acc = load(buf,i); let y = acc +wrap 7_u8; let z = cvt<u8,u16>(y);
set x = z;` under `bound @l s: ile(x, 255_u16)`, `x` a `u16`.

Term set, backward pass, IDENTICAL at both provers:

| step | commit | clause | p | term set after |
| --- | --- | --- | --- | --- |
| start | - | - | `x - 255` | `{x}` |
| 1 | `set x = z` | (c) copy | `z - 255` | `{z}` |
| 2 | `let z = cvt(y)` | (c) widening | `y - 255` | `{y}` |
| 3 | `let y = acc +wrap 7` | (b), `let` binder, no refusal | `o - 255` | `{o, acc}` |
| 4 | `set acc = load(buf,i)` | (e), RHS a call, **`set` destination** | - | REFUSED |

Step 3 puts `acc` in the term set from the pair's SHAPE `o - (acc+7) <= 0` /
`(acc+7) - o <= 0`, which [IND-4] introduces "at every clause (b) commit the
pass visits" whatever the prover derives. So step 4 happens at both versions.

**Slot list, weak prover:** none - the pass hard-errors at [IND-4](e) before
[IND-7] is reached.
**Slot list, strong prover:** none, same commit, same clause, same diagnostic.
**Acceptance:** hard error at [IND-1] on both. IDENTICAL. Not a break.

Price, correctly stated by 3.8.4 and 11.5: N1 was accepted by the weak checker
under the pre-repair text and is now refused everywhere.

## N2

Body: `let a1 = seed +wrap 7_u8; ... let a9 = a8 +wrap 7_u8; let z =
cvt<u8,u16>(a9); set x = z;`, `seed` loop-invariant, same statement.

Backward pass, IDENTICAL at both provers: the two clause (c) commits give
`p = a9 - 255`; each `let a_j = a_{j-1} +wrap 7_u8` is a visited clause (b)
commit whose pair shape `o_j - (a_{j-1} + 7) <= 0` puts `a_{j-1}` in the term
set unconditionally, so all nine are visited; `seed` has no commit on the path.

Slot list at BOTH versions:

```
(1) 1  H1 = x - 255                      the loop's only bound_stmt
(2) 0  path conditions (no branch)
    36 nine visited clause (b) commits x (2 constant bounds + 2 pair slots)
(3) 0  p = o9 - 255 has one degree-1 monomial, so no ordered pair
    ---
    37 slots
```

37 > 32. **Hard error naming the statement at both versions. IDENTICAL.**
Cap arithmetic checked: `1 + 4k` binds at `k = 8` (`33 > 32`); `k = 7` gives 29.

Both of the file's own witnesses hold. The visit set is closed.
