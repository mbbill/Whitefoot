use super::support::{compile_and_run, compile_program, emitted_function};

#[test]
fn sha256_compression_executes_as_a_sustained_workload() {
    let llvm = compile_program("sha256_abc.wf");
    let compression = emitted_function(&llvm, "sha256_abc_word_zero");
    assert!(llvm.contains("call i32 @llvm.fshr.i32"));
    assert!(compression.contains("getelementptr inbounds [64 x i32]"));
    assert!(!compression.contains("@wf_trap"));
    assert!(llvm.contains("i32 3128432319"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
