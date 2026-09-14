//! Typed-IR tests for ordinary calls, source ownership and proof erasure.

#![allow(clippy::panic)]

use crate::lexer::{LexLimits, LexOutcome, lex};
use crate::{
    ACTIVE_KERNEL_SPEC_HASH, CanonicalLimits, CanonicalOutcome, FinalizeLimits, FinalizeOutcome,
    OverlapLowering, ParseLimits, ParseOutcome, ResolutionOutcome, SemanticOutcome, SourceBundle,
    SourceInput, SourceLimits, TerminalLimits, TerminalOutcome, audit_canonical, check_semantics,
    classify_terminals, finalize, parse, resolve,
};

use super::{
    IrBlock, IrDrop, IrDropSubject, IrFunction, IrInstruction, IrIntegerOperation, IrOperation,
    IrProgram, IrSourceArgument, IrSourceCall, IrSourceMode, IrTerminator, IrType, IrValueId,
    lower_checked,
};

const SOURCE_LIMITS: SourceLimits = SourceLimits {
    max_sources: 64,
    max_logical_path_bytes: 128,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_binding_bytes: 1_048_576,
};

const LEX_LIMITS: LexLimits = LexLimits {
    max_sources: 64,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_token_bytes: 16_384,
    max_tokens: 131_072,
    max_lexemes: 262_144,
};

const PARSE_LIMITS: ParseLimits = ParseLimits {
    max_work: 8_000_000,
    max_tasks: 131_072,
    max_frames: 8_192,
    max_elements: 262_144,
};

const FINALIZE_LIMITS: FinalizeLimits = FinalizeLimits {
    max_work: 8_000_000,
    max_roots: 131_072,
    max_shape_tasks: 131_072,
    max_nodes: 131_072,
    max_child_edges: 131_072,
    max_terminals: 131_072,
    max_sources: 64,
};

const CANONICAL_LIMITS: CanonicalLimits = CanonicalLimits {
    max_work: 8_000_000,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_gaps: 131_072,
    max_path_components: 8_192,
};

/// An ordinary function selected by executable fixtures.
const PLAIN_ENTRY: &str =
    "fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

fn with_ir<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_ir_mode(source, OverlapLowering::Off, run)
}

fn with_ir_mode<ResultValue>(
    source: &[u8],
    overlap: OverlapLowering,
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_checked(source, |checked| {
        let ir = lower_checked(checked, overlap).expect("checked system program must lower");
        for function in ir.functions() {
            for source in &function.source_calls {
                let (target, arguments) = call_definition(function, source.result);
                let signature = ir.functions()[target as usize]
                    .source_signature
                    .as_ref()
                    .expect("a source call retains a source callee");
                assert_eq!(source.arguments.len(), arguments.len());
                assert_eq!(signature.parameters.len(), arguments.len());
                if let Some(argument) = source.returned_borrow_argument {
                    assert!(argument < arguments.len());
                    assert_ne!(signature.parameters[argument], IrSourceMode::Own);
                    assert_ne!(signature.result, IrSourceMode::Own);
                }
            }
        }
        run(&ir)
    })
}

fn with_checked<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        crate::semantic::CheckedProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_prelude(&inputs, SOURCE_LIMITS) else {
        panic!("lowering test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("lowering test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("lowering test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("lowering test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("lowering test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("lowering test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("lowering test source must resolve");
    };
    let outcome = check_semantics(resolved);
    let SemanticOutcome::Complete(checked) = outcome else {
        panic!("lowering test source must check: {outcome:?}");
    };
    run(*checked)
}

fn call_definition(function: &IrFunction, result: IrValueId) -> (u32, &[IrValueId]) {
    function
        .blocks()
        .iter()
        .flat_map(IrBlock::instructions)
        .find_map(|instruction| match instruction {
            IrInstruction::Define {
                result: defined,
                operation:
                    IrOperation::Call {
                        function,
                        arguments,
                    },
                ..
            } if *defined == result => Some((*function, arguments.as_slice())),
            _ => None,
        })
        .expect("source metadata must name an actual IR call")
}

const RELEASE_INVENTORY_SOURCE: &[u8] = br#"fn observe['s](cell: &Box<'s, u64>, witness: &Box<'s, u64>) -> result: own unit pure {
  return unit;
}

fn pair['l, 'r](left: &Box<'l, u64>, right: &Box<'r, u64>, left_witness: &Box<'l, u64>, right_witness: &Box<'r, u64>) -> result: own unit pure {
  return unit;
}

fn pass<T: linear>(value: own T) -> result: own T pure {
  return move value;
}

fn relay['s](cell: own Box<'s, u64>) -> result: own Box<'s, u64> pure {
  return pass::<Box<'s, u64>>(value: move cell);
}

