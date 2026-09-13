//! Stable storage for checked reads, writes and borrows of owned content.
//!
//! Checked storage paths are the semantic authority. Finding those explicit nodes
//! before CFG lowering lets an owner use one representation on every branch:
//! its binding carries a stable address, while ordinary value uses load from
//! that address. Lowering never infers a borrow from source shape or type alone.

use std::collections::HashSet;

use crate::semantic::{
    BindingId, CheckedExpression, CheckedFunction, CheckedSetTarget, CheckedStatement,
};

use super::*;

pub(super) fn collect_addressed_bindings(function: &CheckedFunction) -> HashSet<BindingId> {
    let mut bindings = HashSet::new();
    collect_statements(&function.body, &mut bindings);
    bindings
}

fn collect_statements(statements: &[CheckedStatement], bindings: &mut HashSet<BindingId>) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::DestructuringLet { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::Dispose { value, .. }
            | CheckedStatement::DropExpression { value, .. }
            | CheckedStatement::Return { value, .. }
            | CheckedStatement::Give { value, .. } => collect_expression(value, bindings),
            CheckedStatement::PropagateLet { scrutinee, .. } => {
                collect_expression(scrutinee, bindings);
            }
            CheckedStatement::SetList {
                targets, values, ..
            } => {
                for target in targets {
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => {
                            bindings.insert(target.binding);
                            collect_expression(&target.offset, bindings);
                        }
                        CheckedSetTarget::BufferIndex(target) => {
                            collect_expression(&target.offset, bindings);
                        }
                        CheckedSetTarget::Storage(root) => {
                            bindings.insert(root.binding);
                            collect_place(root, bindings);
                        }
                        CheckedSetTarget::SliceIndex(target) => {
                            collect_expression(&target.offset, bindings);
                        }
                    }
                }
                for value in values.expressions() {
                    collect_expression(value, bindings);
                }
            }
            CheckedStatement::Set { target, value, .. }
            | CheckedStatement::Replace { target, value, .. } => {
                match target {
                    CheckedSetTarget::Place(_) => {}
                    CheckedSetTarget::ArrayIndex(target) => {
                        bindings.insert(target.binding);
                        collect_expression(&target.offset, bindings);
                    }
                    CheckedSetTarget::BufferIndex(target) => {
                        collect_expression(&target.offset, bindings);
                    }
                    CheckedSetTarget::Storage(root) => {
                        bindings.insert(root.binding);
                        collect_place(root, bindings);
                    }
                    CheckedSetTarget::SliceIndex(target) => {
                        collect_expression(&target.offset, bindings);
                    }
                }
                collect_expression(value, bindings);
            }
            CheckedStatement::Match {
                scrutinee, arms, ..
            }
            | CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                collect_expression(scrutinee, bindings);
                for arm in arms {
                    collect_statements(&arm.body, bindings);
                }
            }
            CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                collect_statements(body, bindings);
            }
            CheckedStatement::CountedRange {
                lower, upper, body, ..
            } => {
                collect_expression(lower, bindings);
                collect_expression(upper, bindings);
                collect_statements(body, bindings);
            }
            CheckedStatement::Proof(_) | CheckedStatement::Break { .. } => {}
        }
    }
}

fn collect_expression(expression: &CheckedExpression, bindings: &mut HashSet<BindingId>) {
    match expression {
        CheckedExpression::BorrowAddressed { root, .. }
        | CheckedExpression::ReadStorage { root, .. } => {
            bindings.insert(root.binding);
            collect_place(root, bindings);
        }
        CheckedExpression::SliceOf { source, range, .. } => {
            if let crate::semantic::CheckedSliceSource::Run(root) = source {
                bindings.insert(root.binding);
                collect_place(root, bindings);
            }
            if let Some(range) = range {
                collect_expression(&range.start, bindings);
                collect_expression(&range.end, bindings);
            }
        }
        CheckedExpression::ContainerMeasure { root, .. } => collect_place(root, bindings),
        CheckedExpression::UserCall { arguments, .. }
        | CheckedExpression::SystemCall { arguments, .. }
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
                collect_expression(argument, bindings);
            }
        }
        CheckedExpression::NumericConversion { value, .. }
        | CheckedExpression::Reinterpret { value, .. }
        | CheckedExpression::ArrayFill { value, .. }
        | CheckedExpression::BoxNew { value, .. }
        | CheckedExpression::BoxDeref { value, .. }
        | CheckedExpression::ArenaNew { value, .. }
        | CheckedExpression::ArenaDeref { value, .. }
        | CheckedExpression::ProjectValue { value, .. } => collect_expression(value, bindings),
        CheckedExpression::ArrayIndex { offset, .. }
        | CheckedExpression::BufferIndex { offset, .. }
        | CheckedExpression::SliceIndex { offset, .. } => collect_expression(offset, bindings),
        CheckedExpression::BufferFill { length, value, .. } => {
            collect_expression(length, bindings);
            collect_expression(value, bindings);
        }
        CheckedExpression::BufferVacant { length, .. }
        | CheckedExpression::BufferFits { length, .. } => collect_expression(length, bindings),
        CheckedExpression::Constant(_)
        | CheckedExpression::NamedConstant { .. }
        | CheckedExpression::Binding { .. }
        | CheckedExpression::ArrayMeasure { .. }
        | CheckedExpression::BufferMeasure { .. }
        | CheckedExpression::PostconditionResultMeasure { .. }
        | CheckedExpression::SliceMeasure { .. }
        | CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::BorrowSystemResource { .. }
        | CheckedExpression::ReborrowAddressed { .. }
        | CheckedExpression::DerefAddressed { .. }
        | CheckedExpression::Project { .. } => {}
    }
}

