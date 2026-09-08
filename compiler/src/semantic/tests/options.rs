use crate::SemanticOutcome;

use super::with_semantics;

#[test]
fn concrete_options_reuse_the_nominal_path_for_supported_payloads() {
    let source = br#"struct Pair {
  left: u32;
  right: u32;
}

fn scalar(value: own i32) -> result: own Option<i32> pure {
  return Some<i32>(value: value);
}

fn aggregate(value: own Pair) -> result: own Option<Pair> pure {
  return Some<Pair>(value: move value);
}

fn nested() -> result: own Option<Option<u8>> pure {
  let inner = Some<u8>(value: 7_u8);
  return Some<Option<u8>>(value: move inner);
}

fn absent() -> result: own Option<Pair> pure {
  return None<Pair>();
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("resource-free Option payloads must check: {outcome:?}");
        };
        let names = checked
            .data
            .nominals
            .iter()
            .map(|nominal| nominal.name.as_str())
            .collect::<Vec<_>>();
        for expected in [
            "Option<i32>",
            "Option<Pair>",
            "Option<u8>",
            "Option<Option<u8>>",
        ] {
            assert!(
                names.contains(&expected),
                "missing concrete prelude nominal {expected}: {names:?}"
            );
        }
    });
}

/// The payload is a store-backed run rather than the retiring `buffer<u8>`.
/// A run's release spends its store's provider capability [PROV-6], so the
/// scope has to hold that capability: `abandon` receives it as `&uniq
/// Heap<'s>` and declares the `writes(store)` the release spends. The subject
/// is unchanged — one `Option` instance carrying a payload whose drop is
/// variant-dependent, and one drop on the return edge.
#[test]
fn option_of_a_resource_bearing_payload_uses_variant_dependent_cleanup() {
    let source = b"fn abandon['s](value: own Option<Vector<'s, u8>>, store: &uniq Heap<'s>) -> result: own unit writes(store) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("Option<Vector<'s, u8>> must check: {outcome:?}");
        };
        let nominal = checked
            .data
            .nominals
            .iter()
            .find(|nominal| nominal.name == "Option<Vector<'s, u8>>")
            .expect("concrete Option instance must be interned");
        let super::super::model::CheckedStatement::Return { drops, .. } =
            &checked.data.functions[0].body[0]
        else {
            panic!("abandon must end in return");
        };
        assert_eq!(drops.len(), 1);
        assert_eq!(
            drops[0].ty,
            super::super::model::CheckedType::Nominal(nominal.id)
        );
    });
}
