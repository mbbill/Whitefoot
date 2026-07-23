use super::{compile, compile_and_run};

#[test]
fn every_direct_float_operation_executes_for_both_widths() {
    let template = r#"fn main() -> own unit traps {
  let sum: own $TYPE = fadd.strict<$TYPE>(1.5_$TYPE, 2.25_$TYPE);
  check feq<$TYPE>(sum, 3.75_$TYPE) else trap "fadd";
  let difference: own $TYPE = fsub.strict<$TYPE>(sum, 0.75_$TYPE);
  check feq<$TYPE>(difference, 3.0_$TYPE) else trap "fsub";
  let product: own $TYPE = fmul.strict<$TYPE>(difference, 2.0_$TYPE);
  check feq<$TYPE>(product, 6.0_$TYPE) else trap "fmul";
  let quotient: own $TYPE = fdiv.strict<$TYPE>(product, 4.0_$TYPE);
  check feq<$TYPE>(quotient, 1.5_$TYPE) else trap "fdiv";
  let negative: own $TYPE = fneg<$TYPE>(quotient);
  check feq<$TYPE>(negative, -1.5_$TYPE) else trap "fneg";
  let absolute: own $TYPE = fabs<$TYPE>(negative);
  check feq<$TYPE>(absolute, 1.5_$TYPE) else trap "fabs";
  let signed: own $TYPE = fcopysign<$TYPE>(absolute, negative);
  check feq<$TYPE>(signed, -1.5_$TYPE) else trap "fcopysign";
  let minimum: own $TYPE = fmin<$TYPE>(negative, absolute);
  check feq<$TYPE>(minimum, -1.5_$TYPE) else trap "fmin";
  let maximum: own $TYPE = fmax<$TYPE>(negative, absolute);
  check feq<$TYPE>(maximum, 1.5_$TYPE) else trap "fmax";
  let floor: own $TYPE = ffloor<$TYPE>(1.75_$TYPE);
  check feq<$TYPE>(floor, 1.0_$TYPE) else trap "ffloor";
  let ceil: own $TYPE = fceil<$TYPE>(1.25_$TYPE);
  check feq<$TYPE>(ceil, 2.0_$TYPE) else trap "fceil";
  let truncated: own $TYPE = ftrunc<$TYPE>(-1.75_$TYPE);
  check feq<$TYPE>(truncated, -1.0_$TYPE) else trap "ftrunc";
  let rounded: own $TYPE = froundeven<$TYPE>(2.5_$TYPE);
  check feq<$TYPE>(rounded, 2.0_$TYPE) else trap "froundeven";
  let remainder: own $TYPE = frem<$TYPE>(5.5_$TYPE, 2.0_$TYPE);
  check feq<$TYPE>(remainder, 1.5_$TYPE) else trap "frem";
  let root: own $TYPE = fsqrt.strict<$TYPE>(4.0_$TYPE);
  check feq<$TYPE>(root, 2.0_$TYPE) else trap "fsqrt.strict";
  let fused: own $TYPE = ffma.strict<$TYPE>(2.0_$TYPE, 3.0_$TYPE, 1.0_$TYPE);
  check feq<$TYPE>(fused, 7.0_$TYPE) else trap "ffma.strict";
  let infinity: own $TYPE = finf<$TYPE>();
  check fgt<$TYPE>(infinity, fused) else trap "finf";
  let negative_infinity: own $TYPE = fneg<$TYPE>(infinity);
  check flt<$TYPE>(negative_infinity, negative) else trap "negative infinity";
  let nan: own $TYPE = fnan<$TYPE>();
  check fne<$TYPE>(nan, nan) else trap "fnan";
  let minimum_nan: own $TYPE = fmin<$TYPE>(nan, fused);
  check fne<$TYPE>(minimum_nan, minimum_nan) else trap "fmin NaN propagation";
  let negative_zero: own $TYPE = fneg<$TYPE>(0.0_$TYPE);
  let minimum_zero: own $TYPE = fmin<$TYPE>(negative_zero, 0.0_$TYPE);
  let minimum_reciprocal: own $TYPE = fdiv.strict<$TYPE>(1.0_$TYPE, minimum_zero);
  check feq<$TYPE>(minimum_reciprocal, negative_infinity) else trap "fmin signed zero";
  let maximum_zero: own $TYPE = fmax<$TYPE>(negative_zero, 0.0_$TYPE);
  let maximum_reciprocal: own $TYPE = fdiv.strict<$TYPE>(1.0_$TYPE, maximum_zero);
  check feq<$TYPE>(maximum_reciprocal, infinity) else trap "fmax signed zero";
  check fle<$TYPE>(absolute, sum) else trap "fle";
  check fge<$TYPE>(sum, absolute) else trap "fge";
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

#[test]
fn float_constants_work_in_aggregates_arrays_and_buffers() {
    let source = br#"struct Sample {
  value: f32;
}

const values: array<f32, 2> = [1.5_f32, 2.5_f32];

fn main() -> own unit allocates(heap), traps {
  let sample: own Sample = Sample(value: index<f32>(values, 0_u64));
  let storage: own buffer<f32> = buffer_new<f32>(2_u64, 0.0_f32);
  set index<f32>(storage, 1_u64) = sample.value;
  let loaded: own f32 = index<f32>(storage, 1_u64);
  check feq<f32>(loaded, 1.5_f32) else trap "float storage";
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
