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
