use super::{compile, compile_and_run, compile_rejection, emitted_function};

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

/// LEFT ON `array<T, n>` DELIBERATELY: this test's subject is the array's own
/// inline-fill lowering, and the run has no twin for it. `array_new` is one
/// operation that fills every slot, which is what `array.fill.head` and
/// `array.fill.done` name; a run is filled by a source-level counted loop
/// [BLK-1], so there is no emitted fill region to assert over. Dropping those
/// two assertions to migrate the rest would loosen the test, so the retirement
/// batch retires it with that explanation instead.
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

fn read(values: own array<u16, 4>, offset: own u64) -> result: own u16 pure {
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
