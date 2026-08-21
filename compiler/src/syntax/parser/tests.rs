#![allow(clippy::panic)]

use crate::lexer::{LexLimits, LexOutcome, lex};
use crate::syntax::NodeId;
use crate::syntax::grammar::{Production, productions};
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::{ACTIVE_KERNEL_SPEC_HASH, SourceBundle, SourceId, SourceInput, SourceLimits};

use crate::{TerminalLimits, TerminalOutcome, classify_terminals};

use super::finalize::{FinalizeLimits, FinalizeOutcome, finalize};
use super::{
    DerivationElement, ParseInvocationFailure, ParseLimit, ParseLimits, ParseOutcome,
    ParseResourceFailure, SyntaxRule, parse,
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
        SourceInput::new("one.wf", b"fn main() -> result: own unit pure {}"),
        SourceInput::new("two.wf", b"const answer: i32 = 42_i32;"),
    ];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 128 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("minimal multi-source program must parse");
    };
    assert_eq!(parsed.top_level_item_count(), Some(2));
    assert_eq!(parsed.terminal_count(), classified.tokens().len() as u64);
    assert_eq!(parsed.classified_bundle().source_bundle().len(), 2);
}

#[test]
fn ordered_sources_report_the_first_invalid_record() {
    let inputs = [
        SourceInput::new("first.wf", b"fn main() -> result: own unit pure {}"),
        SourceInput::new("second.wf", b"unknown value"),
        SourceInput::new(
            "third.wf",
            b"fn later() -> result: own unit pure { object.member(); }",
        ),
    ];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("ordered source fixture must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 64 },
    ) else {
        panic!("ordered source fixture must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse(&classified, PARSE_LIMITS) else {
        panic!("the first invalid source record must reject");
    };
    assert_eq!(issue.rule(), SyntaxRule::Form1);
    assert_eq!(issue.coordinate().source(), SourceId::from_ordinal(1));
}

#[test]
fn shared_prefix_expression_forms_select_without_priority_or_backtracking() {
    let source = br#"
struct Value { field: i32; }
enum Choice { Some(value: i32); }
fn main() -> result: own unit pure {
let atom = 0_i32;
let positional = user(atom);
let named = user(arg: atom);
let generic = user<i32>(atom);
let made = Value(field: atom);
let selected = match atom { Some(value: payload) => { give payload; } }
let infix = atom + positional;
let suffixed = made.field * named;
let compared = ile(atom, generic);
let chosen = if compared { give atom; } else { give named; }
return unit;
}
"#;
    let inputs = [SourceInput::new("prefixes.wf", source)];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("shared-prefix fixture must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 256 },
    ) else {
        panic!("shared-prefix fixture must classify");
    };
    let outcome = parse(&classified, PARSE_LIMITS);
    assert!(
        matches!(outcome, ParseOutcome::Complete(_)),
        "every shared-prefix form must parse deterministically: {outcome:?}"
    );
}

#[test]
fn one_empty_record_derives_before_the_later_form2_audit() {
    let inputs = [SourceInput::new("empty.wf", b"")];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("empty source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 0 },
    ) else {
        panic!("empty source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
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
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 2 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse(&classified, PARSE_LIMITS) else {
        panic!("unknown construct must be a source issue");
    };
    assert_eq!(issue.rule(), SyntaxRule::Form1);
    assert_eq!(issue.coordinate().source(), SourceId::from_ordinal(0));
    assert_eq!(issue.coordinate().start().value(), 0);
    assert_eq!(issue.coordinate().end().value(), 7);
}

#[test]
fn dotted_call_spelling_uses_bounded_form3_override() {
    let source = b"fn main() -> result: own unit pure { object.member(); }";
    let inputs = [SourceInput::new("dotted.wf", source)];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 32 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse(&classified, PARSE_LIMITS) else {
        panic!("dotted call spelling must be rejected");
    };
    assert_eq!(issue.rule(), SyntaxRule::Form3);
    let start = usize::try_from(issue.coordinate().start().value()).unwrap_or(usize::MAX);
    let end = usize::try_from(issue.coordinate().end().value()).unwrap_or(usize::MAX);
    assert_eq!(&source[start..end], b"object.member");
}

#[test]
fn nested_call_in_atom_only_argument_uses_gram9_override() {
    let source = b"fn main() -> result: own unit pure { outer(inner()); }";
    let inputs = [SourceInput::new("nested.wf", source)];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 32 },
    ) else {
        panic!("test source must classify");
    };
    let outcome = parse(&classified, PARSE_LIMITS);
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("nested call must be rejected: {outcome:?}");
    };
    assert_eq!(issue.rule(), SyntaxRule::Gram9);
    let start = usize::try_from(issue.coordinate().start().value()).unwrap_or(usize::MAX);
    let end = usize::try_from(issue.coordinate().end().value()).unwrap_or(usize::MAX);
    assert_eq!(&source[start..end], b"inner(");
}

