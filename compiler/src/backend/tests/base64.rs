use super::*;

#[test]
fn compiler_independent_base64_rfc_vectors_execute() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-base64-rfc-vectors-run.wf"
    ));
    let encode = emitted_function(&llvm, "encode");
    let main = emitted_function(&llvm, "main");
    assert!(encode.starts_with("define internal %wf.t4 @wf_encode({ ptr, i64 } "));
    assert_eq!(encode.matches("call void @free").count(), 19);
    assert_eq!(main.matches("call %wf.t4 @wf_encode").count(), 3);
    assert_eq!(main.matches("call void @free").count(), 63);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
