//! The deterministic test target: the second column of the [QUAL-1]
//! qualification table and the scripted host its rows name.
//!
//! Some conditions the first slice's contracts fix cannot be produced on
//! demand through a real file or pipe: a close attempt that fails, a read that
//! stops short at a chosen call, a write the host accepts only in part. This
//! module qualifies the same program against a target whose file and
//! descriptor facilities are answered by one small scripted translation unit
//! linked into the compiled test artifact, so the forced condition is observed
//! through the same emitted lowering rather than through a model of it.
//!
//! It supplies exactly the arrangements those contract tests consume and
//! nothing else: it is not a simulator of the host and not an artifact-replay
//! framework. Adding an arrangement here is adding one scripted outcome to one
//! facility, and a facility exists here only once an operation's contract test
//! needs it.

use std::fmt::Write as _;
use std::process::Output;

use crate::backend::emitter::emit_llvm_for_target;
use crate::backend::qualification::SystemTarget;

use super::system::with_ir;
// The same contract programs task 0012 exercises against real files and
// pipes, re-run here under a forced condition no real object can produce on
// demand. Sharing the source is the point: the two targets must make it
// behave the same way for the same host answer.
use super::system_io::{CHUNKED_READ, WRITE_PREFIX, class_arms};
use super::{compile_link_and_run, host_optimized_module};

/// A host error the deterministic host can be scripted to report.
///
/// Each names the C macro the generated unit reports, so the value is the
/// selected host's own, never a number this module invents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HostError {
    /// An interrupted call. A close that reports it leaves the descriptor's
    /// state unknowable, which is exactly why [SYS-5] never retries after one.
    Interrupted,
    /// A readiness refusal. The deterministic adapter consumes the next
    /// scripted answer only after recording the exact readiness wait.
    WouldBlock,
    /// A device or input/output failure: the mid-stream condition a real file
    /// cannot be made to produce at a chosen call.
    DeviceFailure,
}

impl HostError {
    const fn macro_name(self) -> &'static str {
        match self {
            Self::Interrupted => "EINTR",
            Self::WouldBlock => "EAGAIN",
            Self::DeviceFailure => "EIO",
        }
    }
}

/// One scripted outcome of one host call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HostOutcome {
    /// The call succeeds naturally: a read delivers what remains of the
    /// fixture up to the requested capacity, a write accepts the whole
    /// request, a close succeeds.
    Succeed,
    /// A transfer succeeds but the host moves at most this many bytes. On a
    /// close this is simply success — a close transfers nothing.
    Accept(u64),
    /// The call fails and reports this error.
    Fail(HostError),
}

impl HostOutcome {
    /// The script entry this outcome renders as.
    ///
    /// A non-negative entry is a cap on the bytes the call may transfer, so
    /// `Succeed` is the largest representable cap — no cap at all — and a
    /// negative entry is the negated error the call reports.
    fn entry(self) -> String {
        match self {
            Self::Succeed => "LONG_MAX".to_owned(),
            Self::Accept(count) => count.to_string(),
            Self::Fail(error) => format!("-{}", error.macro_name()),
        }
    }
}

/// The one descriptor-status answer the deterministic file reports.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum HostFileStatus {
    /// The ordinary control: the provisional descriptor names a regular file.
    #[default]
    Regular,
    /// The open succeeded, but descriptor inspection identifies a directory.
    Directory,
    /// The descriptor-status facility itself fails with this target error.
    Fail(HostError),
}

/// The scripted state one deterministic-host run answers from.
///
/// A script lists the outcome of each call to one facility in call order; a
/// call past the end of a list succeeds naturally. With one fixture file that
/// is the whole configuration surface — no recorded session, no replay, and
/// no per-test host code.
#[derive(Clone, Debug, Default)]
pub(super) struct HostScript {
    file: Option<Vec<u8>>,
    file_status: HostFileStatus,
    reads: Vec<HostOutcome>,
    writes: Vec<HostOutcome>,
    closes: Vec<HostOutcome>,
}

impl HostScript {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Supplies the one fixture file a directory-relative open produces.
    ///
    /// Without it the open reports `ENOENT`. One file is what the first
    /// slice's contract tests need; a second would be a second arrangement,
    /// not a general filesystem.
    pub(super) fn file(mut self, bytes: &[u8]) -> Self {
        self.file = Some(bytes.to_vec());
        self
    }

    /// Selects the descriptor-status answer for the one fixture file.
    fn file_status(mut self, status: HostFileStatus) -> Self {
        self.file_status = status;
        self
    }

    /// Scripts the outcome of each read attempt, in call order.
    pub(super) fn reads(mut self, outcomes: &[HostOutcome]) -> Self {
        self.reads = outcomes.to_vec();
        self
    }

    /// Scripts the outcome of each write attempt, in call order.
    pub(super) fn writes(mut self, outcomes: &[HostOutcome]) -> Self {
        self.writes = outcomes.to_vec();
        self
    }

    /// Scripts the outcome of each close attempt, in call order.
    pub(super) fn closes(mut self, outcomes: &[HostOutcome]) -> Self {
        self.closes = outcomes.to_vec();
        self
    }

