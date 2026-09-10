#![forbid(unsafe_code)]

// A bounded counterfactual for priority.wf, never a compiler transformation.
// Retire this adapter when a real placement contract replaces the experiment.
use std::{env, fs};

const RUN: &str = "{ [16 x i64], i64, i64 }";
const FUNCTIONS: [(&str, u64); 3] = [
    ("push", 0xff71_7117_dfd0_824d),
    ("pop", 0xe91e_0d56_cd8e_80c0),
    ("priority_round", 0xfd1c_ffc5_22cc_fe9c),
];

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

fn fingerprint(text: &str) -> u64 {
    // A portable accidental-drift guard, not a cryptographic integrity claim.
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn matches_reviewed_input(module: &str) -> bool {
    [3, 6]
        .into_iter()
        .all(|id| module.contains(&format!("%wf.t{id} = type {{ {RUN}, i64 }}\n")))
        && FUNCTIONS
            .iter()
            .all(|(name, hash)| fingerprint(function(module, name)) == *hash)
}

fn retain(body: &str) -> String {
    assert_eq!(body.matches(" #0 {").count(), 1);
    body.replacen(" #0 {", " noinline #0 {", 1)
}

fn placed(body: &str, name: &str, slots: usize) -> String {
    let mut output = Vec::new();
    let mut changed_slots = 0;
    let mut removed_zeroes = 0;
    let mut changed_inputs = 0;
    for (index, line) in body.lines().enumerate() {
        if index == 0 {
            let scalar = if name == "push" { ", i64 %v2" } else { "" };
            output.push(format!(
                "define internal void @wf_{name}_placed(ptr %wf.result{scalar}) noinline #0 {{"
            ));
        } else if line.contains(", ptr %wf.frame, i32 0, i32 ") {
            let (value, _) = line.split_once(" = getelementptr inbounds ").unwrap();
            output.push(format!(
                "{value} = getelementptr inbounds i8, ptr %wf.result, i64 0"
            ));
            changed_slots += 1;
        } else if line == "  store %wf.t6 zeroinitializer, ptr %wf.slot.2"
            || line == "  store %wf.t3 zeroinitializer, ptr %wf.result"
        {
            // These tuple clears precede complete writes of both fields. They
            // are already dead in baseline optimized IR. Keeping them after
            // co-location would incorrectly erase the live input field.
            removed_zeroes += 1;
        } else if line.contains("ptr %wf.arg.v0") {
            assert!(line.contains("@llvm.memmove."));
            output.push(line.replace("ptr %wf.arg.v0", "ptr %wf.result"));
            changed_inputs += 1;
        } else {
            output.push(line.to_owned());
        }
    }
    assert_eq!(changed_slots, slots);
    assert_eq!(removed_zeroes, if name == "pop" { 3 } else { 0 });
    assert_eq!(changed_inputs, 1);
    output.join("\n")
}

fn adapt(input: &str) -> (String, String) {
    assert!(
        matches_reviewed_input(input),
        "review the changed witness IR"
    );
    // The guard must detect both changed storage extent and changed algorithm.
    assert!(!matches_reviewed_input(&input.replacen(
        &format!("%wf.t6 = type {{ {RUN}, i64 }}"),
        "%wf.t6 = type { i64 }",
        1,
    )));
    assert!(!matches_reviewed_input(&input.replacen(
        "%t15 = add i64 %t2, 1",
        "%t15 = add i64 %t2, 2",
        1,
    )));

    let push = function(input, "push");
    let pop = function(input, "pop");
    let round = function(input, "priority_round");
    let baseline = input
        .replacen(push, &retain(push), 1)
        .replacen(pop, &retain(pop), 1);
    let old_calls = [
        "call void @wf_push(ptr %t1, ptr %t2, i64 %v24)",
        "call void @wf_pop(ptr %wf.slot.0, ptr %t3)",
    ];
    let new_calls = [
        "call void @wf_push_placed(ptr %t1, i64 %v24)",
        "call void @wf_pop_placed(ptr %wf.slot.0)",
    ];
    let mut placed_round = round.to_owned();
    for (old, new) in old_calls.into_iter().zip(new_calls) {
        assert_eq!(placed_round.matches(old).count(), 1);
        placed_round = placed_round.replacen(old, new, 1);
    }
    let inserted = format!(
        "{}\n\n{}\n\n{}",
        placed(push, "push", 3),
        placed(pop, "pop", 7),
        placed_round
    );
    let candidate = baseline.replacen(round, &inserted, 1);
    // The original helpers, fixture main, constructors and every other byte
    // outside this round and the two cloned helpers remain identical.
    assert_eq!(candidate.replacen(&inserted, round, 1), baseline);
    (baseline, candidate)
}

fn inspect(module: &str, name: &str, placement: bool) {
    let body = function(module, name);
    let header = body.lines().next().unwrap();
    let group = header.rsplit_once(" #").expect("function attributes").1;
    let number = group.split_whitespace().next().unwrap();
    let attribute_prefix = format!("attributes #{number} = {{ ");
    let attributes = module
        .lines()
        .find(|line| line.starts_with(&attribute_prefix))
        .expect("defined function attributes");
    assert!(attributes.split_whitespace().any(|word| word == "noinline"));
    assert!(!body.contains("@llvm.memset.") && !body.contains("zeroinitializer"));
    let mut copies = 0;
    let mut copy_bytes = 0_u64;
    let mut vector_bytes = 0_u64;
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
    if placement {
        assert_eq!(copies, 0, "placed helper still transfers a whole value");
        assert_eq!(vector_bytes, 0, "inspect vector movement in placed helper");
        assert!(!body.contains("alloca "), "placed helper has local storage");
        let call = format!("@wf_{name}(");
        assert!(
            module
                .lines()
                .any(|line| line.contains("call ") && line.contains(&call))
        );
    }
    println!("{name},{copies},{copy_bytes},{vector_bytes}");
}

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "priority-placement INPUT BASELINE PLACED | --inspect BASELINE_OPT PLACED_OPT"
    );
    if args[1] == "--inspect" {
        let baseline = fs::read_to_string(&args[2]).expect("read optimized baseline");
        let candidate = fs::read_to_string(&args[3]).expect("read optimized placement");
        println!("helper,copy_intrinsics,copy_intrinsic_bytes,vector_load_bytes");
        inspect(&baseline, "push", false);
        inspect(&baseline, "pop", false);
        inspect(&candidate, "push_placed", true);
        inspect(&candidate, "pop_placed", true);
    } else {
        let input = fs::read_to_string(&args[1]).expect("read compiler module");
        let (baseline, candidate) = adapt(&input);
        fs::write(&args[2], baseline).expect("write retained baseline");
        fs::write(&args[3], candidate).expect("write experimental placement");
    }
}
