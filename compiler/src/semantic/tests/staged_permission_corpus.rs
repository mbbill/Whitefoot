//! The [PAR-3] judgment's verdict on the boundary corpus of 2026-08-27.
//!
//! The sibling file next door tests one condition at a time, against a fixture
//! written to violate exactly that condition. This file tests the judgment
//! against whole programs a boundary reviewer wrote to stress it, kept
//! verbatim, with the verdict each one must carry. Two of them were granted by
//! the first implementation and are unsound: a recurrence carried through a
//! struct field, and a `propagate` whose right-hand side is the cut itself.
//! Both were invisible to the condition-at-a-time fixtures, because none of
//! those fixtures builds a struct or a `propagate` — which is the reason this
//! file exists as a corpus rather than as more fixtures.
//!
//! Every program is here, including the ones the first implementation already
//! judged correctly. A corpus that kept only the failures would not notice a
//! repair that traded one widening for an over-denial somewhere else, and the
//! repair that closed these two — reading every place through its [OWN-7]
//! overlap class instead of its exact path — is exactly the kind of change
//! that could.
//!
//! Programs the checker rejects for a reason that is not this judgment's are
//! recorded as [`Outcome::Rejected`] rather than dropped, so a later change
//! that makes one of them check has to come back here and give it a verdict.
//!
//! Every program of the review that judges a loop is here. The review also
//! wrote three bare language probes — a struct field of buffer type, a `deref`
//! of a borrowed field, a `deref` of a borrowed scalar — which hold no loop and
//! therefore carry no staged verdict to keep.

use crate::SemanticOutcome;

use super::super::staged_permission::{StagedPermission, StagedVerdict};
use super::with_semantics;

/// The verdict one staged loop must carry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Expected {
    Permitted,
    /// Denied, by this numbered condition of the rule.
    Denied(u8),
}

/// What one program of the corpus must produce.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Outcome {
    /// The verdict of every staged loop of the named function, in source
    /// order. Empty when that function holds no loop that performs I/O.
    Staged(&'static [Expected]),
    /// The source does not check at all, for a reason unrelated to this
    /// judgment, so there is no staged table to read.
    Rejected,
}

/// One program of the corpus.
struct CorpusCase {
    /// The reviewer's own file name, so a failure names the program a reader
    /// can go and look at.
    name: &'static str,
    source: &'static [u8],
    /// The function whose loops carry the verdict.
    function: &'static str,
    outcome: Outcome,
}

