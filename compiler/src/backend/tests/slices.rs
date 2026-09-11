use super::*;

#[test]
fn borrowed_array_views_preserve_delegation_and_original_storage_across_calls() {
    let source = br#"struct Packet {
  before: u64;
  bytes: array<u64, 3>;
  after: u64;
}

const fixed: array<u64, 3> =[2_u64, 3_u64, 5_u64];

fn relay['r](values: own Slice<'r, u64>) -> result: own Slice<'r, u64> pure contract {
  requires len_of(values) == 3_u64;
  ensures len_of(result) == 3_u64;
} {
  return values;
}

fn read(values: own Slice<u64>, index: own u64) -> result: own u64 reads(values) contract {
  requires index < len_of(values);
} {
  return values[index];
}

fn write(values: &uniq MutSlice<u64>, index: own u64, value: own u64) -> result: own unit writes(values) contract {
  requires index < len_of(deref(values));
} {
  set deref(values)[index] = value;
  return unit;
}

fn delegate(view: own MutSlice<u64>) -> result: own unit writes(view) contract {
  requires len_of(view) == 3_u64;
} {
  let writer = move view;
  region {
    let done = write(values: &uniq writer, index: 1_u64, value: 109_u64);
  }
  return unit;
}

fn shared_read(values: &array<u64, 3>) -> result: own u64 reads(values) {
  region {
    let view = slice_of(&deref(values));
    let copied = view;
    let forwarded = relay(values: copied);
    let first = read(values: view, index: 0_u64);
    let second = read(values: copied, index: 1_u64);
    let third = read(values: forwarded, index: 2_u64);
    let prefix = first +wrap second;
    return prefix +wrap third;
  }
}

