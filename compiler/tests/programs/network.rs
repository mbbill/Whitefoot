use super::support::{compile_and_run, compile_program, emitted_function};

#[test]
fn ipv4_checksum_uses_one_slice_consumer_for_static_and_runtime_storage() {
    let llvm = compile_program("ipv4_checksum.wf");
    let checksum = emitted_function(&llvm, "ipv4_checksum");
    let main = emitted_function(&llvm, "main");
    // The discharged slice reads emit no bounds branch; the loop invariants
    // establish the address domains before the element addresses form.
    assert!(checksum.contains("getelementptr inbounds i8"));
    assert!(!checksum.contains("call void @free"));
    assert_eq!(main.matches("call i16 @wf_ipv4_checksum").count(), 2);
    // B7c4b-1: the runtime copy of the header is a run taken from one bump
    // extent reserved in this activation's frame, so the program reaches the
    // host allocator on no path at all and every validation-failure return
    // leaves the extent with the frame. The free this assertion used to count
    // was the heap buffer's, and there is no heap buffer any more.
    assert!(!main.contains("call void @free"));
    assert!(!llvm.contains("call ptr @malloc"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
