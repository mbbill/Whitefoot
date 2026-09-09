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

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Row, 4>();
  let first = Row(left: 3_u64, right: 4_u64);
  let prefix = place_back(vector: move empty, value: move first);
  let second = Row(left: 5_u64, right: 6_u64);
  let rows = place_back(vector: move prefix, value: move second);
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
        "{}\n\n{body}\n\ncommand fn main() -> status: own ExitStatus pure {{\n  return exit_status(code: 0_u8);\n}}\n",
        BOX_READ_OUT.trim()
    )
}

#[test]
fn box_referent_read_out_reinitializes_direct_borrowed_and_nested_storage() {
    accepts(&box_read_out(
        r#"
fn direct['s](owner: own Box<'s, Payload>) -> result: own Box<'s, Payload> reads(owner), writes(owner) {
  set deref(owner) = move deref(owner);
  return move owner;
}
fn borrowed(owner: &uniq Box<Payload>) -> result: own unit reads(owner), writes(owner) {
  set deref(deref(owner)) = move deref(deref(owner));
  return unit;
}
fn nested['s](owner: own Box<'s, Box<'s, Payload>>) -> result: own Box<'s, Box<'s, Payload>> reads(owner), writes(owner) {
  set deref(deref(owner)) = move deref(deref(owner));
  return move owner;
}
fn field['s](owner: own Box<'s, Pair>) -> result: own Box<'s, Pair> reads(owner), writes(owner) {
  set deref(owner).left = move deref(owner).left;
  return move owner;
}
fn append['s](storage: own Box<'s, FixedVector<u64, 16>>, value: own u64) -> result: own Box<'s, FixedVector<u64, 16>> reads(storage), writes(storage) contract {
  requires room_of(deref(storage)) > 0_u64;
} {
  set deref(storage) = place_back(vector: move deref(storage), value: value);
  return move storage;
}
"#,
    ));
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
fn shared(owner: &Box<Payload>) -> result: own unit reads(owner) {
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

fn suspended(owner: &uniq Box<Payload>) -> result: own unit reads(owner), writes(owner) {
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
fn keep(value: own Payload, alias: &uniq Box<Payload>) -> result: own Payload pure {
  return move value;
}

fn late(owner: &uniq Box<Payload>) -> result: own unit reads(owner), writes(owner) {
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
fn inline_element_borrows_keep_distinct_literal_paths() {
    accepts(&rows(
        r#"  region {
    let left = &rows[0_u64];
    let right = &uniq rows[1_u64];
    let value = deref(left).left;
    set deref(right).right = value;
  }
"#,
    ));
}

#[test]
fn indexed_borrow_results_keep_the_complete_candidate_place() {
    let source = rows(
        r#"  region {
    let saved = identity(value: &rows[0_u64]);
    let replacement = Row(left: 7_u64, right: 8_u64);
    let old = replace rows[1_u64] = move replacement;
    let observed = deref(saved).left;
  }
"#,
    );
    accepts(&source);
    rejects(
        &source.replace("replace rows[1_u64]", "replace rows[0_u64]"),
        SemanticRule::Own5,
    );
}

#[test]
fn element_borrow_cannot_read_a_raw_slot_or_outlive_its_owner() {
    rejects(
        &rows(
            r#"  region {
    let raw = &rows[2_u64];
  }
"#,
        ),
        SemanticRule::Op4,
    );
    rejects(
        &rows(
            r#"  region {
    let saved = &rows[0_u64];
    let moved = move rows;
  }
"#,
        ),
        SemanticRule::Own5,
    );
}

#[test]
fn element_loans_reject_aliases_but_end_with_their_region() {
    rejects(
        &rows(
            r#"  region {
    let shared = &rows[0_u64];
    let unique = &uniq rows[0_u64];
  }
"#,
        ),
        SemanticRule::Own5,
    );
    accepts(&rows(
        r#"  region {
    let shared = &rows[0_u64];
  }
  let replacement = Row(left: 7_u64, right: 8_u64);
  let old = replace rows[0_u64] = move replacement;
"#,
    ));
}

#[test]
fn mutable_offset_borrows_do_not_establish_literal_disjointness() {
    rejects(
        &rows(
            r#"  let index = 0_u64;
  region {
    let saved = &rows[index];
    set index = 1_u64;
    let replacement = Row(left: 7_u64, right: 8_u64);
    let old = replace rows[0_u64] = move replacement;
  }
"#,
        ),
        SemanticRule::Own5,
    );
}