fn recurse['s](cell: &Box<'s, u64>, witness: &Box<'s, u64>, again: own Bool) -> result: own unit pure {
  if again {
    let stop = False();
    return recurse(cell: cell, witness: witness, again: stop);
  }
  return unit;
}

fn borrow_scalar(value: &u64) -> result: own unit pure {
  return unit;
}

fn main['heap](heap: own Heap<'heap>) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region 'a {
    let first = arena_frame::<8, 8, 'a>();
    region 'b {
      let second = arena_frame::<8, 8, 'b>();
      region {
        match heap_box(store: &uniq heap, value: 1_u64) {
          Err(error: back) => {
            return exit_status(code: 70_u8);
          }
          Ok(value: general) => {
            match arena_box(store: &uniq first, value: 2_u64) {
              Err(error: back) => {
                return exit_status(code: 70_u8);
              }
              Ok(value: extent_a) => {
                match arena_box(store: &uniq second, value: 3_u64) {
                  Err(error: back) => {
                    return exit_status(code: 70_u8);
                  }
                  Ok(value: extent_b) => {
                    let general_ready = relay(cell: move general);
                    let extent_ready = relay(cell: move extent_a);
                    region {
                      let again = True();
                      observe(cell: &general_ready, witness: &general_ready);
                      observe(cell: &extent_ready, witness: &extent_ready);
                      observe(cell: &extent_b, witness: &extent_b);
                      pair(left: &general_ready, right: &extent_ready, left_witness: &general_ready, right_witness: &extent_ready);
                      pair(left: &extent_ready, right: &general_ready, left_witness: &extent_ready, right_witness: &general_ready);
                      recurse(cell: &general_ready, witness: &general_ready, again: again);
                      recurse(cell: &extent_b, witness: &extent_b, again: again);
                      return exit_status(code: 0_u8);
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;

#[test]
fn physical_call_inventory_reuses_classes_and_keeps_independent_axes() {
    use crate::semantic::CheckedReleaseClass::{Extent, General};

    with_checked(RELEASE_INVENTORY_SOURCE, |checked| {
        let plan = super::specialize::PhysicalFunctions::build(&checked.data)
            .expect("accepted call inventory must close");
        let variants = |name: &str| {
            let source = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .expect("named source function")
                .id;
            plan.variants
                .iter()
                .filter(|variant| variant.source == source)
                .map(|variant| {
                    variant
                        .releases
                        .iter()
                        .map(|(_, class)| *class)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(variants("observe"), [vec![General], vec![Extent]]);
        assert_eq!(
            variants("pair"),
            [
                vec![General, General],
                vec![General, Extent],
                vec![Extent, General]
            ],
            "ordinary callable definitions retain their default physical ABI as well as the two independently instantiated call environments"
        );
        assert_eq!(
            variants("borrow_scalar"),
            [vec![]],
            "loan-only regions are erased"
        );
        let main = plan
            .variants
            .iter()
            .find(|variant| checked.data.functions[variant.source.0 as usize].name == "main")
            .expect("ordinary main");
        let observe = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "observe")
            .expect("observe declaration")
            .id;
        let calls = main
            .calls
            .iter()
            .filter_map(|(_, target)| {
                (plan.variants[*target as usize].source == observe).then_some(*target)
            })
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 3);
        assert_ne!(calls[0], calls[1]);
        assert_eq!(
            calls[1], calls[2],
            "distinct extent brands share physical code"
        );
        assert!(
            plan.variants
                .windows(2)
                .all(|pair| pair[0].source.0 <= pair[1].source.0)
        );
    });
}

#[test]
fn physical_call_inventory_closes_captured_regions_and_recursive_edges() {
    use crate::semantic::CheckedReleaseClass::{Extent, General};

    with_checked(RELEASE_INVENTORY_SOURCE, |checked| {
        let plan = super::specialize::PhysicalFunctions::build(&checked.data)
            .expect("accepted recursive call inventory must close");
        let pass = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "pass")
            .expect("concrete generic pass instance");
        assert!(
            pass.region_parameters.is_empty(),
            "the store is captured inside T"
        );
        let classes = plan
            .variants
            .iter()
            .filter(|variant| variant.source == pass.id)
            .map(|variant| {
                variant
                    .releases
                    .iter()
                    .map(|(_, class)| *class)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(classes, [vec![General], vec![Extent]]);
        let recursive = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "recurse")
            .expect("recursive source declaration")
            .id;
        let variants = plan
            .variants
            .iter()
            .enumerate()
            .filter(|(_, variant)| variant.source == recursive)
            .collect::<Vec<_>>();
        assert_eq!(variants.len(), 2);
        for (index, variant) in variants {
            assert_eq!(variant.calls.len(), 1);
            assert_eq!(
                variant.calls[0].1 as usize, index,
                "recursive calls retain the class environment"
            );
        }
        for variant in &plan.variants {
            assert!(
                variant
                    .calls
                    .iter()
                    .all(|(_, target)| (*target as usize) < plan.variants.len())
            );
        }
    });
}

fn source_call<'program>(
    program: &'program IrProgram<'_, '_, '_>,
    caller: &str,
    callee: &str,
) -> (&'program IrSourceCall, &'program [IrValueId]) {
    let caller = function(program, caller);
    caller
        .source_calls
        .iter()
        .find_map(|source| {
            let (target, arguments) = call_definition(caller, source.result);
            (program.functions()[target as usize].name() == callee).then_some((source, arguments))
        })
        .expect("the source call must have retained use metadata")
}

fn function<'program>(
    program: &'program IrProgram<'_, '_, '_>,
    name: &str,
) -> &'program IrFunction {
    program
        .functions()
        .iter()
        .find(|function| function.name() == name)
        .unwrap_or_else(|| panic!("lowered program must contain {name}"))
}

/// The one block of a straight-line function.
fn only_block(function: &IrFunction) -> &IrBlock {
    let [block] = function.blocks() else {
        panic!("expected one block, got {}", function.blocks().len());
    };
    block
}

fn return_drops(function: &IrFunction) -> &[IrDrop] {
    let IrTerminator::Return { drops, .. } = only_block(function).terminator() else {
        panic!("expected a return terminator");
    };
    drops
}

#[test]
fn source_signature_modes_distinguish_identical_descriptor_representations() {
    let source = format!(
        "fn owned(value: own buffer<u8>) -> result: own unit pure {{\n  return unit;\n}}\n\nfn shared(value: &buffer<u8>) -> result: own unit pure {{\n  return unit;\n}}\n\nfn unique(value: &uniq buffer<u8>) -> result: own unit pure {{\n  return unit;\n}}\n\n{PLAIN_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let owned = function(program, "owned");
        let representation = owned.parameters()[0].1;
        assert!(matches!(representation, IrType::Buffer { .. }));
        for (name, mode) in [
            ("owned", IrSourceMode::Own),
            ("shared", IrSourceMode::Shared),
            ("unique", IrSourceMode::Unique),
        ] {
            let lowered = function(program, name);
            assert_eq!(lowered.parameters()[0].1, representation);
            let signature = lowered
                .source_signature
                .as_ref()
                .expect("a source signature");
            assert_eq!(signature.parameters, [mode]);
            assert_eq!(signature.result, IrSourceMode::Own);
            assert_eq!(
                return_drops(lowered).len(),
                usize::from(mode == IrSourceMode::Own),
                "equal descriptor types retain different release responsibilities"
            );
        }
    });
}

#[test]
fn source_signature_modes_retain_borrow_results_without_inventing_ownership() {
    let source = format!(
        "fn owned(value: own u64) -> result: own u64 pure {{\n  return value;\n}}\n\nfn shared['r](value: &'r u64) -> result: &'r u64 pure {{\n  return value;\n}}\n\nfn unique['r](value: &uniq 'r u64) -> result: &uniq 'r u64 pure {{\n  return move value;\n}}\n\n{PLAIN_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        for (name, mode) in [
            ("owned", IrSourceMode::Own),
            ("shared", IrSourceMode::Shared),
            ("unique", IrSourceMode::Unique),
        ] {
            let lowered = function(program, name);
            let signature = lowered
                .source_signature
                .as_ref()
                .expect("a source signature");
            assert_eq!(signature.parameters, [mode]);
            assert_eq!(signature.result, mode);
            assert_eq!(
                matches!(lowered.result(), IrType::Address(_)),
                mode != IrSourceMode::Own
            );
        }
        assert_eq!(
            function(program, "shared").result(),
            function(program, "unique").result(),
            "the same address representation does not distinguish loan strength"
        );
    });
}

#[test]
fn source_signature_modes_are_not_invented_for_synthesized_functions() {
    let source = format!(
        "fn folded(lo: own u64, hi: own u64) -> result: own u64 pure {{\n  let total = 0_u64;\n  for @points (i in lo..hi) {{\n    set total = total +wrap i;\n  }}\n  return total;\n}}\n\n{PLAIN_ENTRY}"
    );
    with_ir_mode(source.as_bytes(), OverlapLowering::On, |program| {
        let generated = program
            .functions()
            .iter()
            .filter(|function| function.synthesis().is_some())
            .collect::<Vec<_>>();
        assert!(
            !generated.is_empty(),
            "the reduction must exercise synthesized signatures: {:?}",
            program.actualization_ledger()
        );
        for function in generated {
            assert_eq!(function.source_signature, None);
        }
        let source = function(program, "folded");
        assert_eq!(
            source
                .source_signature
                .as_ref()
                .expect("a source signature")
                .parameters,
            [IrSourceMode::Own, IrSourceMode::Own]
        );
    });
}

#[test]
fn source_call_uses_distinguish_borrow_and_consume_of_the_same_ir_value() {
    let source = format!(
        "fn inspect(value: &buffer<u8>) -> result: own u64 reads(value) {{\n  return len_of(deref(value));\n}}\n\nfn consume(value: own buffer<u8>) -> result: own u64 reads(value) {{\n  return len_of(value);\n}}\n\nfn run() -> result: own u64 pure {{\n  let data = buffer_new(2_u64, 7_u8);\n  region {{\n    let before = inspect(value: &data);\n  }}\n  let after = consume(value: move data);\n  return after;\n}}\n\n{PLAIN_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let (borrow, borrowed_values) = source_call(program, "run", "inspect");
        let (consume, consumed_values) = source_call(program, "run", "consume");
        assert_eq!(borrowed_values, consumed_values);
        assert_ne!(borrow.result, consume.result);
        assert_eq!(borrow.arguments, [IrSourceArgument::Borrow]);
        assert_eq!(
            consume.arguments,
            [IrSourceArgument::Binding { consume_root: true }]
        );
    });
}

#[test]
fn source_call_uses_keep_unique_holder_transfer_distinct_from_owning_storage() {
    let source = format!(
        "fn forward['r](value: &uniq 'r u64) -> result: &uniq 'r u64 pure {{\n  return move value;\n}}\n\nfn relay['r](value: &uniq 'r u64) -> result: &uniq 'r u64 pure {{\n  let next = forward(value: move value);\n  return move next;\n}}\n\n{PLAIN_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let (call, _) = source_call(program, "relay", "forward");
        assert_eq!(
            call.arguments,
            [IrSourceArgument::Binding { consume_root: true }]
        );
        assert_eq!(call.returned_borrow_argument, Some(0));
        let signature = function(program, "forward")
            .source_signature
            .as_ref()
            .expect("a source signature");
        assert_eq!(signature.parameters, [IrSourceMode::Unique]);
        assert_eq!(signature.result, IrSourceMode::Unique);
    });
}

#[test]
fn source_call_uses_retain_projected_root_consumption() {
    let source = format!(
        "struct Packet {{\n  first: box<u64>;\n  second: box<u64>;\n}}\n\nfn consume(value: own box<u64>) -> result: own unit pure {{\n  return unit;\n}}\n\nfn run() -> result: own unit pure {{\n  let first = box_new(3_u64);\n  let second = box_new(5_u64);\n  let packet = Packet(first: move first, second: move second);\n  return consume(value: move packet.first);\n}}\n\n{PLAIN_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let (call, _) = source_call(program, "run", "consume");
        assert_eq!(
            call.arguments,
            [IrSourceArgument::Projection { consume_root: true }]
        );
        assert_eq!(call.returned_borrow_argument, None);
    });
}

