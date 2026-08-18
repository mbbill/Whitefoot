//! End-to-end evidence for `tests/programs/wfgrep.wf`, the first real
//! Whitefoot command program.
//!
//! The oracle is a trusted reference search written here rather than a host
//! `grep`: the two grep families on the supported hosts disagree about
//! unterminated final lines and about patterns that are not valid text, and
//! wfgrep's published bytes must be compared against one fixed contract, not
//! against whichever tool the host installs.
//!
//! Three cases exercise real operating-system mechanisms instead of static
//! fixture content: a destination with no reader, a symbolic link leaving the
//! directory the capability names, and a file whose content changes between
//! two invocations.

use std::os::unix::process::ExitStatusExt;

use super::support::{
    CompiledProgram, build_program, compile_and_run, compile_program, compile_sources,
    fixture_directory,
};

/// The reusable input and output buffer length in `tests/programs/wfgrep.wf`.
///
/// The corpus needs it to build a file that is exactly one buffer long and a
/// match that straddles a read boundary; nothing in the program's contract
/// exposes it.
const BUFFER_LENGTH: usize = 4096;

fn wfgrep() -> &'static CompiledProgram {
    // One compilation shared by every scenario test in this module: the ten
    // tests exercise ten different behaviors of the same immutable artifact,
    // and rebuilding it per test was measured at 136s each against
    // milliseconds of scenario work. Isolation lives in each run's own
    // fixture directory, never in the binary. The static's directory is not
    // dropped at process exit; it is pid-unique under the system temp dir.
    static PROGRAM: std::sync::OnceLock<CompiledProgram> = std::sync::OnceLock::new();
    PROGRAM.get_or_init(|| build_program(&compile_program("wfgrep.wf")))
}

/// Publishes every line of every input containing the literal pattern bytes.
///
/// This is the frozen contract wfgrep implements: a line is a maximal run of
/// bytes between newlines, an unterminated final run is a line, every
/// published line is terminated, the pattern is matched against the line
/// without its terminator, and the empty pattern matches every line.
fn reference(pattern: &[u8], contents: &[&[u8]]) -> (Vec<u8>, i32) {
    let mut published = Vec::new();
    let mut matched = false;
    for content in contents {
        for line in lines(content) {
            if occurs(line, pattern) {
                published.extend_from_slice(line);
                published.push(b'\n');
                matched = true;
            }
        }
    }
    (published, i32::from(!matched))
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
    if pattern.is_empty() {
        return true;
    }
    line.len() >= pattern.len() && line.windows(pattern.len()).any(|window| window == pattern)
}

/// Runs wfgrep over one fixture set and compares it with the reference.
fn assert_reference(program: &CompiledProgram, pattern: &[u8], files: &[(&[u8], Vec<u8>)]) {
    let directory = fixture_directory();
    for (name, content) in files {
        directory.write(name, content);
    }
    let mut arguments: Vec<&[u8]> = vec![pattern];
    arguments.extend(files.iter().map(|(name, _)| *name));
    let output = program.run(directory.path(), &arguments);
    let contents: Vec<&[u8]> = files.iter().map(|(_, bytes)| bytes.as_slice()).collect();
    let (expected, code) = reference(pattern, &contents);
    assert_eq!(
        output.stderr,
        Vec::<u8>::new(),
        "a successful search reports nothing"
    );
    assert_eq!(
        output.stdout,
        expected,
        "published bytes differ from the reference for pattern {:?}",
        String::from_utf8_lossy(pattern)
    );
    assert_eq!(output.status.code(), Some(code));
}

/// Builds `count` lines of `width` content bytes each, plus terminators.
fn numbered_lines(count: usize, width: usize) -> Vec<u8> {
    let mut content = Vec::new();
    for line in 0..count {
        let label = format!("{line:0width$}", width = width);
        content.extend_from_slice(&label.as_bytes()[label.len() - width..]);
        content.push(b'\n');
    }
    content
}

