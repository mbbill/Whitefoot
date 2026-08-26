use std::collections::HashMap;

use super::super::model::{
    BindingId, CheckedExpression, CheckedFunction, CheckedLoopId, CheckedMatchArm,
    CheckedResultAuthorityOrigin, CheckedSetTarget, CheckedStatement,
};
use super::{CheckStop, Checker};

#[derive(Clone, Debug, Eq, PartialEq)]
enum OriginSet {
    Absent,
    Finite {
        may_absent: bool,
        may_fresh: bool,
        formals: Vec<u32>,
    },
    Unknown,
}

impl OriginSet {
    fn fresh(may_absent: bool) -> Self {
        Self::Finite {
            may_absent,
            may_fresh: true,
            formals: Vec::new(),
        }
    }

    fn formal(formal: u32, may_absent: bool) -> Self {
        Self::Finite {
            may_absent,
            may_fresh: false,
            formals: vec![formal],
        }
    }

    fn union(&mut self, other: Self) {
        match (&mut *self, other) {
            (Self::Unknown, _) => {}
            (_, Self::Unknown) => *self = Self::Unknown,
            (Self::Absent, finite @ Self::Finite { .. }) => *self = finite,
            (
                Self::Finite {
                    may_absent: left_absent,
                    may_fresh: left_fresh,
                    formals: left,
                },
                Self::Finite {
                    may_absent: right_absent,
                    may_fresh: right_fresh,
                    formals: right,
                },
            ) => {
                *left_absent |= right_absent;
                *left_fresh |= right_fresh;
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
            let (minimum, maximum) = self.checker.type_capability_cardinality(parameter.ty)?;
            let origin = match maximum {
                0 => OriginSet::Absent,
                1 => u32::try_from(ordinal)
                    .map(|ordinal| OriginSet::formal(ordinal, minimum == 0))
                    .unwrap_or(OriginSet::Unknown),
                _ => OriginSet::Unknown,
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
            CheckedStatement::PropagateLet {
                binding,
                scrutinee,
                ok_type,
                error_type,
                ..
            } => {
                let origin = self.expression(scrutinee, &environment)?;
                let returns = if self.checker.type_capability_root_count(*error_type)? == 0 {
                    OriginSet::Absent
                } else {
                    origin.clone()
                };
                environment.insert(
                    *binding,
                    if self.checker.type_capability_root_count(*ok_type)? == 0 {
                        OriginSet::Absent
                    } else {
                        origin
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
                let whole_place =
                    matches!(target, CheckedSetTarget::Place(place) if place.fields.is_empty());
                environment.insert(
                    binding,
                    if whole_place {
                        origin
                    } else {
                        OriginSet::Unknown
                    },
                );
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
                environment.insert(*binding, previous);
                let whole_place =
                    matches!(target, CheckedSetTarget::Place(place) if place.fields.is_empty());
                environment.insert(
                    target_binding,
                    if whole_place {
                        replacement
                    } else {
                        OriginSet::Unknown
                    },
                );
                Ok(OriginFlow::continuing(environment))
            }
            CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Claim { .. } => Ok(OriginFlow::continuing(environment)),
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
                let origin = match self.checker.type_capability_root_count(binder.ty)? {
                    0 => OriginSet::Absent,
                    1 => scrutinee_origin.clone(),
                    _ => OriginSet::Unknown,
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
        match self.checker.type_capability_root_count(expression.ty())? {
            0 => return Ok(OriginSet::Absent),
            1 => {}
            _ => return Ok(OriginSet::Unknown),
        }
        let (minimum, maximum) = self.checker.type_capability_cardinality(expression.ty())?;
        if maximum == 0 {
            return Ok(OriginSet::Absent);
        }
        if maximum > 1 {
            return Ok(OriginSet::Unknown);
        }
        let origin = match expression {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::Project { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => environment
                .get(binding)
                .cloned()
                .unwrap_or(OriginSet::Unknown),
            CheckedExpression::SystemCall {
                operation,
                arguments,
                ..
            } => match crate::SYSTEM_OPERATIONS
                .get(usize::from(*operation))
                .map(|operation| operation.result_authority)
            {
                Some(crate::SystemResultAuthority::None) => OriginSet::Absent,
                Some(crate::SystemResultAuthority::Fresh) => OriginSet::fresh(minimum == 0),
                Some(crate::SystemResultAuthority::Parameter(ordinal)) => arguments
                    .get(usize::from(ordinal))
                    .map(|argument| self.expression(argument, environment))
                    .transpose()?
                    .unwrap_or(OriginSet::Unknown),
                None => OriginSet::Unknown,
            },
            CheckedExpression::UserCall {
                function,
                arguments,
                ..
            } => match self.summaries.get(function.0 as usize) {
                Some(OriginSet::Finite {
                    may_absent,
                    may_fresh,
                    formals,
                }) => {
                    let mut origin = OriginSet::Finite {
                        may_absent: *may_absent,
                        may_fresh: *may_fresh,
                        formals: Vec::new(),
                    };
                    for formal in formals {
                        let Some(argument) = arguments.get(*formal as usize) else {
                            return Ok(OriginSet::Unknown);
                        };
                        origin.union(self.expression(argument, environment)?);
                    }
                    origin
                }
                Some(OriginSet::Absent) => OriginSet::Absent,
                Some(OriginSet::Unknown) | None => OriginSet::Unknown,
            },
            _ => {
                let mut origin = OriginSet::Absent;
                for child in super::super::model::expression_children(expression) {
                    origin.union(self.expression(child, environment)?);
                }
                origin
            }
        };
        Ok(match origin {
            OriginSet::Absent if minimum == 0 => OriginSet::Finite {
                may_absent: true,
                may_fresh: false,
                formals: Vec::new(),
            },
            // For a required-capability result this remains the fixed-point
            // bottom: a recursive callee may not have published its finite
            // set yet. A component still at bottom after convergence becomes
            // Unknown below; a direct formal/base route can lift it first.
            OriginSet::Absent => OriginSet::Absent,
            OriginSet::Finite {
                may_absent: _,
                may_fresh,
                formals,
            } if minimum == 1 => OriginSet::Finite {
                may_absent: false,
                may_fresh,
                formals,
            },
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
    pub(super) fn derive_result_authority_origins(&mut self) -> Result<(), CheckStop> {
        let initial = self
            .signatures
            .iter()
            .map(
                |signature| match self.type_capability_root_count(signature.result) {
                    Ok(0) => CheckedResultAuthorityOrigin::NoCapability,
                    Ok(_) | Err(_) => CheckedResultAuthorityOrigin::Unknown,
                },
            )
            .collect::<Vec<_>>();
        self.result_authority_origins.replace(initial);

        self.deriving_result_authority.set(true);
        let preliminary = (0..self.signatures.len())
            .map(|index| self.check_function_interning_nominals(index))
            .collect::<Result<Vec<_>, _>>();
        self.deriving_result_authority.set(false);
        let preliminary = preliminary?;
        let functions = preliminary
            .iter()
            .map(|checked| &checked.function)
            .collect::<Vec<_>>();

        let mut summaries = functions
            .iter()
            .map(
                |function| match self.type_capability_root_count(function.result) {
                    Ok(0) => OriginSet::Absent,
                    Ok(1) => OriginSet::Absent,
                    Ok(_) | Err(_) => OriginSet::Unknown,
                },
            )
            .collect::<Vec<_>>();
        loop {
            let mut next = Vec::with_capacity(functions.len());
            for function in &functions {
                next.push(match self.type_capability_root_count(function.result)? {
                    0 => OriginSet::Absent,
                    1 => OriginAnalyzer {
                        checker: self,
                        function,
                        summaries: &summaries,
                    }
                    .analyze()?,
                    _ => OriginSet::Unknown,
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
                if self.type_capability_root_count(function.result)? == 0 {
                    return Ok(CheckedResultAuthorityOrigin::NoCapability);
                }
                Ok(match origin {
                    OriginSet::Finite {
                        may_absent,
                        may_fresh,
                        formals,
                    } => CheckedResultAuthorityOrigin::Finite {
                        may_absent,
                        may_fresh,
                        formals,
                    },
                    OriginSet::Absent | OriginSet::Unknown => CheckedResultAuthorityOrigin::Unknown,
                })
            })
            .collect::<Result<Vec<_>, CheckStop>>()?;
        self.result_authority_origins.replace(resolved);
        Ok(())
    }
}
