#[derive(Clone, PartialEq, PartialOrd)]
struct Custom(u8);

impl core::iter::Step for Custom {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        let distance = end.0.saturating_sub(start.0) as usize;
        (distance, Some(distance))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        u8::try_from(count).ok().and_then(|step| start.0.checked_add(step)).map(Self)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u8::try_from(count).ok().and_then(|step| start.0.checked_sub(step)).map(Self)
    }
}

fn main() {
    let range = core::range::Range { start: Custom(1), end: Custom(3) };
    let _ = range.iter();
}
