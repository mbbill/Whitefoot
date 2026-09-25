# Automatic operation facts

Status: research round complete; awaiting the owner's ruling. No
specification, compiler or design-tree change is made here. If the direction
is approved, the specification amendment, design-tree amendment and
implementation follow on the same branch.

## Question

[ENT-3] admits a narrow set of arithmetic idioms as automatic facts, each
added for one proof pattern, with no general criterion. The owner asked for a
principled rule and approved this investigation first. The concrete case:
`let hi = ishr(x, 32_u32); let narrow = cvt::<u64, u32>(hi);` is rejected,
while masking `hi` with `iand` first is accepted.

## Summary

- **Inventory.** Ten operation-derived fact rows exist today (S5's
  conversion equality; S7's wrapping and exact constant offsets, checked-arm
  offsets, unsigned quotient, remainders, unsigned `iand` and shifted one;
  S9's const-array ranges; S14's product interval), plus [ENT-6]'s affine
  images for exact `+`, `-` and constant-scaled `*`. Of the 34 [OP-1]
  spellings that produce an integer, 24 establish no fact at all, among them
  `ishr`, `ior`, `ixor`, `imin`, `imax`, `iabs`, `ineg`, `ipopcount`, `iclz`
  and the saturating rows. Each existing row was added for one program:
  sha256's schedule offsets, deflate's code-length table and bit mask, the
  compute kernels' block division, and the binary-arithmetic grid product.
- **Evidence.** The corpus under `tests/programs` and `lib/` needs a new
  operation fact at 3 sites (telemetry's shifted byte, dir_walk's byte-length
  clamp and its wrap-order guard); 18 further workarounds are already
  unnecessary today. A constructed sweep of 22 natural integer idioms is
  rejected at every one of them today; 21 of those were predicted rejected
  before the run and one was uncertain. The optimizer deletes the
  proof-only mask and checked conversion, so the cost is source and
  impossible error arms, not runtime.
- **Recommendation.** Replace the S7 menu and S14 with one table: every
  integer-valued operation row publishes the interval its operands' closed
  intervals give its result, rows bounded by an operand (`iand`, `ior`,
  `imin`, `imax`, `ishr`, `/`, `%`, `imulhi`, `iabs`, `+sat`, `-sat`) publish
  that order relation, and additive rows (`+`, `-` and their wrap forms when
  they cannot wrap) publish the offset relation that generalizes S7's
  constant offsets. A checked row's success payload receives the exact row's
  facts through the existing conditional Result context. All facts are
  ordinary L0 difference bounds, so no premise enters `AUTO`'s pair family.
  The pre-recorded selection criterion selects this step (D, whose relations
  the recommendation narrows to order and offset) and rejects the larger
  exact-image step (X).