fn collect_place(root: &crate::semantic::CheckedContainerRoot, bindings: &mut HashSet<BindingId>) {
    for step in &root.path {
        if let crate::semantic::CheckedPlaceStep::Subscript(subscript) = step {
            collect_expression(&subscript.offset, bindings);
        }
    }
}

impl IrBuilder<'_> {
    /// Resolve a mutation's complete address once, before its right-hand side.
    pub(super) fn addressed_target(
        &mut self,
        target: &CheckedSetTarget,
        storage: IrValueId,
    ) -> Result<Option<IrValueId>, LoweringFailure> {
        let address = match target {
            CheckedSetTarget::Storage(root) => self.lower_place_address(root)?,
            CheckedSetTarget::Place(place)
                if matches!(self.value_type(storage)?, IrType::Address(_)) =>
            {
                let path: Vec<_> = place
                    .fields
                    .iter()
                    .copied()
                    .map(crate::semantic::CheckedPlaceStep::Field)
                    .collect();
                self.project_address_path(storage, &path)?
            }
            _ => return Ok(None),
        };
        Ok(Some(address))
    }

    pub(super) fn lower_place_address(
        &mut self,
        root: &crate::semantic::CheckedContainerRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let address = self
            .bindings
            .get(&root.binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let address = self.project_address_path(address, &root.path)?;
        let referent = IrAddressed::of(lower_type(self.erasure, root.ty)?)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        if self.value_type(address)? != IrType::Address(referent) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(address)
    }

    pub(super) fn project_address_path(
        &mut self,
        mut address: IrValueId,
        path: &[crate::semantic::CheckedPlaceStep],
    ) -> Result<IrValueId, LoweringFailure> {
        for step in path {
            let IrType::Address(base) = self.value_type(address)? else {
                return Err(LoweringFailure::InvalidCheckedProgram);
            };
            let (projection, ty) = match step {
                crate::semantic::CheckedPlaceStep::Field(field) => {
                    let IrType::Nominal(nominal) = base.ty() else {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    };
                    let IrNominalKind::Struct { fields } = &self.nominals[nominal.index()].kind
                    else {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    };
                    let ty = fields
                        .get(*field as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?
                        .ty;
                    (
                        IrPlaceProjection::Field {
                            nominal,
                            field: *field,
                        },
                        ty,
                    )
                }
                crate::semantic::CheckedPlaceStep::Subscript(subscript) => {
                    let offset = self.expression(&subscript.offset)?;
                    (
                        IrPlaceProjection::RunElement {
                            offset,
                            target_domain: subscript.target_domain.into(),
                        },
                        lower_type(self.erasure, subscript.element_type)?,
                    )
                }
            };
            let referent = IrAddressed::of(ty).ok_or(LoweringFailure::InvalidCheckedProgram)?;
            address = self.define(
                IrType::Address(referent),
                IrOperation::ProjectAddress {
                    address,
                    projection,
                },
            )?;
        }
        Ok(address)
    }

    pub(super) fn promote_binding_if_needed(
        &mut self,
        binding: BindingId,
    ) -> Result<(), LoweringFailure> {
        if !self.addressed_bindings.contains(&binding) {
            return Ok(());
        }
        let value = self
            .bindings
            .get(&binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let ty = self.value_type(value)?;
        // A borrowed parameter already names its caller's stable storage.
        // Reading a checked child path must not introduce another owner or
        // redirect mutations into a local copy of that referent.
        if matches!(ty, IrType::Address(_)) {
            return Ok(());
        }
        let referent = self.addressed_referent(ty)?;
        let address = self.define(
            IrType::Address(referent),
            IrOperation::AddressOf { value, referent },
        )?;
        if self.bindings.insert(binding, address) != Some(value) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(())
    }

    /// The referent an address may point at.
    ///
    /// A borrow addresses directly stored content only; a descriptor or opaque
    /// handle is already its own borrow and never reaches this path.
    fn addressed_referent(&self, ty: IrType) -> Result<IrAddressed, LoweringFailure> {
        let referent = IrAddressed::of(ty).ok_or(LoweringFailure::InvalidCheckedProgram)?;
        if let IrAddressed::Nominal(nominal) = referent
            && !matches!(
                self.nominals
                    .get(nominal.index())
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?
                    .kind,
                IrNominalKind::Struct { .. } | IrNominalKind::Enum { .. }
            )
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(referent)
    }

    pub(super) fn lower_addressed_borrow(
        &self,
        binding: BindingId,
        ty: IrType,
    ) -> Result<IrValueId, LoweringFailure> {
        let value = self
            .bindings
            .get(&binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        if self.value_type(value)? != IrType::Address(self.addressed_referent(ty)?) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(value)
    }

    pub(super) fn binding_value(
        &mut self,
        binding: BindingId,
    ) -> Result<IrValueId, LoweringFailure> {
        let storage = self
            .bindings
            .get(&binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.load_storage_value(storage)
    }

    pub(super) fn load_storage_value(
        &mut self,
        storage: IrValueId,
    ) -> Result<IrValueId, LoweringFailure> {
        let IrType::Address(referent) = self.value_type(storage)? else {
            return Ok(storage);
        };
        self.define(
            referent.ty(),
            IrOperation::Load {
                address: storage,
                referent,
            },
        )
    }

    pub(super) fn store_addressed(
        &mut self,
        address: IrValueId,
        value: IrValueId,
        referent: IrAddressed,
    ) -> Result<(), LoweringFailure> {
        if self.value_type(address)? != IrType::Address(referent)
            || self.value_type(value)? != referent.ty()
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.current_block_mut()?
            .instructions
            .push(IrInstruction::Store {
                address,
                value,
                referent,
            });
        Ok(())
    }
}
