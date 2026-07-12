#[derive(Debug, PartialEq, Eq)]
enum ParseErr {
    Empty,
    InvalidDigit,
    Overflow,
}

fn parse_u64(bytes: &[u8]) -> Result<u64, ParseErr> {
    if bytes.is_empty() {
        return Err(ParseErr::Empty);
    }
    let mut acc = 0_u64;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return Err(ParseErr::InvalidDigit);
        }
        let digit = u64::from(b - b'0');
        acc = acc.checked_mul(10).ok_or(ParseErr::Overflow)?;
        acc = acc.checked_add(digit).ok_or(ParseErr::Overflow)?;
    }
    Ok(acc)
}

fn main() {
    assert_eq!(parse_u64(b"0"), Ok(0));
    assert_eq!(parse_u64(b"42"), Ok(42));
    assert_eq!(parse_u64(b"18446744073709551615"), Ok(u64::MAX));
    assert_eq!(parse_u64(b"18446744073709551616"), Err(ParseErr::Overflow));
    assert_eq!(parse_u64(b"12x"), Err(ParseErr::InvalidDigit));
    println!("ok");
}