    /// Renders the host translation unit this script configures.
    fn unit(&self) -> String {
        let mut source = String::from(HOST_PRELUDE);
        source.push_str(&fixture_table(self.file.as_deref()));
        source.push_str(&file_status_table(self.file_status));
        source.push_str(&outcome_table("pread", &self.reads));
        source.push_str(&outcome_table("write", &self.writes));
        source.push_str(&outcome_table("close", &self.closes));
        source.push_str(HOST_FACILITIES);
        source
    }
}

/// Renders the one descriptor-status answer used by `wf_test_fstat`.
fn file_status_table(status: HostFileStatus) -> String {
    let (error, mode) = match status {
        HostFileStatus::Regular => ("0", "S_IFREG | S_IRUSR"),
        HostFileStatus::Directory => ("0", "S_IFDIR | S_IRUSR"),
        HostFileStatus::Fail(error) => (error.macro_name(), "0"),
    };
    format!(
        "static const int wf_test_file_status_error = {error};\n\
         static const unsigned int wf_test_file_status_mode = {mode};\n"
    )
}

/// Renders the one fixture file a directory-relative open produces.
fn fixture_table(file: Option<&[u8]>) -> String {
    let present = usize::from(file.is_some());
    let bytes = file.unwrap_or_default();
    let length = bytes.len();
    let mut entries = bytes
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<_>>();
    // A zero-length array is not valid C, so an absent or empty fixture keeps
    // one unused slot and a length of zero.
    if entries.is_empty() {
        entries.push("0".to_owned());
    }
    format!(
        "static const int wf_test_file_present = {present};\n\
         static const unsigned char wf_test_file_bytes[] = {{ {} }};\n\
         static const unsigned long wf_test_file_length = {length};\n",
        entries.join(", ")
    )
}

/// Renders one facility's scripted outcome table.
fn outcome_table(facility: &str, outcomes: &[HostOutcome]) -> String {
    let mut entries = outcomes
        .iter()
        .map(|outcome| outcome.entry())
        .collect::<Vec<_>>();
    let length = entries.len();
    if entries.is_empty() {
        entries.push("LONG_MAX".to_owned());
    }
    let mut rendered = String::new();
    writeln!(
        rendered,
        "static const long wf_test_{facility}_script[] = {{ {} }};\n\
         static const unsigned long wf_test_{facility}_scripted = {length};\n\
         static unsigned long wf_test_{facility}_calls = 0;",
        entries.join(", ")
    )
    .expect("string writes do not fail");
    rendered
}

/// The fixed head of the generated host unit.
const HOST_PRELUDE: &str = "\
/* The deterministic test host for one Whitefoot backend test. Generated by\n\
   compiler/src/backend/tests/deterministic_target.rs; never checked in. */\n\
#include <errno.h>\n\
#include <fcntl.h>\n\
#include <limits.h>\n\
#include <stdint.h>\n\
#include <stdio.h>\n\
#include <string.h>\n\
#include <sys/stat.h>\n\
#include <unistd.h>\n\
\n\
/* The descriptors the scripted opens produce. They are deliberately not real\n\
   open descriptors: nothing in the program may treat them as one, and the\n\
   two differ so a close attempt is attributable to its own resource. */\n\
#define WF_TEST_DIRECTORY 41\n\
#define WF_TEST_FILE 42\n\
\n\
";

