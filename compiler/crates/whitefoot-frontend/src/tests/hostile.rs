use whitefoot_contract::SourceId;

use super::{bundle, complete, generous_limits};
use crate::{LexOutcome, Lexeme, SourceIssueKind, TokenKind};

fn issue(source: &[u8]) -> (SourceIssueKind, u64, u64) {
    let source = bundle(&[("bad.wf", source)]);
    match crate::lex_v0_8(&source, generous_limits()) {
        LexOutcome::SourceIssue(issue) => (
            issue.kind(),
            issue.span().start().value(),
            issue.span().end().value(),
        ),
        outcome => panic!("expected source issue, got {outcome:?}"),
    }
}

#[test]
fn invalid_bytes_and_comment_markers_fail_without_a_partial_tape() {
    for (source, expected) in [
        (b"ok $ bad".as_slice(), SourceIssueKind::UnexpectedByte),
        (b"ok // no".as_slice(), SourceIssueKind::UnexpectedByte),
        (b"ok\tbad".as_slice(), SourceIssueKind::UnexpectedByte),
        (b"ok\rbad".as_slice(), SourceIssueKind::UnexpectedByte),
        (b"ok \xff bad".as_slice(), SourceIssueKind::InvalidUtf8),
    ] {
        assert_eq!(issue(source).0, expected, "source={source:?}");
    }
}

#[test]
fn malformed_prefixed_names_report_the_marker() {
    assert_eq!(issue(b"'"), (SourceIssueKind::MissingRegionName, 0, 1));
    assert_eq!(issue(b"'Upper"), (SourceIssueKind::MissingRegionName, 0, 1));
    assert_eq!(issue(b"@"), (SourceIssueKind::MissingLabelName, 0, 1));
    assert_eq!(issue(b"@9"), (SourceIssueKind::MissingLabelName, 0, 1));
}

