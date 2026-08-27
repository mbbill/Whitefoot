//! Emitted-shape and behaviour evidence for the native I/O cluster of the
//! qualified [SYS-2] system interface: `open_read`, `read_at`,
//! `write_once`, and the [SYS-7] class mapping they share.
//!
//! The behaviour cases compile, link, and run each program against real
//! directories and files, because [SYS-8]'s one-attempt transfer semantics —
//! that a short success is not end of input, that a zero-length range issues
//! no host transfer, and that the returned endpoint advances by exactly the
//! accepted count — are
//! observable only by performing a transfer. The cost-shape assertions read
//! the module the host optimizer leaves, which is what [QUAL-3] says
//! establishes the required emitted shape: inspection of emitted code and
//! symbols, not a machine-checked language judgment.

use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use super::{
    build_executable, compile, compile_rejection, host_optimized_module, optimized_main,
    test_directory,
};

/// Runs one emitted module in a fresh directory holding the given fixtures.
///
/// `command.cwd` then names a directory whose complete content the case
/// fixes, so a directory-relative open resolves against known objects.
fn run_in_directory(
    llvm: &str,
    fixtures: &[(&str, &[u8])],
    arguments: &[&[u8]],
) -> std::process::Output {
    let directory = test_directory();
    let executable = build_executable(llvm, &directory);
    write_fixtures(&directory, fixtures);
    let output = Command::new(&executable)
        .current_dir(&directory)
        .args(
            arguments
                .iter()
                .map(|bytes| std::ffi::OsStr::from_bytes(bytes)),
        )
        .output()
        .expect("run backend test executable");
    std::fs::remove_dir_all(&directory).expect("remove backend test directory");
    output
}

fn write_fixtures(directory: &Path, fixtures: &[(&str, &[u8])]) {
    for (name, bytes) in fixtures {
        let path = directory.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("fixture parent directory");
        }
        std::fs::write(&path, bytes).expect("write fixture");
    }
}

/// Runs one emitted module with standard output on a pipe whose read end is
/// closed before the program's writes are consumed.
fn run_with_closed_output(llvm: &str) -> ExitStatus {
    let directory = test_directory();
    let executable = build_executable(llvm, &directory);
    let mut child = Command::new(&executable)
        .current_dir(&directory)
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn backend test executable");
    // Dropping the read end closes the destination under a program that is
    // already publishing to it.
    drop(child.stdout.take());
    let status = child.wait().expect("wait for backend test executable");
    std::fs::remove_dir_all(&directory).expect("remove backend test directory");
    status
}

/// The [SYS-2] `IoError` class spellings in declared order.
///
/// The corpus reads the inventory rather than restating it, so an exhaustive
/// [ERR-2] match written here cannot drift from the closed class set.
fn io_error_classes() -> Vec<&'static str> {
    let owner = crate::SYSTEM_NOMINALS
        .iter()
        .position(|nominal| nominal.spelling == "IoError")
        .expect("the inventory declares IoError");
    crate::SYSTEM_CONSTRUCTORS
        .iter()
        .filter(|constructor| usize::from(constructor.owner) == owner)
        .map(|constructor| constructor.spelling)
        .collect()
}

/// Renders an exhaustive match over the closed twenty-eight-class set.
///
/// `named` gives the arm body of the classes a case distinguishes and
/// `default` the body of every other class; `indent` is the column the arms
/// start at, because a case is canonical source.
pub(super) fn class_arms(indent: usize, named: &[(&str, &str)], default: &str) -> String {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    let mut arms = String::new();
    for class in io_error_classes() {
        let body = named
            .iter()
            .find(|(spelling, _)| *spelling == class)
            .map_or(default, |(_, body)| body);
        let body: String = body
            .lines()
            .map(|line| format!("{inner}{line}\n"))
            .collect();
        arms.push_str(&format!(
            "{pad}{class}(code: c, origin: o) => {{\n{body}{pad}}}\n"
        ));
    }
    arms
}

