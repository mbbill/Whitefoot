use super::{compile, compile_and_run, compile_rejection, emitted_function};

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
    // The constant lookup is discharged [OP-4]: no bounds compare remains.
    // The source's two terminal result checks are ordinary control flow, not
    // claims, so all three outcomes return an ExitStatus without a trap edge.
    assert!(!main.contains("icmp ult i64"));
    assert_eq!(main.matches("icmp eq").count(), 2);
    assert_eq!(main.matches("call i8 @wf.sys.exit_status.v1").count(), 3);
    assert!(!main.contains("call void @wf_trap"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn filled_arrays_cross_function_boundaries_and_keep_a_checked_read() {
    let source = br#"fn make() -> result: own array<u16, 4> pure {
  return array_new::<u16, 4>(42_u16);
}

fn clamp_three(value: own u64) -> result: own u64 pure contract {
  ensures result < 4_u64;
} {
  if value < 4_u64 {
    return value;
  } else {
    return 3_u64;
  }
}

fn read(values: own array<u16, 4>, offset: own u64) -> result: own u16 pure {
  let bounded = clamp_three(value: offset);
  let value = values[bounded];
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let values = make();
  let length = len_of(values);
  if length != 4_u64 {
    return exit_status(code: 1_u8);
  }
  let value = read(values: move values, offset: 3_u64);
  if value != 42_u16 {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let read = emitted_function(&llvm, "read");
    // The verified callee summary discharges the subscript directly: the
    // caller emits no runtime fallback and no second bounds branch.
    assert!(!read.contains("icmp ult i64"));
    assert!(!read.contains("call void @wf_trap"));
    assert!(read.contains("getelementptr inbounds [4 x i16]"));
    assert!(llvm.contains("array.fill.head"));
    assert!(llvm.contains("array.fill.done"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn an_out_of_bounds_array_read_is_an_op4_compile_rejection() {
    // Under discharge-or-reject [OP-4] no runtime bounds trap exists: the
    // underivable obligation rejects at compile time with the exact
    // [ENT-6] residual.
    let source = br#"const values: FixedVector<u8, 2> =[7_u8, 7_u8];

command fn main() -> status: own ExitStatus pure {
  let value = values[2_u64];
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len_of(values)"));
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
    let source = br#"fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 2>(0_u8);
  set values[1_u64] = replacement();
  let stored = values[1_u64];
  if stored != 9_u8 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // The discharged target carries no bounds branch [OP-4]; SET-1's order
    // remains: the RHS call is emitted after target formation and the store
    // after the RHS.
    let rhs = main
        .find("call i8 @wf_replacement")
        .expect("RHS must be emitted after target evaluation");
    assert!(
        !main[..rhs].contains("call void @wf_trap"),
        "no bounds trap edge precedes the RHS"
    );
    let store = main[rhs..]
        .find("store i8")
        .map(|offset| rhs + offset)
        .expect("array update must store only after the RHS");
    assert!(rhs < store);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn an_out_of_bounds_indexed_set_is_an_op4_compile_rejection() {
    // A target whose obligation is underivable cannot reach runtime: the
    // program rejects at the subscript with the residual [OP-4, ENT-6].
    let source = br#"fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 2>(0_u8);
  set values[2_u64] = replacement();
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len_of(values)"));
}

#[test]
fn a_long_loop_over_a_dynamically_indexed_array_keeps_the_frame_bounded() {
    // Both the read and the indexed set need a stack slot for the array value,
    // and the index is not a compile-time constant, so neither slot can be
    // promoted away. The structural half is what discriminates: no slot may be
    // declared outside the entry block, so a frame that grew once per
    // iteration fails here whatever it would survive. The run is a
    // corroboration rather than the measurement — 200000 iterations of two
    // 64-byte slots is about 25 MB, which used to be past a default 8 MB limit
    // and now fits inside the 1 GiB stack the runtime gives every thread.
    let source = br#"command fn main() -> status: own ExitStatus pure {
  doc "Nested counted loops read and write one fixed array for two hundred thousand iterations.";
  let window = array_new::<u64, 8>(1_u64);
  let completed = 0_u64;
  let total = 0_u64;
  for (batch in 0_u64..25000_u64) {
    for (cursor in 0_u64..8_u64) {
      let previous = window[cursor];
      let mixed = ixor(previous, completed);
      set window[cursor] = mixed *wrap 1099511628211_u64;
      set total = total +wrap previous;
      set completed = completed +wrap 1_u64;
    }
  }
  if completed != 200000_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
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

fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 2>(0_u8);
  let inner = Inner(values: move values, sibling: 77_u16);
  let outer = Outer(prefix: 123_u32, inner: move inner);
  set outer.inner.values[1_u64] = replacement();
  let stored = outer.inner.values[1_u64];
  if stored != 9_u8 {
    return exit_status(code: 1_u8);
  }
  if outer.inner.sibling != 77_u16 {
    return exit_status(code: 2_u8);
  }
  if outer.prefix != 123_u32 {
    return exit_status(code: 3_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // The discharged projected target carries no bounds branch [OP-4].
    let rhs = main
        .find("call i8 @wf_replacement")
        .expect("RHS must follow projected target evaluation");
    assert!(
        !main[..rhs].contains("call void @wf_trap"),
        "no bounds trap edge precedes the RHS"
    );
    let rebuild = main[rhs..]
        .find("insertvalue %wf.t")
        .map(|offset| rhs + offset)
        .expect("projected update must rebuild its enclosing structs");
    assert!(rhs < rebuild);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
