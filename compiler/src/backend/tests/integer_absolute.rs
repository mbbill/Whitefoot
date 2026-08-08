use super::{compile, compile_and_run};

#[test]
fn executes_every_absolute_mode_for_every_signed_width() {
    let template = r#"fn main() -> own unit traps {
  let wrapped: own $TYPE = iabs.wrap<$TYPE>($MIN_$TYPE);
  check ieq<$TYPE>(wrapped, $MIN_$TYPE) else trap "wrapped absolute value drift";
  let trapped: own $TYPE = iabs.trap<$TYPE>(-42_$TYPE);
  check ieq<$TYPE>(trapped, 42_$TYPE) else trap "trapping absolute value drift";
  let safe_result: own Result<$TYPE, Overflow> = iabs.checked<$TYPE>(-42_$TYPE);
  match move safe_result {
    Ok(value: safe_value) => {
      check ieq<$TYPE>(safe_value, 42_$TYPE) else trap "checked absolute value drift";
    }
    Err(error: safe_error) => {
      check False() else trap "safe absolute value took Err";
    }
  }
  let overflow_result: own Result<$TYPE, Overflow> = iabs.checked<$TYPE>($MIN_$TYPE);
  match move overflow_result {
    Ok(value: overflow_value) => {
      check False() else trap "minimum absolute value took Ok";
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
fn trapping_minimum_reports_the_mandatory_op2_record() {
    let source = br#"fn main() -> own unit traps {
  let magnitude = iabs.trap(-128_i8);
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"OP-2\",\"message\":\"\",\"function\":\"main\",\"node_path\":["
    ));
    assert!(stderr.ends_with("]}\n"));
    assert_eq!(stderr.lines().count(), 1);
}
