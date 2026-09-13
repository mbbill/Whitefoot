use super::support::{compile_and_run, compile_program};

#[test]
fn adaptive_quadrature_matches_the_analytic_integral_with_and_without_leaf_offers() {
    use whitefoot::{CompilerLimits, OverlapLowering, SourceInput, compile_with_permission_ledger};

    let source = include_bytes!("../../../tests/programs/adaptive_quadrature.wf");
    let mut outputs = Vec::new();
    for mode in [
        OverlapLowering::Off,
        OverlapLowering::On,
        OverlapLowering::OnWithoutSmallScalarLeaves {
            maximum_operations: 16,
        },
    ] {
        let (module, ledger) = compile_with_permission_ledger(
            &[SourceInput::new("adaptive_quadrature.wf", source)],
            CompilerLimits::default(),
            mode,
        )
        .expect("the same numerical program compiles under each offer policy");
        let omitted = ledger
            .iter()
            .filter(|line| line.contains("scalar leaf limit"));
        if matches!(mode, OverlapLowering::OnWithoutSmallScalarLeaves { .. }) {
            assert_eq!(omitted.count(), 2);
        } else {
            assert_eq!(omitted.count(), 0);
        }
        let output = compile_and_run(&module);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(output.stdout.len(), 64);
        let bits = std::str::from_utf8(&output.stdout).unwrap();
        let actual = f64::from_bits(u64::from_str_radix(bits, 2).unwrap());
        // Integrate the Lorentz profile analytically, without using Simpson
        // subdivision or the generated program's result as the expectation.
        let exact: f64 = (0..2048)
            .map(|sample| {
                let center = 0.25 + f64::from(sample) / 4096.0;
                let width = 1.0 / 64.0;
                width * (((1.0 - center) / width).atan() - (-center / width).atan())
            })
            .sum();
        assert!((actual - exact).abs() <= 1e-8, "{actual} vs {exact}");
        outputs.push(output.stdout);
    }
    assert!(outputs.windows(2).all(|pair| pair[0] == pair[1]));
}

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
