use std::collections::HashMap;

mod buffers;
mod loops;
mod probe;
mod results;
mod slices;
mod split;
mod storage;

use crate::CheckedProgram;
use crate::NodePath;
use crate::semantic::CheckedSetTarget;
use crate::semantic::{
    BindingId, CheckedArrayRoot, CheckedArraySetTarget, CheckedCommitValues, CheckedConstructor,
    CheckedDrop, CheckedEntryForm, CheckedExpression, CheckedMatchArm, CheckedMeasure, CheckedMode,
    CheckedNominalKind, CheckedParameter, CheckedProgramData, CheckedProjectedDrop,
    CheckedStatement, CheckedValue, FunctionPermissions, MeasureCell, MeasuredKind,
};

use super::*;
use loops::LoopTarget;
use split::{Synthesis, SynthesisCell};
use storage::collect_addressed_bindings;

pub fn lower_checked<'classified, 'lexed, 'source>(
    checked: CheckedProgram<'classified, 'lexed, 'source>,
    overlap: OverlapLowering,
) -> Result<IrProgram<'classified, 'lexed, 'source>, LoweringFailure> {
    let entry = lower_entry(&checked.data.entry);
    let nominals = lower_nominals(&checked.data)?;
    let constants = lower_constants(&checked.data)?;
    // Each function's declared IR result carries its result *mode*: a borrow
    // of addressed content is an address. A call site must produce exactly
    // the callee's declared result type, so the declared results are computed
    // once and consulted at every `UserCall` [OWN-2, TYPE-7].
    let function_results = checked
        .data
        .functions
        .iter()
        .map(|function| {
            lower_borrow_mode_type(
                function.result_mode,
                lower_type(function.result)?,
                &nominals,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    // The [PAR-1 candidate] permission table, read exactly as the checker
    // produced it. Lowering selects which permitted groups it can actualize
    // and never widens one — and reads the table at all only when this
    // compilation asked for overlap lowering, so the default emits the same
    // module a compiler with no such lowering emits.
    let permission = match overlap {
        OverlapLowering::On | OverlapLowering::Completion => Some(&checked.data.permission),
        OverlapLowering::Off => None,
    };
    // Where a synthesized function's ordinal starts. A [PAR-2] split appends
    // its two halves after every source function, so nothing renumbers and a
    // `Call` still indexes one flat table.
    let source_functions = u32::try_from(checked.data.functions.len())
        .map_err(|_| LoweringFailure::CounterOverflow)?;
    let synthesis = SynthesisCell::new(Synthesis::new(source_functions));
    let context = LoweringContext {
        nominals: &nominals,
        constants: &constants,
        function_results: &function_results,
        synthesis: &synthesis,
    };
    let mut functions = checked
        .data
        .functions
        .iter()
        .map(|function| {
            lower_function(
                function,
                context,
                permission.and_then(|table| table.of(function.id)),
                overlap,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (synthesized, actualization) = synthesis.into_inner().finish()?;
    functions.extend(synthesized);
    split::assign_weights(&mut functions);
    Ok(IrProgram {
        main: checked.data.main.0,
        _checked: checked,
        nominals,
        constants,
        functions,
        entry,
        actualization,
    })
}

/// What every builder of one lowering shares: the program-wide tables it reads
/// and the cell the synthesized halves of a [PAR-2] split are filed in.
///
/// These travel as one value because a chunk is built by a second builder
/// created inside the first, and a chunk containing another permitted loop
/// creates a third; passing the shared half by name keeps that recursion from
/// growing an argument list at every level.
#[derive(Clone, Copy)]
struct LoweringContext<'program> {
    nominals: &'program [IrNominal],
    constants: &'program [IrGlobalConstant],
    /// Every source function's declared IR result, indexed by [`FunctionId`].
    function_results: &'program [IrType],
    synthesis: &'program SynthesisCell,
}

/// Carries the [FN-7] entry form into the IR.
///
/// [PROG-3] starts an instance by supplying exactly the standard inputs the
/// entry declares and invoking it once. Target-independent lowering records
/// which inputs those are; constructing the values and mapping the returned
/// `ExitStatus` belongs to the target stage.
fn lower_entry(entry: &CheckedEntryForm) -> IrEntry {
    IrEntry::Command {
        inputs: entry.inputs.clone(),
    }
}

fn lower_scalar_constant(value: &CheckedValue) -> Result<IrConstant, LoweringFailure> {
    match value {
        CheckedValue::Unit => Ok(IrConstant::Unit),
        CheckedValue::Bool(value) => Ok(IrConstant::Bool(*value)),
        CheckedValue::Integer { ty, bits } => Ok(IrConstant::Integer {
            ty: lower_type(crate::semantic::CheckedType::Integer(*ty))?,
            bits: *bits,
        }),
        CheckedValue::Float { ty, bits } => Ok(IrConstant::Float {
            ty: lower_type(crate::semantic::CheckedType::Float(*ty))?,
            bits: *bits,
        }),
        CheckedValue::ConstGeneric { .. }
        | CheckedValue::NumericIdentity { .. }
        | CheckedValue::Array { .. }
        | CheckedValue::Struct { .. } => Err(LoweringFailure::InvalidCheckedProgram),
    }
}

fn lower_global_value(value: &CheckedValue) -> Result<IrGlobalValue, LoweringFailure> {
    match value {
        CheckedValue::Array { elements, .. } => Ok(IrGlobalValue::Array(
            elements
                .iter()
                .map(lower_scalar_constant)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        CheckedValue::Struct { fields, .. } => Ok(IrGlobalValue::Struct(
            fields
                .iter()
                .map(lower_global_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        scalar => Ok(IrGlobalValue::Scalar(lower_scalar_constant(scalar)?)),
    }
}

fn lower_constants(data: &CheckedProgramData) -> Result<Vec<IrGlobalConstant>, LoweringFailure> {
    data.constants
        .iter()
        .enumerate()
        .map(|(index, constant)| {
            if constant.id.0 as usize != index || constant.value.ty() != constant.ty {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            Ok(IrGlobalConstant {
                id: IrConstantId(constant.id.0),
                name: constant.name.clone(),
                ty: lower_type(constant.ty)?,
                value: lower_global_value(&constant.value)?,
            })
        })
        .collect()
}

fn lower_nominals(data: &CheckedProgramData) -> Result<Vec<IrNominal>, LoweringFailure> {
    data.nominals
        .get(..data.executable_nominal_count)
        .ok_or(LoweringFailure::InvalidCheckedProgram)?
        .iter()
        .enumerate()
        .map(|(index, nominal)| {
            if nominal.id.0 as usize != index {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            let identity = match &nominal.kind {
                CheckedNominalKind::SystemResource { nominal } => {
                    IrNominalIdentity::System(*nominal)
                }
                CheckedNominalKind::Enum { variants }
                    if matches!(
                        variants.as_slice(),
                        [ok, err]
                            if ok.constructor
                                == CheckedConstructor::Prelude(
                                    crate::PreludeDeclarationId::new(11)
                                )
                                && err.constructor
                                    == CheckedConstructor::Prelude(
                                        crate::PreludeDeclarationId::new(13)
                                    )
                    ) =>
                {
                    IrNominalIdentity::PreludeResult
                }
                CheckedNominalKind::Enum { variants } if !variants.is_empty() => {
                    let mut owner = None;
                    for variant in variants {
                        let CheckedConstructor::System(declaration) = variant.constructor else {
                            owner = None;
                            break;
                        };
                        let constructor =
                            crate::system_constructor_index(declaration, crate::Inventory::ACTIVE)
                                .and_then(|index| {
                                    crate::SYSTEM_CONSTRUCTORS.get(usize::from(index))
                                })
                                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                        match owner {
                            Some(existing) if existing != constructor.owner => {
                                return Err(LoweringFailure::InvalidCheckedProgram);
                            }
                            Some(_) => {}
                            None => owner = Some(constructor.owner),
                        }
                    }
                    owner.map_or(IrNominalIdentity::Ordinary, IrNominalIdentity::System)
                }
                CheckedNominalKind::Struct { .. }
                | CheckedNominalKind::Enum { .. }
                | CheckedNominalKind::Box { .. }
                | CheckedNominalKind::Arena { .. }
                | CheckedNominalKind::ArenaStorage => IrNominalIdentity::Ordinary,
            };
            let kind = match &nominal.kind {
                CheckedNominalKind::Struct { fields } => IrNominalKind::Struct {
                    fields: fields
                        .iter()
                        .map(|field| {
                            Ok(IrField {
                                ty: lower_type(field.ty)?,
                            })
                        })
                        .collect::<Result<Vec<_>, LoweringFailure>>()?,
                },
                CheckedNominalKind::Enum { variants } => IrNominalKind::Enum {
                    variants: variants
                        .iter()
                        .map(|variant| {
                            Ok(IrVariant {
                                tag: variant.tag,
                                fields: variant
                                    .fields
                                    .iter()
                                    .map(|field| {
                                        Ok(IrField {
                                            ty: lower_type(field.ty)?,
                                        })
                                    })
                                    .collect::<Result<Vec<_>, LoweringFailure>>()?,
                            })
                        })
                        .collect::<Result<Vec<_>, LoweringFailure>>()?,
                },
                CheckedNominalKind::Box { referent } => IrNominalKind::Box {
                    referent: lower_type(*referent)?,
                },
                CheckedNominalKind::Arena { content, .. } => IrNominalKind::Arena {
                    content: lower_type(*content)?,
                },
                CheckedNominalKind::ArenaStorage => IrNominalKind::ArenaStorage,
                // The opaque type's own [SYS-2] identity, [SYS-5] release
                // action and row, and [HOST-3] backing class travel into the
                // IR unchanged. A target representation for it is target
                // qualification's business, not this stage's.
                CheckedNominalKind::SystemResource { nominal } => IrNominalKind::SystemResource(
                    crate::system_resource_contract(*nominal)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?,
                ),
            };
            Ok(IrNominal {
                id: IrNominalId(
                    u32::try_from(index).map_err(|_| LoweringFailure::CounterOverflow)?,
                ),
                identity,
                kind,
            })
        })
        .collect()
}

fn lower_function<'program>(
    function: &crate::semantic::CheckedFunction,
    context: LoweringContext<'program>,
    permissions: Option<&'program FunctionPermissions>,
    overlap: OverlapLowering,
) -> Result<IrFunction, LoweringFailure> {
    let uninhabited = matches!(
        function.body_disposition,
        crate::semantic::CheckedBodyDisposition::Uninhabited { .. }
    );
    // An uninhabited body must not be traversed even for storage planning.
    let addressed_bindings = if uninhabited {
        std::collections::HashSet::new()
    } else {
        collect_addressed_bindings(function)
    };
    let result = *context
        .function_results
        .get(function.id.0 as usize)
        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
    // The whole permission table of this function, and only when this
    // compilation asked for actualization: with none, a permitted loop lowers
    // exactly as it did before the split existed and no group is actualized.
    let mut builder = IrBuilder::new(
        context,
        result,
        addressed_bindings,
        permissions,
        overlap,
        &function.symbol,
    )?;
    for parameter in &function.parameters {
        let ty = lower_parameter_type(parameter, context.nominals)?;
        let value = builder.new_parameter(ty)?;
        if builder.bindings.insert(parameter.binding, value).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        builder.promote_binding_if_needed(parameter.binding)?;
    }
    if uninhabited {
        builder.terminate(IrTerminator::Unreachable)?;
    } else {
        builder.lower_statements(&function.body, None)?;
    }
    builder.materialize_staged_driver_plan()?;
    let overlaps = builder.overlaps();
    let completion_steps = builder.completion_steps();
    builder.finish(
        function.symbol.clone(),
        overlaps,
        completion_steps,
        None,
        function.target_action,
    )
}

fn lower_parameter_type(
    parameter: &CheckedParameter,
    nominals: &[IrNominal],
) -> Result<IrType, LoweringFailure> {
    lower_borrow_mode_type(parameter.mode, lower_type(parameter.ty)?, nominals)
}

/// The representation a borrow-mode value carries.
///
/// A borrow of directly stored content is the address of that storage; a
/// descriptor or opaque handle is already its own borrow and keeps its value
/// type [OWN-2, SYS-2].
fn lower_borrow_mode_type(
    mode: CheckedMode,
    ty: IrType,
    nominals: &[IrNominal],
) -> Result<IrType, LoweringFailure> {
    if mode == CheckedMode::Own {
        return Ok(ty);
    }
    let Some(referent) = IrAddressed::of(ty) else {
        return Ok(ty);
    };
    if let IrAddressed::Nominal(nominal) = referent
        && !matches!(
            nominals
                .get(nominal.index())
                .ok_or(LoweringFailure::InvalidCheckedProgram)?
                .kind,
            IrNominalKind::Struct { .. } | IrNominalKind::Enum { .. }
        )
    {
        return Ok(ty);
    }
    Ok(IrType::Address(referent))
}

struct BuildingBlock {
    parameters: Vec<(IrValueId, IrType)>,
    instructions: Vec<IrInstruction>,
    terminator: Option<IrTerminator>,
}

fn block_successors(block: &BuildingBlock) -> Vec<IrBlockId> {
    match block.terminator.as_ref() {
        Some(IrTerminator::Jump { target, .. }) => vec![*target],
        Some(IrTerminator::Match { targets, .. }) => {
            targets.iter().map(|target| target.block()).collect()
        }
        Some(IrTerminator::Return { .. } | IrTerminator::Unreachable) | None => Vec::new(),
    }
}

struct IrBuilder<'program> {
    nominals: &'program [IrNominal],
    constants: &'program [IrGlobalConstant],
    bindings: HashMap<BindingId, IrValueId>,
    parameters: Vec<(IrValueId, IrType)>,
    values: Vec<IrType>,
    blocks: Vec<BuildingBlock>,
    current: Option<IrBlockId>,
    loops: Vec<LoopTarget>,
    result: IrType,
    addressed_bindings: std::collections::HashSet<BindingId>,
    /// Every function's declared IR result, indexed by [`FunctionId`], so a
    /// call defines exactly the callee's declared result type — an address
    /// for a borrow of addressed content [OWN-2, TYPE-7].
    function_results: &'program [IrType],
    /// For each statement holding exactly one named-function call in call
    /// position — a `let` right-hand side or a `match` scrutinee — the block
    /// the call's definition landed in and the value it defined. The
    /// permission table names its sites by call occurrence, which is the one
    /// identity every written call position has, so this is how a permitted
    /// group is found in the IR.
    call_results: HashMap<NodePath, (IrBlockId, IrValueId)>,
    /// The permission table of the source function this body belongs to: the
    /// [PAR-1] groups its statements may overlap and the [PAR-2] verdict of
    /// each of its counted loops.
    ///
    /// `None` when this compilation actualizes nothing, and inside a
    /// synthesized splitter, which has no source statement of its own — a
    /// splitter's overlap group is produced by the lowering that built it.
    /// A synthesized *chunk* carries the table, because its body is the loop's
    /// own statements and a pair inside them is permitted exactly as it was
    /// before the loop was split.
    permissions: Option<&'program FunctionPermissions>,
    /// Which subset of the pure permission judgment this compilation may
    /// actualize.
    overlap: OverlapLowering,
    /// The one permitted staged loop whose checked identity has reached this
    /// function's IR. It remains permission-only unless lowering materializes
    /// either the complete one-slot edge or the bounded-batch driver.
    completion_pipeline: Option<IrCompletionPipeline>,
    /// The selected loop's submitted call occurrence. The loop has already
    /// been selected by [`CheckedLoopId`]; this path is only the existing call
    /// identity used to map that cut to its IR value.
    staged_cut: Option<NodePath>,
    /// Where a split's synthesized halves are filed, shared with every builder
    /// this one creates.
    synthesis: &'program SynthesisCell,
    /// The source function this body belongs to, for the actualization ledger.
    function_name: &'program str,
}

#[derive(Clone)]
struct GiveTarget {
    block: IrBlockId,
    result: IrType,
    carried_bindings: Vec<BindingId>,
}

impl<'program> IrBuilder<'program> {
    fn new(
        context: LoweringContext<'program>,
        result: IrType,
        addressed_bindings: std::collections::HashSet<BindingId>,
        permissions: Option<&'program FunctionPermissions>,
        overlap: OverlapLowering,
        function_name: &'program str,
    ) -> Result<Self, LoweringFailure> {
        let LoweringContext {
            nominals,
            constants,
            function_results,
            synthesis,
        } = context;
        let mut builder = Self {
            nominals,
            constants,
            bindings: HashMap::new(),
            parameters: Vec::new(),
            values: Vec::new(),
            blocks: Vec::new(),
            current: None,
            loops: Vec::new(),
            result,
            addressed_bindings,
            function_results,
            call_results: HashMap::new(),
            permissions,
            overlap,
            completion_pipeline: None,
            staged_cut: None,
            synthesis,
            function_name,
        };
        let (entry, parameters) = builder.new_block(&[])?;
        if !parameters.is_empty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        builder.current = Some(entry);
        Ok(builder)
    }

    /// This builder's own shared half, for the builders it creates.
    const fn context(&self) -> LoweringContext<'program> {
        LoweringContext {
            nominals: self.nominals,
            constants: self.constants,
            function_results: self.function_results,
            synthesis: self.synthesis,
        }
    }

    /// Declares one more parameter of the function being built.
    fn new_parameter(&mut self, ty: IrType) -> Result<IrValueId, LoweringFailure> {
        let value = self.new_value(ty)?;
        self.parameters.push((value, ty));
        Ok(value)
    }

    /// Seals the body into a function, refusing an unterminated one.
    fn finish(
        self,
        name: String,
        overlaps: Vec<IrOverlap>,
        completion_steps: Vec<IrCompletionStep>,
        synthesis: Option<IrSynthesis>,
        target_action: crate::TargetAction,
    ) -> Result<IrFunction, LoweringFailure> {
        if self.current.is_some() || self.blocks.iter().any(|block| block.terminator.is_none()) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(IrFunction {
            name,
            parameters: self.parameters,
            result: self.result,
            values: self.values,
            blocks: self
                .blocks
                .into_iter()
                .map(|block| {
                    Ok(IrBlock {
                        parameters: block.parameters,
                        instructions: block.instructions,
                        terminator: block
                            .terminator
                            .ok_or(LoweringFailure::InvalidCheckedProgram)?,
                    })
                })
                .collect::<Result<Vec<_>, LoweringFailure>>()?,
            overlaps,
            completion_pipeline: self.completion_pipeline,
            completion_steps,
            synthesis,
            target_action,
        })
    }

    fn new_value(&mut self, ty: IrType) -> Result<IrValueId, LoweringFailure> {
        let id = IrValueId(
            u32::try_from(self.values.len()).map_err(|_| LoweringFailure::CounterOverflow)?,
        );
        self.values.push(ty);
        Ok(id)
    }

    /// Connects a permitted [PAR-3] verdict to IR by checked loop identity.
    ///
    /// The loop is selected only by `id`. After that selection, `cut` keeps its
    /// existing role as the identity of the permitted call occurrence; it is
    /// never used to recognize a loop or a source shape. The descriptor stays
    /// pending, so this records authority without changing execution.
    fn note_staged_pipeline(
        &mut self,
        id: CheckedLoopId,
        entry: IrBlockId,
        window: IrCompletionWindow,
    ) {
        let cut = self.unique_staged_cut(id);
        let Some(cut) = cut else {
            return;
        };
        match self.completion_pipeline.as_ref() {
            None => {
                self.completion_pipeline = Some(IrCompletionPipeline::pending(id, entry, window));
                self.staged_cut = Some(cut);
            }
            Some(existing) if existing.source_loop() == id => {}
            Some(_) => {}
        }
    }

    /// Returns the cut only when this function has exactly one permitted
    /// staged loop and it is `id`.
    ///
    /// The IR currently stores one pipeline descriptor per function. Making
    /// this choice before lowering prevents an earlier loop from being
    /// transformed and then losing the descriptor when a second permitted
    /// loop is encountered later in the body.
    fn unique_staged_cut(&self, id: CheckedLoopId) -> Option<NodePath> {
        let mut permitted = self
            .permissions?
            .staged
            .iter()
            .filter(|permission| permission.verdict.is_permitted());
        let selected = permitted.next()?;
        if permitted.next().is_some() || selected.id != id {
            return None;
        }
        Some(selected.cut.clone())
    }

    /// Materializes the first driver topology without yet changing execution.
    ///
    /// The selected operation remains an ordinary synchronous `SystemCall` in
    /// this step. Splitting its result dispatch onto a fresh block therefore
    /// changes no value, effect, cleanup, or ordering; it establishes and
    /// records the edge the asynchronous form will later use.
    fn materialize_staged_driver_plan(&mut self) -> Result<(), LoweringFailure> {
        let Some(cut) = self.staged_cut.as_ref() else {
            return Ok(());
        };
        let Some((feeder, result)) = self.call_results.get(cut).copied() else {
            return Ok(());
        };
        let Some(block) = self.blocks.get(feeder.index()) else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let call_is_last = matches!(
            block.instructions.last(),
            Some(IrInstruction::Define {
                result: defined,
                operation: IrOperation::SystemCall { target_action, .. },
                ..
            }) if *defined == result && target_action.may_suspend()
        );
        let Some(IrTerminator::Match {
            scrutinee, targets, ..
        }) = block.terminator.as_ref()
        else {
            return Ok(());
        };
        if !call_is_last || *scrutinee != result {
            return Ok(());
        }

        // Each arm block is created for this dispatch and has no other entry.
        // After the split, that makes the drain dominate every projection of
        // the delayed result. Decline rather than infer this from numbering.
        let targets = targets
            .iter()
            .map(|target| target.block())
            .collect::<Vec<_>>();
        let each_target_is_private = targets.iter().all(|target| {
            self.blocks
                .iter()
                .enumerate()
                .filter(|(_, candidate)| {
                    block_successors(candidate)
                        .iter()
                        .any(|successor| successor == target)
                })
                .map(|(index, _)| index)
                .eq(std::iter::once(feeder.index()))
        });
        if !each_target_is_private {
            return Ok(());
        }

        let original = self
            .blocks
            .get_mut(feeder.index())
            .and_then(|block| block.terminator.take())
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let (drain, parameters) = self.new_block(&[])?;
        if !parameters.is_empty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.blocks
            .get_mut(feeder.index())
            .ok_or(LoweringFailure::InvalidCheckedProgram)?
            .terminator = Some(IrTerminator::Jump {
            target: drain,
            arguments: Vec::new(),
            drops: Vec::new(),
        });
        self.blocks
            .get_mut(drain.index())
            .ok_or(LoweringFailure::InvalidCheckedProgram)?
            .terminator = Some(original);
        self.completion_pipeline
            .as_mut()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?
            .plan_one_slot(feeder, drain, result);
        Ok(())
    }

    fn new_block(
        &mut self,
        parameter_types: &[IrType],
    ) -> Result<(IrBlockId, Vec<IrValueId>), LoweringFailure> {
        let id = IrBlockId(
            u32::try_from(self.blocks.len()).map_err(|_| LoweringFailure::CounterOverflow)?,
        );
        let mut parameters = Vec::with_capacity(parameter_types.len());
        let mut values = Vec::with_capacity(parameter_types.len());
        for ty in parameter_types {
            let value = self.new_value(*ty)?;
            parameters.push((value, *ty));
            values.push(value);
        }
        self.blocks.push(BuildingBlock {
            parameters,
            instructions: Vec::new(),
            terminator: None,
        });
        Ok((id, values))
    }

    fn current_block_mut(&mut self) -> Result<&mut BuildingBlock, LoweringFailure> {
        let current = self.current.ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.blocks
            .get_mut(current.index())
            .ok_or(LoweringFailure::InvalidCheckedProgram)
    }

    fn terminate(&mut self, terminator: IrTerminator) -> Result<(), LoweringFailure> {
        let block = self.current_block_mut()?;
        if block.terminator.replace(terminator).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.current = None;
        Ok(())
    }

    /// One [MSR-1] measure whose table cell is a compile-time constant.
    /// The reader still executes; it just has no load to perform.
    fn lower_fixed_measure(&mut self, value: u64) -> Result<IrValueId, LoweringFailure> {
        let ty = IrType::Integer {
            width: 64,
            signed: false,
        };
        self.define(
            ty,
            IrOperation::Constant(IrConstant::Integer { ty, bits: value }),
        )
    }

    fn define(&mut self, ty: IrType, operation: IrOperation) -> Result<IrValueId, LoweringFailure> {
        let result = self.new_value(ty)?;
        self.current_block_mut()?
            .instructions
            .push(IrInstruction::Define {
                result,
                ty,
                operation,
            });
        Ok(result)
    }

    /// The overlap groups this body can actualize, from the permitted and
    /// eligible chains the checker recorded.
    ///
    /// The judgment is the checker's; this only narrows it to what the emitted
    /// shape can carry, and every narrowing drops members rather than adding
    /// any. A group is a prefix of a permitted chain, kept only while
    ///
    /// - each site's `let` lowered to exactly one call definition, so a chain
    ///   member whose statement lowered to something else (a `propagate`, for
    ///   instance) ends the group;
    /// - every member's definition is in one block, so the handed-out call
    ///   and its join sit on one straight-line edge; and
    /// - no member but the last is an addressed binding, because promoting one
    ///   reads the call's value at the definition site — between the hand-out
    ///   and the join, where the value does not exist yet.
    ///
    /// A prefix of a permitted chain is itself permitted: the chain's every
    /// ordered pair was judged, so every ordered pair of the prefix was too.
    fn overlaps(&self) -> Vec<IrOverlap> {
        if self.overlap != OverlapLowering::On {
            return Vec::new();
        }
        let Some(permissions) = self.permissions else {
            return Vec::new();
        };
        let mut overlaps = Vec::new();
        for run in &permissions.runs {
            let mut members = Vec::new();
            let mut home = None;
            for site in &run.sites {
                let Some((block, value)) = self.call_results.get(&site.call).copied() else {
                    break;
                };
                if *home.get_or_insert(block) != block {
                    break;
                }
                let addressed = site
                    .binding
                    .is_some_and(|binding| self.addressed_bindings.contains(&binding));
                members.push(value);
                if addressed {
                    // This member must be the group's last, so it ends it.
                    break;
                }
            }
            if members.len() >= 2 {
                overlaps.push(IrOverlap { members });
            }
        }
        overlaps
    }

    /// Lowers consecutive-call completion schedules after every source
    /// binding has acquired its final IR value.
    ///
    /// Permission records ordinary dependencies for every call in a schedule.
    /// This stage narrows submission to direct may-suspend system calls. Inline
    /// and user calls remain ordinary steps so independent writer work can run
    /// between a submission and the schedule's final join.
    fn completion_steps(&self) -> Vec<IrCompletionStep> {
        let Some(permissions) = self.permissions else {
            return Vec::new();
        };
        let mut lowered = Vec::new();
        let mut start = 0;
        while start < permissions.completion_steps.len() {
            let Some(relative_end) = permissions.completion_steps[start..]
                .iter()
                .position(|step| !step.has_later_independent_call)
            else {
                break;
            };
            let end = start + relative_end;
            let source = &permissions.completion_steps[start..=end];
            let resolved = source
                .iter()
                .map(|step| {
                    self.call_results
                        .get(&step.site.call)
                        .copied()
                        .map(|(block, value)| (step, block, value))
                })
                .collect::<Option<Vec<_>>>();
            let Some(resolved) = resolved else {
                start = end + 1;
                continue;
            };
            let Some(home) = resolved.first().map(|(_, block, _)| *block) else {
                start = end + 1;
                continue;
            };
            if resolved.iter().any(|(_, block, _)| *block != home) {
                start = end + 1;
                continue;
            }

            let submitted = resolved
                .iter()
                .filter(|(step, _, value)| {
                    step.has_later_independent_call && self.direct_may_suspend_system_call(*value)
                })
                .map(|(step, _, value)| (&step.site.call, *value))
                .collect::<HashMap<_, _>>();
            if submitted.is_empty() {
                start = end + 1;
                continue;
            }
            for (ordinal, (step, _, value)) in resolved.iter().enumerate() {
                let wait_for = step
                    .wait_for
                    .iter()
                    .filter_map(|call| submitted.get(call).copied())
                    .collect();
                lowered.push(IrCompletionStep::new(
                    *value,
                    wait_for,
                    submitted.contains_key(&step.site.call),
                    ordinal + 1 == resolved.len(),
                ));
            }
            start = end + 1;
        }

        // A permitted staged cut whose one-slot or bounded-batch driver was
        // materialized is itself a complete completion schedule. It may not
        // occur in the ordinary consecutive-call table: that table requires a
        // later independent call, while this driver deliberately submits and
        // retires the cut before its result dispatch.  Preserve any ordinary
        // dependency set when the call is already present; otherwise add the
        // single finite step.  The `driver_ready` gate is what prevents a
        // permission-only descriptor from changing emitted execution.
        if self
            .completion_pipeline
            .as_ref()
            .is_some_and(IrCompletionPipeline::driver_ready)
            && let Some(cut) = self.staged_cut.as_ref()
            && let Some((_, result)) = self.call_results.get(cut).copied()
            && self.direct_may_suspend_system_call(result)
        {
            if let Some(step) = lowered.iter_mut().find(|step| step.call == result) {
                step.submit = true;
                step.finish = true;
            } else {
                lowered.push(IrCompletionStep::new(result, Vec::new(), true, true));
            }
        }
        lowered
    }

    /// Records where a named-function call in call position landed, whatever
    /// written position it was in.
    ///
    /// The permission judgment reaches a call as a `let` right-hand side and as
    /// a `match` scrutinee alike, and both are named by their call occurrence,
    /// so one recording serves both. Which of them a group can actually keep is
    /// decided later and by the IR alone: every member of a group must be
    /// defined in one block, and a scrutinee's own dispatch terminates its
    /// block, so a scrutinee call is only ever a group's last member.
    fn note_call_result(
        &mut self,
        expression: &CheckedExpression,
        value: IrValueId,
    ) -> Result<(), LoweringFailure> {
        let (CheckedExpression::UserCall { call, .. } | CheckedExpression::SystemCall { call, .. }) =
            expression
        else {
            return Ok(());
        };
        // The block the call's own definition landed in, which is the block
        // current after the arguments are lowered.
        let block = self.current.ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.call_results.insert(call.clone(), (block, value));
        Ok(())
    }

    fn direct_may_suspend_system_call(&self, value: IrValueId) -> bool {
        self.blocks.iter().any(|block| {
            block.instructions.iter().any(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::Define {
                        result,
                        operation: IrOperation::SystemCall { target_action, .. },
                        ..
                    } if *result == value && target_action.may_suspend()
                )
            })
        })
    }

    fn lower_statements(
        &mut self,
        statements: &[CheckedStatement],
        give_target: Option<GiveTarget>,
    ) -> Result<(), LoweringFailure> {
        for statement in statements {
            if self.current.is_none() {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            match statement {
                CheckedStatement::Let {
                    binding,
                    value: expression,
                    ..
                } => {
                    let value = self.expression(expression)?;
                    self.note_call_result(expression, value)?;
                    if self.bindings.insert(*binding, value).is_some() {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    self.promote_binding_if_needed(*binding)?;
                }
                CheckedStatement::PropagateLet {
                    binding,
                    scrutinee,
                    result_nominal,
                    return_nominal,
                    ok_type,
                    error_type,
                    error_drops,
                    context,
                    ..
                } => self.lower_propagate(
                    *binding,
                    scrutinee,
                    *result_nominal,
                    *return_nominal,
                    *ok_type,
                    *error_type,
                    error_drops,
                    context,
                )?,
                // [GRAM-4, CALL-4] one evaluation of the call, then one
                // projection per result ordinal in written order. The
                // callable hands back one value of its result-list nominal
                // and the ordinals are its fields, so this is the ordinary
                // struct projection every other field read uses.
                CheckedStatement::DestructuringLet {
                    bindings,
                    nominal,
                    value: expression,
                    ..
                } => {
                    let aggregate = self.expression(expression)?;
                    self.note_call_result(expression, aggregate)?;
                    if self.value_type(aggregate)? != IrType::Nominal(IrNominalId(nominal.0)) {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    for (ordinal, (binding, ty)) in bindings.iter().enumerate() {
                        let field = u32::try_from(ordinal)
                            .map_err(|_| LoweringFailure::InvalidCheckedProgram)?;
                        let value = self.project_struct_path(aggregate, &[field], true)?;
                        if self.value_type(value)? != lower_type(*ty)? {
                            return Err(LoweringFailure::InvalidCheckedProgram);
                        }
                        if self.bindings.insert(*binding, value).is_some() {
                            return Err(LoweringFailure::InvalidCheckedProgram);
                        }
                        self.promote_binding_if_needed(*binding)?;
                    }
                }
                CheckedStatement::SetList {
                    targets, values, ..
                } => self.set_list(targets, values)?,
                CheckedStatement::Set { target, value, .. } => self.set(target, value)?,
                CheckedStatement::Replace {
                    binding,
                    target,
                    value,
                    ..
                } => self.replace(*binding, target, value)?,
                CheckedStatement::Evaluate(expression) => {
                    self.expression(expression)?;
                }
                CheckedStatement::DropExpression {
                    value: expression,
                    release,
                    ..
                } => {
                    let value = self.expression(expression)?;
                    let drop = IrDrop {
                        value,
                        ty: self.value_type(value)?,
                        release: *release,
                    };
                    self.current_block_mut()?
                        .instructions
                        .push(IrInstruction::Drop(drop));
                }
                // PRF-1 proof statements have already contributed their
                // checked fact to semantic flow. They have no runtime value,
                // effect, branch, or instruction.
                CheckedStatement::Proof(_) => {}
                CheckedStatement::Return { value, drops, .. } => {
                    let value = self.expression(value)?;
                    let drops = self.lower_drops(drops)?;
                    self.terminate(IrTerminator::Return { value, drops })?;
                }
                CheckedStatement::Give { value, drops, .. } => {
                    let target = give_target
                        .as_ref()
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    let value = self.expression(value)?;
                    if self.value_type(value)? != target.result {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    let mut arguments = Vec::with_capacity(1 + target.carried_bindings.len());
                    arguments.push(value);
                    arguments.extend(self.binding_values(&target.carried_bindings)?);
                    let drops = self.lower_drops(drops)?;
                    self.terminate(IrTerminator::Jump {
                        target: target.block,
                        arguments,
                        drops,
                    })?;
                }
                CheckedStatement::Loop {
                    id,
                    invariants: _,
                    body,
                    backedge_drops,
                } => self.lower_loop(*id, body, backedge_drops, give_target.clone())?,
                CheckedStatement::CountedRange {
                    id,
                    node_path,
                    binder,
                    lower,
                    upper,
                    // Source proof metadata has no runtime representation.
                    // The semantic checker must accept or reject it before
                    // lowering starts; this pattern deliberately erases it.
                    invariants: _,
                    body,
                    backedge_drops,
                } => self.lower_counted_range(
                    *id,
                    node_path,
                    *binder,
                    lower,
                    upper,
                    body,
                    backedge_drops,
                    give_target.clone(),
                )?,
                CheckedStatement::Break { target, drops } => {
                    let target = self
                        .loops
                        .iter()
                        .rev()
                        .find(|candidate| candidate.id == *target)
                        .cloned()
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    let arguments = self.binding_values(&target.carried_bindings)?;
                    let drops = self.lower_drops(drops)?;
                    self.terminate(IrTerminator::Jump {
                        target: target.block,
                        arguments,
                        drops,
                    })?;
                }
                CheckedStatement::Region {
                    arena_list,
                    body,
                    fallthrough_drops,
                } => {
                    // The region's arena allocation list is materialized at
                    // region entry; its compiler-derived drop on each normal
                    // exit edge is the region's storage release [STOR-3].
                    if let Some(list) = arena_list {
                        let storage = self
                            .nominals
                            .iter()
                            .find(|nominal| nominal.kind == IrNominalKind::ArenaStorage)
                            .map(|nominal| nominal.id)
                            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                        let value =
                            self.define(IrType::Nominal(storage), IrOperation::ArenaListNew)?;
                        if self.bindings.insert(*list, value).is_some() {
                            return Err(LoweringFailure::InvalidCheckedProgram);
                        }
                    }
                    self.lower_statements(body, give_target.clone())?;
                    if self.current.is_some() {
                        let drops = self.lower_drops(fallthrough_drops)?;
                        for drop in drops {
                            self.current_block_mut()?
                                .instructions
                                .push(IrInstruction::Drop(drop));
                        }
                    }
                }
                CheckedStatement::Match {
                    scrutinee,
                    enum_type,
                    arms,
                    continues,
                } => self.lower_match(
                    scrutinee,
                    *enum_type,
                    arms,
                    *continues,
                    None,
                    give_target.clone(),
                )?,
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    scrutinee,
                    enum_type,
                    arms,
                    continues,
                    ..
                } => {
                    self.lower_match(
                        scrutinee,
                        *enum_type,
                        arms,
                        *continues,
                        Some((*binding, lower_type(*result_type)?)),
                        give_target.clone(),
                    )?;
                    if self.current.is_some() {
                        self.promote_binding_if_needed(*binding)?;
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_match(
        &mut self,
        scrutinee: &CheckedExpression,
        enum_type: CheckedEnumType,
        arms: &[CheckedMatchArm],
        continues: bool,
        value_binding: Option<(BindingId, IrType)>,
        outer_give_target: Option<GiveTarget>,
    ) -> Result<(), LoweringFailure> {
        let scrutinee_expression = scrutinee;
        let scrutinee = self.expression(scrutinee)?;
        self.note_call_result(scrutinee_expression, scrutinee)?;
        let one_slot_driver = self.begin_staged_match_drain(scrutinee)?;
        self.lower_match_from_value(
            scrutinee,
            enum_type,
            arms,
            continues,
            value_binding,
            outer_give_target,
        )?;
        if let Some((feeder, drain)) = one_slot_driver {
            self.completion_pipeline
                .as_mut()
                .ok_or(LoweringFailure::InvalidCheckedProgram)?
                .plan_one_slot(feeder, drain, scrutinee);
        }
        Ok(())
    }

    /// Places the one-slot drain immediately after its feeder in block order.
    ///
    /// The selected call may be the match scrutinee itself or a value bound
    /// immediately before the match. In either spelling the checked cut maps
    /// to the same SSA result and feeder. Creating the drain before the match
    /// creates its arm blocks makes the emitter's single forward walk mirror
    /// the generated CFG: submit in `feeder`, cross its only edge, join and
    /// dispatch in `drain`, then emit the arms. No unrelated block has to
    /// carry compiler emission state merely because it was numbered between
    /// the two cut points.
    fn begin_staged_match_drain(
        &mut self,
        scrutinee: IrValueId,
    ) -> Result<Option<(IrBlockId, IrBlockId)>, LoweringFailure> {
        let Some(cut) = self.staged_cut.as_ref() else {
            return Ok(None);
        };
        let Some((feeder, selected)) = self.call_results.get(cut).copied() else {
            return Ok(None);
        };
        if selected != scrutinee
            || self.current != Some(feeder)
            || self
                .completion_pipeline
                .as_ref()
                .is_none_or(IrCompletionPipeline::driver_ready)
        {
            return Ok(None);
        }
        let call_is_last = matches!(
            self.blocks
                .get(feeder.index())
                .and_then(|block| block.instructions.last()),
            Some(IrInstruction::Define {
                result,
                operation: IrOperation::SystemCall { target_action, .. },
                ..
            }) if *result == scrutinee && target_action.may_suspend()
        );
        if !call_is_last {
            return Ok(None);
        }

        let (drain, parameters) = self.new_block(&[])?;
        if !parameters.is_empty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.terminate(IrTerminator::Jump {
            target: drain,
            arguments: Vec::new(),
            drops: Vec::new(),
        })?;
        self.current = Some(drain);
        Ok(Some((feeder, drain)))
    }

    /// Lowers the dispatch and arms after another control-flow owner has
    /// arranged where the scrutinee becomes available.
    ///
    /// Ordinary matches define the value immediately before this call. A
    /// bounded completion batch defines it in its drain block after joining
    /// the slot named by that block, then uses this same arm lowering. No
    /// source rule or ownership decision is repeated here.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn lower_match_from_value(
        &mut self,
        scrutinee: IrValueId,
        enum_type: CheckedEnumType,
        arms: &[CheckedMatchArm],
        continues: bool,
        value_binding: Option<(BindingId, IrType)>,
        outer_give_target: Option<GiveTarget>,
    ) -> Result<(), LoweringFailure> {
        let base_bindings = self.bindings.clone();
        let mut carried_bindings = base_bindings.keys().copied().collect::<Vec<_>>();
        carried_bindings.sort_by_key(|binding| binding.0);
        let join = if continues {
            let mut parameter_types =
                Vec::with_capacity(carried_bindings.len() + usize::from(value_binding.is_some()));
            if let Some((_, ty)) = value_binding {
                parameter_types.push(ty);
            }
            for binding in &carried_bindings {
                let value = base_bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                parameter_types.push(self.value_type(value)?);
            }
            let (block, parameters) = self.new_block(&parameter_types)?;
            Some((block, parameters))
        } else {
            None
        };
        let mut arm_blocks = Vec::with_capacity(arms.len());
        for _ in arms {
            arm_blocks.push(self.new_block(&[])?.0);
        }
        self.terminate(IrTerminator::Match {
            scrutinee,
            enum_type: enum_type.into(),
            targets: arms
                .iter()
                .zip(&arm_blocks)
                .map(|(arm, block)| IrMatchTarget {
                    tag: arm.tag,
                    block: *block,
                })
                .collect(),
        })?;
        for (arm, block) in arms.iter().zip(arm_blocks) {
            self.current = Some(block);
            self.bindings = base_bindings.clone();
            for binder in &arm.binders {
                let CheckedEnumType::Nominal(nominal) = enum_type else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let value = self.define(
                    lower_type(binder.ty)?,
                    IrOperation::ProjectVariant {
                        aggregate: scrutinee,
                        nominal: IrNominalId(nominal.0),
                        variant: arm.tag,
                        field: binder.field,
                    },
                )?;
                if self.bindings.insert(binder.binding, value).is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                if binder.mode == CheckedMode::Own {
                    self.promote_binding_if_needed(binder.binding)?;
                }
            }
            let arm_give_target = match value_binding {
                Some((_, ty)) => join.as_ref().map(|(block, _)| GiveTarget {
                    block: *block,
                    result: ty,
                    carried_bindings: carried_bindings.clone(),
                }),
                None => outer_give_target.clone(),
            };
            self.lower_statements(&arm.body, arm_give_target)?;
            if self.current.is_some() {
                let Some((join, _)) = &join else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                if value_binding.is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let drops = self.lower_drops(&arm.fallthrough_drops)?;
                let arguments = self.binding_values(&carried_bindings)?;
                self.terminate(IrTerminator::Jump {
                    target: *join,
                    arguments,
                    drops,
                })?;
            }
        }
        self.bindings = base_bindings;
        if let Some((join, parameters)) = join {
            self.current = Some(join);
            let carried_start = usize::from(value_binding.is_some());
            if parameters.len() != carried_start + carried_bindings.len() {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            for (binding, value) in carried_bindings.iter().zip(&parameters[carried_start..]) {
                if self.bindings.insert(*binding, *value).is_none() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
            }
            if let Some((binding, _)) = value_binding {
                let value = *parameters
                    .first()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if self.bindings.insert(binding, value).is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
            }
        } else {
            self.current = None;
        }
        Ok(())
    }

    fn expression(&mut self, expression: &CheckedExpression) -> Result<IrValueId, LoweringFailure> {
        match expression {
            CheckedExpression::Binding { binding, ty, .. } => {
                let value = self
                    .bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let expected = lower_type(*ty)?;
                let actual = self.value_type(value)?;
                let value = if self.addressed_bindings.contains(binding) {
                    self.load_storage_value(value)?
                } else {
                    value
                };
                if self.value_type(value)? != expected
                    && !matches!(
                        (actual, expected),
                        (IrType::Address(referent), _) if referent.ty() == expected
                    )
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(value)
            }
            CheckedExpression::Constant(value) => {
                let ty = lower_type(value.ty())?;
                let constant = lower_scalar_constant(value)?;
                self.define(ty, IrOperation::Constant(constant))
            }
            CheckedExpression::NamedConstant { value, .. } => {
                let ty = lower_type(value.ty())?;
                let constant = lower_scalar_constant(value)?;
                self.define(ty, IrOperation::Constant(constant))
            }
            CheckedExpression::UserCall {
                function,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                // The definition takes the callee's declared IR result, which
                // carries the result mode: a borrow-returning callee delivers
                // an address, not a referent value [OWN-2, TYPE-7].
                let result = *self
                    .function_results
                    .get(function.0 as usize)
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                self.define(
                    result,
                    IrOperation::Call {
                        function: function.0,
                        arguments,
                    },
                )
            }
            // A system operation is identified by its target-independent
            // semantic identity [QUAL-1]; no source spelling reaches the IR.
            CheckedExpression::SystemCall {
                operation,
                target_action,
                arguments,
                result,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.define(
                    lower_type(*result)?,
                    IrOperation::SystemCall {
                        operation: IrSystemOperation(*operation),
                        target_action: *target_action,
                        arguments,
                    },
                )
            }
            // An opaque resource value is its own borrow: it has no
            // source-visible content and needs no stable address, exactly as
            // a `box` borrow does.
            CheckedExpression::BorrowSystemResource {
                binding, nominal, ..
            } => {
                let value = self.binding_value(*binding)?;
                if self.value_type(value)? != IrType::Nominal(IrNominalId(nominal.0)) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(value)
            }
            CheckedExpression::IntegerOperation {
                operation,
                operand_type,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.define(
                    lower_type(expression.ty())?,
                    IrOperation::Integer {
                        operation: (*operation).into(),
                        operand_type: lower_type(*operand_type)?,
                        arguments,
                    },
                )
            }
            CheckedExpression::FloatOperation {
                operation,
                operand_type,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.define(
                    lower_type(expression.ty())?,
                    IrOperation::Float {
                        operation: (*operation).into(),
                        operand_type: lower_type(*operand_type)?,
                        arguments,
                    },
                )
            }
            CheckedExpression::NumericConversion {
                source,
                destination,
                value,
                ..
            } => {
                let value = self.expression(value)?;
                self.define(
                    lower_type(expression.ty())?,
                    IrOperation::NumericConversion {
                        source_type: lower_numeric_type(*source),
                        destination_type: lower_numeric_type(*destination),
                        value,
                    },
                )
            }
            CheckedExpression::Reinterpret {
                source,
                destination,
                value,
                ..
            } => {
                let value = self.expression(value)?;
                self.define(
                    lower_numeric_type(*destination),
                    IrOperation::Reinterpret {
                        source_type: lower_numeric_type(*source),
                        destination_type: lower_numeric_type(*destination),
                        value,
                    },
                )
            }
            CheckedExpression::BooleanOperation {
                operation,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.define(
                    IrType::Bool,
                    IrOperation::Boolean {
                        operation: (*operation).into(),
                        arguments,
                    },
                )
            }
            CheckedExpression::EnumEquality {
                equal,
                operand_type,
                arguments,
                ..
            } => {
                let [left, right] = arguments.as_slice() else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                self.define(
                    IrType::Bool,
                    IrOperation::EnumEquality {
                        equal: *equal,
                        operand_type: lower_type(*operand_type)?,
                        arguments: [left, right],
                    },
                )
            }
            CheckedExpression::ArrayFill {
                ty,
                value,
                target_domain,
                ..
            } => {
                let IrType::Array { element, .. } = lower_type(*ty)? else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let value = self.expression(value)?;
                if self.value_type(value)? != element.ty() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(
                    lower_type(*ty)?,
                    IrOperation::ArrayFill {
                        value,
                        target_domain: (*target_domain).into(),
                    },
                )
            }
            CheckedExpression::ArrayMeasure {
                measure,
                root,
                length,
            } => {
                if let Some(constant) = fixed_measure(*measure, MeasuredKind::Array) {
                    return self.lower_fixed_measure(constant);
                }
                let (_, ty) = self.array_root(root)?;
                let IrType::Array { length: actual, .. } = ty else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let length = length
                    .value()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if actual != length {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(
                    IrType::Integer {
                        width: 64,
                        signed: false,
                    },
                    IrOperation::Constant(IrConstant::Integer {
                        ty: IrType::Integer {
                            width: 64,
                            signed: false,
                        },
                        bits: length,
                    }),
                )
            }
            CheckedExpression::ArrayIndex {
                root,
                element_type,
                length,
                offset,
                target_domain,
                ..
            } => {
                let (root, ty) = self.array_root(root)?;
                let IrType::Array {
                    element,
                    length: actual,
                } = ty
                else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let length = length
                    .value()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if element.ty() != lower_type(*element_type)? || actual != length {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let offset = self.expression(offset)?;
                if self.value_type(offset)?
                    != (IrType::Integer {
                        width: 64,
                        signed: false,
                    })
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(
                    element.ty(),
                    IrOperation::ArrayIndex {
                        root,
                        offset,
                        target_domain: (*target_domain).into(),
                    },
                )
            }
            CheckedExpression::BufferFill {
                element,
                length,
                value,
                layout_ceiling,
                target_domains,
                ..
            } => self.lower_buffer_fill(*element, length, value, *layout_ceiling, *target_domains),
            CheckedExpression::BufferVacant {
                element,
                length,
                layout_ceiling,
                target_domains,
                ..
            } => self.lower_buffer_vacant(*element, length, *layout_ceiling, *target_domains),
            CheckedExpression::BufferFits {
                length,
                layout_ceiling,
                ..
            } => {
                let length = self.expression(length)?;
                self.define(
                    IrType::Bool,
                    IrOperation::BufferFits {
                        length,
                        maximum_length: layout_ceiling.stride.allocation_limit(),
                    },
                )
            }
            CheckedExpression::BufferMeasure { measure, root } => {
                match fixed_measure(*measure, MeasuredKind::Buffer) {
                    Some(constant) => self.lower_fixed_measure(constant),
                    None => self.lower_buffer_length(root),
                }
            }
            CheckedExpression::BufferIndex {
                root,
                offset,
                target_domain,
                ..
            } => self.lower_buffer_index(root, offset, *target_domain),
            CheckedExpression::SliceOf {
                source, element, ..
            } => self.lower_slice_of(source, *element),
            CheckedExpression::SliceMeasure { measure, root } => {
                match fixed_measure(*measure, MeasuredKind::Slice) {
                    Some(constant) => self.lower_fixed_measure(constant),
                    None => self.lower_slice_length(root),
                }
            }
            CheckedExpression::SliceIndex {
                root,
                offset,
                target_domain,
                ..
            } => self.lower_slice_index(root, offset, *target_domain),
            CheckedExpression::BoxNew { nominal, value, .. } => {
                let value = self.expression(value)?;
                let nominal = IrNominalId(nominal.0);
                self.define(
                    IrType::Nominal(nominal),
                    IrOperation::BoxNew { nominal, value },
                )
            }
            CheckedExpression::BoxDeref { nominal, value, .. } => {
                let value = self.expression(value)?;
                let nominal = IrNominalId(nominal.0);
                let IrNominalKind::Box { referent } = self
                    .nominals
                    .get(nominal.index())
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?
                    .kind
                else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                self.define(referent, IrOperation::BoxDeref { nominal, value })
            }
            CheckedExpression::ArenaNew {
                nominal,
                list,
                value,
                ..
            } => {
                let value = self.expression(value)?;
                let nominal = IrNominalId(nominal.0);
                let IrNominalKind::Arena { content } = self
                    .nominals
                    .get(nominal.index())
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?
                    .kind
                else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                if self.value_type(value)? != content {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let list = self.binding_value(*list)?;
                self.define(
                    IrType::Nominal(nominal),
                    IrOperation::ArenaNew {
                        nominal,
                        list,
                        value,
                    },
                )
            }
            CheckedExpression::ArenaDeref { nominal, value, .. } => {
                let value = self.expression(value)?;
                let nominal = IrNominalId(nominal.0);
                if self.value_type(value)? != IrType::Nominal(nominal) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let IrNominalKind::Arena { content } = self
                    .nominals
                    .get(nominal.index())
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?
                    .kind
                else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                self.define(content, IrOperation::ArenaDeref { nominal, value })
            }
            CheckedExpression::BorrowBuffer { root, .. } => self.lower_buffer_borrow(root),
            CheckedExpression::BorrowBox {
                binding, nominal, ..
            } => {
                let value = self.binding_value(*binding)?;
                if self.value_type(value)? != IrType::Nominal(IrNominalId(nominal.0)) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(value)
            }
            CheckedExpression::BorrowAddressed { binding, ty, .. }
            | CheckedExpression::ReborrowAddressed { binding, ty, .. } => {
                self.lower_addressed_borrow(*binding, lower_type(*ty)?)
            }
            CheckedExpression::DerefAddressed { binding, ty, .. } => {
                let value = self.binding_value(*binding)?;
                if self.value_type(value)? != lower_type(*ty)? {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(value)
            }
            CheckedExpression::ConstructStruct {
                nominal, fields, ..
            } => {
                let fields = fields
                    .iter()
                    .map(|field| self.expression(field))
                    .collect::<Result<Vec<_>, _>>()?;
                let nominal = IrNominalId(nominal.0);
                self.define(
                    IrType::Nominal(nominal),
                    IrOperation::ConstructStruct { nominal, fields },
                )
            }
            CheckedExpression::ConstructEnum {
                nominal,
                variant,
                fields,
                ..
            } => {
                let fields = fields
                    .iter()
                    .map(|field| self.expression(field))
                    .collect::<Result<Vec<_>, _>>()?;
                let nominal = IrNominalId(nominal.0);
                self.define(
                    IrType::Nominal(nominal),
                    IrOperation::ConstructEnum {
                        nominal,
                        variant: *variant,
                        fields,
                    },
                )
            }
            CheckedExpression::Project {
                binding,
                fields,
                ty,
                consume_root,
                residual_drops,
                ..
            } => {
                let root = self.binding_value(*binding)?;
                let mut lowered_drops = Vec::with_capacity(residual_drops.len());
                for drop in residual_drops {
                    lowered_drops.push(self.lower_projected_drop(root, drop)?);
                }
                let value = self.project_struct_path(root, fields, *consume_root)?;
                if self.value_type(value)? != lower_type(*ty)? {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                for drop in lowered_drops {
                    self.current_block_mut()?
                        .instructions
                        .push(IrInstruction::Drop(drop));
                }
                Ok(value)
            }
            CheckedExpression::ProjectValue {
                value,
                nominal,
                field,
                ty,
                ..
            } => {
                let aggregate = self.expression(value)?;
                let nominal = IrNominalId(nominal.0);
                self.define(
                    lower_type(*ty)?,
                    IrOperation::ProjectStruct {
                        aggregate,
                        nominal,
                        field: *field,
                        consume_root: false,
                    },
                )
            }
        }
    }

    pub(super) fn array_root(
        &mut self,
        root: &CheckedArrayRoot,
    ) -> Result<(IrArrayRoot, IrType), LoweringFailure> {
        match root {
            CheckedArrayRoot::Binding {
                binding, fields, ..
            } => {
                let storage = self
                    .bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let root = self.load_storage_value(storage)?;
                let value = if fields.is_empty() {
                    root
                } else {
                    self.project_struct_path(root, fields, false)?
                };
                Ok((IrArrayRoot::Value(value), self.value_type(value)?))
            }
            CheckedArrayRoot::Constant(constant) => {
                let constant = self
                    .constants
                    .get(constant.0 as usize)
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                Ok((
                    IrArrayRoot::Constant(IrConstantId(constant.id().0)),
                    constant.ty(),
                ))
            }
        }
    }

    /// [SET-2]: read the previous value out of the target place into the
    /// fresh binding, then perform exactly the [SET-1] store of the
    /// replacement. The read-out precedes the store, so no program point
    /// observes an empty place, and nothing is dropped.
    fn replace(
        &mut self,
        binding: BindingId,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
    ) -> Result<(), LoweringFailure> {
        let root_binding = target.binding();
        let storage = self
            .bindings
            .get(&root_binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let root = self.load_storage_value(storage)?;
        let previous = match target {
            CheckedSetTarget::Place(place) => {
                if place.fields.is_empty() {
                    root
                } else {
                    self.project_struct_path(root, &place.fields, false)?
                }
            }
            // A buffer-element replacement [SET-2, TYPE-2] evaluates its
            // target components exactly once: the projected buffer and the
            // offset feed one element read (the previous owner) and one
            // element write (the replacement), so the shared `set` path,
            // which would re-lower the offset, is not reused here.
            CheckedSetTarget::BufferIndex(target) => {
                let previous = self.lower_buffer_replace(root, target, value)?;
                if self.bindings.insert(binding, previous).is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.promote_binding_if_needed(binding)?;
                return Ok(());
            }
            // An array element is copy [TYPE-2], so the checker never forms
            // an element-position replace target over an array [SET-2].
            CheckedSetTarget::ArrayIndex(_) => {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        };
        if self.value_type(previous)? != lower_type(target.ty())? {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        if self.bindings.insert(binding, previous).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.promote_binding_if_needed(binding)?;
        self.set(target, value)
    }

    /// [GRAM-4, SET-1, CALL-4] `set (x, y) = f(...);`.
    ///
    /// One evaluation of the call, then one projection per result ordinal in
    /// written order, each committed to its target exactly as a single-target
    /// `set` commits. Every target of a checked target list is a plain place;
    /// a subscript target stops in the checker.
    /// [LIV-2] one commit of a target list.
    ///
    /// The whole right-hand side is evaluated first — the one call, or every
    /// written value in order — and only then is any target written, so a
    /// statement whose targets and values name the same places, the swap
    /// included, reads every previous value before the first commit.
    fn set_list(
        &mut self,
        targets: &[CheckedSetTarget],
        values: &CheckedCommitValues,
    ) -> Result<(), LoweringFailure> {
        let ordinals = match values {
            CheckedCommitValues::ResultList { nominal, value } => {
                let aggregate = self.expression(value)?;
                self.note_call_result(value, aggregate)?;
                if self.value_type(aggregate)? != IrType::Nominal(IrNominalId(nominal.0)) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let mut ordinals = Vec::with_capacity(targets.len());
                for ordinal in 0..targets.len() {
                    let field = u32::try_from(ordinal)
                        .map_err(|_| LoweringFailure::InvalidCheckedProgram)?;
                    ordinals.push(self.project_struct_path(aggregate, &[field], true)?);
                }
                ordinals
            }
            CheckedCommitValues::Written(values) => {
                if values.len() != targets.len() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let mut ordinals = Vec::with_capacity(values.len());
                for value in values {
                    ordinals.push(self.expression(value)?);
                }
                ordinals
            }
        };
        for (target, value) in targets.iter().zip(ordinals) {
            self.commit_target(target, value)?;
        }
        Ok(())
    }

    /// One target's write of an already-evaluated ordinal value [LIV-2].
    fn commit_target(
        &mut self,
        target: &CheckedSetTarget,
        value: IrValueId,
    ) -> Result<(), LoweringFailure> {
        let binding = target.binding();
        let storage = self
            .bindings
            .get(&binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let root = self.load_storage_value(storage)?;
        if self.value_type(value)? != lower_type(target.ty())? {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let replacement = match target {
            CheckedSetTarget::Place(place) => {
                if place.fields.is_empty() {
                    if self.value_type(root)? != self.value_type(value)? {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    value
                } else {
                    self.replace_struct_path(root, &place.fields, value)?
                }
            }
            CheckedSetTarget::ArrayIndex(target) => {
                self.lower_array_element_commit(root, target, value)?
            }
            CheckedSetTarget::BufferIndex(target) => {
                self.lower_buffer_element_commit(root, target, value)?
            }
        };
        let stored = match self.value_type(storage)? {
            IrType::Address(referent) => {
                if self.value_type(replacement)? != referent.ty() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.store_addressed(storage, replacement, referent)?;
                storage
            }
            _ => replacement,
        };
        if self.bindings.insert(binding, stored) != Some(storage) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(())
    }

    /// [SET-1, LIV-2] the one-target commit: the right-hand side is
    /// evaluated once and completely, then the one target is written by the
    /// same commit a target list uses.
    fn set(
        &mut self,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
    ) -> Result<(), LoweringFailure> {
        let value = self.expression(value)?;
        self.commit_target(target, value)
    }

    /// The array-element half of one commit: the offset is consumed directly,
    /// because its [OP-4] obligation was discharged at the source level and no
    /// runtime branch remains.
    fn lower_array_element_commit(
        &mut self,
        root: IrValueId,
        target: &CheckedArraySetTarget,
        value: IrValueId,
    ) -> Result<IrValueId, LoweringFailure> {
        let array_type = lower_type(target.array_type)?;
        let IrType::Array { element, length } = array_type else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let array = if target.fields.is_empty() {
            root
        } else {
            self.project_struct_path(root, &target.fields, false)?
        };
        if self.value_type(array)? != array_type
            || element.ty() != lower_type(target.element_type)?
            || Some(length) != target.length.value()
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let index = self.expression(&target.offset)?;
        if self.value_type(index)?
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        if self.value_type(value)? != element.ty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let replacement = self.define(
            array_type,
            IrOperation::InsertArray {
                aggregate: array,
                index,
                value,
            },
        )?;
        if target.fields.is_empty() {
            Ok(replacement)
        } else {
            self.replace_struct_path(root, &target.fields, replacement)
        }
    }

    fn project_struct_path(
        &mut self,
        mut value: IrValueId,
        fields: &[u32],
        consume_root: bool,
    ) -> Result<IrValueId, LoweringFailure> {
        if fields.is_empty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        for field in fields {
            let IrType::Nominal(nominal) = self.value_type(value)? else {
                return Err(LoweringFailure::InvalidCheckedProgram);
            };
            let field_ty = match &self
                .nominals
                .get(nominal.index())
                .ok_or(LoweringFailure::InvalidCheckedProgram)?
                .kind
            {
                IrNominalKind::Struct { fields } => {
                    fields
                        .get(*field as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?
                        .ty
                }
                // An opaque system resource has no writer-visible field, so no
                // struct path reaches through one.
                IrNominalKind::Enum { .. }
                | IrNominalKind::Box { .. }
                | IrNominalKind::Arena { .. }
                | IrNominalKind::ArenaStorage
                | IrNominalKind::SystemResource(_) => {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
            };
            value = self.define(
                field_ty,
                IrOperation::ProjectStruct {
                    aggregate: value,
                    nominal,
                    field: *field,
                    consume_root,
                },
            )?;
        }
        Ok(value)
    }

    fn replace_struct_path(
        &mut self,
        aggregate: IrValueId,
        fields: &[u32],
        replacement: IrValueId,
    ) -> Result<IrValueId, LoweringFailure> {
        let Some((field, remaining)) = fields.split_first() else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let IrType::Nominal(nominal) = self.value_type(aggregate)? else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let field_ty = match &self
            .nominals
            .get(nominal.index())
            .ok_or(LoweringFailure::InvalidCheckedProgram)?
            .kind
        {
            IrNominalKind::Struct { fields } => {
                fields
                    .get(*field as usize)
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?
                    .ty
            }
            // An opaque system resource has no writer-visible field, so no
            // struct path reaches through one.
            IrNominalKind::Enum { .. }
            | IrNominalKind::Box { .. }
            | IrNominalKind::Arena { .. }
            | IrNominalKind::ArenaStorage
            | IrNominalKind::SystemResource(_) => {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        };
        let value = if remaining.is_empty() {
            if self.value_type(replacement)? != field_ty {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            replacement
        } else {
            let selected = self.define(
                field_ty,
                IrOperation::ProjectStruct {
                    aggregate,
                    nominal,
                    field: *field,
                    consume_root: false,
                },
            )?;
            self.replace_struct_path(selected, remaining, replacement)?
        };
        self.define(
            IrType::Nominal(nominal),
            IrOperation::InsertStruct {
                aggregate,
                nominal,
                field: *field,
                value,
            },
        )
    }

    fn lower_projected_drop(
        &mut self,
        root: IrValueId,
        drop: &CheckedProjectedDrop,
    ) -> Result<IrDrop, LoweringFailure> {
        let value = self.project_struct_path(root, &drop.fields, false)?;
        let ty = lower_type(drop.ty)?;
        if self.value_type(value)? != ty {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(IrDrop {
            value,
            ty,
            release: drop.release,
        })
    }

    fn value_type(&self, value: IrValueId) -> Result<IrType, LoweringFailure> {
        self.values
            .get(value.index())
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)
    }

    fn binding_values(&self, bindings: &[BindingId]) -> Result<Vec<IrValueId>, LoweringFailure> {
        bindings
            .iter()
            .map(|binding| {
                self.bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)
            })
            .collect()
    }

    fn lower_drops(&mut self, drops: &[CheckedDrop]) -> Result<Vec<IrDrop>, LoweringFailure> {
        let mut lowered = Vec::with_capacity(drops.len());
        for drop in drops {
            let root = self.binding_value(drop.binding)?;
            let value = if drop.fields.is_empty() {
                root
            } else {
                self.project_struct_path(root, &drop.fields, false)?
            };
            let ty = lower_type(drop.ty)?;
            if self.value_type(value)? != ty {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            // The checked program already fixed what this release performs
            // [STOR-3]; lowering preserves the record and the edge's reverse
            // declaration order rather than rederiving either.
            lowered.push(IrDrop {
                value,
                ty,
                release: drop.release,
            });
        }
        Ok(lowered)
    }
}

/// The compile-time value of one [MSR-1] measure cell, when the table fixes
/// it. A cell the table gives the measured value's own extent is loaded at
/// run time instead, which is the `None` case.
const fn fixed_measure(measure: CheckedMeasure, measured: MeasuredKind) -> Option<u64> {
    match measure.cell(measured) {
        MeasureCell::ExactConstant(value) => Some(value),
        MeasureCell::ExactExtent | MeasureCell::Bounded | MeasureCell::Absent => None,
    }
}