fn revise(values: &uniq array<u64, 3>) -> result: own u64 reads(values), writes(values) {
  region {
    let view = slice_of(&deref(values));
    let copied = view;
    let forwarded = relay(values: copied);
    let first = read(values: view, index: 0_u64);
    let second = read(values: copied, index: 1_u64);
    let third = read(values: forwarded, index: 2_u64);
    let prefix = first +wrap second;
    let previous = prefix +wrap third;
    set deref(values)[0_u64] = 101_u64;
    let writable = mut_slice_of(&uniq deref(values));
    region {
      let done = write(values: &uniq writable, index: 1_u64, value: 103_u64);
    }
    return previous;
  }
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u64, 3>(7_u64);
  let before0 = values[0_u64];
  let before1 = values[1_u64];
  region {
    let initial = shared_read(values: &values);
    if initial != 21_u64 {
      return exit_status(code: 1_u8);
    }
  }
  region {
    let previous = revise(values: &uniq values);
    if previous != 21_u64 {
      return exit_status(code: 2_u8);
    }
  }
  if values[0_u64] != 101_u64 {
    return exit_status(code: 3_u8);
  }
  if values[1_u64] != 103_u64 {
    return exit_status(code: 4_u8);
  }
  if values[2_u64] != 7_u64 {
    return exit_status(code: 5_u8);
  }
  if before0 != 7_u64 {
    return exit_status(code: 6_u8);
  }
  if before1 != 7_u64 {
    return exit_status(code: 7_u8);
  }
  region {
    let current = shared_read(values: &values);
    if current != 211_u64 {
      return exit_status(code: 8_u8);
    }
  }
  let bytes = array_new::<u64, 3>(13_u64);
  let packet = Packet(before: 53_u64, bytes: move bytes, after: 59_u64);
  region {
    let holder = &uniq packet.bytes;
    region {
      let writable = mut_slice_of(&uniq deref(holder));
      set writable[2_u64] = 107_u64;
    }
  }
  if packet.before != 53_u64 {
    return exit_status(code: 9_u8);
  }
  if packet.after != 59_u64 {
    return exit_status(code: 10_u8);
  }
  if packet.bytes[0_u64] != 13_u64 {
    return exit_status(code: 11_u8);
  }
  if packet.bytes[1_u64] != 13_u64 {
    return exit_status(code: 12_u8);
  }
  if packet.bytes[2_u64] != 107_u64 {
    return exit_status(code: 13_u8);
  }
  region {
    let holder = &packet.bytes;
    region {
      let view = slice_of(&deref(holder));
      let selected = read(values: view, index: 2_u64);
      if selected != 107_u64 {
        return exit_status(code: 14_u8);
      }
    }
  }
  region {
    let observed = shared_read(values: &fixed);
    if observed != 10_u64 {
      return exit_status(code: 15_u8);
    }
  }
  region {
    let holder = &fixed;
    region {
      let view = slice_of(&deref(holder));
      let selected = read(values: view, index: 1_u64);
      if selected != 3_u64 {
        return exit_status(code: 16_u8);
      }
    }
  }
  let delegated = array_new::<u64, 3>(17_u64);
  region {
    let view = mut_slice_of(&uniq delegated);
    let done = delegate(view: move view);
  }
  if delegated[0_u64] != 17_u64 {
    return exit_status(code: 17_u8);
  }
  if delegated[1_u64] != 109_u64 {
    return exit_status(code: 18_u8);
  }
  if delegated[2_u64] != 17_u64 {
    return exit_status(code: 19_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        OverlapLowering::Off,
        OverlapLowering::On,
        OverlapLowering::Completion,
    ] {
        let module = emit_lowered(source, overlap);
        for module in [&module, &super::owned_places::retain_calls(&module)] {
            let output = compile_and_run(module);
            assert_eq!(output.status.code(), Some(0), "{overlap:?}: {output:?}");
            assert!(output.stdout.is_empty(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn exclusive_array_views_write_original_local_and_field_storage() {
    let source = br#"struct Packet {
  before: u64;
  bytes: array<u64, 4>;
  after: u64;
}

fn overwrite(view: &uniq MutSlice<u64>, index: own u64, value: own u64) -> result: own unit writes(view) contract {
  requires index < len_of(deref(view));
} {
  set deref(view)[index] = value;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u64, 4>(7_u64);
  let before0 = values[0_u64];
  let before2 = values[2_u64];
  region {
    let view = mut_slice_of(&uniq values);
    set view[0_u64] = 19_u64;
    region {
      let done = overwrite(view: &uniq view, index: 2_u64, value: 31_u64);
    }
  }
  if values[0_u64] != 19_u64 {
    return exit_status(code: 1_u8);
  }
  if values[1_u64] != 7_u64 {
    return exit_status(code: 2_u8);
  }
  if values[2_u64] != 31_u64 {
    return exit_status(code: 3_u8);
  }
  if values[3_u64] != 7_u64 {
    return exit_status(code: 4_u8);
  }
  if before0 != 7_u64 {
    return exit_status(code: 5_u8);
  }
  if before2 != 7_u64 {
    return exit_status(code: 6_u8);
  }
  let bytes = array_new::<u64, 4>(13_u64);
  let packet = Packet(before: 53_u64, bytes: move bytes, after: 59_u64);
  region {
    let view = mut_slice_of(&uniq packet.bytes);
    region {
      let done = overwrite(view: &uniq view, index: 1_u64, value: 41_u64);
    }
  }
  if packet.before != 53_u64 {
    return exit_status(code: 7_u8);
  }
  if packet.after != 59_u64 {
    return exit_status(code: 8_u8);
  }
  if packet.bytes[0_u64] != 13_u64 {
    return exit_status(code: 9_u8);
  }
  if packet.bytes[1_u64] != 41_u64 {
    return exit_status(code: 10_u8);
  }
  if packet.bytes[2_u64] != 13_u64 {
    return exit_status(code: 11_u8);
  }
  if packet.bytes[3_u64] != 13_u64 {
    return exit_status(code: 12_u8);
  }
  region {
    let shared = slice_of(&packet.bytes);
    if shared[1_u64] != 41_u64 {
      return exit_status(code: 13_u8);
    }
  }
  let empty = array_new::<u64, 0>(0_u64);
  region {
    let view = mut_slice_of(&uniq empty);
    let length = len_of(view);
    if length != 0_u64 {
      return exit_status(code: 14_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        OverlapLowering::Off,
        OverlapLowering::On,
        OverlapLowering::Completion,
    ] {
        let module = emit_lowered(source, overlap);
        for module in [&module, &super::owned_places::retain_calls(&module)] {
            let output = compile_and_run(module);
            assert_eq!(output.status.code(), Some(0), "{overlap:?}: {output:?}");
            assert!(output.stdout.is_empty(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn formal_view_child_last_use_restores_parent_writes_across_retained_calls() {
    let source = r#"fn child['r](view: &uniq MutSlice<'r, u8>) -> result: own Slice<'r, u8> pure contract {
  requires len_of(deref(view)) == 1_u64;
  ensures len_of(result) == 1_u64;
} {
  let result = slice_of(&'r deref(view));
  return result;
}

fn reuse(view: &uniq MutSlice<u8>) -> result: own u8 reads(view), writes(view) contract {
  requires len_of(deref(view)) == 1_u64;
} {
  region {
    let shared = FORM;
    let copied = shared;
    let previous = shared[0_u64];
    let confirmed = copied[0_u64];
    set deref(view)[0_u64] = 9_u8;
    return previous;
  }
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 1>();
  region {
    place_back(vector: &uniq empty, value: 3_u8);
  }
  let bytes = move empty;
  region {
    let writer = mut_slice_of(&uniq bytes);
    region {
      let previous = reuse(view: &uniq writer);
      if previous != 3_u8 {
        return exit_status(code: 1_u8);
      }
      if writer[0_u64] != 9_u8 {
        return exit_status(code: 2_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for formation in ["slice_of(&deref(view))", "child(view: &uniq deref(view))"] {
        let source = source.replace("FORM", formation);
        for overlap in [
            OverlapLowering::Off,
            OverlapLowering::On,
            OverlapLowering::Completion,
        ] {
            let module = emit_lowered(source.as_bytes(), overlap);
            for module in [&module, &super::owned_places::retain_calls(&module)] {
                let output = compile_and_run(module);
                assert_eq!(
                    output.status.code(),
                    Some(0),
                    "{formation}: {overlap:?}: {output:?}"
                );
                assert!(output.stdout.is_empty(), "{output:?}");
                assert!(output.stderr.is_empty(), "{output:?}");
            }
        }
    }
}

#[test]
fn const_local_and_store_run_slices_share_one_read_only_descriptor_path() {
    // The three view origins are now the three run storages [STOR-1]: the
    // const run's read-only static rodata, a frame-resident `FixedVector`,
    // and one run taken from the general store. One `Slice` consumer reads
    // all three, which is the property the retired array/buffer pair pinned.
    let source = br#"const bytes: FixedVector<u8, 4> =[1_u8, 2_u8, 3_u8, 4_u8];

fn sum(values: own Slice<u8>) -> result: own u64 reads(values) {
  let total = 0_u64;
  let length = len_of(values);
  for (offset in 0_u64..length) {
    let byte = values[offset];
    let word = cvt::<u8, u64>(byte);
    set total = total +wrap word;
  }
  return total;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  let code = 0_u8;
  region {
    let view = slice_of(&bytes);
    let total = sum(values: view);
    if total != 10_u64 {
      set code = 1_u8;
    }
  }
  let empty = fixed_vector::<u8, 4>();
  region {
    place_back(vector: &uniq empty, value: 3_u8);
  }
  let one = move empty;
  region {
    place_back(vector: &uniq one, value: 3_u8);
  }
  let two = move one;
  region {
    place_back(vector: &uniq two, value: 3_u8);
  }
  let three = move two;
  region {
    place_back(vector: &uniq three, value: 3_u8);
  }
  let local = move three;
  region {
    let view = slice_of(&local);
    let total = sum(values: view);
    if total != 12_u64 {
      set code = 2_u8;
    }
  }
  region {
    match heap_vector::<u8>(store: &uniq heap, count: 4_u64) {
      None() => {
        return exit_status(code: 4_u8);
      }
      Some(value: fresh) => {
        let runtime = move fresh;
        for @fill (
          at in 0_u64..4_u64,
          invariant grown: len_of(runtime) >= at,
          invariant spare: room_of(runtime) + at >= 4_u64,
          invariant flat: head_of(runtime) <= 0_u64
        ) {
          place_back(vector: &uniq runtime, value: 2_u8);
        }
        region {
          let view = slice_of(&runtime);
          let total = sum(values: view);
          if total != 8_u64 {
            set code = 3_u8;
          }
        }
      }
    }
  }
  return exit_status(code: code);
}
"#;
    let llvm = compile(source);
    let sum = emitted_function(&llvm, "sum");
    let main = emitted_function(&llvm, "main");
    // The counted range discharges the slice read before lowering, so the
    // element address forms directly without a runtime bounds branch.
    assert!(sum.contains("getelementptr inbounds i8"));
    assert!(!sum.contains("call void @free"));
    // One release, for the reason the buffer's free had: exactly one of the
    // three origins owns store storage, and `main` holds the provider that
    // its release spends [PROV-1, BLK-1].
    assert_eq!(main.matches("call void @free").count(), 1);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn an_out_of_bounds_slice_read_is_an_op4_compile_rejection() {
    // The slice carries its source run's window length, so the constant
    // offset is refutable at compile time and the program rejects with the
    // residual [OP-4, ENT-6] — the same residual the array origin gave.
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq empty, value: 0_u8);
  }
  let one = move empty;
  region {
    place_back(vector: &uniq one, value: 0_u8);
  }
  let bytes = move one;
  region {
    let window = slice_of(&bytes);
    let value = window[2_u64];
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len_of(window)"));
}

#[test]
fn returned_slice_descriptors_execute_without_transferring_storage() {
    let source = br#"const fixed: FixedVector<u8, 2> =[7_u8, 13_u8];

fn pass['r](value: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  return value;
}

fn choose['r](take_left: own Bool, left: own Slice<'r, u8>, right: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  if take_left {
    return left;
  } else {
    return right;
  }
}

fn fixed_view['r]() -> result: own Slice<'r, u8> pure {
  return slice_of(&'r fixed);
}

fn borrowed_first(value: &Slice<u8>) -> result: own u8 reads(value) contract {
  define spare = len_of(deref(value));
  requires 0_u64 < spare;
} {
  return deref(value)[0_u64];
}

command fn main() -> status: own ExitStatus pure {
  let left_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq left_empty, value: 11_u8);
  }
  let left_one = move left_empty;
  region {
    place_back(vector: &uniq left_one, value: 11_u8);
  }
  let left = move left_one;
  let right_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq right_empty, value: 29_u8);
  }
  let right_one = move right_empty;
  region {
    place_back(vector: &uniq right_one, value: 29_u8);
  }
  let right = move right_one;
  region 'view {
    let borrowed_source = slice_of(&left);
    region {
      let borrowed_value = borrowed_first(value: &borrowed_source);
      if borrowed_value != 11_u8 {
        return exit_status(code: 1_u8);
      }
    }
    let initial = slice_of(&left);
    let passed = pass(value: initial);
    let passed_room = len_of(passed);
    if 0_u64 < passed_room {
      let pass_value = passed[0_u64];
      if pass_value != 11_u8 {
        return exit_status(code: 2_u8);
      }
    } else {
      return exit_status(code: 2_u8);
    }
    let left_view = slice_of(&left);
    let right_view = slice_of(&right);
    let take_left = False();
    let selected = choose(take_left: take_left, left: left_view, right: right_view);
    let selected_room = len_of(selected);
    if 0_u64 < selected_room {
      let selected_value = selected[0_u64];
      if selected_value != 29_u8 {
        return exit_status(code: 3_u8);
      }
    } else {
      return exit_status(code: 3_u8);
    }
    let constant = fixed_view::<'view>();
    let constant_room = len_of(constant);
    if 1_u64 < constant_room {
      let constant_value = constant[1_u64];
      if constant_value != 13_u8 {
        return exit_status(code: 4_u8);
      }
    } else {
      return exit_status(code: 4_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    assert!(!emitted_function(&llvm, "pass").contains("call void @free"));
    assert!(!emitted_function(&llvm, "choose").contains("call void @free"));
    assert!(!emitted_function(&llvm, "fixed_view").contains("call void @free"));
    assert!(!emitted_function(&llvm, "borrowed_first").contains("call void @free"));
    assert!(!emitted_function(&llvm, "main").contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// A view of a frame-resident run reaches that run's own slots, and it does so
/// across a may-suspend call [VIEW-2, SYS-8].
///
/// The frame-slot planner and the emission that consumes the slot each decide
/// whether a run keeps its slots inline; a view that the planner did not see
/// would take the address of a slot no entry reserved. The pin is the whole
/// path: the emitted view is a `getelementptr` into the run's own frame slot
/// rather than a copy of a descriptor's pointer word, the run is the source
/// operand of a `write_once` that is `may-suspend` and therefore reaches the
/// completion runtime, and the process publishes exactly the bytes the fill
/// loop wrote.
#[test]
fn a_view_of_a_frame_resident_run_reaches_its_own_slots_across_a_may_suspend_call() {
    let source = br#"command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out) {
  doc "Publishes a frame-resident run through a shared view held across the may-suspend write.";
  let page = fixed_vector::<u8, 4>();
  for @fill (
    at in 0_u64..4_u64,
    invariant grown: len_of(page) >= at,
    invariant spare: room_of(page) + at >= 4_u64,
    invariant flat: head_of(page) <= 0_u64
  ) {
    place_back(vector: &uniq page, value: 65_u8);
  }
  region 'o {
    let window = slice_of(&page);
    region {
      match write_once(output: &uniq 'o out, source: &window, start: 0_u64, end: 4_u64) {
        Ok(value: written) => {
          if written != 4_u64 {
            return exit_status(code: 1_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 2_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // The run's slots are inline, so forming the view stores the aggregate
    // into the frame slot the planner reserved and indexes there. A run whose
    // slots live behind a descriptor pointer would `extractvalue` instead, so
    // this is the shape assertion and not a spelling one.
    assert!(
        main.contains("getelementptr inbounds { [4 x i8], i64, i64 }, ptr %"),
        "the view must index the run's own frame slot:\n{main}"
    );
    // Nothing is allocated or freed: a frame-resident run owns no store
    // storage and a view owns none at all [STOR-1].
    assert!(!main.contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"AAAA");
    assert!(output.stderr.is_empty());
}
