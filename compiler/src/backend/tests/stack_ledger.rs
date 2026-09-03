//! The stack ledger against the machine it describes.
//!
//! The ledger's whole value is that its numbers are true of the binary. A
//! frame size is a whole-program, optimizer-chosen property — adding one
//! shallow call somewhere else in a program has been measured to double the
//! frame of a recursion it never touches — so nothing about the source says
//! what a level costs, and a ledger nobody checks is a ledger nobody should
//! believe.
//!
//! So the case below does not check that the ledger parsed its input. It takes
//! the depth the ledger says the runtime's stack holds, builds the program just
//! inside it and just outside it, and runs both. If the report and the machine
//! ever stop agreeing — a target where the reported frame excludes the return
//! address, a frame the host reports as `dynamic`, a reserve at the top of the
//! stack that this one does not have — this fails, loudly, with both numbers.

use std::process::Command;

use super::super::{Architecture, FLOOR_STACK_BYTES, stack_ledger};
use super::exhaustion::spine_source;
use super::{build_executable, compile, test_directory};

/// A recursion whose activation carries a fixed array, so its frame is six
/// hundred times the tight spine's and its ceiling a hundred thousand levels
/// rather than sixty-seven million.
///
/// The pair of widths is the point: the model under test is one division, and
/// checking it at two frame sizes six hundred times apart says far more about
/// the division than checking it twice at one size would. The array
/// is 256 elements because the host compiler spends nine seconds vectorizing
/// the fill of a seven-thousand-element one and a tenth of a second on this,
/// for the same arithmetic under test.
///
/// The depth comes from the argument count rather than a literal so the host
/// optimizer cannot solve the recursion in closed form, which it does — and
/// then there is no recursion left to measure.
fn wide_frame_source(depth: u64) -> Vec<u8> {
    format!(
        r#"fn spine(depth: own u64, v: own u64, i: own u8) -> result: own u64 pure {{
  let pad = array_new::<u64, 256>(v);
  let wide = cvt::<u8, u64>(i);
  set pad[wide] = depth;
  let done = depth == 0_u64;
  if done {{
    return pad[wide];
  }}
  let next = depth -wrap 1_u64;
  let a = spine(depth: next, v: v, i: i);
  let b = pad[wide];
  return a +wrap b;
}}

command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {{
  let count = 0_u64;
  region 'invocation {{
    set count = args_count::<'invocation>(args: &'invocation args);
  }}
  match cvt::<u64, u8>(count) {{
    Ok(value: idx) => {{
      let depth = count *wrap {depth}_u64;
      let r = spine(depth: depth, v: 3_u64, i: idx);
      let ok = r > 0_u64;
      if ok {{
        return exit_status(code: 0_u8);
      }}
      return exit_status(code: 1_u8);
    }}
    Err(error: e) => {{
      return exit_status(code: 9_u8);
    }}
  }}
}}
"#
    )
    .into_bytes()
}

/// How far the measured ceiling may sit from the reported one.
///
/// A tenth of a percent, and it is slack rather than a fitted constant:
/// measured across frames of 16, 10 272, 34 848, and 291 760 bytes, the first
/// failing depth landed at most 1 136, 2, 0, and 0 levels from the report —
/// the largest of those is 0.0017% of its own ceiling. The band is wide enough
/// that the outermost frames of a program cannot trip it and narrow enough
/// that a systematic error, which is what a wrong model looks like, cannot hide
/// in it.
const TOLERANCE: f64 = 0.001;

/// The ledger for one source, produced the way the driver produces it.
fn ledger_for(source: &[u8], directory: &std::path::Path) -> Vec<String> {
    ledger_lines(&compile(source), directory)
}

