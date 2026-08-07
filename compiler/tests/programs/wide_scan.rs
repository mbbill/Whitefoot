//! Trap-identity oracle for the check-aware wide-probe lowering.
//!
//! The program below carries five recognized byte-walk loops. The
//! equivalence walks pin the exact per-byte results (needle positions at
//! every probe-lane boundary, an absent needle, and a bound below the
//! buffer length). The argument-selected hostile walks run a loop bound
//! past the buffer length and must trap at the exact byte with the exact
//! [DIAG-3] record of that walk's own `index` site: at the first byte of
//! an empty buffer, and one past the last byte at an offset inside a
//! would-be wide stride, with every pre-trap published effect identical
//! to the scalar reference. Expectations are fixed input/output pairs;
//! the compiler has no optimizer-fact channel, so this single ordinary
//! mode is the facts-off mode.

use std::os::unix::process::ExitStatusExt;

use super::support::{build_program, compile_sources, fixture_directory};

const ORACLE: &[u8] = br#"fn publish_all['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, length: own u64) -> own Result<unit, IoError> reads('o 's), writes('o), external, blocks, traps {
  doc "Publishes one prefix of the source buffer, reattempting until the host has accepted every byte or refused it.";
  let sent: own u64 = 0_u64;
  loop @publish {
    let pending: own Bool = ilt<u64>(sent, length);
    match pending {
      True() => {
      }
      False() => {
        break @publish;
      }
    }
    let remaining: own u64 = isub.wrap<u64>(length, sent);
    region 'attempt {
      match write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, offset: sent, count: remaining) {
        Ok(value: written) => {
          set sent = iadd.wrap<u64>(sent, written);
        }
        Err(error: problem) => {
          return Err(error: move problem);
        }
      }
    }
  }
  return Ok(value: unit);
}

