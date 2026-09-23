# Numeric conversion coverage

## Question and scope

The baseline is main at `5dd9d5d7`, after generic struct constants. TYPE-4 and
OP-6 make `cvt` exact and value-preserving: its result type is selected from
the complete source/destination type pair. OP-8 separately supplies same-width
bit reinterpretation and same-type floating-point rounding operations.

This investigation asks which useful numerical transformations can be written
directly, which need an ordinary composition, which require a failure branch
despite a known domain, and which are missing. It does not change the active
specification or select new operation spellings before comparing examples.
The immediate consumer is integer low-bit extraction for binary encoding;
the wider inventory includes integer range/signedness changes, saturation,
integer/float conversion and float precision changes.

## Evidence criteria

For each family, separate three observations:

1. Mathematical contract: preserved value, selected bits, rounded value, or
   clamped value; state the treatment of negative values, bounds and special
   floating-point values where applicable.
2. Accepted source: current result type, required branches or proofs, and
   whether an ordinary source function can express the intended total result.
3. Generated code: inspect emitted and optimized LLVM for parameterized
   helpers, so constant-folded main calls cannot establish a general cost.
   Distinguish a source-level Result from an actual surviving runtime branch.

Use complete small WF programs and boundary controls. Low-bit extraction must
include values beyond the destination range; value-preserving narrowing must
retain a failing out-of-range control. Signedness probes include negative
inputs and the high bit. Float probes include fractions, exactness boundaries,
signed zero, infinity and NaN. Existing conversion conformance cases remain
the baseline for already implemented OP-6 behavior, not a new redundant suite.

Compare retaining ordinary compositions, adding a total bit-truncation family,
and exposing a proof-required exact conversion alongside its checked form.
For rounding/saturation, first determine whether existing operations compose
with precise enough behavior; a missing convenience spelling alone does not
establish a runtime cost. No timing claim will follow from instruction counts.
Use primary Rust/LLVM documentation only where external comparisons help
distinguish semantics or lowering requirements.

The deliverable is a bounded inventory with source and emitted-code evidence,
ranked improvement options, and explicit unresolved policy questions. Retained
probe sources live beside this investigation for reproduction and are removed
or replaced when their observations are superseded. They are not daily test
inputs; any selected implementation extracts necessary regression evidence
into the maintained compiler/conformance suites. Deferred findings update the
existing numeric-conversion TODO rather than creating an unrelated work queue.

## Current coverage and observed limits

The 10 numeric primitive types give 90 distinct ordered pairs. OP-6 supplies
29 total exact conversions and 61 checked exact conversions. These are a
coverage partition, not 61 missing operations. Existing semantic tests cover
the pair classification; backend conversion tests cover the integer lowering
classes, float endpoints and special-value edges.

| Intent | Current source | Observation | Disposition |
|---|---|---|---|
| Exact widening, including unsigned to a strictly wider signed type | Total `cvt` | Direct extension/conversion and ordinary value evidence | Already supported |
| Exact narrowing or signed-to-unsigned value conversion | Checked `cvt` | Correct Result; a call-site range proof does not change that type | Consider an explicit proved form |
| Same-width bits, including signedness and int/float relabeling | `reinterpret` | Preserves bits; differs deliberately from numeric conversion | Keep this distinction |
| Low destination bits of a wider integer | Mask, checked `cvt`, match; reinterpret for a signed result | Total wrapper is expressible, but needs an impossible-error fallback in source | Consider a direct total truncation operation; no measured speedup established |
| Integer saturation | `imin`/`imax`, checked `cvt`, match | The tested i32-to-u8 helper loses the redundant conversion check after optimization | Ordinary composition is viable for this case |
| Float to integer after a chosen rounding rule | `ftrunc`/`ffloor`/`fceil`/`froundeven`, then checked `cvt` | Rounding exists; range/NaN/infinity rejection remains a real requirement | Do not call every Result unnecessary |
| Float to saturated integer | Explicit NaN policy and endpoint branches/clamps, then rounding and `cvt` | Expressible for tested i32/i64 destinations; redundant round-trip checks survive | Candidate for a direct total operation, after selecting its policy |
| Rounded large integer to float, or f64 to f32 | No direct conversion with this contract | Exact `cvt` can reject a value for which a rounded answer is wanted | Missing direct value semantics; float integral rounding is not precision conversion |
| Integer conversion inside a numeric generic | `cvt::<T, f64>` | Explicit compiler `Unsupported: Generics`, even when its result is discarded | Separate implementation gap under existing rules |

An identity `cvt::<T, T>` is deliberately absent under OP-6. Whether a future
uniform generic conversion family should admit identity remains a policy
question; the unsupported generic witness uses integer T and float destination,
so it does not depend on resolving that question.

### Source witnesses

[probes.wf](probes.wf) contains parameterized helpers and boundary observations.
The low-byte helper is the current ordinary composition:

