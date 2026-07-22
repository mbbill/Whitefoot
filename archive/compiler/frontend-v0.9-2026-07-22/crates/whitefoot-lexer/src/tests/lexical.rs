use whitefoot_contract::SourceId;

use super::{bundle, complete};
use crate::{Lexeme, TokenKind, TriviaKind};

fn observed(source: &[u8]) -> Vec<(Vec<u8>, String)> {
    let source = bundle(&[("case.wf", source)]);
    complete(&source)
        .lexemes()
        .iter()
        .map(|lexeme| {
            let label = match lexeme {
                Lexeme::Token(token) => format!("token:{:?}", token.kind()),
                Lexeme::Trivia(trivia) => format!("trivia:{:?}", trivia.kind()),
            };
            (lexeme.span().bytes().to_vec(), label)
        })
        .collect()
}

#[test]
fn recognizes_every_shape_without_keyword_or_operation_resolution() {
    let source = b"fn Thing 'region @again foo.wrap p.field -1_i64 6.022e-23_f64 unit\n";
    let observed = observed(source);
    assert_eq!(
        observed,
        [
            (b"fn".to_vec(), "token:LowerWordForm".into()),
            (b" ".to_vec(), "trivia:Spaces".into()),
            (b"Thing".to_vec(), "token:UpperWordForm".into()),
            (b" ".to_vec(), "trivia:Spaces".into()),
            (b"'region".to_vec(), "token:RegionForm".into()),
            (b" ".to_vec(), "trivia:Spaces".into()),
            (b"@again".to_vec(), "token:LabelForm".into()),
            (b" ".to_vec(), "trivia:Spaces".into()),
            (b"foo.wrap".to_vec(), "token:OperationNameForm".into()),
            (b" ".to_vec(), "trivia:Spaces".into()),
            (b"p".to_vec(), "token:LowerWordForm".into()),
            (b".".to_vec(), "token:Dot".into()),
            (b"field".to_vec(), "token:LowerWordForm".into()),
            (b" ".to_vec(), "trivia:Spaces".into()),
            (b"-1_i64".to_vec(), "token:NumberForm".into()),
            (b" ".to_vec(), "trivia:Spaces".into()),
            (b"6.022e-23_f64".to_vec(), "token:NumberForm".into()),
            (b" ".to_vec(), "trivia:Spaces".into()),
            (b"unit".to_vec(), "token:LowerWordForm".into()),
            (b"\n".to_vec(), "trivia:LineFeed".into()),
        ]
    );
}

#[test]
fn dotted_mode_boundary_is_exact_and_dotless_ops_remain_words() {
    let observed = observed(b"p.wrap p.wrapx p.checked_more foo.checked iadd\n");
    let tokens: Vec<_> = observed
        .iter()
        .filter(|(_, label)| label.starts_with("token:"))
        .cloned()
        .collect();
    assert_eq!(
        tokens,
        [
            (b"p.wrap".to_vec(), "token:OperationNameForm".into()),
            (b"p".to_vec(), "token:LowerWordForm".into()),
            (b".".to_vec(), "token:Dot".into()),
            (b"wrapx".to_vec(), "token:LowerWordForm".into()),
            (b"p".to_vec(), "token:LowerWordForm".into()),
            (b".".to_vec(), "token:Dot".into()),
            (b"checked_more".to_vec(), "token:LowerWordForm".into()),
            (b"foo.checked".to_vec(), "token:OperationNameForm".into()),
            (b"iadd".to_vec(), "token:LowerWordForm".into()),
        ]
    );
}

#[test]
fn all_fixed_symbols_preserve_longest_two_byte_forms() {
    let observed = observed(b"(){}[]<>,:;.= -> => &\n");
    let kinds: Vec<_> = observed
        .iter()
        .filter_map(|(_, label)| label.strip_prefix("token:"))
        .collect();
    assert_eq!(
        kinds,
        [
            "LeftParen",
            "RightParen",
            "LeftBrace",
            "RightBrace",
            "LeftBracket",
            "RightBracket",
            "LeftAngle",
            "RightAngle",
            "Comma",
            "Colon",
            "Semicolon",
            "Dot",
            "Equal",
            "ThinArrow",
            "FatArrow",
            "Ampersand",
        ]
    );
}

