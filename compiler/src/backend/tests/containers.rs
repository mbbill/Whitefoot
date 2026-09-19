//! Allocation identity/refusal and retained aggregate boundaries. Whole-value
//! container behavior lives in the programs corpus; these observers add native
//! ownership evidence an exit code cannot provide.

use super::{compile_link_and_run, emit};

fn observe_allocations(module: &str) -> String {
    for declaration in ["declare ptr @malloc(i64)", "declare void @free(ptr)"] {
        assert_eq!(
            module.lines().filter(|line| *line == declaration).count(),
            1
        );
    }
    for symbol in [
        "wf_observe_allocate",
        "wf_observe_release",
        "wf_fixture_main",
    ] {
        assert!(!module.contains(symbol));
    }
    let result = module
        .replace("@malloc(", "@wf_observe_allocate(")
        .replace("@free(", "@wf_observe_release(")
        .replace("@main(", "@wf_fixture_main(");
    assert_eq!(
        result
            .replace("@wf_observe_allocate(", "@malloc(")
            .replace("@wf_observe_release(", "@free(")
            .replace("@wf_fixture_main(", "@main("),
        module
    );
    result
}

/// Retired subject: the allocation-refusal sweep, which [STOR-8]'s total
/// allocation leaves without a source-visible subject; successor: this test,
/// which keeps the owner-identity half the observer still observes.
#[test]
fn owning_map_collision_and_tombstone_preserve_every_owner_identity() {
    let source = include_bytes!("../../../../tests/programs/containers/owning-map.wf");
    let module = observe_allocations(&emit(source));
    let output = compile_link_and_run(&module, Some(include_str!("owning_map_observer.c")), &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.starts_with("PASS status=0 allocations="), "{report}");
    assert!(report.ends_with(" live=0\n"), "{report}");
}

fn check_migration(source: &[u8], behavior: bool) {
    let mut module = observe_allocations(&emit(source));
    let body = "define i32 @wf__main_body(";
    assert_eq!(module.matches(body).count(), 1);
    module = module.replacen(body, "define i32 @wf_fixture_body(", 1);
    module.push_str("\ndeclare i32 @wf__main_body(i32, ptr)\n");
    let host = format!(
        "{}\n{}",
        if behavior {
            "#define BEHAVIOR_DEMOS"
        } else {
            ""
        },
        include_str!("owning_growth_observer.c")
    );
    let output = compile_link_and_run(&module, Some(&host), &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(
        report.starts_with("owning-growth: 1808 matched executions; "),
        "{report}"
    );
    assert_eq!(report.lines().count(), if behavior { 2 } else { 1 });
    if behavior {
        assert!(report.contains("behavior: 12 stateful, branded-key and hostile-equality executions; every refusal and release checked\n"), "{report}");
    }
}

#[test]
fn direct_migration_budgets_and_refusals_preserve_state_and_allocation_identity() {
    check_migration(
        include_bytes!("../../../../tests/programs/containers/owning-growth.wf"),
        false,
    );
}

#[test]
fn generic_migration_and_behavior_refusals_preserve_state_and_allocation_identity() {
    check_migration(
        include_bytes!("../../../../tests/programs/containers/owning-behavior.wf"),
        true,
    );
}

fn aggregate_transfer_cost(body: &str) -> (u64, u64) {
    let mut copy_bytes = 0;
    let mut vector_bytes = 0;
    for line in body.lines() {
        if line.contains("call ")
            && (line.contains("@llvm.memcpy.") || line.contains("@llvm.memmove."))
        {
            let length = line
                .rsplit_once(", i64 ")
                .expect("constant transfer size")
                .1;
            copy_bytes += length.split(',').next().unwrap().parse::<u64>().unwrap();
        }
        if let Some((_, vector)) = line.split_once(" = load <") {
            let (count, element) = vector.split_once(" x i").expect("integer vector");
            let bits = element.split('>').next().unwrap().parse::<u64>().unwrap();
            vector_bytes += count.parse::<u64>().unwrap() * bits / 8;
        }
    }
    (copy_bytes, vector_bytes)
}

#[test]
fn retained_priority_helpers_do_not_copy_the_inline_run() {
    for source in [
        include_bytes!("../../../../tests/programs/containers/priority.wf").as_slice(),
        include_bytes!("../../../../tests/conformance/cases/run-generic-priority-behavior.wf")
            .as_slice(),
    ] {
        let module = emit(source);
        let mut names = Vec::new();
        let retained = module
            .lines()
            .map(|line| {
                if line.starts_with("define ")
                    && let Some((_, symbol)) = line.split_once('@')
                    && let Some((name, _)) = symbol.split_once('(')
                {
                    let name = name.trim_matches('"');
                    let plain = name.strip_prefix("wf_").unwrap_or(name);
                    if ["push", "pop"].iter().any(|prefix| {
                        plain == *prefix || plain.starts_with(&format!("{prefix}$instance$"))
                    }) {
                        names.push(name.to_owned());
                        return line.strip_suffix(" {").unwrap().to_owned() + " noinline {\n";
                    }
                }
                line.to_owned() + "\n"
            })
            .collect::<String>();
        assert!(matches!(names.len(), 2 | 4), "{names:?}");
        let optimized = super::host_optimized_module(&retained);
        for name in names {
            let header = optimized
                .lines()
                .find(|line| {
                    line.starts_with("define ")
                        && (line.contains(&format!("@{name}("))
                            || line.contains(&format!("@\"{name}\"(")))
                })
                .unwrap_or_else(|| panic!("missing retained {name}"));
            let start = optimized.find(header).unwrap();
            let end = start + optimized[start..].find("\n}").unwrap() + 2;
            let body = &optimized[start..end];
            assert_eq!(aggregate_transfer_cost(body), (0, 0), "{name}: {body}");
            for allocator in ["@malloc(", "@calloc(", "@realloc("] {
                assert!(!body.contains(allocator), "{name}: {body}");
            }
            assert!(
                optimized.lines().any(|line| line.contains("call ")
                    && (line.contains(&format!("@{name}("))
                        || line.contains(&format!("@\"{name}\"(")))),
                "retained {name} is actually called"
            );
        }
    }
}

#[test]
fn aggregate_transfer_inspector_counts_copy_and_vector_traffic() {
    assert_eq!(
        aggregate_transfer_cost(
            "call void @llvm.memcpy.p0.p0.i64(ptr %a, ptr %b, i64 144, i1 false)\n%v = load <4 x i64>, ptr %c\n"
        ),
        (144, 32)
    );
}

#[test]
fn growing_a_run_keeps_the_original_data_on_limit_and_allocation_refusal() {
    let source = include_bytes!("../../../../tests/programs/growable_vec.wf");
    let mut module = observe_allocations(&emit(source));
    assert!(
        module.contains("define i64 @wf_growth_trace(ptr %v0, i8 %v1, i64 %v2, i64 %v3, i64 %v4)")
    );
    module = module.replacen(
        "define i32 @wf__main_body(",
        "define i32 @wf_fixture_body(",
        1,
    );
    module.push_str("\ndeclare i32 @wf__main_body(i32, ptr)\n");
    let output = compile_link_and_run(&module, Some(include_str!("growth_observer.c")), &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        output.stdout,
        b"growth: 108 boundary/refusal traces; all owners returned\n"
    );
}
