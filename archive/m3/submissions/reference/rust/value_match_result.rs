enum Sign {
    Neg,
    Zero,
    Pos,
}

fn sign_of(x: i32) -> Sign {
    if x < 0 {
        Sign::Neg
    } else if x == 0 {
        Sign::Zero
    } else {
        Sign::Pos
    }
}

fn main() {
    let v = match 40_i32.checked_add(2) {
        Some(w) => w,
        None => return,
    };
    assert_eq!(v, 42);
    match sign_of(v) {
        Sign::Pos => assert_eq!(v, 42),
        Sign::Neg | Sign::Zero => return,
    }
}
