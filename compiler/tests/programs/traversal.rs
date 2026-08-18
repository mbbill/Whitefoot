//! The v0.32-candidate [SYS-14] directory-enumeration surface, end to end.
//!
//! Every case here compiles a real corpus program against the candidate
//! inventory, links it, and runs it against a real directory tree the harness
//! writes with ordinary filesystem calls. Nothing is injected into the
//! program's address space: it opens, enumerates, and descends through the
//! host's own facilities, exactly as a shipped command would.

use std::path::Path;

use super::support::{
    build_program, compile_program, compile_program_with_traversal_surface,
    compile_rejection_with_traversal_surface, fixture_directory,
};

/// The traversal program itself: a recursive walk of the invocation
/// directory that collects every entry's kind and relative path into the
/// growable byte-string layer and publishes them in sorted order.
///
/// The fixture is a real three-level tree, so a green run establishes that
/// `open_list` produced an independent enumeration handle, that `list_once`
/// normalized the host's own records into the portable form, and that
/// `open_directory` opened each child capability by name bytes with no path
/// value ever formed.
#[test]
fn the_traversal_program_walks_a_real_tree_and_publishes_it_sorted() {
    let llvm = compile_program_with_traversal_surface("dir_walk.wf");
    // The three approved implementations and the target's own enumeration
    // facility, by symbol rather than by any source name [QUAL-1].
    assert!(llvm.contains("@wf.sys.open_list.v1"));
    assert!(llvm.contains("@wf.sys.list_once.v1"));
    assert!(llvm.contains("@wf.sys.open_directory.v1"));
    assert!(llvm.contains("__getdirentries64"));

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
    let llvm = compile_program_with_traversal_surface("dir_walk.wf");
    let program = build_program(&llvm);
    let fixture = fixture_directory();
    let output = program.run(fixture.path(), &[]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

/// A name the program cannot open still ends the walk cleanly: the entry is
/// recorded from its enumeration record and the failed descent is an ordinary
/// recoverable outcome, not a trap.
#[test]
fn an_unreadable_subdirectory_is_recorded_without_descending_into_it() {
    let llvm = compile_program_with_traversal_surface("dir_walk.wf");
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

/// The same source is not a program under the active specification: with the
/// candidate inventory off, every traversal spelling is an undeclared name,
/// so the switch decides admission rather than the compiler recognizing a
/// source shape.
#[test]
fn the_traversal_source_is_undeclared_without_the_candidate_inventory() {
    let source = std::fs::read(traversal_source()).expect("read the traversal program");
    let inputs = [("dir_walk.wf", source.as_slice())];
    let failure = super::support::compile_rejection(&inputs);
    assert!(
        failure.contains("Resolution"),
        "expected a resolution rejection, got {failure}"
    );
}

/// Every v0.31 program keeps its exact emitted module when the candidate
/// inventory is selected, because the candidate only adds declarations: no
/// v0.31 spelling, ordinal, or lowering decision moves.
#[test]
fn the_candidate_inventory_leaves_every_v031_program_byte_identical() {
    for name in ["wfgrep.wf", "byte_string.wf", "growable_vec.wf"] {
        assert_eq!(
            compile_program(name),
            compile_program_with_traversal_surface(name),
            "{name} emits different bytes under the candidate inventory"
        );
    }
}

/// A held enumeration handle is affine like every other system resource: a
/// source that uses one after moving it is rejected, and the rejection comes
/// from ownership rather than from any traversal-specific rule.
#[test]
fn an_enumeration_handle_is_not_usable_after_it_is_moved() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> own ExitStatus external, blocks, traps {
  doc "Moves one enumeration handle and then uses the moved binding.";
  let scratch = buffer_new(64_u64, 0_u8);
  region 'listing {
    match open_list<'listing>(directory: &'listing cwd) {
      Ok(value: list) => {
        let taken = move list;
        region 'step {
          match list_once<'step, 'step>(list: &uniq 'step list, destination: &uniq 'step scratch, offset: 0_u64, capacity: 64_u64) {
            ListBytes(count: bytes, entries: reported) => {
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
    let failure = compile_rejection_with_traversal_surface(&[("moved_list.wf", source)]);
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
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> own ExitStatus external, blocks, traps {
  doc "Attempts to construct a relative path from program bytes.";
  let name = buffer_new(8_u64, 97_u8);
  region 'attempt {
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
    let failure = compile_rejection_with_traversal_surface(&[("buffer_path.wf", source)]);
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
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> own ExitStatus external, blocks, traps {
  doc "Omits one enumeration outcome from an otherwise complete match.";
  let scratch = buffer_new(64_u64, 0_u8);
  region 'listing {
    match open_list<'listing>(directory: &'listing cwd) {
      Ok(value: list) => {
        region 'step {
          match list_once<'step, 'step>(list: &uniq 'step list, destination: &uniq 'step scratch, offset: 0_u64, capacity: 64_u64) {
            ListBytes(count: bytes, entries: reported) => {
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
    let failure = compile_rejection_with_traversal_surface(&[("partial_list.wf", source)]);
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
/// constructs the portable class and returns, and the one `openat` call site
/// is reachable only after the length and byte scan admitted the range.
#[test]
fn the_component_validation_precedes_every_host_call() {
    let llvm = compile_program_with_traversal_surface("dir_walk.wf");
    let start = llvm
        .find("@wf.sys.open_directory.v1(")
        .expect("the emitted open_directory implementation");
    let body_start = llvm[..start].rfind("\ndefine private").expect("its header") + 1;
    let body_end = body_start + llvm[body_start..].find("\n}\n").expect("its closing brace");
    let shim = &llvm[body_start..body_end];

    assert!(shim.contains("%oversize = icmp ugt i64 %count, 255"));
    assert!(shim.contains("%vacant = icmp eq i64 %count, 0"));
    assert!(shim.contains("%separating = icmp eq i32 %byte.value, 47"));
    assert!(shim.contains("%terminating = icmp eq i32 %byte.value, 0"));

    let open_block = shim.find("\nopen:\n").expect("the admitted-name block");
    let host_call = shim.find("@openat").expect("the directory-relative open");
    assert!(
        host_call > open_block,
        "the host call must be reachable only from the admitted-name block"
    );
    let invalid_block = shim.find("\ninvalid:\n").expect("the rejection block");
    assert!(
        !shim[invalid_block..].contains("@openat"),
        "the rejection path must make no host call"
    );
}

fn traversal_source() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler package must live directly under the repository root")
        .join("tests")
        .join("programs")
        .join("dir_walk.wf")
}
