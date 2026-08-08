//! The pre-pass is checked by its effect on real sources: what it produces
//! must parse and render, and a file already in v0.23 must come back
//! unchanged.

use super::migrate;

fn migrated(source: &[u8]) -> String {
    let (bytes, _) = migrate(source, std::path::Path::new("test.wf")).expect("migrates");
    String::from_utf8(bytes).expect("v0.23 source is text")
}

#[test]
fn a_let_annotation_is_deleted_and_the_result_renders_canonically() {
    let source =
        b"fn main() -> own unit pure {\n  let value: own u64 = 41_u64;\n  return unit;\n}\n";
    assert_eq!(
        migrated(source),
        "fn main() -> own unit pure {\n  let value = 41_u64;\n  return unit;\n}\n"
    );
}

/// The renderer owns layout, so the pre-pass may leave any spacing at all.
#[test]
fn layout_is_the_renderers_and_odd_input_spacing_is_normalized() {
    let source = b"fn main() -> own unit pure {\n      let value:   own   u64   =   41_u64;\n  return unit;\n}\n";
    assert_eq!(
        migrated(source),
        "fn main() -> own unit pure {\n  let value = 41_u64;\n  return unit;\n}\n"
    );
}

/// [O1] the tool ships re-runnable: a migrated file migrates to itself.
#[test]
fn migration_is_idempotent() {
    let source =
        b"fn main() -> own unit pure {\n  let value: own u64 = 41_u64;\n  return unit;\n}\n";
    let once = migrated(source);
    let twice = migrated(once.as_bytes());
    assert_eq!(once, twice);
}

/// A3 and the prelude class are coupled: the constructor's arguments come
/// from the annotation being deleted, so they are captured before it drops.
#[test]
fn a_bare_prelude_constructor_takes_the_arguments_its_annotation_carried() {
    let source = b"enum Bad {\n  Worse();\n}\n\nfn main() -> own unit pure {\n  let held: own Result<u64, Bad> = Ok(value: 1_u64);\n  let absent: own Option<u64> = None();\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(
        out.contains("let held = Ok<u64, Bad>(value: 1_u64);"),
        "{out}"
    );
    assert!(out.contains("let absent = None<u64>();"), "{out}");
}

/// A constructor whose annotation names no generic prelude nominal keeps the
/// spelling it had; nothing is invented for it.
#[test]
fn a_constructor_with_no_generic_annotation_stays_bare() {
    let source = b"struct Pair {\n  left: u64;\n  right: u64;\n}\n\nfn main() -> own unit pure {\n  let pair: own Pair = Pair(left: 1_u64, right: 2_u64);\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(
        out.contains("let pair = Pair(left: 1_u64, right: 2_u64);"),
        "{out}"
    );
}

/// A spelling inside a string is never a token, so the pre-pass cannot see it.
#[test]
fn a_spelling_inside_a_string_is_untouched() {
    let source = b"fn main() -> own unit traps {\n  let flag: own Bool = True();\n  check flag else trap \"let x: own u64 = 1_u64;\";\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("trap \"let x: own u64 = 1_u64;\""), "{out}");
    assert!(out.contains("let flag = True();"), "{out}");
}

/// [OP-1] a respelled row becomes infix, and its written type argument goes
/// with the call form that carried it.
#[test]
fn a_named_arithmetic_row_becomes_its_operator() {
    let source = b"fn main() -> own unit traps {\n  let a: own i32 = 1_i32;\n  let b: own i32 = iadd.wrap<i32>(a, 2_i32);\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("let b = a +wrap 2_i32;"), "{out}");
}

/// All six comparisons keep their named calls after the owner's cancellation
/// of the infix spellings, and lose only their written arguments.
#[test]
fn every_comparison_keeps_its_name_and_loses_only_its_argument() {
    let source = b"fn main() -> own unit traps {\n  let a: own u64 = 1_u64;\n  let same: own Bool = ieq<u64>(a, 1_u64);\n  let under: own Bool = ilt<u64>(a, 2_u64);\n  check same else trap \"eq\";\n  check under else trap \"lt\";\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("let same = ieq(a, 1_u64);"), "{out}");
    assert!(out.contains("let under = ilt(a, 2_u64);"), "{out}");
}

