#![allow(clippy::panic)]

use whitefoot_contract::{
    KERNEL_SPEC_V0_9_HASH, SourceBundle, SourceId, SourceInput, SourceLimits,
};
use whitefoot_lexer::{LexLimits, LexOutcome, lex_v0_9};
use whitefoot_syntax_data::{ProductionV0_9, productions_v0_9};

use crate::{TerminalLimits, TerminalOutcome, classify_terminals_v0_9};

use super::finalize::{FinalizeLimits, FinalizeOutcome, finalize_v0_9};
use super::{
    DerivationElement, ParseInvocationFailure, ParseLimit, ParseLimits, ParseOutcome,
    ParseResourceFailure, SyntaxRuleV0_9, parse_v0_9,
};

const SOURCE_LIMITS: SourceLimits = SourceLimits {
    max_sources: 16,
    max_logical_path_bytes: 128,
    max_source_bytes: 65_536,
    max_total_source_bytes: 262_144,
    max_binding_bytes: 524_288,
};

const LEX_LIMITS: LexLimits = LexLimits {
    max_sources: 16,
    max_source_bytes: 65_536,
    max_total_source_bytes: 262_144,
    max_token_bytes: 4_096,
    max_tokens: 65_536,
    max_lexemes: 131_072,
};

const PARSE_LIMITS: ParseLimits = ParseLimits {
    max_work: 4_000_000,
    max_tasks: 65_536,
    max_frames: 4_096,
    max_elements: 131_072,
};

fn bundle(inputs: &[SourceInput<'_>]) -> SourceBundle {
    let Ok(bundle) = SourceBundle::with_limits(inputs, SOURCE_LIMITS) else {
        panic!("test source bundle must be valid");
    };
    bundle
}

#[test]
fn minimal_function_and_multi_source_items_form_one_program_root() {
    let inputs = [
        SourceInput::new("one.wf", b"fn main() -> own unit pure {}"),
        SourceInput::new("two.wf", b"const answer: i32 = 42_i32;"),
    ];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 128 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse_v0_9(&classified, PARSE_LIMITS) else {
        panic!("minimal multi-source program must parse");
    };
    assert_eq!(parsed.top_level_item_count(), Some(2));
    assert_eq!(parsed.terminal_count(), classified.tokens().len() as u64);
    assert_eq!(parsed.classified_bundle().source_bundle().len(), 2);
}

#[test]
fn ordered_sources_report_the_first_invalid_record() {
    let inputs = [
        SourceInput::new("first.wf", b"fn main() -> own unit pure {}"),
        SourceInput::new("second.wf", b"unknown value"),
        SourceInput::new(
            "third.wf",
            b"fn later() -> own unit pure { object.member(); }",
        ),
    ];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("ordered source fixture must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 64 },
    ) else {
        panic!("ordered source fixture must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse_v0_9(&classified, PARSE_LIMITS) else {
        panic!("the first invalid source record must reject");
    };
    assert_eq!(issue.rule(), SyntaxRuleV0_9::Form1);
    assert_eq!(issue.coordinate().source(), SourceId::from_ordinal(1));
}

#[test]
fn shared_prefix_expression_forms_select_without_priority_or_backtracking() {
    let source = br#"
struct Value { field: i32; }
enum Choice { Some(value: i32); }
fn main() -> own unit pure {
let atom: own i32 = 0_i32;
let positional: own i32 = user(atom);
let named: own i32 = user(arg: atom);
let generic: own i32 = user<i32>(atom);
let made: own Value = Value(field: atom);
let selected: own i32 = match atom { Some(value: payload) => { give payload; } }
return unit;
}
"#;
    let inputs = [SourceInput::new("prefixes.wf", source)];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("shared-prefix fixture must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 256 },
    ) else {
        panic!("shared-prefix fixture must classify");
    };
    let outcome = parse_v0_9(&classified, PARSE_LIMITS);
    assert!(
        matches!(outcome, ParseOutcome::Complete(_)),
        "every shared-prefix form must parse deterministically: {outcome:?}"
    );
}

#[test]
fn one_empty_record_derives_before_the_later_form2_audit() {
    let inputs = [SourceInput::new("empty.wf", b"")];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("empty source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 0 },
    ) else {
        panic!("empty source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse_v0_9(&classified, PARSE_LIMITS) else {
        panic!("empty item sequence must derive");
    };
    assert_eq!(parsed.top_level_item_count(), Some(0));
    assert_eq!(parsed.production_count(), 1);
    assert_eq!(parsed.element_count(), 1);
}

#[test]
fn unknown_ident_construct_uses_closed_form1_override() {
    let inputs = [SourceInput::new("unknown.wf", b"mystery value")];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 2 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse_v0_9(&classified, PARSE_LIMITS) else {
        panic!("unknown construct must be a source issue");
    };
    assert_eq!(issue.rule(), SyntaxRuleV0_9::Form1);
    assert_eq!(issue.coordinate().source(), SourceId::from_ordinal(0));
    assert_eq!(issue.coordinate().start().value(), 0);
    assert_eq!(issue.coordinate().end().value(), 7);
}

#[test]
fn dotted_call_spelling_uses_bounded_form3_override() {
    let source = b"fn main() -> own unit pure { object.member(); }";
    let inputs = [SourceInput::new("dotted.wf", source)];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 32 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse_v0_9(&classified, PARSE_LIMITS) else {
        panic!("dotted call spelling must be rejected");
    };
    assert_eq!(issue.rule(), SyntaxRuleV0_9::Form3);
    let start = usize::try_from(issue.coordinate().start().value()).unwrap_or(usize::MAX);
    let end = usize::try_from(issue.coordinate().end().value()).unwrap_or(usize::MAX);
    assert_eq!(&source[start..end], b"object.member");
}

#[test]
fn nested_call_in_atom_only_argument_uses_gram9_override() {
    let source = b"fn main() -> own unit pure { outer(inner()); }";
    let inputs = [SourceInput::new("nested.wf", source)];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 32 },
    ) else {
        panic!("test source must classify");
    };
    let outcome = parse_v0_9(&classified, PARSE_LIMITS);
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("nested call must be rejected: {outcome:?}");
    };
    assert_eq!(issue.rule(), SyntaxRuleV0_9::Gram9);
    let start = usize::try_from(issue.coordinate().start().value()).unwrap_or(usize::MAX);
    let end = usize::try_from(issue.coordinate().end().value()).unwrap_or(usize::MAX);
    assert_eq!(&source[start..end], b"inner(");
}

