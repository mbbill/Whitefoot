//! The rendered record, pinned byte for byte for one representative of each
//! family a writer meets, in both formats.
//!
//! The per-rule sentences are pinned by `driver::pinned_sentences`; these
//! pin the shape around them: the summary line, the envelope labels and their
//! order, the marker, related positions, escaping, and the JSON object.

use super::{DiagnosticFormat, render_driver_failure};
use crate::{CompilationFailure, CompilerLimits, SourceInput, check};

/// The failure one source-only compilation must stop with.
fn stop(name: &str, source: &[u8]) -> CompilationFailure {
    check(&[SourceInput::new(name, source)], CompilerLimits::default())
        .expect_err("this fixture exists to be rejected")
}

/// No record a writer is shown names a compiler-internal identity.
fn assert_no_internal_identity(rendered: &str) {
    for identity in ["SourceId", "NodePath", "ByteOffset", "SyntaxCoordinate"] {
        assert!(!rendered.contains(identity), "{identity} in {rendered}");
    }
}

const BOUNDS: &[u8] =
    br#"fn drop_spaces(out: &[u8], src: &[u8]) -> kept: u64 reads(src), writes(out) contract {
  requires deref(out).len >= deref(src).len;
} {
  doc "Copies every byte of src that is not a space to the front of out.";
  let kept = 0_u64;
  let count = deref(src).len;
  for (at in 0_u64..count) {
    let byte = deref(src)[at];
    if byte == 32_u8 {
    } else {
      set deref(out)[kept] = byte;
      set kept = kept +wrap 1_u64;
    }
  }
  return kept;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

const OP4_FIX: &str = "when the relation must hold, establish the residual with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when its false edge is intended program behavior; otherwise restructure the access";

#[test]
fn an_undischarged_subscript_prints_its_residual_under_a_marked_line() {
    let failure = stop("bounds.wf", BOUNDS);
    let expected = format!(
        "bounds.wf:11:21: error[OP-4]: UndischargedBoundsObligation
  rule: OP-4
  kind: UndischargedBoundsObligation
  category: Source
  stage: Semantics
  at: bounds.wf:11:21
  bytes: 377..383
  source:       set deref(out)[kept] = byte;
  marker:                     ^^^^^^
  residual: kept < deref(out).len
  mechanical_fix: {OP4_FIX}"
    );
    assert_eq!(failure.to_string(), expected);
    assert_no_internal_identity(&expected);
}

#[test]
fn the_json_record_carries_the_same_fields_on_one_line() {
    let failure = stop("bounds.wf", BOUNDS);
    let expected = format!(
        r#"{{"rule":"OP-4","kind":"UndischargedBoundsObligation","category":"Source","stage":"Semantics","at":{{"file":"bounds.wf","line":11,"column":21}},"bytes":{{"start":377,"end":383}},"source":"      set deref(out)[kept] = byte;","detail":{{"residual":"kept < deref(out).len","mechanical_fix":"{OP4_FIX}"}}}}"#
    );
    let json = failure.render(DiagnosticFormat::Json);
    assert_eq!(json, expected);
    assert!(!json.contains('\n'));
    // One compiler executable renders one stop the same way every time.
    assert_eq!(
        stop("bounds.wf", BOUNDS).render(DiagnosticFormat::Json),
        json
    );
}

#[test]
fn a_call_requirement_names_the_callee_clause_as_a_position() {
    let source =
        br#"fn drop_spaces(out: &[u8], src: &[u8]) -> kept: u64 reads(src), writes(out) contract {
  requires deref(out).len >= deref(src).len;
} {
  doc "Copies every byte of src that is not a space to the front of out.";
  let kept = 0_u64;
  let count = deref(src).len;
  for (
    at in 0_u64..count,
    invariant behind: kept <= at
  ) {
    let byte = deref(src)[at];
    if byte == 32_u8 {
    } else {
      set deref(out)[kept] = byte;
      set kept = kept + 1_u64;
    }
  }
  return kept;
}

fn main() -> status: ExitStatus pure {
  let text = array_filled::<u8, 6>(value: 32_u8);
  let buffer = array_filled::<u8, 5>(value: 0_u8);
  let kept = drop_spaces(out: &buffer[0_u64..5_u64], src: &text[0_u64..6_u64]);
  return exit_status(code: 0_u8);
}
"#;
    let failure = stop("caller.wf", source);
    let expected = format!(
        "caller.wf:24:14: error[FN-8]: UndischargedCallRequirement
  rule: FN-8
  kind: UndischargedCallRequirement
  category: Source
  stage: Semantics
  at: caller.wf:24:14
  bytes: 646..711
  source:   let kept = drop_spaces(out: &buffer[0_u64..5_u64], src: &text[0_u64..6_u64]);
  marker:              {}
  concrete_callee: drop_spaces
  requires_clause: caller.wf:2:3 \"requires deref(out).len >= deref(src).len;\"
  instantiated_goal: buffer[0..5].len >= text[0..6].len
  disposition: Refuted
  mechanical_fix: when the call is required to succeed, establish the entire instantiated callee requirement with a verified requirement, a source invariant, or explicit finite proof steps before the call; use a dominating branch only when rejection is intended program behavior; otherwise restructure the call",
        "^".repeat(65)
    );
    assert_eq!(failure.to_string(), expected);
    assert_no_internal_identity(&expected);
    // The related position is an object with the envelope's own field names.
    assert!(
        failure.render(DiagnosticFormat::Json).contains(
            r#""requires_clause":{"at":{"file":"caller.wf","line":2,"column":3},"bytes":{"start":89,"end":131},"source":"requires deref(out).len >= deref(src).len;"}"#
        ),
        "{}",
        failure.render(DiagnosticFormat::Json)
    );
}

