use super::support::{compile_and_run, compile_program};

#[test]
fn fir_filter_executes_with_nested_fixed_array_state() {
    let llvm = compile_program("fir_filter.wf");
    assert!(llvm.contains("getelementptr inbounds [8 x double]"));
    assert!(llvm.contains("insertvalue %wf.t"));
    assert!(!llvm.contains("call void @wf_trap"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
