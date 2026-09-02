use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

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
        region 'marker {
          let first = write_once<'out, 'bulk>(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1048576_u64);
          let second = write_once<'err, 'marker>(output: &uniq 'err err, source: &'marker marker, start: 0_u64, end: 1_u64);
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
      region 'right {
        let first = read_at<'file, 'left>(file: &'file file, destination: &uniq 'left left, file_offset: 0_u64, start: 0_u64, end: 1_u64);
        let second = read_at<'file, 'right>(file: &'file file, destination: &uniq 'right right, file_offset: 1_u64, start: 0_u64, end: 1_u64);
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
          region 'last_bytes {
            let first = write_once<'out, 'first_bytes>(output: &uniq 'out out, source: &'first_bytes first_bytes, start: 0_u64, end: 1_u64);
            let middle = write_once<'err, 'middle_bytes>(output: &uniq 'err err, source: &'middle_bytes middle_bytes, start: 0_u64, end: 1_u64);
            let last = write_once<'out, 'last_bytes>(output: &uniq 'out out, source: &'last_bytes last_bytes, start: 0_u64, end: 1_u64);
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const HOSTILE_REUSED_OUTPUT: &[u8] = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, out, files), writes(cwd, out, files), allocates(heap) {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            let first_bytes = buffer_new(1_u64, 65_u8);
            let last_bytes = buffer_new(1_u64, 67_u8);
            region 'state {
              let permit = reserve_file<'state>(factory: &uniq 'state files);
              region 'out {
                region 'first_bytes {
                  region 'last_bytes {
                    let first = write_once<'out, 'first_bytes>(output: &uniq 'out out, source: &'first_bytes first_bytes, start: 0_u64, end: 1_u64);
                    let middle = open_read<'state, 'state>(permit: move permit, root: &'state cwd, path: &'state path);
                    let last = write_once<'out, 'last_bytes>(output: &uniq 'out out, source: &'last_bytes last_bytes, start: 0_u64, end: 1_u64);
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
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            let marker = buffer_new(1_u64, 77_u8);
            region 'c {
              region 'p {
                region 'err {
                  region 'marker {
                    let permit = reserve_file<'c>(factory: &uniq 'c files);
                    let opened = open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path);
                    let announced = write_once<'err, 'marker>(output: &uniq 'err err, source: &'marker marker, start: 0_u64, end: 1_u64);
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
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'state {
              let permit = reserve_file<'state>(factory: &uniq 'state files);
              match open_read<'state, 'state>(permit: move permit, root: &'state cwd, path: &'state path) {
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
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            let marker = buffer_new(1_u64, 77_u8);
            region 'state {
              let permit = reserve_file<'state>(factory: &uniq 'state files);
              region 'marker {
                region 'err {
                  let opened = open_read<'state, 'state>(permit: move permit, root: &'state cwd, path: &'state path);
                  let announced = write_once<'err, 'marker>(output: &uniq 'err err, source: &'marker marker, start: 0_u64, end: 1_u64);
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
    let first_permit = reserve_file<'c>(factory: &uniq 'c files);
    let second_permit = reserve_file<'c>(factory: &uniq 'c files);
    region 'n {
      let first = open_directory<'c, 'n>(permit: move first_permit, root: &'c cwd, name: &'n first_name, start: 0_u64, end: 1_u64);
      let second = open_directory<'c, 'n>(permit: move second_permit, root: &'c cwd, name: &'n second_name, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_DIRECTORY_SOURCE_OPENS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region 'listing {
    let first_permit = reserve_file<'listing>(factory: &uniq 'listing files);
    let second_permit = reserve_file<'listing>(factory: &uniq 'listing files);
    let first = open_directory_source<'listing>(permit: move first_permit, directory: &'listing cwd);
    let second = open_directory_source<'listing>(permit: move second_permit, directory: &'listing cwd);
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_REGULAR_FILE_OPENS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let first_name = buffer_new(1_u64, 120_u8);
  let second_name = buffer_new(1_u64, 120_u8);
  region 'c {
    let first_permit = reserve_file<'c>(factory: &uniq 'c files);
    let second_permit = reserve_file<'c>(factory: &uniq 'c files);
    region 'n {
      let first = open_file<'c, 'n>(permit: move first_permit, root: &'c cwd, name: &'n first_name, start: 0_u64, end: 1_u64);
      let second = open_file<'c, 'n>(permit: move second_permit, root: &'c cwd, name: &'n second_name, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_DIRECTORY_READS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let first_bytes = buffer_new(4096_u64, 0_u8);
  let second_bytes = buffer_new(4096_u64, 0_u8);
  region 'listing {
    let first_permit = reserve_file<'listing>(factory: &uniq 'listing files);
    let second_permit = reserve_file<'listing>(factory: &uniq 'listing files);
    match open_directory_source<'listing>(permit: move first_permit, directory: &'listing cwd) {
      Ok(value: first_list) => {
        match open_directory_source<'listing>(permit: move second_permit, directory: &'listing cwd) {
          Ok(value: second_list) => {
            region 'step {
              let first = directory_next<'step, 'step>(source: &uniq 'step first_list, destination: &uniq 'step first_bytes, start: 0_u64, end: 4096_u64);
              let second = directory_next<'step, 'step>(source: &uniq 'step second_list, destination: &uniq 'step second_bytes, start: 0_u64, end: 4096_u64);
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
    region 'source {
      let empty = write_once<'out, 'source>(output: &uniq 'out out, source: &'source bytes, start: 0_u64, end: 0_u64);
      match move empty {
        Ok(value: next) => {
          if ieq(next, 0_u64) {
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
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s first, start: 0_u64, end: 2_u64) {
        Ok(value: written) => {
          let a = write_once<'o, 's>(output: &uniq 'o out, source: &'s first, start: 0_u64, end: 2_u64);
          let b = write_once<'o, 's>(output: &uniq 'o err, source: &'s second, start: 0_u64, end: 2_u64);
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
      region 'source {
        let first = write_once<'out, 'source>(output: &uniq 'out out, source: &'source bytes, start: 0_u64, end: 1_u64);
        let second = write_once<'err, 'source>(output: &uniq 'err err, source: &'source bytes, start: 1_u64, end: 2_u64);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

fn more_than_target_capacity_reads(count: usize) -> Vec<u8> {
    let mut source = String::from(
        "command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files), allocates(heap) {\n  region 'a {\n    match arg_get<'a>(args: &'a args, position: 1_u64) {\n      Ok(value: text) => {\n        match relative_path(value: move text) {\n          Ok(value: path) => {\n            region 'c {\n              region 'p {\n                let permit = reserve_file<'c>(factory: &uniq 'c files);\n                match open_read<'c, 'p>(permit: move permit, root: &'c cwd, path: &'p path) {\n                  Ok(value: file) => {\n",
    );
    for index in 0..count {
        source.push_str(&format!(
            "                    let bytes{index} = buffer_new(1_u64, 0_u8);\n"
        ));
    }
    source.push_str("                    region 'f {\n                      region 'd {\n");
    for index in 0..count {
        source.push_str(&format!(
            "                        let read{index} = read_at<'f, 'd>(file: &'f file, destination: &uniq 'd bytes{index}, file_offset: 0_u64, start: 0_u64, end: 1_u64);\n"
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
    let module = emit(HOSTILE_REUSED_OUTPUT);
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
        region 'marker {
          let first = write_once<'out, 'bulk>(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1_u64);
          let second = write_once<'err, 'marker>(output: &uniq 'err err, source: &'marker marker, start: 0_u64, end: 1_u64);
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
        region 'marker {
          let first = write_once<'out, 'bulk>(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1_u64);
          match write_once<'err, 'marker>(output: &uniq 'err err, source: &'marker marker, start: 0_u64, end: 1_u64) {
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
        region 'marker {
          let first = write_once<'out, 'bulk>(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1_u64);
          let written = match write_once<'err, 'marker>(output: &uniq 'err err, source: &'marker marker, start: 0_u64, end: 1_u64) {
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
        region 'marker {
          match write_once<'out, 'bulk>(output: &uniq 'out out, source: &'bulk bulk, start: 0_u64, end: 1_u64) {
            Ok(value: written) => {
            }
            Err(error: problem) => {
            }
          }
          let second = write_once<'err, 'marker>(output: &uniq 'err err, source: &'marker marker, start: 0_u64, end: 1_u64);
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

/// Completion storage is reserved as an indexed element of a per-site array
/// rather than as a bare shared slot.
///
/// That is the whole of what this observes, and it is worth stating exactly:
/// every `alloca` in the handed-out probe is a one-element array reached
/// through element zero, which is the count and the index the emitter reserves
/// today. It does *not* observe that two hand-outs of one site would get two
/// elements — one hand-out per site is all the current schedule can express,
/// and the emitter refuses a second outstanding one outright, which
/// `a_second_operation_of_one_completion_site_is_refused` is the evidence for.
/// This case is the guard against regressing to the bare shared allocas that
/// made the staged path bug possible, and against the reserved count silently
/// ceasing to be the one the site's hand-outs need.
#[test]
fn completion_storage_is_reserved_as_an_indexed_element_not_a_bare_slot() {
    let module = emit(POSITIONED_READS);
    let body = emitted_function(&module, "probe");
    let storages = body
        .lines()
        .filter_map(|line| line.trim().strip_prefix("%"))
        .filter_map(|line| line.split_once(" = alloca "))
        .map(|(name, ty)| (name.to_owned(), ty.trim().to_owned()))
        .collect::<Vec<_>>();
    assert!(
        !storages.is_empty(),
        "the handed-out probe allocates completion storage"
    );
    for (name, ty) in &storages {
        assert!(
            ty.starts_with("[1 x "),
            "completion storage %{name} is {ty}, not one element per outstanding operation"
        );
        assert!(
            body.contains(&format!(
                "getelementptr inbounds {ty}, ptr %{name}, i64 0, i64 0"
            )),
            "completion storage %{name} is not reached through its hand-out's index"
        );
    }
}

/// One element per site is sound only while a site holds one outstanding
/// operation, and that precondition is enforced rather than assumed.
///
/// No schedule this lowering forms can hand a site a second operation while
/// its first is in flight: `emit_terminator` joins everything outstanding
/// before it writes any terminator, so a completion hand-out never leaves the
/// block that made it and control cannot reach the site again while the
/// operation is live. The shape therefore has to be injected — one submitted,
/// unfinished completion call emitted twice in place — and what this pins is
/// that the emitter refuses it. Sharing the element instead would let the
/// second operation overwrite a result or a staged path the first is still
/// being read from, with no compile error and no crash, which is exactly the
/// class of defect fix 2 of this batch had to repair.
#[test]
fn a_second_operation_of_one_completion_site_is_refused() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    let accepted = with_mutated_completion_ir(INDEPENDENT_WRITES, |program| {
        emit_llvm_for_target(program, target).is_ok()
    });
    assert!(
        accepted,
        "the unmutated program must emit, or the refusal below proves nothing"
    );
    with_mutated_completion_ir(INDEPENDENT_WRITES, |program| {
        assert!(
            program.duplicate_outstanding_completion_call_for_test(),
            "the probe must have a submitted completion call that does not finish its schedule"
        );
        assert_eq!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::SecondOutstandingCompletionOperation),
            "a second operation of one site must be refused, not given the first's storage"
        );
    });
}

/// Two positioned reads, a loop between the submission and the use, and one
/// early typed exit out of that loop.
///
/// Lowering submits the first read and runs the second inline, which is what
/// it already does for two independent transfers. What the loop adds is a back
/// edge between the submission and the join, and what the early return adds is
/// a second way out of the loop that has to retire the same operation. Those
/// are the two edges a staged schedule has to get right, and neither exists in
/// a body with no loop in it.
const READS_ACROSS_A_LOOP: &[u8] = br#"fn probe(file: own ReadFile, rounds: own u64) -> result: own u64 reads(file), writes(file), allocates(heap) {
  let left = buffer_new(1_u64, 0_u8);
  let right = buffer_new(1_u64, 0_u8);
  let total = 0_u64;
  region 'file {
    region 'left {
      region 'right {
        let first = read_at<'file, 'left>(file: &'file file, destination: &uniq 'left left, file_offset: 0_u64, start: 0_u64, end: 1_u64);
        let second = read_at<'file, 'right>(file: &'file file, destination: &uniq 'right right, file_offset: 1_u64, start: 0_u64, end: 1_u64);
        let cursor = 0_u64;
        loop @spin {
          let done = ieq(cursor, rounds);
          if done {
            break @spin;
          }
          let bail = ieq(cursor, 7_u64);
          if bail {
            return 1_u64;
          }
          set total = total +wrap cursor;
          set cursor = cursor +wrap 1_u64;
        }
        match move first {
          ReadBytes(next: produced) => {
            set total = total +wrap produced;
          }
          ReadEnd() => {
          }
          ReadFailed(error: problem) => {
          }
        }
        match move second {
          ReadBytes(next: produced) => {
            set total = total +wrap produced;
          }
          ReadEnd() => {
          }
          ReadFailed(error: problem) => {
          }
        }
      }
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

/// Which IR block of an emitted function each occurrence of `needle` lands in.
///
/// The emitter opens further labels inside one IR block — a completion
/// submission alone opens four — so the answer is the last *block* label seen
/// and not the last label seen.
fn ir_blocks_containing(function: &str, needle: &str) -> Vec<String> {
    let is_block_label = |label: &str| {
        label == "entry"
            || label.strip_prefix("bb").is_some_and(|ordinal| {
                !ordinal.is_empty() && ordinal.bytes().all(|byte| byte.is_ascii_digit())
            })
    };
    let mut current = String::new();
    let mut found = Vec::new();
    for line in function.lines() {
        match line.strip_suffix(':') {
            Some(label) if is_block_label(label) => current = label.to_owned(),
            _ => {
                if line.contains(needle) {
                    found.push(current.clone());
                }
            }
        }
    }
    found
}

/// The blocks of one function from which the loop's back edge is still ahead:
/// the loop's own blocks and everything that reaches them.
///
/// This is the carrying set the staged judgment will supply, computed here by
/// backward reachability from the block that closes the loop. Only the
/// descriptor is a stand-in; everything the emitter does with it is the
/// shipped path.
fn blocks_that_reach_the_back_edge(function: &crate::IrFunction) -> Vec<crate::IrBlockId> {
    let successors = |block: &crate::IrBlock| -> Vec<crate::IrBlockId> {
        match block.terminator() {
            crate::IrTerminator::Jump { target, .. } => vec![*target],
            crate::IrTerminator::Match { targets, .. } => {
                targets.iter().map(|target| target.block()).collect()
            }
            crate::IrTerminator::Return { .. } | crate::IrTerminator::Unreachable => Vec::new(),
        }
    };
    let closes_the_loop = function
        .blocks()
        .iter()
        .enumerate()
        .position(|(index, block)| {
            successors(block)
                .iter()
                .any(|target| target.index() < index)
        })
        .expect("the probe's loop must close");
    let mut reaches = vec![false; function.blocks().len()];
    reaches[closes_the_loop] = true;
    let mut changed = true;
    while changed {
        changed = false;
        for (index, block) in function.blocks().iter().enumerate() {
            if reaches[index] {
                continue;
            }
            if successors(block)
                .iter()
                .any(|target| reaches[target.index()])
            {
                reaches[index] = true;
                changed = true;
            }
        }
    }
    reaches
        .iter()
        .enumerate()
        .filter(|(_, reached)| **reached)
        .map(|(index, _)| crate::IrBlockId::from_index(index).expect("block ordinal"))
        .collect()
}

/// A staged completion result cannot be consumed before the descriptor's
/// drain.
///
/// This source observes `first` after its loop, while the descriptor would
/// defer its completion until an exit from that loop. The current pipeline
/// record carries target storage and mapper facts but deliberately does not
/// rewrite arbitrary source continuations, so accepting this descriptor would
/// either make an incomplete result observable or create a non-dominating LLVM
/// definition.
#[test]
fn a_staged_loop_that_uses_a_result_before_its_drain_is_refused() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    with_mutated_completion_ir(READS_ACROSS_A_LOOP, |program| {
        let probe = program
            .functions()
            .iter()
            .find(|function| function.name() == "probe")
            .expect("the probe function");
        let carrying = blocks_that_reach_the_back_edge(probe);
        assert!(
            carrying.len() > 1,
            "the probe's loop must span more than one block, or the back edge proves nothing"
        );
        assert!(
            program.set_completion_pipeline_for_test(
                "probe",
                crate::IrCompletionPipeline::new(
                    crate::IrBlockId::from_index(0).expect("the entry block"),
                    carrying,
                    crate::IrCompletionWindow::new(0, 65_536, 32),
                ),
            ),
            "the probe function must take a pipeline"
        );
        assert_eq!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::StagedCompletionResultUse),
            "a descriptor cannot defer a result the source consumes before its drain"
        );
    });
}

/// The weak window answer is emitted only where a module asks for one, and it
/// is one.
#[test]
fn the_window_fallback_is_emitted_only_where_a_module_asks_for_one() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    let sequential = with_mutated_completion_ir(A_STAGED_LOOP_BODY, |program| {
        emit_llvm_for_target(program, target)
            .expect("the probe must emit")
            .into_string()
    });
    assert!(
        sequential.contains("define weak i32 @wf__completion_file_read_submit"),
        "the probe must already carry the completion fallbacks"
    );
    assert!(
        !sequential.contains("define weak i64 @wf__completion_window"),
        "a module that asks for no window must define none"
    );
    let staged = with_mutated_completion_ir(A_STAGED_LOOP_BODY, |program| {
        let probe = program
            .functions()
            .iter()
            .find(|function| function.name() == "probe")
            .expect("the probe function");
        let carrying = blocks_that_reach_the_back_edge(probe);
        assert!(program.set_completion_pipeline_for_test(
            "probe",
            crate::IrCompletionPipeline::new(
                crate::IrBlockId::from_index(0).expect("the entry block"),
                carrying,
                crate::IrCompletionWindow::new(8_192, 65_536, 0),
            ),
        ));
        emit_llvm_for_target(program, target)
            .expect("a staged probe must emit")
            .into_string()
    });
    assert!(
        staged.contains(
            "define weak i64 @wf__completion_window(i64 %span, i64 %slot_bytes, i64 %ceiling) \
             #0 {\nentry:\n  ret i64 1\n}"
        ),
        "a link without the completion unit must answer one, which is the sequential program"
    );
}

/// A carrying region no exit leaves is refused.
///
/// Naming every block leaves no drain: on every path an accepted operation
/// would go unjoined and the target would write its result into storage the
/// frame no longer exists to hold. It is a defect of whatever produced the
/// descriptor, and it is refused before a line of the function is emitted
/// rather than diagnosed by the absence of a join.
#[test]
fn a_carrying_region_with_no_exit_is_refused() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    with_mutated_completion_ir(READS_ACROSS_A_LOOP, |program| {
        let probe = program
            .functions()
            .iter()
            .find(|function| function.name() == "probe")
            .expect("the probe function");
        let every_block = (0..probe.blocks().len())
            .map(|index| crate::IrBlockId::from_index(index).expect("block ordinal"))
            .collect();
        assert!(program.set_completion_pipeline_for_test(
            "probe",
            crate::IrCompletionPipeline::new(
                crate::IrBlockId::from_index(0).expect("the entry block"),
                every_block,
                crate::IrCompletionWindow::new(0, 0, 0),
            ),
        ));
        assert_eq!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::UnretiredCompletionOperation),
            "a region with no drain must be refused, not emitted with a missing join"
        );
    });
}

/// Two independent reads inside a loop that already has two ways out.
///
/// This differs from `READS_ACROSS_A_LOOP` in the one way that matters to the
/// emitter's walk. There the reads are handed out before the loop, in the
/// entry block, so every exit from the loop is numbered after the hand-out.
/// Here they are handed out inside the loop body, and lowering numbers blocks
/// in source order, so the block the loop leaves through on `break` — written
/// first in the body — is numbered before the block that starts them.
const READS_ON_TWO_BRANCHES: &[u8] = br#"fn probe(file: own ReadFile, rounds: own u64) -> result: own u64 reads(file), writes(file), allocates(heap) {
  let outer_left = buffer_new(1_u64, 0_u8);
  let outer_right = buffer_new(1_u64, 0_u8);
  let inner_left = buffer_new(1_u64, 0_u8);
  let inner_right = buffer_new(1_u64, 0_u8);
  let total = 0_u64;
  region 'file {
    region 'a {
      region 'b {
        let outer_first = read_at<'file, 'a>(file: &'file file, destination: &uniq 'a outer_left, file_offset: 0_u64, start: 0_u64, end: 1_u64);
        let outer_second = read_at<'file, 'b>(file: &'file file, destination: &uniq 'b outer_right, file_offset: 1_u64, start: 0_u64, end: 1_u64);
      }
    }
    let split = ieq(rounds, 7_u64);
    if split {
      region 'c {
        region 'd {
          let inner_first = read_at<'file, 'c>(file: &'file file, destination: &uniq 'c inner_left, file_offset: 2_u64, start: 0_u64, end: 1_u64);
          let inner_second = read_at<'file, 'd>(file: &'file file, destination: &uniq 'd inner_right, file_offset: 3_u64, start: 0_u64, end: 1_u64);
        }
      }
      let inner_branch = ieq(rounds, 8_u64);
      if inner_branch {
        set total = total +wrap 2_u64;
      } else {
        set total = total +wrap 3_u64;
      }
    } else {
      set total = total +wrap 1_u64;
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

const READS_BELOW_A_LOOP_EXIT: &[u8] = br#"fn probe(file: own ReadFile, rounds: own u64) -> result: own u64 reads(file), writes(file), allocates(heap) {
  let left = buffer_new(1_u64, 0_u8);
  let right = buffer_new(1_u64, 0_u8);
  let total = 0_u64;
  let cursor = 0_u64;
  loop @spin {
    let done = ieq(cursor, rounds);
    if done {
      break @spin;
    }
    let bail = ieq(cursor, 7_u64);
    if bail {
      return 1_u64;
    }
    region 'file {
      region 'left {
        region 'right {
          let first = read_at<'file, 'left>(file: &'file file, destination: &uniq 'left left, file_offset: 0_u64, start: 0_u64, end: 1_u64);
          let second = read_at<'file, 'right>(file: &'file file, destination: &uniq 'right right, file_offset: 1_u64, start: 0_u64, end: 1_u64);
          match move first {
            ReadBytes(next: produced) => {
              set total = total +wrap produced;
            }
            ReadEnd() => {
            }
            ReadFailed(error: problem) => {
            }
          }
          match move second {
            ReadBytes(next: produced) => {
              set total = total +wrap produced;
            }
            ReadEnd() => {
            }
            ReadFailed(error: problem) => {
            }
          }
        }
      }
    }
    set cursor = cursor +wrap 1_u64;
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