/// Opens one argument-named path under `command.cwd` and reads its first
/// bytes, reporting the exact count.
const OPEN_AND_READ: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files), allocates(heap) {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                let permit = reserve_file<'c>(factory: &uniq 'c files);
                match open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let bytes = buffer_new(64_u64, 0_u8);
                    region 'f {
                      region 'd {
                        match read_at<'f, 'd>(file: &'f file, destination: &uniq 'd bytes, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
                          ReadBytes(next: n) => {
                            let narrowed = cvt<u64, u8>(n);
                            match narrowed {
                              Ok(value: code) => {
                                return exit_status(code: code);
                              }
                              Err(error: overflowed) => {
                                return exit_status(code: 200_u8);
                              }
                            }
                          }
                          ReadEnd() => {
                            return exit_status(code: 201_u8);
                          }
                          ReadFailed(error: problem) => {
                            return exit_status(code: 202_u8);
                          }
                        }
                      }
                    }
                  }
                  Err(error: problem) => {
                    return exit_status(code: 203_u8);
                  }
                }
              }
            }
          }
          Err(error: rejected) => {
            return exit_status(code: 204_u8);
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 205_u8);
      }
    }
  }
}
"#;

#[test]
fn open_read_resolves_a_relative_path_through_the_targets_own_facility() {
    let llvm = compile(OPEN_AND_READ);
    // [PATH-2]: resolution uses the target's own directory-relative facility
    // against the supplied directory's descriptor, never a prefix concatenated onto a
    // path and resolved against an ambient working directory.
    assert!(llvm.contains("; QUAL-1 semantic id 7 -> @wf.sys.open_read.v1"));
    assert!(llvm.contains(
        "declare i32 @wf__completion_file_open_at_direct(i32, ptr, i32, i32, i32, i32, ptr, ptr)"
    ));
    assert!(llvm.contains(
        "@wf__completion_file_open_at_direct(i32 %root, ptr %text, i32 0, i32 0, i32 0, i32 1, ptr %open.error.slot, ptr %open.outcome.slot)"
    ));
    for absent in ["@getcwd", "@chdir", "@realpath", "@strcat", "@snprintf"] {
        assert!(
            !llvm.contains(absent),
            "a directory-relative open must not reach {absent}:\n{llvm}"
        );
    }
    // The supplied directory's own descriptor is the resolution root.
    assert!(llvm.contains("%descriptor = call i32 @wf__completion_file_open_at_direct"));

    // A path naming a file in the initial directory opens and reads it; the
    // exact byte count reaches source.
    let output = run_in_directory(&llvm, &[("fixture.txt", b"hello")], &[b"fixture.txt"]);
    assert_eq!(output.status.code(), Some(5));
    // `.` and `..` components resolve exactly as the surrounding process
    // namespace does; the directory value makes no confinement claim [PATH-2].
    let nested = run_in_directory(
        &llvm,
        &[("inner/fixture.txt", b"hello there")],
        &[b"./inner/../inner/fixture.txt"],
    );
    assert_eq!(nested.status.code(), Some(11));
    // An empty file is end of input on the first attempt, not a failure.
    let empty = run_in_directory(&llvm, &[("empty.txt", b"")], &[b"empty.txt"]);
    assert_eq!(empty.status.code(), Some(201));
}

