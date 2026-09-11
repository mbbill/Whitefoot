#![forbid(unsafe_code)]

// Retain the actual exclusive run helpers and inspect their optimized cost.
// This changes only inlining attributes; it never rewrites storage or calls.
use std::{env, fs};

fn function<'a>(module: &'a str, name: &str) -> &'a str {
    let symbol = format!("@wf_{name}(");
    let header = module
        .lines()
        .find(|line| line.starts_with("define ") && line.contains(&symbol))
        .expect("missing function definition");
    let start = module.find(header).unwrap();
    let end = start + module[start..].find("\n}").expect("function end") + 2;
    &module[start..end]
}

fn retain(input: &str) -> String {
    let mut output = input.to_owned();
    for (name, signature) in [
        ("push", "i8 @wf_push(ptr %v0, i64 %v1)"),
        ("pop", "i64 @wf_pop(ptr %v0)"),
    ] {
        let body = function(input, name);
        assert!(body.starts_with(&format!("define internal {signature} #0 {{\n")));
        assert_eq!(body.matches(" #0 {").count(), 1);
        let changed = body.replacen(" #0 {", " noinline #0 {", 1);
        output = output.replacen(body, &changed, 1);
    }
    output
}

fn copy_cost(body: &str) -> (u64, u64, u64) {
    let mut copies = 0;
    let mut copy_bytes = 0;
    let mut vector_bytes = 0;
    for line in body.lines() {
        if line.contains("call ")
            && (line.contains("@llvm.memcpy.") || line.contains("@llvm.memmove."))
        {
            copies += 1;
            let length = line.rsplit_once(", i64 ").expect("constant copy size").1;
            copy_bytes += length.split(',').next().unwrap().parse::<u64>().unwrap();
        }
        if let Some((_, vector)) = line.split_once(" = load <") {
            let (count, element) = vector.split_once(" x i").expect("integer vector");
            let bits = element.split('>').next().unwrap().parse::<u64>().unwrap();
            vector_bytes += count.parse::<u64>().unwrap() * bits / 8;
        }
    }
    (copies, copy_bytes, vector_bytes)
}

fn inspect(module: &str) {
    println!("helper,copy_intrinsics,copy_intrinsic_bytes,vector_load_bytes");
    for name in ["push", "pop"] {
        let body = function(module, name);
        let header = body.lines().next().unwrap();
        let group = header.rsplit_once(" #").expect("function attributes").1;
        let number = group.split_whitespace().next().unwrap();
        let prefix = format!("attributes #{number} = {{ ");
        let attributes = module
            .lines()
            .find(|line| line.starts_with(&prefix))
            .expect("defined function attributes");
        assert!(attributes.split_whitespace().any(|word| word == "noinline"));
        let cost = copy_cost(body);
        assert_eq!(
            cost,
            (0, 0, 0),
            "exclusive helper gained aggregate movement"
        );
        assert!(
            !body.contains("alloca "),
            "exclusive helper gained local storage"
        );
        assert!(!body.contains("@llvm.memset."));
        for allocation in ["@malloc(", "@calloc(", "@realloc("] {
            assert!(
                !body.contains(allocation),
                "inline run moved to allocated storage"
            );
        }
        assert!(
            module
                .lines()
                .any(|line| { line.contains("call ") && line.contains(&format!("@wf_{name}(")) })
        );
        println!("{name},{},{},{}", cost.0, cost.1, cost.2);
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: priority-interface INPUT OUTPUT | --inspect OPTIMIZED"
    );
    if args[1] == "--inspect" {
        inspect(&fs::read_to_string(&args[2]).expect("read optimized module"));
    } else {
        let input = fs::read_to_string(&args[1]).expect("read compiler module");
        fs::write(&args[2], retain(&input)).expect("write retained helpers");
    }
}

#[cfg(test)]
mod tests {
    use super::copy_cost;

    #[test]
    fn the_inspector_counts_intrinsic_and_vector_transfers() {
        assert_eq!(copy_cost("  %value = load i64, ptr %slot"), (0, 0, 0));
        assert_eq!(
            copy_cost(
                "  call void @llvm.memcpy.p0.p0.i64(ptr %to, ptr %from, i64 144, i1 false)\n  %wide = load <2 x i64>, ptr %input"
            ),
            (1, 144, 16)
        );
    }
}
