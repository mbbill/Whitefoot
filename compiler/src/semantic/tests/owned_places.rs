//! Typed owned-storage paths retain their owner, complete projection, and
//! ordinary bounds and loan restrictions.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::{assert_rule_at, assert_rule_kind, assert_unsupported, with_semantics};

const ROWS: &str = r#"struct Row {
  left: u64;
  right: u64;
}

fn identity['r](value: &'r Row) -> result: &'r Row pure {
  return value;
}

fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Row, 4>();
  let first = Row(left: 3_u64, right: 4_u64);
  region {
    place_back(vector: &uniq empty, value: move first);
  }
  let prefix = move empty;
  let second = Row(left: 5_u64, right: 6_u64);
  region {
    place_back(vector: &uniq prefix, value: move second);
  }
  let rows = move prefix;
"#;

fn rows(body: &str) -> String {
    format!("{ROWS}{body}  return exit_status(code: 0_u8);\n}}\n")
}

fn accepts(source: &str) {
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
}

const BOX_READ_OUT: &str = r#"struct Payload {
  value: u64;
}

struct Pair {
  left: Payload;
  right: Payload;
}

fn fresh['s](owner: own Box<'s, Payload>, store: &uniq Heap<'s>) -> result: own Payload writes(store) {
  return Payload(value: 7_u64);
}

"#;

fn box_read_out(body: &str) -> String {
    let body = body.trim().replace("}\nfn", "}\n\nfn");
    format!(
        "{}\n\n{body}\n\nfn main() -> status: own ExitStatus pure {{\n  return exit_status(code: 0_u8);\n}}\n",
        BOX_READ_OUT.trim()
    )
}

#[test]
fn box_referent_read_out_spends_the_selected_storage_once() {
    for statement in [
        "set (deref(owner).left, deref(owner).right) = move deref(owner).left, move deref(owner).left;",
        "set (deref(owner).left, scalar) = move deref(owner).left, deref(owner).left.value;",
        "set (deref(owner).left, moved) = move deref(owner).left, move owner;",
    ] {
        let source = box_read_out(&format!(
            r#"
fn repeat['s](owner: own Box<'s, Pair>) -> result: own Box<'s, Pair> reads(owner), writes(owner) {{
  let scalar = 0_u64;
  {statement}
  return move owner;
}}
"#
        ));
        assert_rule_kind(source.as_bytes(), SemanticRule::Own1, |kind| {
            matches!(kind, SemanticIssueKind::UseAfterMove { .. })
        });
    }
}

#[test]
fn box_referent_target_does_not_turn_an_owner_move_into_a_read_out() {
    let source = box_read_out(
        r#"
fn outer['s](owner: own Box<'s, Payload>, store: &uniq Heap<'s>) -> result: own unit writes(owner, store) {
  set deref(owner) = fresh(owner: move owner, store: move store);
  return unit;
}
"#,
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Own1, |kind| {
        matches!(kind, SemanticIssueKind::UseAfterMove { .. })
    });
}

#[test]
fn box_referent_read_out_preserves_shared_and_non_target_boundaries() {
    let source = box_read_out(
        r#"
fn shared['heap](owner: &Box<'heap, Payload>) -> result: own unit reads(owner) {
  set deref(deref(owner)) = move deref(deref(owner));
  return unit;
}

"#,
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Own5, |kind| {
        matches!(kind, SemanticIssueKind::BorrowConflict)
    });
    for body in [
        "let value = move deref(owner);",
        "set deref(owner) = move deref(other);",
    ] {
        let source = box_read_out(&format!(
            r#"
fn extract['s](owner: own Box<'s, Payload>, other: own Box<'s, Payload>) -> result: own unit reads(owner, other), writes(owner) {{
  {body}
  return unit;
}}
"#
        ));
        assert_unsupported(
            source.as_bytes(),
            UnsupportedSemanticFeature::BoxReferentMove,
        );
    }
}

#[test]
fn box_referent_read_out_preserves_holder_suspension() {
    let source = box_read_out(
        r#"
fn identity_box['r, 's](value: &uniq 'r Box<'s, Payload>) -> result: &uniq 'r Box<'s, Payload> pure {
  return &uniq 'r deref(value);
}

fn suspended['heap](owner: &uniq Box<'heap, Payload>) -> result: own unit reads(owner), writes(owner) {
  region {
    let child = identity_box(value: &uniq deref(owner));
    set deref(deref(owner)) = move deref(deref(owner));
  }
  return unit;
}
"#,
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Own5, |kind| {
        matches!(kind, SemanticIssueKind::BorrowConflict)
    });
}

