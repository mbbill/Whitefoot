use super::support::{compile_and_run, compile_program, compile_rejection, emitted_function};

/// One compiler-derived drop definition, from the first `define` line whose
/// symbol starts with `prefix` through its closing brace.
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

/// The [SET-2] library layer: a growable byte vector over `buffer<u8>` whose
/// push grows by allocate-new + copy + affine field replace + scope-exit
/// release of the superseded buffer, and a byte-string append built on it.
/// The program self-checks length, capacity, and content after three grow
/// cycles (capacity 0 -> 8 -> 16 -> 24) and after appending five bytes; a
/// wrong value at any probe returns a nonzero status and fails the run.
#[test]
fn growable_vector_grows_by_affine_replace_and_runs_its_checks() {
    let llvm = compile_program("growable_vec.wf");
    // The superseded buffer's release is the ordinary compiler-derived heap
    // free of the abandoned old-value binding [SET-2, STOR-3].
    assert!(llvm.contains("call ptr @malloc"));
    assert!(llvm.contains("call void @free"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// The affine-element buffer layer [TYPE-2, SET-2, OP-9, STOR-3]: an
/// `OptVec` over `buffer<Option<u32>>` built by `buffer_vacant`, filled and
/// vacated by element-position replace through a `&uniq` holder, plus a
/// `buffer<Option<box<u64>>>` section whose scope-exit drop is the
/// per-element loop (one remaining `Some` box is freed by the loop). The
/// program self-checks pop order, fill accounting, and the taken payload;
/// a wrong value at any probe returns a nonzero status and fails the run.
#[test]
fn affine_slot_buffers_fill_replace_vacate_and_drop_per_element() {
    let llvm = compile_program("option_slots.wf");
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
    let llvm = compile_program("recursive_tree.wf");
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

/// The byte-string layer over the [SET-2] growable buffer: construction from
/// literal arrays, append and concat by buffer growth, length, bounds-safe
/// byte access, a naive substring search, and decimal formatting, ending in
/// one real publication to standard output.
///
/// The oracle is the published line itself. Intermediate result mismatches
/// return a nonzero status before publication, and every partial operation is
/// admitted by ordinary control-flow facts or a checked contract.
#[test]
fn byte_string_builds_searches_and_publishes_its_report() {
    let llvm = compile_program("byte_string.wf");
    // Growth allocates the wider buffer and releases the superseded one
    // through the ordinary [SET-2], [STOR-3] path.
    assert!(llvm.contains("call ptr @malloc"));
    assert!(llvm.contains("call void @free"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"length=43 brown=10 cat=none\n");
    assert!(output.stderr.is_empty());
}

/// The read-only search layer of `byte_string.wf`, with its own entry.
///
/// Both negative directions below rewrite exactly one construct of these
/// bytes, so each shows that the construct is load-bearing rather than
/// decoration. The extraction ends before `bs_push_decimal`, the first
/// declaration the search layer does not use.
fn search_layer_with_entry() -> String {
    let source = include_str!("../../../tests/programs/byte_string.wf");
    let start = source
        .find("struct ByteString {")
        .expect("byte-string struct");
    let end = source
        .find("\nfn bs_push_decimal")
        .expect("search-layer end");
    let layer = &source[start..end];
    format!(
        "{layer}
command fn main() -> status: own ExitStatus pure {{
  let backing = buffer_new(1_u64, 7_u8);
  let subject = ByteString(buf: move backing, fill: 1_u64);
  let needle_backing = buffer_new(1_u64, 7_u8);
  let needle = ByteString(buf: move needle_backing, fill: 1_u64);
  region {{
    match bs_find(haystack: &subject, needle: &needle) {{
      Some(value: at) => {{
      }}
      None() => {{
      }}
    }}
  }}
  return exit_status(code: 0_u8);
}}
"
    )
}

/// The accessor's inner capacity branch is the bounds discharge itself.
///
/// Deleting it leaves the same subscript with no established bound, and the
/// compiler rejects the program under [OP-4] rather than accepting it with a
/// runtime check: the accessor is safe because the branch proves it, not
/// because a check was left in.
#[test]
fn the_byte_accessor_without_its_capacity_branch_is_an_op4_rejection() {
    let guarded = "  if within {
    let capacity = len_of(deref(s).buf);
    let addressable = index < capacity;
    if addressable {
      let value = deref(s).buf[index];
      return Some<u8>(value: value);
    }
  }";
    let unguarded = "  if within {
    let value = deref(s).buf[index];
    return Some<u8>(value: value);
  }";
    let source = search_layer_with_entry();
    let stripped = source.replace(guarded, unguarded);
    assert_ne!(stripped, source, "the capacity branch must have been found");
    let failure = compile_rejection(&[("byte_string_unguarded.wf", stripped.as_bytes())]);
    assert!(failure.contains("[OP-4]"), "{failure}");
    assert!(
        failure.contains("index < len_of(deref(s).buf)"),
        "{failure}"
    );
}