#[test]
fn wfgrep_matches_the_reference_across_every_required_file_shape() {
    let program = wfgrep();

    // Empty, short, and unterminated inputs.
    assert_reference(program, b"alpha", &[(b"empty.txt", Vec::new())]);
    assert_reference(
        program,
        b"alpha",
        &[(b"short.txt", b"alpha\nbeta\ngamma\nalphabet\n".to_vec())],
    );
    assert_reference(
        program,
        b"second",
        &[(b"tail.txt", b"first\nsecond without a terminator".to_vec())],
    );
    // A file that is nothing but terminators still has lines.
    assert_reference(program, b"", &[(b"blank.txt", b"\n\n\n".to_vec())]);
    // No line contains the pattern: exit 1, nothing published.
    assert_reference(
        program,
        b"absent",
        &[(b"short.txt", b"alpha\nbeta\ngamma\n".to_vec())],
    );

    // Exactly one buffer, ending on a line boundary and ending inside a line.
    let terminated = numbered_lines(BUFFER_LENGTH / 64, 63);
    assert_eq!(terminated.len(), BUFFER_LENGTH);
    assert_reference(program, b"01", &[(b"exact.txt", terminated)]);
    let mut unterminated = numbered_lines(BUFFER_LENGTH / 64 - 1, 63);
    unterminated.extend(std::iter::repeat_n(b'z', 64));
    assert_eq!(unterminated.len(), BUFFER_LENGTH);
    assert_reference(program, b"zzz", &[(b"exact-open.txt", unterminated)]);

    // Several buffers.
    let multichunk = numbered_lines(700, 40);
    assert!(multichunk.len() > 5 * BUFFER_LENGTH);
    assert_reference(program, b"0069", &[(b"multichunk.txt", multichunk)]);

    // A match straddling the first read boundary. The first line fills the
    // buffer to just under its length, so the second line is carried across
    // the boundary and the pattern spans both attempts.
    let mut straddle = vec![b'a'; BUFFER_LENGTH - 96];
    straddle.push(b'\n');
    straddle.extend(std::iter::repeat_n(b'b', 91));
    let boundary = straddle.len();
    straddle.extend_from_slice(b"STRADDLE");
    assert!(boundary < BUFFER_LENGTH && boundary + 8 > BUFFER_LENGTH);
    straddle.extend(std::iter::repeat_n(b'c', 40));
    straddle.push(b'\n');
    assert_reference(program, b"STRADDLE", &[(b"straddle.txt", straddle)]);

    // Bytes and patterns that are not valid text travel the lossless route.
    let binary = b"\xff\xfe head\nplain\n\x00\x01\xc3\x28 tail\n".to_vec();
    assert_reference(program, b"\xc3\x28", &[(b"binary.txt", binary.clone())]);
    assert_reference(program, b"\xff\xfe", &[(b"binary.txt", binary)]);

    // Several files publish in argument order and share one output batch.
    assert_reference(
        program,
        b"e",
        &[
            (b"one.txt", b"alpha\nbeta\n".to_vec()),
            (b"two.txt", Vec::new()),
            (b"three.txt", b"delta\nepsilon".to_vec()),
        ],
    );
    // One long run of matches forces several batch flushes.
    assert_reference(program, b"7", &[(b"flushes.txt", numbered_lines(2000, 20))]);
}

