use crate::backend::target::{TargetLayout, TargetLayoutFailure, TargetObject, validate_program};

use super::system::with_ir;
use super::*;

const BYTE_ARENA_NODE: &[u8] = br#"fn main() -> status: own ExitStatus pure {
  region 'r {
    let item = arena_new::<'r, u8>(7_u8);
    let value = deref(item);
    if value != 7_u8 {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

/// `{ ptr, i8 }` occupies sixteen bytes on every selected 64-bit target:
/// eight bytes for the list link, one byte for content, and seven bytes of
/// tail padding required by the node's eight-byte alignment. The exact
/// boundary is admitted, while either one fewer addressable byte or an
/// allocator alignment below the node alignment is rejected before emission.
#[test]
fn selected_target_validates_the_complete_padded_arena_node() {
    with_ir(BYTE_ARENA_NODE, |program| {
        let host = TargetLayout::host().expect("the backend test runs on a supported host layout");

        let exact = host.with_runtime_allocation_limits_for_test(16, 8);
        assert_eq!(validate_program(exact, program), Ok(()));

        let one_byte_short = host.with_runtime_allocation_limits_for_test(15, 8);
        assert_eq!(
            validate_program(one_byte_short, program),
            Err(TargetLayoutFailure::Unrepresentable(
                TargetObject::RuntimeSizedAllocation
            ))
        );

        let under_aligned = host.with_runtime_allocation_limits_for_test(16, 4);
        assert_eq!(
            validate_program(under_aligned, program),
            Err(TargetLayoutFailure::Unrepresentable(
                TargetObject::RuntimeSizedAllocation
            ))
        );
    });
}

/// The emitted allocation and content address use the same complete LLVM
/// structure whose selected-target layout was validated above. Linking the
/// module through clang verifies that the size expression and GEP are valid
/// LLVM, and execution verifies the content offset.
#[test]
fn arena_node_emission_uses_the_validated_complete_llvm_type() {
    let llvm = compile(BYTE_ARENA_NODE);
    assert!(llvm.contains(
        "call ptr @malloc(i64 ptrtoint (ptr getelementptr ({ ptr, i8 }, ptr null, i64 1) to i64))"
    ));
    assert!(llvm.contains("getelementptr inbounds { ptr, i8 }, ptr"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// [STOR-2, STOR-3, STOR-4, BLK-2] the stor4-pos-arena-confined conformance
/// case: a bump extent stays within its region `'r`, a cell taken at it
/// reads through `deref`, and nothing of it outlives the block. Runs to
/// exit 0.
///
/// The case was migrated from `arena_new::<'r, i32>` to the
/// `arena_frame`/`arena_box` pair, and that changes what the region exit
/// carries rather than merely how it is spelled: [BLK-2] lays a reservation
/// out in the reserving activation's own frame, so the extent contributes no
/// row to [STOR-3]'s release table and the exit has no release work at all.
/// The assertion is therefore the reservation itself — one frame slot for the
/// extent's bytes — together with the absence of any acquisition or release
/// call. `arena_release_covers_loop_reentry_early_return_and_nested_regions`
/// below still covers the released store-resident form.
#[test]
fn a_confined_arena_allocation_reads_and_releases_with_its_region() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/stor4-pos-arena-confined.wf"
    ));
    let main = emitted_function(&llvm, "main");
    let frame = main
        .lines()
        .find(|line| line.contains("%wf.frame = alloca "))
        .expect("the activation must reserve its physical frame");
    assert!(
        frame.contains("[8 x i8]") && frame.ends_with(", align 8"),
        "the reservation must lay the extent out in the reserving frame"
    );
    assert_eq!(main.matches(" = alloca ").count(), 1);
    let provider = main
        .lines()
        .find(|line| line.contains("insertvalue { ptr, i64 } zeroinitializer, ptr "))
        .expect("the provider must retain the reservation address");
    let extent = provider
        .split_once("zeroinitializer, ptr ")
        .expect("provider address operand")
        .1
        .strip_suffix(", 0")
        .expect("provider base field");
    assert!(main.lines().any(|line| {
        line.trim_start()
            .starts_with(&format!("{extent} = getelementptr inbounds "))
            && line.contains("ptr %wf.frame,")
    }));
    assert!(
        !llvm.contains("@wf_arena_release"),
        "a frame-resident extent has no release action of its own"
    );
    assert!(!main.contains("call ptr @malloc"));
    assert!(!main.contains("call void @free"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
