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
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let i8_f32 = cvt<i8, f32>(-8_i8);
  if feq(i8_f32, -8.0_f32) {
  } else {
    return exit_status(code: 1_u8);
  }
  let i16_f32 = cvt<i16, f32>(32767_i16);
  if feq(i16_f32, 32767.0_f32) {
  } else {
    return exit_status(code: 2_u8);
  }
  let u8_f32 = cvt<u8, f32>(8_u8);
  if feq(u8_f32, 8.0_f32) {
  } else {
    return exit_status(code: 3_u8);
  }
  let u16_f32 = cvt<u16, f32>(65535_u16);
  if feq(u16_f32, 65535.0_f32) {
  } else {
    return exit_status(code: 4_u8);
  }
  let i8_f64 = cvt<i8, f64>(-8_i8);
  if feq(i8_f64, -8.0_f64) {
  } else {
    return exit_status(code: 5_u8);
  }
  let i16_f64 = cvt<i16, f64>(-16_i16);
  if feq(i16_f64, -16.0_f64) {
  } else {
    return exit_status(code: 6_u8);
  }
  let i32_f64 = cvt<i32, f64>(2147483647_i32);
  if feq(i32_f64, 2147483647.0_f64) {
  } else {
    return exit_status(code: 7_u8);
  }
  let u8_f64 = cvt<u8, f64>(8_u8);
  if feq(u8_f64, 8.0_f64) {
  } else {
    return exit_status(code: 8_u8);
  }
  let u16_f64 = cvt<u16, f64>(16_u16);
  if feq(u16_f64, 16.0_f64) {
  } else {
    return exit_status(code: 9_u8);
  }
  let u32_f64 = cvt<u32, f64>(4294967295_u32);
  if feq(u32_f64, 4294967295.0_f64) {
  } else {
    return exit_status(code: 10_u8);
  }
  let f32_f64 = cvt<f32, f64>(1.5_f32);
  if feq(f32_f64, 1.5_f64) {
  } else {
    return exit_status(code: 11_u8);
  }
  return exit_status(code: 0_u8);
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
    let mut source = String::from("command fn main() -> status: own ExitStatus pure {\n");
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
    source.push_str("  return exit_status(code: 0_u8);\n}\n");
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
    let source = br#"fn power_f32(exponent: own u32) -> result: own f32 pure {
  let value = 1.0_f32;
  let counter = 0_u32;
  loop @powers {
    let done = ieq(counter, exponent);
    if done {
      break @powers;
    }
    set value = fmul.strict(value, 2.0_f32);
    set counter = counter +wrap 1_u32;
  }
  return value;
}

fn power_f64(exponent: own u32) -> result: own f64 pure {
  let value = 1.0_f64;
  let counter = 0_u32;
  loop @powers {
    let done = ieq(counter, exponent);
    if done {
      break @powers;
    }
    set value = fmul.strict(value, 2.0_f64);
    set counter = counter +wrap 1_u32;
  }
  return value;
}

fn reject_f32_i32(value: own f32) -> result: own Bool pure {
  let rejected = False();
  match cvt<f32, i32>(value) {
    Ok(value: converted) => {
    }
    Err(error: narrow) => {
      set rejected = True();
    }
  }
  return rejected;
}

fn reject_f32_u32(value: own f32) -> result: own Bool pure {
  let rejected = False();
  match cvt<f32, u32>(value) {
    Ok(value: converted) => {
    }
    Err(error: narrow) => {
      set rejected = True();
    }
  }
  return rejected;
}

fn reject_f64_i64(value: own f64) -> result: own Bool pure {
  let rejected = False();
  match cvt<f64, i64>(value) {
    Ok(value: converted) => {
    }
    Err(error: narrow) => {
      set rejected = True();
    }
  }
  return rejected;
}

fn reject_f64_u64(value: own f64) -> result: own Bool pure {
  let rejected = False();
  match cvt<f64, u64>(value) {
    Ok(value: converted) => {
    }
    Err(error: narrow) => {
      set rejected = True();
    }
  }
  return rejected;
}