/// A drain the emission walk reaches before the hand-out it must retire is
/// refused, rather than emitted with the join simply missing.
///
/// Blocks are emitted in index order and a drain can only retire hand-outs
/// that already exist, so a carrying block that starts an operation and is
/// numbered after one of the region's exits leaves that exit with no join at
/// all. Above, the loop's `break` exit is numbered before the block that hands
/// the read out, and walking straight through emits a bare `ret` there while
/// the operation is still owned by a target — which would write its result
/// into storage the frame no longer exists to hold.
///
/// Nothing downstream catches it: a function carrying a pipeline is exempt
/// from the straight-line check at the end of emission, exactly because a
/// carrying block is free to be the last block emitted. So the ordering is a
/// precondition on the descriptor and it is checked, like the region's other
/// precondition, before a line of the function is written.
///
/// It is the ordering and not the shape. `READS_ACROSS_A_LOOP` takes the same
/// kind of carrying set and emits, and its latch is numbered after the typed
/// exit its back edge reaches — which is admitted, because a block that starts
/// no operation leaves that exit nothing to be missing.
#[test]
fn a_drain_emitted_before_the_hand_out_it_retires_is_refused() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    // The source itself is an ordinary accepted program; only the descriptor
    // below is out of order.
    let sequential = with_mutated_completion_ir(READS_BELOW_A_LOOP_EXIT, |program| {
        emit_llvm_for_target(program, target)
            .expect("the probe must emit")
            .into_string()
    });
    let sequential = emitted_function(&sequential, "probe");
    let submissions = ir_blocks_containing(sequential, "@wf__completion_file_pread_submit(");
    assert_eq!(
        submissions.len(),
        1,
        "the probe must hand one of its two independent reads to a target: {submissions:?}"
    );
    assert!(
        submissions.iter().all(|block| block != "entry"),
        "the hand-out must be inside the loop, or nothing is out of order: {submissions:?}"
    );

    with_mutated_completion_ir(READS_BELOW_A_LOOP_EXIT, |program| {
        let probe = program
            .functions()
            .iter()
            .find(|function| function.name() == "probe")
            .expect("the probe function");
        let carrying = blocks_that_reach_the_back_edge(probe);
        assert!(
            carrying.len() > 1,
            "the probe's loop must span more than one block"
        );
        assert!(program.set_completion_pipeline_for_test(
            "probe",
            crate::IrCompletionPipeline::new(
                crate::IrBlockId::from_index(0).expect("the entry block"),
                carrying,
                crate::IrCompletionWindow::new(0, 65_536, 32),
            ),
        ));
        assert_eq!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::UnretiredCompletionOperation),
            "a drain numbered before the hand-out it retires must be refused"
        );
    });
}