#[test]
fn box_referent_descendant_extraction_keeps_its_cleanup_capability_boundary() {
    let source = box_read_out(
        r#"
fn rebuild(value: own Payload) -> result: own Pair pure {
  let other = Payload(value: 9_u64);
  return Pair(left: move value, right: move other);
}

fn descendant['s](owner: own Box<'s, Pair>) -> result: own Box<'s, Pair> reads(owner), writes(owner) {
  set deref(owner) = rebuild(value: move deref(owner).left);
  return move owner;
}
"#,
    );
    assert_unsupported(
        source.as_bytes(),
        UnsupportedSemanticFeature::BoxReferentMove,
    );
}

#[test]
fn box_referent_read_out_rejects_a_later_reborrow_at_its_use() {
    let source = box_read_out(
        r#"
fn keep['heap](value: own Payload, alias: &uniq Box<'heap, Payload>) -> result: own Payload pure {
  return move value;
}

fn late['heap](owner: &uniq Box<'heap, Payload>) -> result: own unit reads(owner), writes(owner) {
  region {
    set deref(deref(owner)) = keep(value: move deref(deref(owner)), alias: &uniq deref(owner));
  }
  return unit;
}
"#,
    );
    // [LIV-2] the target is spent during RHS evaluation. Its later borrow
    // fails at that use, before post-RHS writability can fail at the target.
    assert_rule_at(source.as_bytes(), SemanticRule::Own1, "&uniq deref(owner)");
}

fn rejects(source: &str, rule: SemanticRule) {
    let kind: fn(&SemanticIssueKind) -> bool = match rule {
        SemanticRule::Own5 => |kind| matches!(kind, SemanticIssueKind::BorrowConflict),
        SemanticRule::Op4 => {
            |kind| matches!(kind, SemanticIssueKind::UndischargedBoundsObligation { .. })
        }
        _ => panic!("this suite pairs bounds and loan restrictions"),
    };
    assert_rule_kind(source.as_bytes(), rule, kind);
}

#[test]
fn direct_field_index_reads_and_scalar_commits_use_the_terminal_type() {
    let source = rows(
        r#"  let saved = rows[0_u64].left;
  set (rows[0_u64].left, rows[1_u64].right) = rows[1_u64].right, saved;
  let observed = rows[0_u64].left;
"#,
    );
    accepts(&source);
    rejects(
        &source.replace("let saved = rows[0_u64]", "let saved = rows[2_u64]"),
        SemanticRule::Op4,
    );
    assert_rule_kind(
        source
            .replace("rows[1_u64].right)", "rows[0_u64].left)")
            .as_bytes(),
        SemanticRule::Liv2,
        |kind| matches!(kind, SemanticIssueKind::OverlappingCommitTargets { .. }),
    );
}

#[test]
fn proved_dynamic_indices_separate_one_commit_targets() {
    let scalar = rows(
        r#"  let left = 0_u64;
  let right = 1_u64;
  if left < right {
    set (rows[left].left, rows[right].left) = rows[right].left, rows[left].left;
  }
"#,
    );
    accepts(&scalar);
    assert_rule_kind(
        rows(
            r#"  let left = 0_u64;
  let right = 0_u64;
  set (rows[left].left, rows[right].left) = rows[right].left, rows[left].left;
"#,
        )
        .as_bytes(),
        SemanticRule::Liv2,
        |kind| matches!(kind, SemanticIssueKind::OverlappingCommitTargets { .. }),
    );

    accepts(&rows(
        r#"  let left = 0_u64;
  let right = 1_u64;
  if left < right {
    set (rows[left], rows[right]) = move rows[right], move rows[left];
  }
"#,
    ));

    let named_const = rows("  set rows[first_index] = move rows[first_index];\n")
        .replace("fn main()", "const first_index: u64 = 0_u64;\n\nfn main()");
    accepts(&named_const);
}