#[test]
fn field_index_field_borrow_selects_the_actual_inline_storage() {
    accepts(
        r#"struct Row {
  left: u64;
  right: u64;
}

struct Table {
  rows: FixedVector<Row, 1>;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Row, 1>();
  let item = Row(left: 3_u64, right: 4_u64);
  let rows = place_back(vector: move empty, value: move item);
  let table = Table(rows: move rows);
  region {
    let left = &table.rows[0_u64].left;
    let right = &uniq table.rows[0_u64].right;
    set deref(right) = deref(left);
  }
  return exit_status(code: 0_u8);
}
"#,
    );
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
fn scalar_element_field_selection_keeps_its_type_error_on_legacy_storage() {
    for body in [
        "  let value = values[0_u64].missing;\n",
        "  set values[0_u64].missing = 1_u8;\n",
    ] {
        let source = format!(
            "command fn main() -> status: own ExitStatus pure {{\n  let values = buffer_new(1_u64, 0_u8);\n{body}  return exit_status(code: 0_u8);\n}}\n"
        );
        assert_rule_kind(source.as_bytes(), SemanticRule::Type5, |kind| {
            matches!(kind, SemanticIssueKind::TypeMismatch { .. })
        });
    }
}

#[test]
fn direct_field_index_writes_respect_borrowed_subfields() {
    let source = rows(
        r#"  region {
    let saved = &rows[0_u64].left;
    set rows[0_u64].right = 9_u64;
    let observed = deref(saved);
  }
"#,
    );
    accepts(&source);
    rejects(
        &source.replace("set rows[0_u64].right", "set rows[0_u64].left"),
        SemanticRule::Own5,
    );
    rejects(
        &rows(
            r#"  region {
    let shared = &rows;
    set deref(shared)[0_u64].left = 9_u64;
  }
"#,
        ),
        SemanticRule::Own5,
    );
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
fn returned_scalar_borrow_writes_preserve_the_enclosing_run_measure() {
    let source = rows(
        r#"  region {
    let changed = exclusive(value: &uniq rows[1_u64].right);
    set deref(changed) = 9_u64;
  }
  let observed = rows[0_u64].left;
"#,
    )
    .replace(
        "command fn main()",
        "fn exclusive['r](value: &uniq 'r u64) -> result: &uniq 'r u64 pure {\n  return &uniq 'r deref(value);\n}\n\ncommand fn main()",
    );
    accepts(&source);
    rejects(
        &source.replace(
            "  let observed =",
            "  let replacement = fixed_vector::<Row, 4>();\n  let removed = replace rows = move replacement;\n  let observed =",
        ),
        SemanticRule::Op4,
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

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Entry, 2>();
  let payload = Payload(value: 3_u64);
  let entry = Entry(payload: move payload, other: 5_u64);
  let entries = place_back(vector: move empty, value: move entry);
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

#[test]
fn replacement_commit_conflicts_with_its_rhs_temporary_borrow() {
    rejects(
        r#"struct Payload {
  value: u64;
}

fn update(value: &uniq u64) -> result: own Payload writes(value) {
  set deref(value) = 5_u64;
  let replacement = Payload(value: 7_u64);
  return move replacement;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Payload, 1>();
  let payload = Payload(value: 3_u64);
  let entries = place_back(vector: move empty, value: move payload);
  region {
    let previous = replace entries[0_u64] = update(value: &uniq entries[0_u64].value);
    let observed = previous.value;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
    );
}

const INLINE_VIEW: &str = r#"command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 4>();
  let built = place_back(vector: move empty, value: 7_u8);
  region {
    let view = mut_slice_of(&uniq built);
    set view[0_u64] = 9_u8;
  }
  let observed = built[0_u64];
  let result = observed -wrap 9_u8;
  return exit_status(code: result);
}
"#;

const TEMPORARY_SCALARS: &str = r#"fn change(value: &uniq u64) -> result: own u64 writes(value) {
  set deref(value) = 9_u64;
  return 7_u64;
}

fn inspect(value: &u64) -> result: own u64 reads(value) {
  return deref(value);
}

