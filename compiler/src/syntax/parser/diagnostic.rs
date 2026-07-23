use crate::lexer::TokenKind;
use crate::syntax::grammar::{
    DecisionContextV0_14, DecisionKindV0_14, DecisionV0_14, GrammarNodeIdV0_14,
    GrammarNodeKindV0_14, LookaheadPredicateV0_14, NamePredicateV0_14, ProductionV0_14,
    SelectRowV0_14, grammar_node_v0_14,
};
use crate::syntax::terminal::{FixedTerminalV0_14, TerminalPredicateV0_14};
use crate::{ByteOffset, SourceId};

use crate::ClassifiedToken;

use super::{
    ExpectedBuilder, ParseCompilerFailure, ParseLimit, ParseLimits, ParseResourceFailure,
    ParseStorage, SyntaxCoordinate, SyntaxIssue, SyntaxRuleV0_14, Work,
};

pub(crate) enum DiagnosticResult {
    Issue(SyntaxIssue),
    Resource(ParseResourceFailure),
    Compiler(ParseCompilerFailure),
}

pub(crate) enum DecisionSelection {
    Arm(u8),
    NoMatch,
    Conflict,
}

#[derive(Clone, Copy)]
pub(crate) struct DiagnosticSite<'tokens, 'source> {
    pub(crate) source: SourceId,
    pub(crate) source_len: u64,
    pub(crate) tokens: &'tokens [ClassifiedToken<'source>],
    pub(crate) cursor: usize,
    pub(crate) limits: ParseLimits,
}

#[derive(Clone, Copy)]
pub(crate) struct ProbeContext {
    pub(crate) production: ProductionV0_14,
    pub(crate) atom_only: bool,
}

#[derive(Clone, Copy)]
enum ProbeTask {
    Execute(GrammarNodeIdV0_14, ProbeContext),
    Continue(GrammarNodeIdV0_14, ProbeContext),
    Match(TerminalPredicateV0_14, ProbeContext),
}

