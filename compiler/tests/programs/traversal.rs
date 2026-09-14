//! Ordinary directory values and prelude functions, end to end.
//!
//! Every case here compiles a real corpus program against the declared
//! inventory, links it, and runs it against a real directory tree the harness
//! writes with ordinary filesystem calls. Nothing is injected into the
//! program's address space: it opens, enumerates, and descends through the
//! host's own facilities, exactly as an ordinary linked program would.

use super::support::compile_rejection;
use super::support::{build_program, close_path, compile_program, fixture_directory, reopen_path};

/// The traversal program itself: a recursive walk of the invocation
/// directory that collects every entry's kind and relative path into the
/// growable byte-string layer and publishes them in sorted order.
///
/// The fixture is a real three-level tree, so a green run establishes that
/// `open_directory_source` produced an independent enumeration handle, that `directory_next`
/// normalized the host's own records into the portable form, and that
/// `open_directory` opened each child ordinary directory value by name bytes with no path
/// value ever formed.
#[test]
fn the_traversal_program_walks_a_real_tree_and_publishes_it_sorted() {
    let llvm = compile_program("dir_walk.wf");
    // Compilation itself establishes every source-level array and buffer
    // domain. The emitted module may still carry the separately deferred heap
    // resource-abort path, but no source proof failure may survive lowering.
    assert!(
        !llvm.contains("call void @wf_trap(ptr @.wf_trap."),
        "the traversal must execute only operations admitted by static proof"
    );
    // C2 deletes QUAL-1 and the compiler-owned native wrapper. All three
    // operations use the ordinary callable ABI; the native engine is linked
    // separately and contributes no compiler declaration or permission.
    assert!(llvm.contains("@wf_open_directory_source("));
    assert!(llvm.contains("@wf_directory_next("));
    assert!(llvm.contains("@wf_open_directory("));
    assert!(!llvm.contains("@wf__completion_directory_next_submit("));

    let program = build_program(&llvm);
    let fixture = fixture_directory();
    fixture.write(b"a.txt", b"first\n");
    fixture.write(b"z.txt", b"last\n");
    fixture.write_nested("sub/b.txt", b"second\n");
    fixture.write_nested("sub/deeper/c.txt", b"third\n");

    let output = program.run(fixture.path(), &[]);
    assert!(
        output.status.success(),
        "traversal exited {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "1 a.txt\n2 sub\n1 sub/b.txt\n2 sub/deeper\n1 sub/deeper/c.txt\n1 z.txt\n"
    );
}

/// An empty directory yields exactly the entries the host reports and nothing
/// else: the self and parent entries reach source and the program skips them,
/// which is the whole reason the family contract states that they are
/// delivered rather than filtered.
#[test]
fn an_empty_tree_publishes_nothing_after_the_self_and_parent_entries_are_skipped() {
    let llvm = compile_program("dir_walk.wf");
    let program = build_program(&llvm);
    let fixture = fixture_directory();
    let output = program.run(fixture.path(), &[]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

/// A name the program cannot open still ends the walk cleanly: the entry is
/// recorded from its enumeration record and the failed descent is an ordinary
/// recoverable outcome rather than a failed source proof.
#[test]
fn an_unreadable_subdirectory_is_recorded_without_descending_into_it() {
    let llvm = compile_program("dir_walk.wf");
    let program = build_program(&llvm);
    let fixture = fixture_directory();
    fixture.write(b"a.txt", b"first\n");
    let closed = fixture.directory("closed");
    fixture.write_nested("closed/hidden.txt", b"hidden\n");
    close_path(&closed);

    let output = program.run(fixture.path(), &[]);

    reopen_path(&closed, 0o755);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "1 a.txt\n2 closed\n"
    );
}

/// A held enumeration handle is an ordinary linear value: using its moved
/// place is rejected by OWN-1, without a traversal-specific rule.
#[test]
fn an_enumeration_handle_is_not_usable_after_it_is_moved() {
    let source = br#"fn moved(list: own DirectorySource, destination: &uniq MutSlice<u8>) -> result: own unit reads(list, destination), writes(list, destination) {
  doc "Moves an ordinary linear enumeration value and then borrows its dead place.";
  let taken = move list;
  region {
    let (outcome, endpoint, reported) = directory_next(source: &uniq list, destination: &uniq deref(destination), start: 0_u64, end: 0_u64);
  }
  return unit;
}
"#;
    let failure = compile_rejection(&[("moved_list.wf", source)]);
    assert!(
        failure.contains("Semantics") && failure.contains("Own1"),
        "expected an ownership rejection, got {failure}"
    );
}

/// A run of name bytes is not a path value: no operation turns program bytes
/// into a `RelativePath`: the ordinary PRE-1 `relative_path` declaration
/// takes a `HostString`. The rejection is TYPE-5's type mismatch.
#[test]
fn program_bytes_still_cannot_become_a_path_value() {
    let source = br#"fn make_path['heap](name: own Vector<'heap, u8>) -> result: own unit pure {
  doc "Attempts to construct a path from run bytes rather than the declared HostString parameter.";
  match relative_path(value: move name) {
    Ok(value: path) => {
    }
    Err(error: problem) => {
    }
  }
  return unit;
}
"#;
    let failure = compile_rejection(&[("run_path.wf", source)]);
    assert!(
        failure.contains("Semantics") && failure.contains("Type5"),
        "expected a type rejection, got {failure}"
    );
}

/// The enumeration status is an ordinary Result enum, so portable control
/// flow over it must be exhaustive under ERR-2 and a
/// missing arm is a rejection rather than a silent fallthrough.
#[test]
fn an_enumeration_match_that_omits_an_outcome_is_rejected() {
    let source = br#"fn partial(source: &uniq DirectorySource, destination: &uniq MutSlice<u8>) -> result: own unit reads(source, destination), writes(source, destination) {
  doc "Omits the error arm of the ordinary enumeration status result.";
  region {
    let (outcome, endpoint, reported) = directory_next(source: &uniq deref(source), destination: &uniq deref(destination), start: 0_u64, end: 0_u64);
    match outcome {
      Ok(value: completed) => {
      }
    }
  }
  return unit;
}
"#;
    let failure = compile_rejection(&[("partial_list.wf", source)]);
    assert!(
        failure.contains("Semantics") && failure.contains("Err2"),
        "expected an exhaustiveness rejection, got {failure}"
    );
}
