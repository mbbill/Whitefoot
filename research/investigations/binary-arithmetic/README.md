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

The same measurement found a second gap that is not about nonlinearity at all:
a contract clause admits no binary arithmetic in any spelling. `define needed =
count * 2_u64;` and `define needed = count + 2_u64;` are the same `[FN-8]`
rejection, so `requires len(out) >= 2 * len(src)` — the precondition of every
expansion codec — is unwritable, and a widened multiplication would not fix it.

## The five positions

The same relation is admitted differently depending on where it is written.
Every cell was compiled.

| position | `a + b` | `2 * b` | `a * b` |
| --- | --- | --- | --- |
| `invariant` [INV-1] | admitted | admitted | rejected at formation |
| `use` factor [PRF-1] | admitted | bare decimal only | `[GRAM-4]` |
| `requires`/`ensures` [MSR-5] | `[GRAM-5]` | `[GRAM-5]` | `[GRAM-5]` |
| contract `define` [FN-8] | `[FN-8]` | `[FN-8]` | `[FN-8]` |
| branch condition [ENT-3] | atoms only — a guard on a field projection records nothing |

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

Ordered by what the evidence asked for:

- **L0** — publish the interval product's proved bound on the result. No new
  representation: the bounds are constants carried against the zero term, so
  the ordinary `[ENT-5]` support rule gives the right kill behaviour for free.
- **L1** — one relation vocabulary across the five positions, so a contract can
  state a size precondition.
- **L2** — a non-literal `use` multiplier, `use n * (p <= k - 1);`. The
  identical certificate with a literal factor already compiles.
- **L3** — premise products, which is where Handelman completeness comes from.

L0 covers operands with constant bounds. It cannot reach a bound that is
relative to another runtime value: with a runtime `k`, the interval of `p < k`
is the type range, so `p * n <= (k - 1) * n` is outside it. That family —
matrix multiply's inner index, a transpose's destination stride — is what L2
is for.

## Layout

- `evidence/accumulators/` — loop-carried bounds, `[INV-1]` formation
- `evidence/grid/` — multi-dimensional and strided access, `VERDICTS.txt` is the table
- `evidence/codec/` — codecs, capacity arithmetic, the contract route
- `evidence/corpus/` — the count over the existing tree