#[test]
fn mandatory_name_and_numeric_pattern_mismatches_keep_their_owners() {
    for (source, expected_rule) in [
        (
            b"fn struct() -> result: own unit pure {}".as_slice(),
            SyntaxRule::Form3,
        ),
        (
            b"const value: array<i32, 1_i32> =[0_i32];".as_slice(),
            SyntaxRule::Const1,
        ),
        (b"const value: i32 = 42;".as_slice(), SyntaxRule::Form5),
    ] {
        let inputs = [SourceInput::new("owner.wf", source)];
        let bundle = bundle(&inputs);
        let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
            panic!("test source must lex");
        };
        let TerminalOutcome::Complete(classified) = classify_terminals(
            &lexed,
            ACTIVE_KERNEL_SPEC_HASH,
            TerminalLimits { max_tokens: 64 },
        ) else {
            panic!("test source must classify");
        };
        let outcome = parse(&classified, PARSE_LIMITS);
        let ParseOutcome::SourceIssue(issue) = outcome else {
            panic!("name or numeric mismatch must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), expected_rule, "source: {source:?}");
    }
}

#[test]
fn item_head_with_a_wrong_name_shape_is_a_name_slot_mismatch() {
    // [DIAG-1] row 5 applies only when the first actual token predicate
    // matches no consuming `item` row. `fn` and `enum` each do match one, so
    // the earlier name-slot row owns these frontiers and cites FORM-3 at the
    // offending name token.
    for (source, name) in [
        (
            b"fn Main() -> result: own unit pure {}".as_slice(),
            b"Main".as_slice(),
        ),
        (b"enum sign { Neg(); }".as_slice(), b"sign".as_slice()),
    ] {
        let inputs = [SourceInput::new("item-head.wf", source)];
        let bundle = bundle(&inputs);
        let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
            panic!("test source must lex");
        };
        let TerminalOutcome::Complete(classified) = classify_terminals(
            &lexed,
            ACTIVE_KERNEL_SPEC_HASH,
            TerminalLimits { max_tokens: 32 },
        ) else {
            panic!("test source must classify");
        };
        let outcome = parse(&classified, PARSE_LIMITS);
        let ParseOutcome::SourceIssue(issue) = outcome else {
            panic!("wrong name shape must reject: {source:?}");
        };
        assert_eq!(issue.rule(), SyntaxRule::Form3, "source: {source:?}");
        let start = usize::try_from(issue.coordinate().start().value()).unwrap_or(usize::MAX);
        let end = usize::try_from(issue.coordinate().end().value()).unwrap_or(usize::MAX);
        assert_eq!(&source[start..end], name, "source: {source:?}");
    }
}

