use super::support::{
    build_program, compile_program, compile_program_with_overlap, emitted_function,
};

#[test]
fn sha256_compression_executes_as_a_sustained_workload() {
    let llvm = compile_program("sha256_abc.wf");
    let compression = emitted_function(&llvm, "sha256_abc_word_zero");
    assert!(llvm.contains("call i32 @llvm.fshr.i32"));
    assert!(compression.contains("getelementptr inbounds [64 x i32]"));
    assert!(!compression.contains("@wf_trap"));
    assert!(llvm.contains("i32 3128432319"));

    // Retains the original missing-phi-predecessor regression in both worlds.
    for (module, widths) in [
        (llvm, &["1"][..]),
        (
            compile_program_with_overlap("sha256_abc.wf"),
            &["1", "4"][..],
        ),
    ] {
        let program = build_program(&module);
        for width in widths {
            let output = program.run_with_workers(Some(width));
            assert!(output.status.success(), "{width}: {output:?}");
            assert!(output.stdout.is_empty());
            assert!(output.stderr.is_empty());
        }
    }
}
