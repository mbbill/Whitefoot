//! Program results and host-visible parallel behavior. Compiler permissions,
//! private IR and controlled worker observations live in backend tests.

use super::support::{
    build_program, compile_program, compile_program_with_overlap, compile_program_without_overlap,
};
use whitefoot::module_requires_parallel_runtime;

/// The recursive corpus program publishes one byte sequence whatever the
/// recursion budget cuts, and whatever width it was cut for.
///
/// `adaptive_quadrature.wf` is the corpus's one ordinary recursive component:
/// `adaptive` subdivides until its tolerance is met and hands one half of each
/// subdivision out. Under `--par` that component gets a budget-carrying
/// family, so the tree it evaluates is cut into sequential subtrees at a depth
/// the runtime takes from the pool width — a different depth at one lane than
/// at four — and `--par-recursive-frontier off` is the same program with the
/// family withheld, offering at every node. [PAR-1] fixes every value to the
/// source-order result, so none of that may move a bit, and the reference here
/// is the `--no-overlap` build, the lowering that actualizes nothing at all.
#[test]
fn the_quadrature_program_publishes_one_byte_sequence_at_every_recursion_budget() {
    use whitefoot::{
        CompilerLimits, OverlapLowering, RecursionBudget, SourceInput, compile_with_overlap,
    };

    let source = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tests/programs/adaptive_quadrature.wf"
    ))
    .expect("the corpus holds the quadrature program");
    let sequential = compile_program_without_overlap("adaptive_quadrature.wf");
    assert!(
        !module_requires_parallel_runtime(&sequential),
        "the `--no-overlap` reference must name no part of the runtime"
    );
    let reference = build_program(&sequential).run_with_workers(None);
    assert!(
        reference.status.success(),
        "the sequential reference must succeed: {}",
        String::from_utf8_lossy(&reference.stderr)
    );
    assert_eq!(reference.stdout.len(), 64);

    // The command line's own `--par`, scalar-leaf limit and all, and the same
    // build with the family withheld by the control.
    for budget in [None, Some(RecursionBudget::Off)] {
        let overlap = match budget {
            None => OverlapLowering::OnWithoutSmallScalarLeaves {
                maximum_operations: 16,
            },
            Some(budget) => OverlapLowering::OnWithRecursionBudget {
                budget,
                maximum_scalar_leaf_operations: Some(16),
                sequential_refusal: false,
            },
        };
        let overlapped = compile_with_overlap(
            &[SourceInput::new("adaptive_quadrature.wf", &source)],
            CompilerLimits::default(),
            overlap,
        )
        .expect("the quadrature program compiles under `--par`");
        assert_eq!(
            overlapped.lines().any(|line| {
                line.starts_with("define ") && line.contains(" double @wf__par_budget_adaptive(")
            }),
            budget.is_none(),
            "the default emits a budget-carrying family and `off` withholds it"
        );
        assert_eq!(
            overlapped.contains("call i64 @wf__par_recursion_budget()"),
            budget.is_none(),
            "only a budget-carrying family asks the runtime for its entry budget"
        );

        let program = build_program(&overlapped);
        for workers in ["1", "2", "4"] {
            let output = program.run_with_workers(Some(workers));
            assert!(
                output.status.success(),
                "WF_WORKERS={workers} must succeed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                output.stdout, reference.stdout,
                "WF_WORKERS={workers} moved a byte of the integral"
            );
            assert!(output.stderr.is_empty(), "WF_WORKERS={workers}");
        }
    }
}

/// The migrated tree/window/spine family has independent semantic results;
/// sequential/parallel agreement alone would permit two identical mistakes.
/// Each source constructs one ordinary and one parallel image, then executes
/// one process per selected width. Scheduling-path evidence lives in compiler
/// observers, so these cases do not retry until a steal happens by chance.
#[test]
fn tree_window_and_deep_spine_preserve_their_independent_results() {
    fn mix(left: u64, right: u64) -> u64 {
        left.rotate_left(13) ^ right ^ ((u128::from(right) * 2_654_435_761) >> 64) as u64
    }
    fn fold(values: &[u64]) -> u64 {
        if values.len() == 1 {
            return values[0];
        }
        let middle = values.len() / 2;
        mix(fold(&values[..middle]), fold(&values[middle..]))
    }
    let leaves: Vec<_> = (1..=32).collect();
    let tree = fold(&leaves).to_le_bytes();
    let mut spine_values = vec![1.0009765625_f64];
    for index in 0..4_000 {
        spine_values.push(spine_values[index] * 1.0009765625);
    }
    let mut spine = spine_values[4_000];
    for &value in spine_values[..4_000].iter().rev() {
        spine += value * 0.5;
    }
    let spine = spine.to_bits().to_le_bytes();
    for (name, expected) in [
        ("parallel/tree.wf", tree),
        ("parallel/window.wf", tree),
        ("parallel/spine.wf", spine),
    ] {
        let plain = build_program(&compile_program(name));
        let output = plain.run_with_workers(Some("1"));
        assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
        assert_eq!(output.stdout, expected, "{name}");
        assert!(output.stderr.is_empty(), "{name}: {output:?}");
        let parallel = build_program(&compile_program_with_overlap(name));
        for workers in ["1", "2", "4", "8"] {
            let output = parallel.run_with_workers(Some(workers));
            assert_eq!(
                output.status.code(),
                Some(0),
                "{name}/{workers}: {output:?}"
            );
            assert_eq!(output.stdout, expected, "{name}/{workers}");
            assert!(output.stderr.is_empty(), "{name}/{workers}: {output:?}");
        }
        if name == "parallel/tree.wf" {
            let source = super::support::read_program(name);
            let module = whitefoot::compile_with_overlap(
                &[whitefoot::SourceInput::new(name, &source)],
                whitefoot::CompilerLimits::default(),
                whitefoot::OverlapLowering::OnWithRecursionBudget {
                    budget: whitefoot::RecursionBudget::Pinned(
                        std::num::NonZeroU8::new(2).unwrap(),
                    ),
                    maximum_scalar_leaf_operations: None,
                    sequential_refusal: false,
                },
            )
            .expect("pinned-budget tree must compile");
            let pinned = build_program(&module);
            for workers in ["1", "4"] {
                let output = pinned.run_with_workers(Some(workers));
                assert_eq!(
                    output.status.code(),
                    Some(0),
                    "pinned/{workers}: {output:?}"
                );
                assert_eq!(output.stdout, expected);
                assert!(output.stderr.is_empty());
            }
        }
    }
}