#[test]
fn non_name_program_leftover_expects_only_source_end() {
    let inputs = [SourceInput::new("leftover.wf", b"42_i32")];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 8 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse(&classified, PARSE_LIMITS) else {
        panic!("top-level literal leftover must reject");
    };
    assert_eq!(issue.rule(), SyntaxRule::Gram2);
    assert_eq!(issue.expected().len(), 1);
    assert!(
        issue
            .expected()
            .contains(crate::syntax::grammar::LookaheadPredicate::SourceEnd)
    );
}

#[test]
fn fixed_word_program_leftover_is_a_grammar_shape_mismatch() {
    // Program kinds are now a closed fixed-terminal choice. A statement-only
    // word at top level therefore reaches the item grammar itself instead of
    // being mistaken for a reserved spelling in an IDENT kind slot.
    let inputs = [SourceInput::new("leftover.wf", b"return unit;")];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 8 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::SourceIssue(issue) = parse(&classified, PARSE_LIMITS) else {
        panic!("top-level statement must reject");
    };
    assert_eq!(issue.rule(), SyntaxRule::Gram2);
}

#[test]
fn element_limit_is_explicit_and_failure_atomic() {
    let inputs = [SourceInput::new(
        "main.wf",
        b"fn main() -> result: own unit pure {}",
    )];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
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
    }) = parse(&classified, limits)
    else {
        panic!("first element must hit the exact element ceiling");
    };
}

#[test]
fn envelope_and_each_control_stack_limit_are_distinct() {
    let no_inputs: [SourceInput<'_>; 0] = [];
    let empty_bundle = bundle(&no_inputs);
    let LexOutcome::Complete(empty_lexed) = lex(&empty_bundle, LEX_LIMITS) else {
        panic!("empty transport must lex as an envelope candidate");
    };
    let TerminalOutcome::Complete(empty_classified) = classify_terminals(
        &empty_lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 0 },
    ) else {
        panic!("empty transport must classify as an envelope candidate");
    };
    assert!(matches!(
        parse(&empty_classified, PARSE_LIMITS),
        ParseOutcome::InvocationFailure(ParseInvocationFailure::EmptySourceBundle)
    ));

    let inputs = [SourceInput::new(
        "main.wf",
        b"fn main() -> result: own unit pure {}",
    )];
    let source_bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&source_bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
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
            parse(&classified, limits)
        else {
            panic!("each zero control ceiling must fail explicitly");
        };
        assert_eq!(limit, expected_limit);
    }
}

