//! Derived recursive cleanup and precise diagnostic regressions on real programs.
use super::{compile, compile_and_run, compile_rejection, emitted_function};

fn derived_drop<'module>(llvm: &'module str, prefix: &str) -> &'module str {
    let start = llvm
        .find(prefix)
        .unwrap_or_else(|| panic!("the module must define {prefix}"));
    let end = llvm[start..]
        .find("\n}\n")
        .map(|offset| start + offset)
        .expect("a definition must close");
    &llvm[start..end]
}

fn search_layer_with_entry() -> String {
    let source = include_str!("../../../../tests/programs/byte_string.wf");
    let start = source
        .find("enum Grown['heap] {")
        .expect("byte-string growth outcome");
    let end = source
        .find("\nfn bs_push_decimal")
        .expect("search-layer end");
    let layer = &source[start..end];
    format!(
        "{layer}
fn main['heap](heap: own Heap<'heap>) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {{
  region {{
    match heap_vector::<u8>(store: &uniq heap, count: 1_u64) {{
      None() => {{
        return exit_status(code: 70_u8);
      }}
      Some(value: fresh) => {{
        let subject = move fresh;
        region {{
          place_back(vector: &uniq subject, value: 7_u8);
        }}
        region {{
          match heap_vector::<u8>(store: &uniq heap, count: 1_u64) {{
            None() => {{
              return exit_status(code: 70_u8);
            }}
            Some(value: other) => {{
              let needle = move other;
              region {{
                place_back(vector: &uniq needle, value: 7_u8);
              }}
              region {{
                match bs_find(haystack: &subject, needle: &needle) {{
                  Some(value: at) => {{
                  }}
                  None() => {{
                  }}
                }}
              }}
            }}
          }}
        }}
      }}
    }}
  }}
  return exit_status(code: 0_u8);
}}
"
    )
}

#[test]
fn affine_slot_buffers_fill_replace_vacate_and_drop_per_element() {
    let llvm = compile(include_bytes!("../../../../tests/programs/option_slots.wf"));
    // B7c4b-1: the two slot runs are `FixedVector<Option<T>, n>`s built by the
    // library's own `vacant` generic, so there is no `buffer_vacant` head to
    // name and no per-buffer drop helper. What the row still owes is the
    // release of the one `Some` cell the program leaves in a slot: the run is
    // frame-resident, its element drop is derived over the slots, and the cell
    // it holds is freed to the general store.
    assert!(!llvm.contains("buffer.vacant.head"));
    assert!(llvm.contains("call ptr @malloc"));
    let helper = derived_drop(&llvm, "define private void @wf.drop.t");
    assert!(helper.contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn recursively_boxed_tree_executes_with_derived_cleanup() {
    let llvm = compile(include_bytes!(
        "../../../../tests/programs/recursive_tree.wf"
    ));
    let count = emitted_function(&llvm, "count");
    assert!(count.contains("call") && count.contains("@wf_count"));
    assert!(llvm.contains("call ptr @malloc"));
    assert!(llvm.contains("icmp ne ptr"));
    assert!(llvm.contains("call void @free"));
    // A recursive enum's derived release is one release action per node type
    // that enters itself at the closing edge of its release graph [PROV-6]:
    // the owner deleted the cycle refusal on 2026-09-04 and ruled that the
    // walk may recurse, so the worklist driver that kept the depth off the
    // machine stack (an allocation and an abort on the release path) is gone.
    // Each of the two boxed children is released by the same action, and each
    // block is freed by the action that owns it.
    let entry = derived_drop(&llvm, "define private void @wf.drop.t");
    let name = entry
        .strip_prefix("define private void @")
        .and_then(|rest| rest.split('(').next())
        .expect("a derived release action name");
    assert_eq!(entry.matches(&format!("call void @{name}(")).count(), 2);
    assert_eq!(entry.matches("call void @free(").count(), 2);
    assert!(!llvm.contains("@wf.drop.step."));
    assert!(!llvm.contains("@wf.drop.push"));
    assert!(!llvm.contains("@wf.drop.run"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn the_byte_accessor_without_its_length_branch_is_an_op4_rejection() {
    let guarded = "  let within = index < stored;";
    let unguarded = "  let within = index <= stored;";
    let source = search_layer_with_entry();
    let stripped = source.replace(guarded, unguarded);
    assert_ne!(stripped, source, "the length branch must have been found");
    let failure = compile_rejection(stripped.as_bytes()).to_string();
    assert!(failure.contains("[OP-4]"), "{failure}");
    assert!(failure.contains("index < len_of(deref(s))"), "{failure}");
}