#[test]
fn a_rhs_cannot_mutate_an_index_captured_by_dynamic_commit_targets() {
    let source = rows(
        r#"  let left = 0_u64;
  let right = 1_u64;
  if left < right {
    region {
      set (rows[left], rows[right]) = change_index(index: &uniq left), move rows[left];
    }
  }
"#,
    )
    .replace(
        "fn main()",
        "fn change_index(index: &uniq u64) -> result: own Row writes(index) {\n  set deref(index) = 1_u64;\n  return Row(left: 7_u64, right: 8_u64);\n}\n\nfn main()",
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Own5, |kind| {
        matches!(kind, SemanticIssueKind::BorrowConflict)
    });
}

#[test]
fn an_unproved_candidate_index_pair_does_not_separate_a_cross_path() {
    let source = br#"struct Cell {
  payload: box<u64>;
  tag: u64;
}

fn retag(taken: own Cell, tag: own u64) -> result: own Cell pure {
  let Cell(payload: payload, tag: unused_tag) = move taken;
  return Cell(payload: move payload, tag: tag);
}

fn invalid(values: &uniq FixedVector<array<Cell, 2>, 2>, i: own u64, j: own u64, k: own u64, l: own u64) -> result: own unit reads(values), writes(values) contract {
  requires i < len_of(deref(values));
  requires k < len_of(deref(values));
  requires j < 2_u64;
  requires l < 2_u64;
  requires i < k;
} {
  set (deref(values)[i][j], deref(values)[k][l]) = move deref(values)[i][j], retag(taken: move deref(values)[k][l], tag: deref(values)[i][l].tag);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Own1, |kind| {
        matches!(kind, SemanticIssueKind::UseAfterMove { .. })
    });
}

#[test]
fn scalar_element_field_selection_keeps_its_type_error_on_legacy_storage() {
    for body in [
        "  let value = values[0_u64].missing;\n",
        "  set values[0_u64].missing = 1_u8;\n",
    ] {
        let source = format!(
            "fn main() -> status: own ExitStatus pure {{\n  let values = buffer_new(1_u64, 0_u8);\n{body}  return exit_status(code: 0_u8);\n}}\n"
        );
        assert_rule_kind(source.as_bytes(), SemanticRule::Type5, |kind| {
            matches!(kind, SemanticIssueKind::TypeMismatch { .. })
        });
    }
}

#[test]
fn repeated_affine_element_read_out_keeps_type2_diagnostic_priority() {
    let source = rows("  set (rows[0_u64], rows[1_u64]) = move rows[1_u64], move rows[0_u64];\n");
    accepts(&source);
    assert_rule_kind(
        source
            .replace("move rows[0_u64];", "move rows[1_u64];")
            .as_bytes(),
        SemanticRule::Type2,
        |kind| matches!(kind, SemanticIssueKind::AffineElementMove { .. }),
    );
}

#[test]
fn affine_nested_fields_are_exchanged_or_read_out_once() {
    let source = r#"struct Payload {
  value: u64;
}

struct Entry {
  payload: Payload;
  other: u64;
}

fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Entry, 2>();
  let payload = Payload(value: 3_u64);
  let stored_entry = Entry(payload: move payload, other: 5_u64);
  region {
    place_back(vector: &uniq empty, value: move stored_entry);
  }
  let entries = move empty;
  let replacement = Payload(value: 7_u64);
  let old = replace entries[0_u64].payload = move replacement;
  set entries[0_u64].payload = move entries[0_u64].payload;
  let observed = entries[0_u64].payload.value;
  return exit_status(code: 0_u8);
}
"#;
    accepts(source);
    assert_rule_kind(
        source
            .replace(
                "  set entries[0_u64].payload = move entries[0_u64].payload;",
                "  let moved = move entries[0_u64].payload;",
            )
            .as_bytes(),
        SemanticRule::Type2,
        |kind| matches!(kind, SemanticIssueKind::AffineElementMove { .. }),
    );
    assert_rule_kind(
        source.replace("set entries[0_u64].payload = move entries[0_u64].payload;", "set (entries[0_u64].payload, entries[0_u64].other) = move entries[0_u64].payload, entries[0_u64].payload.value;").as_bytes(),
        SemanticRule::Own1,
        |kind| matches!(kind, SemanticIssueKind::UseAfterMove { .. }),
    );
}