#[test]
fn sufficient_limits_produce_identical_derivation_metrics() {
    let inputs = [SourceInput::new(
        "main.wf",
        b"fn main() -> result: own unit pure { let x = unit; return x; }",
    )];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 64 },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::Complete(first) = parse(&classified, PARSE_LIMITS) else {
        panic!("first sufficient limits must parse");
    };
    let larger = ParseLimits {
        max_work: PARSE_LIMITS.max_work * 2,
        max_tasks: PARSE_LIMITS.max_tasks * 2,
        max_frames: PARSE_LIMITS.max_frames * 2,
        max_elements: PARSE_LIMITS.max_elements * 2,
    };
    let ParseOutcome::Complete(second) = parse(&classified, larger) else {
        panic!("second sufficient limits must parse");
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
fn member['r](x: own T) -> result: own T reads('r), writes('r), allocates(heap arena 'r), traps;
law associative(member);
law identity(member, 0_i32);
}
conform Name<T>: Contract<T> { doc "binding"; member = implementation; }
const zero: i32 = 0_i32;
const alias: i32 = zero;
const table: array<i32, 2> =[0_i32, zero];
command fn entry(command.args as arguments: own i32, command.cwd as directory: own i32)
-> result: own unit external, blocks
{
return unit;
}
fn everything['r](x: own i32, shared: &'r i32, unique: &uniq 'r i32)
-> result: own unit reads('r), writes('r), allocates(heap arena 'r), traps
contract {
define pre = 0_i32 +wrap 1_i32;
define post = 0_i32 +wrap 1_i32;
requires pre;
ensures when Some(value: routed): routed;
}
{
doc "body";
let ordinary = 0_i32 +wrap 1_i32;
let attempted = propagate user(arg: ordinary);
let selected = match ordinary { Some(value: payload) => { give payload; } }
let made = Name<T>(value: ordinary);
let moved = move ordinary;
let borrowed = &'r ordinary;
let unique_borrow = &uniq 'r ordinary;
let loaded = table[ordinary];
let compared = ilt(ordinary, moved);
let chosen = if compared { give ordinary; } else { give moved; }
set deref(pointer).field = ordinary;
let previous = replace deref(pointer).field = ordinary;
user<T, 'r, 2>(arg: ordinary);
return unit;
loop @again { break @again; }
for @range index in 0_u64..1_u64 { break @range; }
region 'inner { give ordinary; }
let parser_flag = True();
claim parser_only: parser_flag because "premises: parser_flag is constructed as True() in this parser-only fixture\nderivation: normalization proves parser_flag without any written theorem authority\nconclusion: this occurrence is checker-proved and therefore semantically non-residual\nchecker gap: none; semantic checking must reject it after the parser derives ClaimStmt\nconsumers: no admissible consumer exists because this fixture covers parsing only";
let named = ordinary;
match ordinary { Some(value: payload) => { give payload; } }
if compared { let then_branch = ordinary; } else if chosen { break @again; } else { return unit; }
}
fn main() -> result: own unit pure {}
"#;
    let inputs = [SourceInput::new("all.wf", source)];
    let bundle = bundle(&inputs);
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("full fixture must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 65_536 },
    ) else {
        panic!("full fixture must classify");
    };
    let outcome = parse(&classified, PARSE_LIMITS);
    let ParseOutcome::Complete(parsed) = outcome else {
        panic!("full fixture must parse: {outcome:?}");
    };
    for production in productions() {
        let present = parsed.tree.elements.iter().any(|element| {
            matches!(
                element,
                DerivationElement::Production { production: actual, .. } if actual == production
            )
        });
        assert!(present, "fixture omitted {production:?}");
    }
    assert_eq!(productions().len(), 74);
    assert_eq!(
        parsed
            .tree
            .elements
            .last()
            .and_then(|element| match element {
                DerivationElement::Production { production, .. } => Some(*production),
                DerivationElement::Terminal { .. } => None,
            }),
        Some(Production::Program)
    );
    let FinalizeOutcome::Complete(finalized) = finalize(
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
    assert!(finalized.node_count() >= productions().len());
}

const KIND_DECLARING_ENTRY: &[u8] = b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus allocates(heap), external, blocks, traps {\n  return unit;\n}\n";

const EXTERNAL_EFFECT_ROW: &[u8] =
    b"fn probe() -> result: own unit external {\n  return unit;\n}\n";

const BLOCKS_EFFECT_ROW: &[u8] = b"fn probe() -> result: own unit blocks {\n  return unit;\n}\n";

const RESERVED_SPELLINGS_AS_IDENTIFIERS: &[u8] =
    b"fn external() -> result: own unit pure {\n  let as = blocks;\n  return unit;\n}\n";

const CLAIM_STATEMENT: &[u8] = b"fn probe() -> result: own unit traps {\n  let flag = True();\n  claim held: flag because \"premises: flag is constructed as True() in this parser-only fixture\\nderivation: normalization proves flag without any written theorem authority\\nconclusion: this occurrence is checker-proved and therefore semantically non-residual\\nchecker gap: none; semantic checking must reject it after the parser derives ClaimStmt\\nconsumers: no admissible consumer exists because this fixture covers parsing only\";\n  return unit;\n}\n";

const CLAIM_SPELLINGS_AS_IDENTIFIERS: &[u8] =
    b"fn probe() -> result: own unit pure {\n  let claim = 0_i32;\n  return unit;\n}\n";

const BODY_CHECK_STATEMENT: &[u8] =
    b"fn probe() -> result: own unit traps {\n  let flag = True();\n  check flag;\n  return unit;\n}\n";

const UNIFIED_CONTRACT: &[u8] = b"fn probe(value: own i32) -> result: own i32 pure contract {\n  define admitted = ieq(value, value);\n  requires admitted;\n  ensures ieq(result, value);\n} {\n  return value;\n}\n";

const COUNTED_RANGE_STATEMENT: &[u8] = b"fn probe(lower: own u64, upper: own u64) -> result: own unit pure {\n  for @range index in lower..upper {\n    break @range;\n  }\n  return unit;\n}\n";

fn parse_active(
    name: &'static str,
    source: &'static [u8],
) -> ParseOutcome<'static, 'static, 'static> {
    // Tests leak their small fixtures so the borrowed pipeline stays simple.
    let inputs = Box::leak(Box::new([SourceInput::new(name, source)]));
    let bundle = Box::leak(Box::new(bundle(inputs)));
    let LexOutcome::Complete(lexed) = lex(bundle, LEX_LIMITS) else {
        panic!("fixture must lex");
    };
    let lexed = Box::leak(Box::new(lexed));
    let TerminalOutcome::Complete(classified) = classify_terminals(
        lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits { max_tokens: 65_536 },
    ) else {
        panic!("fixture must classify");
    };
    let classified = Box::leak(Box::new(classified));
    parse(classified, PARSE_LIMITS)
}

#[test]
fn active_contract_parses_the_kind_declaring_entry() {
    let outcome = parse_active("entry.wf", KIND_DECLARING_ENTRY);
    let ParseOutcome::Complete(parsed) = outcome else {
        panic!("the active tables must derive the kind-declaring entry: {outcome:?}");
    };
    for production in [Production::ProgramKind, Production::InputLabel] {
        let present = parsed.tree.elements.iter().any(|element| {
            matches!(
                element,
                DerivationElement::Production { production: actual, .. } if *actual == production
            )
        });
        assert!(present, "derivation omitted {production:?}");
    }
}

#[test]
fn active_contract_parses_the_external_and_blocks_effect_rows() {
    for source in [EXTERNAL_EFFECT_ROW, BLOCKS_EFFECT_ROW] {
        let outcome = parse_active("effects.wf", source);
        assert!(
            matches!(outcome, ParseOutcome::Complete(_)),
            "the active tables must derive the system effect row: {outcome:?}"
        );
    }
}

#[test]
fn active_contract_parses_the_claim_statement() {
    let outcome = parse_active("claim.wf", CLAIM_STATEMENT);
    let ParseOutcome::Complete(parsed) = outcome else {
        panic!("the active tables must derive a claim statement: {outcome:?}");
    };
    let present = parsed.tree.elements.iter().any(|element| {
        matches!(
            element,
            DerivationElement::Production { production: actual, .. }
                if *actual == Production::ClaimStmt
        )
    });
    assert!(present, "derivation omitted ClaimStmt");
}

#[test]
fn active_contract_reserves_the_claim_spellings() {
    let outcome = parse_active("identifiers.wf", CLAIM_SPELLINGS_AS_IDENTIFIERS);
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("claim/because must be reserved spellings excluded from IDENT: {outcome:?}");
    };
    assert_eq!(issue.rule(), SyntaxRule::Form3);
}