const A01_BASELINE: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Baseline: the granted shape.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    let data = buffer_new(64_u64, 0_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'h {
              region {
                match read_at(file: &'h handle, destination: &uniq data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
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
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A02_HOISTED_SCRATCH: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Design 2.3: the destination buffer is hoisted above the loop, so one iteration's short read leaves the previous iteration's bytes behind it.";
  let data = buffer_new(64_u64, 0_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'h {
              region {
                match read_at(file: &'h handle, destination: &uniq data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
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
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A03_CARRIED_BYTE: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The name buffer is hoisted above the loop and mutated in the remainder, so iteration i+1 opens a name iteration i wrote.";
  let name = buffer_new(16_u64, 97_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
        Ok(value: handle) => {
          set total = total +wrap 1_u64;
        }
        Err(error: problem) => {
        }
      }
    }
    set name[0_u64] = 98_u8;
  }
  return exit_status(code: 0_u8);
}
"#;

const A04_FOLD_BEFORE_READ: &[u8] = br#"fn fold_prefix(source: &buffer<u8>, produced: own u64, seed: own u64) -> result: own u64 reads(source) {
  doc "Folds one read prefix into a running order-sensitive checksum.";
  let spare = len_of(deref(source));
  let sum = seed;
  let at = 0_u64;
  loop @fold {
    let scanned = at >= produced;
    if scanned {
      break @fold;
    }
    let readable = at < spare;
    if readable {
    } else {
      break @fold;
    }
    let byte = deref(source)[at];
    let widened = cvt::<u8, u64>(byte);
    set sum = sum *wrap 31_u64;
    set sum = sum +wrap widened;
    set at = at +wrap 1_u64;
  }
  return sum;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Design 2.5: the fold reads the hoisted destination before this iteration's transfer writes it, so every byte it reads is the previous iteration's.";
  let data = buffer_new(64_u64, 0_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region {
      let digest = fold_prefix(source: &data, produced: 64_u64, seed: 0_u64);
      set total = total +wrap digest;
    }
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'h {
              region {
                match read_at(file: &'h handle, destination: &uniq data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
                  ReadBytes(next: produced) => {
                  }
                  ReadEnd() => {
                  }
                  ReadFailed(error: problem) => {
                  }
                }
              }
            }
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

const A05_RETURN_IN_REMAINDER: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The body returns from the remainder, after later iterations have already submitted opens the source-order execution never performs.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
          }
          Err(error: problem) => {
            return exit_status(code: 4_u8);
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A06_BREAK_ENCLOSING: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The remainder breaks out of a loop enclosing the staged loop.";
  let total = 0_u64;
  loop @outer {
    for @scan (index in 0_u64..4_u64) {
      let name = buffer_new(16_u64, 97_u8);
      region 'f {
        let permit = reserve_file(factory: &uniq files);
        region {
          match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
            Ok(value: handle) => {
              break @outer;
            }
            Err(error: problem) => {
            }
          }
        }
      }
    }
    break @outer;
  }
  return exit_status(code: 0_u8);
}
"#;