#[test]
fn a_failed_loop_invariant_names_its_obligation_and_required_relation() {
    let source =
        br#"fn drop_spaces(out: &[u8], src: &[u8]) -> kept: u64 reads(src), writes(out) contract {
  requires deref(out).len >= deref(src).len;
} {
  doc "Copies every byte of src that is not a space to the front of out.";
  let kept = 0_u64;
  let count = deref(src).len;
  for (
    at in 0_u64..count,
    invariant behind: kept <= at
  ) {
    let byte = deref(src)[at];
    if byte == 32_u8 {
    } else {
      set deref(out)[kept] = byte;
      set kept = kept +wrap 1_u64;
    }
  }
  return kept;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let failure = stop("invariant.wf", source);
    assert_eq!(
        failure.to_string(),
        "invariant.wf:9:5: error[INV-1]: UndischargedLoopInvariant
  rule: INV-1
  kind: UndischargedLoopInvariant
  category: Source
  stage: Semantics
  at: invariant.wf:9:5
  bytes: 297..325
  source:     invariant behind: kept <= at
  marker:     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  name: behind
  obligation: Backedge
  required_relation: kept <= (at + 1_u64)
  mechanical_fix: strengthen the invariant prefix, weaken or correct this invariant, or establish the missing body facts so every reachable normal fallthrough preserves it at the next loop header"
    );
}

#[test]
fn a_grammar_rejection_quotes_the_expected_terminals_and_the_token_it_found() {
    let source = b"fn main() -> status: ExitStatus pure {\n  let a = 42;\n  return exit_status(code: 0_u8);\n}\n";
    let failure = stop("suffix.wf", source);
    // A fixed terminal is quoted as its spelling; a token class is named.
    assert_eq!(
        failure.to_string(),
        r#"suffix.wf:2:11: error[FORM-5]: UnexpectedToken
  rule: FORM-5
  kind: UnexpectedToken
  category: Source
  stage: Parsing
  at: suffix.wf:2:11
  bytes: 49..51
  source:   let a = 42;
  marker:           ^^
  expected: [TYPEID, IDENT, "&", "move", "if", "propagate", "match", literal, "musttail", OPNAME, "deref", "entry"]
  found: "42""#
    );
}

