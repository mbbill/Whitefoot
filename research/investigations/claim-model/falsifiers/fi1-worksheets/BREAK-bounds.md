# N4: row 10's replacement bound is false, and the determinism sentence rests on it

Verdict on 7f600c06: **REFUTED**, not on monotonicity - parts (i)-(v) hold and I
could not break them - but on the two other claims this round makes.

## The three sentences that collide

`[IND-3]`, DESIGN.md:1873-1882:

> Three spec-fixed limits apply to **every polynomial this rule normalizes** -
> the statement polynomial as written, and the substituted obligation `[IND-4]`
> and `[IND-6]` produce from it ... **Their scope is exactly those two
> polynomials**

`[IND-7]`, the *no hard error* paragraph, DESIGN.md:2338-2352:

> degree never rises, because `|b|*p - |a|*h` has the degree of its operands and
> both are at most 4; the monomial count is at most `5*256`; and a coefficient or
> constant is at most `(2*2^127)^5 = 2^640` ... **the obligation starts inside
> `[IND-3]`'s limits and, by the paragraph below, so does every filled slot's own
> polynomial.** Those three bounds are computed from the caps and `[IND-3]`'s
> limits alone

"the paragraph below" is the discard paragraph, whose only relevant sentence is
*"A fact offered for a slot that is not an `[IND-3]`-normalizable polynomial does
not fill it"*.

**A filled slot's polynomial is not always a fact the prover offered.** Group 1
is a statement polynomial (limited at its own admission, fine). Group 2's
constant bounds and group 3's difference bounds are prover facts (covered by the
discard, fine). But **group 2's clause (b)/(d) pair and `[IND-6]`(i)'s path
conditions are polynomials the *rule* writes and then the `[IND-4]` backward pass
substitutes**. Their content is syntactic; the prover decides only fill-or-empty.
So the discard sentence does not reach them, and `[IND-3]`'s narrowed scope
excludes them by name. Nothing bounds them.

That is the gap. Note the irony: those polynomials are *exactly as syntactic* as
the two the scope kept, so a limit on them would have been a legal class-(a) row.
The scoping threw away a guarantee it did not have to throw away, and the
resource argument then quietly assumes it is still there.

## N4, hand-executed - and it is an ordinary program, not an abuse

```whitefoot
fn scan(x: own u64, n: own u64) contract {
  requires ile(x, 1_u64);
} {
  let hits = 0_u64;
  for @scan i in 0_u64..n {
    bound @scan counted: ile(hits, i);
    let m1 = x * x;              // C1  clause (a), exact `*`, in domain by x <= 1
    let m2 = m1 * m1;            // C2  clause (a)
    let m3 = m2 * m2;            // C3  clause (a)
    set hits = hits +wrap m3;    // C4  clause (b), set destination
  }
}
```

Every commit is legal and every domain obligation discharges (`x <= 1` gives
`m1, m2, m3 <= 1`). This is I4's own shape with the increment computed instead of
written as a literal.

**Step `[IND-6]`, the one body path.** Binder shift: `p = hits - i - 1`.
Backward pass, end to entry:

```
C4  set hits = hits +wrap m3   clause (b), P = hits + m3
      hits <- witness o                    p = o - i - 1
      slots: 2 constant bounds on hits-after-the-commit
             E1 = o - hits - m3 <= 0 ,  E2 = hits + m3 - o <= 0
      side condition: hits - Z <= c1 (published `hits - i <= 0`), m3 - Z <= 1,
             c1 + c2 <= max(u64)  ->  DERIVES, no refusal, pair FILLED
      P's terms {hits, m3} enter the term set
C3  let m3 = m2 * m2   clause (a), unconditional, substituted EVERYWHERE
      E1 = o - hits - m2*m2 <= 0 ,  E2 = hits + m2*m2 - o <= 0
C2  let m2 = m1 * m1   ->  E1 = o - hits - m1^4 <= 0
C1  let m1 = x * x     ->  E1 = o - hits - x^8 <= 0 ,  E2 = hits + x^8 - o <= 0
```

`p0 = o - i - 1`: degree 1, three monomials, inside `[IND-3]`. Elimination terms
`o`, `i` - two. Slots: 1 (H1 = `hits - i`) + 4 (C4's two constant bounds and the
pair) + 2 (ordered pairs over `{o, i}`) = **seven**, well under 32. Every
class-(a) row is satisfied; nothing in the sweep fires.

**The deciding certificate.** `sigma(o) = E1`, `sigma(i) = H1`:

