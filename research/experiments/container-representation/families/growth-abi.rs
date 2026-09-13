#![forbid(unsafe_code)]

#[path = "../linkage.rs"]
mod linkage;

use std::{env, fs};

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 3, "usage: growth-abi INPUT OUTPUT");
    if args[1] == "--verify" {
        let optimized = fs::read_to_string(&args[2]).expect("read optimized module");
        let declaration = optimized
            .lines()
            .find(|line| line.starts_with("define i64 @wf_growth_trace("))
            .expect("exported growth definition");
        let (_, parameters) = declaration.split_once('(').expect("parameters");
        let provider = parameters.split(',').next().expect("heap parameter");
        assert!(
            provider.contains("readnone"),
            "heap provider gained memory access"
        );
        assert!(
            provider.contains("nocapture") || provider.contains("captures(none)"),
            "heap provider may escape"
        );
        return;
    }
    let input =
        linkage::closed_helpers(&fs::read_to_string(&args[1]).expect("read compiler module"));
    let signature =
        "define internal i64 @wf_growth_trace(ptr %v0, i8 %v1, i64 %v2, i64 %v3, i64 %v4)";
    assert_eq!(input.matches(signature).count(), 1, "unexpected growth ABI");
    assert_eq!(input.matches("define i32 @main(").count(), 1);
    assert_eq!(input.matches("declare ptr @malloc(i64)").count(), 1);
    assert_eq!(input.matches("declare void @free(ptr)").count(), 1);
    let output = input
        .replacen(signature, &signature.replacen("internal ", "", 1), 1)
        .replacen("define i32 @main(", "define i32 @wf_fixture_main(", 1)
        .replace("@malloc(", "@growth_malloc(")
        .replace("@free(", "@growth_free(");
    fs::write(&args[2], output).expect("write instrumented module");
}
