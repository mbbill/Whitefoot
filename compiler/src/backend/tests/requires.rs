use super::{compile, compile_and_run, compile_rejection, emitted_function};

#[test]
fn returned_borrow_guards_check_the_delivered_value_across_retained_calls() {
    let source = br#"const alternative: u64 = 9_u64;

fn select['r](value: &'r u64) -> result: &'r u64 reads(value) {
  if deref(value) == 0_u64 {
    return &'r alternative;
  } else {
    return value;
  }
}

fn indexed(value: &u64) -> result: own u64 reads(value) contract {
  requires deref(value) < 2_u64;
} {
  let rows = array_new::<u64, 2>(7_u64);
  let index = deref(value);
  return rows[index];
}

fn forward(value: &u64) -> result: own u64 reads(value) {
  let chosen = select(value: value);
  if deref(chosen) < 2_u64 {
    return indexed(value: chosen);
  } else {
    return 99_u64;
  }
}

fn main() -> status: own ExitStatus pure {
  let zero = 0_u64;
  let one = 1_u64;
  region {
    let refused = forward(value: &zero);
    let accepted = forward(value: &one);
    if refused != 99_u64 {
      return exit_status(code: 1_u8);
    }
    if accepted != 7_u64 {
      return exit_status(code: 2_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [super::OverlapLowering::Off, super::OverlapLowering::On] {
        let module = super::emit_lowered(source, overlap)
            .lines()
            .map(|line| {
                if let Some(header) = line
                    .strip_prefix("define ")
                    .and_then(|line| line.strip_suffix(" {"))
                {
                    format!("define {header} noinline {{\n")
                } else {
                    format!("{line}\n")
                }
            })
            .collect::<String>();
        let output = compile_and_run(&module);
        assert_eq!(output.status.code(), Some(0), "{overlap:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

const OUTPUT_CAPACITY: &[u8] = br#"fn copy_bytes['heap](out: &uniq MutSlice<u8>, source: own Vector<'heap, u8>, store: &uniq Heap<'heap>) -> written: own u64 reads(source), writes(out, store) contract {
  define out_length = len_of(deref(out));
  define source_length = len_of(source);
  requires source_length <= out_length;
} {
  let length = len_of(source);
  for (offset in 0_u64..length) {
    let value = source[offset];
    set deref(out)[offset] = value;
  }
  return length;
}

fn main['heap](inputs: own Inputs, heap: own Heap<'heap>) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  let Inputs(args: unused_args, cwd: unused_cwd, stdout: unused_stdout, stderr: unused_stderr, handles: entry_factory, stdin: unused_stdin) = move inputs;
  region {
    close_directory(factory: &uniq entry_factory, directory: move unused_cwd);
  }
  let length = 4_u64;
  region {
    match heap_vector::<u8>(store: &uniq heap, count: length) {
      None() => {
        return exit_status(code: 4_u8);
      }
      Some(value: blank) => {
        let output = move blank;
        for @clear (
          at in 0_u64..4_u64,
          invariant grown: len_of(output) >= at,
          invariant spare: room_of(output) + at >= 4_u64,
          invariant flat: head_of(output) <= 0_u64
        ) {
          place_back(vector: &uniq output, value: 0_u8);
        }
        match heap_vector::<u8>(store: &uniq heap, count: length) {
          None() => {
            return exit_status(code: 3_u8);
          }
          Some(value: fresh) => {
            let source = move fresh;
            for @fill (
              at in 0_u64..4_u64,
              invariant grown: len_of(source) >= at,
              invariant spare: room_of(source) + at >= 4_u64,
              invariant flat: head_of(source) <= 0_u64
            ) {
              place_back(vector: &uniq source, value: 7_u8);
            }
            region {
              let destination = mut_slice_of(&uniq output);
              let room = len_of(destination);
              let held = len_of(source);
              if held <= room {
              } else {
                return exit_status(code: 5_u8);
              }
              region {
                let written = copy_bytes(out: &uniq destination, source: move source, store: &uniq heap);
                if written != length {
                  return exit_status(code: 1_u8);
                }
              }
            }
            let last = output[3_u64];
            if last != 7_u8 {
              return exit_status(code: 2_u8);
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn an_ordinary_selected_function_keeps_its_contract_without_a_wrapper_check() {
    // C2 deletes FN-7's command-entry contract refusal. This ordinary source
    // requirement is statically true; the build caller proves it normally.
    let module = compile(
        br#"fn main() -> status: own ExitStatus pure contract {
  requires 0_u64 == 0_u64;
} {
  return exit_status(code: 0_u8);
}
"#,
    );
    assert!(!emitted_function(&module, "main").contains("icmp"));
    assert!(compile_and_run(&module).status.success());
    // A false requirement does not invalidate an ordinary declaration. The
    // build caller cannot prove it, so no executable entry is supplied.
    let library = compile(
        br#"fn main() -> status: own ExitStatus pure contract {
  requires 0_u64 == 1_u64;
} {
  return exit_status(code: 0_u8);
}
"#,
    );
    assert!(emitted_function(&library, "main").contains("unreachable"));
    assert!(!library.contains("define i32 @main("));
}

#[test]
fn contradictory_requirements_emit_an_unreachable_body_without_a_trap() {
    let llvm = compile(
        br#"fn impossible(value: own i32) -> out: own i32 pure contract {
  requires value == 0_i32;
  requires value != 0_i32;
} {
  return value;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
    let impossible = emitted_function(&llvm, "impossible");
    assert!(impossible.contains("unreachable"));
    assert!(!impossible.contains("call void @wf_trap"));
}

#[test]
fn contract_define_is_symbolic_and_not_emitted_as_runtime_work() {
    let llvm = compile(
        br#"fn identity(value: own u8) -> out: own u8 pure contract {
  define bits = ipopcount(value);
  requires bits == 0_u32;
  ensures out == value;
} {
  return value;
}

fn main() -> status: own ExitStatus pure {
  let bits = ipopcount(0_u8);
  if bits == 0_u32 {
    let zero = identity(value: 0_u8);
    return exit_status(code: zero);
  } else {
    return exit_status(code: 1_u8);
  }
}
"#,
    );
    let identity = emitted_function(&llvm, "identity");
    assert!(!identity.contains("llvm.ctpop"));
    assert!(!identity.contains("call void @wf_trap"));
}

#[test]
fn contract_define_can_hold_a_float_endpoint_conversion_without_runtime_code() {
    let llvm = compile(
        br#"fn identity(value: own u8) -> out: own u8 pure contract {
  define converted = cvt::<u8, f32>(value);
  requires feq(converted, 1.0_f32);
  ensures out == value;
} {
  return value;
}

fn main() -> status: own ExitStatus pure {
  let converted = cvt::<u8, f32>(1_u8);
  if feq(converted, 1.0_f32) {
    let one = identity(value: 1_u8);
    let code = one -wrap 1_u8;
    return exit_status(code: code);
  } else {
    return exit_status(code: 2_u8);
  }
}
"#,
    );
    let identity = emitted_function(&llvm, "identity");
    assert!(!identity.contains("uitofp"));
    assert!(!identity.contains("call void @wf_trap"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn ordinary_requirement_is_not_emitted_as_a_callee_prologue() {
    let llvm = compile(
        br#"fn bounded(value: own i32) -> out: own i32 pure contract {
  requires value >= 0_i32;
  ensures out == value;
} {
  return value;
}

fn main() -> status: own ExitStatus pure {
  let value = 7_i32;
  let returned = bounded(value: value);
  if returned != 7_i32 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    let bounded = emitted_function(&llvm, "bounded");
    assert!(!bounded.contains("br i1"));
    assert!(!bounded.contains("call void @wf_trap"));
    assert!(bounded.contains("ret i32"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn a_requirement_must_be_discharged_at_each_ordinary_call() {
    let failure = compile_rejection(
        br#"fn positive(value: own i32) -> out: own i32 pure contract {
  requires value > 0_i32;
} {
  return value;
}

fn main() -> status: own ExitStatus pure {
  let unknown = 0_i32;
  let returned = positive(value: unknown);
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_eq!(failure.rule_id(), Some("FN-8"));
    assert!(failure.detail().contains("positive"));
}

#[test]
fn borrowed_output_capacity_contract_informs_the_body_without_a_callee_prologue() {
    let llvm = compile(OUTPUT_CAPACITY);
    let copy = emitted_function(&llvm, "copy_bytes");
    assert!(copy.contains("switch i1"));
    // The requirement and counted-range binder facts are proof inputs only.
    // They are absent after lowering, and both discharged subscripts emit no
    // additional bounds comparison or runtime proof check. The sole strict
    // comparison is the counted loop's continuation condition.
    assert_eq!(copy.matches("icmp ule i64").count(), 0);
    assert_eq!(copy.matches("icmp ult i64").count(), 1);
    assert_eq!(copy.matches("call void @wf_trap").count(), 0);
    assert!(copy.contains("load i8"));
    assert!(copy.contains("store i8"));
    // One release, for the reason the buffer's free had: the callee holds
    // the store's provider, so the run it was handed is affine there and its
    // release is derived on the return edge [PROV-1, BLK-1, STOR-3].
    assert_eq!(copy.matches("call void @free").count(), 1);
    assert!(!copy.contains("llvm.assume"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