#[test]
fn open_read_maps_one_native_failure_onto_one_portable_class() {
    // The class is the sole portable discriminator, and every class carries
    // the same two-field inline target detail [SYS-7]. `NotFound` reports its
    // native code and the target's own facility discriminator.
    let arms = class_arms(
        22,
        &[
            (
                "NotFound",
                "if ieq(c, 2_u32) {\n  if ieq(o, 1_u8) {\n    return exit_status(code: 100_u8);\n  } else {\n    return exit_status(code: 101_u8);\n  }\n} else {\n  return exit_status(code: 102_u8);\n}",
            ),
            ("PermissionDenied", "return exit_status(code: 110_u8);"),
            ("NotDirectory", "return exit_status(code: 111_u8);"),
            ("IsDirectory", "return exit_status(code: 112_u8);"),
        ],
        "return exit_status(code: 199_u8);",
    );
    let source = format!(
        r#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files) {{
  region 'a {{
    match arg_get<'a>(args: &'a args, position: 1_u64) {{
      Ok(value: text) => {{
        match relative_path(value: move text) {{
          Ok(value: path) => {{
            region 'c {{
              region 'p {{
                let permit = reserve_file<'c>(factory: &uniq 'c files);
                match open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path) {{
                  Ok(value: file) => {{
                    return exit_status(code: 0_u8);
                  }}
                  Err(error: problem) => {{
                    match move problem {{
{arms}                    }}
                  }}
                }}
              }}
            }}
          }}
          Err(error: rejected) => {{
            return exit_status(code: 204_u8);
          }}
        }}
      }}
      Err(error: absent) => {{
        return exit_status(code: 205_u8);
      }}
    }}
  }}
}}
"#
    );
    let llvm = compile(source.as_bytes());
    // The mapper is one cold function reached only on failure [QUAL-3].
    assert!(llvm.contains("@wf.sys.io.error(i32 %code, i8 %origin) noinline cold"));
    // Its switch names one arm per mapped native code, each code selecting
    // exactly one class, and its default is `Other` — the closed set's own
    // rule for a native error with no portable distinction, not a wildcard
    // that narrows distinguishable failures [SYS-7].
    let other = io_error_classes()
        .iter()
        .position(|class| *class == "Other")
        .expect("the inventory declares Other");
    assert!(llvm.contains(&format!("switch i32 %code, label %class.{other} [")));
    let mut mapped = std::collections::BTreeSet::new();
    for line in llvm.lines() {
        let Some(rest) = line.trim_start().strip_prefix("i32 ") else {
            continue;
        };
        let Some((code, label)) = rest.split_once(", label %class.") else {
            continue;
        };
        assert!(mapped.insert(code.to_owned()), "{code} is mapped twice");
        assert!(
            label.parse::<usize>().expect("a class tag") < io_error_classes().len(),
            "{label} is not a declared class"
        );
    }
    assert!(mapped.len() >= 28, "the mapper carries the target's table");

    let fixtures: &[(&str, &[u8])] = &[("fixture.txt", b"hello")];
    // A present file opens.
    assert_eq!(
        run_in_directory(&llvm, fixtures, &[b"fixture.txt"])
            .status
            .code(),
        Some(0)
    );
    // An absent name is `NotFound`, carrying the target's own native code and
    // the discriminator of the facility that produced it.
    assert_eq!(
        run_in_directory(&llvm, fixtures, &[b"missing.txt"])
            .status
            .code(),
        Some(100)
    );
    // A component that is not a directory is `NotDirectory`.
    assert_eq!(
        run_in_directory(&llvm, fixtures, &[b"fixture.txt/inner"])
            .status
            .code(),
        Some(111)
    );
    // A target-root prefix never reaches the operation at all: it is
    // `PathInvalid` at construction [PATH-1].
    assert_eq!(
        run_in_directory(&llvm, fixtures, &[b"/etc/hosts"])
            .status
            .code(),
        Some(204)
    );
}

/// Drains one file in three-byte requests, reporting `total * 10 + requests`.
pub(super) const CHUNKED_READ: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files), allocates(heap) {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                let permit = reserve_file<'c>(factory: &uniq 'c files);
                match open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let bytes = buffer_new(3_u64, 0_u8);
                    let total = 0_u64;
                    let chunks = 0_u64;
                    let failed = False();
                    loop @drain {
                      region 'f {
                        region 'd {
                          match read_at<'f, 'd>(file: &'f file, destination: &uniq 'd bytes, file_offset: total, start: 0_u64, end: 3_u64) {
                            ReadBytes(next: n) => {
                              set total = total +wrap n;
                              set chunks = chunks +wrap 1_u64;
                            }
                            ReadEnd() => {
                              break @drain;
                            }
                            ReadFailed(error: problem) => {
                              set failed = True();
                              break @drain;
                            }
                          }
                        }
                      }
                    }
                    if failed {
                      return exit_status(code: 202_u8);
                    }
                    let scaled = total *wrap 10_u64;
                    let mixed = scaled +wrap chunks;
                    let narrowed = cvt<u64, u8>(mixed);
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
                    return exit_status(code: 203_u8);
                  }
                }
              }
            }
          }
          Err(error: rejected) => {
            return exit_status(code: 204_u8);
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 205_u8);
      }
    }
  }
}
"#;

#[test]
fn a_short_read_is_progress_and_only_the_observed_end_is_read_end() {
    let llvm = compile(CHUNKED_READ);
    // Five bytes in three-byte requests: three, then two, then end. The short
    // second success is progress, not end of input, and each returned absolute
    // endpoint becomes the next cursor, so the drain totals the file exactly
    // [SYS-8, SYS-11].
    assert_eq!(
        run_in_directory(&llvm, &[("five.txt", b"abcde")], &[b"five.txt"])
            .status
            .code(),
        Some(52)
    );
    // An exact-capacity file still needs the following attempt to observe the
    // end: three bytes in one request, then `ReadEnd`.
    assert_eq!(
        run_in_directory(&llvm, &[("three.txt", b"abc")], &[b"three.txt"])
            .status
            .code(),
        Some(31)
    );
    // An empty file reports the end on the first attempt and nothing else.
    assert_eq!(
        run_in_directory(&llvm, &[("empty.txt", b"")], &[b"empty.txt"])
            .status
            .code(),
        Some(0)
    );
}

