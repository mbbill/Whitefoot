#![forbid(unsafe_code)]

use std::{collections::BTreeMap, env, fs, process::Command};

fn write_changed(path: &str, content: &str) {
    if fs::read_to_string(path).ok().as_deref() != Some(content) {
        fs::write(path, content).expect("write generated artifact");
    }
}

fn element_code(lanes: usize) -> (String, String, String) {
    let mut build = String::new();
    let mut update = String::new();
    let mut consume = String::new();
    let fields = ["a", "b", "c", "d"];
    if lanes == 4 {
        for field in fields {
            update.push_str(&format!("      let {field} = 0_u64;\n"));
        }
        update.push_str("      region {\n        let previous = &values[at];\n");
        consume.push_str("    region {\n      let value = &values[at];\n");
    }
    for field in fields.iter().take(lanes) {
        build.push_str(&format!(
            "    let product_{field} = state *wrap 6364136223846793005_u64;\n    set state = product_{field} +wrap 1442695040888963407_u64;\n    let {field} = state;\n"
        ));
        let previous = if lanes == 1 {
            "previous".to_string()
        } else {
            format!("deref(previous).{field}")
        };
        let indent = if lanes == 1 { "      " } else { "        " };
        let bind = if lanes == 1 { "let" } else { "set" };
        update.push_str(&format!(
            "{indent}let product_{field} = state *wrap 2862933555777941757_u64;\n{indent}set state = product_{field} +wrap {previous};\n{indent}{bind} {field} = state;\n"
        ));
        let value = if lanes == 1 {
            "value".to_string()
        } else {
            format!("deref(value).{field}")
        };
        let indent = if lanes == 1 { "    " } else { "      " };
        consume.push_str(&format!(
            "{indent}let product_{field} = checksum *wrap 1099511628211_u64;\n{indent}set checksum = product_{field} +wrap {value};\n"
        ));
    }
    if lanes == 1 {
        build.push_str("    set values = place_back(vector: move values, value: a);");
        update.push_str("      set values[at] = a;");
    } else {
        build.push_str("    let entry = Wide(a: a, b: b, c: c, d: d);\n    set values = place_back(vector: move values, value: move entry);");
        update.push_str("      }\n      let entry = Wide(a: a, b: b, c: c, d: d);\n      let previous_entry = replace values[at] = move entry;");
        consume.push_str("    }");
    }
    (build, update, consume)
}

fn oracle(size: usize, seed: u64, rounds: u64) -> u64 {
    let mut state = seed;
    let mut values: Vec<u64> = (0..size)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state
        })
        .collect();
    state = seed;
    for _ in 0..rounds {
        for value in &mut values {
            state = state
                .wrapping_mul(2_862_933_555_777_941_757)
                .wrapping_add(*value);
            *value = state;
        }
    }
    values.into_iter().fold(0u64, |checksum, value| {
        checksum.wrapping_mul(1_099_511_628_211).wrapping_add(value)
    })
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    match arguments.get(1).map(String::as_str) {
        Some("generate") if arguments.len() == 5 => {
            let (size, lanes) = arguments[3].split_once('x').map_or_else(
                || (arguments[3].parse::<usize>().expect("positive size"), 1),
                |(size, lanes)| (size.parse().expect("size"), lanes.parse().expect("lanes")),
            );
            assert!(size > 0);
            assert!(lanes == 1 || lanes == 4);
            let template = fs::read_to_string(&arguments[2]).expect("read template");
            let (build, update, consume) = element_code(lanes);
            let mut source = template
                .replace("@N@", &size.to_string())
                .replace("@EXPECTED@", &oracle(size * lanes, 19, 3).to_string())
                .replace("@TYPE@", if lanes == 1 { "u64" } else { "Wide" })
                .replace(
                    "@DECLARATION@",
                    if lanes == 1 {
                        ""
                    } else {
                        "struct Wide {\n  a: u64;\n  b: u64;\n  c: u64;\n  d: u64;\n}\n"
                    },
                )
                .replace("@BUILD_ELEMENT@", &build)
                .replace("@UPDATE_ELEMENT@", update.trim_end())
                .replace("@CONSUME_ELEMENT@", consume.trim_end());
            assert!(!source.contains("@TYPE@"));
            if lanes == 4 {
                source = source
                    .replace("      let previous = values[at];\n", "")
                    .replace("    let value = values[at];\n", "");
            }
            write_changed(&arguments[4], source.trim_start_matches('\n'));
        }
        Some("adapt") if arguments.len() == 4 => {
            let module = fs::read_to_string(&arguments[2]).expect("read emitted module");
            let symbol = "define internal i64 @wf_dense(i64 ";
            assert_eq!(module.matches(symbol).count(), 1, "unexpected measured ABI");
            assert_eq!(module.matches("define i32 @main(").count(), 1);
            let adapted = module
                .replacen(symbol, "define i64 @wf_dense(i64 ", 1)
                .replacen("define i32 @main(", "define i32 @wf_fixture_main(", 1);
            write_changed(&arguments[3], &adapted);
        }
        Some("probe") if arguments.len() == 5 => {
            let output = Command::new(&arguments[2])
                .args(["--emit-llvm", &arguments[3]])
                .output()
                .expect("run wide-record compiler probe");
            let diagnostic = String::from_utf8_lossy(&output.stderr);
            if output.status.success() {
                fs::write(&arguments[4], "supported: add wide runtime measurements\n")
                    .expect("write capability result");
                println!("wide-record probe: supported");
            } else {
                assert!(
                    diagnostic.contains("Semantics/Unsupported")
                        && diagnostic.contains("RegionsAndBorrows"),
                    "unexpected wide-record result: {diagnostic}"
                );
                fs::write(&arguments[4], diagnostic.as_bytes()).expect("write capability result");
                println!("wide-record probe: compiler capability unsupported (RegionsAndBorrows)");
            }
        }
        Some("summarize") if arguments.len() == 3 => {
            let input = fs::read_to_string(&arguments[2]).expect("read measurements");
            let mut groups: BTreeMap<(usize, usize, String, u64), Vec<f64>> = BTreeMap::new();
            for line in input.lines() {
                let fields: Vec<_> = line.split(',').collect();
                assert_eq!(fields.len(), 9);
                assert_eq!(fields[0], "sample");
                let key = (
                    fields[1].parse().expect("size"),
                    fields[2].parse().expect("lanes"),
                    fields[3].to_string(),
                    fields[4].parse().expect("rounds"),
                );
                let iterations = fields[6].parse::<f64>().expect("iterations");
                let elapsed = fields[7].parse::<f64>().expect("elapsed ns");
                groups.entry(key).or_default().push(elapsed / iterations);
            }
            println!("size,lanes,variant,rounds,median_ns,min_ns,max_ns");
            for ((size, lanes, variant, rounds), mut samples) in groups {
                assert_eq!(samples.len(), 7);
                samples.sort_by(f64::total_cmp);
                println!(
                    "{size},{lanes},{variant},{rounds},{:.2},{:.2},{:.2}",
                    samples[3], samples[0], samples[6]
                );
            }
        }
        _ => panic!(
            "usage: driver generate TEMPLATE SIZE OUTPUT | adapt INPUT OUTPUT | probe COMPILER SOURCE REPORT | summarize CSV"
        ),
    }
}
