use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use super::{build_executable, emit, emit_lowered, test_directory};

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
        .split_once("int wf__completion_file_pread_submit_writer(")
        .expect("ordinary write submit precedes writer-dependent positioned read")
        .0;
    assert!(!write.contains("wf_bridge_submit_linux"));
    assert!(write.contains("current-position and"));
    let direct_write = bridge
        .split_once("int64_t wf__completion_file_write_direct(")
        .expect("the bridge exposes direct write progress")
        .1
        .split_once("void wf__completion_file_join(")
        .expect("direct write precedes join")
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

#[test]
fn independent_io_reaches_the_second_operation_before_the_first_unblocks() {
    let module = emit(INDEPENDENT_WRITES);
    for helpers in ["1", "0", "4"] {
        let directory = test_directory();
        let executable = build_executable(&module, &directory);
        let mut child = Command::new(&executable)
            .env("WF_IO_HELPERS", helpers)
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
            .recv_timeout(Duration::from_secs(3))
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
        .recv_timeout(Duration::from_secs(5))
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
    assert!(crate::COMPLETION_FILE_ADAPTER_SOURCE.contains("S_ISREG(status.st_mode)"));
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
