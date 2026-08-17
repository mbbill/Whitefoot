use super::*;

#[test]
fn statement_scoped_child_reborrows_resume_their_parent() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-child-reborrow-run.wf"
    ));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// Borrows of directly stored scalar and enum content are the address of that
/// storage, so a write through `&uniq` is visible to the owner and a read
/// through `&'r` reloads it [OWN-2, OWN-5, TYPE-7].
#[test]
fn general_scalar_and_enum_borrows_execute_through_host_llvm() {
    for source in [
        include_bytes!("../../../../tests/conformance/cases/own2-pos-three-modes.wf").as_slice(),
        include_bytes!("../../../../tests/conformance/cases/own5-pos-read-through-holder.wf")
            .as_slice(),
        include_bytes!("../../../../tests/conformance/cases/own7-pos-distinct-noverlap.wf")
            .as_slice(),
        include_bytes!("../../../../tests/conformance/cases/own11-pos-loop-inner-region.wf")
            .as_slice(),
        include_bytes!("../../../../tests/conformance/cases/x-typ-uniq-deref-write-roundtrip.wf")
            .as_slice(),
        include_bytes!("../../../../tests/conformance/cases/x-enum-borrow-payload-live.wf")
            .as_slice(),
        include_bytes!(
            "../../../../tests/conformance/cases/x-integ-coin-borrow-match-score-twice.wf"
        )
        .as_slice(),
    ] {
        let llvm = compile(source);
        let output = compile_and_run(&llvm);
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

/// A borrow-returning call is typed by the callee's declared result mode: the
/// call site receives an address, and a discarded borrow result is evaluated
/// without any drop of the referent it does not own [OWN-2, TYPE-7, STOR-3].
///
/// Regression: the call definition used the referent value type and the
/// borrow-typed callee result made the module invalid IR, an internal error
/// on accepted source.
#[test]
fn a_discarded_borrow_returning_call_compiles_and_runs() {
    let llvm = compile(
        br#"fn source['r](x: &'r i32) -> &'r i32 pure {
  return x;
}

fn main() -> own unit traps {
  let v = 5_i32;
  region 'a {
    let h = &'a v;
    source<'a>(x: h);
  }
  check ieq(v, 5_i32) else trap "owner value changed";
  return unit;
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// A borrow-mode parameter crosses the call boundary as an address, so the
/// callee's write lands in the caller's storage.
#[test]
fn a_unique_scalar_borrow_parameter_writes_the_callers_storage() {
    let llvm = compile(
        br#"fn bump['r](n: &uniq 'r i32) -> own unit writes('r) {
  set deref(n) = 42_i32;
  return unit;
}

fn main() -> own unit traps {
  let a = 0_i32;
  region 'r {
    bump<'r>(n: &uniq 'r a);
  }
  check ieq(a, 42_i32) else trap "callee write lost";
  return unit;
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
