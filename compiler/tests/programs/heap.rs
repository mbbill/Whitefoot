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
    // The construction is the all-None allocation and the drop of the
    // box-payload buffer is the derived per-element loop plus one free.
    assert!(llvm.contains("buffer.vacant.head"));
    let helper_start = llvm
        .find("define private void @wf.drop.buffer.t")
        .expect("box-payload elements must derive the buffer drop loop");
    let helper_end = llvm[helper_start..]
        .find("\n}\n")
        .map(|offset| helper_start + offset)
        .expect("buffer drop helper must be complete");
    let helper = &llvm[helper_start..helper_end];
    assert!(helper.contains("call void @wf.drop.t"));
    assert_eq!(helper.matches("call void @free").count(), 1);

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
    // A recursive enum's derived drop is a traversal, not a level of one: the
    // entry point sets up a worklist and runs it, and the per-node step hands
    // each of the two boxed children to that worklist instead of descending
    // into it. The two frees the straight-line helper used to perform are the
    // traversal's, one for each block it takes off the list.
    let entry = derived_drop(&llvm, "define private void @wf.drop.t");
    assert!(entry.contains("call void @wf.drop.step."));
    assert!(entry.contains("call void @wf.drop.run(ptr %work)"));
    let step = derived_drop(&llvm, "define private void @wf.drop.step.");
    assert_eq!(step.matches("call void @wf.drop.push").count(), 2);
    assert!(!step.contains("call void @wf.drop.t"));
    let traversal = derived_drop(&llvm, "define private void @wf.drop.run");
    assert!(traversal.contains("call void @free(ptr %node)"));

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
/// return a nonzero status before publication; the one retained claim is the
/// report-capacity representation theorem consumed by `publish_all`.
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
command fn main() -> status: own ExitStatus allocates(heap) {{
  let backing = buffer_new(1_u64, 7_u8);
  let subject = ByteString(buf: move backing, fill: 1_u64);
  let needle_backing = buffer_new(1_u64, 7_u8);
  let needle = ByteString(buf: move needle_backing, fill: 1_u64);
  region 'search {{
    match bs_find<'search, 'search>(haystack: &'search subject, needle: &'search needle) {{
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
    let capacity = len(deref(s).buf);
    let addressable = ilt(index, capacity);
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
    assert!(failure.contains("index < len(deref(s).buf)"), "{failure}");
}

/// `deny_claims` on the search path is enforced, not annotation.
///
/// Injecting one genuine residual claim into `bs_find` is still a [CLM-3]
/// rejection at the claim node. The claim checks a current-function local
/// index and is immediately consumed by that subscript, so neither redundancy
/// nor a later [OP-4] error can masquerade as the `deny_claims` result. The
/// search layer is claim-free because it must be.
#[test]
fn a_claim_injected_into_the_strict_search_is_a_clm3_rejection() {
    let declaration = "deny_claims fn bs_find['h, 'n](haystack: &'h ByteString, needle: &'n ByteString) -> result: own Option<u64> reads(haystack.buf, haystack.fill, needle.buf, needle.fill) {";
    let trapping = "deny_claims fn bs_find['h, 'n](haystack: &'h ByteString, needle: &'n ByteString) -> result: own Option<u64> reads(haystack.buf, haystack.fill, needle.buf, needle.fill), traps {";
    let entry = "command fn main() -> status: own ExitStatus allocates(heap) {";
    let trapping_entry = "command fn main() -> status: own ExitStatus allocates(heap), traps {";
    let anchor = "  let last = hay_length -wrap needle_length;\n";
    let claim_source = "  let last = hay_length -wrap needle_length;
  let proof_values = array_new<u8, 4>(0_u8);
  let bounded_probe = last % 4_u64;
  let probe_inside = ilt(bounded_probe, 4_u64);
  claim search_probe_in_bounds: probe_inside because \"premises: bounded_probe is last remainder 4_u64 computed in the current function\\nderivation: unsigned remainder by four is one of 0_u64 through 3_u64 and is therefore strictly less than 4_u64\\nconclusion: probe_inside is True\\nchecker gap: ENT proves the remainder operation domain but does not publish its result range\\nconsumers: the immediately following proof_values[bounded_probe] subscript requires this exact bound\";
  let consumed_probe = proof_values[bounded_probe];
";
    let source = search_layer_with_entry();
    let retyped_function = source.replace(declaration, trapping);
    assert_ne!(
        retyped_function, source,
        "the bs_find declaration must have been found"
    );
    let retyped = retyped_function.replace(entry, trapping_entry);
    assert_ne!(
        retyped, retyped_function,
        "the command effect row must track the injected trap"
    );
    let claiming = retyped.replace(anchor, claim_source);
    assert_ne!(claiming, retyped, "the claim anchor must have been found");
    let failure = compile_rejection(&[("byte_string_claiming.wf", claiming.as_bytes())]);
    assert!(failure.contains("[CLM-3]"), "{failure}");
    assert!(failure.contains("search_probe_in_bounds"), "{failure}");
}
