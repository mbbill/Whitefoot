//! Native, same-source checking-cost probe; no timing threshold selects a verdict.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Clone, Copy)]
enum Family {
    Fixed,
    Growing,
    Control,
}

impl Family {
    fn name(self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::Growing => "growing",
            Self::Control => "control",
        }
    }
}

fn source(family: Family, count: usize) -> String {
    assert!((3..=4096).contains(&count));
    let groups = if matches!(family, Family::Fixed) {
        3
    } else {
        count
    };
    assert!(
        groups <= 1024,
        "the target has a separate affine node ceiling"
    );
    let uses = if matches!(family, Family::Control) {
        3
    } else {
        count
    };
    let parameters = (0..groups)
        .map(|i| format!("a{i}: own u64, b{i}: own u64"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut text = format!("fn combine({parameters}) -> result: own unit pure contract {{\n");
    for i in 0..groups {
        writeln!(text, "  requires a{i} <= b{i};").unwrap();
    }
    text.push_str("} {\n");
    let mut coefficients = vec![0; groups];
    let mut slack = 0;
    let mut premises = String::new();
    for i in 0..uses {
        let group = i % groups;
        let weakening = i / groups;
        coefficients[group] += 1;
        slack += weakening;
        let offset = if weakening == 0 {
            String::new()
        } else {
            format!(" + {weakening}_u64")
        };
        writeln!(premises, "    use (a{group} <= b{group}{offset});").unwrap();
    }
    let side = |prefix| {
        coefficients
            .iter()
            .enumerate()
            .filter(|(_, coefficient)| **coefficient != 0)
            .map(|(i, coefficient)| match coefficient {
                1 => format!("{prefix}{i}"),
                n => format!("{n}_u64 * {prefix}{i}"),
            })
            .collect::<Vec<_>>()
            .join(" + ")
    };
    let left = side('a');
    let mut right = side('b');
    if slack != 0 {
        write!(right, " + {slack}_u64").unwrap();
    }
    write!(
        text,
        "  invariant total: {left} <= {right} {{\n{premises}  }}\n  return unit;\n}}\n\n\
         fn main() -> status: own ExitStatus pure {{\n  return exit_status(code: 0_u8);\n}}\n"
    )
    .unwrap();
    text
}

fn run(compiler: &Path, root: &Path, label: &str, source: &str, accepts: bool) -> u128 {
    let input = root.join(format!("{label}.wf"));
    let output = root.join(format!("{label}.ll"));
    std::fs::write(&input, source).expect("write generated source");
    let started = Instant::now();
    let child = Command::new(compiler)
        .args(["--emit-llvm", "-o"])
        .arg(output)
        .arg(input)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start ordinary compiler");
    eprintln!("running {label}, compiler pid {}", child.id());
    let result = child
        .wait_with_output()
        .expect("wait for ordinary compiler");
    let elapsed = started.elapsed().as_micros();
    let diagnostic = String::from_utf8_lossy(&result.stderr);
    assert_eq!(result.status.success(), accepts, "{label}: {diagnostic}");
    if !accepts {
        assert!(diagnostic.contains("PRF-1"), "{label}: {diagnostic}");
    }
    elapsed
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    assert!(
        args.len() >= 4,
        "runner COMPILER WORK_ROOT check|bench [SIZES] [REPETITIONS]"
    );
    let compiler = Path::new(&args[1]);
    let root = Path::new(&args[2]);
    std::fs::create_dir_all(root).expect("create scratch root");
    let sizes = if args[3] == "check" {
        vec![3, 6]
    } else {
        assert_eq!(args[3], "bench");
        args.get(4)
            .map_or("16,32,64,128,256", String::as_str)
            .split(',')
            .map(|size| size.parse::<usize>().expect("integer size"))
            .collect()
    };
    let repetitions = args.get(5).map_or(1, |n| n.parse::<usize>().unwrap());
    assert!(repetitions != 0);
    println!("family\tcount\tsource_bytes\trepetition\telapsed_us\tverdict");
    for count in sizes {
        for family in [Family::Fixed, Family::Growing, Family::Control] {
            if count > 1024 && !matches!(family, Family::Fixed) {
                continue;
            }
            let source = source(family, count);
            for repetition in 0..repetitions {
                let elapsed = run(
                    compiler,
                    root,
                    &format!("{}-{count}", family.name()),
                    &source,
                    true,
                );
                println!(
                    "{}\t{count}\t{}\t{repetition}\t{elapsed}\taccept",
                    family.name(),
                    source.len()
                );
                std::io::stdout().flush().unwrap();
            }
        }
    }
    if args[3] == "check" {
        let valid = source(Family::Growing, 3);
        for (label, bad) in [
            (
                "unproved",
                valid.replace("use (a2 <= b2);", "use (b2 <= a2);"),
            ),
            (
                "duplicate",
                valid.replace("use (a2 <= b2);", "use (a0 <= b0);"),
            ),
        ] {
            run(compiler, root, label, &bad, false);
        }
        println!("checks: six accepted fixtures and two PRF-1 negative controls");
    }
}
