use crate::semantic::{BindingId, CheckedDrop, CheckedLoopId, CheckedStatement};
use crate::{IrBlockId, IrTerminator, IrValueId, LoweringFailure};

use super::{GiveTarget, IrBuilder};

#[derive(Clone)]
pub(super) struct LoopTarget {
    pub(super) id: CheckedLoopId,
    pub(super) block: IrBlockId,
    pub(super) carried_bindings: Vec<BindingId>,
}

impl IrBuilder<'_> {
    pub(super) fn lower_loop(
        &mut self,
        id: CheckedLoopId,
        body: &[CheckedStatement],
        backedge_drops: &[CheckedDrop],
        give_target: Option<GiveTarget>,
    ) -> Result<(), LoweringFailure> {
        let base_bindings = self.bindings.clone();
        let mut carried_bindings = base_bindings.keys().copied().collect::<Vec<_>>();
        carried_bindings.sort_by_key(|binding| binding.0);
        let parameter_types = carried_bindings
            .iter()
            .map(|binding| {
                base_bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)
                    .and_then(|value| self.value_type(value))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (header, header_parameters) = self.new_block(&parameter_types)?;
        let (exit, exit_parameters) = self.new_block(&parameter_types)?;
        let entry_arguments = self.binding_values(&carried_bindings)?;
        self.terminate(IrTerminator::Jump {
            target: header,
            arguments: entry_arguments,
            drops: Vec::new(),
        })?;

        self.current = Some(header);
        self.bindings = base_bindings.clone();
        self.bind_parameters(&carried_bindings, &header_parameters)?;
        self.loops.push(LoopTarget {
            id,
            block: exit,
            carried_bindings: carried_bindings.clone(),
        });
        if backedge_drops.is_empty() {
            self.emit_probe_skip_if_recognized(id, body, header, &carried_bindings)?;
        }
        self.lower_statements(body, give_target)?;
        if self.current.is_some() {
            let arguments = self.binding_values(&carried_bindings)?;
            let drops = self.lower_drops(backedge_drops)?;
            self.terminate(IrTerminator::Jump {
                target: header,
                arguments,
                drops,
            })?;
        }
        let Some(target) = self.loops.pop() else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        if target.id != id || target.block != exit {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }

        self.current = Some(exit);
        self.bindings = base_bindings;
        self.bind_parameters(&carried_bindings, &exit_parameters)
    }

    fn bind_parameters(
        &mut self,
        bindings: &[BindingId],
        parameters: &[IrValueId],
    ) -> Result<(), LoweringFailure> {
        if bindings.len() != parameters.len() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        for (binding, value) in bindings.iter().zip(parameters) {
            if self.bindings.insert(*binding, *value).is_none() {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        }
        Ok(())
    }
}