/// The ledger for one already-emitted module, so a caller can ask for the
/// overlapped world as well as the sequential one.
pub(super) fn ledger_lines(emitted: &str, directory: &std::path::Path) -> Vec<String> {
    let module = directory.join("ledger.ll");
    let assembly = directory.join("ledger.s");
    std::fs::write(&module, emitted).expect("write the ledger module");
    let status = Command::new("/usr/bin/clang")
        .arg("-x")
        .arg("ir")
        .arg(&module)
        .arg("-S")
        .arg("-o")
        .arg(&assembly)
        .arg("-fstack-usage")
        .arg("-Wno-override-module")
        .args(crate::HOST_OPTIMIZATION_ARGUMENTS)
        .status()
        .expect("run the host compiler for the ledger");
    assert!(status.success(), "the ledger compilation failed: {status}");
    let usage =
        std::fs::read_to_string(directory.join("ledger.su")).expect("read the stack-usage report");
    let text = std::fs::read_to_string(&assembly).expect("read the ledger assembly");
    stack_ledger(&usage, &text, FLOOR_STACK_BYTES, Architecture::HOST)
}

/// What the ledger says one level of a named recursion costs, in bytes.
pub(super) fn reported_frame_bytes(lines: &[String], name: &str) -> u64 {
    let row = cycle_row(lines, name);
    let bytes = row
        .split_whitespace()
        .zip(row.split_whitespace().skip(1))
        .find_map(|(value, unit)| (unit == "B/level").then_some(value))
        .unwrap_or_else(|| panic!("the cycle row states no frame width: {row}"));
    bytes
        .parse()
        .unwrap_or_else(|_| panic!("the frame width is not a number: {row}"))
}

fn cycle_row<'a>(lines: &'a [String], name: &str) -> &'a String {
    lines
        .iter()
        .find(|line| {
            line.starts_with("STACK cycle")
                && line
                    .split_whitespace()
                    .nth(2)
                    .is_some_and(|symbol| symbol == name)
        })
        .unwrap_or_else(|| panic!("the ledger reports no cycle for {name}: {lines:#?}"))
}

/// The levels the ledger says one named recursion's stack holds.
fn reported_levels(lines: &[String], name: &str) -> u64 {
    let row = cycle_row(lines, name);
    let levels = row
        .split_whitespace()
        .zip(row.split_whitespace().skip(1))
        .find_map(|(value, unit)| (unit == "levels").then_some(value))
        .unwrap_or_else(|| panic!("the cycle row states no level count: {row}"));
    levels
        .parse()
        .unwrap_or_else(|_| panic!("the level count is not a number: {row}"))
}

/// Builds the program at one depth and reports whether it completed.
///
/// A run that did not complete must have run out of stack and said so. Exit 0
/// is not the only thing this fixture can produce — it has its own `r > 0`
/// false arm at exit 1 — and a bare signal is a third outcome, so a bare
/// "did not exit 0" would let the ceiling half pass against a floor that had
/// stopped reporting altogether.
fn completes(source: Vec<u8>, directory: &std::path::Path) -> bool {
    let executable = build_executable(&compile(&source), directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the depth probe");
    if output.status.code() == Some(0) {
        return true;
    }
    super::exhaustion::assert_resource_record(&output.stderr, "stack");
    false
}

/// A row is what one activation costs, not what the function allocated.
///
/// [`the_reported_ceiling_is_the_measured_one`] is the whole truth about this
/// arithmetic and it is also the expensive way to state it: it compiles four
/// programs and runs them, and it can only ever speak about the machine it is
/// running on. This states the same rule from a synthetic report, in a
/// millisecond, for both architectures at once — the eight bytes x86-64 leaves
/// out of every figure it reports are exactly what the ledger got wrong before
/// batch 0090, and an arm64 machine can now be the one that says so.
///
/// The cycle row is checked with the frame row because the cost is what the
/// division consumes: an activation count is the runtime's stack over what one
/// level really spends, and a ledger that carried the eight bytes into the
/// table but not into the division would over-promise depth exactly as before.
#[test]
fn a_row_is_what_one_activation_costs() {
    // `-fstack-usage` writes `locator\tbytes\tqualifier`, and clang's locator
    // is `file:line:column:name`.
    const REPORTED: u64 = 64;
    const STACK: u64 = 4_096;
    let usage = format!("probe.wf:1:1:wf_probe\t{REPORTED}\tstatic\n");
    // Column zero is a label and an indented line is an instruction, which is
    // all the graph reader needs to see a function that reaches itself.
    let assembly = "wf_probe:\n\tcall wf_probe\n\tret\n";

    for (architecture, level) in [
        (Architecture::X86_64, REPORTED + 8),
        (Architecture::Arm64, REPORTED),
    ] {
        let lines = stack_ledger(&usage, assembly, STACK, architecture);
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("STACK frame")
                    && line.contains(&format!("{level} B"))),
            "on {architecture:?} one activation costs {level} bytes: {lines:#?}"
        );
        assert_eq!(reported_frame_bytes(&lines, "wf_probe"), level);
        assert_eq!(reported_levels(&lines, "wf_probe"), STACK / level);
    }
}

