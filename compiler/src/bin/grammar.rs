#![forbid(unsafe_code)]

use std::fmt;
use std::path::Path;

use whitefoot::{
    ACTIVE_KERNEL_SPEC_HASH, ALL_FIXED_TERMINALS, ALL_TERMINAL_PREDICATES, GrammarNodeKind,
    LexLimits, LexOutcome, LookaheadPredicate, ParseLimits, ParseOutcome, SourceBundle,
    SourceInput, SourceLimits, TerminalLimits, TerminalOutcome, TerminalPredicate,
    classify_terminals, diagnostic_terminal_order, grammar_node, lex, parse, productions,
};

/// The unlabelled entry and the canonical complete command-entry header
/// ([FN-7]) over the same minimal body.
const PARSER_PROBES: [&[u8]; 2] = [
    b"fn main() -> own unit pure {\n  return unit;\n}\n",
    b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {\n  return unit;\n}\n",
];

const FRONTEND_SECTIONS: [(&str, &str); 3] = [
    ("[FORM-1]", "## 4. Types"),
    ("[CONST-1]", "## 5. Ownership"),
    ("[EFF-1]", "[EFF-2]"),
];

const USAGE: &str = "usage: whitefoot-grammar PATH-TO-BASELINE PATH-TO-CANDIDATE";

