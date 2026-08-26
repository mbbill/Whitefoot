use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use super::system::with_ir_overlap;
use super::{build_executable, emit, emit_lowered, test_directory};

const INDEPENDENT_WRITES: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus writes(out err), allocates(heap) {
  let bulk = buffer_new(1048576_u64, 65_u8);
  let marker = buffer_new(1_u64, 77_u8);
  region 'out {
    region 'err {
      region 'bulk {
        region 'marker {
          let first = write_once<'out, 'bulk>(output: &'out out, source: &'bulk bulk, start: 0_u64, end: 1048576_u64);
          let second = write_once<'err, 'marker>(output: &'err err, source: &'marker marker, start: 0_u64, end: 1_u64);
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

const ORDERED_WRITES: &[u8] = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus writes(out), allocates(heap) {
  let bytes = buffer_new(2_u64, 65_u8);
  set bytes[1_u64] = 66_u8;
  region 'out {
    region 'source {
      let first = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 0_u64, end: 1_u64);
      let second = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 1_u64, end: 2_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const NONADJACENT_ORDERED_WRITES: &[u8] = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus writes(out err), allocates(heap) {
  let bytes = buffer_new(3_u64, 65_u8);
  set bytes[1_u64] = 66_u8;
  set bytes[2_u64] = 67_u8;
  region 'out {
    region 'err {
      region 'source {
        let first = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 0_u64, end: 1_u64);
        let middle = write_once<'err, 'source>(output: &'err err, source: &'source bytes, start: 1_u64, end: 2_u64);
        let last = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 2_u64, end: 3_u64);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const ORDERED_WRITES_WITH_EMPTY_MIDDLE: &[u8] = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus writes(out), allocates(heap) {
  let bytes = buffer_new(2_u64, 65_u8);
  set bytes[1_u64] = 66_u8;
  region 'out {
    region 'source {
      let first = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 0_u64, end: 1_u64);
      let empty = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 1_u64, end: 1_u64);
      let third = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 1_u64, end: 2_u64);
      match move first {
        Ok(value: next) => {
          if ieq(next, 1_u64) {
          } else {
            return exit_status(code: 201_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 202_u8);
        }
      }
      match move empty {
        Ok(value: next) => {
          if ieq(next, 1_u64) {
          } else {
            return exit_status(code: 203_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 204_u8);
        }
      }
      match move third {
        Ok(value: next) => {
          if ieq(next, 2_u64) {
          } else {
            return exit_status(code: 205_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 206_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const ORDERED_ALL_EMPTY: &[u8] = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus writes(out), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region 'out {
    region 'source {
      let first = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 0_u64, end: 0_u64);
      let second = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 0_u64, end: 0_u64);
      match move first {
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
      match move second {
        Ok(value: next) => {
          if ieq(next, 0_u64) {
          } else {
            return exit_status(code: 213_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 214_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const ORDERED_DIRECTORY: &[u8] = br#"fn probe(source: own DirectorySource) -> result: own unit writes(source), allocates(heap) {
  let first_bytes = buffer_new(64_u64, 0_u8);
  let second_bytes = buffer_new(64_u64, 0_u8);
  region 'source {
    region 'first {
      region 'second {
        let first = directory_next<'source, 'first>(source: &'source source, destination: &uniq 'first first_bytes, start: 0_u64, end: 64_u64);
        let second = directory_next<'source, 'second>(source: &'source source, destination: &uniq 'second second_bytes, start: 0_u64, end: 64_u64);
      }
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
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

command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus writes(out err), allocates(heap) {
  let left = choose(value: 1_u64);
  let right = choose(value: 2_u64);
  let total = imax(left, right);
  let bytes = buffer_new(2_u64, 65_u8);
  set bytes[1_u64] = 66_u8;
  region 'out {
    region 'err {
      region 'source {
        let first = write_once<'out, 'source>(output: &'out out, source: &'source bytes, start: 0_u64, end: 1_u64);
        let second = write_once<'err, 'source>(output: &'err err, source: &'source bytes, start: 1_u64, end: 2_u64);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

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
            .matches("call i32 @wf__completion_file_batch_claim")
            .count(),
        2,
        "both the compute-overlap world and its sequential clone need completion I/O"
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
    let claim = module
        .find("call i32 @wf__completion_file_batch_claim")
        .expect("the independent group must reserve all slots first");
    let first = module[claim..]
        .find("call void @wf__completion_file_write_submit_reserved")
        .map(|offset| claim + offset)
        .expect("submit the first reserved write");
    let second = module[first + 1..]
        .find("call void @wf__completion_file_write_submit_reserved")
        .map(|offset| first + 1 + offset)
        .expect("submit the source-last reserved write");
    let join = module[second..]
        .find("call void @wf__completion_file_join")
        .map(|offset| second + offset)
        .expect("ownership completion must join after every submission");
    assert!(claim < first && first < second && second < join);
    assert!(!module.contains("call i32 @wf__completion_file_write_submit(i32"));
    assert!(!module.contains("wf__par_publish_io"));
    assert!(!module.contains("wf__par_thunk_"));
    assert!(!module.contains("call i64 @wf__completion_output_batch_begin"));
    assert!(!crate::COMPLETION_FILE_ADAPTER_HEADER.contains("(*"));
    assert!(!crate::COMPLETION_BRIDGE_SOURCE.contains("void (*"));
}

#[test]
fn positioned_read_emits_a_checked_typed_pread_request() {
    let module = emit(POSITIONED_READS);
    assert!(crate::module_requires_completion_runtime(&module));
    assert!(module.contains("call i32 @wf__completion_file_batch_claim"));
    assert_eq!(
        module
            .matches("call void @wf__completion_file_pread_submit_reserved")
            .count(),
        2
    );
    assert!(module.contains("%offset.fits = icmp ule i64 %file_offset"));
    assert!(module.contains("9223372036854775807"));
    assert!(module.contains("call i64 @wf__completion_file_pread_direct(i32"));
    assert!(crate::COMPLETION_BRIDGE_SOURCE.contains("request.kind = WF_FILE_PREAD"));
    assert!(crate::COMPLETION_BRIDGE_SOURCE.contains("file_offset > (uint64_t)INT64_MAX"));
    let reserved_pread = crate::COMPLETION_BRIDGE_SOURCE
        .split_once("void wf__completion_file_pread_submit_reserved(")
        .expect("the bridge exposes reserved positioned read")
        .1
        .split_once("void wf__completion_file_write_submit_reserved(")
        .expect("reserved positioned read precedes reserved write")
        .0;
    assert!(reserved_pread.contains("file_offset > (uint64_t)INT64_MAX"));
    assert!(
        reserved_pread.find("file_offset > (uint64_t)INT64_MAX")
            < reserved_pread.find("wf_bridge_submit_linux_pread")
    );
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
        .split_once("void wf__completion_file_pread_submit_reserved(")
        .expect("ordinary write submit precedes reserved positioned read")
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
fn output_order_edges_are_retained_and_actualized_as_one_batch() {
    with_ir_overlap(
        ORDERED_WRITES,
        crate::OverlapLowering::Completion,
        |program| {
            let main = &program.functions()[program.main_ordinal() as usize];
            let [edge] = main.authority_orders() else {
                panic!(
                    "the two writes must retain one source-order reservation edge: {:?}",
                    main.authority_orders()
                );
            };
            assert!(edge.earlier().ordinal() < edge.later().ordinal());
            assert_eq!(edge.family(), crate::SystemAuthorityFamily::Output);
            assert_eq!(
                edge.earlier_fragment(),
                crate::SystemAuthorityFragment::OutputSequence
            );
            assert_eq!(edge.later_fragment(), edge.earlier_fragment());
            assert_eq!(
                edge.attribution(),
                crate::SystemAuthorityAttribution::OutputBytes
            );
            let [overlap] = main.overlaps() else {
                panic!("the supported ordered pair must form one actualization");
            };
            assert_eq!(
                overlap.ordered_attribution(),
                Some(crate::SystemAuthorityAttribution::OutputBytes)
            );
            assert_eq!(overlap.dispatched().len(), 2);
        },
    );
    let module = emit(ORDERED_WRITES);
    let begin = module
        .find("call i64 @wf__completion_output_batch_begin")
        .expect("reserve the bounded root batch");
    let first = module[begin..]
        .find("call void @wf__completion_output_batch_submit")
        .map(|offset| begin + offset)
        .expect("submit the first reservation");
    let second = module[first + 1..]
        .find("call void @wf__completion_output_batch_submit")
        .map(|offset| first + 1 + offset)
        .expect("submit the second reservation");
    let commit = module[second..]
        .find("call void @wf__completion_output_batch_commit")
        .map(|offset| second + offset)
        .expect("commit only after every reservation exists");
    assert!(begin < first && first < second && second < commit);
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    for helpers in ["0", "1", "4"] {
        let output = Command::new(&executable)
            .env("WF_IO_HELPERS", helpers)
            .output()
            .expect("run ordered OutputSequence batch");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"AB", "WF_IO_HELPERS={helpers}");
    }
    std::fs::remove_file(executable).expect("remove ordered output probe");
    std::fs::remove_dir(directory).expect("remove ordered output directory");
}

#[test]
fn a_nonadjacent_same_root_edge_survives_into_ir_and_actualization() {
    with_ir_overlap(
        NONADJACENT_ORDERED_WRITES,
        crate::OverlapLowering::Completion,
        |program| {
            let main = &program.functions()[program.main_ordinal() as usize];
            let [edge] = main.authority_orders() else {
                panic!("the first and third Output use must retain one edge");
            };
            assert_eq!(edge.family(), crate::SystemAuthorityFamily::Output);
            assert_eq!(
                edge.attribution(),
                crate::SystemAuthorityAttribution::OutputBytes
            );
            assert!(edge.earlier().ordinal() < edge.later().ordinal());
            let [overlap] = main.overlaps() else {
                panic!("the three permitted calls must retain one run");
            };
            assert_eq!(overlap.members().len(), 3);
        },
    );
    let module = emit(NONADJACENT_ORDERED_WRITES);
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    for helpers in ["0", "1", "4"] {
        let output = Command::new(&executable)
            .env("WF_IO_HELPERS", helpers)
            .output()
            .expect("run nonadjacent ordered-output probe");
        assert!(
            output.status.success(),
            "WF_IO_HELPERS={helpers}: {output:?}"
        );
        assert_eq!(output.stdout, b"AC", "WF_IO_HELPERS={helpers}");
        assert_eq!(output.stderr, b"B", "WF_IO_HELPERS={helpers}");
    }
    std::fs::remove_file(executable).expect("remove nonadjacent ordered probe");
    std::fs::remove_dir(directory).expect("remove nonadjacent ordered directory");
}

#[test]
fn ordered_batches_preserve_empty_write_success_without_a_host_write() {
    for (source, expected) in [
        (ORDERED_WRITES_WITH_EMPTY_MIDDLE, b"AB".as_slice()),
        (ORDERED_ALL_EMPTY, b"".as_slice()),
    ] {
        let module = emit(source);
        assert!(module.contains("%empty = icmp eq i64 %extent, 0"));
        assert!(module.contains("label %vacant, label %nonempty"));
        let directory = test_directory();
        let executable = build_executable(&module, &directory);
        for helpers in ["0", "1", "4"] {
            let output = Command::new(&executable)
                .env("WF_IO_HELPERS", helpers)
                .output()
                .expect("run ordered empty-write batch");
            assert!(
                output.status.success(),
                "WF_IO_HELPERS={helpers}: {output:?}"
            );
            assert_eq!(output.stdout, expected, "WF_IO_HELPERS={helpers}");
        }
        std::fs::remove_file(executable).expect("remove ordered empty-write probe");
        std::fs::remove_dir(directory).expect("remove ordered empty-write directory");
    }
}

#[test]
fn directory_cursor_order_remains_explicitly_declined() {
    with_ir_overlap(
        ORDERED_DIRECTORY,
        crate::OverlapLowering::Completion,
        |program| {
            let probe = &program.functions()[0];
            assert!(probe.overlaps().is_empty());
            assert!(probe.authority_orders().iter().all(|edge| {
                edge.family() == crate::SystemAuthorityFamily::DirectorySource
                    && edge.earlier_fragment() == crate::SystemAuthorityFragment::DirectoryCursor
                    && edge.later_fragment() == edge.earlier_fragment()
                    && edge.attribution() == crate::SystemAuthorityAttribution::DirectoryEntries
            }));
            assert!(!probe.authority_orders().is_empty());
        },
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
