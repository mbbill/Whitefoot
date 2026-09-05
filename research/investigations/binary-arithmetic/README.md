# Binary arithmetic in the proof surface

Selection ground for the amendments that follow. Every verdict here was
compiled with the v0.44 `whitefootc` built from `main` at 82f6d6a; the probe
files are beside this one and each report names its own command.

## The question

The proof surface is affine. `[INV-1]` admits `*` only with a direct integer
literal operand, and `[ENT-6]`'s interval-product rule "publishes no product
inequality or intermediate premise". Does ordinary systems code hit that wall
often enough, and expensively enough, to widen it?

## What was measured

Four independent sweeps: three constructed programs in their own domain
(loop-carried bounds, multi-dimensional and strided access, codecs and
capacity arithmetic), one counted the existing tree.

| sweep | method | verdict |
| --- | --- | --- |
| accumulators | 46 programs | blocking cases rare; hit constantly at spelling level |
| grid | 55 programs | common — 9 of 11 natural forms rejected; rewrites cost 1.8x-2.4x source |
| codec | 36 programs | rare for sequential work; T3 every time for random access with a runtime stride |
| corpus | 1229 files scanned | 62 sites in 21 files (1.7%); 19 of the 21 cost nothing |

The corpus count and the constructed count disagree because the corpus cannot
measure this: it is written in the language that has the gap. One of its own
files records the dodge — `research/investigations/proof-derived-parallelism/loop/probes/r2_grid_loop_d21_w256.wf`
picks a power-of-two grid width so the index splits with a mask and a shift.
The two candidates that could have hit the wall use a literal stride or a
power-of-two dimension.

## The finding

All four sweeps converged on one sentence of the specification, not on the
strength of the prover. `[ENT-6]` computes the four endpoint products of a
non-constant multiplication, proves all four are in range, and then discards
them. The multiply is admitted and the value that comes out of it carries no
bound, so the `+` that follows fails.

```
E1_bounded_product.wf       accept   let base = r * w;
E5_product_then_add.wf      reject   let at = base + c;      [OP-2]
E6_invariant_on_product_result.wf
                            reject   invariant small: base <= ...;   [INV-1]
```

Supplying exactly the discarded bound as a written guard turns `E5` into an
accept, so that bound is the whole missing ingredient.

The same measurement found a second gap that is not about nonlinearity at all,
though its shape is not what the first reading suggested. A contract clause
does admit binary arithmetic, including nonlinear arithmetic: `define needed =
count *sat count;` forms, and so does `count +wrap 2_u64`. What a clause
refuses is the **exact** rows — `requires.rs:813-816` rejects any operation
whose `is_exact()` holds, which is `+ - * / % iabs ineg ishl ishr`. So the
divide is exact against total, not addition against multiplication:

```
define needed = count +wrap 2_u64;   forms; fails at the caller, Unproved
define needed = count *sat count;    forms (nonlinear, admitted); same
define needed = count + 2_u64;       [FN-8] InvalidRequires at formation
```

A second, independent wall is arity: `clause_expr` admits one operator, so
`requires a >= b + 1_u64;` is a `[GRAM-5]` parse rejection whatever the row.

The consequence stands — `requires len(out) >= 2 * len(src)`, the precondition
of every expansion codec, is unwritable — but the cause is a formation-time
policy over one operator set, not an absent arithmetic. The exact rows are
banned because a clause is runtime-typed (`requires.rs:236` runs
`check_expression`), while `affine_expr` "denotes a mathematical integer
expression and never a runtime evaluation" (`spec:3250`). An `invariant`
already has that carve-out; a clause does not.

## The five positions

The same relation is admitted differently depending on where it is written.
Every cell was compiled.

| position | `a + b` (exact) | `2 * b` (exact) | `a * b` (exact) | `a +wrap b`, `a *sat b` (total) |
| --- | --- | --- | --- | --- |
| `invariant` [INV-1] | admitted | admitted | rejected at formation | n/a — clauses are mathematical |
| `use` factor [PRF-1] | admitted | bare decimal only | `[GRAM-4]` | n/a |
| `requires`/`ensures` [MSR-5] | `[FN-8]` exact ban; second operator also `[GRAM-5]` | same | same | **admitted**, nonlinear included |
| contract `define` [FN-8] | `[FN-8]` | `[FN-8]` | `[FN-8]` | **admitted** |
| branch condition [ENT-3] | atoms only — a guard on a field projection records nothing | | | |

So the four positions disagree along two independent axes: which operator
rows are admitted (exact against total), and how many operators one relation
may contain.

## Direction

