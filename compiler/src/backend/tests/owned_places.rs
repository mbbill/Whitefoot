//! Execute ownership transfers over inline aggregates. These cases observe
//! values and effects rather than the emitter's choice of instructions.

use super::{compile, compile_and_run, compile_link_and_run};

/// Retain the ordinary emitted call boundaries for a second execution. This
/// changes only optimization permission, so a passing inlined body cannot hide
/// a broken aggregate argument or result ABI.
pub(super) fn retain_calls(module: &str) -> String {
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
fn consumed_result_fields_preserve_padding_siblings_and_smaller_returns() {
    let source = br#"struct Row {
  left: u64;
  right: u64;
}

fn split(value: own Row, bias: own u64) -> (before: own u8, updated: own Row, after: own u16) reads(value.left, value.right), writes(value.left, value.right) {
  set value.left = value.left +wrap bias;
  set value.right = value.right +wrap 3_u64;
  return 7_u8, move value, 513_u16;
}

fn relay(value: own Row, bias: own u64) -> result: own Row reads(value.left, value.right), writes(value.left, value.right) {
  let (before, updated, after) = split(value: move value, bias: bias);
  let stamp = 0_u64;
  if before == 7_u8 {
    set stamp = stamp +wrap 1_u64;
  }
  if after == 513_u16 {
    set stamp = stamp +wrap 2_u64;
  }
  set updated.left = updated.left +wrap stamp;
  return move updated;
}

fn repeat(value: own Row, count: own u64) -> result: own Row reads(value.left, value.right), writes(value.left, value.right) {
  let before = 0_u8;
  let after = 0_u16;
  for (iteration in 0_u64..count) {
    set (before, value, after) = split(value: move value, bias: 5_u64);
    if before != 7_u8 {
      return Row(left: 0_u64, right: 0_u64);
    }
    if after != 513_u16 {
      return Row(left: 0_u64, right: 0_u64);
    }
  }
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  let input = Row(left: 11_u64, right: 29_u64);
  let result = relay(value: move input, bias: 5_u64);
  if result.left != 19_u64 {
    return exit_status(code: 1_u8);
  }
  if result.right != 32_u64 {
    return exit_status(code: 2_u8);
  }
  let empty_input = Row(left: 41_u64, right: 53_u64);
  let empty = repeat(value: move empty_input, count: 0_u64);
  if empty.left != 41_u64 {
    return exit_status(code: 3_u8);
  }
  if empty.right != 53_u64 {
    return exit_status(code: 4_u8);
  }
  let loop_input = Row(left: 41_u64, right: 53_u64);
  let loop_result = repeat(value: move loop_input, count: 3_u64);
  if loop_result.left != 56_u64 {
    return exit_status(code: 5_u8);
  }
  if loop_result.right != 62_u64 {
    return exit_status(code: 6_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        // Retaining calls makes the callee write the complete padded result
        // before its caller extracts the middle field and the later sibling.
        // relay returns only Row, whose output is smaller than split's tuple.
        assert_success(&module);
        assert_success(&retain_calls(&module));
    }
}

#[test]
fn replaced_array_snapshots_remain_independent_through_multi_result_calls() {
    // Arrays are affine even with copy elements. Replacement supplies a valid
    // independent old value while leaving the containing packet writable.
    let source = br#"struct Packet {
  items: array<u64, 2>;
  stamp: u64;
}

fn split(items: own array<u64, 2>) -> (before: own u8, updated: own array<u64, 2>, after: own u16) writes(items) {
  set items[0_u64] = 17_u64;
  return 7_u8, move items, 513_u16;
}

command fn main() -> status: own ExitStatus pure {
  let original = array_new::<u64, 2>(29_u64);
  let packet = Packet(items: move original, stamp: 97_u64);
  let replacement = array_new::<u64, 2>(61_u64);
  let extracted = replace packet.items = move replacement;
  let (before, updated, after) = split(items: move extracted);
  set packet.items[0_u64] = 83_u64;
  if before != 7_u8 {
    return exit_status(code: 1_u8);
  }
  if after != 513_u16 {
    return exit_status(code: 2_u8);
  }
  if updated[0_u64] != 17_u64 {
    return exit_status(code: 3_u8);
  }
  if updated[1_u64] != 29_u64 {
    return exit_status(code: 4_u8);
  }
  if packet.items[0_u64] != 83_u64 {
    return exit_status(code: 5_u8);
  }
  let next = array_new::<u64, 2>(43_u64);
  let snapshot = replace packet.items = move next;
  set packet.items[1_u64] = 107_u64;
  if snapshot[0_u64] != 83_u64 {
    return exit_status(code: 6_u8);
  }
  if snapshot[1_u64] != 61_u64 {
    return exit_status(code: 7_u8);
  }
  if packet.items[1_u64] != 107_u64 {
    return exit_status(code: 8_u8);
  }
  if packet.stamp != 97_u64 {
    return exit_status(code: 9_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        assert_success(&module);
        assert_success(&retain_calls(&module));
    }
}

#[test]
fn indexed_child_reborrows_update_only_the_selected_field() {
    let source = br#"struct Point {
  x: u64;
  y: u64;
}

fn write(value: &uniq u64) -> result: own unit writes(value) {
  set deref(value) = 7_u64;
  return unit;
}

fn update(points: &uniq array<Point, 2>, index: own u64) -> result: own unit writes(points) contract {
  requires index < 2_u64;
} {
  region {
    write(value: &uniq deref(points)[index].x);
    write(value: &uniq deref(points)[index].x);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let first = Point(x: 17_u64, y: 29_u64);
  let second = Point(x: 41_u64, y: 53_u64);
  let empty = fixed_vector::<Point, 2>();
  let one = place_back(vector: move empty, value: move first);
  let full = place_back(vector: move one, value: move second);
  let points = array_from_fixed(vector: move full);
  region {
    update(points: &uniq points, index: 1_u64);
  }
  if points[0_u64].x != 17_u64 {
    return exit_status(code: 1_u8);
  }
  if points[0_u64].y != 29_u64 {
    return exit_status(code: 2_u8);
  }
  if points[1_u64].x != 7_u64 {
    return exit_status(code: 3_u8);
  }
  if points[1_u64].y != 53_u64 {
    return exit_status(code: 4_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        assert_success(&module);
        assert_success(&retain_calls(&module));
    }
}

#[test]
fn loop_owner_sources_cover_later_iterations_and_counted_exhaustion() {
    let source = br#"fn rotate(first: own box<u64>, second: own box<u64>, count: own u64) -> result: own u64 reads(first, second), writes(first, second) {
  for (iteration in 0_u64..count) {
    set (first, second) = move second, move first;
  }
  return deref(first);
}

fn once(first: own box<u64>, second: own box<u64>) -> result: own u64 reads(first, second), writes(first, second) {
  let repeat = True();
  loop {
    let observed = deref(first);
    if repeat {
      set repeat = False();
      set (first, second) = move second, move first;
    } else {
      break;
    }
  }
  return deref(first);
}

command fn main() -> status: own ExitStatus pure {
  let a = box_new(17_u64);
  let b = box_new(29_u64);
  let zero = rotate(first: move a, second: move b, count: 0_u64);
  let c = box_new(17_u64);
  let d = box_new(29_u64);
  let one = rotate(first: move c, second: move d, count: 1_u64);
  let e = box_new(17_u64);
  let f = box_new(29_u64);
  let two = rotate(first: move e, second: move f, count: 2_u64);
  let g = box_new(17_u64);
  let h = box_new(29_u64);
  let ordinary = once(first: move g, second: move h);
  if zero != 17_u64 {
    return exit_status(code: 1_u8);
  }
  if one != 29_u64 {
    return exit_status(code: 2_u8);
  }
  if two != 17_u64 {
    return exit_status(code: 3_u8);
  }
  if ordinary != 29_u64 {
    return exit_status(code: 4_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        assert_success(&module);
        assert_success(&retain_calls(&module));
    }
}

#[test]
fn wide_result_returns_preserve_success_refusal_and_owned_children() {
    let source = br#"struct Record {
  words: array<u64, 512>;
  first: box<u64>;
  second: box<u64>;
}

fn make_record(seed: own u64) -> result: own Result<Record, u8> pure {
  let first = box_new(seed);
  if seed == 0_u64 {
    return Err<Record, u8>(error: 1_u8);
  }
  let words = array_new::<u64, 512>(seed);
  let second = box_new(29_u64);
  let record = Record(words: move words, first: move first, second: move second);
  return Ok<Record, u8>(value: move record);
}

fn relay(seed: own u64) -> result: own Result<Record, u8> pure {
  return make_record(seed: seed);
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_vector::<Record>(store: &uniq heap, count: 1_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: reserved) => {
        match relay(seed: 0_u64) {
          Err(error: code) => {
            if code != 1_u8 {
              return exit_status(code: 2_u8);
            }
          }
          Ok(value: unexpected) => {
            return exit_status(code: 3_u8);
          }
        }
        match relay(seed: 17_u64) {
          Err(error: code) => {
            return exit_status(code: 4_u8);
          }
          Ok(value: made) => {
            let filled = place_back(vector: move reserved, value: move made);
            let (empty, observed) = take_back(vector: move filled);
            if observed.words[0_u64] != 17_u64 {
              return exit_status(code: 5_u8);
            }
            if observed.words[511_u64] != 17_u64 {
              return exit_status(code: 6_u8);
            }
            if deref(observed.first) != 17_u64 {
              return exit_status(code: 7_u8);
            }
            if deref(observed.second) != 29_u64 {
              return exit_status(code: 8_u8);
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
        for refused in [0, 1] {
            let host = allocation_observer(4, refused);
            let output = compile_link_and_run(&observed, Some(&host), &[]);
            let (status, ledger): (i32, &[u8]) = if refused == 0 {
                // Failure cleans the first child. Success moves both children
                // through Result and the reserved run, releasing each once.
                (0, b"A1;A2;F2;A3;A4;F3;F4;F1;")
            } else {
                // Refusing the reservation does not evaluate the producer.
                (70, b"X1;")
            };
            assert_eq!(output.status.code(), Some(status), "{output:?}");
            assert_eq!(output.stdout, ledger, "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn competing_wide_returns_preserve_live_and_borrowed_contents() {
    let source = br#"fn choose_live(seed: own u64) -> result: own array<u64, 512> pure {
  let original = array_new::<u64, 512>(seed);
  if seed == 0_u64 {
    return move original;
  }
  let candidate = array_new::<u64, 512>(37_u64);
  if original[511_u64] != seed {
    return move original;
  }
  return move candidate;
}

fn choose_borrowed(seed: own u64) -> result: own array<u64, 512> pure {
  let original = array_new::<u64, 512>(seed);
  if seed == 0_u64 {
    return move original;
  }
  region {
    let reader = slice_of(&original);
    let candidate = array_new::<u64, 512>(43_u64);
    if reader[511_u64] != seed {
      return array_new::<u64, 512>(99_u64);
    }
    return move candidate;
  }
}

command fn main() -> status: own ExitStatus pure {
  let first = choose_live(seed: 0_u64);
  let second = choose_live(seed: 17_u64);
  let third = choose_borrowed(seed: 0_u64);
  let fourth = choose_borrowed(seed: 19_u64);
  if first[0_u64] != 0_u64 {
    return exit_status(code: 1_u8);
  }
  if first[511_u64] != 0_u64 {
    return exit_status(code: 2_u8);
  }
  if second[0_u64] != 37_u64 {
    return exit_status(code: 3_u8);
  }
  if second[511_u64] != 37_u64 {
    return exit_status(code: 4_u8);
  }
  if third[0_u64] != 0_u64 {
    return exit_status(code: 5_u8);
  }
  if third[511_u64] != 0_u64 {
    return exit_status(code: 6_u8);
  }
  if fourth[0_u64] != 43_u64 {
    return exit_status(code: 7_u8);
  }
  if fourth[511_u64] != 43_u64 {
    return exit_status(code: 8_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        assert_success(&retain_calls(&module));
    }
}

#[test]
fn an_owned_parameter_uses_same_or_distinct_result_storage_after_entry_transfer() {
    let source = br#"fn append(items: own FixedVector<u64, 16>, value: own u64, watch: &u64) -> updated: own FixedVector<u64, 16> reads(items, watch), writes(items) contract {
  requires room_of(items) >= 1_u64;
  ensures len_of(updated) == len_of(items) + 1_u64;
} {
  let bias = deref(watch);
  let adjusted = value +wrap bias;
  let filled = place_back(vector: move items, value: adjusted);
  return move filled;
}

command fn main() -> status: own ExitStatus pure {
  let watch = 5_u64;
  let same = fixed_vector::<u64, 16>();
  region {
    set same = append(items: move same, value: 12_u64, watch: &watch);
    let vacant = fixed_vector::<u64, 16>();
    let distinct = append(items: move vacant, value: 24_u64, watch: &watch);
    let same_count = len_of(same);
    let distinct_count = len_of(distinct);
    if same_count != 1_u64 {
      return exit_status(code: 1_u8);
    }
    if distinct_count != 1_u64 {
      return exit_status(code: 2_u8);
    }
    if same[0_u64] != 17_u64 {
      return exit_status(code: 3_u8);
    }
    if distinct[0_u64] != 29_u64 {
      return exit_status(code: 4_u8);
    }
    if watch != 5_u64 {
      return exit_status(code: 5_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let module = super::emit_lowered(source, super::OverlapLowering::Off);
    let append = super::emitted_function(&module, "append");
    assert!(!append.contains("alloca"), "{append}");
    assert_eq!(append.matches("call void @llvm.memmove.").count(), 1);
    let main = super::emitted_function(&module, "main");
    let calls = main
        .lines()
        .filter_map(|line| line.split_once("@wf_append(").map(|(_, tail)| tail))
        .map(|arguments| {
            let mut arguments = arguments.split(',').map(str::trim);
            let result = arguments.next().expect("result pointer");
            let input = arguments.next().expect("input pointer");
            result == input
        })
        .collect::<Vec<_>>();
    assert!(
        calls.contains(&true),
        "exercise a consumed input/result alias"
    );
    assert!(
        calls.contains(&false),
        "exercise an independent result destination"
    );
    assert_success(&retain_calls(&module));
}

#[test]
fn returned_parameter_snapshots_precede_result_alias_writes() {
    let source = br#"struct Row {
  value: u64;
}

struct Holder {
  row: Row;
}

fn discard(value: own Row) -> result: own unit pure {
  return unit;
}

fn choose(left: own Row, right: own Row, watch: &u64) -> result: own Row reads(right.value, watch) {
  let expected = deref(watch);
  if right.value != expected {
    let ignored = discard(value: move left);
    return Row(value: 99_u64);
  }
  return move left;
}

fn relay(held: own Row, watch: &u64, offered: own u64) -> result: own Row reads(held.value, watch) {
  let row = Row(value: offered);
  let fresh = Holder(row: move row);
  return choose(left: move fresh.row, right: move held, watch: watch);
}

command fn main() -> status: own ExitStatus pure {
  let watch = 29_u64;
  let first_input = Row(value: 29_u64);
  let second_input = Row(value: 31_u64);
  region {
    let first = relay(held: move first_input, watch: &watch, offered: 11_u64);
    let second = relay(held: move second_input, watch: &watch, offered: 17_u64);
    if first.value != 11_u64 {
      return exit_status(code: 1_u8);
    }
    if second.value != 99_u64 {
      return exit_status(code: 2_u8);
    }
    if watch != 29_u64 {
      return exit_status(code: 3_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        // The call inside relay can place its result in its consumed second
        // input's backing while choose returns its first input. Retained calls
        // keep snapshot ordering observable: save the second input privately
        // before the first input initializes result storage, or the branch
        // changes and returns 99 instead of 11.
        assert_success(&retain_calls(&module));
    }
}

#[test]
fn zero_sized_aggregate_replacement_preserves_adjacent_fields() {
    let source = br#"struct Envelope {
  before: u64;
  empty: array<u64, 0>;
  after: u64;
}

fn replace_empty(target: &uniq Envelope, value: own array<u64, 0>) -> result: own unit reads(target.empty), writes(target.empty) {
  let previous = replace deref(target).empty = move value;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let empty = array_new::<u64, 0>(0_u64);
  let envelope = Envelope(before: 17_u64, empty: move empty, after: 29_u64);
  let replacement = array_new::<u64, 0>(43_u64);
  region {
    let ignored = replace_empty(target: &uniq envelope, value: move replacement);
  }
  if envelope.before != 17_u64 {
    return exit_status(code: 1_u8);
  }
  if envelope.after != 29_u64 {
    return exit_status(code: 2_u8);
  }
  let size = len_of(envelope.empty);
  if size != 0_u64 {
    return exit_status(code: 3_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        assert_success(&retain_calls(&module));
    }
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
fn region_polymorphic_box_calls_preserve_each_independent_release_class() {
    let source = br#"struct Holder['s] {
  cell: Box<'s, u64>;
}

struct Reversed['l, 'r] {
  first: Holder<'r>;
  second: Holder<'l>;
}

fn pass<T: linear>(value: own T) -> result: own T pure {
  return move value;
}

fn identity['s](held: own Holder<'s>) -> result: own Holder<'s> pure {
  return pass::<Holder<'s>>(value: move held);
}

fn reverse['l, 'r](left: own Holder<'l>, right: own Holder<'r>) -> result: own Reversed<'l, 'r> pure {
  return Reversed(first: move right, second: move left);
}

fn reverse_results['l, 'r](left: own Holder<'l>, right: own Holder<'r>) -> (first: own Holder<'r>, second: own Holder<'l>) pure {
  return move right, move left;
}

fn extract['s](cell: own Box<'s, u64>, witness: &Box<'s, u64>) -> value: own u64 pure {
  let Box(value: value) = move cell;
  return value;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region 'a {
    let store = arena_frame::<16, 8, 'a>();
    region {
      match heap_box(store: &uniq heap, value: 11_u64) {
        Err(error: back) => {
          return exit_status(code: 70_u8);
        }
        Ok(value: general) => {
          match heap_box(store: &uniq heap, value: 44_u64) {
            Err(error: back) => {
              return exit_status(code: 70_u8);
            }
            Ok(value: general_witness) => {
              match arena_box(store: &uniq store, value: 22_u64) {
                Err(error: back) => {
                  return exit_status(code: 70_u8);
                }
                Ok(value: extent) => {
                  match arena_box(store: &uniq store, value: 33_u64) {
                    Err(error: back) => {
                      return exit_status(code: 70_u8);
                    }
                    Ok(value: extent_witness) => {
                      let left_source = Holder(cell: move general);
                      let right_source = Holder(cell: move extent);
                      let left = identity(held: move left_source);
                      let right = identity(held: move right_source);
                      let reversed = reverse(left: move left, right: move right);
                      let Reversed(first: first, second: second) = move reversed;
                      let Holder(cell: extent_cell) = move first;
                      let Holder(cell: general_cell) = move second;
                      region {
                        let value = extract(cell: move general_cell, witness: &general_witness);
                        if value != 11_u64 {
                          return exit_status(code: 1_u8);
                        }
                      }
                      region {
                        let value = extract(cell: move extent_cell, witness: &extent_witness);
                        if value != 22_u64 {
                          return exit_status(code: 2_u8);
                        }
                      }
                      let remaining_left = Holder(cell: move extent_witness);
                      let remaining_right = Holder(cell: move general_witness);
                      let remaining = reverse(left: move remaining_left, right: move remaining_right);
                      let Reversed(first: remaining_first, second: remaining_second) = move remaining;
                      if deref(remaining_first.cell) != 44_u64 {
                        return exit_status(code: 3_u8);
                      }
                      if deref(remaining_second.cell) != 33_u64 {
                        return exit_status(code: 4_u8);
                      }
                      let (extent_result, general_result) = reverse_results(left: move remaining_first, right: move remaining_second);
                      let (general_back, extent_back) = reverse_results(left: move extent_result, right: move general_result);
                      if deref(general_back.cell) != 44_u64 {
                        return exit_status(code: 5_u8);
                      }
                      if deref(extent_back.cell) != 33_u64 {
                        return exit_status(code: 6_u8);
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
        for (refused, status, expected) in
            [(0, 0, "A1;A2;F1;F2;"), (1, 70, "X1;"), (2, 70, "A1;X2;F1;")]
        {
            let host = allocation_observer(2, refused);
            let output = compile_link_and_run(&observed, Some(&host), &[]);
            assert_eq!(output.status.code(), Some(status), "{output:?}");
            // The two calls reverse the actual classes of independent
            // formal regions. Explicit destructuring releases the first
            // general cell; neither extent cell may reach the host free.
            assert_eq!(output.stdout, expected.as_bytes(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn ordinary_box_owner_transfer_keeps_values_and_release_across_two_helpers() {
    let source = br#"struct Observed {
  previous: u64;
  current: u64;
}

fn exchange['s](slot: &uniq Box<'s, u64>, incoming: own Box<'s, u64>) -> previous: own Box<'s, u64> reads(slot), writes(slot) {
  let displaced = replace deref(slot) = move incoming;
  return move displaced;
}

fn observe['s](owner: own Box<'s, u64>, incoming: own Box<'s, u64>, store: &uniq Heap<'s>) -> result: own Observed reads(owner, incoming), writes(owner, store) {
  let previous_value = 0_u64;
  region {
    let previous = exchange(slot: &uniq owner, incoming: move incoming);
    set previous_value = deref(previous);
  }
  let current_value = deref(owner);
  return Observed(previous: previous_value, current: current_value);
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 11_u64) {
      Err(error: back) => {
        return exit_status(code: 70_u8);
      }
      Ok(value: owner) => {
        match heap_box(store: &uniq heap, value: 22_u64) {
          Err(error: back) => {
            return exit_status(code: 71_u8);
          }
          Ok(value: incoming) => {
            let seen = observe(owner: move owner, incoming: move incoming, store: &uniq heap);
            if seen.previous != 11_u64 {
              return exit_status(code: 1_u8);
            }
            if seen.current != 22_u64 {
              return exit_status(code: 2_u8);
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
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
pub(super) fn allocation_observer(limit: usize, refused: usize) -> String {
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

/// A full bounded append returns its input rather than overwriting an element.
/// The same helper updates heap and extent cells without replacing either cell.
#[test]
fn boxed_fixed_vector_read_out_preserves_storage_and_elements() {
    let source = br#"struct Checked['s] {
  storage: Box<'s, FixedVector<box<u64>, 2>>;
  code: u8;
}

fn append['s](storage: own Box<'s, FixedVector<box<u64>, 2>>, value: own box<u64>) -> result: own Box<'s, FixedVector<box<u64>, 2>> reads(storage), writes(storage) contract {
  requires room_of(deref(storage)) > 0_u64;
} {
  set deref(storage) = place_back(vector: move deref(storage), value: move value);
  return move storage;
}

fn try_append['s](storage: own Box<'s, FixedVector<box<u64>, 2>>, value: own box<u64>) -> (result: own Box<'s, FixedVector<box<u64>, 2>>, returned: own Option<box<u64>>) reads(storage), writes(storage) {
  let room = room_of(deref(storage));
  if room > 0_u64 {
    let updated = append(storage: move storage, value: move value);
    return move updated, None<box<u64>>();
  }
  return move storage, Some<box<u64>>(value: move value);
}

fn inspect(values: own FixedVector<box<u64>, 2>) -> code: own u8 reads(values), writes(values) {
  let length = len_of(values);
  if length != 2_u64 {
    return 4_u8;
  }
  let (one, second) = take_back(vector: move values);
  let (empty, first) = take_back(vector: move one);
  if deref(first) != 17_u64 {
    return 5_u8;
  }
  if deref(second) != 29_u64 {
    return 6_u8;
  }
  return 0_u8;
}

fn exercise['s](storage: own Box<'s, FixedVector<box<u64>, 2>>) -> result: own Checked<'s> reads(storage), writes(storage) {
  let first = box_new(17_u64);
  let (one, first_returned) = try_append(storage: move storage, value: move first);
  match first_returned {
    None() => {
    }
    Some(value: unwanted) => {
      return Checked(storage: move one, code: 1_u8);
    }
  }
  let second = box_new(29_u64);
  let (two, second_returned) = try_append(storage: move one, value: move second);
  match second_returned {
    None() => {
    }
    Some(value: unwanted) => {
      return Checked(storage: move two, code: 2_u8);
    }
  }
  let third = box_new(41_u64);
  let (full, third_returned) = try_append(storage: move two, value: move third);
  match third_returned {
    None() => {
      return Checked(storage: move full, code: 3_u8);
    }
    Some(value: refused) => {
      if deref(refused) != 41_u64 {
        return Checked(storage: move full, code: 7_u8);
      }
      return Checked(storage: move full, code: 0_u8);
    }
  }
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region 'a {
    let store = arena_frame::<128, 16, 'a>();
    let empty_heap = fixed_vector::<box<u64>, 2>();
    region {
      match heap_box(store: &uniq heap, value: move empty_heap) {
        Err(error: back) => {
          return exit_status(code: 70_u8);
        }
        Ok(value: heap_cell) => {
          let heap_result = exercise(storage: move heap_cell);
          let Checked(storage: heap_storage, code: heap_code) = move heap_result;
          if heap_code != 0_u8 {
            return exit_status(code: heap_code);
          }
          let Box(value: heap_values) = move heap_storage;
          let heap_inspected = inspect(values: move heap_values);
          if heap_inspected != 0_u8 {
            return exit_status(code: heap_inspected);
          }
          let empty_arena = fixed_vector::<box<u64>, 2>();
          match arena_box(store: &uniq store, value: move empty_arena) {
            Err(error: back) => {
              return exit_status(code: 70_u8);
            }
            Ok(value: arena_cell) => {
              let arena_result = exercise(storage: move arena_cell);
              let Checked(storage: arena_storage, code: arena_code) = move arena_result;
              if arena_code != 0_u8 {
                return exit_status(code: arena_code);
              }
              let Box(value: arena_values) = move arena_storage;
              let arena_inspected = inspect(values: move arena_values);
              return exit_status(code: arena_inspected);
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
        for (refused, status, expected) in [
            (0, 0, "A1;A2;A3;A4;F4;F1;F2;F3;A5;A6;A7;F7;F5;F6;"),
            (1, 70, "X1;"),
        ] {
            let host = allocation_observer(7, refused);
            let output = compile_link_and_run(&observed, Some(&host), &[]);
            assert_eq!(output.status.code(), Some(status), "{output:?}");
            // Refused elements leave first. Destructuring releases only the
            // heap cell's backing; inspecting consumes both remaining elements.
            assert_eq!(output.stdout, expected.as_bytes(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn statement_children_keep_displaced_owners_and_provider_results_alive() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/own6-pos-statement-children-use-a-longer-local-region.wf"
    );
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        assert_success(&module);
        assert_success(&retain_calls(&module));
    }
}
