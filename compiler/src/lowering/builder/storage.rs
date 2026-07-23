//! Stable storage for checked whole-struct borrows.
//!
//! `BorrowStruct` is the semantic authority. Finding those explicit nodes
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
    collect_statements(&function.requires, &mut bindings);
    collect_statements(&function.body, &mut bindings);
    bindings
}

fn collect_statements(statements: &[CheckedStatement], bindings: &mut HashSet<BindingId>) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::DropExpression(value)
            | CheckedStatement::Return { value, .. }
            | CheckedStatement::Give { value, .. } => collect_expression(value, bindings),
            CheckedStatement::PropagateLet { scrutinee, .. } => {
                collect_expression(scrutinee, bindings);
            }
            CheckedStatement::Set { target, value } => {
                match target {
                    CheckedSetTarget::Place(_) => {}
                    CheckedSetTarget::ArrayIndex(target) => {
                        collect_expression(&target.offset, bindings);
                    }
                    CheckedSetTarget::BufferIndex(target) => {
                        collect_expression(&target.offset, bindings);
                    }
                }
                collect_expression(value, bindings);
            }
            CheckedStatement::Check { condition, .. } => collect_expression(condition, bindings),
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
            CheckedStatement::Break { .. } => {}
        }
    }
}

fn collect_expression(expression: &CheckedExpression, bindings: &mut HashSet<BindingId>) {
    match expression {
        CheckedExpression::BorrowStruct { binding, .. } => {
            bindings.insert(*binding);
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
                collect_expression(argument, bindings);
            }
        }
        CheckedExpression::NumericConversion { value, .. }
        | CheckedExpression::Reinterpret { value, .. }
        | CheckedExpression::ArrayFill { value, .. }
        | CheckedExpression::BoxNew { value, .. }
        | CheckedExpression::BoxDeref { value, .. }
        | CheckedExpression::ProjectValue { value, .. } => collect_expression(value, bindings),
        CheckedExpression::ArrayIndex { offset, .. }
        | CheckedExpression::BufferIndex { offset, .. }
        | CheckedExpression::SliceIndex { offset, .. } => collect_expression(offset, bindings),
        CheckedExpression::BufferFill { length, value, .. } => {
            collect_expression(length, bindings);
            collect_expression(value, bindings);
        }
        CheckedExpression::Constant(_)
        | CheckedExpression::Binding { .. }
        | CheckedExpression::ArrayLength { .. }
        | CheckedExpression::BufferLength { .. }
        | CheckedExpression::SliceOf { .. }
        | CheckedExpression::SliceLength { .. }
        | CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::ReborrowStruct { .. }
        | CheckedExpression::Project { .. } => {}
    }
}

impl IrBuilder<'_> {
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
        let IrType::Nominal(nominal) = self.value_type(value)? else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        if !matches!(
            self.nominals
                .get(nominal.index())
                .ok_or(LoweringFailure::InvalidCheckedProgram)?
                .kind,
            IrNominalKind::Struct { .. }
        ) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let address = self.define(
            IrType::NominalAddress(nominal),
            IrOperation::AddressOfNominal { value, nominal },
        )?;
        if self.bindings.insert(binding, address) != Some(value) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(())
    }

    pub(super) fn lower_struct_borrow(
        &self,
        binding: BindingId,
        nominal: IrNominalId,
    ) -> Result<IrValueId, LoweringFailure> {
        let value = self
            .bindings
            .get(&binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        if self.value_type(value)? != IrType::NominalAddress(nominal) {
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
        let IrType::NominalAddress(nominal) = self.value_type(storage)? else {
            return Ok(storage);
        };
        self.define(
            IrType::Nominal(nominal),
            IrOperation::LoadNominal {
                address: storage,
                nominal,
            },
        )
    }

    pub(super) fn store_nominal(
        &mut self,
        address: IrValueId,
        value: IrValueId,
        nominal: IrNominalId,
    ) -> Result<(), LoweringFailure> {
        if self.value_type(address)? != IrType::NominalAddress(nominal)
            || self.value_type(value)? != IrType::Nominal(nominal)
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.current_block_mut()?
            .instructions
            .push(IrInstruction::StoreNominal {
                address,
                value,
                nominal,
            });
        Ok(())
    }
}