/// The blocks of one function that hand an operation to a target.
fn blocks_that_hand_out(function: &crate::IrFunction) -> Vec<crate::IrBlockId> {
    function
        .blocks()
        .iter()
        .enumerate()
        .filter(|(_, block)| {
            block.instructions().iter().any(|instruction| {
                matches!(
                    instruction,
                    crate::IrInstruction::Define { result, .. }
                        if function
                            .completion_steps()
                            .iter()
                            .any(|step| step.call() == *result && step.submit())
                )
            })
        })
        .map(|(index, _)| crate::IrBlockId::from_index(index).expect("block ordinal"))
        .collect()
}

/// The successors of one block, by index.
fn block_targets(block: &crate::IrBlock) -> Vec<usize> {
    match block.terminator() {
        crate::IrTerminator::Jump { target, .. } => vec![target.index()],
        crate::IrTerminator::Match { targets, .. } => targets
            .iter()
            .map(|target| target.block().index())
            .collect(),
        crate::IrTerminator::Return { .. } | crate::IrTerminator::Unreachable => Vec::new(),
    }
}

/// A staged descriptor cannot carry a completion result through a source
/// branch just to drop it later.
///
/// The source does not inspect either outcome, but lowering must still thread
/// each result through branch edges until its owning scope ends. Those edge
/// arguments are real SSA uses: a later drain would otherwise have to define
/// the result on a path that already needed it. Refuse the descriptor instead
/// of making an incomplete slot look like a completed result.
#[test]
fn a_branched_staged_descriptor_that_threads_results_is_refused() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    with_mutated_completion_ir(READS_ON_TWO_BRANCHES, |program| {
        let probe = program
            .functions()
            .iter()
            .find(|function| function.name() == "probe")
            .expect("the probe function");
        let carrying = blocks_that_hand_out(probe);
        assert_eq!(
            carrying.len(),
            2,
            "the probe must start one operation on each of two branches"
        );
        assert!(program.set_completion_pipeline_for_test(
            "probe",
            crate::IrCompletionPipeline::new(
                crate::IrBlockId::from_index(0).expect("the entry block"),
                carrying,
                crate::IrCompletionWindow::new(0, 65_536, 32),
            ),
        ));
        assert_eq!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::StagedCompletionResultUse),
            "a staged result cannot be threaded through a source branch"
        );
    });
}

