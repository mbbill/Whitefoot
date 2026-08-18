use super::support::{compile_and_run, compile_program, compile_rejection, emitted_function};

/// The [SET-2] library layer: a growable byte vector over `buffer<u8>` whose
/// push grows by allocate-new + copy + affine field replace + scope-exit
/// release of the superseded buffer, and a byte-string append built on it.
/// The program self-checks length, capacity, and content after three grow
/// cycles (capacity 0 -> 8 -> 16 -> 24) and after appending five bytes; a
/// wrong value at any probe traps and fails the run.
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

#[test]
fn recursively_boxed_tree_executes_with_derived_cleanup() {
    let llvm = compile_program("recursive_tree.wf");
    let count = emitted_function(&llvm, "count");
    assert!(count.contains("call i64 @wf_count"));
    assert!(llvm.contains("call ptr @malloc"));
    assert!(llvm.contains("icmp ne ptr"));
    assert!(llvm.contains("call void @free"));
    let drop_start = llvm
        .find("define private void @wf.drop")
        .expect("recursive enum must have a derived drop helper");
    let drop_end = llvm[drop_start..]
        .find("\n}\n\n")
        .map(|offset| drop_start + offset)
        .expect("drop helper must be complete");
    let drop_helper = &llvm[drop_start..drop_end];
    assert!(drop_helper.contains("call void @wf.drop"));
    assert_eq!(drop_helper.matches("call void @free").count(), 2);

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
/// The oracle is the published line itself. Every intermediate result is also
/// self-checked in source, so a wrong length, byte, or match position traps
/// and fails the run before anything is published.
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
fn main() -> own unit allocates(heap), traps {{
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
  return unit;
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
/// Injecting one claim into `bs_find` — a claim the checker can itself
/// prove, so its lifecycle is `Redundant` — is still a [CLM-3] rejection at
/// the claim node. The search layer is claim-free because it must be.
#[test]
fn a_claim_injected_into_the_strict_search_is_a_clm3_rejection() {
    let declaration = "deny_claims fn bs_find['h, 'n](haystack: &'h ByteString, needle: &'n ByteString) -> own Option<u64> reads('h 'n) {";
    let trapping = "deny_claims fn bs_find['h, 'n](haystack: &'h ByteString, needle: &'n ByteString) -> own Option<u64> reads('h 'n), traps {";
    let anchor = "  let last = hay_length -wrap needle_length;\n";
    let asserted = "  let last = hay_length -wrap needle_length;
  let ordered = ile(needle_length, hay_length);
  claim needle_fits: ordered because \"the earlier branch left the needle no longer than the haystack\";
";
    let source = search_layer_with_entry();
    let retyped = source.replace(declaration, trapping);
    assert_ne!(
        retyped, source,
        "the bs_find declaration must have been found"
    );
    let claiming = retyped.replace(anchor, asserted);
    assert_ne!(claiming, retyped, "the claim anchor must have been found");
    let failure = compile_rejection(&[("byte_string_claiming.wf", claiming.as_bytes())]);
    assert!(failure.contains("[CLM-3]"), "{failure}");
    assert!(failure.contains("needle_fits"), "{failure}");
}
