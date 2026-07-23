use crate::SemanticCompilerFailure;

use super::super::model::{CheckedNominalKind, CheckedType};
use super::{CheckStop, Checker};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn drop_paths(
        &self,
        ty: CheckedType,
        fields: Vec<u32>,
    ) -> Result<Vec<(Vec<u32>, CheckedType)>, CheckStop> {
        let mut drops = Vec::new();
        let mut pending = vec![(ty, fields, false)];
        while let Some((current, path, postorder)) = pending.pop() {
            if postorder {
                drops.push((path, current));
                continue;
            }
            match current {
                CheckedType::Unit
                | CheckedType::Bool
                | CheckedType::Integer(_)
                | CheckedType::Float(_)
                | CheckedType::GenericInt(_)
                | CheckedType::Generic(_) => {}
                CheckedType::Array { .. }
                | CheckedType::Slice { .. }
                | CheckedType::Buffer { .. } => {
                    drops.push((path, current));
                }
                CheckedType::Nominal(id) => {
                    let nominal = self.nominal(id)?;
                    if nominal.is_copy() {
                        continue;
                    }
                    match &nominal.kind {
                        CheckedNominalKind::Struct { fields } => {
                            pending.push((current, path.clone(), true));
                            for (index, field) in fields.iter().enumerate() {
                                if self.is_copy_type(field.ty)? {
                                    continue;
                                }
                                let mut child = path.clone();
                                child.push(
                                    u32::try_from(index)
                                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                                );
                                pending.push((field.ty, child, false));
                            }
                        }
                        CheckedNominalKind::Enum { .. } | CheckedNominalKind::Box { .. } => {
                            drops.push((path, current));
                        }
                    }
                }
            }
        }
        Ok(drops)
    }

    pub(super) fn residual_drop_paths(
        &self,
        ty: CheckedType,
        moved: &[u32],
    ) -> Result<Vec<(Vec<u32>, CheckedType)>, CheckStop> {
        let mut drops = Vec::new();
        let mut pending = vec![(ty, Vec::new(), true, 0_usize, false)];
        while let Some((current, path, selected, depth, postorder)) = pending.pop() {
            if selected && depth == moved.len() {
                continue;
            }
            if postorder {
                drops.push((path, current));
                continue;
            }
            match current {
                CheckedType::Unit
                | CheckedType::Bool
                | CheckedType::Integer(_)
                | CheckedType::Float(_)
                | CheckedType::GenericInt(_)
                | CheckedType::Generic(_)
                | CheckedType::Array { .. }
                | CheckedType::Slice { .. }
                | CheckedType::Buffer { .. }
                    if selected =>
                {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                CheckedType::Unit
                | CheckedType::Bool
                | CheckedType::Integer(_)
                | CheckedType::Float(_)
                | CheckedType::GenericInt(_)
                | CheckedType::Generic(_) => {}
                CheckedType::Array { .. }
                | CheckedType::Slice { .. }
                | CheckedType::Buffer { .. } => {
                    drops.push((path, current));
                }
                CheckedType::Nominal(id) => {
                    let nominal = self.nominal(id)?;
                    if nominal.is_copy() {
                        if selected {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        continue;
                    }
                    let CheckedNominalKind::Struct { fields } = &nominal.kind else {
                        if selected {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        drops.push((path, current));
                        continue;
                    };
                    if !selected {
                        pending.push((current, path.clone(), false, depth, true));
                    }
                    let selected_field = if selected {
                        Some(
                            moved
                                .get(depth)
                                .copied()
                                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                        )
                    } else {
                        None
                    };
                    if let Some(selected_field) = selected_field {
                        let field = fields
                            .get(selected_field as usize)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                        if self.is_copy_type(field.ty)? {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                    }
                    for (index, field) in fields.iter().enumerate() {
                        if self.is_copy_type(field.ty)? {
                            continue;
                        }
                        let index = u32::try_from(index)
                            .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                        let mut child = path.clone();
                        child.push(index);
                        let child_selected = selected_field == Some(index);
                        pending.push((
                            field.ty,
                            child,
                            child_selected,
                            depth + usize::from(child_selected),
                            false,
                        ));
                    }
                }
            }
        }
        Ok(drops)
    }
}
