use super::{compile, compile_and_run};

#[test]
fn executes_every_negation_mode_for_every_signed_width() {
    let template = r#"fn main() -> own unit traps {
  let wrapped = ineg.wrap($MIN_$TYPE);
  claim wrapped_negation_drift: ieq(wrapped, $MIN_$TYPE) because "wrapped negation drift";
  let exact = ineg(-42_$TYPE);
  claim exact_negation_drift: ieq(exact, 42_$TYPE) because "exact negation drift";
  let ordinary_defined = ineg.defined(-42_$TYPE);
  claim ordinary_negation_is_defined: ordinary_defined because "ordinary negation must be defined";
  let minimum_defined = ineg.defined($MIN_$TYPE);
  claim minimum_negation_is_undefined: bnot(minimum_defined) because "minimum negation must be undefined";
  let safe_result = ineg.checked(-42_$TYPE);
  match move safe_result {
    Ok(value: safe_value) => {
      claim checked_negation_drift: ieq(safe_value, 42_$TYPE) because "checked negation drift";
    }
    Err(error: safe_error) => {
      claim safe_negation_took_err: False() because "safe negation took Err";
    }
  }
  let overflow_result = ineg.checked($MIN_$TYPE);
  match move overflow_result {
    Ok(value: overflow_value) => {
      claim minimum_negation_took_ok: False() because "minimum negation took Ok";
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
    let source = br#"fn main() -> own unit traps {
  let is_defined = ineg.defined(-128_i8);
  claim minimum_is_not_defined: bnot(is_defined) because "minimum negation must be undefined";
  return unit;
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("icmp ne i8"));
    assert!(!llvm.contains("sub i8 0, -128"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}
