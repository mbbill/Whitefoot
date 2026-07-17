use core::ascii::Char;
use core::range::Range;

fn rejected(start: Char, end: Char) {
    let range = Range { start, end };
    let _ = range.iter();
}

fn main() {}
