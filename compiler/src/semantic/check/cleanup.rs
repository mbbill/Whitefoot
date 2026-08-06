use std::collections::HashSet;

use crate::SemanticCompilerFailure;

use super::super::model::{
    BindingId, CheckedDrop, CheckedExpression, CheckedNominalKind, CheckedSetTarget,
    CheckedStatement, CheckedType, NominalId,
};
use super::{CheckStop, Checker, EffectSet};

/// The owner whose compiler-derived release contributed one release site.
pub(super) enum ReleaseOwner {
    /// A named parameter, let binding, or match binder.
    Binding(BindingId),
    /// An unnamed discarded expression result.
    ExpressionResult,
}

/// One compiler-derived release recorded in the checked function, with the
/// [SYS-5] row its released type contributes to [EFF-2]'s release
/// contribution. Sites are collected in deterministic statement traversal
/// order, which is DIAG-1's implementation-defined deterministic traversal.
pub(super) struct ReleaseSite {
    pub(super) owner: ReleaseOwner,
    pub(super) effects: EffectSet,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// Returns the effect row a compiler-derived release of one value of
    /// this type may perform: the union of the fixed [SYS-5] release rows
    /// of every system resource the value may transitively own. `buffer`,
    /// `box`, arena, and `const` releases carry the empty row [STOR-3], so
    /// only a contained system resource family contributes anything.
    pub(super) fn release_effects_of_type(&self, ty: CheckedType) -> Result<EffectSet, CheckStop> {
        let mut visited = HashSet::new();
        self.release_effects_of_type_inner(ty, &mut visited)
    }

    fn release_effects_of_type_inner(
        &self,
        ty: CheckedType,
        visited: &mut HashSet<NominalId>,
    ) -> Result<EffectSet, CheckStop> {
        let CheckedType::Nominal(id) = ty else {
            // Scalars carry no release action, and array, slice, and buffer
            // elements are flat copy data with no release of their own.
            return Ok(EffectSet::NONE);
        };
        if !visited.insert(id) {
            return Ok(EffectSet::NONE);
        }
        match &self.nominal(id)?.kind {
            CheckedNominalKind::SystemResource { nominal } => {
                let row = crate::system_release_row(*nominal);
                let mut effects = EffectSet::NONE;
                effects.external = row.external;
                effects.blocks = row.blocks;
                Ok(effects)
            }
            CheckedNominalKind::Struct { fields } => {
                let component_types = fields.iter().map(|field| field.ty).collect::<Vec<_>>();
                let mut effects = EffectSet::NONE;
                for ty in component_types {
                    effects = effects.union(self.release_effects_of_type_inner(ty, visited)?);
                }
                Ok(effects)
            }
            CheckedNominalKind::Enum { variants } => {
                let component_types = variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                    .collect::<Vec<_>>();
                let mut effects = EffectSet::NONE;
                for ty in component_types {
                    effects = effects.union(self.release_effects_of_type_inner(ty, visited)?);
                }
                Ok(effects)
            }
            CheckedNominalKind::Box { referent } => {
                self.release_effects_of_type_inner(*referent, visited)
            }
        }
    }

