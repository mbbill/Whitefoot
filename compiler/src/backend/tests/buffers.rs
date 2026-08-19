use super::*;

#[test]
fn primitive_buffers_cross_functions_update_and_free_once() {
    let source = br#"fn make(n: own u64) -> result: own buffer<u16> allocates(heap), traps {
  let fits = buffer_fits<u16>(n);
  claim allocation_fits: fits because "the requested u16 buffer fits every selected target";
  return buffer_new(n, 3_u16);
}

fn replacement() -> result: own u16 pure {
  return 9_u16;
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let values = make(n: 4_u64);
  let length = len(values);
  let room = ilt(2_u64, length);
  claim sized_by_make: room because "make allocates n slots and main passes four";
  set values[2_u64] = replacement();
  let stored = values[2_u64];
  claim length_drift: ieq(length, 4_u64) because "length drift";
  claim store_drift: ieq(stored, 9_u16) because "store drift";
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // The discharging claim is the retained runtime check; its trap edge
    // precedes the RHS, and the discharged target commits one store.
    let guard = main
        .find("call void @wf_trap")
        .expect("the claim must retain its CLM-1 trap edge");
    let rhs = main
        .find("call i16 @wf_replacement")
        .expect("SET-1 must evaluate its RHS once");
    let store = main
        .find("store i16 %v")
        .expect("SET-1 must commit one element store");
    assert!(guard < rhs && rhs < store);
    assert_eq!(main.matches("call void @free").count(), 1);
    assert!(!emitted_function(&llvm, "make").contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn op9_overflow_traps_before_allocation() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap), traps {
  let values = buffer_new(18446744073709551615_u64, 0_u64);
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let multiply = main
        .find("@llvm.umul.with.overflow.i64")
        .expect("buffer_new must retain checked byte multiplication");
    let overflow = main
        .find("buffer.fill.overflow")
        .expect("overflow must have its OP-9 trap edge");
    let allocation = main
        .find("call ptr @malloc")
        .expect("allocation must remain after the overflow branch");
    assert!(multiply < overflow && overflow < allocation);

    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"OP-9\",\"message\":\"\",\"function\":\"main\",\"node_path\":["
    ));
    assert_eq!(stderr.lines().count(), 1);
}

#[test]
fn target_domain_failure_aborts_before_allocation_without_a_language_record() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap), traps {
  let values = buffer_new(18446744073709551615_u64, 0_u8);
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let multiply = main
        .find("@llvm.umul.with.overflow.i64")
        .expect("OP-9 multiplication must remain first");
    let target_check = main
        .find("buffer.fill.target.check")
        .expect("STOR-6 must retain its target-domain guard");
    let target_failure = main
        .find("buffer.fill.target.failure")
        .expect("the target-domain guard needs a non-continuing edge");
    let allocation = main
        .find("call ptr @malloc")
        .expect("allocation must follow both guards");
    assert!(
        multiply < target_check && target_check < target_failure && target_failure < allocation
    );
    assert!(main[target_failure..allocation].contains("call void @abort()"));
    assert!(!main[target_failure..allocation].contains("@wf_trap"));

    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn an_out_of_bounds_buffer_set_is_an_op4_compile_rejection() {
    // The allocation-length equality proves 2 < 2 underivable, so the
    // program rejects at compile time with the residual [OP-4, ENT-6].
    let source = br#"fn replacement() -> result: own u8 traps {
  claim rhs_evaluated: False() because "RHS evaluated";
  return 9_u8;
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let values = buffer_new(2_u64, 0_u8);
  set values[2_u64] = replacement();
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len(values)"));
}

#[test]
fn empty_buffer_has_zero_length_and_a_normal_free() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap), traps {
  let values = buffer_new(0_u64, 7_u8);
  let length = len(values);
  claim length_drift: ieq(length, 0_u64) because "length drift";
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
    let source = br#"fn cleanup(flag: own Bool) -> result: own unit allocates(heap) {
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

command fn main() -> status: own ExitStatus allocates(heap) {
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
    assert_eq!(main.matches("call void @free").count(), 2);
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

fn update['r](pool: &uniq 'r Pool) -> result: own unit reads('r), writes('r), traps {
  let room = len(deref(pool).left);
  let ok = ilt(1_u64, room);
  claim left_sized: ok because "main pools two slots per column";
  set deref(pool).left[1_u64] = 13_u64;
  set deref(pool).count = 1_u64;
  return unit;
}

fn observe['r](pool: &'r Pool) -> result: own u64 reads('r), traps {
  let room = len(deref(pool).left);
  let ok = ilt(1_u64, room);
  claim left_sized: ok because "main pools two slots per column";
  let value = deref(pool).left[1_u64];
  let count = deref(pool).count;
  claim observed_sum_defined: value +defined count because "the observed fields have a representable sum";
  return value + count;
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let left = buffer_new(2_u64, 0_u64);
  let right = buffer_new(2_u64, 0_u64);
  let pool = Pool(left: move left, right: move right, count: 0_u64);
  let apply = True();
  if apply {
    region 'write {
      update<'write>(pool: &uniq 'write pool);
    }
  }
  region 'read {
    let observed = observe<'read>(pool: &'read pool);
    claim borrowed_struct_update_drift: ieq(observed, 14_u64) because "borrowed struct update drift";
  }
  return exit_status(code: 0_u8);
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
    assert!(build.starts_with("define internal i64 @wf_build(ptr "));
    assert!(build.contains(", i32 "));
    assert!(checksum.starts_with("define internal i64 @wf_checksum(ptr "));
    assert!(checksum.contains(", i64 "));
    assert!(!build.contains("call void @free"));
    assert!(!checksum.contains("call void @free"));
    assert_eq!(main.matches("call void @free").count(), 2);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_wc_chunk_summary_executes() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-wc-chunk-summary-run.wf"
    ));
    let summarize = emitted_function(&llvm, "summarize");
    let combine = emitted_function(&llvm, "combine");
    assert!(summarize.starts_with("define internal i8 @wf_summarize(ptr "));
    assert!(summarize.contains(", { ptr, i64 } "));
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
    assert_eq!(summarize.matches("call void @free").count(), 1);
    assert!(!combine.contains("call void @free"));

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

