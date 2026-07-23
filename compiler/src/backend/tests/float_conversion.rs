use std::fmt::Write;

use super::{compile, compile_and_run};

#[derive(Clone, Copy, Eq, PartialEq)]
enum NumericKind {
    SignedInteger,
    UnsignedInteger,
    Float,
}

#[derive(Clone, Copy)]
struct NumericType {
    spelling: &'static str,
    width: u8,
    kind: NumericKind,
}

const NUMERIC_TYPES: [NumericType; 10] = [
    NumericType {
        spelling: "i8",
        width: 8,
        kind: NumericKind::SignedInteger,
    },
    NumericType {
        spelling: "i16",
        width: 16,
        kind: NumericKind::SignedInteger,
    },
    NumericType {
        spelling: "i32",
        width: 32,
        kind: NumericKind::SignedInteger,
    },
    NumericType {
        spelling: "i64",
        width: 64,
        kind: NumericKind::SignedInteger,
    },
    NumericType {
        spelling: "u8",
        width: 8,
        kind: NumericKind::UnsignedInteger,
    },
    NumericType {
        spelling: "u16",
        width: 16,
        kind: NumericKind::UnsignedInteger,
    },
    NumericType {
        spelling: "u32",
        width: 32,
        kind: NumericKind::UnsignedInteger,
    },
    NumericType {
        spelling: "u64",
        width: 64,
        kind: NumericKind::UnsignedInteger,
    },
    NumericType {
        spelling: "f32",
        width: 32,
        kind: NumericKind::Float,
    },
    NumericType {
        spelling: "f64",
        width: 64,
        kind: NumericKind::Float,
    },
];

#[test]
fn every_total_conversion_with_a_float_endpoint_executes() {
    let source = br#"fn main() -> own unit traps {
  let i8_f32: own f32 = cvt<i8, f32>(-8_i8);
  check feq<f32>(i8_f32, -8.0_f32) else trap "i8 to f32";
  let i16_f32: own f32 = cvt<i16, f32>(32767_i16);
  check feq<f32>(i16_f32, 32767.0_f32) else trap "i16 to f32";
  let u8_f32: own f32 = cvt<u8, f32>(8_u8);
  check feq<f32>(u8_f32, 8.0_f32) else trap "u8 to f32";
  let u16_f32: own f32 = cvt<u16, f32>(65535_u16);
  check feq<f32>(u16_f32, 65535.0_f32) else trap "u16 to f32";
  let i8_f64: own f64 = cvt<i8, f64>(-8_i8);
  check feq<f64>(i8_f64, -8.0_f64) else trap "i8 to f64";
  let i16_f64: own f64 = cvt<i16, f64>(-16_i16);
  check feq<f64>(i16_f64, -16.0_f64) else trap "i16 to f64";
  let i32_f64: own f64 = cvt<i32, f64>(2147483647_i32);
  check feq<f64>(i32_f64, 2147483647.0_f64) else trap "i32 to f64";
  let u8_f64: own f64 = cvt<u8, f64>(8_u8);
  check feq<f64>(u8_f64, 8.0_f64) else trap "u8 to f64";
  let u16_f64: own f64 = cvt<u16, f64>(16_u16);
  check feq<f64>(u16_f64, 16.0_f64) else trap "u16 to f64";
  let u32_f64: own f64 = cvt<u32, f64>(4294967295_u32);
  check feq<f64>(u32_f64, 4294967295.0_f64) else trap "u32 to f64";
  let f32_f64: own f64 = cvt<f32, f64>(1.5_f32);
  check feq<f64>(f32_f64, 1.5_f64) else trap "f32 to f64";
  return unit;
}
"#;
    let llvm = compile(source);
    for instruction in [
        "sitofp i8",
        "sitofp i16",
        "sitofp i32",
        "uitofp i8",
        "uitofp i16",
        "uitofp i32",
        "fpext float",
    ] {
        assert!(
            llvm.contains(instruction),
            "total conversion family must exercise {instruction}"
        );
    }
    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "total floating conversion family failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn every_partial_conversion_with_a_float_endpoint_has_exact_success_and_failure() {
    let mut source = String::from("fn main() -> own unit traps {\n");
    let mut conversion = 0;
    for source_type in NUMERIC_TYPES {
        for destination_type in NUMERIC_TYPES {
            if source_type.spelling == destination_type.spelling
                || !has_float_endpoint(source_type, destination_type)
                || converts_totally(source_type, destination_type)
            {
                continue;
            }
            emit_success_case(&mut source, conversion, source_type, destination_type);
            emit_failure_case(&mut source, conversion, source_type, destination_type);
            conversion += 1;
        }
    }
    source.push_str("  return unit;\n}\n");
    assert_eq!(conversion, 23);

    let llvm = compile(source.as_bytes());
    for instruction in [
        "@llvm.fptosi.sat.",
        "@llvm.fptoui.sat.",
        "fptrunc double",
        "fpext float",
    ] {
        assert!(
            llvm.contains(instruction),
            "partial conversion matrix must exercise {instruction}"
        );
    }
    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "partial floating conversion matrix failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn partial_conversion_boundaries_never_execute_poisoning_llvm_casts() {
    let source = br#"fn power_f32(exponent: own u32) -> own f32 pure {
  let value: own f32 = 1.0_f32;
  let counter: own u32 = 0_u32;
  loop @powers {
    let done: own Bool = ieq<u32>(counter, exponent);
    match done {
      True() => {
        break @powers;
      }
      False() => {
      }
    }
    set value = fmul.strict<f32>(value, 2.0_f32);
    set counter = iadd.wrap<u32>(counter, 1_u32);
  }
  return value;
}