/// The fixed body of the generated host unit: one function per facility a
/// [QUAL-1] deterministic-target row names.
const HOST_FACILITIES: &str = "\
\n\
/* Every facility appends one line to the real standard error so a test can\n\
   observe host-visible facts the source cannot see — how many attempts a\n\
   release made, and against which descriptor. */\n\
static void wf_test_trace(const char *line) {\n\
    unsigned long length = 0;\n\
    while (line[length] != '\\0') {\n\
        length++;\n\
    }\n\
    ssize_t written = write(2, line, length);\n\
    (void)written;\n\
}\n\
\n\
/* One call's scripted entry. A call past the end of its script succeeds\n\
   naturally, which for a transfer is the largest possible cap. */\n\
static long wf_test_step(const long *script, unsigned long scripted,\n\
                         unsigned long *calls) {\n\
    unsigned long call = *calls;\n\
    *calls = call + 1;\n\
    return call < scripted ? script[call] : LONG_MAX;\n\
}\n\
\n\
/* Applies a non-negative script entry as a cap on a transfer length. */\n\
static unsigned long wf_test_capped(long entry, unsigned long length) {\n\
    unsigned long cap = (unsigned long)entry;\n\
    return cap < length ? cap : length;\n\
}\n\
\n\
int wf_test_open(const char *path, int flags, ...) {\n\
    char line[128];\n\
    (void)path;\n\
    (void)flags;\n\
    snprintf(line, sizeof line, \"wf_test open fd=%d\\n\", WF_TEST_DIRECTORY);\n\
    wf_test_trace(line);\n\
    return WF_TEST_DIRECTORY;\n\
}\n\
\n\
int wf_test_openat(int directory, const char *path, int flags, ...) {\n\
    char line[128];\n\
    (void)path;\n\
    /* The one fixture is a regular file, so a directory-only open of it\n\
       reports ENOTDIR exactly as a real host would. Without this the scripted\n\
       host would answer a directory open with a regular file, which no real\n\
       target does and which no test should be allowed to assume. */\n\
    if (wf_test_file_present && (flags & O_DIRECTORY) != 0) {\n\
        snprintf(line, sizeof line, \"wf_test openat root=%d -> notdir\\n\",\n\
                 directory);\n\
        wf_test_trace(line);\n\
        errno = ENOTDIR;\n\
        return -1;\n\
    }\n\
    if (!wf_test_file_present) {\n\
        snprintf(line, sizeof line, \"wf_test openat root=%d -> absent\\n\",\n\
                 directory);\n\
        wf_test_trace(line);\n\
        errno = ENOENT;\n\
        return -1;\n\
    }\n\
    snprintf(line, sizeof line, \"wf_test openat root=%d fd=%d\\n\", directory,\n\
             WF_TEST_FILE);\n\
    wf_test_trace(line);\n\
    return WF_TEST_FILE;\n\
}\n\
\n\
int wf_test_fstat(int descriptor, struct stat *status) {\n\
    char line[128];\n\
    if (descriptor != WF_TEST_FILE) {\n\
        errno = EBADF;\n\
        return -1;\n\
    }\n\
    if (wf_test_file_status_error != 0) {\n\
        snprintf(line, sizeof line,\n\
                 \"wf_test fstat fd=%d outcome=error code=%d\\n\", descriptor,\n\
                 wf_test_file_status_error);\n\
        wf_test_trace(line);\n\
        errno = wf_test_file_status_error;\n\
        return -1;\n\
    }\n\
    memset(status, 0, sizeof *status);\n\
    status->st_mode = wf_test_file_status_mode;\n\
    snprintf(line, sizeof line, \"wf_test fstat fd=%d outcome=ok\\n\", descriptor);\n\
    wf_test_trace(line);\n\
    return 0;\n\
}\n\
\n\
ssize_t wf_test_pread(int descriptor, void *destination, size_t capacity,\n\
                      int64_t offset) {\n\
    char line[192];\n\
    long entry;\n\
    for (;;) {\n\
        entry = wf_test_step(wf_test_pread_script, wf_test_pread_scripted,\n\
                             &wf_test_pread_calls);\n\
        if (entry == -EINTR) {\n\
            snprintf(line, sizeof line,\n\
                     \"wf_test pread fd=%d progress=eintr\\n\", descriptor);\n\
            wf_test_trace(line);\n\
            continue;\n\
        }\n\
        if (entry == -EAGAIN) {\n\
            snprintf(line, sizeof line,\n\
                     \"wf_test pread fd=%d wait=readable\\n\", descriptor);\n\
            wf_test_trace(line);\n\
            continue;\n\
        }\n\
        break;\n\
    }\n\
    if (entry < 0) {\n\
        snprintf(line, sizeof line,\n\
                 \"wf_test pread fd=%d offset=%lld capacity=%lu -> error\\n\",\n\
                 descriptor, (long long)offset, (unsigned long)capacity);\n\
        wf_test_trace(line);\n\
        errno = (int)(-entry);\n\
        return -1;\n\
    }\n\
    unsigned long position = (unsigned long)offset;\n\
    unsigned long remaining = position < wf_test_file_length\n\
        ? wf_test_file_length - position : 0;\n\
    unsigned long delivered = capacity < remaining ? capacity : remaining;\n\
    delivered = wf_test_capped(entry, delivered);\n\
    memcpy(destination, wf_test_file_bytes + position, delivered);\n\
    snprintf(line, sizeof line,\n\
             \"wf_test pread fd=%d offset=%lld capacity=%lu delivered=%lu\\n\",\n\
             descriptor, (long long)offset, (unsigned long)capacity, delivered);\n\
    wf_test_trace(line);\n\
    return (ssize_t)delivered;\n\
}\n\
\n\
ssize_t wf_test_write(int descriptor, const void *source, size_t count) {\n\
    char line[192];\n\
    long entry;\n\
    for (;;) {\n\
        entry = wf_test_step(wf_test_write_script, wf_test_write_scripted,\n\
                             &wf_test_write_calls);\n\
        if (entry == -EINTR) {\n\
            snprintf(line, sizeof line,\n\
                     \"wf_test write fd=%d progress=eintr\\n\", descriptor);\n\
            wf_test_trace(line);\n\
            continue;\n\
        }\n\
        if (entry == -EAGAIN) {\n\
            snprintf(line, sizeof line,\n\
                     \"wf_test write fd=%d wait=writable\\n\", descriptor);\n\
            wf_test_trace(line);\n\
            continue;\n\
        }\n\
        break;\n\
    }\n\
    if (entry < 0) {\n\
        snprintf(line, sizeof line, \"wf_test write fd=%d count=%lu -> error\\n\",\n\
                 descriptor, (unsigned long)count);\n\
        wf_test_trace(line);\n\
        errno = (int)(-entry);\n\
        return -1;\n\
    }\n\
    unsigned long accepted = wf_test_capped(entry, (unsigned long)count);\n\
    /* The accepted bytes are echoed so a test can see exactly what the sink\n\
       received, which is what a real destination would show. */\n\
    snprintf(line, sizeof line, \"wf_test write fd=%d count=%lu accepted=%lu \"\n\
             \"bytes=%.*s\\n\", descriptor, (unsigned long)count, accepted,\n\
             (int)accepted, (const char *)source);\n\
    wf_test_trace(line);\n\
    return (ssize_t)accepted;\n\
}\n\
\n\
int wf_test_close(int descriptor) {\n\
    char line[128];\n\
    long outcome = wf_test_step(wf_test_close_script, wf_test_close_scripted,\n\
                                &wf_test_close_calls);\n\
    snprintf(line, sizeof line, \"wf_test close fd=%d outcome=%s\\n\", descriptor,\n\
             outcome < 0 ? \"error\" : \"ok\");\n\
    wf_test_trace(line);\n\
    if (outcome < 0) {\n\
        errno = (int)(-outcome);\n\
        return -1;\n\
    }\n\
    return 0;\n\
}\n";

