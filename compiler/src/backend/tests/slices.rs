use super::*;

#[test]
fn array_and_buffer_slices_share_one_read_only_descriptor_path() {
    let source = br#"const bytes: array<u8, 4> =[1_u8, 2_u8, 3_u8, 4_u8];

fn sum(values: own slice<u8>) -> result: own u64 reads(values) {
  let total = 0_u64;
  let length = len(values);
  for (offset in 0_u64..length) {
    let byte = values[offset];
    let word = cvt::<u8, u64>(byte);
    set total = total +wrap word;
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let code = 0_u8;
  region {
    let view = slice_of(&bytes);
    let total = sum(values: move view);
    if total != 10_u64 {
      set code = 1_u8;
    }
  }
  let local = array_new::<u8, 4>(3_u8);
  region {
    let view = slice_of(&local);
    let total = sum(values: move view);
    if total != 12_u64 {
      set code = 2_u8;
    }
  }
  let runtime = buffer_new(4_u64, 2_u8);
  region {
    let view = slice_of(&runtime);
    let total = sum(values: move view);
    if total != 8_u64 {
      set code = 3_u8;
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
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let bytes = array_new::<u8, 2>(0_u8);
  region {
    let window = slice_of(&bytes);
    let value = window[2_u64];
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len(window)"));
}

#[test]
fn returned_slice_descriptors_execute_without_transferring_storage() {
    let source = br#"const fixed: array<u8, 2> =[7_u8, 13_u8];

fn pass['r](value: own slice<'r, u8>) -> result: own slice<'r, u8> pure {
  return move value;
}

fn choose['r](take_left: own Bool, left: own slice<'r, u8>, right: own slice<'r, u8>) -> result: own slice<'r, u8> pure {
  if take_left {
    return move left;
  } else {
    return move right;
  }
}

fn fixed_view['r]() -> result: own slice<'r, u8> pure {
  return slice_of(&'r fixed);
}

fn borrowed_first(value: &slice<u8>) -> result: own u8 reads(value) contract {
  define room = len(deref(value));
  requires 0_u64 < room;
} {
  return deref(value)[0_u64];
}

command fn main() -> status: own ExitStatus pure {
  let left = array_new::<u8, 2>(11_u8);
  let right = array_new::<u8, 2>(29_u8);
  region 'view {
    let borrowed_source = slice_of(&left);
    region {
      let borrowed_value = borrowed_first(value: &borrowed_source);
      if borrowed_value != 11_u8 {
        return exit_status(code: 1_u8);
      }
    }
    let initial = slice_of(&left);
    let passed = pass(value: move initial);
    let passed_room = len(passed);
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
    let selected = choose(take_left: take_left, left: move left_view, right: move right_view);
    let selected_room = len(selected);
    if 0_u64 < selected_room {
      let selected_value = selected[0_u64];
      if selected_value != 29_u8 {
        return exit_status(code: 3_u8);
      }
    } else {
      return exit_status(code: 3_u8);
    }
    let constant = fixed_view::<'view>();
    let constant_room = len(constant);
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