/// Reports a zero-length read's endpoint, then the following request's endpoint.
const VACANT_READ: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files), allocates(heap) {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                let permit = reserve_file<'c>(factory: &uniq 'c files);
                match open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let bytes = buffer_new(8_u64, 0_u8);
                    let vacant = 0_u64;
                    region 'f {
                      region 'd {
                        match read_at<'f, 'd>(file: &'f file, destination: &uniq 'd bytes, file_offset: 0_u64, start: 0_u64, end: 0_u64) {
                          ReadBytes(next: n) => {
                            set vacant = n;
                          }
                          ReadEnd() => {
                            return exit_status(code: 210_u8);
                          }
                          ReadFailed(error: problem) => {
                            return exit_status(code: 211_u8);
                          }
                        }
                      }
                    }
                    if ieq(vacant, 0_u64) {
                    } else {
                      return exit_status(code: 212_u8);
                    }
                    region 'g {
                      region 'e {
                        match read_at<'g, 'e>(file: &'g file, destination: &uniq 'e bytes, file_offset: 0_u64, start: 0_u64, end: 8_u64) {
                          ReadBytes(next: n) => {
                            let narrowed = cvt<u64, u8>(n);
                            match narrowed {
                              Ok(value: code) => {
                                return exit_status(code: code);
                              }
                              Err(error: overflowed) => {
                                return exit_status(code: 200_u8);
                              }
                            }
                          }
                          ReadEnd() => {
                            return exit_status(code: 213_u8);
                          }
                          ReadFailed(error: problem) => {
                            return exit_status(code: 214_u8);
                          }
                        }
                      }
                    }
                  }
                  Err(error: problem) => {
                    return exit_status(code: 203_u8);
                  }
                }
              }
            }
          }
          Err(error: rejected) => {
            return exit_status(code: 204_u8);
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 205_u8);
      }
    }
  }
}
"#;

#[test]
fn a_zero_length_read_reports_no_bytes_without_issuing_a_host_transfer() {
    let llvm = compile(VACANT_READ);
    // A zero-length range reports `next = start` and issues no host transfer,
    // and is never reported as `ReadEnd` [SYS-8]. The following
    // request still reads from the same position, so no cursor moved.
    assert_eq!(
        run_in_directory(&llvm, &[("five.txt", b"abcde")], &[b"five.txt"])
            .status
            .code(),
        Some(5)
    );
    // The witness that no transfer was issued: at end of input the host read
    // would have reported zero bytes, which is exactly `ReadEnd`. The
    // zero-length request reports `ReadBytes(0)` there too, and only the
    // following nonempty request observes the end.
    assert_eq!(
        run_in_directory(&llvm, &[("empty.txt", b"")], &[b"empty.txt"])
            .status
            .code(),
        Some(213)
    );
}

/// Reads three bytes into the middle of a sentinel buffer and digests the
/// complete buffer afterwards.
const EXACT_PREFIX: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files), allocates(heap) {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                let permit = reserve_file<'c>(factory: &uniq 'c files);
                match open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let bytes = buffer_new(8_u64, 7_u8);
                    region 'f {
                      region 'd {
                        match read_at<'f, 'd>(file: &'f file, destination: &uniq 'd bytes, file_offset: 0_u64, start: 2_u64, end: 5_u64) {
                          ReadBytes(next: n) => {
                            if ieq(n, 5_u64) {
                            } else {
                              return exit_status(code: 250_u8);
                            }
                          }
                          ReadEnd() => {
                            return exit_status(code: 251_u8);
                          }
                          ReadFailed(error: problem) => {
                            return exit_status(code: 252_u8);
                          }
                        }
                      }
                    }
                    let digest = 0_u64;
                    let cursor = 0_u64;
                    loop @fold {
                      if ieq(cursor, 8_u64) {
                        break @fold;
                      }
                      let fold_ok = ilt(cursor, 8_u64);
                      if fold_ok {
                        let byte = bytes[cursor];
                        let widened = cvt<u8, u64>(byte);
                        let scaled = digest *wrap 31_u64;
                        set digest = scaled +wrap widened;
                        set cursor = cursor +wrap 1_u64;
                      } else {
                        return exit_status(code: 253_u8);
                      }
                    }
                    let masked = iand(digest, 255_u64);
                    let narrowed = cvt<u64, u8>(masked);
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
                    return exit_status(code: 203_u8);
                  }
                }
              }
            }
          }
          Err(error: rejected) => {
            return exit_status(code: 204_u8);
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 205_u8);
      }
    }
  }
}
"#;