#[test]
fn source_call_uses_retain_the_actual_indexed_borrow_candidate() {
    let source = br#"struct Row {
  value: u64;
}

fn select['r](stamp: own u64, value: &'r Row) -> result: &'r Row pure {
  return value;
}

fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Row, 2>();
  let first = Row(value: 3_u64);
  region {
    place_back(vector: &uniq empty, value: move first);
  }
  let prefix = move empty;
  let second = Row(value: 5_u64);
  region {
    place_back(vector: &uniq prefix, value: move second);
  }
  let rows = move prefix;
  region {
    let chosen = select(stamp: 7_u64, value: &rows[1_u64]);
    let observed = deref(chosen).value;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let (call, arguments) = source_call(program, "main", "select");
        assert_eq!(call.returned_borrow_argument, Some(1));
        assert_eq!(
            call.arguments,
            [IrSourceArgument::Value, IrSourceArgument::Borrow]
        );
        assert!(
            function(program, "main")
                .blocks()
                .iter()
                .flat_map(IrBlock::instructions)
                .any(|instruction| matches!(
                    instruction,
                    IrInstruction::Define {
                        result,
                        operation: IrOperation::ProjectAddress {
                            projection: super::IrPlaceProjection::RunElement { .. },
                            ..
                        },
                        ..
                    } if *result == arguments[1]
                ))
        );
    });
}

