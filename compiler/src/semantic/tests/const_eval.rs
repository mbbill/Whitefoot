//! Const-expression evaluation [CONST-1].
//!
//! The candidate grammar admits one operation inside a `const`, so the
//! evaluator below is reachable from source. These tests pin the const-eval
//! overflow policy's exact u64 domain judgments and that a const-position
//! operation reaches that evaluator through the ordinary path.

use crate::SemanticOutcome;

use super::super::model::{ConstOperation, evaluate_const_operation};
use super::with_semantics;

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

/// The positive source-level case: one operation written at a `const`
/// position parses under the candidate tables and reaches the evaluator, so
/// `fixed_vector<u64, 2 * 3>` is an accepted six-slot run rather than the
/// syntax rejection v0.30 gave it. `cap_of` is the measure the written const
/// generic fixes on a fresh run [BLK-1], which is where the evaluated `2 * 3`
/// is observable.
#[test]
fn const_position_arithmetic_parses_and_evaluates() {
    let source = b"command fn main() -> status: own ExitStatus pure {\n  let filled = fixed_vector::<u64, 2 * 3>();\n  let count = cap_of(filled);\n  return exit_status(code: 0_u8);\n}\n";
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "const-position arithmetic is an accepted const-expression: {outcome:?}",
        );
    });
}
