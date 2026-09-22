//! Generate matched source forms and record current compiler verdicts.
//! Run through the command in DESIGN.md; all outputs go to its scratch path.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

const PRODUCER: &str = r#"fn selected(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Ok(value: payload): payload == value;
} {
  return Ok<i32, Overflow>(value: value);
}"#;

const GUARD: &str = r#"fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left == right;
} {
  return unit;
}"#;

const ENTRY: &str = r#"fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

struct Case {
    name: &'static str,
    expected_rule: Option<&'static str>,
    source: String,
}

fn indent(source: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    source
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn matched(scrutinee: &str, success: &str) -> String {
    format!(
        "match {scrutinee} {{\n  Ok(value: payload) => {{\n{}\n  }}\n  Err(error: problem) => {{\n  }}\n}}",
        indent(success, 4)
    )
}

fn consumer(body: &str) -> String {
    format!(
        "{PRODUCER}\n\n{GUARD}\n\nfn consumer(input: own i32, choose: own Bool) -> result: own Result<unit, Overflow> pure {{\n{}\n  return Ok<unit, Overflow>(value: unit);\n}}\n\n{ENTRY}",
        indent(body, 2)
    )
}

fn wrapper(body: &str) -> String {
    format!(
        "{PRODUCER}\n\nfn relay(input: own i32) -> result: own Result<i32, Overflow> pure contract {{\n  ensures when Ok(value: payload): payload == input;\n}} {{\n{}\n}}\n\n{ENTRY}",
        indent(body, 2)
    )
}

fn indexed(body: &str) -> String {
    format!(
        "fn bounded(count: own u64) -> result: own Result<u64, Overflow> pure contract {{\n  requires count > 0_u64;\n  ensures when Ok(value: index): index < count;\n}} {{\n  return Ok<u64, Overflow>(value: 0_u64);\n}}\n\nfn read(run: &Slots<u8, 4>) -> result: own Result<unit, Overflow> reads(run) contract {{\n  requires deref(run).len > 0_u64;\n}} {{\n{}\n  return Ok<unit, Overflow>(value: unit);\n}}\n\n{ENTRY}",
        indent(body, 2)
    )
}

fn cases() -> Vec<Case> {
    let use_payload = "guard(left: payload, right: input);";
    let named = matched("outcome", use_payload);
    let mut cases = vec![
        Case {
            name: "direct-match",
            expected_rule: None,
            source: consumer(&matched("selected(value: input)", use_payload)),
        },
        Case {
            name: "named-match",
            expected_rule: Some("FN-8"),
            source: consumer(&format!("let outcome = selected(value: input);\n{named}")),
        },
        Case {
            name: "copied-match",
            expected_rule: Some("FN-8"),
            source: consumer(&format!(
                "let outcome = selected(value: input);\nlet retained = outcome;\n{}",
                matched("retained", use_payload)
            )),
        },
        Case {
            name: "direct-propagate",
            expected_rule: Some("FN-8"),
            source: consumer(&format!(
                "let payload = propagate selected(value: input);\n{use_payload}"
            )),
        },
        Case {
            name: "named-propagate",
            expected_rule: Some("FN-8"),
            source: consumer(&format!(
                "let outcome = selected(value: input);\nlet payload = propagate outcome;\n{use_payload}"
            )),
        },
        Case {
            name: "value-match-delivery",
            expected_rule: Some("FN-8"),
            source: consumer(&format!(
                "let payload = match selected(value: input) {{\n  Ok(value: retained) => {{\n    give retained;\n  }}\n  Err(error: problem) => {{\n    return Err<unit, Overflow>(error: problem);\n  }}\n}}\n{use_payload}"
            )),
        },
        Case {
            name: "direct-unrelated-statement",
            expected_rule: None,
            source: consumer(&matched(
                "selected(value: input)",
                &format!("let unrelated = 0_i32;\n{use_payload}"),
            )),
        },
        Case {
            name: "direct-stale-scalar",
            expected_rule: Some("FN-8"),
            source: consumer(&matched(
                "selected(value: input)",
                &format!("set input = 0_i32;\n{use_payload}"),
            )),
        },
        Case {
            name: "overwritten-outcome",
            expected_rule: Some("FN-8"),
            source: consumer(&format!(
                "let outcome = selected(value: input);\nset outcome = Ok<i32, Overflow>(value: 0_i32);\n{named}"
            )),
        },
        Case {
            name: "mixed-join",
            expected_rule: Some("FN-8"),
            source: consumer(&format!(
                "let outcome = if choose {{\n  give selected(value: input);\n}} else {{\n  give Ok<i32, Overflow>(value: 0_i32);\n}}\n{named}"
            )),
        },
        Case {
            name: "wrapper-direct-return",
            expected_rule: Some("FN-9"),
            source: wrapper("return selected(value: input);"),
        },
        Case {
            name: "wrapper-named-return",
            expected_rule: Some("FN-9"),
            source: wrapper("let outcome = selected(value: input);\nreturn outcome;"),
        },
        Case {
            name: "wrapper-rebuild",
            expected_rule: None,
            source: wrapper(
                "match selected(value: input) {\n  Ok(value: retained) => {\n    return Ok<i32, Overflow>(value: retained);\n  }\n  Err(error: problem) => {\n    return Err<i32, Overflow>(error: problem);\n  }\n}",
            ),
        },
    ];
    let owned_source = consumer(&format!(
        "let outcome = selected(value: input);\nlet retained = move outcome;\n{}",
        matched("retained", use_payload)
    ))
    .replace("Overflow", "Problem");
    cases.push(Case {
        name: "moved-affine-outcome",
        expected_rule: Some("FN-8"),
        source: format!("nocopy enum Problem {{\n  Failed();\n}}\n\n{owned_source}"),
    });
    let identity = "fn identity(value: own i32) -> result: own i32 pure contract {\n  ensures result == value;\n} {\n  return value;\n}";
    cases.push(Case {
        name: "plain-result-copy-control",
        expected_rule: None,
        source: format!(
            "{identity}\n\n{}",
            consumer("let produced = identity(value: input);\nlet payload = produced;\nguard(left: payload, right: input);")
        ),
    });
    cases.push(Case {
        name: "value-if-delivery-control",
        expected_rule: None,
        source: format!(
            "{identity}\n\n{}",
            consumer("let produced = identity(value: input);\nlet payload = if choose {\n  give produced;\n} else {\n  give produced;\n}\nguard(left: payload, right: input);")
        ),
    });
    cases.extend([
        Case {
            name: "index-direct-match",
            expected_rule: None,
            source: indexed(&matched(
                "bounded(count: deref(run).len)",
                "let observed = deref(run)[payload];",
            )),
        },
        Case {
            name: "index-named-match",
            expected_rule: Some("OP-4"),
            source: indexed(&format!(
                "let outcome = bounded(count: deref(run).len);\n{}",
                matched("outcome", "let observed = deref(run)[payload];")
            )),
        },
        Case {
            name: "index-direct-propagate",
            expected_rule: Some("OP-4"),
            source: indexed("let index = propagate bounded(count: deref(run).len);\nlet observed = deref(run)[index];"),
        },
    ]);
    cases
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("usage: probe COMPILER SCRATCH_DIRECTORY".into());
    }
    let scratch = Path::new(&args[2]);
    fs::create_dir_all(scratch)?;
    let mut results = String::from("case,expected,exit,matched\n");
    let mut failures = Vec::new();
    for case in cases() {
        let source = scratch.join(format!("{}.wf", case.name));
        fs::write(&source, case.source)?;
        let output = Command::new(&args[1])
            .arg("--emit-llvm")
            .arg("-o")
            .arg(scratch.join(format!("{}.ll", case.name)))
            .arg(&source)
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let matched = match case.expected_rule {
            None => output.status.success(),
            Some(rule) => {
                output.status.code() == Some(1)
                    && stderr.starts_with(&format!("whitefootc: Semantics/Source [{rule}]:"))
            }
        };
        fs::write(
            scratch.join(format!("{}.stderr", case.name)),
            &output.stderr,
        )?;
        fs::write(
            scratch.join(format!("{}.stdout", case.name)),
            &output.stdout,
        )?;
        results.push_str(&format!(
            "{},{},{},{}\n",
            case.name,
            case.expected_rule.unwrap_or("accept"),
            output
                .status
                .code()
                .map_or("signal".to_owned(), |n| n.to_string()),
            matched
        ));
        if !matched {
            failures.push(format!("{}: {}\n{stderr}", case.name, output.status));
        }
    }
    fs::write(scratch.join("results.csv"), &results)?;
    print!("{results}");
    if !failures.is_empty() {
        return Err(failures.join("\n").into());
    }
    Ok(())
}