- **Owner decisions** are listed in [Decisions for the owner](#decisions-for-the-owner).

## Constraints recovered before choosing

- [ENT-1] and the [checks-and-proofs](../../../design/language/checks-and-proofs.md)
  decisions: automatic derivation is a fixed, deterministic, terminating
  family run to completion, and "an author can determine from this rule alone
  whether a target is automatic" ([ENT-6]). A new rule must therefore be table
  data a writer can evaluate by hand, with no search and no fixed point.
- A nonempty certificate is rejected when `AUTO` already proves its target
  ([PRF-1]). Stronger automatic facts are therefore not purely widening: they
  can turn an existing `use` block into a redundant-certificate rejection.
- The [automatic-facts](../../../design/language/checks-and-proofs/automatic-facts.md)
  node already publishes S14's product interval "because the checker had
  already computed that interval ... and discarding it made a writer restate
  by hand what the checker had just proved". Its division-image decision
  remains grounded in the compute-model consumers.
- Cost: `AUTO` tries every pair of listed premises (the two-premise-cutoff and
  entering-context entries in [todo](../../../docs/todo.md)). The
  [incremental closure](../../../design/compiler/incremental-closure.md) and
  its middle-term selection determine what added L0 facts cost.
- The [constitution](../../../docs/constitution.md) prefers runtime
  performance over writing ease and accepts proof work, but requires checking
  cost to permit practical iteration. The
  [expected-failure decision](../../../design/language/checks-and-proofs/obligation-discharge.md)
  refuses impossible-case branches added only to satisfy the checker.

## 1. Inventory

### 1.1 Fact sources derived from an operation

Unless its row says otherwise, a source establishes at an `ordinary_let_rhs`
binding and, through [ENT-3.S5]'s commit rule, at a [SET-1] commit value.
None applies to a `return`, `give`, argument or contract-clause position.

| Source | Operation | Required shape and operands | Fact |
|---|---|---|---|
| S5 | integer `cvt` | operand a term or constant, after [OP-6] succeeds | `y = p` |
| S7 wrapping offset | `+wrap`, `-wrap` | constant `k` (either side of `+`, subtrahend of `-`); the closed state must prove `p ± k` in range | `s = p ± k` |
| S7 exact offset | `+`, `-` | constant `k` as above | `s = p ± k` |
| S7 checked arm | `+checked`, `-checked` | `match` scrutinee directly the call, or a bare let-bound name with no intervening kill or `set`; constant `k` | `Ok` binder `w = p ± k` |
| S7 quotient | unsigned `/` | direct binding; operands admitted terms or constants | `q <= a`; for a positive literal divisor `k*q <= a` (affine premise); captured images for the later product consequence `product <= a` |
| S7 remainder | `%` | direct binding; unsigned divisor term or constant; signed divisor a nonzero literal or named const | unsigned `r < d`; signed `-(abs(d)-1) <= r <= abs(d)-1` |
| S7 bit-and | unsigned `iand` | direct binding; each operand that is a term or constant | `r <= a`, `r <= b` |
| S7 shifted one | unsigned `ishl.wrap` | direct binding; value operand spelled as a typed literal or earlier named const equal to one | `r != 0` |
| S9 | const-array subscript | `let x = c[i]`, `c` a bare named const `Array<T, N>` | `min(c) <= x <= max(c)` |
| S14 | nonconstant `*` | domain discharged by the interval-product route only | the four-product hull |
| [ENT-6] images | exact `+`, `-`; `*` with a constant operand; integer `cvt` | any binding | exact affine image; every other integer operation receives a fresh full-range atom |
| [ENT-5] | `cvt.checked` | evaluation | conditional context: the payload equals the input |

### 1.2 Every integer row of the operation table

"None" means the result is a fresh term and atom carrying only its type
range.

| [OP-1] row | Result | L0 fact today | Affine image today |
|---|---|---|---|
| `+wrap`, `-wrap` | T | constant offset when provably in range | none |
| `*wrap` | T | none | none |
| `+`, `-` | T | constant offset | sum or difference |
| `*` | T | S14 hull on the interval-product route only | scaled when one operand is constant |
| `+defined`, `-defined`, `*defined` | Bool | goal origin | — |
| `+checked`, `-checked` | Result | constant-offset `Ok` arm in a direct match | — |
| `*checked` | Result | none | — |
| `/` | T | unsigned `q <= a`; literal `k*q <= a`; product consequence | none |
| `%` | T | unsigned `r < d`; signed literal divisor interval | none |
| `/defined`, `%defined`, `/checked`, `%checked` | Bool, Result | goal origin; none | — |
| `ineg.wrap`, `ineg` | T | none | none |
| `ineg.defined`, `ineg.checked` | Bool, Result | goal origin; none | — |
| `==` `!=` `<` `<=` `>` `>=` | Bool | comparison origin | — |
| `cvt` (integer pairs) | Dst | equality | copy |
| `cvt.checked`, `cvt.defined` | Result, Bool | conditional context; goal origin | — |
| `iand` | T | unsigned `r <= a`, `r <= b` | none |
| `ior`, `ixor`, `inot` | T | none | none |
| `ishl.wrap` | T | unsigned `r != 0` for a spelled one | none |
| `ishr.wrap`, `ishl`, `ishr` | T | none | none |
| `ishl.defined`, `ishr.defined` | Bool | goal origin | — |
| `irotl`, `irotr`, `ibswap`, `imulhi` | T | none | none |
| `ipopcount`, `iclz`, `ictz` | u32 | none | none |
| `+sat`, `-sat`, `*sat` | T | none | none |
| `imin`, `imax` | T | none | none |
| `iabs.wrap`, `iabs` | T | none | none |
| `iabs.defined`, `iabs.checked` | Bool, Result | goal origin; none | — |
| `reinterpret` (integer pairs) | T | none | none |

Three probes on the measured compiler confirm rows the table marks "none"
where a fact is obviously true: `iand(value_i32, 255_i32)` cannot prove
`cvt::<i32, u8>`, `slot *wrap 8_u64` with `slot < 8` cannot index a 64-slot
array, and exact `ineg` of a value in `0..=100` cannot prove
`negated + 100_i32`.

### 1.3 Why each row exists

Recovered with `git log -S` on the rule text and the compiler's derivation
kinds:

- **S7 constant offsets, checked arms, S9 and the retired S8.** The
  obligation-discharge batch-1 candidate (`81b73664b`, 2026-08-06) followed
  the [hand simulation](../obligation-discharge/SIMULATION.md) of utf8parse,
  deflate and sha256, whose L0 assumed "constant propagation incl. const-array
  element ranges, linear arithmetic (transitivity, ± constants, halving)".
  sha256's schedule reads `extend_index -wrap 2_u64`; deflate's
  `code_lengths[code_length_order[code_index]]` needed S9. S8's midpoint
  halving was struck by the [candidate review](../obligation-discharge/CANDIDATE-REVIEW.md)
  because no corpus program wrote it, which also noted that "the manifestly
  more-demanded shape ... the `irem` remainder bound `r < n` ... is absent
  while midpoint is present, i.e. selection tracked anticipation, not need".
- **Unsigned `iand` and the shifted one.** v0.28 (`1d0633e85`, 2026-08-15)
  added "only two measured unsigned direct-binding sources" for deflate's
  `read_bits`: the [acceptance probe](../obligation-discharge/ACCEPTANCE.md)
  isolated the chain `high != 0`, `mask = high - 1`, `value <= mask`,
  `value < high`, and recorded ior, ixor, right-shift, signed and
  local-one shapes as deliberate near misses.
- **Unsigned quotient with a literal divisor.** `d5fd20985` (2026-09-01),
  within the source-carried proof work that became v0.41; its tests are
  halving (`count / 2 <= count`, `2 * (count / 2) <= count`), the image
  binary-search midpoint certificates name.
- **Remainder bounds.** The source-carried proof implementation (`1c8c596b9`,
  2026-09-02, activated as v0.41), after the review quoted above.
- **Runtime divisors and the product consequence.** v0.57 (`cabb96de4`,
  2026-09-13) for the compute kernels' runtime block decomposition
  ([runtime division images](../compute-model/DESIGN.md#runtime-division-images)).
- **S14.** v0.45 (`69cec6cca`) from the
  [binary-arithmetic sweep](../binary-arithmetic/README.md#the-finding): an
  admitted product discarded its interval and the following `+` failed.
- **S5 for every integer pair.** Widened from total pairs by the proved
  conversion redesign (`62e4a744f`, 2026-09-23).

### 1.4 Asymmetries

- `/` by `2^k` bounds its quotient; `ishr` by `k`, the same value for an
  unsigned operand, bounds nothing. `*` by a constant has an exact image;
  `ishl` by a constant, which is that multiplication when no bit is lost, and
  `*wrap` by a constant that cannot wrap have none.
- `iand` has upper bounds; `ior` has no lower bound and `ixor` no bit-length
  bound. Signed `iand` by a nonnegative mask gets nothing.
- `ishl.wrap(one, n) != 0` depends on how the one is spelled: a local bound
  to one gets nothing, and the conformance case `ent3-neg-stage8b-local-one`
  asserts that rejection.
- A product's interval depends on which of three domain routes succeeded.
- `ineg` and `inot` are affine functions of their operand but receive fresh
  atoms; `imin`, `imax`, `iabs`, the saturating rows and the counting rows,
  whose ranges are elementary, publish nothing.
- The checked-arm fact exists only for a constant offset in a direct match,
  while `cvt.checked` has the general conditional transport.
- The signed remainder needs a literal divisor; the unsigned one accepts a
  term.

## 2. Evidence

All verdicts come from `whitefootc` built at `efe40194a` (this branch's base)
with `perl .github/run-check.pl aof-build cargo build --manifest-path
compiler/Cargo.toml --profile gate --bin whitefootc --locked --offline`
(exit 0, 70.8 s; binary SHA-256
`a0043d9ffd148132365d033616c942afed4bf0495f179476f646fc18f6499285`; Rust
1.98.1, Linux x86_64), invoked as `whitefootc --emit-llvm -o
<scratch-root>/<name>.ll <sources>`. Exit 0 is acceptance and exit 1 a source
rejection; every rejection below is a `Semantics/Source` issue whose rule and
residual are quoted.

### 2.1 The shift case

| Source of `high_word` | Result |
|---|---|
| `let hi = ishr(x, 32_u32); let narrow = cvt::<u64, u32>(hi);` | OP-6, residual `cvt.defined::<u64, u32>(ishr(x, 32_u32))` |
| the same with `ishr.wrap` | OP-6, residual `cvt.defined::<u64, u32>(ishr.wrap(x, 32_u32))` |
| `let masked = iand(hi, 4294967295_u64);` then `cvt` of `masked` | accepted |
| `match cvt.checked::<u64, u32>(hi)` with an `Err` arm returning zero | accepted |

`opt -O2` (LLVM 18.1.3) reduces both accepted forms to the same body:
`%v2 = lshr i64 %v0, 32`, `trunc`, `ret`. The mask and the checked branch
exist only in the source. The same shape is
[i01](probes/i01-high-word.wf) of the sweep below.

### 2.2 Corpus census

Method: in `tests/programs` (88 files) and `lib/containers` (6 files), list
every narrowing conversion, checked conversion, constant mask, shift, bit
operation, division, remainder and clamp with `grep`, read the guards and
proof statements around each, and for every workaround found remove it in a
copy and check the copy with the program's own bundle. Eleven unchanged
originals were accepted as controls. The search is by these operation
spellings, so a workaround spelled some other way would be missed.

**Sites that need a new operation fact (3).** Each copy is rejected today:

| Program | Workaround | Rejection without it |
|---|---|---|
| `telemetry_packet.wf` | `propagate cvt.checked::<u32, u8>(first_word)` after `ishr.wrap(value, 24_u32)` | OP-6 `cvt.defined::<u32, u8>(ishr.wrap(value, 24_u32))` |
| `dir_walk.wf` | a `value_if` clamp to 126 plus `cvt.checked` with an unreachable fallback; the literal `give` delivers no relation, and `imin(cursor, 126_u64)` gets no fact | OP-6 `cvt.defined::<u64, u8>(imin(cursor, 126_u64))` |
| `dir_walk.wf` | guard `name_at <= name_end` after `name_end = name_at +wrap name_count`, where `name_count` is `lo \| (hi << 8)` and `name_at <= 4096` | FN-8 `name_at <= name_end` at `open_directory_from_factory` |

**Workarounds already unnecessary today (18).** Each copy is accepted:
14 checked conversions (the four `low_byte` helpers in `tests/programs/parallel/`,
`io_complete_first_slice`, `io_open_and_read`, `tcp_client`, telemetry's
three masked bytes, and four in the deflate decoders) and 4 guards (deflate's
`code_count <= 19`, `distance_count <= 32` and `bounded < 19` after `% 19`,
and wfgrep's `digit < glyphs` after `% 10`). The existing facts that discharge
them are the `iand` and remainder rows, dominating comparisons, the
`read_bits` and `read_at` postconditions and the counted-range binder.
`io_propagate_open`, `io_vacant_read` and `io_write_prefix` have the same
shape and were not checked individually. The checked conversions are the
mechanical translation of the former Result-returning `cvt` when the proved
form arrived (`62e4a744f`), not evidence about the rule proposed here.

**Proof steps.** The corpus has 41 `use` lines in 8 files and 293
`invariant` lines. The `use` blocks scale a premise by a runtime or literal
factor or chain more than two premises (bfs, histogram, prefix, stencil,
radix scatter, grow-vector, priority-queue, byte_string); none bounds an
operation result. The local invariants state lengths, capacities, coverage
and products; the two priority-queue lemmas about `(at - 1) / 2` are already
automatic. Operation facts remove none of them.

The corpus undercounts demand because it was written against the current
menu: deflate passes both `count` and `mask` to `read_bits` so that the
existing `iand` row can bound the result, and wfgrep's guards are input-length
checks that no operation fact could remove.

### 2.3 Idiom sweep: alternatives, predictions and selection criterion

This section was committed in `d17282bb5` before any probe was compiled; its
text below is unchanged.

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

### 2.4 Sweep results

Every "Today" prediction held. All 21 idioms are rejected, each at its
predicted consumer:

| Probe | Rule | Residual |
|---|---|---|
| i01 | OP-6 | `cvt.defined::<u64, u32>(ishr(x, 32_u32))` |
| i02 | OP-6 | `cvt.defined::<u32, u8>(ishr.wrap(word, 24_u32))` |
| i03 | OP-4 | `index < alphabet.len` |
| i04 | OP-4 | `index < digits.len` |
| i05 | OP-4 | `word_index < deref(words).len` |
| i06 | OP-4 | `index < heads.len` |
| i07 | OP-4 | `at < deref(values).len` |
| i08 | OP-4 | `at < deref(destination).len` |
| i09 | OP-2 | `first -defined 1_u64` |
| i10 | OP-4 | `index < classes.len` |
| i11 | OP-2 | `64_u32 -defined leading` |
| i14 | OP-6 | `cvt.defined::<u32, u8>(ior(iand(low, 15_u32), ishl(iand(high, 15_u32), 4_u32)))` |
| i15 | OP-6 | `cvt.defined::<u32, u8>(ixor(iand(left, 255_u32), iand(right, 255_u32)))` |
| i16 | OP-6 | `cvt.defined::<u64, u16>(ior(cvt::<u8, u64>(deref(bytes)[at]), ishl(cvt::<u8, u64>(deref(bytes)[at + 1_u64]), 8_u32)))` |
| i17 | OP-2 | `end -defined start` |
| i18 | OP-6 | `cvt.defined::<i32, u32>(iabs(delta))` |
| i19 | OP-4 | `parent < deref(keys).len` |
| i20 | OP-4 | `left < deref(values).len` |
| i22 | OP-6 | `cvt.defined::<u64, u16>(ishr(hash, 64_u32 - bits))` |
| i23 | OP-4 | `bits < table.len` |
| i24 | OP-2 | `budget -defined remaining` |

Controls c12, c17 and c21 are accepted. The uncertain c13 is rejected with
OP-4 `row < rows.len`. The proof it needs adds the listed image
`8*row <= cell` to the L0 bound `cell <= 63` and tightens by 8; that is not one
of `AUTO`'s families, which pair only listed premises and try a strongest L0
image alone. So even the existing division row cannot bound a quotient by its
dividend's constant bound. Its alternative verdicts were not recorded in
advance; derived afterwards, K cannot bound it (the dividend's type range
divided by 8) and I can (`63 / 8 = 7`).

Two probes were respelled after the first run for reasons unrelated to
proofs, keeping their idiom and obligations: i20 exchanged two `u64` slots
with `swap`, which [OP-11] refuses for copy places, and now reads and writes
them back; i23 named a binding `entry`, a reserved spelling.

Applying the criterion to the 21 idioms recorded in advance:

| Step | Idioms the larger accepts and the smaller does not | Count | Outcome |
|---|---|---|---|
| M to K | i07, i09, i10, i11, i18 | 5 | justified |
| K to I | i04, i05, i14, i15, i16, i22, i23 | 7 | justified |
| I to D | i08, i17, i19, i20, i24 | 5 | justified |
| D to X | none | 0 | stop |

Counting c13 by its later derivation would add one idiom to the K-to-I step
and change no outcome.

The criterion selects D. M accepts i19 through its `r <= a` relation, which K
lacks; that does not change any step. Within D, every relation the five
D-step idioms need is either an order relation (`imin`, `imax`, `ishr`,
`-sat` against an operand) or an offset relation (`+wrap` that cannot wrap).
The recommendation therefore publishes those two kinds only (section 3.3);
the general difference interval of D adds relations no probe uses.

### 2.5 Sufficiency of the predicted facts

The table verdicts above are hand derivations. To check the consumer side
mechanically, [sufficiency.wf](probes/sufficiency.wf) replaces each idiom's
operation by a pure helper whose verified `ensures` states exactly the fact
the smallest predicted accepting alternative would establish (for example
`result <= 4294967295_u64` for i01, `result <= room` for i08,
`result >= start` for i17); the helper proves it with a guard, and
[ENT-3.S12] publishes it at the caller's binding. The whole file, covering all 22
rejected probes, is accepted.

Three mutations of that file show the check is not vacuous: widening i01's
bound to `4294967296_u64` is rejected at the `cvt`, relating i08's result to
`count` instead of `room` is rejected at the subscript, and weakening i17's
relation to `result >= 0_u64` is rejected at `end - start`.

The three corpus sites pass the same simulation in their full programs:
telemetry with `result <= 255_u32` for the shifted word; dir_walk with
`result <= 126_u64` for the clamp; and dir_walk without its order guard, with
`result <= 65280_u64` for the shifted high byte, `result <= 65535_u64` for the
merged length and `result >= start` for the wrapping sum. Adding the no-wrap
precondition `start <= 18446744073709486080_u64` to the simulated sum, which is
what the offset relation requires, is still accepted, so the condition is
derivable at that call.

This establishes that the stated facts suffice where they are established. It
does not establish the table's arithmetic or an implementation's cost.

## 3. Candidate criteria

### 3.1 Alternatives

The sweep ladder (M, K, I, D, X) is defined in section 2.3. Alternatives
outside it:

- **Keep the menu (status quo).** Leaves every asymmetry of section 1.4.
- **Declared operation contracts.** Give OP-1 rows prelude-style `ensures`.
  A one-datum-a-side template can state `r <= a` but not an interval that
  depends on operand intervals, so the rule would split across two mechanisms.
- **Verified helpers in libraries.** What section 2.5 simulates. Every project
  would restate operation semantics as contracts, and the helper still needs
  a guard or mask inside.
- **A bit-pattern (known-bits) domain.** More precise for masks, but a second
  fact family with its own joins, kills and closure; intervals and order
  relations cover every observed idiom.
- **Intervals on nested goal trees.** Source has no nested operation
  ([GRAM-9]); only contract-clause trees nest, where no probe needs a bound.

### 3.2 Result intervals and their soundness

A **corner hull** is the least and greatest value of an expression over every
combination of operand-interval endpoints. It is exact for an expression that
is monotone in each operand whenever the other operands are fixed, in either
direction: fixing all but one operand puts that operand's extreme at an
endpoint, and repeating this for each operand reaches a corner. Addition,
subtraction, multiplication, truncating division on each sign of the divisor,
`floor(a / 2^s)`, `a * 2^s`, `-a`, `M + m - a` (`inot`), `abs` on each sign,
`min`, `max`, clamping and `floor(a * b / 2^K)` are all such expressions, so
the hull is sound for them. Where the direction depends on a sign (the
divisor, `abs`), the interval is split into its negative and nonnegative
parts, a divisor's nonnegative part starting at one because the discharged
domain excludes zero.

Bit rows are not monotone and have fixed formulas valid when the operands are
nonnegative, because a nonnegative two's-complement value is its unsigned
binary number: `a & b <= min(a, b)`; `max(a, b) <= a | b <= a + b` and
`a | b < 2^bitlen(max(a, b))`; `a ^ b <= a | b`; `popcount(a) <= bitlen(a)`;
`clz(a) = K - bitlen(a)`; `ctz(a) <= bitlen(a) - 1` for `a >= 1`. `iand`
needs only one nonnegative operand: the result's sign bit is then clear and
its bits are a subset of that operand's.

Wrap modes: [OP-2] returns `wrap_T(z)`, which equals `z` whenever `z` is in
T, so a wrap row whose exact corner hull lies in T has the exact row's
interval, which is S7's existing condition for constant offsets; otherwise it
has only its type range. `ineg.wrap` and `iabs.wrap` return the minimum for
the minimum and follow the exact row only when `a0 > m`.

Shifts: [OP-8] masks a `.wrap` amount to `k & (K - 1)`. The amount interval
`[k0, k1]` is therefore the effective amount when `k1 <= K - 1`; otherwise any
amount in `[0, K - 1]` is possible. An exact shift's discharged domain already
bounds the amount by `K - 1`. `ishr` is arithmetic (floor) for signed T and
logical for unsigned T, both `floor(a / 2^s)` on their operand values. `ishl`
has a result interval only when every corner of `a * 2^s` is in T with
`a0 >= 0`, because then no bit leaves the value and none reaches a signed
sign bit. This subsumes the shifted one: `1 * 2^s` for `s` in `[0, K - 1]` is
`[1, 2^(K-1)]`, within an unsigned T, whatever the one's spelling.

Arithmetic: every endpoint is computed with checked i128 arithmetic, as
[ENT-6]'s interval-product rule already does; an unrepresentable endpoint (a
`u64` `imulhi` corner, for instance) establishes nothing, so a failure can
only lose a fact. Exact rows establish only after their own obligation
succeeds, so their operands lie in the admitted domain.

Generic bodies: a type-parameter-typed value is not an L0 fragment type in
the schema judgment ([ENT-1]), so no fact arises there; a const-generic
operand contributes its type's range. Concrete instances compute numerically.

### 3.3 Relational facts: order and offset

The relation criterion is stated per row, not per demanded idiom.

- **Order relations** hold for every admitted operand value (for the marked
  rows, every nonnegative value): `iand` below each nonnegative operand; `ior`
  above each operand; `imin` below and `imax` above each operand; `ishr` below
  its value operand; unsigned `/` and `%` below the dividend and `%` below the
  divisor; `imulhi` below each operand; `iabs` above its operand; `+sat` above
  and `-sat` below the first operand. Rows where the order depends on the
  other operand's value (`*`, `ishl`, `*sat`) or does not exist (`ixor`,
  `inot`, `ineg`, rotations) have none.
- **Offset relations** belong to the additive rows: `r - a` lies in `b`'s
  interval for `+`, and in the negated interval for `-`, for the exact rows
  and for their wrap forms when the hull proves no wrap. With a constant `b`
  this is S7's constant-offset equality.

Considered and not selected: D's full difference interval of `r - x` for
every row and operand (for example `r - a` in `[3 a0, 3 a1]` for `4 * a`).
It is sound by the same corner argument, but no probe uses it and each
relation adds closure work.

### 3.4 Where facts attach

At the destinations S5 already images: an `ordinary_let_rhs` binding and a
[SET-1] commit value, with operand intervals read before the commit's target
kill; and, through [ENT-5]'s existing conditional transport, a checked row's
private success payload (replacing S7's direct-match shape for `+checked` and
`-checked`). A `return`, `give` or call-argument value has no term, and
contract clauses keep their goal trees; writers bind first, as they already
must. An operand that is not a term (a subscripted read, for instance)
contributes its type range, and S9 remains the source for const-array reads.

An admitted term includes a measure term. The current S7 rows read their
operands through a reader that omits measure terms, so
`let r = x % deref(src).len;` establishes nothing although the specification
admits the measure (the todo item "Some ENT-3 sources read no measure
operand"). The implementation must read every S7 operand with one complete
ENT-2 term reader, which repairs that item for the replaced rows.

### 3.5 Interaction with the fact system

- **[ENT-4].** Results are ordinary difference bounds; no closure rule is
  added. A contradictory state already derives everything.
- **[ENT-5].** A result interval names only `r` and Z, so writes to operands
  leave it true, exactly as S14's interval behaves; an order or offset
  relation dies with either term. Joins keep the weakest common bound; facts
  established in a loop body do not reach the next head, and loop-carried
  values need header invariants as before.
- **[ENT-6] and `AUTO`.** No fact enters the listed affine premises, so the
  pair family and the two-premise cutoff are untouched. Implicit type bounds
  through Z already relate every pair of terms, so the L0-to-affine index keeps
  its size and only its bounds tighten. The division images remain listed
  premises, unchanged.
- **[PRF-1] redundancy.** A block becomes redundant only if the new facts let
  `AUTO` prove its target. The corpus blocks scale a premise or chain more
  than two. Among the 45 conformance cases with blocks, the operations whose
  facts change are mostly exact `+` and `-`, whose offset relations restate
  the affine image the checker already has, plus the two binary-search
  midpoint blocks (`2*half <= span` with `lo < hi`), which still need their
  pair because no order relation is scaled. This is a reading, not a proof;
  the implementation round must recheck every block mechanically.
- **Conformance verdicts.** `ent3-neg-stage8b-local-one` flips from reject
  to accept. Seven other negative cases whose facts the table touches
  (`fn9-neg-wrapped-negation-nonnegative`, `fn9-neg-saturation-is-not-wrapping`,
  `op2-neg-zero-quotient-predecessor`, `prf1-neg-wrapped-index-certificate`,
  `ent5-neg-wrapped-index-replacement`, `op4-neg-join-wrapped-increment-bound`,
  `op2-neg-break-removes-exhaustion-bound`) keep their verdicts by hand
  analysis; two of them may change from unproved to refuted, still under
  FN-9. The compiler test enumerating "no S7 source" shapes (local one,
  `ior`, exact `ishl`) changes with the rule.
- **Other consumers.** More proved separations can widen parallel
  permission, so `--par-ledger` output may change; lowering receives no new
  fact family ([backend facts](../../../design/compiler/backend-facts.md)).

### 3.6 Checking cost

- **Premises.** Zero new listed premises.
- **Closure reads.** Each operation binding reads its operands' closed
  bounds, which S7's wrapping-offset row already does. The corpus has about
  1,180 integer operation bindings (589 `+wrap`, 62 `-wrap`, 79 `*wrap`, 122
  `+`, 69 `-`, 37 `*`, 20 `/`, 45 `%`, 158 named rows), of which about 490
  constant-offset wrap bindings read the closure today: roughly 2.4 times as
  many reads. The remembered closed view and incremental insertion make a read
  after one or two new facts a repair, not a full closure.
- **Closure middles.** The closure skips terms that carry only implicit range
  edges. A term whose only live edges run to Z cannot improve a path either
  (entering and leaving it adds a nonnegative cycle in a consistent state), so
  result intervals need not add middles if that exclusion is extended; order
  and offset relations do add middles.
- **Derivation roots.** [DIAG-2] retains every S7 fact "even when no later
  query consumes it". With S7 facts at most of the roughly 1,180 operation
  bindings, the proposal switches S7 roots to the ordinary reachability
  pruning.
- **Not measured here.** No prototype was built in this round. The
  implementation round measures under the criterion in section 4.5.

## 4. Recommendation

### 4.1 The rule

Replace [ENT-3.S7] with one operation-fact source: every integer-valued
operation row publishes on the value it binds the result interval that its
fixed table computes from its operands' closed intervals, plus the row's order
or offset relations to each operand term; a checked row's success payload
receives the same facts through the conditional Result context. Retire S14
into the `*` row. Keep S5 and S9 and the unsigned division images (the
literal scaled image and the captured product consequence) unchanged.

Grounds: the rule is a fixed table evaluated once per binding from the closed
state that already exists there, so it is deterministic and terminating with
no search and no fixed point; a writer can evaluate it by hand; each entry
follows from [OP-2] and [OP-8] by the corner argument or an elementary
bit-level fact; the pre-recorded criterion selects this step; the three
corpus sites and all 22 sweep probes become provable; and no premise enters
`AUTO`. What could change it: measured checking cost failing the criterion
below, or a program that needs the exact sum identity of a non-wrapping wrap
operation (which would reopen X).

### 4.2 Rejected alternatives

- **Status quo and menu extension (M).** An `ishr` row fixes the reported
  case and repeats the pattern the todo entry describes; K alone accepts five
  more sweep idioms.
- **Static intervals (K).** Reads no closure, but loses every chain in which
  one bounded operation feeds another (seven recorded idioms and c13, among
  them the 16-bit decode). It is the fallback if closure reads prove too
  costly.
- **Closed intervals without relations (I).** Misses five sweep idioms whose
  bound is another term: a clamp to a runtime length, a heap parent by shift,
  a pair bound by `imax`, a saturating subtraction and a wrapping sum.
- **Full difference intervals (D as defined).** Adds relations no probe uses.
- **Exact images for non-wrapping wrap forms and lossless shifts (X).** No
  probe distinguishes it from D; the exact row already states the identity,
  and with the new intervals the exact row is provable where X would apply
  (dir_walk can write `name_at + name_count`). Reopen when a proof needs the
  identity and the writer cannot use the exact row.
- **Declared operation contracts, library helpers, a bit-pattern domain,
  nested goal-tree intervals.** Section 3.1.

### 4.3 Proposed specification text

Replace the complete [ENT-3.S7] item with:

> [ENT-3.S7]
> - S7 (operation facts).
> S7 is the source of facts about the value an integer operation row produces; an integer `cvt` keeps [ENT-3.S5]'s equality instead.
> At an `ordinary_let_rhs` binding and at a [SET-1] commit value whose right-hand side is one row of the table below with integer operands, after that row's own obligations have succeeded, establish on the bound value r the row's result bounds and, for each operand that is an admitted [ENT-2] term x, the row's relations to x.
> Each operand has one interval, read in the closed state where the right-hand side is evaluated, which for a commit precedes the target kill: a typed literal or integer-typed named const has its value; an admitted term has the least and greatest values its strongest closed L0 bounds through Z permit; every other operand, a const generic included, has its type's interval.
> At a contradictory point S7 establishes nothing, every relation being derivable there already.
> Below, T is the selected type with width K and interval [m, M]; a in [a0, a1] and b in [b0, b1] are the first and second operands' intervals; for a shift, b is the amount and s is [b0, b1] when b1 <= K - 1 and [0, K - 1] otherwise; and bitlen(x) is the number of binary digits of x >= 0, with bitlen(0) = 0.
> The corner hull of an expression is the least and greatest of its values at every combination of operand-interval endpoints; where a row names a split, the operand interval is first divided into its negative part and its nonnegative part, a divisor's nonnegative part starting at one, and the hull is taken over the nonempty parts.
> Every value is computed over mathematical integers with checked i128 arithmetic.
> When every operand interval is one value, the result bounds are the row's exact value whether or not its condition holds.
> Otherwise a row establishes nothing when its condition does not hold or a computation is unrepresentable.
> Result bounds are established as `lo <= r` and `r <= hi` through Z, each limited to T's interval; an order relation is the stated difference bound; an offset relation is the pair of difference bounds placing `r - x` in the stated interval.
> These facts have the ordinary [ENT-5] support of their terms.
>
> | Row | Condition | Result bounds | Relations |
> |---|---|---|---|
> | `+` | none | corner hull of a + b | offset: r - a in [b0, b1], r - b in [a0, a1] |
> | `+wrap` | the corner hull of a + b lies in [m, M] | that hull | as `+` |
> | `-` | none | corner hull of a - b | offset: r - a in [-b1, -b0] |
> | `-wrap` | the corner hull of a - b lies in [m, M] | that hull | as `-` |
> | `*` | none | corner hull of a * b, over the operand intervals [ENT-6]'s interval-product rule selected when that rule discharged the domain | none |
> | `*wrap` | the corner hull of a * b lies in [m, M] | that hull | none |
> | `+sat`, `-sat`, `*sat` | none | the corner hull of the unsaturated result, each end clamped to [m, M] | when a0 >= 0 and b0 >= 0: `+sat` a <= r and b <= r; `-sat` r <= a |
> | `/` | none | corner hull of the truncating quotient, b split | when a0 >= 0 and b0 >= 0: r <= a |
> | `%` | none | [min(0, max(a0, 1 - D)), max(0, min(a1, D - 1))] with D = max(abs(b0), abs(b1)) | when a0 >= 0 and b0 >= 0: r <= a and r < b |
> | `ineg` | none | corner hull of -a | none |
> | `ineg.wrap` | a0 > m | corner hull of -a | none |
> | `iabs` | none | corner hull of abs(a), a split | a <= r |
> | `iabs.wrap` | a0 > m | corner hull of abs(a), a split | a <= r |
> | `inot` | none | corner hull of M + m - a | none |
> | `iand` | a0 >= 0 or b0 >= 0 | 0 <= r, and r <= x1 for each operand interval [x0, x1] with x0 >= 0 | r <= x for each such operand term x |
> | `ior` | a0 >= 0 and b0 >= 0 | [max(a0, b0), min(a1 + b1, 2^bitlen(max(a1, b1)) - 1)] | a <= r and b <= r |
> | `ixor` | a0 >= 0 and b0 >= 0 | [0, min(a1 + b1, 2^bitlen(max(a1, b1)) - 1)] | none |
> | `ishl`, `ishl.wrap` | a0 >= 0 and the corner hull of a * 2^s lies in [m, M] | that hull | none |
> | `ishr`, `ishr.wrap` | none | corner hull of floor(a / 2^s) | when a0 >= 0: r <= a |
> | `imin` | none | [min(a0, b0), min(a1, b1)] | r <= a and r <= b |
> | `imax` | none | [max(a0, b0), max(a1, b1)] | a <= r and b <= r |
> | `imulhi` | none | corner hull of floor(a * b / 2^K) | when a0 >= 0 and b0 >= 0: r <= a and r <= b |
> | `ipopcount` | a0 >= 0 | [1 when a0 >= 1, otherwise 0; bitlen(a1)] | none |
> | `ipopcount` | a0 < 0 | [0, K] | none |
> | `iclz` | a0 >= 0 | [K - bitlen(a1), K - bitlen(a0)] | none |
> | `iclz` | a0 < 0 | [0, K] | none |
> | `ictz` | a0 >= 1 | [0, bitlen(a1) - 1] | none |
> | `ictz` | a0 <= 0 | [0, K] | none |
> | `reinterpret` between integer types | [a0, a1] lies in both endpoint types' intervals | [a0, a1] | offset: r - a in [0, 0] |
> | `reinterpret` between integer types | a1 < 0 and the destination is unsigned | [a0 + 2^K, a1 + 2^K] | none |
> | `reinterpret` between integer types | a0 > the signed destination's maximum | [a0 - 2^K, a1 - 2^K] | none |
>
> `irotl`, `irotr` and `ibswap` establish only the single-value case above.
> An unsigned exact division `q = a / d` whose operands are admitted terms or constants also captures the exact immutable value images of q, a, and d, and when d is a positive written integer literal k it retains the separate affine value image `k*q <= a`.
> [The following three paragraphs of the current item, from "At a later direct ordinary exact-multiplication binding" through "cannot retarget a captured relation to the replacement; a still-live alias of an old value may continue to use the old relation under [ENT-5].", stay unchanged.]
> A signed division, a nonterm operand, and a division at any other destination capture no image.
> No other operation shape establishes an S7 fact.

Dependent edits, stated as before and after:

- [ENT-3.S14]: delete the item; add "The label S14 is retired, not reused: its
  product interval is the `*` row of [ENT-3.S7]."
- [ENT-5], after the paragraph beginning "Evaluating
  `cvt.checked::<Src, Dst>(x)` with integer Src and Dst", add: "Evaluating
  `+checked`, `-checked`, `*checked`, `/checked`, `%checked`, `ineg.checked`
  or `iabs.checked` creates a conditional context the same way: its private
  success parameter denotes the exact row's mathematical result, and at that
  parameter it establishes exactly the [ENT-3.S7] facts the exact row
  establishes on an ordinary let of its result; its `Err` outcome carries
  none." This replaces the current S7 sentence beginning "For a `match` whose
  scrutinee is directly `p +checked k`".
- [ENT-6] interval-product paragraph: "established by [ENT-3.S14]" becomes
  "established by [ENT-3.S7]'s `*` row".
- [ENT-2] clause (g): "used only to carry constant bounds, S7's exact
  mathematical-zero disequality, and [ENT-6]'s normalized integer-domain
  components" becomes "used only to carry constant bounds and [ENT-6]'s
  normalized integer-domain components".
- [DIAG-2]: remove the sentence "Every new S7 fact is retained even when no
  later query consumes it.", the sentences introducing `BitAndBound`,
  `ShiftOneNonzero`, `UnsignedDivisionBound`, `UnsignedRemainderBound` and
  `SignedRemainderBound`, and the sentence beginning "A signed row where
  unsigned is required"; keep the `UnsignedDivisionProduct`,
  `RequirementAffineImage` and closing sentences; and add: "`OperationFact`
  roots each [ENT-3.S7] result bound, order relation and offset relation and
  carries the selected row, its destination, the relation, and as parents the
  closed operand bounds the row read; a literal or named-constant operand
  contributes its value and no parent. [ENT-3.S7]'s literal scaled division
  image cites the root of the quotient's order relation together with the
  exact q and a value images." S7 roots then follow the ordinary reachability
  pruning.

The amendment archives v0.69 and declares v0.70. Its accepted set widens
except where [PRF-1] finds a newly redundant block (none expected) and where
`ent3-neg-stage8b-local-one` flips.

### 4.4 Proposed design-tree revision

Node `language/checks-and-proofs/automatic-facts`. Replace the decision
beginning "An admitted nonconstant integer multiplication's proved interval
is published" with:

> Decision: Every integer operation row other than a conversion publishes on the value it binds the interval one fixed table computes from its operands' closed intervals, rows bounded by an operand publish that order, additive rows publish their offset, and a checked row's success payload receives the same facts, because rows added one proof pattern at a time bounded a division by a power of two but not the equal right shift, bounded a masked shift but not the shift alone, and bounded a shifted one only when the one was spelled as a literal, which forced proof-only masks, guards and checked conversions, instead of a row per demanded idiom or intervals read from types and literals alone.

and add to its `Rejected:` list:

> - Intervals from operand types and literals alone: rejected because a bounded operation feeding another loses its bound, which seven of the 21 recorded [sweep idioms](../../../research/investigations/automatic-operation-facts/DESIGN.md#24-sweep-results) need.
> - The difference interval of every result against every operand: rejected because no probe needs a relation beyond order and offset, and each relation adds closure work.
> - Exact affine images for wrap forms that cannot wrap: rejected because the exact row already states that identity and becomes provable wherever these images would apply.

The division-image decision stays; its first words become "An unsigned exact
division ... publishes the quotient's order relation and captured images".

### 4.5 Validation plan

**Specification-derived evidence.** For each table row, one accepted case at
the exact endpoint the row gives and one rejected case one past it (for
example `ishr(x_u64, 32)` converts to u32 while `ishr(x_u64, 31)` does not;
`imin(count, 16)` bounds a loop over 16 slots and `imin(count, 17)` does
not). Additional cases:

- signed rows: `iand` with one nonnegative operand accepted, with none
  rejected; `ishl` into the sign bit rejected; arithmetic `ishr` of a negative
  interval; `%` with a runtime signed divisor;
- wrap forms: a non-wrapping `+wrap` publishes its offset, a possibly wrapping
  one publishes nothing; a `.wrap` shift whose amount interval exceeds `K - 1`
  uses `[0, K - 1]`;
- support: a result interval survives an operand write; an order relation dies
  with its operand; a commit publishes through its commit value;
- measure operands: each relation row with a direct `deref(p).len` operand and
  with the same length bound first, accepted alike, and a write that kills the
  measure;
- checked rows: the `Ok` payload of `start +checked count` carries the offset
  relation, the `Err` arm nothing;
- the single-value case (`irotl(1_u32, 3_u32)` indexing a nine-slot table),
  an unrepresentable `u64` `imulhi` (nothing), and a generic `T` operation
  (nothing in the schema);
- the sweep probes as accepted cases, with `ent3-neg-stage8b-local-one`
  replaced by an accepted local-one case and a rejected case whose shifted
  value may be zero.

**Programs to simplify in the same change.** `telemetry_packet.wf` (four
bare conversions) and `dir_walk.wf` (`imin` clamp; drop the order guard or
use exact `+`). The 18 workarounds of section 2.2 are independent of the
ruling and can be removed with it or separately. Update
[P16](../../../docs/patterns.md#p16-choose-a-conversion-interface-from-the-intended-behavior)
to say that operation results carry their table interval.

**Soundness tests of the table.** For 8-bit types, enumerate operand
intervals drawn from a fixed endpoint set and compare each row's result
bounds, order and offset relations with the exact image computed by
enumerating every operand value; include every wrap and shift-amount edge.

**Cost, criterion recorded before timing.** Build the merge base and the
candidate with the gate profile in separate worktrees; on one host under the
verification lock, warm both once, then run five alternating pairs of
`whitefootc --emit-llvm` over every program bundle the program tests compile,
the `lib/containers` programs, and the
[checking-cost fixtures](../proof-certificate-architecture/CHECKING-COST.md#flow-selection-2026-09-16),
and time the conformance run once per compiler. Select the candidate when:
the summed median over the program bundles rises by at most 10%; no bundle
rises by both more than 25% and more than 50 ms; the checking-cost fixtures
rise by at most 10%; every unchanged source emits byte-identical LLVM; and
every `--par-ledger` difference is explained. Temporary counters report the
S7 facts established, closure reads at bindings and middle terms per closure,
and the checking time is split into formation, automatic derivation,
certificate checking and fact propagation before any cost is attributed.
The predicted cause of a regression is fact propagation at operation
bindings; its falsifier is a same-source variant of the candidate that reads
only static operand intervals, which must remove most of the regression if
the prediction holds. On failure, attribute the cost, apply the section 3.6
mitigations and remeasure; if it still fails, return the measured tradeoff
between D and the static-interval fallback K to the owner.

**Gate.** `make check` on the exact revision, including the full conformance
adapter and every program test.

### Decisions for the owner

1. Approve the direction: one S7 table of result intervals with order and
   offset relations, replacing the S7 menu and S14 (D as refined in 3.3), or
   select K, I or M instead.
2. Approve reading operand intervals from the closed state at each operation
   binding, about 2.4 times today's binding-time reads, subject to the cost
   criterion; K is the fallback that reads none.
3. Approve moving the checked-arm fact into the conditional Result context for
   every checked integer row.
4. Approve the flip of `ent3-neg-stage8b-local-one` and the end of
   spelling-keyed shift facts.
5. Approve or adjust the cost thresholds in section 4.5.
6. Confirm X stays deferred with the stated reopening condition.
7. Decide whether the 18 already-unnecessary workarounds are removed with the
   implementation or separately.

## Concerns and opportunities

Within the assessed scope (ENT-3's operation-derived sources, their ENT-4,
ENT-5, ENT-6, PRF-1 and DIAG-2 consumers, and the corpus and probes above),
the recommended rule removes the asymmetries of section 1.4 by one table
instead of more rows, reuses the existing L0 representation, supports, joins
and conditional Result transport, and adds no premise family. Three concerns
remain and are addressed by the plan rather than assumed away: the checking
cost of about 2.4 times more binding-time closure reads is unmeasured; a
stronger automatic set can make an existing certificate redundant under
PRF-1, which only a mechanical recheck settles; and the table itself joins
the trusted base, so its arithmetic needs the enumerative soundness tests.
One opportunity is declined: S9 could read the index interval and bound an
element read by the entries the index can reach, but no program needs it.

## Reproduction

Probe sources are in [probes/](probes/). Each probe and variant was checked
with the command in section 2 from `<repository-root>`; the corpus variants
edit one site of a copy placed under `<scratch-root>` and check it with its
program's bundle (the deflate bundle is `raw_deflate.wf`,
`raw_deflate_dynamic.wf`, `raw_deflate_dynamic_decode.wf`,
`raw_deflate_boundary.wf`). The optimized bodies come from `opt -O2 -S` on the
emitted modules. Remove this directory's probes when the conformance cases
derived from them land, or if the owner refuses the direction.