fn update(columns: own Columns) -> result: own Columns traps {
  let room = len(columns.left);
  let ok = ilt(1_u64, room);
  claim left_sized: ok because "main sizes both columns to two slots";
  set columns.left[1_u64] = replacement();
  return move columns;
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let left = buffer_new(2_u64, 0_u16);
  let right = buffer_new(2_u64, 0_u16);
  let columns = Columns(left: move left, right: move right);
  let updated = update(columns: move columns);
  let updated_room = len(updated.left);
  let updated_ok = ilt(1_u64, updated_room);
  claim updated_sized: updated_ok because "update returns the two-slot columns";
  let value = updated.left[1_u64];
  claim projected_store_drift: ieq(value, 9_u16) because "projected store drift";
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let update = emitted_function(&llvm, "update");
    // The length read projects the field once for the claim; the discharged
    // target projects it once more at the store, with no bounds branch.
    assert_eq!(update.matches("extractvalue %wf.t0").count(), 2);
    let guard = update
        .find("call void @wf_trap")
        .expect("the claim must retain its CLM-1 trap edge");
    let rhs = update
        .find("call i16 @wf_replacement")
        .expect("the RHS must execute once");
    let store = update
        .find("store i16")
        .expect("the target must receive one store");
    assert!(guard < rhs && rhs < store);

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

command fn main() -> status: own ExitStatus allocates(heap) {
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

command fn main() -> status: own ExitStatus allocates(heap) {
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
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap), traps {
  let slots = buffer_vacant<box<u64>>(3_u64);
  let first = box_new(11_u64);
  let wrapped = Some<box<u64>>(value: move first);
  let vacant = replace slots[0_u64] = move wrapped;
  match vacant {
    None() => {
      claim unreachable: True() because "unreachable";
    }
    Some(value: stray) => {
      claim fresh_slot_must_be_vacant: False() because "fresh slot must be vacant";
    }
  }
  let second = box_new(22_u64);
  let wrapped2 = Some<box<u64>>(value: move second);
  let vacant2 = replace slots[2_u64] = move wrapped2;
  match vacant2 {
    None() => {
      claim unreachable_2: True() because "unreachable";
    }
    Some(value: stray2) => {
      claim fresh_slot_must_be_vacant_2: False() because "fresh slot must be vacant";
    }
  }
  let taken = replace slots[0_u64] = None<box<u64>>();
  match taken {
    None() => {
      claim slot_zero_must_be_full: False() because "slot zero must be full";
    }
    Some(value: payload) => {
      let observed = deref(payload);
      claim payload_zero: ieq(observed, 11_u64) because "payload zero";
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // OP-9: the byte size is the target layout of the element aggregate,
    // multiplied with the same overflow check as buffer_new.
    assert!(main.contains("@llvm.umul.with.overflow.i64"));
    assert!(main.contains("ptrtoint (ptr getelementptr (%wf.t"));
    assert!(main.contains("buffer.vacant.overflow"));
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
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let slots = buffer_vacant<u32>(4_u64);
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
fn buffer_vacant_op9_overflow_traps_before_allocation() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap), traps {
  let slots = buffer_vacant<u64>(18446744073709551615_u64);
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let multiply = main
        .find("@llvm.umul.with.overflow.i64")
        .expect("buffer_vacant must retain checked byte multiplication");
    let overflow = main
        .find("buffer.vacant.overflow")
        .expect("overflow must have its OP-9 trap edge");
    let allocation = main
        .find("call ptr @malloc")
        .expect("allocation must remain after the overflow branch");
    assert!(multiply < overflow && overflow < allocation);

    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"OP-9\",\"message\":\"\",\"function\":\"main\",\"node_path\":["
    ));
    assert_eq!(stderr.lines().count(), 1);
}
