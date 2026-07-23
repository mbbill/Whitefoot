use super::support::{compile_and_run, compile_program};

#[test]
fn grayscale_pixels_execute_through_exact_float_to_byte_conversion() {
    let llvm = compile_program("grayscale_pixels.wf");
    assert!(llvm.contains("fptoui.sat"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