#[test]
fn a_successful_read_changes_exactly_the_requested_prefix() {
    let llvm = compile(EXACT_PREFIX);
    // On `ReadBytes(next)` exactly `[start, next)` may have changed and every
    // other byte of the buffer is unchanged [SYS-8]. The digest is over the
    // whole buffer, so any other write shows.
    let expected = [7_u8, 7, b'a', b'b', b'c', 7, 7, 7]
        .iter()
        .fold(0_u64, |digest, byte| {
            digest.wrapping_mul(31).wrapping_add(u64::from(*byte))
        });
    let status = u8::try_from(expected & 255).expect("the mask fits a command code");
    assert_eq!(
        run_in_directory(&llvm, &[("five.txt", b"abcde")], &[b"five.txt"])
            .status
            .code(),
        Some(i32::from(status))
    );
}

/// Writes nothing, then the two-byte prefix at offset one.
pub(super) const WRITE_PREFIX: &[u8] = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(4_u64, 119_u8);
  set bytes[1_u64] = 120_u8;
  set bytes[2_u64] = 121_u8;
  set bytes[3_u64] = 122_u8;
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 0_u64) {
        Ok(value: written) => {
          if ieq(written, 0_u64) {
          } else {
            return exit_status(code: 210_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 211_u8);
        }
      }
    }
  }
  region 'p {
    region 't {
      match write_once<'p, 't>(output: &uniq 'p out, source: &'t bytes, start: 1_u64, end: 3_u64) {
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
          return exit_status(code: 212_u8);
        }
      }
    }
  }
}
"#;

#[test]
fn write_once_publishes_the_requested_range_and_reports_its_absolute_endpoint() {
    let llvm = compile(WRITE_PREFIX);
    let output = run_in_directory(&llvm, &[], &[]);
    // The zero-length range issued no host transfer and reported its start as
    // the endpoint; the nonempty range published exactly the requested prefix
    // of the source and reported the absolute endpoint three [SYS-8, SYS-12].
    assert_eq!(output.stdout, b"xy");
    assert_eq!(output.status.code(), Some(3));
    // A host zero-length write is `Err(WriteZero())` and never `Ok(0)`; no
    // host produces it for a nonempty request against these destinations, so
    // the emitted shape is the evidence.
    assert!(llvm.contains("%refused = icmp eq i64 %accepted, 0"));
    assert!(llvm.contains("br i1 %refused, label %zero, label %failure"));
}

/// Requests a range that runs past the end of its source buffer.
const OUT_OF_RANGE_WRITE: &[u8] = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(4_u64, 65_u8);
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 1_u64, end: 9_u64) {
        Ok(value: written) => {
          return exit_status(code: 10_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 20_u8);
        }
      }
    }
  }
}
"#;

#[test]
fn an_out_of_range_transfer_is_a_static_sys8_rejection() {
    let failure = compile_rejection(OUT_OF_RANGE_WRITE);
    assert_eq!(failure.rule_id(), Some("SYS-8"));
    assert!(failure.detail().contains("9_u64 <= len(buffer)"));
}

/// Publishes three reservations through one Output root.
const ORDERED_WRITES: &[u8] = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(3_u64, 65_u8);
  set bytes[1_u64] = 66_u8;
  set bytes[2_u64] = 67_u8;
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 1_u64) {
        Ok(value: written) => {
        }
        Err(error: problem) => {
          return exit_status(code: 210_u8);
        }
      }
    }
  }
  region 'p {
    region 't {
      match write_once<'p, 't>(output: &uniq 'p out, source: &'t bytes, start: 1_u64, end: 2_u64) {
        Ok(value: written) => {
        }
        Err(error: problem) => {
          return exit_status(code: 211_u8);
        }
      }
    }
  }
  region 'q {
    region 'u {
      match write_once<'q, 'u>(output: &uniq 'q out, source: &'u bytes, start: 2_u64, end: 3_u64) {
        Ok(value: written) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 212_u8);
        }
      }
    }
  }
}
"#;

#[test]
fn ordered_reservations_on_one_output_preserve_source_order() {
    let llvm = compile(ORDERED_WRITES);
    let output = run_in_directory(&llvm, &[], &[]);
    // One logical Output root assigns all three reservations before target
    // completion can race. Environment aliasing is deliberately irrelevant
    // to this ordering oracle [EFF-5, SYS-12].
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"ABC");
}