#[test]
fn counted_range_cfg_emits_with_distinct_header_update_and_exit_interfaces() {
    let source = br#"fn count() -> result: own u64 pure {
  let total = 0_u64;
  for @items (i in 18446744073709551614_u64..18446744073709551615_u64) {
    set total = i;
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let counted = function(program, "count");
        if let Err(error) = crate::emit_llvm(program) {
            panic!("counted IR must emit: {error:?}\n{counted:#?}");
        }
        assert!(counted.blocks().len() >= 6);
        assert!(counted.blocks().iter().any(|block| {
            matches!(
                block.terminator(),
                IrTerminator::Match {
                    enum_type: super::IrEnumType::Bool,
                    ..
                }
            )
        }));
        let hidden_updates = counted
            .blocks()
            .iter()
            .flat_map(IrBlock::instructions)
            .filter(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::Define {
                        operation: IrOperation::Integer {
                            operation: IrIntegerOperation::AddWrap,
                            operand_type: IrType::Integer {
                                width: 64,
                                signed: false,
                            },
                            ..
                        },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            hidden_updates, 1,
            "MAX-1..MAX must retain exactly the compiler-owned unit update"
        );
    });
}

#[test]
fn counted_break_and_return_edges_do_not_enter_the_hidden_update() {
    let source = br#"fn leave_by_break(stop: own Bool) -> result: own u64 pure {
  for @scan (i in 0_u64..2_u64) {
    if stop {
      break @scan;
    }
  }
  return 7_u64;
}