/// A loop whose body both submits and joins, and whose [PAR-3] verdict is
/// `permitted`.
///
/// Each iteration constructs the two buffers it reads into, and leaves the
/// first result deliberately unused. That is the completion pipeline's
/// current valid boundary: it can carry a target request and its mapper facts
/// around the loop, but it does not yet rewrite a source continuation that
/// consumes the result before a drain. The second read is independent work
/// after the first, which is what makes the first a hand-out.
const A_STAGED_LOOP_BODY: &[u8] = br#"fn probe(file: own ReadFile, rounds: own u64) -> result: own u64 reads(file), writes(file), allocates(heap) {
  let total = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let left = buffer_new(1_u64, 0_u8);
    let right = buffer_new(1_u64, 0_u8);
    region 'file {
      region 'left {
        region 'right {
          let first = read_at<'file, 'left>(file: &'file file, destination: &uniq 'left left, file_offset: 0_u64, start: 0_u64, end: 1_u64);
          let second = read_at<'file, 'right>(file: &'file file, destination: &uniq 'right right, file_offset: 1_u64, start: 0_u64, end: 1_u64);
        }
      }
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

/// A staged region cannot apply a source dependency while work remains
/// carried on another path.  The current outstanding ledger is deliberately
/// source ordered rather than per edge, so admitting that descriptor could
/// erase an operation which a sibling path still owns.
#[test]
fn a_staged_region_with_an_early_completion_dependency_is_refused() {
    let original = std::str::from_utf8(A_STAGED_LOOP_BODY).expect("the fixture is UTF-8");
    let second = "          let second = read_at<'file, 'right>(file: &'file file, destination: &uniq 'right right, file_offset: 1_u64, start: 0_u64, end: 1_u64);";
    let replacement = format!(
        "{second}\n          let third = read_at<'file, 'left>(file: &'file file, destination: &uniq 'left left, file_offset: 2_u64, start: 0_u64, end: 1_u64);"
    );
    let source = original.replace(second, &replacement);
    assert_ne!(
        source, original,
        "the dependency fixture must add a third read"
    );
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");

    with_mutated_completion_ir(source.as_bytes(), |program| {
        let probe = program
            .functions()
            .iter()
            .find(|function| function.name() == "probe")
            .expect("the probe function");
        let carrying = blocks_that_reach_the_back_edge(probe);
        let has_carried_dependency = probe.blocks().iter().enumerate().any(|(index, block)| {
            let id = crate::IrBlockId::from_index(index).expect("a block ordinal");
            carrying.contains(&id)
                && block.instructions().iter().any(|instruction| {
                    matches!(instruction, crate::IrInstruction::Define { result, .. }
                        if probe
                            .completion_steps()
                            .iter()
                            .any(|step| step.call() == *result && !step.wait_for().is_empty()))
                })
        });
        assert!(
            has_carried_dependency,
            "the third read must wait for an earlier carried read"
        );
        assert!(program.set_completion_pipeline_for_test(
            "probe",
            crate::IrCompletionPipeline::new(
                crate::IrBlockId::from_index(0).expect("the entry block"),
                carrying,
                crate::IrCompletionWindow::new(0, 65_536, 32),
            ),
        ));
        assert_eq!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::StagedCompletionDependency),
            "a carrying dependency requires path-specific outstanding state"
        );
    });
}