#[test]
fn a_closed_destination_arrives_as_a_recoverable_broken_pipe() {
    let arms = class_arms(
        14,
        &[("BrokenPipe", "set status = 42_u8;")],
        "set status = 43_u8;",
    );
    let source = format!(
        r#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {{
  let bytes = buffer_new(1_u64, 65_u8);
  let attempts = 0_u64;
  let status = 44_u8;
  loop @publish {{
    if ige(attempts, 200000_u64) {{
      break @publish;
    }}
    set attempts = attempts +wrap 1_u64;
    region 'o {{
      region 's {{
        match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 1_u64) {{
          Ok(value: written) => {{
          }}
          Err(error: problem) => {{
            match move problem {{
{arms}            }}
            break @publish;
          }}
        }}
      }}
    }}
  }}
  return exit_status(code: status);
}}
"#
    );
    let llvm = compile(source.as_bytes());
    // The bootstrap installed the ignored write-to-closed-pipe disposition
    // once, before entry, so a closed destination reaches source as the
    // recoverable `BrokenPipe` class instead of ending the process, and no
    // transfer performs a per-call signal-disposition operation [QUAL-3,
    // SYS-12].
    assert_eq!(llvm.matches("@signal(i32 13,").count(), 1);
    let status = run_with_closed_output(&llvm);
    assert_eq!(
        status.code(),
        Some(42),
        "a closed destination must return a recoverable outcome"
    );
}

/// Drains one file through a reusable buffer and publishes one byte.
const TRANSFER_SHAPE: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, out, files), writes(cwd, out, files), allocates(heap) {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                let permit = reserve_file<'c>(factory: &uniq 'c files);
                match open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let bytes = buffer_new(4096_u64, 0_u8);
                    let total = 0_u64;
                    loop @drain {
                      region 'f {
                        region 'd {
                          match read_at<'f, 'd>(file: &'f file, destination: &uniq 'd bytes, file_offset: total, start: 0_u64, end: 4096_u64) {
                            ReadBytes(next: n) => {
                              set total = total +wrap n;
                            }
                            ReadEnd() => {
                              break @drain;
                            }
                            ReadFailed(error: problem) => {
                              return exit_status(code: 202_u8);
                            }
                          }
                        }
                      }
                    }
                    region 'o {
                      region 's {
                        match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 1_u64) {
                          Ok(value: written) => {
                            let masked = iand(total, 255_u64);
                            let narrowed = cvt<u64, u8>(masked);
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
                            return exit_status(code: 212_u8);
                          }
                        }
                      }
                    }
                  }
                  Err(error: problem) => {
                    return exit_status(code: 203_u8);
                  }
                }
              }
            }
          }
          Err(error: rejected) => {
            return exit_status(code: 204_u8);
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 205_u8);
      }
    }
  }
}
"#;

#[test]
fn the_transfer_path_carries_no_allocation_copy_dispatch_or_lock() {
    let llvm = compile(TRANSFER_SHAPE);
    // Selection is static for the whole build: each identity resolved to one
    // private ABI symbol before emission, and the module contains no runtime
    // operation-ID switch, target tag, per-call dispatch table, or handle
    // lookup [QUAL-1, QUAL-3].
    assert!(llvm.contains("; QUAL-1 semantic id 7 -> @wf.sys.open_read.v1"));
    assert!(llvm.contains("; QUAL-1 semantic id 8 -> @wf.sys.read_at.v1"));
    assert!(llvm.contains("; QUAL-1 semantic id 9 -> @wf.sys.write_once.v1"));
    assert!(!llvm.contains("@wf.sys.dispatch"));

    let optimized = host_optimized_module(&llvm);
    let entry = optimized_main(&optimized);
    // The compiler wrapper is inlined, which is the condition of
    // qualification [QUAL-3].
    assert!(
        !entry.contains("call") || !entry.contains("@wf.sys."),
        "no approved-implementation wrapper survives on the transfer path:\n{entry}"
    );
    // One source transfer is one call into the compiler-owned progress
    // adapter. EINTR/readiness retries remain inside that call and only its
    // first progress-producing or terminal answer reaches this path.
    assert_eq!(
        entry.matches("@wf__completion_file_pread_direct(").count(),
        1,
        "{entry}"
    );
    assert_eq!(
        entry.matches("@wf__completion_file_write_direct(").count(),
        1,
        "{entry}"
    );
    assert_eq!(
        entry
            .matches("@wf__completion_file_open_at_direct(")
            .count(),
        1,
        "{entry}"
    );
    // The transfer performs no heap allocation and copies no transferred
    // byte: the only allocation in the program is the source buffer the
    // writer asked for, reused across every read [QUAL-3].
    assert_eq!(
        entry.matches("@calloc(").count() + entry.matches("@malloc(").count(),
        1,
        "{entry}"
    );
    for forbidden in [
        "@llvm.memcpy",
        "@llvm.memmove",
        "@memcpy",
        "@realloc",
        "@pthread_mutex_lock",
        "@flockfile",
        "@funlockfile",
        "@fwrite",
        "@sigaction",
        "@sigprocmask",
    ] {
        assert!(
            !entry.contains(forbidden),
            "the transfer path must not contain {forbidden}:\n{entry}"
        );
    }
    // One-time normalization belongs to the bootstrap, not to any transfer.
    assert_eq!(entry.matches("@signal(i32 13,").count(), 1, "{entry}");
    // No indirect call: every call names a symbol.
    for indirect in ["call i64 %", "call i32 %", "call void %", "call ptr %"] {
        assert!(!entry.contains(indirect), "{entry}");
    }

    // The program still computes the right answer through that shape: five
    // bytes drained through the reused buffer, whose first byte is then
    // published.
    let output = run_in_directory(&llvm, &[("fixture.txt", b"abcde")], &[b"fixture.txt"]);
    assert_eq!(output.status.code(), Some(5));
    assert_eq!(output.stdout, b"a");
}

