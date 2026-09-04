//! Call typing for the [BLK-0] kernel declaration domain.
//!
//! A kernel-domain row is checked exactly as a source function call is —
//! named [GRAM-11] arguments in declared order, per-argument written
//! arguments [BLK-0], borrow formation and overlap checking, and [EFF-2]
//! call-boundary effect projection — except that its signature is the
//! compiler-owned record rather than a source declaration, and that its
//! declared requirement list is submitted at the call under [MSR-4] instead of
//! being installed from the source inventory.
//!
//! Nothing here is keyed on a spelling. The record's own shapes decide which
//! parameter supplies which of the row's type, const and region parameters,
//! and the [`KernelRow`] discriminant selects only what the record itself
//! cannot say.

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, KernelRow, Production, SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::super::goal::{ConcreteGoal, GoalDatum, GoalExpression, GoalOperation};
use super::super::super::super::kernel::{
    KernelComparison, KernelConst, KernelGenericKind, KernelMode, KernelOperand, KernelPlace,
    KernelShape, KernelSignature, kernel_signature,
};
use super::super::super::super::model::{
    CheckedConst, CheckedExpression, CheckedIntegerOperation, CheckedKernelInstance,
    CheckedMeasure, CheckedType, CheckedValue, IntegerType, MeasuredKind,
};
use super::super::super::borrows::{AccessKind, BorrowInfo, BorrowKind, places_overlap};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PendingNominal, TypedExpression,
};
use super::user::ModeExpectation;

/// What one call has fixed of a row's own type, const and region parameters
/// while its written arguments and its operands are read in order.
#[derive(Clone, Copy, Default)]
struct PartialInstance {
    element: Option<CheckedType>,
    run: Option<CheckedType>,
    capacity: Option<CheckedConst>,
    bytes: Option<CheckedConst>,
    align: Option<CheckedConst>,
    region: Option<DeclarationId>,
}

impl PartialInstance {
    fn set_constant(&mut self, which: KernelConst, value: CheckedConst) {
        match which {
            KernelConst::Capacity => self.capacity = Some(value),
            KernelConst::Bytes => self.bytes = Some(value),
            KernelConst::Align => self.align = Some(value),
        }
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_kernel_call(
        &self,
        node: NodeId,
        operation_index: u8,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let record = crate::KERNEL_OPERATIONS
            .get(usize::from(operation_index))
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let signature = kernel_signature(record.row);
        // TEMPORARY capability stop, judged before any operand is read: the
        // four store-backed rows of [BLK-2] need the bump take, the extent
        // reservation and the general store's provider value, none of which
        // this version lowers. It is an explicit unsupported capability and
        // never a source rejection [BLK-0].
        if matches!(
            record.row,
            KernelRow::SeqArena
                | KernelRow::SeqArenaProved
                | KernelRow::SeqHeap
                | KernelRow::ArenaFrame
        ) {
            return self.unsupported(crate::UnsupportedSemanticFeature::ContainerRuntime, node);
        }
        let mut instance = self.kernel_written_arguments(node, signature, record, function)?;

        let fields = if let Some(list) = self
            .tree
            .first_child_with(node, Production::FieldinitList)?
        {
            self.tree.children_with(list, Production::Fieldinit)?
        } else {
            Vec::new()
        };
        if self
            .tree
            .first_child_with(node, Production::AtomList)?
            .is_some()
            || fields.len() != signature.parameters.len()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                Self::invalid_kernel_arguments(record),
            );
        }

        let call = self.tree.path(node)?.clone();
        let mut arguments = Vec::with_capacity(fields.len());
        let mut argument_nodes = Vec::with_capacity(fields.len());
        let mut goal_arguments = Vec::with_capacity(fields.len());
        let mut checked_borrows = Vec::with_capacity(fields.len());
        let mut argument_holders = Vec::with_capacity(fields.len());
        let mut state_origins = Vec::with_capacity(fields.len());
        let mut argument_places = Vec::with_capacity(fields.len());
        let mut call_scoped_borrows: Vec<BorrowInfo> = Vec::new();
        let mut effects = EffectSet::NONE;