The compiler verifies writer-supplied certificates and never searches for a
proof. `[PRF-1]` today is a Farkas certificate: the writer names the premises
and the positive integer multipliers, and the checker sums and checks. The
nonlinear form of the same thing is a Handelman certificate, `S = sum of
c_i * m_i * P_i` over monomials `m_i` in provably nonnegative terms. Because
every Whitefoot integer type is bounded, the constraint domain is a compact
polytope and such a certificate exists for any true strict consequence;
finding it is the writer's job and checking it is normalization. Search stays
out, so the acceptance set remains a function of the written text and the
[ENT-1] monotonicity obligation is discharged structurally rather than argued.

## What shipped

Three amendments, in the order the evidence ranked them, each landed with its
own selection ground and merge-time record.

| version | change | probes moved |
| --- | --- | --- |
| v0.45 | `[ENT-3.S14]` publishes the interval an admitted product already proved | 2 |
| v0.46 | a clause states an affine relation, `len(P)` is an affine atom, and the affine route needs no L0 projection | 4 |
| v0.47 | an integer-typed named const is an affine atom | 1 |
| v0.48 | a `use` cites one premise, and its multiplicity may name an unsigned value | 2 new |

Seven of the 186 constructed probes move from refusal to acceptance through
v0.45 to v0.47, and the snapshot corpus holds at 491 pass with zero flips
through all four versions. v0.48 is counted differently and honestly: no
existing probe becomes an accept, because the two that wrote the term
multiplicity — `grid/G5_scaled_use_runtime_factor.wf` and the earlier
`l2-probe/named_source_term_factor.wf` — were written to demonstrate the
grammar's refusal and carry targets AUTO proves on its own. Both now parse,
and the capability is measured by two probes in `evidence/l2-probe/` that
state a real one: `matmul_term_stride_certificate.wf` and a rewritten
`named_source_term_factor.wf`.

The probe sources carry the current `use` spelling. The verdicts each report
records were measured at the version that report names, and the v0.48
migration from `use 3 * X;` to `use 3 times X;` is a mechanical rewrite of the
same premises — it changes no probe's meaning and no recorded verdict except
G5's, which `evidence/grid/VERDICTS.txt` re-measures in place.

Ordered by what the evidence asked for:

- **L0** — publish the interval product's proved bound on the result. No new
  representation: the bounds are constants carried against the zero term, so
  the ordinary `[ENT-5]` support rule gives the right kill behaviour for free.
- **L1** — admit the exact affine rows in clause position, so a contract can
  state a size precondition. The prover is already unified —
  `ProofGoal::Ordering` carries both a `Relation` and an `AffineForm` — and
  `affine_goal_value` already implements `AddExact`, `SubtractExact` and
  literal-sided `MultiplyExact`; those rows reach it today only from the
  caller side. What is genuinely duplicated is narrower than a relation model:
  `check/publication.rs` is a hand-rolled difference-bound closure beside the
  entailment one, and three separate converters into affine exist
  (`proofs.rs:531`, `flow.rs:8724`, `flow.rs:1337`).
- **L2** — a non-literal `use` multiplicity, `use n times (p < k);`, shipped as
  v0.48. The measured case is matrix multiply's inner index at a runtime
  stride, where the residual after the step cancels to zero. The polynomial the
  step passes through stays inside the certificate check: the accumulator
  becomes degree two at the first term multiplicity and every nonlinear
  monomial folds back to the value image an admitted exact multiplication
  already bound, so `AffineInequality`'s segregated `upper: i128`, the fact
  state, the kill and the join are all untouched. The nonnegativity of each
  written multiplicity is structural rather than an obligation, because
  [PRF-1] admits only an unsigned type there. The spelling was the actual
  blocker and it is recorded in `PROOF-SURFACE.md`.
- **L3** — premise products, which is where Handelman completeness comes from.

L0 covers operands with constant bounds. It cannot reach a bound that is
relative to another runtime value: with a runtime `k`, the interval of `p < k`
is the type range, so `p * n <= (k - 1) * n` is outside it. That family —
matrix multiply's inner index, a transpose's destination stride — is what L2
is for, and v0.48 admits it.

The order matters and it is not the order the surface suggests. Widening a
written surface before the fact base can hold what it states converts a clean
rejection into a silent non-fact: `checked_affine_relation_l0` publishes only
unit-coefficient relations, so `ensures len(out) >= 2 * len(src)` would form
and publish nothing, and a measure has no affine image to fall back on
(`spec:1429` — a measure term "is a clause operand and not yet an affine
atom"). So the fact base is fed first. L0 is exactly that, which is why it
comes before the surface work rather than after it.

## Layout

- `evidence/accumulators/` — loop-carried bounds, `[INV-1]` formation
- `evidence/grid/` — multi-dimensional and strided access, `VERDICTS.txt` is the table
- `evidence/codec/` — codecs, capacity arithmetic, the contract route
- `evidence/corpus/` — the count over the existing tree
