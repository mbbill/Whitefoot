use super::support::{compile_and_run, compile_program, emitted_function};

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
