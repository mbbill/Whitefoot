#[no_mangle]
pub extern "C" fn accumulate(acc: &mut u64, addend: &u64, n: u64) {
    let mut i: u64 = 0;
    while i < n {
        *acc = (*acc ^ *addend).wrapping_mul(0x9E3779B97F4A7C15);
        i = i.wrapping_add(1);
    }
}
