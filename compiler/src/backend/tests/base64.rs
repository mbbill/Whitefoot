use super::*;

#[test]
fn compiler_independent_base64_rfc_vectors_execute() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-base64-rfc-vectors-run.wf"
    ));
    let encode = emitted_function(&llvm, "encode");
    let main = emitted_function(&llvm, "main");
    // B7c4b-1: the three inputs are const runs and the three outputs come from
    // one bump extent, so nothing in this program reaches the host allocator
    // and no edge carries a free.
    assert!(encode.starts_with("define internal %wf.t4 @wf_encode({ ptr, i64 } "));
    assert_eq!(encode.matches("call void @free").count(), 0);
    assert_eq!(main.matches("call %wf.t4 @wf_encode").count(), 3);
    assert_eq!(main.matches("call void @free").count(), 0);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
