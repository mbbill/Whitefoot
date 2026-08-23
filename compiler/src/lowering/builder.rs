use std::collections::HashMap;

mod buffers;
mod loops;
mod probe;
mod results;
mod slices;
mod split;
mod storage;

use crate::CheckedProgram;
use crate::semantic::CheckedSetTarget;
use crate::semantic::{
    BindingId, CheckedArrayRoot, CheckedConstructor, CheckedDrop, CheckedEntryForm,
    CheckedExpression, CheckedMatchArm, CheckedMode, CheckedNominalKind, CheckedParameter,
    CheckedProgramData, CheckedProjectedDrop, CheckedStatement, CheckedValue, FunctionPermissions,
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
        OverlapLowering::On => Some(&checked.data.permission),
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
/// which inputs those are and the conservative alias links [SYS-12] fixes
/// between them; constructing the values and mapping the returned
/// `ExitStatus` belongs to the target stage.
fn lower_entry(entry: &CheckedEntryForm) -> IrEntry {
    IrEntry::Command {
        inputs: entry.inputs.clone(),
        aliases: entry
            .aliases
            .iter()
            .map(|alias| IrResourceAlias {
                left: alias.left,
                right: alias.right,
            })
            .collect(),
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
        CheckedValue::NumericIdentity { .. }
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
    let overlaps = builder.overlaps();
    builder.finish(function.symbol.clone(), overlaps, None)
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
    /// For each `let` whose right-hand side is exactly one named-function
    /// call, the block the call's definition landed in and the value it
    /// defined. The permission table names its sites by the binding each
    /// defines, so this is how a permitted group is found in the IR.
    call_results: HashMap<BindingId, (IrBlockId, IrValueId)>,
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
        synthesis: Option<IrSynthesis>,
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
            synthesis,
        })
    }

    fn new_value(&mut self, ty: IrType) -> Result<IrValueId, LoweringFailure> {
        let id = IrValueId(
            u32::try_from(self.values.len()).map_err(|_| LoweringFailure::CounterOverflow)?,
        );
        self.values.push(ty);
        Ok(id)
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
        let Some(permissions) = self.permissions else {
            return Vec::new();
        };
        let mut overlaps = Vec::new();
        for run in &permissions.runs {
            let mut members = Vec::new();
            let mut home = None;
            for site in &run.sites {
                let Some((block, value)) = self.call_results.get(&site.binding).copied() else {
                    break;
                };
                if *home.get_or_insert(block) != block {
                    break;
                }
                let addressed = self.addressed_bindings.contains(&site.binding);
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
                    // The block the call's own definition landed in, which is
                    // the block current after the arguments are lowered.
                    let block = self.current.ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    if self.bindings.insert(*binding, value).is_some() {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    if matches!(expression, CheckedExpression::UserCall { .. }) {
                        self.call_results.insert(*binding, (block, value));
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
                // A claim is the sole writer-visible runtime assertion. Its
                // trap record carries rule CLM-1 and the claim name [DIAG-3];
                // the justification is compile-time data and lowers to
                // nothing.
                CheckedStatement::Claim {
                    condition, site, ..
                } => {
                    let condition = self.expression(condition)?;
                    self.current_block_mut()?
                        .instructions
                        .push(IrInstruction::Claim {
                            condition,
                            site: site.clone().into(),
                        });
                }
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
                    body,
                    backedge_drops,
                } => self.lower_loop(*id, body, backedge_drops, give_target.clone())?,
                CheckedStatement::CountedRange {
                    id,
                    node_path,
                    binder,
                    lower,
                    upper,
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
        let scrutinee = self.expression(scrutinee)?;
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
            CheckedExpression::ArrayLength { root, length, .. } => {
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
            CheckedExpression::BufferLength { root, .. } => self.lower_buffer_length(root),
            CheckedExpression::BufferIndex {
                root,
                offset,
                target_domain,
                ..
            } => self.lower_buffer_index(root, offset, *target_domain),
            CheckedExpression::SliceOf {
                source, element, ..
            } => self.lower_slice_of(source, *element),
            CheckedExpression::SliceLength { root, .. } => self.lower_slice_length(root),
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

    fn set(
        &mut self,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
    ) -> Result<(), LoweringFailure> {
        let binding = target.binding();
        let storage = self
            .bindings
            .get(&binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let root = self.load_storage_value(storage)?;
        let replacement = match target {
            CheckedSetTarget::Place(target) => {
                let value = self.expression(value)?;
                if self.value_type(value)? != lower_type(target.ty)? {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                if target.fields.is_empty() {
                    if self.value_type(root)? != self.value_type(value)? {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    value
                } else {
                    self.replace_struct_path(root, &target.fields, value)?
                }
            }
            CheckedSetTarget::ArrayIndex(target) => {
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
                // The subscript's bounds obligation is discharged at the
                // source level [OP-4]; the offset is consumed directly with
                // no runtime branch.
                let index = self.expression(&target.offset)?;
                if self.value_type(index)?
                    != (IrType::Integer {
                        width: 64,
                        signed: false,
                    })
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let value = self.expression(value)?;
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
                    replacement
                } else {
                    self.replace_struct_path(root, &target.fields, replacement)?
                }
            }
            CheckedSetTarget::BufferIndex(target) => self.lower_buffer_set(root, target, value)?,
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
