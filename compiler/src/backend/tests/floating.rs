use super::{compile, compile_and_run};

#[test]
fn every_direct_float_operation_executes_for_both_widths() {
    let template = r#"fn main() -> own unit traps {
  let sum = fadd.strict(1.5_$TYPE, 2.25_$TYPE);
  claim fadd: feq(sum, 3.75_$TYPE) because "fadd";
  let difference = fsub.strict(sum, 0.75_$TYPE);
  claim fsub: feq(difference, 3.0_$TYPE) because "fsub";
  let product = fmul.strict(difference, 2.0_$TYPE);
  claim fmul: feq(product, 6.0_$TYPE) because "fmul";
  let quotient = fdiv.strict(product, 4.0_$TYPE);
  claim fdiv: feq(quotient, 1.5_$TYPE) because "fdiv";
  let negative = fneg(quotient);
  claim fneg: feq(negative, -1.5_$TYPE) because "fneg";
  let absolute = fabs(negative);
  claim fabs: feq(absolute, 1.5_$TYPE) because "fabs";
  let signed = fcopysign(absolute, negative);
  claim fcopysign: feq(signed, -1.5_$TYPE) because "fcopysign";
  let minimum = fmin(negative, absolute);
  claim fmin: feq(minimum, -1.5_$TYPE) because "fmin";
  let maximum = fmax(negative, absolute);
  claim fmax: feq(maximum, 1.5_$TYPE) because "fmax";
  let floor = ffloor(1.75_$TYPE);
  claim ffloor: feq(floor, 1.0_$TYPE) because "ffloor";
  let ceil = fceil(1.25_$TYPE);
  claim fceil: feq(ceil, 2.0_$TYPE) because "fceil";
  let truncated = ftrunc(-1.75_$TYPE);
  claim ftrunc: feq(truncated, -1.0_$TYPE) because "ftrunc";
  let rounded = froundeven(2.5_$TYPE);
  claim froundeven: feq(rounded, 2.0_$TYPE) because "froundeven";
  let remainder = frem(5.5_$TYPE, 2.0_$TYPE);
  claim frem: feq(remainder, 1.5_$TYPE) because "frem";
  let root = fsqrt.strict(4.0_$TYPE);
  claim fsqrt_strict: feq(root, 2.0_$TYPE) because "fsqrt.strict";
  let fused = ffma.strict(2.0_$TYPE, 3.0_$TYPE, 1.0_$TYPE);
  claim ffma_strict: feq(fused, 7.0_$TYPE) because "ffma.strict";
  let infinity = finf<$TYPE>();
  claim finf: fgt(infinity, fused) because "finf";
  let negative_infinity = fneg(infinity);
  claim negative_infinity: flt(negative_infinity, negative) because "negative infinity";
  let nan = fnan<$TYPE>();
  claim fnan: fne(nan, nan) because "fnan";
  let minimum_nan = fmin(nan, fused);
  claim fmin_nan_propagation: fne(minimum_nan, minimum_nan) because "fmin NaN propagation";
  let negative_zero = fneg(0.0_$TYPE);
  let minimum_zero = fmin(negative_zero, 0.0_$TYPE);
  let minimum_reciprocal = fdiv.strict(1.0_$TYPE, minimum_zero);
  claim fmin_signed_zero: feq(minimum_reciprocal, negative_infinity) because "fmin signed zero";
  let maximum_zero = fmax(negative_zero, 0.0_$TYPE);
  let maximum_reciprocal = fdiv.strict(1.0_$TYPE, maximum_zero);
  claim fmax_signed_zero: feq(maximum_reciprocal, infinity) because "fmax signed zero";
  claim fle: fle(absolute, sum) because "fle";
  claim fge: fge(sum, absolute) because "fge";
  return unit;
}
"#;
    for ty in ["f32", "f64"] {
        let llvm = compile(template.replace("$TYPE", ty).as_bytes());
        for forbidden in ["fadd fast", "fsub fast", "fmul fast", "fdiv fast"] {
            assert!(!llvm.contains(forbidden));
        }
        for intrinsic in [
            "llvm.fabs.",
            "llvm.copysign.",
            "llvm.minimum.",
            "llvm.maximum.",
            "llvm.floor.",
            "llvm.ceil.",
            "llvm.trunc.",
            "llvm.roundeven.",
            "llvm.sqrt.",
            "llvm.fma.",
        ] {
            assert!(llvm.contains(intrinsic), "missing {intrinsic} for {ty}");
        }
        let output = compile_and_run(&llvm);
        assert!(
            output.status.success(),
            "float operations failed for {ty}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

/// [OP-3] the rounding float ops carry `.strict`: IEEE 754, no reassociation,
/// no contraction. The absent `fast` flags the test above asserts are a
/// statement about the emitted module; this one is a statement about the
/// executed program, because the link step optimizes at the same level every
/// Whitefoot executable uses.
///
/// The witness is an addition that is not associative in binary32. With
/// `half_ulp` below one half of 1.0's ulp, `(1.0 + h) + h` rounds each step
/// back to 1.0, while `1.0 + (h + h)` rounds once and lands one ulp above it.
/// A run that reassociated, contracted, or widened any of the four additions
/// would make the two results agree and trap.
///
/// A green run establishes exactly that: it does not establish that any other
/// float op resists reassociation, only these additions under this host and
/// optimization level.
#[test]
fn strict_float_addition_rounds_every_step_and_is_never_reassociated() {
    let source = br#"fn left(a: own f32, b: own f32, c: own f32) -> own f32 pure {
  let ab = fadd.strict(a, b);
  return fadd.strict(ab, c);
}

fn right(a: own f32, b: own f32, c: own f32) -> own f32 pure {
  let bc = fadd.strict(b, c);
  return fadd.strict(a, bc);
}

fn main() -> own unit traps {
  let one = 1.0_f32;
  let half_ulp = 4.0e-8_f32;
  let stepwise = left(a: one, b: half_ulp, c: half_ulp);
  let regrouped = right(a: one, b: half_ulp, c: half_ulp);
  claim each_strict_addition_must_round_on_its_own: feq(stepwise, one) because "each strict addition must round on its own";
  claim strict_addition_was_reassociated: fne(stepwise, regrouped) because "strict addition was reassociated";
  return unit;
}
"#;
    let llvm = compile(source);
    for forbidden in ["fadd fast", "reassoc", "contract"] {
        assert!(
            !llvm.contains(forbidden),
            "strict addition must not carry {forbidden}"
        );
    }
    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "strict float association failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn float_constants_work_in_aggregates_arrays_and_buffers() {
    let source = br#"struct Sample {
  value: f32;
}

const values: array<f32, 2> =[1.5_f32, 2.5_f32];

fn main() -> own unit allocates(heap), traps {
  let sample = Sample(value: values[0_u64]);
  let storage = buffer_new(2_u64, 0.0_f32);
  set storage[1_u64] = sample.value;
  let loaded = storage[1_u64];
  claim float_storage: feq(loaded, 1.5_f32) because "float storage";
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(
        output.status.success(),
        "float storage failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
