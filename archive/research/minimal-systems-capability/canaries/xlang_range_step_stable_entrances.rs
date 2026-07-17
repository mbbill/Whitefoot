use core::net::{Ipv4Addr, Ipv6Addr};
use core::num::NonZero;
use core::range::{Range, RangeFrom, RangeInclusive};

macro_rules! check_range_entrances {
    ($ty:ty, $start:expr, $end:expr) => {{
        let start: $ty = $start;
        let end: $ty = $end;

        let half_open = Range { start, end };
        let mut half_open_iter = half_open.iter();
        let _: Option<$ty> = half_open_iter.next();

        let from = RangeFrom { start };
        let mut from_iter = from.iter();
        let _: Option<$ty> = from_iter.next();

        let inclusive = RangeInclusive { start, last: end };
        let mut inclusive_iter = inclusive.iter();
        let _: Option<$ty> = inclusive_iter.next();
    }};
}

fn main() {
    check_range_entrances!(u8, 1, 3);
    check_range_entrances!(u16, 1, 3);
    check_range_entrances!(u32, 1, 3);
    check_range_entrances!(u64, 1, 3);
    check_range_entrances!(u128, 1, 3);
    check_range_entrances!(usize, 1, 3);
    check_range_entrances!(i8, 1, 3);
    check_range_entrances!(i16, 1, 3);
    check_range_entrances!(i32, 1, 3);
    check_range_entrances!(i64, 1, 3);
    check_range_entrances!(i128, 1, 3);
    check_range_entrances!(isize, 1, 3);
    check_range_entrances!(char, 'a', 'c');
    check_range_entrances!(Ipv4Addr, Ipv4Addr::from(1_u32), Ipv4Addr::from(3_u32));
    check_range_entrances!(Ipv6Addr, Ipv6Addr::from(1_u128), Ipv6Addr::from(3_u128));
    check_range_entrances!(NonZero<u8>, NonZero::new(1_u8).unwrap(), NonZero::new(3_u8).unwrap());
    check_range_entrances!(NonZero<u16>, NonZero::new(1_u16).unwrap(), NonZero::new(3_u16).unwrap());
    check_range_entrances!(NonZero<u32>, NonZero::new(1_u32).unwrap(), NonZero::new(3_u32).unwrap());
    check_range_entrances!(NonZero<u64>, NonZero::new(1_u64).unwrap(), NonZero::new(3_u64).unwrap());
    check_range_entrances!(NonZero<u128>, NonZero::new(1_u128).unwrap(), NonZero::new(3_u128).unwrap());
    check_range_entrances!(NonZero<usize>, NonZero::new(1_usize).unwrap(), NonZero::new(3_usize).unwrap());
}