#[derive(Debug)]
enum VerifyError {
    Invocation(&'static str),
    Read(String, std::io::Error),
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
            Self::Read(path, error) => write!(formatter, "cannot read {path}: {error}"),
            Self::NonUtf8 => formatter.write_str("a specification is not UTF-8"),
            Self::MissingSection(marker) => {
                write!(
                    formatter,
                    "a specification is missing frontend section {marker}"
                )
            }
            Self::ChangedFrontendContract => formatter.write_str(
                "candidate changes the lexer or source grammar of the baseline; a structural change must first extend the native grammar path",
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
    let (Some(baseline), Some(candidate), None) =
        (arguments.next(), arguments.next(), arguments.next())
    else {
        return Err(VerifyError::Invocation(USAGE));
    };
    let baseline = read_specification(&baseline)?;
    let candidate = read_specification(&candidate)?;
    let report = verify_candidate(&baseline, &candidate)?;
    println!(
        "grammar-preserving candidate verified by the active compiler: {} productions, {} decisions, {} terminal predicates",
        report.productions, report.decisions, report.terminals
    );
    Ok(())
}

fn read_specification(path: &std::ffi::OsStr) -> Result<Vec<u8>, VerifyError> {
    let path = Path::new(path);
    std::fs::read(path).map_err(|error| VerifyError::Read(path.display().to_string(), error))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VerifyReport {
    productions: usize,
    decisions: usize,
    terminals: usize,
}

/// Check that `candidate` keeps `baseline`'s frontend contract, then that the
/// compiler's own grammar data is consistent and parses.
///
/// Both specifications are arguments because the interesting comparison is
/// between two files chosen at the call site. Comparing against the compiled-in
/// active bytes made the check vacuous the moment a candidate was installed as
/// the active specification, which is exactly when the workflow reruns it.
fn verify_candidate(baseline: &[u8], candidate: &[u8]) -> Result<VerifyReport, VerifyError> {
    if frontend_contract(candidate)? != frontend_contract(baseline)? {
        return Err(VerifyError::ChangedFrontendContract);
    }
    let report = verify_compiler_grammar()?;
    run_parser_probes()?;
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

fn verify_compiler_grammar() -> Result<VerifyReport, VerifyError> {
    for (left_index, left) in ALL_FIXED_TERMINALS.iter().enumerate() {
        for right in &ALL_FIXED_TERMINALS[left_index + 1..] {
            if left.spelling() == right.spelling() {
                return Err(VerifyError::InvalidCompilerGrammar(
                    "two fixed terminals have the same spelling",
                ));
            }
        }
    }

    let order = diagnostic_terminal_order();
    if order.len() != ALL_TERMINAL_PREDICATES.len() {
        return Err(VerifyError::InvalidCompilerGrammar(
            "terminal inventory and diagnostic order differ",
        ));
    }
    for predicate in ALL_TERMINAL_PREDICATES {
        if order
            .iter()
            .filter(|candidate| **candidate == LookaheadPredicate::Terminal(predicate))
            .count()
            != 1
        {
            return Err(VerifyError::InvalidCompilerGrammar(
                "terminal diagnostic order is not a permutation",
            ));
        }
    }

    let mut decisions = 0_usize;
    for production in productions() {
        let mut stack = vec![production.root()];
        while let Some(node_id) = stack.pop() {
            let node = grammar_node(node_id).ok_or(VerifyError::InvalidCompilerGrammar(
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
        productions: productions().len(),
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

fn run_parser_probes() -> Result<(), VerifyError> {
    for probe in PARSER_PROBES {
        run_parser_probe(probe)?;
    }
    Ok(())
}

fn run_parser_probe(probe: &[u8]) -> Result<(), VerifyError> {
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
    let classified = match classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 256 },
    ) {
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
    use super::{VerifyError, run_parser_probes, verify_candidate, verify_compiler_grammar};
    use whitefoot::ACTIVE_KERNEL_SPEC_BYTES;

    /// The compiler's own grammar data, checked directly. This used to run
    /// through `verify_candidate(ACTIVE, ACTIVE)`, whose contract comparison
    /// could not fail; the counts and the probes were always the real content.
    #[test]
    fn active_compiler_grammar_is_consistent() {
        let report = verify_compiler_grammar().expect("compiler grammar data must be consistent");
        assert_eq!(report.productions, 69);
        assert_eq!(report.decisions, 84);
        assert_eq!(report.terminals, 93);
        run_parser_probes().expect("the compiler must parse its own probes");
    }

    #[test]
    fn prose_outside_the_frontend_contract_may_change() {
        let mut proposal = ACTIVE_KERNEL_SPEC_BYTES.to_vec();
        proposal.extend_from_slice(b"\nSemantic-only proposal text.\n");
        verify_candidate(ACTIVE_KERNEL_SPEC_BYTES, &proposal)
            .expect("semantic-only text must preserve the grammar");
    }

    /// The comparison is between the two arguments, not against the compiled-in
    /// active bytes. Two files sharing a frontend contract the active
    /// specification does not have must verify against each other, and the
    /// active specification must then fail against that baseline.
    #[test]
    fn the_baseline_argument_is_the_one_compared() {
        let active = std::str::from_utf8(ACTIVE_KERNEL_SPEC_BYTES).expect("active spec is UTF-8");
        let changed = active.replacen(
            "return_stmt := \"return\" expr \";\"",
            "return_stmt := \"return\" atom \";\"",
            1,
        );
        assert_ne!(changed, active);
        verify_candidate(changed.as_bytes(), changed.as_bytes())
            .expect("a candidate matching its own baseline must verify");
        assert!(matches!(
            verify_candidate(changed.as_bytes(), ACTIVE_KERNEL_SPEC_BYTES),
            Err(VerifyError::ChangedFrontendContract)
        ));
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
            verify_candidate(ACTIVE_KERNEL_SPEC_BYTES, changed.as_bytes()),
            Err(VerifyError::ChangedFrontendContract)
        ));
    }

    #[test]
    fn changed_effect_order_fails_closed() {
        let active = std::str::from_utf8(ACTIVE_KERNEL_SPEC_BYTES).expect("active spec is UTF-8");
        let changed = active.replacen(
            "| \"external\" | \"blocks\" | \"traps\"",
            "| \"blocks\" | \"external\" | \"traps\"",
            1,
        );
        assert_ne!(changed, active);
        assert!(matches!(
            verify_candidate(ACTIVE_KERNEL_SPEC_BYTES, changed.as_bytes()),
            Err(VerifyError::ChangedFrontendContract)
        ));
    }

    #[test]
    fn changed_input_label_spelling_fails_closed() {
        let active = std::str::from_utf8(ACTIVE_KERNEL_SPEC_BYTES).expect("active spec is UTF-8");
        let changed = active.replacen(
            "input_label  := IDENT \".\" IDENT \"as\"",
            "input_label  := IDENT \".\" IDENT \"from\"",
            1,
        );
        assert_ne!(changed, active);
        assert!(matches!(
            verify_candidate(ACTIVE_KERNEL_SPEC_BYTES, changed.as_bytes()),
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
            verify_candidate(ACTIVE_KERNEL_SPEC_BYTES, changed.as_bytes()),
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
            verify_candidate(ACTIVE_KERNEL_SPEC_BYTES, changed.as_bytes()),
            Err(VerifyError::ChangedFrontendContract)
        ));
    }
}