```
t = o : a = +1 in p0, b = +1 in E1, a*b > 0
        p := 1*p0 - 1*E1 = (o - i - 1) - (o - hits - x^8) = hits - i - 1 + x^8
                                                            ^^^ DEGREE 8
t = i : a = -1, b = -1 in H1 = hits - i, a*b > 0
        p := 1*p - 1*H1 = x^8 - 1
RELAX(x^8 - 1) : cu(x) = 1, corner max of x^8 = 1  ->  0 ;  floor(0/1) = 0 <= 0
VERIFIED
```

It is the only route: `E2`'s coefficient on `o` is `-1` against `a = +1`, so that
step is skipped; the constant-bound slots on the destination give only
`o - max(u64) <= 0`, whose residual relaxes positive; the empty certificate
relaxes `o` to `max(u64)` and fails. Exactly I4's situation - *"`sigma(o) = E1` is
the only step that eliminates the witness"*.

**So `[IND-7]`'s stated bound is false.** `degree never rises ... both are at
most 4` requires `h` to be degree <= 4; `E1` is degree 8. The same construction
breaks the other two figures: chain `let s = p1 + p2; ...` to a four-term sum and
square it four times and a filled pair carries `C(19,3) = 969 > 256` monomials;
chain `let t = s * 1000000_u64` seven times and it carries a coefficient past
`2^127`. Only "at most four steps" survives. `(2*2^127)^5 = 2^640` is arithmetically
right *given* the premise (`(2M)^(k+1)` at `k = 4`), and the prose "each step at
most doubles the bound" describes multiplying by `2^128`, not doubling - but the
premise is what fails.

## What that costs: the determinism claim, with a witness

The rule text says, one paragraph above the bound:

> A certificate a conforming implementation cannot evaluate does not succeed;
> the check moves to the next one.

That is an **implementation-capability discard that silently changes the
predicate**, and the only thing making it dead letter is the syntactic bound. It
is not dead letter.

- Implementation **A** evaluates exactly over the mathematical integers, as the
  rule says the arithmetic is. It evaluates the degree-8 step and `scan`
  **verifies**.
- Implementation **B** sizes the check to the bound this file publishes - degree
  at most 4, at most `5*256` monomials, magnitude at most `2^640`. It cannot
  evaluate the degree-8 step, so by the sentence above that certificate "does not
  succeed"; no other certificate succeeds; the obligation is undischarged and
  `[IND-1]` raises a **hard error**.

Both conform to the text. Both derive the same facts at the check point. They
decide different predicates on the same program. So

> **Two conforming implementations that derive the same facts at the check point
> therefore decide the same predicate on the same inputs** (`[IND-7]`)

is refuted, and with it 2.4 property 4 and 9's D1 ("nothing inside the space
raises ... an implementation may stop at the first success"). Note this is *not*
enumeration order - the repaired paragraph closes order and closes it correctly.
It is the other implementation-chosen degree of freedom the same paragraph
introduces, and `[ENT-1]` 2835-2836 forbids an implementation-chosen strategy,
not only an implementation-chosen order.

It is also the defect `AGENTS.md` names in as many words: *compiler capability,
an internal error, a timeout, or an unimplemented feature is not a
source-language rejection*. Row 13 turns capability into a refusal.

## Is it also a monotonicity break? No - and part (v) is why

Weak prover: C4's pair empty, `sigma(o) = E1` skips, the statement is refused
anyway (no route). Strong prover: pair fills, the big step executes. The
`sigma'` lemma covers the reverse direction exactly: any certificate that
succeeded while a slot was empty has an omitting twin that runs only the weak
version's arithmetic, which is by construction as evaluable as it was. So the
capability discard cannot cost a *previously accepted* program either.
`[ENT-1]` monotonicity survives N4. Determinism does not.

## The repairs this needs

1. Widen `[IND-3]`'s scope from "exactly those two polynomials" to **every
   polynomial `[IND-4]`'s shape rule produces** - the statement polynomial, the
   substituted obligation, and every hypothesis shape carried into the head
   frame. All three are prover-independent in content, so all three are legal
   class-(a) hard errors, and the sweep gains rows rather than losing its
   argument. This restores `[IND-7]`'s premise verbatim. Price: it rejects `scan`
   above, which nothing in this file needs.
2. Or keep the wide hypotheses and **delete the "cannot evaluate" sentence**,
   making exact arbitrary-precision evaluation a conformance requirement, and
   restate the resource bound honestly: bounded by a syntactic function of the
   path's text, not by a spec-fixed constant. Row 13 then reads "a certificate
   that fails **on its residual**", and row 10's last column loses "degree 4,
   `5*256` monomials, `2^640`".

Either closes it. Doing neither leaves the round's own determinism sentence
unsupported by a witness that is an ordinary program.
