use super::{compile, compile_and_run, compile_rejection, emitted_function};

const OUTPUT_CAPACITY: &[u8] = br#"fn copy_bytes['r](out: &uniq 'r buffer<u8>, source: own buffer<u8>) -> written: own u64 writes('r), traps contract {
  define out_length = len(deref(out));
  define source_length = len(source);
  requires ile(source_length, out_length);
} {
  let length = len(source);
  let offset = 0_u64;
  loop @copy {
    let done = ieq(offset, length);
    if done {
      break @copy;
    } else {
      let copy_ok = ilt(offset, length);
      claim copy_offset_in_source: copy_ok because "premises: offset starts at 0_u64, the loop exits when offset equals length, and each continuing iteration increments offset once; the contract establishes length at most len(deref(out))\nderivation: induction keeps offset at most length; in a continuing iteration offset is strictly below length, so the increment cannot wrap and both the source and output indices are in range\nconclusion: copy_ok is true\nchecker gap: ENT does not synthesize the monotone loop invariant relating offset to length\nconsumers: the following source read consumes this OP-4 bound directly and the following output set combines it with the contract bound";
      let value = source[offset];
      set deref(out)[offset] = value;
      set offset = offset +wrap 1_u64;
    }
  }
  return length;
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let length = 4_u64;
  let source = buffer_new(length, 7_u8);
  let output = buffer_new(length, 0_u8);
  region 'copy_region {
    let destination = &uniq 'copy_region output;
    let written = copy_bytes<'copy_region>(out: move destination, source: move source);
    if ine(written, length) {
      return exit_status(code: 1_u8);
    }
  }
  let last = output[3_u64];
  if ine(last, 7_u8) {
    return exit_status(code: 2_u8);
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
  requires ieq(value, 0_i32);
  requires ine(value, 0_i32);
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
  requires ieq(bits, 0_u32);
  ensures ieq(out, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let bits = ipopcount(0_u8);
  if ieq(bits, 0_u32) {
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
  define converted = cvt<u8, f32>(value);
  requires feq(converted, 1.0_f32);
  ensures ieq(out, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus traps {
  let converted = cvt<u8, f32>(1_u8);
  claim one_converts_exactly: feq(converted, 1.0_f32) because "premises: converted is the f32 conversion of the exactly representable u8 value 1_u8\nderivation: f32 represents the integer one exactly, so the conversion has the bit-exact value 1.0_f32\nconclusion: feq(converted, 1.0_f32) is true\nchecker gap: ENT does not evaluate integer-to-float conversion on literal endpoints\nconsumers: the following identity call requires this exact FN-8 equality";
  let one = identity(value: 1_u8);
  let code = one -wrap 1_u8;
  return exit_status(code: code);
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
  requires ige(value, 0_i32);
  ensures ieq(out, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let value = 7_i32;
  let returned = bounded(value: value);
  if ine(returned, 7_i32) {
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
  requires igt(value, 0_i32);
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
    assert!(copy.contains("br i1"));
    // The requirement's `ile` is absent. The body's one claim comparison
    // remains, while its discharged subscripts emit no bounds compares.
    assert_eq!(copy.matches("icmp ule i64").count(), 0);
    assert_eq!(copy.matches("icmp ult i64").count(), 1);
    assert!(copy.contains("load i8"));
    assert!(copy.contains("store i8"));
    assert_eq!(copy.matches("call void @free").count(), 1);
    assert!(!copy.contains("llvm.assume"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
