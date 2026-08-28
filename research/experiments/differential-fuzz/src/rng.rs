//! One seeded generator, so a campaign is a range of integers and a finding is
//! one integer. Nothing here is cryptographic and nothing here may change
//! without invalidating recorded seeds: a probe file records the seed that
//! produced it, and the campaign report cites seeds as evidence.

/// SplitMix64, the reference constants. It is the smallest generator with
/// enough quality for structural choices, and it needs no state beyond one
/// word, so a seed names one program exactly.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            // Distinct seeds must not produce near-identical first draws, and
            // the raw counter does. One mixing step separates them.
            state: seed
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0xBF58_476D_1CE4_E5B9),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `0..bound`. `bound` of zero is a caller error and yields zero
    /// rather than dividing by it.
    pub fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        self.next_u64() % bound
    }

    /// An inclusive range, the form most of the generator's size choices take.
    pub fn between(&mut self, low: u64, high: u64) -> u64 {
        if high <= low {
            return low;
        }
        low + self.below(high - low + 1)
    }

    pub fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let index = self.below(items.len() as u64) as usize;
        &items[index]
    }
}
