#[no_mangle]
#[inline(never)]
pub extern "C" fn mix_n(seed: u64, n: u64) -> u64 {
    let mut z: u64 = seed;
    let mut acc: u64 = 0;
    let mut i: u64 = 0;
    while i < n {
        z = z.wrapping_add(0x9E3779B97F4A7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z = z ^ (z >> 31);
        acc ^= z;
        i = i.wrapping_add(1);
    }
    acc
}

fn main() {
    let r = mix_n(0x0123456789ABCDEF, 200000000);
    println!("{}", r);
    std::process::exit((r & 0xFF) as i32);
}
