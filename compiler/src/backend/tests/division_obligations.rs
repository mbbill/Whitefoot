//! Emission of the division dissolution [OP-2, ENT-6]: a discharged
//! divisor-class site compiles to the plain `udiv`/`sdiv` with no
//! zero-divisor or signed-overflow branch, while a signed site with two
//! non-constant operands stays outside the class and keeps its complete
//! runtime test. The switch is on, so the shipped emission and the forced-on
//! entry are one path and must agree byte for byte.

use super::{emit, emit_division_obligations};

/// One function holding both classes: the `check` establishes `d != 0`, so
/// the unsigned `n / d` discharges, while `p / q` over `i32` has two
/// non-constant operands and keeps its trap. The `check` and the retained
/// site keep the `traps` effect row correct under both switches.
const BOTH_CLASSES: &[u8] =
    br#"fn combine(n: own u64, d: own u64, p: own i32, q: own i32) -> own u64 traps {
  claim positive_divisor: igt(d, 0_u64) because "positive divisor";
  let quotient = n / d;
  let signed = p / q;
  return quotient;
}

fn main() -> own unit traps {
  let total = combine(n: 12_u64, d: 4_u64, p: 9_i32, q: 3_i32);
  claim combined_total: ieq(total, 3_u64) because "combined total";
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

/// Only the retained signed site carries a guard set: the unsigned site's
/// zero-divisor conjunct is discharged by the dominating claim, which is the
/// sole authority that may drop the check, and the site becomes a plain
/// `udiv`. v0.31 emitted four guards here; three remain.
#[test]
fn a_discharged_class_site_emits_no_division_guard() {
    let shipped = emit(BOTH_CLASSES);
    let candidate = emit_division_obligations(BOTH_CLASSES);
    assert_eq!(
        division_guard_count(&shipped),
        3,
        "the discharged unsigned site loses its zero-divisor guard; the \
         retained signed site keeps zero, minimum, and minus one",
    );
    assert_eq!(opcode_count(&shipped, "udiv"), 1);
    assert_eq!(opcode_count(&shipped, "sdiv"), 1);
    assert_eq!(
        shipped, candidate,
        "the shipped path and the forced-on entry are one judgment",
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
  claim halved: ieq(half, 4_i32) because "halved";
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

/// The instance evidence behind [EFF-2]'s written-body contribution: one
/// generic body whose written selected type is a type parameter emits both
/// dispositions from the same written `traps` row. The unsigned instance is
/// in [OP-2]'s divisor class, discharges through the requirement, and emits
/// a plain `udiv` with no guard; the signed instance is outside the class
/// and keeps its complete zero, minimum, and minus-one test around `sdiv`.
#[test]
fn a_generic_divisor_site_emits_a_guard_only_at_the_retained_instance() {
    const GENERIC_DIVISION: &[u8] = br#"fn ratio<T: Int>(n: own T, d: own T) -> own T traps requires {
  check ine(d, 0_T) else trap "nonzero divisor";
} {
  let q = n / d;
  return q;
}

fn main() -> own unit traps {
  let unsigned = ratio<u32>(n: 12_u32, d: 4_u32);
  let signed = ratio<i32>(n: 9_i32, d: 3_i32);
  return unit;
}
"#;
    let module = emit(GENERIC_DIVISION);
    assert_eq!(
        opcode_count(&module, "udiv"),
        1,
        "the unsigned instance emits its division",
    );
    assert_eq!(
        opcode_count(&module, "sdiv"),
        1,
        "the signed instance emits its division",
    );
    assert_eq!(
        division_guard_count(&module),
        3,
        "only the retained signed instance tests zero, the minimum, and minus one",
    );
}