command fn main(command.args as args: own Args, command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  doc "Runs three equivalence byte walks, publishes their recorded positions, then runs one argument-selected hostile walk that must trap at its exact byte.";
  let selector: own u8 = 111_u8;
  let choice: own buffer<u8> = buffer_new<u8>(8_u64, 0_u8);
  let chosen: own u64 = 0_u64;
  region 'pick {
    match arg_get<'pick>(args: &'pick args, position: 1_u64) {
      Ok(value: text) => {
        region 'copy {
          match host_copy_bytes<'copy, 'copy>(value: &'copy text, destination: &uniq 'copy choice, offset: 0_u64, capacity: 8_u64) {
            Ok(value: copied) => {
              set chosen = copied;
            }
            Err(error: too_small) => {
            }
          }
        }
      }
      Err(error: absent) => {
      }
    }
  }
  match ige<u64>(chosen, 1_u64) {
    True() => {
      set selector = choice[0_u64];
    }
    False() => {
    }
  }
  let data: own buffer<u8> = buffer_new<u8>(37_u64, 97_u8);
  set data[0_u64] = 88_u8;
  set data[1_u64] = 88_u8;
  set data[15_u64] = 88_u8;
  set data[16_u64] = 88_u8;
  set data[17_u64] = 88_u8;
  set data[31_u64] = 10_u8;
  set data[36_u64] = 88_u8;
  let found: own buffer<u8> = buffer_new<u8>(64_u64, 0_u8);
  let count: own u64 = 0_u64;
  let mark: own u8 = 88_u8;
  let stop: own u64 = len<u8>(data);
  let cursor: own u64 = 0_u64;
  loop @first_walk {
    let done: own Bool = ige<u64>(cursor, stop);
    match done {
      True() => {
        break @first_walk;
      }
      False() => {
      }
    }
    let byte: own u8 = data[cursor];
    let newline: own Bool = ieq<u8>(byte, 10_u8);
    match newline {
      True() => {
        match cvt<u64, u8>(cursor) {
          Ok(value: narrow) => {
            set found[count] = narrow;
            set count = iadd.wrap<u64>(count, 1_u64);
          }
          Err(error: wide_position) => {
          }
        }
      }
      False() => {
      }
    }
    let lead: own Bool = ieq<u8>(byte, mark);
    match lead {
      True() => {
        match cvt<u64, u8>(cursor) {
          Ok(value: narrow_lead) => {
            set found[count] = narrow_lead;
            set count = iadd.wrap<u64>(count, 1_u64);
          }
          Err(error: wide_lead) => {
          }
        }
      }
      False() => {
      }
    }
    set cursor = iadd.wrap<u64>(cursor, 1_u64);
  }
  set found[count] = 200_u8;
  set count = iadd.wrap<u64>(count, 1_u64);
  let blank: own buffer<u8> = buffer_new<u8>(40_u64, 97_u8);
  let blank_stop: own u64 = len<u8>(blank);
  let blank_cursor: own u64 = 0_u64;
  loop @second_walk {
    let blank_done: own Bool = ige<u64>(blank_cursor, blank_stop);
    match blank_done {
      True() => {
        break @second_walk;
      }
      False() => {
      }
    }
    let blank_byte: own u8 = blank[blank_cursor];
    let blank_newline: own Bool = ieq<u8>(blank_byte, 10_u8);
    match blank_newline {
      True() => {
        set found[count] = 210_u8;
        set count = iadd.wrap<u64>(count, 1_u64);
      }
      False() => {
      }
    }
    let blank_lead: own Bool = ieq<u8>(blank_byte, mark);
    match blank_lead {
      True() => {
        set found[count] = 211_u8;
        set count = iadd.wrap<u64>(count, 1_u64);
      }
      False() => {
      }
    }
    set blank_cursor = iadd.wrap<u64>(blank_cursor, 1_u64);
  }
  set found[count] = 201_u8;
  set count = iadd.wrap<u64>(count, 1_u64);
  let short_stop: own u64 = 20_u64;
  let short_cursor: own u64 = 0_u64;
  loop @third_walk {
    let short_done: own Bool = ige<u64>(short_cursor, short_stop);
    match short_done {
      True() => {
        break @third_walk;
      }
      False() => {
      }
    }
    let short_byte: own u8 = data[short_cursor];
    let short_newline: own Bool = ieq<u8>(short_byte, 10_u8);
    match short_newline {
      True() => {
        match cvt<u64, u8>(short_cursor) {
          Ok(value: short_narrow) => {
            set found[count] = short_narrow;
            set count = iadd.wrap<u64>(count, 1_u64);
          }
          Err(error: short_wide) => {
          }
        }
      }
      False() => {
      }
    }
    let short_lead: own Bool = ieq<u8>(short_byte, mark);
    match short_lead {
      True() => {
        match cvt<u64, u8>(short_cursor) {
          Ok(value: short_narrow_lead) => {
            set found[count] = short_narrow_lead;
            set count = iadd.wrap<u64>(count, 1_u64);
          }
          Err(error: short_wide_lead) => {
          }
        }
      }
      False() => {
      }
    }
    set short_cursor = iadd.wrap<u64>(short_cursor, 1_u64);
  }
  set found[count] = 202_u8;
  set count = iadd.wrap<u64>(count, 1_u64);
  region 'phase_publish {
    match publish_all<'phase_publish, 'phase_publish>(output: &uniq 'phase_publish out, source: &'phase_publish found, length: count) {
      Ok(value: published) => {
      }
      Err(error: problem) => {
      }
    }
  }
  match ieq<u8>(selector, 102_u8) {
    True() => {
      let empty: own buffer<u8> = buffer_new<u8>(0_u64, 0_u8);
      let empty_bound: own u64 = 5_u64;
      let empty_cursor: own u64 = 0_u64;
      loop @empty_walk {
        let empty_done: own Bool = ige<u64>(empty_cursor, empty_bound);
        match empty_done {
          True() => {
            break @empty_walk;
          }
          False() => {
          }
        }
        let empty_byte: own u8 = empty[empty_cursor];
        let empty_newline: own Bool = ieq<u8>(empty_byte, 10_u8);
        match empty_newline {
          True() => {
          }
          False() => {
          }
        }
        set empty_cursor = iadd.wrap<u64>(empty_cursor, 1_u64);
      }
    }
    False() => {
    }
  }
  match ieq<u8>(selector, 109_u8) {
    True() => {
      let field: own buffer<u8> = buffer_new<u8>(37_u64, 97_u8);
      set field[21_u64] = 88_u8;
      set field[36_u64] = 89_u8;
      let scratch: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
      let wall: own u64 = 64_u64;
      let probe: own u64 = 0_u64;
      loop @hostile_walk {
        let hostile_done: own Bool = ige<u64>(probe, wall);
        match hostile_done {
          True() => {
            break @hostile_walk;
          }
          False() => {
          }
        }
        let hostile_byte: own u8 = field[probe];
        let hostile_lead: own Bool = ieq<u8>(hostile_byte, mark);
        match hostile_lead {
          True() => {
            set scratch[0_u64] = 88_u8;
            region 'lead_write {
              match publish_all<'lead_write, 'lead_write>(output: &uniq 'lead_write out, source: &'lead_write scratch, length: 1_u64) {
                Ok(value: lead_published) => {
                }
                Err(error: lead_problem) => {
                }
              }
            }
          }
          False() => {
          }
        }
        let hostile_tail: own Bool = ieq<u8>(hostile_byte, 89_u8);
        match hostile_tail {
          True() => {
            set scratch[0_u64] = 89_u8;
            region 'tail_write {
              match publish_all<'tail_write, 'tail_write>(output: &uniq 'tail_write out, source: &'tail_write scratch, length: 1_u64) {
                Ok(value: tail_published) => {
                }
                Err(error: tail_problem) => {
                }
              }
            }
          }
          False() => {
          }
        }
        set probe = iadd.wrap<u64>(probe, 1_u64);
      }
    }
    False() => {
    }
  }
  return exit_status(code: 0_u8);
}
"#;