/// One run of one program against the deterministic test target.
pub(super) struct DeterministicRun {
    /// The compiled program's own result.
    pub(super) output: Output,
}

impl DeterministicRun {
    /// The host trace the scripted facilities produced.
    pub(super) fn trace(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).into_owned()
    }

    /// How many times one facility was reached.
    pub(super) fn attempts(&self, facility: &str) -> usize {
        let prefix = format!("wf_test {facility} ");
        self.trace()
            .lines()
            .filter(|line| line.starts_with(&prefix))
            .count()
    }
}

/// Emits one source against the deterministic test target.
pub(super) fn emit_for_deterministic_target(source: &[u8]) -> String {
    with_ir(source, |program| {
        emit_llvm_for_target(program, SystemTarget::deterministic_test())
            .expect("the deterministic test target admits the program")
            .into_string()
    })
}

/// Compiles one source against the deterministic test target, links it with
/// the unit the script configures, and runs it.
pub(super) fn run_on_deterministic_host(
    source: &[u8],
    script: &HostScript,
    arguments: &[&[u8]],
) -> DeterministicRun {
    let llvm = emit_for_deterministic_target(source);
    run_emitted_on_deterministic_host(&llvm, script, arguments)
}

/// Runs an already-emitted deterministic-target module, so a caller holding a
/// shared emission does not pay a second front-end pass for the same source.
pub(super) fn run_emitted_on_deterministic_host(
    llvm: &str,
    script: &HostScript,
    arguments: &[&[u8]],
) -> DeterministicRun {
    let output = compile_link_and_run(llvm, Some(&script.unit()), arguments);
    DeterministicRun { output }
}

/// A command that binds the initial working directory and returns a fixed
/// status, so its only host activity is the [SYS-5] release of one
/// `DirectoryRead`.
const RELEASES_ONE_DIRECTORY: &[u8] =
    br#"command fn main(command.cwd as cwd: own DirectoryRead) -> status: own ExitStatus writes(cwd) {
  return exit_status(code: 0_u8);
}
"#;

/// A command that reads its own invocation vector and reaches no host object
/// at all, so every row it uses is one both target columns share.
const READS_ITS_ARGUMENTS: &[u8] =
    br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region 'a {
    let total = args_count<'a>(args: &'a args);
    let narrowed = cvt<u64, u8>(total);
    match narrowed {
      Ok(value: code) => {
        return exit_status(code: code);
      }
      Err(error: overflowed) => {
        return exit_status(code: 200_u8);
      }
    }
  }
}
"#;

/// Publishes three bytes to standard output and returns the accepted count,
/// while also binding the initial working directory so exactly one resource
/// in the program releases with a close.
const WRITES_THEN_RELEASES_BOTH: &[u8] =
    br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(cwd, out), allocates(heap) {
  let bytes = buffer_new(3_u64, 65_u8);
  set bytes[1_u64] = 66_u8;
  set bytes[2_u64] = 67_u8;
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 3_u64) {
        Ok(value: written) => {
          let narrowed = cvt<u64, u8>(written);
          match narrowed {
            Ok(value: code) => {
              return exit_status(code: code);
            }
            Err(error: overflowed) => {
              return exit_status(code: 200_u8);
            }
          }
        }
        Err(error: problem) => {
          return exit_status(code: 211_u8);
        }
      }
    }
  }
}
"#;

/// Opens the deterministic fixture through `open_file` and makes a selected
/// error class and its detail visible as the command status. Descriptor
/// inspection and provisional cleanup are therefore on the same emitted path
/// under test.
fn opens_one_file(named: &[(&str, &str)], default: &str) -> String {
    let arms = class_arms(12, named, default);
    format!(
        r#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {{
  let name = buffer_new(1_u64, 65_u8);
  region 'c {{
    region 'n {{
      let permit = reserve_file<'c>(factory: &uniq 'c files);
      match open_file<'c, 'n>(permit: move permit, root: &'c cwd, name: &'n name, start: 0_u64, end: 1_u64) {{
        Ok(value: file) => {{
          return exit_status(code: 24_u8);
        }}
        Err(error: problem) => {{
          match move problem {{
{arms}          }}
        }}
      }}
    }}
  }}
}}
"#
    )
}

