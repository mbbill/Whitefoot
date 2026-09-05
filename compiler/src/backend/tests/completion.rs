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
              match reserve_file(factory: &uniq files) {
                Ok(value: permit) => {
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
                Err(error: spent) => {
                  return exit_status(code: 8_u8);
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
                    match reserve_file(factory: &uniq 'c files) {
                      Ok(value: permit) => {
                        let opened = open_read(permit: move permit, root: &'c cwd, path: &'p path);
                        let announced = write_once(output: &uniq 'err err, source: &marker, start: 0_u64, end: 1_u64);
                      }
                      Err(error: spent) => {
                        return exit_status(code: 8_u8);
                      }
                    }
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
              match reserve_file(factory: &uniq files) {
                Ok(value: permit) => {
                  match open_read(permit: move permit, root: &cwd, path: &path) {
                    FileOpened(value: file) => {
                      return exit_status(code: 1_u8);
                    }
                    FileOpenFailed(error: problem, permit: refused_2) => {
                      return exit_status(code: 0_u8);
                    }
                  }
                }
                Err(error: spent) => {
                  return exit_status(code: 8_u8);
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
              match reserve_file(factory: &uniq files) {
                Ok(value: permit) => {
                  region 'marker {
                    region {
                      let opened = open_read(permit: move permit, root: &'state cwd, path: &'state path);
                      let announced = write_once(output: &uniq err, source: &'marker marker, start: 0_u64, end: 1_u64);
                      match move opened {
                        FileOpened(value: file) => {
                          return exit_status(code: 1_u8);
                        }
                        FileOpenFailed(error: problem, permit: refused) => {
                          return exit_status(code: 0_u8);
                        }
                      }
                    }
                  }
                }
                Err(error: spent) => {
                  return exit_status(code: 8_u8);
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
    match reserve_file(factory: &uniq files) {
      Ok(value: first_permit) => {
        match reserve_file(factory: &uniq files) {
          Ok(value: second_permit) => {
            region {
              let first = open_directory(permit: move first_permit, root: &'c cwd, name: &first_name, start: 0_u64, end: 1_u64);
              let second = open_directory(permit: move second_permit, root: &'c cwd, name: &second_name, start: 0_u64, end: 1_u64);
            }
          }
          Err(error: spent) => {
            return exit_status(code: 8_u8);
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_DIRECTORY_SOURCE_OPENS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region {
    match reserve_file(factory: &uniq files) {
      Ok(value: first_permit) => {
        match reserve_file(factory: &uniq files) {
          Ok(value: second_permit) => {
            let first = open_directory_source(permit: move first_permit, directory: &cwd);
            let second = open_directory_source(permit: move second_permit, directory: &cwd);
          }
          Err(error: spent) => {
            return exit_status(code: 8_u8);
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_REGULAR_FILE_OPENS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let first_name = buffer_new(1_u64, 120_u8);
  let second_name = buffer_new(1_u64, 120_u8);
  region 'c {
    match reserve_file(factory: &uniq files) {
      Ok(value: first_permit) => {
        match reserve_file(factory: &uniq files) {
          Ok(value: second_permit) => {
            region {
              let first = open_file(permit: move first_permit, root: &'c cwd, name: &first_name, start: 0_u64, end: 1_u64);
              let second = open_file(permit: move second_permit, root: &'c cwd, name: &second_name, start: 0_u64, end: 1_u64);
            }
          }
          Err(error: spent) => {
            return exit_status(code: 8_u8);
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const INDEPENDENT_DIRECTORY_READS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let first_bytes = buffer_new(4096_u64, 0_u8);
  let second_bytes = buffer_new(4096_u64, 0_u8);
  region {
    match reserve_file(factory: &uniq files) {
      Ok(value: first_permit) => {
        match reserve_file(factory: &uniq files) {
          Ok(value: second_permit) => {
            match open_directory_source(permit: move first_permit, directory: &cwd) {
              SourceOpened(value: first_list) => {
                match open_directory_source(permit: move second_permit, directory: &cwd) {
                  SourceOpened(value: second_list) => {
                    region {
                      let first = directory_next(source: &uniq first_list, destination: &uniq first_bytes, start: 0_u64, end: 4096_u64);
                      let second = directory_next(source: &uniq second_list, destination: &uniq second_bytes, start: 0_u64, end: 4096_u64);
                    }
                    return exit_status(code: 0_u8);
                  }
                  SourceOpenFailed(error: problem, permit: refused_2) => {
                    return exit_status(code: 201_u8);
                  }
                }
              }
              SourceOpenFailed(error: problem, permit: refused_2) => {
                return exit_status(code: 202_u8);
              }
            }
          }
          Err(error: spent) => {
            return exit_status(code: 8_u8);
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
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
    match reserve_file(factory: &uniq files) {
      Ok(value: permit) => {
        region {
          match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
            FileOpened(value: handle) => {
              set opened = opened +wrap 1_u64;
            }
            FileOpenFailed(error: problem, permit: refused_2) => {
            }
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
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
    match reserve_file(factory: &uniq files) {
      Ok(value: permit) => {
        region {
          match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
            FileOpened(value: handle) => {
            }
            FileOpenFailed(error: problem, permit: refused_2) => {
            }
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
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
      match reserve_file(factory: &uniq files) {
        Ok(value: permit) => {
          region {
            match open_file(permit: move permit, root: &'f cwd, name: &names, start: index, end: end) {
              FileOpened(value: handle) => {
                set opened = opened +wrap 1_u64;
              }
              FileOpenFailed(error: problem, permit: refused_2) => {
              }
            }
          }
        }
        Err(error: spent) => {
          return exit_status(code: 8_u8);
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
        "command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(args, cwd, files), writes(cwd, files), allocates(heap) {\n  region {\n    match arg_get(args: &args, position: 1_u64) {\n      Ok(value: text) => {\n        match relative_path(value: move text) {\n          Ok(value: path) => {\n            region 'c {\n              region {\n                match reserve_file(factory: &uniq 'c files) {\n                  Ok(value: permit) => {\n                    match open_read(permit: move permit, root: &'c cwd, path: &path) {\n                      FileOpened(value: file) => {\n",
    );
    for index in 0..count {
        source.push_str(&format!(
            "                        let bytes{index} = buffer_new(1_u64, 0_u8);\n"
        ));
    }
    source.push_str("                        region {\n                          region {\n");
    for index in 0..count {
        source.push_str(&format!(
            "                            let read{index} = read_at(file: &file, destination: &uniq bytes{index}, file_offset: 0_u64, start: 0_u64, end: 1_u64);\n"
        ));
    }
    source.push_str(
        "                          }\n                        }\n                        return exit_status(code: 0_u8);\n                      }\n                      FileOpenFailed(error: problem, permit: refused) => {\n                        return exit_status(code: 201_u8);\n                      }\n                    }\n                  }\n                  Err(error: spent) => {\n                    return exit_status(code: 8_u8);\n                  }\n                }\n              }\n            }\n          }\n          Err(error: problem) => {\n            return exit_status(code: 202_u8);\n          }\n        }\n      }\n      Err(error: problem) => {\n        return exit_status(code: 203_u8);\n      }\n    }\n  }\n}\n",
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
    // The `wf__completion_wait_core_capacity` entry is retired here with the
    // Windows verdict fork that called it, on the owner's ruling recorded in
    // design section 8: a submit answers nothing, so core pressure is the
    // target runtime's own business and never reaches emitted code.
    for declaration in [
        "declare void @wf__completion_file_pread_submit(i32, ptr, i64, i64, ptr)",
        "declare void @wf__completion_file_open_at_submit(i32, ptr, i32, i32, i32, i32, i32, ptr)",
        "declare void @wf__completion_file_join(ptr, ptr, ptr)",
    ] {
        assert!(
            module.contains(declaration),
            "Windows completion must name the native ABI `{declaration}`:\n{module}"
        );
    }
    assert!(
        !module.contains("@wf__completion_wait_core_capacity"),
        "the retired capacity wait must not be named at all:\n{module}"
    );
    assert!(
        !module.contains("define weak i32 @wf__completion_file_read_submit"),
        "a missing Windows runtime must be a link error, not a direct backend"
    );
    assert!(
        !module.contains("define weak void @wf__completion_file_join"),
        "Windows joins must not resolve to an empty optional-runtime body"
    );
}

/// Core pressure on the Windows target must never become an inline arm of the
/// emitted function.
///
/// The retry-shape assertions of
/// `windows_core_pressure_materializes_the_oldest_owned_result_and_retries`
/// are retired here on the owner's ruling recorded in design section 8: the
/// verdict fork they described is gone with the inline arm it selected, so a
/// submit answers nothing and there is no oldest owned result to materialize
/// and no retry to shape. The assertion design section 8 says **stays** is the
/// one below, and it is now the stronger statement the one lowering makes:
/// there is no second arm in the emitted function at all — the direct family
/// it would have selected is gone from the compiler.
#[test]
fn windows_core_pressure_never_becomes_inline_execution() {
    let source = more_than_target_capacity_reads(3);
    let module = emit_windows_completion(&source);
    let body = emitted_function(&module, "main");
    let submissions = body
        .matches("call void @wf__completion_file_pread_submit")
        .count();

    // Two of the three reads are handed out and submit here; the source-last
    // read submits from inside its own qualified wrapper, which is a separate
    // definition this body does not hold (design §8).
    assert_eq!(
        submissions, 2,
        "the two handed-out reads submit in the body"
    );
    assert!(
        module.contains(
            "call void @wf__completion_file_pread_submit(i32 %file, ptr %target, \
             i64 %extent, i64 %file_offset, ptr %record)"
        ),
        "the source-last read submits from its wrapper's own record"
    );
    assert!(
        !body.contains("completion.inline."),
        "core pressure must never become direct execution:\n{body}"
    );
    assert!(
        !body.contains("completion.verdict."),
        "a submit that answers nothing has no verdict to branch on:\n{body}"
    );
    assert!(
        !body.contains("completion.capacity."),
        "capacity is the target runtime's own business:\n{body}"
    );
    assert!(
        !body.contains("@wf__completion_wait_core_capacity"),
        "the emitted module must not name the retired capacity wait:\n{body}"
    );
    assert_eq!(
        body.matches("call void @wf__completion_file_join").count(),
        submissions,
        "each submitted read is joined exactly once, with no pressure-path \
         materialization beside it"
    );
    assert_eq!(
        body.matches("@wf.sys.read_at.v1(").count(),
        1,
        "only the source-last read, which was never handed out, runs directly"
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
    // Three writer-scheduler assertions retired here with the stackless plan
    // (design section 8): no module can publish a writer frame any more, so
    // there is no predicate left to ask.
    // One lowering: a source-order call and a handed-out call both submit and
    // join (`research/investigations/io-model/PARK-ON-MISS.md` §8), so the
    // question the predicate answers is only whether the module submits
    // anything at all. What still separates the two worlds is *where* the
    // record lives: the sequential module submits from inside the qualified
    // wrapper, against the block that wrapper reserves in its own frame, and
    // never from a call site against a planned-frame element.
    assert!(sequential.contains(
        "call void @wf__completion_file_write_submit(i32 %output, ptr %target, i64 %extent, \
         ptr %record)"
    ));
    for line in sequential.lines() {
        assert!(
            !line.contains("call void @wf__completion_file_write_submit(")
                || line.ends_with("ptr %record)"),
            "a sequential call submits from its wrapper's own record:\n{line}"
        );
    }
    assert!(!pure.contains("wf__completion_"));
    assert_eq!(pure, pure_sequential);
}

#[test]
fn compute_world_selection_does_not_disable_completion_io() {
    let module = super::emit_with_overlap(COMPUTE_AND_IO);
    assert!(crate::module_requires_parallel_runtime(&module));
    assert!(crate::module_requires_completion_runtime(&module));
    // Two handed-out submissions, one per compute world, and the source-last
    // call submits from inside its own qualified wrapper — one lowering, three
    // submissions (design §8).
    assert_eq!(
        module
            .matches("call void @wf__completion_file_write_submit")
            .count(),
        3,
        "both compute worlds submit the earlier I/O, and the source-last call \
         submits from its wrapper"
    );
    assert_eq!(
        module
            .matches(
                "call void @wf__completion_file_write_submit(i32 %output, ptr %target, \
                      i64 %extent, ptr %record)"
            )
            .count(),
        1,
        "exactly one of the three is the wrapper's own"
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
        .find("call void @wf__completion_file_open_at_submit")
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

/// Every route through a staged iteration writes that slot's submission state
/// before any drain can read it.
///
/// The entry-prelude `zeroinitializer` this used to pin is retired with the
/// Windows pressure fork that needed it (design section 8): nothing inspects
/// another iteration's slot any more, and both routes an iteration can take --
/// the submit and the refused component name -- store the flag on the way to
/// the drain. That is the property asserted here instead, and it is what makes
/// the pre-initialization unnecessary rather than merely absent.
#[test]
fn windows_staged_ring_initializes_submission_state_before_pressure_recovery() {
    let module = emit_windows_completion(BOUNDED_BATCH_OPENS);
    let body = emitted_function(&module, "main");
    assert!(
        !body.contains("completion.capacity.v"),
        "Windows emits no capacity-recovery path any more:\n{body}"
    );
    assert!(
        !body.contains("store [2 x i1] zeroinitializer, ptr "),
        "no route reads a submission state it did not write:\n{body}"
    );
    let submit = body
        .find("call void @wf__completion_file_open_at_submit")
        .expect("the source-derived batch submits an open");
    let refused = body
        .find("completion.not_submitted.v")
        .expect("a refused component name is the one route without a submission");
    let stored = body
        .match_indices("store i1 ")
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    assert_eq!(
        stored.len(),
        1,
        "one iteration owns one submission-state element:\n{body}"
    );
    let loaded = body
        .find("load i1, ptr ")
        .expect("the drain reads the submission state of the slot it retires");
    assert!(
        submit < stored[0] && refused < stored[0] && stored[0] < loaded,
        "both routes reach the store, and the store precedes the drain's load"
    );
    assert!(
        module.contains("declare void @abort() noreturn"),
        "the Windows floor remains fail-closed"
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
    assert!(module.contains("call ") && module.contains("@wf_test_open_at_submit("));
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
        .find("call void @wf__completion_file_write_submit")
        .expect("submit the first owned operation");
    let join = module[first..]
        .find("call void @wf__completion_file_join")
        .map(|offset| first + offset)
        .expect("the earlier operation must join after the source-last direct call");
    assert!(first < join);
    // One lowering: every call submits and joins (design §8). The call with
    // later independent work is handed out and submits at its call site; the
    // source-last call submits from inside the qualified wrapper, against the
    // record that wrapper reserved in its own frame. What the hand-out buys is
    // the distance between the submission and the join, which the two offsets
    // above are what this case reads.
    assert_eq!(
        module
            .matches("call void @wf__completion_file_write_submit")
            .count(),
        2,
        "both calls submit; only one is handed out"
    );
    assert_eq!(
        module
            .matches(
                "call void @wf__completion_file_write_submit(i32 %output, ptr %target, \
                      i64 %extent, ptr %record)"
            )
            .count(),
        1,
        "the source-last call submits from its wrapper's own record"
    );
    assert_eq!(
        module
            .matches("call void @wf__completion_file_join")
            .count(),
        2,
        "each submitted operation owns and consumes its own record"
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
    // One lowering (design §8): the handed-out read submits at its call site
    // and the source-last read submits from inside the qualified wrapper,
    // against the record that wrapper reserved in its own frame.
    assert_eq!(
        module
            .matches("call void @wf__completion_file_pread_submit")
            .count(),
        2,
        "both positioned reads submit"
    );
    assert_eq!(
        module
            .matches(
                "call void @wf__completion_file_pread_submit(i32 %file, ptr %target, \
                      i64 %extent, i64 %file_offset, ptr %record)"
            )
            .count(),
        1,
        "the source-last read submits from its wrapper's own record"
    );
    // The offset the target ABI cannot express is refused by the runtime and
    // published as the host's own `EINVAL`, so no wrapper carries a second arm
    // for it any more.
    assert!(!module.contains("%offset.fits = icmp ule i64 %file_offset"));
    assert!(crate::COMPLETION_BRIDGE_SOURCE.contains("request.kind = WF_FILE_PREAD"));
    assert!(crate::COMPLETION_BRIDGE_SOURCE.contains("file_offset > (uint64_t)INT64_MAX"));
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let pread = bridge
        .split_once("void wf__completion_file_pread_submit(")
        .expect("the bridge exposes positioned read")
        .1
        .split_once("void wf__completion_file_write_submit(")
        .expect("positioned read precedes write")
        .0;
    assert!(
        pread.find("wf_bridge_submit_linux_pread") < pread.find("request.kind = WF_FILE_PREAD"),
        "Linux must try native completion before constructing the typed fallback"
    );
    let write = bridge
        .split_once("void wf__completion_file_write_submit(")
        .expect("the bridge exposes write_once")
        .1
        .split_once("void wf__completion_file_open_at_submit(")
        .expect("ordinary write submit precedes the open submission")
        .0;
    assert!(!write.contains("wf_bridge_submit_linux"));
    assert!(write.contains("current-position and"));
    // The direct family is gone from the bridge with the second lowering that
    // named it (design §8). What executed a typed request inside the bridge
    // for it is the same engine an operation with no kernel completion form
    // still reaches, and there the host attempt is timed rather than plain, so
    // the adapter keeps measuring what its own operations cost.
    assert!(!bridge.contains("_direct("));
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
fn one_empty_write_uses_the_one_normal_operation_path() {
    let module = emit(EMPTY_WRITE);
    // An empty transfer takes the same lowering as every other: it is
    // submitted, completed by the runtime with no external action, and the
    // mapper answers `Ok(start)` for it, which is the arm read here
    // (`research/investigations/io-model/PARK-ON-MISS.md` §8).
    assert!(module.contains("%empty = icmp eq i64 %extent, 0"));
    assert!(module.contains("label %vacant, label %nonempty"));
    assert!(module.contains("call void @wf__completion_file_write_submit"));
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
/// label; naming the plain `bbN` header where the predecessor ends somewhere
/// else produced a module `clang` rejects outright. Building the executable is
/// the assertion: an invalid module never links.
///
/// The prediction this pins was `%par.done.`, which is retired here with the
/// two-arm join (design section 8): a write submits and joins in straight line
/// now, so its hand-out opens no block and the prediction is the plain header
/// again. Getting that wrong is the same link failure in the other direction,
/// so the same test still holds it.
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
                !phi.contains("%par.done."),
                "{lowering:?}: a write's one-route join opens no block of its \
                 own, so no edge can leave one: {phi}"
            );
            for edge in phi.split("], [") {
                let label = edge
                    .rsplit_once("%")
                    .expect("every phi edge names a predecessor label")
                    .1
                    .trim_end_matches([' ', ']']);
                assert!(
                    joined.contains(&format!("\n{label}:")),
                    "{lowering:?}: the predicted label `{label}` is not a block \
                     of this function: {phi}"
                );
            }
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
    // The staged tree keeps the repository's own two directories, because the
    // completion header reaches the scheduler core by the relative path it
    // uses in the tree: the completion record begins with a `wf_sched_record`.
    let units: [(&str, &str); 17] = [
        ("completion/contract.h", crate::COMPLETION_CONTRACT_HEADER),
        (
            "completion/file_adapter.h",
            crate::COMPLETION_FILE_ADAPTER_HEADER,
        ),
        ("completion/bridge.h", crate::COMPLETION_BRIDGE_HEADER),
        (
            "completion/writer_scheduler.h",
            crate::WRITER_SCHEDULER_HEADER,
        ),
        (
            "completion/linux_io_uring.h",
            crate::COMPLETION_LINUX_IO_URING_HEADER,
        ),
        ("sched/core.h", crate::SCHED_CORE_HEADER),
        ("sched/prim.h", crate::SCHED_PRIM_HEADER),
        ("sched/switch.h", crate::SCHED_SWITCH_HEADER),
        ("completion/runtime.c", crate::COMPLETION_RUNTIME_SOURCE),
        (
            "completion/file_adapter.c",
            crate::COMPLETION_FILE_ADAPTER_SOURCE,
        ),
        ("completion/bridge.c", crate::COMPLETION_BRIDGE_SOURCE),
        (
            "completion/writer_scheduler.c",
            crate::WRITER_SCHEDULER_SOURCE,
        ),
        (
            "completion/linux_io_uring.c",
            crate::COMPLETION_LINUX_IO_URING_SOURCE,
        ),
        ("sched/core.c", crate::SCHED_CORE_SOURCE),
        ("sched/prim_host.c", crate::SCHED_PRIM_HOST_SOURCE),
        ("completion/floor.c", crate::FLOOR_RUNTIME_SOURCE),
        ("completion/par_runtime.c", crate::PARALLEL_RUNTIME_SOURCE),
    ];
    for staged in ["completion", "sched"] {
        std::fs::create_dir_all(directory.join(staged)).expect("stage runtime directory");
    }
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
            .arg(directory.join("completion"))
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
    for staged in ["completion", "sched"] {
        std::fs::remove_dir(directory.join(staged)).expect("remove staged runtime directory");
    }
    std::fs::remove_dir(directory).expect("remove the default-dialect directory");
}

/// The writer-ready cells still name one capacity source, and the completion
/// runtime no longer has one at all.
///
/// This case used to tie three numbers together: the completion slot count,
/// the bridge's operation capacity, and the writer scheduler's ready cells.
/// The first two are deleted with the record pool — the record is a block of
/// the submitting frame, so there is no slot to run out of and no operation
/// capacity anywhere in the runtime
/// (`research/investigations/io-model/PARK-ON-MISS.md` §7, "The record's pool
/// machinery: deleted, not answered"). What is left is the writer scheduler's
/// own ready array, which still derives its bound from one constant, and the
/// deletion itself, which is asserted here so a capacity cannot creep back in
/// unnoticed.
#[test]
fn the_writer_ready_cells_have_one_capacity_source() {
    let header = crate::WRITER_SCHEDULER_HEADER;
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let scheduler = crate::WRITER_SCHEDULER_SOURCE;

    assert_eq!(
        header
            .matches("#define WF_COMPLETION_SLOT_CAPACITY 64u")
            .count(),
        1,
        "the writer scheduler must name its capacity once"
    );
    assert!(header.contains("#define WF_WRITER_READY_CAPACITY WF_COMPLETION_SLOT_CAPACITY"));
    assert!(!scheduler.contains("#define WF_WRITER_READY_COUNT"));
    assert!(scheduler.contains("wf_writer_ready[WF_WRITER_READY_CAPACITY]"));
    assert!(scheduler.contains("wf_writer_count == WF_WRITER_READY_CAPACITY"));

    // The bridge keeps no operation capacity, no slot array and no queue
    // array, so nothing there can refuse an operation.
    for gone in [
        "WF_BRIDGE_OPERATION_CAPACITY",
        "WF_BRIDGE_SLOT_COUNT",
        "WF_BRIDGE_QUEUE_COUNT",
        "wf_bridge_slots",
        "wf_bridge_queue",
        "wf_bridge_linux_entries",
        "wf_completion_claim",
        "WAIT_CAPACITY",
        "wf_completion_notify_capacity",
    ] {
        assert!(
            !bridge.contains(gone),
            "the bridge still names the deleted pool machinery: {gone}"
        );
    }
    assert!(
        !crate::COMPLETION_CONTRACT_HEADER.contains("wf_completion_slot"),
        "the contract header still declares a slot pool"
    );
    assert!(
        !crate::COMPLETION_CONTRACT_HEADER.contains("wf_completion_token"),
        "the contract header still declares a token"
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
        .split_once("static void wf_completion_notify_scheduler(wf_completion_runtime *runtime) {")
        .expect("completion runtime has one scheduler announcer")
        .1
        .split_once("\nuint64_t wf_completion_wake_epoch")
        .expect("announcer precedes the epoch reader")
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
        .match_indices("call void @wf__completion_file_write_submit")
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
    // One direct call, not three: A and B have no inline arm to name the
    // sequential wrapper from any more, so the only direct call left is C,
    // which was never handed out (design section 8).
    assert_eq!(
        direct_calls.len(),
        1,
        "only the call that was never handed out stays direct"
    );
    assert!(
        submits[0] < submits[1]
            && submits[1] < joins[0]
            && joins[0] < direct_calls[0]
            && direct_calls[0] < joins[1],
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
    // Sixty-five handed-out submissions, one per call with later work, plus
    // the one the qualified wrapper makes for the source-last call: one
    // lowering, so no call is left without a submission (design §8).
    assert_eq!(
        module
            .matches("call void @wf__completion_file_pread_submit")
            .count(),
        66,
        "every call submits, and the sixty-five with later work are handed out"
    );
    assert_eq!(
        module
            .matches(
                "call void @wf__completion_file_pread_submit(i32 %file, ptr %target, \
                      i64 %extent, i64 %file_offset, ptr %record)"
            )
            .count(),
        1,
        "exactly one of them is the wrapper's own"
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
    assert!(module.contains("call void @wf__completion_file_open_at_submit"));
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
    // One lowering: both modules submit and join. What separates them is where
    // the record lives — the source-order open reserves it inside the
    // qualified wrapper, the handed-out open in the submitting function's own
    // planned frame (design §8). Both reject the FIFO the same way, which is
    // what the runs below observe.
    assert_eq!(
        direct
            .matches("call void @wf__completion_file_open_at_submit")
            .count(),
        1
    );
    assert!(direct.contains("call void @wf__completion_file_open_at_submit(i32 %root, ptr %text,"));
    assert!(completion.contains("call void @wf__completion_file_open_at_submit"));
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
    // The handed-out open submits at its call site and the qualified wrapper
    // submits from its own frame: one lowering, two submissions of the one
    // route (`research/investigations/io-model/PARK-ON-MISS.md` §8).
    assert_eq!(
        module
            .matches("call void @wf__completion_file_open_at_submit")
            .count(),
        2
    );
    assert_eq!(
        module
            .matches("call void @wf__completion_file_open_at_submit(i32 %root, ptr %component,")
            .count(),
        1,
        "one of the two is the wrapper's own"
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
        .split_once("call void @wf__completion_file_open_at_submit")
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
    // The handed-out open and the qualified wrapper both submit: one lowering
    // (design §8).
    assert_eq!(
        module
            .matches("call void @wf__completion_file_open_at_submit")
            .count(),
        2
    );
    assert_eq!(
        module
            .matches("call void @wf__completion_file_open_at_submit(i32 %directory,")
            .count(),
        1,
        "one of the two is the wrapper's own"
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
    // The handed-out open and the qualified wrapper both submit: one lowering
    // (design §8).
    assert_eq!(
        module
            .matches("call void @wf__completion_file_open_at_submit")
            .count(),
        2
    );
    assert!(module.contains("@wf.sys.open_file.completion"));
    // The release closes the same way: submitted into the record the releasing
    // frame reserved, and joined there.
    assert!(module.contains("call void @wf__completion_file_close_submit(i32 %descriptor,"));
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
    // The handed-out enumeration and the qualified wrapper both submit: one
    // lowering (design §8).
    assert_eq!(
        module
            .matches("call void @wf__completion_directory_next_submit")
            .count(),
        2
    );
    assert_eq!(
        module
            .matches("call void @wf__completion_directory_next_submit(i32 %list, ptr %window,")
            .count(),
        1,
        "one of the two is the wrapper's own"
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

/// A join that cannot read its record yet waits in place: nothing runs above
/// it, and it sleeps on the one primitive rather than spinning.
///
/// The park guard this case used to pin — `wf_bridge_target_work_needs_this_thread`,
/// which refused to park while the target queue held anything — is deleted
/// with the drain it protected (design §7). The arm that replaces it is §2's
/// fourth line for an I/O target: read the record, yield through COMPLETING,
/// run one bounded progress pass, register this thread as the record's
/// in-place waiter, capture the epoch, re-check, and only then park. The
/// property that survives is the one the old guard bought — a thread with
/// nothing to do sleeps rather than spinning — and it is now a property of the
/// arm's order rather than of a predicate.
#[test]
fn a_join_waits_in_place_and_sleeps_on_the_one_primitive() {
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let arm = bridge
        .split_once("static void wf_bridge_wait_in_place(wf_completion_record *record) {")
        .expect("the I/O arm of the fourth line is one named function")
        .1
        .split_once("\n}\n")
        .expect("the arm ends with the function")
        .0;
    assert!(
        arm.contains("wf_prim_yield()"),
        "COMPLETING is DONE a few instructions away and is yielded through: {arm}"
    );
    assert!(
        arm.contains("wf_bridge_progress()"),
        "the arm makes one bounded progress pass before it sleeps: {arm}"
    );
    assert!(
        arm.contains("WF_SCHED_WAITER_IN_PLACE"),
        "the arm registers itself as the record's in-place waiter: {arm}"
    );
    assert!(
        arm.contains("wf_bridge_park(epoch)"),
        "the arm sleeps on the one primitive: {arm}"
    );
    assert!(
        arm.find("WF_SCHED_WAITER_IN_PLACE") < arm.find("wf_completion_wake_epoch"),
        "the registration goes up before the epoch is captured: {arm}"
    );
    assert!(
        arm.find("wf_completion_wake_epoch") < arm.find("wf_bridge_park(epoch)"),
        "the epoch is captured before the park: {arm}"
    );
    // The deleted guard, and the drain it protected, are gone from every site.
    for gone in [
        "wf_bridge_target_work_needs_this_thread",
        "wf_bridge_drain",
        "wf_completion_ready_event_count",
        "wf__par_help_once",
    ] {
        assert!(
            !bridge.contains(gone),
            "the bridge still names the deleted drain machinery: {gone}"
        );
    }
    // Every join runs the one arm.
    assert_eq!(
        bridge.matches("wf_bridge_wait_in_place(held)").count(),
        3,
        "each of the three joins waits in place the same way"
    );
}

/// A positioned read the submitting thread would run itself is executed there
/// and published, rather than queued.
///
/// The completion path exists so a program is not stalled by a wait it could
/// have overlapped. When the bounded adapter holds no helper, has nothing
/// queued, and has measured its own operations as not waiting, the queued read
/// would be executed by the submitting thread anyway — at its join, after a
/// queue crossing. On the `macos-14` runner that machinery is about 400 ns
/// against a warm 4 KiB read of about 1.2 us, which is why the eight-wide warm
/// program cost 41.78 ms with the pool off against 32.80 ms for the sequential
/// build of the same source.
///
/// What changed is the answer, not the question. The rule used to refuse the
/// submission and leave the caller its own direct call; there is no refusal
/// left to give, because every submit ends in a published record and the
/// emitted program has one lowering (design §7, "Every submit path ends in a
/// published record"). So the same host call is made on the same thread inside
/// submit, and the record is completed there.
///
/// Two limits are what make it safe rather than merely fast.
///
/// Only a *positioned* transfer takes it. An offset is meaningful only on a
/// seekable object and the typed opens that produce one admit nothing but a
/// regular file, so a positioned read waits on storage. A non-positioned read
/// or write may be waiting on something another part of the same program has
/// to do, and running one where it was stated could stall the thread that
/// would unblock it — which is exactly what
/// `independent_io_reaches_the_second_operation_before_the_first_unblocks`
/// pins, and it writes to a pipe.
///
/// And a written `WF_IO_HELPERS` takes nothing inline. It pins the route with
/// the count, which is what makes a pinned line of a measurement a measurement
/// of the completion path rather than of the policy that may leave it.
#[test]
fn a_positioned_read_the_submitting_thread_would_run_itself_runs_there() {
    let bridge = crate::COMPLETION_BRIDGE_SOURCE;
    let adapter = crate::COMPLETION_FILE_ADAPTER_SOURCE;

    let rule = bridge
        .split_once("static int wf_bridge_positioned_read_runs_on_caller(uint64_t count) {")
        .expect("one rule decides whether a positioned read runs on its caller")
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
        "a written helper count takes nothing inline: {rule}"
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
        "only the positioned read may run on its caller"
    );
    let pread = bridge
        .split_once("void wf__completion_file_pread_submit(")
        .expect("the bridge exposes positioned read")
        .1
        .split_once("void wf__completion_file_write_submit(")
        .expect("positioned read precedes write")
        .0;
    assert!(
        pread.find("wf_bridge_positioned_read_runs_on_caller(count)")
            < pread.find("wf_bridge_dispatch(held);\n}"),
        "the decision comes before the record reaches an engine: {pread}"
    );
    assert!(
        pread.find("wf_linux_io_uring_carries(held)")
            < pread.find("wf_bridge_positioned_read_runs_on_caller(count)"),
        "a native completion path is tried before the bounded adapter's rule"
    );
    // Whichever arm it takes, the record is published: there is no `0`.
    assert!(
        pread.contains("wf_bridge_execute_here(held);"),
        "the inline arm publishes the record rather than answering 0: {pread}"
    );
    // The one refusal a writer can spell: an offset the target ABI cannot
    // express. It is the host's own EINVAL, published into the record, and no
    // longer a reason to terminate now that no direct wrapper can take the
    // shape instead (design section 8).
    assert!(
        pread.contains("wf_bridge_complete_refused(held, EINVAL);"),
        "an offset above INT64_MAX is published as EINVAL: {pread}"
    );
    let submits = bridge
        .split_once("void wf__completion_file_read_submit(")
        .expect("the submit family starts at the plain read")
        .1
        .split_once("/* ------------------------------------------------------------ the window */")
        .expect("the submit family ends before the window query")
        .0;
    assert!(
        !submits.contains("return 0;"),
        "no submit answers 0 any more: {submits}"
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
        decision.contains("fstat(record->opened_descriptor"),
        "the kind check reads the mode of the descriptor the open produced: \
         {decision}"
    );
    // Every submit routes through one dispatcher, which asks the ring whether
    // it has a form for this kind before it reaches the bounded adapter. The
    // question is asked before the record is offered, so a kind the ring does
    // not carry is never refused after the operation was already the ring's.
    let dispatch = bridge
        .split_once("static void wf_bridge_dispatch(wf_completion_record *record) {")
        .expect("one dispatcher")
        .1
        .split_once("\n}\n")
        .expect("the dispatcher ends with the function")
        .0;
    let native = dispatch
        .find("wf_linux_io_uring_carries(record)")
        .expect("the dispatcher asks the ring first");
    let fallback = dispatch
        .find("wf_bridge_submit_file(record);")
        .expect("the dispatcher keeps the bounded adapter");
    assert!(
        native < fallback,
        "the ring is asked before the bounded POSIX adapter: {dispatch}"
    );
    assert!(
        ring.contains("int wf_linux_io_uring_carries(const wf_completion_record *record) {"),
        "the ring answers which kinds it has a form for"
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
        body.matches("call void @wf__completion_file_write_submit")
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
    // One direct call, not two: the handed-out write no longer has an inline
    // arm to name the sequential wrapper from, so the only direct call left is
    // the source-last write that was never handed out (design section 8).
    assert_eq!(
        completion_write_shape(&bound),
        (1, 1, 1),
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
