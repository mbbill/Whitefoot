//! The non-normative permission ledger: one developer-channel line per
//! analyzed sibling-call pair.
//!
//! The ledger is the visible half of the permission judgment. It states, for
//! every pair the judgment looked at, whether overlap is permitted, whether a
//! permitted overlap is actualizable, and — when it is not permitted — exactly
//! which numbered condition refused it and at which source text. That makes a
//! sequentialization a reported fact rather than a silent one, and gives a
//! writer a gradient: a condition-2 line names the two places to separate, a
//! not-actualizable line names the claim that keeps the overlap out of reach.
//!
//! Nothing here participates in acceptance, in lowering, or in any mandatory
//! [DIAG-3] record. It reads the finished permission table and the source
//! text, renders text, and returns it to the caller to print on the developer
//! channel.

use crate::NodePath;

use super::permission::{
    Access, Denial, ExitKind, PairSide, PermissionMetadata, PermissionVerdict,
};

/// The source facts one ledger line needs, supplied by the checker because
/// only the checker still holds the syntax tree and the source bundle.
pub(crate) trait LedgerSource {
    /// Why a node path could not be resolved.
    type Error;

    /// The logical source name and one-based line of one node.
    fn location(&self, path: &NodePath) -> Result<(String, u64), Self::Error>;

    /// The exact canonical source spelling of one node.
    fn spelling(&self, path: &NodePath) -> Result<String, Self::Error>;
}

/// Renders the whole permission table as ledger lines in source order.
///
/// The table is dense by `FunctionId`, so one source function that is
/// monomorphized more than once contributes its pairs more than once. Lines
/// that come out byte-identical are therefore collapsed: the ledger reports
/// source sites, and two instances of one generic that agree on the verdict
/// are one reported site. Two instances that disagree keep both lines.
pub(crate) fn render_ledger<Source: LedgerSource>(
    metadata: &PermissionMetadata,
    source: &Source,
) -> Result<Vec<String>, Source::Error> {
    let mut entries = Vec::new();
    for permissions in &metadata.functions {
        for pair in &permissions.pairs {
            let (logical_path, line) = source.location(&pair.first.statement)?;
            let verdict = match &pair.verdict {
                PermissionVerdict::PermittedEligible
                | PermissionVerdict::PermittedNotActualizable { .. } => "permitted",
                PermissionVerdict::Denied(_) => "denied",
            };
            let detail = detail(&pair.verdict, source)?;
            entries.push((
                logical_path.clone(),
                line,
                format!(
                    "PAR {verdict:<10}  {logical_path}:{line}  pair({}, {})  {detail}",
                    pair.first.callee_name, pair.second.callee_name
                ),
            ));
        }
    }
    entries.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then(left.1.cmp(&right.1))
            .then(left.2.cmp(&right.2))
    });
    entries.dedup_by(|left, right| left.2 == right.2);
    Ok(entries.into_iter().map(|(_, _, line)| line).collect())
}

/// The part of the line that states the outcome.
fn detail<Source: LedgerSource>(
    verdict: &PermissionVerdict,
    source: &Source,
) -> Result<String, Source::Error> {
    Ok(match verdict {
        PermissionVerdict::PermittedEligible => "eligible".to_owned(),
        PermissionVerdict::PermittedNotActualizable {
            claim_sites,
            witness,
        } => format!(
            "not-actualizable: {claim_sites} claim {} via {}",
            if *claim_sites == 1 { "site" } else { "sites" },
            witness.function
        ),
        PermissionVerdict::Denied(denial) => denied_detail(denial, source)?,
    })
}

fn denied_detail<Source: LedgerSource>(
    denial: &Denial,
    source: &Source,
) -> Result<String, Source::Error> {
    // The number comes from the judgment itself, so the reported condition
    // cannot drift from the condition that actually refused the pair.
    let condition = denial.condition();
    let reason = match denial {
        // The judged pair is two `let` statements, so the one binding s1
        // defines is its result; naming it that way needs no binding table.
        Denial::Dataflow { .. } => "an argument of s2 uses the result of s1".to_owned(),
        Denial::Footprint { left, right } => format!(
            "writes overlap at {} vs {}",
            access(left, source)?,
            access(right, source)?
        ),
        Denial::UnresolvedFootprint { side, argument } => format!(
            "unresolved footprint through {} of {}",
            source.spelling(argument)?,
            statement_name(*side)
        ),
        Denial::Row {
            side,
            external,
            blocks,
        } => {
            let mut categories = Vec::new();
            if *external {
                categories.push("external");
            }
            if *blocks {
                categories.push("blocks");
            }
            format!(
                "the row of {} carries {}",
                statement_name(*side),
                categories.join(", ")
            )
        }
        Denial::SkippingExit {
            kind: ExitKind::PropagateError,
        } => "Err edge of s1 skips s2".to_owned(),
    };
    Ok(format!("condition {condition}: {reason}"))
}

/// One footprint element as the writer wrote it.
fn access<Source: LedgerSource>(access: &Access, source: &Source) -> Result<String, Source::Error> {
    match access {
        Access::Place { argument, .. } => source.spelling(argument),
        // An arena row reaches its region through no actual, so the citation
        // is the call that allocates into it.
        Access::Arena { call, .. } => Ok(format!("the arena of {}", source.spelling(call)?)),
    }
}

const fn statement_name(side: PairSide) -> &'static str {
    match side {
        PairSide::First => "s1",
        PairSide::Second => "s2",
    }
}
