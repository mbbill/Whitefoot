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
  let shifted_trap = ishl.trap(1_u8, 7_u32);
  claim ishl_trap: ieq(shifted_trap, 128_u8) because "ishl.trap";
  let right_trap = ishr.trap(128_u8, 7_u32);
  claim ishr_trap: ieq(right_trap, 1_u8) because "ishr.trap";
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
  claim idiv_trap: ieq(quotient, 4_i32) because "idiv.trap";
  let remainder = 9_i32 % 2_i32;
  claim irem_trap: ieq(remainder, 1_i32) because "irem.trap";
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
fn trapping_shift_reports_op8_before_executing_an_invalid_shift() {
    let source = br#"fn main() -> own unit traps {
  let shifted = ishl.trap(1_u8, 8_u32);
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(
        stderr.starts_with(
            "{\"rule_id\":\"OP-8\",\"message\":\"\",\"function\":\"main\",\"node_path\":["
        ),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn trapping_division_checks_zero_before_the_partial_instruction() {
    let source = br#"fn main() -> own unit traps {
  let quotient = 1_i32 / 0_i32;
  return unit;
}
"#;
    let llvm = compile(source);
    let trap = llvm.find("call void @wf_trap").expect("trap branch");
    let divide = llvm.find(" = sdiv i32").expect("safe divide");
    assert!(trap < divide);
    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with(
            "{\"rule_id\":\"OP-2\",\"message\":\"\",\"function\":\"main\",\"node_path\":["
        ),
        "unexpected stderr: {stderr}"
    );
}
