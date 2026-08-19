use super::{compile, compile_and_run};

/// [ERR-3] execution through the normal path: `let y = propagate step(...);`
/// binds the Ok payload and continues past the statement, and on `Err(err)`
/// the enclosing function returns `Err(err)` with the same error value.
///
/// The continuation transforms the bound payload (+1), which separates the
/// specified payload binding from a lowering that merely forwards `step`'s
/// whole result: only a run continuation can produce `Ok(42)` from input 41.
#[test]
fn propagate_binds_ok_payloads_and_returns_err_values() {
    let source = br#"enum StepError {
  Negative();
}

fn step(x: own i32) -> result: own Result<i32, StepError> pure {
  if ilt(x, 0_i32) {
    let negative_atom_0001 = Negative();
    return Err<i32, StepError>(error: negative_atom_0001);
  } else {
    return Ok<i32, StepError>(value: x);
  }
}

fn forward(x: own i32) -> result: own Result<i32, StepError> pure {
  doc "Ok binds y and the continuation runs; Err returns Err(err) with the same E [ERR-3].";
  let y = propagate step(x: x);
  let next = y +wrap 1_i32;
  return Ok<i32, StepError>(value: next);
}

command fn main() -> status: own ExitStatus traps {
  let accepted = forward(x: 41_i32);
  match move accepted {
    Ok(value: accepted_value) => {
      claim ok_payload_drift: ieq(accepted_value, 42_i32) because "Ok payload drift";
    }
    Err(error: accepted_error) => {
      claim ok_input_took_the_error_edge: False() because "Ok input took the error edge";
    }
  }
  let rejected = forward(x: -1_i32);
  match move rejected {
    Ok(value: rejected_value) => {
      claim err_input_took_the_normal_edge: False() because "Err input took the normal edge";
    }
    Err(error: rejected_error) => {
      match rejected_error {
        Negative() => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(
        output.status.success(),
        "propagation program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// The conformance case `err3-pos-propagate` reaches accept and runs to
/// exit 0 through the ordinary compiler path [ERR-3]. Its manifest status
/// stays a separate protected change; this pin only guards the compiler
/// behavior behind that promotion.
#[test]
fn err3_pos_propagate_case_compiles_and_runs() {
    let source = include_bytes!("../../../../tests/conformance/cases/err3-pos-propagate.wf");
    let output = compile_and_run(&compile(source));
    assert!(
        output.status.success(),
        "err3-pos-propagate failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
