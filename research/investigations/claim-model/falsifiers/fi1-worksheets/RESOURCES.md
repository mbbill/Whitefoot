# Every computation the check performs, and what bounds it

The charter's second job. One row per computation; "in text?" means the file
publishes a bound for it.

| # | computation | bound | in text? |
| --- | --- | --- | --- |
| 1 | body-path enumeration | <= 64, hard error beyond | yes ([IND-4]) |
| 2 | **[IND-4] backward substitution** | **none** | **NO - finding N6** |
| 3 | normalizing a shape against [IND-3] | none (it is the same computation as 2) | NO |
| 4 | deciding a prover fact [IND-3]-normalizable | none; but a discard, and the fact is the prover's object | no, harmless |
| 5 | certificate space | sum C(4,k)P(32,k) = 988,161 (recomputed: 1+128+5952+119040+863040) | yes |
| 6 | one certificate's steps | k<=4, degree<=4, <=1280 monomials, coeff < 2^640, s <= 2^508 | yes, and correct |
| 7 | RELAX corner products | < 2^1148 each, < 2^1159 summed | yes, and correct |
| 8 | **[IND-8] projection / corner minimum** | row 15 says "no limit, for RELAX's reason" and publishes **no figure**, unlike row 12 which now does | **NO, asymmetric** |
| 9 | [IND-8.V] two views | doubles 1-8 | not stated, harmless |

## N6 - the substitution's cost is assumed, not bounded

`[IND-3]`'s three limits are limits on the **output** of the shape rule. Nothing
limits the intermediate polynomial the backward pass materializes, and an
implementation **cannot** abort early on an over-limit intermediate, because
cancellation can bring the output back inside the limits.

Clause (a) admits "the `Ok`-arm binding of `+checked`, `-checked` or `*checked`:
the exact polynomial, **unconditionally**", and those bindings have **no domain
obligation to discharge** (the F3 deletion). So a chain of `*checked` Ok-arm
bindings raises the degree and the monomial count without any `[ENT-6]`
obligation to stop it. In a straight-line region under `[IND-10]` - whose text
says outright "the region bounds the substitution, and **no depth limit is
needed**":

```
v0 = Ok-arm of a +checked b                      // a + b
u1 = Ok-arm of v0 *checked v0 ; ... ; u40        // (a+b)^(2^40)
w1 = Ok-arm of v0 *checked v0 ; ... ; w40        // a parallel chain, distinct binders
z  = Ok-arm of u40 -checked w40                  // u40 - w40
bound region_zero: ile(z, 0_u64);
```

`p = z` substitutes to `u40 - w40` and then, through both chains, to
`(a+b)^(2^40) - (a+b)^(2^40) = 0`. The **output** is `0 <= 0`: degree 0, one
monomial, verified by the empty certificate, inside every `[IND-3]` limit. The
**intermediate** is `(a+b)^(2^40)`: about `10^12` monomials with binomial
coefficients of order `10^300000`. An implementation that expands it as the rule
prescribes cannot finish; one that stops early rejects a program the first would
accept.

This is `N4`'s species with the prover taken out of it. The *discard* paragraph's
new dead-letter argument is scoped to certificates - "every certificate in the
space is evaluable within four steps, degree 4, 5*256 monomials, coefficients
under 2^640 and a residual RELAX under 2^1159" - and the substitution happens
**before** any certificate exists, outside every published bound. So the
sentence "an implementation that declines work inside those bounds is not
conforming" says nothing about the one computation in the check that has no
bound at all, and `AGENTS.md`'s "compiler capability is not a source-language
rejection" is left standing over it.

Cheapest honest closes: limit the *intermediate* as well as the output (a hard
error on a shape whose intermediate crosses the limits is still class (a), since
the whole pass is prover-independent), or publish a bound as a function of the
path's text and say the work is exponential in the commit chain and that the
limits are checked incrementally.

## Rows 19-22, attacked

- 19 `ine` refused: reads the `rel_term`'s head name. No fact.               (a)
- 20 *Typing*: a type test on operands as written. Types are not derived.    (a)
- 21 *Vocabulary fence*: a grammar test; `[ENT-2]` 2870(a) decides it.       (a)
- 22 per-`fn_decl` name uniqueness: a name comparison over the text.         (a)

All four hold as class (a) and none has an `[ENT-1]` consequence. But rows 20
and 21 are stated as **admission**-time, and the widened `[IND-3]` is now
invoked at *substitution* time on branch conditions. If `[IND-3]`'s *Typing* and
*Vocabulary fence* travel with it, an ordinary `if` whose condition is a
`rel_term` over non-integer operands, or names an element of an indexable place,
becomes a hard error on any loop that carries a `bound_stmt`. `[IND-7]` group (2)
handles "a condition `[IND-3]` gives no polynomial at all" with an always-empty
slot, which reads as the intended answer - but rows 20 and 21 read as hard
errors. Third under-specification of the same widening.
