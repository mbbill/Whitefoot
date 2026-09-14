use super::*;

/// Use the benchmark's actual binding path, including pool-off world selection.
fn bind_compute_host_adapter(module: &str, adapter: &str) -> String {
    let directory = test_directory();
    std::fs::create_dir_all(&directory).expect("create adapter directory");
    let input = directory.join("module.ll");
    let host = directory.join("adapter.ll");
    std::fs::write(&input, module).expect("write adapter module");
    std::fs::write(&host, adapter).expect("write host adapter");
    let output = Command::new("awk")
        .arg("-f")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../research/experiments/compute-bench/host-adapter.awk"
        ))
        .arg(input)
        .arg(host)
        .output()
        .expect("bind compute host adapter");
    assert!(output.status.success(), "{output:?}");
    std::fs::remove_dir_all(directory).expect("remove adapter directory");
    String::from_utf8(output.stdout).expect("adapter LLVM text")
}

fn run_compute_oracle(executable: &Path, name: &str, parallel: bool) {
    for workers in [1, 2, 4] {
        // As in the counted-program tests, observing a steal is existential
        // across schedules. Every attempt checks the entire result matrix;
        // only its distinct no-steal outcome may be resampled, never a result
        // or inactive-pool failure.
        let runs = if parallel && workers > 1 {
            super::parallel::GRANT_OBSERVATION_RUNS
        } else {
            1
        };
        for run in 0..runs {
            let output = Command::new(executable)
                .env("WF_WORKERS", workers.to_string())
                .env_remove("WF_SPLIT_WORK")
                .output()
                .expect("run independent compute oracle");
            if output.status.code() == Some(2)
                && output.stderr == format!("{name}: oracle observed no steals\n").as_bytes()
                && run + 1 < runs
            {
                continue;
            }
            assert!(
                output.status.success(),
                "{name} workers={workers} run={run}: {output:?}"
            );
            assert!(
                String::from_utf8_lossy(&output.stdout).contains(&format!("{name} oracle PASS:"))
            );
            break;
        }
    }
}

#[test]
fn runtime_work_estimates_are_total_for_empty_and_inverted_ranges() {
    let source = br#"fn count_work(lower: own u64, upper: own u64) -> result: own u64 pure {
  let total = 0_u64;
  for (i in lower..upper) {
    set total = total +wrap i;
  }
  return total;
}

fn write_work(count: own u64, lower: own u64, upper: own u64) -> result: own buffer<u64> pure contract {
  requires count <= 2_u64;
} {
  let output = buffer_new(count, 0_u64);
  for (i in 0_u64..count) {
    set output[i] = count_work(lower: lower, upper: upper);
  }
  return move output;
}

fn release_work(output: own buffer<u64>) -> result: own u64 pure {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  let output = write_work(count: 2_u64, lower: 0_u64, upper: 17_u64);
  return exit_status(code: 0_u8);
}
"#;
    let mut llvm = emit_with_overlap(source)
        .replace("@main(", "@wf_total_work_main(")
        .replace("@wf__main_body(", "@wf_total_work_body(")
        .replace(
            "call i64 @wf__par_split_budget(",
            "call i64 @wf_work_budget(",
        );
    llvm.push_str(
        r#"
declare i64 @wf_work_budget(i64, i64)
define void @wf_probe_work(i64 %count, i64 %lower, i64 %upper, ptr %out, ptr %length) {
  %r = call { ptr, i64 } @wf_write_work(i64 %count, i64 %lower, i64 %upper)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %length
  ret void
}
define void @wf_probe_release(ptr %data, i64 %count) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %r = call i64 @wf_release_work({ ptr, i64 } %b)
  ret void
}
"#,
    );
    let host = r#"#include <stdint.h>
#include <stdio.h>
extern int wf__floor_run(int, char **);
extern void wf_probe_work(uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_probe_release(uint64_t *, uint64_t);
static uint64_t price;
uint64_t wf_work_budget(uint64_t span, uint64_t weight) {
    (void)span; price = weight; return 0;
}
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    uint64_t *output, length;
    wf_probe_work(2, 0, 17, &output, &length);
    if (length != 2 || output[0] != 136 || output[1] != 136) return 1;
    wf_probe_release(output, length);
    wf_probe_work(2, 17, 1, &output, &length);
    if (length != 2 || output[0] || output[1]) return 2;
    wf_probe_release(output, length);
    wf_probe_work(0, 17, 1, &output, &length);
    if (length) return 3;
    uint64_t inverted = price;
    wf_probe_release(output, length);
    wf_probe_work(0, 0, 0, &output, &length);
    if (length || inverted != price) return 3;
    wf_probe_release(output, length);
    /* No inner source iteration runs. Pricing must nevertheless remain total
       when its independent work-count arithmetic exceeds the machine word. */
    wf_probe_work(0, 0, UINT64_MAX, &output, &length);
    if (length || price <= UINT64_MAX / 1024) return 4;
    wf_probe_release(output, length);
    printf("saturated work price: %llu\n", (unsigned long long)price);
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;
    let directory = test_directory();
    let executable = build_linked_executable(&llvm, Some(host), &[], &directory);
    let output = Command::new(executable)
        .output()
        .expect("run total scheduling estimate probe");
    assert!(output.status.success(), "{output:?}");
    std::fs::remove_dir_all(directory).expect("remove total scheduling estimate probe");
}