/// The blocks that dominate `block`, computed from the successor relation the
/// probe helpers already use.
///
/// A slot index is rendered straight into the `getelementptr` its block emits,
/// so the value has to dominate that block; the emitter trusts that the same
/// way it trusts every other operand, and these tests earn the trust rather
/// than assuming it.
fn blocks_dominating(function: &crate::IrFunction, block: crate::IrBlockId) -> Vec<usize> {
    let count = function.blocks().len();
    let all: Vec<usize> = (0..count).collect();
    let mut dominators: Vec<Vec<usize>> = (0..count)
        .map(|index| if index == 0 { vec![0] } else { all.clone() })
        .collect();
    let mut changed = true;
    while changed {
        changed = false;
        for index in 1..count {
            let predecessors: Vec<usize> = function
                .blocks()
                .iter()
                .enumerate()
                .filter(|(_, candidate)| block_targets(candidate).contains(&index))
                .map(|(ordinal, _)| ordinal)
                .collect();
            let mut next: Vec<usize> = match predecessors.split_first() {
                None => vec![index],
                Some((first, rest)) => {
                    let mut shared = dominators[*first].clone();
                    for predecessor in rest {
                        shared.retain(|candidate| dominators[*predecessor].contains(candidate));
                    }
                    shared.push(index);
                    shared.sort_unstable();
                    shared.dedup();
                    shared
                }
            };
            next.sort_unstable();
            if next != dominators[index] {
                dominators[index] = next;
                changed = true;
            }
        }
    }
    dominators[block.index()].clone()
}