command fn main() -> status: own ExitStatus pure {
  let i32_boundary = power_f32(exponent: 31_u32);
  let rejected_i32_boundary = reject_f32_i32(value: i32_boundary);
  if rejected_i32_boundary {
  } else {
    return exit_status(code: 1_u8);
  }
  let u32_boundary = power_f32(exponent: 32_u32);
  let rejected_u32_boundary = reject_f32_u32(value: u32_boundary);
  if rejected_u32_boundary {
  } else {
    return exit_status(code: 2_u8);
  }
  let i64_boundary = power_f64(exponent: 63_u32);
  let rejected_i64_boundary = reject_f64_i64(value: i64_boundary);
  if rejected_i64_boundary {
  } else {
    return exit_status(code: 3_u8);
  }
  let u64_boundary = power_f64(exponent: 64_u32);
  let rejected_u64_boundary = reject_f64_u64(value: u64_boundary);
  if rejected_u64_boundary {
  } else {
    return exit_status(code: 4_u8);
  }
  let nan_f32 = fnan<f32>();
  let rejected_nan_f32 = reject_f32_i32(value: nan_f32);
  if rejected_nan_f32 {
  } else {
    return exit_status(code: 5_u8);
  }
  let infinity_f32 = finf<f32>();
  let rejected_infinity_f32 = reject_f32_i32(value: infinity_f32);
  if rejected_infinity_f32 {
  } else {
    return exit_status(code: 6_u8);
  }
  let infinity_f64 = finf<f64>();
  let negative_infinity = fneg(infinity_f64);
  let rejected_negative_infinity = reject_f64_u64(value: negative_infinity);
  if rejected_negative_infinity {
  } else {
    return exit_status(code: 7_u8);
  }
  let two_to_52 = power_f64(exponent: 52_u32);
  let one_ulp = fdiv.strict(1.0_f64, two_to_52);
  let not_f32 = fadd.strict(1.0_f64, one_ulp);
  match cvt<f64, f32>(not_f32) {
    Ok(value: rounded) => {
      return exit_status(code: 8_u8);
    }
    Err(error: narrow) => {
    }
  }
  let nan_f64 = fnan<f64>();
  match cvt<f64, f32>(nan_f64) {
    Ok(value: narrow_nan) => {
      if fne(narrow_nan, narrow_nan) {
      } else {
        return exit_status(code: 9_u8);
      }
    }
    Err(error: narrow_error) => {
      return exit_status(code: 10_u8);
    }
  }
  let narrowable_infinity = finf<f64>();
  match cvt<f64, f32>(narrowable_infinity) {
    Ok(value: narrow_infinity) => {
      let expected_infinity = finf<f32>();
      if feq(narrow_infinity, expected_infinity) {
      } else {
        return exit_status(code: 11_u8);
      }
    }
    Err(error: infinity_error) => {
      return exit_status(code: 12_u8);
    }
  }
  let narrow_nan_source = fnan<f32>();
  let wide_nan = cvt<f32, f64>(narrow_nan_source);
  if fne(wide_nan, wide_nan) {
  } else {
    return exit_status(code: 13_u8);
  }
  return exit_status(code: 0_u8);
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
    // [OP-7] the domain prefix is the only difference: `ieq` and `feq` are
    // both named calls, so the two destinations share one call shape.
    let domain = if destination_type.kind == NumericKind::Float {
        'f'
    } else {
        'i'
    };
    let equality = format!("{domain}eq(success_value{conversion}, {destination_value})");
    writeln!(
        source,
        "  let success{conversion} = cvt<{source_type}, {destination}>({source_value});\n  match move success{conversion} {{\n    Ok(value: success_value{conversion}) => {{\n      if {equality} {{\n      }} else {{\n        return exit_status(code: 1_u8);\n      }}\n    }}\n    Err(error: success_error{conversion}) => {{\n      return exit_status(code: 1_u8);\n    }}\n  }}",
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
        "  let failure{conversion} = cvt<{source_type}, {destination}>({source_value});\n  match move failure{conversion} {{\n    Ok(value: failure_value{conversion}) => {{\n      return exit_status(code: 1_u8);\n    }}\n    Err(error: failure_error{conversion}) => {{\n    }}\n  }}",
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
