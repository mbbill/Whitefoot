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

    // [PROV-4, EFF-1] the ambient heap's reachability closure, over the same
    // call graph and in the same pass. That store has no provider value, so
    // no `effect_path` names it and no row carries it [S23]; its reachability
    // is therefore the compiler's own retained record rather than a declared
    // row, and it is exact for the same reason every other closure here is —
    // the compilation unit is closed and there are no function values.
    let mut reaches: Vec<bool> = functions
        .iter()
        .map(|function| function.reaches_ambient_heap)
        .collect();
    loop {
        let mut changed = false;
        for index in 0..functions.len() {
            if reaches[index] {
                continue;
            }
            if edges[index].iter().any(|callee| {
                reaches
                    .get(callee.0 as usize)
                    .copied()
                    .unwrap_or_default()
            }) {
                reaches[index] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for (function, reached) in functions.iter_mut().zip(reaches) {
        function.reaches_ambient_heap = reached;
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
            // [PROV-6] `dispose p;` runs the release walk at the point it is
            // written, so every action that walk performs is this
            // statement's own.
            CheckedStatement::Dispose { value, drops, .. } => {
                collect_expression(value, direct, edges);
                for drop in drops {
                    *direct = direct.union(drop.release.row.target_action);
                }
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
        CheckedSetTarget::RunIndex(target) => {
            collect_expression(&target.offset, direct, edges);
        }
        CheckedSetTarget::SliceIndex(target) => {
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