command fn main() -> status: own ExitStatus pure {
  let value = 3_u64;
  let left = 0_u64;
  let right = 0_u64;
  region {
BODY
  }
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn argument_loans_cover_later_rhs_accesses_and_the_commit() {
    for body in [
        "    set value = change(value: &uniq value);",
        "    set value = inspect(value: &value);",
        "    set (left, right) = change(value: &uniq value), value;",
        "    set (left, right) = change(value: &uniq value), change(value: &uniq value);",
        "    set (left, right) = inspect(value: &value), change(value: &uniq value);",
    ] {
        rejects(&TEMPORARY_SCALARS.replace("BODY", body), SemanticRule::Own5);
    }
    for body in [
        "    set (left, right) = inspect(value: &value), value;",
        "    set (left, right) = inspect(value: &value), inspect(value: &value);",
        "    let observed = change(value: &uniq value);\n    set value = 5_u64;",
    ] {
        accepts(&TEMPORARY_SCALARS.replace("BODY", body));
    }
}

#[test]
fn owned_match_headers_end_their_non_escaping_temporary_loans() {
    let source = TEMPORARY_SCALARS.replace(
        "command fn main()",
        "fn decide(value: &uniq u64) -> result: own Option<u64> pure {\n  return Some<u64>(value: 7_u64);\n}\n\ncommand fn main()",
    );
    let arms = "      None() => {\n      }\n      Some(value: chosen) => {\n        set value = chosen;\n      }\n    }";
    accepts(&source.replace(
        "BODY",
        &format!("    match decide(value: &uniq value) {{\n{arms}"),
    ));
    accepts(&source.replace(
        "BODY",
        &format!("    let decided = decide(value: &uniq value);\n    match decided {{\n{arms}"),
    ));
}

#[test]
fn control_headers_resume_only_their_own_temporary_children() {
    accepts(include_str!(
        "../../../../tests/conformance/cases/own6-pos-owned-control-headers-return-temporaries.wf"
    ));
    for source in [
        include_str!(
            "../../../../tests/conformance/cases/own5-neg-owned-header-keeps-bound-borrow.wf"
        ),
        include_str!(
            "../../../../tests/conformance/cases/own5-neg-owned-header-keeps-result-parent-suspended.wf"
        ),
        include_str!(
            "../../../../tests/conformance/cases/own13-neg-borrowed-header-keeps-parent-suspended.wf"
        ),
        include_str!(
            "../../../../tests/conformance/cases/own5-neg-owned-header-keeps-view-origin.wf"
        ),
    ] {
        rejects(source, SemanticRule::Own5);
    }
}

#[test]
fn statement_children_allow_disjoint_siblings_but_suspend_parent_transfer() {
    let source = r#"struct Pair {
  left: u64;
  right: u64;
}

fn change(value: &uniq u64) -> result: own u64 writes(value) {
  set deref(value) = 9_u64;
  return 7_u64;
}

fn sink(value: &uniq Pair) -> result: own u64 pure {
  return 11_u64;
}

command fn main() -> status: own ExitStatus pure {
  let pair = Pair(left: 3_u64, right: 5_u64);
  let left = 0_u64;
  let right = 0_u64;
  region {
    let holder = &uniq pair;
    region {
      set (left, right) = change(value: &uniq deref(holder).left), change(value: &uniq deref(holder).right);
    }
    set deref(holder).left = 13_u64;
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_unsupported(
        source.as_bytes(),
        UnsupportedSemanticFeature::RegionsAndBorrows,
    );
    let buffers = source
        .replace("left: u64;", "left: buffer<u8>;")
        .replace("right: u64;", "right: buffer<u8>;")
        .replace("fn change(value: &uniq u64) -> result: own u64 writes(value) {\n  set deref(value) = 9_u64;", "fn change(value: &uniq buffer<u8>) -> result: own u64 reads(value), writes(value) {\n  let size = len_of(deref(value));\n  let available = size != 0_u64;\n  if available {\n    set deref(value)[0_u64] = 9_u8;\n  }")
        .replace("  let pair = Pair(left: 3_u64, right: 5_u64);", "  let first = buffer_new(1_u64, 3_u8);\n  let second = buffer_new(1_u64, 5_u8);\n  let pair = Pair(left: move first, right: move second);")
        .replace("    set deref(holder).left = 13_u64;", "    region {\n      let resumed = change(value: &uniq deref(holder).left);\n    }");
    accepts(&buffers);
    rejects(
        &buffers.replace(
            "change(value: &uniq deref(holder).right)",
            "change(value: &uniq deref(holder).left)",
        ),
        SemanticRule::Own5,
    );
    let source = source.replace(
        "fn sink(value:",
        "fn touch(value: &uniq Pair) -> result: own u64 writes(value.left) {\n  set deref(value).left = 9_u64;\n  return 7_u64;\n}\n\nfn sink(value:",
    ).replace("change(value: &uniq deref(holder).left)", "touch(value: &uniq deref(holder))");
    rejects(
        &source.replace(
            "change(value: &uniq deref(holder).right)",
            "sink(value: move holder)",
        ),
        SemanticRule::Own5,
    );
    rejects(
        &source.replace(
            "change(value: &uniq deref(holder).right)",
            "change(value: &uniq deref(holder).left)",
        ),
        SemanticRule::Own5,
    );
    rejects(
        &source.replace(
            "change(value: &uniq deref(holder).right)",
            "deref(holder).right",
        ),
        SemanticRule::Own5,
    );
}

#[test]
fn view_descriptor_loans_cover_element_reads_measures_and_commits() {
    let source = r#"fn inspect(view: &uniq MutSlice<u8>) -> result: own u64 pure {
  return 7_u64;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 1>();
  let bytes = place_back(vector: move empty, value: 3_u8);
  let left = 0_u64;
  let right = 0_u64;
  let byte = 0_u8;
  region {
    let view = mut_slice_of(&uniq bytes);
    region {
BODY
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for body in [
        "      set (left, byte) = inspect(view: &uniq view), view[0_u64];",
        "      set (left, right) = inspect(view: &uniq view), len_of(view);",
        "      set (left, view[0_u64]) = inspect(view: &uniq view), 5_u8;",
    ] {
        rejects(&source.replace("BODY", body), SemanticRule::Own5);
    }
    accepts(&source.replace(
        "BODY",
        "      let observed = inspect(view: &uniq view);\n      set view[0_u64] = 5_u8;",
    ));
}

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

#[test]
fn exclusive_inline_field_view_keeps_sibling_storage_independent() {
    accepts(
        r#"struct Pair {
  bytes: FixedVector<u8, 2>;
  other: u64;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  let bytes = place_back(vector: move empty, value: 3_u8);
  let pair = Pair(bytes: move bytes, other: 4_u64);
  region {
    let view = mut_slice_of(&uniq pair.bytes);
    let sibling = &uniq pair.other;
    set deref(sibling) = 5_u64;
    set view[0_u64] = 9_u8;
  }
  let observed = pair.bytes[0_u64];
  return exit_status(code: 0_u8);
}
"#,
    );
}

const GUARDED_ELEMENT: &str = r#"fn overwrite(value: &uniq u64) -> result: own unit writes(value) {
  set deref(value) = 9_u64;
  return unit;
}

fn read_if_valid(index: own u64) -> result: own unit pure {
  let no_indices = fixed_vector::<u64, 2>();
  let prefix = place_back(vector: move no_indices, value: index);
  let indices = place_back(vector: move prefix, value: 9_u64);
  let no_data = fixed_vector::<u8, 1>();
  let data = place_back(vector: move no_data, value: 7_u8);
  region {
    let guarded = &uniq indices[0_u64];
    let valid = deref(guarded) < 1_u64;
    if valid {
      let observed = data[deref(guarded)];
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  read_if_valid(index: 0_u64);
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn writes_through_an_indexed_holder_kill_its_old_range_fact() {
    accepts(GUARDED_ELEMENT);
    rejects(
        &GUARDED_ELEMENT.replace(
            "      let observed =",
            "      region {\n        overwrite(value: &uniq deref(guarded));\n      }\n      let observed =",
        ),
        SemanticRule::Op4,
    );
}

#[test]
fn writes_to_a_distinct_element_preserve_the_guarded_range_fact() {
    accepts(&GUARDED_ELEMENT.replace(
        "      let observed =",
        "      region {\n        overwrite(value: &uniq indices[1_u64]);\n      }\n      let observed =",
    ));
}

const BORROWED_BUFFER_REPLACEMENT: &str = r#"fn renew(values: &uniq buffer<u64>, replacement: own buffer<u64>) -> result: own u64 reads(values), writes(values) {
  let previous = replace deref(values) = move replacement;
  return 11_u64;
}

command fn main() -> status: own ExitStatus pure {
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
        r#"command fn main() -> status: own ExitStatus pure {
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

command fn main() -> status: own ExitStatus pure {
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