#[test]
fn a_canonical_rejection_quotes_the_trivia_and_marks_the_line_it_ends_in() {
    let source = b"fn main() -> status: ExitStatus pure {\n    let a = 1_i32;\n  return exit_status(code: 0_u8);\n}\n";
    let failure = stop("indent.wf", source);
    assert_eq!(
        failure.to_string(),
        r#"indent.wf:2:1: error[FORM-2]: NonCanonicalTrivia
  rule: FORM-2
  kind: NonCanonicalTrivia
  category: Source
  stage: CanonicalSource
  at: indent.wf:2:1
  bytes: 38..43
  source:     let a = 1_i32;
  marker: ^^^^
  expected: "\n  "
  found: "\n    ""#
    );
    // JSON carries the trivia itself, in JSON's own escapes.
    assert!(
        failure
            .render(DiagnosticFormat::Json)
            .ends_with(r#""detail":{"expected":"\n  ","found":"\n    "}}"#),
        "{}",
        failure.render(DiagnosticFormat::Json)
    );
}

/// Source text is ASCII up to its first defect [DIAG-1], so a multi-byte
/// scalar on the quoted line is the defect itself or lies after it. The
/// defect is marked by one caret per character, not per byte, the line is
/// shown as written, and the field that quotes the defect escapes it; `bytes`
/// stays the exact interval.
#[test]
fn a_multi_byte_scalar_is_marked_as_one_character_and_escaped_where_quoted() {
    let source = "fn main() -> status: ExitStatus pure {\n  let x = \u{2192} caf\u{e9};\n  return exit_status(code: 0_u8);\n}\n";
    let failure = stop("utf8.wf", source.as_bytes());
    assert_eq!(
        failure.to_string(),
        "utf8.wf:2:11: error[FORM-1]: UnexpectedByte
  rule: FORM-1
  kind: UnexpectedByte
  category: Source
  stage: Lexing
  at: utf8.wf:2:11
  bytes: 49..52
  source:   let x = \u{2192} caf\u{e9};
  marker:           ^
  found: \"\\u{2192}\""
    );
    // JSON carries the scalar itself, and the same character column.
    assert_eq!(
        failure.render(DiagnosticFormat::Json),
        "{\"rule\":\"FORM-1\",\"kind\":\"UnexpectedByte\",\"category\":\"Source\",\"stage\":\"Lexing\",\"at\":{\"file\":\"utf8.wf\",\"line\":2,\"column\":11},\"bytes\":{\"start\":49,\"end\":52},\"source\":\"  let x = \u{2192} caf\u{e9};\",\"detail\":{\"found\":\"\u{2192}\"}}"
    );
}

/// A byte that is not a scalar, or a scalar that is not source text, is often
/// invisible, so the field that quotes it escapes it.
#[test]
fn an_invalid_byte_is_escaped_where_it_is_quoted() {
    let failure = stop(
        "byte.wf",
        b"fn main() -> status: ExitStatus pure {\n  let x = \xff;\n  return exit_status(code: 0_u8);\n}\n",
    );
    let rendered = failure.to_string();
    assert!(
        rendered.starts_with("byte.wf:2:11: error[FORM-2]: InvalidUtf8\n"),
        "{rendered}"
    );
    assert!(
        rendered.contains("\n  source:   let x = \u{fffd};\n"),
        "{rendered}"
    );
    assert!(rendered.ends_with("\n  found: \"\\xff\""), "{rendered}");
    assert_eq!(failure.detail(), "found: \"\\xff\"");
}

#[test]
fn a_capability_stop_is_located_but_never_cites_a_rule() {
    let failure = stop(
        "arms.wf",
        b"enum Flag {\n  A();\n  B();\n}\n\nfn main() -> status: ExitStatus pure {\n  let flag = A();\n  match flag {\n    A() => {\n    }\n    A() => {\n    }\n    B() => {\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
    );
    let rendered = failure.to_string();
    assert!(
        rendered.starts_with("arms.wf:11:5: unsupported capability: DuplicateMatchArm\n  kind: DuplicateMatchArm\n  category: Unsupported\n  stage: Semantics\n  at: arms.wf:11:5\n"),
        "{rendered}"
    );
    assert!(!rendered.contains("rule:"), "{rendered}");
    assert_no_internal_identity(&rendered);
}

#[test]
fn a_compiler_facing_stop_keeps_its_payload_under_its_category() {
    let failure = check(&[], CompilerLimits::default()).expect_err("no source record");
    assert_eq!(
        failure.to_string(),
        "whitefootc: invocation failure: EmptySourceSequence
  kind: EmptySourceSequence
  category: Invocation
  stage: SourceEnvelope
  payload: EmptySourceSequence"
    );
    assert_eq!(
        failure.render(DiagnosticFormat::Json),
        r#"{"kind":"EmptySourceSequence","category":"Invocation","stage":"SourceEnvelope","detail":{"payload":"EmptySourceSequence"}}"#
    );
}

#[test]
fn a_driver_stop_is_one_sentence_in_text_and_an_envelope_in_json() {
    let message = "cannot read \"missing.wf\": No such file or directory (os error 2)";
    assert_eq!(
        render_driver_failure("Invocation", message, DiagnosticFormat::Text),
        format!("whitefootc: {message}")
    );
    assert_eq!(
        render_driver_failure("Invocation", message, DiagnosticFormat::Json),
        r#"{"category":"Invocation","stage":"Driver","detail":{"message":"cannot read \"missing.wf\": No such file or directory (os error 2)"}}"#
    );
}
