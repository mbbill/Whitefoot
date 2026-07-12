fn main() {
    let mut values = vec![0_u64; 1024];
    for i in 0..values.len() {
        values[i] = i as u64;
    }
    let mut sum = 0_u64;
    for i in 0..values.len() {
        sum = sum.checked_add(values[i]).expect("sum overflow");
    }
    assert_eq!(sum, 523_776);
    assert!(values.get(1024).is_none());
    println!("ok");
}
