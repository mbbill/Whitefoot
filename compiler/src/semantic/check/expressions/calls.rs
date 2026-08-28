mod conversions;
mod floating;
mod reinterpret;
mod system;
mod user;

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedBooleanOperation, CheckedExpression, CheckedIntegerArgument,
    CheckedIntegerArgumentSource, CheckedIntegerErrorClass, CheckedIntegerOperation, CheckedMode,
    CheckedNominalKind, CheckedNumericType, CheckedType,
};
use super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PendingNominal, PreludeType,
    TypedExpression,
};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(in crate::semantic::check) fn check_call(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let callee = self
            .tree
            .first_child_with(node, Production::Callee)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let usage = self.use_at_roles(
            callee,
            &[
                LexicalUseRole::IdentifierCallee,
                LexicalUseRole::OperationCallee,
            ],
        )?;
        match usage.target() {
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Function,
            } => self.check_user_call(node, declaration, function, bindings, loop_depth),
            ResolvedTarget::Operation(operation) => {
                self.check_operation(node, operation, function, bindings, loop_depth)
            }
            ResolvedTarget::System(id) => {
                let operation = crate::system_operation_index(id, self.inventory())
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                self.check_system_call(node, operation, function, bindings, loop_depth)
            }
            _ => Err(SemanticCompilerFailure::InvalidResolution.into()),
        }
    }

    fn check_operation(
        &self,
        node: NodeId,
        operation_id: crate::OperationFamilyId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let spelling = crate::operation_family_spelling(operation_id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if matches!(spelling, "band" | "bor" | "bxor" | "bnot") {
            return self.check_boolean_operation(node, spelling, function, bindings, loop_depth);
        }
        if matches!(spelling, "eeq" | "ene") {
            return self.check_enum_equality(
                node,
                spelling == "eeq",
                function,
                bindings,
                loop_depth,
            );
        }
        if floating::is_float_operation(spelling) {
            return self.check_float_operation(node, spelling, function, bindings, loop_depth);
        }
        if spelling == "arena_new" {
            return self.check_arena_new(node, function, bindings, loop_depth);
        }
        if spelling == "array_new" {
            return self.check_array_new(node, function, bindings, loop_depth);
        }
        if spelling == "buffer_new" {
            return self.check_buffer_new(node, function, bindings, loop_depth);
        }
        if spelling == "buffer_vacant" {
            return self.check_buffer_vacant(node, function, bindings, loop_depth);
        }
        if spelling == "buffer_fits" {
            return self.check_buffer_fits(node, function, bindings, loop_depth);
        }
        if spelling == "box_new" {
            return self.check_box_new(node, function, bindings, loop_depth);
        }
        if spelling == "len" {
            return self.check_flat_length(node, function, bindings, loop_depth);
        }
        if spelling == "slice_of" {
            return self.check_slice_of(node, function, bindings, loop_depth);
        }
        if spelling == "cvt" {
            return self.check_conversion(node, function, bindings, loop_depth);
        }
        if spelling == "reinterpret" {
            return self.check_reinterpret(node, function, bindings, loop_depth);
        }
        let operation = match spelling {
            "iabs.wrap" => CheckedIntegerOperation::AbsoluteWrap,
            "iabs" => CheckedIntegerOperation::AbsoluteExact,
            "iabs.defined" => CheckedIntegerOperation::AbsoluteDefined,
            "iabs.checked" => CheckedIntegerOperation::AbsoluteChecked,
            "ineg.wrap" => CheckedIntegerOperation::NegateWrap,
            "ineg" => CheckedIntegerOperation::NegateExact,
            "ineg.defined" => CheckedIntegerOperation::NegateDefined,
            "ineg.checked" => CheckedIntegerOperation::NegateChecked,
            "iand" => CheckedIntegerOperation::BitAnd,
            "ior" => CheckedIntegerOperation::BitOr,
            "ixor" => CheckedIntegerOperation::BitXor,
            "inot" => CheckedIntegerOperation::BitNot,
            "ishl.wrap" => CheckedIntegerOperation::ShiftLeftWrap,
            "ishr.wrap" => CheckedIntegerOperation::ShiftRightWrap,
            "ishl" => CheckedIntegerOperation::ShiftLeftExact,
            "ishr" => CheckedIntegerOperation::ShiftRightExact,
            "ishl.defined" => CheckedIntegerOperation::ShiftLeftDefined,
            "ishr.defined" => CheckedIntegerOperation::ShiftRightDefined,
            "irotl" => CheckedIntegerOperation::RotateLeft,
            "irotr" => CheckedIntegerOperation::RotateRight,
            "ipopcount" => CheckedIntegerOperation::PopulationCount,
            "iclz" => CheckedIntegerOperation::LeadingZeros,
            "ictz" => CheckedIntegerOperation::TrailingZeros,
            "ibswap" => CheckedIntegerOperation::ByteSwap,
            "imulhi" => CheckedIntegerOperation::MultiplyHigh,
            "iadd.sat" => CheckedIntegerOperation::AddSaturating,
            "isub.sat" => CheckedIntegerOperation::SubtractSaturating,
            "imul.sat" => CheckedIntegerOperation::MultiplySaturating,
            "imin" => CheckedIntegerOperation::Minimum,
            "imax" => CheckedIntegerOperation::Maximum,
            "ieq" => CheckedIntegerOperation::Equal,
            "ine" => CheckedIntegerOperation::NotEqual,
            "ilt" => CheckedIntegerOperation::Less,
            "ile" => CheckedIntegerOperation::LessEqual,
            "igt" => CheckedIntegerOperation::Greater,
            "ige" => CheckedIntegerOperation::GreaterEqual,
            _ => {
                return self.unsupported(UnsupportedSemanticFeature::OperationFamily, node);
            }
        };
        if self
            .tree
            .first_child_with(node, Production::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: spelling.to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        self.reject_written_operation_type_argument(node)?;
        let atoms = self.operation_atoms(node, operation.operand_count())?;
        self.check_integer_operation_row(node, operation, &atoms, function, bindings, loop_depth)
    }

    /// [OP-1] one integer row over its operand atoms, whichever spelling
    /// selected it.
    ///
    /// The named call and the infix form share this judgment exactly, so the
    /// two spellings of one operation cannot drift apart: [OP-2]'s
    /// operand-derived selection, proof obligation identity, and checked-error
    /// result are decided here once.
    pub(in crate::semantic::check) fn check_integer_operation_row(
        &self,
        node: NodeId,
        operation: CheckedIntegerOperation,
        atoms: &[NodeId],
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let operand_count = operation.operand_count();
        if atoms.len() != operand_count {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        let mut arguments = Vec::with_capacity(operand_count);
        let mut argument_metadata = Vec::with_capacity(operand_count);
        let mut effects = EffectSet::NONE;
        // [OP-2] the selected type is derived from the operands: the first
        // operand's exact type is it, and every later operand must be
        // exactly the row's argument type for that selection — which for the
        // two-operand arithmetic and comparison rows is the selected type
        // itself, so "both operands must have one identical exact type"
        // falls out and cites TYPE-5 at the second operand atom.
        let mut operand_type = None;
        for (index, atom) in atoms.iter().copied().enumerate() {
            let argument = self.check_atom(function, atom, bindings, loop_depth)?;
            if argument.mode != CheckedMode::Own {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        format!("own {}", self.checked_type_name(argument.expression.ty())?),
                        self.checked_value_name(argument.mode, argument.expression.ty())?,
                    ),
                );
            }
            let selected = match operand_type {
                Some(selected) => selected,
                None => {
                    let selected = argument.expression.ty();
                    if !operation.accepts_operand_type(selected) {
                        return self.issue_node(
                            SemanticRule::Op1,
                            node,
                            SemanticIssueKind::InvalidOperation,
                        );
                    }
                    operand_type = Some(selected);
                    selected
                }
            };
            if Some(argument.expression.ty()) != operation.argument_type(selected, index) {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        match operation.argument_type(selected, index) {
                            Some(ty) => format!("own {}", self.checked_type_name(ty)?),
                            None => format!("no operand in position {index} for this row"),
                        },
                        self.checked_value_name(argument.mode, argument.expression.ty())?,
                    ),
                );
            }
            effects = effects.union(argument.effects);
            let source = match &argument.expression {
                CheckedExpression::NamedConstant { declaration, .. } => {
                    CheckedIntegerArgumentSource::NamedConstant {
                        declaration: *declaration,
                    }
                }
                CheckedExpression::Constant(_) => match self
                    .tree
                    .direct_token_with(atom, TerminalPredicate::Literal)?
                {
                    Some(literal) if matches!(self.tree.token_bytes(literal)?, b"0_T" | b"1_T") => {
                        CheckedIntegerArgumentSource::GenericNumericIdentity
                    }
                    Some(_) => CheckedIntegerArgumentSource::TypedLiteral,
                    None => CheckedIntegerArgumentSource::Other,
                },
                _ => CheckedIntegerArgumentSource::Other,
            };
            argument_metadata.push(CheckedIntegerArgument {
                node_path: self.tree.path(atom)?.clone(),
                source,
            });
            arguments.push(argument.expression);
        }
        // `operation_atoms` already rejected a wrong operand count, and no
        // integer row is nullary, so the selection is always made by here.
        let operand_type = operand_type.ok_or(SemanticCompilerFailure::InvalidResolution)?;
        // The row's own `signature` cell decides this, so the mapping lives
        // once on the operation and an extraction lock compares it against the
        // specification's cell.
        let checked_error = operation.checked_error().map(|class| match class {
            CheckedIntegerErrorClass::Overflow => PreludeType::Overflow,
            CheckedIntegerErrorClass::DivError => PreludeType::DivError,
        });
        let result = if let Some(error) = checked_error {
            CheckedType::Nominal(self.prelude_nominal(PreludeType::Result(
                operand_type,
                CheckedType::Nominal(self.prelude_nominal(error)?),
            ))?)
        } else {
            operation
                .scalar_result_type(operand_type)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
        };
        Ok(TypedExpression::owned(
            CheckedExpression::IntegerOperation {
                carrier: self.tree.path(node)?.clone(),
                operation,
                operand_type,
                argument_metadata,
                arguments,
                result,
            },
            effects,
        ))
    }

    /// The written type pair of `cvt` and `reinterpret`, two of the closed
    /// retained-argument rows.
    ///
    /// [DIAG-1] selects the cited rule by the callee's class rather than by
    /// the kind of argument problem: a table operation cites what [OP-2]
    /// selects and never FN-2, which belongs to a user-generic call. [TYPE-5]
    /// is what mandates these arguments, so their absence is its violation —
    /// the same reading [`Self::retained_operation_type_argument`] already
    /// applies to `finf` and `fnan`.
    fn numeric_type_arguments(
        &self,
        node: NodeId,
        function: &FunctionSignature,
    ) -> Result<[CheckedNumericType; 2], CheckStop> {
        let targs = self
            .tree
            .first_child_with(node, Production::Targs)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Type5,
                    node,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let [source, destination] = arguments.as_slice() else {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        };
        let mut parsed = Vec::with_capacity(2);
        for argument in [*source, *destination] {
            let type_node = self
                .tree
                .first_child_with(argument, Production::Type)?
                .ok_or_else(|| {
                    self.issue_value(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation)
                })?;
            parsed.push(
                match self.parse_type_with(type_node, &function.substitution)? {
                    CheckedType::Integer(ty) => CheckedNumericType::Integer(ty),
                    CheckedType::Float(ty) => CheckedNumericType::Float(ty),
                    CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => {
                        return self.unsupported(UnsupportedSemanticFeature::Generics, type_node);
                    }
                    _ => {
                        return self.issue_node(
                            SemanticRule::Op1,
                            node,
                            SemanticIssueKind::InvalidOperation,
                        );
                    }
                },
            );
        }
        parsed
            .try_into()
            .map_err(|_| SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    /// [STOR-2] `arena_new<'r, T>(v)` returns `own arena<'r, T>`: the content
    /// moves into storage owned by region `'r` and registered on that
    /// region's allocation list, which the region's exits release [STOR-3].
    fn check_arena_new(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        // GRAM-11 named-argument rejection plus [STOR-5] on the written
        // content argument.
        self.reject_region_bearing_storage_operation_argument(node, "arena_new", function, 2, 1)?;
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let [region_argument, content_argument] = arguments.as_slice() else {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        };
        if self
            .tree
            .first_child_with(*region_argument, Production::Type)?
            .is_some()
        {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        let region_use = self.use_at(*region_argument, LexicalUseRole::TypeArgumentRegion)?;
        let ResolvedTarget::Source {
            declaration: region,
            class: DeclarationClass::Region,
        } = region_use.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let Some(content_node) = self
            .tree
            .first_child_with(*content_argument, Production::Type)?
        else {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        };
        let content = self.parse_type_with(content_node, &function.substitution)?;
        // The implemented content fragment carries no release action of its
        // own [STOR-3]: flat scalars and arrays of them. Wider content stays
        // an explicit capability stop rather than a silent storage leak.
        if self.flat_element(content)?.is_none() && !matches!(content, CheckedType::Array { .. }) {
            return self.unsupported(UnsupportedSemanticFeature::ArenaRuntime, content_node);
        }
        let atoms = self.operation_atoms(node, 1)?;
        let value = self.check_atom(function, atoms[0], bindings, loop_depth)?;
        if value.mode != CheckedMode::Own || value.expression.ty() != content {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(
                    format!("own {}", self.checked_type_name(content)?),
                    self.checked_value_name(value.mode, value.expression.ty())?,
                ),
            );
        }
        // The region's allocation list exists exactly for the local region
        // blocks this checker opened [STOR-3]. A caller-supplied region has
        // no local list; allocation into it is the explicit unimplemented
        // remainder of the arena runtime.
        let Some(list) = bindings.get(&region).map(|local| local.binding) else {
            return self.unsupported(UnsupportedSemanticFeature::ArenaRuntime, node);
        };
        let Some(nominal) = self.arena_nominals.get(&(region, content)).copied() else {
            self.pending_nominals
                .borrow_mut()
                .push(PendingNominal::Arena(region, content));
            return Err(CheckStop::DeferredNominal);
        };
        Ok(TypedExpression::owned(
            CheckedExpression::ArenaNew {
                carrier: self.tree.path(node)?.clone(),
                nominal,
                list,
                value: Box::new(value.expression),
            },
            value.effects,
        ))
    }

    fn check_box_new(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.reject_named_operation_arguments(node, "box_new")?;
        self.reject_written_operation_type_argument(node)?;
        let atoms = self.operation_atoms(node, 1)?;
        // [STOR-2] `box_new(v)` returns `own box<T>` for `v`'s exact type T.
        let value = self.check_atom(function, atoms[0], bindings, loop_depth)?;
        if value.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(
                    format!("own {}", self.checked_type_name(value.expression.ty())?),
                    self.checked_value_name(value.mode, value.expression.ty())?,
                ),
            );
        }
        let referent = value.expression.ty();
        // [STOR-5] box content may not bear a region. The written referent
        // type used to carry this judgment; the derived one carries it here.
        // A directly slice-typed operand is the only way a region reaches
        // box content: struct fields and enum payloads are held to STOR-5 at
        // their own declarations, `CheckedFlatElement` cannot be a slice, so
        // no array, buffer, or nominal referent can smuggle one in.
        if matches!(referent, CheckedType::Slice { .. }) {
            return self.issue_node(
                SemanticRule::Stor5,
                atoms[0],
                SemanticIssueKind::RegionBearingStorage {
                    mechanical_fix:
                        "keep the slice or arena as a direct local, parameter, or result; do not store it inside another value",
                },
            );
        }
        // [STOR-2] the box nominal is derived from the operand, so the pass
        // that interns from a written `box<T>` cannot have reached it: a
        // purely local box names that type nowhere. Record the referent and
        // let the driver intern it and check this function again.
        let Some(nominal) = self.box_nominals.get(&referent).copied() else {
            self.pending_nominals
                .borrow_mut()
                .push(PendingNominal::Box(referent));
            return Err(CheckStop::DeferredNominal);
        };
        Ok(TypedExpression::owned(
            CheckedExpression::BoxNew {
                carrier: self.tree.path(node)?.clone(),
                nominal,
                value: Box::new(value.expression),
            },
            value.effects.union(EffectSet::ALLOCATES_HEAP),
        ))
    }

    fn check_boolean_operation(
        &self,
        node: NodeId,
        spelling: &str,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let operation = match spelling {
            "band" => CheckedBooleanOperation::And,
            "bor" => CheckedBooleanOperation::Or,
            "bxor" => CheckedBooleanOperation::ExclusiveOr,
            "bnot" => CheckedBooleanOperation::Not,
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        self.reject_named_operation_arguments(node, spelling)?;
        self.reject_written_operation_type_argument(node)?;
        // The Bool row has no type parameter to select: every operand is
        // checked against `Bool` below, which is the whole derivation.
        let expected = usize::from(operation != CheckedBooleanOperation::Not) + 1;
        let atoms = self.operation_atoms(node, expected)?;
        let mut arguments = Vec::with_capacity(atoms.len());
        let mut effects = EffectSet::NONE;
        for atom in atoms {
            let argument = self.check_atom(function, atom, bindings, loop_depth)?;
            if argument.expression.ty() != CheckedType::Bool || argument.mode != CheckedMode::Own {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        "own Bool",
                        self.checked_value_name(argument.mode, argument.expression.ty())?,
                    ),
                );
            }
            effects = effects.union(argument.effects);
            arguments.push(argument.expression);
        }
        Ok(TypedExpression::owned(
            CheckedExpression::BooleanOperation {
                carrier: self.tree.path(node)?.clone(),
                operation,
                arguments,
            },
            effects,
        ))
    }

    fn check_enum_equality(
        &self,
        node: NodeId,
        equal: bool,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let spelling = if equal { "eeq" } else { "ene" };
        self.reject_named_operation_arguments(node, spelling)?;
        self.reject_written_operation_type_argument(node)?;
        let atoms = self.operation_atoms(node, 2)?;
        // [OP-2] the selected tag-only nominal is the first operand's exact
        // type; the second is then checked against it.
        let first = self.check_atom(function, atoms[0], bindings, loop_depth)?;
        if first.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(
                    format!("own {}", self.checked_type_name(first.expression.ty())?),
                    self.checked_value_name(first.mode, first.expression.ty())?,
                ),
            );
        }
        let operand_type = first.expression.ty();
        let tag_only = match operand_type {
            CheckedType::Bool => true,
            CheckedType::Nominal(id) => matches!(
                &self.nominal(id)?.kind,
                CheckedNominalKind::Enum { variants }
                    if variants.iter().all(|variant| variant.fields.is_empty())
            ),
            _ => false,
        };
        if !tag_only {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        // The first operand is already checked, and checking an atom can
        // consume its place, so only the remaining one is checked here.
        let mut effects = first.effects;
        let mut arguments = vec![first.expression];
        for atom in &atoms[1..] {
            let argument = self.check_atom(function, *atom, bindings, loop_depth)?;
            if argument.expression.ty() != operand_type || argument.mode != CheckedMode::Own {
                return self.issue_node(
                    SemanticRule::Type5,
                    *atom,
                    SemanticIssueKind::type_mismatch(
                        format!("own {}", self.checked_type_name(operand_type)?),
                        self.checked_value_name(argument.mode, argument.expression.ty())?,
                    ),
                );
            }
            effects = effects.union(argument.effects);
            arguments.push(argument.expression);
        }
        Ok(TypedExpression::owned(
            CheckedExpression::EnumEquality {
                carrier: self.tree.path(node)?.clone(),
                equal,
                operand_type,
                arguments,
            },
            effects,
        ))
    }

    /// [TYPE-5] every table operation outside the closed retained-argument
    /// class carries no written type argument, because its operands supply
    /// the selected type. [OP-2] a written one is a hard error citing OP-1.
    pub(in crate::semantic::check) fn reject_written_operation_type_argument(
        &self,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        if self
            .tree
            .first_child_with(node, Production::Targs)?
            .is_some()
        {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        Ok(())
    }

    /// Reads the single written type argument of a retained-argument table
    /// operation. [TYPE-5] keeps these exactly where no operand can supply
    /// the type — here, `finf` and `fnan`, whose rows are nullary.
    pub(in crate::semantic::check) fn retained_operation_type_argument(
        &self,
        node: NodeId,
        function: &FunctionSignature,
    ) -> Result<CheckedType, CheckStop> {
        let targs = self
            .tree
            .first_child_with(node, Production::Targs)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Type5,
                    node,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let targs = self.tree.children_with(targs, Production::Targ)?;
        if targs.len() != 1 {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        let ty = self
            .tree
            .first_child_with(targs[0], Production::Type)?
            .ok_or_else(|| {
                self.issue_value(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation)
            })?;
        self.parse_type_with(ty, &function.substitution)
    }

    /// The [GRAM-11] half of the old type-argument reader: a table operation
    /// takes positional atom operands, never named arguments.
    pub(in crate::semantic::check) fn reject_named_operation_arguments(
        &self,
        node: NodeId,
        spelling: &str,
    ) -> Result<(), CheckStop> {
        if self
            .tree
            .first_child_with(node, Production::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: spelling.to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        Ok(())
    }

    pub(in crate::semantic::check) fn reject_region_bearing_storage_operation_argument(
        &self,
        node: NodeId,
        spelling: &str,
        function: &FunctionSignature,
        expected_argument_count: usize,
        type_argument_index: usize,
    ) -> Result<(), CheckStop> {
        if self
            .tree
            .first_child_with(node, Production::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: spelling.to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        let Some(arguments) = self.tree.first_child_with(node, Production::Targs)? else {
            return Ok(());
        };
        let arguments = self.tree.children_with(arguments, Production::Targ)?;
        if arguments.len() != expected_argument_count {
            return Ok(());
        }
        let argument = *arguments
            .get(type_argument_index)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if let Some(ty) = self.tree.first_child_with(argument, Production::Type)? {
            self.reject_region_bearing_storage_type(ty, &function.substitution)?;
        }
        Ok(())
    }

    /// Checks [GRAM-11] named arguments at every call to a system operation.
    ///
    /// [GRAM-11] applies the [GRAM-8] discipline to calls: a `call` whose
    /// callee resolves to an admitted system operation [SYS-1] writes its
    /// value arguments as a `fieldinit_list` whose IDENTs equal the callee's
    /// [SYS-2] declared parameter names in declared order, and positional
    /// operands are not admitted at all. A missing, extra, repeated,
    /// misspelled, or out-of-order name is a hard error citing GRAM-11 and the
    /// callee's parameter list.
    ///
    /// The judgment runs whole-unit on resolved facts because the rest of a
    /// system call's semantic path is still an unsupported capability: an
    /// unsupported capability establishes no source violation [DIAG-1], so it
    /// must not swallow the argument-spelling rejection this checker can
    /// already establish. Region `targs`, argument types, modes, effects, and
    /// lowering stay outside this judgment.
    pub(in crate::semantic::check) fn check_system_call_arguments(&self) -> Result<(), CheckStop> {
        for usage in self.resolved.lexical_uses() {
            if usage.role() != LexicalUseRole::IdentifierCallee {
                continue;
            }
            let ResolvedTarget::System(id) = usage.target() else {
                continue;
            };
            let Some(crate::SystemEntity::Operation(operation)) =
                crate::system_entity(id, self.inventory())
            else {
                continue;
            };
            let callee = self
                .tree
                .node_with_path(usage.origin().node())
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let call = self
                .tree
                .parent(callee)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.check_system_call_argument_names(call, operation)?;
        }
        Ok(())
    }

    fn check_system_call_argument_names(
        &self,
        call: NodeId,
        operation: &'static crate::SystemOperation,
    ) -> Result<(), CheckStop> {
        let invalid = || SemanticIssueKind::InvalidNamedArguments {
            callee: operation.spelling.to_owned(),
            declared_parameters: operation
                .parameters
                .iter()
                .map(|parameter| parameter.name.to_owned())
                .collect(),
        };
        let fields = match self
            .tree
            .first_child_with(call, Production::FieldinitList)?
        {
            Some(list) => self.tree.children_with(list, Production::Fieldinit)?,
            None => Vec::new(),
        };
        if self
            .tree
            .first_child_with(call, Production::AtomList)?
            .is_some()
            || fields.len() != operation.parameters.len()
        {
            return self.issue_node(SemanticRule::Gram11, call, invalid());
        }
        for (field, parameter) in fields.into_iter().zip(operation.parameters) {
            if self.identifier(field)? != parameter.name {
                return self.issue_node(SemanticRule::Gram11, field, invalid());
            }
        }
        Ok(())
    }

    fn invalid_named_arguments(signature: &FunctionSignature) -> SemanticIssueKind {
        SemanticIssueKind::InvalidNamedArguments {
            callee: signature.name.clone(),
            declared_parameters: signature
                .parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect(),
        }
    }

    pub(in crate::semantic::check) fn operation_atoms(
        &self,
        node: NodeId,
        expected: usize,
    ) -> Result<Vec<NodeId>, CheckStop> {
        let Some(list) = self.tree.first_child_with(node, Production::AtomList)? else {
            if expected == 0 {
                return Ok(Vec::new());
            }
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        };
        let atoms = self.tree.children_with(list, Production::Atom)?;
        if atoms.len() < expected {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        if atoms.len() > expected {
            return self.issue_node(
                SemanticRule::Op1,
                atoms[expected],
                SemanticIssueKind::InvalidOperation,
            );
        }
        Ok(atoms)
    }
}
