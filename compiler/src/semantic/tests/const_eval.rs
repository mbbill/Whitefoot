//! Const-expression evaluation [CONST-1 candidate].
//!
//! The active v0.30 grammar admits no operation inside a `const`, so the
//! evaluator below is reachable from source only after the v0.31 candidate's
//! regenerated grammar tables land. These tests pin two things that hold
//! either way: the const-eval overflow policy's exact u64 domain judgments,
//! and the v0.30 inertness of the whole family — arithmetic written at a
//! const position is still a syntax rejection, never an accepted program.

use crate::lexer::{LexOutcome, lex};
use crate::{
    ACTIVE_KERNEL_SPEC_HASH, ParseOutcome, SourceBundle, SourceInput, TerminalLimits,
    TerminalOutcome, classify_terminals, parse,
};

use super::super::model::{ConstOperation, evaluate_const_operation};
use super::{LEX_LIMITS, PARSE_LIMITS, SOURCE_LIMITS};

#[test]
fn const_evaluation_covers_the_exact_u64_domain() {
    // In-domain results evaluate exactly.
    assert_eq!(evaluate_const_operation(ConstOperation::Add, 3, 4), Some(7));
    assert_eq!(
        evaluate_const_operation(ConstOperation::Subtract, 4, 3),
        Some(1)
    );
    assert_eq!(
        evaluate_const_operation(ConstOperation::Multiply, 6, 7),
        Some(42)
    );
    assert_eq!(
        evaluate_const_operation(ConstOperation::Divide, 42, 5),
        Some(8)
    );
    assert_eq!(
        evaluate_const_operation(ConstOperation::Remainder, 42, 5),
        Some(2)
    );
    // The domain boundaries themselves are legal values.
    assert_eq!(
        evaluate_const_operation(ConstOperation::Add, u64::MAX, 0),
        Some(u64::MAX)
    );
    assert_eq!(
        evaluate_const_operation(ConstOperation::Subtract, 3, 3),
        Some(0)
    );
}

#[test]
fn const_evaluation_rejects_every_out_of_domain_result() {
    // The const-eval overflow policy: no u64 result means rejection, with no
    // wrap, saturation, or runtime trap alternative.
    assert_eq!(
        evaluate_const_operation(ConstOperation::Add, u64::MAX, 1),
        None
    );
    assert_eq!(
        evaluate_const_operation(ConstOperation::Subtract, 3, 4),
        None
    );
    assert_eq!(
        evaluate_const_operation(ConstOperation::Multiply, 1_u64 << 32, 1_u64 << 32),
        None
    );
    assert_eq!(evaluate_const_operation(ConstOperation::Divide, 1, 0), None);
    assert_eq!(
        evaluate_const_operation(ConstOperation::Remainder, 1, 0),
        None
    );
}

/// Negative control for the family's v0.30 inertness: an operation written
/// at a `const` position is a syntax rejection under the active grammar
/// tables, so no semantic const-arithmetic path is reachable from source.
#[test]
fn const_position_arithmetic_is_a_syntax_rejection_under_v030() {
    let source = b"fn main() -> own unit pure {\n  let filled = array_new<u64, 2 * 3>(0_u64);\n  return unit;\n}\n";
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("const arithmetic control bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("const arithmetic control must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("const arithmetic control must classify");
    };
    let ParseOutcome::SourceIssue(_) = parse(&classified, PARSE_LIMITS) else {
        panic!("const-position arithmetic must stay a v0.30 syntax rejection");
    };
}