/// One ordinary and one parallel build cover the real default-grain fold and
/// unhooked runtime reports. The reference is independent Rust arithmetic.
#[test]
fn range_fold_preserves_bytes_and_ordinary_runtime_reports() {
    let expected = (0_u64..400_000)
        .fold(0_u64, |sum, seed| {
            let mixed = (0..24).fold(seed, |state, _| {
                (state.rotate_left(27) ^ state.wrapping_mul(6364136223846793005))
                    .wrapping_add(1442695040888963407)
            });
            sum.wrapping_add(mixed)
        })
        .to_le_bytes();
    let plain = build_program(&compile_program("parallel/range_fold.wf"));
    let reference = plain.run_with_workers(Some("1"));
    assert!(reference.status.success());
    assert_eq!(reference.stdout, expected);
    assert!(reference.stderr.is_empty());
    let parallel = build_program(&compile_program_with_overlap("parallel/range_fold.wf"));
    for workers in [None, Some("0"), Some("1"), Some("2"), Some("4"), Some("8")] {
        let output = parallel.run_with_workers(workers);
        assert!(output.status.success(), "workers={workers:?}: {output:?}");
        assert_eq!(output.stdout, expected, "workers={workers:?}");
        assert!(output.stderr.is_empty());
    }
    for workers in ["1", "4"] {
        for report in ["0", "1", "2"] {
            let output = parallel.run_with_settings(
                Some(workers),
                &[("WF_SPLIT_WORK", "60000"), ("WF_SCHED_REPORT", report)],
            );
            assert!(output.status.success(), "{output:?}");
            assert_eq!(output.stdout, expected);
            if report == "2" {
                let text = String::from_utf8_lossy(&output.stderr);
                assert_eq!(text.lines().count(), 1, "{text}");
                assert!(
                    text.starts_with(&format!("compute: threads={workers} ")),
                    "{text}"
                );
                let started = if workers == "1" { "0" } else { "3" };
                assert!(
                    text.contains(&format!("workers_started={started} ")),
                    "{text}"
                );
            } else {
                assert!(output.stderr.is_empty(), "{output:?}");
            }
        }
    }
    let disabled = parallel.run_with_settings(Some("4"), &[("WF_SPLIT_WORK", "0")]);
    assert!(disabled.status.success());
    assert_eq!(disabled.stdout, expected);
    for (name, value, expected_error) in [
        (
            "WF_SCHED_REPORT",
            "3",
            "whitefoot scheduler: WF_SCHED_REPORT must be an integer from 0 through 2\n",
        ),
        (
            "WF_SPLIT_WORK",
            "no",
            "whitefoot scheduler: WF_SPLIT_WORK must be an integer from 0 through 1000000000\n",
        ),
    ] {
        let output = parallel.run_with_settings(Some("4"), &[(name, value)]);
        assert_eq!(output.status.code(), Some(1));
        assert!(
            output.stdout.is_empty(),
            "configuration must be refused before WF output"
        );
        assert_eq!(String::from_utf8_lossy(&output.stderr), expected_error);
    }
}

/// Each fold overwrites its prior output without reading it. The former
/// 800-pass fixture published only the final exactly representable seed
/// (16 + 799/16 = 65.9375). Check both complete established result words.
#[test]
fn layout_preserves_both_results_without_benchmark_repetition() {
    let expected = b"420a993efa7437a1 41fa962893d45299\n";
    let plain = build_program(&compile_program("par_layout.wf"));
    let output = plain.run_with_workers(Some("1"));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty());
    let parallel = build_program(&compile_program_with_overlap("par_layout.wf"));
    for workers in [None, Some("1"), Some("2"), Some("4")] {
        let output = parallel.run_with_workers(workers);
        assert!(output.status.success(), "workers={workers:?}: {output:?}");
        assert_eq!(output.stdout, expected, "workers={workers:?}");
        assert!(output.stderr.is_empty());
    }
}
