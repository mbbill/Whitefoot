use super::{compile, compile_and_run};

#[test]
fn concrete_option_and_result_instances_share_the_nominal_backend() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/x-enum-twostate-result-payload.wf");
    let output = compile_and_run(&compile(source));
    assert!(
        output.status.success(),
        "Option and Result program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// A generic `Result` instance whose Ok payload is an owned struct is
/// represented without word erasure [PRE-1, TYPE-2, TYPE-5]: both fields of
/// the aggregate payload survive construction, return, and match binding,
/// and the Err variant of the same instance still carries its scalar value.
#[test]
fn generic_result_instances_carry_owned_aggregate_payloads() {
    let source = br#"struct Extent {
  offset: u64;
  width: u64;
}

fn locate(offset: own u64, width: own u64) -> own Result<Extent, u64> pure {
  let extent = Extent(offset: offset, width: width);
  return Ok<Extent, u64>(value: move extent);
}

fn missing(code: own u64) -> own Result<Extent, u64> pure {
  return Err<Extent, u64>(error: code);
}

fn main() -> own unit traps {
  let found = locate(offset: 3_u64, width: 4_u64);
  match move found {
    Ok(value: found_extent) => {
      claim offset_drift: ieq(found_extent.offset, 3_u64) because "offset drift";
      claim width_drift: ieq(found_extent.width, 4_u64) because "width drift";
    }
    Err(error: found_code) => {
      claim locate_took_err: False() because "locate took Err";
    }
  }
  let absent = missing(code: 9_u64);
  match move absent {
    Ok(value: absent_extent) => {
      claim missing_took_ok: False() because "missing took Ok";
    }
    Err(error: absent_code) => {
      claim error_payload_drift: ieq(absent_code, 9_u64) because "error payload drift";
    }
  }
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(
        output.status.success(),
        "aggregate Result payload program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_byte_scanner_returns_option_offsets() {
    let source = include_bytes!("../../../../tests/conformance/cases/x-option-byte-scanner-run.wf");
    let output = compile_and_run(&compile(source));
    assert!(
        output.status.success(),
        "Option byte scanner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
