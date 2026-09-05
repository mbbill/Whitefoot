# The proof surface: what may be written where

Design input for the amendment that follows. Written before the grammar is
touched, because the previous attempt changed the EBNF first, hit a parser
conflict, and only then discovered the design was wrong.

Every "today" verdict below was compiled — most against v0.47 when this note
was written, the rest re-measured against v0.48 after two of them turned out
to have been reasoned rather than run, and wrong. Where a verdict names a rule
it is the rule the compiler actually cited.

## The one rule

> A **proof position** is the text after `invariant NAME:`, after `use`, and
> inside a `requires`, `ensures`, or `define` clause. In a proof position every
> expression is over the **mathematical integers**: `+`, `-` and `*` never
> wrap, never trap, and raise no domain obligation, and a bare name denotes its
> value. Nothing else in a proof position is an expression.

That is the whole separation, and the line keyword is what carries it. An
agent needs to know one thing: which side of the keyword it is on.

The main language keeps its own reading of the same bytes, and the difference
is not cosmetic. In a body, bare `+` is the **exact** row of the operation
table — signature `(T, T) -> own T`, but partial, admitted only after its
domain is proved, which is what an `[OP-2]` obligation is. The `defined` row
is the Bool predicate that spells that domain. In a proof position there is no
domain to prove because nothing is evaluated: `[FN-9]` erases every clause and
`[ENT-1]` erases every invariant and certificate before lowering.

Reusing `+` across the two readings is a deliberate convention, not an
accident, and it is the same one Dafny, Why3 and ACSL use. It is legible
because the line keyword precedes it. What is *not* legible is a form that
poses as an expression while being something else, and that is the defect this
amendment removes.

## The defect being removed

`use 3 * (a <= b);` spells the Farkas coefficient with the multiplication
operator. The coefficient is not a multiplication: it is the **multiplicity**
of a premise, how many times that premise is added into the certificate sum.
Its right operand is a relation, not a number.

Three symptoms, all from that one category error:

1. It reads as `n * bool`, which means nothing in the main language.
2. It is unparseable when the multiplier is a term. `proof_use` already admits
   a bare affine relation as a source and `affine_term := affine_factor ("*"
   affine_factor)?`, so `use a * b <= c;` is legal today; after `use IDENT *`
   the token that separates a certificate step from an affine relation is
   arbitrarily far away, and the tables are strong-LL(2). Both candidate
   productions were written, the tables regenerated, and the parser run — both
   raise `PredictiveConflict`.
3. It forced a whitespace rule. `spec:84` makes `use 3 * (a <= b);` render
   with exactly one space before the `(` while an affine `a *(b + 1) <= c`
   renders with none, so the two are told apart by a byte the parser cannot
   see. That rule is a scar, and it is deleted with the wound.

The literal multiplier works today only by an accident of lexing: `"[0-9]+"`
in that position is the grammar's sole pattern predicate, a bare decimal with
no type suffix, while an affine literal must carry one. `use 3_u64 * (a<=b);`
is therefore a rejection — measured, `[GRAM-5]`, because the suffixed literal
takes the bare-relation alternative and then finds no `compare_op`. A term has
no such distinguishing class.

## The `use` form

**Before**

```
proof_use := "use" ( "[0-9]+" "*" (IDENT | "(" affine_expr compare_op affine_expr ")")
             | IDENT | affine_expr compare_op affine_expr ) ";"
```

**After**

```
proof_use  := "use" (("[0-9]+" | IDENT) "times")? use_premise ";"
use_premise := IDENT | "(" affine_expr compare_op affine_expr ")"
```

Read as: **`use` cites one premise — an invariant name, or a parenthesized
relation — optionally prefixed by `N times` to state its multiplicity.**

`times` is an evidence-selected word rather than a chosen one. It has zero
uses as an identifier anywhere in the corpus, and of its fifteen appearances
in doc strings, four are the corpus explaining this exact construct in prose:
"used premise pair_bound three times instead of one", "earlier invariant cited
three times", "named premise is used three times instead of one". The writers
already reached for it.

It also makes the multiplier's nonnegativity legible. "Use it n times" is
plainly nonsense for negative n, so the spelling teaches the constraint the
rule imposes.

### Parentheses become mandatory on the relation form

Today a relation premise is bare when unmultiplied and parenthesized when
multiplied, which is not a choice — it is the second scar from the same
ambiguity. With `*` gone the split has no reason to exist, and one of the two
shapes has to win.

The proof-theoretic answer settles it: a certificate is a list of premises
with multiplicities, and `use` names one line of that list. It **cites** a
premise rather than containing a relation, so a premise deserves one shape.
The parentheses are not grouping there; they delimit one premise.

Measured, this touches the dominant form: of 124 `use` sites in the corpus, 61
are bare relations, 29 are names, 29 are name-with-factor, 5 are
relation-with-factor.

### Decidability

Every form is decided within two tokens of `use`, which is what the
strong-LL(2) tables provide.