/// The block the unstaged emission submits its first target operation in.
fn the_block_that_submits(module: &str) -> crate::IrBlockId {
    let labels = ir_blocks_containing(emitted_function(module, "probe"), "_submit(");
    let label = labels.first().expect("the probe must submit somewhere");
    let ordinal = match label.as_str() {
        "entry" => 0,
        other => other
            .strip_prefix("bb")
            .expect("a block label")
            .parse::<usize>()
            .expect("a block ordinal"),
    };
    crate::IrBlockId::from_index(ordinal).expect("a block ordinal")
}

/// A `u64` the submitting block may address a ring through: a parameter of a
/// block that dominates it, which is where a driver's loop-carried slot would
/// live.
///
/// Be exact about what that resolves to here. The probe's loop header is the
/// first dominator that is not the entry, and it carries six parameters, five
/// of them `u64`: the carried copy of the caller's `rounds` argument, the
/// running total, and the loop's index and its bounds. This helper takes the
/// first `u64` parameter of that header, which is the carried copy of
/// `rounds`, threaded around the back edge unchanged. The slot these tests
/// hand the emitter is therefore a caller-supplied, loop-invariant `u64`,
/// not an index that advances with the iteration; it exercises the
/// addressing such an index would take, and nothing more.
fn a_slot_index_for(function: &crate::IrFunction, block: crate::IrBlockId) -> crate::IrValueId {
    let u64_type = crate::IrType::Integer {
        width: 64,
        signed: false,
    };
    blocks_dominating(function, block)
        .iter()
        .filter(|dominator| **dominator != 0)
        .find_map(|dominator| {
            function.blocks()[*dominator]
                .parameters()
                .iter()
                .find(|(_, ty)| *ty == u64_type)
                .map(|(value, _)| *value)
        })
        .expect("the loop must carry a u64 the body can address a ring through")
}

/// Which slot the descriptor under test gives the block that submits.
#[derive(Clone, Copy)]
enum SlotChoice {
    /// A `u64` a dominating block carries — what a driver threads in.
    Carried,
    /// A value of the wrong type.
    NotAnIndex,
    /// A `u64` from a block that cannot reach the addressed block on every
    /// path.
    NonDominating,
    /// Nothing, which is the descriptor that would silently share one record.
    None,
}

/// A non-`u64` the submitting block could name, to prove the type is checked.
fn a_value_that_is_not_an_index(
    function: &crate::IrFunction,
    block: crate::IrBlockId,
) -> crate::IrValueId {
    let u64_type = crate::IrType::Integer {
        width: 64,
        signed: false,
    };
    blocks_dominating(function, block)
        .iter()
        .find_map(|dominator| {
            function.blocks()[*dominator]
                .parameters()
                .iter()
                .find(|(_, ty)| *ty != u64_type)
                .map(|(value, _)| *value)
        })
        .expect("the probe's loop must carry something that is not an index")
}

/// A `u64` block parameter that cannot dominate `block`.
///
/// The ring address is emitted in `block`, so choosing a sibling's parameter
/// would make an SSA use on a path where that parameter has no definition.
/// This gives the descriptor validator a real dominance failure rather than
/// merely an instruction-local value it already rejects conservatively.
fn a_u64_that_does_not_dominate(
    function: &crate::IrFunction,
    block: crate::IrBlockId,
) -> crate::IrValueId {
    let u64_type = crate::IrType::Integer {
        width: 64,
        signed: false,
    };
    let dominators = blocks_dominating(function, block);
    function
        .blocks()
        .iter()
        .enumerate()
        .filter(|(index, _)| !dominators.contains(index))
        .find_map(|(_, candidate)| {
            candidate
                .parameters()
                .iter()
                .find(|(_, ty)| *ty == u64_type)
                .map(|(value, _)| *value)
        })
        .expect("the probe must have a non-dominating u64 block parameter")
}

fn the_unstaged_probe() -> String {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    with_mutated_completion_ir(A_STAGED_LOOP_BODY, |program| {
        emit_llvm_for_target(program, target)
            .expect("the probe must emit")
            .into_string()
    })
}