/// Check dissolution (#47): `check_stmt` left the [GRAM-4] `stmt`
/// alternation, so a body `check` no longer selects any statement and the
/// parser rejects it as an unknown statement construct under [FORM-1].
/// `claim` [CLM-1] is the sole
/// writer-stated trap construct a body may spell.
#[test]
fn active_contract_rejects_a_body_check_statement() {
    let outcome = parse_active("body-check.wf", BODY_CHECK_STATEMENT);
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("a body check must not parse once check_stmt leaves stmt: {outcome:?}");
    };
    assert_eq!(issue.rule(), SyntaxRule::Form1);
    let start = usize::try_from(issue.coordinate().start().value()).unwrap_or(usize::MAX);
    let end = usize::try_from(issue.coordinate().end().value()).unwrap_or(usize::MAX);
    assert_eq!(&BODY_CHECK_STATEMENT[start..end], b"check");
}

/// The unified contract owns erased definitions followed by plural requires
/// and ensures clauses, with no body-statement wrapper.
#[test]
fn active_contract_parses_definitions_and_clauses_as_direct_children() {
    let outcome = parse_active("contract.wf", UNIFIED_CONTRACT);
    let ParseOutcome::Complete(parsed) = outcome else {
        panic!("the unified contract must parse: {outcome:?}");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(
        parsed,
        FinalizeLimits {
            max_work: 4_000_000,
            max_roots: 65_536,
            max_shape_tasks: 65_536,
            max_nodes: 65_536,
            max_child_edges: 65_536,
            max_terminals: 65_536,
            max_sources: 16,
        },
    ) else {
        panic!("the unified contract fixture must finalize");
    };
    let topology = &finalized.topology;
    let mut selected = Vec::new();
    for (index, node) in topology.nodes.iter().enumerate() {
        if node.production != Production::ContractBlock {
            continue;
        }
        let id = NodeId::from_index(index).expect("contract node id");
        selected.extend(
            topology
                .node_children(id)
                .expect("contract has children")
                .iter()
                .filter_map(|child| topology.node(*child).map(|record| record.production))
                .filter(|production| {
                    matches!(
                        production,
                        Production::ContractDefine
                            | Production::RequiresClause
                            | Production::EnsuresClause
                    )
                }),
        );
    }
    assert_eq!(
        selected,
        vec![
            Production::ContractDefine,
            Production::RequiresClause,
            Production::EnsuresClause,
        ],
        "definitions and clauses retain source order in one contract"
    );
}

#[test]
fn active_contract_reserves_the_new_fixed_spellings() {
    let outcome = parse_active("identifiers.wf", RESERVED_SPELLINGS_AS_IDENTIFIERS);
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("as/external/blocks must be reserved spellings excluded from IDENT: {outcome:?}");
    };
    assert_eq!(issue.rule(), SyntaxRule::Form3);
}

