use std::collections::HashMap;

use super::super::model::{
    BindingId, CheckedBorrowedStateOrigin, CheckedCommitValues, CheckedExpression, CheckedFunction,
    CheckedLoopId, CheckedMatchArm, CheckedMode, CheckedPlaceStep, CheckedResultStateOrigin,
    CheckedResultStatePath, CheckedSetTarget, CheckedStateOrigin, CheckedStateOrigins,
    CheckedStatePath, CheckedStatement,
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
            (Self::Finite { .. }, Self::Absent) => Self::Absent,
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OriginEnvironment {
    values: HashMap<BindingId, OriginSet>,
    // A missing entry is an ordinary owner. None is a borrowed result whose
    // signature gives a provenance ceiling, not an exact returned location.
    aliases: HashMap<BindingId, Option<(BindingId, Vec<u32>)>>,
}

impl OriginEnvironment {
    fn new() -> Self {
        Self::default()
    }
    fn insert(&mut self, binding: BindingId, origin: OriginSet) {
        self.values.insert(binding, origin);
    }
    fn place(&self, binding: BindingId, fields: &[u32]) -> Option<(BindingId, Vec<u32>)> {
        let (root, mut path) = match self.aliases.get(&binding) {
            Some(alias) => alias.clone()?,
            None => (binding, Vec::new()),
        };
        path.extend_from_slice(fields);
        Some((root, path))
    }
    fn read(&self, binding: BindingId, fields: &[u32]) -> OriginSet {
        self.place(binding, fields)
            .map_or(OriginSet::Unknown, |(root, fields)| {
                self.values
                    .get(&root)
                    .cloned()
                    .unwrap_or(OriginSet::Unknown)
                    .projected(&fields)
            })
    }
    fn write(&mut self, place: Option<(BindingId, Vec<u32>)>, origin: OriginSet) {
        if let Some((root, fields)) = place {
            let current = self.values.remove(&root).unwrap_or(OriginSet::Unknown);
            self.values
                .insert(root, current.replace_path(&fields, origin));
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OriginSummary {
    result: OriginSet,
    borrowed: Vec<(u32, OriginSet)>,
}

struct OriginFlow {
    continuation: Option<OriginEnvironment>,
    returns: OriginSet,
    gives: Vec<(OriginSet, OriginEnvironment)>,
    breaks: Vec<(CheckedLoopId, OriginEnvironment)>,
    exits: Vec<OriginEnvironment>,
}

impl OriginFlow {
    fn continuing(environment: OriginEnvironment) -> Self {
        Self {
            continuation: Some(environment),
            returns: OriginSet::Absent,
            gives: Vec::new(),
            breaks: Vec::new(),
            exits: Vec::new(),
        }
    }
}

struct OriginAnalyzer<'a, 'b, 'unit, 'classified, 'lexed, 'source> {
    checker: &'a Checker<'unit, 'classified, 'lexed, 'source>,
    function: &'b CheckedFunction,
    summaries: &'b [OriginSummary],
}

impl<'a, 'b, 'unit, 'classified, 'lexed, 'source>
    OriginAnalyzer<'a, 'b, 'unit, 'classified, 'lexed, 'source>
{
    fn analyze(&self) -> Result<OriginSummary, CheckStop> {
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
            if parameter.mode != CheckedMode::Own {
                environment
                    .aliases
                    .insert(parameter.binding, Some((parameter.binding, Vec::new())));
            }
        }
        let flow = self.scan_block(&self.function.body, environment)?;
        let exit = join_environments(flow.exits);
        let mut borrowed = Vec::new();
        for (ordinal, parameter) in self.function.parameters.iter().enumerate() {
            if matches!(parameter.mode, CheckedMode::Unique(_))
                && self.checker.type_carries_identity(parameter.ty)?
            {
                borrowed.push((
                    u32::try_from(ordinal)
                        .map_err(|_| crate::SemanticCompilerFailure::CounterOverflow)?,
                    exit.as_ref()
                        .map_or(OriginSet::Absent, |exit| exit.read(parameter.binding, &[])),
                ));
            }
        }
        Ok(OriginSummary {
            result: flow.returns,
            borrowed,
        })
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
            flow.exits.extend(next.exits);
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
                let origin = self.expression(value, &mut environment)?;
                if self.is_borrow_expression(value, &environment) {
                    let place = self.borrow_place(value, &environment);
                    environment.aliases.insert(*binding, place);
                }
                environment.insert(*binding, origin);
                Ok(OriginFlow::continuing(environment))
            }
            // [PROV-6] a disposed value binds nothing and leaves no origin.
            CheckedStatement::Dispose { value, .. } => {
                self.expression(value, &mut environment)?;
                Ok(OriginFlow::continuing(environment))
            }
            // [CALL-4] binder i takes result ordinal i, which is field i of
            // the one result-list value the call produced.
            CheckedStatement::DestructuringLet {
                bindings, value, ..
            } => {
                let origin = self.expression(value, &mut environment)?;
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
                        let origin = self.expression(value, &mut environment)?;
                        for ordinal in 0..targets.len() {
                            let field = u32::try_from(ordinal)
                                .map_err(|_| crate::SemanticCompilerFailure::CounterOverflow)?;
                            ordinal_origins.push(origin.clone().projected(&[field]));
                        }
                    }
                    CheckedCommitValues::Written(values) => {
                        for value in values {
                            ordinal_origins.push(self.expression(value, &mut environment)?);
                        }
                    }
                }
                for (target, ordinal_origin) in targets.iter().zip(ordinal_origins) {
                    if self.checker.type_carries_identity(target.ty())? {
                        let place = self.target_place(target, &environment);
                        environment.write(place, ordinal_origin);
                    }
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
                let origin = self.expression(scrutinee, &mut environment)?;
                let returns = if !self.checker.type_carries_identity(*error_type)? {
                    OriginSet::Absent
                } else {
                    origin.clone().enum_payload(1, 0)
                };
                let exits = vec![environment.clone()];
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
                    exits,
                })
            }
            CheckedStatement::Set { target, value, .. } => {
                let origin = self.expression(value, &mut environment)?;
                if self.checker.type_carries_identity(target.ty())? {
                    let place = self.target_place(target, &environment);
                    environment.write(place, origin);
                }
                Ok(OriginFlow::continuing(environment))
            }
            CheckedStatement::Replace {
                binding,
                target,
                value,
                ..
            } => {
                let replacement = self.expression(value, &mut environment)?;
                let place = self.target_place(target, &environment);
                let extracted = if self.checker.type_carries_identity(target.ty())? {
                    place.as_ref().map_or(OriginSet::Unknown, |(root, fields)| {
                        environment.read(*root, fields)
                    })
                } else {
                    OriginSet::Absent
                };
                environment.insert(*binding, extracted);
                if self.checker.type_carries_identity(target.ty())? {
                    environment.write(place, replacement);
                }
                Ok(OriginFlow::continuing(environment))
            }
            CheckedStatement::Evaluate(value) | CheckedStatement::DropExpression { value, .. } => {
                self.expression(value, &mut environment)?;
                Ok(OriginFlow::continuing(environment))
            }
            CheckedStatement::Proof(_) => Ok(OriginFlow::continuing(environment)),
            CheckedStatement::Return { value, .. } => {
                let returns = self.expression(value, &mut environment)?;
                Ok(OriginFlow {
                    continuation: None,
                    returns,
                    gives: Vec::new(),
                    breaks: Vec::new(),
                    exits: vec![environment],
                })
            }
            CheckedStatement::Give { value, .. } => Ok(OriginFlow {
                continuation: None,
                returns: OriginSet::Absent,
                gives: vec![(self.expression(value, &mut environment)?, environment)],
                exits: Vec::new(),
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
                exits: Vec::new(),
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
        mut environment: OriginEnvironment,
    ) -> Result<OriginFlow, CheckStop> {
        let scrutinee_origin = self.expression(scrutinee, &mut environment)?;
        let mut returns = OriginSet::Absent;
        let mut exits = Vec::new();
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
                if binder.mode != CheckedMode::Own {
                    arm_environment.aliases.insert(binder.binding, None);
                }
            }
            let arm_flow = self.scan_block(&arm.body, arm_environment)?;
            returns.union(arm_flow.returns);
            exits.extend(arm_flow.exits);
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
            exits,
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
            exits: body_flow.exits,
            gives: body_flow.gives,
            breaks: outer_breaks,
        })
    }

    fn target_place(
        &self,
        target: &CheckedSetTarget,
        environment: &OriginEnvironment,
    ) -> Option<(BindingId, Vec<u32>)> {
        match target {
            CheckedSetTarget::Place(place) => environment.place(place.binding, &place.fields),
            CheckedSetTarget::Storage(root) => {
                let fields = static_fields(&root.path)?;
                environment.place(root.binding()?, &fields)
            }
            _ => None,
        }
    }

    fn is_borrow_expression(
        &self,
        expression: &CheckedExpression,
        environment: &OriginEnvironment,
    ) -> bool {
        match expression {
            CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::ReborrowAddressed { .. } => true,
            CheckedExpression::Binding { binding, .. } => environment.aliases.contains_key(binding),
            CheckedExpression::UserCall { result_borrow, .. } => result_borrow.is_some(),
            _ => false,
        }
    }

    fn borrow_place(
        &self,
        expression: &CheckedExpression,
        environment: &OriginEnvironment,
    ) -> Option<(BindingId, Vec<u32>)> {
        match expression {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. } => {
                environment.place(*binding, &[])
            }
            CheckedExpression::BorrowSystemResource {
                binding, fields, ..
            } => environment.place(*binding, fields),
            CheckedExpression::BorrowAddressed { root, .. } => {
                environment.place(root.binding()?, &static_fields(&root.path)?)
            }
            _ => None,
        }
    }

    // Replay and the ordinary call checker share the same field substitution.
    // The bridge uses this function's real formal declarations; no synthetic
    // source identity or recursive call-history node is introduced.
    fn instantiate(&self, image: &OriginSet, arguments: &[OriginSet]) -> OriginSet {
        let OriginSet::Finite { formals } = image else {
            return image.clone();
        };
        if formals.iter().any(|route| {
            matches!(
                arguments.get(route.parameter as usize),
                Some(OriginSet::Absent)
            )
        }) {
            return OriginSet::Absent;
        }
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                OriginSet::Absent => None,
                OriginSet::Unknown => Some(CheckedStateOrigins::unknown()),
                OriginSet::Finite { formals } => {
                    let mut origins = CheckedStateOrigins::fresh();
                    for route in formals {
                        let Some(parameter) =
                            self.function.parameters.get(route.parameter as usize)
                        else {
                            return Some(CheckedStateOrigins::unknown());
                        };
                        origins.formals.push(CheckedStateOrigin {
                            value_fields: route.result_fields.clone(),
                            variant: route.result_variant,
                            source: CheckedStatePath {
                                root: parameter.declaration,
                                fields: route.parameter_fields.clone(),
                            },
                        });
                    }
                    Some(origins)
                }
            })
            .collect::<Vec<_>>();
        let instantiated = CheckedStateOrigins::instantiate(
            &CheckedResultStateOrigin::Finite {
                formals: formals.clone(),
            },
            &arguments,
        );
        if instantiated.unknown {
            return OriginSet::Unknown;
        }
        let mut formals = Vec::new();
        for origin in instantiated.formals {
            let Some(ordinal) = self
                .function
                .parameters
                .iter()
                .position(|parameter| parameter.declaration == origin.source.root)
            else {
                return OriginSet::Unknown;
            };
            let Ok(parameter) = u32::try_from(ordinal) else {
                return OriginSet::Unknown;
            };
            formals.push(CheckedResultStatePath {
                result_fields: origin.value_fields,
                result_variant: origin.variant,
                parameter,
                parameter_fields: origin.source.fields,
            });
        }
        formals.sort_unstable();
        formals.dedup();
        OriginSet::Finite { formals }
    }

    fn expression(
        &self,
        expression: &CheckedExpression,
        environment: &mut OriginEnvironment,
    ) -> Result<OriginSet, CheckStop> {
        let origin = match expression {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => environment.read(*binding, &[]),
            CheckedExpression::BorrowAddressed { root, .. }
            | CheckedExpression::ReadStorage { root, .. } => {
                let Some(binding) = root.binding() else {
                    return Ok(OriginSet::fresh());
                };
                let fields = root
                    .path
                    .iter()
                    .map(|step| match step {
                        crate::semantic::model::CheckedPlaceStep::Field(field) => Some(*field),
                        crate::semantic::model::CheckedPlaceStep::BoxReferent(_)
                        | crate::semantic::model::CheckedPlaceStep::Subscript(_) => None,
                    })
                    .collect::<Option<Vec<_>>>();
                fields.map_or(OriginSet::Unknown, |fields| {
                    environment.read(binding, &fields)
                })
            }
            CheckedExpression::Project {
                binding, fields, ..
            }
            | CheckedExpression::BorrowSystemResource {
                binding, fields, ..
            } => environment.read(*binding, fields),
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
            } => {
                let mut entry = Vec::with_capacity(arguments.len());
                let mut places = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    entry.push(self.expression(argument, environment)?);
                    places.push(self.borrow_place(argument, environment));
                }
                if let Some(summary) = self.summaries.get(function.0 as usize) {
                    let result = self.instantiate(&summary.result, &entry);
                    let updates = summary
                        .borrowed
                        .iter()
                        .map(|(ordinal, image)| {
                            (
                                places.get(*ordinal as usize).cloned().flatten(),
                                self.instantiate(image, &entry),
                            )
                        })
                        .collect::<Vec<_>>();
                    for (place, image) in updates {
                        environment.write(place, image);
                    }
                    result
                } else {
                    OriginSet::Unknown
                }
            }
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
        if !self.checker.type_carries_identity(expression.ty())? {
            return Ok(OriginSet::Absent);
        }
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
        for (binding, origin) in environment.values {
            match joined.values.get_mut(&binding) {
                Some(current) => current.union(origin),
                None => {
                    joined.insert(binding, origin);
                }
            }
        }
        for (binding, alias) in environment.aliases {
            joined
                .aliases
                .entry(binding)
                .and_modify(|old| {
                    if *old != alias {
                        *old = None;
                    }
                })
                .or_insert(alias);
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
            .map(|function| {
                let mut borrowed = Vec::new();
                for (ordinal, parameter) in function.parameters.iter().enumerate() {
                    if matches!(parameter.mode, CheckedMode::Unique(_))
                        && self.type_carries_identity(parameter.ty)?
                    {
                        let ordinal = u32::try_from(ordinal)
                            .map_err(|_| crate::SemanticCompilerFailure::CounterOverflow)?;
                        borrowed.push((ordinal, OriginSet::Absent));
                    }
                }
                Ok(OriginSummary {
                    result: OriginSet::Absent,
                    borrowed,
                })
            })
            .collect::<Result<Vec<_>, CheckStop>>()?;
        loop {
            let next = functions
                .iter()
                .map(|function| {
                    OriginAnalyzer {
                        checker: self,
                        function,
                        summaries: &summaries,
                    }
                    .analyze()
                })
                .collect::<Result<Vec<_>, _>>()?;
            if next == summaries {
                break;
            }
            summaries = next;
        }
        let mut resolved_results = Vec::with_capacity(functions.len());
        let mut resolved_borrows = Vec::with_capacity(functions.len());
        for (function, summary) in functions.iter().zip(summaries) {
            resolved_results.push(if self.type_carries_identity(function.result)? {
                export_origin(summary.result)
            } else {
                CheckedResultStateOrigin::NoState
            });
            resolved_borrows.push(
                summary
                    .borrowed
                    .into_iter()
                    .map(|(parameter, origin)| CheckedBorrowedStateOrigin {
                        parameter,
                        origin: export_origin(origin),
                    })
                    .collect(),
            );
        }
        self.result_state_origins.replace(resolved_results);
        self.borrowed_state_origins.replace(resolved_borrows);
        Ok(())
    }
}

fn static_fields(path: &[CheckedPlaceStep]) -> Option<Vec<u32>> {
    path.iter()
        .map(|step| match step {
            CheckedPlaceStep::Field(field) => Some(*field),
            _ => None,
        })
        .collect()
}

fn export_origin(origin: OriginSet) -> CheckedResultStateOrigin {
    match origin {
        OriginSet::Finite { formals } => CheckedResultStateOrigin::Finite { formals },
        OriginSet::Absent | OriginSet::Unknown => CheckedResultStateOrigin::Unknown,
    }
}
