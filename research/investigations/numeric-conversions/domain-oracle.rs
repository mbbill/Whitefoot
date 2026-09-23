//! Research-only conversion-domain comparison; invocation is in DESIGN.md.
//! The reference side decodes binary values using integer arithmetic. The
//! candidate side exercises the rounded/saturating host casts used to model
//! current lowering. No WF implementation is imported or modified.
#![forbid(unsafe_code)]

fn finite(bits: u64) -> Option<(bool, u64, i32)> {
    let sign = bits >> 63 != 0;
    let exponent = ((bits >> 52) & 2047) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    match exponent {
        2047 => None,
        0 => Some((sign, fraction, -1074)),
        _ => Some((sign, (1_u64 << 52) | fraction, exponent - 1023 - 52)),
    }
}

fn integer_domain(bits: u64, width: u32, signed: bool) -> bool {
    let Some((negative, mut significand, mut exponent)) = finite(bits) else {
        return false;
    };
    if significand == 0 {
        return true;
    }
    let zeros = significand.trailing_zeros();
    significand >>= zeros;
    exponent += zeros as i32;
    if exponent < 0 || 64 - significand.leading_zeros() + exponent as u32 > 65 {
        return false;
    }
    let magnitude = (significand as u128) << exponent;
    if negative {
        signed && magnitude <= 1_u128 << (width - 1)
    } else {
        magnitude < 1_u128 << (width - u32::from(signed))
    }
}

fn narrow_float_domain(bits: u64) -> bool {
    let Some((_, significand, exponent)) = finite(bits) else {
        return true; // OP-6 admits infinities and canonicalizes every NaN.
    };
    if significand == 0 {
        return true;
    }
    let zeros = significand.trailing_zeros();
    let significant_bits = 64 - significand.leading_zeros() - zeros;
    significant_bits <= 24
        && exponent + zeros as i32 >= -149
        && exponent + (64 - significand.leading_zeros()) as i32 - 1 <= 127
}

fn integer_float_domain(magnitude: u64, precision: u32) -> bool {
    magnitude == 0 || 64 - magnitude.leading_zeros() - magnitude.trailing_zeros() <= precision
}

fn float_integer_candidate(value: f64, width: u32, signed: bool, single: bool) -> bool {
    let precision = if single { 24 } else { 53 };
    if signed {
        let maximum = ((1_i128 << (width - 1)) - 1) as i64;
        let minimum = -(1_i128 << (width - 1)) as i64;
        let converted = (value as i64).clamp(minimum, maximum);
        let recovered = if single {
            (converted as f32) as f64
        } else {
            converted as f64
        };
        value == recovered && (width - 1 <= precision || converted != maximum)
    } else {
        let maximum = ((1_u128 << width) - 1) as u64;
        let converted = (value as u64).min(maximum);
        let recovered = if single {
            (converted as f32) as f64
        } else {
            converted as f64
        };
        value == recovered && (width <= precision || converted != maximum)
    }
}

fn check_float(value: f64, single: bool, count: &mut u64) {
    for width in [8, 16, 32, 64] {
        for signed in [false, true] {
            assert_eq!(
                integer_domain(value.to_bits(), width, signed),
                float_integer_candidate(value, width, signed, single),
                "float/int: {:016x}, {width}, {signed}, {single}",
                value.to_bits()
            );
            *count += 1;
        }
    }
    if !single {
        let candidate = value.is_nan() || value == (value as f32) as f64;
        assert_eq!(narrow_float_domain(value.to_bits()), candidate);
        *count += 1;
    }
}

fn check_integer(value: u64, count: &mut u64) {
    for single in [false, true] {
        let precision = if single { 24 } else { 53 };
        let rounded = if single {
            (value as f32) as f64
        } else {
            value as f64
        };
        assert_eq!(
            integer_float_domain(value, precision),
            rounded as u64 == value && value != u64::MAX
        );
        *count += 1;
        if value <= (1_u64 << 63) {
            let negative = -(value as i128) as i64;
            let rounded = if single {
                (negative as f32) as f64
            } else {
                negative as f64
            };
            assert_eq!(
                integer_float_domain(value, precision),
                rounded as i64 == negative
            );
            *count += 1;
        }
        if value <= i64::MAX as u64 {
            let positive = value as i64;
            let rounded = if single {
                (positive as f32) as f64
            } else {
                positive as f64
            };
            assert_eq!(
                integer_float_domain(value, precision),
                rounded as i64 == positive && positive != i64::MAX
            );
            *count += 1;
        }
    }
}

