use super::{compile, compile_and_run};

#[test]
fn every_direct_float_operation_executes_for_both_widths() {
    let template = r#"fn main() -> own unit traps {
  let sum = fadd.strict(1.5_$TYPE, 2.25_$TYPE);
  check feq(sum, 3.75_$TYPE) else trap "fadd";
  let difference = fsub.strict(sum, 0.75_$TYPE);
  check feq(difference, 3.0_$TYPE) else trap "fsub";
  let product = fmul.strict(difference, 2.0_$TYPE);
  check feq(product, 6.0_$TYPE) else trap "fmul";
  let quotient = fdiv.strict(product, 4.0_$TYPE);
  check feq(quotient, 1.5_$TYPE) else trap "fdiv";
  let negative = fneg(quotient);
  check feq(negative, -1.5_$TYPE) else trap "fneg";
  let absolute = fabs(negative);
  check feq(absolute, 1.5_$TYPE) else trap "fabs";
  let signed = fcopysign(absolute, negative);
  check feq(signed, -1.5_$TYPE) else trap "fcopysign";
  let minimum = fmin(negative, absolute);
  check feq(minimum, -1.5_$TYPE) else trap "fmin";
  let maximum = fmax(negative, absolute);
  check feq(maximum, 1.5_$TYPE) else trap "fmax";
  let floor = ffloor(1.75_$TYPE);
  check feq(floor, 1.0_$TYPE) else trap "ffloor";
  let ceil = fceil(1.25_$TYPE);
  check feq(ceil, 2.0_$TYPE) else trap "fceil";
  let truncated = ftrunc(-1.75_$TYPE);
  check feq(truncated, -1.0_$TYPE) else trap "ftrunc";
  let rounded = froundeven(2.5_$TYPE);
  check feq(rounded, 2.0_$TYPE) else trap "froundeven";
  let remainder = frem(5.5_$TYPE, 2.0_$TYPE);
  check feq(remainder, 1.5_$TYPE) else trap "frem";
  let root = fsqrt.strict(4.0_$TYPE);
  check feq(root, 2.0_$TYPE) else trap "fsqrt.strict";
  let fused = ffma.strict(2.0_$TYPE, 3.0_$TYPE, 1.0_$TYPE);
  check feq(fused, 7.0_$TYPE) else trap "ffma.strict";
  let infinity = finf<$TYPE>();
  check fgt(infinity, fused) else trap "finf";
  let negative_infinity = fneg(infinity);
  check flt(negative_infinity, negative) else trap "negative infinity";
  let nan = fnan<$TYPE>();
  check fne(nan, nan) else trap "fnan";
  let minimum_nan = fmin(nan, fused);
  check fne(minimum_nan, minimum_nan) else trap "fmin NaN propagation";
  let negative_zero = fneg(0.0_$TYPE);
  let minimum_zero = fmin(negative_zero, 0.0_$TYPE);
  let minimum_reciprocal = fdiv.strict(1.0_$TYPE, minimum_zero);
  check feq(minimum_reciprocal, negative_infinity) else trap "fmin signed zero";
  let maximum_zero = fmax(negative_zero, 0.0_$TYPE);
  let maximum_reciprocal = fdiv.strict(1.0_$TYPE, maximum_zero);
  check feq(maximum_reciprocal, infinity) else trap "fmax signed zero";
  check fle(absolute, sum) else trap "fle";
  check fge(sum, absolute) else trap "fge";
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
  check feq(stepwise, one) else trap "each strict addition must round on its own";
  check fne(stepwise, regrouped) else trap "strict addition was reassociated";
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
  check feq(loaded, 1.5_f32) else trap "float storage";
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
