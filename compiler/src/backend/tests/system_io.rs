//! Observable ordinary linked-library behavior against real files and pipes.
//! C2 retires compiler qualification and implicit-release shape assertions;
//! explicit-close behavior and independent native error observations remain.

use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use super::{build_executable, compile, compile_rejection, test_directory};

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

fn io_error_classes() -> Vec<&'static str> {
    let declaration = crate::prelude::DECLARATIONS
        .iter()
        .find_map(|(_, _, source)| {
            source
                .split_once("enum IoError {\n")
                .and_then(|(_, rest)| rest.split_once("\n}").map(|(body, _)| body))
        })
        .expect("the ordinary library declares IoError");
    declaration
        .lines()
        .filter_map(|line| line.trim().split_once('(').map(|(name, _)| name))
        .collect()
}

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

const OPEN_AND_READ: &[u8] = br#"fn exercise(args: &Args, cwd: &DirectoryRead, files: &uniq HandleFactory) -> status: own ExitStatus reads(args, cwd, files), writes(files) {
  region {
    match arg_get(args: args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region {
              region {
                match open_read(factory: &uniq deref(files), root: cwd, path: &path) {
                  FileOpened(value: file) => {
                    let bytes = buffer_new(64_u64, 0_u8);
                    region {
                      region {
                        region {
                          let native_window_2 = mut_slice_of(&uniq bytes);
                          region {
                            match read_at(factory: &uniq deref(files), file: &uniq file, destination: &uniq native_window_2, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
                              Ok(value: n) => {
                                let narrowed = cvt::<u64, u8>(n);
                                match narrowed {
                                  Ok(value: code) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: code);
                                  }
                                  Err(error: overflowed) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 200_u8);
                                  }
                                }
                              }
                              Err(error: read_stop_1) => {
                                match read_stop_1 {
                                  ReadEnd() => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 201_u8);
                                  }
                                  ReadFailed(error: problem) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 202_u8);
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                  FileOpenFailed(error: problem) => {
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

fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  doc "PRE-1 ordinary Inputs are destructured once; the borrowed operation chain returns before the initial directory is explicitly closed on every exit.";
  let Inputs(args: args, cwd: cwd, stdout: unused_stdout, stderr: unused_stderr, handles: files, stdin: unused_stdin) = move inputs;
  region {
    let outcome = exercise(args: &args, cwd: &cwd, files: &uniq files);
    close_directory(factory: &uniq files, directory: move cwd);
    return move outcome;
  }
}
"#;

pub(super) const CHUNKED_READ: &[u8] = br#"fn exercise(args: &Args, cwd: &DirectoryRead, files: &uniq HandleFactory) -> status: own ExitStatus reads(args, cwd, files), writes(files) {
  region {
    match arg_get(args: args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region {
              region {
                match open_read(factory: &uniq deref(files), root: cwd, path: &path) {
                  FileOpened(value: file) => {
                    let bytes = buffer_new(3_u64, 0_u8);
                    let total = 0_u64;
                    let chunks = 0_u64;
                    let failed = False();
                    loop @drain {
                      let native_window_4 = mut_slice_of(&uniq bytes);
                      region {
                        match read_at(factory: &uniq deref(files), file: &uniq file, destination: &uniq native_window_4, file_offset: total, start: 0_u64, end: 3_u64) {
                          Ok(value: n) => {
                            set total = total +wrap n;
                            set chunks = chunks +wrap 1_u64;
                          }
                          Err(error: read_stop_3) => {
                            match read_stop_3 {
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
                    }
                    if failed {
                      close_read(factory: &uniq deref(files), file: move file);
                      return exit_status(code: 202_u8);
                    }
                    let scaled = total *wrap 10_u64;
                    let mixed = scaled +wrap chunks;
                    let narrowed = cvt::<u64, u8>(mixed);
                    match narrowed {
                      Ok(value: code) => {
                        close_read(factory: &uniq deref(files), file: move file);
                        return exit_status(code: code);
                      }
                      Err(error: overflowed) => {
                        close_read(factory: &uniq deref(files), file: move file);
                        return exit_status(code: 200_u8);
                      }
                    }
                  }
                  FileOpenFailed(error: problem) => {
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

fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  doc "PRE-1 ordinary Inputs are destructured once; the borrowed operation chain returns before the initial directory is explicitly closed on every exit.";
  let Inputs(args: args, cwd: cwd, stdout: unused_stdout, stderr: unused_stderr, handles: files, stdin: unused_stdin) = move inputs;
  region {
    let outcome = exercise(args: &args, cwd: &cwd, files: &uniq files);
    close_directory(factory: &uniq files, directory: move cwd);
    return move outcome;
  }
}
"#;

const VACANT_READ: &[u8] = br#"fn exercise(args: &Args, cwd: &DirectoryRead, files: &uniq HandleFactory) -> status: own ExitStatus reads(args, cwd, files), writes(files) {
  region {
    match arg_get(args: args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region {
              region {
                match open_read(factory: &uniq deref(files), root: cwd, path: &path) {
                  FileOpened(value: file) => {
                    let bytes = buffer_new(8_u64, 0_u8);
                    let vacant = 0_u64;
                    region {
                      region {
                        region {
                          let native_window_7 = mut_slice_of(&uniq bytes);
                          region {
                            match read_at(factory: &uniq deref(files), file: &uniq file, destination: &uniq native_window_7, file_offset: 0_u64, start: 0_u64, end: 0_u64) {
                              Ok(value: n) => {
                                set vacant = n;
                              }
                              Err(error: read_stop_5) => {
                                match read_stop_5 {
                                  ReadEnd() => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 210_u8);
                                  }
                                  ReadFailed(error: problem) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 211_u8);
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                    if vacant == 0_u64 {
                    } else {
                      close_read(factory: &uniq deref(files), file: move file);
                      return exit_status(code: 212_u8);
                    }
                    region {
                      region {
                        region {
                          let native_window_8 = mut_slice_of(&uniq bytes);
                          region {
                            match read_at(factory: &uniq deref(files), file: &uniq file, destination: &uniq native_window_8, file_offset: 0_u64, start: 0_u64, end: 8_u64) {
                              Ok(value: n) => {
                                let narrowed = cvt::<u64, u8>(n);
                                match narrowed {
                                  Ok(value: code) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: code);
                                  }
                                  Err(error: overflowed) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 200_u8);
                                  }
                                }
                              }
                              Err(error: read_stop_6) => {
                                match read_stop_6 {
                                  ReadEnd() => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 213_u8);
                                  }
                                  ReadFailed(error: problem) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 214_u8);
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                  FileOpenFailed(error: problem) => {
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

fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  doc "PRE-1 ordinary Inputs are destructured once; the borrowed operation chain returns before the initial directory is explicitly closed on every exit.";
  let Inputs(args: args, cwd: cwd, stdout: unused_stdout, stderr: unused_stderr, handles: files, stdin: unused_stdin) = move inputs;
  region {
    let outcome = exercise(args: &args, cwd: &cwd, files: &uniq files);
    close_directory(factory: &uniq files, directory: move cwd);
    return move outcome;
  }
}
"#;

const EXACT_PREFIX: &[u8] = br#"fn exercise(args: &Args, cwd: &DirectoryRead, files: &uniq HandleFactory) -> status: own ExitStatus reads(args, cwd, files), writes(files) {
  region {
    match arg_get(args: args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region {
              region {
                match open_read(factory: &uniq deref(files), root: cwd, path: &path) {
                  FileOpened(value: file) => {
                    let bytes = buffer_new(8_u64, 7_u8);
                    region {
                      region {
                        region {
                          let native_window_10 = mut_slice_of(&uniq bytes);
                          region {
                            match read_at(factory: &uniq deref(files), file: &uniq file, destination: &uniq native_window_10, file_offset: 0_u64, start: 2_u64, end: 5_u64) {
                              Ok(value: n) => {
                                if n == 5_u64 {
                                } else {
                                  close_read(factory: &uniq deref(files), file: move file);
                                  return exit_status(code: 250_u8);
                                }
                              }
                              Err(error: read_stop_9) => {
                                match read_stop_9 {
                                  ReadEnd() => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 251_u8);
                                  }
                                  ReadFailed(error: problem) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 252_u8);
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                    let digest = 0_u64;
                    let cursor = 0_u64;
                    loop @fold {
                      if cursor == 8_u64 {
                        break @fold;
                      }
                      let fold_ok = cursor < 8_u64;
                      if fold_ok {
                        let byte = bytes[cursor];
                        let widened = cvt::<u8, u64>(byte);
                        let scaled = digest *wrap 31_u64;
                        set digest = scaled +wrap widened;
                        set cursor = cursor +wrap 1_u64;
                      } else {
                        close_read(factory: &uniq deref(files), file: move file);
                        return exit_status(code: 253_u8);
                      }
                    }
                    let masked = iand(digest, 255_u64);
                    let narrowed = cvt::<u64, u8>(masked);
                    match narrowed {
                      Ok(value: code) => {
                        close_read(factory: &uniq deref(files), file: move file);
                        return exit_status(code: code);
                      }
                      Err(error: overflowed) => {
                        close_read(factory: &uniq deref(files), file: move file);
                        return exit_status(code: 200_u8);
                      }
                    }
                  }
                  FileOpenFailed(error: problem) => {
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

fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  doc "PRE-1 ordinary Inputs are destructured once; the borrowed operation chain returns before the initial directory is explicitly closed on every exit.";
  let Inputs(args: args, cwd: cwd, stdout: unused_stdout, stderr: unused_stderr, handles: files, stdin: unused_stdin) = move inputs;
  region {
    let outcome = exercise(args: &args, cwd: &cwd, files: &uniq files);
    close_directory(factory: &uniq files, directory: move cwd);
    return move outcome;
  }
}
"#;

pub(super) const WRITE_PREFIX: &[u8] = br#"fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  let Inputs(args: unused_args, cwd: unused_cwd, stdout: out, stderr: unused_stderr, handles: entry_factory, stdin: unused_stdin) = move inputs;
  region {
    close_directory(factory: &uniq entry_factory, directory: move unused_cwd);
  }
  let bytes = buffer_new(4_u64, 119_u8);
  set bytes[1_u64] = 120_u8;
  set bytes[2_u64] = 121_u8;
  set bytes[3_u64] = 122_u8;
  region {
    region {
      region {
        let native_window_11 = slice_of(&bytes);
        region {
          match write_once(factory: &uniq entry_factory, output: &uniq out, source: &native_window_11, start: 0_u64, end: 0_u64) {
            Ok(value: written) => {
              if written == 0_u64 {
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
    }
  }
  region {
    region {
      region {
        let native_window_12 = slice_of(&bytes);
        region {
          match write_once(factory: &uniq entry_factory, output: &uniq out, source: &native_window_12, start: 1_u64, end: 3_u64) {
            Ok(value: written) => {
              let narrowed = cvt::<u64, u8>(written);
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
  }
}
"#;

const OUT_OF_RANGE_WRITE: &[u8] = br#"fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  let Inputs(args: unused_args, cwd: unused_cwd, stdout: out, stderr: unused_stderr, handles: entry_factory, stdin: unused_stdin) = move inputs;
  region {
    close_directory(factory: &uniq entry_factory, directory: move unused_cwd);
  }
  let bytes = buffer_new(4_u64, 65_u8);
  region {
    region {
      region {
        let native_window_13 = slice_of(&bytes);
        region {
          match write_once(factory: &uniq entry_factory, output: &uniq out, source: &native_window_13, start: 1_u64, end: 9_u64) {
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
  }
}
"#;

const ORDERED_WRITES: &[u8] = br#"fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  let Inputs(args: unused_args, cwd: unused_cwd, stdout: out, stderr: unused_stderr, handles: entry_factory, stdin: unused_stdin) = move inputs;
  region {
    close_directory(factory: &uniq entry_factory, directory: move unused_cwd);
  }
  let bytes = buffer_new(3_u64, 65_u8);
  set bytes[1_u64] = 66_u8;
  set bytes[2_u64] = 67_u8;
  region {
    region {
      region {
        let native_window_14 = slice_of(&bytes);
        region {
          match write_once(factory: &uniq entry_factory, output: &uniq out, source: &native_window_14, start: 0_u64, end: 1_u64) {
            Ok(value: written) => {
            }
            Err(error: problem) => {
              return exit_status(code: 210_u8);
            }
          }
        }
      }
    }
  }
  region {
    region {
      region {
        let native_window_15 = slice_of(&bytes);
        region {
          match write_once(factory: &uniq entry_factory, output: &uniq out, source: &native_window_15, start: 1_u64, end: 2_u64) {
            Ok(value: written) => {
            }
            Err(error: problem) => {
              return exit_status(code: 211_u8);
            }
          }
        }
      }
    }
  }
  region {
    region {
      region {
        let native_window_16 = slice_of(&bytes);
        region {
          match write_once(factory: &uniq entry_factory, output: &uniq out, source: &native_window_16, start: 2_u64, end: 3_u64) {
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
  }
}
"#;

const COMPLETE_FIRST_SLICE: &[u8] = br#"fn exercise(args: &Args, cwd: &DirectoryRead, out: &uniq OutputStream, err: &uniq OutputStream, files: &uniq HandleFactory) -> status: own ExitStatus reads(args, cwd, out, err, files), writes(out, err, files) {
  let echo = buffer_new(64_u64, 0_u8);
  let name_length = 0_u64;
  region {
    let arguments = args_count(args: args);
    if arguments == 2_u64 {
    } else {
      return exit_status(code: 2_u8);
    }
    match arg_get(args: args, position: 1_u64) {
      Ok(value: text) => {
        region {
          set name_length = host_bytes_len(value: &text);
          region {
            region {
              let native_window_21 = mut_slice_of(&uniq echo);
              region {
                match host_copy_bytes(value: &text, destination: &uniq native_window_21, start: 0_u64, end: 64_u64) {
                  Ok(value: copied) => {
                  }
                  Err(error: problem) => {
                    return exit_status(code: 3_u8);
                  }
                }
              }
            }
          }
          match host_utf8_len(value: &text) {
            Ok(value: measured) => {
            }
            Err(error: invalid) => {
              return exit_status(code: 4_u8);
            }
          }
          region {
            region {
              let native_window_22 = mut_slice_of(&uniq echo);
              region {
                match host_copy_utf8(value: &text, destination: &uniq native_window_22, start: 0_u64, end: 64_u64) {
                  Ok(value: encoded) => {
                  }
                  Err(error: problem) => {
                    return exit_status(code: 5_u8);
                  }
                }
              }
            }
          }
        }
        match relative_path(value: move text) {
          Ok(value: path) => {
            region {
              region {
                match open_read(factory: &uniq deref(files), root: cwd, path: &path) {
                  FileOpened(value: file) => {
                    let page = buffer_new(16_u64, 0_u8);
                    let total = 0_u64;
                    let file_offset = 0_u64;
                    let failed = 0_u8;
                    loop @copy {
                      let chunk = 0_u64;
                      region {
                        region {
                          region {
                            let native_window_23 = mut_slice_of(&uniq page);
                            region {
                              match read_at(factory: &uniq deref(files), file: &uniq file, destination: &uniq native_window_23, file_offset: file_offset, start: 0_u64, end: 16_u64) {
                                Ok(value: n) => {
                                  set chunk = n;
                                  set file_offset = file_offset +wrap n;
                                }
                                Err(error: read_stop_20) => {
                                  match read_stop_20 {
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
                            }
                          }
                        }
                      }
                      let page_length = len_of(page);
                      let chunk_fits = chunk <= page_length;
                      if chunk_fits {
                      } else {
                        close_read(factory: &uniq deref(files), file: move file);
                        return exit_status(code: 12_u8);
                      }
                      region {
                        region {
                          region {
                            let native_window_24 = slice_of(&page);
                            region {
                              match write_once(factory: &uniq deref(files), output: &uniq deref(out), source: &native_window_24, start: 0_u64, end: chunk) {
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
                      }
                    }
                    if failed == 0_u8 {
                    } else {
                      close_read(factory: &uniq deref(files), file: move file);
                      return exit_status(code: failed);
                    }
                    let echo_length = len_of(echo);
                    let name_fits = name_length <= echo_length;
                    if name_fits {
                    } else {
                      close_read(factory: &uniq deref(files), file: move file);
                      return exit_status(code: 13_u8);
                    }
                    region {
                      region {
                        region {
                          let native_window_25 = slice_of(&echo);
                          region {
                            match write_once(factory: &uniq deref(files), output: &uniq deref(err), source: &native_window_25, start: 0_u64, end: name_length) {
                              Ok(value: written) => {
                                let masked = iand(total, 255_u64);
                                let narrowed = cvt::<u64, u8>(masked);
                                match narrowed {
                                  Ok(value: code) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: code);
                                  }
                                  Err(error: overflowed) => {
                                    close_read(factory: &uniq deref(files), file: move file);
                                    return exit_status(code: 200_u8);
                                  }
                                }
                              }
                              Err(error: problem) => {
                                close_read(factory: &uniq deref(files), file: move file);
                                return exit_status(code: 10_u8);
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                  FileOpenFailed(error: problem) => {
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

fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  doc "PRE-1 ordinary Inputs are destructured once; the borrowed operation chain returns before the initial directory is explicitly closed on every exit.";
  let Inputs(args: args, cwd: cwd, stdout: out, stderr: err, handles: files, stdin: unused_stdin) = move inputs;
  region {
    let outcome = exercise(args: &args, cwd: &cwd, out: &uniq out, err: &uniq err, files: &uniq files);
    close_directory(factory: &uniq files, directory: move cwd);
    return move outcome;
  }
}
"#;

#[test]
fn a_short_read_is_progress_and_only_the_observed_end_is_read_end() {
    let llvm = compile(CHUNKED_READ);
    // Five bytes in three-byte requests: three, then two, then end. The short
    // second success is progress, not end of input, and each returned absolute
    // endpoint becomes the next cursor, so the drain totals the file exactly
    // under the ordinary read_at library contract.
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

#[test]
fn a_successful_read_changes_exactly_the_requested_prefix() {
    let llvm = compile(EXACT_PREFIX);
    // On `Ok(next)` exactly `[start, next)` may have changed and every
    // other byte of the buffer is unchanged. The digest is over the
    // whole buffer, so any other write shows.
    let expected = [7_u8, 7, b'a', b'b', b'c', 7, 7, 7]
        .iter()
        .fold(0_u64, |digest, byte| {
            digest.wrapping_mul(31).wrapping_add(u64::from(*byte))
        });
    let status = u8::try_from(expected & 255).expect("the mask fits an exit code");
    assert_eq!(
        run_in_directory(&llvm, &[("five.txt", b"abcde")], &[b"five.txt"])
            .status
            .code(),
        Some(i32::from(status))
    );
}

#[test]
fn write_once_publishes_the_requested_range_and_reports_its_absolute_endpoint() {
    let llvm = compile(WRITE_PREFIX);
    let output = run_in_directory(&llvm, &[], &[]);
    // The zero-length range issued no host transfer and reported its start as
    // the endpoint; the nonempty range published exactly the requested prefix
    // of the source and reported the absolute endpoint three.
    assert_eq!(output.stdout, b"xy");
    assert_eq!(output.status.code(), Some(3));
    // C2 moves this implementation into the linked library. Force a zero-byte
    // native result and observe WriteZero, including its code and origin,
    // through the same ordinary call ABI instead of inspecting its old IR.
    super::deterministic_target::assert_zero_write_outcome();
}

#[test]
fn ordinary_output_calls_preserve_source_order() {
    let llvm = compile(ORDERED_WRITES);
    let output = run_in_directory(&llvm, &[], &[]);
    // The shared factory and output are ordinary exclusive arguments. Every
    // call returns before the next can borrow the same state.
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"ABC");
}

#[test]
fn open_read_resolves_against_its_ordinary_directory_argument() {
    let llvm = compile(OPEN_AND_READ);
    assert_eq!(
        run_in_directory(&llvm, &[("fixture.txt", b"hello")], &[b"fixture.txt"])
            .status
            .code(),
        Some(5)
    );
    assert_eq!(
        run_in_directory(
            &llvm,
            &[("inner/fixture.txt", b"hello")],
            &[b"./inner/../inner/fixture.txt"]
        )
        .status
        .code(),
        Some(5)
    );
    assert_eq!(
        run_in_directory(&llvm, &[], &[b"missing.txt"])
            .status
            .code(),
        Some(203)
    );
    assert_eq!(
        run_in_directory(&llvm, &[], &[b"/absent"]).status.code(),
        Some(204)
    );
}

#[test]
fn a_zero_length_read_transfers_nothing_and_the_following_read_still_progresses() {
    let llvm = compile(VACANT_READ);
    assert_eq!(
        run_in_directory(&llvm, &[("five.txt", b"abcde")], &[b"five.txt"])
            .status
            .code(),
        Some(5)
    );
    assert_eq!(
        run_in_directory(&llvm, &[("empty.txt", b"")], &[b"empty.txt"])
            .status
            .code(),
        Some(213)
    );
}

#[test]
fn an_out_of_range_transfer_is_an_ordinary_requirement_rejection() {
    assert_eq!(
        compile_rejection(OUT_OF_RANGE_WRITE).rule_id(),
        Some("FN-8")
    );
}

#[test]
fn the_complete_ordinary_io_chain_compiles_links_and_runs() {
    let output = run_in_directory(
        &compile(COMPLETE_FIRST_SLICE),
        &[("page.txt", b"one line and then a longer second line\n")],
        &[b"page.txt"],
    );
    assert_eq!(output.stdout, b"one line and then a longer second line\n");
    assert_eq!(output.stderr, b"page.txt");
    assert_eq!(output.status.code(), Some(39));
}

#[test]
fn explicit_close_restores_the_factory_credit_for_the_next_open() {
    let source = super::system::corpus_source("run-sysfile-close-returns-permit");
    let output = run_in_directory(&compile(&source), &[("one.txt", b"x")], &[b"one.txt"]);
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn a_refused_open_preserves_the_factory_for_the_next_open() {
    let source = super::system::corpus_source("run-sysfile-failed-open-returns-permit");
    let output = run_in_directory(
        &compile(&source),
        &[("present.txt", b"A")],
        &[b"missing.txt", b"present.txt"],
    );
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn a_closed_destination_arrives_as_a_recoverable_broken_pipe() {
    let arms = class_arms(
        14,
        &[("BrokenPipe", "set status = 42_u8;")],
        "set status = 43_u8;",
    );
    let source = format!(
        r#"fn main(inputs: own Inputs) -> result: own ExitStatus pure {{
  let Inputs(args: args, cwd: cwd, stdout: out, stderr: err, handles: factory, stdin: input) = move inputs;
  region {{
    close_directory(factory: &uniq factory, directory: move cwd);
  }}
  let bytes = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq bytes, value: 65_u8);
  }}
  let attempts = 0_u64;
  let status = 44_u8;
  region {{
    let window = slice_of(&bytes);
    loop @publish {{
      if attempts >= 200000_u64 {{
        break @publish;
      }}
      set attempts = attempts +wrap 1_u64;
      region {{
        match write_once(factory: &uniq factory, output: &uniq out, source: &window, start: 0_u64, end: 1_u64) {{
          Ok(value: written) => {{
          }}
          Err(error: problem) => {{
            match problem {{
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
    assert_eq!(
        run_with_closed_output(&compile(source.as_bytes())).code(),
        Some(42)
    );
}