/// The architecture the ledger calls the host's is the machine's own.
///
/// Read from [`std::env::consts::ARCH`] rather than from a `cfg!` of its own,
/// so this is an independent statement and not a copy of the ledger's
/// predicate: a `HOST` that named the wrong architecture would satisfy its own
/// `cfg!` and fail here.
#[test]
fn the_hosts_architecture_is_the_one_the_machine_reports() {
    let expected = match std::env::consts::ARCH {
        "x86_64" => Architecture::X86_64,
        "aarch64" => Architecture::Arm64,
        other => panic!("no qualified target runs on {other}, so the ledger has no rule for it"),
    };
    assert_eq!(Architecture::HOST, expected);
}

/// The reported ceiling and the measured one agree, at two frame widths.
///
/// This is the case that makes the ledger evidence rather than a description.
/// Both halves matter and they fail differently: a program that dies *inside*
/// the reported ceiling means the ledger is promising depth the machine does
/// not have, which is the dangerous direction; one that survives well past it
/// means the ledger is understating what the program can do, which would send a
/// writer to restructure code that was fine.
#[test]
fn the_reported_ceiling_is_the_measured_one() {
    for (name, source) in [
        ("wf_spine", (&wide_frame_source) as &dyn Fn(u64) -> Vec<u8>),
        ("wf_spine", &spine_source),
    ] {
        let directory = test_directory();
        let lines = ledger_for(&source(1_000), &directory);
        let levels = reported_levels(&lines, name);
        let inside = (levels as f64 * (1.0 - TOLERANCE)) as u64;
        let outside = (levels as f64 * (1.0 + TOLERANCE)) as u64 + 1;

        assert!(
            completes(source(inside), &directory),
            "the ledger reports {levels} levels but the program died at \
             {inside}, so it is promising depth the machine does not have"
        );
        assert!(
            !completes(source(outside), &directory),
            "the ledger reports {levels} levels and the program survived \
             {outside}, so the reported ceiling is not the real one"
        );
        std::fs::remove_dir_all(&directory).expect("remove the test directory");
    }
}

/// The recursion the compiler generates is in the ledger like any other.
///
/// The invisible-recursion class this batch removed was invisible in exactly
/// this sense: it had no name in the source, so no writer could look for it.
/// Its rows are the ledger's answer to that, and they are also how anyone
/// checks the removal held — a `wf.drop` cycle row reappearing means the
/// destruction path is descending the stack again.
#[test]
fn the_compilers_own_drop_glue_has_rows_and_no_cycle() {
    let directory = test_directory();
    let lines = ledger_for(RECURSIVE_VALUE, &directory);
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("STACK frame") && line.contains("wf.drop.")),
        "a program with a recursive nominal must have drop-glue frame rows: \
         {lines:#?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("STACK cycle") && line.contains("wf.drop.")),
        "the compiler-generated drop glue is descending the stack again: \
         {lines:#?}"
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A recursive nominal, built and destroyed, with nothing recursive written.
const RECURSIVE_VALUE: &[u8] = br#"enum Tree {
  Leaf();
  Branch(left: box<Tree>, right: box<Tree>);
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let left = Leaf();
  let right = Leaf();
  let boxed_left = box_new(move left);
  let boxed_right = box_new(move right);
  let branch = Branch(left: move boxed_left, right: move boxed_right);
  let root = box_new(move branch);
  return exit_status(code: 0_u8);
}
"#;
