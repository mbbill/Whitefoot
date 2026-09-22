//! Explicit source-check measurements for loop-reference summaries.
//! Built against the ordinary optimized compiler library; see DESIGN.md.
use std::{env, fs, time::Instant};
use whitefoot::{CompilerLimits, SourceInput, check};

fn chain(holders: usize, nesting: usize) -> Vec<u8> {
    let mut source = String::from(
        "struct Node {\n  value: u64;\n  next: Option<Box<Node>>;\n}\n\nfn walk(root: &Node) -> result: own u64 reads(root) {\n",
    );
    for holder in 0..holders {
        source.push_str(&format!("  let cursor{holder} = root;\n"));
    }
    for level in 0..nesting {
        source.push_str(&format!(
            "{}for (step{level} in 0_u64..2_u64) {{\n",
            "  ".repeat(level + 1)
        ));
    }
    let indent = "  ".repeat(nesting + 1);
    let last = holders - 1;
    source.push_str(&format!(
        "{indent}match deref(cursor{last}).next {{\n{indent}  Some(value: child) => {{\n"
    ));
    for holder in 0..last {
        source.push_str(&format!(
            "{indent}    set cursor{holder} = cursor{};\n",
            holder + 1
        ));
    }
    source.push_str(&format!("{indent}    set cursor{last} = &deref(child).inner;\n{indent}  }}\n{indent}  None() => {{\n{indent}  }}\n{indent}}}\n"));
    for level in (0..nesting).rev() {
        source.push_str(&format!("{}}}\n", "  ".repeat(level + 1)));
    }
    source.push_str("  return deref(cursor0).value;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n");
    source.into_bytes()
}

fn main() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .expect("start compiler thread")
        .join()
        .expect("compiler thread");
}

fn joined(roots: usize) -> Vec<u8> {
    let mut source =
        String::from("struct Node {\n  value: u64;\n  next: Option<Box<Node>>;\n}\n\nfn walk(");
    source.push_str(
        &(0..roots)
            .map(|root| format!("root{root}: &Node"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    source.push_str(") -> result: own u64 ");
    source.push_str(
        &(0..roots)
            .map(|root| format!("reads(root{root})"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    source.push_str(" {\n  let cursor = root0;\n  for (i in 0_u64..2_u64) {\n");
    for root in 0..roots {
        source.push_str(&format!(
            "    if i == {root}_u64 {{\n      set cursor = root{root};\n    }}\n"
        ));
    }
    source.push_str("    match deref(cursor).next {\n      Some(value: child) => {\n        set cursor = &deref(child).inner;\n      }\n      None() => {\n      }\n    }\n  }\n  return deref(cursor).value;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n");
    source.into_bytes()
}

fn run() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().is_some_and(|arg| arg == "--check") {
        for path in &arguments[1..] {
            let source = fs::read(path).expect("read source");
            match check(
                &[SourceInput::new(path, &source)],
                CompilerLimits::default(),
            ) {
                Ok(()) => println!("{path}\taccept"),
                Err(failure) => println!("{path}\t{failure}"),
            }
        }
        return;
    }
    let mut cases = vec![(
        "program".to_owned(),
        fs::read("tests/programs/owned_link_cursors.wf").expect("cursor program"),
    )];
    for holders in [1, 8, 16, 32, 64, 128] {
        cases.push((format!("chain-{holders}"), chain(holders, 1)));
    }
    for nesting in [2, 4, 8] {
        cases.push((format!("nested-{nesting}"), chain(4, nesting)));
    }
    for roots in [2, 8, 32] {
        cases.push((format!("joined-{roots}"), joined(roots)));
    }
    println!("case\tbytes\trun\tcheck_ms");
    for (name, source) in cases {
        for run in 0..5 {
            let start = Instant::now();
            check(
                &[SourceInput::new("measurement.wf", &source)],
                CompilerLimits::default(),
            )
            .unwrap_or_else(|failure| panic!("{name}: {failure}"));
            println!(
                "{name}\t{}\t{}\t{:.3}",
                source.len(),
                run + 1,
                start.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
}
