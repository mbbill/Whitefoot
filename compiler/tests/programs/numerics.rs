use super::support::{compile_and_run, compile_program};

#[test]
fn feedback_controller_executes_as_a_sustained_float_workload() {
    let llvm = compile_program("feedback_controller.wf");
    assert!(llvm.contains("call double @llvm.fma.f64"));
    assert!(llvm.contains("fadd double"));
    assert!(!llvm.contains("fadd fast"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn mandelbrot_grid_executes_through_total_numeric_conversion() {
    let llvm = compile_program("mandelbrot_grid.wf");
    assert!(llvm.contains("uitofp i32"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn precision_polymorphic_geometry_executes_for_both_float_widths() {
    let llvm = compile_program("geometry_vectors.wf");
    assert!(llvm.contains("fmul float"));
    assert!(llvm.contains("fmul double"));
    assert!(llvm.contains("dot3$instance$"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