fn leave_by_return(stop: own Bool) -> result: own u64 pure {
  for @scan (i in 0_u64..2_u64) {
    if stop {
      return 9_u64;
    }
  }
  return 7_u64;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let hidden_update_count = |function: &IrFunction| {
            function
                .blocks()
                .iter()
                .flat_map(IrBlock::instructions)
                .filter(|instruction| {
                    matches!(
                        instruction,
                        IrInstruction::Define {
                            operation: IrOperation::Integer {
                                operation: IrIntegerOperation::AddWrap,
                                operand_type: IrType::Integer {
                                    width: 64,
                                    signed: false,
                                },
                                ..
                            },
                            ..
                        }
                    )
                })
                .count()
        };

        let breaking = function(program, "leave_by_break");
        assert_eq!(
            hidden_update_count(breaking),
            1,
            "the normal fallthrough keeps one hidden update"
        );
        let exit_blocks = breaking
            .blocks()
            .iter()
            .enumerate()
            .filter_map(|(index, block)| {
                matches!(block.terminator(), IrTerminator::Return { .. }).then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            exit_blocks.len(),
            1,
            "the break fixture has one final return"
        );
        let exit = exit_blocks[0];
        let jumps_to_exit = breaking
            .blocks()
            .iter()
            .filter(|block| {
                matches!(
                    block.terminator(),
                    IrTerminator::Jump { target, .. } if target.index() == exit
                )
            })
            .count();
        assert_eq!(
            jumps_to_exit, 2,
            "the false header and break edge must reach the exit directly"
        );

        let returning = function(program, "leave_by_return");
        assert_eq!(
            hidden_update_count(returning),
            1,
            "the non-returning branch keeps one hidden update"
        );
        let returns = returning
            .blocks()
            .iter()
            .filter(|block| matches!(block.terminator(), IrTerminator::Return { .. }))
            .count();
        assert_eq!(
            returns, 2,
            "the body return must remain a return edge beside the false-header exit"
        );
    });
}

