use std::io::Read;
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

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
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

#[test]
fn only_an_actualized_target_operation_selects_the_completion_runtime() {
    let sequential = emit_lowered(INDEPENDENT_WRITES, crate::OverlapLowering::Off);
    let completion = emit(INDEPENDENT_WRITES);
    let pure_sequential = emit_lowered(PURE_COMPUTE, crate::OverlapLowering::Off);
    let pure = emit(PURE_COMPUTE);

    assert!(crate::module_requires_completion_runtime(&sequential));
    assert!(crate::module_requires_completion_runtime(&completion));
    assert!(!crate::module_requires_completion_runtime(&pure));
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
    assert!(direct_write.contains("wf_file_execute_direct"));
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

/// Host-limited like its enumeration siblings above: opening a directory
/// source is a [SYS-14] operation, and Linux has no approved enumeration row
/// in `backend/qualification.rs`, so this program does not lower there at all.
#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
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

/// The helper count is target policy, and an unset `WF_IO_HELPERS` must not
/// pin it at the one value that measured worst.
///
/// A written setting still pins the count exactly, which is what every test
/// that names `0`, `1`, or `4` depends on. Unset asks for the measured
/// policy: start at one, grow only on evidence that the program exposed width
/// the pool cannot absorb, and never pass the machine's own bound. A host with
/// a native completion path starts at none, because there the ring already
/// carries every transfer and a helper can only add a handoff to the
/// operations it does not carry.
///
/// The evidence of width is the queue depth at the moment a request is
/// enqueued. A rule that instead grew whenever a submission found no helper
/// *waiting* was tried and measured worse on the same programs: a helper that
/// has been signalled but not yet scheduled still counts as waiting, so a run
/// of consecutive submissions sees an available helper every time and the pool
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
    // Unset scales the ceiling to the machine rather than to a constant.
    assert!(policy.contains("sysconf(_SC_NPROCESSORS_ONLN)"));
    assert!(policy.contains("WF_BRIDGE_MAX_HELPERS"));

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
    // Growth runs inside the one enqueue that already holds the queue lock,
    // so it creates at most one helper per submission and needs no second
    // lock, and every kind of queued work reaches the pool the same way.
    let enqueue = adapter
        .split_once("static void wf_file_enqueue_locked(")
        .expect("one place appends an accepted queue entry")
        .1
        .split_once("\n}\n")
        .expect("the enqueue ends with the function")
        .0;
    assert!(
        enqueue.contains("wf_file_grow_helpers_locked(adapter)"),
        "the enqueue is where growth happens: {enqueue}"
    );
    // One queued request wakes one helper, never every helper.
    assert!(
        enqueue.contains("pthread_cond_signal(&adapter->queue_available)"),
        "a submission announces to exactly one helper"
    );
    assert!(
        !enqueue.contains("pthread_cond_broadcast(&adapter->queue_available)"),
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

/// A staged loop leaves its target operations outstanding across the back
/// edge, and every way out of the loop retires them.
///
/// Two facts make one schedule. The back edge no longer joins, which is what
/// gives a loop the right to keep work in flight across its own iterations —
/// today's unconditional join at every terminator is the whole of the round
/// barrier the design measures. And every block the pipeline does not name
/// still joins everything outstanding, which is the drain: the loop's normal
/// exit and the typed exit out of its body each retire the window, and neither
/// leaves an accepted operation owned by nobody.
///
/// The descriptor is the loop judgment's product and the judgment does not
/// exist yet, so the test supplies the block set. Everything the emitter does
/// with it — where a join lands, where it does not, and how many there are —
/// is the shipped path.
#[test]
fn a_staged_loop_carries_completion_across_its_back_edge() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    let sequential = with_mutated_completion_ir(READS_ACROSS_A_LOOP, |program| {
        emit_llvm_for_target(program, target)
            .expect("the probe must emit")
            .into_string()
    });
    let sequential = emitted_function(&sequential, "probe");
    let carried = with_mutated_completion_ir(READS_ACROSS_A_LOOP, |program| {
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
        emit_llvm_for_target(program, target)
            .expect("a staged probe must emit")
            .into_string()
    });
    let carried = emitted_function(&carried, "probe");

    // Without the pipeline the schedule joins where it submitted: one join, in
    // the block the two reads are in.
    let sequential_joins = ir_blocks_containing(sequential, "@wf__completion_file_join(");
    assert_eq!(
        sequential_joins,
        vec!["entry".to_owned()],
        "an unstaged schedule joins in the block that submitted it"
    );

    // With it, the submitting block ends with the operation still in flight
    // and each exit from the loop retires it.
    let carried_joins = ir_blocks_containing(carried, "@wf__completion_file_join(");
    assert!(
        !carried_joins.contains(&"entry".to_owned()),
        "a carrying block must not join: found joins in {carried_joins:?}"
    );
    assert_eq!(
        carried_joins.len(),
        2,
        "the loop's normal exit and its typed exit are two drains, not one: {carried_joins:?}"
    );
    assert!(
        carried_joins[0] != carried_joins[1],
        "the two drains must be two different blocks: {carried_joins:?}"
    );

    // The window is asked once, at the loop's entry block, and the weak
    // answer a link without the completion unit gets is one — the sequential
    // program.
    assert_eq!(
        ir_blocks_containing(carried, "@wf__completion_window(").len(),
        1,
        "the window is asked once per loop entry, never per iteration"
    );
    assert!(
        carried.contains("call i64 @wf__completion_window(i64 0, i64 65536, i64 32)"),
        "the query must carry the compiler's own three bounds"
    );
    assert!(
        !sequential.contains("@wf__completion_window("),
        "a module that stages no loop must name no window symbol"
    );
}

/// The weak window answer is emitted only where a module asks for one, and it
/// is one.
#[test]
fn the_window_fallback_is_emitted_only_where_a_module_asks_for_one() {
    let target = SystemTarget::for_triple("aarch64-apple-darwin").expect("the probe target");
    let sequential = with_mutated_completion_ir(READS_ACROSS_A_LOOP, |program| {
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
    let staged = with_mutated_completion_ir(READS_ACROSS_A_LOOP, |program| {
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
