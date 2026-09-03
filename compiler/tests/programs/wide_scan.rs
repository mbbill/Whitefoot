//! Typed-outcome oracle for proof-gated wide-probe lowering.
//!
//! The program below carries five recognized byte-walk loops. The
//! equivalence walks pin the exact per-byte results (needle positions at
//! every probe-lane boundary, an absent needle, and a bound below the
//! buffer length). The argument-selected boundary walks deliberately use
//! loop bounds past their buffers, but express exhaustion as distinct exit
//! statuses: at the first byte of an empty buffer, and one past the last
//! byte at an offset inside a would-be wide stride, with every pre-failure
//! published effect identical to the scalar reference. Expectations are
//! fixed input/output/status triples;
//! the compiler has no optimizer-fact channel, so this single ordinary
//! mode is the facts-off mode.

use super::support::{build_program, compile_sources, fixture_directory};

const ORACLE: &[u8] = br#"fn opaque_length(n: own u64) -> result: own u64 pure contract {
  requires ile(n, 0_u64);
  ensures ile(result, 0_u64);
} {
  return n;
}

fn publish_all['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, length: own u64) -> result: own Result<unit, IoError> reads(output, source), writes(output) contract {
  define source_length = len(deref(source));
  requires ile(length, source_length);
} {
  doc "Publishes one prefix of the source buffer, reattempting until the host has accepted every byte or refused it.";
  let sent = 0_u64;
  loop @publish {
    let pending = ilt(sent, length);
    if pending {
    } else {
      break @publish;
    }
    region 'attempt {
      match write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, start: sent, end: length) {
        Ok(value: next) => {
          set sent = next;
        }
        Err(error: problem) => {
          return Err<unit, IoError>(error: move problem);
        }
      }
    }
  }
  return Ok<unit, IoError>(value: unit);
}

