use crate::SemanticCompilerFailure;

use super::super::model::{
    CheckedDrop, CheckedExpression, CheckedNominalKind, CheckedSetTarget, CheckedStatement,
    CheckedType,
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
                // [PROV-6, EFF-2] `dispose p;` is a written statement, so the
                // walk it runs contributes to the body-syntactic row where
                // the checker formed it, not to the release contribution.
                CheckedStatement::Dispose { value, .. } => {
                    self.collect_expression_release_effects(function, value, effects)?;
                }
                CheckedStatement::SetList {
                    targets, values, ..
                } => {
                    for target in targets {
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
                            CheckedSetTarget::Storage(target) => {
                                for offset in target.offsets() {
                                    self.collect_expression_release_effects(
                                        function, offset, effects,
                                    )?;
                                }
                            }
                            CheckedSetTarget::SliceIndex(target) => {
                                self.collect_expression_release_effects(
                                    function,
                                    &target.offset,
                                    effects,
                                )?;
                            }
                        }
                    }
                    for value in values.expressions() {
                        self.collect_expression_release_effects(function, value, effects)?;
                    }
                }
                CheckedStatement::PropagateLet {
                    scrutinee,
                    error_drops,
                    ..
                } => {
                    self.collect_expression_release_effects(function, scrutinee, effects)?;
                    self.collect_drop_release_effects(function, error_drops, effects)?;
                }
                CheckedStatement::Set { target, value, .. }
                | CheckedStatement::Replace { target, value, .. } => {
                    // A [SET-2] commit derives no release of its own
                    // [STOR-3]; only its offset and right-hand side can
                    // carry release sites, exactly as for a Set commit.
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
                        CheckedSetTarget::Storage(target) => {
                            for offset in target.offsets() {
                                self.collect_expression_release_effects(function, offset, effects)?;
                            }
                        }
                        CheckedSetTarget::SliceIndex(target) => {
                            self.collect_expression_release_effects(
                                function,
                                &target.offset,
                                effects,
                            )?;
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
                CheckedStatement::Region {
                    body,
                    fallthrough_drops,
                    ..
                } => {
                    self.collect_release_effects(function, body, effects)?;
                    self.collect_drop_release_effects(function, fallthrough_drops, effects)?;
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
            for path in self.resolved_provider_writes(function, drop.ty)? {
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
                    for path in self.resolved_provider_writes(function, drop.ty)? {
                        effects.add_write(path);
                    }
                }
            }
            CheckedExpression::UserCall { arguments, .. }
            | CheckedExpression::KernelCall { arguments, .. }
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
            CheckedExpression::NumericConversion { value, .. }
            | CheckedExpression::Reinterpret { value, .. }
            | CheckedExpression::ArrayFill { value, .. }
            | CheckedExpression::BoxNew { value, .. }
            | CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaNew { value, .. }
            | CheckedExpression::ArenaDeref { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => {
                self.collect_expression_release_effects(function, value, effects)?;
            }
            CheckedExpression::ReadStorage { root, .. } => {
                for offset in root.offsets() {
                    self.collect_expression_release_effects(function, offset, effects)?;
                }
            }
            CheckedExpression::ArrayIndex { offset, .. }
            | CheckedExpression::BufferIndex { offset, .. }
            | CheckedExpression::SliceIndex { offset, .. } => {
                self.collect_expression_release_effects(function, offset, effects)?;
            }
            CheckedExpression::BufferFill { length, value, .. } => {
                self.collect_expression_release_effects(function, length, effects)?;
                self.collect_expression_release_effects(function, value, effects)?;
            }
            CheckedExpression::BufferVacant { length, .. }
            | CheckedExpression::BufferFits { length, .. } => {
                self.collect_expression_release_effects(function, length, effects)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayMeasure { .. }
            | CheckedExpression::BufferMeasure { .. }
            | CheckedExpression::ContainerMeasure { .. }
            | CheckedExpression::PostconditionResultMeasure { .. }
            | CheckedExpression::SliceOf { .. }
            | CheckedExpression::SliceMeasure { .. }
            | CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::ReborrowAddressed { .. }
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
                // A `Heap` is dropped with the empty row and an `Arena` is
                // released with its own region, so neither derives an owner-
                // scope drop [STOR-3, BLK-2].
                CheckedType::Heap { .. } | CheckedType::Extent { .. } => {}
                CheckedType::Array { .. }
                | CheckedType::Slice { .. }
                | CheckedType::Buffer { .. }
                | CheckedType::FixedVector { .. }
                | CheckedType::Vector { .. } => {
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
                        | CheckedNominalKind::Opaque
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
                | CheckedType::FixedVector { .. }
                | CheckedType::Vector { .. }
                | CheckedType::Heap { .. }
                | CheckedType::Extent { .. }
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
                // A `Heap` is dropped with the empty row and an `Arena` is
                // released with its own region, so neither derives an owner-
                // scope drop [STOR-3, BLK-2].
                CheckedType::Heap { .. } | CheckedType::Extent { .. } => {}
                CheckedType::Array { .. }
                | CheckedType::Slice { .. }
                | CheckedType::Buffer { .. }
                | CheckedType::FixedVector { .. }
                | CheckedType::Vector { .. } => {
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