```wf
fn low_byte(value: own u32) -> result: own u8 pure {
  let masked = iand(value, 255_u32);
  match cvt::<u32, u8>(masked) {
    Ok(value: byte) => {
      return byte;
    }
    Err(error: refused) => {
      return 0_u8;
    }
  }
}
```

The fallback has defined behavior, but is mathematically unreachable for this
mask. Writing it does not make `cvt` a proof-required operation. A source
change to the mask can make that fallback reachable without a type error.

[proved-result-type.wf](proved-result-type.wf) gives `value <= 255_u32` as a
verified requirement, then directly returns `cvt::<u32, u8>(value)` from a
u8-returning function. It is rejected at the return with FN-1 ReturnMismatch:
the result remains `Result<u8, NarrowError>`. This is specified behavior,
not failed interval reasoning or an implementation defect.

[checked-equality.wf](checked-equality.wf) has the stronger practical loss:

```wf
match cvt::<u64, u8>(index) {
  Ok(value: small) => {
    let restored = cvt::<u8, u64>(small);
    return values[restored];
  }
  Err(error: refused) => {
    return 0_u8;
  }
}
```

The function requires `index < 4_u64` and the table has four elements. The
compiler nevertheless reports OP-4 at `restored`, with residual
`restored < values.len`. Replacing only the selected subscript by `index`
is accepted. ENT-3.S5 publishes conversion equality only for total pairs;
the checked Ok payload has its own u8 range but no connection to the original
operand. The existing semantic test
`a_narrowing_conversion_carries_no_equality_into_its_ok_arm` expressly records
that rule. Any change here must amend the proof rule and its evidence together.

[generic-conversion.wf](generic-conversion.wf) instantiates one Int-generic
helper with u32 and i64 and discards its `cvt::<T, f64>` result. All pairs are
distinct numeric pairs and no uniform result type is demanded by the helper.
The compiler reports `Semantics/Unsupported: Generics`; the existing
`generic_conversion_is_reported_as_unsupported_instead_of_invalid_source`
test exposes the same capability boundary. Concrete conversion coverage does
not establish generic conversion support.

### Optimized helpers

With Apple Clang 21.0.0, arm64 Darwin and `-O2`, the unchanged emitted module
keeps ordinary externally visible helper definitions. No linkage rewriting,
forced inlining, proof insertion or constant substitution is used for this
comparison. Main's constant checks fold separately and are not cost evidence.

| Helper | Optimized LLVM body observation |
|---|---|
| `low_byte` and `signed_low_byte` | One `trunc i32 ... to i8` and return; no mask, Result or validity check |
| `bounded_byte` with a proved requirement | Still compares with 256 and selects the fallback; the entry range was not conveyed to LLVM |
| `saturating_byte` | Clamp and truncation; no separate conversion-validity check |
| `bit_resign` | Returns the input bits unchanged |
| `wide_unsigned` | One zero extension |
| `exact_small_float` / `exact_small_double` | One `uitofp` for their total type pairs |
| `exact_float`, `exact_integer`, `narrow_float` | Conversion plus reverse/exactness tests and Result construction, implementing their selected contract |
| `truncated_integer` / `rounded_integer` | Float rounding plus saturating cast, reverse cast, exactness test and Result construction |
| `saturating_integer` / `saturating_wide_integer` | NaN/bounds handling and remaining cast round-trip tests, despite the wrappers' mathematically total contracts |

Arm64 assembly agrees with the relevant distinction: both low-byte helpers
are just a return under the narrow-result ABI, while `bounded_byte` retains a
compare and conditional selections. This is evidence about these parameterized
helpers on this toolchain, not a wall-time measurement or a universal optimizer
guarantee. Inlining into a caller may change the result.

A separate exploratory control inserts only the already required
`value <= 255` as `llvm.assume` at `bounded_byte`'s entry. Optimization then
leaves that erased assumption and a truncation. This control is not shipped
compiler behavior. It shows that conveying the proof could remove this cost
without changing the source conversion API. A proof-required conversion and
general proof-to-backend transport are therefore alternatives with different
scope; their benefit must not be counted twice.

### Float boundaries matter

The probe checks that exact `u32 -> f32` accepts 2^24 and rejects 2^24+1;
exact `f64 -> i32` rejects 1.75; integral truncation before conversion maps
-1.75 to -1; ties-to-even maps 2.5 to 2; and infinity remains a checked error.
Exact `f64 -> f32` accepts 1.5, rejects the f64 value denoted by 1.1, preserves
negative zero and produces the specified canonical NaN. `froundeven(1.1)`
would instead round to the integer-valued float 1.0, so it cannot supply the
missing float precision conversion.

