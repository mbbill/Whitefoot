//! Emission of exact division [OP-2, ENT-6]: every accepted site has a proof
//! of its complete integer domain and compiles to a plain `udiv`/`sdiv` with
//! no zero-divisor or signed-overflow branch. The shipped emission and the
//! forced-on entry are one path; each case names the entry whose judgment it
//! means.

use super::{emit, emit_division_obligations};

/// The claim establishes the complete unsigned domain `d != 0`; the claim is
/// the only runtime rejection point and the accepted exact division has no
/// backend guard.
const PROVED_UNSIGNED: &[u8] = br#"fn ratio(n: own u64, d: own u64) -> own u64 traps {
  claim positive_divisor: igt(d, 0_u64) because "positive divisor";
  let quotient = n / d;
  return quotient;
}

fn main() -> own unit traps {
  let total = ratio(n: 12_u64, d: 4_u64);
  claim ratio_total: ieq(total, 3_u64) because "ratio total";
  return unit;
}
"#;

/// Each emitted function body, so a count means what it says about one
/// function rather than about every function in the module.
fn function_bodies(module: &str) -> Vec<&str> {
    module
        .split("\ndefine ")
        .skip(1)
        .map(|body| {
            body.split_once("\n}")
                .expect("a defined function is terminated")
                .0
        })
        .collect()
}

/// The emitted body of one named function.
fn function_body<'module>(module: &'module str, symbol: &str) -> &'module str {
    let marker = format!("@{symbol}(");
    function_bodies(module)
        .into_iter()
        .find(|body| body.contains(&marker))
        .unwrap_or_else(|| panic!("the module defines {symbol}"))
}

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

/// The unsigned domain is discharged by the dominating claim, which is the
/// sole runtime authority, and the exact site becomes a plain `udiv` with no
/// residual guard.
#[test]
fn a_proved_unsigned_site_emits_no_division_guard() {
    let ratio = function_body(&emit(PROVED_UNSIGNED), "wf_ratio").to_owned();
    assert_eq!(
        division_guard_count(&ratio),
        0,
        "an accepted exact unsigned division has no runtime guard",
    );
    assert_eq!(opcode_count(&ratio, "udiv"), 1);
    assert_eq!(opcode_count(&ratio, "sdiv"), 0);
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
/// generic exact operation proves its domain from the typed constant divisor
/// for both unsigned and signed instances. Both lower to plain instructions;
/// neither may acquire a runtime division guard.
#[test]
fn generic_exact_division_emits_no_runtime_guards() {
    const GENERIC_DIVISION: &[u8] = br#"fn divide_by_one<T: Int>(n: own T) -> own T pure {
  let q = n / 1_T;
  return q;
}

fn main() -> own unit pure {
  let unsigned = divide_by_one<u32>(n: 12_u32);
  let signed = divide_by_one<i32>(n: 9_i32);
  return unit;
}
"#;
    // Both instances lower to the same LLVM integer type, so each is
    // identified by the division it emits rather than by its symbol.
    let module = emit(GENERIC_DIVISION);
    let bodies = function_bodies(&module);
    let unsigned = bodies
        .iter()
        .find(|body| body.contains("= udiv "))
        .expect("the unsigned instance emits its division");
    let signed = bodies
        .iter()
        .find(|body| body.contains("= sdiv "))
        .expect("the signed instance emits its division");
    assert_eq!(
        division_guard_count(unsigned),
        0,
        "the discharged unsigned instance executes no runtime test",
    );
    assert_eq!(
        division_guard_count(signed),
        0,
        "the proved signed instance executes no runtime test",
    );
}
