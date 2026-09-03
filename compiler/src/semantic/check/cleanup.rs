use crate::{SemanticCompilerFailure, SystemRelease, SystemReleaseRow};

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
    /// Returns the complete [STOR-3] release record of one value of this
    /// type: its own [SYS-5] action when it is a system resource, and the
    /// union of the fixed [SYS-5] rows of every system resource the value may
    /// transitively own. `buffer`, `box`, arena, and `const` releases carry
    /// the empty row [STOR-3], so only a contained system resource family
    /// contributes anything.
    pub(super) fn release_of_type(&self, ty: CheckedType) -> Result<SystemRelease, CheckStop> {
        let mut visited = HashSet::new();
        let row = self.release_row_of_type(ty, &mut visited)?;
        let action = match ty {
            CheckedType::Nominal(id) => match &self.nominal(id)?.kind {
                CheckedNominalKind::SystemResource { nominal } => {
                    crate::system_resource_contract(*nominal).map(|contract| contract.action)
                }
                CheckedNominalKind::Struct { .. }
                | CheckedNominalKind::Enum { .. }
                | CheckedNominalKind::Box { .. }
                | CheckedNominalKind::Arena { .. }
                | CheckedNominalKind::ArenaStorage => None,
            },
            _ => None,
        };
        Ok(SystemRelease { action, row })
    }

    fn release_row_of_type(
        &self,
        ty: CheckedType,
        visited: &mut HashSet<NominalId>,
    ) -> Result<SystemReleaseRow, CheckStop> {
        if let CheckedType::Buffer { element } = ty {
            // An affine buffer element drops with its owning buffer
            // [STOR-3], so a contained resource row reaches the buffer's
            // release contribution exactly as a box referent's does.
            return self.release_row_of_type(element.ty(), visited);
        }
        let CheckedType::Nominal(id) = ty else {
            // Scalars carry no release action, and array and slice elements
            // are flat copy data with no release of their own.
            return Ok(SystemReleaseRow::EMPTY);
        };
        if !visited.insert(id) {
            return Ok(SystemReleaseRow::EMPTY);
        }
        let component_types: Vec<CheckedType> = match &self.nominal(id)?.kind {
            CheckedNominalKind::SystemResource { nominal } => {
                return Ok(crate::system_release_row(*nominal));
            }
            CheckedNominalKind::Struct { fields } => fields.iter().map(|field| field.ty).collect(),
            CheckedNominalKind::Enum { variants } => variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                .collect(),
            CheckedNominalKind::Box { referent } => vec![*referent],
            // The region release walks the arena content exactly as an owner
            // drop walks a box referent, so a contained resource row still
            // reaches [EFF-2]'s release contribution.
            CheckedNominalKind::Arena { content, .. } => vec![*content],
            // The allocation list frees flat memory only [STOR-3]; content
            // with its own release action is gated before arena_new admits it.
            CheckedNominalKind::ArenaStorage => Vec::new(),
        };
        let mut row = SystemReleaseRow::EMPTY;
        for ty in component_types {
            let component = self.release_row_of_type(ty, visited)?;
            row = row.union(component);
        }
        Ok(row)
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
                CheckedStatement::Set { target, value, .. }
                | CheckedStatement::Replace { target, value, .. } => {
                    // A [SET-2] commit derives no release of its own
                    // [STOR-3]; only its offset and right-hand side can
                    // carry release sites, exactly as for a Set commit.
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
                CheckedStatement::DropExpression {
                    value,
                    state_origins,
                    release,
                } => {
                    self.collect_expression_release_sites(value, sites)?;
                    let effects = self.effects_of_row(release.row, state_origins.as_ref())?;
                    if effects != EffectSet::NONE {
                        sites.push(ReleaseSite {
                            owner: ReleaseOwner::ExpressionResult,
                            effects,
                        });
                    }
                }
                CheckedStatement::Proof(_) => {}
                CheckedStatement::Return { value, drops, .. } => {
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
                CheckedStatement::Give { value, drops, .. } => {
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
                CheckedStatement::CountedRange {
                    lower,
                    upper,
                    body,
                    backedge_drops,
                    ..
                } => {
                    self.collect_expression_release_sites(lower, sites)?;
                    self.collect_expression_release_sites(upper, sites)?;
                    self.collect_release_sites(body, sites)?;
                    self.collect_drop_release_sites(backedge_drops, sites)?;
                }
                CheckedStatement::Break { drops, .. } => {
                    self.collect_drop_release_sites(drops, sites)?;
                }
                CheckedStatement::Region {
                    body,
                    fallthrough_drops,
                    ..
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
            // The drop record already carries its [SYS-5] row, so attribution
            // reads the checked program rather than rederiving it.
            let effects = self.effects_of_row(drop.release.row, drop.state_origins.as_ref())?;
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
                    let effects =
                        self.effects_of_row(drop.release.row, drop.state_origins.as_ref())?;
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
            | CheckedExpression::ArenaNew { value, .. }
            | CheckedExpression::ArenaDeref { value, .. }
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
            CheckedExpression::BufferVacant { length, .. }
            | CheckedExpression::BufferFits { length, .. } => {
                self.collect_expression_release_sites(length, sites)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayLength { .. }
            | CheckedExpression::BufferLength { .. }
            | CheckedExpression::SliceOf { .. }
            | CheckedExpression::SliceLength { .. }
            | CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. }
            | CheckedExpression::DerefAddressed { .. } => {}
        }
        Ok(())
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// Releases carry target execution metadata, not source effect atoms. A
    /// state-writing release must carry the ordinary structural origins of
    /// every state leaf it releases.
    fn effects_of_row(
        &self,
        row: SystemReleaseRow,
        origins: Option<&super::super::model::CheckedStateOrigins>,
    ) -> Result<EffectSet, CheckStop> {
        let mut effects = EffectSet::NONE;
        if row.state_write {
            let Some(origins) = origins else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            if origins.unknown && !self.deriving_result_state_origin.get() {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            for origin in &origins.formals {
                effects.add_write(origin.source.clone());
            }
        }
        Ok(effects)
    }

    /// Attaches the [STOR-3] release record to each derived drop path, so
    /// every construction site records the same fact for the same type.
    pub(super) fn released_paths(
        &self,
        paths: Vec<(Vec<u32>, CheckedType)>,
    ) -> Result<Vec<(Vec<u32>, CheckedType, SystemRelease)>, CheckStop> {
        paths
            .into_iter()
            .map(|(fields, ty)| Ok((fields, ty, self.release_of_type(ty)?)))
            .collect()
    }

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
                        | CheckedNominalKind::SystemResource { .. }
                        // The region's allocation list drops at the region
                        // block's exits, and that drop IS the region's
                        // storage release [STOR-3].
                        | CheckedNominalKind::ArenaStorage => {
                            drops.push((path, current));
                        }
                        // An arena value's storage is released with its
                        // region, never with an owner scope [STOR-3, STOR-4],
                        // so the value derives no drop here.
                        CheckedNominalKind::Arena { .. } => {}
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
use std::collections::HashSet;