fn check_wrap(value: i128, width: u32, signed: bool, count: &mut u64) {
    let modulus = 1_i128 << width;
    let residue = value.rem_euclid(modulus);
    let expected = if signed && residue >= modulus / 2 {
        residue - modulus
    } else {
        residue
    };
    let actual = match (width, signed) {
        (8, false) => value as u8 as i128,
        (8, true) => value as i8 as i128,
        (16, false) => value as u16 as i128,
        (16, true) => value as i16 as i128,
        (32, false) => value as u32 as i128,
        (32, true) => value as i32 as i128,
        (64, false) => value as u64 as i128,
        (64, true) => value as i64 as i128,
        _ => unreachable!(),
    };
    assert_eq!(expected, actual);
    *count += 1;
}

fn main() {
    let mut count = 0;
    for value in 0..=u16::MAX as u64 {
        check_integer(value, &mut count);
    }
    for bit in 0..64 {
        for delta in -3_i128..=3 {
            let value = (1_i128 << bit) + delta;
            if (0..=u64::MAX as i128).contains(&value) {
                check_integer(value as u64, &mut count);
            }
        }
    }
    for value in [u64::MAX - 1, u64::MAX, i64::MAX as u64] {
        check_integer(value, &mut count);
    }
    for exponent in 0_u64..=2047 {
        for fraction in [0, 1, 2, (1 << 23) - 1, 1 << 28, 1 << 51, (1 << 52) - 1] {
            for sign in [0, 1_u64 << 63] {
                check_float(
                    f64::from_bits(sign | exponent << 52 | fraction),
                    false,
                    &mut count,
                );
            }
        }
    }
    for exponent in 0_u32..=255 {
        for fraction in [0, 1, 2, 1 << 22, (1 << 23) - 1] {
            for sign in [0, 1_u32 << 31] {
                check_float(
                    f32::from_bits(sign | exponent << 23 | fraction) as f64,
                    true,
                    &mut count,
                );
            }
        }
    }
    // Deterministic bit samples supplement, rather than replace, the edges.
    let mut sample = 0x8d26_0b68_4afe_3397_u64;
    for _ in 0..10000 {
        sample = sample.wrapping_mul(6364136223846793005).wrapping_add(1);
        check_float(f64::from_bits(sample), false, &mut count);
        check_integer(sample, &mut count);
    }
    let maximum_roundtrip = (u64::MAX as f64) as u64;
    assert_eq!(maximum_roundtrip, u64::MAX);
    assert!(!integer_float_domain(u64::MAX, 53));
    let upper = (1_u64 << 63) as f64;
    assert_eq!((upper as i64) as f64, upper);
    assert!(!integer_domain(upper.to_bits(), 64, true));
    assert!(narrow_float_domain(f64::NAN.to_bits()));
    assert!(narrow_float_domain((-0.0_f64).to_bits()));
    assert_eq!((-0.0_f64 as f32).to_bits(), 1_u32 << 31);
    let mut wrap_count = 0;
    for source_width in [8, 16, 32, 64] {
        for source_signed in [false, true] {
            let minimum = if source_signed {
                -(1_i128 << (source_width - 1))
            } else {
                0
            };
            let maximum = (1_i128 << (source_width - u32::from(source_signed))) - 1;
            for value in [minimum, minimum + 1, -1, 0, 1, maximum - 1, maximum] {
                if value < minimum || value > maximum {
                    continue;
                }
                for destination_width in [8, 16, 32, 64] {
                    for destination_signed in [false, true] {
                        check_wrap(
                            value,
                            destination_width,
                            destination_signed,
                            &mut wrap_count,
                        );
                    }
                }
            }
        }
    }
    assert_eq!(-1_i8 as u32, u32::MAX);
    assert_ne!(-1_i8 as u8 as u32, u32::MAX);
    println!(
        "{count} domain comparisons and {wrap_count} modular comparisons passed; maximum round-trip and zero-extension controls rejected"
    );
}
