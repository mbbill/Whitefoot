use super::*;

#[test]
fn base64_has_typed_results_and_no_host_allocation() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-base64-rfc-vectors-run.wf"
    ));
    let encode = emitted_function(&llvm, "encode");
    let main = emitted_function(&llvm, "main");
    // All three input/output pairs are inline arrays, so nothing in this
    // program reaches the host allocator and no edge carries a free.
    // Each range reference crosses the call as its element pointer, carrying
    // the reference facts, and its count (compiler/backend-facts). The
    // three-leaf Result returns in registers, so no destination precedes them.
    let result = register_result_type(&llvm, encode, &["i32", "i64", "i1"]);
    assert!(encode.contains(" @wf_encode(ptr noalias nonnull "));
    assert!(encode.contains(" %wf.arg.v0.data, i64 %wf.arg.v0.len, ptr noalias nonnull "));
    assert!(encode.contains(" %wf.arg.v1.data, i64 %wf.arg.v1.len)"));
    // Both result routes initialize the typed result slot: tag, u64
    // success, and fieldless one-bit IndexError on the error route.
    assert_scalar_result_fields(&llvm, encode, &["i32", "i64", "i1"]);
    assert_eq!(encode.matches("call ptr @malloc").count(), 0);
    assert_eq!(encode.matches("call void @free").count(), 0);
    assert_eq!(
        main.matches(&format!(" = call {result} @wf_encode(ptr "))
            .count(),
        3
    );
    assert_eq!(main.matches("call ptr @malloc").count(), 0);
    assert_eq!(main.matches("call void @free").count(), 0);
}
