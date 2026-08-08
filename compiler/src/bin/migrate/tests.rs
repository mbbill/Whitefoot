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

/// The four dotless comparisons respell; `ilt` and `igt` keep their names and
/// lose only their arguments (O1).
#[test]
fn comparisons_respell_and_the_two_named_rows_keep_their_spelling() {
    let source = b"fn main() -> own unit traps {\n  let a: own u64 = 1_u64;\n  let same: own Bool = ieq<u64>(a, 1_u64);\n  let under: own Bool = ilt<u64>(a, 2_u64);\n  check same else trap \"eq\";\n  check under else trap \"lt\";\n  return unit;\n}\n";
    let out = migrated(source);
    assert!(out.contains("let same = a == 1_u64;"), "{out}");
    assert!(out.contains("let under = ilt(a, 2_u64);"), "{out}");
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
