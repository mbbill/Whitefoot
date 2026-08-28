//! The non-normative permission ledger: one developer-channel line per
//! analyzed sibling-call pair, one per eligible chain, one per counted loop,
//! and one more for a refused loop a hand-written index split would reach.
//!
//! The ledger is the visible half of the permission judgment. It states, for
//! every pair the judgment looked at, whether overlap is permitted and — when
//! it is not — exactly which numbered condition refused it and at which source
//! text. That makes a sequentialization a reported fact rather than a silent
//! one, and gives a writer a gradient: a condition-2 line names the two places
//! to separate.
//!
//! Pairs alone do not say what will be handed out. A chain of three permitted
//! pairs and three separate permitted pairs read identically as pairs and are
//! completely different work, so every eligible chain gets a `run` line naming
//! its members. What the *backend* then keeps is narrower still — one call
//! definition per site, all members in one block, no addressed binding but the
//! last — and that narrowing happens after this ledger is rendered, so a `run`
//! line states what the judgment permits and not what the emitter actualizes.
//!
//! A `loop` line is the same statement for a counted loop, whose iterations
//! are judged by their own rule rather than as a pair. It names the condition
//! that refused the loop, or, for a permitted one, the operation its
//! accumulator recombines under — the fact that makes the overlap safe. See
//! [`super::loop_permission`] for what is admitted and why no float operation
//! ever is.
//!
//! A `hint` line survives underneath it for exactly one refusal: a loop this
//! version declines only because it carries several accumulators is still one
//! a hand-written recursion can split, and the line says so and names the
//! condition that refused the loop itself. Every other refusal is a reason the
//! rewrite would be refused too, and advice a writer cannot safely take is
//! worse than silence.
//!
//! A `stage` line is the [PAR-3] verdict of one loop whose body performs I/O,
//! and it is followed by one `place` line for every place that judgment
//! classified. Those `place` lines are the teaching channel: a denial without
//! them says only that a loop lost its pipeline, while the table says which
//! place cost it, on which condition, and what the writer may write instead.
//! The `stage` line therefore always names its condition, its offending node,
//! and one admitted writer form, and the `place` lines always print the whole
//! table, granted or denied — a granted loop's table is what a reader checks a
//! later change against. Both are anchored at the loop's cut, because a
//! `loop_stmt` carries no node path of its own and the cut identifies the loop
//! exactly.
//!
//! Nothing here participates in acceptance, in lowering, or in any mandatory
//! [DIAG-3] record. It reads the finished permission table and the source
//! text, renders text, and returns it to the caller to print on the developer
//! channel.

use crate::NodePath;

