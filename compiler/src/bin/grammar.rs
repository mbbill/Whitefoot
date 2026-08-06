#![forbid(unsafe_code)]

use std::fmt;
use std::path::Path;

use whitefoot::{
    ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_HASH, ALL_FIXED_TERMINALS,
    ALL_TERMINAL_PREDICATES, FixedTerminal, GrammarNodeKind, GrammarStage, LexLimits, LexOutcome,
    LookaheadPredicate, ParseLimits, ParseOutcome, STAGED_FIXED_TERMINALS,
    STAGED_FRONTEND_CONTRACT_TEXT, STAGED_SYNTAX_CONTRACT_HASH, STAGED_TERMINAL_PREDICATES,
    SourceBundle, SourceInput, SourceLimits, SpecHash, TerminalLimits, TerminalOutcome,
    TerminalPredicate, classify_terminals, lex, parse,
};

const PARSER_PROBE: &[u8] = b"fn main() -> own unit pure {\n  return unit;\n}\n";

/// The staged candidate's canonical complete command-entry header ([FN-7])
/// over the same minimal body, plus the unchanged unlabelled entry.
const STAGED_PARSER_PROBES: [&[u8]; 2] = [
    PARSER_PROBE,
    b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {\n  return unit;\n}\n",
];

const FRONTEND_SECTIONS: [(&str, &str); 3] = [
    ("[FORM-1]", "## 4. Types"),
    ("[CONST-1]", "## 5. Ownership"),
    ("[EFF-1]", "[EFF-2]"),
];

#[derive(Debug)]
enum VerifyError {
    Invocation(&'static str),
    Read(std::io::Error),
    NonUtf8,
    MissingSection(&'static str),
    ChangedFrontendContract,
    InvalidCompilerGrammar(&'static str),
    ParserProbe(String),
}

impl fmt::Display for VerifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invocation(message) => formatter.write_str(message),
            Self::Read(error) => write!(formatter, "cannot read candidate: {error}"),
            Self::NonUtf8 => formatter.write_str("candidate is not UTF-8"),
            Self::MissingSection(marker) => {
                write!(formatter, "candidate is missing frontend section {marker}")
            }
            Self::ChangedFrontendContract => formatter.write_str(
                "candidate changes the lexer or source grammar beyond the contracts this compiler carries; a structural change must first extend the native active or staged grammar path",
            ),
            Self::InvalidCompilerGrammar(message) => {
                write!(formatter, "compiler grammar data is inconsistent: {message}")
            }
            Self::ParserProbe(message) => {
                write!(formatter, "compiler parser probe failed: {message}")
            }
        }
    }
}

impl std::error::Error for VerifyError {}