        for (ordinal, (field, parameter)) in
            fields.into_iter().zip(signature.parameters).enumerate()
        {
            if self.identifier(field)? != parameter.name {
                return self.issue_node(
                    SemanticRule::Gram11,
                    field,
                    Self::invalid_kernel_arguments(record),
                );
            }
            let atom = self
                .tree
                .first_child_with(field, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let explicit_borrow = self
                .tree
                .first_child_with(atom, Production::BorrowExpr)?
                .is_some();
            let argument =
                self.check_call_argument_atom(function, atom, bindings, loop_depth, true, false)?;
            for access in &argument.accesses {
                for borrow in &call_scoped_borrows {
                    if places_overlap(&access.place, &borrow.place)
                        && match access.kind {
                            AccessKind::Read => borrow.kind == BorrowKind::Unique,
                            AccessKind::Write
                            | AccessKind::Move
                            | AccessKind::SharedBorrow
                            | AccessKind::UniqueBorrow => true,
                        }
                    {
                        return self.issue_node(
                            SemanticRule::Own12,
                            atom,
                            SemanticIssueKind::BorrowConflict,
                        );
                    }
                }
            }
            // A parameter whose shape supplies the row's own parameters reads
            // them off the actual; every other position is checked against
            // the shape the instance already fixed [BLK-0].
            let expected_type =
                self.kernel_parameter_type(parameter.shape, &mut instance, &argument, atom)?;
            if argument.expression.ty() != expected_type {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        self.checked_type_name(expected_type)?,
                        self.checked_type_name(argument.expression.ty())?,
                    ),
                );
            }
            let expectation = match parameter.mode {
                KernelMode::Own => ModeExpectation::Own,
                KernelMode::Unique => ModeExpectation::Borrow {
                    kind: BorrowKind::Unique,
                    region: None,
                },
            };
            let (passed_borrow, expected_mode) =
                self.call_argument_borrow(expectation, &argument, atom)?;
            state_origins.push(self.state_origins_of_value(&argument, bindings)?);
            argument_places.push(
                argument
                    .accesses
                    .iter()
                    .map(|access| access.place.clone())
                    .collect::<Vec<_>>(),
            );
            let ordinal =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            goal_arguments.push(self.call_goal_argument(
                function.id,
                &call,
                ordinal,
                atom,
                expected_mode,
                expected_type,
                &argument,
                passed_borrow.as_ref(),
                bindings,
            )?);
            argument_nodes.push(self.tree.path(atom)?.clone());
            if explicit_borrow && let Some(borrow) = &argument.borrow {
                call_scoped_borrows.push(borrow.clone());
            }
            checked_borrows.push(passed_borrow);
            argument_holders.push(argument.holder);
            effects = effects.union(argument.effects);
            arguments.push(argument.expression);
        }
        let no_slices = vec![None; checked_borrows.len()];
        self.check_call_borrow_overlap(node, &checked_borrows, &no_slices)?;
        self.project_kernel_call_effects(
            node,
            signature,
            &checked_borrows,
            &argument_holders,
            &state_origins,
            &argument_places,
            function,
            bindings,
            &mut effects,
        )?;

        let instance = self.complete_kernel_instance(node, record, instance)?;
        let result = self.kernel_result_type(node, record, signature, &instance)?;
        let requirements = self.kernel_requirements(signature, &instance, &goal_arguments)?;

        Ok(TypedExpression::owned(
            CheckedExpression::KernelCall {
                operation: operation_index,
                row: record.row,
                call,
                instance: Box::new(instance),
                argument_nodes,
                arguments,
                goal_arguments,
                requirements,
                result,
            },
            effects,
        ))
    }

    /// [BLK-0]'s per-argument written-argument judgment.
    ///
    /// A row's type, const and region parameter is written exactly when no
    /// operand of that row supplies it, which the record states per parameter;
    /// a written argument the criterion does not require, or a missing one it
    /// does, is a hard error citing BLK-0 at the `call`, naming the operation.
    fn kernel_written_arguments(
        &self,
        node: NodeId,
        signature: &KernelSignature,
        record: &crate::KernelOperation,
        function: &FunctionSignature,
    ) -> Result<PartialInstance, CheckStop> {
        let written: Vec<_> = signature
            .generics
            .iter()
            .filter(|generic| !generic.supplied)
            .collect();
        let arguments = match self.tree.first_child_with(node, Production::Targs)? {
            Some(targs) => self.tree.children_with(targs, Production::Targ)?,
            None => Vec::new(),
        };
        if arguments.len() != written.len() {
            return self.issue_node(
                SemanticRule::Blk0,
                node,
                SemanticIssueKind::type_mismatch(
                    Self::written_kernel_arguments(record, &written),
                    crate::semantic::written_count(arguments.len(), "argument"),
                ),
            );
        }
        let mut instance = PartialInstance::default();
        for (generic, argument) in written.iter().zip(arguments) {
            match generic.kind {
                KernelGenericKind::Type => {
                    let Some(ty) = self.tree.first_child_with(argument, Production::Type)? else {
                        return self.issue_node(
                            SemanticRule::Blk0,
                            argument,
                            SemanticIssueKind::type_mismatch(
                                "a type in this written-argument position",
                                "an argument that is not a type",
                            ),
                        );
                    };
                    instance.element = Some(self.parse_type_with(ty, &function.substitution)?);
                }
                KernelGenericKind::Const(which) => {
                    let Some(value) = self.tree.first_child_with(argument, Production::Const)?
                    else {
                        return self.issue_node(
                            SemanticRule::Blk0,
                            argument,
                            SemanticIssueKind::type_mismatch(
                                "a const argument in this written-argument position",
                                "an argument that is not a const",
                            ),
                        );
                    };
                    instance.set_constant(
                        which,
                        self.parse_const_expression_with(value, &function.substitution)?,
                    );
                }
                KernelGenericKind::Region => {
                    if self
                        .tree
                        .first_child_with(argument, Production::Type)?
                        .is_some()
                        || self
                            .tree
                            .first_child_with(argument, Production::Const)?
                            .is_some()
                    {
                        return self.issue_node(
                            SemanticRule::Blk0,
                            argument,
                            SemanticIssueKind::type_mismatch(
                                "a region argument in this written-argument position",
                                "an argument that does not name a region",
                            ),
                        );
                    }
                    let usage = self.use_at(argument, crate::LexicalUseRole::TypeArgumentRegion)?;
                    let crate::ResolvedTarget::Source {
                        declaration,
                        class: crate::DeclarationClass::Region,
                    } = usage.target()
                    else {
                        return self.issue_node(
                            SemanticRule::Blk0,
                            argument,
                            SemanticIssueKind::type_mismatch(
                                "a region argument in this written-argument position",
                                "an argument that does not name a region",
                            ),
                        );
                    };
                    instance.region = Some(declaration);
                }
            }
        }
        Ok(instance)
    }

    /// The type one value parameter position requires, reading the row's own
    /// parameters off an operand whose shape supplies them [BLK-0].
    fn kernel_parameter_type(
        &self,
        shape: KernelShape,
        instance: &mut PartialInstance,
        argument: &TypedExpression,
        atom: NodeId,
    ) -> Result<CheckedType, CheckStop> {
        match shape {
            KernelShape::U64 => Ok(CheckedType::Integer(IntegerType::U64)),
            KernelShape::Element => instance
                .element
                .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into()),
            // `V` is supplied by the `vector` operand and never written
            // [BLK-3]; its admitted arguments are exactly the two runs, and
            // its element type is that run's own.
            KernelShape::Run => {
                if let Some(run) = instance.run {
                    return Ok(run);
                }
                let actual = argument.expression.ty();
                let element = match actual {
                    CheckedType::FixedVector { element, .. }
                    | CheckedType::Vector { element, .. } => element,
                    _ => {
                        return self.issue_node(
                            SemanticRule::Type5,
                            atom,
                            SemanticIssueKind::type_mismatch(
                                "a run: `FixedVector<T, n>` or `Vector<'s, T>`",
                                self.checked_type_name(actual)?,
                            ),
                        );
                    }
                };
                if let CheckedType::Vector { region, .. } = actual {
                    instance.region = Some(region);
                }
                if let CheckedType::FixedVector { length, .. } = actual {
                    instance.capacity = Some(length);
                }
                instance.run = Some(actual);
                instance.element = Some(element.ty());
                Ok(actual)
            }
            // A provider operand supplies its own store region and, for a
            // bump extent, both of its constants.
            KernelShape::Extent => {
                let actual = argument.expression.ty();
                let CheckedType::Extent {
                    region,
                    bytes,
                    align,
                } = actual
                else {
                    return self.issue_node(
                        SemanticRule::Type5,
                        atom,
                        SemanticIssueKind::type_mismatch(
                            "an `Arena<'s, bytes, align>` provider",
                            self.checked_type_name(actual)?,
                        ),
                    );
                };
                instance.region = Some(region);
                instance.bytes = Some(bytes);
                instance.align = Some(align);
                Ok(actual)
            }
            KernelShape::Heap => {
                let actual = argument.expression.ty();
                let CheckedType::Heap { region } = actual else {
                    return self.issue_node(
                        SemanticRule::Type5,
                        atom,
                        SemanticIssueKind::type_mismatch(
                            "a `Heap<'s>` provider",
                            self.checked_type_name(actual)?,
                        ),
                    );
                };
                instance.region = Some(region);
                Ok(actual)
            }
            KernelShape::FixedVector | KernelShape::Vector | KernelShape::OptionVector => {
                Err(SemanticCompilerFailure::InvalidResolution.into())
            }
        }
    }

    /// The complete instance, once every written argument and every operand
    /// has been read.
    fn complete_kernel_instance(
        &self,
        node: NodeId,
        record: &crate::KernelOperation,
        instance: PartialInstance,
    ) -> Result<CheckedKernelInstance, CheckStop> {
        let element = instance
            .element
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let element_ceiling = self.layout_ceiling(element, node)?;
        // [BLK-2] each arena row requires `align >= align_ceiling(T)` as a
        // compile-time comparison of two constants, which is what makes the
        // bump cursor a multiple of `align` at every program point.
        if let Some(CheckedConst::Value(align)) = instance.align
            && align < element_ceiling.align
        {
            return self.issue_node(
                SemanticRule::Blk0,
                node,
                SemanticIssueKind::type_mismatch(
                    "a store whose alignment is at least the element's own layout ceiling",
                    format!(
                        "the operation `{}` over an element of alignment {}",
                        record.spelling, element_ceiling.align
                    ),
                ),
            );
        }
        Ok(CheckedKernelInstance {
            element,
            run: instance.run,
            capacity: instance.capacity,
            bytes: instance.bytes,
            align: instance.align,
            region: instance.region,
            element_ceiling,
        })
    }

    /// The row's declared result type, or the compiler-owned result-list
    /// nominal that carries its ordered result list [CALL-4].
    fn kernel_result_type(
        &self,
        node: NodeId,
        record: &crate::KernelOperation,
        signature: &KernelSignature,
        instance: &CheckedKernelInstance,
    ) -> Result<CheckedType, CheckStop> {
        let mut results = Vec::with_capacity(signature.results.len());
        for (result, name) in signature.results.iter().zip(record.results) {
            results.push((
                (*name).to_owned(),
                self.kernel_shape_type(node, result.shape, instance)?,
            ));
        }
        let [(_, single)] = results.as_slice() else {
            let Some(id) = self.result_list_nominal(&results) else {
                self.pending_nominals
                    .borrow_mut()
                    .push(PendingNominal::ResultList(results));
                return Err(CheckStop::DeferredNominal);
            };
            return Ok(CheckedType::Nominal(id));
        };
        Ok(*single)
    }

    /// One record shape at one resolved instance.
    fn kernel_shape_type(
        &self,
        node: NodeId,
        shape: KernelShape,
        instance: &CheckedKernelInstance,
    ) -> Result<CheckedType, CheckStop> {
        let region = || {
            instance
                .region
                .ok_or(SemanticCompilerFailure::InvalidResolution)
        };
        Ok(match shape {
            KernelShape::U64 => CheckedType::Integer(IntegerType::U64),
            KernelShape::Element => instance.element,
            KernelShape::Run => instance
                .run
                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            KernelShape::FixedVector => CheckedType::FixedVector {
                element: self.kernel_element(instance.element, node)?,
                length: instance
                    .capacity
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            },
            KernelShape::Vector => CheckedType::Vector {
                region: region()?,
                element: self.kernel_element(instance.element, node)?,
            },
            KernelShape::OptionVector => {
                let payload = CheckedType::Vector {
                    region: region()?,
                    element: self.kernel_element(instance.element, node)?,
                };
                CheckedType::Nominal(
                    self.prelude_nominal(super::super::super::PreludeType::Option(payload))?,
                )
            }
            KernelShape::Extent => CheckedType::Extent {
                region: region()?,
                bytes: instance
                    .bytes
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                align: instance
                    .align
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            },
            KernelShape::Heap => CheckedType::Heap { region: region()? },
        })
    }

    /// The element type of a run at one instance [BLK-1].
    ///
    /// A run's element domain is the flat-element domain a `buffer`
    /// formation already has in this version; an element type outside it is
    /// an explicit unsupported capability and never a source rejection.
    fn kernel_element(
        &self,
        element: CheckedType,
        node: NodeId,
    ) -> Result<super::super::super::super::model::CheckedFlatElement, CheckStop> {
        match self.flat_element(element)? {
            Some(element) => Ok(element),
            None => self.unsupported(crate::UnsupportedSemanticFeature::CompositeValues, node),
        }
    }

    /// The row's declared requirement list, instantiated at this call.
    ///
    /// Each clause is one ordinary comparison over [ENT-2] terms, so the goal
    /// this builds is the same shape a source `requires` produces and is
    /// judged by exactly the same [MSR-4] disposition.
    fn kernel_requirements(
        &self,
        signature: &KernelSignature,
        instance: &CheckedKernelInstance,
        goal_arguments: &[GoalExpression],
    ) -> Result<Vec<ConcreteGoal>, CheckStop> {
        let mut goals = Vec::with_capacity(signature.requires.len());
        for clause in signature.requires {
            // A clause both of whose operands are compile-time constants is
            // the comparison `complete_kernel_instance` already judged; it
            // submits no runtime obligation.
            let (Some(left), Some(right)) = (
                self.kernel_goal_operand(clause.left.operand, instance, goal_arguments)?,
                self.kernel_goal_operand(clause.right.operand, instance, goal_arguments)?,
            ) else {
                continue;
            };
            let operation = match clause.comparison {
                KernelComparison::Equal => CheckedIntegerOperation::Equal,
                KernelComparison::Less => CheckedIntegerOperation::Less,
                KernelComparison::LessOrEqual => CheckedIntegerOperation::LessEqual,
                KernelComparison::Greater => CheckedIntegerOperation::Greater,
                KernelComparison::GreaterOrEqual => CheckedIntegerOperation::GreaterEqual,
            };
            goals.push(ConcreteGoal::new(GoalExpression::Operation {
                row: GoalOperation::Integer {
                    operation,
                    operand_type: CheckedType::Integer(IntegerType::U64),
                },
                type_arguments: Vec::new(),
                const_arguments: Vec::new(),
                result: CheckedType::Bool,
                arguments: vec![left, right],
            }));
        }
        Ok(goals)
    }

    /// One requirement operand as a caller-side goal expression.
    fn kernel_goal_operand(
        &self,
        operand: KernelOperand,
        instance: &CheckedKernelInstance,
        goal_arguments: &[GoalExpression],
    ) -> Result<Option<GoalExpression>, CheckStop> {
        Ok(match operand {
            KernelOperand::Zero => Some(GoalExpression::Datum(GoalDatum::Literal(
                CheckedValue::Integer {
                    ty: IntegerType::U64,
                    bits: 0,
                },
            ))),
            KernelOperand::Const(which) => {
                let constant = match which {
                    KernelConst::Capacity => instance.capacity,
                    KernelConst::Bytes => instance.bytes,
                    KernelConst::Align => instance.align,
                };
                match constant {
                    Some(CheckedConst::Value(value)) => Some(GoalExpression::Datum(
                        GoalDatum::Literal(CheckedValue::Integer {
                            ty: IntegerType::U64,
                            bits: value,
                        }),
                    )),
                    _ => None,
                }
            }
            KernelOperand::AlignCeiling => Some(GoalExpression::Datum(GoalDatum::Literal(
                CheckedValue::Integer {
                    ty: IntegerType::U64,
                    bits: instance.element_ceiling.align,
                },
            ))),
            KernelOperand::Value(ordinal) => goal_arguments.get(ordinal as usize).cloned(),
            KernelOperand::Measure(measure, place) => {
                let KernelPlace::Parameter(ordinal) = place else {
                    // A requirement names no result and no routed payload.
                    return Ok(None);
                };
                let Some(argument) = goal_arguments.get(ordinal as usize) else {
                    return Ok(None);
                };
                self.kernel_measure_goal(measure, argument)
            }
        })
    }

    /// One measure of one already-captured caller image [MSR-1].
    fn kernel_measure_goal(
        &self,
        measure: CheckedMeasure,
        argument: &GoalExpression,
    ) -> Option<GoalExpression> {
        let (measured, element, constant) = match argument.ty() {
            CheckedType::FixedVector { element, length } => {
                (MeasuredKind::FixedVector, Some(element), Some(length))
            }
            CheckedType::Vector { element, .. } => (MeasuredKind::Vector, Some(element), None),
            CheckedType::Extent { bytes, .. } => (MeasuredKind::Extent, None, Some(bytes)),
            _ => return None,
        };
        Some(GoalExpression::Operation {
            row: GoalOperation::ContainerMeasure {
                measure,
                measured,
                element,
                constant,
            },
            type_arguments: Vec::new(),
            const_arguments: Vec::new(),
            result: CheckedType::Integer(IntegerType::U64),
            arguments: vec![argument.clone()],
        })
    }

    /// [EFF-2] the row's declared effect row projected through its actuals,
    /// exactly as an ordinary call's is: a `&uniq` state operand projects
    /// through its loan, and every operand projects through the resolved
    /// places and state origins of the actual itself.
    #[allow(clippy::too_many_arguments)]
    fn project_kernel_call_effects(
        &self,
        node: NodeId,
        signature: &KernelSignature,
        borrows: &[Option<BorrowInfo>],
        holders: &[Option<DeclarationId>],
        state_origins: &[Option<super::super::super::super::model::CheckedStateOrigins>],
        argument_places: &[Vec<super::super::super::borrows::ResolvedPlace>],
        caller: &FunctionSignature,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        effects: &mut EffectSet,
    ) -> Result<(), CheckStop> {
        for (access, declared) in [
            (AccessKind::Read, signature.effects.reads),
            (AccessKind::Write, signature.effects.writes),
        ] {
            let Some(ordinal) = declared else {
                continue;
            };
            let index = ordinal as usize;
            let parameter = signature
                .parameters
                .get(index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let mut paths = Vec::new();
            if parameter.mode != KernelMode::Own {
                let borrow = borrows
                    .get(index)
                    .and_then(Option::as_ref)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                self.check_loan_access(
                    bindings,
                    holders.get(index).copied().flatten(),
                    &borrow.place,
                    access,
                    node,
                )?;
                paths.extend(self.effect_paths_for_place(&borrow.place, bindings)?);
            }
            for place in argument_places.get(index).into_iter().flatten() {
                paths.push(self.state_path(place, bindings)?);
            }
            if let Some(origins) = state_origins.get(index).and_then(Option::as_ref) {
                if origins.unknown && !self.deriving_result_state_origin.get() {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                for origin in &origins.formals {
                    paths.push(origin.source.clone());
                }
            }
            for path in paths {
                if !caller
                    .parameters
                    .iter()
                    .any(|parameter| parameter.declaration == path.root)
                {
                    continue;
                }
                match access {
                    AccessKind::Read => effects.add_read(path),
                    AccessKind::Write => effects.add_write(path),
                    _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                }
            }
        }
        Ok(())
    }

    fn invalid_kernel_arguments(record: &crate::KernelOperation) -> SemanticIssueKind {
        SemanticIssueKind::InvalidNamedArguments {
            callee: record.spelling.to_owned(),
            declared_parameters: record
                .parameters
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
        }
    }

    fn written_kernel_arguments(
        record: &crate::KernelOperation,
        written: &[&super::super::super::super::kernel::KernelGenericParameter],
    ) -> String {
        let names = written
            .iter()
            .map(|generic| generic.name)
            .collect::<Vec<_>>()
            .join(", ");
        if names.is_empty() {
            return format!("no written argument, which `{}` declares", record.spelling);
        }
        format!(
            "the written arguments `{}` of the operation `{}`",
            names, record.spelling
        )
    }
}