#[test]
fn counted_range_carries_one_stable_binder_address_for_body_local_shared_borrows() {
    let source = br#"fn count() -> result: own u64 pure {
  let total = 0_u64;
  let upper = 2_u64;
  for @items (i in 0_u64..upper) {
    region {
      let held = &i;
      let seen = deref(held);
      set total = total +wrap seen;
    }
    set upper = 0_u64;
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let counted = function(program, "count");
        if let Err(error) = crate::emit_llvm(program) {
            panic!("addressed counted binder IR must emit: {error:?}\n{counted:#?}");
        }
        let address_count = counted
            .blocks()
            .iter()
            .flat_map(IrBlock::instructions)
            .filter(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::Define {
                        operation: IrOperation::AddressOf { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(address_count, 1, "the binder storage is allocated once");
    });
}

#[test]
fn nested_counted_breaks_keep_each_exit_interface_local_to_its_range() {
    let source = br#"fn count() -> result: own u64 pure {
  let total = 0_u64;
  for @outer (i in 0_u64..4_u64) {
    for @inner (j in 0_u64..4_u64) {
      set total = total +wrap 1_u64;
      break @inner;
    }
    if i == 1_u64 {
      break @outer;
    }
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let counted = function(program, "count");
        if let Err(error) = crate::emit_llvm(program) {
            panic!("nested counted IR must emit: {error:?}\n{counted:#?}");
        }
    });
}

#[test]
fn addressed_cleanup_keeps_places_instead_of_whole_owner_snapshots() {
    let source = format!(
        r#"struct Holder {{
  bytes: buffer<u8>;
  stamp: u64;
}}

fn touch(value: &uniq Holder) -> result: own unit writes(value.stamp) {{
  set deref(value).stamp = 41_u64;
  return unit;
}}

fn release_holder(value: own Holder, early: own Bool) -> result: own unit writes(value, value.stamp) {{
  region {{
    touch(value: &uniq value);
  }}
  if early {{
    dispose value;
    return unit;
  }}
  return unit;
}}

{PLAIN_ENTRY}"#
    );
    with_ir(source.as_bytes(), |program| {
        let function = function(program, "release_holder");
        let mut groups = Vec::new();
        for block in function.blocks() {
            for instruction in block.instructions() {
                assert!(
                    !matches!(
                        instruction,
                        IrInstruction::Define {
                            operation: IrOperation::Load {
                                referent: super::IrAddressed::Nominal(_),
                                ..
                            },
                            ..
                        }
                    ),
                    "cleanup must not capture a second aggregate owner"
                );
                if let IrInstruction::Drops(drops) = instruction {
                    groups.push(drops.as_slice());
                }
            }
            if let IrTerminator::Return { drops, .. } = block.terminator()
                && !drops.is_empty()
            {
                groups.push(drops.as_slice());
            }
        }
        assert_eq!(groups.len(), 2, "explicit disposal and normal scope exit");
        for drops in groups {
            let [field, owner] = drops else {
                panic!("field release then owner node");
            };
            assert!(matches!(field.ty(), IrType::Buffer { .. }));
            assert!(matches!(owner.ty(), IrType::Nominal(_)));
            for drop in drops {
                let IrDropSubject::Place(address) = drop.subject() else {
                    panic!("an addressed owner keeps its cleanup place");
                };
                let Some(IrType::Address(referent)) = function.value_type(address) else {
                    panic!("cleanup place must retain its content type");
                };
                assert_eq!(referent.ty(), drop.ty());
            }
        }
    });
}

