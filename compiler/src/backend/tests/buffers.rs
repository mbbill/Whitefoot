use crate::backend::qualification::{SystemTarget, qualify_program};
use crate::backend::target::{TargetLayout, TargetLayoutFailure, TargetObject, validate_program};

use super::system::with_ir;
use super::*;

const AFFINE_INVARIANT_BOUNDED_ALLOCATION: &[u8] =
    br#"fn allocate(n: own u64, half: own u64) -> result: own unit pure contract {
  requires half <= 500_u64;
} {
  let doubled = half * 2_u64;
  let within = n <= doubled;
  if within {
    invariant tight: n <= 1000_u64;
    let values = buffer_new(n, 0_u16);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

const U64_BUFFER_ALLOCATION: &[u8] = br#"command fn main() -> status: own ExitStatus pure {
  let values = buffer_new(1_u64, 0_u64);
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn affine_invariant_ceiling_controls_the_exact_selected_target_boundary() {
    with_ir(AFFINE_INVARIANT_BOUNDED_ALLOCATION, |program| {
        let host = TargetLayout::host().expect("the backend test runs on a qualified host");
        let system_target = SystemTarget::for_triple(host.triple())
            .expect("the host triple has one qualified system target");
        let qualification = qualify_program(system_target, program)
            .expect("the invariant-bounded allocation fixture must qualify");

        let exact = host.with_runtime_allocation_limits_for_test(2000, 8);
        assert_eq!(validate_program(exact, &qualification, program), Ok(()));

        let one_byte_short = host.with_runtime_allocation_limits_for_test(1999, 8);
        assert_eq!(
            validate_program(one_byte_short, &qualification, program),
            Err(TargetLayoutFailure::Unrepresentable(
                TargetObject::RuntimeSizedAllocation
            ))
        );
    });
}

#[test]
fn buffer_representation_alignment_must_fit_the_selected_allocator_alignment() {
    with_ir(U64_BUFFER_ALLOCATION, |program| {
        let host = TargetLayout::host().expect("the backend test runs on a qualified host");
        let system_target = SystemTarget::for_triple(host.triple())
            .expect("the host triple has one qualified system target");
        let qualification = qualify_program(system_target, program)
            .expect("the buffer-alignment fixture must qualify");

        let exact = host.with_runtime_allocation_limits_for_test(8, 8);
        assert_eq!(validate_program(exact, &qualification, program), Ok(()));

        let one_alignment_step_short = host.with_runtime_allocation_limits_for_test(8, 4);
        assert_eq!(
            validate_program(one_alignment_step_short, &qualification, program),
            Err(TargetLayoutFailure::Unrepresentable(
                TargetObject::RuntimeSizedAllocation
            ))
        );
    });
}

#[test]
fn weigh_invariant_proves_domains_then_erases_before_llvm() {
    let source = br#"fn weigh(weights: &buffer<u8>, count: own u64) -> total: own u32 reads(weights) contract {
  define capacity = len_of(deref(weights));
  requires count <= capacity;
  requires count <= 1000_u64;
  ensures total <= 255000_u32;
} {
  let sum = 0_u32;
  for (
    i in 0_u64..count,
    invariant per_byte: sum <= 255_u32 * i
  ) {
    let w = deref(weights)[i];
    let wide = cvt::<u8, u32>(w);
    set sum = sum + wide;
  }
  return sum;
}

command fn main() -> status: own ExitStatus pure {
  let weights = buffer_new(4_u64, 7_u8);
  let code = 0_u8;
  region {
    let total = weigh(weights: &weights, count: 4_u64);
    if total != 28_u32 {
      set code = 1_u8;
    }
  }
  return exit_status(code: code);
}
"#;
    let llvm = compile(source);
    let weigh = emitted_function(&llvm, "weigh");

    // INV-1 and OP-2 discharge before lowering. The loop therefore contains
    // one plain integer addition and no runtime representation of `per_byte`.
    assert!(weigh.contains("add i32"));
    assert!(!weigh.contains(".with.overflow."));
    assert!(!weigh.contains("call void @wf_trap"));
    assert!(!llvm.contains("per_byte"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn primitive_buffers_cross_functions_update_and_free_once() {
    let source = br#"fn bounded_count(n: own u64) -> result: own u64 pure contract {
  ensures result <= 4611686018427387903_u64;
} {
  if n <= 4611686018427387903_u64 {
    return n;
  } else {
    return 4611686018427387903_u64;
  }
}

fn make(n: own u64) -> result: own buffer<u16> pure {
  let bounded = bounded_count(n: n);
  return buffer_new(bounded, 3_u16);
}

fn replacement() -> result: own u16 pure {
  return 9_u16;
}

command fn main() -> status: own ExitStatus pure {
  let values = make(n: 4_u64);
  let length = len_of(values);
  let stored = 0_u16;
  let code = 0_u8;
  if 2_u64 < length {
    set values[2_u64] = replacement();
    set stored = values[2_u64];
  } else {
    set code = 3_u8;
  }
  if code == 0_u8 {
    if length != 4_u64 {
      set code = 1_u8;
    }
    if stored != 9_u16 {
      set code = 2_u8;
    }
  }
  return exit_status(code: code);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let make = emitted_function(&llvm, "make");
    // The verified scalar normalizer summary discharges allocation, while a
    // local length branch discharges both indexed sites. The RHS is evaluated
    // once before the target commits one store, with no runtime proof fallback.
    let rhs = main
        .find("call i16 @wf_replacement")
        .expect("SET-1 must evaluate its RHS once");
    let store = main
        .find("store i16 %v")
        .expect("SET-1 must commit one element store");
    assert!(rhs < store);
    assert!(!main.contains("call void @wf_trap"));
    assert_eq!(main.matches("call void @free").count(), 1);
    assert!(!make.contains("call void @free"));

    // The proved count ceiling times the u16 stride fits the selected target's
    // byte domain. Target layout therefore admits the dynamic allocation and
    // the emitter needs only the allocator's null-result edge.
    assert!(make.contains("call ptr @malloc"));
    assert!(make.contains("icmp ne ptr"));
    assert!(make.contains("buffer.fill.oom."));
    assert!(make.contains("call void @wf_resource_abort()"));
    for absent in [
        "buffer.fill.target.",
        "@wf_target_domain_abort",
        "@.wf_resource.target_domain",
    ] {
        assert!(
            !llvm.contains(absent),
            "a target-qualified allocation must not emit {absent}:\n{llvm}"
        );
    }

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn buffer_length_qualifies_same_element_reallocation_without_a_target_guard() {
    let source = br#"fn refill(source: own buffer<u8>) -> result: own buffer<u8> reads(source) {
  let length = len_of(source);
  return buffer_new(length, 0_u8);
}

command fn main() -> status: own ExitStatus pure {
  let initial = buffer_new(4_u64, 7_u8);
  let copied = refill(source: move initial);
  let length = len_of(copied);
  if length != 4_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let refill = emitted_function(&llvm, "refill");
    assert!(refill.contains("call ptr @malloc"));
    for absent in [
        "buffer.fill.target.",
        "@wf_target_domain_abort",
        "@.wf_resource.target_domain",
    ] {
        assert!(
            !llvm.contains(absent),
            "a buffer-length target invariant must not emit {absent}:\n{llvm}"
        );
    }

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn op9_overflow_is_rejected_before_lowering() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let values = buffer_new(18446744073709551615_u64, 0_u64);
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-9"));
    assert!(
        failure
            .detail()
            .contains("UndischargedAllocationFitObligation")
    );
}

#[test]
fn an_out_of_bounds_buffer_set_is_an_op4_compile_rejection() {
    // The allocation-length equality proves 2 < 2 underivable, so the
    // program rejects at compile time with the residual [OP-4, ENT-6].
    let source = br#"fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let values = buffer_new(2_u64, 0_u8);
  set values[2_u64] = replacement();
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len_of(values)"));
}

#[test]
fn empty_buffer_has_zero_length_and_a_normal_free() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let values = buffer_new(0_u64, 7_u8);
  let length = len_of(values);
  if length != 0_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn buffer_cleanup_is_explicit_on_return_and_break_edges() {
    let source = br#"fn cleanup(flag: own Bool) -> result: own unit pure {
  let values = buffer_new(2_u64, 0_u8);
  if flag {
    return unit;
  }
  loop @done {
    let scratch = buffer_new(1_u64, 0_u16);
    break @done;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let true_value = True();
  let false_value = False();
  cleanup(flag: true_value);
  cleanup(flag: false_value);
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let cleanup = emitted_function(&llvm, "cleanup");
    assert_eq!(cleanup.matches("call void @free").count(), 3);
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_mutable_buffer_checksum_executes() {
    let output = compile_and_run(&compile(include_bytes!(
        "../../../../tests/conformance/cases/x-buffer-mutable-checksum-run.wf"
    )));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn borrowed_columns_cross_helpers_without_transferring_ownership() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-buffer-borrowed-columns-run.wf"
    ));
    let fill = emitted_function(&llvm, "fill");
    let fold = emitted_function(&llvm, "fold");
    let main = emitted_function(&llvm, "main");
    assert!(fill.contains("store i64"));
    assert!(fold.contains("load i64"));
    assert!(!fill.contains("call void @free"));
    assert!(!fold.contains("call void @free"));
    // Both declared length requirements and the counted-range binder facts are
    // checked before lowering. Neither helper retains a runtime proof check.
    assert_eq!(fill.matches("call void @wf_trap").count(), 0);
    assert_eq!(fold.matches("call void @wf_trap").count(), 0);
    // Each counted loop retains exactly its own continuation comparison. No
    // second comparison remains for either proved buffer bound.
    assert_eq!(fill.matches("icmp ult i64").count(), 1);
    assert_eq!(fold.matches("icmp ult i64").count(), 1);
    // B7c4b-1: the two columns are store runs held in one struct and lent as
    // views, so the requirement branches are the four length checks, the two
    // non-wrap checks the view formations submit, the two store refusals, the
    // checksum branch and the success exit — ten in all — and the releases are
    // the general store's, one per run on each edge that leaves holding them.
    assert!(!main.contains("call void @wf_trap"));
    assert_eq!(main.matches("call i8 @wf.sys.exit_status.v1").count(), 10);
    assert_eq!(main.matches("call void @free").count(), 17);
    assert!(main.contains("call i8 @wf_fill"));
    assert!(main.contains("call i64 @wf_fold"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn borrowed_struct_projection_updates_caller_storage_through_one_address_path() {
    let source = br#"struct Pool {
  left: buffer<u64>;
  right: buffer<u64>;
  count: u64;
}

fn update(pool: &uniq Pool) -> result: own unit reads(pool.left), writes(pool.left, pool.count) {
  let spare = len_of(deref(pool).left);
  let ok = 1_u64 < spare;
  if ok {
    set deref(pool).left[1_u64] = 13_u64;
    set deref(pool).count = 1_u64;
  }
  return unit;
}

fn observe(pool: &Pool) -> result: own u64 reads(pool.left, pool.count) {
  let spare = len_of(deref(pool).left);
  let ok = 1_u64 < spare;
  let count = deref(pool).count;
  if ok {
    let value = deref(pool).left[1_u64];
    return value +wrap count;
  } else {
    return count;
  }
}

command fn main() -> status: own ExitStatus pure {
  let left = buffer_new(2_u64, 0_u64);
  let right = buffer_new(2_u64, 0_u64);
  let pool = Pool(left: move left, right: move right, count: 0_u64);
  let code = 0_u8;
  let apply = True();
  if apply {
    region {
      update(pool: &uniq pool);
    }
  }
  region {
    let observed = observe(pool: &pool);
    if observed != 14_u64 {
      set code = 1_u8;
    }
  }
  return exit_status(code: code);
}
"#;
    let llvm = compile(source);
    let update = emitted_function(&llvm, "update");
    let observe = emitted_function(&llvm, "observe");
    let main = emitted_function(&llvm, "main");
    assert!(update.starts_with("define internal i8 @wf_update(ptr "));
    assert!(observe.starts_with("define internal i64 @wf_observe(ptr "));
    assert!(main.contains("call i8 @wf_update(ptr "));
    assert!(main.contains("call i64 @wf_observe(ptr "));
    assert!(!update.contains("call void @free"));
    assert!(!observe.contains("call void @free"));
    assert_eq!(main.matches("call void @free").count(), 2);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_borrowed_pool_tree_executes() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-borrowed-pool-tree-run.wf"
    ));
    let build = emitted_function(&llvm, "build");
    let checksum = emitted_function(&llvm, "checksum");
    let main = emitted_function(&llvm, "main");
    // B7c4b-1: [BLK-4] refuses a `&uniq` whose referent reaches a run, so the
    // pool is lent as two views and a scalar borrow rather than as one struct;
    // each helper therefore takes two descriptors by value where it took one
    // struct pointer.
    assert!(build.starts_with("define internal %wf.t4 @wf_build({ ptr, i64 } "));
    assert!(build.contains(", i32 "));
    assert!(checksum.starts_with("define internal %wf.t4 @wf_checksum({ ptr, i64 } "));
    assert!(checksum.contains(", i64 "));
    assert!(!build.contains("call void @free"));
    assert!(!checksum.contains("call void @free"));
    // Bounds and arithmetic failures are typed results rather than written proofs,
    // so build and checksum contain no trap edge and main still has its five
    // status exits. B7c4b-1: the two runs come from one bump extent laid out in
    // this activation's frame, so the program reaches the host allocator on no
    // path and there is no free to count.
    assert!(!build.contains("call void @wf_trap"));
    assert!(!checksum.contains("call void @wf_trap"));
    assert!(!main.contains("call void @wf_trap"));
    assert_eq!(main.matches("call i8 @wf.sys.exit_status.v1").count(), 5);
    assert_eq!(main.matches("call void @free").count(), 0);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// The case counts lines, words and bytes over two chunks and combines the
/// two summaries. It was migrated from `buffer<u8>` to `FixedVector<u8, n>`,
/// which makes `summarize` a const generic over its chunk length: the two
/// chunk lengths therefore reach the backend as two monomorphized instances,
/// each taking its own inline run by value instead of one heap descriptor,
/// and the whole program allocates nothing. The assertions are those facts.
#[test]
fn compiler_independent_wc_chunk_summary_executes() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-wc-chunk-summary-run.wf"
    ));
    let four_bytes = emitted_function(&llvm, "summarize$instance$2");
    let one_byte = emitted_function(&llvm, "summarize$instance$3");
    let combine = emitted_function(&llvm, "combine");
    assert!(four_bytes.starts_with("define internal i8 @wf_summarize$instance$2(ptr "));
    assert!(four_bytes.contains(", { [4 x i8], i64, i64 } "));
    assert!(one_byte.starts_with("define internal i8 @wf_summarize$instance$3(ptr "));
    assert!(one_byte.contains(", { [1 x i8], i64, i64 } "));
    assert!(combine.starts_with("define internal i8 @wf_combine(ptr "));
    assert_eq!(
        combine
            .lines()
            .next()
            .expect("combine signature")
            .matches("ptr %v")
            .count(),
        3
    );
    assert!(!llvm.contains("call ptr @malloc"));
    assert!(!llvm.contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn projected_buffer_target_is_formed_once_before_rhs() {
    let source = br#"struct Columns {
  left: buffer<u16>;
  right: buffer<u16>;
}

fn replacement() -> result: own u16 pure {
  return 9_u16;
}

fn update(columns: own Columns) -> result: own Columns reads(columns.left), writes(columns.left) {
  let spare = len_of(columns.left);
  let ok = 1_u64 < spare;
  if ok {
    set columns.left[1_u64] = replacement();
  }
  return move columns;
}

command fn main() -> status: own ExitStatus pure {
  let left = buffer_new(2_u64, 0_u16);
  let right = buffer_new(2_u64, 0_u16);
  let columns = Columns(left: move left, right: move right);
  let updated = update(columns: move columns);
  let updated_room = len_of(updated.left);
  let updated_ok = 1_u64 < updated_room;
  if updated_ok {
    let value = updated.left[1_u64];
    if value != 9_u16 {
      return exit_status(code: 1_u8);
    }
  } else {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let update = emitted_function(&llvm, "update");
    // The length read projects the field once for the explicit control; the
    // target projects it once more at the store, with no language trap.
    assert_eq!(update.matches("extractvalue %wf.t0").count(), 2);
    let guard = update
        .find("icmp ult i64")
        .expect("the explicit control must test the projected buffer length");
    let rhs = update
        .find("call i16 @wf_replacement")
        .expect("the RHS must execute once");
    let store = update
        .find("store i16")
        .expect("the target must receive one store");
    assert!(guard < rhs && rhs < store);
    assert!(!update.contains("call void @wf_trap"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn nested_struct_cleanup_frees_buffer_fields_in_reverse_order() {
    let source = br#"struct Pair {
  first: buffer<u8>;
  second: buffer<u16>;
}

struct Owner {
  prefix: buffer<u32>;
  pair: Pair;
  suffix: buffer<u64>;
}

command fn main() -> status: own ExitStatus pure {
  let first = buffer_new(1_u64, 0_u8);
  let second = buffer_new(1_u64, 0_u16);
  let pair = Pair(first: move first, second: move second);
  let prefix = buffer_new(1_u64, 0_u32);
  let suffix = buffer_new(1_u64, 0_u64);
  let owner = Owner(prefix: move prefix, pair: move pair, suffix: move suffix);
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    assert_eq!(main.matches("call void @free").count(), 4);
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn projected_buffer_move_frees_only_residual_siblings() {
    let source = br#"struct Pair {
  first: buffer<u8>;
  second: buffer<u8>;
}

struct Owner {
  prefix: buffer<u8>;
  pair: Pair;
  suffix: buffer<u8>;
}

fn take(owner: own Owner) -> result: own buffer<u8> pure {
  return move owner.pair.first;
}

command fn main() -> status: own ExitStatus pure {
  let first = buffer_new(1_u64, 0_u8);
  let second = buffer_new(1_u64, 0_u8);
  let pair = Pair(first: move first, second: move second);
  let prefix = buffer_new(1_u64, 0_u8);
  let suffix = buffer_new(1_u64, 0_u8);
  let owner = Owner(prefix: move prefix, pair: move pair, suffix: move suffix);
  let retained = take(owner: move owner);
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let take = emitted_function(&llvm, "take");
    assert_eq!(take.matches("call void @free").count(), 3);
    assert_eq!(
        emitted_function(&llvm, "main")
            .matches("call void @free")
            .count(),
        1
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_struct_of_buffers_checksum_executes() {
    let output = compile_and_run(&compile(include_bytes!(
        "../../../../tests/conformance/cases/x-struct-of-buffers-checksum-run.wf"
    )));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn affine_element_buffers_construct_replace_vacate_and_drop_per_element() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let slots = buffer_vacant::<box<u64>>(3_u64);
  let first = box_new(11_u64);
  let wrapped = Some<box<u64>>(value: move first);
  let vacant = replace slots[0_u64] = move wrapped;
  match vacant {
    None() => {
    }
    Some(value: stray) => {
      return exit_status(code: 1_u8);
    }
  }
  let second = box_new(22_u64);
  let wrapped2 = Some<box<u64>>(value: move second);
  let vacant2 = replace slots[2_u64] = move wrapped2;
  match vacant2 {
    None() => {
    }
    Some(value: stray2) => {
      return exit_status(code: 2_u8);
    }
  }
  let taken = replace slots[0_u64] = None<box<u64>>();
  match taken {
    None() => {
      return exit_status(code: 3_u8);
    }
    Some(value: payload) => {
      let observed = deref(payload);
      if observed != 11_u64 {
        return exit_status(code: 4_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // OP-9 was discharged statically. Target qualification supplies the
    // selected layout, so no language overflow guard remains.
    assert!(!main.contains("@llvm.umul.with.overflow.i64"));
    assert!(main.contains("ptrtoint (ptr getelementptr (%wf.t"));
    assert!(!main.contains("buffer.vacant.overflow"));
    // Every element starts as the tag-zero None(): the aggregate
    // zeroinitializer stored through the init loop.
    let body = main
        .find("buffer.vacant.body")
        .expect("the all-None init loop must be emitted");
    assert!(main[body..].contains("zeroinitializer"));
    // The SET-2 element commit is one aggregate load and one aggregate
    // store through the same element address arithmetic.
    assert!(main.contains("load %wf.t"));
    assert!(main.contains("store %wf.t"));
    // The scope-exit drop is the per-element loop [STOR-3]: the buffer
    // helper drops each element through the enum helper, then frees.
    let helper_start = llvm
        .find("define private void @wf.drop.buffer.t")
        .expect("an element type with a drop derives the buffer drop loop");
    let helper_end = llvm[helper_start..]
        .find("\n}\n")
        .map(|offset| helper_start + offset)
        .expect("buffer drop helper must be complete");
    let helper = &llvm[helper_start..helper_end];
    assert!(helper.contains("call void @wf.drop.t"));
    assert_eq!(helper.matches("call void @free").count(), 1);
    assert!(main.contains("call void @wf.drop.buffer.t"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn trivially_droppable_affine_elements_keep_the_single_free() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let slots = buffer_vacant::<u32>(4_u64);
  let filled = Some<u32>(value: 7_u32);
  let vacant = replace slots[2_u64] = move filled;
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    // An element type whose own drop derives no action keeps the composite
    // action exactly the heap free [STOR-3]: no drop loop is generated.
    assert!(!llvm.contains("@wf.drop.buffer"));
    let main = emitted_function(&llvm, "main");
    assert_eq!(main.matches("call void @free").count(), 1);
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn buffer_vacant_op9_overflow_is_rejected_before_lowering() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let slots = buffer_vacant::<u64>(18446744073709551615_u64);
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-9"));
    assert!(
        failure
            .detail()
            .contains("UndischargedAllocationFitObligation")
    );
}