/// The reverse class: a corpus already migrated to the cancelled spelling
/// comes back to the named call, operands in the order they were written.
#[test]
fn an_infix_comparison_returns_to_its_named_call() {
    let source = b"fn main() -> own unit traps {\n  let a = 1_u64;\n  let same = a == 1_u64;\n  let other = a != 2_u64;\n  let under = a <= 2_u64;\n  let over = a >= 0_u64;\n  check same else trap \"eq\";\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("let same = ieq(a, 1_u64);"), "{out}");
    assert!(out.contains("let other = ine(a, 2_u64);"), "{out}");
    assert!(out.contains("let under = ile(a, 2_u64);"), "{out}");
    assert!(out.contains("let over = ige(a, 0_u64);"), "{out}");
}

/// The operand recovery is the whole risk in the reverse class, so the atom
/// forms [GRAM-9] admits are exercised where they actually occur: a `deref`
/// group on both sides, a subscripted place, and a field suffix after a group.
#[test]
fn the_reverse_class_recovers_every_atom_form_it_can_meet() {
    let source = b"fn main() -> own unit traps {\n  let a = 1_u64;\n  region 'r {\n    let p = &'r a;\n    check deref(p) == a else trap \"deref left\";\n    check a == deref(p) else trap \"deref right\";\n  }\n  let b = buffer_new(2_u64, 0_u8);\n  check b[0_u64] <= b[1_u64] else trap \"subscript\";\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("check ieq(deref(p), a) else"), "{out}");
    assert!(out.contains("check ieq(a, deref(p)) else"), "{out}");
    assert!(out.contains("check ile(b[0_u64], b[1_u64]) else"), "{out}");
}

/// The statement keyword before an operand is not part of it. This is the one
/// boundary a keyword blacklist would have got wrong, so it is asserted in
/// every position that introduces an expression.
#[test]
fn a_statement_keyword_is_never_swallowed_into_an_operand() {
    let source = b"fn pick(x: own i32) -> own Bool traps {\n  check x == 0_i32 else trap \"check\";\n  if x >= 1_i32 {\n    return x != 2_i32;\n  }\n  return x <= 3_i32;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("check ieq(x, 0_i32) else"), "{out}");
    assert!(out.contains("if ige(x, 1_i32) {"), "{out}");
    assert!(out.contains("return ine(x, 2_i32);"), "{out}");
    assert!(out.contains("return ile(x, 3_i32);"), "{out}");
}

/// A comparison inside a string is not a token either, so the reverse class
/// cannot reach it — the same property the annotation class relies on.
#[test]
fn an_infix_comparison_inside_a_string_is_untouched() {
    let source = b"fn main() -> own unit traps {\n  let a = 1_u64;\n  check a == 1_u64 else trap \"want a == 1\";\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("trap \"want a == 1\""), "{out}");
    assert!(out.contains("check ieq(a, 1_u64) else"), "{out}");
}

/// The reverse class is re-runnable like every other: a named call holds no
/// operator, so a second pass is the identity.
#[test]
fn the_reverse_class_is_idempotent() {
    let source = b"fn main() -> own unit traps {\n  let a = 1_u64;\n  check a == 1_u64 else trap \"eq\";\n  return unit;\n}\n";
    let once = migrated(source);
    let twice = migrated(once.as_bytes());
    assert_eq!(once, twice);
}

/// The retained-argument class keeps what no operand can supply.
#[test]
fn the_retained_argument_class_keeps_its_written_arguments() {
    let source = b"fn main() -> own unit pure {\n  let total: own u64 = 300_u64;\n  let narrowed: own Result<u8, NarrowError> = cvt<u64, u8>(total);\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("cvt<u64, u8>(total)"), "{out}");
}

/// Two classes in one statement, which is where their ordering shows: the
/// annotation supplies the constructor's arguments while the initializer is
/// itself a respelled row, and a de-argumented row sits in the same function.
#[test]
fn interacting_classes_in_one_function_compose() {
    let source = b"fn main() -> own unit traps {\n  let data: own buffer<u8> = buffer_new<u8>(4_u64, 0_u8);\n  let size: own u64 = len<u8>(data);\n  let bigger: own u64 = iadd.wrap<u64>(size, 1_u64);\n  let held: own Option<u64> = Some(value: bigger);\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("let data = buffer_new(4_u64, 0_u8);"), "{out}");
    assert!(out.contains("let size = len(data);"), "{out}");
    assert!(out.contains("let bigger = size +wrap 1_u64;"), "{out}");
    assert!(
        out.contains("let held = Some<u64>(value: bigger);"),
        "{out}"
    );
}