#[test]
fn an_unused_state_writing_call_reaches_ir() {
    let source = format!(
        "struct Pair {{\n  left: u64;\n}}\n\n\
         fn mutate(pair: &uniq Pair) -> result: own unit writes(pair.left) {{\n  \
         set deref(pair).left = 1_u64;\n  return unit;\n}}\n\n\
         fn wrapper(pair: &uniq Pair) -> result: own unit writes(pair.left) {{\n  \
         mutate(pair: move pair);\n  return unit;\n}}\n\n\
         {PLAIN_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let wrapper = function(program, "wrapper");
        assert!(wrapper.blocks().iter().any(|block| {
            block.instructions().iter().any(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::Define {
                        operation: IrOperation::Call { function: 0, .. },
                        ..
                    }
                )
            })
        }));
    });
}

#[test]
fn ordinary_requires_is_not_lowered_as_a_callee_prologue() {
    let source = br#"fn bounded(value: own u64) -> result: own u64 pure contract {
  requires value < 8_u64;
} {
  return value;
}

fn main() -> status: own ExitStatus pure {
  let value = 4_u64;
  let result = bounded(value: value);
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let bounded = function(program, "bounded");
        assert!(
            bounded
                .blocks()
                .iter()
                .flat_map(IrBlock::instructions)
                .next()
                .is_none(),
            "an ordinary requirement is a call-site obligation and contributes no executable callee prologue"
        );
    });
}

#[test]
fn source_proof_is_erased_before_typed_ir() {
    let source = br#"fn plain(left: own u64, left_limit: own u64, middle: own u64, middle_limit: own u64, right: own u64, right_limit: own u64) -> result: own unit pure contract {
  requires left <= left_limit;
  requires middle <= middle_limit;
  requires right <= right_limit;
} {
  return unit;
}

fn prove_only(left: own u64, left_limit: own u64, middle: own u64, middle_limit: own u64, right: own u64, right_limit: own u64) -> result: own unit pure contract {
  requires left <= left_limit;
  requires middle <= middle_limit;
  requires right <= right_limit;
} {
  invariant combined: left + middle + right <= left_limit + middle_limit + right_limit {
    use (left <= left_limit);
    use (middle <= middle_limit);
    use (right <= right_limit);
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let plain = function(program, "plain");
        let proved = function(program, "prove_only");
        assert_eq!(
            proved.parameters(),
            plain.parameters(),
            "the erased proof contributes no parameter or value dependency"
        );
        assert_eq!(proved.result(), plain.result());
        assert_eq!(
            proved.blocks(),
            plain.blocks(),
            "PRF-1 contributes no instruction, effect, branch, runtime check, or terminator change"
        );
        assert_eq!(proved.overlaps(), plain.overlaps());
    });
}

#[test]
fn buffer_allocations_lower_the_source_proved_length_ceiling_into_target_obligations() {
    let source = br#"fn allocate(n: own u64) -> result: own unit pure contract {
  requires n <= 1000_u64;
} {
  let filled = buffer_new(n, 7_u16);
  let vacant = buffer_vacant::<u16>(n);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  allocate(n: 4_u64);
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let allocate = function(program, "allocate");
        let bounds = allocate
            .blocks()
            .iter()
            .flat_map(IrBlock::instructions)
            .filter_map(|instruction| {
                let IrInstruction::Define { operation, .. } = instruction else {
                    return None;
                };
                match operation {
                    IrOperation::BufferFill { target_domains, .. }
                    | IrOperation::BufferVacant { target_domains, .. } => {
                        Some(target_domains.source_length_upper_bound())
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(bounds, vec![1000, 1000]);
    });
}

#[test]
fn an_uninhabited_function_keeps_its_abi_and_lowers_to_one_unreachable_block() {
    let source = br#"fn impossible(value: own i32) -> out: own i32 pure contract {
  requires value == 0_i32;
  requires value != 0_i32;
} {
  return value;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let impossible = function(program, "impossible");
        assert_eq!(impossible.parameters().len(), 1);
        assert_eq!(
            impossible.result(),
            IrType::Integer {
                width: 32,
                signed: true,
            }
        );
        let [entry] = impossible.blocks() else {
            panic!("an uninhabited function must lower to exactly one block");
        };
        assert!(entry.instructions().is_empty());
        assert_eq!(entry.terminator(), &IrTerminator::Unreachable);
    });
}

#[test]
fn physical_call_inventory_omits_proof_closed_body_edges() {
    let source = br#"fn child() -> result: own unit pure {
  return unit;
}