#[test]
fn runtime_work_prices_post_loop_arithmetic_once_per_enclosing_iteration() {
    let mut source = String::new();
    for (extent, bound) in [("known", "upper"), ("fallback", "i")] {
        for position in ["before", "after"] {
            let bias = "    let bias = upper *wrap 3_u64;\n";
            let (before, after) = if position == "before" {
                (bias, "")
            } else {
                ("", bias)
            };
            source.push_str(&format!(
                r#"fn count_{extent}_{position}(upper: own u64, repeats: own u64) -> result: own u64 pure {{
  let total = 0_u64;
  for (i in 0_u64..repeats) {{
{before}    for (j in 0_u64..{bound}) {{
      let multiplied = total *wrap 3_u64;
      set total = multiplied +wrap j;
    }}
{after}    set total = total +wrap bias;
  }}
  return total;
}}

fn write_{extent}_{position}(upper: own u64, repeats: own u64) -> result: own buffer<u64> pure {{
  let output = buffer_new(2_u64, 0_u64);
  for (i in 0_u64..2_u64) {{
    set output[i] = count_{extent}_{position}(upper: upper, repeats: repeats);
  }}
  return move output;
}}

"#
            ));
        }
    }
    source.push_str(
        r#"fn release_work(output: own buffer<u64>) -> result: own u64 pure {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
    let mut llvm = emit_with_overlap(source.as_bytes())
        .replace("@main(", "@wf_continuation_main(")
        .replace("@wf__main_body(", "@wf_continuation_body(")
        .replace(
            "call i64 @wf__par_split_budget(",
            "call i64 @wf_work_budget(",
        );
    llvm.push_str("\ndeclare i64 @wf_work_budget(i64, i64)\n");
    for extent in ["known", "fallback"] {
        for position in ["before", "after"] {
            llvm.push_str(&format!(
                r#"define i64 @wf_probe_{extent}_{position}(i64 %upper, i64 %repeats) {{
  %r = call {{ ptr, i64 }} @wf_write_{extent}_{position}(i64 %upper, i64 %repeats)
  %p = extractvalue {{ ptr, i64 }} %r, 0
  %first = load i64, ptr %p
  %second_ptr = getelementptr i64, ptr %p, i64 1
  %second = load i64, ptr %second_ptr
  %same = icmp eq i64 %first, %second
  %answer = select i1 %same, i64 %first, i64 -1
  %released = call i64 @wf_release_work({{ ptr, i64 }} %r)
  ret i64 %answer
}}
"#
            ));
        }
    }
    let host = r#"#include <stdint.h>
#include <stdio.h>
extern int wf__floor_run(int, char **);
extern uint64_t wf_probe_known_before(uint64_t, uint64_t), wf_probe_known_after(uint64_t, uint64_t);
extern uint64_t wf_probe_fallback_before(uint64_t, uint64_t), wf_probe_fallback_after(uint64_t, uint64_t);
static uint64_t price;
uint64_t wf_work_budget(uint64_t span, uint64_t weight) {
    (void)span; price = weight; return 0;
}
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    const uint64_t extents[] = {0, 1, 16, 17}, repetitions[] = {1, 2, 4};
    uint64_t outer_prices[3];
    for (unsigned r = 0; r < 3; ++r) {
      uint64_t repeats = repetitions[r];
      (void)wf_probe_known_before(16, repeats);
      uint64_t fallback_price = price;
      for (unsigned i = 0; i < 4; ++i) {
        uint64_t upper = extents[i];
        uint64_t expected = 0, fallback_expected = 0;
        for (uint64_t pass = 0; pass < repeats; ++pass) {
            for (uint64_t j = 0; j < upper; ++j) expected = expected * 3 + j;
            expected += 3 * upper;
            for (uint64_t j = 0; j < pass; ++j) fallback_expected = fallback_expected * 3 + j;
            fallback_expected += 3 * upper;
        }
        if (wf_probe_known_before(upper, repeats) != expected) return 1;
        uint64_t before = price;
        if (!i) outer_prices[r] = before;
        if (wf_probe_known_after(upper, repeats) != expected) return 2;
        printf("known %llu: before=%llu after=%llu\n",
               (unsigned long long)upper, (unsigned long long)before, (unsigned long long)price);
        if (!before || before != price) return 3;
        if (wf_probe_fallback_before(upper, repeats) != fallback_expected) return 4;
        before = price;
        if (wf_probe_fallback_after(upper, repeats) != fallback_expected) return 5;
        printf("fallback %llu: before=%llu after=%llu\n",
               (unsigned long long)upper, (unsigned long long)before, (unsigned long long)price);
        if (!before || before != price || price != fallback_price) return 6;
      }
    }
    /* Moving the arithmetic must not erase the enclosing trip multiplier.
       The unknown inner extent must retain the calibrated static factor 16. */
    if (outer_prices[1] <= outer_prices[0] ||
        outer_prices[2] - outer_prices[1] != 2 * (outer_prices[1] - outer_prices[0])) return 7;
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;
    let directory = test_directory();
    let executable = build_linked_executable(&llvm, Some(host), &[], &directory);
    let output = Command::new(executable)
        .env("WF_WORKERS", "2")
        .output()
        .expect("run continuation work price probe");
    assert!(output.status.success(), "{output:?}");
    std::fs::remove_dir_all(directory).expect("remove continuation work price probe");
}

