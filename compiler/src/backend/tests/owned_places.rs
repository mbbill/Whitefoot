//! Execute ownership transfers over inline aggregates. These cases observe
//! values and effects rather than the emitter's choice of instructions.

use super::{compile, compile_and_run, compile_link_and_run};

/// Retain the ordinary emitted call boundaries for a second execution. This
/// changes only optimization permission, so a passing inlined body cannot hide
/// a broken aggregate argument or result ABI.
fn retain_calls(module: &str) -> String {
    let mut retained = 0;
    let result = module
        .lines()
        .map(|line| {
            if line.starts_with("define internal ")
                && let Some(header) = line.strip_suffix(" {")
            {
                retained += 1;
                format!("{header} noinline {{\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    assert!(retained > 0, "the fixture must retain an emitted function");
    result
}

fn assert_success(module: &str) {
    let output = compile_and_run(module);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn general_and_extent_boxes_keep_distinct_cleanup_actions() {
    let source = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 11_u64) {
      Err(error: back) => {
        return exit_status(code: 70_u8);
      }
      Ok(value: general) => {
        region 'a {
          let store = arena_frame::<8, 8, 'a>();
          region {
            match arena_box(store: &uniq store, value: 22_u64) {
              Err(error: back) => {
                return exit_status(code: 70_u8);
              }
              Ok(value: extent) => {
                if deref(general) != 11_u64 {
                  return exit_status(code: 1_u8);
                }
                if deref(extent) != 22_u64 {
                  return exit_status(code: 2_u8);
                }
                return exit_status(code: 0_u8);
              }
            }
          }
        }
      }
    }
  }
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        let observed = retain_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        let host = allocation_observer(1, 0);
        let output = compile_link_and_run(&observed, Some(&host), &[]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        // The general cell is the sole host allocation. The extent cell is
        // reclaimed by its enclosing arena reset and must never reach free.
        assert_eq!(output.stdout, b"A1;F1;");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn borrowed_box_replacement_updates_the_owner_and_releases_each_cell_once() {
    let source = br#"fn exchange['s](slot: &uniq Box<'s, u64>, incoming: own Box<'s, u64>) -> previous: own Box<'s, u64> reads(slot), writes(slot) {
  let displaced = replace deref(slot) = move incoming;
  return move displaced;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 11_u64) {
      Err(error: back) => {
        return exit_status(code: 70_u8);
      }
      Ok(value: cell) => {
        match heap_box(store: &uniq heap, value: 22_u64) {
          Err(error: back) => {
            return exit_status(code: 70_u8);
          }
          Ok(value: incoming) => {
            region {
              let old = exchange(slot: &uniq cell, incoming: move incoming);
              if deref(old) != 11_u64 {
                return exit_status(code: 1_u8);
              }
            }
            if deref(cell) != 22_u64 {
              return exit_status(code: 2_u8);
            }
            return exit_status(code: 0_u8);
          }
        }
      }
    }
  }
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        let observed = retain_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        let host = allocation_observer(2, 0);
        let output = compile_link_and_run(&observed, Some(&host), &[]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stdout, b"A1;A2;F1;F2;");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn box_field_borrows_keep_the_owner_slot_across_reborrow_and_return() {
    let source = br#"struct Holder['s] {
  cell: Box<'s, u64>;
}

fn shared['r, 's](slot: &'r Box<'s, u64>) -> result: &'r Box<'s, u64> pure {
  return slot;
}

fn exclusive['r, 's](slot: &uniq 'r Box<'s, u64>) -> result: &uniq 'r Box<'s, u64> pure {
  return &uniq 'r deref(slot);
}

fn exchange['s](slot: &uniq Box<'s, u64>, incoming: own Box<'s, u64>) -> previous: own Box<'s, u64> reads(slot), writes(slot) {
  let displaced = replace deref(slot) = move incoming;
  return move displaced;
}

fn relay['s](slot: &uniq Box<'s, u64>, incoming: own Box<'s, u64>) -> previous: own Box<'s, u64> reads(slot), writes(slot) {
  region {
    return exchange(slot: &uniq deref(slot), incoming: move incoming);
  }
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 11_u64) {
      Err(error: back) => {
        return exit_status(code: 70_u8);
      }
      Ok(value: cell) => {
        let holder = Holder(cell: move cell);
        match heap_box(store: &uniq heap, value: 22_u64) {
          Err(error: back) => {
            return exit_status(code: 70_u8);
          }
          Ok(value: incoming) => {
            region {
              let borrowed = exclusive(slot: &uniq holder.cell);
              let old = relay(slot: move borrowed, incoming: move incoming);
              if deref(old) != 11_u64 {
                return exit_status(code: 1_u8);
              }
            }
            region {
              let borrowed = shared(slot: &holder.cell);
              if deref(deref(borrowed)) != 22_u64 {
                return exit_status(code: 2_u8);
              }
            }
            return exit_status(code: 0_u8);
          }
        }
      }
    }
  }
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        let observed = retain_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        let host = allocation_observer(2, 0);
        let output = compile_link_and_run(&observed, Some(&host), &[]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stdout, b"A1;A2;F1;F2;");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn borrowed_enum_payload_replacement_updates_the_child_owner_in_its_box() {
    let source = br#"enum Node['s] {
  Marker(prefix: u64, suffix: u64);
  HasChildren(left: Box<'s, u64>, right: Box<'s, u64>);
}

fn exchange_child['s](tree: &uniq Box<'s, Node<'s>>, incoming: own Box<'s, u64>) -> (previous: own Box<'s, u64>, kept: own u64) reads(tree), writes(tree) {
  match deref(deref(tree)) {
    Marker(prefix: marker_prefix, suffix: marker_suffix) => {
      return move incoming, 0_u64;
    }
    HasChildren(left: left_slot, right: child_slot) => {
      let previous = replace deref(child_slot) = move incoming;
      let kept = deref(deref(left_slot));
      return move previous, kept;
    }
  }
}

fn read_child['s](tree: &Box<'s, Node<'s>>) -> result: own u64 reads(tree) {
  match deref(deref(tree)) {
    Marker(prefix: marker_prefix, suffix: marker_suffix) => {
      return 0_u64;
    }
    HasChildren(left: left_slot, right: child_slot) => {
      if deref(deref(left_slot)) != 33_u64 {
        return 0_u64;
      }
      return deref(deref(child_slot));
    }
  }
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 11_u64) {
      Err(error: back) => {
        return exit_status(code: 70_u8);
      }
      Ok(value: first) => {
        match heap_box(store: &uniq heap, value: 22_u64) {
          Err(error: back) => {
            return exit_status(code: 70_u8);
          }
          Ok(value: incoming) => {
            match heap_box(store: &uniq heap, value: 33_u64) {
              Err(error: back) => {
                return exit_status(code: 70_u8);
              }
              Ok(value: sibling) => {
                let node = HasChildren(left: move sibling, right: move first);
                match heap_box(store: &uniq heap, value: move node) {
                  Err(error: back) => {
                    return exit_status(code: 70_u8);
                  }
                  Ok(value: tree) => {
                    region {
                      let (old, kept) = exchange_child(tree: &uniq tree, incoming: move incoming);
                      if deref(old) != 11_u64 {
                        return exit_status(code: 1_u8);
                      }
                      if kept != 33_u64 {
                        return exit_status(code: 3_u8);
                      }
                    }
                    region {
                      let observed = read_child(tree: &tree);
                      if observed != 22_u64 {
                        return exit_status(code: 2_u8);
                      }
                      return exit_status(code: 0_u8);
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        let observed = retain_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        let host = allocation_observer(4, 0);
        let output = compile_link_and_run(&observed, Some(&host), &[]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        // The displaced child is released before the caller rereads its tree.
        // That tree must own the replacement child, not a copied enum's slot.
        // PROV-6 then visits the sibling and replacement in field order.
        assert_eq!(output.stdout, b"A1;A2;A3;A4;F1;F3;F2;F4;");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn indexed_targets_are_captured_before_disjoint_rhs_effects() {
    let module = compile(
        br#"fn advance(offset: &uniq u64, trace: &uniq u64) -> result: own u64 reads(trace), writes(offset, trace) {
  set deref(offset) = 1_u64;
  let shifted = deref(trace) *wrap 10_u64;
  set deref(trace) = shifted +wrap 1_u64;
  return 41_u64;
}

fn finish(offset: &uniq u64, trace: &uniq u64) -> result: own u64 reads(offset, trace), writes(offset, trace) {
  let captured = deref(offset);
  set deref(offset) = 0_u64;
  let shifted = deref(trace) *wrap 10_u64;
  set deref(trace) = shifted +wrap 2_u64;
  return captured +wrap 50_u64;
}

command fn main() -> status: own ExitStatus pure {
  let empty_left = fixed_vector::<u64, 2>();
  let prefix_left = place_back(vector: move empty_left, value: 3_u64);
  let left = place_back(vector: move prefix_left, value: 5_u64);
  let empty_right = fixed_vector::<u64, 2>();
  let prefix_right = place_back(vector: move empty_right, value: 7_u64);
  let right = place_back(vector: move prefix_right, value: 11_u64);
  let offset = 0_u64;
  let trace = 0_u64;
  invariant single_target_bound: offset < len_of(left);
  region {
    set left[offset] = advance(offset: &uniq offset, trace: &uniq trace);
  }
  if left[0_u64] != 41_u64 {
    return exit_status(code: 1_u8);
  }
  if left[1_u64] != 5_u64 {
    return exit_status(code: 2_u8);
  }
  if offset != 1_u64 {
    return exit_status(code: 3_u8);
  }
  if trace != 1_u64 {
    return exit_status(code: 4_u8);
  }
  set offset = 0_u64;
  set trace = 0_u64;
  set left[0_u64] = 3_u64;
  let next_offset = 1_u64;
  let next_trace = 0_u64;
  invariant first_target_bound: offset < len_of(left);
  invariant second_target_bound: offset < len_of(right);
  region {
    set (left[offset], right[offset]) = advance(offset: &uniq offset, trace: &uniq trace), finish(offset: &uniq next_offset, trace: &uniq next_trace);
  }
  if left[0_u64] != 41_u64 {
    return exit_status(code: 5_u8);
  }
  if left[1_u64] != 5_u64 {
    return exit_status(code: 6_u8);
  }
  if right[0_u64] != 51_u64 {
    return exit_status(code: 7_u8);
  }
  if right[1_u64] != 11_u64 {
    return exit_status(code: 8_u8);
  }
  if offset != 1_u64 {
    return exit_status(code: 9_u8);
  }
  if next_offset != 0_u64 {
    return exit_status(code: 11_u8);
  }
  if next_trace != 2_u64 {
    return exit_status(code: 12_u8);
  }
  if trace != 1_u64 {
    return exit_status(code: 10_u8);
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_success(&module);
    assert_success(&retain_calls(&module));
}

#[test]
fn replace_captures_the_target_before_rhs_changes_its_index() {
    let module = compile(
        br#"struct Row {
  left: u64;
  right: u64;
}

fn replacement(offset: &uniq u64) -> result: own Row writes(offset) {
  set deref(offset) = 1_u64;
  return Row(left: 19_u64, right: 23_u64);
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Row, 2>();
  let first = Row(left: 3_u64, right: 5_u64);
  let prefix = place_back(vector: move empty, value: move first);
  let second = Row(left: 11_u64, right: 13_u64);
  let rows = place_back(vector: move prefix, value: move second);
  let offset = 0_u64;
  invariant target_bound: offset < len_of(rows);
  region {
    let old = replace rows[offset] = replacement(offset: &uniq offset);
    if old.left != 3_u64 {
      return exit_status(code: 1_u8);
    }
    if old.right != 5_u64 {
      return exit_status(code: 2_u8);
    }
    if rows[0_u64].left != 19_u64 {
      return exit_status(code: 3_u8);
    }
    if rows[0_u64].right != 23_u64 {
      return exit_status(code: 4_u8);
    }
    if rows[1_u64].left != 11_u64 {
      return exit_status(code: 5_u8);
    }
    if rows[1_u64].right != 13_u64 {
      return exit_status(code: 6_u8);
    }
    if offset != 1_u64 {
      return exit_status(code: 7_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_success(&module);
    assert_success(&retain_calls(&module));
}

#[test]
fn replaced_aggregate_snapshots_survive_writes_and_helper_returns() {
    // Structs are affine even when every leaf is copy. `replace` supplies the
    // owned old value; this does not invent a whole-struct copying operation.
    let module = compile(
        br#"struct Row {
  left: u64;
  right: u64;
}

struct Entry {
  row: Row;
  tag: u64;
}

fn rewrite(input: own Entry) -> result: own Entry reads(input.row.left, input.row.right, input.tag), writes(input.row.left, input.row.right, input.tag) {
  let saved = input.row.left;
  set (input.row.left, input.row.right) = input.row.right, saved;
  set input.tag = input.tag +wrap 100_u64;
  return move input;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Entry, 2>();
  let first_row = Row(left: 3_u64, right: 5_u64);
  let first = Entry(row: move first_row, tag: 7_u64);
  let prefix = place_back(vector: move empty, value: move first);
  let second_row = Row(left: 11_u64, right: 13_u64);
  let second = Entry(row: move second_row, tag: 17_u64);
  let entries = place_back(vector: move prefix, value: move second);
  let replacement_row = Row(left: 19_u64, right: 23_u64);
  let replacement = Entry(row: move replacement_row, tag: 29_u64);
  let old = replace entries[0_u64] = move replacement;
  set entries[0_u64].row.left = 31_u64;
  let rewritten = rewrite(input: move old);
  set rewritten.row.right = 37_u64;
  if rewritten.row.left != 5_u64 {
    return exit_status(code: 1_u8);
  }
  if rewritten.row.right != 37_u64 {
    return exit_status(code: 2_u8);
  }
  if rewritten.tag != 107_u64 {
    return exit_status(code: 3_u8);
  }
  if entries[0_u64].row.left != 31_u64 {
    return exit_status(code: 4_u8);
  }
  if entries[0_u64].row.right != 23_u64 {
    return exit_status(code: 5_u8);
  }
  if entries[0_u64].tag != 29_u64 {
    return exit_status(code: 6_u8);
  }
  if entries[1_u64].row.left != 11_u64 {
    return exit_status(code: 7_u8);
  }
  if entries[1_u64].row.right != 13_u64 {
    return exit_status(code: 8_u8);
  }
  if entries[1_u64].tag != 17_u64 {
    return exit_status(code: 9_u8);
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_success(&module);
    assert_success(&retain_calls(&module));
}

#[test]
fn aggregate_loop_carries_and_element_swaps_preserve_parallel_assignment() {
    let module = compile(
        br#"struct Row {
  left: u64;
  right: u64;
}

struct Table {
  rows: FixedVector<Row, 2>;
  tag: u64;
}

command fn main() -> status: own ExitStatus pure {
  let first = Row(left: 1_u64, right: 10_u64);
  let second = Row(left: 2_u64, right: 20_u64);
  for (round in 0_u64..5_u64) {
    set (first, second) = move second, move first;
    set first.left = first.left +wrap 100_u64;
    set second.right = second.right +wrap 1_u64;
  }
  if first.left != 302_u64 {
    return exit_status(code: 1_u8);
  }
  if first.right != 22_u64 {
    return exit_status(code: 2_u8);
  }
  if second.left != 201_u64 {
    return exit_status(code: 3_u8);
  }
  if second.right != 13_u64 {
    return exit_status(code: 4_u8);
  }
  let empty = fixed_vector::<Row, 2>();
  let prefix = place_back(vector: move empty, value: move first);
  let rows = place_back(vector: move prefix, value: move second);
  let table = Table(rows: move rows, tag: 41_u64);
  set (table.rows[0_u64], table.rows[1_u64]) = move table.rows[1_u64], move table.rows[0_u64];
  set (table.rows[0_u64].left, table.rows[1_u64].right) = table.rows[1_u64].right, table.rows[0_u64].left;
  if table.rows[0_u64].left != 22_u64 {
    return exit_status(code: 5_u8);
  }
  if table.rows[0_u64].right != 13_u64 {
    return exit_status(code: 6_u8);
  }
  if table.rows[1_u64].left != 302_u64 {
    return exit_status(code: 7_u8);
  }
  if table.rows[1_u64].right != 201_u64 {
    return exit_status(code: 8_u8);
  }
  if table.tag != 41_u64 {
    return exit_status(code: 9_u8);
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_success(&module);
}

#[test]
fn returned_element_borrows_and_inline_views_reach_the_owners_storage() {
    let module = compile(
        br#"struct Row {
  left: u64;
  right: u64;
}

struct Table {
  rows: FixedVector<Row, 2>;
  bytes: FixedVector<u8, 2>;
  tag: u64;
}

fn identity['r](value: &'r Row) -> result: &'r Row pure {
  return value;
}

fn exclusive['r](value: &uniq 'r u64) -> result: &uniq 'r u64 pure {
  return &uniq 'r deref(value);
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<Row, 2>();
  let first = Row(left: 3_u64, right: 5_u64);
  let prefix = place_back(vector: move empty, value: move first);
  let second = Row(left: 7_u64, right: 11_u64);
  let rows = place_back(vector: move prefix, value: move second);
  let raw = fixed_vector::<u8, 2>();
  let bytes = place_back(vector: move raw, value: 13_u8);
  let table = Table(rows: move rows, bytes: move bytes, tag: 17_u64);
  region {
    let saved = identity(value: &table.rows[0_u64]);
    let changed = exclusive(value: &uniq table.rows[1_u64].right);
    set deref(changed) = deref(saved).left +wrap 19_u64;
    let view = mut_slice_of(&uniq table.bytes);
    set view[0_u64] = 23_u8;
    let sibling = &uniq table.tag;
    set deref(sibling) = 29_u64;
  }
  if table.rows[0_u64].left != 3_u64 {
    return exit_status(code: 1_u8);
  }
  if table.rows[0_u64].right != 5_u64 {
    return exit_status(code: 2_u8);
  }
  if table.rows[1_u64].left != 7_u64 {
    return exit_status(code: 3_u8);
  }
  if table.rows[1_u64].right != 22_u64 {
    return exit_status(code: 4_u8);
  }
  if table.bytes[0_u64] != 23_u8 {
    return exit_status(code: 5_u8);
  }
  if table.tag != 29_u64 {
    return exit_status(code: 6_u8);
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_success(&module);
    assert_success(&retain_calls(&module));
}

#[test]
fn partial_construction_refusal_preserves_effects_values_and_release_order() {
    let module = compile(
        br#"struct Pair['s] {
  first: Box<'s, u64>;
  second: Box<'s, u64>;
}

fn construct['s](store: &uniq Heap<'s>, counter: &uniq u64) -> result: own Result<Pair<'s>, u64> reads(store, counter), writes(store, counter), allocates(store) {
  let first_tick = deref(counter) +wrap 1_u64;
  set deref(counter) = first_tick;
  let first_value = first_tick *wrap 11_u64;
  region {
    match heap_box(store: &uniq deref(store), value: first_value) {
      Err(error: first_back) => {
        return Err<Pair<'s>, u64>(error: first_back);
      }
      Ok(value: first_cell) => {
        let second_tick = deref(counter) +wrap 1_u64;
        set deref(counter) = second_tick;
        let second_value = second_tick *wrap 11_u64;
        region {
          match heap_box(store: &uniq deref(store), value: second_value) {
            Err(error: second_back) => {
              return Err<Pair<'s>, u64>(error: second_back);
            }
            Ok(value: second_cell) => {
              let pair = Pair(first: move first_cell, second: move second_cell);
              return Ok<Pair<'s>, u64>(value: move pair);
            }
          }
        }
      }
    }
  }
}

fn prepare['s](value: own Pair<'s>, counter: &uniq u64) -> result: own Pair<'s> reads(counter), writes(counter) {
  set deref(counter) = deref(counter) +wrap 1_u64;
  return move value;
}

fn finish['s](value: own Pair<'s>, store: &uniq Heap<'s>) -> result: own ExitStatus reads(value.first, value.second), writes(value.first, value.second, store) {
  let Pair(first: first_cell, second: second_cell) = move value;
  let first_value = deref(first_cell);
  let second_value = deref(second_cell);
  dispose second_cell;
  dispose first_cell;
  if first_value != 11_u64 {
    return exit_status(code: 1_u8);
  }
  if second_value != 22_u64 {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  let counter = 0_u64;
  region {
    match construct(store: &uniq heap, counter: &uniq counter) {
      Err(error: returned) => {
        let expected = counter *wrap 11_u64;
        if returned != expected {
          return exit_status(code: 3_u8);
        }
        return exit_status(code: 0_u8);
      }
      Ok(value: built) => {
        let prepared = prepare(value: move built, counter: &uniq counter);
        match heap_box(store: &uniq heap, value: move prepared) {
          Err(error: returned_pair) => {
            if counter != 3_u64 {
              return exit_status(code: 4_u8);
            }
            return finish(value: move returned_pair, store: &uniq heap);
          }
          Ok(value: outer_cell) => {
            if counter != 3_u64 {
              return exit_status(code: 5_u8);
            }
            let Box(value: returned_pair) = move outer_cell;
            return finish(value: move returned_pair, store: &uniq heap);
          }
        }
      }
    }
  }
}
"#,
    );
    // Substitute only the emitted allocator facilities. The host floor keeps
    // its own allocator, and all source ownership, branches, constructors,
    // returned error payloads, and derived cleanup remain the compiler's.
    // An external call with an observable log also prevents LLVM deleting an
    // allocation/free pair and silently losing a tested refusal edge.
    let observed = retain_calls(&module)
        .replace("@malloc(", "@wf_test_allocate(")
        .replace("@free(", "@wf_test_release(");
    assert!(observed.contains("@wf_test_allocate("));
    assert!(observed.contains("@wf_test_release("));
    for (refused, expected) in [
        (1, "X1;"),
        (2, "A1;X2;F1;"),
        (3, "A1;A2;X3;F2;F1;"),
        (0, "A1;A2;A3;F3;F2;F1;"),
    ] {
        let host = allocation_observer(3, refused);
        let output = compile_link_and_run(&observed, Some(&host), &[]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "refusal {refused}: {output:?}"
        );
        assert_eq!(output.stdout, expected.as_bytes(), "refusal {refused}");
        assert!(output.stderr.is_empty(), "refusal {refused}: {output:?}");
    }
}

#[test]
fn value_if_cleans_unchosen_owners_before_reusing_delivery_storage() {
    let module = compile(
        br#"struct Cell['s] {
  value: Box<'s, u64>;
}

fn choose['s](left: own Cell<'s>, right: own Cell<'s>, flag: own Bool, store: &uniq Heap<'s>) -> result: own u64 reads(left.value, right.value), writes(store) {
  let selected = if flag {
    let first = move left;
    let second = move right;
    give move first;
  } else {
    let first = move left;
    let second = move right;
    give move second;
  }
  return deref(selected.value);
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  for (round in 0_u64..2_u64) {
    match heap_box(store: &uniq heap, value: 11_u64) {
      Err(error: first_back) => {
        return exit_status(code: 1_u8);
      }
      Ok(value: first_cell) => {
        match heap_box(store: &uniq heap, value: 22_u64) {
          Err(error: second_back) => {
            return exit_status(code: 2_u8);
          }
          Ok(value: second_cell) => {
            let left = Cell(value: move first_cell);
            let right = Cell(value: move second_cell);
            let flag = round == 0_u64;
            let value = choose(left: move left, right: move right, flag: flag, store: &uniq heap);
            let expected = if flag {
              give 11_u64;
            } else {
              give 22_u64;
            }
            if value != expected {
              return exit_status(code: 3_u8);
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    let observed = retain_calls(&module)
        .replace("@malloc(", "@wf_test_allocate(")
        .replace("@free(", "@wf_test_release(");
    let host = allocation_observer(4, 0);
    let output = compile_link_and_run(&observed, Some(&host), &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"A1;A2;F2;F1;A3;A4;F3;F4;");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn addressed_owner_cleanup_releases_descriptor_fields_in_checked_order() {
    let module = compile(br#"struct Holder['s] {
  cell: Box<'s, u64>;
  bytes: buffer<u8>;
  stamp: u64;
}

fn touch(value: &uniq u64) -> result: own unit writes(value) {
  set deref(value) = 41_u64;
  return unit;
}

fn release['s](value: own Holder<'s>, store: &uniq Heap<'s>, early: own Bool) -> result: own u8 reads(value.stamp), writes(value.cell, value.bytes, value.stamp, store) {
  region {
    touch(value: &uniq value.stamp);
  }
  if value.stamp != 41_u64 {
    return 2_u8;
  }
  if early {
    dispose value;
    return 0_u8;
  }
  return 0_u8;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  for (round in 0_u64..2_u64) {
    match heap_box(store: &uniq heap, value: 17_u64) {
      Err(error: returned) => {
        return exit_status(code: 1_u8);
      }
      Ok(value: cell) => {
        let bytes = buffer_new(3_u64, 23_u8);
        let holder = Holder(cell: move cell, bytes: move bytes, stamp: 0_u64);
        let early = round == 0_u64;
        let status = release(value: move holder, store: &uniq heap, early: early);
        if status != 0_u8 {
          return exit_status(code: status);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#);
    let observed = retain_calls(&module)
        .replace("@malloc(", "@wf_test_allocate(")
        .replace("@free(", "@wf_test_release(");
    let host = allocation_observer(4, 0);
    let output = compile_link_and_run(&observed, Some(&host), &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    // PROV-6 visits fields in declaration order. Both explicit disposal and
    // scope exit must preserve the cell/descriptor identity after a real
    // borrowed helper call, without releasing either field twice.
    assert_eq!(output.stdout, b"A1;A2;F1;F2;A3;A4;F3;F4;");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn nested_and_residual_cleanup_preserve_release_graph_order() {
    let module = compile(
        br#"struct Pair {
  first: box<u64>;
  second: box<u64>;
}

struct Triple {
  first: box<u64>;
  selected: box<u64>;
  last: box<u64>;
}

enum Packet {
  Held(pair: Pair, extra: box<u64>);
  Empty();
}

command fn main() -> status: own ExitStatus pure {
  region {
    let first = box_new(11_u64);
    let second = box_new(22_u64);
    let pair = Pair(first: move first, second: move second);
    let extra = box_new(33_u64);
    let packet = Held(pair: move pair, extra: move extra);
  }
  region {
    let first = box_new(44_u64);
    let selected = box_new(55_u64);
    let last = box_new(66_u64);
    let triple = Triple(first: move first, selected: move selected, last: move last);
    let retained = move triple.selected;
    if deref(retained) != 55_u64 {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    let observed = retain_calls(&module)
        .replace("@malloc(", "@wf_test_allocate(")
        .replace("@free(", "@wf_test_release(");
    let host = allocation_observer(6, 0);
    let output = compile_link_and_run(&observed, Some(&host), &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    // The active payload walks its nested struct before the next payload
    // field. A partial consume releases only the residual fields, in their
    // declaration order; the selected owner remains live to scope exit.
    assert_eq!(output.stdout, b"A1;A2;A3;F1;F2;F3;A4;A5;A6;F4;F6;F5;");
    assert!(output.stderr.is_empty(), "{output:?}");
}

/// Observe only source allocations, without changing the host floor allocator.
/// A duplicate or unknown release aborts instead of allowing a use-after-free
/// to appear successful because its bytes happened to remain unchanged.
fn allocation_observer(limit: usize, refused: usize) -> String {
    let slots = limit + 1;
    format!(
        r#"#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

static void *held[{slots}];
static unsigned attempts;

void *wf_test_allocate(size_t size) {{
    unsigned id = ++attempts;
    if (id > {limit}) abort();
    if (id == {refused}) {{
        printf("X%u;", id);
        return NULL;
    }}
    void *allocation = malloc(size);
    if (allocation == NULL) abort();
    held[id] = allocation;
    printf("A%u;", id);
    return allocation;
}}

void wf_test_release(void *allocation) {{
    for (unsigned id = 1; id <= attempts && id <= {limit}; ++id) {{
        if (allocation != NULL && held[id] == allocation) {{
            held[id] = NULL;
            printf("F%u;", id);
            free(allocation);
            return;
        }}
    }}
    abort();
}}
"#
    )
}