#[test]
fn a_program_reaching_no_host_object_emits_identically_on_both_targets() {
    // The target column changes only the facilities that reach a real
    // operating-system object. Argument access, the host-string routes, path
    // construction, and `exit_status` resolve to the same approved row on both
    // columns, so this program's module is the same byte for byte.
    assert_eq!(
        super::compile(READS_ITS_ARGUMENTS),
        emit_for_deterministic_target(READS_ITS_ARGUMENTS)
    );

    // And it observes the same invocation vector: the deterministic target
    // scripts host objects, never the arguments the harness already controls
    // exactly.
    let run = run_on_deterministic_host(READS_ITS_ARGUMENTS, &HostScript::new(), &[b"a", b"b"]);
    assert_eq!(run.output.status.code(), Some(3));
    assert_eq!(run.attempts("close"), 0);
    assert_eq!(run.attempts("open"), 0);
}

#[test]
fn the_deterministic_target_qualifies_the_same_program_as_the_native_target() {
    // The second column is a different implementation of the same
    // specification, not a relaxed one: it qualifies exactly the facilities
    // the native target qualifies, with the same [QUAL-2] guarantees.
    with_ir(RELEASES_ONE_DIRECTORY, |program| {
        let native = SystemTarget::for_triple(
            crate::backend::target::TargetLayout::host()
                .expect("a supported host")
                .triple(),
        )
        .expect("the host triple is a qualified target");
        let deterministic = SystemTarget::deterministic_test();
        assert_eq!(
            crate::backend::qualification::qualify_program(native, program)
                .expect("the native target admits the program")
                .kind(),
            crate::backend::qualification::qualify_program(deterministic, program)
                .expect("the deterministic target admits the program")
                .kind()
        );
    });
}

#[test]
fn only_the_host_facing_rows_differ_between_the_two_targets() {
    // Selecting the deterministic target redirects exactly the rows that
    // reach a real operating-system object. Everything else — the wrappers,
    // the bootstrap, the release shape, the emitted call count — is the same
    // code under test.
    let native = super::compile(RELEASES_ONE_DIRECTORY);
    let deterministic = emit_for_deterministic_target(RELEASES_ONE_DIRECTORY);

    assert!(native.contains("declare i32 @wf__completion_file_close_direct(i32)"));
    assert!(native.contains("declare i32 @open(ptr, i32, ...)"));
    assert!(!native.contains("wf_test"));

    assert!(deterministic.contains("declare i32 @wf_test_close(i32)"));
    assert!(deterministic.contains("declare i32 @wf_test_open(ptr, i32, ...)"));
    // The redirect is complete: no use site keeps calling the native facility.
    assert!(!deterministic.contains("@wf__completion_file_close_direct(i32 "));
    assert!(!deterministic.contains("@open(ptr "));

    // One release, one close attempt, on either target [SYS-5].
    assert_eq!(
        native
            .matches("call i32 @wf__completion_file_close_direct(i32")
            .count(),
        1
    );
    assert_eq!(
        deterministic.matches("call i32 @wf_test_close(i32").count(),
        1
    );

    // The rest of the module is identical, so a condition forced on the
    // deterministic host is forced on the same lowering the native target
    // emits.
    assert_eq!(
        native
            .replace("@wf__completion_file_close_direct", "@wf_test_close")
            .replace("@open", "@wf_test_open")
            .replace(
                "declare i32 @wf_test_open(ptr, i32, ...)\ndeclare i32 @wf_test_close(i32)",
                "declare i32 @wf_test_close(i32)\ndeclare i32 @wf_test_open(ptr, i32, ...)",
            ),
        deterministic
    );
}

#[test]
fn a_release_close_that_fails_is_attempted_once_and_never_retried() {
    // [SYS-5]: a consuming close is one attempt whose diagnostic is
    // discarded. An interrupted close leaves the descriptor's state
    // unknowable, so retrying could close a descriptor the host has already
    // reused; the release must not. A real directory close cannot be made to
    // report `EINTR` on demand, which is why this case is the deterministic
    // target's.
    let run = run_on_deterministic_host(
        RELEASES_ONE_DIRECTORY,
        &HostScript::new().closes(&[HostOutcome::Fail(HostError::Interrupted)]),
        &[],
    );

    assert_eq!(
        run.attempts("close"),
        1,
        "an interrupted close is never retried; trace was {:?}",
        run.trace()
    );
    // The descriptor closed is the one the scripted directory open produced,
    // and the attempt reported a failure: the value itself is the selected
    // host's own `EINTR`, not a number this test fixes.
    assert!(run.trace().contains("wf_test close fd=41 outcome=error"));
    // The failed release changes nothing the source can observe: the command
    // still produces its own status.
    assert_eq!(run.output.status.code(), Some(0));
}