| written | T1 | T2 | selected |
| --- | --- | --- | --- |
| `use 3 times (p < k);` | bare decimal | — | multiplicity, at T1 |
| `use n times (p < k);` | IDENT | `times` | multiplicity |
| `use n times lt;` | IDENT | `times` | multiplicity |
| `use lt;` | IDENT | `;` | named premise |
| `use (p < k);` | `(` | — | relation premise |

`times` is a fixed atom, so it can never be the second token of a premise
name or of a relation.

## Position and form

Verified against v0.47. "math" means the [ENT] affine reading; "exact row"
means the operation-table row with its `[OP-2]` obligation.

This table says which operator *rows* each position admits at formation. It is
not a table of what is writable there: a clause carries one operator whatever
its row, which the last row's footnote records.

| position | `a + b` | `2 * b` | `a * b` | `a +wrap b` | bare name | bare decimal |
| --- | --- | --- | --- | --- | --- | --- |
| function body | exact row | exact row | exact row | wrap row | no — needs a `place` | no — needs a suffix |
| `invariant NAME:` | math | math | rejected at formation, `[INV-1] InvalidInvariant` | n/a | yes | no — needs a suffix |
| `use` premise | math | math | rejected at formation, `[PRF-1] InvalidSourceProof` | n/a | yes | no |
| `use` multiplicity | — | — | — | — | yes, unsigned only | **yes, the only such position** |
| `requires`/`ensures`/`define` | row admitted (v0.46) | row admitted (v0.46) | row admitted (v0.46) | row admitted | yes | no |

An admitted row in the `invariant` and `use` rows means the relation forms; it
still has to be proved, and an unprovable one is `[INV-1]
UndischargedLocalInvariant` rather than a formation error. The two rejected
cells cite different rules for the same restriction because a relation source
is owned diagnostically by PRF-1, which is the reading the specification's own
attribution sentence takes.

**The clause row's caveat, measured.** `requires a * b <= c;` and `requires a
+wrap b <= c;` are both `[GRAM-5]` rejections, and neither is about the row:
`clause_expr` carries one operator, so a relation with an arithmetic operator
*and* a comparison has two and does not derive. v0.46 widened which rows a
clause admits and left that arity alone; a second operator is written through
a `contract_define`. This note's first version reported the row admission as
though it were writability, which is the same conflation the investigation's
own README warns about.

The one row that is unique to a proof position is the multiplicity: a bare
decimal with no type suffix appears nowhere else in the language except a
`const` declaration.

## Alternatives considered

**Repetition as the coefficient.** `use (p<k); use (p<k); use (p<k);` with
`[PRF-1]`'s duplicate-premise ban inverted so repetition *is* the multiplicity.
Costs no atom. **Rejected**: a runtime `n` cannot be written as n repetitions,
and that is the only case this work needs.

**The coefficient inside the relation.** `use (3 * p < 3 * k);`. **Rejected**:
that is a different relation, which must itself be proved, and proving it
needs the scaling step. Circular.

**A name-only premise.** `use n times lt;` as the only form, which would kill
the operator pose outright. **Rejected by measurement**: 61 of 124 sites are
one-off inline relations of the shape `use x <= mid;`, and naming each would
double the lines while inventing names that carry nothing the relation does
not already say.

So a new atom is necessary rather than preferred.

## Known residues

**`times` still poses as an operator.** `use 3 times (a <= b);` reads as
though `times` were a binary operator binding looser than `*`, and that
reading is a lie: its right operand must be a premise, not a value. Mandatory
parentheses reduce the pose — after `times` the next token is always `IDENT`
or `(`, never the start of an arbitrary expression — but they do not remove
it. Three plausible-but-wrong writings follow from the pose. All three were
compiled against v0.48, and the pose costs less than this note first claimed:
two are parse errors and the third is a resolution failure that names the
right domain.

| written | rejection |
| --- | --- |
| `use a + 3 times (b <= c);` | `[GRAM-4]` at the `+`, `expected: [";", "times"]` |
| `use 3 times 4 times (a <= b);` | `[GRAM-4]` at the second decimal, `expected: ["IDENT", "("]` |
| `use 3 times x;` where `x` is a value | `[INV-1] UnresolvedUse { spelling: "x", role: InvariantFact, admissible: [Invariant] }` |

The third is the one the pose actually reaches, and it is not a confusing
message: a premise IDENT queries only the invariant-name domain, so a value
spelling cannot resolve there however plausibly it reads. What no diagnostic
says today is *why* — that this position wants an invariant name and `x` is a
value — and that is the residue worth watching if a writer trial hits it.

**Parentheses carry two roles.** Inside a relation they group an
`affine_expr`; around a relation in `use` they delimit a premise. This is the
same reuse the amendment objects to, one level down. It is accepted on
frequency — the affine fragment is small enough that internal grouping is rare
and appears in none of the 61 measured relation premises — and it is the price
of not introducing a second delimiter. If it turns out to bite, a distinct
premise delimiter is the repair and it is additive.

**The shared look is unchanged.** `invariant b: sum <= 255 * i` still shares
its bytes with body arithmetic. That is the deliberate convention above, and
this amendment does not address it. If a stronger separation is wanted later,
delimiting proof relations is the candidate and it does not conflict with
anything here.