    /// Collects every compiler-derived release the checked statements carry
    /// whose released type contributes a nonempty release row, in
    /// deterministic traversal order. The checked program records exactly
    /// one disposition per owner per normal edge [STOR-3], so these drop
    /// records are the complete release contribution of [EFF-2].
    pub(super) fn collect_release_sites(
        &self,
        statements: &[CheckedStatement],
        sites: &mut Vec<ReleaseSite>,
    ) -> Result<(), CheckStop> {
        for statement in statements {
            match statement {
                CheckedStatement::Let { value, .. } => {
                    self.collect_expression_release_sites(value, sites)?;
                }
                CheckedStatement::PropagateLet {
                    scrutinee,
                    error_drops,
                    ..
                } => {
                    self.collect_expression_release_sites(scrutinee, sites)?;
                    self.collect_drop_release_sites(error_drops, sites)?;
                }
                CheckedStatement::Set { target, value } => {
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => {
                            self.collect_expression_release_sites(&target.offset, sites)?;
                        }
                        CheckedSetTarget::BufferIndex(target) => {
                            self.collect_expression_release_sites(&target.offset, sites)?;
                        }
                    }
                    self.collect_expression_release_sites(value, sites)?;
                }
                CheckedStatement::Evaluate(value) => {
                    self.collect_expression_release_sites(value, sites)?;
                }
                CheckedStatement::DropExpression(value) => {
                    self.collect_expression_release_sites(value, sites)?;
                    let effects = self.release_effects_of_type(value.ty())?;
                    if effects != EffectSet::NONE {
                        sites.push(ReleaseSite {
                            owner: ReleaseOwner::ExpressionResult,
                            effects,
                        });
                    }
                }
                CheckedStatement::Check { condition, .. } => {
                    self.collect_expression_release_sites(condition, sites)?;
                }
                CheckedStatement::Return { value, drops } => {
                    self.collect_expression_release_sites(value, sites)?;
                    self.collect_drop_release_sites(drops, sites)?;
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                }
                | CheckedStatement::ValueMatchLet {
                    scrutinee, arms, ..
                } => {
                    self.collect_expression_release_sites(scrutinee, sites)?;
                    for arm in arms {
                        self.collect_release_sites(&arm.body, sites)?;
                        self.collect_drop_release_sites(&arm.fallthrough_drops, sites)?;
                    }
                }
                CheckedStatement::Give { value, drops } => {
                    self.collect_expression_release_sites(value, sites)?;
                    self.collect_drop_release_sites(drops, sites)?;
                }
                CheckedStatement::Loop {
                    body,
                    backedge_drops,
                    ..
                } => {
                    self.collect_release_sites(body, sites)?;
                    self.collect_drop_release_sites(backedge_drops, sites)?;
                }
                CheckedStatement::Break { drops, .. } => {
                    self.collect_drop_release_sites(drops, sites)?;
                }
                CheckedStatement::Region {
                    body,
                    fallthrough_drops,
                } => {
                    self.collect_release_sites(body, sites)?;
                    self.collect_drop_release_sites(fallthrough_drops, sites)?;
                }
            }
        }
        Ok(())
    }

    fn collect_drop_release_sites(
        &self,
        drops: &[CheckedDrop],
        sites: &mut Vec<ReleaseSite>,
    ) -> Result<(), CheckStop> {
        for drop in drops {
            let effects = self.release_effects_of_type(drop.ty)?;
            if effects != EffectSet::NONE {
                sites.push(ReleaseSite {
                    owner: ReleaseOwner::Binding(drop.binding),
                    effects,
                });
            }
        }
        Ok(())
    }

    fn collect_expression_release_sites(
        &self,
        expression: &CheckedExpression,
        sites: &mut Vec<ReleaseSite>,
    ) -> Result<(), CheckStop> {
        match expression {
            CheckedExpression::Project {
                binding,
                residual_drops,
                ..
            } => {
                for drop in residual_drops {
                    let effects = self.release_effects_of_type(drop.ty)?;
                    if effects != EffectSet::NONE {
                        sites.push(ReleaseSite {
                            owner: ReleaseOwner::Binding(*binding),
                            effects,
                        });
                    }
                }
            }
            CheckedExpression::UserCall { arguments, .. }
            | CheckedExpression::SystemCall { arguments, .. }
            | CheckedExpression::IntegerOperation { arguments, .. }
            | CheckedExpression::FloatOperation { arguments, .. }
            | CheckedExpression::BooleanOperation { arguments, .. }
            | CheckedExpression::EnumEquality { arguments, .. }
            | CheckedExpression::ConstructStruct {
                fields: arguments, ..
            }
            | CheckedExpression::ConstructEnum {
                fields: arguments, ..
            } => {
                for argument in arguments {
                    self.collect_expression_release_sites(argument, sites)?;
                }
            }
            CheckedExpression::NumericConversion { value, .. }
            | CheckedExpression::Reinterpret { value, .. }
            | CheckedExpression::ArrayFill { value, .. }
            | CheckedExpression::BoxNew { value, .. }
            | CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => {
                self.collect_expression_release_sites(value, sites)?;
            }
            CheckedExpression::ArrayIndex { offset, .. }
            | CheckedExpression::BufferIndex { offset, .. }
            | CheckedExpression::SliceIndex { offset, .. } => {
                self.collect_expression_release_sites(offset, sites)?;
            }
            CheckedExpression::BufferFill { length, value, .. } => {
                self.collect_expression_release_sites(length, sites)?;
                self.collect_expression_release_sites(value, sites)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayLength { .. }
            | CheckedExpression::BufferLength { .. }
            | CheckedExpression::SliceOf { .. }
            | CheckedExpression::SliceLength { .. }
            | CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowStruct { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowStruct { .. } => {}
        }
        Ok(())
    }
}

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
                | CheckedType::GenericFloat(_)
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
                        CheckedNominalKind::Enum { .. }
                        | CheckedNominalKind::Box { .. }
                        | CheckedNominalKind::SystemResource { .. } => {
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
                | CheckedType::GenericFloat(_)
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
                | CheckedType::GenericFloat(_)
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
