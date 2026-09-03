use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use super::deterministic_target::{HostScript, run_emitted_on_deterministic_host};
use super::system::with_mutated_completion_ir;
use super::{build_executable, emit, emit_lowered, emitted_function, test_directory};
use crate::OverlapLowering;
use crate::backend::emitter::emit_llvm_for_target;
use crate::backend::qualification::SystemTarget;

const INDEPENDENT_WRITES: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let bulk = buffer_new(1048576_u64, 65_u8);
  let marker = buffer_new(1_u64, 77_u8);
  region 'out {
    region 'err {
      region 'bulk {
        region {
          let first = write_once(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1048576_u64);
          let second = write_once(output: &uniq 'err err, source: &marker, start: 0_u64, end: 1_u64);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const POSITIONED_READS: &[u8] = br#"fn probe(file: own ReadFile) -> result: own unit reads(file), writes(file), allocates(heap) {
  let left = buffer_new(1_u64, 0_u8);
  let right = buffer_new(1_u64, 0_u8);
  region 'file {
    region 'left {
      region {
        let first = read_at(file: &'file file, destination: &uniq 'left left, file_offset: 0_u64, start: 0_u64, end: 1_u64);
        let second = read_at(file: &'file file, destination: &uniq right, file_offset: 1_u64, start: 0_u64, end: 1_u64);
      }
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

const REUSED_OUTPUT_AROUND_INDEPENDENT_OUTPUT: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let first_bytes = buffer_new(1_u64, 65_u8);
  let middle_bytes = buffer_new(1_u64, 66_u8);
  let last_bytes = buffer_new(1_u64, 67_u8);
  region 'out {
    region 'err {
      region 'first_bytes {
        region 'middle_bytes {
          region {
            let first = write_once(output: &uniq 'out out, source: &'first_bytes first_bytes, start: 0_u64, end: 1_u64);
            let middle = write_once(output: &uniq 'err err, source: &'middle_bytes middle_bytes, start: 0_u64, end: 1_u64);
            let last = write_once(output: &uniq 'out out, source: &last_bytes, start: 0_u64, end: 1_u64);
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const REUSED_OUTPUT_EDGE_CASE: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, out, files), writes(cwd, out, files), allocates(heap) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            let first_bytes = buffer_new(1_u64, 65_u8);
            let last_bytes = buffer_new(1_u64, 67_u8);
            region 'state {
              let permit = reserve_file(factory: &uniq files);
              region 'out {
                region 'first_bytes {
                  region {
                    let first = write_once(output: &uniq 'out out, source: &'first_bytes first_bytes, start: 0_u64, end: 1_u64);
                    let middle = open_read(permit: move permit, root: &'state cwd, path: &'state path);
                    let last = write_once(output: &uniq 'out out, source: &last_bytes, start: 0_u64, end: 1_u64);
                  }
                }
              }
            }
            return exit_status(code: 0_u8);
          }
          Err(error: problem) => {
            return exit_status(code: 201_u8);
          }
        }
      }
      Err(error: problem) => {
        return exit_status(code: 202_u8);
      }
    }
  }
}
"#;

const BLOCKING_OPEN_AND_MARKER: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stderr as err: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, err, files), writes(cwd, err, files), allocates(heap) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            let marker = buffer_new(1_u64, 77_u8);
            region 'c {
              region 'p {
                region 'err {
                  region {
                    let permit = reserve_file(factory: &uniq 'c files);
                    let opened = open_read(permit: move permit, root: &'c cwd, path: &'p path);
                    let announced = write_once(output: &uniq 'err err, source: &marker, start: 0_u64, end: 1_u64);
                  }
                }
              }
            }
            return exit_status(code: 0_u8);
          }
          Err(error: problem) => {
            return exit_status(code: 201_u8);
          }
        }
      }
      Err(error: problem) => {
        return exit_status(code: 202_u8);
      }
    }
  }
}
"#;

const DIRECT_NONREGULAR_OPEN: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region {
              let permit = reserve_file(factory: &uniq files);
              match open_read(permit: move permit, root: &cwd, path: &path) {
                Ok(value: file) => {
                  return exit_status(code: 1_u8);
                }
                Err(error: problem) => {
                  return exit_status(code: 0_u8);
                }
              }
            }
          }
          Err(error: problem) => {
            return exit_status(code: 2_u8);
          }
        }
      }
      Err(error: problem) => {
        return exit_status(code: 3_u8);
      }
    }
  }
}
"#;

const COMPLETION_NONREGULAR_OPEN: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stderr as err: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, err, files), writes(cwd, err, files), allocates(heap) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            let marker = buffer_new(1_u64, 77_u8);
            region 'state {
              let permit = reserve_file(factory: &uniq files);
              region 'marker {
                region {
                  let opened = open_read(permit: move permit, root: &'state cwd, path: &'state path);
                  let announced = write_once(output: &uniq err, source: &'marker marker, start: 0_u64, end: 1_u64);
                  match move opened {
                    Ok(value: file) => {
                      return exit_status(code: 1_u8);
                    }
                    Err(error: problem) => {
                      return exit_status(code: 0_u8);
                    }
                  }
                }
              }
            }
          }
          Err(error: problem) => {
            return exit_status(code: 2_u8);
          }
        }
      }
      Err(error: problem) => {
        return exit_status(code: 3_u8);
      }
    }
  }
}
"#;

