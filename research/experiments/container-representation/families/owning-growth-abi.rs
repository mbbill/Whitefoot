#![forbid(unsafe_code)]

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
        .find(|line| line.starts_with("define ") && line.contains(&format!("@{name}(")))
        .expect("optimized function definition");
    let begin = module.find(header).unwrap();
    let end = module[begin..].find("\n}\n").expect("function end") + begin + 3;
    &module[begin..end]
}

fn descriptor_writes(body: &str) -> u64 {
    let mut places = std::collections::BTreeMap::from([("%descriptor", 0_u64)]);
    for line in body.lines() {
        if let Some((name, gep)) = line.trim().split_once(" = getelementptr ")
            && let Some((_, offset)) = gep.split_once("i8, ptr %descriptor, i64 ")
        {
            places.insert(name, offset.parse().expect("constant descriptor offset"));
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
    assert_eq!(writes, (0..32).collect());
    32
}

fn inspect(paths: &[String]) {
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
            let calls_helper = body.contains(&format!("@{helper}("));
            if *variant == "retained" {
                assert!(calls_helper, "retained helper call disappeared");
            }
            if *variant == "inlined" {
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
            let bytes = if bridge == "wf_map_find_bridge" {
                0
            } else {
                descriptor_writes(body)
            };
            println!(
                "owning-growth IR: {variant} {bridge}: helper_call={calls_helper}; receiver_descriptor_write={bytes} bytes"
            );
        }
        if *variant == "retained" {
            assert!(optimized_function(&module, "wf_rehash_all").contains("@wf_advance("));
            for helper in ["wf_rehash_all", "wf_advance"] {
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
    if args.get(1).map(String::as_str) == Some("--inspect") {
        assert_eq!(
            args.len(),
            5,
            "usage: owning-growth-abi --inspect NORMAL RETAINED INLINED"
        );
        inspect(&args[2..]);
        return;
    }
    assert_eq!(
        args.len(),
        4,
        "usage: owning-growth-abi INPUT OUTPUT normal|retained|inlined"
    );
    let input = fs::read_to_string(&args[1]).expect("read compiler module");
    let variant = args[3].as_str();
    assert!(matches!(variant, "normal" | "retained" | "inlined"));
    let put = result_type(&input, "wf_put");
    let find = result_type(&input, "wf_find");
    let grow = result_type(&input, "wf_rehash_all");
    let put_fields = fields(&input, &put);
    let outcome = put_fields
        .strip_prefix(&format!("{{ {RUN}, "))
        .and_then(|text| text.strip_suffix(" }"))
        .expect("put returns one run and one outcome");
    assert_eq!(fields(&input, outcome), "{ i32, ptr, ptr }");
    let option = fields(&input, &find)
        .strip_prefix("{ ")
        .and_then(|text| text.strip_suffix(", i64 }"))
        .expect("find returns one option and the examination count");
    assert_eq!(fields(&input, option), "{ i32, i64 }");
    assert_eq!(fields(&input, &grow), format!("{{ {RUN}, i8, i64, i64 }}"));
    for (name, signature) in [
        (
            "wf_put",
            format!("ptr %wf.result, {RUN} %v0, i64 %v2, ptr %v3"),
        ),
        ("wf_find", "ptr %wf.result, ptr %v0, i64 %v1".to_owned()),
        (
            "wf_rehash_all",
            format!("ptr %wf.result, ptr %v0, {RUN} %v1, i64 %v2"),
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
    let selected: &[&str] = match variant {
        "retained" => &retained,
        "inlined" => &inlined,
        _ => &[],
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
    output.push_str(&format!(
        r#"
define i32 @wf_map_put_bridge(ptr %descriptor, i64 %key, ptr %owner, ptr %returned) noinline {{
entry:
  %result = alloca {put}, align 8
  %run = load {RUN}, ptr %descriptor
  call void @wf_put(ptr %result, {RUN} %run, i64 %key, ptr %owner)
  %new.run = load {RUN}, ptr %result
  store {RUN} %new.run, ptr %descriptor
  %outcome = getelementptr inbounds {put}, ptr %result, i32 0, i32 1
  %tag = load i32, ptr %outcome
  %old.at = getelementptr inbounds {outcome}, ptr %outcome, i32 0, i32 1
  %old = load ptr, ptr %old.at
  %full.at = getelementptr inbounds {outcome}, ptr %outcome, i32 0, i32 2
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
  %run = load {RUN}, ptr %descriptor
  call void @wf_rehash_all(ptr %result, ptr null, {RUN} %run, i64 %count)
  %new.run = load {RUN}, ptr %result
  store {RUN} %new.run, ptr %descriptor
  %code.at = getelementptr inbounds {grow}, ptr %result, i32 0, i32 1
  %status = load i8, ptr %code.at
  store i8 %status, ptr %code
  %work.at = getelementptr inbounds {grow}, ptr %result, i32 0, i32 2
  %work = load i64, ptr %work.at
  %moved.at = getelementptr inbounds {grow}, ptr %result, i32 0, i32 3
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
  %run = load {RUN}, ptr %descriptor
  call void @wf_rehash_all(ptr %result, ptr %heap, {RUN} %run, i64 %count)
  ret void
}}
"#
    ));
    assert_eq!(input.matches("declare ptr @malloc(i64)").count(), 1);
    assert_eq!(input.matches("declare void @free(ptr)").count(), 1);
    assert_eq!(output.matches("define i32 @wf_fixture_main(").count(), 1);
    fs::write(&args[2], output).expect("write measurement module");
}
