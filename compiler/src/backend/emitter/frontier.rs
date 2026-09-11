//! Private same-signature layers for opt-in recursive compute grain experiments.
//! Retire with that control; ordinary calls and runtime interfaces stay shared.

use std::collections::HashSet;

use crate::{IrInstruction, IrOperation, IrProgram};

use super::parallel::sequential_clone_symbol;
use super::source_symbol;

/// Recognize a private parallel layer for post-codegen stack attribution.
pub(crate) fn is_recursive_frontier_symbol(symbol: &str) -> bool {
    symbol
        .strip_prefix("wf__par_frontier_")
        .and_then(|suffix| suffix.split_once('_'))
        .is_some_and(|(level, name)| {
            level.parse::<std::num::NonZeroU8>().is_ok() && !name.is_empty()
        })
}

#[derive(Clone, Copy)]
pub(super) struct Layer {
    component: usize,
    level: u8,
}

#[derive(Default)]
pub(super) struct RecursiveFrontiers {
    levels: u8,
    component_of: Vec<Option<usize>>,
}

impl RecursiveFrontiers {
    pub(super) fn new(program: &IrProgram<'_, '_, '_>, clones: &HashSet<u32>) -> Self {
        let Some(levels) = program.recursive_compute_frontier() else {
            return Self::default();
        };
        let functions = program.functions();
        let mut edges = vec![Vec::new(); functions.len()];
        for (ordinal, function) in functions.iter().enumerate() {
            for instruction in function.blocks().iter().flat_map(|b| b.instructions()) {
                match instruction {
                    IrInstruction::Define {
                        operation: IrOperation::Call { function, .. },
                        ..
                    } => {
                        edges[ordinal].push(*function as usize);
                    }
                    IrInstruction::Define {
                        operation:
                            IrOperation::LoopSplit {
                                splitter, chunk, ..
                            },
                        ..
                    } => {
                        edges[ordinal].extend([*splitter as usize, *chunk as usize]);
                    }
                    _ => {}
                }
            }
        }
        let mut component_of = vec![None; functions.len()];
        for (id, component) in super::super::graph::components(&edges)
            .into_iter()
            .enumerate()
        {
            let cyclic = component.len() > 1 || edges[component[0]].contains(&component[0]);
            let ordinary = component.iter().all(|&ordinal| {
                let function = &functions[ordinal];
                u32::try_from(ordinal).is_ok_and(|i| clones.contains(&i))
                    && !function.target_action().may_suspend()
                    && function.completion_pipeline().is_none()
                    && function.synthesis().is_none()
            });
            if cyclic && ordinary {
                for ordinal in component {
                    component_of[ordinal] = Some(id);
                }
            }
        }
        Self {
            levels: levels.get(),
            component_of,
        }
    }

    pub(super) fn is_used(&self) -> bool {
        self.component_of.iter().any(Option::is_some)
    }

    pub(super) fn levels(&self) -> u8 {
        self.levels
    }

    pub(super) fn layer(&self, ordinal: usize, level: u8) -> Option<Layer> {
        Some(Layer {
            component: self.component_of.get(ordinal).copied().flatten()?,
            level,
        })
    }

    pub(super) fn definition(name: &str, layer: Layer) -> String {
        if layer.level == 0 {
            source_symbol(name)
        } else {
            format!("wf__par_frontier_{}_{}", layer.level, name)
        }
    }

    pub(super) fn callee(&self, ordinal: u32, name: &str, layer: Layer) -> String {
        if self.component_of.get(ordinal as usize).copied().flatten() != Some(layer.component) {
            return source_symbol(name);
        }
        if layer.level + 1 == self.levels {
            sequential_clone_symbol(name)
        } else {
            Self::definition(
                name,
                Layer {
                    level: layer.level + 1,
                    ..layer
                },
            )
        }
    }
}
