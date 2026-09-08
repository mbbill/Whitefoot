#![forbid(unsafe_code)]

use std::{env, fs};

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 4, "usage: abi INPUT OUTPUT SYMBOL");
    let input = fs::read_to_string(&args[1]).expect("read compiler module");
    let old = format!("define internal i64 @{}(i64 ", args[3]);
    let new = format!("define i64 @{}(i64 ", args[3]);
    assert_eq!(input.matches(&old).count(), 1, "unexpected scalar ABI");
    assert_eq!(input.matches("define i32 @main(").count(), 1);
    let output = input.replacen(&old, &new, 1).replacen(
        "define i32 @main(",
        "define i32 @wf_fixture_main(",
        1,
    );
    fs::write(&args[2], output).expect("write linkage-only adapter");
}
