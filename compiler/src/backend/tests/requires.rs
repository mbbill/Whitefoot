use super::{compile, compile_and_run, compile_rejection, emitted_function};

const OUTPUT_CAPACITY: &[u8] = br#"fn copy_bytes(out: &uniq MutSlice<u8>, source: own Vector<u8>, store: &uniq Heap) -> written: own u64 reads(source), writes(out, store) contract {
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

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
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
          set output = place_back(vector: move output, value: 0_u8);
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
              set source = place_back(vector: move source, value: 7_u8);
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
fn command_entry_rejects_a_contract_instead_of_emitting_a_wrapper_check() {
    let failure = compile_rejection(
        br#"command fn main() -> status: own ExitStatus pure contract {
  requires True();
} {
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_eq!(failure.rule_id(), Some("FN-7"));
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

command fn main() -> status: own ExitStatus pure {
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

command fn main() -> status: own ExitStatus pure {
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

command fn main() -> status: own ExitStatus pure {
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

command fn main() -> status: own ExitStatus pure {
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

command fn main() -> status: own ExitStatus pure {
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
