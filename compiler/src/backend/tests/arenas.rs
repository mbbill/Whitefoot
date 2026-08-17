use super::*;

/// [STOR-2, STOR-3, STOR-4] the stor4-pos-arena-confined conformance case:
/// an `arena<'r, i32>` stays within its region, its content reads through
/// `deref`, and its storage is released at the block exit. Runs to exit 0.
#[test]
fn a_confined_arena_allocation_reads_and_releases_with_its_region() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/stor4-pos-arena-confined.wf"
    ));
    assert!(
        llvm.contains("@wf_arena_release"),
        "the region exit must carry the storage release"
    );
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
        br#"fn early() -> own i32 traps {
  region 'e {
    let a = arena_new<'e, i32>(5_i32);
    let v = deref(a);
    check ieq(v, 5_i32) else trap "early read";
    return v;
  }
}

fn main() -> own unit traps {
  for @turns i in 0_u64..200_u64 {
    region 'r {
      let a = arena_new<'r, i32>(1_i32);
      let b = arena_new<'r, i32>(2_i32);
      let first = deref(a);
      let second = deref(b);
      check ieq(first, 1_i32) else trap "loop first";
      check ieq(second, 2_i32) else trap "loop second";
    }
  }
  let got = early();
  check ieq(got, 5_i32) else trap "early return";
  region 'outer {
    let base = arena_new<'outer, i32>(7_i32);
    region 'inner {
      let extra = arena_new<'inner, i32>(30_i32);
      let left = deref(base);
      let right = deref(extra);
      check ieq(left, 7_i32) else trap "nested outer read";
      check ieq(right, 30_i32) else trap "nested inner read";
    }
    let after = deref(base);
    check ieq(after, 7_i32) else trap "outer survives inner exit";
  }
  return unit;
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
        br#"fn main() -> own unit traps {
  for @turns i in 0_u64..4_u64 {
    region 'r {
      let a = arena_new<'r, i32>(9_i32);
      let v = deref(a);
      check ieq(v, 9_i32) else trap "break read";
      break @turns;
    }
  }
  region 'outer {
    region 'inner {
      let a = arena_new<'outer, i32>(3_i32);
      let b = arena_new<'inner, i32>(4_i32);
      let x = deref(a);
      let y = deref(b);
      check ieq(x, 3_i32) else trap "enclosing alloc";
      check ieq(y, 4_i32) else trap "inner alloc";
    }
  }
  return unit;
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
        br#"fn main() -> own unit traps {
  let flag = True();
  region 'r {
    let picked = if flag {
      let a = arena_new<'r, i32>(11_i32);
      give move a;
    } else {
      let b = arena_new<'r, i32>(22_i32);
      give move b;
    }
    let v = deref(picked);
    check ieq(v, 11_i32) else trap "delivered read";
  }
  return unit;
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
