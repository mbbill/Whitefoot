//! Closed-world target-action summaries for concrete functions.
//!
//! Target execution is compiler metadata. This pass joins direct system and
//! derived-release records through the ordinary call graph without adding a
//! source effect atom or consulting entailment facts.

use crate::TargetAction;

use super::model::{
    CheckedDrop, CheckedExpression, CheckedFunction, CheckedSetTarget, CheckedStatement,
    FunctionId, expression_children,
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
}

fn collect_statements(
    statements: &[CheckedStatement],
    direct: &mut TargetAction,
    edges: &mut Vec<FunctionId>,
) {
    for statement in statements {
        match statement {
            CheckedStatement::Proof(_) => {}
            CheckedStatement::Let { value, .. }
            | CheckedStatement::DestructuringLet { value, .. }
            | CheckedStatement::Evaluate(value) => collect_expression(value, direct, edges),
            CheckedStatement::SetList {
                targets, values, ..
            } => {
                for target in targets {
                    collect_set_target(target, direct, edges);
                }
                for value in values.expressions() {
                    collect_expression(value, direct, edges);
                }
            }
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