const A07_DIRECTORY_SOURCE: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A retained exclusive loan on an enclosing DirectorySource cursor.";
  let total = 0_u64;
  region {
    let permit = reserve_file(factory: &uniq files);
    match open_directory_source(permit: move permit, directory: &cwd) {
      Ok(value: list) => {
        for @scan (index in 0_u64..4_u64) {
          let entries = buffer_new(1024_u64, 0_u8);
          region {
            match directory_next(source: &uniq list, destination: &uniq entries, start: 0_u64, end: 1024_u64) {
              ListBytes(next: bytes, entries: reported) => {
                set total = total +wrap reported;
              }
              ListEnd() => {
              }
              ListFailed(error: problem) => {
              }
            }
          }
        }
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A08_READONLY_NAME: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A may-suspend call retains a shared borrow of an enclosing buffer the body never writes.";
  let name = buffer_new(16_u64, 97_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
        Ok(value: handle) => {
          set total = total +wrap 1_u64;
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A09_REMAINDER_CURSOR: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The remainder reads an enclosing cursor to pick the file offset of its second submission and then overwrites that cursor from the loop binder alone, so the final value matches source order whatever the schedule while the offset the host reads at does not.";
  let cursor = 0_u64;
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    let data = buffer_new(64_u64, 0_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let at = cursor;
            let stride = index *wrap 64_u64;
            set cursor = stride;
            region 'h {
              region {
                match read_at(file: &'h handle, destination: &uniq data, file_offset: at, start: 0_u64, end: 64_u64) {
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
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A10_PROLOGUE_ACCUMULATOR: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The accumulator is written in the prologue only. Prologues run in index order and never overlap, so the sum is the source-order sum.";
  let attempted = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    set attempted = attempted +wrap 1_u64;
    let name = buffer_new(16_u64, 97_u8);
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

const A12_NESTED_INNER_IO: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Nested loops with the inner loop doing the I/O: the outer body has no single cut, the inner loop is judged on its own terms.";
  let total = 0_u64;
  for @outer (step in 0_u64..2_u64) {
    let shared = buffer_new(16_u64, 97_u8);
    for @scan (index in 0_u64..4_u64) {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &cwd, name: &shared, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
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

const A13_PROOF_REMAINDER: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A checked local invariant in the remainder erases before execution and does not narrow staged permission.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            invariant bounded: index <= 4_u64;
            set total = total +wrap 1_u64;
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

const A13A_REMAINDER_PROLOGUE: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "An automatically proved remainder bound admits a prologue subscript before staged permission computes its footprint.";
  let total = 0_u64;
  let table = array_new::<u8, 8>(3_u8);
  for @scan (index in 0_u64..4_u64) {
    let seed = index *wrap 3_u64;
    let slot = seed % 8_u64;
    let picked = table[slot];
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
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

// This paired case uses a dominating branch instead of the automatic remainder
// interval. Both proof routes must produce the same staged footprint.
const A13C_PROVED_PROLOGUE: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A branch-proved subscript in the prologue. Permission is determined by its checked footprint, not by the proof route that admitted the partial operation.";
  let total = 0_u64;
  let table = array_new::<u8, 8>(3_u8);
  for @scan (index in 0_u64..4_u64) {
    let seed = index *wrap 3_u64;
    let slot = seed % 8_u64;
    invariant two_steps: 0_u64 <= 2_u64;
    if slot < 8_u64 {
      let picked = table[slot];
    }
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
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

const A13B_PROOF_REMAINDER_STORAGE: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A checked local invariant in the remainder may mention locally constructed storage facts without changing staged permission.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let slot = buffer_new(8_u64, 0_u8);
            let spare = len_of(slot);
            invariant fixed_step: 0_u64 <= 2_u64;
            set total = total +wrap 1_u64;
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

const A14_INTERPOSED: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "An ordinary statement written between the submission and the statement that consumes its outcome. The judgment cuts at the submission statement, so the interposed statement is in the remainder.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    let data = buffer_new(64_u64, 0_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let squared = index *wrap index;
            set total = total +wrap squared;
            region 'h {
              region {
                match read_at(file: &'h handle, destination: &uniq data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
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
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A15_BODY_BOUND_BORROW: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A borrow of enclosing storage bound to a body-introduced name, then handed to the submission. If the judgment read the binding rather than its referent, the enclosing buffer would carry no disposition at all.";
  let name = buffer_new(16_u64, 97_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      let borrowed = &name;
      match open_file(permit: move permit, root: &cwd, name: borrowed, start: 0_u64, end: 0_u64) {
        Ok(value: handle) => {
          set total = total +wrap 1_u64;
        }
        Err(error: problem) => {
        }
      }
    }
    set name[0_u64] = 98_u8;
  }
  return exit_status(code: 0_u8);
}
"#;

const A16_GIVE_OUT: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A match arm of the remainder gives out of the loop, delivering to a value initializer written outside it.";
  let seed = Some<u64>(value: 1_u64);
  let picked = match seed {
    Some(value: carried) => {
      for @scan (index in 0_u64..4_u64) {
        let name = buffer_new(16_u64, 97_u8);
        region 'f {
          let permit = reserve_file(factory: &uniq files);
          region {
            match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
              Ok(value: handle) => {
                give 7_u64;
              }
              Err(error: problem) => {
              }
            }
          }
        }
      }
      return exit_status(code: 9_u8);
    }
    None() => {
      give 0_u64;
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A17_NO_CLEAN_CUT: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The submission is written inside one branch, and a statement after the branch is neither before it on every path nor reached only through it.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    let first = index == 0_u64;
    if first {
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
    set total = total +wrap 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#;

const A18_FIELD_ALIAS: &[u8] = br#"struct Work {
  seen: u64;
  code: u64;
}

fn probe(w: &Work, root: &DirectoryRead, name: &buffer<u8>, permit: own FilePermit) -> result: own Result<ReadFile, IoError> reads(w, root, name, permit), writes(permit) {
  doc "Opens a prefix of the name whose length is the carried count.";
  let n = deref(w).seen;
  return open_file(permit: move permit, root: root, name: name, start: 0_u64, end: n);
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The submission reads the whole carried record; the remainder writes one field of it. The two are the same storage.";
  let work = Work(seen: 1_u64, code: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region 'w {
        region {
          match probe(w: &'w work, root: &'f cwd, name: &name, permit: move permit) {
            Ok(value: handle) => {
              set work.seen = work.seen +wrap 1_u64;
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

const A19_FIELD_RECURRENCE: &[u8] = br#"struct Work {
  seen: u64;
  code: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The carried count is read in the prologue as a field and rewritten in the remainder as the whole record. Sequentially work.seen takes 0,1,2,3; with prologues running ahead of remainders every iteration reads the same value.";
  let work = Work(seen: 0_u64, code: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    let carried = work.seen;
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let bumped = carried +wrap 1_u64;
            let previous = replace work = Work(seen: bumped, code: 0_u64);
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

const A19B_CONTROL_SCALAR: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Control for A19: the identical recurrence carried in a bare u64 instead of a struct field.";
  let seen = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let carried = seen;
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let bumped = carried +wrap 1_u64;
            set seen = bumped;
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

const A19C_OBSERVABLE: &[u8] = br#"struct Work {
  seen: u64;
  code: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The carried count selects the name prefix the open uses, so the divergence reaches the host: sequentially the four opens name four different prefixes, pipelined they name one.";
  let work = Work(seen: 0_u64, code: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    let carried = work.seen;
    let short = carried < 4_u64;
    if short {
    } else {
      break @scan;
    }
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: carried, end: 8_u64) {
          Ok(value: handle) => {
            let bumped = carried +wrap 1_u64;
            let previous = replace work = Work(seen: bumped, code: 0_u64);
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

const A20_PROPAGATE_CUT: &[u8] = br#"fn scan_all(cwd: &DirectoryRead, files: own FileFactory) -> result: own Result<u64, IoError> reads(cwd, files), writes(files) {
  doc "The submission statement is itself the exit: propagate leaves the loop and the function on the operation's own Err outcome.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region {
      let permit = reserve_file(factory: &uniq files);
      region {
        let handle = propagate open_file(permit: move permit, root: cwd, name: &name, start: 0_u64, end: 4_u64);
        set total = total +wrap 1_u64;
      }
    }
  }
  return Ok<u64, IoError>(value: total);
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Drives the propagating scan.";
  region {
    match scan_all(cwd: &cwd, files: move files) {
      Ok(value: counted) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A20B_MATCH_TWIN: &[u8] = br#"fn scan_all(cwd: &DirectoryRead, files: own FileFactory) -> result: own Result<u64, IoError> reads(cwd, files), writes(files) {
  doc "The same exit as A20, spelled as a match arm instead of a propagate.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
          }
          Err(error: problem) => {
            return Err<u64, IoError>(error: move problem);
          }
        }
      }
    }
  }
  return Ok<u64, IoError>(value: total);
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Drives the matching scan.";
  region {
    match scan_all(cwd: &cwd, files: move files) {
      Ok(value: counted) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A20C_PROPAGATE_SECOND: &[u8] = br#"fn scan_all(cwd: &DirectoryRead, files: own FileFactory) -> result: own Result<u64, IoError> reads(cwd, files), writes(files) {
  doc "A propagate on a second submission, written in the remainder.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region {
              let again = reserve_file(factory: &uniq files);
              region {
                let second = propagate open_file(permit: move again, root: cwd, name: &name, start: 0_u64, end: 4_u64);
                set total = total +wrap 1_u64;
              }
            }
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return Ok<u64, IoError>(value: total);
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Drives the scan.";
  region {
    match scan_all(cwd: &cwd, files: move files) {
      Ok(value: counted) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A22_EXPR_STATEMENT: &[u8] = br#"fn stamp(slot: &uniq MutSlice<u8>, index: own u64) -> result: own unit reads(slot), writes(slot) {
  doc "Writes one byte of the borrowed slot.";
  let spare = len_of(deref(slot));
  let wide = 0_u64 < spare;
  if wide {
    set deref(slot)[0_u64] = 7_u8;
  }
  return unit;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "An expression statement in the prologue.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region {
      let window = mut_slice_of(&uniq name);
      region {
        stamp(slot: &uniq window, index: index);
      }
    }
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
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

const A23_GIVE_INSIDE: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A value initializer written inside the remainder: its gives deliver to a binding of the same iteration and leave nothing.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let weight = match Some<u64>(value: 2_u64) {
              Some(value: carried) => {
                give carried;
              }
              None() => {
                give 0_u64;
              }
            }
            set total = total +wrap weight;
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

const A24_SLICE_READONLY: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "A shared slice of an enclosing buffer the body never writes.";
  let table = buffer_new(16_u64, 97_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region {
      let view = slice_of(&table);
      let seen = len_of(view);
      set total = total +wrap seen;
    }
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

const A25_LOAN_EXTENT: &[u8] = br#"command fn main(command.files as files: own FileFactory) -> status: own ExitStatus reads(files), writes(files) {
  doc "Discriminator: two reserve_file calls inside one region. If the unique factory loan lasted the region rather than the call, the second would be an OWN-5 overlap.";
  region {
    let first = reserve_file(factory: &uniq files);
    let second = reserve_file(factory: &uniq files);
  }
  return exit_status(code: 0_u8);
}
"#;

const A26_STRUCT_NAME_SWAP: &[u8] = br#"struct Holder {
  name: buffer<u8>;
  seen: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The name the submission opens is a field of a record the remainder replaces wholesale. Sequentially iteration 0 opens one name and iterations 1 to 3 open another; with prologues running ahead of remainders all four open the first.";
  let seed = buffer_new(16_u64, 97_u8);
  let held = Holder(name: move seed, seen: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &held.name, start: 0_u64, end: 0_u64) {
        Ok(value: handle) => {
          let fresh = buffer_new(16_u64, 98_u8);
          let previous = replace held = Holder(name: move fresh, seen: 1_u64);
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const A27_OUTPUT_WRITE: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, out, files), writes(cwd, out, files) {
  doc "The remainder writes an enclosing Output. Two remainders coexist, so the bytes reaching the stream would not be in iteration order.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    let line = buffer_new(8_u64, 65_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'o {
              region {
                match write_once(output: &uniq 'o out, source: &line, start: 0_u64, end: 8_u64) {
                  Ok(value: written) => {
                    set total = total +wrap written;
                  }
                  Err(error: problem) => {
                  }
                }
              }
            }
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

const A28_NESTED_FIELD: &[u8] = br#"struct Inner {
  a: u64;
  b: u64;
}

struct Outer {
  inner: Inner;
  tag: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The prologue reads a doubly nested field and the remainder replaces its parent.";
  let start = Inner(a: 0_u64, b: 0_u64);
  let carrier = Outer(inner: move start, tag: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    let carried = carrier.inner.a;
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let bumped = carried +wrap 1_u64;
            let replacement = Inner(a: bumped, b: 0_u64);
            let previous = replace carrier.inner = move replacement;
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

const A29_TWO_SUBMISSIONS: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "Two submissions on disjoint branches: neither is a single cut.";
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    let first = index == 0_u64;
    if first {
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
    } else {
      region 'g {
        let other = reserve_file(factory: &uniq files);
        region {
          match open_file(permit: move other, root: &'g cwd, name: &name, start: 0_u64, end: 4_u64) {
            Ok(value: handle) => {
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

const A34_MIRROR_PROLOGUE_WRITE: &[u8] = br#"struct Carrier {
  tag: u64;
  spare: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  doc "The mirror of A19: the prologue replaces the whole record and the remainder reads one of its fields. Sequentially the remainder of iteration i reads the tag its own prologue wrote; with prologues running ahead it reads a later iteration's.";
  let carrier = Carrier(tag: 0_u64, spare: 0_u64);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let previous = replace carrier = Carrier(tag: index, spare: 0_u64);
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let seen = carrier.tag;
            set total = total +wrap seen;
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

/// Every program the review of 2026-08-27 ran, with the verdict the rule text
/// gives it.
///
/// The five entries the first implementation got wrong are the field
/// recurrence and its two variants (A19, A19c, A28), its mirror (A34), the
/// retained borrow into replaced storage (A26), and the `propagate` at the cut
/// (A20). Each of them sits beside the control that proved it wrong: A19b is
/// the same recurrence in a bare `u64`, which was always denied, and A20b is
/// the same exit written as a `match` arm, which was always denied.
const CORPUS: &[CorpusCase] = &[
    // The shape the rule exists for: one file per iteration, everything the
    // body writes either iteration-own or confined to one segment.
    CorpusCase {
        name: "A01-baseline.wf",
        source: A01_BASELINE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // A scratch buffer hoisted out of the loop, borrowed by the submission and
    // written by the body: condition 3's central case.
    CorpusCase {
        name: "A02-hoisted-scratch.wf",
        source: A02_HOISTED_SCRATCH,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(3)]),
    },
    // The same hazard hidden in one carried byte of an enclosing name buffer.
    CorpusCase {
        name: "A03-carried-byte.wf",
        source: A03_CARRIED_BYTE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(3)]),
    },
    // A fold over the enclosing destination before the read that fills it.
    CorpusCase {
        name: "A04-fold-before-read.wf",
        source: A04_FOLD_BEFORE_READ,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(3)]),
    },
    // A `return` after the submission: the decision to leave would be taken
    // after later prologues already submitted.
    CorpusCase {
        name: "A05-return-in-remainder.wf",
        source: A05_RETURN_IN_REMAINDER,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(2)]),
    },
    // Two loops in source order: the enclosing `loop @outer`, whose first
    // submission is written inside a loop of its own body and so has no cut,
    // and the inner `for @scan`, from whose remainder a `break` naming the
    // enclosing loop leaves.
    CorpusCase {
        name: "A06-break-enclosing.wf",
        source: A06_BREAK_ENCLOSING,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(1), Expected::Denied(2)]),
    },
    // A retained exclusive loan on an enclosing enumeration cursor, which no
    // replication ever repairs: the denial says so instead of offering the
    // per-iteration form as advice.
    CorpusCase {
        name: "A07-directory-source.wf",
        source: A07_DIRECTORY_SOURCE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(3)]),
    },
    // A retained *shared* borrow of enclosing storage the body never writes is
    // the read-only disposition, and it is granted.
    CorpusCase {
        name: "A08-readonly-name.wf",
        source: A08_READONLY_NAME,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // A cursor the remainder reads and then overwrites. It is granted, and the
    // rule owes it the read half of the remainder's index ordering: without
    // that sentence E(i) could read what E(j) has not yet written.
    CorpusCase {
        name: "A09-remainder-cursor.wf",
        source: A09_REMAINDER_CURSOR,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // An accumulator written in the prologue alone: serialized there, because
    // prologues never overlap.
    CorpusCase {
        name: "A10-prologue-accumulator.wf",
        source: A10_PROLOGUE_ACCUMULATOR,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // Two nested loops: the outer one's first submission is written inside the
    // inner loop, so the outer has no cut and the inner is judged on its own.
    CorpusCase {
        name: "A12-nested-inner-io.wf",
        source: A12_NESTED_INNER_IO,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(1), Expected::Permitted]),
    },
    // A checked source proof in the remainder erases before permission and
    // leaves the permitted overlap unchanged.
    CorpusCase {
        name: "A13-proof-remainder.wf",
        source: A13_PROOF_REMAINDER,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // The deterministic remainder interval authorizes the load-bearing
    // subscript before staged permission computes its footprint.
    CorpusCase {
        name: "A13a-remainder-prologue.wf",
        source: A13A_REMAINDER_PROLOGUE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    CorpusCase {
        name: "A13b-proof-remainder-storage.wf",
        source: A13B_PROOF_REMAINDER_STORAGE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // The same prologue computation with an ordinary dominating proof remains
    // a real accepted PAR-3 case.
    CorpusCase {
        name: "A13c-proved-prologue.wf",
        source: A13C_PROVED_PROLOGUE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // A statement written between the submission and the join is judged like
    // any other statement of the remainder.
    CorpusCase {
        name: "A14-interposed.wf",
        source: A14_INTERPOSED,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // A borrow of enclosing storage bound by a body statement rather than
    // written as a call argument carries no stateable loan, so the form is
    // refused rather than read as loan-free.
    CorpusCase {
        name: "A15-body-bound-borrow.wf",
        source: A15_BODY_BOUND_BORROW,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(7)]),
    },
    // A `give` delivering to an initializer written outside the loop leaves
    // it, and it is written after the submission.
    CorpusCase {
        name: "A16-give-out.wf",
        source: A16_GIVE_OUT,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(2)]),
    },
    // A body with no single-entry single-exit cut at its first submission.
    CorpusCase {
        name: "A17-no-clean-cut.wf",
        source: A17_NO_CLEAN_CUT,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(1)]),
    },
    // The coarse `reads(w)` row this program declares is refused by [EFF-2]
    // before this judgment runs; A19 is the same boundary case with a row [EFF-2]
    // accepts.
    CorpusCase {
        name: "A18-field-alias.wf",
        source: A18_FIELD_ALIAS,
        function: "main",
        outcome: Outcome::Rejected,
    },
    // THE FIRST WIDENING. `work.seen` is read in the prologue and `work` is
    // replaced in the remainder. Keyed by exact path the two are independent
    // rows, each with a safe disposition; under [OWN-7] they are one storage
    // the body reaches on both sides of the cut, and condition 5 denies.
    CorpusCase {
        name: "A19-field-recurrence.wf",
        source: A19_FIELD_RECURRENCE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(5)]),
    },
    // The control that proved it: the byte-identical recurrence carried in a
    // bare `u64`, which the first implementation already denied.
    CorpusCase {
        name: "A19b-control-scalar.wf",
        source: A19B_CONTROL_SCALAR,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(5)]),
    },
    // The same recurrence feeding the submission's `start` argument, so the
    // divergence a grant would admit reaches the host.
    CorpusCase {
        name: "A19c-observable.wf",
        source: A19C_OBSERVABLE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(5)]),
    },
    // THE SECOND WIDENING. The `propagate` whose right-hand side is the cut:
    // its `Err` edge is selected by the submission's own outcome, so it is an
    // edge of the remainder however the statement's footprint is segmented.
    CorpusCase {
        name: "A20-propagate-cut.wf",
        source: A20_PROPAGATE_CUT,
        function: "scan_all",
        outcome: Outcome::Staged(&[Expected::Denied(2)]),
    },
    // The control that proved it: the same exit written as a `match` arm,
    // which the first implementation already denied.
    CorpusCase {
        name: "A20b-match-twin.wf",
        source: A20B_MATCH_TWIN,
        function: "scan_all",
        outcome: Outcome::Staged(&[Expected::Denied(2)]),
    },
    // A `propagate` on a second submission, which is unambiguously in the
    // remainder.
    CorpusCase {
        name: "A20c-propagate-second.wf",
        source: A20C_PROPAGATE_SECOND,
        function: "scan_all",
        outcome: Outcome::Staged(&[Expected::Denied(2)]),
    },
    // An over-denial the rule sanctions: an expression statement anywhere in
    // the body refuses the loop, because its reach projects onto no actual.
    CorpusCase {
        name: "A22-expr-statement.wf",
        source: A22_EXPR_STATEMENT,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(7)]),
    },
    // A `give` delivering to an initializer written inside the loop reaches a
    // binding of the same iteration and leaves nothing.
    CorpusCase {
        name: "A23-give-inside.wf",
        source: A23_GIVE_INSIDE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Permitted]),
    },
    // The second sanctioned over-denial: a slice of enclosing storage the body
    // never writes is a footprint element this judgment does not resolve.
    CorpusCase {
        name: "A24-slice-readonly.wf",
        source: A24_SLICE_READONLY,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(7)]),
    },
    // The discriminator behind serialized-P: two `reserve_file` calls in one
    // region are accepted, so the factory's unique loan is call-scoped rather
    // than region-scoped. It holds no loop, so it carries no staged verdict —
    // its content is that it checks at all.
    CorpusCase {
        name: "A25-loan-extent.wf",
        source: A25_LOAN_EXTENT,
        function: "main",
        outcome: Outcome::Staged(&[]),
    },
    // THE SEVERE FORM OF THE FIRST WIDENING. The submission borrows
    // `held.name`, and the remainder drops that buffer by replacing `held`
    // while a later iteration's open is still outstanding on it. This is the
    // precise hazard condition 3 exists to prevent.
    CorpusCase {
        name: "A26-struct-name-swap.wf",
        source: A26_STRUCT_NAME_SWAP,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(3)]),
    },
    // A `write_once` to an enclosing Output from the remainder.
    CorpusCase {
        name: "A27-output-write.wf",
        source: A27_OUTPUT_WRITE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(3)]),
    },
    // The first widening at nesting depth two: `carrier.inner.a` read in the
    // prologue, `carrier.inner` replaced in the remainder. The overlap
    // relation is a prefix test, so depth costs it nothing.
    CorpusCase {
        name: "A28-nested-field.wf",
        source: A28_NESTED_FIELD,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(5)]),
    },
    // Two submissions on disjoint branches: neither is a cut.
    CorpusCase {
        name: "A29-two-submissions.wf",
        source: A29_TWO_SUBMISSIONS,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(1)]),
    },
    // The mirror of the first widening, which is why the repair is stated over
    // the class and not over one disposition: `carrier` is replaced in the
    // prologue and `carrier.tag` is read in the remainder, so the unsound
    // grant came out as serialized-P rather than as read-only.
    CorpusCase {
        name: "A34-mirror-prologue-write.wf",
        source: A34_MIRROR_PROLOGUE_WRITE,
        function: "main",
        outcome: Outcome::Staged(&[Expected::Denied(5)]),
    },
];

