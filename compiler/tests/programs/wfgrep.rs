//! End-to-end evidence for `tests/programs/wfgrep.wf`, the first real
//! Whitefoot command program, now a recursive search.
//!
//! wfgrep takes a pattern and one search root, walks the tree with the
//! [SYS-14] enumeration surface, opens each regular file it reaches by the
//! enumerated name with active [SYS-11] `open_file`, reads it, and
//! publishes `PATH:LINE:TEXT` for every matching line.
//!
//! Two oracles check it. The first is a trusted reference search written
//! here rather than a host `grep`: the two grep families on the supported
//! hosts disagree about unterminated final lines and about patterns that are
//! not valid text, and wfgrep's published bytes must be compared against one
//! fixed contract, not against whichever tool the host installs. The second
//! is the host's own `grep -rn` over the same fixture tree, on the fixture
//! shape where both families agree — that is the cross-check the plan item
//! asks for, and it is what establishes that the walk reaches the same files
//! a real recursive searcher reaches.

use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

use super::support::{
    CompiledProgram, build_program, compile_program, compile_program_rejection_with,
    fixture_directory,
};
use whitefoot::Inventory;

/// The reusable input buffer length in `tests/programs/wfgrep.wf`.
///
/// The corpus needs it to build a file that is exactly one buffer long and a
/// match that straddles a read boundary; nothing in the program's contract
/// exposes it.
const BUFFER_LENGTH: usize = 4096;

/// One emitted module shared by every case in this module.
///
/// Entailment over the nested walk and matcher is still the dominant compile
/// cost, so the module is produced once and every case reads it. Isolation
/// lives in each run's own fixture directory, never in the artifact.
fn wfgrep_module() -> &'static str {
    static MODULE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    MODULE.get_or_init(|| compile_program("wfgrep.wf"))
}

fn wfgrep() -> &'static CompiledProgram {
    static PROGRAM: std::sync::OnceLock<CompiledProgram> = std::sync::OnceLock::new();
    PROGRAM.get_or_init(|| build_program(wfgrep_module()))
}

/// The trusted reference search, over a real directory tree.
///
/// This is the frozen contract wfgrep implements: entries of one directory
/// are visited in ascending name-byte order with a shorter prefix first, a
/// directory is descended as soon as it is reached, a line is a maximal run
/// of bytes between newlines, an unterminated final run is a line, the
/// pattern is matched against the line without its terminator, the empty
/// pattern matches every line, and every published record is
/// `PATH:LINE:TEXT` with the line's own terminator.
fn reference(root: &Path, display: &[u8], pattern: &[u8]) -> (Vec<u8>, i32) {
    let mut published = Vec::new();
    let mut matched = false;
    visit(root, display, pattern, &mut published, &mut matched);
    (published, i32::from(!matched))
}

fn visit(
    directory: &Path,
    display: &[u8],
    pattern: &[u8],
    published: &mut Vec<u8>,
    hit: &mut bool,
) {
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .expect("read the reference fixture directory")
        .map(|entry| entry.expect("one reference fixture entry"))
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name();
        let name = name.as_bytes().to_vec();
        let mut path = display.to_vec();
        if !path.is_empty() {
            path.push(b'/');
        }
        path.extend_from_slice(&name);
        let kind = entry.file_type().expect("one reference fixture entry kind");
        if kind.is_file() {
            let content = std::fs::read(entry.path()).expect("read one reference fixture file");
            for (ordinal, line) in lines(&content).into_iter().enumerate() {
                if occurs(line, pattern) {
                    *hit = true;
                    published.extend_from_slice(&path);
                    published.push(b':');
                    published.extend_from_slice((ordinal + 1).to_string().as_bytes());
                    published.push(b':');
                    published.extend_from_slice(line);
                    published.push(b'\n');
                }
            }
        } else if kind.is_dir() {
            visit(&entry.path(), &path, pattern, published, hit);
        }
    }
}

fn lines(content: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, byte) in content.iter().enumerate() {
        if *byte == b'\n' {
            lines.push(&content[start..index]);
            start = index + 1;
        }
    }
    if start < content.len() {
        lines.push(&content[start..]);
    }
    lines
}

fn occurs(line: &[u8], pattern: &[u8]) -> bool {
    pattern.is_empty()
        || line
            .windows(pattern.len().max(1))
            .any(|window| window == pattern)
}