#[test]
fn strings_retain_quotes_and_exact_escape_bytes() {
    let observed = observed(br#""raw printable" "slash:\\ quote:\" newline:\n""#);
    let strings: Vec<_> = observed
        .into_iter()
        .filter(|(_, label)| label == "token:StringForm")
        .map(|(bytes, _)| bytes)
        .collect();
    assert_eq!(
        strings,
        [
            br#""raw printable""#.to_vec(),
            br#""slash:\\ quote:\" newline:\n""#.to_vec(),
        ]
    );
}

#[test]
fn token_identity_is_source_bound_and_coordinate_visible() {
    let first_bundle = bundle(&[("same.wf", b"name")]);
    let first = complete(&first_bundle);
    let id = match first.lexemes()[0] {
        Lexeme::Token(token) => token.id(),
        Lexeme::Trivia(_) => panic!("expected token"),
    };
    assert_eq!(id.source(), SourceId::from_ordinal(0));
    assert_eq!(id.start().value(), 0);
    assert_eq!(id.end().value(), 4);
    assert_eq!(id.span().bytes(), b"name");
}

#[test]
fn equal_coordinates_in_different_bundles_remain_distinct_handles() {
    let first_bundle = bundle(&[("first.wf", b"fn")]);
    let second_bundle = bundle(&[("second.wf", b"xx")]);
    let first = complete(&first_bundle);
    let second = complete(&second_bundle);
    let first_id = match first.lexemes()[0] {
        Lexeme::Token(token) => token.id(),
        Lexeme::Trivia(_) => panic!("expected first token"),
    };
    let second_id = match second.lexemes()[0] {
        Lexeme::Token(token) => token.id(),
        Lexeme::Trivia(_) => panic!("expected second token"),
    };
    assert_eq!(first_id.source(), second_id.source());
    assert_eq!(first_id.start(), second_id.start());
    assert_eq!(first_id.end(), second_id.end());
    assert_eq!(first_id.span().bytes(), b"fn");
    assert_eq!(second_id.span().bytes(), b"xx");
    assert_eq!(first_id.span().file().logical_path().as_str(), "first.wf");
    assert_eq!(second_id.span().file().logical_path().as_str(), "second.wf");
}

#[test]
fn token_and_trivia_kinds_are_shape_only() {
    let source = bundle(&[("shape.wf", b"requires iadd 000_i8\n")]);
    let lexed = complete(&source);
    assert!(matches!(
        lexed.lexemes()[0],
        Lexeme::Token(token) if token.kind() == TokenKind::LowerWordForm
    ));
    assert!(matches!(
        lexed.lexemes()[1],
        Lexeme::Trivia(trivia) if trivia.kind() == TriviaKind::Spaces
    ));
    assert!(matches!(
        lexed.lexemes()[2],
        Lexeme::Token(token) if token.kind() == TokenKind::LowerWordForm
    ));
    assert!(matches!(
        lexed.lexemes()[4],
        Lexeme::Token(token) if token.kind() == TokenKind::NumberForm
    ));
}

#[test]
fn every_closed_operation_suffix_and_near_miss_has_the_expected_shape() {
    for suffix in ["wrap", "trap", "checked", "sat", "strict"] {
        let exact = format!("base.{suffix}");
        let source = bundle(&[("exact.wf", exact.as_bytes())]);
        assert!(matches!(
            complete(&source).lexemes(),
            [Lexeme::Token(token)] if token.kind() == TokenKind::OperationNameForm
        ));

        for near_miss in [format!("base.{suffix}x"), format!("base.x{suffix}")] {
            let source = bundle(&[("near.wf", near_miss.as_bytes())]);
            let lexed = complete(&source);
            assert!(matches!(
                lexed.lexemes(),
                [Lexeme::Token(base), Lexeme::Token(dot), Lexeme::Token(mode)]
                    if base.kind() == TokenKind::LowerWordForm
                        && dot.kind() == TokenKind::Dot
                        && mode.kind() == TokenKind::LowerWordForm
            ));
        }
    }
}

#[test]
fn arrow_and_borrow_seams_do_not_change_neighboring_shapes() {
    let observed = observed(b"-1_i64 -> => &'r &uniq 'r\n");
    let tokens: Vec<_> = observed
        .into_iter()
        .filter(|(_, label)| label.starts_with("token:"))
        .collect();
    assert_eq!(
        tokens,
        [
            (b"-1_i64".to_vec(), "token:NumberForm".into()),
            (b"->".to_vec(), "token:ThinArrow".into()),
            (b"=>".to_vec(), "token:FatArrow".into()),
            (b"&".to_vec(), "token:Ampersand".into()),
            (b"'r".to_vec(), "token:RegionForm".into()),
            (b"&".to_vec(), "token:Ampersand".into()),
            (b"uniq".to_vec(), "token:LowerWordForm".into()),
            (b"'r".to_vec(), "token:RegionForm".into()),
        ]
    );
}
