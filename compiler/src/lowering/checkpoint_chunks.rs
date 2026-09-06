//! Keep the original inner loop between cooperative checkpoints.
//!
//! This post-checking experiment recognizes unsigned unit-stride natural loops
//! with an invariant upper bound. It replaces their upper limit by a bounded
//! chunk limit and checks the scheduler only between chunks. Other loops keep
//! the backedge counter. Owned by io-model/SCHEDULER-EXPERIMENT.md; supersede
//! this pass when the checkpoint representation is selected.

use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;

use super::{
    IrBlock, IrBlockId, IrConstant, IrEnumType, IrFunction, IrInstruction, IrIntegerOperation,
    IrMatchTarget, IrOperation, IrTerminator, IrType, IrValueId,
};

const U64: IrType = IrType::Integer {
    width: 64,
    signed: false,
};

pub(crate) struct ChunkedFunction {
    pub(crate) function: IrFunction,
    pub(crate) inner_backedges: HashSet<usize>,
    pub(crate) checkpoints: HashSet<usize>,
}

struct Candidate {
    header: usize,
    latch: usize,
    entry: usize,
    index: IrValueId,
    upper: IrValueId,
    guard: IrValueId,
    exit_tag: u32,
    exit: IrBlockId,
}

/// A failed representation-size conversion declines this optimization. It
/// cannot reject source or change the ordinary backedge-counter fallback.
pub(crate) fn chunk(function: &IrFunction, interval: NonZeroU32) -> Option<ChunkedFunction> {
    // A driven completion pipeline already replaces its source loop edges.
    // Leave that topology to its existing driver and ordinary checkpoints.
    if function.driven_completion_pipeline().is_some() {
        return None;
    }
    let candidates = candidates(function);
    if candidates.is_empty() {
        return None;
    }
    let mut result = ChunkedFunction {
        function: function.clone(),
        inner_backedges: HashSet::new(),
        checkpoints: HashSet::new(),
    };
    for candidate in candidates {
        apply(&mut result, candidate, interval)?;
    }
    Some(result)
}

