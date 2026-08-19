use super::{compile, compile_and_run, emit, emitted_function, optimized_main};

const OUTPUT_CAPACITY: &[u8] =
    include_bytes!("../../../../tests/conformance/cases/x-requires-output-capacity-run.wf");

#[test]
fn unlabelled_entry_goal_is_inlined_once_in_the_process_wrapper() {
    let llvm = compile(
        br#"command fn main() -> status: own ExitStatus pure requires {
  check ieq(7_u8, 7_u8) else trap "entry equality";
} {
  return unit;
}
"#,
    );
    let wrapper = optimized_main(&llvm);
    assert_eq!(wrapper.matches("call i8 @wf_main()").count(), 1);
    assert_eq!(wrapper.matches("call void @wf_trap").count(), 1);
    assert!(wrapper.find("icmp eq i8").unwrap() < wrapper.find("call i8 @wf_main()").unwrap());
    assert!(!emitted_function(&llvm, "main").contains("br i1"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn false_command_entry_goal_traps_after_setup_without_calling_or_cleaning_the_body() {
    let llvm = compile(
        br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus external, blocks requires {
  check ieq(0_u8, 1_u8) else trap "entry rejected";
} {
  return exit_status(code: 0_u8);
}
"#,
    );
    let wrapper = optimized_main(&llvm);
    let setup = wrapper
        .find("%cwd.opened")
        .expect("cwd setup must complete");
    let goal = wrapper
        .find("icmp eq i8")
        .expect("entry goal must be inline");
    let body = wrapper
        .find("call i8 @wf_main(")
        .expect("the true edge owns the sole body call");
    assert!(setup < goal && goal < body);
    assert_eq!(wrapper.matches("call i8 @wf_main(").count(), 1);
    assert!(!wrapper.contains("call i32 @close"));

    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("trap record must be UTF-8");
    assert!(stderr.contains("\"rule_id\":\"OP-5\""));
    assert!(stderr.contains("\"message\":\"entry rejected\""));
}

#[test]
fn entry_only_intrinsic_is_declared_before_the_wrapper_uses_it() {
    let llvm = emit(
        br#"command fn main() -> status: own ExitStatus pure requires {
  let count = ipopcount(0_u8);
  check ieq(count, 0_u32) else trap "entry count";
} {
  return unit;
}
"#,
    );
    let declaration = llvm
        .find("declare i8 @llvm.ctpop.i8(i8)")
        .expect("entry-only intrinsic declaration must be collected");
    let wrapper = llvm
        .find("define i32 @main()")
        .expect("the process wrapper must be emitted");
    assert!(declaration < wrapper);
    assert!(optimized_main(&llvm).contains("call i8 @llvm.ctpop.i8"));
}

#[test]
fn entry_float_endpoint_conversion_uses_the_wrapper_value_namespace() {
    let llvm = compile(
        br#"command fn main() -> status: own ExitStatus pure requires {
  let converted = cvt<u8, f32>(1_u8);
  check feq(converted, 1.0_f32) else trap "entry conversion";
} {
  return unit;
}
"#,
    );
    let wrapper = optimized_main(&llvm);
    assert!(wrapper.contains("%entry.goal.v0"));
    assert!(wrapper.contains("%entry.goal.v1"));
    assert!(!wrapper.contains("%v0"));
    assert!(!wrapper.contains("%v1"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn ordinary_requirement_is_not_emitted_as_a_callee_prologue() {
    let llvm = compile(
        br#"fn bounded(value: own i32) -> result: own i32 pure requires {
  check ige(value, 0_i32) else trap "nonnegative";
} {
  return value;
}

command fn main() -> status: own ExitStatus traps {
  let value = 7_i32;
  claim caller_evidence: ige(value, 0_i32) because "caller evidence";
  let result = bounded(value: value);
  claim result_drift: ieq(result, 7_i32) because "result drift";
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
fn false_unlabelled_entry_requirement_traps_in_the_wrapper() {
    let llvm = compile(
        br#"command fn main() -> status: own ExitStatus pure requires {
  check ieq(0_u8, 1_u8) else trap "entry rejected";
} {
  return unit;
}
"#,
    );
    assert!(!emitted_function(&llvm, "main").contains("br i1"));
    assert_eq!(
        optimized_main(&llvm).matches("call i8 @wf_main()").count(),
        1
    );
    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("trap record must be UTF-8");
    assert!(stderr.contains("\"rule_id\":\"OP-5\""));
    assert!(stderr.contains("\"message\":\"entry rejected\""));
}

#[test]
fn borrowed_output_capacity_requirement_informs_the_body_without_a_callee_prologue() {
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