command fn main(command.args as args: own Args, command.stdout as out: own Output) -> status: own ExitStatus reads(args, out), writes(out), allocates(heap) {
  doc "Runs three equivalence byte walks, publishes their recorded positions, then runs one argument-selected boundary walk with a typed exhaustion status.";
  let selector = 111_u8;
  let choice = buffer_new(8_u64, 0_u8);
  let chosen = 0_u64;
  region 'pick {
    match arg_get<'pick>(args: &'pick args, position: 1_u64) {
      Ok(value: text) => {
        region 'copy {
          match host_copy_bytes<'copy, 'copy>(value: &'copy text, destination: &uniq 'copy choice, start: 0_u64, end: 8_u64) {
            Ok(value: next) => {
              set chosen = next;
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
  if ige(chosen, 1_u64) {
    set selector = choice[0_u64];
  }
  let data = buffer_new(37_u64, 97_u8);
  set data[0_u64] = 88_u8;
  set data[1_u64] = 88_u8;
  set data[15_u64] = 88_u8;
  set data[16_u64] = 88_u8;
  set data[17_u64] = 88_u8;
  set data[31_u64] = 10_u8;
  set data[36_u64] = 88_u8;
  let found = buffer_new(64_u64, 0_u8);
  let count = 0_u64;
  let mark = 88_u8;
  let stop = len(data);
  let cursor = 0_u64;
  loop @first_walk {
    let done = ige(cursor, stop);
    if done {
      break @first_walk;
    }
    let byte = data[cursor];
    let newline = ieq(byte, 10_u8);
    if newline {
      match cvt<u64, u8>(cursor) {
        Ok(value: narrow) => {
          let first_newline_ok = ilt(count, 64_u64);
          if first_newline_ok {
            set found[count] = narrow;
            set count = count +wrap 1_u64;
          } else {
            return exit_status(code: 6_u8);
          }
        }
        Err(error: wide_position) => {
        }
      }
    }
    let lead = ieq(byte, mark);
    if lead {
      match cvt<u64, u8>(cursor) {
        Ok(value: narrow_lead) => {
          let first_lead_ok = ilt(count, 64_u64);
          if first_lead_ok {
            set found[count] = narrow_lead;
            set count = count +wrap 1_u64;
          } else {
            return exit_status(code: 6_u8);
          }
        }
        Err(error: wide_lead) => {
        }
      }
    }
    set cursor = cursor +wrap 1_u64;
  }
  let first_sentinel_ok = ilt(count, 64_u64);
  if first_sentinel_ok {
    set found[count] = 200_u8;
    set count = count +wrap 1_u64;
  } else {
    return exit_status(code: 6_u8);
  }
  let blank = buffer_new(40_u64, 97_u8);
  let blank_stop = len(blank);
  let blank_cursor = 0_u64;
  loop @second_walk {
    let blank_done = ige(blank_cursor, blank_stop);
    if blank_done {
      break @second_walk;
    }
    let blank_byte = blank[blank_cursor];
    let blank_newline = ieq(blank_byte, 10_u8);
    if blank_newline {
      return exit_status(code: 4_u8);
    }
    let blank_lead = ieq(blank_byte, mark);
    if blank_lead {
      return exit_status(code: 5_u8);
    }
    set blank_cursor = blank_cursor +wrap 1_u64;
  }
  let second_sentinel_ok = ilt(count, 64_u64);
  if second_sentinel_ok {
    set found[count] = 201_u8;
    set count = count +wrap 1_u64;
  } else {
    return exit_status(code: 6_u8);
  }
  let short_stop = 20_u64;
  let short_cursor = 0_u64;
  loop @third_walk {
    let short_done = ige(short_cursor, short_stop);
    if short_done {
      break @third_walk;
    }
    let short_byte = data[short_cursor];
    let short_newline = ieq(short_byte, 10_u8);
    if short_newline {
      match cvt<u64, u8>(short_cursor) {
        Ok(value: short_narrow) => {
          let short_newline_ok = ilt(count, 64_u64);
          if short_newline_ok {
            set found[count] = short_narrow;
            set count = count +wrap 1_u64;
          } else {
            return exit_status(code: 6_u8);
          }
        }
        Err(error: short_wide) => {
        }
      }
    }
    let short_lead = ieq(short_byte, mark);
    if short_lead {
      match cvt<u64, u8>(short_cursor) {
        Ok(value: short_narrow_lead) => {
          let short_lead_ok = ilt(count, 64_u64);
          if short_lead_ok {
            set found[count] = short_narrow_lead;
            set count = count +wrap 1_u64;
          } else {
            return exit_status(code: 6_u8);
          }
        }
        Err(error: short_wide_lead) => {
        }
      }
    }
    set short_cursor = short_cursor +wrap 1_u64;
  }
  let third_sentinel_ok = ilt(count, 64_u64);
  if third_sentinel_ok {
    set found[count] = 202_u8;
    set count = count +wrap 1_u64;
  } else {
    return exit_status(code: 6_u8);
  }
  let phase_room = len(found);
  let phase_fits = ile(count, phase_room);
  if phase_fits {
    region 'phase_publish {
      match publish_all<'phase_publish, 'phase_publish>(output: &uniq 'phase_publish out, source: &'phase_publish found, length: count) {
        Ok(value: published) => {
        }
        Err(error: problem) => {
        }
      }
    }
  }
  if ieq(selector, 102_u8) {
    let empty_length = opaque_length(n: 0_u64);
    let empty = buffer_new(empty_length, 0_u8);
    let empty_room = len(empty);
    let empty_bound = 5_u64;
    let empty_cursor = 0_u64;
    loop @empty_walk {
      let empty_done = ige(empty_cursor, empty_bound);
      if empty_done {
        break @empty_walk;
      }
      let empty_walk_ok = ilt(empty_cursor, empty_room);
      if empty_walk_ok {
        let empty_byte = empty[empty_cursor];
        let empty_newline = ieq(empty_byte, 10_u8);
        if empty_newline {
        }
      } else {
        return exit_status(code: 2_u8);
      }
      set empty_cursor = empty_cursor +wrap 1_u64;
    }
  }
  if ieq(selector, 109_u8) {
    let field = buffer_new(37_u64, 97_u8);
    set field[21_u64] = 88_u8;
    set field[36_u64] = 89_u8;
    let scratch = buffer_new(1_u64, 0_u8);
    let field_room = len(field);
    let wall = 64_u64;
    let probe = 0_u64;
    loop @bounded_walk {
      let walk_done = ige(probe, wall);
      if walk_done {
        break @bounded_walk;
      }
      let walk_in_range = ilt(probe, field_room);
      if walk_in_range {
        let selected_byte = field[probe];
        let selected_lead = ieq(selected_byte, mark);
        if selected_lead {
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
        let selected_tail = ieq(selected_byte, 89_u8);
        if selected_tail {
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
      } else {
        return exit_status(code: 3_u8);
      }
      set probe = probe +wrap 1_u64;
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

#[test]
fn wide_probe_walks_keep_exact_results_and_typed_boundary_failures() {
    let llvm = compile_sources(&[("wide_scan.wf", ORACLE)]);
    // The three equivalence walks keep the wide probe: their explicit
    // capacity checks sit inside hit arms, off the skip path. Every guarded
    // subscript is statically discharged from its true-edge fact. The two
    // boundary walks return typed exhaustion statuses, so the probe correctly
    // leaves them scalar [OP-4].
    assert_eq!(
        llvm.matches("load <16 x i8>").count(),
        3,
        "the equivalence walks carry the wide probe; the status-returning boundary walks stay scalar"
    );
    let program = build_program(&llvm);
    let directory = fixture_directory();

    let ok = program.run(directory.path(), &[]);
    assert!(ok.status.success());
    assert_eq!(ok.stdout, PHASE_ONE);
    assert!(ok.stderr.is_empty());

    let first = program.run(directory.path(), &[b"first"]);
    assert_eq!(first.status.code(), Some(2));
    assert_eq!(first.stdout, PHASE_ONE);
    assert!(first.stderr.is_empty());

    let mid = program.run(directory.path(), &[b"mid"]);
    assert_eq!(mid.status.code(), Some(3));
    let expected: Vec<u8> = PHASE_ONE.iter().copied().chain([88_u8, 89_u8]).collect();
    assert_eq!(
        mid.stdout, expected,
        "every pre-failure hit must publish exactly as the scalar walk would"
    );
    assert!(mid.stderr.is_empty());
}