#[test]
fn runtime_block_sizes_change_budget_prices_without_changing_scan_results() {
    let source =
        include_bytes!("../../../../research/experiments/compute-bench/programs/prefix.wf");
    let module = emit_with_overlap(source);
    let adapter = include_str!("../../../../research/experiments/compute-bench/prefix_host.ll");
    let mut llvm = bind_compute_host_adapter(&module, adapter)
        .replace("@main(", "@wf_extent_test_main(")
        .replace("@wf__main_body(", "@wf_extent_test_body(")
        .replace(
            "call i64 @wf__par_split_budget(",
            "call i64 @wf_work_budget(",
        );
    llvm.push_str("\ndeclare i64 @wf_work_budget(i64, i64)\n");
    let host = r#"#include <stdint.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
extern int wf__floor_run(int, char **);
extern int wf__par_pool_active(void);
extern uint64_t wf__par_split_budget(uint64_t, uint64_t);
extern void wf_bench_prefix(const uint64_t *, uint64_t, uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_bench_prefix_release(uint64_t *, uint64_t);
static _Atomic uint64_t observed;
uint64_t wf_work_budget(uint64_t span, uint64_t weight) {
    if (span == 128) {
        uint64_t previous = atomic_load(&observed);
        while (previous < weight && !atomic_compare_exchange_weak(&observed, &previous, weight)) {}
    }
    return wf__par_split_budget(span, weight);
}
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    const uint64_t widths[] = {17, 1024, 4096};
    uint64_t previous_price = 0;
    for (unsigned shape = 0; shape < 3; ++shape) {
        uint64_t width = widths[shape], count = 128 * width;
        uint64_t *input = calloc((size_t)count, sizeof(*input));
        if (!input) return 1;
        for (uint64_t i = 0; i < count; ++i) input[i] = i + 1;
        uint64_t *output = NULL, length = 0;
        atomic_store(&observed, 0);
        wf_bench_prefix(input, count, width, 1, &output, &length);
        if (!output || length != count) return 2;
        uint64_t expected = 0;
        for (uint64_t i = 0; i < count; ++i) {
            if (output[i] != expected || input[i] != i + 1) return 3;
            expected += i + 1;
        }
        uint64_t price = atomic_load(&observed);
        printf("%llu %llu\n", (unsigned long long)width, (unsigned long long)price);
        if (wf__par_pool_active()) {
            if (!price || (shape == 1 && price <= previous_price * 10) ||
                (shape == 2 && price <= previous_price * 3)) return 4;
        } else if (price) return 5;
        previous_price = price;
        wf_bench_prefix_release(output, length);
        free(input);
    }
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;
    let directory = test_directory();
    let executable = build_linked_executable(&llvm, Some(host), &[], &directory);
    for workers in [1, 2, 4] {
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers.to_string())
            .env_remove("WF_SPLIT_WORK")
            .output()
            .expect("run runtime extent price probe");
        assert!(output.status.success(), "workers={workers}: {output:?}");
    }
    std::fs::remove_dir_all(directory).expect("remove runtime extent probe");
}

#[test]
fn compute_oracle_sampling_rejects_wrong_values_and_missing_observations() {
    let source = include_str!("../../../../research/experiments/compute-bench/programs/prefix.wf");
    let adapter = include_str!("../../../../research/experiments/compute-bench/prefix_host.ll");
    let oracle = format!(
        "#define WFB_BLOCKED_ORACLE\n#define WFB_PREFIX\n{}",
        include_str!("../../../../research/experiments/compute-bench/blocked_bench.c")
    );
    for corrupt in [true, false] {
        let program = if corrupt {
            let changed = source.replacen("total +wrap input[i]", "total -wrap input[i]", 1);
            assert_ne!(
                changed, source,
                "the block-sum mutant must change the algorithm"
            );
            changed
        } else {
            source.to_owned()
        };
        let module = emit_with_overlap(program.as_bytes());
        let llvm = bind_compute_host_adapter(&module, adapter)
            .replace("@main(", "@wf_oracle_negative_main(")
            .replace("@wf__main_body(", "@wf_oracle_negative_body(");
        let directory = test_directory();
        let mut defines = vec!["WFB_ORACLE_PARALLEL=1".to_owned()];
        if !corrupt {
            // The real algorithm still runs, but no attempt can observe a
            // steal through the intentionally disabled counter.
            defines.push("WF_SCHED_STATS=0".to_owned());
        }
        let executable = build_linked_executable(&llvm, Some(&oracle), &defines, &directory);
        let failure = std::panic::catch_unwind(|| run_compute_oracle(&executable, "prefix", true))
            .expect_err("a wrong result or absent observation must fail the oracle test");
        let message = failure
            .downcast_ref::<String>()
            .expect("oracle assertion message");
        if corrupt {
            assert!(message.contains("workers=1 run=0"), "{message}");
            assert!(message.contains("wrong result"), "{message}");
        } else {
            let final_run = super::parallel::GRANT_OBSERVATION_RUNS - 1;
            assert!(
                message.contains(&format!("workers=2 run={final_run}")),
                "{message}"
            );
            assert!(message.contains("oracle observed no steals"), "{message}");
        }
        std::fs::remove_dir_all(directory).expect("remove oracle negative-test files");
    }
}

#[test]
fn borrowed_storage_subranges_write_the_original_array_and_run() {
    let source = r#"struct Packet {
  before: u64;
  values: STORAGE;
  after: u64;
}

