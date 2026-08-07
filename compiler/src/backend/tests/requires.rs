use super::{compile, compile_and_run, emitted_function};

const OUTPUT_CAPACITY: &[u8] =
    include_bytes!("../../../../tests/conformance/cases/x-requires-output-capacity-run.wf");

#[test]
fn checked_requires_runs_before_the_function_body() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/fn8-pos-requires-run.wf"
    ));
    let bounded = emitted_function(&llvm, "bounded");
    let guard = bounded.find("br i1").expect("requires check must branch");
    let body_return = bounded
        .rfind("ret i32")
        .expect("function body must retain its return");
    assert!(guard < body_return);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn false_requires_traps_before_the_body() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/fn8-trap-requires-false.wf"
    ));
    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("trap record must be UTF-8");
    assert!(stderr.contains("\"rule_id\":\"OP-5\""));
    assert!(stderr.contains("\"message\":\"x must be positive\""));
}

#[test]
fn borrowed_output_capacity_prologue_executes_through_the_normal_pipeline() {
    let llvm = compile(OUTPUT_CAPACITY);
    let copy = emitted_function(&llvm, "copy_bytes");
    assert!(copy.contains("br i1"));
    // The requires compare plus the one claim compare: the discharged
    // subscripts themselves emit no bounds compares [OP-4].
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