#[test]
fn a_release_close_that_succeeds_is_also_exactly_one_attempt() {
    // The control for the case above: the single attempt is a property of the
    // release, not of the failure.
    let run = run_on_deterministic_host(
        RELEASES_ONE_DIRECTORY,
        &HostScript::new().closes(&[HostOutcome::Succeed]),
        &[],
    );

    assert_eq!(run.attempts("close"), 1);
    assert!(run.trace().contains("wf_test close fd=41 outcome=ok"));
    assert_eq!(run.output.status.code(), Some(0));
}

#[test]
fn an_inspection_error_survives_a_failed_provisional_close() {
    // `fstat` supplies the source-visible error. [SYS-5] makes the following
    // close exactly one best-effort cleanup attempt whose own diagnostic is
    // discarded, so even an interrupted close cannot replace the typed error
    // with a process abort.
    let source = opens_one_file(
        &[(
            "DeviceFailure",
            "if ieq(o, 4_u8) {\n  let narrowed = cvt<u32, u8>(c);\n  match narrowed {\n    Ok(value: code) => {\n      return exit_status(code: code);\n    }\n    Err(error: overflowed) => {\n      return exit_status(code: 250_u8);\n    }\n  }\n} else {\n  return exit_status(code: 251_u8);\n}",
        )],
        "return exit_status(code: 199_u8);",
    );
    let run = run_on_deterministic_host(
        source.as_bytes(),
        &HostScript::new()
            .file(b"x")
            .file_status(HostFileStatus::Fail(HostError::DeviceFailure))
            .closes(&[
                HostOutcome::Fail(HostError::Interrupted),
                HostOutcome::Succeed,
            ]),
        &[],
    );

    let inspection_code = run
        .trace()
        .lines()
        .find_map(|line| {
            line.strip_prefix("wf_test fstat fd=42 outcome=error code=")
                .and_then(|code| code.parse::<i32>().ok())
        })
        .expect("the failing inspection reports its native code");
    assert_eq!(run.output.status.code(), Some(inspection_code));
    assert_eq!(run.attempts("fstat"), 1);
    assert_eq!(run.attempts("close"), 2);
    assert!(run.trace().contains("wf_test fstat fd=42 outcome=error"));
    assert!(run.trace().contains("wf_test close fd=42 outcome=error"));
    assert!(run.trace().contains("wf_test close fd=41 outcome=ok"));
}

#[test]
fn a_nonregular_result_survives_a_failed_provisional_close() {
    // The classification error is compiler-owned, but provisional cleanup has
    // the same SYS-5 rule: one close attempt, no retry, and no replacement of
    // the already selected source-visible outcome.
    let source = opens_one_file(
        &[(
            "IsDirectory",
            "if ieq(c, 0_u32) {\n  if ieq(o, 0_u8) {\n    return exit_status(code: 23_u8);\n  } else {\n    return exit_status(code: 24_u8);\n  }\n} else {\n  return exit_status(code: 25_u8);\n}",
        )],
        "return exit_status(code: 199_u8);",
    );
    let run = run_on_deterministic_host(
        source.as_bytes(),
        &HostScript::new()
            .file(b"x")
            .file_status(HostFileStatus::Directory)
            .closes(&[
                HostOutcome::Fail(HostError::Interrupted),
                HostOutcome::Succeed,
            ]),
        &[],
    );

    assert_eq!(
        run.output.status.code(),
        Some(23),
        "trace was {:?}",
        run.trace()
    );
    assert_eq!(run.attempts("fstat"), 1);
    assert_eq!(run.attempts("close"), 2);
    assert!(run.trace().contains("wf_test fstat fd=42 outcome=ok"));
    assert!(run.trace().contains("wf_test close fd=42 outcome=error"));
    assert!(run.trace().contains("wf_test close fd=41 outcome=ok"));
}

#[test]
fn the_deterministic_release_keeps_the_native_optimized_shape() {
    // [QUAL-3]: the wrapper inlines and the release stays one direct call on
    // the selected target's own facility, with no dispatch table, no handle
    // lookup, and no allocation introduced by the second column.
    let optimized = host_optimized_module(&emit_for_deterministic_target(RELEASES_ONE_DIRECTORY));
    assert_eq!(optimized.matches("@wf_test_close(").count(), 2);
    assert!(!optimized.contains("@malloc"));
}

