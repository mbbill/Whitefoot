use super::{compile, compile_and_run, emitted_function};

/// Counts the stack slots one emitted function declares, and how many of those
/// declarations sit outside its entry block.
///
/// A slot declared anywhere else is reached once per execution of its block, so
/// a slot inside a loop grows the frame once per iteration. The emitter must
/// therefore keep every declaration in the entry block, which runs exactly once
/// per call, and leave only the store at the use site.
fn slot_declarations(function: &str) -> (usize, usize) {
    let mut in_entry = false;
    let mut total = 0;
    let mut outside_entry = 0;
    for line in function.lines() {
        if let Some(label) = line.strip_suffix(':')
            && !label.starts_with(' ')
        {
            in_entry = label == "entry";
            continue;
        }
        if line.contains(" = alloca ") {
            total += 1;
            if !in_entry {
                outside_entry += 1;
            }
        }
    }
    (total, outside_entry)
}

#[test]
fn const_arrays_are_immutable_globals_and_execute_through_index_and_len() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/const2-pos-array-lookup.wf"
    ));
    assert!(llvm.contains(
        "@.wf_const.0 = private unnamed_addr constant [4 x i8] [i8 10, i8 20, i8 30, i8 40]"
    ));
    let main = emitted_function(&llvm, "main");
    assert!(main.contains("icmp ult i64"));
    assert!(main.contains("call void @wf_trap"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn filled_arrays_cross_function_boundaries_and_keep_a_checked_read() {
    let source = br#"fn make() -> own array<u16, 4> pure {
  return array_new<u16, 4>(42_u16);
}

fn read(values: own array<u16, 4>, offset: own u64) -> own u16 traps {
  let value: own u16 = values[offset];
  return value;
}

fn main() -> own unit traps {
  let values: own array<u16, 4> = make();
  let length: own u64 = len<u16>(values);
  check ieq<u64>(length, 4_u64) else trap "length drift";
  let value: own u16 = read(values: move values, offset: 3_u64);
  check ieq<u16>(value, 42_u16) else trap "fill drift";
  return unit;
}
"#;
    let llvm = compile(source);
    let read = emitted_function(&llvm, "read");
    let bounds = read
        .find("icmp ult i64")
        .expect("array read must compare its offset with the fixed length");
    let trap = read[bounds..]
        .find("call void @wf_trap")
        .map(|offset| bounds + offset)
        .expect("array read must retain its OP-4 trap edge");
    let load = read[trap..]
        .find("getelementptr inbounds [4 x i16]")
        .map(|offset| trap + offset)
        .expect("array read must address the element only on the safe edge");
    assert!(bounds < trap && trap < load);
    assert!(llvm.contains("array.fill.head"));
    assert!(llvm.contains("array.fill.done"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn out_of_bounds_array_read_reports_op4_before_abort() {
    let source = br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(7_u8);
  let value: own u8 = values[2_u64];
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"OP-4\",\"message\":\"\",\"function\":\"main\",\"node_path\":["
    ));
    assert!(stderr.ends_with("]}\n"));
    assert_eq!(stderr.lines().count(), 1);
}

#[test]
fn compiler_independent_array_checksum_executes() {
    let output = compile_and_run(&compile(include_bytes!(
        "../../../../tests/conformance/cases/x-array-const-checksum-run.wf"
    )));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn indexed_set_checks_before_rhs_and_updates_the_array() {
    let source = br#"fn replacement() -> own u8 traps {
  check True() else trap "replacement drift";
  return 9_u8;
}

fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  set values[1_u64] = replacement();
  let stored: own u8 = values[1_u64];
  check ieq<u8>(stored, 9_u8) else trap "set drift";
  return unit;
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let bounds = main
        .find("icmp ult i64")
        .expect("indexed target must retain its bounds check");
    let target_trap = main[bounds..]
        .find("call void @wf_trap")
        .map(|offset| bounds + offset)
        .expect("indexed target must retain its trap edge");
    let rhs = main[target_trap..]
        .find("call i8 @wf_replacement")
        .map(|offset| target_trap + offset)
        .expect("RHS must be emitted after target evaluation");
    let store = main[rhs..]
        .find("store i8")
        .map(|offset| rhs + offset)
        .expect("array update must store only after the RHS");
    assert!(bounds < target_trap && target_trap < rhs && rhs < store);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn failing_indexed_set_target_never_evaluates_rhs() {
    let source = br#"fn replacement() -> own u8 traps {
  check False() else trap "RHS evaluated";
  return 9_u8;
}

fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  set values[2_u64] = replacement();
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
fn a_long_loop_over_a_dynamically_indexed_array_keeps_the_frame_bounded() {
    // Both the read and the indexed set need a stack slot for the array value,
    // and the index is not a compile-time constant, so neither slot can be
    // promoted away. The trip count is far past the point where one slot per
    // iteration would exhaust the process stack: 200000 iterations of two
    // 64-byte slots is about 25 MB, against a default 8 MB limit.
    let source = br#"fn main() -> own unit traps {
  doc "A long loop reads and writes one fixed array through a rotating index.";
  let window: own array<u64, 8> = array_new<u64, 8>(1_u64);
  let step: own u64 = 0_u64;
  let cursor: own u64 = 0_u64;
  let total: own u64 = 0_u64;
  loop @stream {
    match ige<u64>(step, 200000_u64) {
      True() => {
        break @stream;
      }
      False() => {
      }
    }
    let previous: own u64 = window[cursor];
    let mixed: own u64 = ixor<u64>(previous, step);
    set window[cursor] = imul.wrap<u64>(mixed, 1099511628211_u64);
    set total = iadd.wrap<u64>(total, previous);
    let at_end: own Bool = ieq<u64>(cursor, 7_u64);
    let next_cursor: own u64 = match at_end {
      True() => {
        give 0_u64;
      }
      False() => {
        give iadd.wrap<u64>(cursor, 1_u64);
      }
    }
    set cursor = next_cursor;
    set step = iadd.trap<u64>(step, 1_u64);
  }
  check ieq<u64>(step, 200000_u64) else trap "stream length drift";
  check ieq<u64>(cursor, 0_u64) else trap "stream cursor drift";
  return unit;
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let (total, outside_entry) = slot_declarations(main);
    assert!(
        total > 0,
        "the kernel must still need stack slots for this test to mean anything:\n{main}"
    );
    assert_eq!(
        outside_entry, 0,
        "{outside_entry} of {total} stack slots are declared outside the entry block, so the frame grows once per iteration:\n{main}"
    );

    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "the loop must run to completion instead of exhausting the stack: {:?}",
        output.status
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_mutable_array_checksum_executes() {
    let output = compile_and_run(&compile(include_bytes!(
        "../../../../tests/conformance/cases/x-array-mutable-checksum-run.wf"
    )));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn nested_struct_array_updates_rebuild_every_aggregate_layer_after_the_rhs() {
    let source = br#"struct Inner {
  values: array<u8, 2>;
  sibling: u16;
}

struct Outer {
  prefix: u32;
  inner: Inner;
}

fn replacement() -> own u8 traps {
  check True() else trap "replacement drift";
  return 9_u8;
}

fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  let inner: own Inner = Inner(values: move values, sibling: 77_u16);
  let outer: own Outer = Outer(prefix: 123_u32, inner: move inner);
  set outer.inner.values[1_u64] = replacement();
  let stored: own u8 = outer.inner.values[1_u64];
  check ieq<u8>(stored, 9_u8) else trap "array update";
  check ieq<u16>(outer.inner.sibling, 77_u16) else trap "inner sibling";
  check ieq<u32>(outer.prefix, 123_u32) else trap "outer sibling";
  return unit;
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let bounds = main
        .find("icmp ult i64")
        .expect("projected array target must retain its bounds check");
    let target_trap = main[bounds..]
        .find("call void @wf_trap")
        .map(|offset| bounds + offset)
        .expect("projected array target must retain its OP-4 trap edge");
    let rhs = main[target_trap..]
        .find("call i8 @wf_replacement")
        .map(|offset| target_trap + offset)
        .expect("RHS must follow projected target evaluation");
    let rebuild = main[rhs..]
        .find("insertvalue %wf.t")
        .map(|offset| rhs + offset)
        .expect("projected update must rebuild its enclosing structs");
    assert!(bounds < target_trap && target_trap < rhs && rhs < rebuild);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
