use super::{compile, compile_and_run};

#[test]
fn every_reinterpret_pair_preserves_its_source_bits() {
    let source = br#"fn main() -> own unit traps {
  let u8_bits = reinterpret<i8, u8>(-1_i8);
  check u8_bits == 255_u8 else trap "i8 to u8";
  let i8_bits = reinterpret<u8, i8>(255_u8);
  check i8_bits == -1_i8 else trap "u8 to i8";
  let u16_bits = reinterpret<i16, u16>(-1_i16);
  check u16_bits == 65535_u16 else trap "i16 to u16";
  let i16_bits = reinterpret<u16, i16>(65535_u16);
  check i16_bits == -1_i16 else trap "u16 to i16";
  let u32_bits = reinterpret<i32, u32>(-1_i32);
  check u32_bits == 4294967295_u32 else trap "i32 to u32";
  let i32_bits = reinterpret<u32, i32>(4294967295_u32);
  check i32_bits == -1_i32 else trap "u32 to i32";
  let u64_bits = reinterpret<i64, u64>(-1_i64);
  check u64_bits == 18446744073709551615_u64 else trap "i64 to u64";
  let i64_bits = reinterpret<u64, i64>(18446744073709551615_u64);
  check i64_bits == -1_i64 else trap "u64 to i64";
  let f32_from_i32 = reinterpret<i32, f32>(2143289345_i32);
  check fne(f32_from_i32, f32_from_i32) else trap "i32 to f32 payload";
  let i32_from_f32 = reinterpret<f32, i32>(f32_from_i32);
  check i32_from_f32 == 2143289345_i32 else trap "f32 to i32 payload";
  let f32_from_u32 = reinterpret<u32, f32>(2143289346_u32);
  check fne(f32_from_u32, f32_from_u32) else trap "u32 to f32 payload";
  let u32_from_f32 = reinterpret<f32, u32>(f32_from_u32);
  check u32_from_f32 == 2143289346_u32 else trap "f32 to u32 payload";
  let f64_from_i64 = reinterpret<i64, f64>(9221120237041090561_i64);
  check fne(f64_from_i64, f64_from_i64) else trap "i64 to f64 payload";
  let i64_from_f64 = reinterpret<f64, i64>(f64_from_i64);
  check i64_from_f64 == 9221120237041090561_i64 else trap "f64 to i64 payload";
  let f64_from_u64 = reinterpret<u64, f64>(9221120237041090562_u64);
  check fne(f64_from_u64, f64_from_u64) else trap "u64 to f64 payload";
  let u64_from_f64 = reinterpret<f64, u64>(f64_from_u64);
  check u64_from_f64 == 9221120237041090562_u64 else trap "f64 to u64 payload";
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
