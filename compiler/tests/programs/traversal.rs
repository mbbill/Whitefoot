//! The [SYS-14] directory-enumeration surface, end to end.
//!
//! Every case here compiles a real corpus program against the declared
//! inventory, links it, and runs it against a real directory tree the harness
//! writes with ordinary filesystem calls. Nothing is injected into the
//! program's address space: it opens, enumerates, and descends through the
//! host's own facilities, exactly as a shipped command would.

use super::support::compile_rejection;
use super::support::{
    build_program, compile_program, compile_program_rejection_with, fixture_directory,
};
use whitefoot::Inventory;

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
    // The three approved implementations and the compiler-owned target
    // progress wrapper, by symbol rather than by any source name [QUAL-1].
    assert!(llvm.contains("@wf.sys.open_directory_source.v1"));
    assert!(llvm.contains("@wf.sys.directory_next.v1"));
    assert!(llvm.contains("@wf.sys.open_directory.v1"));
    assert!(llvm.contains("wf__completion_directory_next_direct"));
    assert!(!llvm.contains("call i64 @__getdirentries64"));

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
    let mut permissions = std::fs::metadata(&closed)
        .expect("fixture directory metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o000);
    std::fs::set_permissions(&closed, permissions).expect("close the fixture directory");

    let output = program.run(fixture.path(), &[]);

    let mut restored = std::fs::metadata(&closed)
        .expect("fixture directory metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut restored, 0o755);
    std::fs::set_permissions(&closed, restored).expect("reopen the fixture directory");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "1 a.txt\n2 closed\n"
    );
}

/// The current traversal source requires the complete file-permit inventory,
/// not just the older traversal rows. This is an honest source dependency:
/// its entry receives FileFactory and every open calls reserve_file.
#[test]
fn the_traversal_source_requires_the_complete_file_permit_inventory() {
    let _ = compile_program("dir_walk.wf");
    let failure = compile_program_rejection_with("dir_walk.wf", Inventory::OpenByName);
    assert!(
        failure.contains("UnresolvedUse")
            && (failure.contains("FileFactory") || failure.contains("reserve_file")),
        "the pre-permit inventory must reject the explicit authority surface: {failure}"
    );
}

// The former byte-identical comparison between the traversal and open-by-name
// inventories was retired with the file-permit amendment. The amendment adds
// two nominal types and changes every open signature, so byte identity across
// those superseded inventories is no longer a valid invariant. Catalog tests
// retain their exact counted membership; current programs compile only against
// the complete active inventory.

/// A held enumeration handle is affine like every other system resource: a
/// source that uses one after moving it is rejected, and the rejection comes
/// from ownership rather than from any traversal-specific rule.
#[test]
fn an_enumeration_handle_is_not_usable_after_it_is_moved() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Moves one enumeration handle and then uses the moved binding.";
  let scratch = buffer_new(64_u64, 0_u8);
  region {
    let permit = reserve_file(factory: &uniq files);
    match open_directory_source(permit: move permit, directory: &cwd) {
      Ok(value: list) => {
        let taken = move list;
        region {
          match directory_next(source: &uniq list, destination: &uniq scratch, start: 0_u64, end: 64_u64) {
            ListBytes(next: endpoint, entries: reported) => {
            }
            ListEnd() => {
            }
            ListFailed(error: problem) => {
            }
          }
        }
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(&[("moved_list.wf", source)]);
    assert!(
        failure.contains("Semantics"),
        "expected an ownership rejection, got {failure}"
    );
}

/// A name buffer is not a path value: no operation turns program bytes into a
/// `RelativePath`, so the deferred path algebra of [PATH-1] stays deferred
/// even with the traversal surface admitted.
#[test]
fn program_bytes_still_cannot_become_a_path_value() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> status: own ExitStatus writes(cwd) {
  doc "Attempts to construct a relative path from program bytes.";
  let name = buffer_new(8_u64, 97_u8);
  region {
    match relative_path(value: move name) {
      Ok(value: path) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(&[("buffer_path.wf", source)]);
    assert!(
        failure.contains("Semantics"),
        "expected a type rejection, got {failure}"
    );
}

/// The enumeration outcome is a three-constructor enum like every other
/// [SYS-6] outcome type, so portable control flow over it is exhaustive and a
/// missing arm is a rejection rather than a silent fallthrough.
#[test]
fn an_enumeration_match_that_omits_an_outcome_is_rejected() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Omits one enumeration outcome from an otherwise complete match.";
  let scratch = buffer_new(64_u64, 0_u8);
  region {
    let permit = reserve_file(factory: &uniq files);
    match open_directory_source(permit: move permit, directory: &cwd) {
      Ok(value: list) => {
        region {
          match directory_next(source: &uniq list, destination: &uniq scratch, start: 0_u64, end: 64_u64) {
            ListBytes(next: endpoint, entries: reported) => {
            }
            ListEnd() => {
            }
          }
        }
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(&[("partial_list.wf", source)]);
    assert!(
        failure.contains("Semantics"),
        "expected an exhaustiveness rejection, got {failure}"
    );
}

/// The component-name validation precedes the host call, so a name that is
/// not one path component is refused with no directory-relative open at all.
///
/// This is structural evidence about the emitted implementation rather than a
/// runtime observation: the rejection path is a separate block that
/// constructs the portable class and returns, and the one typed completion
/// call site is reachable only after the length and byte scan admitted the
/// range. The target adapter below this emitted boundary owns `openat`.
#[test]
fn the_component_validation_precedes_every_host_call() {
    let llvm = compile_program("dir_walk.wf");
    let start = llvm
        .find("@wf.sys.open_directory.v1(")
        .expect("the emitted open_directory implementation");
    let body_start = llvm[..start].rfind("\ndefine private").expect("its header") + 1;
    let body_end = body_start + llvm[body_start..].find("\n}\n").expect("its closing brace");
    let shim = &llvm[body_start..body_end];

    // The component limit is the selected target's own [SYS-14]: 1023 bytes
    // on the Darwin family, 255 on the Linux family. The constant is asserted
    // exactly on each host rather than matched loosely on both.
    let component_limit = if cfg!(target_os = "macos") { 1023 } else { 255 };
    assert!(shim.contains(&format!(
        "%oversize = icmp ugt i64 %extent, {component_limit}"
    )));
    assert!(shim.contains("%vacant = icmp eq i64 %extent, 0"));
    assert!(shim.contains("%separating = icmp eq i32 %byte.value, 47"));
    assert!(shim.contains("%terminating = icmp eq i32 %byte.value, 0"));

    let open_block = shim.find("\nopen:\n").expect("the admitted-name block");
    let host_call = shim
        .find("@wf__completion_file_open_at_direct")
        .expect("the typed directory-relative open");
    assert!(
        host_call > open_block,
        "the host call must be reachable only from the admitted-name block"
    );
    let invalid_block = shim.find("\ninvalid:\n").expect("the rejection block");
    assert!(
        !shim[invalid_block..].contains("@wf__completion_file_open_at_direct"),
        "the rejection path must make no host call"
    );
}