#[test]
fn an_opened_file_releases_with_one_direct_close_that_is_never_retried() {
    let llvm = compile(OPEN_AND_READ);
    // `DirectoryRead` and `ReadFile` release with at most one native close
    // attempt [SYS-5]; every release site is one direct call to the one
    // declared close symbol.
    assert!(llvm.contains("declare i32 @wf__completion_file_close_release(i32)"));
    let releases = llvm
        .matches("call i32 @wf__completion_file_close_release(i32")
        .count();
    assert!(releases >= 2, "both closing owners must release:\n{llvm}");
    // The close diagnostic is discarded and an ambiguous close is never
    // retried: every release value is produced by a close, named once, and
    // never read again, so nothing branches on it.
    let mut inspected = 0;
    for line in llvm.lines() {
        let Some(rest) = line.trim_start().strip_prefix("%release.") else {
            continue;
        };
        assert!(
            line.contains("call i32 @wf__completion_file_close_release(i32"),
            "a release value comes from one direct close:\n{line}"
        );
        let ordinal = rest
            .split_whitespace()
            .next()
            .expect("a release value has an ordinal");
        let name = format!("%release.{ordinal} ");
        assert_eq!(
            llvm.matches(&name).count(),
            1,
            "the close diagnostic must be discarded, not inspected"
        );
        inspected += 1;
    }
    assert_eq!(inspected, releases);
}

