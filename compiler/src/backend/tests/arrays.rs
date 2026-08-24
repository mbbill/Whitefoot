use super::{compile, compile_and_run, compile_rejection, emitted_function, system};

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
  return array_new<u16, 4>(42_u16);
}

fn clamp_three(value: own u64) -> result: own u64 pure contract {
  ensures ilt(result, 4_u64);
} {
  if ilt(value, 4_u64) {
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
  let length = len(values);
  if ine(length, 4_u64) {
    return exit_status(code: 1_u8);
  }
  let value = read(values: move values, offset: 3_u64);
  if ine(value, 42_u16) {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let read = emitted_function(&llvm, "read");
    // The verified callee summary discharges the subscript directly: the
    // caller emits neither a claim trap nor a second bounds branch.
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
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let values = array_new<u8, 2>(7_u8);
  let value = values[2_u64];
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len(values)"));
}

#[test]
fn a_failing_claim_reports_its_clm1_record_before_abort() {
    // The source claim first passes the complete residual judgment as a true,
    // load-bearing theorem. The test-only IR mutation then redirects exactly
    // that named claim to an existing false value so DIAG-3 can exercise the
    // runtime record without granting source writers a false-claim escape.
    let source = br#"command fn main() -> status: own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  let bounded = 0_u64;
  let step = 0_u64;
  loop @preserve_zero {
    if ige(step, 4_u64) {
      break @preserve_zero;
    }
    set bounded = bounded +wrap 0_u64;
    set step = step +wrap 1_u64;
  }
  let in_range = ilt(bounded, 4_u64);
  let injected_false = False();
  claim expected_true: in_range because "premises: bounded starts at 0_u64 and each completed preserve_zero iteration adds wrapping zero\nderivation: adding wrapping zero preserves bounded at 0_u64 through every completed iteration\nconclusion: in_range is true\nchecker gap: ENT does not synthesize the loop invariant that bounded remains zero\nconsumers: values[bounded] requires this exact bound";
  let ignored = values[bounded];
  return exit_status(code: 0_u8);
}
"#;
    let llvm = system::with_mutated_ir(source, |program| {
        assert!(program.force_claim_false_for_test("main", "expected_true"));
        crate::emit_llvm(program)
            .expect("fault-injected checked IR must emit")
            .into_string()
    });
    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"CLM-1\",\"message\":\"expected_true\",\"function\":\"main\",\"node_path\":["
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
    let source = br#"fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new<u8, 2>(0_u8);
  set values[1_u64] = replacement();
  let stored = values[1_u64];
  if ine(stored, 9_u8) {
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
  let values = array_new<u8, 2>(0_u8);
  set values[2_u64] = replacement();
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len(values)"));
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
    let source = br#"command fn main() -> status: own ExitStatus traps {
  doc "A long loop reads and writes one fixed array through a rotating index.";
  let window = array_new<u64, 8>(1_u64);
  let step = 0_u64;
  let cursor = 0_u64;
  let total = 0_u64;
  loop @stream {
    if ige(step, 200000_u64) {
      break @stream;
    }
    let cursor_ok = ilt(cursor, 8_u64);
    claim cursor_in_window: cursor_ok because "premises: cursor starts at 0_u64 and every continuing iteration replaces 7_u64 by 0_u64 or increments a value below 7_u64 by one\nderivation: induction over completed iterations keeps cursor in the closed interval 0_u64 through 7_u64 before each access; the only increment starts below 7_u64 and cannot wrap\nconclusion: cursor_ok is true\nchecker gap: ENT does not synthesize the loop induction invariant for the rotating cursor\nconsumers: the following read and indexed set each require this exact OP-4 bound";
    let previous = window[cursor];
    let mixed = ixor(previous, step);
    set window[cursor] = mixed *wrap 1099511628211_u64;
    set total = total +wrap previous;
    let at_end = ieq(cursor, 7_u64);
    let next_cursor = if at_end {
      give 0_u64;
    } else {
      give cursor +wrap 1_u64;
    }
    set cursor = next_cursor;
    set step = step + 1_u64;
  }
  if ine(step, 200000_u64) {
    return exit_status(code: 1_u8);
  }
  if ine(cursor, 0_u64) {
    return exit_status(code: 2_u8);
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
  let values = array_new<u8, 2>(0_u8);
  let inner = Inner(values: move values, sibling: 77_u16);
  let outer = Outer(prefix: 123_u32, inner: move inner);
  set outer.inner.values[1_u64] = replacement();
  let stored = outer.inner.values[1_u64];
  if ine(stored, 9_u8) {
    return exit_status(code: 1_u8);
  }
  if ine(outer.inner.sibling, 77_u16) {
    return exit_status(code: 2_u8);
  }
  if ine(outer.prefix, 123_u32) {
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