fn power_f64(exponent: own u32) -> own f64 pure {
  let value: own f64 = 1.0_f64;
  let counter: own u32 = 0_u32;
  loop @powers {
    let done: own Bool = ieq<u32>(counter, exponent);
    match done {
      True() => {
        break @powers;
      }
      False() => {
      }
    }
    set value = fmul.strict<f64>(value, 2.0_f64);
    set counter = iadd.wrap<u32>(counter, 1_u32);
  }
  return value;
}

fn reject_f32_i32(value: own f32) -> own unit traps {
  match cvt<f32, i32>(value) {
    Ok(value: converted) => {
      check False() else trap "f32 to i32 boundary succeeded";
    }
    Err(error: narrow) => {
    }
  }
  return unit;
}

fn reject_f32_u32(value: own f32) -> own unit traps {
  match cvt<f32, u32>(value) {
    Ok(value: converted) => {
      check False() else trap "f32 to u32 boundary succeeded";
    }
    Err(error: narrow) => {
    }
  }
  return unit;
}

fn reject_f64_i64(value: own f64) -> own unit traps {
  match cvt<f64, i64>(value) {
    Ok(value: converted) => {
      check False() else trap "f64 to i64 boundary succeeded";
    }
    Err(error: narrow) => {
    }
  }
  return unit;
}

fn reject_f64_u64(value: own f64) -> own unit traps {
  match cvt<f64, u64>(value) {
    Ok(value: converted) => {
      check False() else trap "f64 to u64 boundary succeeded";
    }
    Err(error: narrow) => {
    }
  }
  return unit;
}

fn main() -> own unit traps {
  let i32_boundary: own f32 = power_f32(exponent: 31_u32);
  reject_f32_i32(value: i32_boundary);
  let u32_boundary: own f32 = power_f32(exponent: 32_u32);
  reject_f32_u32(value: u32_boundary);
  let i64_boundary: own f64 = power_f64(exponent: 63_u32);
  reject_f64_i64(value: i64_boundary);
  let u64_boundary: own f64 = power_f64(exponent: 64_u32);
  reject_f64_u64(value: u64_boundary);
  let nan_f32: own f32 = fnan<f32>();
  reject_f32_i32(value: nan_f32);
  let infinity_f32: own f32 = finf<f32>();
  reject_f32_i32(value: infinity_f32);
  let infinity_f64: own f64 = finf<f64>();
  let negative_infinity: own f64 = fneg<f64>(infinity_f64);
  reject_f64_u64(value: negative_infinity);
  let two_to_52: own f64 = power_f64(exponent: 52_u32);
  let one_ulp: own f64 = fdiv.strict<f64>(1.0_f64, two_to_52);
  let not_f32: own f64 = fadd.strict<f64>(1.0_f64, one_ulp);
  match cvt<f64, f32>(not_f32) {
    Ok(value: rounded) => {
      check False() else trap "inexact f64 to f32 succeeded";
    }
    Err(error: narrow) => {
    }
  }
  let nan_f64: own f64 = fnan<f64>();
  match cvt<f64, f32>(nan_f64) {
    Ok(value: narrow_nan) => {
      check fne<f32>(narrow_nan, narrow_nan) else trap "narrow NaN";
    }
    Err(error: narrow_error) => {
      check False() else trap "NaN conversion failed";
    }
  }
  let narrowable_infinity: own f64 = finf<f64>();
  match cvt<f64, f32>(narrowable_infinity) {
    Ok(value: narrow_infinity) => {
      let expected_infinity: own f32 = finf<f32>();
      check feq<f32>(narrow_infinity, expected_infinity) else trap "narrow infinity";
    }
    Err(error: infinity_error) => {
      check False() else trap "infinity conversion failed";
    }
  }
  let narrow_nan_source: own f32 = fnan<f32>();
  let wide_nan: own f64 = cvt<f32, f64>(narrow_nan_source);
  check fne<f64>(wide_nan, wide_nan) else trap "wide NaN";
  return unit;
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("@llvm.fptosi.sat.i32.f32"));
    assert!(llvm.contains("@llvm.fptoui.sat.i64.f64"));
    assert!(!llvm.contains(" = fptosi "));
    assert!(!llvm.contains(" = fptoui "));
    assert!(llvm.contains("fcmp uno"));
    assert!(llvm.contains("0x7FF8000000000000"));

    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "partial conversion boundary failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

