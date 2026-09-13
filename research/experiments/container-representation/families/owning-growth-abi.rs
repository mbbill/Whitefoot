#![forbid(unsafe_code)]

#[path = "../linkage.rs"]
mod linkage;

// Native measurement entry points for the compiler's actual helper ABIs.
// They transfer only valid owners prepared by the C harness. No compiler or
// source acceptance rule is changed. The complete WF construction/refusal path
// has its separate three-mode allocator observer.
use std::{env, fs};

const RUN: &str = "{ ptr, i64, i64, i64 }";

fn function<'a>(module: &'a str, name: &str) -> &'a str {
    let prefix = format!("define internal void @{name}(");
    let begin = module.find(&prefix).expect("expected helper definition");
    let end = module[begin..].find("\n}\n").expect("function end") + begin + 3;
    &module[begin..end]
}

fn result_type(module: &str, name: &str) -> String {
    let body = function(module, name);
    let types: Vec<_> = body
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("  store %wf.t")?;
            let (number, tail) = rest.split_once(' ')?;
            tail.starts_with("zeroinitializer, ptr %wf.result")
                .then(|| format!("%wf.t{number}"))
        })
        .collect();
    assert!(
        !types.is_empty(),
        "helper must materialize its complete result"
    );
    assert!(types.iter().all(|ty| ty == &types[0]));
    types[0].clone()
}

fn fields<'a>(module: &'a str, ty: &str) -> &'a str {
    let prefix = format!("{ty} = type ");
    module
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .expect("nominal declaration")
}

fn optimized_function<'a>(module: &'a str, name: &str) -> &'a str {
    let header = module
        .lines()
        .find(|line| emitted_name(line) == Some(name))
        .expect("optimized function definition");
    let begin = module.find(header).unwrap();
    let end = module[begin..].find("\n}\n").expect("function end") + begin + 3;
    &module[begin..end]
}

fn emitted_name(line: &str) -> Option<&str> {
    line.strip_prefix("define ")?;
    Some(line.split_once('@')?.1.split_once('(')?.0.trim_matches('"'))
}

fn calls(body: &str, name: &str) -> bool {
    body.lines().any(|line| {
        line.contains("call ")
            && (line.contains(&format!("@{name}(")) || line.contains(&format!("@\"{name}\"(")))
    })
}

fn behavior_names(module: &str, helper: &str) -> Vec<String> {
    let prefix = format!("wf_key_{helper}$instance$");
    module
        .lines()
        .filter_map(emitted_name)
        .filter(|name| name.starts_with(&prefix))
        .map(str::to_owned)
        .collect()
}

fn descriptor_writes(body: &str) -> u64 {
    let mut places = std::collections::BTreeMap::from([("%descriptor", 0_u64)]);
    for line in body.lines() {
        if let Some((name, gep)) = line.trim().split_once(" = getelementptr ")
            && let Some((element, indices)) = gep.split_once(", ptr %descriptor, ")
        {
            // Older LLVM retains aggregate-field GEPs where newer LLVM emits
            // byte offsets. Both denote the same four-word descriptor.
            let offset = if element.ends_with(RUN) {
                let (zero, field) = indices.split_once(", i32 ").expect("run field GEP");
                assert!(zero == "i32 0" || zero == "i64 0");
                let field = field.parse::<u64>().expect("constant descriptor field");
                assert!(field < 4);
                field * 8
            } else {
                let index = indices.strip_prefix("i64 ").expect("constant GEP index");
                let index = index.parse::<u64>().expect("constant descriptor offset");
                if element.ends_with(" i8") {
                    index
                } else {
                    assert!(element.ends_with(" i64"), "unexpected descriptor GEP type");
                    index * 8
                }
            };
            places.insert(name, offset);
        }
    }
    let mut writes = std::collections::BTreeSet::new();
    for line in body.lines() {
        let Some(store) = line.trim().strip_prefix("store ") else {
            continue;
        };
        let Some((value, destination)) = store.split_once(", ptr ") else {
            continue;
        };
        let destination = destination.split(',').next().unwrap();
        if let Some(offset) = places.get(destination) {
            let bytes = if value.starts_with("ptr ") || value.starts_with("i64 ") {
                8
            } else if let Some(vector) = value.strip_prefix('<') {
                let (lanes, _) = vector
                    .split_once(" x i64> ")
                    .expect("descriptor vector type");
                lanes.parse::<u64>().expect("vector lane count") * 8
            } else {
                panic!("descriptor store type changed");
            };
            for byte in *offset..*offset + bytes {
                assert!(writes.insert(byte), "overlapping descriptor stores");
            }
        }
    }
    assert_eq!(
        writes,
        (0..writes.len() as u64).collect(),
        "descriptor write coverage:\n{body}"
    );
    assert!(
        writes.is_empty() || writes.len() == 32,
        "descriptor write coverage:\n{body}"
    );
    writes.len() as u64
}

