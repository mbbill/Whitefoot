//! Emission of the division dissolution [OP-2, ENT-6]: a discharged
//! divisor-class site compiles to the plain `udiv`/`sdiv` with no
//! zero-divisor or signed-overflow branch, while a signed site with two
//! non-constant operands stays outside the class and keeps its complete
//! runtime test. The switch is off, so the shipped emission is the v0.31
//! one and the forced-on entry must differ exactly at the class sites.

use super::{emit, emit_division_obligations};

/// One function holding both classes: the `check` establishes `d != 0`, so
/// the unsigned `n / d` discharges, while `p / q` over `i32` has two
/// non-constant operands and keeps its trap. The `check` and the retained
/// site keep the `traps` effect row correct under both switches.
const BOTH_CLASSES: &[u8] =
    br#"fn combine(n: own u64, d: own u64, p: own i32, q: own i32) -> own u64 traps {
  check igt(d, 0_u64) else trap "positive divisor";
  let quotient = n / d;
  let signed = p / q;
  return quotient;
}

fn main() -> own unit traps {
  let total = combine(n: 12_u64, d: 4_u64, p: 9_i32, q: 3_i32);
  check ieq(total, 3_u64) else trap "combined total";
  return unit;
}
"#;

/// Every `icmp eq` a bare division emits against a zero divisor, the type
/// minimum, or `-1` before its trap branch.
fn division_guard_count(module: &str) -> usize {
    module
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            line.contains("icmp eq")
                && (line.ends_with(", 0")
                    || line.ends_with(", -1")
                    || line.ends_with(", -2147483648"))
        })
        .count()
}

fn opcode_count(module: &str, opcode: &str) -> usize {
    module
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            line.starts_with('%') && line.contains(&format!("= {opcode} "))
        })
        .count()
}

/// With the switch off both sites carry their complete guard set; with it
/// on, only the retained signed site does, and the discharged unsigned site
/// is a plain `udiv`.
#[test]
fn a_discharged_class_site_emits_no_division_guard() {
    let shipped = emit(BOTH_CLASSES);
    let candidate = emit_division_obligations(BOTH_CLASSES);
    assert_eq!(
        division_guard_count(&shipped),
        4,
        "v0.31: the unsigned site guards a zero divisor and the signed site \
         guards zero, minimum, and minus one",
    );
    assert_eq!(
        division_guard_count(&candidate),
        3,
        "the discharged unsigned site loses its zero-divisor guard; the \
         retained signed site keeps its three",
    );
    assert_eq!(opcode_count(&candidate, "udiv"), 1);
    assert_eq!(opcode_count(&candidate, "sdiv"), 1);
    assert_ne!(
        shipped, candidate,
        "the candidate judgment removes a runtime check the shipped one keeps",
    );
}

/// A constant divisor decides both conditions statically, so the whole site
/// becomes one plain instruction with no branch at all and the function
/// needs no `traps` row.
#[test]
fn a_constant_divisor_site_emits_one_plain_instruction() {
    const CONSTANT_DIVISOR: &[u8] = br#"fn halve(n: own i32) -> own i32 pure {
  let q = n / 2_i32;
  return q;
}

fn main() -> own unit traps {
  let half = halve(n: 9_i32);
  check ieq(half, 4_i32) else trap "halved";
  return unit;
}
"#;
    let candidate = emit_division_obligations(CONSTANT_DIVISOR);
    assert_eq!(
        division_guard_count(&candidate),
        0,
        "a constant divisor leaves no runtime condition to test",
    );
    assert_eq!(opcode_count(&candidate, "sdiv"), 1);
}