#[test]
fn mandatory_name_and_numeric_pattern_mismatches_keep_their_owners() {
    for (source, expected_rule) in [
        (
            b"fn struct() -> own unit pure {}".as_slice(),
            SyntaxRuleV0_9::Form3,
        ),
        (
            b"const value: array<i32, 1_i32> = [0_i32];".as_slice(),
            SyntaxRuleV0_9::Const1,
        ),
        (b"const value: i32 = 42;".as_slice(), SyntaxRuleV0_9::Form5),
    ] {
        let inputs = [SourceInput::new("owner.wf", source)];
        let bundle = bundle(&inputs);
        let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
            panic!("test source must lex");
        };
        let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
            &lexed,
            KERNEL_SPEC_V0_9_HASH,
            TerminalLimits { max_tokens: 64 },
        ) else {
            panic!("test source must classify");
        };
        let outcome = parse_v0_9(&classified, PARSE_LIMITS);
        let ParseOutcome::SourceIssue(issue) = outcome else {
            panic!("name or numeric mismatch must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), expected_rule, "source: {source:?}");
    }
}

#[test]
fn non_ident_program_leftover_expects_only_source_end() {
    let inputs = [SourceInput::new("leftover.wf", b"return unit;")];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 8 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse_v0_9(&classified, PARSE_LIMITS) else {
        panic!("top-level statement must reject");
    };
    assert_eq!(issue.rule(), SyntaxRuleV0_9::Gram2);
    assert_eq!(issue.expected().len(), 1);
    assert!(
        issue
            .expected()
            .contains(whitefoot_syntax_data::LookaheadPredicateV0_9::SourceEnd)
    );
}

#[test]
fn element_limit_is_explicit_and_failure_atomic() {
    let inputs = [SourceInput::new(
        "main.wf",
        b"fn main() -> own unit pure {}",
    )];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 16 },
    ) else {
        panic!("test source must classify");
    };
    let limits = ParseLimits {
        max_elements: 0,
        ..PARSE_LIMITS
    };
    let ParseOutcome::ResourceFailure(ParseResourceFailure::LimitExceeded {
        limit: ParseLimit::Elements,
        maximum: 0,
        actual: 1,
    }) = parse_v0_9(&classified, limits)
    else {
        panic!("first element must hit the exact element ceiling");
    };
}

#[test]
fn envelope_and_each_control_stack_limit_are_distinct() {
    let no_inputs: [SourceInput<'_>; 0] = [];
    let empty_bundle = bundle(&no_inputs);
    let LexOutcome::Complete(empty_lexed) = lex_v0_9(&empty_bundle, LEX_LIMITS) else {
        panic!("empty transport must lex as an envelope candidate");
    };
    let TerminalOutcome::Complete(empty_classified) = classify_terminals_v0_9(
        &empty_lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 0 },
    ) else {
        panic!("empty transport must classify as an envelope candidate");
    };
    assert!(matches!(
        parse_v0_9(&empty_classified, PARSE_LIMITS),
        ParseOutcome::InvocationFailure(ParseInvocationFailure::EmptySourceBundle)
    ));

    let inputs = [SourceInput::new(
        "main.wf",
        b"fn main() -> own unit pure {}",
    )];
    let source_bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&source_bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 16 },
    ) else {
        panic!("test source must classify");
    };
    for (limits, expected_limit) in [
        (
            ParseLimits {
                max_work: 0,
                ..PARSE_LIMITS
            },
            ParseLimit::Work,
        ),
        (
            ParseLimits {
                max_tasks: 0,
                ..PARSE_LIMITS
            },
            ParseLimit::Tasks,
        ),
        (
            ParseLimits {
                max_frames: 0,
                ..PARSE_LIMITS
            },
            ParseLimit::Frames,
        ),
    ] {
        let ParseOutcome::ResourceFailure(ParseResourceFailure::LimitExceeded { limit, .. }) =
            parse_v0_9(&classified, limits)
        else {
            panic!("each zero control ceiling must fail explicitly");
        };
        assert_eq!(limit, expected_limit);
    }
}

