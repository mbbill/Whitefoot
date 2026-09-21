//! Implementation obligations beyond the source-level FN-10 corpus verdicts.

use super::system::with_mutated_ir_lowering;
use super::{compile_and_run, emitted_function};
use crate::{IrNominalKind, IrTerminator, IrType, OverlapLowering, emit_llvm};

#[test]
fn musttail_is_a_backedge_before_host_optimization_in_both_worlds() {
    let source = include_bytes!("../../../../tests/conformance/cases/fn10-pos-self-transfer.wf");
    for overlap in [OverlapLowering::Off, OverlapLowering::On] {
        let module = with_mutated_ir_lowering(source, overlap, |program| {
            let exchange = program
                .functions()
                .iter()
                .find(|function| function.name() == "exchange")
                .expect("exchange function");
            let IrTerminator::Jump { target: entry, .. } = exchange.blocks()[0].terminator() else {
                panic!("the prologue must enter the parameterized body");
            };
            let backedges = exchange
                .blocks()
                .iter()
                .skip(1)
                .filter_map(|block| match block.terminator() {
                    IrTerminator::Jump { target, drops, .. } if target == entry => Some(drops),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(backedges.len(), 1);
            assert_eq!(backedges[0].iter().filter(|drop| matches!(drop.ty(), IrType::Nominal(id) if matches!(program.nominal(id).expect("drop type").kind(), IrNominalKind::Box { .. }))).count(), 1, "the spare cell must be released exactly once on the tail edge");
            let mut module = emit_llvm(program)
                .expect("tail transfer emits")
                .into_string();
            module.push_str(
                &crate::driver::launcher::render(program, "main").expect("ordinary test launcher"),
            );
            module
        });
        for name in ["sum", "exchange", "swap_pairs"] {
            let body = emitted_function(&module, name);
            assert_eq!(
                body.matches(&format!("@wf_{name}(")).count(),
                1,
                "self calls must already be jumps: {body}"
            );
            assert!(
                body.contains("phi "),
                "entry parameters must carry the new activation: {body}"
            );
            let (_, repeated) = body.split_once("bb1:").expect("parameterized body entry");
            assert!(
                !repeated.contains("alloca "),
                "the repeated body must reuse its physical frame: {body}"
            );
        }
        // The conformance adapter executes the sequential module; this native
        // run additionally protects actualized parallel lowering.
        if overlap == OverlapLowering::On {
            assert!(
                module.contains("call void @wf__par_publish(ptr "),
                "the parallel control must actually offer work"
            );
            assert!(
                module.contains("@wf__par_seq_rotate$instance$"),
                "the handed-out tail-recursive function must have a sequential clone"
            );
            let output = compile_and_run(&module);
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}
