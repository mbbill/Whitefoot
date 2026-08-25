//! Deterministic non-normative I/O audit records.
//!
//! The builder below consumes checked calls, checked releases, callable rows,
//! and the finished permission table once. Rendering later adds only the set
//! of source-call keys the selected lowering actually used; it never repeats
//! a semantic or permission judgment.

use std::collections::{HashMap, HashSet};

use crate::{
    DeclarationId, NodePath, SYSTEM_OPERATIONS, SystemReleaseAction, SystemResourceType,
    TargetAction,
};

use super::model::{
    CheckedDrop, CheckedExpression, CheckedFunction, CheckedNominal, CheckedNominalKind,
    CheckedSetTarget, CheckedStatement, CheckedType, NominalId, expression_children,
};
use super::permission::{PermissionMetadata, PermissionSignature, PermissionVerdict};

/// Stable checked source-call identity shared with lowering.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct IoSiteKey {
    pub(crate) function: u32,
    pub(crate) call: NodePath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IoCallRecord {
    key: IoSiteKey,
    logical_path: String,
    line: u64,
    function: String,
    semantic_id: String,
    worlds: Vec<String>,
    origins: String,
    reads: Vec<String>,
    writes: Vec<String>,
    target_action: TargetAction,
    permission: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IoReleaseRecord {
    logical_path: String,
    line: u64,
    function: String,
    semantic_id: String,
    worlds: Vec<String>,
    origins: String,
    reads: Vec<String>,
    writes: Vec<String>,
    target_action: TargetAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IoPermissionRecord {
    logical_path: String,
    line: u64,
    function: u32,
    function_name: String,
    first: IoSiteKey,
    second: IoSiteKey,
    first_id: String,
    second_id: String,
    result: String,
}

/// Checked records behind `whitefootc --io-ledger`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct IoLedger {
    calls: Vec<IoCallRecord>,
    releases: Vec<IoReleaseRecord>,
    permissions: Vec<IoPermissionRecord>,
}

impl IoLedger {
    /// Renders the checked records with the exact source calls selected by the
    /// lowering. Every unselected site is stated as sequential.
    pub(crate) fn render(&self, actualized: &[(u32, NodePath)]) -> Vec<String> {
        const CALL: u8 = 0;
        const RELEASE: u8 = 1;
        const PERMISSION: u8 = 2;
        let actualized = actualized
            .iter()
            .cloned()
            .map(|(function, call)| IoSiteKey { function, call })
            .collect::<HashSet<_>>();
        let mut lines = Vec::new();
        for call in &self.calls {
            let lowering = if actualized.contains(&call.key) {
                "actualized"
            } else {
                "sequential"
            };
            lines.push((
                call.logical_path.clone(),
                call.line,
                CALL,
                format!(
                    "IO call         {}:{}  function={} id={} worlds={} origins={} footprint={} target={} permission={} lowering={lowering}",
                    call.logical_path,
                    call.line,
                    call.function,
                    call.semantic_id,
                    list(&call.worlds),
                    call.origins,
                    footprint(&call.reads, &call.writes),
                    target_action(call.target_action),
                    call.permission,
                ),
            ));
        }
        for release in &self.releases {
            lines.push((
                release.logical_path.clone(),
                release.line,
                RELEASE,
                format!(
                    "IO release      {}:{}  function={} id={} worlds={} origins={} footprint={} target={} permission=n/a lowering=sequential",
                    release.logical_path,
                    release.line,
                    release.function,
                    release.semantic_id,
                    list(&release.worlds),
                    release.origins,
                    footprint(&release.reads, &release.writes),
                    target_action(release.target_action),
                ),
            ));
        }
        for permission in &self.permissions {
            let lowering = if actualized.contains(&permission.first)
                && actualized.contains(&permission.second)
            {
                "actualized"
            } else {
                "sequential"
            };
            lines.push((
                permission.logical_path.clone(),
                permission.line,
                PERMISSION,
                format!(
                    "IO permission   {}:{}  function={} pair=({}, {}) result={} lowering={lowering}",
                    permission.logical_path,
                    permission.line,
                    permission.function_name,
                    permission.first_id,
                    permission.second_id,
                    permission.result,
                ),
            ));
        }
        lines.sort();
        lines.dedup_by(|left, right| left.3 == right.3);
        lines.into_iter().map(|(_, _, _, line)| line).collect()
    }
}

/// Source identities needed while checked records are still beside the tree.
pub(crate) trait IoLedgerSource {
    type Error;

    fn location(&self, path: &NodePath) -> Result<(String, u64), Self::Error>;
    fn declaration_location(
        &self,
        declaration: DeclarationId,
    ) -> Result<(String, u64), Self::Error>;
    fn region(&self, declaration: DeclarationId) -> Result<String, Self::Error>;
}

/// Builds all I/O audit records from the finished checked-program tables.
pub(crate) fn build_io_ledger<Source: IoLedgerSource>(
    functions: &[CheckedFunction],
    nominals: &[CheckedNominal],
    signatures: &[PermissionSignature],
    permission: &PermissionMetadata,
    source: &Source,
) -> Result<IoLedger, Source::Error> {
    let mut ledger = IoLedger::default();
    for function in functions {
        let fallback = source.declaration_location(function.declaration)?;
        collect_statements(
            function,
            &function.body,
            &fallback,
            functions,
            nominals,
            signatures,
            permission,
            source,
            &mut ledger,
        )?;
    }

    let call_ids = ledger
        .calls
        .iter()
        .map(|call| (call.key.clone(), call.semantic_id.clone()))
        .collect::<HashMap<_, _>>();
    for (function_index, permissions) in permission.functions.iter().enumerate() {
        for pair in &permissions.pairs {
            let first = IoSiteKey {
                function: function_index as u32,
                call: pair.first.call.clone(),
            };
            let second = IoSiteKey {
                function: function_index as u32,
                call: pair.second.call.clone(),
            };
            if !call_ids.contains_key(&first) && !call_ids.contains_key(&second) {
                continue;
            }
            let (logical_path, line) = source.location(&pair.first.statement)?;
            ledger.permissions.push(IoPermissionRecord {
                logical_path,
                line,
                function: function_index as u32,
                function_name: permissions.function.clone(),
                first_id: call_ids
                    .get(&first)
                    .cloned()
                    .unwrap_or_else(|| semantic_id(&pair.first.callee_name)),
                second_id: call_ids
                    .get(&second)
                    .cloned()
                    .unwrap_or_else(|| semantic_id(&pair.second.callee_name)),
                first,
                second,
                result: permission_result(&pair.verdict),
            });
        }
    }
    Ok(ledger)
}

#[allow(clippy::too_many_arguments)]
fn collect_statements<Source: IoLedgerSource>(
    function: &CheckedFunction,
    statements: &[CheckedStatement],
    fallback: &(String, u64),
    functions: &[CheckedFunction],
    nominals: &[CheckedNominal],
    signatures: &[PermissionSignature],
    permission: &PermissionMetadata,
    source: &Source,
    ledger: &mut IoLedger,
) -> Result<(), Source::Error> {
    for statement in statements {
        let location = statement_path(statement)
            .map(|path| source.location(path))
            .transpose()?
            .unwrap_or_else(|| fallback.clone());
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::Claim {
                condition: value, ..
            } => collect_expression(
                function, value, functions, signatures, permission, source, ledger,
            )?,
            CheckedStatement::PropagateLet {
                scrutinee,
                error_drops,
                ..
            } => {
                collect_expression(
                    function, scrutinee, functions, signatures, permission, source, ledger,
                )?;
                collect_drops(function, error_drops, &location, nominals, source, ledger)?;
            }
            CheckedStatement::Set { target, value, .. }
            | CheckedStatement::Replace { target, value, .. } => {
                collect_set_target(
                    function, target, functions, signatures, permission, source, ledger,
                )?;
                collect_expression(
                    function, value, functions, signatures, permission, source, ledger,
                )?;
            }
            CheckedStatement::DropExpression { value, release } => {
                collect_expression(
                    function, value, functions, signatures, permission, source, ledger,
                )?;
                collect_release_type(
                    function,
                    value.ty(),
                    *release,
                    &location,
                    nominals,
                    source,
                    ledger,
                    true,
                )?;
            }
            CheckedStatement::Return { value, drops, .. }
            | CheckedStatement::Give { value, drops, .. } => {
                collect_expression(
                    function, value, functions, signatures, permission, source, ledger,
                )?;
                collect_drops(function, drops, &location, nominals, source, ledger)?;
            }
            CheckedStatement::Match {
                scrutinee, arms, ..
            }
            | CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                collect_expression(
                    function, scrutinee, functions, signatures, permission, source, ledger,
                )?;
                for arm in arms {
                    collect_statements(
                        function, &arm.body, &location, functions, nominals, signatures,
                        permission, source, ledger,
                    )?;
                    collect_drops(
                        function,
                        &arm.fallthrough_drops,
                        &location,
                        nominals,
                        source,
                        ledger,
                    )?;
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
                collect_statements(
                    function, body, &location, functions, nominals, signatures, permission, source,
                    ledger,
                )?;
                collect_drops(
                    function,
                    backedge_drops,
                    &location,
                    nominals,
                    source,
                    ledger,
                )?;
            }
            CheckedStatement::Break { drops, .. } => {
                collect_drops(function, drops, &location, nominals, source, ledger)?;
            }
            CheckedStatement::Region {
                body,
                fallthrough_drops,
                ..
            } => {
                collect_statements(
                    function, body, &location, functions, nominals, signatures, permission, source,
                    ledger,
                )?;
                collect_drops(
                    function,
                    fallthrough_drops,
                    &location,
                    nominals,
                    source,
                    ledger,
                )?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn collect_expression<Source: IoLedgerSource>(
    function: &CheckedFunction,
    expression: &CheckedExpression,
    functions: &[CheckedFunction],
    signatures: &[PermissionSignature],
    permission: &PermissionMetadata,
    source: &Source,
    ledger: &mut IoLedger,
) -> Result<(), Source::Error> {
    match expression {
        CheckedExpression::SystemCall {
            operation,
            target_action,
            call,
            regions,
            ..
        } => {
            if let Some(operation_row) = SYSTEM_OPERATIONS.get(usize::from(*operation)) {
                let reads = operation_row
                    .reads
                    .iter()
                    .filter(|index| operation_row.world_regions.contains(index))
                    .filter_map(|index| regions.get(usize::from(*index)).copied())
                    .map(|region| source.region(region))
                    .collect::<Result<Vec<_>, _>>()?;
                let writes = operation_row
                    .writes
                    .iter()
                    .filter(|index| operation_row.world_regions.contains(index))
                    .filter_map(|index| regions.get(usize::from(*index)).copied())
                    .map(|region| source.region(region))
                    .collect::<Result<Vec<_>, _>>()?;
                if !reads.is_empty() || !writes.is_empty() {
                    let worlds = operation_row
                        .world_regions
                        .iter()
                        .filter_map(|index| regions.get(usize::from(*index)).copied())
                        .map(|region| source.region(region))
                        .collect::<Result<Vec<_>, _>>()?;
                    let (logical_path, line) = source.location(call)?;
                    let key = IoSiteKey {
                        function: function.id.0,
                        call: call.clone(),
                    };
                    ledger.calls.push(IoCallRecord {
                        permission: call_permission(permission, &key),
                        key,
                        logical_path,
                        line,
                        function: function.symbol.clone(),
                        semantic_id: format!("sys.{}", operation_row.spelling),
                        origins: system_origins(operation_row, regions, source)?,
                        worlds,
                        reads,
                        writes,
                        target_action: *target_action,
                    });
                }
            }
        }
        CheckedExpression::UserCall {
            function: callee_id,
            call,
            goal_regions,
            ..
        } => {
            if let (Some(callee), Some(signature)) = (
                functions.get(callee_id.0 as usize),
                signatures.get(callee_id.0 as usize),
            ) {
                let reads = projected_worlds(&signature.reads, signature, goal_regions, source)?;
                let writes = projected_worlds(&signature.writes, signature, goal_regions, source)?;
                if !reads.is_empty() || !writes.is_empty() {
                    let worlds = projected_worlds(
                        &signature.world_regions,
                        signature,
                        goal_regions,
                        source,
                    )?;
                    let (logical_path, line) = source.location(call)?;
                    let key = IoSiteKey {
                        function: function.id.0,
                        call: call.clone(),
                    };
                    ledger.calls.push(IoCallRecord {
                        permission: call_permission(permission, &key),
                        key,
                        logical_path,
                        line,
                        function: function.symbol.clone(),
                        semantic_id: format!("fn.{}", callee.symbol),
                        worlds,
                        origins: "boundary-projection".to_owned(),
                        reads,
                        writes,
                        target_action: callee.target_action,
                    });
                }
            }
        }
        _ => {}
    }
    for child in expression_children(expression) {
        collect_expression(
            function, child, functions, signatures, permission, source, ledger,
        )?;
    }
    Ok(())
}

fn projected_worlds<Source: IoLedgerSource>(
    selected: &[DeclarationId],
    signature: &PermissionSignature,
    actual: &[DeclarationId],
    source: &Source,
) -> Result<Vec<String>, Source::Error> {
    selected
        .iter()
        .filter(|formal| signature.world_regions.contains(formal))
        .filter_map(|formal| {
            signature
                .region_parameters
                .iter()
                .position(|candidate| candidate == formal)
                .and_then(|index| actual.get(index))
                .copied()
        })
        .map(|region| source.region(region))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn collect_set_target<Source: IoLedgerSource>(
    function: &CheckedFunction,
    target: &CheckedSetTarget,
    functions: &[CheckedFunction],
    signatures: &[PermissionSignature],
    permission: &PermissionMetadata,
    source: &Source,
    ledger: &mut IoLedger,
) -> Result<(), Source::Error> {
    match target {
        CheckedSetTarget::Place(_) => Ok(()),
        CheckedSetTarget::ArrayIndex(target) => collect_expression(
            function,
            &target.offset,
            functions,
            signatures,
            permission,
            source,
            ledger,
        ),
        CheckedSetTarget::BufferIndex(target) => collect_expression(
            function,
            &target.offset,
            functions,
            signatures,
            permission,
            source,
            ledger,
        ),
    }
}

fn collect_drops<Source: IoLedgerSource>(
    function: &CheckedFunction,
    drops: &[CheckedDrop],
    location: &(String, u64),
    nominals: &[CheckedNominal],
    source: &Source,
    ledger: &mut IoLedger,
) -> Result<(), Source::Error> {
    for drop in drops {
        collect_release_type(
            function,
            drop.ty,
            drop.release,
            location,
            nominals,
            source,
            ledger,
            drop.release.action.is_some(),
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn collect_release_type<Source: IoLedgerSource>(
    function: &CheckedFunction,
    ty: CheckedType,
    release: crate::SystemRelease,
    location: &(String, u64),
    nominals: &[CheckedNominal],
    source: &Source,
    ledger: &mut IoLedger,
    include_aggregate: bool,
) -> Result<(), Source::Error> {
    if release.row.target_action == TargetAction::INLINE
        && !release.row.writes_command_order
        && !release.row.writes_handle_lifetime
    {
        return Ok(());
    }
    let mut resources = Vec::new();
    release_resources(
        ty,
        nominals,
        &mut HashSet::new(),
        &mut resources,
        include_aggregate,
    );
    for (resource, worlds, action) in resources {
        let contract = crate::system_resource_contract(resource)
            .expect("a checked resource release has a catalog contract");
        let worlds = worlds
            .into_iter()
            .map(|region| source.region(region))
            .collect::<Result<Vec<_>, _>>()?;
        let mut writes = Vec::new();
        if contract.row.writes_command_order
            && let Some(region) = worlds.first()
        {
            writes.push(region.clone());
        }
        if contract.row.writes_handle_lifetime
            && let Some(region) = worlds.get(1)
        {
            writes.push(region.clone());
        }
        ledger.releases.push(IoReleaseRecord {
            logical_path: location.0.clone(),
            line: location.1,
            function: function.symbol.clone(),
            semantic_id: format!(
                "release.{}.{}",
                resource_spelling(contract.resource),
                release_action_spelling(action)
            ),
            origins: "preserve-capability; mutate-handle-lifetime".to_owned(),
            worlds,
            reads: Vec::new(),
            writes,
            target_action: contract.row.target_action,
        });
    }
    Ok(())
}

fn release_resources(
    ty: CheckedType,
    nominals: &[CheckedNominal],
    visited: &mut HashSet<NominalId>,
    output: &mut Vec<(u8, Vec<DeclarationId>, SystemReleaseAction)>,
    include_aggregate: bool,
) {
    let CheckedType::Nominal(id) = ty else {
        if let CheckedType::Buffer { element } = ty {
            release_resources(element.ty(), nominals, visited, output, true);
        }
        return;
    };
    if !visited.insert(id) {
        return;
    }
    let Some(nominal) = nominals.get(id.0 as usize) else {
        return;
    };
    match &nominal.kind {
        CheckedNominalKind::SystemResource {
            nominal,
            world_regions,
        } => {
            if let Some(contract) = crate::system_resource_contract(*nominal)
                && contract.row.target_action != TargetAction::INLINE
            {
                output.push((*nominal, world_regions.clone(), contract.action));
            }
        }
        CheckedNominalKind::Enum { variants } if include_aggregate => {
            for field in variants.iter().flat_map(|variant| &variant.fields) {
                release_resources(field.ty, nominals, visited, output, true);
            }
        }
        CheckedNominalKind::Box { referent } if include_aggregate => {
            release_resources(*referent, nominals, visited, output, true);
        }
        CheckedNominalKind::Struct { fields } if include_aggregate => {
            for field in fields {
                release_resources(field.ty, nominals, visited, output, true);
            }
        }
        CheckedNominalKind::Arena { content, .. } if include_aggregate => {
            release_resources(*content, nominals, visited, output, true);
        }
        CheckedNominalKind::Struct { .. }
        | CheckedNominalKind::Enum { .. }
        | CheckedNominalKind::Box { .. }
        | CheckedNominalKind::Arena { .. }
        | CheckedNominalKind::ArenaStorage => {}
    }
}

fn statement_path(statement: &CheckedStatement) -> Option<&NodePath> {
    match statement {
        CheckedStatement::Let { node_path, .. }
        | CheckedStatement::PropagateLet { node_path, .. }
        | CheckedStatement::Set { node_path, .. }
        | CheckedStatement::Replace { node_path, .. }
        | CheckedStatement::Return { node_path, .. }
        | CheckedStatement::ValueMatchLet { node_path, .. }
        | CheckedStatement::Give { node_path, .. }
        | CheckedStatement::CountedRange { node_path, .. } => Some(node_path),
        CheckedStatement::Evaluate(value) | CheckedStatement::DropExpression { value, .. } => {
            expression_path(value)
        }
        CheckedStatement::Claim { site, .. } => Some(&site.node_path),
        CheckedStatement::Match { scrutinee, .. } => expression_path(scrutinee),
        CheckedStatement::Loop { .. }
        | CheckedStatement::Break { .. }
        | CheckedStatement::Region { .. } => None,
    }
}

fn expression_path(expression: &CheckedExpression) -> Option<&NodePath> {
    match expression {
        CheckedExpression::UserCall { call, .. } | CheckedExpression::SystemCall { call, .. } => {
            Some(call)
        }
        _ => None,
    }
}

fn call_permission(permission: &PermissionMetadata, key: &IoSiteKey) -> String {
    let Some(function) = permission.functions.get(key.function as usize) else {
        return "conservative-denial(unresolved-function)".to_owned();
    };
    let mut results = function
        .pairs
        .iter()
        .filter(|pair| pair.first.call == key.call || pair.second.call == key.call)
        .map(|pair| permission_result(&pair.verdict))
        .collect::<Vec<_>>();
    results.sort();
    results.dedup();
    match results.as_slice() {
        [] => "not-analyzed".to_owned(),
        [one] => one.clone(),
        many => format!("mixed({})", many.join(",")),
    }
}

fn permission_result(verdict: &PermissionVerdict) -> String {
    match verdict {
        PermissionVerdict::PermittedEligible => "permitted".to_owned(),
        PermissionVerdict::Denied(denial) => format!("denied(condition={})", denial.condition()),
    }
}

fn semantic_id(callee: &str) -> String {
    if SYSTEM_OPERATIONS.iter().any(|row| row.spelling == callee) {
        format!("sys.{callee}")
    } else {
        format!("fn.{callee}")
    }
}

fn system_origins<Source: IoLedgerSource>(
    operation: &crate::SystemOperation,
    actual: &[DeclarationId],
    source: &Source,
) -> Result<String, Source::Error> {
    let mut rendered = Vec::with_capacity(operation.origin_relations.len());
    for relation in operation.origin_relations {
        let formal = operation
            .regions
            .get(usize::from(relation.region))
            .copied()
            .unwrap_or("'unresolved");
        let instantiated = match actual.get(usize::from(relation.region)).copied() {
            Some(region) => source.region(region)?,
            None => "unresolved".to_owned(),
        };
        rendered.push(format!(
            "{}({formal}={instantiated})",
            relation.kind.spelling()
        ));
    }
    Ok(if rendered.is_empty() {
        "none".to_owned()
    } else {
        rendered.join(";")
    })
}

fn target_action(action: TargetAction) -> String {
    format!(
        "({},{},{})",
        action.dispatch.spelling(),
        action.host_wait.spelling(),
        action.loan_end.spelling()
    )
}

fn list(values: &[String]) -> String {
    format!("[{}]", values.join(","))
}

fn footprint(reads: &[String], writes: &[String]) -> String {
    format!("R{} W{}", list(reads), list(writes))
}

fn resource_spelling(resource: SystemResourceType) -> &'static str {
    match resource {
        SystemResourceType::Args => "args",
        SystemResourceType::HostString => "host-string",
        SystemResourceType::RelativePath => "relative-path",
        SystemResourceType::DirectoryRead => "directory-read",
        SystemResourceType::ReadFile => "read-file",
        SystemResourceType::Output => "output",
        SystemResourceType::ExitStatus => "exit-status",
        SystemResourceType::DirectoryList => "directory-list",
    }
}

fn release_action_spelling(action: SystemReleaseAction) -> &'static str {
    match action {
        SystemReleaseAction::LogicalConsume => "logical-consume",
        SystemReleaseAction::NativeCloseAttempt => "native-close-attempt",
        SystemReleaseAction::SourceDetach => "source-detach",
    }
}
