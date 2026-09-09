use super::{compile, compile_and_run, compile_rejection, emitted_function};

#[test]
fn constant_typed_places_execute_shared_calls_and_nested_projections() {
    let source = br#"struct Entry {
  tag: u64;
  samples: array<array<u64, 2>, 2>;
}

const entries: array<Entry, 2> =[Entry(tag: 17_u64, samples:[[19_u64, 23_u64],[29_u64, 31_u64]]), Entry(tag: 37_u64, samples:[[41_u64, 43_u64],[47_u64, 53_u64]])];

const entry: Entry = Entry(tag: 59_u64, samples:[[61_u64, 67_u64],[71_u64, 73_u64]]);

fn retain['r](values: &'r array<Entry, 2>) -> result: &'r array<Entry, 2> pure {
  return values;
}

fn read(values: &array<Entry, 2>, outer: own u64, row: own u64, column: own u64) -> result: own u64 reads(values) contract {
  requires outer < 2_u64;
  requires row < 2_u64;
  requires column < 2_u64;
} {
  return deref(values)[outer].samples[row][column];
}

fn read_row(values: &array<u64, 2>, index: own u64) -> result: own u64 reads(values) contract {
  requires index < 2_u64;
} {
  return deref(values)[index];
}

fn read_entry(value: &Entry) -> result: own u64 reads(value.tag) {
  return deref(value).tag;
}

