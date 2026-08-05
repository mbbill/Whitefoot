#![forbid(unsafe_code)]

use std::path::Path;

fn replace_once(input: &str, from: &str, to: &str) -> Result<String, String> {
    let count = input.matches(from).count();
    if count != 1 {
        return Err(format!("expected one {from:?}, observed {count}"));
    }
    Ok(input.replacen(from, to, 1))
}

fn run(input: &Path, output: &Path) -> Result<(), String> {
    let module = std::fs::read_to_string(input)
        .map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    let module = replace_once(
        &module,
        "define internal i64 @wf_literal_line(",
        "define i64 @wf_literal_line(",
    )?;
    let module = replace_once(
        &module,
        "define i32 @main()",
        "define internal i32 @wf_experiment_unused_main()",
    )?;
    std::fs::write(output, module)
        .map_err(|error| format!("cannot write {}: {error}", output.display()))
}

fn main() {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let result = match arguments.as_slice() {
        [input, output] => run(Path::new(input), Path::new(output)),
        _ => Err("usage: expose INPUT.ll OUTPUT.ll".to_owned()),
    };
    if let Err(message) = result {
        eprintln!("expose: {message}");
        std::process::exit(1);
    }
}