use super::loop_permission::{LoopDenial, LoopPermission, LoopVerdict};
use super::permission::{
    Access, Denial, ExitKind, PairSide, PermissionMetadata, PermissionVerdict,
};
use super::staged_permission::{StagedDenial, StagedPermission, StagedVerdict};

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
    const LOOP: u8 = 2;
    const HINT: u8 = 3;
    const STAGE: u8 = 4;
    const PLACE: u8 = 5;
    // Every entry outside a disposition table is the only one of its kind at
    // its position, so its ordinal is zero; a table's rows carry the source
    // order of the walk that found them, which the text alone would not
    // preserve.
    const ONLY: u32 = 0;
    let mut entries = Vec::new();
    for permissions in &metadata.functions {
        for pair in &permissions.pairs {
            let (logical_path, line) = source.location(&pair.first.statement)?;
            let verdict = match &pair.verdict {
                PermissionVerdict::PermittedEligible => "permitted",
                PermissionVerdict::Denied(_) => "denied",
            };
            let detail = detail(&pair.verdict, source)?;
            entries.push((
                logical_path.clone(),
                line,
                PAIR,
                ONLY,
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
                ONLY,
                format!(
                    "PAR chain       {logical_path}:{line}  run({members})  {} members through line {last_line}",
                    run.sites.len()
                ),
            ));
        }
        for judged in &permissions.loops {
            let (logical_path, line) = source.location(&judged.statement)?;
            let verdict = if judged.verdict.is_permitted() {
                "permitted"
            } else {
                "denied"
            };
            let detail = loop_detail(judged, source)?;
            entries.push((
                logical_path.clone(),
                line,
                LOOP,
                ONLY,
                format!("PAR loop        {logical_path}:{line}  loop  {verdict:<10}  {detail}"),
            ));
            if !judged.advises_split {
                continue;
            }
            let condition = judged
                .verdict
                .denied_condition()
                .expect("only a refused loop carries split advice");
            entries.push((
                logical_path.clone(),
                line,
                HINT,
                ONLY,
                format!(
                    "PAR hint        {logical_path}:{line}  loop  refused by condition {condition}; a recursive split over its index range would be eligible, combining under {}",
                    judged.combines.join(", ")
                ),
            ));
        }
        for judged in &permissions.staged {
            // The loop head when the checked tree carries one, so two nested
            // loops that share a cut do not print two lines at one anchor; a
            // `loop_stmt` carries none and falls back to the cut.
            let anchor = judged.head.as_ref().unwrap_or(&judged.cut);
            let (logical_path, line) = source.location(anchor)?;
            let verdict = if judged.verdict.is_permitted() {
                "permitted"
            } else {
                "denied"
            };
            let detail = staged_detail(judged, source)?;
            entries.push((
                logical_path.clone(),
                line,
                STAGE,
                ONLY,
                format!(
                    "PAR stage       {logical_path}:{line}  {:<4}  {verdict:<10}  {detail}",
                    judged.form
                ),
            ));
            for (ordinal, place) in judged.dispositions.iter().enumerate() {
                let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
                entries.push((
                    logical_path.clone(),
                    line,
                    PLACE,
                    ordinal,
                    format!(
                        "PAR place       {logical_path}:{line}  {:<12}  {}  {}",
                        place.disposition.spelling(),
                        source.spelling(&place.citation)?,
                        place.reason
                    ),
                ));
            }
        }
    }
    entries.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then(left.1.cmp(&right.1))
            .then(left.2.cmp(&right.2))
            .then(left.3.cmp(&right.3))
            .then(left.4.cmp(&right.4))
    });
    entries.dedup_by(|left, right| left.4 == right.4);
    Ok(entries.into_iter().map(|(.., line)| line).collect())
}