/// Uses every one of the eleven [SYS-2] operations once: it counts the
/// invocation vector, leases the argument, measures and copies it by both
/// routes, retypes it as a relative path, opens that path under the initial
/// directory, copies the file to standard output through a reused buffer,
/// echoes the argument to standard error, and returns a command code.
const COMPLETE_FIRST_SLICE: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, out, err, files), writes(cwd, out, err, files), allocates(heap) {
  let echo = buffer_new(64_u64, 0_u8);
  let name_length = 0_u64;
  region 'a {
    let arguments = args_count<'a>(args: &'a args);
    if ieq(arguments, 2_u64) {
    } else {
      return exit_status(code: 2_u8);
    }
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        region 'v {
          set name_length = host_bytes_len<'v>(value: &'v text);
          region 'd {
            match host_copy_bytes<'v, 'd>(value: &'v text, destination: &uniq 'd echo, start: 0_u64, end: 64_u64) {
              Ok(value: copied) => {
              }
              Err(error: problem) => {
                return exit_status(code: 3_u8);
              }
            }
          }
          match host_utf8_len<'v>(value: &'v text) {
            Ok(value: measured) => {
            }
            Err(error: invalid) => {
              return exit_status(code: 4_u8);
            }
          }
          region 'e {
            match host_copy_utf8<'v, 'e>(value: &'v text, destination: &uniq 'e echo, start: 0_u64, end: 64_u64) {
              Ok(value: encoded) => {
              }
              Err(error: problem) => {
                return exit_status(code: 5_u8);
              }
            }
          }
        }
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                let permit = reserve_file<'c>(factory: &uniq 'c files);
                match open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let page = buffer_new(16_u64, 0_u8);
                    let total = 0_u64;
                    let file_offset = 0_u64;
                    let failed = 0_u8;
                    loop @copy {
                      let chunk = 0_u64;
                      region 'f {
                        region 'g {
                          match read_at<'f, 'g>(file: &'f file, destination: &uniq 'g page, file_offset: file_offset, start: 0_u64, end: 16_u64) {
                            ReadBytes(next: n) => {
                              set chunk = n;
                              set file_offset = file_offset +wrap n;
                            }
                            ReadEnd() => {
                              break @copy;
                            }
                            ReadFailed(error: problem) => {
                              set failed = 8_u8;
                              break @copy;
                            }
                          }
                        }
                      }
                      let page_length = len(page);
                      let chunk_fits = ile(chunk, page_length);
                      if chunk_fits {
                      } else {
                        return exit_status(code: 12_u8);
                      }
                      region 'o {
                        region 's {
                          match write_once<'o, 's>(output: &uniq 'o out, source: &'s page, start: 0_u64, end: chunk) {
                            Ok(value: written) => {
                              set total = total +wrap written;
                            }
                            Err(error: problem) => {
                              set failed = 9_u8;
                              break @copy;
                            }
                          }
                        }
                      }
                    }
                    if ieq(failed, 0_u8) {
                    } else {
                      return exit_status(code: failed);
                    }
                    let echo_length = len(echo);
                    let name_fits = ile(name_length, echo_length);
                    if name_fits {
                    } else {
                      return exit_status(code: 13_u8);
                    }
                    region 'x {
                      region 'y {
                        match write_once<'x, 'y>(output: &uniq 'x err, source: &'y echo, start: 0_u64, end: name_length) {
                          Ok(value: written) => {
                            let masked = iand(total, 255_u64);
                            let narrowed = cvt<u64, u8>(masked);
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
                            return exit_status(code: 10_u8);
                          }
                        }
                      }
                    }
                  }
                  Err(error: problem) => {
                    return exit_status(code: 6_u8);
                  }
                }
              }
            }
          }
          Err(error: rejected) => {
            return exit_status(code: 7_u8);
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 11_u8);
      }
    }
  }
}
"#;

#[test]
fn the_complete_first_slice_compiles_links_and_runs() {
    let llvm = compile(COMPLETE_FIRST_SLICE);
    // Every one of the eleven [SYS-2] semantic identities now resolves to an
    // approved implementation on this target, so no part of the first slice
    // stops before emission [QUAL-1].
    for ordinal in 0..11 {
        assert!(
            llvm.contains(&format!("; QUAL-1 semantic id {ordinal} -> @wf.sys.")),
            "semantic id {ordinal} has no approved implementation:\n{llvm}"
        );
    }
    let output = run_in_directory(
        &llvm,
        &[("page.txt", b"one line and then a longer second line\n")],
        &[b"page.txt"],
    );
    assert_eq!(output.stdout, b"one line and then a longer second line\n");
    assert_eq!(output.stderr, b"page.txt");
    assert_eq!(output.status.code(), Some(39));
}

#[test]
fn every_portable_class_is_mapped_exactly_once_in_inventory_order() {
    use crate::backend::qualification::SystemTarget;

    let declared = io_error_classes();
    assert_eq!(declared.len(), 28);
    for triple in [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
    ] {
        let target = SystemTarget::for_triple(triple).expect("a qualified command target");
        let rows = target.error_classes();
        // The table is the complete closed class set in declared order, so no
        // class is narrowed away by omission [SYS-7].
        assert_eq!(rows.len(), declared.len());
        for (row, class) in rows.iter().zip(&declared) {
            assert_eq!(row.class, *class, "{triple}");
        }
        // One native error maps onto exactly one class.
        let mut seen = std::collections::BTreeSet::new();
        for row in rows {
            for code in row.codes {
                assert!(seen.insert(*code), "{triple} maps {code} twice");
                assert!(*code > 0, "{triple} maps a non-error code");
            }
        }
        // The classes a native error never produces carry no code, and
        // `Other` is the default arm rather than a mapped code.
        for row in rows {
            if matches!(row.class, "WriteZero" | "UnexpectedEnd" | "Other") {
                assert!(row.codes.is_empty(), "{triple} {}", row.class);
            } else {
                assert!(!row.codes.is_empty(), "{triple} {}", row.class);
            }
        }
    }
}