fn impossible(value: own i32) -> result: own unit pure contract {
  requires value == 0_i32;
  requires value != 0_i32;
} {
  child();
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_checked(source, |checked| {
        let impossible = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "impossible")
            .expect("impossible declaration");
        assert!(matches!(
            impossible.body_disposition,
            crate::semantic::CheckedBodyDisposition::Uninhabited { .. }
        ));
        assert!(impossible.body.iter().flatten().any(|statement| matches!(
            statement,
            crate::semantic::CheckedStatement::Evaluate(
                crate::semantic::CheckedExpression::UserCall { .. }
            )
        )));
        let plan = super::specialize::PhysicalFunctions::build(&checked.data)
            .expect("proof-closed functions retain their physical signature");
        let variant = plan
            .variants
            .iter()
            .find(|variant| variant.source == impossible.id)
            .expect("unreferenced source definition still emitted");
        assert!(
            variant.calls.is_empty(),
            "a proof-closed body has no executable calls"
        );
        assert!(variant.releases.is_empty());
    });
}

#[test]
fn a_buffer_release_retains_its_owned_storage_type() {
    // STOR-3 release names ordinary owned storage; opaque drops are empty.
    with_ir(
        b"fn drop_buffer(values: own buffer<u8>) -> result: own unit pure {\n  return unit;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |program| {
            let [drop] = return_drops(function(program, "drop_buffer")) else {
                panic!("the buffer owner must be released once");
            };
            assert!(matches!(drop.ty(), IrType::Buffer { .. }));
        },
    );
}

/// The recognized byte-walk loop for the wide-probe tests, with `{MIDDLE}`
/// and `{STEP}` varied per case.
fn byte_walk_source(middle: &str, step: &str) -> Vec<u8> {
    format!(
        "fn main() -> status: own ExitStatus pure {{\n  let data = buffer_new(64_u64, 97_u8);\n  let mark = 88_u8;\n  let seen = 0_u64;\n  let stop = len_of(data);\n  let cursor = 0_u64;\n  loop @walk {{\n    let done = cursor >= stop;\n    if done {{\n      break @walk;\n    }}\n    let byte = data[cursor];\n{middle}    set cursor = cursor +wrap {step};\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
    )
    .into_bytes()
}

const NEUTRAL_MIDDLE: &str = "    let newline = byte == 10_u8;\n    if newline {\n      set seen = seen +wrap 1_u64;\n    }\n    let lead = byte == mark;\n    if lead {\n      set seen = seen +wrap 2_u64;\n    }\n";

fn probe_needle_counts(program: &IrProgram<'_, '_, '_>) -> Vec<usize> {
    program
        .functions()
        .iter()
        .flat_map(IrFunction::blocks)
        .flat_map(IrBlock::instructions)
        .filter_map(|instruction| {
            let IrInstruction::Define {
                operation: IrOperation::BufferProbeSkip { needles, .. },
                ..
            } = instruction
            else {
                return None;
            };
            Some(needles.len())
        })
        .collect()
}

#[test]
fn a_recognized_byte_walk_gains_one_wide_probe_with_its_needles() {
    with_ir(&byte_walk_source(NEUTRAL_MIDDLE, "1_u64"), |program| {
        assert_eq!(probe_needle_counts(program), vec![2]);
    });
}

#[test]
fn an_effect_on_the_quiet_path_declines_the_wide_probe() {
    let middle = format!("{NEUTRAL_MIDDLE}    set seen = seen +wrap 1_u64;\n");
    with_ir(&byte_walk_source(&middle, "1_u64"), |program| {
        assert_eq!(probe_needle_counts(program), Vec::<usize>::new());
    });
}

#[test]
fn a_non_single_step_increment_declines_the_wide_probe() {
    with_ir(&byte_walk_source(NEUTRAL_MIDDLE, "2_u64"), |program| {
        assert_eq!(probe_needle_counts(program), Vec::<usize>::new());
    });
}

#[test]
fn a_needle_declared_inside_the_loop_declines_the_wide_probe() {
    let middle = "    let inner_mark = 88_u8;\n    let lead = byte == inner_mark;\n    if lead {\n      set seen = seen +wrap 2_u64;\n    }\n";
    with_ir(&byte_walk_source(middle, "1_u64"), |program| {
        assert_eq!(probe_needle_counts(program), Vec::<usize>::new());
    });
}
