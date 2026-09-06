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

## The operand axis, measured on the domain that motivated the feature

A stride sweep across 19 padded-bitmap, DIB, alignment, tiling, volume and
interleaved-audio programs found the second axis is the one that bites, and it
bites harder than the ledger first recorded. Held to one shape, varying only
where the stride comes from:

| `stride` | verdict |
| --- | --- |
| `let stride = stride_src;` (a copy of a parameter) | accept |
| `let stride = stride_src + padding;` | reject, `NonlinearCertificateSum` |
| `let stride = stride_src + 4_u64;` | reject, `NonlinearCertificateSum` |
| `let stride = 2_u64 * stride_src;` | reject, `NonlinearCertificateSum` |

Any derivation kills it, and a stride in this domain is *definitionally*
derived — width plus padding, a row rounded up to four bytes, a tile area, a
channel count, an alignment. The feature admits the case where the stride is
already a parameter, which is matrix multiply's, and refuses the case the
sweep was written to test.

The ledger's first remedy, "the writer must bind the sum first", is wrong:
`let stride = width + padding;` *is* binding it, and the binding's image is
still the sum, so no product is recorded. The compiler's own mechanical fix
said the same wrong thing and told a writer who had already bound the product
to bind the product. Both now name the real condition — the operand must be
one the checker holds as a single value, a parameter or a call result — and
the sweep's only working workaround is a whole extra function to make it so.

The repair this points at is not more proof power. [PRF-1] already resolves a
**named premise** by declaration identity, explicitly not by re-deriving its
current value; the same treatment applied to the **named multiplicity** and to
the product's operands would fold `stride * row` to the binding the writer
wrote rather than to whatever sum it currently denotes.

## v0.49: the operand axis, closed

The same controlled shape, re-measured after the fold began naming the
declaration rather than its expansion:

| `stride` | v0.48 | v0.49 |
| --- | --- | --- |
| `let stride = stride_src;` | accept | accept |
| `let stride = stride_src + padding;` | reject | **accept** |
| `let stride = stride_src + 4_u64;` | reject | **accept** |
| `let stride = 2_u64 * stride_src;` | reject | **accept** |

Two designs were built and measured before the one that shipped, and both are
worth recording because each is the obvious first idea.

**Publish the handle's defining equality as a fact.** The handle then is not an
opaque unknown, and what was provable about `width + padding` stays provable
about `stride`. It does not help: the residual is the direct L0 route by rule,
and an affine fact is not something that route reads. Measured, the sum folded
and the residual came out as exactly the published equality rather than zero.

**Replace the binding's image with the handle.** Then every reader agrees and
the residual closes. It costs the other side: an ordinary premise about the
binding — `use (width <= stride);` — now needs the equality to prove, and
measured, two of the four rows moved from a fold failure to a premise failure.
Transparency helps premises and opacity helps the fold; picking one globally
breaks the other.

What shipped keeps the handle between the fold and the residual and unfolds it
before anything is proved. The first axis above is unchanged by this: a
multiplicity whose image carries a constant term still rejects, and still
correctly, for the residual reason worked out there.