fn accepts(
    predicate: LookaheadPredicateV0_14,
    tokens: &[ClassifiedToken<'_>],
    cursor: usize,
    position: usize,
) -> Result<bool, ParseCompilerFailure> {
    let index = cursor
        .checked_add(position)
        .ok_or(ParseCompilerFailure::CounterOverflow)?;
    Ok(match (tokens.get(index), predicate) {
        (Some(token), LookaheadPredicateV0_14::Terminal(expected)) => {
            token.terminals().contains(expected)
        }
        (None, LookaheadPredicateV0_14::SourceEnd) => true,
        _ => false,
    })
}

fn row_score(
    row: SelectRowV0_14,
    tokens: &[ClassifiedToken<'_>],
    cursor: usize,
) -> Result<u8, ParseCompilerFailure> {
    let first = row
        .position(0)
        .ok_or(ParseCompilerFailure::InvalidGrammarData)?;
    if !accepts(first.predicate(), tokens, cursor, 0)? {
        return Ok(0);
    }
    let second = row
        .position(1)
        .ok_or(ParseCompilerFailure::InvalidGrammarData)?;
    Ok(if accepts(second.predicate(), tokens, cursor, 1)? {
        2
    } else {
        1
    })
}

pub(crate) fn select_arm(
    decision: DecisionV0_14,
    tokens: &[ClassifiedToken<'_>],
    cursor: usize,
    work: &mut Work,
) -> Result<DecisionSelection, DiagnosticResult> {
    let mut selected = None;
    for row in decision.rows() {
        work.spend(1).map_err(DiagnosticResult::Resource)?;
        if row_score(*row, tokens, cursor).map_err(DiagnosticResult::Compiler)? != 2 {
            continue;
        }
        match selected {
            Some(arm) if arm != row.arm() => return Ok(DecisionSelection::Conflict),
            Some(_) => {}
            None => selected = Some(row.arm()),
        }
    }
    Ok(selected.map_or(DecisionSelection::NoMatch, DecisionSelection::Arm))
}

fn boundary_coordinate(
    source: SourceId,
    source_len: u64,
    tokens: &[ClassifiedToken<'_>],
    cursor: usize,
    offset: usize,
) -> Result<SyntaxCoordinate, ParseCompilerFailure> {
    let index = cursor
        .checked_add(offset)
        .ok_or(ParseCompilerFailure::CounterOverflow)?;
    if let Some(token) = tokens.get(index) {
        let id = token.token().id();
        Ok(SyntaxCoordinate::new(source, id.start(), id.end()))
    } else {
        let end = ByteOffset::new(source_len);
        Ok(SyntaxCoordinate::new(source, end, end))
    }
}

fn has(token: &ClassifiedToken<'_>, predicate: TerminalPredicateV0_14) -> bool {
    token.terminals().contains(predicate)
}

fn fixed(token: &ClassifiedToken<'_>, terminal: FixedTerminalV0_14) -> bool {
    has(token, TerminalPredicateV0_14::Fixed(terminal))
}

fn dotted_override(
    source: SourceId,
    tokens: &[ClassifiedToken<'_>],
    boundary: usize,
    expected: super::ExpectedTerminalsV0_14,
    work: &mut Work,
) -> Result<Option<SyntaxIssue>, ParseResourceFailure> {
    if boundary >= tokens.len() {
        return Ok(None);
    }
    let first_start = boundary.saturating_sub(3);
    for start in first_start..=boundary {
        work.spend(1)?;
        let Some(end) = start.checked_add(4) else {
            continue;
        };
        if boundary >= end {
            continue;
        }
        let Some(window) = tokens.get(start..end) else {
            continue;
        };
        if has(&window[0], TerminalPredicateV0_14::Identifier)
            && fixed(&window[1], FixedTerminalV0_14::Dot)
            && has(&window[2], TerminalPredicateV0_14::Identifier)
            && (fixed(&window[3], FixedTerminalV0_14::LeftParen)
                || fixed(&window[3], FixedTerminalV0_14::LeftAngle))
        {
            return Ok(Some(SyntaxIssue {
                rule: SyntaxRuleV0_14::Form3,
                coordinate: SyntaxCoordinate::new(
                    source,
                    window[0].token().id().start(),
                    window[2].token().id().end(),
                ),
                expected,
            }));
        }
    }
    Ok(None)
}

fn forbidden_atom_override(
    source: SourceId,
    tokens: &[ClassifiedToken<'_>],
    cursor: usize,
    atom_only: bool,
    expected: super::ExpectedTerminalsV0_14,
) -> Option<SyntaxIssue> {
    if !atom_only {
        return None;
    }
    let first = tokens.get(cursor)?;
    let second = tokens.get(cursor.checked_add(1)?)?;
    let call_head = has(first, TerminalPredicateV0_14::Identifier)
        || has(first, TerminalPredicateV0_14::OperationName)
        || has(first, TerminalPredicateV0_14::TypeIdentifier);
    if call_head
        && (fixed(second, FixedTerminalV0_14::LeftParen)
            || fixed(second, FixedTerminalV0_14::LeftAngle))
    {
        return Some(SyntaxIssue {
            rule: SyntaxRuleV0_14::Gram9,
            coordinate: SyntaxCoordinate::new(
                source,
                first.token().id().start(),
                second.token().id().end(),
            ),
            expected,
        });
    }
    None
}

fn raw_restriction_owner(
    token: &ClassifiedToken<'_>,
    expected: super::ExpectedTerminalsV0_14,
) -> Option<SyntaxRuleV0_14> {
    for predicate in expected.iter() {
        match predicate {
            LookaheadPredicateV0_14::Terminal(TerminalPredicateV0_14::Identifier)
                if token.token().kind() == TokenKind::LowerWordForm
                    && !has(token, TerminalPredicateV0_14::Identifier) =>
            {
                return Some(SyntaxRuleV0_14::Form3);
            }
            LookaheadPredicateV0_14::Terminal(TerminalPredicateV0_14::Literal)
                if token.token().kind() == TokenKind::NumberForm
                    && !has(token, TerminalPredicateV0_14::Literal) =>
            {
                return Some(SyntaxRuleV0_14::Form5);
            }
            LookaheadPredicateV0_14::Terminal(TerminalPredicateV0_14::Digits)
                if token.token().kind() == TokenKind::NumberForm
                    && !has(token, TerminalPredicateV0_14::Digits) =>
            {
                return Some(SyntaxRuleV0_14::Const1);
            }
            _ => {}
        }
    }
    None
}

fn actual_name(token: &ClassifiedToken<'_>) -> Option<NamePredicateV0_14> {
    [
        NamePredicateV0_14::Identifier,
        NamePredicateV0_14::TypeIdentifier,
        NamePredicateV0_14::RegionIdentifier,
        NamePredicateV0_14::Label,
        NamePredicateV0_14::OperationName,
    ]
    .into_iter()
    .find(|predicate| has(token, predicate.terminal()))
}

fn name_slot_owner(
    token: &ClassifiedToken<'_>,
    transparent: Option<NamePredicateV0_14>,
    paths_agree: bool,
) -> Option<SyntaxRuleV0_14> {
    let actual = actual_name(token)?;
    let expected = transparent?;
    (paths_agree && actual != expected).then_some(SyntaxRuleV0_14::Form3)
}

fn construct_override(
    context: DecisionContextV0_14,
    source: SourceId,
    tokens: &[ClassifiedToken<'_>],
    cursor: usize,
    expected: super::ExpectedTerminalsV0_14,
) -> Option<SyntaxIssue> {
    if !matches!(
        context,
        DecisionContextV0_14::ConstructEntry | DecisionContextV0_14::ProgramItems
    ) {
        return None;
    }
    let token = tokens.get(cursor)?;
    if !has(token, TerminalPredicateV0_14::Identifier) {
        return None;
    }
    let id = token.token().id();
    Some(SyntaxIssue {
        rule: SyntaxRuleV0_14::Form1,
        coordinate: SyntaxCoordinate::new(source, id.start(), id.end()),
        expected,
    })
}

fn program_leftover(
    context: DecisionContextV0_14,
    source: SourceId,
    tokens: &[ClassifiedToken<'_>],
    cursor: usize,
) -> Option<SyntaxIssue> {
    if context != DecisionContextV0_14::ProgramItems {
        return None;
    }
    let token = tokens.get(cursor)?;
    let id = token.token().id();
    Some(SyntaxIssue {
        rule: SyntaxRuleV0_14::Gram2,
        coordinate: SyntaxCoordinate::new(source, id.start(), id.end()),
        expected: ExpectedBuilder::only_end().finish(),
    })
}

struct Frontier {
    maximum: u8,
    expected: super::ExpectedTerminalsV0_14,
    best_arm: Option<u8>,
    best_arm_internal: bool,
    transparent_name: Option<NamePredicateV0_14>,
    transparent_disagreement: bool,
    atom_only: bool,
}

fn frontier(
    decision: DecisionV0_14,
    tokens: &[ClassifiedToken<'_>],
    cursor: usize,
    work: &mut Work,
) -> Result<Frontier, DiagnosticResult> {
    let arm_count = usize::from(decision.arm_count());
    if arm_count == 0 || arm_count > 64 {
        return Err(DiagnosticResult::Compiler(
            ParseCompilerFailure::InvalidGrammarData,
        ));
    }
    let mut arm_scores = [0_u8; 64];
    let mut maximum = 0_u8;
    for row in decision.rows() {
        work.spend(1).map_err(DiagnosticResult::Resource)?;
        let score = row_score(*row, tokens, cursor).map_err(DiagnosticResult::Compiler)?;
        if score == 2 {
            return Err(DiagnosticResult::Compiler(
                ParseCompilerFailure::PredictiveConflict,
            ));
        }
        let arm = usize::from(row.arm());
        if arm >= arm_count {
            return Err(DiagnosticResult::Compiler(
                ParseCompilerFailure::InvalidGrammarData,
            ));
        }
        arm_scores[arm] = arm_scores[arm].max(score);
        maximum = maximum.max(score);
    }
    let mut expected = ExpectedBuilder::empty();
    let mut transparent_name = None;
    let mut transparent_disagreement = false;
    let mut atom_only = false;
    for row in decision.rows() {
        work.spend(1).map_err(DiagnosticResult::Resource)?;
        if row_score(*row, tokens, cursor).map_err(DiagnosticResult::Compiler)? != maximum {
            continue;
        }
        let atom = row
            .position(usize::from(maximum))
            .ok_or(DiagnosticResult::Compiler(
                ParseCompilerFailure::InvalidGrammarData,
            ))?;
        expected.insert(atom.predicate());
        atom_only |= atom.is_atom_only();
        if let Some(name) = atom.transparent_name() {
            match transparent_name {
                Some(previous) if previous != name => transparent_disagreement = true,
                Some(_) => {}
                None => transparent_name = Some(name),
            }
        }
    }
    let best_arms = arm_scores[..arm_count]
        .iter()
        .filter(|score| **score == maximum)
        .count();
    let best_arm = if best_arms == 1 {
        arm_scores[..arm_count]
            .iter()
            .position(|score| *score == maximum)
            .and_then(|index| u8::try_from(index).ok())
    } else {
        None
    };
    let mut best_arm_internal = best_arm.is_some();
    if let Some(arm) = best_arm {
        for row in decision.rows() {
            work.spend(1).map_err(DiagnosticResult::Resource)?;
            if row.arm() != arm
                || row_score(*row, tokens, cursor).map_err(DiagnosticResult::Compiler)? != maximum
            {
                continue;
            }
            let atom = row
                .position(usize::from(maximum))
                .ok_or(DiagnosticResult::Compiler(
                    ParseCompilerFailure::InvalidGrammarData,
                ))?;
            best_arm_internal &= atom.is_inside_arm();
        }
    }
    Ok(Frontier {
        maximum,
        expected: expected.finish(),
        best_arm,
        best_arm_internal,
        transparent_name,
        transparent_disagreement,
        atom_only,
    })
}

fn override_issue(
    decision: DecisionV0_14,
    frontier: &Frontier,
    site: DiagnosticSite<'_, '_>,
    atom_only: bool,
    work: &mut Work,
) -> Result<Option<SyntaxIssue>, DiagnosticResult> {
    let boundary = site
        .cursor
        .checked_add(usize::from(frontier.maximum))
        .ok_or(DiagnosticResult::Compiler(
            ParseCompilerFailure::CounterOverflow,
        ))?;
    if let Some(issue) =
        dotted_override(site.source, site.tokens, boundary, frontier.expected, work)
            .map_err(DiagnosticResult::Resource)?
    {
        return Ok(Some(issue));
    }
    if let Some(issue) = forbidden_atom_override(
        site.source,
        site.tokens,
        site.cursor,
        atom_only || frontier.atom_only,
        frontier.expected,
    ) {
        return Ok(Some(issue));
    }
    if let Some(token) = site.tokens.get(boundary) {
        if let Some(rule) = raw_restriction_owner(token, frontier.expected) {
            return Ok(Some(SyntaxIssue {
                rule,
                coordinate: boundary_coordinate(
                    site.source,
                    site.source_len,
                    site.tokens,
                    site.cursor,
                    usize::from(frontier.maximum),
                )
                .map_err(DiagnosticResult::Compiler)?,
                expected: frontier.expected,
            }));
        }
        if let Some(rule) = name_slot_owner(
            token,
            frontier.transparent_name,
            !frontier.transparent_disagreement,
        ) {
            return Ok(Some(SyntaxIssue {
                rule,
                coordinate: boundary_coordinate(
                    site.source,
                    site.source_len,
                    site.tokens,
                    site.cursor,
                    usize::from(frontier.maximum),
                )
                .map_err(DiagnosticResult::Compiler)?,
                expected: frontier.expected,
            }));
        }
    }
    if let Some(issue) = construct_override(
        decision.context(),
        site.source,
        site.tokens,
        site.cursor,
        frontier.expected,
    ) {
        return Ok(Some(issue));
    }
    Ok(program_leftover(
        decision.context(),
        site.source,
        site.tokens,
        site.cursor,
    ))
}

fn push_probe(
    tasks: &mut Vec<ProbeTask>,
    task: ProbeTask,
    limits: ParseLimits,
) -> Result<(), ParseResourceFailure> {
    let actual = u64::try_from(tasks.len())
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or(ParseResourceFailure::AddressSpaceExceeded {
            storage: ParseStorage::Tasks,
            requested: u64::MAX,
        })?;
    if actual > limits.max_tasks {
        return Err(ParseResourceFailure::LimitExceeded {
            limit: ParseLimit::Tasks,
            maximum: limits.max_tasks,
            actual,
        });
    }
    tasks
        .try_reserve(1)
        .map_err(|_| ParseResourceFailure::StorageUnavailable {
            storage: ParseStorage::Tasks,
            requested: actual,
        })?;
    tasks.push(task);
    Ok(())
}

fn arm_node(
    decision: DecisionV0_14,
    arm: u8,
) -> Result<Option<GrammarNodeIdV0_14>, ParseCompilerFailure> {
    let node =
        grammar_node_v0_14(decision.node()).ok_or(ParseCompilerFailure::MissingGrammarNode)?;
    match decision.kind() {
        DecisionKindV0_14::Choice => node
            .children()
            .get(usize::from(arm))
            .copied()
            .map(Some)
            .ok_or(ParseCompilerFailure::InvalidGrammarData),
        DecisionKindV0_14::Optional | DecisionKindV0_14::Repeat0 | DecisionKindV0_14::Repeat1 => {
            match arm {
                0 => node
                    .children()
                    .first()
                    .copied()
                    .map(Some)
                    .ok_or(ParseCompilerFailure::InvalidGrammarData),
                1 => Ok(None),
                _ => Err(ParseCompilerFailure::InvalidGrammarData),
            }
        }
    }
}

fn descend_or_issue(
    decision: DecisionV0_14,
    context: ProbeContext,
    site: DiagnosticSite<'_, '_>,
    work: &mut Work,
    tasks: &mut Vec<ProbeTask>,
) -> Result<Option<SyntaxIssue>, DiagnosticResult> {
    let value = frontier(decision, site.tokens, site.cursor, work)?;
    if let Some(issue) = override_issue(decision, &value, site, context.atom_only, work)? {
        return Ok(Some(issue));
    }
    if value.best_arm_internal {
        let arm = value.best_arm.ok_or(DiagnosticResult::Compiler(
            ParseCompilerFailure::InvalidGrammarData,
        ))?;
        let node = arm_node(decision, arm).map_err(DiagnosticResult::Compiler)?;
        let Some(node) = node else {
            return Ok(Some(SyntaxIssue {
                rule: SyntaxRuleV0_14::from(decision.production().owner()),
                coordinate: boundary_coordinate(
                    site.source,
                    site.source_len,
                    site.tokens,
                    site.cursor,
                    usize::from(value.maximum),
                )
                .map_err(DiagnosticResult::Compiler)?,
                expected: value.expected,
            }));
        };
        tasks.clear();
        push_probe(tasks, ProbeTask::Execute(node, context), site.limits)
            .map_err(DiagnosticResult::Resource)?;
        return Ok(None);
    }
    Ok(Some(SyntaxIssue {
        rule: SyntaxRuleV0_14::from(decision.production().owner()),
        coordinate: boundary_coordinate(
            site.source,
            site.source_len,
            site.tokens,
            site.cursor,
            usize::from(value.maximum),
        )
        .map_err(DiagnosticResult::Compiler)?,
        expected: value.expected,
    }))
}

pub(crate) fn direct_mismatch(
    expected_terminal: TerminalPredicateV0_14,
    context: ProbeContext,
    site: DiagnosticSite<'_, '_>,
    work: &mut Work,
) -> DiagnosticResult {
    let mut builder = ExpectedBuilder::empty();
    builder.insert(LookaheadPredicateV0_14::Terminal(expected_terminal));
    let expected = builder.finish();
    match dotted_override(site.source, site.tokens, site.cursor, expected, work) {
        Ok(Some(issue)) => return DiagnosticResult::Issue(issue),
        Ok(None) => {}
        Err(failure) => return DiagnosticResult::Resource(failure),
    }
    if let Some(issue) = forbidden_atom_override(
        site.source,
        site.tokens,
        site.cursor,
        context.atom_only,
        expected,
    ) {
        return DiagnosticResult::Issue(issue);
    }
    if let Some(token) = site.tokens.get(site.cursor) {
        if let Some(rule) = raw_restriction_owner(token, expected) {
            return DiagnosticResult::Issue(SyntaxIssue {
                rule,
                coordinate: SyntaxCoordinate::new(
                    site.source,
                    token.token().id().start(),
                    token.token().id().end(),
                ),
                expected,
            });
        }
        let transparent = [
            NamePredicateV0_14::Identifier,
            NamePredicateV0_14::TypeIdentifier,
            NamePredicateV0_14::RegionIdentifier,
            NamePredicateV0_14::Label,
            NamePredicateV0_14::OperationName,
        ]
        .into_iter()
        .find(|name| name.terminal() == expected_terminal);
        if let Some(rule) = name_slot_owner(token, transparent, true) {
            return DiagnosticResult::Issue(SyntaxIssue {
                rule,
                coordinate: SyntaxCoordinate::new(
                    site.source,
                    token.token().id().start(),
                    token.token().id().end(),
                ),
                expected,
            });
        }
    }
    match boundary_coordinate(site.source, site.source_len, site.tokens, site.cursor, 0) {
        Ok(coordinate) => DiagnosticResult::Issue(SyntaxIssue {
            rule: SyntaxRuleV0_14::from(context.production.owner()),
            coordinate,
            expected,
        }),
        Err(failure) => DiagnosticResult::Compiler(failure),
    }
}

fn probe(
    initial: GrammarNodeIdV0_14,
    context: ProbeContext,
    site: DiagnosticSite<'_, '_>,
    work: &mut Work,
) -> DiagnosticResult {
    let mut cursor = site.cursor;
    let mut tasks = Vec::new();
    if let Err(failure) = push_probe(
        &mut tasks,
        ProbeTask::Execute(initial, context),
        site.limits,
    ) {
        return DiagnosticResult::Resource(failure);
    }
    while let Some(task) = tasks.pop() {
        if let Err(failure) = work.spend(1) {
            return DiagnosticResult::Resource(failure);
        }
        match task {
            ProbeTask::Match(expected, task_context) => {
                let matches = site
                    .tokens
                    .get(cursor)
                    .is_some_and(|token| token.terminals().contains(expected));
                if !matches {
                    return direct_mismatch(
                        expected,
                        task_context,
                        DiagnosticSite { cursor, ..site },
                        work,
                    );
                }
                let Some(next) = cursor.checked_add(1) else {
                    return DiagnosticResult::Compiler(ParseCompilerFailure::CounterOverflow);
                };
                cursor = next;
            }
            ProbeTask::Execute(node_id, task_context) => {
                let Some(node) = grammar_node_v0_14(node_id) else {
                    return DiagnosticResult::Compiler(ParseCompilerFailure::MissingGrammarNode);
                };
                match node.kind() {
                    GrammarNodeKindV0_14::Production(production) => {
                        let nested = ProbeContext {
                            production,
                            atom_only: node.is_atom_only_reference(),
                        };
                        if let Err(failure) = push_probe(
                            &mut tasks,
                            ProbeTask::Execute(production.root(), nested),
                            site.limits,
                        ) {
                            return DiagnosticResult::Resource(failure);
                        }
                    }
                    GrammarNodeKindV0_14::TerminalSequence => {
                        for terminal in node.terminals().iter().rev() {
                            let LookaheadPredicateV0_14::Terminal(predicate) = terminal else {
                                return DiagnosticResult::Compiler(
                                    ParseCompilerFailure::InvalidGrammarData,
                                );
                            };
                            if let Err(failure) = push_probe(
                                &mut tasks,
                                ProbeTask::Match(*predicate, task_context),
                                site.limits,
                            ) {
                                return DiagnosticResult::Resource(failure);
                            }
                        }
                    }
                    GrammarNodeKindV0_14::Sequence => {
                        for child in node.children().iter().rev() {
                            if let Err(failure) = push_probe(
                                &mut tasks,
                                ProbeTask::Execute(*child, task_context),
                                site.limits,
                            ) {
                                return DiagnosticResult::Resource(failure);
                            }
                        }
                    }
                    GrammarNodeKindV0_14::Group => {
                        let Some(child) = node.children().first() else {
                            return DiagnosticResult::Compiler(
                                ParseCompilerFailure::InvalidGrammarData,
                            );
                        };
                        if let Err(failure) = push_probe(
                            &mut tasks,
                            ProbeTask::Execute(*child, task_context),
                            site.limits,
                        ) {
                            return DiagnosticResult::Resource(failure);
                        }
                    }
                    GrammarNodeKindV0_14::RepeatOne => {
                        let Some(child) = node.children().first() else {
                            return DiagnosticResult::Compiler(
                                ParseCompilerFailure::InvalidGrammarData,
                            );
                        };
                        for next in [
                            ProbeTask::Continue(node_id, task_context),
                            ProbeTask::Execute(*child, task_context),
                        ] {
                            if let Err(failure) = push_probe(&mut tasks, next, site.limits) {
                                return DiagnosticResult::Resource(failure);
                            }
                        }
                    }
                    GrammarNodeKindV0_14::Choice
                    | GrammarNodeKindV0_14::Optional
                    | GrammarNodeKindV0_14::RepeatZero => {
                        let Some(decision) = node.decision().copied() else {
                            return DiagnosticResult::Compiler(
                                ParseCompilerFailure::InvalidGrammarData,
                            );
                        };
                        match select_arm(decision, site.tokens, cursor, work) {
                            Ok(DecisionSelection::Arm(arm)) => {
                                let selected = match arm_node(decision, arm) {
                                    Ok(selected) => selected,
                                    Err(failure) => return DiagnosticResult::Compiler(failure),
                                };
                                if let Some(selected) = selected {
                                    if decision.kind() == DecisionKindV0_14::Repeat0
                                        && let Err(failure) = push_probe(
                                            &mut tasks,
                                            ProbeTask::Continue(node_id, task_context),
                                            site.limits,
                                        )
                                    {
                                        return DiagnosticResult::Resource(failure);
                                    }
                                    if let Err(failure) = push_probe(
                                        &mut tasks,
                                        ProbeTask::Execute(selected, task_context),
                                        site.limits,
                                    ) {
                                        return DiagnosticResult::Resource(failure);
                                    }
                                }
                            }
                            Ok(DecisionSelection::NoMatch) => match descend_or_issue(
                                decision,
                                task_context,
                                DiagnosticSite { cursor, ..site },
                                work,
                                &mut tasks,
                            ) {
                                Ok(Some(issue)) => return DiagnosticResult::Issue(issue),
                                Ok(None) => {}
                                Err(result) => return result,
                            },
                            Ok(DecisionSelection::Conflict) => {
                                return DiagnosticResult::Compiler(
                                    ParseCompilerFailure::PredictiveConflict,
                                );
                            }
                            Err(result) => return result,
                        }
                    }
                }
            }
            ProbeTask::Continue(node_id, task_context) => {
                let Some(node) = grammar_node_v0_14(node_id) else {
                    return DiagnosticResult::Compiler(ParseCompilerFailure::MissingGrammarNode);
                };
                let Some(decision) = node.decision().copied() else {
                    return DiagnosticResult::Compiler(ParseCompilerFailure::InvalidGrammarData);
                };
                match select_arm(decision, site.tokens, cursor, work) {
                    Ok(DecisionSelection::Arm(0)) => {
                        let Some(child) = node.children().first() else {
                            return DiagnosticResult::Compiler(
                                ParseCompilerFailure::InvalidGrammarData,
                            );
                        };
                        for next in [
                            ProbeTask::Continue(node_id, task_context),
                            ProbeTask::Execute(*child, task_context),
                        ] {
                            if let Err(failure) = push_probe(&mut tasks, next, site.limits) {
                                return DiagnosticResult::Resource(failure);
                            }
                        }
                    }
                    Ok(DecisionSelection::Arm(1)) => {}
                    Ok(DecisionSelection::Arm(_)) => {
                        return DiagnosticResult::Compiler(
                            ParseCompilerFailure::InvalidGrammarData,
                        );
                    }
                    Ok(DecisionSelection::NoMatch) => match descend_or_issue(
                        decision,
                        task_context,
                        DiagnosticSite { cursor, ..site },
                        work,
                        &mut tasks,
                    ) {
                        Ok(Some(issue)) => return DiagnosticResult::Issue(issue),
                        Ok(None) => {}
                        Err(result) => return result,
                    },
                    Ok(DecisionSelection::Conflict) => {
                        return DiagnosticResult::Compiler(
                            ParseCompilerFailure::PredictiveConflict,
                        );
                    }
                    Err(result) => return result,
                }
            }
        }
    }
    DiagnosticResult::Compiler(ParseCompilerFailure::DiagnosticReachedSuccessfulEnd)
}

pub(crate) fn diagnose_decision(
    decision: DecisionV0_14,
    context: ProbeContext,
    site: DiagnosticSite<'_, '_>,
    work: &mut Work,
) -> DiagnosticResult {
    let value = match frontier(decision, site.tokens, site.cursor, work) {
        Ok(value) => value,
        Err(result) => return result,
    };
    match override_issue(decision, &value, site, context.atom_only, work) {
        Ok(Some(issue)) => return DiagnosticResult::Issue(issue),
        Ok(None) => {}
        Err(result) => return result,
    }
    if value.best_arm_internal {
        let Some(arm) = value.best_arm else {
            return DiagnosticResult::Compiler(ParseCompilerFailure::InvalidGrammarData);
        };
        let initial = match arm_node(decision, arm) {
            Ok(Some(node)) => node,
            Ok(None) => {
                return DiagnosticResult::Compiler(
                    ParseCompilerFailure::DiagnosticReachedSuccessfulEnd,
                );
            }
            Err(failure) => return DiagnosticResult::Compiler(failure),
        };
        return probe(initial, context, site, work);
    }
    let coordinate = match boundary_coordinate(
        site.source,
        site.source_len,
        site.tokens,
        site.cursor,
        usize::from(value.maximum),
    ) {
        Ok(coordinate) => coordinate,
        Err(failure) => return DiagnosticResult::Compiler(failure),
    };
    DiagnosticResult::Issue(SyntaxIssue {
        rule: SyntaxRuleV0_14::from(decision.production().owner()),
        coordinate,
        expected: value.expected,
    })
}