/// Emits the probe with a ring of `slots` records per site, addressed as
/// `choice` says.
///
/// The descriptor is one the emitter accepts, not one a driver has been shown
/// to produce. Its slot is the loop-invariant `rounds` parameter
/// `a_slot_index_for` resolves. The emitted checked-slot helper makes that
/// otherwise unbounded input safe for every ring address.
fn emit_a_ring_for_target(
    slots: u64,
    choice: SlotChoice,
    target: SystemTarget,
) -> Result<String, crate::BackendFailure> {
    let submitting = the_block_that_submits(&the_unstaged_probe());
    with_mutated_completion_ir(A_STAGED_LOOP_BODY, |program| {
        let probe = program
            .functions()
            .iter()
            .find(|function| function.name() == "probe")
            .expect("the probe function");
        let carrying = blocks_that_reach_the_back_edge(probe);
        assert!(
            carrying.contains(&submitting),
            "the probe's submission must be inside the carrying region, or the ring proves nothing"
        );
        // A driver threads the slot into every block of its region, because
        // the block that retires an operation need not be the block that
        // started it: here the loop's exit is what drains the window.
        let addressed: Vec<(crate::IrBlockId, crate::IrValueId)> = (1..probe.blocks().len())
            .map(|index| {
                let id = crate::IrBlockId::from_index(index).expect("a block ordinal");
                (id, a_slot_index_for(probe, id))
            })
            .collect();
        let slot_index = match choice {
            SlotChoice::Carried => addressed,
            SlotChoice::NotAnIndex => addressed
                .into_iter()
                .map(|(block, slot)| {
                    if block == submitting {
                        (block, a_value_that_is_not_an_index(probe, block))
                    } else {
                        (block, slot)
                    }
                })
                .collect(),
            SlotChoice::NonDominating => addressed
                .into_iter()
                .map(|(block, slot)| {
                    if block == submitting {
                        (block, a_u64_that_does_not_dominate(probe, block))
                    } else {
                        (block, slot)
                    }
                })
                .collect(),
            SlotChoice::None => Vec::new(),
        };
        assert!(
            program.set_completion_pipeline_for_test(
                "probe",
                crate::IrCompletionPipeline::with_slots(
                    crate::IrBlockId::from_index(0).expect("the entry block"),
                    carrying,
                    crate::IrCompletionWindow::new(0, 65_536, 32),
                    slots,
                    slot_index,
                ),
            ),
            "the probe function must take a pipeline"
        );
        emit_llvm_for_target(program, target).map(super::super::emitter::LlvmModule::into_string)
    })
}

fn emit_a_ring(slots: u64, choice: SlotChoice) -> Result<String, crate::BackendFailure> {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    emit_a_ring_for_target(slots, choice, target)
}

/// The types this function reserves, in reservation order and without the
/// temporary names, which shift when a module also asks for a window.
fn reserved_types(function: &str) -> Vec<String> {
    function
        .lines()
        .filter_map(|line| line.split_once("= alloca "))
        .map(|(_, reserved)| reserved.trim().to_owned())
        .collect()
}

