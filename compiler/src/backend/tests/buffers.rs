use super::*;

#[test]
fn primitive_buffers_cross_functions_update_and_free_once() {
    let source = br#"fn make(n: own u64) -> own buffer<u16> allocates(heap), traps {
  return buffer_new<u16>(n, 3_u16);
}

fn replacement() -> own u16 pure {
  return 9_u16;
}

fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u16> = make(n: 4_u64);
  set index<u16>(values, 2_u64) = replacement();
  let length: own u64 = len<u16>(values);
  let stored: own u16 = index<u16>(values, 2_u64);
  check ieq<u64>(length, 4_u64) else trap "length drift";
  check ieq<u16>(stored, 9_u16) else trap "store drift";
  return unit;
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let guard = main
        .find("buffer.bounds.cont")
        .expect("SET-1 must retain an OP-4 buffer guard");
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
    let source = br#"fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u64> = buffer_new<u64>(18446744073709551615_u64, 0_u64);
  return unit;
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
    let source = br#"fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u8> = buffer_new<u8>(18446744073709551615_u64, 0_u8);
  return unit;
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
fn failing_buffer_set_target_never_evaluates_rhs() {
    let source = br#"fn replacement() -> own u8 traps {
  check False() else trap "RHS evaluated";
  return 9_u8;
}

fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u8> = buffer_new<u8>(2_u64, 0_u8);
  set index<u8>(values, 2_u64) = replacement();
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"OP-4\",\"message\":\"\",\"function\":\"main\",\"node_path\":["
    ));
    assert!(!stderr.contains("RHS evaluated"));
    assert_eq!(stderr.lines().count(), 1);
}

#[test]
fn empty_buffer_has_zero_length_and_a_normal_free() {
    let source = br#"fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u8> = buffer_new<u8>(0_u64, 7_u8);
  let length: own u64 = len<u8>(values);
  check ieq<u64>(length, 0_u64) else trap "length drift";
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn buffer_cleanup_is_explicit_on_return_and_break_edges() {
    let source = br#"fn cleanup(flag: own Bool) -> own unit allocates(heap), traps {
  let values: own buffer<u8> = buffer_new<u8>(2_u64, 0_u8);
  match flag {
    True() => {
      return unit;
    }
    False() => {
    }
  }
  loop @done {
    let scratch: own buffer<u16> = buffer_new<u16>(1_u64, 0_u16);
    break @done;
  }
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let true_value: own Bool = True();
  let false_value: own Bool = False();
  cleanup(flag: true_value);
  cleanup(flag: false_value);
  return unit;
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

fn update ['r](pool: &uniq 'r Pool) -> own unit writes('r), traps {
  set index<u64>(deref(pool).left, 1_u64) = 13_u64;
  set deref(pool).count = 1_u64;
  return unit;
}

fn observe ['r](pool: &'r Pool) -> own u64 reads('r), traps {
  let value: own u64 = index<u64>(deref(pool).left, 1_u64);
  let count: own u64 = deref(pool).count;
  return iadd.trap<u64>(value, count);
}

fn main() -> own unit allocates(heap), traps {
  let left: own buffer<u64> = buffer_new<u64>(2_u64, 0_u64);
  let right: own buffer<u64> = buffer_new<u64>(2_u64, 0_u64);
  let pool: own Pool = Pool(left: move left, right: move right, count: 0_u64);
  let apply: own Bool = True();
  match apply {
    True() => {
      region 'write {
        update<'write>(pool: &uniq 'write pool);
      }
    }
    False() => {
    }
  }
  region 'read {
    let observed: own u64 = observe<'read>(pool: &'read pool);
    check ieq<u64>(observed, 14_u64) else trap "borrowed struct update drift";
  }
  return unit;
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

fn replacement() -> own u16 pure {
  return 9_u16;
}

fn update(columns: own Columns) -> own Columns traps {
  set index<u16>(columns.left, 1_u64) = replacement();
  return move columns;
}

fn main() -> own unit allocates(heap), traps {
  let left: own buffer<u16> = buffer_new<u16>(2_u64, 0_u16);
  let right: own buffer<u16> = buffer_new<u16>(2_u64, 0_u16);
  let columns: own Columns = Columns(left: move left, right: move right);
  let updated: own Columns = update(columns: move columns);
  let value: own u16 = index<u16>(updated.left, 1_u64);
  check ieq<u16>(value, 9_u16) else trap "projected store drift";
  return unit;
}
"#;
    let llvm = compile(source);
    let update = emitted_function(&llvm, "update");
    let projection = update
        .find("extractvalue %wf.t0")
        .expect("the buffer field must be projected once");
    assert_eq!(update.matches("extractvalue %wf.t0").count(), 1);
    let guard = update
        .find("buffer.bounds.cont")
        .expect("the projected target must retain OP-4");
    let rhs = update
        .find("call i16 @wf_replacement")
        .expect("the RHS must execute once");
    let store = update
        .find("store i16")
        .expect("the target must receive one store");
    assert!(projection < guard && guard < rhs && rhs < store);

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

fn main() -> own unit allocates(heap), traps {
  let first: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let second: own buffer<u16> = buffer_new<u16>(1_u64, 0_u16);
  let pair: own Pair = Pair(first: move first, second: move second);
  let prefix: own buffer<u32> = buffer_new<u32>(1_u64, 0_u32);
  let suffix: own buffer<u64> = buffer_new<u64>(1_u64, 0_u64);
  let owner: own Owner = Owner(prefix: move prefix, pair: move pair, suffix: move suffix);
  return unit;
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

fn take(owner: own Owner) -> own buffer<u8> pure {
  return move owner.pair.first;
}

fn main() -> own unit allocates(heap), traps {
  let first: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let second: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let pair: own Pair = Pair(first: move first, second: move second);
  let prefix: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let suffix: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let owner: own Owner = Owner(prefix: move prefix, pair: move pair, suffix: move suffix);
  let retained: own buffer<u8> = take(owner: move owner);
  return unit;
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