#[cfg(test)]
mod tests {
    use super::{RUN, descriptor_writes};

    fn stores(aggregate: bool) -> String {
        let mut body = String::from("  store ptr %backing, ptr %descriptor, align 8\n");
        for field in 1..4 {
            let address = if aggregate {
                format!("{RUN}, ptr %descriptor, i64 0, i32 {field}")
            } else {
                format!("i8, ptr %descriptor, i64 {}", field * 8)
            };
            body.push_str(&format!(
                "  %field{field} = getelementptr inbounds {address}\n  store i64 %word{field}, ptr %field{field}, align 8\n"
            ));
        }
        body
    }

    #[test]
    fn aggregate_and_byte_geps_cover_the_same_descriptor() {
        assert_eq!(descriptor_writes(&stores(true)), 32);
        assert_eq!(descriptor_writes(&stores(false)), 32);
    }

    #[test]
    #[should_panic(expected = "descriptor write coverage")]
    fn missing_aggregate_field_is_not_a_complete_transfer() {
        let body = stores(true).replace("  store i64 %word2, ptr %field2, align 8\n", "");
        descriptor_writes(&body);
    }
}

fn inspect(paths: &[String], behavior: bool) {
    for (variant, path) in ["normal", "retained", "inlined"].iter().zip(paths) {
        let module = fs::read_to_string(path).expect("read optimized module");
        let probe = optimized_function(&module, "wf_map_heap_probe");
        let provider = probe.lines().next().unwrap().split_once('(').unwrap().1;
        let provider = provider.split(',').next().unwrap();
        assert!(
            provider.contains("readnone"),
            "Heap provider gained memory access"
        );
        assert!(provider.contains("nocapture") || provider.contains("captures(none)"));
        for (bridge, helper) in [
            ("wf_map_put_bridge", "wf_put"),
            ("wf_map_find_bridge", "wf_find"),
            ("wf_map_grow_bridge", "wf_rehash_all"),
        ] {
            let body = optimized_function(&module, bridge);
            let names = if behavior {
                behavior_names(&module, helper.strip_prefix("wf_").unwrap())
            } else {
                vec![helper.to_owned()]
            };
            let calls_helper = names.iter().any(|name| calls(body, name));
            if *variant == "retained" {
                assert!(calls_helper, "retained helper call disappeared");
            }
            if *variant == "inlined" {
                for name in module
                    .lines()
                    .filter_map(emitted_name)
                    .filter(|name| name.starts_with("wf_key_"))
                {
                    assert!(
                        !calls(body, name),
                        "instantiated core helper did not inline"
                    );
                }
                for name in [
                    "wf_put",
                    "wf_find",
                    "wf_rehash_all",
                    "wf_advance",
                    "wf_begin",
                    "wf_fill_slots",
                    "wf_probe",
                    "wf_read_slot",
                    "wf_occupied",
                    "wf_slot_key",
                    "wf_fingerprint",
                    "wf_probe_index",
                ] {
                    assert!(
                        !body.contains(&format!("@{name}(")),
                        "selected helper did not inline"
                    );
                }
                assert!(
                    !body.contains("@llvm.memcpy") && !body.contains("@llvm.memmove"),
                    "aggregate transfer returned to the inlined body"
                );
            }
            let bytes = descriptor_writes(body);
            if bridge != "wf_map_grow_bridge" || *variant == "retained" {
                assert_eq!(
                    bytes, 0,
                    "an exclusive helper bridge must not copy a run back"
                );
            }
            if bridge == "wf_map_grow_bridge" && *variant == "inlined" {
                assert_eq!(bytes, 32, "growth must commit its genuinely new backing");
            }
            println!(
                "owning-growth IR: {variant} {bridge}: helper_call={calls_helper}; receiver_descriptor_write={bytes} bytes"
            );
        }
        if *variant == "retained" {
            let grows = if behavior {
                behavior_names(&module, "rehash_all")
            } else {
                vec!["wf_rehash_all".to_owned()]
            };
            let advances = if behavior {
                behavior_names(&module, "advance")
            } else {
                vec!["wf_advance".to_owned()]
            };
            assert!(!grows.is_empty() && !advances.is_empty());
            for grow in &grows {
                assert!(
                    advances
                        .iter()
                        .any(|advance| calls(optimized_function(&module, grow), advance))
                );
            }
            for helper in grows.iter().chain(&advances) {
                for line in optimized_function(&module, helper).lines().filter(|line| {
                    line.contains("call ")
                        && (line.contains("@llvm.memcpy") || line.contains("@llvm.memmove"))
                }) {
                    println!("owning-growth IR: retained {helper}: {}", line.trim());
                }
            }
        }
        if *variant == "inlined" {
            let grow = optimized_function(&module, "wf_map_grow_bridge");
            assert!(
                grow.lines()
                    .any(|line| line.contains("@llvm.memset") && line.contains("i64 24,")),
                "target slot initialization shape changed; reassess attribution"
            );
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if matches!(
        args.get(1).map(String::as_str),
        Some("--inspect" | "--inspect-behavior")
    ) {
        assert_eq!(
            args.len(),
            5,
            "usage: owning-growth-abi --inspect NORMAL RETAINED INLINED"
        );
        inspect(&args[2..], args[1] == "--inspect-behavior");
        return;
    }
    assert_eq!(
        args.len(),
        4,
        "usage: owning-growth-abi INPUT OUTPUT normal|retained|inlined"
    );
    let input =
        linkage::closed_helpers(&fs::read_to_string(&args[1]).expect("read compiler module"));
    let behavior = !behavior_names(&input, "put").is_empty();
    let variant = args[3].as_str();
    assert!(matches!(variant, "normal" | "retained" | "inlined"));
    let put = result_type(&input, "wf_put");
    let find = result_type(&input, "wf_find");
    let grow = result_type(&input, "wf_rehash_all");
    assert_eq!(fields(&input, &put), "{ i32, ptr, ptr }");
    let option = fields(&input, &find)
        .strip_prefix("{ ")
        .and_then(|text| text.strip_suffix(", i64 }"))
        .expect("find returns one option and the examination count");
    assert_eq!(fields(&input, option), "{ i32, i64 }");
    assert_eq!(fields(&input, &grow), "{ i8, i64, i64 }");
    for (name, signature) in [
        (
            "wf_put",
            "ptr %wf.result, ptr %v0, i64 %v1, ptr %v2".to_owned(),
        ),
        ("wf_find", "ptr %wf.result, ptr %v0, i64 %v1".to_owned()),
        (
            "wf_rehash_all",
            "ptr %wf.result, ptr %v0, ptr %v1, i64 %v2".to_owned(),
        ),
    ] {
        assert!(function(&input, name).starts_with(&format!(
            "define internal void @{name}({signature}) #0 {{\n"
        )));
    }
    let mut output = input
        .replace("@malloc(", "@wf_cost_allocate(")
        .replace("@free(", "@wf_cost_release(")
        .replace("define i32 @main(", "define i32 @wf_fixture_main(");
    let retained = ["wf_put", "wf_find", "wf_rehash_all", "wf_advance"];
    let inlined = [
        "wf_put",
        "wf_find",
        "wf_rehash_all",
        "wf_advance",
        "wf_begin",
        "wf_fill_slots",
        "wf_probe",
        "wf_read_slot",
        "wf_occupied",
        "wf_slot_key",
        "wf_fingerprint",
        "wf_probe_index",
    ];
    let selected: &[&str] = if behavior {
        &[]
    } else {
        match variant {
            "retained" => &retained,
            "inlined" => &inlined,
            _ => &[],
        }
    };
    for name in selected {
        let header = output
            .lines()
            .find(|line| {
                line.starts_with("define internal ") && line.contains(&format!("@{name}("))
            })
            .expect("selected helper exists")
            .to_owned();
        let attribute = if variant == "retained" {
            "noinline"
        } else {
            "alwaysinline"
        };
        let changed = header.replace(" #0 {", &format!(" {attribute} #0 {{"));
        assert_ne!(header, changed);
        output = output.replacen(&header, &changed, 1);
    }
    if behavior && variant != "normal" {
        let mut retained_core = std::collections::BTreeSet::new();
        for header in input
            .lines()
            .filter(|line| line.starts_with("define internal "))
        {
            let name = emitted_name(header).unwrap();
            let core = name
                .strip_prefix("wf_key_")
                .and_then(|name| name.split_once("$instance$"))
                .map(|(name, _)| name);
            let boundary = matches!(core, Some("put" | "find" | "rehash_all" | "advance"));
            let adapter = retained.contains(&name);
            let attribute = if variant == "retained" && boundary {
                retained_core.insert(core.unwrap());
                Some("noinline")
            } else if adapter
                || (variant == "inlined" && (core.is_some() || inlined.contains(&name)))
            {
                Some("alwaysinline")
            } else {
                None
            };
            if let Some(attribute) = attribute {
                let changed = header.replace(" #0 {", &format!(" {attribute} #0 {{"));
                assert_ne!(header, changed);
                output = output.replacen(header, &changed, 1);
            }
        }
        if variant == "retained" {
            assert_eq!(
                retained_core,
                std::collections::BTreeSet::from(["put", "find", "rehash_all", "advance"])
            );
        }
    }
    output.push_str(&format!(
        r#"
define i32 @wf_map_put_bridge(ptr %descriptor, i64 %key, ptr %owner, ptr %returned) noinline {{
entry:
  %result = alloca {put}, align 8
  call void @wf_put(ptr %result, ptr %descriptor, i64 %key, ptr %owner)
  %tag = load i32, ptr %result
  %old.at = getelementptr inbounds {put}, ptr %result, i32 0, i32 1
  %old = load ptr, ptr %old.at
  %full.at = getelementptr inbounds {put}, ptr %result, i32 0, i32 2
  %full = load ptr, ptr %full.at
  %replaced = icmp eq i32 %tag, 1
  %back = select i1 %replaced, ptr %old, ptr %full
  store ptr %back, ptr %returned
  ret i32 %tag
}}

define i64 @wf_map_find_bridge(ptr %descriptor, i64 %key) noinline {{
entry:
  %result = alloca {find}, align 8
  call void @wf_find(ptr %result, ptr %descriptor, i64 %key)
  %tag = load i32, ptr %result
  %id.at = getelementptr inbounds {find}, ptr %result, i32 0, i32 0, i32 1
  %id = load i64, ptr %id.at
  %present = icmp eq i32 %tag, 1
  %observed = select i1 %present, i64 %id, i64 0
  %work.at = getelementptr inbounds {find}, ptr %result, i32 0, i32 1
  %work = load i64, ptr %work.at
  %prefix = mul i64 %observed, 131
  %checksum = add i64 %prefix, %work
  ret i64 %checksum
}}

define i64 @wf_map_grow_bridge(ptr %descriptor, i64 %count, ptr %code) noinline {{
entry:
  %result = alloca {grow}, align 8
  call void @wf_rehash_all(ptr %result, ptr null, ptr %descriptor, i64 %count)
  %code.at = getelementptr inbounds {grow}, ptr %result, i32 0, i32 0
  %status = load i8, ptr %code.at
  store i8 %status, ptr %code
  %work.at = getelementptr inbounds {grow}, ptr %result, i32 0, i32 1
  %work = load i64, ptr %work.at
  %moved.at = getelementptr inbounds {grow}, ptr %result, i32 0, i32 2
  %moved = load i64, ptr %moved.at
  %prefix = mul i64 %work, 131
  %checksum = add i64 %prefix, %moved
  ret i64 %checksum
}}

; An externally supplied, unknown Heap prevents null-argument specialization
; from hiding a future provider access. The inspector requires this parameter
; to be readnone and noncapturing before the timed bridge may supply null.
define void @wf_map_heap_probe(ptr %heap, ptr %result, ptr %descriptor, i64 %count) noinline {{
entry:
  call void @wf_rehash_all(ptr %result, ptr %heap, ptr %descriptor, i64 %count)
  ret void
}}
"#
    ));
    assert_eq!(input.matches("declare ptr @malloc(i64)").count(), 1);
    assert_eq!(input.matches("declare void @free(ptr)").count(), 1);
    assert_eq!(output.matches("define i32 @wf_fixture_main(").count(), 1);
    fs::write(&args[2], output).expect("write measurement module");
}