#[test]
fn active_contract_parses_the_complete_counted_range_statement() {
    let outcome = parse_active("range.wf", COUNTED_RANGE_STATEMENT);
    let ParseOutcome::Complete(parsed) = outcome else {
        panic!("the active tables must derive a counted range: {outcome:?}");
    };
    assert!(parsed.tree.elements.iter().any(|element| {
        matches!(
            element,
            DerivationElement::Production { production, .. }
                if *production == Production::ForStmt
        )
    }));
}

#[test]
fn counted_range_fixed_words_are_not_identifier_spellings() {
    for source in [
        b"fn for() -> result: own unit pure {\n  return unit;\n}\n".as_slice(),
        b"fn probe() -> result: own unit pure {\n  let in = 0_u64;\n  return unit;\n}\n",
    ] {
        let ParseOutcome::SourceIssue(issue) = parse_active("reserved-range.wf", source) else {
            panic!("for/in must be excluded from IDENT");
        };
        assert_eq!(issue.rule(), SyntaxRule::Form3);
    }
}

#[test]
fn malformed_counted_ranges_stop_at_their_first_grammar_boundary() {
    for (source, boundary) in [
        (
            b"fn probe(lower: own u64, upper: own u64) -> result: own unit pure {\n  for index in lower..upper {\n  }\n  return unit;\n}\n".as_slice(),
            b"index".as_slice(),
        ),
        (
            b"fn probe(lower: own u64, upper: own u64) -> result: own unit pure {\n  for @range in lower..upper {\n  }\n  return unit;\n}\n",
            b"in",
        ),
        (
            b"fn probe(lower: own u64, upper: own u64) -> result: own unit pure {\n  for @range index lower..upper {\n  }\n  return unit;\n}\n",
            b"lower",
        ),
        (
            b"fn probe(lower: own u64, upper: own u64) -> result: own unit pure {\n  for @range index in ..upper {\n  }\n  return unit;\n}\n",
            b"..",
        ),
        (
            b"fn probe() -> result: own unit pure {\n  for @range index in 0_u64 . 1_u64 {\n  }\n  return unit;\n}\n",
            b".",
        ),
        (
            b"fn probe(lower: own u64) -> result: own unit pure {\n  for @range index in lower.. {\n  }\n  return unit;\n}\n",
            b"{",
        ),
        (
            b"fn probe(lower: own u64, upper: own u64) -> result: own unit pure {\n  for @range index in lower..upper..upper {\n  }\n  return unit;\n}\n",
            b"..",
        ),
    ] {
        let outcome = parse_active("malformed-range.wf", source);
        let ParseOutcome::SourceIssue(issue) = outcome else {
            panic!("malformed counted range must reject: {outcome:?}");
        };
        assert_eq!(issue_bytes(source, issue), boundary);
    }
}

