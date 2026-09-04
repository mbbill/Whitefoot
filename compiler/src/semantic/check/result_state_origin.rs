use std::collections::HashMap;

use super::super::model::{
    BindingId, CheckedCommitValues, CheckedExpression, CheckedFunction, CheckedLoopId,
    CheckedMatchArm, CheckedResultStateOrigin, CheckedResultStatePath, CheckedSetTarget,
    CheckedStatement,
};
use super::{CheckStop, Checker};

#[derive(Clone, Debug, Eq, PartialEq)]
enum OriginSet {
    Absent,
    Finite {
        formals: Vec<CheckedResultStatePath>,
    },
    Unknown,
}

impl OriginSet {
    fn fresh() -> Self {
        Self::Finite {
            formals: Vec::new(),
        }
    }

    fn formal_leaves(formal: u32, leaves: Vec<Vec<u32>>) -> Self {
        Self::Finite {
            formals: leaves
                .into_iter()
                .map(|fields| CheckedResultStatePath {
                    result_fields: fields.clone(),
                    result_variant: None,
                    parameter: formal,
                    parameter_fields: fields,
                })
                .collect(),
        }
    }

    fn union(&mut self, other: Self) {
        match (&mut *self, other) {
            (Self::Unknown, _) => {}
            (_, Self::Unknown) => *self = Self::Unknown,
            (Self::Absent, finite @ Self::Finite { .. }) => *self = finite,
            (Self::Finite { formals: left }, Self::Finite { formals: right }) => {
                for formal in right {
                    if !left.contains(&formal) {
                        left.push(formal);
                    }
                }
                left.sort_unstable();
            }
            (Self::Absent, Self::Absent) | (Self::Finite { .. }, Self::Absent) => {}
        }
    }

    fn projected(mut self, fields: &[u32]) -> Self {
        if let Self::Finite { formals, .. } = &mut self {
            formals.retain_mut(|formal| {
                if !formal.result_fields.starts_with(fields) {
                    return false;
                }
                formal.result_fields.drain(..fields.len());
                true
            });
        }
        self
    }

    fn enum_payload(mut self, variant: u32, field: u32) -> Self {
        if let Self::Finite { formals } = &mut self {
            formals.retain_mut(|origin| match origin.result_variant {
                Some(actual) if actual != variant => false,
                Some(_) => {
                    if origin.result_fields.first() != Some(&field) {
                        return false;
                    }
                    origin.result_fields.remove(0);
                    origin.result_variant = None;
                    true
                }
                None => {
                    origin.result_fields.clear();
                    true
                }
            });
        }
        self
    }

    fn replace_path(self, fields: &[u32], replacement: Self) -> Self {
        if fields.is_empty() {
            return replacement;
        }
        match (self, replacement) {
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::Absent, replacement) => replacement,
            (current @ Self::Finite { .. }, Self::Absent) => current,
            (
                Self::Finite {
                    formals: mut current,
                },
                Self::Finite {
                    formals: replacement,
                },
            ) => {
                current.retain(|origin| !origin.result_fields.starts_with(fields));
                for mut origin in replacement {
                    let mut result_fields = fields.to_vec();
                    result_fields.extend_from_slice(&origin.result_fields);
                    origin.result_fields = result_fields;
                    if !current.contains(&origin) {
                        current.push(origin);
                    }
                }
                current.sort_unstable();
                Self::Finite { formals: current }
            }
        }
    }
}

type OriginEnvironment = HashMap<BindingId, OriginSet>;

struct OriginFlow {
    continuation: Option<OriginEnvironment>,
    returns: OriginSet,
    gives: Vec<(OriginSet, OriginEnvironment)>,
    breaks: Vec<(CheckedLoopId, OriginEnvironment)>,
}

impl OriginFlow {
    fn continuing(environment: OriginEnvironment) -> Self {
        Self {
            continuation: Some(environment),
            returns: OriginSet::Absent,
            gives: Vec::new(),
            breaks: Vec::new(),
        }
    }
}

struct OriginAnalyzer<'a, 'b, 'unit, 'classified, 'lexed, 'source> {
    checker: &'a Checker<'unit, 'classified, 'lexed, 'source>,
    function: &'b CheckedFunction,
    summaries: &'b [OriginSet],
}