/// Runs wfgrep over one fixture tree and compares it with the reference.
fn assert_reference(program: &CompiledProgram, root: &Path, tree: &str, pattern: &[u8]) -> Vec<u8> {
    let output = program.run(root, &[pattern, tree.as_bytes()]);
    let (expected, status) = reference(&root.join(tree), tree.as_bytes(), pattern);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&expected),
        "published bytes disagree with the reference search"
    );
    assert_eq!(
        output.status.code(),
        Some(status),
        "status disagrees with the reference search; diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

/// The host `grep -rn` over the same tree, as an independent hit set.
///
/// Both sides are sorted because `grep -r` visits the tree in the host's own
/// enumeration order while wfgrep visits it in sorted order; the hit *set* is
/// the property under test, not the order the two tools chose.
fn grep_rn(root: &Path, tree: &str, pattern: &[u8]) -> Vec<String> {
    let output = Command::new("/usr/bin/grep")
        .arg("-rn")
        .arg("-e")
        .arg(std::ffi::OsStr::from_bytes(pattern))
        .arg(tree)
        .current_dir(root)
        .output()
        .expect("invoke the host grep");
    sorted_lines(&output.stdout)
}

fn sorted_lines(bytes: &[u8]) -> Vec<String> {
    let mut lines: Vec<String> = String::from_utf8_lossy(bytes)
        .lines()
        .map(str::to_owned)
        .collect();
    lines.sort();
    lines
}

/// One fixture tree both oracles read, deliberately built out of the shapes
/// the two grep families agree on: every file is terminated text, every name
/// is ordinary, and no symbolic link is involved.
fn search_tree() -> super::support::FixtureDirectory {
    let fixture = fixture_directory();
    fixture.directory("tree");
    fixture.write_nested("tree/alpha.txt", b"needle here\nplain\nneedle again\n");
    fixture.write_nested("tree/beta.txt", b"nothing\nat all\n");
    fixture.write_nested("tree/.hidden.txt", b"hidden needle\n");
    fixture.write_nested("tree/sub/gamma.txt", b"deep needle\n");
    fixture.write_nested("tree/sub/delta.txt", b"no match here\n");
    fixture.write_nested("tree/sub/deeper/epsilon.txt", b"deepest needle\nlast\n");
    fixture.write_nested("tree/zeta.txt", b"needle at the end\n");
    fixture
}

/// The headline evidence: a real recursive search over a real tree, checked
/// against the reference contract and against the host's own `grep -rn`.
#[test]
fn wfgrep_searches_a_real_tree_and_agrees_with_grep() {
    let fixture = search_tree();
    let published = assert_reference(wfgrep(), fixture.path(), "tree", b"needle");
    assert_eq!(
        sorted_lines(&published),
        grep_rn(fixture.path(), "tree", b"needle"),
        "wfgrep and grep -rn disagree about the hit set"
    );
}

/// The same cross-check with a pattern that matches nothing, so the empty
/// hit set is a real agreement rather than a vacuous one, and with a pattern
/// that matches every line, so the full hit set is too.
#[test]
fn wfgrep_agrees_with_grep_on_the_empty_and_the_total_hit_set() {
    let fixture = search_tree();
    let absent = wfgrep().run(fixture.path(), &[b"absent-pattern", b"tree"]);
    assert_eq!(absent.status.code(), Some(1));
    assert!(absent.stdout.is_empty());
    assert_eq!(
        grep_rn(fixture.path(), "tree", b"absent-pattern"),
        Vec::<String>::new()
    );

    let published = assert_reference(wfgrep(), fixture.path(), "tree", b"e");
    assert_eq!(
        sorted_lines(&published),
        grep_rn(fixture.path(), "tree", b"e"),
        "wfgrep and grep -rn disagree about the total hit set"
    );
}

/// The descent carries no depth cap of its own, and the failure it used to
/// have was the worst kind: a file below sixteen levels was left unsearched,
/// the walk returned normally, and the answer was byte-identical to a real
/// absence.
///
/// Three hundred levels is not a round number chosen for effect — it is well
/// past the deleted cap and well inside the two bounds that do still apply.
/// The host bounds one absolute path, so the fixture root's own length is part
/// of the budget; and `walk` refuses a display path past a thousand bytes.
/// That refusal is arithmetic on the *display* path, so the level it lands at
/// depends on the root name's length, which is why quoting a level without the
/// root is not reproducible: for this fixture's four-byte root `tree`,
/// `4 + 2n + len("/bottom.txt") <= 1000` gives n <= 492, and 493 is the first
/// level that fails — measured, and measured again at 493 completing with a
/// three-byte root. Neither program bound is the stack: on this test target,
/// the stack ledger prices one `wf_walk` activation at 1744 bytes and the
/// current runtime's configured stack at 615,677 of them. This comparison
/// establishes that the test reaches the program's display-buffer boundary;
/// it makes no portable guarantee about external stack availability.
#[test]
fn a_tree_far_deeper_than_the_deleted_cap_is_searched_completely() {
    let fixture = fixture_directory();
    let mut relative = String::from("tree");
    for _ in 0..300 {
        relative.push_str("/d");
    }
    relative.push_str("/bottom.txt");
    fixture.write_nested(&relative, b"needle at the bottom\n");
    fixture.write_nested("tree/top.txt", b"needle at the top\n");

    let published = assert_reference(wfgrep(), fixture.path(), "tree", b"needle");
    assert_eq!(
        sorted_lines(&published).len(),
        2,
        "the deep file and the shallow one are both hits"
    );
}

/// A match that straddles a read boundary is still one match, and the line
/// number a straddling line receives is still its ordinal in the file.
#[test]
fn a_match_across_a_read_boundary_keeps_its_line_number() {
    let fixture = fixture_directory();
    fixture.directory("tree");
    let mut content = Vec::new();
    for ordinal in 0..8 {
        content.extend_from_slice(format!("line {ordinal}\n").as_bytes());
    }
    while content.len() < BUFFER_LENGTH - 4 {
        content.extend_from_slice(b"filler line\n");
    }
    content.extend_from_slice(b"xx needle straddles here\n");
    content.extend_from_slice(b"tail\n");
    fixture.write_nested("tree/wide.txt", &content);

    let published = assert_reference(wfgrep(), fixture.path(), "tree", b"needle");
    assert_eq!(
        sorted_lines(&published),
        grep_rn(fixture.path(), "tree", b"needle")
    );
}

/// An empty tree publishes nothing and reports no match, which is the
/// no-match status rather than an error.
#[test]
fn an_empty_tree_publishes_nothing_and_reports_no_match() {
    let fixture = fixture_directory();
    fixture.directory("tree");
    let output = wfgrep().run(fixture.path(), &[b"needle", b"tree"]);
    assert!(output.stdout.is_empty());
    assert_eq!(output.status.code(), Some(1));
}

/// A subdirectory the process cannot open is an ordinary recoverable outcome:
/// the search reports it on standard error, keeps searching everything else,
/// and ends with the error status.
#[test]
fn an_unreadable_subdirectory_is_reported_and_the_rest_is_still_searched() {
    let fixture = fixture_directory();
    fixture.directory("tree");
    fixture.write_nested("tree/open.txt", b"needle visible\n");
    let closed = fixture.directory("tree/closed");
    fixture.write_nested("tree/closed/buried.txt", b"needle buried\n");
    let mut permissions = std::fs::metadata(&closed)
        .expect("fixture directory metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o000);
    std::fs::set_permissions(&closed, permissions).expect("close the fixture directory");

    let output = wfgrep().run(fixture.path(), &[b"needle", b"tree"]);

    let mut restored = std::fs::metadata(&closed)
        .expect("fixture directory metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut restored, 0o755);
    std::fs::set_permissions(&closed, restored).expect("reopen the fixture directory");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "tree/open.txt:1:needle visible\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "wfgrep: tree/closed: permission denied\n"
    );
    assert_eq!(output.status.code(), Some(2));
}

/// A file the process cannot open is reported by its complete relative path
/// and does not stop the walk.
#[test]
fn an_unreadable_file_is_reported_by_path_and_the_walk_continues() {
    let fixture = fixture_directory();
    fixture.directory("tree");
    let denied = fixture.write_nested("tree/denied.txt", b"needle denied\n");
    fixture.write_nested("tree/open.txt", b"needle visible\n");
    let mut permissions = std::fs::metadata(&denied)
        .expect("fixture file metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o000);
    std::fs::set_permissions(&denied, permissions).expect("close the fixture file");

    let output = wfgrep().run(fixture.path(), &[b"needle", b"tree"]);

    let mut restored = std::fs::metadata(&denied)
        .expect("fixture file metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut restored, 0o644);
    std::fs::set_permissions(&denied, restored).expect("reopen the fixture file");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "tree/open.txt:1:needle visible\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "wfgrep: tree/denied.txt: permission denied\n"
    );
    assert_eq!(output.status.code(), Some(2));
}

/// A search root the capability cannot open is reported once and is the
/// error status, not an empty successful search.
#[test]
fn a_missing_search_root_is_reported_once() {
    let fixture = fixture_directory();
    let output = wfgrep().run(fixture.path(), &[b"needle", b"absent"]);
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "wfgrep: absent: no such file or directory\n"
    );
    assert_eq!(output.status.code(), Some(2));
}

/// An invocation naming no root reports its usage and the error status.
#[test]
fn wfgrep_reports_its_usage_when_the_invocation_names_no_root() {
    let fixture = fixture_directory();
    let output = wfgrep().run(fixture.path(), &[b"needle"]);
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "usage: wfgrep PATTERN ROOT\n"
    );
    assert_eq!(output.status.code(), Some(2));
}

/// A pattern that is not valid text travels the lossless route unchanged and
/// matches the same bytes in the file: nothing on the route from the
/// invocation argument to the comparison passes through text.
///
/// The host is not asked to hold a file *name* that is not text — the
/// supported hosts' filesystems refuse one — so the non-text bytes travel
/// through the argument and the file content, which is where [HOST-2]'s
/// lossless route actually runs. This case has no `grep` side: the two grep
/// families disagree about a pattern that is not text.
#[test]
fn a_pattern_that_is_not_text_travels_the_lossless_route_unchanged() {
    let fixture = fixture_directory();
    fixture.directory("tree");
    fixture.write_nested(
        "tree/raw.txt",
        b"prefix \xff\xfe marker suffix\nplain line\n",
    );

    let output = wfgrep().run(fixture.path(), &[b"\xff\xfe marker", b"tree"]);
    assert_eq!(
        output.stdout,
        b"tree/raw.txt:1:prefix \xff\xfe marker suffix\n".to_vec(),
        "diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

/// A symbolic link the walk enumerates is not followed.
///
/// [SYS-14] reports it as kind `3 symbolic link`, and the program acts on
/// exactly the kinds it was told about — a regular file it opens, a directory
/// it descends, and everything else it leaves alone. That is a property of
/// this program, not of the capability: [PATH-2]'s resolution is still
/// process-equivalent and would follow a link a program actually named. This
/// case has no `grep` side, because the two grep families disagree about
/// links found during a traversal.
#[test]
fn an_enumerated_symbolic_link_is_not_followed() {
    let fixture = fixture_directory();
    fixture.directory("tree");
    fixture.write_nested("tree/visible.txt", b"needle inside\n");
    let outside = fixture.write(b"outside.txt", b"needle outside\n");
    let outside_directory = fixture.directory("elsewhere");
    fixture.write_nested("elsewhere/buried.txt", b"needle elsewhere\n");
    fixture.symlink("tree/link.txt", &outside);
    fixture.symlink("tree/linkdir", &outside_directory);

    let output = wfgrep().run(fixture.path(), &[b"needle", b"tree"]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "tree/visible.txt:1:needle inside\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

/// Admission is decided by the inventory the specification declares, never by
/// the compiler recognizing a source shape: the identical search source
/// compiles against the complete active inventory and is an undeclared name
/// against the pre-permit inventory.
#[test]
fn the_search_source_requires_the_complete_file_permit_inventory() {
    let llvm = compile_program("wfgrep.wf");
    // The approved implementations, by symbol rather than by any source name
    // [QUAL-1].
    assert!(llvm.contains("@wf.sys.open_file.v1"));
    assert!(llvm.contains("@wf.sys.open_directory_source.v1"));
    assert!(llvm.contains("@wf.sys.directory_next.v1"));
    assert!(llvm.contains("@wf.sys.open_directory.v1"));
    assert!(llvm.contains("@wf.sys.read_at.v1"));

    let failure = compile_program_rejection_with("wfgrep.wf", Inventory::OpenByName);
    assert!(
        failure.contains("UnresolvedUse")
            && (failure.contains("HandleFactory") || failure.contains("reserve_handle")),
        "the pre-permit inventory must reject explicit file authority: {failure}"
    );
}

// The old traversal/open-by-name byte differential ended when the file-permit
// amendment changed open signatures and added nominal types. It is not muted:
// its premise no longer exists. The active-inventory program tests and catalog
// count/ordinal tests now cover the two separate obligations.