/// Returns the exact source bytes a grammar rejection selected.
fn issue_bytes(source: &'static [u8], issue: super::SyntaxIssue) -> &'static [u8] {
    let start = usize::try_from(issue.coordinate().start().value()).unwrap_or(usize::MAX);
    let end = usize::try_from(issue.coordinate().end().value()).unwrap_or(usize::MAX);
    &source[start..end]
}

#[test]
fn malformed_input_labels_reject_at_their_exact_grammar_boundary() {
    // `input_label := IDENT "." IDENT "as"` has no other legal spelling, so
    // each near miss stops at the first token that cannot continue it. The
    // reserved `as` in a label-tail IDENT slot is DIAG-1 attribution row 3.
    for (source, rule, boundary, expected) in [
        (
            b"command fn main(command.args args: own Args) -> status: own ExitStatus pure {\n  return unit;\n}\n".as_slice(),
            SyntaxRule::Gram2,
            b"args".as_slice(),
            TerminalPredicate::Fixed(FixedTerminal::As),
        ),
        (
            b"command fn main(command.args.more as args: own Args) -> status: own ExitStatus pure {\n  return unit;\n}\n",
            SyntaxRule::Gram2,
            b".",
            TerminalPredicate::Fixed(FixedTerminal::As),
        ),
        (
            b"command fn main(command. as args: own Args) -> status: own ExitStatus pure {\n  return unit;\n}\n",
            SyntaxRule::Form3,
            b"as",
            TerminalPredicate::Identifier,
        ),
        (
            b"command fn main(command.args as: own Args) -> status: own ExitStatus pure {\n  return unit;\n}\n",
            SyntaxRule::Gram2,
            b":",
            TerminalPredicate::Identifier,
        ),
        (
            b"command fn main(command.args as args own Args) -> status: own ExitStatus pure {\n  return unit;\n}\n",
            SyntaxRule::Gram2,
            b"own",
            TerminalPredicate::Fixed(FixedTerminal::Colon),
        ),
    ] {
        let outcome = parse_active("label.wf", source);
        let ParseOutcome::SourceIssue(issue) = outcome else {
            panic!("malformed label must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), rule, "source: {:?}", String::from_utf8_lossy(source));
        assert_eq!(issue_bytes(source, issue), boundary);
        assert!(
            issue
                .expected()
                .contains(crate::syntax::grammar::LookaheadPredicate::Terminal(expected))
        );
    }
}

#[test]
fn declaration_and_parameter_optionals_report_their_complete_expected_sets() {
    // `fn_decl := "deny_claims"? program_kind? "fn" ...`: once the fixed
    // program kind is consumed, a non-`fn` continuation is a GRAM-2 shape
    // failure at the first invalid continuation, with `fn` retained as the
    // sole expected terminal.
    let outcome = parse_active(
        "kind.wf",
        b"command struct Thing {\n  a: i32;\n}\n\nfn main() -> result: own unit pure {\n  return unit;\n}\n",
    );
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("a program_kind not followed by `fn` must reject: {outcome:?}");
    };
    assert_eq!(issue.rule(), SyntaxRule::Gram2);
    assert_eq!(
        issue_bytes(
            b"command struct Thing {\n  a: i32;\n}\n\nfn main() -> result: own unit pure {\n  return unit;\n}\n",
            issue,
        ),
        b"struct"
    );
    assert_eq!(issue.expected().len(), 1);
    assert!(
        issue
            .expected()
            .contains(crate::syntax::grammar::LookaheadPredicate::Terminal(
                TerminalPredicate::Fixed(FixedTerminal::Fn)
            ))
    );

    // Once the fixed marker is consumed, either the fixed `command` program
    // kind or an ordinary `fn` may follow. The program kind is no longer an
    // arbitrary identifier, so the complete continuation set is two fixed
    // terminals.
    let marked_non_function = b"deny_claims struct Thing {\n  a: i32;\n}\n".as_slice();
    let outcome = parse_active("marked-kind.wf", marked_non_function);
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("deny_claims not followed by a function must reject: {outcome:?}");
    };
    assert_eq!(issue.rule(), SyntaxRule::Gram2);
    assert_eq!(issue_bytes(marked_non_function, issue), b"struct");
    assert_eq!(issue.expected().len(), 2);
    assert!(
        issue
            .expected()
            .contains(crate::syntax::grammar::LookaheadPredicate::Terminal(
                TerminalPredicate::Fixed(FixedTerminal::Command)
            ))
    );
    assert!(
        issue
            .expected()
            .contains(crate::syntax::grammar::LookaheadPredicate::Terminal(
                TerminalPredicate::Fixed(FixedTerminal::Fn)
            ))
    );

    // `param := input_label? IDENT ":" mode type`: `input_label` begins with
    // the fixed `command` terminal, so after an ordinary parameter IDENT only
    // the colon continuation remains live.
    let unresolved_param =
        b"command fn main(args own Args) -> status: own ExitStatus pure {\n  return unit;\n}\n"
            .as_slice();
    let outcome = parse_active("param.wf", unresolved_param);
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("an IDENT continuing neither param arm must reject: {outcome:?}");
    };
    assert_eq!(issue.rule(), SyntaxRule::Gram2);
    assert_eq!(issue_bytes(unresolved_param, issue), b"own");
    assert_eq!(issue.expected().len(), 1);
    assert!(
        issue
            .expected()
            .contains(crate::syntax::grammar::LookaheadPredicate::Terminal(
                TerminalPredicate::Fixed(FixedTerminal::Colon)
            )),
        "an ordinary parameter name must be followed by a colon"
    );
}

#[test]
fn a_program_kind_and_an_input_label_derive_outside_the_entry() {
    // The grammar attaches `program_kind` to every `fn_decl` and `input_label`
    // to every `param`. FN-7 restricts both to the unit's entry, so the parser
    // must derive these units and leave the rejection to semantic checking
    // rather than reporting invalid source here.
    for source in [
        b"deny_claims command fn helper(command.args as args: own Args) -> result: own unit pure {\n  return unit;\n}\n\nfn main() -> result: own unit pure {\n  return unit;\n}\n".as_slice(),
        b"command fn helper(command.args as args: own Args) -> result: own unit pure {\n  return unit;\n}\n\nfn main() -> result: own unit pure {\n  return unit;\n}\n".as_slice(),
        b"fn helper(command.args as args: own Args) -> result: own unit pure {\n  return unit;\n}\n\nfn main() -> result: own unit pure {\n  return unit;\n}\n",
    ] {
        let outcome = parse_active("outside.wf", source);
        assert!(
            matches!(outcome, ParseOutcome::Complete(_)),
            "FN-7 placement is not a grammar decision: {outcome:?}"
        );
    }
}
