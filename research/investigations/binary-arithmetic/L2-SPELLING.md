# L2 needed a spelling decision, not more machinery

Measured 2026-09-05 against v0.47. **Settled: v0.48 shipped it.** The decision
is recorded in `PROOF-SURFACE.md` and the amendment in
`governance/APPROVALS.md`; this file is kept as the measurement that framed it,
and its "what the decision is" section below names the four candidates as they
stood before the choice.

What shipped is none of the four verbatim. `times` was chosen, with the
multiplicity moved *before* the premise and the premise delimited, which makes
one spelling serve both the decimal and the term form — the fourth candidate's
goal, reached by moving the marker rather than adding one to the literal form.
Its cost, a break to every `use 3 * X;` already written, was paid by a
mechanical migration of all 124 sites.

The rest of this file is as measured against v0.47.

## The machinery is already right

The matmul inner index is `n*p + j < n*k` from `p < k` and `j < n`. Its
certificate is one term-scaled premise and one plain one:

```
P1  p < k          normalized   p - k + 1 <= 0
    scaled by n                 n*p - n*k + n <= 0
P2  j < n          normalized   j - n + 1 <= 0
S = n*P1 + P2                   n*p - n*k + j + 1 <= 0
T   n*p + j < n*k  normalized   n*p + j - n*k + 1 <= 0
T - S = 0
```

The degree-2 monomials cancel exactly and the residual is the constant zero,
so `DIRECT` closes it with nothing further. The same shape at a literal stride
compiles today, verified:

```wf
invariant fits: 4_u64 * p + j < span {
  use 4 * (p < k);
  use j < 4_u64;
}
```

accepts, with `span = 4_u64 * k`. So the certificate pipeline, the scaled sum,
the residual check and the diagnostics are all already correct for this class.
What is missing is only that the multiplier may not be a term.

This also settles a scoping question the kernel-side scoping raised. It read
the break as `AffineInequality`'s segregated `upper: i128`, which cannot hold
`m * u` when a premise is scaled by a term. That is true of a design where the
polynomial lives in the shared type, and it is avoidable: the polynomial is
transient. If a written certificate is required to reduce to an affine
residual — which this class does, to zero — then the monomials need to exist
only inside the certificate's own accumulator, and `AffineForm`,
`AffineInequality`, the fact state, the kill and the join are all untouched.
The restriction is stated, checkable, and refuses nothing this evidence asked
for.

## The grammar does not admit the spelling

`use n * (p < k);` cannot be added to the normative EBNF. Both candidate
productions were written, the tables regenerated, and the parser run:

| production tried | result |
| --- | --- |
| `IDENT "*" (IDENT \| "(" affine_expr compare_op affine_expr ")")` | tables regenerate; parser raises `PredictiveConflict` |
| `IDENT "*" IDENT` | same |

The cause is structural. `proof_use` already admits a bare affine relation as
a source, and

```
affine_expr   := affine_term (affine_add_op affine_term)*
affine_term   := affine_factor ("*" affine_factor)?
affine_factor := literal | IDENT | "(" affine_expr ")"
```

so `use a * b <= c;` and `use a * (b + 1) <= c;` are already legal. After
`use IDENT *` the parser cannot tell a scaled certificate step from an affine
relation source, and the token that would distinguish them — the `<` inside
the parentheses against the `<=` after them — is arbitrarily far away. The
tables are strong-LL(2), which is two tokens from the decision point, so no
amount of table regeneration reaches it.

The literal multiplier escapes this only by accident of lexing: `"[0-9]+"` in
that position is the grammar's sole pattern predicate, a bare decimal with no
type suffix, and an affine literal must carry one. It is a different token
class, so `use 3 * …` is decidable at the first token. A term has no such
distinguishing class.

## What the decision is

Admitting a term multiplier needs one marker that makes the alternative
decidable at the first token or two. That is a new fixed lowercase grammar
atom, which META-5 counts, and the shape is a taste question the constitution
does not let the compiler settle: R3 chooses among candidate spellings by
evidence measured under W1, and no writer trial has been run on any of these.

Candidates, with what each costs:

- `use scale n * (p < k);` — decidable at the first token. Keeps `*` and the
  operand order of the literal form, so the two forms read alike, but the
  scaled form carries a word the literal form does not.
- `use n times (p < k);` — decidable at the second token. Reads as English and
  needs no `*`, but now one concept has two spellings that share nothing,
  which is what R3 exists to prevent.
- `use by n * (p < k);` — same class as the first, shorter word, weaker
  reading.
- Give the literal form the same marker, so there is one spelling for one
  concept. Unambiguous and R3-clean, but it is a breaking change to every
  `use 3 * X;` already written and to the specification archive's rendering
  rules.

The fourth is the only one that leaves the language with one way to say the
thing, and it is the most disruptive. That trade is the decision, and it wants
a writer trial rather than a preference.

**Resolved.** The fourth's goal was reached without its cost by rejecting the
premise all four share: that the marker is an operator sitting where `*` sat.
Moving it in front of the premise, as `N times X`, makes one spelling serve the
decimal and the term alike, and mandatory parentheses on the relation premise
remove the second spelling split at the same time. See `PROOF-SURFACE.md`.

## What was not blocked on it

Everything else in L2 was understood and sized before the spelling was chosen,
and all of it shipped as written here: the factor became a literal-or-value in
`CheckedProofUse`, the multiplicity is restricted to an unsigned type so its
nonnegativity is structural rather than an obligation, the accumulator is local
and degree-bounded by construction, and the residual must be affine.

One detail was decided during implementation and is worth recording, because it
is the part this note got wrong by omission. The sum's nonlinear monomials are
folded *down* to the value image of an admitted exact product, not expanded
*up* from the target's product bindings. Expanding rewrites a target that is
already affine and can turn a provable residual unprovable; folding only ever
removes monomials, and what comes out is an ordinary affine inequality that
reaches the existing residual, its integer tightenings, and the L0 route
unchanged.
