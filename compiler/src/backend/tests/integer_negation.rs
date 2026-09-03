use super::{compile, compile_and_run};

#[test]
fn executes_every_negation_mode_for_every_signed_width() {
    let template = r#"command fn main() -> status: own ExitStatus pure {
  let wrapped = ineg.wrap($MIN_$TYPE);
  if wrapped == $MIN_$TYPE {
  } else {
    return exit_status(code: 1_u8);
  }
  let exact = ineg(-42_$TYPE);
  if exact == 42_$TYPE {
  } else {
    return exit_status(code: 2_u8);
  }
  let ordinary_defined = ineg.defined(-42_$TYPE);
  if ordinary_defined {
  } else {
    return exit_status(code: 3_u8);
  }
  let minimum_defined = ineg.defined($MIN_$TYPE);
  if bnot(minimum_defined) {
  } else {
    return exit_status(code: 4_u8);
  }
  let safe_result = ineg.checked(-42_$TYPE);
  match move safe_result {
    Ok(value: safe_value) => {
      if safe_value == 42_$TYPE {
      } else {
        return exit_status(code: 5_u8);
      }
    }
    Err(error: safe_error) => {
      return exit_status(code: 6_u8);
    }
  }
  let overflow_result = ineg.checked($MIN_$TYPE);
  match move overflow_result {
    Ok(value: overflow_value) => {
      return exit_status(code: 7_u8);
    }
    Err(error: overflow_error) => {
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for (ty, width, minimum) in [
        ("i8", 8, "-128"),
        ("i16", 16, "-32768"),
        ("i32", 32, "-2147483648"),
        ("i64", 64, "-9223372036854775808"),
    ] {
        let source = template.replace("$TYPE", ty).replace("$MIN", minimum);
        let llvm = compile(source.as_bytes());
        let intrinsic = format!("@llvm.ssub.with.overflow.i{width}");
        assert!(
            llvm.contains(&format!("sub i{width} 0,")),
            "wrapping and proved-exact negation must be plain subtraction for {ty}"
        );
        assert!(
            llvm.matches(&format!("{intrinsic}(i{width}")).count() == 3,
            "only the declaration and two checked {ty} negations need the overflow intrinsic"
        );
        assert!(llvm.contains(&format!("icmp ne i{width}")));
        assert!(!llvm.contains(" nsw "));
        assert!(!llvm.contains(" nuw "));
        let output = compile_and_run(&llvm);
        assert!(
            output.status.success(),
            "negation program failed for {ty}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn defined_minimum_reports_false_without_executing_negation() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let is_defined = ineg.defined(-128_i8);
  if bnot(is_defined) {
  } else {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("icmp ne i8"));
    assert!(!llvm.contains("sub i8 0, -128"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}