#[test]
fn a_mid_stream_read_failure_stops_the_drain_after_the_bytes_it_delivered() {
    // A file that reads normally and then fails part way through cannot be
    // arranged on a real filesystem at a chosen call, so this is the
    // deterministic target's case. The first attempt delivers three bytes and
    // the second reports a device failure; the drain must observe
    // `ReadBytes(3)` then `ReadFailed`, not a silent end of input [SYS-8,
    // SYS-11].
    let run = run_on_deterministic_host(
        CHUNKED_READ,
        &HostScript::new().file(b"abcdefgh").reads(&[
            HostOutcome::Succeed,
            HostOutcome::Fail(HostError::DeviceFailure),
        ]),
        &[b"eight.txt"],
    );

    // 202 is the program's own `ReadFailed` status: the failure reached
    // source as its own outcome and was never reported as the end of input.
    assert_eq!(
        run.output.status.code(),
        Some(202),
        "trace was {:?}",
        run.trace()
    );
    // Exactly two attempts: the failing one ended the drain and nothing
    // retried it.
    assert_eq!(run.attempts("pread"), 2);
    assert!(
        run.trace()
            .contains("wf_test pread fd=42 offset=0 capacity=3 delivered=3")
    );
    assert!(
        run.trace()
            .contains("wf_test pread fd=42 offset=3 capacity=3 -> error")
    );

    // The control: the same program over the same fixture with nothing
    // scripted drains the file to its end and reports its own total.
    let clean = run_on_deterministic_host(
        CHUNKED_READ,
        &HostScript::new().file(b"abcdefgh"),
        &[b"eight.txt"],
    );
    assert_eq!(clean.output.status.code(), Some(83));
    assert_eq!(clean.attempts("pread"), 4);
}

#[test]
fn read_no_progress_answers_are_internal_until_one_positioned_read_progresses() {
    let run = run_on_deterministic_host(
        CHUNKED_READ,
        &HostScript::new().file(b"abc").reads(&[
            HostOutcome::Fail(HostError::Interrupted),
            HostOutcome::Fail(HostError::WouldBlock),
            HostOutcome::Succeed,
        ]),
        &[b"three.txt"],
    );
    assert_eq!(
        run.output.status.code(),
        Some(31),
        "trace was {:?}",
        run.trace()
    );
    assert_eq!(run.trace().matches("progress=eintr").count(), 1);
    assert_eq!(run.trace().matches("wait=readable").count(), 1);
    assert_eq!(run.trace().matches("delivered=3").count(), 1);
    assert!(!run.trace().contains("-> error"));
}

#[test]
fn write_no_progress_answers_are_internal_until_one_write_progresses() {
    let run = run_on_deterministic_host(
        WRITE_PREFIX,
        &HostScript::new().writes(&[
            HostOutcome::Fail(HostError::Interrupted),
            HostOutcome::Fail(HostError::WouldBlock),
            HostOutcome::Succeed,
        ]),
        &[],
    );
    assert_eq!(
        run.output.status.code(),
        Some(3),
        "trace was {:?}",
        run.trace()
    );
    assert_eq!(run.trace().matches("progress=eintr").count(), 1);
    assert_eq!(run.trace().matches("wait=writable").count(), 1);
    assert_eq!(run.trace().matches("accepted=2").count(), 1);
    assert!(run.trace().contains("bytes=xy"));
    assert!(!run.trace().contains("-> error"));
}

#[test]
fn a_forced_short_write_reports_the_absolute_endpoint_after_the_host_prefix() {
    // A destination that accepts only part of one request is not something a
    // regular file or a pipe can be made to do on demand at a chosen call.
    // [SYS-8] makes one `write_once` at most one host attempt, so a partial
    // acceptance is `Ok(next)` with the exact absolute endpoint — never a silent
    // loop that finishes the range, and never an error.
    let run = run_on_deterministic_host(
        WRITE_PREFIX,
        // The program's first request is zero-length and issues no host
        // transfer, so the first scripted entry meets its second request of
        // two bytes.
        &HostScript::new().writes(&[HostOutcome::Accept(1)]),
        &[],
    );

    // The request starts at one, so accepting one byte reports endpoint two.
    assert_eq!(
        run.output.status.code(),
        Some(2),
        "trace was {:?}",
        run.trace()
    );
    // One request, one attempt, and exactly the accepted prefix reached the
    // sink: the unaccepted byte was not written by a retry.
    assert_eq!(run.attempts("write"), 1);
    assert!(
        run.trace()
            .contains("wf_test write fd=1 count=2 accepted=1 bytes=x")
    );

    // The control: with nothing scripted the same request is accepted whole
    // and reports endpoint three.
    let whole = run_on_deterministic_host(WRITE_PREFIX, &HostScript::new(), &[]);
    assert_eq!(whole.output.status.code(), Some(3));
    assert!(
        whole
            .trace()
            .contains("wf_test write fd=1 count=2 accepted=2 bytes=xy")
    );
}

#[test]
fn an_output_sink_that_fails_only_at_close_is_never_closed_by_its_release() {
    // [SYS-12]: releasing an `Output` is a logical source detach — no close,
    // no flush, no target call. A sink whose failure appears only at close or
    // writeback therefore cannot reach the program: every accepted write
    // stands and the command still produces its own status. The scripted
    // close proves the point by never firing for the output.
    let run = run_on_deterministic_host(
        WRITES_THEN_RELEASES_BOTH,
        &HostScript::new()
            // Every close this run makes fails. Only the directory's release
            // closes anything, so only it can consume an entry.
            .closes(&[
                HostOutcome::Fail(HostError::DeviceFailure),
                HostOutcome::Fail(HostError::DeviceFailure),
            ]),
        &[],
    );

    assert_eq!(
        run.output.status.code(),
        Some(3),
        "trace was {:?}",
        run.trace()
    );
    // The write was accepted in full and observed as such by the program.
    assert!(
        run.trace()
            .contains("wf_test write fd=1 count=3 accepted=3 bytes=ABC")
    );
    // Exactly one close attempt, and it is the `DirectoryRead`'s. Neither
    // `Output` owner closed its descriptor, so the sink's close-time failure
    // is outside what any release can observe.
    assert_eq!(run.attempts("close"), 1);
    assert!(run.trace().contains("wf_test close fd=41 outcome=error"));
    assert!(!run.trace().contains("wf_test close fd=1 "));
    assert!(!run.trace().contains("wf_test close fd=2 "));
}

