use super::{compile, compile_and_run};

#[test]
fn every_reinterpret_pair_preserves_its_source_bits() {
    let source = br#"fn main() -> own unit traps {
  let u8_bits = reinterpret<i8, u8>(-1_i8);
  claim i8_to_u8: ieq(u8_bits, 255_u8) because "i8 to u8";
  let i8_bits = reinterpret<u8, i8>(255_u8);
  claim u8_to_i8: ieq(i8_bits, -1_i8) because "u8 to i8";
  let u16_bits = reinterpret<i16, u16>(-1_i16);
  claim i16_to_u16: ieq(u16_bits, 65535_u16) because "i16 to u16";
  let i16_bits = reinterpret<u16, i16>(65535_u16);
  claim u16_to_i16: ieq(i16_bits, -1_i16) because "u16 to i16";
  let u32_bits = reinterpret<i32, u32>(-1_i32);
  claim i32_to_u32: ieq(u32_bits, 4294967295_u32) because "i32 to u32";
  let i32_bits = reinterpret<u32, i32>(4294967295_u32);
  claim u32_to_i32: ieq(i32_bits, -1_i32) because "u32 to i32";
  let u64_bits = reinterpret<i64, u64>(-1_i64);
  claim i64_to_u64: ieq(u64_bits, 18446744073709551615_u64) because "i64 to u64";
  let i64_bits = reinterpret<u64, i64>(18446744073709551615_u64);
  claim u64_to_i64: ieq(i64_bits, -1_i64) because "u64 to i64";
  let f32_from_i32 = reinterpret<i32, f32>(2143289345_i32);
  claim i32_to_f32_payload: fne(f32_from_i32, f32_from_i32) because "i32 to f32 payload";
  let i32_from_f32 = reinterpret<f32, i32>(f32_from_i32);
  claim f32_to_i32_payload: ieq(i32_from_f32, 2143289345_i32) because "f32 to i32 payload";
  let f32_from_u32 = reinterpret<u32, f32>(2143289346_u32);
  claim u32_to_f32_payload: fne(f32_from_u32, f32_from_u32) because "u32 to f32 payload";
  let u32_from_f32 = reinterpret<f32, u32>(f32_from_u32);
  claim f32_to_u32_payload: ieq(u32_from_f32, 2143289346_u32) because "f32 to u32 payload";
  let f64_from_i64 = reinterpret<i64, f64>(9221120237041090561_i64);
  claim i64_to_f64_payload: fne(f64_from_i64, f64_from_i64) because "i64 to f64 payload";
  let i64_from_f64 = reinterpret<f64, i64>(f64_from_i64);
  claim f64_to_i64_payload: ieq(i64_from_f64, 9221120237041090561_i64) because "f64 to i64 payload";
  let f64_from_u64 = reinterpret<u64, f64>(9221120237041090562_u64);
  claim u64_to_f64_payload: fne(f64_from_u64, f64_from_u64) because "u64 to f64 payload";
  let u64_from_f64 = reinterpret<f64, u64>(f64_from_u64);
  claim f64_to_u64_payload: ieq(u64_from_f64, 9221120237041090562_u64) because "f64 to u64 payload";
  return unit;
}
"#;
    let llvm = compile(source);
    for instruction in [
        "bitcast i32",
        "bitcast float",
        "bitcast i64",
        "bitcast double",
        "or i8",
        "or i16",
        "or i32",
        "or i64",
    ] {
        assert!(
            llvm.contains(instruction),
            "reinterpret family must exercise {instruction}"
        );
    }
    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "reinterpret family failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
