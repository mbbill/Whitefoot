#![forbid(unsafe_code)]

#[inline(never)]
pub fn full_scan(input: &[u8]) -> u64 {
    let mut lines = 0_u64;
    let mut candidates = 0_u64;
    let mut offset = 0_usize;
    while offset < input.len() {
        let byte = input[offset];
        lines = lines.wrapping_add((byte == 10) as u64);
        candidates = candidates.wrapping_add((byte == 80) as u64);
        offset += 1;
    }
    lines.wrapping_add(candidates.wrapping_mul(1_000_000_007))
}

#[inline(never)]
fn find_byte(input: &[u8], needle: u8) -> u64 {
    let mut offset = 0_usize;
    while offset < input.len() {
        if input[offset] == needle {
            return offset as u64;
        }
        offset += 1;
    }
    input.len() as u64
}

#[inline(never)]
pub fn early_scan(input: &[u8]) -> u64 {
    find_byte(input, 251)
        .wrapping_add(find_byte(input, 252))
        .wrapping_add(find_byte(input, 253))
        .wrapping_add(find_byte(input, 254))
}
