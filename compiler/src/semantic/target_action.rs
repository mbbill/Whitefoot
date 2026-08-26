//! Closed-world target-action summaries for concrete functions.
//!
//! Target execution is compiler metadata. This pass joins direct system and
//! derived-release records through the ordinary call graph without adding a
//! source effect atom or consulting entailment facts.

use crate::TargetAction;

use super::model::{
    CheckedAuthoritySummary, CheckedAuthorityUse, CheckedDrop, CheckedExpression, CheckedFunction,
    CheckedSetTarget, CheckedStatement, FunctionId, expression_children,
};

/// Installs the least conservative target-action fixed point.
pub(crate) fn derive_target_actions(functions: &mut [CheckedFunction]) {
    let mut direct = vec![TargetAction::INLINE; functions.len()];
    let mut edges = vec![Vec::new(); functions.len()];
    for (index, function) in functions.iter().enumerate() {
        if function.id.0 as usize != index {
            direct[index] = TargetAction::CONSERVATIVE;
        }
        collect_statements(&function.body, &mut direct[index], &mut edges[index]);
    }

    let mut summaries = direct.clone();
    loop {
        let mut changed = false;
        for index in 0..functions.len() {
            let mut next = direct[index];
            for callee in &edges[index] {
                next = next.union(
                    summaries
                        .get(callee.0 as usize)
                        .copied()
                        .unwrap_or(TargetAction::CONSERVATIVE),
                );
            }
            if summaries[index] != next {
                summaries[index] = next;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for (function, summary) in functions.iter_mut().zip(summaries) {
        function.target_action = summary;
    }
    derive_authority_summaries(functions);
}

fn derive_authority_summaries(functions: &mut [CheckedFunction]) {
    let mut summaries = vec![CheckedAuthoritySummary::default(); functions.len()];
    loop {
        let mut next = Vec::with_capacity(functions.len());
        for function in functions.iter() {
            let parameters = function
                .parameters
                .iter()
                .map(|parameter| parameter.declaration)
                .collect::<Vec<_>>();
            let mut summary = CheckedAuthoritySummary::default();
            collect_authority_statements(&function.body, &parameters, &summaries, &mut summary);
            next.push(summary);
        }
        if next == summaries {
            break;
        }
        summaries = next;
    }
    for (function, summary) in functions.iter_mut().zip(summaries) {
        function.authority_summary = summary;
    }
}

fn collect_authority_statements(
    statements: &[CheckedStatement],
    parameters: &[crate::DeclarationId],
    summaries: &[CheckedAuthoritySummary],
    summary: &mut CheckedAuthoritySummary,
) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::Claim {
                condition: value, ..
            } => {
                collect_authority_expression(value, parameters, summaries, summary);
            }
            CheckedStatement::DropExpression {
                value,
                capability_origins,
                release,
            } => {
                collect_authority_expression(value, parameters, summaries, summary);
                collect_release_authority(
                    capability_origins.as_ref(),
                    release.row,
                    parameters,
                    summary,
                );
            }
            CheckedStatement::PropagateLet {
                scrutinee,
                error_drops,
                ..
            } => {
                collect_authority_expression(scrutinee, parameters, summaries, summary);
                collect_authority_drops(error_drops, parameters, summary);
            }
            CheckedStatement::Set { target, value, .. }
            | CheckedStatement::Replace { target, value, .. } => {
                collect_authority_target(target, parameters, summaries, summary);
                collect_authority_expression(value, parameters, summaries, summary);
            }
            CheckedStatement::Return { value, drops, .. }
            | CheckedStatement::Give { value, drops, .. } => {
                collect_authority_expression(value, parameters, summaries, summary);
                collect_authority_drops(drops, parameters, summary);
            }
            CheckedStatement::Match {
                scrutinee, arms, ..
            }
            | CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                collect_authority_expression(scrutinee, parameters, summaries, summary);
                for arm in arms {
                    collect_authority_statements(&arm.body, parameters, summaries, summary);
                    collect_authority_drops(&arm.fallthrough_drops, parameters, summary);
                }
            }
            CheckedStatement::Loop {
                body,
                backedge_drops,
                ..
            }
            | CheckedStatement::CountedRange {
                body,
                backedge_drops,
                ..
            } => {
                collect_authority_statements(body, parameters, summaries, summary);
                collect_authority_drops(backedge_drops, parameters, summary);
            }
            CheckedStatement::Region {
                body,
                fallthrough_drops,
                ..
            } => {
                collect_authority_statements(body, parameters, summaries, summary);
                collect_authority_drops(fallthrough_drops, parameters, summary);
            }
            CheckedStatement::Break { drops, .. } => {
                collect_authority_drops(drops, parameters, summary);
            }
        }
    }
}

