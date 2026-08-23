//! The resource-exhaustion floor: what an execution does when it runs out.
//!
//! Exhaustion is the one abnormal end a *correct* program can reach. A false
//! `claim` cannot happen in a reviewed program and yet gets a byte-exact
//! [DIAG-3] record; running out of stack or heap needs no source defect at all
//! and, before this floor, produced zero bytes and a bare host signal. These
//! cases pin the floor that closes that asymmetry.
//!
//! Two separate obligations live here and must not be confused:
//!
//! - *containment* — a frame larger than the guard region walks its pages on
//!   the way down, so it cannot step over the guard into whatever is mapped
//!   below. This is a safety property, not a reporting one: without it an
//!   accepted program can silently overwrite a neighbouring thread's live
//!   stack. [`every_generated_definition_carries_the_stack_probe`] is its
//!   case.
//! - *reporting* — exhaustion ends the process by a defined abort that first
//!   writes one fixed record naming only the resource class. The record
//!   carries no `rule_id`, no function, and no node path, and that absence is
//!   what mechanically distinguishes it from a [DIAG-3] record.
//!
//! The record's bytes are fixed by two independent constraints that happen to
//! agree. A signal handler may only reach async-signal-safe facilities, which
//! admits a constant string written with `write` and essentially nothing else;
//! and [PAR-1] requires observables to be identical under every permitted
//! schedule, which forbids the record from naming a worker, a thread, a depth,
//! or an address. Either one alone would force a fixed constant.

use std::process::Command;

use super::{build_executable, compile, test_directory};

/// The attribute group [`crate::backend::emitter`] gives every definition, and
/// the value it carries on this host.
///
/// The value is the host C compiler's own frame-probing helper, so a generated
/// frame walks its pages exactly the way the runtime's translation unit does.
#[cfg(target_os = "macos")]
const HOST_STACK_PROBE: &str = "\"probe-stack\"=\"__chkstk_darwin\"";
#[cfg(not(target_os = "macos"))]
const HOST_STACK_PROBE: &str = "\"probe-stack\"=\"inline-asm\"";

/// A program that reaches several kinds of generated definition at once: a
/// heap box and its compiler-generated drop glue, a recursive walk, a system
/// transfer, and the entry itself.
const MIXED_DEFINITIONS: &[u8] = br#"enum Chain {
  End();
  More(next: box<Chain>);
}

fn depth['r](chain: &'r box<Chain>) -> result: own u64 reads('r) {
  match deref(deref(chain)) {
    End() => {
      return 0_u64;
    }
    More(next: inner) => {
      let below = depth<'r>(chain: inner);
      return below +wrap 1_u64;
    }
  }
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let end = End();
  let bottom = box_new(move end);
  let one = More(next: move bottom);
  let boxed = box_new(move one);
  region 'chain {
    let measured = depth<'chain>(chain: &'chain boxed);
    if ieq(measured, 1_u64) {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

/// Every definition the module emits carries the probe attribute, and the
/// group it names is the host's.
///
/// The count equality is the point of the case. Containment is a completeness
/// property — one unprobed large frame is enough to jump the guard region into
/// a neighbouring thread's live stack — so a case that merely found *some*
/// probed definition would pass against a module that left the drop glue, a
/// clone, or a thunk unprobed. Counting both sides is what makes "every
/// generated function" checkable rather than asserted.
#[test]
fn every_generated_definition_carries_the_stack_probe() {
    for module in [
        compile(MIXED_DEFINITIONS),
        super::emit_with_overlap(MIXED_DEFINITIONS),
    ] {
        let definitions = module
            .lines()
            .filter(|line| line.starts_with("define "))
            .count();
        assert!(
            definitions > 1,
            "the fixture must reach more than one definition:\n{module}"
        );
        let probed = module
            .lines()
            .filter(|line| line.starts_with("define ") && line.ends_with(" #0 {"))
            .count();
        assert_eq!(
            probed,
            definitions,
            "every generated definition must carry the probe group; \
             {} of {definitions} did not:\n{module}",
            definitions - probed
        );
        assert_eq!(
            module.matches("attributes #0 = { ").count(),
            1,
            "the module declares its one attribute group once:\n{module}"
        );
        assert!(
            module.contains(&format!("attributes #0 = {{ {HOST_STACK_PROBE} }}")),
            "the group must name this host's probing helper:\n{module}"
        );
    }
}

/// The probe is emitted only for a frame past the page threshold, so an
/// ordinary program pays nothing for it.
///
/// This is what makes the containment fix free rather than a trade: the
/// attribute changes the machine code of exactly the functions whose frames
/// could jump the guard, and leaves every other function alone.
#[test]
fn an_ordinary_frame_emits_no_probe_call() {
    let directory = test_directory();
    let executable = build_executable(&compile(MIXED_DEFINITIONS), &directory);
    let symbols = Command::new("/usr/bin/nm")
        .arg(&executable)
        .output()
        .expect("read the linked symbol table");
    let listed = String::from_utf8_lossy(&symbols.stdout);
    assert!(
        !listed.contains("chkstk"),
        "no ordinary frame reaches the probing helper:\n{listed}"
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}