/// Every program of the corpus, judged in one pass.
///
/// The drifts are collected rather than asserted one at a time, so a change
/// that moves several verdicts reports all of them at once instead of hiding
/// the rest behind the first.
#[test]
fn the_judgment_holds_its_verdict_on_the_whole_boundary_corpus() {
    let drifts: Vec<String> = CORPUS.iter().filter_map(drift).collect();
    assert!(
        drifts.is_empty(),
        "the staged judgment drifted on {} of {} programs:\n{}",
        drifts.len(),
        CORPUS.len(),
        drifts.join("\n")
    );
}

/// How one program's verdict differs from the one the rule gives it, when it
/// differs.
fn drift(case: &CorpusCase) -> Option<String> {
    with_semantics(case.source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            return (case.outcome != Outcome::Rejected)
                .then(|| format!("{}: the source no longer checks", case.name));
        };
        let Outcome::Staged(expected) = case.outcome else {
            return Some(format!(
                "{}: the source now checks, and owes a staged verdict",
                case.name
            ));
        };
        let judged: Vec<&StagedPermission> = program
            .data
            .permission
            .named(case.function)
            .unwrap_or_else(|| panic!("no permission table for {} of {}", case.function, case.name))
            .staged
            .iter()
            .collect();
        if judged.len() != expected.len() {
            return Some(format!(
                "{}: {} staged loops, expected {}",
                case.name,
                judged.len(),
                expected.len()
            ));
        }
        for (index, (judgement, want)) in judged.iter().zip(expected).enumerate() {
            let held = match &judgement.verdict {
                StagedVerdict::Permitted => Expected::Permitted,
                StagedVerdict::Denied(denial) => Expected::Denied(denial.condition()),
            };
            if held != *want {
                return Some(format!(
                    "{} loop {index}: {held:?}, expected {want:?} ({:?})",
                    case.name, judgement.verdict
                ));
            }
        }
        None
    })
}
