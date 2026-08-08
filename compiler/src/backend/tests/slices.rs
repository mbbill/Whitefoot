use super::*;

#[test]
fn array_and_buffer_slices_share_one_read_only_descriptor_path() {
    let source = br#"const bytes: array<u8, 4> =[1_u8, 2_u8, 3_u8, 4_u8];

fn sum['r](values: own slice<'r, u8>) -> own u64 reads('r), traps {
  let offset = 0_u64;
  let total = 0_u64;
  let length = len(values);
  loop @items {
    let done = offset == length;
    if done {
      break @items;
    }
    let read_ok = ilt(offset, length);
    claim offset_in_values: read_ok because "the walk stops at the slice length";
    let byte = values[offset];
    let word = cvt<u8, u64>(byte);
    set total = total +wrap word;
    set offset = offset +wrap 1_u64;
  }
  return total;
}

fn main() -> own unit allocates(heap), traps {
  region 'static_view {
    let view = slice_of(&'static_view bytes);
    let total = sum<'static_view>(values: move view);
    check total == 10_u64 else trap "array slice";
  }
  let local = array_new<u8, 4>(3_u8);
  region 'local_view {
    let view = slice_of(&'local_view local);
    let total = sum<'local_view>(values: move view);
    check total == 12_u64 else trap "local array slice";
  }
  let runtime = buffer_new(4_u64, 2_u8);
  region 'runtime_view {
    let view = slice_of(&'runtime_view runtime);
    let total = sum<'runtime_view>(values: move view);
    check total == 8_u64 else trap "buffer slice";
  }
  return unit;
}
"#;
    let llvm = compile(source);
    let sum = emitted_function(&llvm, "sum");
    let main = emitted_function(&llvm, "main");
    // The discharged slice read emits no bounds branch; the claim is the
    // one retained check and the element address forms directly.
    assert!(sum.contains("getelementptr inbounds i8"));
    assert!(!sum.contains("call void @free"));
    assert_eq!(main.matches("call void @free").count(), 1);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn an_out_of_bounds_slice_read_is_an_op4_compile_rejection() {
    // The slice carries its source array's length, so the constant offset
    // is refutable at compile time and the program rejects with the
    // residual [OP-4, ENT-6].
    let source = br#"fn main() -> own unit pure {
  let bytes = array_new<u8, 2>(0_u8);
  region 'view {
    let window = slice_of(&'view bytes);
    let value = window[2_u64];
  }
  return unit;
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len(window)"));
}

#[test]
fn returned_slice_descriptors_execute_without_transferring_storage() {
    let source = br#"const fixed: array<u8, 2> =[7_u8, 13_u8];

fn pass['r](value: own slice<'r, u8>) -> own slice<'r, u8> pure {
  return move value;
}

fn choose['r](take_left: own Bool, left: own slice<'r, u8>, right: own slice<'r, u8>) -> own slice<'r, u8> pure {
  if take_left {
    return move left;
  } else {
    return move right;
  }
}

fn fixed_view['r]() -> own slice<'r, u8> pure {
  return slice_of(&'r fixed);
}

fn borrowed_first['descriptor, 'data](value: &'descriptor slice<'data, u8>) -> own u8 reads('descriptor 'data), traps {
  let room = len(deref(value));
  let ok = ilt(0_u64, room);
  claim nonempty: ok because "callers pass a two-byte view";
  return deref(value)[0_u64];
}

fn main() -> own unit traps {
  let left = array_new<u8, 2>(11_u8);
  let right = array_new<u8, 2>(29_u8);
  region 'view {
    let borrowed_source = slice_of(&'view left);
    region 'descriptor {
      let borrowed_value = borrowed_first<'descriptor, 'view>(value: &'descriptor borrowed_source);
      check borrowed_value == 11_u8 else trap "borrowed";
    }
    let initial = slice_of(&'view left);
    let passed = pass<'view>(value: move initial);
    let passed_room = len(passed);
    let passed_ok = ilt(0_u64, passed_room);
    claim passed_nonempty: passed_ok because "pass returns the two-byte view";
    let pass_value = passed[0_u64];
    check pass_value == 11_u8 else trap "pass";
    let left_view = slice_of(&'view left);
    let right_view = slice_of(&'view right);
    let take_left = False();
    let selected = choose<'view>(take_left: take_left, left: move left_view, right: move right_view);
    let selected_room = len(selected);
    let selected_ok = ilt(0_u64, selected_room);
    claim selected_nonempty: selected_ok because "choose returns one two-byte view";
    let selected_value = selected[0_u64];
    check selected_value == 29_u8 else trap "choice";
    let constant = fixed_view<'view>();
    let constant_room = len(constant);
    let constant_ok = ilt(1_u64, constant_room);
    claim constant_sized: constant_ok because "fixed_view returns the two-byte constant view";
    let constant_value = constant[1_u64];
    check constant_value == 13_u8 else trap "const";
  }
  return unit;
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
