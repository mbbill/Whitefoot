use super::{compile, compile_and_run};

#[test]
fn executes_width_sensitive_integer_edges_for_every_unsigned_width() {
    let template = r#"fn main() -> own unit traps {
  let shifted: own $TYPE = ishl.wrap<$TYPE>(1_$TYPE, $AMOUNT_u32);
  check ieq<$TYPE>(shifted, 2_$TYPE) else trap "masked shift";
  let rotated: own $TYPE = irotl<$TYPE>(1_$TYPE, $AMOUNT_u32);
  check ieq<$TYPE>(rotated, 2_$TYPE) else trap "modular rotate";
  let population: own u32 = ipopcount<$TYPE>($MAX_$TYPE);
  check ieq<u32>(population, $WIDTH_u32) else trap "population count";
  let leading: own u32 = iclz<$TYPE>(0_$TYPE);
  check ieq<u32>(leading, $WIDTH_u32) else trap "zero leading count";
  let trailing: own u32 = ictz<$TYPE>(0_$TYPE);
  check ieq<u32>(trailing, $WIDTH_u32) else trap "zero trailing count";
  let saturated: own $TYPE = imul.sat<$TYPE>($MAX_$TYPE, 2_$TYPE);
  check ieq<$TYPE>(saturated, $MAX_$TYPE) else trap "saturating multiply";
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
                "  let swapped: own {ty} = ibswap<{ty}>(1_{ty});\n  check ieq<{ty}>(swapped, {expected}_{ty}) else trap \"byte swap\";\n"
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
  check anded == 0_u8 else trap "iand";
  let ored = ior(240_u8, 15_u8);
  check ored == 255_u8 else trap "ior";
  let xored = ixor(240_u8, 15_u8);
  check xored == 255_u8 else trap "ixor";
  let inverted = inot(0_u8);
  check inverted == 255_u8 else trap "inot";
  let shifted_wrap = ishl.wrap(1_u8, 9_u32);
  check shifted_wrap == 2_u8 else trap "ishl.wrap";
  let right_signed = ishr.wrap(-4_i8, 1_u32);
  check right_signed == -2_i8 else trap "ishr.wrap";
  let shifted_trap = ishl.trap(1_u8, 7_u32);
  check shifted_trap == 128_u8 else trap "ishl.trap";
  let right_trap = ishr.trap(128_u8, 7_u32);
  check right_trap == 1_u8 else trap "ishr.trap";
  let rotated_left = irotl(1_u8, 1_u32);
  check rotated_left == 2_u8 else trap "irotl";
  let rotated_right = irotr(1_u8, 1_u32);
  check rotated_right == 128_u8 else trap "irotr";
  let population = ipopcount(240_u8);
  check population == 4_u32 else trap "ipopcount";
  let leading = iclz(1_u8);
  check leading == 7_u32 else trap "iclz";
  let trailing = ictz(0_u8);
  check trailing == 8_u32 else trap "ictz";
  let swapped = ibswap(4660_u16);
  check swapped == 13330_u16 else trap "ibswap";
  let high_unsigned = imulhi(255_u8, 2_u8);
  check high_unsigned == 1_u8 else trap "imulhi unsigned";
  let high_signed = imulhi(-128_i8, 2_i8);
  check high_signed == -1_i8 else trap "imulhi signed";
  let add_unsigned = 250_u8 +sat 10_u8;
  check add_unsigned == 255_u8 else trap "iadd.sat unsigned";
  let add_signed = 120_i8 +sat 20_i8;
  check add_signed == 127_i8 else trap "iadd.sat signed";
  let subtract_unsigned = 1_u8 -sat 2_u8;
  check subtract_unsigned == 0_u8 else trap "isub.sat unsigned";
  let subtract_signed = -120_i8 -sat 20_i8;
  check subtract_signed == -128_i8 else trap "isub.sat signed";
  let multiply_unsigned = 20_u8 *sat 20_u8;
  check multiply_unsigned == 255_u8 else trap "imul.sat unsigned";
  let multiply_high = 20_i8 *sat 20_i8;
  check multiply_high == 127_i8 else trap "imul.sat signed high";
  let multiply_low = -20_i8 *sat 20_i8;
  check multiply_low == -128_i8 else trap "imul.sat signed low";
  let minimum = imin(-2_i8, 1_i8);
  check minimum == -2_i8 else trap "imin signed";
  let maximum = imax(254_u8, 1_u8);
  check maximum == 254_u8 else trap "imax unsigned";
  let quotient = 9_i32 / 2_i32;
  check quotient == 4_i32 else trap "idiv.trap";
  let remainder = 9_i32 % 2_i32;
  check remainder == 1_i32 else trap "irem.trap";
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
