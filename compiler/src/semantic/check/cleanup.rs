use crate::SemanticCompilerFailure;

use super::super::model::{
    CheckedDrop, CheckedExpression, CheckedNominalKind, CheckedReleaseMode, CheckedSetTarget,
    CheckedStatement, CheckedType,
};
use super::{CheckStop, Checker, EffectSet};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [STOR-3, PROV-6, EFF-2] Derived memory reclamation writes each
    /// required provider. An opaque value contributes no release effect.
    pub(super) fn collect_release_effects(
        &self,
        function: &super::FunctionSignature,
        statements: &[CheckedStatement],
        effects: &mut EffectSet,
    ) -> Result<(), CheckStop> {
        for statement in statements {
            match statement {
                CheckedStatement::Let { value, .. }
                | CheckedStatement::DestructuringLet { value, .. } => {
                    self.collect_expression_release_effects(function, value, effects)?;
                }
                CheckedStatement::PropagateLet {
                    scrutinee,
                    error_drops,
                    ..
                } => {
                    self.collect_expression_release_effects(function, scrutinee, effects)?;
                    self.collect_drop_release_effects(function, error_drops, effects)?;
                }
                CheckedStatement::Set { target, value, .. } => {
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => {
                            self.collect_expression_release_effects(
                                function,
                                &target.offset,
                                effects,
                            )?;
                        }
                        CheckedSetTarget::BufferIndex(target) => {
                            self.collect_expression_release_effects(
                                function,
                                &target.offset,
                                effects,
                            )?;
                        }
                        CheckedSetTarget::RangeIndex(target) => {
                            for offset in target.offsets() {
                                self.collect_expression_release_effects(function, offset, effects)?;
                            }
                        }
                        CheckedSetTarget::Storage(target) => {
                            for offset in target.offsets() {
                                self.collect_expression_release_effects(function, offset, effects)?;
                            }
                        }
                    }
                    self.collect_expression_release_effects(function, value, effects)?;
                }
                CheckedStatement::Evaluate(value) => {
                    self.collect_expression_release_effects(function, value, effects)?;
                }
                CheckedStatement::DropExpression { value } => {
                    self.collect_expression_release_effects(function, value, effects)?;
                    for path in self.resolved_provider_writes(function, value.ty())? {
                        effects.add_write(path);
                    }
                }
                CheckedStatement::Proof(_) => {}
                CheckedStatement::Return { value, drops, .. } => {
                    self.collect_expression_release_effects(function, value, effects)?;
                    self.collect_drop_release_effects(function, drops, effects)?;
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                }
                | CheckedStatement::ValueMatchLet {
                    scrutinee, arms, ..
                } => {
                    self.collect_expression_release_effects(function, scrutinee, effects)?;
                    for arm in arms {
                        self.collect_release_effects(function, &arm.body, effects)?;
                        self.collect_drop_release_effects(
                            function,
                            &arm.fallthrough_drops,
                            effects,
                        )?;
                    }
                }
                CheckedStatement::Give { value, drops, .. } => {
                    self.collect_expression_release_effects(function, value, effects)?;
                    self.collect_drop_release_effects(function, drops, effects)?;
                }
                CheckedStatement::Loop {
                    body,
                    backedge_drops,
                    ..
                } => {
                    self.collect_release_effects(function, body, effects)?;
                    self.collect_drop_release_effects(function, backedge_drops, effects)?;
                }
                CheckedStatement::CountedRange {
                    lower,
                    upper,
                    body,
                    backedge_drops,
                    ..
                } => {
                    self.collect_expression_release_effects(function, lower, effects)?;
                    self.collect_expression_release_effects(function, upper, effects)?;
                    self.collect_release_effects(function, body, effects)?;
                    self.collect_drop_release_effects(function, backedge_drops, effects)?;
                }
                CheckedStatement::Break { drops, .. } => {
                    self.collect_drop_release_effects(function, drops, effects)?;
                }
            }
        }
        Ok(())
    }

    fn collect_drop_release_effects(
        &self,
        function: &super::FunctionSignature,
        drops: &[CheckedDrop],
        effects: &mut EffectSet,
    ) -> Result<(), CheckStop> {
        for drop in drops {
            for path in self.resolved_provider_writes_for(function, drop.ty, drop.release)? {
                effects.add_write(path);
            }
        }
        Ok(())
    }

    fn collect_expression_release_effects(
        &self,
        function: &super::FunctionSignature,
        expression: &CheckedExpression,
        effects: &mut EffectSet,
    ) -> Result<(), CheckStop> {
        match expression {
            CheckedExpression::Project { residual_drops, .. } => {
                for drop in residual_drops {
                    for path in
                        self.resolved_provider_writes_for(function, drop.ty, drop.release)?
                    {
                        effects.add_write(path);
                    }
                }
            }
            CheckedExpression::UserCall { arguments, .. }
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
                    self.collect_expression_release_effects(function, argument, effects)?;
                }
            }
            CheckedExpression::BoxTake { cleanup, .. } => {
                for action in cleanup {
                    if let super::super::model::CheckedOwnedTakeCleanup::Drop { ty, .. } = action {
                        for path in self.resolved_provider_writes_for(
                            function,
                            *ty,
                            CheckedReleaseMode::Full,
                        )? {
                            effects.add_write(path);
                        }
                    }
                }
            }
            CheckedExpression::NumericConversion { value, .. }
            | CheckedExpression::Reinterpret { value, .. }
            | CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => {
                self.collect_expression_release_effects(function, value, effects)?;
            }
            CheckedExpression::ReadStorage { root, .. } => {
                for offset in root.offsets() {
                    self.collect_expression_release_effects(function, offset, effects)?;
                }
            }
            CheckedExpression::ArrayIndex { offset, .. }
            | CheckedExpression::BufferIndex { offset, .. } => {
                self.collect_expression_release_effects(function, offset, effects)?;
            }
            CheckedExpression::RangeElementMeasure { place, .. }
            | CheckedExpression::RangeIndex { place, .. }
            | CheckedExpression::BorrowRangeIndex { place, .. } => {
                for offset in place.offsets() {
                    self.collect_expression_release_effects(function, offset, effects)?;
                }
            }
            CheckedExpression::RangeOf {
                source, start, end, ..
            } => {
                if let crate::semantic::CheckedRangeSource::Storage(root) = source {
                    for offset in root.offsets() {
                        self.collect_expression_release_effects(function, offset, effects)?;
                    }
                }
                self.collect_expression_release_effects(function, start, effects)?;
                self.collect_expression_release_effects(function, end, effects)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayMeasure { .. }
            | CheckedExpression::BufferMeasure { .. }
            | CheckedExpression::ContainerMeasure { .. }
            | CheckedExpression::RangeMeasure { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::DerefAddressed { .. } => {}
        }
        Ok(())
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
                | CheckedType::Buffer { .. }
                | CheckedType::Window { .. } => {
                    // [OWN-1, STOR-3] an `Array` of copy elements is copy and
                    // a copy value has an empty release.
                    if !self.is_copy_type(current)? {
                        drops.push((path, current));
                    }
                }
                CheckedType::Nominal(id) => {
                    let nominal = self.nominal(id)?;
                    // [OWN-1, STOR-3] a nominal whose every owned part is
                    // copy, and whose declaration removes nothing, is copy
                    // and has an empty release. A tag-only enum declared
                    // `nocopy` is affine and owns nothing, so its release is
                    // empty as well.
                    if self.is_copy_type(current)? || nominal.is_tag_only_enum() {
                        continue;
                    }
                    match &nominal.kind {
                        CheckedNominalKind::Struct { fields } => {
                            pending.push((current, path.clone(), true));
                            // PROV-6 visits fields in declaration order; the
                            // explicit work stack is last-in, first-out.
                            for (index, field) in fields.iter().enumerate().rev() {
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
                        | CheckedNominalKind::Opaque => {
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
                | CheckedType::Buffer { .. }
                | CheckedType::Window { .. }
                    if selected =>
                {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                CheckedType::Unit
                | CheckedType::Bool
                | CheckedType::Integer(_)
                | CheckedType::Float(_)
                | CheckedType::GenericInt(_)
                | CheckedType::GenericFloat(_) => {}
                CheckedType::Generic(_)
                | CheckedType::Array { .. }
                | CheckedType::Buffer { .. }
                | CheckedType::Window { .. } => {
                    if !self.is_copy_type(current)? {
                        drops.push((path, current));
                    }
                }
                CheckedType::Nominal(id) => {
                    let nominal = self.nominal(id)?;
                    if self.is_copy_type(current)? {
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
                    for (index, field) in fields.iter().enumerate().rev() {
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
