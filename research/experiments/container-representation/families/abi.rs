#![forbid(unsafe_code)]

use std::{env, fs};

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "usage: abi INPUT OUTPUT SYMBOL | abi --observe-allocations INPUT OUTPUT"
    );
    if args[1] == "--observe-allocations" {
        let input = fs::read_to_string(&args[2]).expect("read compiler module");
        fs::write(&args[3], observe_allocations(&input))
            .expect("write allocation observer adapter");
        return;
    }
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

fn observe_allocations(input: &str) -> String {
    for declaration in ["declare ptr @malloc(i64)", "declare void @free(ptr)"] {
        assert_eq!(
            input.lines().filter(|line| *line == declaration).count(),
            1,
            "unexpected allocator declaration: {declaration}"
        );
    }
    let main_headers: Vec<_> = input
        .lines()
        .filter(|line| line.starts_with("define ") && line.contains("@main("))
        .collect();
    assert_eq!(main_headers.len(), 1, "expected one fixture entry point");
    assert!(
        main_headers[0].starts_with("define i32 @main(i32 %argc, ptr %argv) "),
        "unexpected fixture main signature"
    );

    let symbols = [
        ("@malloc", "@wf_observe_allocate"),
        ("@free", "@wf_observe_release"),
        ("@main", "@wf_fixture_main"),
    ];
    for (_, destination) in symbols {
        assert!(
            symbol_positions(input, destination).next().is_none(),
            "observer symbol already exists: {destination}"
        );
    }
    let mut output = input.to_owned();
    for (source, destination) in symbols {
        output = rename_symbol(&output, source, destination);
    }
    let mut restored = output.clone();
    for (source, destination) in symbols {
        restored = rename_symbol(&restored, destination, source);
    }
    assert_eq!(restored, input, "adapter changed more than symbol names");
    output
}

// Compiler-emitted unquoted LLVM identifiers may contain punctuation, so a
// substring replacement must not also rename a function such as @free_list.
fn symbol_positions<'a>(module: &'a str, symbol: &'a str) -> impl Iterator<Item = usize> + 'a {
    module
        .match_indices(symbol)
        .filter_map(move |(position, _)| {
            let next = module.as_bytes().get(position + symbol.len());
            let continues = next.is_some_and(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'$' | b'.' | b'_')
            });
            (!continues).then_some(position)
        })
}

fn rename_symbol(module: &str, source: &str, destination: &str) -> String {
    let mut output = String::with_capacity(module.len());
    let mut copied = 0;
    for position in symbol_positions(module, source) {
        output.push_str(&module[copied..position]);
        output.push_str(destination);
        copied = position + source.len();
    }
    output.push_str(&module[copied..]);
    output
}
