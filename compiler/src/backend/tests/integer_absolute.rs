use super::{compile, compile_and_run};

#[test]
fn executes_every_absolute_mode_for_every_signed_width() {
    let template = r#"fn main() -> own unit traps {
  let wrapped = iabs.wrap($MIN_$TYPE);
  claim wrapped_absolute_value_drift: ieq(wrapped, $MIN_$TYPE) because "wrapped absolute value drift";
  let exact = iabs(-42_$TYPE);
  claim exact_absolute_value_drift: ieq(exact, 42_$TYPE) because "exact absolute value drift";
  let ordinary_defined = iabs.defined(-42_$TYPE);
  claim ordinary_absolute_value_is_defined: ordinary_defined because "ordinary absolute value must be defined";
  let minimum_defined = iabs.defined($MIN_$TYPE);
  claim minimum_absolute_value_is_undefined: bnot(minimum_defined) because "minimum absolute value must be undefined";
  let safe_result = iabs.checked(-42_$TYPE);
  match move safe_result {
    Ok(value: safe_value) => {
      claim checked_absolute_value_drift: ieq(safe_value, 42_$TYPE) because "checked absolute value drift";
    }
    Err(error: safe_error) => {
      claim safe_absolute_value_took_err: False() because "safe absolute value took Err";
    }
  }
  let overflow_result = iabs.checked($MIN_$TYPE);
  match move overflow_result {
    Ok(value: overflow_value) => {
      claim minimum_absolute_value_took_ok: False() because "minimum absolute value took Ok";
    }
    Err(error: overflow_error) => {
    }
  }
  return unit;
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
        let intrinsic = format!("@llvm.abs.i{width}");
        assert!(
            llvm.contains(&format!("{intrinsic}(i{width}")),
            "missing absolute-value intrinsic for {ty}"
        );
        assert!(
            llvm.matches(&format!("{intrinsic}(i{width}")).count() >= 4,
            "each {ty} mode must use the same defined-edge intrinsic"
        );
        assert!(llvm.contains("i1 false"));
        let output = compile_and_run(&llvm);
        assert!(
            output.status.success(),
            "absolute-value program failed for {ty}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn defined_minimum_reports_false_without_executing_absolute_value() {
    let source = br#"fn main() -> own unit traps {
  let is_defined = iabs.defined(-128_i8);
  claim minimum_is_not_defined: bnot(is_defined) because "minimum absolute value must be undefined";
  return unit;
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("icmp ne i8"));
    assert!(!llvm.contains("call i8 @llvm.abs.i8(i8 -128"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}