fn main() {
    if let Err(error) = run() {
        eprintln!("whitefoot-grammar: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), VerifyError> {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();
    let candidate = arguments.next().ok_or(VerifyError::Invocation(
        "usage: whitefoot-grammar PATH-TO-CANDIDATE",
    ))?;
    if arguments.next().is_some() {
        return Err(VerifyError::Invocation(
            "usage: whitefoot-grammar PATH-TO-CANDIDATE",
        ));
    }
    let bytes = std::fs::read(Path::new(&candidate)).map_err(VerifyError::Read)?;
    let report = verify_candidate(&bytes)?;
    match report.stage {
        GrammarStage::Active => println!(
            "grammar-preserving candidate verified by the active compiler: {} productions, {} decisions, {} terminal predicates",
            report.productions, report.decisions, report.terminals
        ),
        GrammarStage::Staged => println!(
            "staged candidate contract verified by the active compiler's staged grammar path: {} productions, {} decisions, {} terminal predicates",
            report.productions, report.decisions, report.terminals
        ),
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VerifyReport {
    stage: GrammarStage,
    productions: usize,
    decisions: usize,
    terminals: usize,
}

fn verify_candidate(candidate: &[u8]) -> Result<VerifyReport, VerifyError> {
    let contract = frontend_contract(candidate)?;
    let stage = if contract == frontend_contract(ACTIVE_KERNEL_SPEC_BYTES)? {
        GrammarStage::Active
    } else if contract == frontend_contract(STAGED_FRONTEND_CONTRACT_TEXT.as_bytes())? {
        GrammarStage::Staged
    } else {
        return Err(VerifyError::ChangedFrontendContract);
    };
    let report = verify_compiler_grammar(stage)?;
    run_parser_probes(stage)?;
    Ok(report)
}

fn frontend_contract(specification: &[u8]) -> Result<Vec<u8>, VerifyError> {
    let text = std::str::from_utf8(specification).map_err(|_| VerifyError::NonUtf8)?;
    let mut contract = Vec::new();
    for (start_marker, end_marker) in FRONTEND_SECTIONS {
        let start =
            line_start(text, start_marker).ok_or(VerifyError::MissingSection(start_marker))?;
        let end = line_start(&text[start..], end_marker)
            .map(|offset| start + offset)
            .ok_or(VerifyError::MissingSection(end_marker))?;
        let section =
            text.as_bytes()
                .get(start..end)
                .ok_or(VerifyError::InvalidCompilerGrammar(
                    "frontend section bounds are invalid",
                ))?;
        let length = u64::try_from(section.len())
            .map_err(|_| VerifyError::InvalidCompilerGrammar("frontend section is too large"))?;
        contract.extend_from_slice(&length.to_be_bytes());
        contract.extend_from_slice(section);
    }
    Ok(contract)
}

fn line_start(text: &str, marker: &str) -> Option<usize> {
    text.match_indices(marker)
        .map(|(index, _)| index)
        .find(|index| *index == 0 || text.as_bytes().get(index - 1) == Some(&b'\n'))
}

fn stage_fixed_terminals(stage: GrammarStage) -> &'static [FixedTerminal] {
    match stage {
        GrammarStage::Active => ALL_FIXED_TERMINALS.as_slice(),
        GrammarStage::Staged => STAGED_FIXED_TERMINALS.as_slice(),
    }
}

fn stage_terminal_predicates(stage: GrammarStage) -> &'static [TerminalPredicate] {
    match stage {
        GrammarStage::Active => ALL_TERMINAL_PREDICATES.as_slice(),
        GrammarStage::Staged => STAGED_TERMINAL_PREDICATES.as_slice(),
    }
}

fn verify_compiler_grammar(stage: GrammarStage) -> Result<VerifyReport, VerifyError> {
    let fixed_terminals = stage_fixed_terminals(stage);
    for (left_index, left) in fixed_terminals.iter().enumerate() {
        for right in &fixed_terminals[left_index + 1..] {
            if left.spelling() == right.spelling() {
                return Err(VerifyError::InvalidCompilerGrammar(
                    "two fixed terminals have the same spelling",
                ));
            }
        }
    }

    let order = stage.diagnostic_terminal_order();
    let predicates = stage_terminal_predicates(stage);
    if order.len() != predicates.len() {
        return Err(VerifyError::InvalidCompilerGrammar(
            "terminal inventory and diagnostic order differ",
        ));
    }
    for predicate in predicates {
        if order
            .iter()
            .filter(|candidate| **candidate == LookaheadPredicate::Terminal(*predicate))
            .count()
            != 1
        {
            return Err(VerifyError::InvalidCompilerGrammar(
                "terminal diagnostic order is not a permutation",
            ));
        }
    }

    let mut decisions = 0_usize;
    for production in stage.productions() {
        let root =
            stage
                .production_root(*production)
                .ok_or(VerifyError::InvalidCompilerGrammar(
                    "a production has no root node",
                ))?;
        let mut stack = vec![root];
        while let Some(node_id) = stack.pop() {
            let node = stage
                .node(node_id)
                .ok_or(VerifyError::InvalidCompilerGrammar(
                    "a production references a missing node",
                ))?;
            if let Some(decision) = node.decision() {
                decisions = decisions
                    .checked_add(1)
                    .ok_or(VerifyError::InvalidCompilerGrammar(
                        "decision count overflowed",
                    ))?;
                let mut covered = vec![false; usize::from(decision.arm_count())];
                for row in decision.rows() {
                    let arm = covered.get_mut(usize::from(row.arm())).ok_or(
                        VerifyError::InvalidCompilerGrammar("a SELECT row has an invalid arm"),
                    )?;
                    *arm = true;
                    if row.position(0).is_none() || row.position(1).is_none() {
                        return Err(VerifyError::InvalidCompilerGrammar(
                            "a SELECT row does not have two positions",
                        ));
                    }
                }
                if covered.iter().any(|covered| !covered) {
                    return Err(VerifyError::InvalidCompilerGrammar(
                        "a decision arm has no SELECT row",
                    ));
                }
                verify_disjoint_rows(decision.rows())?;
            }
            if matches!(
                node.kind(),
                GrammarNodeKind::Sequence
                    | GrammarNodeKind::Choice
                    | GrammarNodeKind::Group
                    | GrammarNodeKind::Optional
                    | GrammarNodeKind::RepeatZero
                    | GrammarNodeKind::RepeatOne
            ) {
                stack.extend_from_slice(node.children());
            }
        }
    }
    Ok(VerifyReport {
        stage,
        productions: stage.productions().len(),
        decisions,
        terminals: order.len(),
    })
}

fn verify_disjoint_rows(rows: &[whitefoot::SelectRow]) -> Result<(), VerifyError> {
    for (left_index, left) in rows.iter().enumerate() {
        for right in &rows[left_index + 1..] {
            if left.arm() == right.arm() {
                continue;
            }
            let first_overlaps = predicates_overlap(
                left.position(0)
                    .ok_or(VerifyError::InvalidCompilerGrammar(
                        "a SELECT row is missing position zero",
                    ))?
                    .predicate(),
                right
                    .position(0)
                    .ok_or(VerifyError::InvalidCompilerGrammar(
                        "a SELECT row is missing position zero",
                    ))?
                    .predicate(),
            );
            let second_overlaps = predicates_overlap(
                left.position(1)
                    .ok_or(VerifyError::InvalidCompilerGrammar(
                        "a SELECT row is missing position one",
                    ))?
                    .predicate(),
                right
                    .position(1)
                    .ok_or(VerifyError::InvalidCompilerGrammar(
                        "a SELECT row is missing position one",
                    ))?
                    .predicate(),
            );
            if first_overlaps && second_overlaps {
                return Err(VerifyError::InvalidCompilerGrammar(
                    "two source arms have overlapping SELECT_2 rows",
                ));
            }
        }
    }
    Ok(())
}

fn predicates_overlap(left: LookaheadPredicate, right: LookaheadPredicate) -> bool {
    if left == right {
        return true;
    }
    matches!(
        (left, right),
        (
            LookaheadPredicate::Terminal(TerminalPredicate::Fixed(whitefoot::FixedTerminal::Unit)),
            LookaheadPredicate::Terminal(TerminalPredicate::Literal)
        ) | (
            LookaheadPredicate::Terminal(TerminalPredicate::Literal),
            LookaheadPredicate::Terminal(TerminalPredicate::Fixed(whitefoot::FixedTerminal::Unit))
        )
    )
}

fn run_parser_probes(stage: GrammarStage) -> Result<(), VerifyError> {
    let (contract, probes): (SpecHash, &[&[u8]]) = match stage {
        GrammarStage::Active => (ACTIVE_KERNEL_SPEC_HASH, &[PARSER_PROBE]),
        GrammarStage::Staged => (STAGED_SYNTAX_CONTRACT_HASH, &STAGED_PARSER_PROBES),
    };
    for probe in probes {
        run_parser_probe(contract, probe)?;
    }
    Ok(())
}

fn run_parser_probe(contract: SpecHash, probe: &[u8]) -> Result<(), VerifyError> {
    let bundle = SourceBundle::with_limits(
        &[SourceInput::new("grammar-probe.wf", probe)],
        SourceLimits {
            max_sources: 1,
            max_logical_path_bytes: 64,
            max_source_bytes: 4_096,
            max_total_source_bytes: 4_096,
            max_binding_bytes: 8_192,
        },
    )
    .map_err(|error| VerifyError::ParserProbe(format!("source bundle: {error}")))?;
    let lexed = match lex(
        &bundle,
        LexLimits {
            max_sources: 1,
            max_source_bytes: 4_096,
            max_total_source_bytes: 4_096,
            max_token_bytes: 256,
            max_tokens: 256,
            max_lexemes: 512,
        },
    ) {
        LexOutcome::Complete(lexed) => lexed,
        outcome => return Err(VerifyError::ParserProbe(format!("lexing: {outcome:?}"))),
    };
    let classified = match classify_terminals(&lexed, contract, TerminalLimits { max_tokens: 256 })
    {
        TerminalOutcome::Complete(classified) => classified,
        outcome => {
            return Err(VerifyError::ParserProbe(format!(
                "terminal classification: {outcome:?}"
            )));
        }
    };
    match parse(
        &classified,
        ParseLimits {
            max_work: 100_000,
            max_tasks: 4_096,
            max_frames: 512,
            max_elements: 4_096,
        },
    ) {
        ParseOutcome::Complete(_) => Ok(()),
        outcome => Err(VerifyError::ParserProbe(format!(
            "grammar derivation: {outcome:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ACTIVE_KERNEL_SPEC_BYTES, GrammarStage, STAGED_FRONTEND_CONTRACT_TEXT, VerifyError,
        verify_candidate,
    };

    #[test]
    fn exact_active_frontend_contract_verifies() {
        let report =
            verify_candidate(ACTIVE_KERNEL_SPEC_BYTES).expect("active grammar must verify");
        assert_eq!(report.stage, GrammarStage::Active);
        assert_eq!(report.productions, 62);
        assert_eq!(report.decisions, 72);
        assert_eq!(report.terminals, 72);
    }

    #[test]
    fn prose_outside_the_frontend_contract_may_change() {
        let mut proposal = ACTIVE_KERNEL_SPEC_BYTES.to_vec();
        proposal.extend_from_slice(b"\nSemantic-only proposal text.\n");
        let report =
            verify_candidate(&proposal).expect("semantic-only text must preserve the grammar");
        assert_eq!(report.stage, GrammarStage::Active);
    }

    #[test]
    fn exact_staged_frontend_contract_verifies() {
        let report = verify_candidate(STAGED_FRONTEND_CONTRACT_TEXT.as_bytes())
            .expect("staged contract must verify through the staged tables");
        assert_eq!(report.stage, GrammarStage::Staged);
        assert_eq!(report.productions, 64);
        assert_eq!(report.decisions, 74);
        assert_eq!(report.terminals, 75);
    }

    #[test]
    fn changed_source_grammar_fails_closed() {
        let active = std::str::from_utf8(ACTIVE_KERNEL_SPEC_BYTES).expect("active spec is UTF-8");
        let changed = active.replacen(
            "return_stmt := \"return\" expr \";\"",
            "return_stmt := \"return\" atom \";\"",
            1,
        );
        assert!(matches!(
            verify_candidate(changed.as_bytes()),
            Err(VerifyError::ChangedFrontendContract)
        ));
    }

    #[test]
    fn changed_staged_effect_order_fails_closed() {
        let changed = STAGED_FRONTEND_CONTRACT_TEXT.replacen(
            "| \"external\" | \"blocks\" | \"traps\"",
            "| \"blocks\" | \"external\" | \"traps\"",
            1,
        );
        assert_ne!(changed, STAGED_FRONTEND_CONTRACT_TEXT);
        assert!(matches!(
            verify_candidate(changed.as_bytes()),
            Err(VerifyError::ChangedFrontendContract)
        ));
    }

    #[test]
    fn changed_staged_input_label_spelling_fails_closed() {
        let changed = STAGED_FRONTEND_CONTRACT_TEXT.replacen(
            "input_label  := IDENT \".\" IDENT \"as\"",
            "input_label  := IDENT \".\" IDENT \"from\"",
            1,
        );
        assert_ne!(changed, STAGED_FRONTEND_CONTRACT_TEXT);
        assert!(matches!(
            verify_candidate(changed.as_bytes()),
            Err(VerifyError::ChangedFrontendContract)
        ));
    }

    #[test]
    fn changed_comment_lexing_fails_closed() {
        let active = std::str::from_utf8(ACTIVE_KERNEL_SPEC_BYTES).expect("active spec is UTF-8");
        let changed = active.replacen(
            "[FORM-4] There are no comments.",
            "[FORM-4] Line comments begin with two slash bytes.",
            1,
        );
        assert!(matches!(
            verify_candidate(changed.as_bytes()),
            Err(VerifyError::ChangedFrontendContract)
        ));
    }

    #[test]
    fn changed_unit_lexing_fails_closed() {
        let active = std::str::from_utf8(ACTIVE_KERNEL_SPEC_BYTES).expect("active spec is UTF-8");
        let changed = active.replacen(
            "[FORM-6] The token `unit` names the unit type in type position and the unit value in expression position",
            "[FORM-6] The tokens `unit` and `void` name unit values in expression position",
            1,
        );
        assert!(matches!(
            verify_candidate(changed.as_bytes()),
            Err(VerifyError::ChangedFrontendContract)
        ));
    }
}