const INDEPENDENT_COMPONENT_OPENS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let first_name = buffer_new(1_u64, 46_u8);
  let second_name = buffer_new(1_u64, 46_u8);
  region 'c {
    let first_permit = reserve_file(factory: &uniq files);
    let second_permit = reserve_file(factory: &uniq files);
    region {
      let first = open_directory(permit: move first_permit, root: &'c cwd, name: &first_name, start: 0_u64, end: 1_u64);
      let second = open_directory(permit: move second_permit, root: &'c cwd, name: &second_name, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_DIRECTORY_SOURCE_OPENS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region {
    let first_permit = reserve_file(factory: &uniq files);
    let second_permit = reserve_file(factory: &uniq files);
    let first = open_directory_source(permit: move first_permit, directory: &cwd);
    let second = open_directory_source(permit: move second_permit, directory: &cwd);
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_REGULAR_FILE_OPENS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let first_name = buffer_new(1_u64, 120_u8);
  let second_name = buffer_new(1_u64, 120_u8);
  region 'c {
    let first_permit = reserve_file(factory: &uniq files);
    let second_permit = reserve_file(factory: &uniq files);
    region {
      let first = open_file(permit: move first_permit, root: &'c cwd, name: &first_name, start: 0_u64, end: 1_u64);
      let second = open_file(permit: move second_permit, root: &'c cwd, name: &second_name, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_DIRECTORY_READS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let first_bytes = buffer_new(4096_u64, 0_u8);
  let second_bytes = buffer_new(4096_u64, 0_u8);
  region {
    let first_permit = reserve_file(factory: &uniq files);
    let second_permit = reserve_file(factory: &uniq files);
    match open_directory_source(permit: move first_permit, directory: &cwd) {
      Ok(value: first_list) => {
        match open_directory_source(permit: move second_permit, directory: &cwd) {
          Ok(value: second_list) => {
            region {
              let first = directory_next(source: &uniq first_list, destination: &uniq first_bytes, start: 0_u64, end: 4096_u64);
              let second = directory_next(source: &uniq second_list, destination: &uniq second_bytes, start: 0_u64, end: 4096_u64);
            }
            return exit_status(code: 0_u8);
          }
          Err(error: problem) => {
            return exit_status(code: 201_u8);
          }
        }
      }
      Err(error: problem) => {
        return exit_status(code: 202_u8);
      }
    }
  }
}
"#;

const EMPTY_WRITE: &[u8] = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region 'out {
    region {
      let empty = write_once(output: &uniq 'out out, source: &bytes, start: 0_u64, end: 0_u64);
      match move empty {
        Ok(value: next) => {
          if next == 0_u64 {
          } else {
            return exit_status(code: 211_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 212_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

/// A permitted completion window that is the last thing a `match` arm does.
///
/// The arm's exit edge is emitted from the window's completion join block, not
/// from the arm's own `bbN` header, so the join block's phis have to name that
/// block. Nothing else in this corpus puts a hand-out in a block whose
/// successor carries block parameters.
const OVERLAP_BEFORE_A_BLOCK_JOIN: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let first = buffer_new(2_u64, 65_u8);
  let second = buffer_new(2_u64, 66_u8);
  region 'o {
    region {
      match write_once(output: &uniq 'o out, source: &first, start: 0_u64, end: 2_u64) {
        Ok(value: written) => {
          let a = write_once(output: &uniq 'o out, source: &first, start: 0_u64, end: 2_u64);
          let b = write_once(output: &uniq 'o err, source: &second, start: 0_u64, end: 2_u64);
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const PURE_COMPUTE: &[u8] = br#"fn choose(value: own u64) -> result: own u64 pure {
  return imax(value, value);
}

command fn main() -> status: own ExitStatus pure {
  let first = choose(value: 1_u64);
  let second = choose(value: 2_u64);
  let total = imax(first, second);
  return exit_status(code: 0_u8);
}
"#;

const COMPUTE_AND_IO: &[u8] = br#"fn choose(value: own u64) -> result: own u64 pure {
  return imax(value, value);
}

command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let left = choose(value: 1_u64);
  let right = choose(value: 2_u64);
  let total = imax(left, right);
  let bytes = buffer_new(2_u64, 65_u8);
  set bytes[1_u64] = 66_u8;
  region 'out {
    region 'err {
      region {
        let first = write_once(output: &uniq 'out out, source: &bytes, start: 0_u64, end: 1_u64);
        let second = write_once(output: &uniq 'err err, source: &bytes, start: 1_u64, end: 2_u64);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const BOUNDED_BATCH_OPENS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let opened = 0_u64;
  let name = buffer_new(4_u64, 97_u8);
  for @scan (index in 0_u64..12_u64) {
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set opened = opened +wrap 1_u64;
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  match cvt::<u64, u8>(opened) {
    Ok(value: code) => {
      return exit_status(code: code);
    }
    Err(error: overflowed) => {
      return exit_status(code: 255_u8);
    }
  }
}
"#;

const ONE_SLOT_STAGED_OPEN: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let name = buffer_new(4_u64, 97_u8);
  for @scan (index in 0_u64..1_u64) {
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const ODD_BATCH_WITH_DISTINCT_PATHS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let opened = 0_u64;
  let names = buffer_new(5_u64, 97_u8);
  set names[1_u64] = 98_u8;
  set names[2_u64] = 99_u8;
  set names[3_u64] = 100_u8;
  set names[4_u64] = 101_u8;
  for @scan (index in 0_u64..5_u64) {
    let end = index + 1_u64;
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &names, start: index, end: end) {
          Ok(value: handle) => {
            set opened = opened +wrap 1_u64;
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  match cvt::<u64, u8>(opened) {
    Ok(value: code) => {
      return exit_status(code: code);
    }
    Err(error: overflowed) => {
      return exit_status(code: 255_u8);
    }
  }
}
"#;

fn more_than_target_capacity_reads(count: usize) -> Vec<u8> {
    let mut source = String::from(
        "command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files), allocates(heap) {\n  region {\n    match arg_get(args: &args, position: 1_u64) {\n      Ok(value: text) => {\n        match relative_path(value: move text) {\n          Ok(value: path) => {\n            region 'c {\n              region {\n                let permit = reserve_file(factory: &uniq 'c files);\n                match open_read(permit: move permit, root: &'c cwd, path: &path) {\n                  Ok(value: file) => {\n",
    );
    for index in 0..count {
        source.push_str(&format!(
            "                    let bytes{index} = buffer_new(1_u64, 0_u8);\n"
        ));
    }
    source.push_str("                    region {\n                      region {\n");
    for index in 0..count {
        source.push_str(&format!(
            "                        let read{index} = read_at(file: &file, destination: &uniq bytes{index}, file_offset: 0_u64, start: 0_u64, end: 1_u64);\n"
        ));
    }
    source.push_str(
        "                      }\n                    }\n                    return exit_status(code: 0_u8);\n                  }\n                  Err(error: problem) => {\n                    return exit_status(code: 201_u8);\n                  }\n                }\n              }\n            }\n          }\n          Err(error: problem) => {\n            return exit_status(code: 202_u8);\n          }\n        }\n      }\n      Err(error: problem) => {\n        return exit_status(code: 203_u8);\n      }\n    }\n  }\n}\n",
    );
    source.into_bytes()
}

fn emit_windows_completion(source: &[u8]) -> String {
    let target = SystemTarget::for_triple("x86_64-pc-windows-msvc")
        .expect("the native Windows completion target");
    with_mutated_completion_ir(source, |program| {
        emit_llvm_for_target(program, target)
            .expect("the Windows completion probe must emit")
            .into_string()
    })
}

#[test]
fn windows_completion_modules_require_the_native_runtime_at_link_time() {
    let module = emit_windows_completion(POSITIONED_READS);

    assert!(crate::module_requires_completion_runtime(&module));
    for declaration in [
        "declare i32 @wf__completion_file_pread_submit(i32, ptr, i64, i64, ptr)",
        "declare i32 @wf__completion_file_open_at_submit(i32, ptr, i32, i32, i32, i32, i32, ptr)",
        "declare void @wf__completion_file_join(ptr, ptr, ptr)",
        "declare void @wf__completion_wait_core_capacity()",
    ] {
        assert!(
            module.contains(declaration),
            "Windows completion must name the native ABI `{declaration}`:\n{module}"
        );
    }
    assert!(
        !module.contains("define weak i32 @wf__completion_file_read_submit"),
        "a missing Windows runtime must be a link error, not a direct backend"
    );
    assert!(
        !module.contains("define weak void @wf__completion_file_join"),
        "Windows joins must not resolve to an empty optional-runtime body"
    );
}

#[test]
fn windows_core_pressure_materializes_the_oldest_owned_result_and_retries() {
    let source = more_than_target_capacity_reads(3);
    let module = emit_windows_completion(&source);
    let body = emitted_function(&module, "main");
    let submissions = body
        .matches("call i32 @wf__completion_file_pread_submit")
        .count();

    assert_eq!(submissions, 2, "the source-last read remains direct");
    assert_eq!(
        body.matches(" = icmp eq i32 ").count(),
        submissions * 3,
        "each submit distinguishes DIRECT_ONLY, ACCEPTED, and WAIT_CORE_CAPACITY"
    );
    assert_eq!(
        body.matches(", 2\n  br i1 ").count(),
        submissions,
        "status 2 must have its own branch at every submit"
    );
    assert_eq!(
        body.matches("call void @wf__completion_wait_core_capacity()")
            .count(),
        submissions,
        "every site has a no-owned-token capacity wait before retry"
    );
    assert_eq!(
        body.matches("completion.capacity.consume.").count(),
        2,
        "the second submit has one consume label and one branch to it"
    );
    assert_eq!(
        body.matches("call void @wf__completion_file_join").count(),
        3,
        "two source joins plus one pressure-path materialization consume each token at most once"
    );

    let lines = body.lines().collect::<Vec<_>>();
    let wait_verdicts = lines
        .windows(3)
        .filter(|window| {
            window[0].trim().starts_with("completion.verdict.wait.")
                && window[0].trim().ends_with(':')
        })
        .map(|window| window[2])
        .collect::<Vec<_>>();
    assert_eq!(wait_verdicts.len(), submissions);
    for branch in wait_verdicts {
        assert!(branch.contains("label %completion.capacity."), "{branch}");
        assert!(
            branch.contains("label %completion.verdict.invalid."),
            "{branch}"
        );
        assert!(
            !branch.contains("completion.inline."),
            "core pressure must never become direct execution: {branch}"
        );
    }

    let consume = body
        .split_once("\ncompletion.capacity.consume.")
        .expect("the second submit can consume the first request")
        .1
        .split_once("\ncompletion.capacity.next.")
        .expect("the consume arm rejoins the owner scan")
        .0;
    assert!(consume.contains("call void @wf__completion_file_join"));
    let mapped = consume
        .find("@wf.sys.read.completion(")
        .expect("the pressure path maps the raw result");
    let stored = consume[mapped..]
        .find("\n  store ")
        .map(|offset| mapped + offset)
        .expect("the pressure path stores the typed result");
    let cleared = consume
        .find("store i1 false")
        .expect("the pressure path relinquishes target ownership");
    assert!(mapped < stored && stored < cleared, "{consume}");
    assert!(consume.contains("br label %completion.submit."));
    assert!(
        body.matches("call void @abort()").count() >= submissions,
        "an unknown runtime verdict must abort"
    );
}

/// The logical field pointers of the one target-planned frame in `body`.
///
/// The first GEP level must be derived from the exact struct type allocated at
/// `%wf.frame`; this is the materialized connection between the target layout
/// plan and each operation emitter's semantic slot.
fn planned_function_frame_fields(body: &str) -> Vec<String> {
    let allocations = body
        .lines()
        .filter(|line| line.contains(" = alloca "))
        .collect::<Vec<_>>();
    assert_eq!(
        allocations.len(),
        1,
        "a function with completion storage must have one planned frame:\n{body}"
    );
    let declaration = allocations[0]
        .trim()
        .strip_prefix("%wf.frame = alloca ")
        .unwrap_or_else(|| panic!("the allocation is not the planned function frame:\n{body}"));
    let (frame_type, alignment) = declaration
        .rsplit_once(", align ")
        .unwrap_or_else(|| panic!("the planned frame has no selected-target alignment:\n{body}"));
    assert!(
        frame_type.starts_with("{ ") && frame_type.ends_with(" }"),
        "the planned frame is not one complete struct: {frame_type}"
    );
    let alignment = alignment
        .parse::<u64>()
        .expect("the planned frame alignment is a decimal integer");
    assert!(alignment.is_power_of_two());

    let field_prefix = format!("getelementptr inbounds {frame_type}, ptr %wf.frame, i32 0, i32 ");
    let fields = body
        .lines()
        .filter_map(|line| {
            let (pointer, address) = line.trim().split_once(" = ")?;
            pointer.starts_with("%wf.slot.").then(|| {
                assert!(
                    address.starts_with(&field_prefix),
                    "planned slot {pointer} is not a field of the allocated frame:\n{line}"
                );
                pointer.to_owned()
            })
        })
        .collect::<Vec<_>>();
    assert!(
        !fields.is_empty(),
        "the planned frame exposes no logical field pointers:\n{body}"
    );
    let mut unique = fields.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        fields.len(),
        "a planned field is defined twice"
    );
    fields
}

/// The second GEP level for completion storage with exactly `slots` elements:
/// `(planned frame field pointer, chosen element index)`.
fn completion_element_accesses(body: &str, slots: u64) -> Vec<(String, String)> {
    let array_prefix = format!("[{slots} x ");
    body.lines()
        .filter_map(|line| {
            let (_, address) = line.trim().split_once(" = getelementptr inbounds ")?;
            if !address.starts_with(&array_prefix) {
                return None;
            }
            let (_, addressed) = address.split_once(", ptr ")?;
            let (field, index) = addressed.split_once(", i64 0, i64 ")?;
            field
                .starts_with("%wf.slot.")
                .then(|| (field.to_owned(), index.to_owned()))
        })
        .collect()
}

fn assert_planned_completion_frame(body: &str, slots: u64) {
    let fields = planned_function_frame_fields(body);
    let accesses = completion_element_accesses(body, slots);
    assert!(
        !accesses.is_empty(),
        "the planned frame has no K={slots} completion element GEP:\n{body}"
    );
    for (field, index) in &accesses {
        assert!(
            fields.contains(field),
            "completion element address starts outside the planned frame: {field}"
        );
        if slots == 1 {
            assert_eq!(index, "0", "a K=1 site must select its sole element");
        } else {
            assert!(
                index.starts_with('%'),
                "a K={slots} site must use the driver-provided dynamic index, got {index}"
            );
        }
    }
}

#[test]
fn only_an_actualized_target_operation_selects_the_completion_runtime() {
    let sequential = emit_lowered(INDEPENDENT_WRITES, crate::OverlapLowering::Off);
    let completion = emit(INDEPENDENT_WRITES);
    let pure_sequential = emit_lowered(PURE_COMPUTE, crate::OverlapLowering::Off);
    let pure = emit(PURE_COMPUTE);

    assert!(crate::module_requires_completion_runtime(&sequential));
    assert!(crate::module_requires_completion_runtime(&completion));
    assert!(!crate::module_requires_completion_runtime(&pure));
    assert!(!crate::module_requires_writer_scheduler(&sequential));
    assert!(!crate::module_requires_writer_scheduler(&completion));
    assert!(!crate::module_requires_writer_scheduler(&pure));
    assert!(sequential.contains("@wf__completion_file_write_direct"));
    assert!(!sequential.contains("call i32 @wf__completion_file_write_submit"));
    assert!(!pure.contains("wf__completion_"));
    assert_eq!(pure, pure_sequential);
}

#[test]
fn compute_world_selection_does_not_disable_completion_io() {
    let module = super::emit_with_overlap(COMPUTE_AND_IO);
    assert!(crate::module_requires_parallel_runtime(&module));
    assert!(crate::module_requires_completion_runtime(&module));
    assert_eq!(
        module
            .matches("call i32 @wf__completion_file_write_submit")
            .count(),
        2,
        "both compute worlds submit the earlier I/O and keep the source-last call direct"
    );
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    for workers in ["0", "2"] {
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers)
            .env("WF_IO_HELPERS", "1")
            .output()
            .expect("run mixed compute/completion program");
        assert!(output.status.success(), "WF_WORKERS={workers}: {output:?}");
        assert_eq!(output.stdout, b"A", "WF_WORKERS={workers}");
        assert_eq!(output.stderr, b"B", "WF_WORKERS={workers}");
    }
    std::fs::remove_file(executable).expect("remove mixed compute/completion probe");
    std::fs::remove_dir(directory).expect("remove mixed compute/completion directory");
}

#[test]
fn one_slot_submission_crosses_its_edge_before_the_drain_joins_and_dispatches() {
    let (module, feeder, drain) = with_mutated_completion_ir(ONE_SLOT_STAGED_OPEN, |program| {
        let main = program
            .functions()
            .iter()
            .find(|function| function.name() == "main")
            .expect("the command entry must lower");
        let driver = main
            .completion_pipeline()
            .and_then(crate::IrCompletionPipeline::planned_driver)
            .expect("the one-iteration loop must materialize its one-slot driver");
        let module = crate::emit_llvm(program)
            .expect("the one-slot driver must emit")
            .into_string();
        (module, driver.feeder().ordinal(), driver.drain().ordinal())
    });
    let body = emitted_function(&module, "main");
    assert_planned_completion_frame(body, 1);

    let feeder_label = if feeder == 0 {
        "entry".to_owned()
    } else {
        format!("bb{feeder}")
    };
    let drain_label = if drain == 0 {
        "entry".to_owned()
    } else {
        format!("bb{drain}")
    };
    let feeder_start = body
        .find(&format!("{feeder_label}:"))
        .expect("the emitted function must contain the feeder block");
    let drain_start = body
        .find(&format!("\n{drain_label}:"))
        .expect("the emitted function must contain the drain block");
    assert!(feeder_start < drain_start);
    let feeder_body = &body[feeder_start..drain_start];
    let submit = feeder_body
        .find("call i32 @wf__completion_file_open_at_submit")
        .expect("the feeder must submit the open");
    let branch = feeder_body
        .find(&format!("br label %{drain_label}"))
        .expect("the feeder must cross its mandatory edge");
    assert!(submit < branch, "submission must precede the drain edge");
    assert!(
        !feeder_body.contains("call void @wf__completion_file_open_join"),
        "the feeder must not join before crossing its mandatory edge:\n{feeder_body}"
    );

    let drain_tail = &body[drain_start..];
    let drain_end = drain_tail[1..]
        .find("\nbb")
        .map_or(drain_tail.len(), |offset| offset + 1);
    let drain_body = &drain_tail[..drain_end];
    let join = drain_body
        .find("call void @wf__completion_file_open_join")
        .expect("the exact drain block must join the submitted open");
    let dispatch = drain_body[join..]
        .find("switch ")
        .map(|offset| join + offset)
        .expect("the drain must dispatch the joined result");
    assert!(
        join < dispatch,
        "the join must define the result before dispatch"
    );

    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    let output = Command::new(&executable)
        .env("WF_IO_HELPERS", "1")
        .output()
        .expect("run the linked one-slot driver");
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    std::fs::remove_file(executable).expect("remove one-slot executable");
    std::fs::remove_dir(directory).expect("remove one-slot directory");
}

#[test]
fn source_derived_two_slot_batch_links_and_preserves_every_iteration() {
    let sequential = emit_lowered(BOUNDED_BATCH_OPENS, OverlapLowering::Off);
    let batched = emit(BOUNDED_BATCH_OPENS);
    assert!(!sequential.contains("@wf__completion_window("));
    assert!(!sequential.contains("define weak i64 @wf__completion_window"));
    assert!(
        completion_element_accesses(emitted_function(&sequential, "main"), 2).is_empty(),
        "the sequential reference must have no K=2 completion element"
    );
    assert_eq!(
        emitted_function(&batched, "main")
            .matches("call i64 @wf__completion_window(i64 12, i64 0, i64 2)")
            .count(),
        1,
        "one nonempty source loop asks once for a window capped by its two static slots"
    );
    assert!(batched.contains(
        "define weak i64 @wf__completion_window(i64 %span, i64 %slot_bytes, i64 %ceiling) \
         #0 {\nentry:\n  ret i64 1\n}"
    ));
    assert_planned_completion_frame(emitted_function(&batched, "main"), 2);

    let directory = test_directory();
    std::fs::write(directory.join("aaaa"), b"batch completion fixture")
        .expect("create the regular file opened by every iteration");
    let sequential_executable = build_executable(&sequential, &directory);
    for helpers in ["0", "1", "4"] {
        let sequential_status = Command::new(&sequential_executable)
            .current_dir(&directory)
            .env("WF_IO_HELPERS", helpers)
            .status()
            .expect("run the sequential reference");
        assert_eq!(sequential_status.code(), Some(12), "helpers={helpers}");
    }
    std::fs::remove_file(sequential_executable).expect("remove sequential executable");
    let batched_executable = build_executable(&batched, &directory);
    for helpers in ["0", "1", "4"] {
        let batched_status = Command::new(&batched_executable)
            .current_dir(&directory)
            .env("WF_IO_HELPERS", helpers)
            .status()
            .expect("run the two-slot completion batch");
        assert_eq!(batched_status.code(), Some(12), "helpers={helpers}");
    }
    std::fs::remove_file(batched_executable).expect("remove batched executable");
    std::fs::remove_file(directory.join("aaaa")).expect("remove fixture file");
    std::fs::remove_dir(directory).expect("remove completion test directory");
}

/// Windows pressure recovery may inspect a ring element before that iteration
/// takes its submit arm. Its submission-state array must therefore start false
/// in the entry prelude, not on a path through the loop body.
#[test]
fn windows_staged_ring_initializes_submission_state_before_pressure_recovery() {
    let module = emit_windows_completion(BOUNDED_BATCH_OPENS);
    let body = emitted_function(&module, "main");
    let initialized = body
        .find("store [2 x i1] zeroinitializer, ptr ")
        .expect("the submission-state ring is initialized in the entry prelude");
    let submit = body
        .find("call i32 @wf__completion_file_open_at_submit")
        .expect("the source-derived batch submits an open");
    let pressure = body
        .find("completion.capacity.v")
        .expect("Windows emits a capacity-recovery path");
    assert!(
        initialized < submit && initialized < pressure,
        "the state ring starts false before either an accepted submit or a pressure path"
    );
    assert!(
        module.contains("declare void @abort() noreturn"),
        "the invalid Windows submit verdict remains fail-closed"
    );
}

#[test]
fn an_odd_batch_keeps_each_iterations_path_and_result_in_its_own_slot() {
    let sequential = emit_lowered(ODD_BATCH_WITH_DISTINCT_PATHS, OverlapLowering::Off);
    let batched = emit(ODD_BATCH_WITH_DISTINCT_PATHS);
    assert!(batched.contains("call i64 @wf__completion_window(i64 5, i64 0, i64 2)"));
    assert!(batched.contains("getelementptr inbounds [2 x"));

    let directory = test_directory();
    for name in ["a", "b", "c", "d"] {
        std::fs::write(directory.join(name), name.as_bytes()).expect("create one named fixture");
    }
    for (label, module) in [("sequential", sequential), ("batched", batched)] {
        let executable = build_executable(&module, &directory);
        for helpers in ["0", "1", "4"] {
            let status = Command::new(&executable)
                .current_dir(&directory)
                .env("WF_IO_HELPERS", helpers)
                .status()
                .expect("run the distinct-path batch");
            assert_eq!(status.code(), Some(4), "{label}, helpers={helpers}");
        }
        std::fs::remove_file(executable).expect("remove distinct-path executable");
    }
    for name in ["a", "b", "c", "d"] {
        std::fs::remove_file(directory.join(name)).expect("remove one named fixture");
    }
    std::fs::remove_dir(directory).expect("remove distinct-path directory");
}

#[test]
fn a_target_without_native_completion_runs_the_same_batch_one_iteration_at_a_time() {
    let module = with_mutated_completion_ir(BOUNDED_BATCH_OPENS, |program| {
        emit_llvm_for_target(program, SystemTarget::deterministic_test())
            .expect("the deterministic target admits the source-derived batch")
            .into_string()
    });
    assert!(!module.contains("call i64 @wf__completion_window("));
    assert!(
        completion_element_accesses(emitted_function(&module, "main"), 2).is_empty(),
        "the direct target must not materialize a native K=2 completion ring"
    );
    assert!(module.contains("= add i64 0, 1"));
    assert!(module.contains("call ") && module.contains("@wf_test_openat("));
    assert!(!crate::module_requires_completion_runtime(&module));

    let run =
        run_emitted_on_deterministic_host(&module, &HostScript::new().file(b"batch fixture"), &[]);
    assert_eq!(
        run.output.status.code(),
        Some(12),
        "trace was {:?}",
        run.trace()
    );
    assert_eq!(run.attempts("openat"), 12);
}

#[test]
fn the_file_helper_receives_a_typed_request_and_never_a_writer_thunk() {
    let module = emit(INDEPENDENT_WRITES);
    let first = module
        .find("call i32 @wf__completion_file_write_submit")
        .expect("submit the first owned operation");
    let join = module[first..]
        .find("call void @wf__completion_file_join")
        .map(|offset| first + offset)
        .expect("the earlier operation must join after the source-last direct call");
    assert!(first < join);
    assert_eq!(
        module
            .matches("call i32 @wf__completion_file_write_submit")
            .count(),
        1,
        "only a call with later independent work is submitted"
    );
    assert_eq!(
        module
            .matches("call void @wf__completion_file_join")
            .count(),
        1,
        "each submitted operation owns and consumes its own token"
    );
    assert!(!module.contains("wf__completion_file_batch_claim"));
    assert!(!module.contains("submit_reserved"));
    assert!(!module.contains("wf__par_publish_io"));
    assert!(!module.contains("wf__par_thunk_"));
    assert!(!module.contains("wf__completion_output_batch"));
    assert!(!crate::COMPLETION_FILE_ADAPTER_HEADER.contains("(*"));
    assert!(!crate::COMPLETION_BRIDGE_SOURCE.contains("void (*"));
}

#[test]
fn positioned_read_emits_a_checked_typed_pread_request() {
    let module = emit(POSITIONED_READS);
    assert!(crate::module_requires_completion_runtime(&module));
    assert_eq!(
        module
            .matches("call i32 @wf__completion_file_pread_submit")
            .count(),
        1,
        "the final positioned read keeps the direct specialization"
    );
    assert!(module.contains("%offset.fits = icmp ule i64 %file_offset"));
    assert!(module.contains("9223372036854775807"));
    assert!(module.contains("call i64 @wf__completion_file_pread_direct(i32"));
    assert!(crate::COMPLETION_BRIDGE_SOURCE.contains("request.kind = WF_FILE_PREAD"));
    assert!(crate::COMPLETION_BRIDGE_SOURCE.contains("file_offset > (uint64_t)INT64_MAX"));
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let pread = bridge
        .split_once("int wf__completion_file_pread_submit(")
        .expect("the bridge exposes positioned read")
        .1
        .split_once("int wf__completion_file_write_submit(")
        .expect("positioned read precedes write")
        .0;
    assert!(
        pread.find("wf_bridge_submit_linux_pread") < pread.find("request.kind = WF_FILE_PREAD"),
        "Linux must try native completion before constructing the typed fallback"
    );
    let write = bridge
        .split_once("int wf__completion_file_write_submit(")
        .expect("the bridge exposes write_once")
        .1
        .split_once("int wf__completion_file_open_at_submit(")
        .expect("ordinary write submit precedes the open submission")
        .0;
    assert!(!write.contains("wf_bridge_submit_linux"));
    assert!(write.contains("current-position and"));
    let direct_write = bridge
        .split_once("int64_t wf__completion_file_write_direct(")
        .expect("the bridge exposes direct write progress")
        .1
        .split_once("int wf__completion_file_open_at_direct(")
        .expect("direct write precedes the direct open")
        .0;
    assert!(!direct_write.contains("wf_bridge_submit_linux"));
    // A direct call executes the typed request through the bridge's own
    // executor, which is what enters it in the process-wide retirement ledger
    // for as long as it runs.
    assert!(direct_write.contains("wf_bridge_execute_direct"));
    // Inside that executor the host attempt is timed rather than plain, so
    // the adapter's measurement of what its own operations cost keeps
    // running on the route it may have declined a submission in favour of.
    assert!(bridge.contains("wf_file_execute_timed(&wf_bridge_adapter"));
    assert!(crate::COMPLETION_LINUX_IO_URING_HEADER.contains("#if defined(__linux__)"));
    assert!(
        crate::COMPLETION_LINUX_IO_URING_SOURCE
            .contains("wf_linux_io_uring_translation_unit_is_target_guarded")
    );

    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    let status = Command::new(&executable)
        .env("WF_IO_HELPERS", "0")
        .status()
        .expect("run the positioned-read module");
    assert!(status.success());
    std::fs::remove_file(executable).expect("remove positioned-read module");
    std::fs::remove_dir(directory).expect("remove positioned-read directory");
}

#[test]
fn one_empty_write_uses_the_normal_direct_operation_path() {
    let module = emit(EMPTY_WRITE);
    assert!(module.contains("%empty = icmp eq i64 %extent, 0"));
    assert!(module.contains("label %vacant, label %nonempty"));
    assert!(!module.contains("wf__completion_file_batch_claim"));
    assert!(!module.contains("wf__completion_output_batch"));
    assert!(!module.contains("submit_reserved"));

    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    for helpers in ["0", "1", "4"] {
        let output = Command::new(&executable)
            .env("WF_IO_HELPERS", helpers)
            .output()
            .expect("run empty write operation");
        assert!(
            output.status.success(),
            "WF_IO_HELPERS={helpers}: {output:?}"
        );
        assert!(output.stdout.is_empty(), "WF_IO_HELPERS={helpers}");
    }
    std::fs::remove_file(executable).expect("remove empty-write probe");
    std::fs::remove_dir(directory).expect("remove empty-write directory");
}

/// A block whose terminator follows a completion hand-out exits from the
/// hand-out's join block, and the successor's phis must say so.
///
/// The emitter writes a block's phis before it writes the blocks that reach
/// it, so an incoming edge is named by predicting the label its predecessor
/// will end at. Every construct that opens a further LLVM block moves that
/// label; a direct completion hand-out does too, and naming the plain `bbN`
/// header instead produced a module `clang` rejects outright. Building the
/// executable is the assertion: an invalid module never links.
#[test]
fn a_completion_window_before_a_block_join_names_its_join_block() {
    for lowering in [OverlapLowering::Completion, OverlapLowering::On] {
        let module = emit_lowered(OVERLAP_BEFORE_A_BLOCK_JOIN, lowering);
        let joined = emitted_function(&module, "main");
        // The `match` join block's phis are the ones naming the `Err` arm's
        // plain block; the other incoming edge is the overlapped arm's.
        let block_parameters = joined
            .lines()
            .filter(|line| line.contains(" = phi ") && line.contains(", %bb"))
            .collect::<Vec<_>>();
        assert!(
            !block_parameters.is_empty(),
            "{lowering:?}: the join block carries block parameters"
        );
        for phi in block_parameters {
            assert!(
                phi.contains("%par.done."),
                "{lowering:?}: the arm's edge leaves the completion join block: {phi}"
            );
        }

        let directory = test_directory();
        let executable = build_executable(&module, &directory);
        for helpers in ["0", "1", "4"] {
            let output = Command::new(&executable)
                .env("WF_IO_HELPERS", helpers)
                .output()
                .expect("run the overlapped window before a block join");
            assert!(
                output.status.success(),
                "{lowering:?} WF_IO_HELPERS={helpers}: {output:?}"
            );
            assert_eq!(
                output.stdout, b"AAAA",
                "{lowering:?} WF_IO_HELPERS={helpers}"
            );
            assert_eq!(output.stderr, b"BB", "{lowering:?} WF_IO_HELPERS={helpers}");
        }
        std::fs::remove_file(executable).expect("remove block-join probe");
        std::fs::remove_dir(directory).expect("remove block-join directory");
    }
}
/// The compiler-owned C units compile in the host compiler's default dialect,
/// not only in the `-std=c11` the repository gate names.
///
/// The driver names `-std=c11` for its own link, so this is not about which
/// dialect ships; it is about the units not *depending* on the pin. A GNU
/// dialect predefines object-like macros — `linux`, `unix` — whose spellings
/// are ordinary identifiers in C11, and a member named for one of them
/// compiles under the gate and fails everywhere else. The gate never noticed,
/// because the gate only ever compiled these units one way.
///
/// Host-limited by construction: this checks the units the host actually
/// preprocesses, so a macOS run leaves the `__linux__` bodies unchecked.
/// Batch 0085 ran the same units on Linux under Docker for that half.
#[test]
fn the_compiler_owned_c_units_compile_in_the_default_dialect() {
    let directory = test_directory();
    let units: [(&str, &str); 12] = [
        ("contract.h", crate::COMPLETION_CONTRACT_HEADER),
        ("file_adapter.h", crate::COMPLETION_FILE_ADAPTER_HEADER),
        ("bridge.h", crate::COMPLETION_BRIDGE_HEADER),
        ("writer_scheduler.h", crate::WRITER_SCHEDULER_HEADER),
        ("linux_io_uring.h", crate::COMPLETION_LINUX_IO_URING_HEADER),
        ("runtime.c", crate::COMPLETION_RUNTIME_SOURCE),
        ("file_adapter.c", crate::COMPLETION_FILE_ADAPTER_SOURCE),
        ("bridge.c", crate::COMPLETION_BRIDGE_SOURCE),
        ("writer_scheduler.c", crate::WRITER_SCHEDULER_SOURCE),
        ("linux_io_uring.c", crate::COMPLETION_LINUX_IO_URING_SOURCE),
        ("floor.c", crate::FLOOR_RUNTIME_SOURCE),
        (
            "par_completion.c",
            crate::PARALLEL_COMPLETION_RUNTIME_SOURCE,
        ),
    ];
    for (name, source) in units {
        std::fs::write(directory.join(name), source).expect("write compiler-owned C unit");
    }
    for (name, _) in units {
        if !name.ends_with(".c") {
            continue;
        }
        let checked = Command::new("/usr/bin/clang")
            .arg("-fsyntax-only")
            .arg("-pthread")
            .arg("-I")
            .arg(&directory)
            .arg("-x")
            .arg("c")
            .arg(directory.join(name))
            .output()
            .expect("invoke host clang");
        assert!(
            checked.status.success(),
            "{name} needs a dialect the shipped link may not select:\n{}",
            String::from_utf8_lossy(&checked.stderr)
        );
    }
    for (name, _) in units {
        std::fs::remove_file(directory.join(name)).expect("remove compiler-owned C unit");
    }
    std::fs::remove_dir(directory).expect("remove the default-dialect directory");
}

#[test]
fn completion_slots_and_writer_ready_cells_have_one_capacity_source() {
    let header = crate::WRITER_SCHEDULER_HEADER;
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let scheduler = crate::WRITER_SCHEDULER_SOURCE;

    assert_eq!(
        header
            .matches("#define WF_COMPLETION_SLOT_CAPACITY 64u")
            .count(),
        1,
        "the bounded process runtime must name its capacity once"
    );
    assert!(header.contains("#define WF_WRITER_READY_CAPACITY WF_COMPLETION_SLOT_CAPACITY"));
    assert!(bridge.contains("#define WF_BRIDGE_OPERATION_CAPACITY WF_COMPLETION_SLOT_CAPACITY"));
    assert!(bridge.contains("WF_BRIDGE_SLOT_COUNT == WF_WRITER_READY_CAPACITY"));
    assert!(!bridge.contains("#define WF_BRIDGE_OPERATION_CAPACITY 64u"));
    assert!(!scheduler.contains("#define WF_WRITER_READY_COUNT"));
    assert!(scheduler.contains("wf_writer_ready[WF_WRITER_READY_CAPACITY]"));
    assert!(scheduler.contains("wf_writer_count == WF_WRITER_READY_CAPACITY"));
    assert!(
        include_str!("../completion/harness.c")
            .contains("#define WF_HARNESS_OPERATION_CAPACITY WF_COMPLETION_SLOT_CAPACITY")
    );
}

#[test]
fn linux_native_wait_unifies_cq_compute_and_capacity_without_polling() {
    let adapter = crate::COMPLETION_LINUX_IO_URING_SOURCE;
    let contract = crate::COMPLETION_CONTRACT_HEADER;
    let runtime = crate::COMPLETION_RUNTIME_SOURCE;
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;

    assert!(adapter.contains("epoll_create1(EPOLL_CLOEXEC)"));
    assert!(adapter.contains("eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK)"));
    assert!(adapter.contains("EPOLL_CTL_ADD"));
    assert!(adapter.contains("adapter->ring_descriptor"));
    assert!(adapter.contains("wf_completion_wake_epoch(adapter->runtime) != observed_epoch"));
    assert!(adapter.contains("wf_linux_drain_wake_descriptor"));
    assert!(adapter.contains("errno == EAGAIN"));
    assert!(contract.contains("wf_completion_set_wake_callback"));

    let notify = runtime
        .split_once("static void wf_completion_notify_scheduler")
        .expect("completion core has one scheduler announcer")
        .1
        .split_once("static enum wf_completion_publish_result")
        .expect("announcer precedes publication")
        .0;
    let parked = notify
        .find("parked_schedulers")
        .expect("host wake is conditional on an announced sleeper");
    let callback = notify
        .find("runtime->wake_callback(runtime->wake_context)")
        .expect("announced native sleeper receives the unified wake");
    assert!(parked < callback);

    let progress = bridge
        .split_once("static int wf_bridge_progress(void)")
        .expect("bridge has bounded progress")
        .1
        .split_once("void wf__writer_scheduler_notify")
        .expect("progress precedes writer notification")
        .0;
    assert!(progress.contains("int error = wf_linux_io_uring_progress"));
    assert!(progress.contains("if (error != 0)"));
    assert!(progress.contains("abort();"));
    assert!(!progress.contains("(void)wf_linux_io_uring_progress"));
    assert!(!bridge.contains("wf_completion_park_if_unchanged(\n                    &wf_bridge_runtime,\n                    epoch,\n                    1u"));
}

/// How long a probe child's expected output may take to arrive before the case
/// calls it stuck.
///
/// Both cases that wait on a child prove an *order*, not a latency, and in both
/// the failing behaviour produces the bytes never rather than late:
///
/// - `independent_io_reaches_the_second_operation_before_the_first_unblocks`
///   reads nothing from the child's stdout until the marker has arrived, so the
///   child's one-megabyte write to that pipe is blocked for the whole wait and a
///   marker byte at any point in it is proof that the second operation ran while
///   the first was outstanding.
/// - `reused_output_progress_preserves_ac_around_an_independent_rejected_open`
///   fails when C is serialized behind an open that does not come back, and
///   then `AC` never appears at all.
///
/// The bound is therefore a liveness cut-off in both, and making it generous
/// weakens neither assertion.
///
/// It used to be three seconds and five, which are inside the scheduling delay
/// a loaded host produces: each case spawns a child and waits for one of its
/// threads to be scheduled, and between them they failed on five separate gate
/// runs across three people, every one on a host running more than one compiler
/// gate at once, while passing every time in isolation — the second of them in
/// under a second, three orders of magnitude inside its own bound. Each such
/// failure reported a regression that had not happened. Sixty seconds is far
/// outside any scheduling delay and still bounded, so the regression each case
/// exists to catch fails the run rather than hanging it.
const PROBE_OUTPUT_LIMIT: Duration = Duration::from_secs(60);

#[test]
fn independent_io_reaches_the_second_operation_before_the_first_unblocks() {
    let module = emit(INDEPENDENT_WRITES);
    // `None` is the shipped default: no WF_IO_HELPERS in the environment at
    // all, which selects the demand-driven policy rather than any pinned
    // count. It belongs in this loop rather than in a test of its own so that
    // only one probe is ever blocked on a full pipe at a time.
    for helpers in [Some("1"), Some("0"), Some("4"), None] {
        let helpers = helpers.unwrap_or("unset");
        let directory = test_directory();
        let executable = build_executable(&module, &directory);
        let mut command = Command::new(&executable);
        if helpers == "unset" {
            command.env_remove("WF_IO_HELPERS");
        } else {
            command.env("WF_IO_HELPERS", helpers);
        }
        let mut child = command
            .env("WF_WORKERS", "0")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run completion overlap probe");
        let mut stderr = child.stderr.take().expect("capture marker output");
        let (send, receive) = mpsc::channel();
        let marker_reader = std::thread::spawn(move || {
            let mut marker = [0_u8; 1];
            let read = stderr.read_exact(&mut marker);
            let _ = send.send((read, marker));
        });

        let (read, marker) = receive
            .recv_timeout(PROBE_OUTPUT_LIMIT)
            .unwrap_or_else(|_| {
                let _ = child.kill();
                panic!(
                    "the independent marker write did not run while the first pipe write was blocked (WF_IO_HELPERS={helpers})"
                )
            });
        read.expect("read the independent marker");
        assert_eq!(marker, [b'M']);

        let mut stdout = child.stdout.take().expect("capture bulk output");
        let mut bytes = Vec::new();
        stdout
            .read_to_end(&mut bytes)
            .expect("drain the blocked pipe");
        let status = child.wait().expect("wait for completion overlap probe");
        marker_reader.join().expect("join marker reader");
        assert!(status.success(), "completion probe exited with {status}");
        assert!(!bytes.is_empty());

        std::fs::remove_file(executable).expect("remove completion probe");
        std::fs::remove_dir(directory).expect("remove completion probe directory");
    }
}

#[test]
fn a_reused_unique_output_waits_only_for_its_own_prior_operation() {
    let module = emit(REUSED_OUTPUT_AROUND_INDEPENDENT_OUTPUT);
    let body = module
        .split_once("@wf_main(")
        .expect("command entry is emitted")
        .1;
    let submits = body
        .match_indices("call i32 @wf__completion_file_write_submit")
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    let joins = body
        .match_indices("call void @wf__completion_file_join")
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    let direct_calls = body
        .match_indices("@wf.sys.write_once.v1(")
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    assert_eq!(
        submits.len(),
        2,
        "A and B submit; source-last C stays direct"
    );
    assert_eq!(joins.len(), 2, "each submitted operation is consumed once");
    assert_eq!(
        direct_calls.len(),
        3,
        "each path retains its direct fallback"
    );
    assert!(
        submits[0] < submits[1]
            && submits[1] < joins[0]
            && joins[0] < direct_calls[2]
            && direct_calls[2] < joins[1],
        "C must wait for A, then run while unrelated B remains pending"
    );
}

#[test]
fn reused_output_progress_preserves_ac_around_an_independent_rejected_open() {
    let module = emit(REUSED_OUTPUT_EDGE_CASE);
    let directory = test_directory();
    let fifo = directory.join("unrelated-open");
    let created = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("create A/B/C FIFO");
    assert!(created.success());
    let executable = build_executable(&module, &directory);
    let mut child = Command::new(&executable)
        .current_dir(&directory)
        .arg("unrelated-open")
        .env("WF_IO_HELPERS", "2")
        .env("WF_WORKERS", "0")
        .stdout(Stdio::piped())
        .spawn()
        .expect("run A/B/C dependency probe");
    let mut stdout = child.stdout.take().expect("capture A/C output");
    let (send, receive) = mpsc::channel();
    let stdout_reader = std::thread::spawn(move || {
        let mut observed = [0_u8; 2];
        let read = stdout.read_exact(&mut observed);
        let _ = send.send((read, observed));
    });
    let (read, observed) = receive
        .recv_timeout(PROBE_OUTPUT_LIMIT)
        .unwrap_or_else(|_| {
            let _ = child.kill();
            panic!("C(out) waited for the independent rejected B(open)")
        });
    read.expect("read A/C output");
    assert_eq!(observed, *b"AC");

    let status = child.wait().expect("wait for A/B/C probe");
    stdout_reader.join().expect("join A/C reader");
    assert!(status.success(), "{status}");
    std::fs::remove_file(executable).expect("remove A/B/C probe");
    std::fs::remove_file(fifo).expect("remove A/B/C FIFO");
    std::fs::remove_dir(directory).expect("remove A/B/C directory");
}

#[test]
fn more_than_sixty_four_calls_progress_by_falling_back_only_the_full_call() {
    let source = more_than_target_capacity_reads(66);
    let module = emit(&source);
    assert_eq!(
        module
            .matches("call i32 @wf__completion_file_pread_submit")
            .count(),
        65,
        "every call with later work attempts its own submission"
    );

    let directory = test_directory();
    let input = directory.join("capacity-input");
    std::fs::write(&input, b"x").expect("write capacity probe input");
    let executable = build_executable(&module, &directory);
    let output = Command::new(&executable)
        .current_dir(&directory)
        .arg("capacity-input")
        .env("WF_IO_HELPERS", "0")
        .output()
        .expect("run more-than-capacity completion probe");
    assert!(output.status.success(), "{output:?}");
    std::fs::remove_file(executable).expect("remove capacity probe");
    std::fs::remove_file(input).expect("remove capacity input");
    std::fs::remove_dir(directory).expect("remove capacity directory");
}

#[test]
fn a_rejected_nonregular_open_does_not_delay_independent_writer_work() {
    let module = emit(BLOCKING_OPEN_AND_MARKER);
    assert!(module.contains("call i32 @wf__completion_file_open_at_submit"));
    let directory = test_directory();
    let fifo = directory.join("blocking-open");
    let created = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("create blocking-open FIFO");
    assert!(created.success());
    let executable = build_executable(&module, &directory);

    for helpers in ["0", "1"] {
        let output = Command::new(&executable)
            .current_dir(&directory)
            .arg("blocking-open")
            .env("WF_IO_HELPERS", helpers)
            .output()
            .expect("run nonregular-open completion probe");
        assert!(
            output.status.success(),
            "WF_IO_HELPERS={helpers}: {output:?}"
        );
        assert_eq!(output.stderr, b"M");
    }

    std::fs::remove_file(executable).expect("remove blocking-open probe");
    std::fs::remove_file(fifo).expect("remove blocking-open FIFO");
    std::fs::remove_dir(directory).expect("remove blocking-open directory");
}

#[test]
fn direct_and_completion_open_read_reject_a_fifo_without_blocking() {
    let direct = emit_lowered(DIRECT_NONREGULAR_OPEN, crate::OverlapLowering::Off);
    let completion = emit(COMPLETION_NONREGULAR_OPEN);
    assert!(!direct.contains("@wf__completion_file_open_at_submit"));
    assert!(completion.contains("call i32 @wf__completion_file_open_at_submit"));
    assert!(completion.contains("i32 1, ptr %"));

    let directory = test_directory();
    let fifo = directory.join("nonregular");
    let created = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("create nonregular-open FIFO");
    assert!(created.success());
    for (name, module, marker) in [
        ("direct", &direct, b"".as_slice()),
        ("completion", &completion, b"M".as_slice()),
    ] {
        let executable = build_executable(module, &directory);
        for helpers in ["0", "1"] {
            let output = Command::new(&executable)
                .current_dir(&directory)
                .arg("nonregular")
                .env("WF_IO_HELPERS", helpers)
                .output()
                .unwrap_or_else(|error| panic!("run {name} FIFO rejection: {error}"));
            assert!(
                output.status.success(),
                "{name}, WF_IO_HELPERS={helpers}: {output:?}"
            );
            assert_eq!(output.stderr, marker);
        }
        std::fs::remove_file(executable).expect("remove FIFO rejection probe");
    }
    std::fs::remove_file(fifo).expect("remove nonregular FIFO");
    std::fs::remove_dir(directory).expect("remove nonregular directory");
}

#[test]
fn component_directory_open_uses_the_same_typed_completion_route() {
    let module = emit(INDEPENDENT_COMPONENT_OPENS);
    assert_eq!(
        module
            .matches("call i32 @wf__completion_file_open_at_submit")
            .count(),
        1
    );
    assert!(module.contains("completion.component.scan"));
    assert!(module.contains("i32 2, ptr %"));
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    for helpers in ["0", "1"] {
        let output = Command::new(&executable)
            .current_dir(&directory)
            .env("WF_IO_HELPERS", helpers)
            .output()
            .expect("run component-open completion probe");
        assert!(
            output.status.success(),
            "WF_IO_HELPERS={helpers}: {output:?}"
        );
    }
    std::fs::remove_file(executable).expect("remove component-open probe");
    std::fs::remove_dir(directory).expect("remove component-open directory");
}

#[test]
fn windows_component_completion_stages_one_terminated_utf16_name() {
    let module = emit_windows_completion(INDEPENDENT_COMPONENT_OPENS);
    assert!(
        module.contains("i32 2, i32 2, ptr %"),
        "a component directory result must be registered as DIRECTORY_ROOT: {module}"
    );
    let component = module
        .split_once("completion.component.entry.")
        .expect("the Windows completion route validates the component")
        .1
        .split_once("call i32 @wf__completion_file_open_at_submit")
        .expect("the validated component reaches the typed submit")
        .0;

    assert!(component.contains("load i16"), "{component}");
    assert!(component.contains("icmp eq i16"), "{component}");
    assert!(component.contains("add i64") && component.contains(", 2\n"));
    assert!(component.contains("store i16 0"), "{component}");
    assert!(
        !component.contains("store i8 0"),
        "a one-byte terminator can expose an uninitialized UTF-16 high byte: {component}"
    );
}

#[test]
fn directory_source_open_uses_the_typed_completion_route() {
    let module = emit(INDEPENDENT_DIRECTORY_SOURCE_OPENS);
    assert_eq!(
        module
            .matches("call i32 @wf__completion_file_open_at_submit")
            .count(),
        1
    );
    assert!(module.contains("@wf.sys.open_directory_source.completion"));
    assert!(module.contains("i32 2, ptr %"));
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    for helpers in ["0", "1"] {
        let output = Command::new(&executable)
            .current_dir(&directory)
            .env("WF_IO_HELPERS", helpers)
            .output()
            .expect("run directory-source completion probe");
        assert!(
            output.status.success(),
            "WF_IO_HELPERS={helpers}: {output:?}"
        );
    }
    std::fs::remove_file(executable).expect("remove directory-source probe");
    std::fs::remove_dir(directory).expect("remove directory-source directory");
}

#[test]
fn regular_file_open_maps_status_and_release_after_completion() {
    let module = emit(INDEPENDENT_REGULAR_FILE_OPENS);
    assert_eq!(
        module
            .matches("call i32 @wf__completion_file_open_at_submit")
            .count(),
        1
    );
    assert!(module.contains("@wf.sys.open_file.completion"));
    assert!(module.contains("@wf__completion_file_close_direct"));
    assert!(module.contains("i32 1, ptr %"));
    // The kind decision reads the mode of the descriptor the open produced.
    // It moved out of this adapter into the one shared rule every target
    // answers with, so the adapter now calls that rule with its own fstat.
    assert!(crate::COMPLETION_FILE_ADAPTER_HEADER.contains("S_ISREG(file_mode)"));
    assert!(crate::COMPLETION_FILE_ADAPTER_SOURCE.contains("fstat(descriptor, &status)"));
    assert!(crate::COMPLETION_FILE_ADAPTER_SOURCE.contains(
        "wf_file_kind_outcome(\n                request->operation.open_at.expected_kind,"
    ));
    let directory = test_directory();
    let input = directory.join("x");
    std::fs::write(&input, b"regular").expect("write regular-file probe input");
    let executable = build_executable(&module, &directory);
    for helpers in ["0", "1"] {
        let output = Command::new(&executable)
            .current_dir(&directory)
            .env("WF_IO_HELPERS", helpers)
            .output()
            .expect("run regular-file open completion probe");
        assert!(
            output.status.success(),
            "WF_IO_HELPERS={helpers}: {output:?}"
        );
    }
    std::fs::remove_file(executable).expect("remove regular-file probe");
    std::fs::remove_file(input).expect("remove regular-file input");
    std::fs::remove_dir(directory).expect("remove regular-file directory");
}

#[test]
fn directory_enumeration_completes_before_writer_normalization() {
    let module = emit(INDEPENDENT_DIRECTORY_READS);
    assert_eq!(
        module
            .matches("call i32 @wf__completion_directory_next_submit")
            .count(),
        1
    );
    assert!(module.contains("@wf.sys.directory_next.completion"));
    let directory = test_directory();
    std::fs::write(directory.join("visible-entry"), b"x")
        .expect("write directory completion fixture");
    let executable = build_executable(&module, &directory);
    for helpers in ["0", "1"] {
        let output = Command::new(&executable)
            .current_dir(&directory)
            .env("WF_IO_HELPERS", helpers)
            .output()
            .expect("run directory completion probe");
        assert!(
            output.status.success(),
            "WF_IO_HELPERS={helpers}: {output:?}"
        );
    }
    std::fs::remove_file(executable).expect("remove directory completion probe");
    std::fs::remove_file(directory.join("visible-entry"))
        .expect("remove directory completion fixture");
    std::fs::remove_dir(directory).expect("remove directory completion directory");
}

/// A scheduler waiting for a completion must sleep rather than spin whenever
/// some other thread owns the work it is waiting for.
///
/// The park guard used to refuse to park while the target queue held anything
/// at all. With helpers that is a busy wait for exactly as long as a helper
/// keeps the queue non-empty, because `wf_bridge_progress` deliberately does
/// not let a waiting scheduler execute an unrelated queued request when
/// helpers exist — so the loop spins, makes no progress, and refuses to sleep.
/// Measured on a four-wide many-file program, the one-helper configuration
/// burned about 270 ms of user CPU against 71 ms for the same program at four
/// helpers. Only the zero-helper configuration, where the waiting scheduler
/// really is the target's engine, may refuse to park.
#[test]
fn a_waiting_scheduler_parks_unless_it_is_itself_the_target_engine() {
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let predicate = bridge
        .split_once("static int wf_bridge_target_work_needs_this_thread(void) {")
        .expect("the park guard is one named predicate")
        .1
        .split_once('}')
        .expect("the predicate has a body")
        .0;
    assert!(
        predicate.contains("wf_file_adapter_helper_count"),
        "the guard must read the helper count: {predicate}"
    );
    assert!(
        predicate.contains("wf_file_adapter_queued"),
        "the guard must read the queue: {predicate}"
    );
    // Every join's park decision goes through the predicate, so no site may
    // still spell the old queue-only condition.
    assert!(
        !bridge.contains("|| wf_file_adapter_queued(&wf_bridge_adapter) == 0)"),
        "a join still refuses to park on a non-empty queue alone"
    );
    assert_eq!(
        bridge
            .matches("!wf_bridge_target_work_needs_this_thread()")
            .count(),
        3,
        "each of the three joins asks the same question"
    );
}

/// A positioned read the submitting thread would run itself is not submitted.
///
/// The completion path exists so a program is not stalled by a wait it could
/// have overlapped. When the bounded adapter holds no helper, has nothing
/// queued, and has measured its own operations as not waiting, the submitted
/// read would be executed by the submitting thread anyway — at its join, after
/// a queue crossing, a claim, four slot transitions and a drain. On the
/// `macos-14` runner that machinery is about 400 ns against a warm 4 KiB read
/// of about 1.2 us, which is why the eight-wide warm program cost 41.78 ms
/// with the pool off against 32.80 ms for the sequential build of the same
/// source. Declining the submission leaves the caller the ordinary direct call
/// the emitter already emits for a refused one.
///
/// Two limits are what make it safe rather than merely fast.
///
/// Only a *positioned* transfer is declined. An offset is meaningful only on a
/// seekable object and the typed opens that produce one admit nothing but a
/// regular file, so a positioned read waits on storage. A non-positioned read
/// or write may be waiting on something another part of the same program has
/// to do, and running one where it was stated could stall the thread that
/// would unblock it — which is exactly what
/// `independent_io_reaches_the_second_operation_before_the_first_unblocks`
/// pins, and it writes to a pipe.
///
/// And a written `WF_IO_HELPERS` declines nothing. It pins the route with the
/// count, which is what makes a pinned line of a measurement a measurement of
/// the completion path rather than of the policy that may decline it.
#[test]
fn a_positioned_read_the_submitting_thread_would_run_itself_is_not_submitted() {
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let adapter = crate::COMPLETION_FILE_ADAPTER_SOURCE;

    let rule = bridge
        .split_once("static int wf_bridge_positioned_read_runs_on_caller(uint64_t count) {")
        .expect("one rule decides whether a positioned read is declined")
        .1
        .split_once("\n}\n")
        .expect("the rule ends with the function")
        .0;
    assert!(
        rule.contains("count != 0"),
        "a read of nothing makes no host call and is left alone: {rule}"
    );
    assert!(
        rule.contains("wf_bridge_helpers_pinned == 0"),
        "a written helper count declines nothing: {rule}"
    );
    assert!(
        rule.contains("wf_file_adapter_transfer_runs_on_caller(&wf_bridge_adapter)"),
        "the adapter answers whether the submitting thread would run it: {rule}"
    );

    // Exactly one submission entry point asks, and it is the positioned one.
    assert_eq!(
        bridge
            .matches("wf_bridge_positioned_read_runs_on_caller(count)")
            .count(),
        1,
        "only the positioned read may be declined"
    );
    let pread = bridge
        .split_once("int wf__completion_file_pread_submit(")
        .expect("the bridge exposes positioned read")
        .1
        .split_once("int wf__completion_file_write_submit(")
        .expect("positioned read precedes write")
        .0;
    assert!(
        pread.find("wf_bridge_positioned_read_runs_on_caller(count)")
            < pread.find("request.kind = WF_FILE_PREAD"),
        "the decision comes before anything is claimed: {pread}"
    );
    assert!(
        pread.find("wf_bridge_submit_linux_pread")
            < pread.find("wf_bridge_positioned_read_runs_on_caller(count)"),
        "a native completion path is tried before the bounded adapter's rule"
    );

    // The adapter's half: no helper, nothing queued, and a measured
    // non-wait — never the absence of a measurement.
    let answer = adapter
        .split_once("int wf_file_adapter_transfer_runs_on_caller(const wf_file_adapter *adapter) {")
        .expect("the adapter answers in one place")
        .1
        .split_once("\n}\n")
        .expect("the answer ends with the function")
        .0;
    assert!(answer.contains("!= WF_FILE_WAIT_SHORT"));
    assert!(answer.contains("wf_file_adapter_helper_count(adapter) != 0"));
    assert!(answer.contains("wf_file_adapter_queued(adapter) == 0"));
    let verdict = adapter
        .split_once("enum wf_file_wait_verdict wf_file_adapter_wait_verdict(")
        .expect("one verdict function")
        .1
        .split_once("\n}\n")
        .expect("the verdict ends with the function")
        .0;
    assert!(
        verdict.contains("return WF_FILE_WAIT_UNMEASURED;"),
        "an adapter that has executed nothing must say so: {verdict}"
    );
}

/// The helper count is target policy, and an unset `WF_IO_HELPERS` must ask
/// for helpers on evidence rather than on principle.
///
/// A written setting still pins the count exactly, which is what every test
/// that names `0`, `1`, or `4` depends on. Unset asks for the measured policy,
/// and batch 0096 changed what that policy is on both of its ends.
///
/// It starts at **none**. Starting at one gave every program a thread handoff
/// whether or not it had a wait to overlap, and on a warm page cache that
/// handoff was the whole cost of the completion path: on the `macos-14`
/// runner the eight-wide 4 KiB read program cost 41.78 ms with the pool off
/// against 89.19 ms at one helper and 96.10 ms at the old default, with the
/// sequential build at 32.80 ms. With no helper the waiting scheduler is the
/// queue's own engine and the path costs a queue crossing.
///
/// Its ceiling is the bridge's operation bound rather than the machine's core
/// count. A helper inside a host call holds no CPU, so what limits useful I/O
/// concurrency is how many operations a program can have outstanding. Sizing
/// the pool by cores capped the same three-core runner at three outstanding
/// reads for a program that states eight: cold, that program cost 938.79 ms at
/// the core-sized default against 585.58 ms at eight helpers and 433.57 ms for
/// a native pool of eight.
///
/// Growth needs two facts, not one. Queue depth at the moment of an enqueue
/// says the program stated width; it does not say the width is over anything.
/// The adapter's measured host-call time is the other half, and only both
/// together add a helper. A rule that instead grew whenever a submission found
/// no helper *waiting* was tried and measured worse: a helper that has been
/// signalled but not yet scheduled still counts as waiting, so a run of
/// consecutive submissions sees an available helper every time and the pool
/// never grows at all. On the quiet macOS host that left the four-wide program
/// at 919 ms against 625 ms for the depth rule. Queue depth is a lagging
/// signal, but it is a true one.
#[test]
fn an_unset_helper_setting_selects_a_bounded_demand_driven_pool() {
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let adapter = crate::COMPLETION_FILE_ADAPTER_SOURCE;
    let policy = bridge
        .split_once("static void wf_bridge_helper_policy(size_t *initial, size_t *cap) {")
        .expect("the helper policy is one function")
        .1
        .split_once("\nstatic ")
        .expect("the policy ends before the next definition")
        .0;
    // A written value pins both ends, so growth cannot move it.
    assert!(policy.contains("*initial = (size_t)parsed;"));
    assert!(policy.contains("*cap = (size_t)parsed;"));
    // Unset starts with no helper and lets the operation bound, not the core
    // count, be the ceiling.
    assert!(
        policy.contains("*initial = 0u;"),
        "an unset setting must start with no helper: {policy}"
    );
    assert!(
        policy.contains("*cap = WF_BRIDGE_MAX_HELPERS;"),
        "the ceiling is the bridge's operation bound: {policy}"
    );
    assert!(
        !policy.contains("(size_t)online < WF_BRIDGE_MAX_HELPERS"),
        "the core count must not cap outstanding I/O: {policy}"
    );

    let growth = adapter
        .split_once("static void wf_file_grow_helpers_locked(wf_file_adapter *adapter) {")
        .expect("growth is one named function")
        .1
        .split_once("\n}\n")
        .expect("growth ends with the function")
        .0;
    assert!(
        growth.contains("held >= adapter->helper_cap"),
        "growth must stop at the cap: {growth}"
    );
    assert!(
        growth.contains("adapter->queue_count <= held"),
        "growth must require a queue that has outrun the pool: {growth}"
    );
    assert!(
        growth.contains("wf_file_adapter_wait_verdict(adapter) != WF_FILE_WAIT_LONG"),
        "growth must also require a measured wait to overlap: {growth}"
    );
    // Growth runs inside the one enqueue that already holds the queue lock,
    // so it creates at most one helper per submission and needs no second
    // lock, and every kind of queued work reaches the pool the same way.
    let enqueue = adapter
        .split_once("static int wf_file_enqueue_locked(")
        .expect("one place appends an accepted queue entry")
        .1
        .split_once("\n}\n")
        .expect("the enqueue ends with the function")
        .0;
    assert!(
        enqueue.contains("wf_file_grow_helpers_locked(adapter)"),
        "the enqueue is where growth happens: {enqueue}"
    );
    // One queued request wakes one helper, never every helper, only a helper
    // that is actually asleep, and never from inside the queue lock: a signal
    // issued under that lock wakes a helper whose next act is to block on it.
    assert!(
        enqueue.contains("wake = adapter->blocked_helpers != 0;"),
        "the enqueue decides the wake under the lock: {enqueue}"
    );
    assert!(
        !enqueue.contains("pthread_cond_signal"),
        "the enqueue must not issue the wake while it holds the lock: {enqueue}"
    );
    let submit = adapter
        .split_once("enum wf_file_submit_result wf_file_adapter_submit(")
        .expect("one submission entry point")
        .1
        .split_once("\n}\n")
        .expect("the submission ends with the function")
        .0;
    let unlock = submit
        .find("(void)pthread_mutex_unlock(&adapter->queue_lock);\n    if (wake != 0) {")
        .expect("the wake follows the unlock");
    let signal = submit
        .find("pthread_cond_signal(&adapter->queue_available)")
        .expect("a submission announces to exactly one helper");
    assert!(unlock < signal, "the wake is issued outside the queue lock");
    assert!(
        !submit.contains("pthread_cond_broadcast(&adapter->queue_available)"),
        "a submission must not wake every helper"
    );
    // The bridge owns no second helper pool layered over this one.
    assert!(
        !bridge.contains("wf_bridge_target_helpers"),
        "the bridge must not keep a helper pool of its own"
    );
}

/// Where a native completion path exists, an open and a close are operations
/// on it, and one rule decides the open's typed outcome on every target.
///
/// While the ring carried only transfers, every open cost a blocking `openat`
/// on the submitting scheduler: in the zero-helper configuration a native ring
/// selects, that is the one thread which could otherwise run any other ready
/// frame, so a slow path resolution stalls all of them. The path resolution is
/// what the ring now carries.
///
/// The kind check that makes an open typed — refusing a FIFO, refusing a
/// directory — stays a host call on the reaping thread, and that placement is
/// measured rather than preferred. Written first as a linked `IORING_OP_STATX`
/// of the same descriptor, it kept the reaping thread free of host calls and
/// cost two ring round trips per open: on the two-CPU Linux container the
/// eight-wide many-file program ran 152 ms against 116 ms for the bounded
/// adapter it replaced. Done as one `fstat` of an already-open descriptor —
/// an inode read that cannot wait on anything — the same program runs 119 ms.
///
/// The refusal rule itself lives once, so a FIFO is refused identically
/// whether the open ran on a helper thread, on the scheduler, or in the
/// kernel.
#[test]
fn a_native_ring_carries_opens_and_closes_under_one_kind_rule() {
    let ring = crate::COMPLETION_LINUX_IO_URING_SOURCE;
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let adapter_header = crate::COMPLETION_FILE_ADAPTER_HEADER;

    for opcode in ["IORING_OP_OPENAT", "IORING_OP_CLOSE"] {
        assert!(
            ring.contains(opcode),
            "the ring adapter must submit {opcode}"
        );
    }
    // The kind decision is one rule, stated once, called by both adapters.
    assert!(
        adapter_header.contains("wf_file_kind_outcome("),
        "the open-kind rule belongs to the shared typed file contract"
    );
    assert!(
        ring.contains("wf_file_kind_outcome("),
        "the ring adapter must answer with the shared open-kind rule"
    );
    assert!(
        crate::COMPLETION_FILE_ADAPTER_SOURCE.contains("wf_file_kind_outcome("),
        "the bounded POSIX adapter must answer with the shared open-kind rule"
    );
    // The part of an open that can wait is the path resolution, and no
    // scheduler thread may perform that: it is a ring operation or nothing.
    let decision = ring
        .split_once("static void wf_linux_decide_open(")
        .expect("the open decision is one named function")
        .1
        .split_once("\n}\n")
        .expect("the open decision ends with the function")
        .0;
    assert!(
        !decision.contains("openat("),
        "an open's path resolution belongs to the ring: {decision}"
    );
    assert!(
        !ring.contains("submission->opcode = IORING_OP_STATX"),
        "the kind check is one fstat of an open descriptor, not a second ring \
         round trip that measured 31 percent slower"
    );
    assert!(
        decision.contains("fstat(entry->opened_descriptor"),
        "the kind check reads the mode of the descriptor the open produced: \
         {decision}"
    );
    // The bridge offers both operations to the ring before the bounded
    // fallback, and answers -1 rather than claiming an operation it cannot
    // then hand over.
    for route in [
        "wf_bridge_submit_linux_open_at(",
        "wf_bridge_submit_linux_close(",
    ] {
        assert!(bridge.contains(route), "the bridge must offer {route}");
    }
    let open_submit = bridge
        .split_once("int wf__completion_file_open_at_submit(")
        .expect("one open submission entry point")
        .1;
    let native = open_submit
        .find("wf_bridge_submit_linux_open_at(")
        .expect("the open tries the ring");
    let fallback = open_submit
        .find("request.kind = WF_FILE_OPEN_AT;")
        .expect("the open keeps its bounded fallback");
    assert!(
        native < fallback,
        "the ring is tried before the bounded POSIX adapter"
    );
}

/// No C unit the compiler links may spell an identifier the host compiler
/// predefines as a macro.
///
/// `whitefootc` compiles these units with the host compiler's default
/// dialect, which is a GNU dialect, and that dialect predefines `linux` and
/// `unix` as `1`. A union member spelled `linux` therefore made every Linux
/// link of a completion program fail to compile — not a test, a link, so no
/// macOS run could see it. The units are compiled here as they are shipped,
/// so the rule is about the shipped bytes rather than about a probe's flags.
#[test]
fn linked_c_units_avoid_identifiers_the_host_compiler_predefines() {
    for (name, source) in [
        ("bridge.c", crate::COMPLETION_BRIDGE_SOURCE),
        ("runtime.c", crate::COMPLETION_RUNTIME_SOURCE),
        ("file_adapter.c", crate::COMPLETION_FILE_ADAPTER_SOURCE),
        ("linux_io_uring.c", crate::COMPLETION_LINUX_IO_URING_SOURCE),
        ("writer_scheduler.c", crate::WRITER_SCHEDULER_SOURCE),
        ("contract.h", crate::COMPLETION_CONTRACT_HEADER),
        ("bridge.h", crate::COMPLETION_BRIDGE_HEADER),
        ("file_adapter.h", crate::COMPLETION_FILE_ADAPTER_HEADER),
        ("linux_io_uring.h", crate::COMPLETION_LINUX_IO_URING_HEADER),
        ("writer_scheduler.h", crate::WRITER_SCHEDULER_HEADER),
    ] {
        for reserved in ["linux", "unix"] {
            for shape in [format!(" {reserved};"), format!(".{reserved}")] {
                assert!(
                    !source.contains(&shape),
                    "{name} spells `{shape}`, and the host compiler's default \
                     dialect predefines `{reserved}` as a macro"
                );
            }
        }
    }
}

/// The same two independent writes, differing only in whether the second call
/// is written as a `let` right-hand side or directly as a `match` scrutinee.
///
/// A call is a call in either position: both programs perform the same two
/// operations in the same order, publish the same bytes, and are judged by the
/// same [PAR-1] pair. Before the schedule saw scrutinee calls, the second
/// program had no pair at all — one candidate is not a window — so its first
/// write was never handed out and the two spellings compiled to different
/// work for no semantic reason.
const SCRUTINEE_TAIL_LET_FORM: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  doc "Two independent writes whose second call is bound before it is matched.";
  let bulk = buffer_new(1_u64, 65_u8);
  let marker = buffer_new(1_u64, 77_u8);
  region 'out {
    region 'err {
      region 'bulk {
        region {
          let first = write_once(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1_u64);
          let second = write_once(output: &uniq 'err err, source: &marker, start: 0_u64, end: 1_u64);
          match second {
            Ok(value: written) => {
            }
            Err(error: problem) => {
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const SCRUTINEE_TAIL_MATCH_FORM: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  doc "Two independent writes whose second call is written in scrutinee position.";
  let bulk = buffer_new(1_u64, 65_u8);
  let marker = buffer_new(1_u64, 77_u8);
  region 'out {
    region 'err {
      region 'bulk {
        region {
          let first = write_once(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1_u64);
          match write_once(output: &uniq 'err err, source: &marker, start: 0_u64, end: 1_u64) {
            Ok(value: written) => {
            }
            Err(error: problem) => {
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

/// The same second call written as the scrutinee of a *value* match, whose
/// binding names the match's result rather than the call's.
///
/// `CheckedStatement::Match` and `CheckedStatement::ValueMatchLet` are two
/// statements with one scrutinee expression between them, and the judgment
/// reaches the call through the scrutinee in both. This form is the one where
/// a binding exists and is the wrong identity for the site — `written` is what
/// the arms give, not what `write_once` returned — which is why the site's
/// identity had to become the call occurrence.
const SCRUTINEE_VALUE_MATCH_FORM: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  doc "Two independent writes whose second call is a value match's scrutinee.";
  let bulk = buffer_new(1_u64, 65_u8);
  let marker = buffer_new(1_u64, 77_u8);
  region 'out {
    region 'err {
      region 'bulk {
        region {
          let first = write_once(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1_u64);
          let written = match write_once(output: &uniq 'err err, source: &marker, start: 0_u64, end: 1_u64) {
            Ok(value: count) => {
              give count;
            }
            Err(error: problem) => {
              give 0_u64;
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

/// The scrutinee call written *first*, with an independent call after it.
///
/// This one must stay sequential, and not by accident: the match's own
/// dispatch and the arm it selects read the call's result, so every statement
/// after the match already stands behind that read. Handing the scrutinee call
/// out would run the second write before the first write's arms.
const SCRUTINEE_HEAD_MATCH_FORM: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  doc "A scrutinee call followed by an independent call, which cannot overlap.";
  let bulk = buffer_new(1_u64, 65_u8);
  let marker = buffer_new(1_u64, 77_u8);
  region 'out {
    region 'err {
      region 'bulk {
        region {
          match write_once(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1_u64) {
            Ok(value: written) => {
            }
            Err(error: problem) => {
            }
          }
          let second = write_once(output: &uniq 'err err, source: &marker, start: 0_u64, end: 1_u64);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

/// The submissions, joins, and direct calls of one emitted command entry.
fn completion_write_shape(module: &str) -> (usize, usize, usize) {
    let body = module
        .split_once("@wf_main(")
        .expect("command entry is emitted")
        .1;
    (
        body.matches("call i32 @wf__completion_file_write_submit")
            .count(),
        body.matches("call void @wf__completion_file_join").count(),
        body.matches("@wf.sys.write_once.v1(").count(),
    )
}

/// Runs one emitted module in every helper configuration and returns nothing
/// but the assurance that each run published the same two bytes.
fn assert_publishes_marked_streams(module: &str) {
    for helpers in ["0", "1", "4"] {
        let directory = test_directory();
        let executable = build_executable(module, &directory);
        let output = Command::new(&executable)
            .env("WF_IO_HELPERS", helpers)
            .env("WF_WORKERS", "0")
            .output()
            .expect("run the scrutinee-position probe");
        assert!(
            output.status.success(),
            "probe exited with {} at WF_IO_HELPERS={helpers}",
            output.status
        );
        assert_eq!(output.stdout, b"A", "WF_IO_HELPERS={helpers}");
        assert_eq!(output.stderr, b"M", "WF_IO_HELPERS={helpers}");
        std::fs::remove_file(executable).expect("remove the probe");
        std::fs::remove_dir(directory).expect("remove the probe directory");
    }
}

#[test]
fn a_call_in_scrutinee_position_is_handed_out_exactly_as_a_bound_call_is() {
    let bound = emit(SCRUTINEE_TAIL_LET_FORM);
    let scrutinee = emit(SCRUTINEE_TAIL_MATCH_FORM);
    assert_eq!(
        completion_write_shape(&bound),
        (1, 1, 2),
        "the bound form submits the first write and leaves the second direct"
    );
    assert_eq!(
        completion_write_shape(&scrutinee),
        completion_write_shape(&bound),
        "a call written in scrutinee position is the same call"
    );
    assert_publishes_marked_streams(&bound);
    assert_publishes_marked_streams(&scrutinee);
}

/// A value match's scrutinee is the same call in the same position, and the
/// binding its statement defines is not the call's result.
///
/// This is the second of the two statement forms fix 1 opened, and it is the
/// one that shows why a site is identified by its call occurrence: the
/// statement here *does* define a binding, and taking that binding for the
/// site's identity would name the value the arms give rather than the value
/// the call returned.
#[test]
fn a_value_match_scrutinee_call_is_handed_out_exactly_as_a_bound_call_is() {
    let bound = emit(SCRUTINEE_TAIL_LET_FORM);
    let value_match = emit(SCRUTINEE_VALUE_MATCH_FORM);
    assert_eq!(
        completion_write_shape(&value_match),
        completion_write_shape(&bound),
        "a call in a value match's scrutinee is the same call"
    );
    assert_publishes_marked_streams(&value_match);
}

#[test]
fn a_scrutinee_call_before_an_independent_call_stays_sequential() {
    let module = emit(SCRUTINEE_HEAD_MATCH_FORM);
    assert_eq!(
        completion_write_shape(&module),
        (0, 0, 2),
        "the match's own arms read the first result, so nothing may overtake it"
    );
    assert_publishes_marked_streams(&module);
}

/// Completion storage is part of one selected-target function frame.
///
/// The first GEP selects each logical completion field from `%wf.frame`; the
/// second selects element zero from that field's one-element array. This
/// ordinary non-loop schedule executes each static site once, so K=1 is its
/// exact capacity. The source-derived bounded-batch test above checks the same
/// two-level address path with K=2 and a driver-provided dynamic index.
#[test]
fn completion_storage_uses_one_planned_frame_and_k1_elements() {
    let module = emit(POSITIONED_READS);
    let body = emitted_function(&module, "probe");
    assert_planned_completion_frame(body, 1);
}
