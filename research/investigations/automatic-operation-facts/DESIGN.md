# Automatic operation facts

Status: investigation in progress; no specification or compiler change.

## Question

[ENT-3] admits a narrow set of arithmetic idioms as automatic facts, each
added for one proof pattern, with no general criterion. The owner asked for a
principled rule and approved this investigation first. The concrete case:
`let hi = ishr(x, 32_u32); let narrow = cvt::<u64, u32>(hi);` is rejected,
while masking `hi` with `iand` first is accepted.

## Idiom sweep: alternatives, predictions and selection criterion

This section is recorded before the sweep is run. The corpus under
`tests/programs` was written in the language that has the gap, so it
under-reports demand (the
[binary-arithmetic sweep](../binary-arithmetic/README.md#what-was-measured)
made the same observation). The sweep therefore also checks natural forms of
common integer idioms, written without masks or guards whose only purpose is
a proof, each with a real consumer obligation (a narrowing `cvt`, a subscript,
an exact operation's domain, or a call requirement). The probes are in
[probes/](probes/); names starting `c` are controls expected to be accepted
today.

Candidate alternatives, from smallest to largest:

- **M, menu extension.** [ENT-3.S7] gains one row: a direct binding of
  unsigned `ishr` or `ishr.wrap` by a written constant amount `k` below the
  width establishes `r <= max(T) >> k` and, for an admitted term operand `a`,
  `r <= a`.
- **K, static intervals.** Every pure total integer-valued row establishes
  the constant interval its fixed transfer table computes from each operand's
  static interval: a literal or named constant is its value, every other
  operand its full type range, restricted by the row's own proved domain.
  No closure is read.
- **I, closed intervals.** The same table, with each term operand's interval
  taken from its strongest closed L0 bounds through Z at the binding, the
  query [ENT-3.S7]'s wrapping-offset row already makes.
- **D, difference intervals.** I, plus for each admitted term operand `x` the
  table's interval of `r - x` over the operand box, published only where it is
  narrower than what the result and operand intervals already imply through Z.
- **X, exact images.** D, plus a wrapping add, subtract or multiply proved not
  to wrap, and a shift by a constant proved to lose no bits, take the affine
  image of the corresponding exact operation.

Predicted verdicts (A accept, R reject) before any probe was compiled:

| Probe | Consumer obligation | Today | M | K | I | D | X |
|---|---|---|---|---|---|---|---|
| i01-high-word | `cvt` u64 to u32 | R | A | A | A | A | A |
| i02-top-byte | `cvt` u32 to u8 | R | A | A | A | A | A |
| i03-base64-sextet | subscript below 64 | R | A | A | A | A | A |
| i04-field-then-shift | subscript below 16 | R | R | R | A | A | A |
| i05-bitset-word | subscript below 4 | R | R | R | A | A | A |
| i06-hash-top-bits | subscript below 64 | R | A | A | A | A | A |
| i07-clamp-constant | subscript through a loop bound | R | R | A | A | A | A |
| i08-clamp-length | subscript through a loop bound | R | R | R | R | A | A |
| i09-floor-then-decrement | exact `-` domain | R | R | A | A | A | A |
| i10-popcount-table | subscript below 65 | R | R | A | A | A | A |
| i11-leading-zero-class | exact `-` domain, subscript | R | R | A | A | A | A |
| i14-pack-fields | `cvt` u32 to u8 | R | R | R | A | A | A |
| i15-xor-bytes | `cvt` u32 to u8 | R | R | R | A | A | A |
| i16-u16-decode | `cvt` u64 to u16 | R | R | R | A | A | A |
| i17-wrap-sum-order | exact `-` domain | R | R | R | R | A | A |
| i18-absolute-unsigned | `cvt` i32 to u32 | R | R | A | A | A | A |
| i19-heap-parent-shift | subscript below a length | R | A | R | R | A | A |
| i20-swap-under-max | two subscripts below a length | R | R | R | R | A | A |
| i22-variable-top-bits | `cvt` u64 to u16 | R | R | R | A | A | A |
| i23-low-bits-table | subscript below 32 | R | R | R | A | A | A |
| i24-saturating-consumed | exact `-` domain | R | R | R | R | A | A |
| c12-mask-ring-slot | subscript below 1024 | A | A | A | A | A | A |
| c13-quotient-row | subscript below 8 | uncertain | | | | | |
| c17-exact-sum-order | exact `-` domain | A | A | A | A | A | A |
| c21-residue-byte | `cvt` u64 to u8 | A | A | A | A | A | A |

Selection criterion. Each step up the ladder M, K, I, D, X must be justified
by at least two sweep idioms that today's compiler rejects, that the larger
alternative accepts by its written table, and that the smaller one does not.
Selection stops at the first step that fails. An idiom that today's compiler
accepts leaves the demand count. A rejected control stops the analysis until
its cause is understood. The corpus census is reported separately and does not
enter this count, because it was measured before the criterion was written.
The hand derivations are predictions; the implementation round checks them
mechanically, and measured checking cost may still move the selection down a
step under the cost criterion recorded with the validation plan.