#[test]
fn strings_reject_unknown_escapes_raw_controls_unicode_and_eof() {
    assert_eq!(issue(br#""bad\t""#).0, SourceIssueKind::InvalidStringEscape);
    assert_eq!(issue(b"\"bad\n\"").0, SourceIssueKind::InvalidStringByte);
    assert_eq!(
        issue("\"snowman ☃\"".as_bytes()).0,
        SourceIssueKind::InvalidStringByte
    );
    assert_eq!(
        issue(b"\"unterminated").0,
        SourceIssueKind::UnterminatedString
    );
    assert_eq!(issue(b"\"bad \\ ").0, SourceIssueKind::InvalidStringEscape);
}

#[test]
fn noncanonical_spacing_and_terminal_lfs_remain_complete_lexical_inputs() {
    for source in [
        b"fn  name".as_slice(),
        b"fn name\n\n".as_slice(),
        b"fn name".as_slice(),
        b"\n\n\n".as_slice(),
        b"".as_slice(),
    ] {
        let source = bundle(&[("layout.wf", source)]);
        assert!(matches!(
            crate::lex_v0_8(&source, generous_limits()),
            LexOutcome::Complete(_)
        ));
    }
}

#[test]
fn source_boundaries_prevent_cross_file_tokens() {
    let source = bundle(&[("one.wf", b"foo"), ("two.wf", b".wrap")]);
    let lexed = complete(&source);
    let first = lexed.source_lexemes(SourceId::from_ordinal(0)).unwrap();
    let second = lexed.source_lexemes(SourceId::from_ordinal(1)).unwrap();
    assert!(matches!(
        first,
        [Lexeme::Token(token)] if token.kind() == TokenKind::LowerWordForm
    ));
    assert!(matches!(
        second,
        [Lexeme::Token(dot), Lexeme::Token(word)]
            if dot.kind() == TokenKind::Dot && word.kind() == TokenKind::LowerWordForm
    ));
}

#[test]
fn every_complete_source_is_reconstructible_byte_for_byte() {
    let inputs = [
        ("empty.wf", b"".as_slice()),
        (
            "forms.wf",
            b"fn  x('r: &uniq 'r T) -> own unit\n".as_slice(),
        ),
        ("strings.wf", br#"doc "a\\b\n";"#.as_slice()),
    ];
    let source = bundle(&inputs);
    let lexed = complete(&source);
    for (source_id, file) in source.iter() {
        let mut cursor = 0_u64;
        let mut rebuilt = Vec::new();
        for lexeme in lexed.source_lexemes(source_id).unwrap() {
            let span = lexeme.span();
            assert_eq!(span.source(), source_id);
            assert_eq!(span.start().value(), cursor);
            assert!(span.end() > span.start());
            assert!(span.end().value() <= file.byte_len());
            cursor = span.end().value();
            rebuilt.extend_from_slice(span.bytes());
        }
        assert_eq!(cursor, file.byte_len());
        assert_eq!(rebuilt, file.bytes());
    }
}

#[test]
fn repeated_scans_are_deterministic() {
    let source = bundle(&[("repeat.wf", b"@label p.field iadd.checked -1_i64\n")]);
    let render = |lexed: &crate::LexedBundle<'_>| {
        lexed
            .lexemes()
            .iter()
            .map(|lexeme| {
                let kind = match lexeme {
                    Lexeme::Token(token) => format!("t:{:?}", token.kind()),
                    Lexeme::Trivia(trivia) => format!("v:{:?}", trivia.kind()),
                };
                (
                    lexeme.span().start().value(),
                    lexeme.span().end().value(),
                    kind,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(render(&complete(&source)), render(&complete(&source)));
}

#[test]
fn every_single_top_level_byte_has_a_controlled_lossless_outcome() {
    for byte in 0_u8..=u8::MAX {
        let bytes = [byte];
        let source = bundle(&[("byte.wf", &bytes)]);
        let expected_complete = byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b' ' | b'\n'
                    | b'('
                    | b')'
                    | b'{'
                    | b'}'
                    | b'['
                    | b']'
                    | b'<'
                    | b'>'
                    | b','
                    | b':'
                    | b';'
                    | b'.'
                    | b'='
                    | b'&'
            );
        match (
            expected_complete,
            crate::lex_v0_8(&source, generous_limits()),
        ) {
            (true, LexOutcome::Complete(lexed)) => {
                let rebuilt: Vec<_> = lexed
                    .lexemes()
                    .iter()
                    .flat_map(|lexeme| lexeme.span().bytes().iter().copied())
                    .collect();
                assert_eq!(rebuilt, bytes, "byte=0x{byte:02x}");
            }
            (false, LexOutcome::SourceIssue(_)) => {}
            (expected, outcome) => {
                panic!("wrong byte outcome for 0x{byte:02x}; complete={expected}: {outcome:?}");
            }
        }
    }
}

#[test]
fn every_single_string_interior_byte_has_a_controlled_outcome() {
    for byte in 0_u8..=u8::MAX {
        let bytes = [b'"', byte, b'"'];
        let source = bundle(&[("string-byte.wf", &bytes)]);
        let expected_complete = matches!(byte, 0x20..=0x7e) && !matches!(byte, b'"' | b'\\');
        match (
            expected_complete,
            crate::lex_v0_8(&source, generous_limits()),
        ) {
            (true, LexOutcome::Complete(lexed)) => {
                let rebuilt: Vec<_> = lexed
                    .lexemes()
                    .iter()
                    .flat_map(|lexeme| lexeme.span().bytes().iter().copied())
                    .collect();
                assert_eq!(rebuilt, bytes, "byte=0x{byte:02x}");
            }
            (false, LexOutcome::SourceIssue(_)) => {}
            (expected, outcome) => {
                panic!("wrong string outcome for 0x{byte:02x}; complete={expected}: {outcome:?}");
            }
        }
    }
}

#[test]
fn each_closed_string_escape_is_a_complete_exact_partition() {
    for bytes in [
        b"\"\\\\\"".as_slice(),
        b"\"\\\"\"".as_slice(),
        b"\"\\n\"".as_slice(),
    ] {
        let source = bundle(&[("escape.wf", bytes)]);
        let lexed = complete(&source);
        assert!(matches!(
            lexed.lexemes(),
            [Lexeme::Token(token)] if token.kind() == TokenKind::StringForm
        ));
        assert_eq!(lexed.lexemes()[0].span().bytes(), bytes);
    }
}

#[test]
fn numeric_candidates_are_never_converted_or_canonicalized() {
    let mut huge = vec![b'9'; 16_384];
    huge.extend_from_slice(b"_i64");
    let inputs = [
        b"-2147483648_i32".as_slice(),
        b"-0_i32".as_slice(),
        b"01_i32".as_slice(),
        b"0_T".as_slice(),
        b"1_T".as_slice(),
        b"42".as_slice(),
        b"1.00_f64".as_slice(),
        b"1.0e-999999_f64".as_slice(),
        b"1.0E2_f64".as_slice(),
        b"1._f64".as_slice(),
        b"1.0e_f64".as_slice(),
        b"1.0e+2_f64".as_slice(),
        huge.as_slice(),
    ];
    for bytes in inputs {
        let source = bundle(&[("number.wf", bytes)]);
        let lexed = complete(&source);
        assert!(matches!(
            lexed.lexemes(),
            [Lexeme::Token(token)] if token.kind() == TokenKind::NumberForm
        ));
        assert_eq!(lexed.lexemes()[0].span().bytes(), bytes);
    }
}

#[test]
fn failures_and_tokens_never_continue_across_source_boundaries() {
    for inputs in [
        [("one.wf", b"-".as_slice()), ("two.wf", b"1_i64".as_slice())],
        [
            ("one.wf", b"\"open".as_slice()),
            ("two.wf", b"\"".as_slice()),
        ],
        [
            ("one.wf", b"foo.".as_slice()),
            ("two.wf", b"wrap".as_slice()),
        ],
    ] {
        let source = bundle(&inputs);
        match crate::lex_v0_8(&source, generous_limits()) {
            LexOutcome::SourceIssue(issue) => {
                assert_eq!(issue.span().source(), SourceId::from_ordinal(0));
            }
            LexOutcome::Complete(lexed) => {
                assert!(lexed.source_lexemes(SourceId::from_ordinal(0)).is_some());
                assert!(lexed.source_lexemes(SourceId::from_ordinal(1)).is_some());
            }
            outcome => panic!("unexpected boundary outcome: {outcome:?}"),
        }
    }
}

#[test]
fn source_issue_selection_follows_bundle_then_byte_order() {
    let source = bundle(&[("first.wf", b"ok $ \xff"), ("second.wf", b"\xff")]);
    match crate::lex_v0_8(&source, generous_limits()) {
        LexOutcome::SourceIssue(issue) => {
            assert_eq!(issue.span().source(), SourceId::from_ordinal(0));
            assert_eq!(issue.span().start().value(), 3);
            assert_eq!(issue.kind(), SourceIssueKind::UnexpectedByte);
        }
        outcome => panic!("expected ordered source issue, got {outcome:?}"),
    }
}