fn fill(values: &uniq STORAGE) -> result: own unit writes(values) contract {
  requires len_of(deref(values)) == 8_u64;
  requires head_of(deref(values)) == 0_u64;
  ensures len_of(deref(values)) == len_of(deref(entry(values)));
  ensures head_of(deref(values)) == head_of(deref(entry(values)));
} {
  region {
    let left = mut_slice_of(&uniq deref(values), 0_u64, 4_u64);
    let right = mut_slice_of(&uniq deref(values), 4_u64, 8_u64);
    set left[0_u64] = 11_u64;
    set left[3_u64] = 13_u64;
    set right[0_u64] = 17_u64;
    set right[3_u64] = 19_u64;
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let initial = array_new::<u64, 8>(7_u64);
  let values = INITIAL;
  let packet = Packet(before: 53_u64, values: move values, after: 59_u64);
  region {
    let done = fill(values: &uniq packet.values);
  }
  if packet.before != 53_u64 {
    return exit_status(code: 1_u8);
  }
  if packet.after != 59_u64 {
    return exit_status(code: 2_u8);
  }
  if packet.values[0_u64] != 11_u64 {
    return exit_status(code: 3_u8);
  }
  if packet.values[3_u64] != 13_u64 {
    return exit_status(code: 4_u8);
  }
  if packet.values[4_u64] != 17_u64 {
    return exit_status(code: 5_u8);
  }
  if packet.values[7_u64] != 19_u64 {
    return exit_status(code: 6_u8);
  }
  if packet.values[1_u64] != 7_u64 {
    return exit_status(code: 7_u8);
  }
  if packet.values[6_u64] != 7_u64 {
    return exit_status(code: 8_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for (storage, initial) in [
        ("array<u64, 8>", "move initial"),
        (
            "FixedVector<u64, 8>",
            "fixed_from_array(values: move initial)",
        ),
    ] {
        let source = source
            .replace("STORAGE", storage)
            .replace("INITIAL", initial);
        for overlap in [OverlapLowering::Off, OverlapLowering::On] {
            let module = emit_lowered(source.as_bytes(), overlap);
            for module in [&module, &super::owned_places::retain_calls(&module)] {
                let output = compile_and_run(module);
                assert_eq!(
                    output.status.code(),
                    Some(0),
                    "{storage}: {overlap:?}: {output:?}"
                );
                assert!(output.stdout.is_empty(), "{output:?}");
                assert!(output.stderr.is_empty(), "{output:?}");
            }
        }
    }
}

#[test]
fn blocked_compute_matches_independent_oracles_at_runtime_dimensions() {
    let kernels: [(&str, &[u8], &str, &str); 2] = [
        (
            "prefix",
            include_bytes!("../../../../research/experiments/compute-bench/programs/prefix.wf"),
            include_str!("../../../../research/experiments/compute-bench/prefix_host.ll"),
            "WFB_PREFIX",
        ),
        (
            "histogram",
            include_bytes!("../../../../research/experiments/compute-bench/programs/histogram.wf"),
            include_str!("../../../../research/experiments/compute-bench/histogram_host.ll"),
            "WFB_HISTOGRAM",
        ),
    ];
    for (name, source, adapter, selection) in kernels {
        let oracle = format!(
            "#define WFB_BLOCKED_ORACLE\n#define {selection}\n{}",
            include_str!("../../../../research/experiments/compute-bench/blocked_bench.c")
        );
        for emitted in [compile(source), emit_with_overlap(source)] {
            let llvm = bind_compute_host_adapter(&emitted, adapter)
                .replace("@main(", "@wf_blocked_smoke_main(")
                .replace("@wf__main_body(", "@wf_blocked_smoke_body(");
            let directory = test_directory();
            let defines = if emitted.contains("call void @wf__par_publish(") {
                vec!["WFB_ORACLE_PARALLEL=1".to_owned()]
            } else {
                Vec::new()
            };
            let executable = build_linked_executable(&llvm, Some(&oracle), &defines, &directory);
            run_compute_oracle(&executable, name, !defines.is_empty());
            std::fs::remove_dir_all(directory).expect("remove blocked-compute test files");
        }
    }
}

#[test]
fn runtime_stencil_ranges_reach_their_backing_buffers() {
    let source =
        include_bytes!("../../../../research/experiments/compute-bench/programs/stencil.wf");
    let llvm = compile(source);
    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
    let parallel = emit_with_overlap(source);
    assert!(parallel.contains("call void @wf__par_publish("));
    let output = compile_and_run(&parallel);
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn stencil_matches_an_independent_dimension_and_step_matrix() {
    let source =
        include_bytes!("../../../../research/experiments/compute-bench/programs/stencil.wf");
    let adapter = include_str!("../../../../research/experiments/compute-bench/stencil_host.ll");
    let oracle = format!(
        "#define WFB_STENCIL_ORACLE\n{}",
        include_str!("../../../../research/experiments/compute-bench/stencil_bench.c")
    );
    for emitted in [compile(source), emit_with_overlap(source)] {
        let llvm = bind_compute_host_adapter(&emitted, adapter)
            .replace("@main(", "@wf_stencil_smoke_main(")
            .replace("@wf__main_body(", "@wf_stencil_smoke_body(");
        let directory = test_directory();
        let defines = if emitted.contains("call void @wf__par_publish(") {
            vec!["WFB_ORACLE_PARALLEL=1".to_owned()]
        } else {
            Vec::new()
        };
        let executable = build_linked_executable(&llvm, Some(&oracle), &defines, &directory);
        run_compute_oracle(&executable, "stencil", !defines.is_empty());
        std::fs::remove_dir_all(directory).expect("remove native stencil test files");
    }
}

#[test]
fn recursive_child_ranges_restore_parent_access() {
    let source =
        include_bytes!("../../../../research/experiments/compute-bench/programs/range_split.wf");
    let llvm = compile(source);
    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
    let parallel = emit_with_overlap(source);
    assert!(parallel.contains("call void @wf__par_publish("));
    let output = compile_and_run(&parallel);
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn range_endpoints_are_captured_and_empty_ranges_are_admitted() {
    let source = br#"const values: FixedVector<u64, 8> =[1_u64, 2_u64, 3_u64, 4_u64, 5_u64, 6_u64, 7_u64, 8_u64];

fn window_sum(start: own u64, end: own u64) -> result: own u64 pure contract {
  requires start <= end;
  requires end <= 8_u64;
} {
  region {
    let view = slice_of(&values, start, end);
    set start = end;
    let count = len_of(view);
    let total = 0_u64;
    for (i in 0_u64..count) {
      let value = view[i];
      set total = total +wrap value;
    }
    return total;
  }
}

fn main() -> status: own ExitStatus pure {
  let middle = window_sum(start: 2_u64, end: 5_u64);
  let empty = window_sum(start: 8_u64, end: 8_u64);
  if middle != 12_u64 {
    return exit_status(code: 1_u8);
  }
  if empty != 0_u64 {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn borrowed_array_views_preserve_delegation_and_original_storage_across_calls() {
    let source = br#"struct Packet {
  before: u64;
  bytes: array<u64, 3>;
  after: u64;
}

const fixed: array<u64, 3> =[2_u64, 3_u64, 5_u64];

fn relay['r](values: own Slice<'r, u64>) -> result: own Slice<'r, u64> pure contract {
  requires len_of(values) == 3_u64;
  ensures len_of(result) == 3_u64;
} {
  return values;
}

fn read(values: own Slice<u64>, index: own u64) -> result: own u64 reads(values) contract {
  requires index < len_of(values);
} {
  return values[index];
}

fn write(values: &uniq MutSlice<u64>, index: own u64, value: own u64) -> result: own unit writes(values) contract {
  requires index < len_of(deref(values));
} {
  set deref(values)[index] = value;
  return unit;
}

fn delegate(view: own MutSlice<u64>) -> result: own unit writes(view) contract {
  requires len_of(view) == 3_u64;
} {
  let writer = move view;
  region {
    let done = write(values: &uniq writer, index: 1_u64, value: 109_u64);
  }
  return unit;
}

fn shared_read(values: &array<u64, 3>) -> result: own u64 reads(values) {
  region {
    let view = slice_of(&deref(values));
    let copied = view;
    let forwarded = relay(values: copied);
    let first = read(values: view, index: 0_u64);
    let second = read(values: copied, index: 1_u64);
    let third = read(values: forwarded, index: 2_u64);
    let prefix = first +wrap second;
    return prefix +wrap third;
  }
}

fn revise(values: &uniq array<u64, 3>) -> result: own u64 reads(values), writes(values) {
  region {
    let view = slice_of(&deref(values));
    let copied = view;
    let forwarded = relay(values: copied);
    let first = read(values: view, index: 0_u64);
    let second = read(values: copied, index: 1_u64);
    let third = read(values: forwarded, index: 2_u64);
    let prefix = first +wrap second;
    let previous = prefix +wrap third;
    set deref(values)[0_u64] = 101_u64;
    let writable = mut_slice_of(&uniq deref(values));
    region {
      let done = write(values: &uniq writable, index: 1_u64, value: 103_u64);
    }
    return previous;
  }
}

fn main() -> status: own ExitStatus pure {
  let values = array_new::<u64, 3>(7_u64);
  let before0 = values[0_u64];
  let before1 = values[1_u64];
  region {
    let initial = shared_read(values: &values);
    if initial != 21_u64 {
      return exit_status(code: 1_u8);
    }
  }
  region {
    let previous = revise(values: &uniq values);
    if previous != 21_u64 {
      return exit_status(code: 2_u8);
    }
  }
  if values[0_u64] != 101_u64 {
    return exit_status(code: 3_u8);
  }
  if values[1_u64] != 103_u64 {
    return exit_status(code: 4_u8);
  }
  if values[2_u64] != 7_u64 {
    return exit_status(code: 5_u8);
  }
  if before0 != 7_u64 {
    return exit_status(code: 6_u8);
  }
  if before1 != 7_u64 {
    return exit_status(code: 7_u8);
  }
  region {
    let current = shared_read(values: &values);
    if current != 211_u64 {
      return exit_status(code: 8_u8);
    }
  }
  let bytes = array_new::<u64, 3>(13_u64);
  let packet = Packet(before: 53_u64, bytes: move bytes, after: 59_u64);
  region {
    let holder = &uniq packet.bytes;
    region {
      let writable = mut_slice_of(&uniq deref(holder));
      set writable[2_u64] = 107_u64;
    }
  }
  if packet.before != 53_u64 {
    return exit_status(code: 9_u8);
  }
  if packet.after != 59_u64 {
    return exit_status(code: 10_u8);
  }
  if packet.bytes[0_u64] != 13_u64 {
    return exit_status(code: 11_u8);
  }
  if packet.bytes[1_u64] != 13_u64 {
    return exit_status(code: 12_u8);
  }
  if packet.bytes[2_u64] != 107_u64 {
    return exit_status(code: 13_u8);
  }
  region {
    let holder = &packet.bytes;
    region {
      let view = slice_of(&deref(holder));
      let selected = read(values: view, index: 2_u64);
      if selected != 107_u64 {
        return exit_status(code: 14_u8);
      }
    }
  }
  region {
    let observed = shared_read(values: &fixed);
    if observed != 10_u64 {
      return exit_status(code: 15_u8);
    }
  }
  region {
    let holder = &fixed;
    region {
      let view = slice_of(&deref(holder));
      let selected = read(values: view, index: 1_u64);
      if selected != 3_u64 {
        return exit_status(code: 16_u8);
      }
    }
  }
  let delegated = array_new::<u64, 3>(17_u64);
  region {
    let view = mut_slice_of(&uniq delegated);
    let done = delegate(view: move view);
  }
  if delegated[0_u64] != 17_u64 {
    return exit_status(code: 17_u8);
  }
  if delegated[1_u64] != 109_u64 {
    return exit_status(code: 18_u8);
  }
  if delegated[2_u64] != 17_u64 {
    return exit_status(code: 19_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [OverlapLowering::Off, OverlapLowering::On] {
        let module = emit_lowered(source, overlap);
        for module in [&module, &super::owned_places::retain_calls(&module)] {
            let output = compile_and_run(module);
            assert_eq!(output.status.code(), Some(0), "{overlap:?}: {output:?}");
            assert!(output.stdout.is_empty(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn exclusive_array_views_write_original_local_and_field_storage() {
    let source = br#"struct Packet {
  before: u64;
  bytes: array<u64, 4>;
  after: u64;
}

fn overwrite(view: &uniq MutSlice<u64>, index: own u64, value: own u64) -> result: own unit writes(view) contract {
  requires index < len_of(deref(view));
} {
  set deref(view)[index] = value;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let values = array_new::<u64, 4>(7_u64);
  let before0 = values[0_u64];
  let before2 = values[2_u64];
  region {
    let view = mut_slice_of(&uniq values);
    set view[0_u64] = 19_u64;
    region {
      let done = overwrite(view: &uniq view, index: 2_u64, value: 31_u64);
    }
  }
  if values[0_u64] != 19_u64 {
    return exit_status(code: 1_u8);
  }
  if values[1_u64] != 7_u64 {
    return exit_status(code: 2_u8);
  }
  if values[2_u64] != 31_u64 {
    return exit_status(code: 3_u8);
  }
  if values[3_u64] != 7_u64 {
    return exit_status(code: 4_u8);
  }
  if before0 != 7_u64 {
    return exit_status(code: 5_u8);
  }
  if before2 != 7_u64 {
    return exit_status(code: 6_u8);
  }
  let bytes = array_new::<u64, 4>(13_u64);
  let packet = Packet(before: 53_u64, bytes: move bytes, after: 59_u64);
  region {
    let view = mut_slice_of(&uniq packet.bytes);
    region {
      let done = overwrite(view: &uniq view, index: 1_u64, value: 41_u64);
    }
  }
  if packet.before != 53_u64 {
    return exit_status(code: 7_u8);
  }
  if packet.after != 59_u64 {
    return exit_status(code: 8_u8);
  }
  if packet.bytes[0_u64] != 13_u64 {
    return exit_status(code: 9_u8);
  }
  if packet.bytes[1_u64] != 41_u64 {
    return exit_status(code: 10_u8);
  }
  if packet.bytes[2_u64] != 13_u64 {
    return exit_status(code: 11_u8);
  }
  if packet.bytes[3_u64] != 13_u64 {
    return exit_status(code: 12_u8);
  }
  region {
    let shared = slice_of(&packet.bytes);
    if shared[1_u64] != 41_u64 {
      return exit_status(code: 13_u8);
    }
  }
  let empty = array_new::<u64, 0>(0_u64);
  region {
    let view = mut_slice_of(&uniq empty);
    let length = len_of(view);
    if length != 0_u64 {
      return exit_status(code: 14_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [OverlapLowering::Off, OverlapLowering::On] {
        let module = emit_lowered(source, overlap);
        for module in [&module, &super::owned_places::retain_calls(&module)] {
            let output = compile_and_run(module);
            assert_eq!(output.status.code(), Some(0), "{overlap:?}: {output:?}");
            assert!(output.stdout.is_empty(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn formal_view_child_last_use_restores_parent_writes_across_retained_calls() {
    let source = r#"fn child['r](view: &uniq MutSlice<'r, u8>) -> result: own Slice<'r, u8> pure contract {
  requires len_of(deref(view)) == 1_u64;
  ensures len_of(result) == 1_u64;
} {
  let result = slice_of(&'r deref(view));
  return result;
}

fn reuse(view: &uniq MutSlice<u8>) -> result: own u8 reads(view), writes(view) contract {
  requires len_of(deref(view)) == 1_u64;
} {
  region {
    let shared = FORM;
    let copied = shared;
    let previous = shared[0_u64];
    let confirmed = copied[0_u64];
    set deref(view)[0_u64] = 9_u8;
    return previous;
  }
}

fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 1>();
  region {
    place_back(vector: &uniq empty, value: 3_u8);
  }
  let bytes = move empty;
  region {
    let writer = mut_slice_of(&uniq bytes);
    region {
      let previous = reuse(view: &uniq writer);
      if previous != 3_u8 {
        return exit_status(code: 1_u8);
      }
      if writer[0_u64] != 9_u8 {
        return exit_status(code: 2_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for formation in ["slice_of(&deref(view))", "child(view: &uniq deref(view))"] {
        let source = source.replace("FORM", formation);
        for overlap in [OverlapLowering::Off, OverlapLowering::On] {
            let module = emit_lowered(source.as_bytes(), overlap);
            for module in [&module, &super::owned_places::retain_calls(&module)] {
                let output = compile_and_run(module);
                assert_eq!(
                    output.status.code(),
                    Some(0),
                    "{formation}: {overlap:?}: {output:?}"
                );
                assert!(output.stdout.is_empty(), "{output:?}");
                assert!(output.stderr.is_empty(), "{output:?}");
            }
        }
    }
}

#[test]
fn const_local_and_store_run_slices_share_one_read_only_descriptor_path() {
    // The three view origins are now the three run storages [STOR-1]: the
    // const run's read-only static rodata, a frame-resident `FixedVector`,
    // and one run taken from the general store. One `Slice` consumer reads
    // all three, which is the property the retired array/buffer pair pinned.
    let source = br#"const bytes: FixedVector<u8, 4> =[1_u8, 2_u8, 3_u8, 4_u8];

fn sum(values: own Slice<u8>) -> result: own u64 reads(values) {
  let total = 0_u64;
  let length = len_of(values);
  for (offset in 0_u64..length) {
    let byte = values[offset];
    let word = cvt::<u8, u64>(byte);
    set total = total +wrap word;
  }
  return total;
}

fn main['heap](inputs: own Inputs, heap: own Heap<'heap>) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  let Inputs(args: unused_args, cwd: unused_cwd, stdout: unused_stdout, stderr: unused_stderr, handles: entry_factory, stdin: unused_stdin) = move inputs;
  region {
    close_directory(factory: &uniq entry_factory, directory: move unused_cwd);
  }
  let code = 0_u8;
  region {
    let view = slice_of(&bytes);
    let total = sum(values: view);
    if total != 10_u64 {
      set code = 1_u8;
    }
  }
  let empty = fixed_vector::<u8, 4>();
  region {
    place_back(vector: &uniq empty, value: 3_u8);
  }
  let one = move empty;
  region {
    place_back(vector: &uniq one, value: 3_u8);
  }
  let two = move one;
  region {
    place_back(vector: &uniq two, value: 3_u8);
  }
  let three = move two;
  region {
    place_back(vector: &uniq three, value: 3_u8);
  }
  let local = move three;
  region {
    let view = slice_of(&local);
    let total = sum(values: view);
    if total != 12_u64 {
      set code = 2_u8;
    }
  }
  region {
    match heap_vector::<u8>(store: &uniq heap, count: 4_u64) {
      None() => {
        return exit_status(code: 4_u8);
      }
      Some(value: fresh) => {
        let runtime = move fresh;
        for @fill (
          at in 0_u64..4_u64,
          invariant grown: len_of(runtime) >= at,
          invariant spare: room_of(runtime) + at >= 4_u64,
          invariant flat: head_of(runtime) <= 0_u64
        ) {
          place_back(vector: &uniq runtime, value: 2_u8);
        }
        region {
          let view = slice_of(&runtime);
          let total = sum(values: view);
          if total != 8_u64 {
            set code = 3_u8;
          }
        }
      }
    }
  }
  return exit_status(code: code);
}
"#;
    let llvm = compile(source);
    let sum = emitted_function(&llvm, "sum");
    let main = emitted_function(&llvm, "main");
    // The counted range discharges the slice read before lowering, so the
    // element address forms directly without a runtime bounds branch.
    assert!(sum.contains("getelementptr inbounds i8"));
    assert!(!sum.contains("call void @free"));
    // One release, for the reason the buffer's free had: exactly one of the
    // three origins owns store storage, and `main` holds the provider that
    // its release spends [PROV-1, BLK-1].
    assert_eq!(main.matches("call void @free").count(), 1);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn an_out_of_bounds_slice_read_is_an_op4_compile_rejection() {
    // The slice carries its source run's window length, so the constant
    // offset is refutable at compile time and the program rejects with the
    // residual [OP-4, ENT-6] — the same residual the array origin gave.
    let source = br#"fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq empty, value: 0_u8);
  }
  let one = move empty;
  region {
    place_back(vector: &uniq one, value: 0_u8);
  }
  let bytes = move one;
  region {
    let window = slice_of(&bytes);
    let value = window[2_u64];
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len_of(window)"));
}

#[test]
fn returned_slice_descriptors_execute_without_transferring_storage() {
    let source = br#"const fixed: FixedVector<u8, 2> =[7_u8, 13_u8];

fn pass['r](value: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  return value;
}

fn choose['r](take_left: own Bool, left: own Slice<'r, u8>, right: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  if take_left {
    return left;
  } else {
    return right;
  }
}

fn fixed_view['r]() -> result: own Slice<'r, u8> pure {
  return slice_of(&'r fixed);
}

fn borrowed_first(value: &Slice<u8>) -> result: own u8 reads(value) contract {
  define spare = len_of(deref(value));
  requires 0_u64 < spare;
} {
  return deref(value)[0_u64];
}

fn main() -> status: own ExitStatus pure {
  let left_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq left_empty, value: 11_u8);
  }
  let left_one = move left_empty;
  region {
    place_back(vector: &uniq left_one, value: 11_u8);
  }
  let left = move left_one;
  let right_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq right_empty, value: 29_u8);
  }
  let right_one = move right_empty;
  region {
    place_back(vector: &uniq right_one, value: 29_u8);
  }
  let right = move right_one;
  region 'view {
    let borrowed_source = slice_of(&left);
    region {
      let borrowed_value = borrowed_first(value: &borrowed_source);
      if borrowed_value != 11_u8 {
        return exit_status(code: 1_u8);
      }
    }
    let initial = slice_of(&left);
    let passed = pass(value: initial);
    let passed_room = len_of(passed);
    if 0_u64 < passed_room {
      let pass_value = passed[0_u64];
      if pass_value != 11_u8 {
        return exit_status(code: 2_u8);
      }
    } else {
      return exit_status(code: 2_u8);
    }
    let left_view = slice_of(&left);
    let right_view = slice_of(&right);
    let take_left = False();
    let selected = choose(take_left: take_left, left: left_view, right: right_view);
    let selected_room = len_of(selected);
    if 0_u64 < selected_room {
      let selected_value = selected[0_u64];
      if selected_value != 29_u8 {
        return exit_status(code: 3_u8);
      }
    } else {
      return exit_status(code: 3_u8);
    }
    let constant = fixed_view::<'view>();
    let constant_room = len_of(constant);
    if 1_u64 < constant_room {
      let constant_value = constant[1_u64];
      if constant_value != 13_u8 {
        return exit_status(code: 4_u8);
      }
    } else {
      return exit_status(code: 4_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    assert!(!emitted_function(&llvm, "pass").contains("call void @free"));
    assert!(!emitted_function(&llvm, "choose").contains("call void @free"));
    assert!(!emitted_function(&llvm, "fixed_view").contains("call void @free"));
    assert!(!emitted_function(&llvm, "borrowed_first").contains("call void @free"));
    assert!(!emitted_function(&llvm, "main").contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// A view of a frame-resident run reaches that run's own slots, and it does so
/// until an ordinary borrowed call returns [VIEW-2, OWN-6].
///
/// The frame-slot planner and the emission that consumes the slot each decide
/// whether a run keeps its slots inline; a view that the planner did not see
/// would take the address of a slot no entry reserved. The pin is the whole
/// path: the emitted view is a `getelementptr` into the run's own frame slot
/// rather than a copy of a descriptor's pointer word, the run is the source
/// operand of an ordinary linked `write_once` whose loan lasts until return, and the process publishes exactly the bytes the fill
/// loop wrote.
#[test]
fn a_view_of_a_frame_resident_run_reaches_its_own_slots_until_call_return() {
    let source = br#"fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  doc "Publishes a frame-resident run through a shared view held until the linked write returns.";
  let Inputs(args: unused_args, cwd: unused_cwd, stdout: out, stderr: unused_stderr, handles: entry_factory, stdin: unused_stdin) = move inputs;
  region {
    close_directory(factory: &uniq entry_factory, directory: move unused_cwd);
  }
  let page = fixed_vector::<u8, 4>();
  for @fill (
    at in 0_u64..4_u64,
    invariant grown: len_of(page) >= at,
    invariant spare: room_of(page) + at >= 4_u64,
    invariant flat: head_of(page) <= 0_u64
  ) {
    place_back(vector: &uniq page, value: 65_u8);
  }
  region {
    let window = slice_of(&page);
    region {
      match write_once(factory: &uniq entry_factory, output: &uniq out, source: &window, start: 0_u64, end: 4_u64) {
        Ok(value: written) => {
          if written != 4_u64 {
            return exit_status(code: 1_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 2_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // The run's slots are inline, so forming the view stores the aggregate
    // into the frame slot the planner reserved and indexes there. A run whose
    // slots live behind a descriptor pointer would `extractvalue` instead, so
    // this is the shape assertion and not a spelling one.
    assert!(
        main.contains("getelementptr inbounds { [4 x i8], i64, i64 }, ptr %"),
        "the view must index the run's own frame slot:\n{main}"
    );
    // Nothing is allocated or freed: a frame-resident run owns no store
    // storage and a view owns none at all [STOR-1].
    assert!(!main.contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"AAAA");
    assert!(output.stderr.is_empty());
}

#[test]
fn a_returning_loop_with_no_break_has_a_valid_unreachable_continuation() {
    let source = br#"fn count_down(count: own u64) -> result: own u64 pure {
  let remaining = count;
  loop {
    if remaining == 0_u64 {
      return 7_u64;
    }
    set remaining = remaining - 1_u64;
  }
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  let value = count_down(count: 17_u64);
  if value != 7_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for module in [compile(source), emit_with_overlap(source)] {
        let output = compile_and_run(&module);
        assert!(output.status.success(), "{output:?}");
    }
}

#[test]
fn irregular_compute_matches_independent_sort_and_graph_oracles() {
    let kernels: [(&str, &[u8], &str, &str, &str); 2] = [
        (
            "merge_sort",
            include_bytes!("../../../../research/experiments/compute-bench/programs/merge_sort.wf"),
            include_str!("../../../../research/experiments/compute-bench/merge_sort_host.ll"),
            "WFB_SORT_ORACLE",
            include_str!("../../../../research/experiments/compute-bench/merge_sort_bench.c"),
        ),
        (
            "bfs",
            include_bytes!("../../../../research/experiments/compute-bench/programs/bfs.wf"),
            include_str!("../../../../research/experiments/compute-bench/bfs_host.ll"),
            "WFB_BFS_ORACLE",
            include_str!("../../../../research/experiments/compute-bench/bfs_bench.c"),
        ),
    ];
    for (name, source, adapter, selection, harness) in kernels {
        let oracle = format!("#define {selection}\n{harness}");
        for emitted in [compile(source), emit_with_overlap(source)] {
            if name == "merge_sort" && emitted.contains("call void @wf__par_publish(") {
                for stage in ["_par_budget_sort_values", "_par_budget_merge_values"] {
                    assert!(
                        emitted_function(&emitted, stage).contains("call void @wf__par_publish("),
                        "{stage} must offer recursive work independently of initialization"
                    );
                }
            }
            let llvm = bind_compute_host_adapter(&emitted, adapter)
                .replace("@main(", "@wf_irregular_smoke_main(")
                .replace("@wf__main_body(", "@wf_irregular_smoke_body(");
            let directory = test_directory();
            let defines = if emitted.contains("call void @wf__par_publish(") {
                vec!["WFB_ORACLE_PARALLEL=1".to_owned()]
            } else {
                Vec::new()
            };
            let executable = build_linked_executable(&llvm, Some(&oracle), &defines, &directory);
            run_compute_oracle(&executable, name, !defines.is_empty());
            std::fs::remove_dir_all(directory).expect("remove irregular-compute test files");
        }
    }
}