fn candidates(function: &IrFunction) -> Vec<Candidate> {
    let mut definitions = HashMap::new();
    let mut incoming: HashMap<IrValueId, Vec<IrValueId>> = HashMap::new();
    let mut predecessors = vec![Vec::new(); function.blocks.len()];
    for (block_id, block) in function.blocks.iter().enumerate() {
        for instruction in &block.instructions {
            if let IrInstruction::Define {
                result, operation, ..
            } = instruction
            {
                definitions.insert(*result, operation);
            }
        }
        match &block.terminator {
            IrTerminator::Jump {
                target, arguments, ..
            } => {
                predecessors[target.index()].push(block_id);
                for ((parameter, _), argument) in function.blocks[target.index()]
                    .parameters
                    .iter()
                    .zip(arguments)
                {
                    incoming.entry(*parameter).or_default().push(*argument);
                }
            }
            IrTerminator::Match { targets, .. } => {
                for target in targets {
                    predecessors[target.block.index()].push(block_id);
                }
            }
            IrTerminator::Return { .. } | IrTerminator::Unreachable => {}
        }
    }
    super::control_flow::backedge_sources(&function.blocks)
        .into_iter()
        .filter_map(|latch| {
            let IrTerminator::Jump {
                target,
                arguments: back,
                ..
            } = &function.blocks[latch].terminator
            else {
                return None;
            };
            let header = target.index();
            let entries = &predecessors[header];
            if entries.len() != 2 {
                return None;
            }
            let entry = *entries.iter().find(|&&entry| entry != latch)?;
            let IrTerminator::Jump {
                arguments: initial, ..
            } = &function.blocks[entry].terminator
            else {
                return None;
            };
            let block = &function.blocks[header];
            let IrTerminator::Match {
                scrutinee,
                enum_type: IrEnumType::Bool,
                targets,
            } = &block.terminator
            else {
                return None;
            };
            // The repeated header must contain only the tested bound comparison.
            // Re-entering it at a chunk edge must not duplicate a source effect.
            let [
                IrInstruction::Define {
                    result: guard,
                    operation:
                        IrOperation::Integer {
                            operation,
                            operand_type: U64,
                            arguments,
                        },
                    ..
                },
            ] = block.instructions.as_slice()
            else {
                return None;
            };
            if guard != scrutinee {
                return None;
            }
            let exit_tag = match operation {
                IrIntegerOperation::Less => 0,
                IrIntegerOperation::GreaterEqual => 1,
                IrIntegerOperation::Equal => 1,
                IrIntegerOperation::NotEqual => 0,
                _ => return None,
            };
            let [index, upper] = arguments.as_slice() else {
                return None;
            };
            let index_position = block
                .parameters
                .iter()
                .position(|(value, ty)| value == index && *ty == U64)?;
            let upper_position = block
                .parameters
                .iter()
                .position(|(value, ty)| value == upper && *ty == U64)?;
            // Equality termination also denotes a bounded forward range when its
            // initial index is zero: every u64 upper bound is at least that value.
            // A dynamic start could exceed upper and intentionally wrap around;
            // retain its original counter path rather than changing that loop.
            if matches!(
                operation,
                IrIntegerOperation::Equal | IrIntegerOperation::NotEqual
            ) && !matches!(
                definitions.get(initial.get(index_position)?),
                Some(IrOperation::Constant(IrConstant::Integer {
                    ty: U64,
                    bits: 0
                }))
            ) {
                return None;
            }
            if !equivalent(*back.get(upper_position)?, *upper, &incoming) {
                return None;
            }
            let IrOperation::Integer {
                operation:
                    IrIntegerOperation::AddWrap
                    | IrIntegerOperation::AddExact
                    | IrIntegerOperation::AddDefined,
                operand_type: U64,
                arguments: step,
            } = definitions.get(back.get(index_position)?)?
            else {
                return None;
            };
            let [before, one] = step.as_slice() else {
                return None;
            };
            if !equivalent(*before, *index, &incoming)
                || !matches!(
                    definitions.get(one),
                    Some(IrOperation::Constant(IrConstant::Integer {
                        ty: U64,
                        bits: 1
                    }))
                )
            {
                return None;
            }
            let exit = targets.iter().find(|target| target.tag == exit_tag)?.block;
            // This comparison polarity must actually leave the natural loop.
            // Breaking below the bound and running above it is another loop.
            let mut members = HashSet::from([header]);
            let mut pending = vec![latch];
            while let Some(block) = pending.pop() {
                if members.insert(block) {
                    pending.extend(&predecessors[block]);
                }
            }
            if members.contains(&exit.index()) {
                return None;
            }
            Some(Candidate {
                header,
                latch,
                entry,
                index: *index,
                upper: *upper,
                guard: *guard,
                exit_tag,
                exit,
            })
        })
        .collect()
}

/// Follow block-parameter forwarding only. Every incoming origin must be the
/// requested value. Cycles without that origin do not establish equality.
fn equivalent(
    value: IrValueId,
    wanted: IrValueId,
    incoming: &HashMap<IrValueId, Vec<IrValueId>>,
) -> bool {
    let mut pending = vec![value];
    let mut seen = HashSet::new();
    let mut reached = false;
    while let Some(value) = pending.pop() {
        if value == wanted {
            reached = true;
            continue;
        }
        if !seen.insert(value) {
            continue;
        }
        let Some(origins) = incoming.get(&value) else {
            return false;
        };
        if origins.is_empty() {
            return false;
        }
        pending.extend(origins);
    }
    reached
}

fn value(function: &mut IrFunction, ty: IrType) -> Option<IrValueId> {
    let result = IrValueId(u32::try_from(function.values.len()).ok()?);
    function.values.push(ty);
    Some(result)
}

fn limit(
    function: &mut IrFunction,
    index: IrValueId,
    upper: IrValueId,
    interval: NonZeroU32,
) -> Option<(IrValueId, Vec<IrInstruction>)> {
    let amount = value(function, U64)?;
    let advanced = value(function, U64)?;
    let end = value(function, U64)?;
    Some((
        end,
        vec![
            IrInstruction::Define {
                result: amount,
                ty: U64,
                operation: IrOperation::Constant(IrConstant::Integer {
                    ty: U64,
                    bits: u64::from(interval.get()),
                }),
            },
            IrInstruction::Define {
                result: advanced,
                ty: U64,
                operation: IrOperation::Integer {
                    operation: IrIntegerOperation::AddSaturating,
                    operand_type: U64,
                    arguments: vec![index, amount],
                },
            },
            IrInstruction::Define {
                result: end,
                ty: U64,
                operation: IrOperation::Integer {
                    operation: IrIntegerOperation::Minimum,
                    operand_type: U64,
                    arguments: vec![advanced, upper],
                },
            },
        ],
    ))
}