fn emit_success_case(
    source: &mut String,
    conversion: usize,
    source_type: NumericType,
    destination_type: NumericType,
) {
    let source_value = if source_type.kind == NumericKind::Float {
        format!("1.0_{}", source_type.spelling)
    } else {
        format!("1_{}", source_type.spelling)
    };
    let destination_value = if destination_type.kind == NumericKind::Float {
        format!("1.0_{}", destination_type.spelling)
    } else {
        format!("1_{}", destination_type.spelling)
    };
    let equality = if destination_type.kind == NumericKind::Float {
        "feq"
    } else {
        "ieq"
    };
    writeln!(
        source,
        "  let success{conversion}: own Result<{destination}, NarrowError> = cvt<{source_type}, {destination}>({source_value});\n  match move success{conversion} {{\n    Ok(value: success_value{conversion}) => {{\n      check {equality}<{destination}>(success_value{conversion}, {destination_value}) else trap \"partial success value {conversion}\";\n    }}\n    Err(error: success_error{conversion}) => {{\n      check False() else trap \"partial success became error {conversion}\";\n    }}\n  }}",
        destination = destination_type.spelling,
        source_type = source_type.spelling,
    )
    .expect("write partial success case");
}

fn emit_failure_case(
    source: &mut String,
    conversion: usize,
    source_type: NumericType,
    destination_type: NumericType,
) {
    let source_value = match (source_type.kind, destination_type.kind) {
        (NumericKind::SignedInteger, NumericKind::Float) => {
            format!(
                "{}_{}",
                (1_u128 << (source_type.width - 1)) - 1,
                source_type.spelling
            )
        }
        (NumericKind::UnsignedInteger, NumericKind::Float) => {
            format!(
                "{}_{}",
                (1_u128 << source_type.width) - 1,
                source_type.spelling
            )
        }
        (NumericKind::Float, NumericKind::SignedInteger | NumericKind::UnsignedInteger) => {
            format!("1.5_{}", source_type.spelling)
        }
        (NumericKind::Float, NumericKind::Float) => "1.0000000000000002_f64".to_owned(),
        _ => panic!("selected conversion must have a float endpoint"),
    };
    writeln!(
        source,
        "  let failure{conversion}: own Result<{destination}, NarrowError> = cvt<{source_type}, {destination}>({source_value});\n  match move failure{conversion} {{\n    Ok(value: failure_value{conversion}) => {{\n      check False() else trap \"inexact conversion succeeded {conversion}\";\n    }}\n    Err(error: failure_error{conversion}) => {{\n    }}\n  }}",
        destination = destination_type.spelling,
        source_type = source_type.spelling,
    )
    .expect("write partial failure case");
}

const fn has_float_endpoint(source: NumericType, destination: NumericType) -> bool {
    matches!(source.kind, NumericKind::Float) || matches!(destination.kind, NumericKind::Float)
}

const fn converts_totally(source: NumericType, destination: NumericType) -> bool {
    match (source.kind, destination.kind) {
        (NumericKind::SignedInteger | NumericKind::UnsignedInteger, NumericKind::Float) => {
            (destination.width == 32 && source.width <= 16)
                || (destination.width == 64 && source.width <= 32)
        }
        (NumericKind::Float, NumericKind::Float) => source.width == 32 && destination.width == 64,
        _ => false,
    }
}
