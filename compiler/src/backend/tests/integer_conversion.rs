use std::fmt::Write;

use super::{compile, compile_and_run};

#[derive(Clone, Copy)]
struct IntegerType {
    spelling: &'static str,
    width: u8,
    signed: bool,
}

const INTEGER_TYPES: [IntegerType; 8] = [
    IntegerType {
        spelling: "i8",
        width: 8,
        signed: true,
    },
    IntegerType {
        spelling: "i16",
        width: 16,
        signed: true,
    },
    IntegerType {
        spelling: "i32",
        width: 32,
        signed: true,
    },
    IntegerType {
        spelling: "i64",
        width: 64,
        signed: true,
    },
    IntegerType {
        spelling: "u8",
        width: 8,
        signed: false,
    },
    IntegerType {
        spelling: "u16",
        width: 16,
        signed: false,
    },
    IntegerType {
        spelling: "u32",
        width: 32,
        signed: false,
    },
    IntegerType {
        spelling: "u64",
        width: 64,
        signed: false,
    },
];

#[test]
fn executes_exact_success_and_failure_edges_for_every_integer_pair() {
    let mut source = String::from("fn main() -> own unit traps {\n");
    let mut total_count = 0;
    let mut checked_count = 0;
    for source_type in INTEGER_TYPES {
        for destination_type in INTEGER_TYPES {
            if source_type.spelling == destination_type.spelling {
                continue;
            }
            if converts_totally(source_type, destination_type) {
                let value = total_value(source_type);
                writeln!(
                    source,
                    "  let total{total_count} = cvt<{source_type}, {destination}>({value}_{source_type});\n  check total{total_count} == {value}_{destination} else trap \"total conversion {total_count}\";",
                    destination = destination_type.spelling,
                    source_type = source_type.spelling,
                )
                .expect("write total conversion");
                total_count += 1;
                continue;
            }

            let failure = failing_value(source_type, destination_type);
            writeln!(
                source,
                "  let success{checked_count} = cvt<{source_type}, {destination}>(1_{source_type});\n  match move success{checked_count} {{\n    Ok(value: success_value{checked_count}) => {{\n      check success_value{checked_count} == 1_{destination} else trap \"checked success value {checked_count}\";\n    }}\n    Err(error: success_error{checked_count}) => {{\n      check False() else trap \"checked success became error {checked_count}\";\n    }}\n  }}\n  let failure{checked_count} = cvt<{source_type}, {destination}>({failure}_{source_type});\n  match move failure{checked_count} {{\n    Ok(value: failure_value{checked_count}) => {{\n      check False() else trap \"unrepresentable conversion succeeded {checked_count}\";\n    }}\n    Err(error: failure_error{checked_count}) => {{\n      match failure_error{checked_count} {{\n        NarrowError() => {{\n        }}\n      }}\n    }}\n  }}",
                destination = destination_type.spelling,
                source_type = source_type.spelling,
            )
            .expect("write checked conversion");
            checked_count += 1;
        }
    }
    source.push_str("  return unit;\n}\n");
    assert_eq!(total_count, 18);
    assert_eq!(checked_count, 38);

    let llvm = compile(source.as_bytes());
    for instruction in [
        " = sext ",
        " = zext ",
        " = trunc ",
        " = icmp sge ",
        " = icmp ule ",
    ] {
        assert!(
            llvm.contains(instruction),
            "conversion matrix must exercise {instruction}"
        );
    }
    assert!(!llvm.contains(" nsw "));
    assert!(!llvm.contains(" nuw "));
    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "integer conversion matrix failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_crc32_vector_executes() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/x-crc32-standard-vector-run.wf");
    let output = compile_and_run(&compile(source));
    assert!(
        output.status.success(),
        "CRC32 vector failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

const fn converts_totally(source: IntegerType, destination: IntegerType) -> bool {
    source.width < destination.width
        && (source.signed == destination.signed || (!source.signed && destination.signed))
}

fn total_value(source: IntegerType) -> String {
    if source.signed {
        "-1".to_owned()
    } else {
        ((1_u128 << source.width) - 1).to_string()
    }
}

fn failing_value(source: IntegerType, destination: IntegerType) -> String {
    match (source.signed, destination.signed) {
        (true, false) => "-1".to_owned(),
        (true, true) | (false, true) => (1_u128 << (destination.width - 1)).to_string(),
        (false, false) => (1_u128 << destination.width).to_string(),
    }
}
