use super::{compile, compile_and_run};

#[test]
fn guards_every_integer_error_before_llvm() {
    let template = r#"fn main() -> own unit traps {
  let quotient = 84_$TYPE /checked 2_$TYPE;
  match move quotient {
    Ok(value: quotient_value) => {
      claim quotient_drift: ieq(quotient_value, 42_$TYPE) because "quotient drift";
    }
    Err(error: quotient_error) => {
      claim quotient_took_err: False() because "quotient took Err";
    }
  }
  let remainder = 85_$TYPE %checked 43_$TYPE;
  match move remainder {
    Ok(value: remainder_value) => {
      claim remainder_drift: ieq(remainder_value, 42_$TYPE) because "remainder drift";
    }
    Err(error: remainder_error) => {
      claim remainder_took_err: False() because "remainder took Err";
    }
  }
  let divide_zero = 42_$TYPE /checked 0_$TYPE;
  match move divide_zero {
    Ok(value: divide_zero_value) => {
      claim zero_division_took_ok: False() because "zero division took Ok";
    }
    Err(error: divide_zero_error) => {
      match divide_zero_error {
        DivideByZero() => {
        }
        DivOverflow() => {
          claim zero_division_became_overflow: False() because "zero division became overflow";
        }
      }
    }
  }
  let remainder_zero = 42_$TYPE %checked 0_$TYPE;
  match move remainder_zero {
    Ok(value: remainder_zero_value) => {
      claim zero_remainder_took_ok: False() because "zero remainder took Ok";
    }
    Err(error: remainder_zero_error) => {
      match remainder_zero_error {
        DivideByZero() => {
        }
        DivOverflow() => {
          claim zero_remainder_became_overflow: False() because "zero remainder became overflow";
        }
      }
    }
  }
$SIGNED_CASES  return unit;
}
"#;
    for (ty, width, signed, minimum) in [
        ("i8", 8, true, "-128"),
        ("i16", 16, true, "-32768"),
        ("i32", 32, true, "-2147483648"),
        ("i64", 64, true, "-9223372036854775808"),
        ("u8", 8, false, ""),
        ("u16", 16, false, ""),
        ("u32", 32, false, ""),
        ("u64", 64, false, ""),
    ] {
        let signed_cases = if signed {
            format!(
                r#"  let divide_overflow = {minimum}_{ty} /checked -1_{ty};
  match move divide_overflow {{
    Ok(value: divide_overflow_value) => {{
      claim division_overflow_took_ok: False() because "division overflow took Ok";
    }}
    Err(error: divide_overflow_error) => {{
      match divide_overflow_error {{
        DivideByZero() => {{
          claim division_overflow_became_zero: False() because "division overflow became zero";
        }}
        DivOverflow() => {{
        }}
      }}
    }}
  }}
  let remainder_overflow = {minimum}_{ty} %checked -1_{ty};
  match move remainder_overflow {{
    Ok(value: remainder_overflow_value) => {{
      claim remainder_overflow_took_ok: False() because "remainder overflow took Ok";
    }}
    Err(error: remainder_overflow_error) => {{
      match remainder_overflow_error {{
        DivideByZero() => {{
          claim remainder_overflow_became_zero: False() because "remainder overflow became zero";
        }}
        DivOverflow() => {{
        }}
      }}
    }}
  }}
"#
            )
        } else {
            String::new()
        };
        let source = template
            .replace("$TYPE", ty)
            .replace("$SIGNED_CASES", &signed_cases);
        let llvm = compile(source.as_bytes());
        let division = if signed { "sdiv" } else { "udiv" };
        let remainder = if signed { "srem" } else { "urem" };
        for opcode in [division, remainder] {
            let instruction = format!(" = {opcode} i{width} ");
            let operation = llvm
                .find(&instruction)
                .unwrap_or_else(|| panic!("missing {opcode} lowering for {ty}"));
            let guarded_prefix = &llvm[..operation];
            let safe = guarded_prefix
                .rfind("integer.safe.")
                .unwrap_or_else(|| panic!("{opcode} for {ty} is not in a safe block"));
            let branch = guarded_prefix[..safe]
                .rfind("br i1")
                .unwrap_or_else(|| panic!("{opcode} for {ty} has no preceding error branch"));
            assert!(branch < safe && safe < operation);
        }
        let output = compile_and_run(&llvm);
        assert!(
            output.status.success(),
            "checked division program failed for {ty}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}
