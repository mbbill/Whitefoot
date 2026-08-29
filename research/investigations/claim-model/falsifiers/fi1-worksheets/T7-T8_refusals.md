# T7/T8 - the two refusal traces (DESIGN.md 3.8.3), re-executed

Both are the shapes the judges broke the drafted rules with. Both are REFUSED by
the final text, and the certificate search does not reopen either.

## T7 - A16 / judge-2 FATAL-1: head value vs exit value
```
loop @l {
  bound @l s: ile(x, 0_u64);
  let more = ilt(rounds, 3_u64);
  if more { } else { break @l; }
  set out[x] = 1_u8;
  set x = cursor;
  set cursor = 0_u64;
  set rounds = rounds +wrap 1_u64;
}
```
with `cursor = 7` before the loop.  P = `x <= 0`.
One backward pass over the non-breaking body path, end -> entry:
  `set rounds = ...` rounds not in p; `set cursor = 0_u64` cursor not in p;
  `set x = cursor` -> copy, clause (c): p = `cursor`;
  `set out[x] = 1_u8` destination is a subscripted place, not a term; nothing.
p0 = `cursor`, in the HEAD frame.
Elimination terms: { `cursor` } - one term, so H group 3 contributes no ordered
pair. H = { H1 = `x` (the statement as written), the path condition
`rounds - 2 <= 0` }.
No member of H has a nonzero coefficient on `cursor`, so no step is admissible
and every certificate is the empty one.
RELAX(`cursor`) in the HEAD state = cu(cursor) = 7 (join of preheader 7 and
back-edge 0). floor(7/1) = 7 > 0  ->  REFUSED. Matches the file.
The drafted rule relaxed the same p0 in the body-exit state, where cu(cursor)=0,
and verified. The repair is the frame sentence in `[IND-6]`, not the certificate
form; the certificate form neither helps nor hurts here.
Compiled: j3_ind6_checkpoint_break.wf REJECT `[OP-4] residual: x < len(out)`,
j3b_ind6_consumer.wf ACCEPT. The pair still stands.

## T8 - A2 / judge-1 FF2: self-cancelling substitution
```
bound @weigh per_byte: ile(sum, 255_u32 * i);
set sum = sum + 1000_u32;
```
Binder shift + substitution: p0 = `sum + 1000 - 255*i - 255` = `sum - 255*i + 745`.
H1 = `sum - 255*i` - the statement AS WRITTEN, never substituted ([IND-6]).
Elimination terms: `i`, `sum`. Enumerating every certificate:
  sigma = {}            : RELAX = cu(sum) + 0 + 745 > 0. FAIL.
  sigma(i) = H1         : a = -255, b = -255 -> p := 255*p0 - 255*H1 = 190125,
                          s = 255, floor(190125/255) = 745 > 0. FAIL.
  sigma(sum) = H1       : a = +1, b = +1 -> p := p0 - H1 = 745, s = 1. FAIL.
  sigma(i) = H1, sigma(sum) = h : after the first step sum's coefficient is 0,
                          so a = 0 and no second step is admissible.
  no other H member exists (no path condition, no witness, and neither
  `sum - i <= c` nor `i - sum <= c` is derivable in this program).
REFUSED. Matches the file's 190125.
The exhaustive search therefore does NOT resurrect FF2: the break needed the
hypothesis to BE the substituted polynomial, and `[IND-6]` removes that
hypothesis from the space entirely.

## What both re-executions confirm
The two fatal shapes are closed by `[IND-6]`'s frame sentences. Neither closure
depends on the certificate form, and the certificate form opens no new route to
either. That part of the synthesis holds.