#[test]
fn wfgrep_reports_one_unreadable_file_and_still_searches_the_others() {
    let program = wfgrep();
    let directory = fixture_directory();
    directory.write(b"present.txt", b"alpha\nbeta\n");
    let output = program.run(directory.path(), &[b"alpha", b"absent.txt", b"present.txt"]);
    assert_eq!(output.stdout, b"alpha\n");
    assert_eq!(
        output.stderr,
        b"wfgrep: absent.txt: no such file or directory\n"
    );
    // An error outranks a match, exactly as the frozen mapping requires.
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_path_that_is_not_text_travels_the_lossless_route_unchanged() {
    let program = wfgrep();
    let directory = fixture_directory();
    // The two supported host families disagree about whether a file name may
    // be arbitrary bytes — APFS refuses one that is not valid UTF-8 — so the
    // portable witness names a file that does not exist and checks that the
    // exact argument bytes survive `relative_path`, `open_read`, and the
    // diagnostic's `host_copy_bytes` with no normalization or substitution.
    let name: &[u8] = b"na\xffme\xc3\x28.txt";
    let output = program.run(directory.path(), &[b"alpha", name]);
    let mut expected = b"wfgrep: ".to_vec();
    expected.extend_from_slice(name);
    expected.extend_from_slice(b": no such file or directory\n");
    assert_eq!(output.stderr, expected);
    assert_eq!(output.stdout, b"");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn wfgrep_reports_a_path_the_directory_capability_does_not_admit() {
    let program = wfgrep();
    let directory = fixture_directory();
    directory.write(b"present.txt", b"alpha\n");
    // A target-root prefix is not constructible as a relative path, so the
    // failure is the path constructor's, not the open's.
    let output = program.run(directory.path(), &[b"alpha", b"/etc/hosts"]);
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"wfgrep: /etc/hosts: invalid relative path\n"
    );
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn wfgrep_reports_a_line_longer_than_its_reusable_buffer() {
    let program = wfgrep();
    let directory = fixture_directory();
    // wfgrep never materializes a whole file, so its one reusable input
    // buffer also fixes the longest line it can assemble. Exceeding it is a
    // reported error, never a truncated or silently split line.
    let mut long = vec![b'x'; BUFFER_LENGTH + 1];
    long.push(b'\n');
    directory.write(b"long.txt", &long);
    let output = program.run(directory.path(), &[b"xxx", b"long.txt"]);
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"wfgrep: long.txt: line too long\n");
    assert_eq!(output.status.code(), Some(2));

    // One byte shorter is an ordinary line.
    let mut fitting = vec![b'x'; BUFFER_LENGTH - 1];
    fitting.push(b'\n');
    assert_reference(program, b"xxx", &[(b"fitting.txt", fitting)]);
}

#[test]
fn a_closed_destination_reaches_source_as_broken_pipe_not_a_signal() {
    let program = wfgrep();
    let directory = fixture_directory();
    // More matching output than any pipe buffer holds, so at least one
    // `write_once` must reach the closed destination.
    directory.write(b"wide.txt", &numbered_lines(20000, 24));
    let (status, diagnostics) =
        program.run_with_closed_output(directory.path(), &[b"1", b"wide.txt"]);
    // The bootstrap's one-time SIGPIPE normalization is what makes this a
    // value rather than a death: without it the process would carry signal 13
    // and no exit code at all.
    assert_eq!(
        status.signal(),
        None,
        "a write to a closed destination must not kill the process"
    );
    assert_eq!(status.code(), Some(2));
    // The class, not merely the failure, reaches source.
    assert_eq!(diagnostics, b"wfgrep: broken pipe\n");
}

#[test]
fn open_read_follows_a_symbolic_link_out_of_the_directory_it_names() {
    let program = wfgrep();
    let outside = fixture_directory();
    let linked = outside.write(b"linked.txt", b"alpha outside\nbeta outside\n");
    let inside = fixture_directory();
    inside.write(b"inside.txt", b"alpha inside\n");
    inside.symlink("escape.txt", &linked);

    // `command.cwd` is process-equivalent, not confined: resolution follows
    // the link exactly as the surrounding process namespace does, so the
    // resolved object legitimately lies outside the directory the capability
    // names.
    let output = program.run(inside.path(), &[b"alpha", b"escape.txt"]);
    assert_eq!(output.stdout, b"alpha outside\n");
    assert_eq!(output.stderr, b"");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn each_invocation_observes_only_what_the_file_held_for_that_attempt() {
    let program = wfgrep();
    let directory = fixture_directory();
    let first = b"alpha one\nbeta one\n";
    directory.write(b"changing.txt", first);
    let before = program.run(directory.path(), &[b"alpha", b"changing.txt"]);
    assert_eq!(before.stdout, b"alpha one\n");
    assert_eq!(before.status.code(), Some(0));

    // A longer replacement, so the second run cannot reproduce the first by
    // reading a stale length.
    let second = b"gamma two\nalpha two\ndelta two\nalpha again\n";
    directory.write(b"changing.txt", second);
    let after = program.run(directory.path(), &[b"alpha", b"changing.txt"]);
    let (expected, code) = reference(b"alpha", &[second]);
    assert_eq!(after.stdout, expected);
    assert_eq!(after.status.code(), Some(code));

    // A shorter replacement, so no attempt may assume a monotonic size.
    let third = b"alpha\n";
    directory.write(b"changing.txt", third);
    let shrunk = program.run(directory.path(), &[b"alpha", b"changing.txt"]);
    assert_eq!(shrunk.stdout, b"alpha\n");
    assert_eq!(shrunk.status.code(), Some(0));
}

#[test]
fn wfgrep_reports_its_usage_when_the_invocation_names_no_file() {
    let program = wfgrep();
    let directory = fixture_directory();
    let output = program.run(directory.path(), &[b"alpha"]);
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"usage: wfgrep PATTERN FILE...\n");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn wfgrep_append_preserves_its_clause_stripped_invalid_domain_behavior() {
    let source = include_str!("../../../tests/programs/wfgrep.wf");
    let start = source.find("fn append_slice").expect("append_slice start");
    let rest = &source[start..];
    let end = rest
        .find("\nfn copy_range")
        .expect("append_slice declaration end");
    let declaration = &rest[..end];
    let clauses = declaration
        .find(" requires {")
        .expect("append_slice requires clause");
    let body = declaration
        .find("} {\n  doc")
        .expect("append_slice body after ensures");
    let declaration = format!("{} {}", &declaration[..clauses], &declaration[body + 2..])
        .trim_end()
        .to_owned();
    let control = format!(
        r#"{declaration}

fn main() -> own unit allocates(heap), traps {{
  let empty_text = buffer_new(0_u64, 1_u8);
  let empty_destination = buffer_new(3_u64, 9_u8);
  region 'empty_text_view {{
    let text = slice_of(&'empty_text_view empty_text);
    region 'empty_destination_view {{
      let result = append_slice<'empty_destination_view, 'empty_text_view>(destination: &uniq 'empty_destination_view empty_destination, filled: 4_u64, text: move text);
      claim empty_result: ieq(result, 4_u64) because "empty result";
    }}
  }}
  claim empty_byte_zero: ieq(empty_destination[0_u64], 9_u8) because "empty byte zero";
  claim empty_byte_one: ieq(empty_destination[1_u64], 9_u8) because "empty byte one";
  claim empty_byte_two: ieq(empty_destination[2_u64], 9_u8) because "empty byte two";
  let nonempty_text = buffer_new(2_u64, 1_u8);
  let nonempty_destination = buffer_new(3_u64, 9_u8);
  region 'nonempty_text_view {{
    let text = slice_of(&'nonempty_text_view nonempty_text);
    region 'nonempty_destination_view {{
      let result = append_slice<'nonempty_destination_view, 'nonempty_text_view>(destination: &uniq 'nonempty_destination_view nonempty_destination, filled: 4_u64, text: move text);
      claim nonempty_result: ieq(result, 4_u64) because "nonempty result";
    }}
  }}
  claim nonempty_byte_zero: ieq(nonempty_destination[0_u64], 9_u8) because "nonempty byte zero";
  claim nonempty_byte_one: ieq(nonempty_destination[1_u64], 9_u8) because "nonempty byte one";
  claim nonempty_byte_two: ieq(nonempty_destination[2_u64], 9_u8) because "nonempty byte two";
  return unit;
}}
"#
    );
    let llvm = compile_sources(&[("wfgrep_append_invalid_domain.wf", control.as_bytes())]);
    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