impl<'a, 'b, 'unit, 'classified, 'lexed, 'source>
    OriginAnalyzer<'a, 'b, 'unit, 'classified, 'lexed, 'source>
{
    fn analyze(&self) -> Result<OriginSet, CheckStop> {
        let mut environment = OriginEnvironment::new();
        for (ordinal, parameter) in self.function.parameters.iter().enumerate() {
            let leaves = self.checker.type_state_leaf_paths(parameter.ty)?;
            let origin = if leaves.is_empty() {
                OriginSet::Absent
            } else {
                u32::try_from(ordinal)
                    .map(|ordinal| OriginSet::formal_leaves(ordinal, leaves))
                    .unwrap_or(OriginSet::Unknown)
            };
            environment.insert(parameter.binding, origin);
        }
        Ok(self.scan_block(&self.function.body, environment)?.returns)
    }

    fn scan_block(
        &self,
        statements: &[CheckedStatement],
        environment: OriginEnvironment,
    ) -> Result<OriginFlow, CheckStop> {
        let mut flow = OriginFlow::continuing(environment);
        for statement in statements {
            let Some(environment) = flow.continuation.take() else {
                break;
            };
            let next = self.scan_statement(statement, environment)?;
            flow.returns.union(next.returns);
            flow.gives.extend(next.gives);
            flow.breaks.extend(next.breaks);
            flow.continuation = next.continuation;
        }
        Ok(flow)
    }

    fn scan_statement(
        &self,
        statement: &CheckedStatement,
        mut environment: OriginEnvironment,
    ) -> Result<OriginFlow, CheckStop> {
        match statement {
            CheckedStatement::Let { binding, value, .. } => {
                let origin = self.expression(value, &environment)?;
                environment.insert(*binding, origin);
                Ok(OriginFlow::continuing(environment))
            }
            // [PROV-6] a disposed value binds nothing and leaves no origin.
            CheckedStatement::Dispose { value, .. } => {
                self.expression(value, &environment)?;
                Ok(OriginFlow::continuing(environment))
            }
            // [CALL-4] binder i takes result ordinal i, which is field i of
            // the one result-list value the call produced.
            CheckedStatement::DestructuringLet {
                bindings, value, ..
            } => {
                let origin = self.expression(value, &environment)?;
                for (ordinal, (binding, _)) in bindings.iter().enumerate() {
                    let field = u32::try_from(ordinal)
                        .map_err(|_| crate::SemanticCompilerFailure::CounterOverflow)?;
                    environment.insert(*binding, origin.clone().projected(&[field]));
                }
                Ok(OriginFlow::continuing(environment))
            }
            CheckedStatement::SetList {
                targets, values, ..
            } => {
                // [LIV-2] ordinal i's origin is result ordinal i of the one
                // call, or the whole origin of written value i.
                let mut ordinal_origins = Vec::with_capacity(targets.len());
                match values {
                    CheckedCommitValues::ResultList { value, .. } => {
                        let origin = self.expression(value, &environment)?;
                        for ordinal in 0..targets.len() {
                            let field = u32::try_from(ordinal)
                                .map_err(|_| crate::SemanticCompilerFailure::CounterOverflow)?;
                            ordinal_origins.push(origin.clone().projected(&[field]));
                        }
                    }
                    CheckedCommitValues::Written(values) => {
                        for value in values {
                            ordinal_origins.push(self.expression(value, &environment)?);
                        }
                    }
                }
                for (target, ordinal_origin) in targets.iter().zip(ordinal_origins) {
                    let binding = target.binding();
                    let current = environment
                        .get(&binding)
                        .cloned()
                        .unwrap_or(OriginSet::Unknown);
                    let updated = match target {
                        CheckedSetTarget::Place(place) if place.fields.is_empty() => ordinal_origin,
                        CheckedSetTarget::Place(place)
                            if self.checker.type_carries_identity(target.ty())? =>
                        {
                            current.replace_path(&place.fields, ordinal_origin)
                        }
                        CheckedSetTarget::Place(_)
                        | CheckedSetTarget::ArrayIndex(_)
                        | CheckedSetTarget::BufferIndex(_) => current,
                    };
                    environment.insert(binding, updated);
                }
                Ok(OriginFlow::continuing(environment))
            }
            CheckedStatement::PropagateLet {
                binding,
                scrutinee,
                ok_type,
                error_type,
                ..
            } => {
                let origin = self.expression(scrutinee, &environment)?;
                let returns = if !self.checker.type_carries_identity(*error_type)? {
                    OriginSet::Absent
                } else {
                    origin.clone().enum_payload(1, 0)
                };
                environment.insert(
                    *binding,
                    if !self.checker.type_carries_identity(*ok_type)? {
                        OriginSet::Absent
                    } else {
                        origin.enum_payload(0, 0)
                    },
                );
                Ok(OriginFlow {
                    continuation: Some(environment),
                    returns,
                    gives: Vec::new(),
                    breaks: Vec::new(),
                })
            }
            CheckedStatement::Set { target, value, .. } => {
                let origin = self.expression(value, &environment)?;
                let binding = target.binding();
                let current = environment
                    .get(&binding)
                    .cloned()
                    .unwrap_or(OriginSet::Unknown);
                let updated = match target {
                    CheckedSetTarget::Place(place) if place.fields.is_empty() => origin,
                    CheckedSetTarget::Place(place)
                        if self.checker.type_carries_identity(target.ty())? =>
                    {
                        current.replace_path(&place.fields, origin)
                    }
                    CheckedSetTarget::Place(_)
                    | CheckedSetTarget::ArrayIndex(_)
                    | CheckedSetTarget::BufferIndex(_) => current,
                };
                environment.insert(binding, updated);
                Ok(OriginFlow::continuing(environment))
            }
            CheckedStatement::Replace {
                binding,
                target,
                value,
                ..
            } => {
                let replacement = self.expression(value, &environment)?;
                let target_binding = target.binding();
                let previous = environment
                    .get(&target_binding)
                    .cloned()
                    .unwrap_or(OriginSet::Unknown);
                let extracted = match target {
                    CheckedSetTarget::Place(place)
                        if self.checker.type_carries_identity(target.ty())? =>
                    {
                        previous.clone().projected(&place.fields)
                    }
                    _ => OriginSet::Absent,
                };
                environment.insert(*binding, extracted);
                let updated = match target {
                    CheckedSetTarget::Place(place) if place.fields.is_empty() => replacement,
                    CheckedSetTarget::Place(place)
                        if self.checker.type_carries_identity(target.ty())? =>
                    {
                        previous.replace_path(&place.fields, replacement)
                    }
                    CheckedSetTarget::Place(_)
                    | CheckedSetTarget::ArrayIndex(_)
                    | CheckedSetTarget::BufferIndex(_) => previous,
                };
                environment.insert(target_binding, updated);
                Ok(OriginFlow::continuing(environment))
            }
            CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Proof(_) => Ok(OriginFlow::continuing(environment)),
            CheckedStatement::Return { value, .. } => Ok(OriginFlow {
                continuation: None,
                returns: self.expression(value, &environment)?,
                gives: Vec::new(),
                breaks: Vec::new(),
            }),
            CheckedStatement::Give { value, .. } => Ok(OriginFlow {
                continuation: None,
                returns: OriginSet::Absent,
                gives: vec![(self.expression(value, &environment)?, environment)],
                breaks: Vec::new(),
            }),
            CheckedStatement::Match {
                scrutinee,
                arms,
                continues,
                ..
            } => self.scan_match(scrutinee, arms, *continues, false, None, environment),
            CheckedStatement::ValueMatchLet {
                binding,
                scrutinee,
                arms,
                continues,
                ..
            } => self.scan_match(
                scrutinee,
                arms,
                *continues,
                true,
                Some(*binding),
                environment,
            ),
            CheckedStatement::Loop { id, body, .. } => {
                self.scan_loop(*id, body, environment, false)
            }
            CheckedStatement::CountedRange { id, body, .. } => {
                self.scan_loop(*id, body, environment, true)
            }
            CheckedStatement::Break { target, .. } => Ok(OriginFlow {
                continuation: None,
                returns: OriginSet::Absent,
                gives: Vec::new(),
                breaks: vec![(*target, environment)],
            }),
            CheckedStatement::Region { body, .. } => self.scan_block(body, environment),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_match(
        &self,
        scrutinee: &CheckedExpression,
        arms: &[CheckedMatchArm],
        continues: bool,
        value_match: bool,
        result_binding: Option<BindingId>,
        environment: OriginEnvironment,
    ) -> Result<OriginFlow, CheckStop> {
        let scrutinee_origin = self.expression(scrutinee, &environment)?;
        let mut returns = OriginSet::Absent;
        let mut outer_gives = Vec::new();
        let mut outer_breaks = Vec::new();
        let mut continuations = Vec::new();
        let mut delivered = OriginSet::Absent;
        for arm in arms {
            let mut arm_environment = environment.clone();
            for binder in &arm.binders {
                let origin = if self.checker.type_carries_identity(binder.ty)? {
                    scrutinee_origin.clone().enum_payload(arm.tag, binder.field)
                } else {
                    OriginSet::Absent
                };
                arm_environment.insert(binder.binding, origin);
            }
            let arm_flow = self.scan_block(&arm.body, arm_environment)?;
            returns.union(arm_flow.returns);
            outer_breaks.extend(arm_flow.breaks);
            if value_match {
                for (origin, give_environment) in arm_flow.gives {
                    delivered.union(origin);
                    continuations.push(give_environment);
                }
            } else {
                outer_gives.extend(arm_flow.gives);
                if let Some(continuation) = arm_flow.continuation {
                    continuations.push(continuation);
                }
            }
        }
        let continuation = if continues {
            join_environments(continuations).map(|mut environment| {
                if let Some(binding) = result_binding {
                    environment.insert(binding, delivered);
                }
                environment
            })
        } else {
            None
        };
        Ok(OriginFlow {
            continuation,
            returns,
            gives: outer_gives,
            breaks: outer_breaks,
        })
    }

    fn scan_loop(
        &self,
        id: CheckedLoopId,
        body: &[CheckedStatement],
        entry: OriginEnvironment,
        may_skip: bool,
    ) -> Result<OriginFlow, CheckStop> {
        let mut header = entry.clone();
        loop {
            let body_flow = self.scan_block(body, header.clone())?;
            let mut candidates = vec![entry.clone()];
            if let Some(backedge) = body_flow.continuation {
                candidates.push(backedge);
            }
            let next = join_environments(candidates).unwrap_or_else(|| entry.clone());
            if next == header {
                break;
            }
            header = next;
        }

        let mut body_flow = self.scan_block(body, header)?;
        let mut exits = Vec::new();
        if may_skip {
            exits.push(entry);
        }
        if let Some(backedge) = body_flow.continuation.take()
            && may_skip
        {
            exits.push(backedge);
        }
        let mut outer_breaks = Vec::new();
        for (target, environment) in body_flow.breaks {
            if target == id {
                exits.push(environment);
            } else {
                outer_breaks.push((target, environment));
            }
        }
        Ok(OriginFlow {
            continuation: join_environments(exits),
            returns: body_flow.returns,
            gives: body_flow.gives,
            breaks: outer_breaks,
        })
    }

    fn expression(
        &self,
        expression: &CheckedExpression,
        environment: &OriginEnvironment,
    ) -> Result<OriginSet, CheckStop> {
        if self
            .checker
            .type_state_leaf_paths(expression.ty())?
            .is_empty()
        {
            return Ok(OriginSet::Absent);
        }
        let origin = match expression {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => environment
                .get(binding)
                .cloned()
                .unwrap_or(OriginSet::Unknown),
            CheckedExpression::Project {
                binding, fields, ..
            } => environment
                .get(binding)
                .cloned()
                .unwrap_or(OriginSet::Unknown)
                .projected(fields),
            CheckedExpression::SystemCall { operation, .. } => match crate::SYSTEM_OPERATIONS
                .get(usize::from(*operation))
                .map(|operation| operation.result_state_origin)
            {
                Some(crate::SystemResultStateOrigin::None) => OriginSet::Absent,
                Some(crate::SystemResultStateOrigin::Fresh) => OriginSet::fresh(),
                None => OriginSet::Unknown,
            },
            CheckedExpression::UserCall {
                function,
                arguments,
                ..
            } => match self.summaries.get(function.0 as usize) {
                Some(OriginSet::Finite { formals }) => {
                    let mut origin = OriginSet::Finite {
                        formals: Vec::new(),
                    };
                    for formal in formals {
                        let Some(argument) = arguments.get(formal.parameter as usize) else {
                            return Ok(OriginSet::Unknown);
                        };
                        let mut mapped = self
                            .expression(argument, environment)?
                            .projected(&formal.parameter_fields);
                        if let OriginSet::Finite { formals, .. } = &mut mapped {
                            for mapped in formals {
                                let mut result_fields = formal.result_fields.clone();
                                result_fields.extend_from_slice(&mapped.result_fields);
                                mapped.result_fields = result_fields;
                                if formal.result_variant.is_some() {
                                    mapped.result_variant = formal.result_variant;
                                }
                            }
                        }
                        origin.union(mapped);
                    }
                    origin
                }
                Some(OriginSet::Absent) => OriginSet::Absent,
                Some(OriginSet::Unknown) | None => OriginSet::Unknown,
            },
            CheckedExpression::ConstructStruct { fields, .. } => {
                let mut origin = OriginSet::Absent;
                for (ordinal, field) in fields.iter().enumerate() {
                    let ordinal = u32::try_from(ordinal)
                        .map_err(|_| crate::SemanticCompilerFailure::CounterOverflow)?;
                    let mut field_origin = self.expression(field, environment)?;
                    if let OriginSet::Finite { formals, .. } = &mut field_origin {
                        for formal in formals {
                            formal.result_fields.insert(0, ordinal);
                        }
                    }
                    origin.union(field_origin);
                }
                origin
            }
            CheckedExpression::ConstructEnum {
                variant, fields, ..
            } => {
                let mut origin = OriginSet::Absent;
                for (field, value) in fields.iter().enumerate() {
                    let field = u32::try_from(field)
                        .map_err(|_| crate::SemanticCompilerFailure::CounterOverflow)?;
                    let mut field_origin = self.expression(value, environment)?;
                    if let OriginSet::Finite { formals, .. } = &mut field_origin {
                        for formal in formals {
                            formal.result_fields.insert(0, field);
                            formal.result_variant = Some(*variant);
                        }
                    }
                    origin.union(field_origin);
                }
                origin
            }
            CheckedExpression::BoxNew { value, .. } | CheckedExpression::ArenaNew { value, .. } => {
                let mut origin = self.expression(value, environment)?;
                if let OriginSet::Finite { formals, .. } = &mut origin {
                    for formal in formals {
                        formal.result_fields.clear();
                        formal.result_variant = None;
                    }
                }
                origin
            }
            _ => {
                let mut origin = OriginSet::Absent;
                for child in super::super::model::expression_children(expression) {
                    origin.union(self.expression(child, environment)?);
                }
                origin
            }
        };
        Ok(match origin {
            // `Absent` is the recursive fixed-point bottom only for a user
            // call whose callee has not published a route yet. Every other
            // affine-producing expression creates an invocation-local owner.
            OriginSet::Absent if matches!(expression, CheckedExpression::UserCall { .. }) => {
                OriginSet::Absent
            }
            OriginSet::Absent => OriginSet::fresh(),
            other => other,
        })
    }
}

fn join_environments(environments: Vec<OriginEnvironment>) -> Option<OriginEnvironment> {
    let mut environments = environments.into_iter();
    let mut joined = environments.next()?;
    for environment in environments {
        for (binding, origin) in environment {
            match joined.get_mut(&binding) {
                Some(current) => current.union(origin),
                None => {
                    joined.insert(binding, origin);
                }
            }
        }
    }
    Some(joined)
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn derive_result_state_origins(&mut self) -> Result<(), CheckStop> {
        let initial = self
            .signatures
            .iter()
            .map(
                |signature| match self.type_state_leaf_paths(signature.result) {
                    Ok(paths) if paths.is_empty() => CheckedResultStateOrigin::NoState,
                    Ok(_) | Err(_) => CheckedResultStateOrigin::Unknown,
                },
            )
            .collect::<Vec<_>>();
        self.result_state_origins.replace(initial);

        self.deriving_result_state_origin.set(true);
        let preliminary = (0..self.signatures.len())
            .map(|index| self.check_function_interning_nominals(index))
            .collect::<Result<Vec<_>, _>>();
        self.deriving_result_state_origin.set(false);
        let preliminary = preliminary?;
        let functions = preliminary
            .iter()
            .map(|checked| &checked.function)
            .collect::<Vec<_>>();

        let mut summaries = functions
            .iter()
            .map(
                |function| match self.type_state_leaf_paths(function.result) {
                    Ok(_) => OriginSet::Absent,
                    Err(_) => OriginSet::Unknown,
                },
            )
            .collect::<Vec<_>>();
        loop {
            let mut next = Vec::with_capacity(functions.len());
            for function in &functions {
                next.push(if self.type_state_leaf_paths(function.result)?.is_empty() {
                    OriginSet::Absent
                } else {
                    OriginAnalyzer {
                        checker: self,
                        function,
                        summaries: &summaries,
                    }
                    .analyze()?
                });
            }
            if next == summaries {
                break;
            }
            summaries = next;
        }

        let resolved = functions
            .iter()
            .zip(summaries)
            .map(|(function, origin)| {
                if self.type_state_leaf_paths(function.result)?.is_empty() {
                    return Ok(CheckedResultStateOrigin::NoState);
                }
                Ok(match origin {
                    OriginSet::Finite { formals } => CheckedResultStateOrigin::Finite { formals },
                    OriginSet::Absent | OriginSet::Unknown => CheckedResultStateOrigin::Unknown,
                })
            })
            .collect::<Result<Vec<_>, CheckStop>>()?;
        self.result_state_origins.replace(resolved);
        Ok(())
    }
}