/// The three equivalence walks' exact published bytes: needle positions
/// 0, 1, 15, 16, 17, the newline at 31, and the last byte 36; nothing
/// from the absent-needle walk; positions below the short bound 20; one
/// marker after each walk.
const PHASE_ONE: &[u8] = &[0, 1, 15, 16, 17, 31, 36, 200, 201, 0, 1, 15, 16, 17, 202];

const RECORD_PREFIX: &str =
    "{\"rule_id\":\"OP-4\",\"message\":\"\",\"function\":\"main\",\"node_path\":[";

fn record(stderr: Vec<u8>) -> String {
    let record = String::from_utf8(stderr).expect("trap record is UTF-8");
    assert!(
        record.starts_with(RECORD_PREFIX),
        "hostile walk must report its own index site: {record}"
    );
    assert!(record.ends_with("]}\n"));
    assert_eq!(record.lines().count(), 1);
    record
}

#[test]
fn wide_probe_walks_keep_exact_results_and_exact_trap_identity() {
    let llvm = compile_sources(&[("wide_scan.wf", ORACLE)]);
    assert_eq!(
        llvm.matches("load <16 x i8>").count(),
        5,
        "all five recognized walks must carry the wide probe"
    );
    let program = build_program(&llvm);
    let directory = fixture_directory();

    let ok = program.run(directory.path(), &[]);
    assert!(ok.status.success());
    assert_eq!(ok.stdout, PHASE_ONE);
    assert!(ok.stderr.is_empty());

    let first = program.run(directory.path(), &[b"first"]);
    assert!(!first.status.success());
    assert_eq!(first.status.signal(), Some(SIGABRT));
    assert_eq!(first.stdout, PHASE_ONE);
    let first_record = record(first.stderr);

    let mid = program.run(directory.path(), &[b"mid"]);
    assert!(!mid.status.success());
    assert_eq!(mid.status.signal(), Some(SIGABRT));
    let expected: Vec<u8> = PHASE_ONE.iter().copied().chain([88_u8, 89_u8]).collect();
    assert_eq!(
        mid.stdout, expected,
        "every pre-trap hit must publish exactly as the scalar walk would"
    );
    let mid_record = record(mid.stderr);

    assert_ne!(
        first_record, mid_record,
        "the two hostile walks trap at distinct index sites"
    );
}

/// `abort` raises this signal; the harness sees it as the exit signal.
const SIGABRT: i32 = 6;