fn collect_authority_drops(
    drops: &[CheckedDrop],
    parameters: &[crate::DeclarationId],
    summary: &mut CheckedAuthoritySummary,
) {
    for drop in drops {
        collect_release_authority(
            drop.capability_origins.as_ref(),
            drop.release.row,
            parameters,
            summary,
        );
    }
}

fn collect_release_authority(
    origins: Option<&super::model::CheckedCapabilityOrigins>,
    row: crate::SystemReleaseRow,
    parameters: &[crate::DeclarationId],
    summary: &mut CheckedAuthoritySummary,
) {
    let authority = match row.authority {
        crate::SystemReleaseAuthority::None => return,
        crate::SystemReleaseAuthority::Known(authority) => authority,
        crate::SystemReleaseAuthority::Unknown => {
            summary.unknown = true;
            return;
        }
    };
    let Some(origins) = origins else {
        summary.unknown = true;
        return;
    };
    for origin in &origins.formals {
        let Some(parameter) = parameters.iter().position(|candidate| candidate == origin) else {
            summary.unknown = true;
            continue;
        };
        let Ok(parameter) = u32::try_from(parameter) else {
            summary.unknown = true;
            continue;
        };
        add_authority_use(
            summary,
            CheckedAuthorityUse {
                parameter,
                family: authority.family,
                fragment: authority.fragment,
            },
        );
    }
}

fn collect_authority_target(
    target: &CheckedSetTarget,
    parameters: &[crate::DeclarationId],
    summaries: &[CheckedAuthoritySummary],
    summary: &mut CheckedAuthoritySummary,
) {
    match target {
        CheckedSetTarget::Place(_) => {}
        CheckedSetTarget::ArrayIndex(target) => {
            collect_authority_expression(&target.offset, parameters, summaries, summary);
        }
        CheckedSetTarget::BufferIndex(target) => {
            collect_authority_expression(&target.offset, parameters, summaries, summary);
        }
    }
}

fn collect_authority_expression(
    expression: &CheckedExpression,
    parameters: &[crate::DeclarationId],
    summaries: &[CheckedAuthoritySummary],
    summary: &mut CheckedAuthoritySummary,
) {
    match expression {
        CheckedExpression::SystemCall {
            operation,
            arguments,
            ..
        } => {
            let Some(authority) = crate::SYSTEM_OPERATIONS
                .get(usize::from(*operation))
                .and_then(|operation| operation.authority)
            else {
                if crate::SYSTEM_OPERATIONS
                    .get(usize::from(*operation))
                    .is_none()
                {
                    summary.unknown = true;
                }
                return;
            };
            project_authority_use(
                arguments.get(usize::from(authority.parameter)),
                authority.family,
                authority.fragment,
                parameters,
                summary,
            );
        }
        CheckedExpression::UserCall {
            function,
            arguments,
            ..
        } => {
            let Some(callee) = summaries.get(function.0 as usize) else {
                summary.unknown = true;
                return;
            };
            summary.unknown |= callee.unknown;
            for usage in &callee.uses {
                project_authority_use(
                    arguments.get(usage.parameter as usize),
                    usage.family,
                    usage.fragment,
                    parameters,
                    summary,
                );
            }
        }
        CheckedExpression::Project { residual_drops, .. } => {
            for drop in residual_drops {
                collect_release_authority(
                    drop.capability_origins.as_ref(),
                    drop.release.row,
                    parameters,
                    summary,
                );
            }
        }
        _ => {}
    }
    for child in expression_children(expression) {
        collect_authority_expression(child, parameters, summaries, summary);
    }
}