/// The part of the line that states the outcome.
fn detail<Source: LedgerSource>(
    verdict: &PermissionVerdict,
    source: &Source,
) -> Result<String, Source::Error> {
    Ok(match verdict {
        PermissionVerdict::PermittedEligible => "eligible".to_owned(),
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
        Denial::Loan {
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
                source.spelling(left)?,
                source.spelling(right)?
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

/// The part of a loop line that states the outcome.
fn loop_detail<Source: LedgerSource>(
    judged: &LoopPermission,
    source: &Source,
) -> Result<String, Source::Error> {
    // A permitted loop's accumulator is the fact that makes the overlap safe,
    // so the line carries it rather than leaving a reader to infer it from
    // silence.
    let carried = match judged.combines.as_slice() {
        [] => "no accumulator".to_owned(),
        combines => format!("one accumulator under {}", combines.join(", ")),
    };
    Ok(match &judged.verdict {
        LoopVerdict::PermittedEligible => format!("eligible; {carried}"),
        LoopVerdict::Denied(denial) => loop_denied_detail(denial, source)?,
    })
}

fn loop_denied_detail<Source: LedgerSource>(
    denial: &LoopDenial,
    source: &Source,
) -> Result<String, Source::Error> {
    // The number comes from the judgment itself, so the reported condition
    // cannot drift from the condition that actually refused the loop.
    let condition = denial.condition();
    let reason = match denial {
        LoopDenial::NotAReduction { statement } => format!(
            "the loop writes storage outliving the iteration that no exactly associative operation reduces, at {}",
            source.spelling(statement)?
        ),
        LoopDenial::ManyAccumulators { accumulators } => {
            format!("the body carries {accumulators} accumulators, and this rule recombines one")
        }
        LoopDenial::AccumulatorRead { statement, reads } => format!(
            "the accumulator is read {reads} times in the body and a reduction reads it once, at {}",
            source.spelling(statement)?
        ),
        LoopDenial::Loan { argument } => format!(
            "an iteration holds an exclusive loan on storage the iteration does not introduce, at {}",
            source.spelling(argument)?
        ),
        LoopDenial::SharedWrite { argument } => format!(
            "the body writes storage that is neither introduced by the iteration nor the accumulator, at {}",
            source.spelling(argument)?
        ),
        LoopDenial::UnresolvedWrite { argument } => format!(
            "unresolved write footprint through {}",
            source.spelling(argument)?
        ),
        // The disclosure half: a form this judgment does not account for is
        // reported here rather than passed over silently, so the writer sees
        // the statement that costs the overlap.
        LoopDenial::BodyForm { form } => format!("the body contains {form}"),
        LoopDenial::Exit { edge } => format!("{edge} leaves the loop"),
    };
    Ok(format!("condition {condition}: {reason}"))
}

/// The part of a `stage` line that states the outcome.
///
/// A permitted loop states the cut it was granted over and how many places it
/// classified, so the `place` lines underneath are read as a complete table
/// rather than a selection. A denied loop states the condition, the node, and
/// one admitted writer form, which is the whole of what a writer can act on.
fn staged_detail<Source: LedgerSource>(
    judged: &StagedPermission,
    source: &Source,
) -> Result<String, Source::Error> {
    Ok(match &judged.verdict {
        StagedVerdict::Permitted => format!(
            "staged at {}; {} places classified",
            source.spelling(&judged.cut)?,
            judged.dispositions.len()
        ),
        StagedVerdict::Denied(denial) => staged_denied_detail(denial, source)?,
    })
}

fn staged_denied_detail<Source: LedgerSource>(
    denial: &StagedDenial,
    source: &Source,
) -> Result<String, Source::Error> {
    // The number and the writer form both come from the judgment itself, so
    // neither can drift from the condition that actually refused the loop. The
    // cited node comes last, because a statement's canonical spelling carries
    // its own terminator and anything appended after one reads as a typo.
    let condition = denial.condition();
    // The edge already names which loop a `break_stmt` leaves, which is the
    // only identity a break has in the checked tree.
    let exit = |edge: &str| format!("{edge} leaves the loop from the remainder");
    // The second node a denial names, with the phrase that says how it stands
    // to the first. Two denials name a second node for two different reasons
    // and one phrase cannot carry both: an [OWN-7] pair is an overlap claim,
    // and a write of the borrowed place itself is not.
    const OVERLAPS: &str = "which overlaps";
    const WRITES: &str = "and the body writes it at";
    let (reason, node, paired) = match denial {
        StagedDenial::NoCut { reason, statement } => {
            ((*reason).to_owned(), statement.as_ref(), None)
        }
        StagedDenial::ExitInRemainder { edge, statement } => {
            (exit(edge), statement.as_ref(), None)
        }
        StagedDenial::RetainedBorrow {
            argument,
            written_at,
            overlapping,
            ..
        } => (
            "a may-suspend call retains a borrow past its own submission on storage the body writes and the iteration does not introduce"
                .to_owned(),
            Some(argument),
            written_at
                .as_ref()
                .map(|node| (if *overlapping { OVERLAPS } else { WRITES }, node)),
        ),
        StagedDenial::RemainderExclusiveLoan { argument, .. } => (
            "a call of the remainder holds an exclusive loan on storage the iteration does not introduce"
                .to_owned(),
            Some(argument),
            None,
        ),
        StagedDenial::NoDisposition {
            argument,
            overlapping,
        } => (
            "the body reaches storage rooted outside the loop that no disposition of this rule covers"
                .to_owned(),
            Some(argument),
            overlapping.as_ref().map(|node| (OVERLAPS, node)),
        ),
        StagedDenial::NotReplicable { statement } => (
            "per-iteration storage whose element type is not a resolved copy type".to_owned(),
            Some(statement),
            None,
        ),
        StagedDenial::BodyForm { form, .. } => {
            return Ok(format!(
                "condition {condition}: the body contains {form}; instead, {}",
                denial.writer_form()
            ));
        }
        StagedDenial::Unresolved { argument } => (
            "a footprint element this judgment does not resolve".to_owned(),
            Some(argument),
            None,
        ),
    };
    // A `break_stmt`, a `loop_stmt`, and a `region_stmt` carry no node path of
    // their own, so a denial citing one names the condition without a source
    // node rather than naming a node the writer did not write there.
    let cited = match node {
        Some(node) => format!(", at {}", source.spelling(node)?),
        None => String::new(),
    };
    // [OWN-7] makes a place and its prefix one storage, so a denial the overlap
    // decided names both halves: one statement alone never shows the reader why
    // the loop refused, because the statement that refused it names a different
    // path. A denial whose write is on the borrowed place itself names that
    // write under its own phrase instead, so no line ever reports one place as
    // overlapping itself.
    let paired = match paired {
        Some((phrase, node)) => format!(", {phrase} {}", source.spelling(node)?),
        None => String::new(),
    };
    Ok(format!(
        "condition {condition}: {reason}; instead, {}{cited}{paired}",
        denial.writer_form()
    ))
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
