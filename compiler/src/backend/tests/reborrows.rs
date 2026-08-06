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

/// A borrow-mode parameter crosses the call boundary as an address, so the
/// callee's write lands in the caller's storage.
#[test]
fn a_unique_scalar_borrow_parameter_writes_the_callers_storage() {
    let llvm = compile(
        br#"fn bump ['r](n: &uniq 'r i32) -> own unit writes('r) {
  set deref(n) = 42_i32;
  return unit;
}

fn main() -> own unit traps {
  let a: own i32 = 0_i32;
  region 'r {
    bump<'r>(n: &uniq 'r a);
  }
  check ieq<i32>(a, 42_i32) else trap "callee write lost";
  return unit;
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
