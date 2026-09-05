use crate::backend::qualification::{SystemTarget, qualify_program};
use crate::backend::target::{TargetLayout, TargetLayoutFailure, TargetObject, validate_program};

use super::system::with_ir;
use super::*;

const BYTE_ARENA_NODE: &[u8] = br#"command fn main() -> status: own ExitStatus pure {
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
        let host = TargetLayout::host().expect("the backend test runs on a qualified host");
        let system_target = SystemTarget::for_triple(host.triple())
            .expect("the host triple has one qualified system target");
        let qualification =
            qualify_program(system_target, program).expect("the arena fixture must qualify");

        let exact = host.with_runtime_allocation_limits_for_test(16, 8);
        assert_eq!(validate_program(exact, &qualification, program), Ok(()));

        let one_byte_short = host.with_runtime_allocation_limits_for_test(15, 8);
        assert_eq!(
            validate_program(one_byte_short, &qualification, program),
            Err(TargetLayoutFailure::Unrepresentable(
                TargetObject::RuntimeSizedAllocation
            ))
        );

        let under_aligned = host.with_runtime_allocation_limits_for_test(16, 4);
        assert_eq!(
            validate_program(under_aligned, &qualification, program),
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
    assert!(
        main.contains("%wf.frame = alloca { [8 x i8], { ptr, i64 } }, align 8"),
        "the reservation must lay the extent out in the reserving frame"
    );
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

/// The region allocation list is correct across the block shapes [STOR-3]
/// fixes releases for: a region re-entered by a counted loop resets and
/// releases per entry, a `return` from inside a region releases on that
/// edge, and an inner region's exit releases only its own allocations while
/// the enclosing region's content stays readable.
#[test]
fn arena_release_covers_loop_reentry_early_return_and_nested_regions() {
    let llvm = compile(
        br#"fn early() -> result: own i32 pure {
  region 'e {
    let a = arena_new::<'e, i32>(5_i32);
    let v = deref(a);
    return v;
  }
}

command fn main() -> status: own ExitStatus pure {
  for @turns (i in 0_u64..200_u64) {
    region 'r {
      let a = arena_new::<'r, i32>(1_i32);
      let b = arena_new::<'r, i32>(2_i32);
      let first = deref(a);
      let second = deref(b);
      if first != 1_i32 {
        return exit_status(code: 1_u8);
      }
      if second != 2_i32 {
        return exit_status(code: 2_u8);
      }
    }
  }
  let got = early();
  if got != 5_i32 {
    return exit_status(code: 3_u8);
  }
  region 'outer {
    let base = arena_new::<'outer, i32>(7_i32);
    region 'inner {
      let extra = arena_new::<'inner, i32>(30_i32);
      let left = deref(base);
      let right = deref(extra);
      if left != 7_i32 {
        return exit_status(code: 4_u8);
      }
      if right != 30_i32 {
        return exit_status(code: 5_u8);
      }
    }
    let after = deref(base);
    if after != 7_i32 {
      return exit_status(code: 6_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// The two remaining region-exit shapes [STOR-3] fixes releases for: a
/// `break` leaving the region block from inside a loop, and an allocation
/// naming an enclosing region from within an inner block. The second is the
/// reason `arena_new` selects its list by resolved region rather than by the
/// nearest enclosing block: the outer allocation must survive the inner
/// region's exit and be freed only by the outer one.
#[test]
fn arena_release_covers_break_edges_and_enclosing_region_allocation() {
    let llvm = compile(
        br#"command fn main() -> status: own ExitStatus pure {
  for @turns (i in 0_u64..4_u64) {
    region 'r {
      let a = arena_new::<'r, i32>(9_i32);
      let v = deref(a);
      if v != 9_i32 {
        return exit_status(code: 1_u8);
      }
      break @turns;
    }
  }
  region 'outer {
    region 'inner {
      let a = arena_new::<'outer, i32>(3_i32);
      let b = arena_new::<'inner, i32>(4_i32);
      let x = deref(a);
      let y = deref(b);
      if x != 3_i32 {
        return exit_status(code: 2_u8);
      }
      if y != 4_i32 {
        return exit_status(code: 3_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// [GIVE-1, STOR-4] a value delivery whose destination stays inside the
/// arena's region block is legal and executes: the delivered arena value is
/// one pointer, and the region's own exit still releases the storage.
#[test]
fn a_within_region_arena_delivery_executes() {
    let llvm = compile(
        br#"command fn main() -> status: own ExitStatus pure {
  let flag = True();
  region 'r {
    let picked = if flag {
      let a = arena_new::<'r, i32>(11_i32);
      give move a;
    } else {
      let b = arena_new::<'r, i32>(22_i32);
      give move b;
    }
    let v = deref(picked);
    if v != 11_i32 {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