fn project_authority_use(
    argument: Option<&CheckedExpression>,
    family: crate::SystemAuthorityFamily,
    fragment: crate::SystemAuthorityFragment,
    parameters: &[crate::DeclarationId],
    summary: &mut CheckedAuthoritySummary,
) {
    let Some(argument) = argument else {
        summary.unknown = true;
        return;
    };
    let origins = match argument {
        CheckedExpression::Binding {
            capability_origins, ..
        }
        | CheckedExpression::Project {
            capability_origins, ..
        }
        | CheckedExpression::BorrowSystemResource {
            capability_origins, ..
        } => capability_origins.as_ref(),
        _ => {
            summary.unknown = true;
            return;
        }
    };
    let Some(origins) = origins else {
        summary.unknown = true;
        return;
    };
    for origin in &origins.formals {
        let Some(parameter) = parameters.iter().position(|candidate| candidate == origin) else {
            summary.unknown = true;
            continue;
        };
        let Ok(parameter) = u32::try_from(parameter) else {
            summary.unknown = true;
            continue;
        };
        add_authority_use(
            summary,
            CheckedAuthorityUse {
                parameter,
                family,
                fragment,
            },
        );
    }
}

fn add_authority_use(summary: &mut CheckedAuthoritySummary, usage: CheckedAuthorityUse) {
    if summary.uses.contains(&usage) {
        return;
    }
    summary.uses.push(usage);
    summary.uses.sort_by_key(|usage| {
        (
            usage.parameter,
            usage.family.spelling(),
            usage.fragment.spelling(),
        )
    });
}

fn collect_statements(
    statements: &[CheckedStatement],
    direct: &mut TargetAction,
    edges: &mut Vec<FunctionId>,
) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::Claim {
                condition: value, ..
            } => collect_expression(value, direct, edges),
            CheckedStatement::PropagateLet {
                scrutinee,
                error_drops,
                ..
            } => {
                collect_expression(scrutinee, direct, edges);
                collect_drops(error_drops, direct);
            }
            CheckedStatement::Set { target, value, .. }
            | CheckedStatement::Replace { target, value, .. } => {
                collect_set_target(target, direct, edges);
                collect_expression(value, direct, edges);
            }
            CheckedStatement::DropExpression { value, release, .. } => {
                collect_expression(value, direct, edges);
                *direct = direct.union(release.row.target_action);
            }
            CheckedStatement::Return { value, drops, .. }
            | CheckedStatement::Give { value, drops, .. } => {
                collect_expression(value, direct, edges);
                collect_drops(drops, direct);
            }
            CheckedStatement::Match {
                scrutinee, arms, ..
            }
            | CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                collect_expression(scrutinee, direct, edges);
                for arm in arms {
                    collect_statements(&arm.body, direct, edges);
                    collect_drops(&arm.fallthrough_drops, direct);
                }
            }
            CheckedStatement::Loop {
                body,
                backedge_drops,
                ..
            }
            | CheckedStatement::CountedRange {
                body,
                backedge_drops,
                ..
            } => {
                collect_statements(body, direct, edges);
                collect_drops(backedge_drops, direct);
            }
            CheckedStatement::Break { drops, .. } => collect_drops(drops, direct),
            CheckedStatement::Region {
                body,
                fallthrough_drops,
                ..
            } => {
                collect_statements(body, direct, edges);
                collect_drops(fallthrough_drops, direct);
            }
        }
    }
}

fn collect_set_target(
    target: &CheckedSetTarget,
    direct: &mut TargetAction,
    edges: &mut Vec<FunctionId>,
) {
    match target {
        CheckedSetTarget::Place(_) => {}
        CheckedSetTarget::ArrayIndex(target) => {
            collect_expression(&target.offset, direct, edges);
        }
        CheckedSetTarget::BufferIndex(target) => {
            collect_expression(&target.offset, direct, edges);
        }
    }
}

fn collect_expression(
    expression: &CheckedExpression,
    direct: &mut TargetAction,
    edges: &mut Vec<FunctionId>,
) {
    match expression {
        CheckedExpression::UserCall { function, .. } => edges.push(*function),
        CheckedExpression::SystemCall { target_action, .. } => {
            *direct = direct.union(*target_action);
        }
        CheckedExpression::Project { residual_drops, .. } => {
            for drop in residual_drops {
                *direct = direct.union(drop.release.row.target_action);
            }
        }
        _ => {}
    }
    for child in expression_children(expression) {
        collect_expression(child, direct, edges);
    }
}

fn collect_drops(drops: &[CheckedDrop], direct: &mut TargetAction) {
    for drop in drops {
        *direct = direct.union(drop.release.row.target_action);
    }
}