/// Almost every prelude constructor in the corpus is returned, not bound, so
/// its arguments come from the signature — which A3 does not delete.
#[test]
fn a_returned_constructor_takes_the_signature_result_arguments() {
    let source = b"enum Bad {\n  Worse();\n}\n\nfn pick(flag: own Bool) -> own Result<u64, Bad> pure {\n  return Ok(value: 1_u64);\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("return Ok<u64, Bad>(value: 1_u64);"), "{out}");
}

/// A returned constructor in a function whose result names no generic
/// prelude nominal is left alone.
#[test]
fn a_returned_constructor_with_a_plain_result_stays_bare() {
    let source =
        b"fn main() -> own unit pure {\n  let held: own Option<u64> = None();\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("let held = None<u64>();"), "{out}");
}

/// [GRAM-6] the Bool match becomes `if`, and its arms' own braces become the
/// conditional's — the reshape the renderer cannot rescue if it is wrong.
#[test]
fn a_bool_match_becomes_an_if_with_both_branches() {
    let source = b"fn main() -> own unit traps {\n  let flag: own Bool = True();\n  match flag {\n    True() => {\n      check flag else trap \"then\";\n    }\n    False() => {\n      check flag else trap \"else\";\n    }\n  }\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("if flag {"), "{out}");
    assert!(out.contains("} else {"), "{out}");
    assert!(!out.contains("match"), "{out}");
}

/// [ERR-2] an empty alternative is spelled by the else-free `if`, because
/// GRAM-6 rejects the empty `else`.
#[test]
fn an_empty_false_arm_becomes_the_else_free_if() {
    let source = b"fn main() -> own unit traps {\n  let flag: own Bool = True();\n  match flag {\n    True() => {\n      check flag else trap \"then\";\n    }\n    False() => {\n    }\n  }\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("if flag {"), "{out}");
    // `check ... else trap` also spells `else`, so the join line is the test.
    assert!(!out.contains("} else {"), "{out}");
}

/// [ERR-2]'s asymmetry: the empty then-block is admitted where the empty
/// else is not, so this one keeps both branches.
#[test]
fn an_empty_true_arm_keeps_the_empty_then_block() {
    let source = b"fn main() -> own unit traps {\n  let flag: own Bool = True();\n  match flag {\n    True() => {\n    }\n    False() => {\n      check flag else trap \"else\";\n    }\n  }\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("if flag {"), "{out}");
    assert!(out.contains("} else {"), "{out}");
}

/// [GRAM-6] an `else` block holding exactly one conditional must flatten.
#[test]
fn a_nested_match_in_the_false_arm_flattens_to_else_if() {
    let source = b"fn main() -> own unit traps {\n  let flag: own Bool = True();\n  let other: own Bool = False();\n  match flag {\n    True() => {\n      check flag else trap \"a\";\n    }\n    False() => {\n      match other {\n        True() => {\n          check flag else trap \"b\";\n        }\n        False() => {\n          check flag else trap \"c\";\n        }\n      }\n    }\n  }\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("} else if other {"), "{out}");
}

/// A value initializer's `else` is mandatory by grammar, so an empty one is
/// kept rather than dropped — dropping it would demote it to a statement.
#[test]
fn a_value_match_keeps_its_else_even_when_empty() {
    let source = b"fn main() -> own unit pure {\n  let flag: own Bool = True();\n  let picked: own i32 = match flag {\n    True() => {\n      give 1_i32;\n    }\n    False() => {\n      give 2_i32;\n    }\n  }\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("let picked = if flag {"), "{out}");
    assert!(out.contains("} else {"), "{out}");
}

/// The exclusion is a rule over the manifest, not a list of file names.
///
/// The real manifest is read rather than a fixture, because the defect this
/// closes was a hand list drifting out of agreement with the corpus: a
/// seventeenth `FORM-*` case must change this count, and a case leaving the
/// family must too.
#[test]
fn the_surface_form_exclusion_is_derived_from_the_real_manifest() {
    let manifest = include_str!("../../../../tests/conformance/manifest.jsonl");
    let ids = super::manifest::surface_form_ids(manifest).expect("the manifest reads");
    let mut named: Vec<_> = ids.iter().map(String::as_str).collect();
    named.sort_unstable();
    assert_eq!(
        named,
        [
            "form1-neg-unknown-construct",
            "form2-neg-noncanonical-ws",
            "form3-neg-opname-bad-suffix",
            "form3-neg-region-param-missing-apostrophe",
            "form3-neg-requires-binding",
            "form3-neg-reserved-mode-field",
            "form3-neg-typeid-fn-name",
            "form4-neg-comment",
            "form5-neg-missing-suffix",
            "form7-neg-leading-zero",
            "form7-neg-out-of-range",
            "x-form-form2-tab-indent",
            "x-form-form3-enum-name-ident",
            "x-form-form4-block-comment",
            "x-form-form5-op-arg-missing-suffix",
            "x-form-form7-i32-max-plus-one",
        ]
    );
}

/// The two row shapes the reader must tell apart, and the one that must not be
/// read as a case: an annotation row carries a top-level `rule` and no `id`.
#[test]
fn only_reject_rows_citing_a_form_rule_are_excluded() {
    let manifest = concat!(
        "# a comment line\n",
        "\n",
        r#"{"id": "layout", "rules": ["FORM-2"], "expect": {"kind": "reject", "rule": "FORM-2"}, "status": "runnable", "doc": "d"}"#,
        "\n",
        r#"{"id":"semantic","rules":["FORM-2","OP-1"],"expect":{"kind":"reject","rule":"OP-1"},"status":"runnable","doc":"d"}"#,
        "\n",
        r#"{"id": "positive", "rules": ["FORM-2"], "expect": {"kind": "run", "status": 0}, "status": "runnable", "doc": "d"}"#,
        "\n",
        r#"{"rule": "FORM-6", "covered_by": "policy", "reason": "r"}"#,
        "\n",
    );
    let ids = super::manifest::surface_form_ids(manifest).expect("the manifest reads");
    let mut named: Vec<_> = ids.iter().map(String::as_str).collect();
    named.sort_unstable();
    assert_eq!(named, ["layout"]);
}

/// [GIVE-1] a value initializer's `give` is a third position where a bare
/// prelude constructor needs the arguments its binder's annotation carries.
/// The direct rule never reached it, because the constructor sits inside an
/// arm rather than after the `=`.
#[test]
fn a_delivered_constructor_takes_the_binder_annotation_arguments() {
    let source = b"fn choose(b: own Bool) -> own Result<u64, u64> pure {\n  let result: own Result<u64, u64> = match b {\n    True() => {\n      give Ok(value: 1_u64);\n    }\n    False() => {\n      give Err(error: 2_u64);\n    }\n  }\n  return move result;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("give Ok<u64, u64>(value: 1_u64);"), "{out}");
    assert!(out.contains("give Err<u64, u64>(error: 2_u64);"), "{out}");
}

/// [ERR-3] the propagated position is the one whose arguments come from two
/// places: the Ok half is the binder's annotation and the error half is the
/// function's declared result error, so neither source alone is enough.
#[test]
fn a_propagated_constructor_joins_the_annotation_and_the_result_error() {
    let source = b"enum StepError {\n  Failed();\n}\n\nstruct Pair {\n  value: i32;\n}\n\nfn direct(error: own StepError) -> own Result<Pair, StepError> pure {\n  let accepted: own i32 = propagate Err(error: error);\n  let pair: own Pair = Pair(value: accepted);\n  return Ok(value: move pair);\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(
        out.contains("propagate Err<i32, StepError>(error: error);"),
        "{out}"
    );
    // The control: the returned constructor in the same function still takes
    // both of its arguments from the signature, so the two rules are not
    // reading each other's source.
    assert!(
        out.contains("return Ok<Pair, StepError>(value: move pair);"),
        "{out}"
    );
}
