#[no_mangle]
pub extern "C" fn accumulate(acc: &mut u64, addend: &u64, n: u64) {
    let mut i: u64 = 0;
    while i < n {
        *acc = (*acc).wrapping_add(*addend);
        i = i.wrapping_add(1);
    }
}
