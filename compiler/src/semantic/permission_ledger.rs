//! The non-normative permission ledger: one developer-channel line per
//! analyzed sibling-call pair, and one per eligible chain.
//!
//! The ledger is the visible half of the permission judgment. It states, for
//! every pair the judgment looked at, whether overlap is permitted, whether a
//! permitted overlap is actualizable, and — when it is not permitted — exactly
//! which numbered condition refused it and at which source text. That makes a
//! sequentialization a reported fact rather than a silent one, and gives a
//! writer a gradient: a condition-2 line names the two places to separate, a
//! not-actualizable line names the claim that keeps the overlap out of reach.
//!
//! Pairs alone do not say what will be handed out. A chain of three permitted
//! pairs and three separate permitted pairs read identically as pairs and are
//! completely different work, so every eligible chain gets a `run` line naming
//! its members. What the *backend* then keeps is narrower still — one call
//! definition per site, all members in one block, no addressed binding but the
//! last — and that narrowing happens after this ledger is rendered, so a `run`
//! line states what the judgment permits and not what the emitter actualizes.
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
    // A chain starts at its first member's statement, so it shares a location
    // with the pair that opens it. This ordinal sorts the pairs of one
    // location ahead of the chain they compose into, which is the order a
    // writer reads them in.
    const PAIR: u8 = 0;
    const CHAIN: u8 = 1;
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
                PAIR,
                format!(
                    "PAR {verdict:<10}  {logical_path}:{line}  pair({}, {})  {detail}",
                    pair.first.callee_name, pair.second.callee_name
                ),
            ));
        }
        for run in &permissions.runs {
            let (logical_path, line) = source.location(&run.sites[0].statement)?;
            let last = run.sites.last().expect("a chain has at least two members");
            let (_, last_line) = source.location(&last.statement)?;
            let members = run
                .sites
                .iter()
                .map(|site| site.callee_name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            entries.push((
                logical_path.clone(),
                line,
                CHAIN,
                format!(
                    "PAR chain       {logical_path}:{line}  run({members})  {} members through line {last_line}",
                    run.sites.len()
                ),
            ));
        }
    }
    entries.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then(left.1.cmp(&right.1))
            .then(left.2.cmp(&right.2))
            .then(left.3.cmp(&right.3))
    });
    entries.dedup_by(|left, right| left.3 == right.3);
    Ok(entries.into_iter().map(|(.., line)| line).collect())
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
        // Every window statement that defines a binding defines exactly one,
        // so naming the two ends locates the link without a binding table.
        Denial::Dataflow {
            definer, reader, ..
        } => format!(
            "the operands of {} read what {} defines",
            statement_name(*reader),
            statement_name(*definer)
        ),
        // The kind comes from the conflict loop that found it, so a read/write
        // conflict is never reported as two writes, and the sides come from
        // the same place, so a conflict with an interposed statement is never
        // reported against s1 or s2.
        Denial::Footprint {
            kind,
            left,
            right,
            sides,
        } => {
            let (left_half, right_half) = kind.halves();
            format!(
                "the {left_half} of {} overlaps the {right_half} of {} at {} vs {}",
                statement_name(sides.0),
                statement_name(sides.1),
                access(left, source)?,
                access(right, source)?
            )
        }
        Denial::UnresolvedFootprint { side, argument } => format!(
            "unresolved footprint through {} of {}",
            source.spelling(argument)?,
            statement_name(*side)
        ),
        // F3's disclosure half: a form this judgment does not account for is
        // reported here rather than ending the enumeration silently, so the
        // writer sees the statement that costs the overlap.
        Denial::InterposedForm { side, form } => {
            format!("{} between s1 and s2 is {form}", statement_name(*side))
        }
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
        Denial::SkippingExit { side, kind } => {
            let edge = match kind {
                ExitKind::PropagateError => "Err edge",
                ExitKind::ClaimTrap => "trap edge",
                ExitKind::BlockExit => "exit edge",
            };
            format!("the {edge} of {} skips s2", statement_name(*side))
        }
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

/// How the ledger names one window statement. The interposed ones are counted
/// from one in source order, so "interposed statement 2" is the second
/// statement the writer put between the two calls.
fn statement_name(side: PairSide) -> String {
    match side {
        PairSide::First => "s1".to_owned(),
        PairSide::Second => "s2".to_owned(),
        PairSide::Between(index) => format!("interposed statement {}", index + 1),
    }
}
