use super::*;

const RAW_DEFLATE: &[u8] = include_bytes!("../../../../tests/programs/raw_deflate.wf");

#[test]
fn raw_deflate_stored_blocks_execute_with_data_failures() {
    let llvm = compile(RAW_DEFLATE);
    let inflate = emitted_function(&llvm, "inflate");
    assert!(inflate.contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