#[test]
fn sufficient_resource_profiles_produce_identical_derivation_metrics() {
    let inputs = [SourceInput::new(
        "main.wf",
        b"fn main() -> own unit pure { let x: own unit = unit; return x; }",
    )];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 64 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::Complete(first) = parse_v0_9(&classified, PARSE_LIMITS) else {
        panic!("first sufficient profile must parse");
    };
    let larger = ParseLimits {
        max_work: PARSE_LIMITS.max_work * 2,
        max_tasks: PARSE_LIMITS.max_tasks * 2,
        max_frames: PARSE_LIMITS.max_frames * 2,
        max_elements: PARSE_LIMITS.max_elements * 2,
    };
    let ParseOutcome::Complete(second) = parse_v0_9(&classified, larger) else {
        panic!("second sufficient profile must parse");
    };
    assert_eq!(first.terminal_count(), second.terminal_count());
    assert_eq!(first.production_count(), second.production_count());
    assert_eq!(first.element_count(), second.element_count());
    assert_eq!(first.top_level_item_count(), second.top_level_item_count());
}

#[test]
fn complete_fixture_reaches_every_normative_production_kind() {
    let source = br#"
struct Types<T: Bound, const n: array<u8, 4>> {
doc "types";
a: i8; b: i16; c: i32; d: i64; e: u8; f: u16; g: u32; h: u64;
i: f32; j: f64; k: unit; l: Name<T, 'r, n>; m: array<u8, n>;
n: slice<'r, u8>; o: box<u8>; p: arena<'r, u8>; q: buffer<u8>;
}
enum Choice<T> { doc "choice"; None(); Some(value: T); }
contract Contract<T> {
doc "contract";
fn member['r](x: own T) -> own T reads('r), writes('r), allocates(heap arena 'r), traps;
law associative(member);
law identity(member, 0_i32);
}
conform Name<T>: Contract<T> { doc "binding"; member = implementation; }
const zero: i32 = 0_i32;
const alias: i32 = zero;
const table: array<i32, 2> = [0_i32, zero];
fn everything['r](x: own i32, shared: &'r i32, unique: &uniq 'r i32)
-> own unit reads('r), writes('r), allocates(heap arena 'r), traps
requires { let pre: own i32 = iadd.wrap(0_i32, 1_i32); check pre else trap "pre"; }
{
doc "body";
let ordinary: own i32 = iadd.wrap(0_i32, 1_i32);
let attempted: own i32 = try user(arg: ordinary);
let selected: own i32 = match ordinary { Some(value: payload) => { give payload; } }
let made: own Name<T> = Name<T>(value: ordinary);
let moved: own i32 = move ordinary;
let borrowed: &'r i32 = &'r ordinary;
let unique_borrow: &uniq 'r i32 = &uniq 'r ordinary;
let loaded: own i32 = index<i32>(table, ordinary);
set deref(pointer).field = ordinary;
user<T, 'r, 2>(arg: ordinary);
return unit;
loop @again { break @again; }
region 'inner { give ordinary; }
check ordinary else trap "check";
match ordinary { Some(value: payload) => { give payload; } }
}
fn main() -> own unit pure {}
"#;
    let inputs = [SourceInput::new("all.wf", source)];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("full fixture must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits { max_tokens: 65_536 },
    ) else {
        panic!("full fixture must classify");
    };
    let outcome = parse_v0_9(&classified, PARSE_LIMITS);
    let ParseOutcome::Complete(parsed) = outcome else {
        panic!("full fixture must parse: {outcome:?}");
    };
    for production in productions_v0_9() {
        let present = parsed.tree.elements.iter().any(|element| {
            matches!(
                element,
                DerivationElement::Production { production: actual, .. } if actual == production
            )
        });
        assert!(present, "fixture omitted {production:?}");
    }
    assert_eq!(productions_v0_9().len(), 62);
    assert_eq!(
        parsed
            .tree
            .elements
            .last()
            .and_then(|element| match element {
                DerivationElement::Production { production, .. } => Some(*production),
                DerivationElement::Terminal { .. } => None,
            }),
        Some(ProductionV0_9::Program)
    );
    let FinalizeOutcome::Complete(finalized) = finalize_v0_9(
        parsed,
        FinalizeLimits {
            max_work: 8_000_000,
            max_roots: 131_072,
            max_shape_tasks: 131_072,
            max_nodes: 131_072,
            max_child_edges: 131_072,
            max_terminals: 131_072,
            max_sources: 16,
        },
    ) else {
        panic!("the all-production derivation must pass the independent shape finalizer");
    };
    assert!(finalized.node_count() >= productions_v0_9().len());
}
