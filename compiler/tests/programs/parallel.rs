//! The corpus program that carries the permission path end to end.
//!
//! `par_layout.wf` is a box-tree layout pass written twice over one tree: once
//! with a per-node measure whose walk is bounded by the metric table's own
//! length, and once with a measure whose bound comes from the caller and is
//! therefore carried by a claim. The two folds differ in nothing else, so the
//! program isolates exactly what the permission judgment adds: the same
//! sibling pair is permitted in both, and only the claim-free one is eligible
//! for actualization.
//!
//! The program publishes the exact bits of both folds, so any divergence
//! anywhere in either tree is a divergence in the published bytes. Whether the
//! runtime grants lanes at all is pinned by the in-crate runtime test, which
//! reads the pool's own grant counter; the cases here pin what this program's
//! two folds compile to and that granting lanes moves none of its bytes.
//!
//! Actualization is compile-time opt-in, so the cases that ask about hand-outs
//! compile through [`compile_program_with_overlap`] — `whitefootc --par`. The
//! default compilation of the same program is the subject of its own case
//! below and hands nothing out at all.

use super::support::{
    build_program, compile_program, compile_program_with_overlap, program_permission_ledger,
};
use whitefoot::module_requires_parallel_runtime;

/// The claim-free fold is handed out and the claim-bearing one is not, in the
/// same module, from the same source shape.
///
/// This is the whole point of the demo: the difference between the two folds
/// is one claim site in a callee, and that difference is visible in the
/// emitted code as an outlined thunk, a lane offer, and a join present in one
/// function and absent from the other.
#[test]
fn only_the_claim_free_fold_is_handed_out() {
    let llvm = compile_program_with_overlap("par_layout.wf");
    assert!(
        module_requires_parallel_runtime(&llvm),
        "a module with an eligible site must ask for the runtime"
    );

    let layout = function_body(&llvm, "@wf_layout");
    assert!(
        layout.contains("= call ptr @wf__par_claim(i64 ptrtoint"),
        "the claim-free fold must claim a lane for its first child call:\n{layout}"
    );
    assert!(
        layout.contains(", ptr @wf__par_thunk_"),
        "the claimed lane must be published the outlined call:\n{layout}"
    );
    assert!(
        layout.contains("call void @wf__par_join(ptr"),
        "the claim-free fold must join what it offered:\n{layout}"
    );

    let banded = function_body(&llvm, "@wf_layout_banded");
    assert!(
        !banded.contains("wf_par"),
        "a fold whose closure reaches a claim must name no part of the runtime:\n{banded}"
    );
}

/// The ledger says which pair was permitted, which was eligible, and why the
/// other one was not, naming the callee the claim was found through.
#[test]
fn the_ledger_separates_the_two_folds_by_their_claim_closures() {
    let ledger = program_permission_ledger("par_layout.wf").join("\n");
    assert!(
        ledger.contains("pair(layout, layout)  eligible"),
        "the claim-free fold's child pair must be reported eligible:\n{ledger}"
    );
    assert!(
        ledger.contains(
            "pair(layout_banded, layout_banded)  not-actualizable: 1 claim site via measure_band"
        ),
        "the claim-bearing fold's child pair must be reported permitted and \
         not actualizable, naming the callee that carries the claim:\n{ledger}"
    );
}

/// The default compilation of this same program hands nothing out and needs no
/// runtime, so the shipped build of a program full of eligible sites is the
/// build it was before this path existed.
///
/// The eligibility is real — the case above compiles the same file with `--par`
/// and finds the thunk, the offer, and the join — so what this pins is the
/// option, not the program.
#[test]
fn the_default_compilation_of_the_demo_names_no_runtime() {
    let llvm = compile_program("par_layout.wf");
    assert!(
        !llvm.contains("wf__par_"),
        "the default compilation must name no runtime symbol"
    );
    assert!(
        !module_requires_parallel_runtime(&llvm),
        "no link path may add the runtime to a default build"
    );

    let program = build_program(&llvm);
    let published = program.run_with_workers(None);
    assert!(published.status.success());
    assert_eq!(
        published.stdout,
        program.run_with_workers(Some("4")).stdout,
        "a program with no lane offer cannot answer to WF_WORKERS"
    );
}

/// Offering lanes moves no byte of the program's published result.
///
/// The reference is `WF_WORKERS=1` — the same executable's own sequential
/// world, where the pool never starts and every offer is refused — so what the
/// overlapped runs are compared against is an execution that overlapped
/// nothing. An absent setting is now the shipped default and starts a pool, so
/// it is one of the compared runs rather than the reference: taking it as the
/// reference would compare parallel executions with each other and would go
/// green on a defect present in all of them.
#[test]
fn the_layout_program_publishes_one_byte_sequence_at_every_worker_count() {
    let llvm = compile_program_with_overlap("par_layout.wf");
    let program = build_program(&llvm);

    let reference = program.run_with_workers(Some("1"));
    assert!(
        reference.status.success(),
        "the sequential execution must succeed: {}",
        String::from_utf8_lossy(&reference.stderr)
    );
    assert_eq!(
        reference.stdout.len(),
        34,
        "the program publishes two 16-digit values, a separator, and a newline"
    );
    assert!(reference.stderr.is_empty());

    for workers in [None, Some("2"), Some("4")] {
        let named = workers.unwrap_or("absent");
        let overlapped = program.run_with_workers(workers);
        assert!(
            overlapped.status.success(),
            "WF_WORKERS={named} must succeed: {}",
            String::from_utf8_lossy(&overlapped.stderr)
        );
        assert_eq!(
            overlapped.stdout, reference.stdout,
            "WF_WORKERS={named} moved a byte of the result"
        );
        assert!(overlapped.stderr.is_empty(), "WF_WORKERS={named}");
    }
}

/// The text of one emitted function definition, from its `define` line to its
/// closing brace.
fn function_body<'module>(module: &'module str, symbol: &str) -> &'module str {
    let opening = format!("{symbol}(");
    let start = module
        .match_indices(&opening)
        .find_map(|(offset, _)| {
            let line = module[..offset]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            module[line..offset].starts_with("define").then_some(line)
        })
        .unwrap_or_else(|| panic!("missing definition of {symbol}"));
    let end = module[start..]
        .find("\n}\n")
        .map(|offset| start + offset + 3)
        .expect("a function definition must close");
    &module[start..end]
}