fn apply(result: &mut ChunkedFunction, candidate: Candidate, interval: NonZeroU32) -> Option<()> {
    let function = &mut result.function;
    let Candidate {
        header,
        latch,
        entry,
        index,
        upper,
        guard,
        exit_tag,
        exit,
    } = candidate;
    let chunk_end = value(function, U64)?;
    let more = value(function, IrType::Bool)?;
    let original_parameters = function.blocks[header].parameters.clone();
    let resume_arguments: Vec<_> = original_parameters
        .iter()
        .map(|(value, _)| *value)
        .collect();
    let mut outer_parameters = Vec::new();
    for (_, ty) in &original_parameters {
        outer_parameters.push((value(function, *ty)?, *ty));
    }
    let outer_index = outer_parameters[original_parameters
        .iter()
        .position(|(value, _)| *value == index)?]
    .0;
    let outer_upper = outer_parameters[original_parameters
        .iter()
        .position(|(value, _)| *value == upper)?]
    .0;
    let (next_limit, next_chunk) = limit(function, outer_index, outer_upper, interval)?;
    let mut inner_arguments: Vec<_> = outer_parameters.iter().map(|(value, _)| *value).collect();
    inner_arguments.push(next_limit);
    let outer = IrBlockId(u32::try_from(function.blocks.len()).ok()?);
    let test = IrBlockId(outer.0.checked_add(1)?);
    let resume = IrBlockId(test.0.checked_add(1)?);

    // Give the chunk loop its own header. Reusing the source header for both
    // backedges lets LLVM form a loop around the checkpoint path instead of
    // the computation; even the no-checkpoint execution then loses ordinary
    // inner-loop optimizations. The original latch now has exactly one entry
    // from this outer header, with a chunk-invariant limit.
    function.blocks[header].parameters.push((chunk_end, U64));
    let [
        IrInstruction::Define {
            result: found,
            operation: IrOperation::Integer { arguments, .. },
            ..
        },
    ] = function.blocks[header].instructions.as_mut_slice()
    else {
        return None;
    };
    if *found != guard {
        return None;
    }
    arguments[1] = chunk_end;
    let IrTerminator::Match { targets, .. } = &mut function.blocks[header].terminator else {
        return None;
    };
    targets
        .iter_mut()
        .find(|target| target.tag == exit_tag)?
        .block = test;
    let IrTerminator::Jump { target, .. } = &mut function.blocks[entry].terminator else {
        return None;
    };
    *target = outer;
    let IrTerminator::Jump { arguments, .. } = &mut function.blocks[latch].terminator else {
        return None;
    };
    arguments.push(chunk_end);

    function.blocks.push(IrBlock {
        parameters: outer_parameters,
        instructions: next_chunk,
        terminator: IrTerminator::Jump {
            target: IrBlockId(u32::try_from(header).ok()?),
            arguments: inner_arguments,
            drops: Vec::new(),
        },
    });
    function.blocks.push(IrBlock {
        parameters: Vec::new(),
        instructions: vec![IrInstruction::Define {
            result: more,
            ty: IrType::Bool,
            operation: IrOperation::Integer {
                operation: IrIntegerOperation::Less,
                operand_type: U64,
                arguments: vec![index, upper],
            },
        }],
        terminator: IrTerminator::Match {
            scrutinee: more,
            enum_type: IrEnumType::Bool,
            targets: vec![
                IrMatchTarget {
                    tag: 1,
                    block: resume,
                },
                IrMatchTarget {
                    tag: 0,
                    block: exit,
                },
            ],
        },
    });
    function.blocks.push(IrBlock {
        parameters: Vec::new(),
        instructions: Vec::new(),
        terminator: IrTerminator::Jump {
            target: outer,
            arguments: resume_arguments,
            drops: Vec::new(),
        },
    });
    result.inner_backedges.insert(latch);
    result.checkpoints.insert(resume.index());
    Some(())
}