const INLINE_VIEW: &str = r#"fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 4>();
  region {
    place_back(vector: &uniq empty, value: 7_u8);
  }
  let built = move empty;
  region {
    let view = mut_slice_of(&uniq built);
    set view[0_u64] = 9_u8;
  }
  let observed = built[0_u64];
  let result = observed -wrap 9_u8;
  return exit_status(code: result);
}
"#;

#[test]
fn exclusive_inline_view_retains_bounds_and_owner_loan() {
    accepts(INLINE_VIEW);
    rejects(
        &INLINE_VIEW.replace("view[0_u64]", "view[1_u64]"),
        SemanticRule::Op4,
    );
    rejects(
        &INLINE_VIEW.replace("set view[0_u64]", "set built[0_u64]"),
        SemanticRule::Own5,
    );
    assert_rule_kind(
        INLINE_VIEW
            .replace("set view[0_u64] = 9_u8;", "let moved = move built;")
            .as_bytes(),
        SemanticRule::Own5,
        |kind| matches!(kind, SemanticIssueKind::BorrowConflict),
    );
}

const BORROWED_BUFFER_REPLACEMENT: &str = r#"fn renew(values: &uniq buffer<u64>, replacement: own buffer<u64>) -> result: own u64 reads(values), writes(values) {
  let previous = replace deref(values) = move replacement;
  return 11_u64;
}

fn main() -> status: own ExitStatus pure {
  let first = buffer_new(1_u64, 3_u64);
  let second = buffer_new(1_u64, 7_u64);
  region {
    set first[0_u64] = renew(values: &uniq first, replacement: move second);
  }
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn borrowed_buffer_descriptor_replacement_is_an_explicit_capability_stop() {
    assert_unsupported(
        BORROWED_BUFFER_REPLACEMENT.as_bytes(),
        UnsupportedSemanticFeature::BorrowedBufferDescriptorMutation,
    );
    rejects(
        &BORROWED_BUFFER_REPLACEMENT
            .replace("values: &uniq buffer", "values: &buffer")
            .replace("reads(values), writes(values)", "reads(values)"),
        SemanticRule::Own5,
    );
    assert_rule_kind(
        BORROWED_BUFFER_REPLACEMENT
            .replace("let previous = replace deref(values)", "set deref(values)")
            .as_bytes(),
        SemanticRule::Stor1,
        |kind| matches!(kind, SemanticIssueKind::AffineSetTarget { .. }),
    );
    assert_rule_kind(
        BORROWED_BUFFER_REPLACEMENT
            .replace(
                "  let previous =",
                "  let consumed = move replacement;\n  let previous =",
            )
            .as_bytes(),
        SemanticRule::Own1,
        |kind| matches!(kind, SemanticIssueKind::UseAfterMove { .. }),
    );
    accepts(
        r#"fn main() -> status: own ExitStatus pure {
  let first = buffer_new(1_u64, 3_u64);
  let second = buffer_new(1_u64, 7_u64);
  let previous = replace first = move second;
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn borrowed_aggregate_buffer_replacement_cannot_bypass_the_capability_stop() {
    let source = r#"struct Holder {
  values: buffer<u64>;
  count: u64;
}

fn renew(holder: &uniq Holder, replacement: own buffer<u64>) -> result: own unit reads(holder.values), writes(holder.values) {
  let previous = replace deref(holder).values = move replacement;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_unsupported(
        source.as_bytes(),
        UnsupportedSemanticFeature::BorrowedBufferDescriptorMutation,
    );
    assert_unsupported(
        source
            .replace("replacement: own buffer<u64>", "replacement: own Holder")
            .replace("deref(holder).values =", "deref(holder) =")
            .replace(
                "reads(holder.values), writes(holder.values)",
                "reads(holder), writes(holder)",
            )
            .as_bytes(),
        UnsupportedSemanticFeature::BorrowedBufferDescriptorMutation,
    );
    accepts(
        &source
            .replace(
                "let previous = replace deref(holder).values = move replacement;",
                "set deref(holder).count = 11_u64;",
            )
            .replace(
                "reads(holder.values), writes(holder.values)",
                "writes(holder.count)",
            ),
    );
    accepts(
        &source.replace(
            "  let previous = replace deref(holder).values = move replacement;",
            "  let count = len_of(deref(holder).values);\n  if count > 0_u64 {\n    set deref(holder).values[0_u64] = 11_u64;\n  }",
        ),
    );
}