The i32 saturation composition explicitly chooses NaN-to-zero and clamps to
endpoints exactly representable in f64. That construction does not generalize
by substituting i64 endpoints: i64::MAX is not exactly representable in f64.
The wide helper instead compares with the exact exclusive upper bound 2^63,
returns the integer maximum on that edge, and handles the lower bound and NaN
separately. It checks 2^63, its predecessor float, negative infinity and NaN.
These are interface obligations, not details an unspecified cast may choose.

## Options for the next design discussion

All spellings below are sketches, not accepted WF syntax or selected rules.

**A. Additive operations.** Keep existing `cvt` unchanged, add an explicitly
proved integer conversion such as `cvt.proved::<Src, Dst>(x)` returning Dst,
and add a total strict-width integer truncation such as
`itrunc::<Src, Dst>(x)`. This keeps current meanings and separates intent,
but retains the old type-pair-dependent `cvt` result shape alongside a second
exact-conversion interface. Migration cost alone is not a reason to prefer it.

**B. A uniform exact/checked/domain family.** Make bare `cvt` return Dst after
static domain proof, `cvt.checked` return Result, and `cvt.defined` return the
total domain predicate, on the same model as exact integer arithmetic. Total
type pairs discharge immediately. Truncation remains a separate bit operation.
This makes the selected failure policy explicit and simplifies a generic
function's result shape; it changes TYPE-4/OP-6 and every dependent contract,
operation inventory and conformance case. Integer range domains fit existing
proof terms, but float exactness needs a precisely specified predicate and
proof route; it must not be advertised as automatically inferable from ranges.
The concept is more coherent, but a full family still needs a rule-level trial.

**C. Keep the interface and improve evidence/optimization.** Publish the
checked integer Ok payload's equality through existing value-associated Result
evidence, and transport selected proven range facts to lowering. This can
recover the indexed example and remove the demonstrated redundant check without
new operation spellings. It leaves impossible-failure arms in source and does
not provide rounded float conversion or direct total truncation.

These options are partly composable. The recommended next comparison is B
against C for exact conversion, with A retained as the narrower alternative;
no option has been adopted. Do not change `cvt` to truncation or rounding under
its existing exact meaning. A checked equality extension must capture the
operand's value at conversion, survive only the specified copy/join routes,
and never relate a payload to a later replacement of its original variable.
Direct and named Results, source replacement, joins and loops need paired
controls before selecting the extension; deterministic completion without
implementation work budgets and proof-state cost remain obligations.

Direct rounded conversions to float and total float-to-integer saturation are
separate candidates. Their policy includes ties, overflow/underflow, signed
zero and NaNs. The [Rust numeric cast rules](https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast)
provide a useful comparison: narrowing integers keeps low bits; float-to-int
truncates and saturates with NaN-to-zero; conversions to float round to nearest,
ties-to-even. WF need not put those different contracts behind one spelling.
[LLVM saturating float-to-int intrinsics](https://llvm.org/docs/LangRef.html#llvm-fptosi-sat-intrinsic)
provide a defined total primitive for the former policy, while ordinary raw
float-to-int instructions have a domain obligation. A broad general cast or
software float-conversion library is not selected by this study.

## Reproduction and evidence boundary

Use the compiler built from `5dd9d5d7` (its compiler tree equals `033f44a9`).
The recorded probe run reused that already-built, gate-validated compiler;
the executable SHA-256 was
`c57f989b1b0073769d4999266f3353143d758b2f400c26a74e8d260b684afe33`.
The commands below use an existing scratch directory. Follow the repository's
shared verification guard when constructing/running native programs.

```sh
whitefootc --emit-llvm -o "$scratch/conversions.ll" research/investigations/numeric-conversions/probes.wf
clang -O2 -S -emit-llvm -x ir "$scratch/conversions.ll" -o "$scratch/conversions-O2.ll"
clang -O2 -S -x ir "$scratch/conversions.ll" -o "$scratch/conversions-O2.s"
perl .github/run-check.pl numeric-native-probe whitefootc -o "$scratch/conversions" research/investigations/numeric-conversions/probes.wf
"$scratch/conversions"
```

The three separate negative/capability probes use the same `--emit-llvm` path:
`proved-result-type.wf` expects FN-1 at its return;
`checked-equality.wf` expects OP-4 at the restored index;
`generic-conversion.wf` expects compiler `Unsupported: Generics`, not a source
rejection. The indexed control replaces only `values[restored]` with
`values[index]` and must be accepted.

The exploratory assumption control adds `declare void @llvm.assume(i1)` to
the emitted module and, immediately inside `wf_bounded_byte`'s entry, adds:

```llvm
%audit.range = icmp ule i32 %v0, 255
call void @llvm.assume(i1 %audit.range)
```

Apply the same optimization command and compare only that function. This is
a stated hypothetical transport of its existing precondition, not permission
to insert unchecked assumptions in source or implementation. Generated LLVM,
assembly and executables are scratch outputs and are not retained in the
repository. No compiler/specification/conformance file is changed by this
investigation, and no timing result or accepted new operation is claimed.