#[test]
fn the_trap_record_writer_stays_native_on_the_deterministic_target() {
    // The mandatory [DIAG-3] record and `write_once` both reach a write
    // facility, but only the operation row has a target column: the record
    // writer is the compiler's own and must never be scriptable, or a forced
    // short write could truncate a trap record. One module declares both.
    let source = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap), traps {
  let bytes = buffer_new(1_u64, 65_u8);
  let bounded = 0_u64;
  let step = 0_u64;
  loop @preserve_zero {
    if ige(step, 4_u64) {
      break @preserve_zero;
    }
    set bounded = bounded +wrap 0_u64;
    set step = step +wrap 1_u64;
  }
  claim record_writer_probe: ilt(bounded, 1_u64) because "premises: bounded starts at 0_u64 and every completed preserve_zero iteration adds wrapping zero\nderivation: adding wrapping zero preserves bounded at 0_u64 through every completed iteration\nconclusion: ilt(bounded, 1_u64) is true\nchecker gap: ENT does not synthesize the loop invariant that bounded remains zero\nconsumers: the following bounded + 1_u64 exact addition requires this bound for its OP-2 domain obligation";
  let successor = bounded + 1_u64;
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 1_u64) {
        Ok(value: next) => {
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let module = emit_for_deterministic_target(source);
    assert!(module.contains("declare i64 @wf_test_write(i32, ptr, i64)"));
    assert!(module.contains("declare i64 @write(i32, ptr, i64)"));
    assert!(module.contains("%written = call i64 @write(i32 2, ptr %cursor"));
    assert!(module.contains("%accepted = call i64 @wf_test_write(i32 %output"));

    // And the native target still declares exactly one `@write` for both.
    let native = super::compile(source);
    assert_eq!(
        native.matches("declare i64 @write(i32, ptr, i64)").count(),
        1
    );
}

#[test]
fn a_host_that_accepts_nothing_reaches_source_as_write_zero() {
    // [SYS-8] makes a host write that accepts zero bytes of a nonempty
    // request `Err(WriteZero())` and never `Ok(0)`, because `Ok(0)` would
    // report progress that did not happen and a write-until-accepted loop
    // would spin on it forever. No real destination produces that answer for a
    // nonempty request, so task 0012 could only establish the outcome from the
    // emitted shape. Scripting the host to accept nothing makes it behavioural:
    // the program observes the class itself.
    //
    // [SYS-7] leaves both detail fields zero for this class, because no native
    // error code produced it, so the case reads them too — the class is not
    // being smuggled in with a borrowed native code or facility origin.
    let arms = class_arms(
        12,
        &[(
            "WriteZero",
            "if ieq(c, 0_u32) {\n  if ieq(o, 0_u8) {\n    return exit_status(code: 120_u8);\n  } else {\n    return exit_status(code: 121_u8);\n  }\n} else {\n  return exit_status(code: 122_u8);\n}",
        )],
        "return exit_status(code: 199_u8);",
    );
    let source = format!(
        r#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {{
  let bytes = buffer_new(2_u64, 119_u8);
  region 'o {{
    region 's {{
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 2_u64) {{
        Ok(value: written) => {{
          let narrowed = cvt<u64, u8>(written);
          match narrowed {{
            Ok(value: code) => {{
              return exit_status(code: code);
            }}
            Err(error: overflowed) => {{
              return exit_status(code: 200_u8);
            }}
          }}
        }}
        Err(error: problem) => {{
          match move problem {{
{arms}          }}
        }}
      }}
    }}
  }}
}}
"#
    );

    let run = run_on_deterministic_host(
        source.as_bytes(),
        &HostScript::new().writes(&[HostOutcome::Accept(0)]),
        &[],
    );
    // 120 is the program's own `WriteZero` status with both detail fields
    // observed as zero.
    assert_eq!(
        run.output.status.code(),
        Some(120),
        "trace was {:?}",
        run.trace()
    );
    // One request, one attempt: the refusal ended the operation and nothing
    // retried it inside `write_once` [SYS-8].
    assert_eq!(run.attempts("write"), 1);
    assert!(
        run.trace()
            .contains("wf_test write fd=1 count=2 accepted=0 bytes=")
    );

    // The control: with nothing scripted the same request is accepted whole
    // and the same program reports the accepted count instead of any class.
    let whole = run_on_deterministic_host(source.as_bytes(), &HostScript::new(), &[]);
    assert_eq!(whole.output.status.code(), Some(2));
    assert!(
        whole
            .trace()
            .contains("wf_test write fd=1 count=2 accepted=2 bytes=ww")
    );
}
