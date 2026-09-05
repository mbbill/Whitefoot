# What the multiplicity's own value image may be

Measured against v0.48. A named multiplicity need not be a bare atom — its
value image participates in the scaling like any other affine form. This
records how far that reaches, because the reach is not obvious and the obvious
guess is wrong in both directions.

Target held constant at `base_a < span_a`, with `let span_a = a * k; let base_a
= a * p;` and `use m times (p < k);`. Only `m`'s image varies.

| `m` | verdict |
| --- | --- |
| `a` | accept |
| `2_u64 * a` | accept |
| `a + 1_u64` | reject, `Combination` |
| `a - 1_u64` | reject, `Combination` |
| `a + 2_u64` | reject, `Combination` |
| `a - 2_u64` | reject, `Combination` |

The line is not addition against subtraction — those behave identically — but
whether the image is a pure scaling of the atom the products are over. A
scaling folds and the residual closes through the existing GCD tightening. An
image carrying a constant term does not, and **that rejection is correct
rather than a completeness gap**: scaling `p - k + 1 <= 0` by `a + 1` gives
`a*p - a*k + a + p - k + 1 <= 0`, which against the target `a*p - a*k + 1 <= 0`
leaves the residual `k - p - a <= 0`. That says `k <= p + a`, which `p < k` and
`a >= 2` do not imply — `p = 0, k = 1000, a = 2` falsifies it.

A separate axis, and the one the derivation ledger registers as open: when the
**product's own operand** is the composite, `let base = m * p;` with `m = a +
1_u64`, no product atom is recorded at all and the certificate stops at
`NonlinearCertificateSum`. The writer binds the sum first. The two axes are
easy to conflate and their failures read differently, which is why both are
written down here.

This table was measured after an adversarial sweep reported an asymmetry
between `a + 1` and `a - 1`. There is none; that sweep's three probes carried
three different targets, so the multiplicity was not the variable under test.