/// Sends emitted IR through LLVM's parser and verifier.
///
/// `clang -emit-llvm -c` rejects malformed IR before it can produce bitcode,
/// including non-dominating SSA uses. Keeping this as an external check makes
/// the staged tests exercise the same parser/verifier a downstream LLVM tool
/// sees rather than only checking strings the emitter happened to write.
fn verify_llvm(module: &str) {
    let sink = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let mut child = Command::new("clang")
        .args(["-x", "ir", "-emit-llvm", "-c", "-o", sink, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the LLVM verifier must be available as clang");
    let mut input = child.stdin.take().expect("clang must accept LLVM IR");
    input
        .write_all(module.as_bytes())
        .expect("the complete LLVM module must reach clang");
    drop(input);
    let output = child
        .wait_with_output()
        .expect("the LLVM verifier must return a status");
    assert!(
        output.status.success(),
        "LLVM rejected staged completion IR:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// A staged region reserves one operation record per slot and addresses the
/// one the block names.
///
/// This is what makes a back edge with work in flight correct rather than
/// merely admitted. The carrying block is emitted once and reached once per
/// iteration; with a single record the second iteration would hand the target
/// a token and a result slot the first iteration's operation is still being
/// written into. So the reservation becomes an array, and the element is
/// chosen where the operation is started, from the index the driver carried
/// into the block.
#[test]
fn a_staged_region_reserves_one_operation_record_per_slot() {
    let unstaged = the_unstaged_probe();
    assert!(
        !unstaged.contains("alloca [4 x"),
        "the unstaged probe must reserve one record per site, not a ring"
    );
    let module = emit_a_ring(4, SlotChoice::Carried).expect("a staged probe must emit");
    assert!(
        module.contains("call i64 @wf__completion_window(i64 0, i64 65536, i64 4)"),
        "the runtime window is capped by the four allocated records"
    );
    let staged = emitted_function(&module, "probe");

    // The body's first read is the hand-out — the second is independent work
    // after it, and has none of its own — so one site reserves seven rings,
    // each four elements wide. Besides the target-facing token/result/raw
    // fields, the record retains submission state and both mapper facts for a
    // join in another block.
    let rings = reserved_types(staged)
        .into_iter()
        .filter(|reserved| reserved.starts_with("[4 x "))
        .count();
    assert_eq!(
        rings, 7,
        "the handed-out site reserves a ring for target state and mapper facts"
    );

    // And every element pointer is that ring indexed by the slot, in the block
    // that names it — never a constant element the two iterations would share.
    let indexed: Vec<&str> = staged
        .lines()
        .filter(|line| line.contains("getelementptr inbounds [4 x"))
        .collect();
    let checked = staged
        .matches("call i64 @wf__completion_checked_slot(i64 ")
        .count();
    assert_eq!(
        indexed.len(),
        checked,
        "every ring GEP must use the helper-checked slot: {indexed:?}"
    );
    assert_eq!(
        indexed.len(),
        12,
        "submission addresses target state and mapper facts; retirement reloads \
         all target state and facts: {indexed:?}"
    );
    for line in &indexed {
        let (_, index) = line
            .rsplit_once(", i64 0, i64 ")
            .unwrap_or_else(|| panic!("an element pointer indexes its ring: {line}"));
        assert!(
            index.starts_with("%t"),
            "an element pointer must be indexed by the helper-checked slot, never \
             a constant element the two iterations would share: {line}"
        );
    }
}

/// One slot keeps every completion record fixed and needs no ring proof.
///
/// A window of one is always a legal answer — it is the schedule the
/// sequential program already runs. A staged record still retains its
/// submission state and mapper facts, but it addresses its only element with
/// no run-time slot and emits no checked-slot helper.
#[test]
fn one_slot_needs_no_dynamic_ring_index() {
    let one = emit_a_ring(1, SlotChoice::Carried).expect("a one-slot probe must emit");
    assert!(
        !one.contains("wf__completion_checked_slot"),
        "a single fixed record has no ring GEP to guard"
    );
    assert!(
        !one.contains(", i64 0, i64 %v"),
        "a one-slot region indexes its records by no run-time value: the element \
         is the reservation's only one"
    );
}

/// The accepted staged ring is real LLVM, not merely emitter-shaped text.
#[test]
fn staged_completion_ring_passes_the_llvm_parser_and_verifier() {
    let module = emit_a_ring(4, SlotChoice::Carried).expect("a staged probe must emit");
    assert!(
        module.contains("define private i64 @wf__completion_checked_slot"),
        "a multi-slot ring must publish its range guard"
    );
    verify_llvm(&module);
}

/// Windows pressure recovery may inspect a ring element before that iteration
/// takes its submit arm. Its submission-state array must therefore start false
/// in the entry prelude, not on a path through the loop body.
#[test]
fn windows_staged_ring_initializes_submission_state_before_pressure_recovery() {
    let target = SystemTarget::for_triple("x86_64-pc-windows-msvc")
        .expect("the native Windows completion target");
    let module = emit_a_ring_for_target(4, SlotChoice::Carried, target)
        .expect("the Windows staged probe must emit");
    let body = emitted_function(&module, "probe");
    let initialized = body
        .find("store [4 x i1] zeroinitializer, ptr ")
        .expect("the submission-state ring is initialized in the entry prelude");
    let submit = body
        .find("call i32 @wf__completion_file_pread_submit")
        .expect("the staged probe submits a positioned read");
    let pressure = body
        .find("completion.capacity.v")
        .expect("Windows emits a capacity-recovery path");
    assert!(
        initialized < submit && initialized < pressure,
        "the state ring starts false before either an accepted submit or a pressure path"
    );
    assert!(
        module.contains("declare void @abort() noreturn"),
        "the checked ring helper has an abort declaration even without a local match"
    );
}

/// A ring with no elements is refused.
#[test]
fn a_ring_with_no_elements_is_refused() {
    assert_eq!(
        emit_a_ring(0, SlotChoice::Carried).err(),
        Some(crate::BackendFailure::MisaddressedCompletionSlot),
        "a descriptor claiming no slots would reserve a zero-length array and index into it"
    );
}

/// A slot that is not the `u64` the ring is indexed with is refused.
#[test]
fn a_slot_that_is_not_an_index_is_refused() {
    assert_eq!(
        emit_a_ring(4, SlotChoice::NotAnIndex).err(),
        Some(crate::BackendFailure::MisaddressedCompletionSlot),
        "an index of the wrong type emits a module that does not verify"
    );
}

/// A `u64` alone is not enough: it must be defined on every route to the ring
/// address.
#[test]
fn a_slot_that_does_not_dominate_its_ring_address_is_refused() {
    assert_eq!(
        emit_a_ring(4, SlotChoice::NonDominating).err(),
        Some(crate::BackendFailure::MisaddressedCompletionSlot),
        "a non-dominating slot would make an invalid LLVM use"
    );
}

/// An unreachable loop must not acquire every block as a dominator merely
/// because its SCC has no path from entry.  The old universe-initialized
/// fixed point did exactly that and could admit an arbitrary block parameter
/// as a ring index in every other unreachable block.
#[test]
fn an_unreachable_completion_scc_cannot_invent_slot_dominance() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    with_mutated_completion_ir(A_STAGED_LOOP_BODY, |program| {
        assert!(
            program.disconnect_function_body_for_test("probe"),
            "the probe entry must be disconnected"
        );
        let probe = program
            .functions()
            .iter()
            .find(|function| function.name() == "probe")
            .expect("the probe function");
        let carrying = blocks_that_reach_the_back_edge(probe);
        assert!(
            !carrying.contains(&crate::IrBlockId::from_index(0).expect("the entry block")),
            "the retained loop must be structurally unreachable from entry"
        );
        let index_type = crate::IrType::Integer {
            width: 64,
            signed: false,
        };
        let slot = carrying
            .iter()
            .find_map(|block| {
                probe.blocks()[block.index()]
                    .parameters()
                    .iter()
                    .find(|(_, ty)| *ty == index_type)
                    .map(|(value, _)| *value)
            })
            .expect("the unreachable loop carries a u64 parameter");
        let addressed = (1..probe.blocks().len())
            .map(|index| {
                (
                    crate::IrBlockId::from_index(index).expect("a block ordinal"),
                    slot,
                )
            })
            .collect();
        assert!(program.set_completion_pipeline_for_test(
            "probe",
            crate::IrCompletionPipeline::with_slots(
                crate::IrBlockId::from_index(0).expect("the entry block"),
                carrying,
                crate::IrCompletionWindow::new(0, 65_536, 32),
                4,
                addressed,
            ),
        ));
        assert_eq!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::MisaddressedCompletionSlot),
            "an unreachable SCC cannot make an external block parameter dominate its peers"
        );
    });
}

/// A carrying block that submits with no slot is refused, not handed element
/// zero.
///
/// This is the refusal that matters. Falling back to the first element there
/// is exactly the sharing the ring exists to prevent, and it would show up
/// only as two iterations reading one buffer — never as a diagnostic.
#[test]
fn a_carrying_block_with_no_slot_is_refused_rather_than_sharing_one_record() {
    assert_eq!(
        emit_a_ring(4, SlotChoice::None).err(),
        Some(crate::BackendFailure::MisaddressedCompletionSlot),
        "a submission inside a ring must address a slot"
    );
}