command fn main() -> status: own ExitStatus pure {
  if entries[1_u64].samples[1_u64][1_u64] != 53_u64 {
    return exit_status(code: 1_u8);
  }
  if entry.tag != 59_u64 {
    return exit_status(code: 2_u8);
  }
  let length = len_of(entry.samples[1_u64]);
  if length != 2_u64 {
    return exit_status(code: 3_u8);
  }
  region 'outer {
    region {
      let shared = retain(values: &'outer entries);
      let first = read(values: shared, outer: 0_u64, row: 0_u64, column: 1_u64);
      let last = read(values: shared, outer: 1_u64, row: 1_u64, column: 1_u64);
      if first != 23_u64 {
        return exit_status(code: 4_u8);
      }
      if last != 53_u64 {
        return exit_status(code: 5_u8);
      }
      let projected = read_row(values: &entry.samples[1_u64], index: 0_u64);
      if projected != 71_u64 {
        return exit_status(code: 6_u8);
      }
      let tag = read_entry(value: &entry);
      if tag != 59_u64 {
        return exit_status(code: 7_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    // Independent calls with storage unavailable to the WF optimizer retain
    // even the identity helper's returned-pointer ABI. The WF caller above
    // separately supplies actual immutable globals through the same helpers.
    let observer = r#"#include <stdint.h>
#include <stdlib.h>
struct Entry { uint64_t tag; uint64_t samples[2][2]; };
extern const struct Entry *wf_retain(const struct Entry *);
extern uint64_t wf_read(const struct Entry *, uint64_t, uint64_t, uint64_t);
extern uint64_t wf_read_row(const uint64_t *, uint64_t);
extern uint64_t wf_read_entry(const struct Entry *);
__attribute__((constructor)) static void check_shared_abi(void) {
    const struct Entry values[2] = {
        {101, {{103, 107}, {109, 113}}},
        {127, {{131, 137}, {139, 149}}}
    };
    const struct Entry *retained = wf_retain(values);
    if (retained != values) exit(81);
    if (wf_read(retained, 1, 1, 0) != 139) exit(82);
    if (wf_read_row(values[0].samples[1], 1) != 113) exit(83);
    if (wf_read_entry(&values[1]) != 127) exit(84);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        // Keep an externally callable pointer ABI as well as call boundaries:
        // noinline alone still lets IPSCCP specialize an internal helper to
        // this caller's one constant global and remove its pointer argument.
        let module = retain_nested_run_calls(&super::emit_lowered(source, overlap))
            .replace("define internal ", "define ");
        let output = super::compile_link_and_run(&module, Some(observer), &[]);
        assert_eq!(output.status.code(), Some(0), "{overlap:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn nested_constant_arrays_emit_complete_recursive_global_layouts() {
    let source = br#"struct Entry {
  tag: u64;
  samples: array<u64, 2>;
}

const rows: array<array<u64, 2>, 2> =[[7_u64, 9_u64],[11_u64, 13_u64]];

const entries: array<Entry, 2> =[Entry(tag: 17_u64, samples:[19_u64, 23_u64]), Entry(tag: 29_u64, samples:[31_u64, 37_u64])];

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // This observes recursive global layout independently of the WF access
    // test above, using the host's ordinary C array and struct layout.
    let module = compile(source)
        .replace("@.wf_const.0", "@wf_test_rows")
        .replace("@.wf_const.1", "@wf_test_entries")
        .replace("= private unnamed_addr constant", "= constant");
    let observer = r#"#include <stdint.h>
#include <stdlib.h>
struct Entry { uint64_t tag; uint64_t samples[2]; };
extern const uint64_t wf_test_rows[2][2];
extern const struct Entry wf_test_entries[2];
__attribute__((constructor)) static void check_global_layout(void) {
    if (wf_test_rows[0][0] != 7 || wf_test_rows[0][1] != 9 ||
        wf_test_rows[1][0] != 11 || wf_test_rows[1][1] != 13) exit(81);
    if (wf_test_entries[0].tag != 17 || wf_test_entries[0].samples[0] != 19 ||
        wf_test_entries[0].samples[1] != 23 || wf_test_entries[1].tag != 29 ||
        wf_test_entries[1].samples[0] != 31 || wf_test_entries[1].samples[1] != 37) exit(82);
}
"#;
    let output = super::compile_link_and_run(&module, Some(observer), &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn full_owning_arrays_preserve_wrapped_order_replacement_and_exact_cleanup() {
    let source = br#"struct Record {
  payload: array<u64, 16>;
  owner: box<u64>;
}

fn make_record(tag: own u64) -> result: own Record pure {
  let payload = array_new::<u64, 16>(tag);
  let owner = box_new(tag);
  return Record(payload: move payload, owner: move owner);
}

fn seal(values: own FixedVector<Record, 3>) -> result: own array<Record, 3> reads(values) contract {
  requires len_of(values) == 3_u64;
} {
  return array_from_fixed(vector: move values);
}

fn reopen(values: own array<Record, 3>) -> result: own FixedVector<Record, 3> pure contract {
  ensures len_of(result) == 3_u64;
  ensures head_of(result) == 0_u64;
} {
  let full = fixed_from_array(values: move values);
  return move full;
}

fn relay<T: affine>(values: own T) -> result: own T pure {
  return move values;
}

fn read(values: &array<Record, 3>, index: own u64) -> result: own u64 reads(values) contract {
  requires index < 3_u64;
} {
  return deref(values)[index].payload[7_u64];
}

command fn main() -> status: own ExitStatus pure {
  let first = make_record(tag: 11_u64);
  let second_tag = first.payload[0_u64] +wrap 11_u64;
  let second = make_record(tag: second_tag);
  let third_tag = second.payload[0_u64] +wrap 11_u64;
  let third = make_record(tag: third_tag);
  let empty = fixed_vector::<Record, 3>();
  let one = place_back(vector: move empty, value: move first);
  let two = place_back(vector: move one, value: move second);
  let wrapped = place_front(vector: move two, value: move third);
  let values = seal(values: move wrapped);
  region {
    let a = read(values: &values, index: 0_u64);
    let b = read(values: &values, index: 1_u64);
    let c = read(values: &values, index: 2_u64);
    if a != 33_u64 {
      return exit_status(code: 1_u8);
    }
    if b != 11_u64 {
      return exit_status(code: 1_u8);
    }
    if c != 22_u64 {
      return exit_status(code: 1_u8);
    }
  }
  let old = replace values[1_u64] = make_record(tag: 99_u64);
  let Record(payload: old_payload, owner: old_owner) = move old;
  if old_payload[7_u64] != 11_u64 {
    return exit_status(code: 2_u8);
  }
  if deref(old_owner) != 11_u64 {
    return exit_status(code: 2_u8);
  }
  let passed = relay::<array<Record, 3>>(values: move values);
  let full = reopen(values: move passed);
  let dense = seal(values: move full);
  region {
    let a = read(values: &dense, index: 0_u64);
    let b = read(values: &dense, index: 1_u64);
    let c = read(values: &dense, index: 2_u64);
    if a != 33_u64 {
      return exit_status(code: 3_u8);
    }
    if b != 99_u64 {
      return exit_status(code: 3_u8);
    }
    if c != 22_u64 {
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
        let observed = retain_nested_run_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        let host = super::owned_places::allocation_observer(4, 0);
        let output = super::compile_link_and_run(&observed, Some(&host), &[]);
        assert_eq!(output.status.code(), Some(0), "{overlap:?}: {output:?}");
        assert_eq!(
            output.stdout, b"A1;A2;A3;A4;F3;F4;F2;F1;",
            "{overlap:?}: {output:?}"
        );
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn fallible_full_array_construction_releases_each_initialized_prefix_once() {
    let source = br#"struct Record['s] {
  tag: u64;
  owner: Box<'s, u64>;
}

fn relay<T: affine>(value: own T) -> result: own T pure {
  return move value;
}

fn build['s](store: &uniq Heap<'s>) -> result: own Result<array<Record<'s>, 3>, u64> reads(store), writes(store), allocates(store) {
  let empty = fixed_vector::<Record<'s>, 3>();
  region {
    match heap_box(store: &uniq deref(store), value: 11_u64) {
      Err(error: back) => {
        return Err<array<Record<'s>, 3>, u64>(error: back);
      }
      Ok(value: first) => {
        let first_record = Record(tag: 11_u64, owner: move first);
        let one = place_back(vector: move empty, value: move first_record);
        region {
          match heap_box(store: &uniq deref(store), value: 22_u64) {
            Err(error: back) => {
              return Err<array<Record<'s>, 3>, u64>(error: back);
            }
            Ok(value: second) => {
              let second_record = Record(tag: 22_u64, owner: move second);
              let two = place_back(vector: move one, value: move second_record);
              region {
                match heap_box(store: &uniq deref(store), value: 33_u64) {
                  Err(error: back) => {
                    return Err<array<Record<'s>, 3>, u64>(error: back);
                  }
                  Ok(value: third) => {
                    let third_record = Record(tag: 33_u64, owner: move third);
                    let full = place_back(vector: move two, value: move third_record);
                    let values = array_from_fixed(vector: move full);
                    let passed = relay::<array<Record<'s>, 3>>(value: move values);
                    return Ok<array<Record<'s>, 3>, u64>(value: move passed);
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

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match build(store: &uniq heap) {
      Err(error: refused) => {
        if refused == 11_u64 {
          return exit_status(code: 71_u8);
        }
        if refused == 22_u64 {
          return exit_status(code: 72_u8);
        }
        if refused == 33_u64 {
          return exit_status(code: 73_u8);
        }
        return exit_status(code: 74_u8);
      }
      Ok(value: values) => {
        if values[0_u64].tag != 11_u64 {
          return exit_status(code: 1_u8);
        }
        if values[1_u64].tag != 22_u64 {
          return exit_status(code: 2_u8);
        }
        if values[2_u64].tag != 33_u64 {
          return exit_status(code: 3_u8);
        }
        return exit_status(code: 0_u8);
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
        let observed = retain_nested_run_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        // Every next allocation uses the same exclusive provider after the
        // preceding result is matched, so allocation IDs identify source slots.
        for (refused, status, ledger) in [
            (0, 0, b"A1;A2;A3;F1;F2;F3;".as_slice()),
            (1, 71, b"X1;".as_slice()),
            (2, 72, b"A1;X2;F1;".as_slice()),
            (3, 73, b"A1;A2;X3;F1;F2;".as_slice()),
        ] {
            let host = super::owned_places::allocation_observer(3, refused);
            let output = super::compile_link_and_run(&observed, Some(&host), &[]);
            assert_eq!(
                output.status.code(),
                Some(status),
                "{overlap:?}: {output:?}"
            );
            assert_eq!(output.stdout, ledger, "{overlap:?}: {output:?}");
            assert!(output.stderr.is_empty(), "{overlap:?}: {output:?}");
        }
    }
}

/// Counts the stack slots one emitted function declares, and how many of those
/// declarations sit outside its entry block.
///
/// A slot declared anywhere else is reached once per execution of its block, so
/// a slot inside a loop grows the frame once per iteration. The emitter must
/// therefore keep every declaration in the entry block, which runs exactly once
/// per call, and leave only the store at the use site.
fn slot_declarations(function: &str) -> (usize, usize) {
    let mut in_entry = false;
    let mut total = 0;
    let mut outside_entry = 0;
    for line in function.lines() {
        if let Some(label) = line.strip_suffix(':')
            && !label.starts_with(' ')
        {
            in_entry = label == "entry";
            continue;
        }
        if line.contains(" = alloca ") {
            total += 1;
            if !in_entry {
                outside_entry += 1;
            }
        }
    }
    (total, outside_entry)
}

/// Full-array extents survive a recursive return through their type. This
/// is the original indexing witness before migration to a variable-length run;
/// it needs no recursive postcondition summary.
#[test]
fn recursive_full_arrays_merge_without_a_postcondition_summary() {
    let source = br#"fn merge_step(left: own array<u32, 3>, right: own array<u32, 3>, out: own array<u32, 3>, i: own u64, j: own u64, k: own u64) -> result: own array<u32, 3> reads(left, right), writes(out) contract {
  requires i <= 3_u64;
  requires j <= 3_u64;
  requires k <= 3_u64;
} {
  doc "ACCEPT: merges two sorted 3-slot arrays by recursively advancing whichever index holds the smaller current element; each recursive step re-establishes the same three-index contract, so no subscript needs a written certificate.";
  let i_more = i < 3_u64;
  if i_more {
  } else {
    return move out;
  }
  let j_more = j < 3_u64;
  if j_more {
  } else {
    return move out;
  }
  let k_more = k < 3_u64;
  if k_more {
  } else {
    return move out;
  }
  let lv = left[i];
  let rv = right[j];
  let take_left = lv <= rv;
  let work = move out;
  if take_left {
    set work[k] = lv;
    let next_i = i + 1_u64;
    let next_k = k + 1_u64;
    return merge_step(left: move left, right: move right, out: move work, i: next_i, j: j, k: next_k);
  } else {
    set work[k] = rv;
    let next_j = j + 1_u64;
    let next_k = k + 1_u64;
    return merge_step(left: move left, right: move right, out: move work, i: i, j: next_j, k: next_k);
  }
}

command fn main() -> status: own ExitStatus pure {
  let left = array_new::<u32, 3>(0_u32);
  set left[0_u64] = 1_u32;
  set left[1_u64] = 4_u32;
  set left[2_u64] = 6_u32;
  let right = array_new::<u32, 3>(0_u32);
  set right[0_u64] = 2_u32;
  set right[1_u64] = 3_u32;
  set right[2_u64] = 9_u32;
  let out = array_new::<u32, 3>(0_u32);
  let merged = merge_step(left: move left, right: move right, out: move out, i: 0_u64, j: 0_u64, k: 0_u64);
  let first = merged[0_u64];
  let second = merged[1_u64];
  let third = merged[2_u64];
  if first == 1_u32 {
  } else {
    return exit_status(code: 1_u8);
  }
  if second == 2_u32 {
  } else {
    return exit_status(code: 2_u8);
  }
  if third == 3_u32 {
  } else {
    return exit_status(code: 3_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn full_arrays_preserve_heap_and_arena_element_ownership_through_generic_helpers() {
    let source = br#"struct Record['s] {
  tag: u64;
  owner: Box<'s, u64>;
}

fn pass<T: linear>(value: own T) -> result: own T pure {
  return move value;
}

fn make['s](owner: own Box<'s, u64>, tag: own u64) -> result: own array<Record<'s>, 1> reads(owner) {
  let record = Record(tag: tag, owner: move owner);
  let empty = fixed_vector::<Record<'s>, 1>();
  let full = place_back(vector: move empty, value: move record);
  return array_from_fixed(vector: move full);
}

fn relay['s](values: own array<Record<'s>, 1>) -> result: own array<Record<'s>, 1> reads(values) {
  let passed = pass::<array<Record<'s>, 1>>(value: move values);
  let full = fixed_from_array(values: move passed);
  return array_from_fixed(vector: move full);
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region 'a {
    let store = arena_frame::<8, 8, 'a>();
    region {
      match heap_box(store: &uniq heap, value: 17_u64) {
        Err(error: back) => {
          return exit_status(code: 70_u8);
        }
        Ok(value: general) => {
          match arena_box(store: &uniq store, value: 29_u64) {
            Err(error: back) => {
              return exit_status(code: 71_u8);
            }
            Ok(value: extent) => {
              let heap_array = make(owner: move general, tag: 17_u64);
              let arena_array = make(owner: move extent, tag: 29_u64);
              let heap_returned = relay(values: move heap_array);
              let arena_returned = relay(values: move arena_array);
              region {
                let heap_tag = heap_returned[0_u64].tag;
                let arena_tag = arena_returned[0_u64].tag;
                if heap_tag != 17_u64 {
                  return exit_status(code: 1_u8);
                }
                if arena_tag != 29_u64 {
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
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        let observed = retain_nested_run_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        for (refused, status, ledger) in [(0, 0, b"A1;F1;".as_slice()), (1, 70, b"X1;".as_slice())]
        {
            let host = super::owned_places::allocation_observer(1, refused);
            let output = super::compile_link_and_run(&observed, Some(&host), &[]);
            assert_eq!(output.status.code(), Some(status), "{output:?}");
            assert_eq!(output.stdout, ledger, "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn full_array_zero_extents_and_zero_byte_elements_execute_without_payload_access() {
    let source = br#"struct Empty {
}

struct Recursive {
  children: array<Recursive, 0>;
}

fn relay<T: affine>(value: own T) -> result: own T pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  let none = fixed_vector::<box<u64>, 0>();
  let zero_owners = array_from_fixed(vector: move none);
  let zero_returned = relay::<array<box<u64>, 0>>(value: move zero_owners);
  let zero_run = fixed_from_array(values: move zero_returned);
  let zero_again = array_from_fixed(vector: move zero_run);
  let first = Empty();
  let second = Empty();
  let empty = fixed_vector::<Empty, 2>();
  let one = place_back(vector: move empty, value: move first);
  let wrapped = place_front(vector: move one, value: move second);
  let empty_values = array_from_fixed(vector: move wrapped);
  let empty_returned = relay::<array<Empty, 2>>(value: move empty_values);
  let previous = replace empty_returned[1_u64] = Empty();
  let empty_run = fixed_from_array(values: move empty_returned);
  let empty_again = array_from_fixed(vector: move empty_run);
  let recursion = fixed_vector::<Recursive, 0>();
  let children = array_from_fixed(vector: move recursion);
  let node = Recursive(children: move children);
  let carried = relay::<Recursive>(value: move node);
  let zero_length = len_of(zero_again);
  let empty_length = len_of(empty_again);
  if zero_length != 0_u64 {
    return exit_status(code: 1_u8);
  }
  if empty_length != 2_u64 {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = retain_nested_run_calls(&super::emit_lowered(source, overlap));
        let output = compile_and_run(&module);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn const_runs_are_immutable_globals_and_execute_through_index_and_len() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/const2-pos-array-lookup.wf"
    ));
    assert!(llvm.contains(
        "@.wf_const.0 = private unnamed_addr constant [4 x i8] [i8 10, i8 20, i8 30, i8 40]"
    ));
    let main = emitted_function(&llvm, "main");
    // The constant lookup is discharged [OP-4]: no bounds compare remains.
    // The source's two terminal result checks are ordinary control flow, not
    // claims, so all three outcomes return an ExitStatus without a trap edge.
    assert!(!main.contains("icmp ult i64"));
    assert_eq!(main.matches("icmp eq").count(), 2);
    assert_eq!(main.matches("call i8 @wf.sys.exit_status.v1").count(), 3);
    assert!(!main.contains("call void @wf_trap"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// Copy-fill remains a distinct array operation. Its emitted fill region and
/// the proved read across retained source function boundaries are both checked;
/// owning-element construction instead uses the consuming full-run conversion.
#[test]
fn filled_arrays_cross_function_boundaries_and_keep_a_checked_read() {
    let source = br#"fn make() -> result: own array<u16, 4> pure {
  return array_new::<u16, 4>(42_u16);
}

fn clamp_three(value: own u64) -> result: own u64 pure contract {
  ensures result < 4_u64;
} {
  if value < 4_u64 {
    return value;
  } else {
    return 3_u64;
  }
}

fn read(values: own array<u16, 4>, offset: own u64) -> result: own u16 reads(values) {
  let bounded = clamp_three(value: offset);
  let value = values[bounded];
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let values = make();
  let length = len_of(values);
  if length != 4_u64 {
    return exit_status(code: 1_u8);
  }
  let value = read(values: move values, offset: 3_u64);
  if value != 42_u16 {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let read = emitted_function(&llvm, "read");
    // The verified callee summary discharges the subscript directly: the
    // caller emits no runtime fallback and no second bounds branch.
    assert!(!read.contains("icmp ult i64"));
    assert!(!read.contains("call void @wf_trap"));
    assert!(read.contains("getelementptr inbounds [4 x i16]"));
    assert!(llvm.contains("array.fill.head"));
    assert!(llvm.contains("array.fill.done"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn an_out_of_bounds_run_read_is_an_op4_compile_rejection() {
    // Under discharge-or-reject [OP-4] no runtime bounds trap exists: the
    // underivable obligation rejects at compile time with the exact
    // [ENT-6] residual.
    let source = br#"const values: FixedVector<u8, 2> =[7_u8, 7_u8];

command fn main() -> status: own ExitStatus pure {
  let value = values[2_u64];
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len_of(values)"));
}

#[test]
fn compiler_independent_array_checksum_executes() {
    let output = compile_and_run(&compile(include_bytes!(
        "../../../../tests/conformance/cases/x-array-const-checksum-run.wf"
    )));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn indexed_set_checks_before_rhs_and_updates_the_run() {
    let source = br#"fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  let one = place_back(vector: move empty, value: 0_u8);
  let values = place_back(vector: move one, value: 0_u8);
  set values[1_u64] = replacement();
  let stored = values[1_u64];
  if stored != 9_u8 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // The discharged target carries no bounds branch [OP-4]; SET-1's order
    // remains: the RHS call is emitted after target formation and the store
    // after the RHS.
    let rhs = main
        .find("call i8 @wf_replacement")
        .expect("RHS must be emitted after target evaluation");
    assert!(
        !main[..rhs].contains("call void @wf_trap"),
        "no bounds trap edge precedes the RHS"
    );
    let store = main[rhs..]
        .find("store i8")
        .map(|offset| rhs + offset)
        .expect("run element update must store only after the RHS");
    assert!(rhs < store);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn an_out_of_bounds_indexed_set_is_an_op4_compile_rejection() {
    // A target whose obligation is underivable cannot reach runtime: the
    // program rejects at the subscript with the residual [OP-4, ENT-6].
    let source = br#"fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  let one = place_back(vector: move empty, value: 0_u8);
  let values = place_back(vector: move one, value: 0_u8);
  set values[2_u64] = replacement();
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len_of(values)"));
}

#[test]
fn a_long_loop_over_a_dynamically_indexed_run_keeps_the_frame_bounded() {
    // Both the read and the indexed set need a stack slot for the run value,
    // and the index is not a compile-time constant, so neither slot can be
    // promoted away. The structural half is what discriminates: no slot may be
    // declared outside the entry block, so a frame that grew once per
    // iteration fails here whatever it would survive. The run is a
    // corroboration rather than the measurement — 200000 iterations of a
    // 64-byte slot is about 12 MB, which used to be past a default 8 MB limit
    // and now fits inside the 1 GiB stack the runtime gives every thread. A
    // run's length is not a fact of its type [BLK-1], so the two loops carry
    // the `len_of` invariant the array place had standing.
    let source = br#"command fn main() -> status: own ExitStatus pure {
  doc "Nested counted loops read and write one fixed run for two hundred thousand iterations.";
  let built = fixed_vector::<u64, 8>();
  for @fill (
    at in 0_u64..8_u64,
    invariant grown: len_of(built) >= at,
    invariant spare: room_of(built) + at >= 8_u64,
    invariant flat: head_of(built) <= 0_u64
  ) {
    set built = place_back(vector: move built, value: 1_u64);
  }
  let window = move built;
  let completed = 0_u64;
  let total = 0_u64;
  for @outer (
    batch in 0_u64..25000_u64,
    invariant held: len_of(window) >= 8_u64
  ) {
    for @inner (
      cursor in 0_u64..8_u64,
      invariant still: len_of(window) >= 8_u64
    ) {
      let previous = window[cursor];
      let mixed = ixor(previous, completed);
      set window[cursor] = mixed *wrap 1099511628211_u64;
      set total = total +wrap previous;
      set completed = completed +wrap 1_u64;
    }
  }
  if completed != 200000_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let (total, outside_entry) = slot_declarations(main);
    assert!(
        total > 0,
        "the kernel must still need stack slots for this test to mean anything:\n{main}"
    );
    assert_eq!(
        outside_entry, 0,
        "{outside_entry} of {total} stack slots are declared outside the entry block, so the frame grows once per iteration:\n{main}"
    );

    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "the loop must run to completion instead of exhausting the stack: {:?}",
        output.status
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_mutable_array_checksum_executes() {
    let output = compile_and_run(&compile(include_bytes!(
        "../../../../tests/conformance/cases/x-array-mutable-checksum-run.wf"
    )));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn nested_struct_run_update_uses_the_element_address_prepared_before_the_rhs() {
    let source = br#"struct Inner {
  values: FixedVector<u8, 2>;
  sibling: u16;
}

struct Outer {
  prefix: u32;
  inner: Inner;
}

fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  let one = place_back(vector: move empty, value: 0_u8);
  let values = place_back(vector: move one, value: 0_u8);
  let inner = Inner(values: move values, sibling: 77_u16);
  let outer = Outer(prefix: 123_u32, inner: move inner);
  set outer.inner.values[1_u64] = replacement();
  let stored = outer.inner.values[1_u64];
  if stored != 9_u8 {
    return exit_status(code: 1_u8);
  }
  if outer.inner.sibling != 77_u16 {
    return exit_status(code: 2_u8);
  }
  if outer.prefix != 123_u32 {
    return exit_status(code: 3_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // The discharged projected target carries no bounds branch [OP-4].
    let rhs = main
        .find("call i8 @wf_replacement")
        .expect("RHS must follow projected target evaluation");
    assert!(
        !main[..rhs].contains("call void @wf_trap"),
        "no bounds trap edge precedes the RHS"
    );
    assert_eq!(main.matches("call i8 @wf_replacement").count(), 1);
    let replacement = main[..rhs]
        .lines()
        .next_back()
        .expect("RHS result definition")
        .trim()
        .strip_suffix(" =")
        .expect("RHS assigns its result");
    let store_text = format!("store i8 {replacement}, ptr ");
    let store = main
        .find(&store_text)
        .expect("one element receives the RHS value");
    assert_eq!(main.matches(&store_text).count(), 1);
    let address = main[store..]
        .lines()
        .next()
        .expect("element store")
        .strip_prefix(&store_text)
        .expect("element address operand");
    let prepared = main
        .find(&format!("{address} = getelementptr "))
        .expect("the complete element address is prepared once");
    assert!(prepared < rhs && rhs < store);
    assert!(
        !main[rhs..store].contains("store %wf.t"),
        "the element write must not rebuild its enclosing structs"
    );

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn general_run_elements_preserve_array_places_and_standing_extents() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let row = array_new::<u64, 2>(7_u64);
  let empty = fixed_vector::<array<u64, 2>, 2>();
  let rows = place_back(vector: move empty, value: move row);
  let width = len_of(rows[0_u64]);
  let capacity = cap_of(rows[0_u64]);
  let head = head_of(rows[0_u64]);
  let room = room_of(rows[0_u64]);
  set rows[0_u64][1_u64] = 9_u64;
  if rows[0_u64][0_u64] != 7_u64 {
    return exit_status(code: 1_u8);
  }
  if rows[0_u64][1_u64] != 9_u64 {
    return exit_status(code: 2_u8);
  }
  if width != 2_u64 {
    return exit_status(code: 3_u8);
  }
  if capacity != 2_u64 {
    return exit_status(code: 4_u8);
  }
  if head != 0_u64 {
    return exit_status(code: 5_u8);
  }
  if room != 0_u64 {
    return exit_status(code: 6_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        crate::OverlapLowering::Off,
        crate::OverlapLowering::On,
        crate::OverlapLowering::Completion,
    ] {
        let llvm = super::emit_lowered(source, overlap);
        let output = compile_and_run(&llvm);
        assert!(output.status.success(), "{overlap:?}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

/// An owning element crosses more than the former one-level run lift. The
/// allocation ledger observes the complete nested window and generic handoff,
/// rather than only whether a deeply nested type can be named.
#[test]
fn general_run_elements_preserve_nested_owners_across_generic_calls() {
    let source = br#"fn pass<T: linear>(value: own T) -> result: own T pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  let first = box_new(17_u64);
  let second = box_new(29_u64);
  let leaf = fixed_vector::<box<u64>, 2>();
  set leaf = place_back(vector: move leaf, value: move first);
  set leaf = place_back(vector: move leaf, value: move second);
  let middle = fixed_vector::<FixedVector<box<u64>, 2>, 1>();
  set middle = place_back(vector: move middle, value: move leaf);
  let outer = fixed_vector::<FixedVector<FixedVector<box<u64>, 2>, 1>, 1>();
  set outer = place_back(vector: move outer, value: move middle);
  let carried = pass::<FixedVector<FixedVector<FixedVector<box<u64>, 2>, 1>, 1>>(value: move outer);
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        super::OverlapLowering::Off,
        super::OverlapLowering::On,
        super::OverlapLowering::Completion,
    ] {
        let module = super::emit_lowered(source, overlap);
        let observed = retain_nested_run_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        let host = super::owned_places::allocation_observer(2, 0);
        let output = super::compile_link_and_run(&observed, Some(&host), &[]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stdout, b"A1;A2;F1;F2;", "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

/// The same unconstrained region and captured generic call must preserve the
/// different release classes inside three inline run layers.
#[test]
fn general_run_elements_preserve_box_brands_across_region_polymorphic_calls() {
    let source = br#"fn pass<T: linear>(value: own T) -> result: own T pure {
  return move value;
}

fn nest['s](value: own Box<'s, u64>) -> result: own FixedVector<FixedVector<FixedVector<Box<'s, u64>, 1>, 1>, 1> pure {
  let leaf = fixed_vector::<Box<'s, u64>, 1>();
  set leaf = place_back(vector: move leaf, value: move value);
  let middle = fixed_vector::<FixedVector<Box<'s, u64>, 1>, 1>();
  set middle = place_back(vector: move middle, value: move leaf);
  let outer = fixed_vector::<FixedVector<FixedVector<Box<'s, u64>, 1>, 1>, 1>();
  set outer = place_back(vector: move outer, value: move middle);
  return move outer;
}

fn carry['s](value: own FixedVector<FixedVector<FixedVector<Box<'s, u64>, 1>, 1>, 1>) -> result: own FixedVector<FixedVector<FixedVector<Box<'s, u64>, 1>, 1>, 1> pure {
  return pass::<FixedVector<FixedVector<FixedVector<Box<'s, u64>, 1>, 1>, 1>>(value: move value);
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region 'a {
    let store = arena_frame::<8, 8, 'a>();
    region {
      match heap_box(store: &uniq heap, value: 17_u64) {
        Err(error: back) => {
          return exit_status(code: 70_u8);
        }
        Ok(value: general) => {
          match arena_box(store: &uniq store, value: 29_u64) {
            Err(error: back) => {
              return exit_status(code: 70_u8);
            }
            Ok(value: extent) => {
              let general_nested = nest(value: move general);
              let extent_nested = nest(value: move extent);
              let general_carried = carry(value: move general_nested);
              let extent_carried = carry(value: move extent_nested);
              return exit_status(code: 0_u8);
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
        let observed = retain_nested_run_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        let host = super::owned_places::allocation_observer(1, 0);
        let output = super::compile_link_and_run(&observed, Some(&host), &[]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        // The arena cell remains owned by its extent. Only the heap cell may
        // reach the host allocator, exactly once after the returned run dies.
        assert_eq!(output.stdout, b"A1;F1;", "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

/// Vector's descriptor has finite layout even when its elements own another
/// Vector of the same node type. The release graph follows actual initialized
/// windows, and refusal must clean up the already constructed child.
#[test]
fn general_run_elements_close_recursive_descriptor_layout_and_cleanup() {
    let source = br#"struct Tree['s] {
  children: Vector<'s, Tree<'s>>;
}

fn build['s](store: &uniq Heap<'s>) -> result: own Option<Tree<'s>> reads(store), writes(store), allocates(store) {
  region {
    match heap_vector::<Tree<'s>>(store: &uniq deref(store), count: 1_u64) {
      None() => {
        return None<Tree<'s>>();
      }
      Some(value: empty_children) => {
        let child = Tree(children: move empty_children);
        region {
          match heap_vector::<Tree<'s>>(store: &uniq deref(store), count: 1_u64) {
            None() => {
              return None<Tree<'s>>();
            }
            Some(value: parent_children) => {
              let populated = place_back(vector: move parent_children, value: move child);
              let root = Tree(children: move populated);
              return Some<Tree<'s>>(value: move root);
            }
          }
        }
      }
    }
  }
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match build(store: &uniq heap) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: tree) => {
        return exit_status(code: 0_u8);
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
        let observed = retain_nested_run_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        for (refused, status, expected) in
            [(0, 0, "A1;A2;F1;F2;"), (1, 70, "X1;"), (2, 70, "A1;X2;F1;")]
        {
            let host = super::owned_places::allocation_observer(2, refused);
            let output = super::compile_link_and_run(&observed, Some(&host), &[]);
            assert_eq!(output.status.code(), Some(status), "{output:?}");
            assert_eq!(output.stdout, expected.as_bytes(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

/// A complete owning array can reside in a store-owned Box while its element
/// owners survive helper calls, borrowed reads, replacement, and refusal.
#[test]
fn heap_full_arrays_preserve_elements_across_calls_replacement_and_refusal() {
    let source = br#"struct Record {
  payload: array<u64, 16>;
  owner: box<u64>;
}

fn make_record(tag: own u64) -> result: own Record pure {
  let payload = array_new::<u64, 16>(tag);
  let owner = box_new(tag);
  return Record(payload: move payload, owner: move owner);
}

fn relay<T: affine>(value: own T) -> result: own T pure {
  return move value;
}

fn place['s](store: &uniq Heap<'s>, values: own array<Record, 2>) -> result: own Result<Box<'s, array<Record, 2>>, array<Record, 2>> reads(store), writes(store), allocates(store) {
  region {
    return heap_box(store: &uniq deref(store), value: move values);
  }
}

fn read(storage: &Box<array<Record, 2>>, index: own u64) -> result: own u64 reads(storage) contract {
  requires index < 2_u64;
} {
  return deref(deref(storage))[index].payload[7_u64];
}

fn pass_box['s](value: own Box<'s, array<Record, 2>>) -> result: own Box<'s, array<Record, 2>> pure {
  return relay::<Box<'s, array<Record, 2>>>(value: move value);
}

fn update['s](storage: own Box<'s, array<Record, 2>>) -> (result: own Box<'s, array<Record, 2>>, old: own Record) reads(storage), writes(storage) {
  let previous = replace deref(storage)[0_u64] = make_record(tag: 99_u64);
  return move storage, move previous;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  let first = make_record(tag: 11_u64);
  let second_tag = first.payload[0_u64] +wrap 11_u64;
  let second = make_record(tag: second_tag);
  let empty = fixed_vector::<Record, 2>();
  let one = place_back(vector: move empty, value: move first);
  let two = place_back(vector: move one, value: move second);
  let values = array_from_fixed(vector: move two);
  region {
    match place(store: &uniq heap, values: move values) {
      Err(error: returned) => {
        if returned[0_u64].payload[7_u64] != 11_u64 {
          return exit_status(code: 71_u8);
        }
        if returned[1_u64].payload[7_u64] != 22_u64 {
          return exit_status(code: 72_u8);
        }
        return exit_status(code: 70_u8);
      }
      Ok(value: storage) => {
        let passed = pass_box(value: move storage);
        region {
          let before = read(storage: &passed, index: 0_u64);
          if before != 11_u64 {
            return exit_status(code: 1_u8);
          }
        }
        let (updated, previous) = update(storage: move passed);
        if previous.payload[7_u64] != 11_u64 {
          return exit_status(code: 4_u8);
        }
        region {
          let first_tag = read(storage: &updated, index: 0_u64);
          let remaining_tag = read(storage: &updated, index: 1_u64);
          if first_tag != 99_u64 {
            return exit_status(code: 2_u8);
          }
          if remaining_tag != 22_u64 {
            return exit_status(code: 3_u8);
          }
        }
        return exit_status(code: 0_u8);
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
        let observed = retain_nested_run_calls(&module)
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        // The second record depends on the first, and the outer allocation
        // consumes both, so allocation IDs identify these source owners.
        // Refusing that outer allocation must return the original full array.
        for (refused, status, ledger) in [
            (0, 0, b"A1;A2;A3;A4;F1;F4;F2;F3;".as_slice()),
            (3, 70, b"A1;A2;X3;F1;F2;".as_slice()),
        ] {
            let host = super::owned_places::allocation_observer(4, refused);
            let output = super::compile_link_and_run(&observed, Some(&host), &[]);
            assert_eq!(
                output.status.code(),
                Some(status),
                "{overlap:?}: {output:?}"
            );
            assert_eq!(output.stdout, ledger, "{overlap:?}: {output:?}");
            assert!(output.stderr.is_empty(), "{overlap:?}: {output:?}");
        }
    }
}

fn retain_nested_run_calls(module: &str) -> String {
    module
        .lines()
        .map(|line| {
            if line.starts_with("define internal ")
                && let Some(header) = line.strip_suffix(" {")
            {
                format!("{header} noinline {{\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect()
}
