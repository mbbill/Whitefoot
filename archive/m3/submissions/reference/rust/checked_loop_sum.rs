fn main() {
    let mut i: i32 = 0;
    let mut sum: i32 = 0;
    while i < 5 {
        sum = sum.checked_add(i).expect("sum overflow");
        i = i.checked_add(1).expect("i overflow");
    }
    assert_eq!(sum, 10);
    let doubled = sum.checked_mul(2).expect("double overflow");
    assert_eq!(doubled, 20);
}
