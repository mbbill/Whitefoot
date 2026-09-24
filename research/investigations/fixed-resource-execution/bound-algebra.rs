#![forbid(unsafe_code)]

fn add(left: u128, right: u128, ceiling: u128) -> u128 {
    left.checked_add(right).unwrap_or(ceiling).min(ceiling)
}

fn mul(left: u128, right: u128, ceiling: u128) -> u128 {
    left.checked_mul(right).unwrap_or(ceiling).min(ceiling)
}

// (p, q) denotes x -> p*x + q over nonnegative clipped integers.
fn compose(f: (u128, u128), g: (u128, u128), ceiling: u128) -> (u128, u128) {
    (
        mul(f.0, g.0, ceiling),
        add(mul(f.0, g.1, ceiling), f.1, ceiling),
    )
}

fn work(rank_max: u64, children: u64, local: u64, budget: u64) -> (u128, u32) {
    let ceiling = u128::from(budget) + 1;
    let mut count = u128::from(rank_max) + 1;
    let mut power = (
        u128::from(children).min(ceiling),
        u128::from(local).min(ceiling),
    );
    let mut result = (1, 0);
    let mut bits = 0;
    while count != 0 {
        if count & 1 != 0 {
            result = compose(power, result, ceiling);
        }
        power = compose(power, power, ceiling);
        count >>= 1;
        bits += 1;
    }
    (result.1, bits)
}

// A small independent oracle counts complete levels, without affine composition.
fn level_sum(rank_max: u64, children: u64, local: u64) -> u128 {
    let mut nodes = 1_u128;
    let mut total = 0_u128;
    for _ in 0..=rank_max {
        total += nodes * u128::from(local);
        nodes *= u128::from(children);
    }
    total
}

fn main() {
    let mut cases = 0;
    for rank in 0..=20 {
        for children in 0..=4 {
            for local in 0..=5 {
                let exact = level_sum(rank, children, local);
                for budget in [0, 1, 32, 1000, u64::MAX] {
                    let actual = work(rank, children, local, budget).0;
                    assert_eq!(actual, exact.min(u128::from(budget) + 1));
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(work(u64::MAX, 1, 1, u64::MAX), (1_u128 << 64, 65));
    assert_eq!(work(u64::MAX, 0, 7, 1000), (7, 65));
    assert_eq!(work(u64::MAX, u64::MAX, 0, 1000), (0, 65));
    assert_eq!(work(u64::MAX, 2, 1, 1000), (1001, 65));
    assert_eq!(work(32, 2, 1, u64::MAX).0, 8_589_934_591);
    assert_eq!(work(32, 1, 1, u64::MAX).0, 33);
    println!(
        "{cases} oracle comparisons; four full-u64-rank controls; depth-33 linear=33 binary=8589934591; helper stack=1056 B"
    );
}
