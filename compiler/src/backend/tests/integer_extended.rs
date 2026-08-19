use super::{compile, compile_and_run};

#[test]
fn executes_width_sensitive_integer_edges_for_every_unsigned_width() {
    let template = r#"fn main() -> own unit traps {
  let shifted = ishl.wrap(1_$TYPE, $AMOUNT_u32);
  claim masked_shift: ieq(shifted, 2_$TYPE) because "masked shift";
  let rotated = irotl(1_$TYPE, $AMOUNT_u32);
  claim modular_rotate: ieq(rotated, 2_$TYPE) because "modular rotate";
  let population = ipopcount($MAX_$TYPE);
  claim population_count: ieq(population, $WIDTH_u32) because "population count";
  let leading = iclz(0_$TYPE);
  claim zero_leading_count: ieq(leading, $WIDTH_u32) because "zero leading count";
  let trailing = ictz(0_$TYPE);
  claim zero_trailing_count: ieq(trailing, $WIDTH_u32) because "zero trailing count";
  let saturated = $MAX_$TYPE *sat 2_$TYPE;
  claim saturating_multiply: ieq(saturated, $MAX_$TYPE) because "saturating multiply";
$BSWAP  return unit;
}
"#;
    for (ty, width, maximum, swapped) in [
        ("u8", 8, "255", None),
        ("u16", 16, "65535", Some("256")),
        ("u32", 32, "4294967295", Some("16777216")),
        ("u64", 64, "18446744073709551615", Some("72057594037927936")),
    ] {
        let bswap = swapped.map_or_else(String::new, |expected| {
            format!(
                "  let swapped = ibswap(1_{ty});\n  claim byte_swap: ieq(swapped, {expected}_{ty}) because \"byte swap\";\n"
            )
        });
        let source = template
            .replace("$TYPE", ty)
            .replace("$WIDTH", &width.to_string())
            .replace("$AMOUNT", &(width + 1).to_string())
            .replace("$MAX", maximum)
            .replace("$BSWAP", &bswap);
        let output = compile_and_run(&compile(source.as_bytes()));
        assert!(
            output.status.success(),
            "width-sensitive program failed for {ty}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn executes_the_remaining_integer_family_and_defined_edges() {
    let source = br#"fn main() -> own unit traps {
  let anded = iand(240_u8, 15_u8);
  claim iand: ieq(anded, 0_u8) because "iand";
  let ored = ior(240_u8, 15_u8);
  claim ior: ieq(ored, 255_u8) because "ior";
  let xored = ixor(240_u8, 15_u8);
  claim ixor: ieq(xored, 255_u8) because "ixor";
  let inverted = inot(0_u8);
  claim inot: ieq(inverted, 255_u8) because "inot";
  let shifted_wrap = ishl.wrap(1_u8, 9_u32);
  claim ishl_wrap: ieq(shifted_wrap, 2_u8) because "ishl.wrap";
  let right_signed = ishr.wrap(-4_i8, 1_u32);
  claim ishr_wrap: ieq(right_signed, -2_i8) because "ishr.wrap";
  let shifted_exact = ishl(1_u8, 7_u32);
  claim ishl_exact: ieq(shifted_exact, 128_u8) because "ishl";
  let right_exact = ishr(128_u8, 7_u32);
  claim ishr_exact: ieq(right_exact, 1_u8) because "ishr";
  let rotated_left = irotl(1_u8, 1_u32);
  claim irotl: ieq(rotated_left, 2_u8) because "irotl";
  let rotated_right = irotr(1_u8, 1_u32);
  claim irotr: ieq(rotated_right, 128_u8) because "irotr";
  let population = ipopcount(240_u8);
  claim ipopcount: ieq(population, 4_u32) because "ipopcount";
  let leading = iclz(1_u8);
  claim iclz: ieq(leading, 7_u32) because "iclz";
  let trailing = ictz(0_u8);
  claim ictz: ieq(trailing, 8_u32) because "ictz";
  let swapped = ibswap(4660_u16);
  claim ibswap: ieq(swapped, 13330_u16) because "ibswap";
  let high_unsigned = imulhi(255_u8, 2_u8);
  claim imulhi_unsigned: ieq(high_unsigned, 1_u8) because "imulhi unsigned";
  let high_signed = imulhi(-128_i8, 2_i8);
  claim imulhi_signed: ieq(high_signed, -1_i8) because "imulhi signed";
  let add_unsigned = 250_u8 +sat 10_u8;
  claim iadd_sat_unsigned: ieq(add_unsigned, 255_u8) because "iadd.sat unsigned";
  let add_signed = 120_i8 +sat 20_i8;
  claim iadd_sat_signed: ieq(add_signed, 127_i8) because "iadd.sat signed";
  let subtract_unsigned = 1_u8 -sat 2_u8;
  claim isub_sat_unsigned: ieq(subtract_unsigned, 0_u8) because "isub.sat unsigned";
  let subtract_signed = -120_i8 -sat 20_i8;
  claim isub_sat_signed: ieq(subtract_signed, -128_i8) because "isub.sat signed";
  let multiply_unsigned = 20_u8 *sat 20_u8;
  claim imul_sat_unsigned: ieq(multiply_unsigned, 255_u8) because "imul.sat unsigned";
  let multiply_high = 20_i8 *sat 20_i8;
  claim imul_sat_signed_high: ieq(multiply_high, 127_i8) because "imul.sat signed high";
  let multiply_low = -20_i8 *sat 20_i8;
  claim imul_sat_signed_low: ieq(multiply_low, -128_i8) because "imul.sat signed low";
  let minimum = imin(-2_i8, 1_i8);
  claim imin_signed: ieq(minimum, -2_i8) because "imin signed";
  let maximum = imax(254_u8, 1_u8);
  claim imax_unsigned: ieq(maximum, 254_u8) because "imax unsigned";
  let quotient = 9_i32 / 2_i32;
  claim idiv_exact: ieq(quotient, 4_i32) because "idiv exact";
  let remainder = 9_i32 % 2_i32;
  claim irem_exact: ieq(remainder, 1_i32) because "irem exact";
  return unit;
}
"#;
    let llvm = compile(source);
    for fragment in [
        "@llvm.fshl.i8",
        "@llvm.fshr.i8",
        "@llvm.ctpop.i8",
        "@llvm.ctlz.i8",
        "@llvm.cttz.i8",
        "@llvm.bswap.i16",
        "@llvm.uadd.sat.i8",
        "@llvm.sadd.sat.i8",
        "@llvm.usub.sat.i8",
        "@llvm.ssub.sat.i8",
        "@llvm.smin.i8",
        "@llvm.umax.i8",
    ] {
        assert!(llvm.contains(fragment), "missing lowering {fragment}");
    }
    assert!(!llvm.contains(" nsw "));
    assert!(!llvm.contains(" nuw "));
    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "integer-family program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn defined_shift_reports_false_without_executing_an_invalid_shift() {
    let source = br#"fn main() -> own unit traps {
  let is_defined = ishl.defined(1_u8, 8_u32);
  claim out_of_range_shift_is_undefined: bnot(is_defined) because "out-of-range shift must be undefined";
  return unit;
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("icmp ult i32"));
    assert!(!llvm.contains("shl i8 1, 8"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn defined_division_checks_zero_and_signed_overflow_without_dividing() {
    let source = br#"fn division_is_defined(n: own i32, d: own i32) -> own Bool pure {
  return n /defined d;
}

fn main() -> own unit traps {
  let zero = 0_i32;
  let one = 1_i32;
  let zero_defined = division_is_defined(n: one, d: zero);
  claim division_by_zero_is_undefined: bnot(zero_defined) because "division by zero must be undefined";
  let minimum = -2147483648_i32;
  let minus_one = -1_i32;
  let overflow_defined = division_is_defined(n: minimum, d: minus_one);
  claim signed_division_overflow_is_undefined: bnot(overflow_defined) because "signed division overflow must be undefined";
  let ordinary_defined = division_is_defined(n: one, d: one);
  claim ordinary_division_is_defined: ordinary_defined because "ordinary division must be defined";
  return unit;
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("icmp ne i32"));
    assert!(llvm.contains("icmp ne i32 %v0, -2147483648"));
    assert!(llvm.contains("icmp ne i32 %v1, -1"));
    assert!(!llvm.contains(" = sdiv i32"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}
