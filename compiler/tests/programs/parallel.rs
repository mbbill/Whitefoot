//! The corpus program that carries the permission path end to end.
//!
//! `par_layout.wf` is a box-tree layout pass written twice over one tree: once
//! with a per-node measure whose walk is bounded by the metric table's own
//! length, and once with a measure whose bound comes from the caller and is
//! therefore carried by a claim. The two folds differ in nothing else, so the
//! program isolates exactly what the permission judgment does with a claim in
//! the call closure — and since batch 0077 the answer is *nothing*: both folds
//! are permitted, both are eligible, and both are handed out. The claim is the
//! writer's lemma, so a correct program traps under no schedule and the
//! guarantee it would otherwise buy is one only a defective program collects.
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
    build_program, compile_program, compile_program_with_overlap, compile_programs,
    corpus_program_files, program_permission_ledger, run_counting_grants,
    try_compile_programs_with_overlap,
};
use whitefoot::{CompilationFailureKind, module_requires_parallel_runtime};

/// Both folds are handed out, in the same module, from the same source shape.
///
/// This is the redirect made visible in emitted code. The two folds differ by
/// one claim site in a callee, and that difference used to be the difference
/// between an outlined thunk, a lane offer, and a join — and none of the
/// three. Both now carry all three.
///
/// The negative control is `@wf_measure_band`, the callee that actually holds
/// the claim: it sits in no permitted pair, so it must still name no part of
/// the runtime. It is checked for the runtime's real symbol prefix `wf__par_`.
/// The assertion this case replaced looked for `wf_par` — with one underscore
/// — which no emitted symbol can contain, since every runtime symbol reserves
/// `wf__par_`. It could not have failed, and so said nothing about the fold it
/// was written to describe.
#[test]
fn both_folds_are_handed_out() {
    let llvm = compile_program_with_overlap("par_layout.wf");
    assert!(
        module_requires_parallel_runtime(&llvm),
        "a module with an eligible site must ask for the runtime"
    );

    for symbol in ["@wf_layout", "@wf_layout_banded"] {
        let fold = function_body(&llvm, symbol);
        assert!(
            fold.contains("= call ptr @wf__par_claim(i64 ptrtoint"),
            "{symbol} must claim a lane for its first child call:\n{fold}"
        );
        assert!(
            fold.contains(", ptr @wf__par_thunk_"),
            "{symbol} must publish the outlined call to the claimed lane:\n{fold}"
        );
        assert!(
            fold.contains("call void @wf__par_join(ptr"),
            "{symbol} must join what it offered:\n{fold}"
        );
    }

    let measure = function_body(&llvm, "@wf_measure_band");
    assert!(
        measure.contains("call void @wf_trap("),
        "the negative control must be the function that carries the claim:\n{measure}"
    );
    assert!(
        !measure.contains("wf__par_"),
        "a callee in no permitted pair must name no part of the runtime:\n{measure}"
    );
}

/// The ledger reports both folds identically, and reports no claim-derived
/// verdict at all: `not-actualizable` is a verdict class that no longer exists.
#[test]
fn the_ledger_reports_both_folds_eligible() {
    let ledger = program_permission_ledger("par_layout.wf").join("\n");
    assert!(
        ledger.contains("pair(layout, layout)  eligible"),
        "the claim-free fold's child pair must be reported eligible:\n{ledger}"
    );
    assert!(
        ledger.contains("pair(layout_banded, layout_banded)  eligible"),
        "the claim-bearing fold's child pair must be reported eligible too:\n{ledger}"
    );
    assert!(
        !ledger.contains("not-actualizable"),
        "no verdict may still be withheld for a reachable claim:\n{ledger}"
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

/// The claim-bearing fold is granted real lanes, and granting them moves no
/// byte of what the program publishes.
///
/// This is the redirect's payoff measured rather than argued. Every other case
/// here would pass against a runtime that refused every lane, because refusing
/// is a correct execution and the permission is never an obligation — so
/// "`layout_banded` now actualizes" has to be read off the runtime's own grant
/// counter. Before batch 0077 this program's only eligible sites were `build`
/// and `layout`; `layout_banded` is the fold whose call closure reaches the
/// claim in `measure_band`.
#[test]
fn the_claim_bearing_fold_is_granted_lanes_and_publishes_the_same_bytes() {
    let llvm = compile_program_with_overlap("par_layout.wf");

    let (sequential_grants, sequential) = run_counting_grants(&llvm, Some("1"));
    assert_eq!(
        sequential.status.code(),
        Some(0),
        "the sequential execution must succeed"
    );
    assert_eq!(
        sequential_grants, 0,
        "WF_WORKERS=1 never starts the pool, so it is the honest reference"
    );

    for workers in [Some("2"), Some("4"), None] {
        let spelling = workers.unwrap_or("absent");
        let (granted, published) = run_counting_grants(&llvm, workers);
        assert_eq!(
            published.status.code(),
            Some(0),
            "WF_WORKERS={spelling} must succeed"
        );
        let mut observed_grants = granted;
        for retry in 0..4 {
            if observed_grants != 0 {
                break;
            }
            let (retried, retry_output) = run_counting_grants(&llvm, workers);
            assert_eq!(
                retry_output.status.code(),
                Some(0),
                "retry {retry} with WF_WORKERS={spelling} must succeed"
            );
            assert_eq!(
                retry_output.stdout, sequential.stdout,
                "retry {retry} with WF_WORKERS={spelling} moved a byte"
            );
            observed_grants += retried;
        }
        assert!(
            observed_grants > 0,
            "WF_WORKERS={spelling} was granted no lane, so nothing was overlapped"
        );
        assert_eq!(
            published.stdout, sequential.stdout,
            "WF_WORKERS={spelling} moved a byte of the result"
        );
    }
}

/// Every program the corpus holds, as the source list it compiles from.
///
/// A program written across several files compiles from all of them at once,
/// so the unit of compilation is this list and not the file. The list is
/// checked against the corpus directory by
/// [`the_corpus_units_cover_every_program_file`], which is what keeps it from
/// silently falling behind the corpus it claims to cover.
const CORPUS_UNITS: &[&[&str]] = &[
    &["byte_string.wf"],
    &["dir_walk.wf"],
    &["feedback_controller.wf"],
    &["fir_filter.wf"],
    &["generic_instances.wf"],
    &["generic_nominals.wf"],
    &["geometry_vectors.wf"],
    &["grayscale_pixels.wf"],
    &["growable_vec.wf"],
    &["ipv4_checksum.wf"],
    &["mandelbrot_grid.wf"],
    &["option_slots.wf"],
    &["par_layout.wf"],
    &["percent_decode.wf"],
    &["prefix_expression.wf"],
    &["recursive_tree.wf"],
    &["sha256_abc.wf"],
    &["telemetry_packet.wf"],
    &["utf8parse.wf"],
    &["wfgrep.wf"],
    &[
        "raw_deflate.wf",
        "raw_deflate_dynamic.wf",
        "raw_deflate_dynamic_decode.wf",
        "raw_deflate_boundary.wf",
    ],
    &[
        "raw_deflate.wf",
        "raw_deflate_dynamic.wf",
        "raw_deflate_dynamic_decode.wf",
        "raw_deflate_vectors.wf",
    ],
];

/// Adding a program to the corpus puts it under the `--par` case below.
#[test]
fn the_corpus_units_cover_every_program_file() {
    for file in corpus_program_files() {
        assert!(
            CORPUS_UNITS
                .iter()
                .any(|unit| unit.contains(&file.as_str())),
            "{file} is in the program corpus but in no unit of CORPUS_UNITS, so the \
             --par case does not compile it"
        );
    }
}

/// Every corpus program compiles to a module the host accepts under `--par`,
/// and every one the overlap lowering actually changes publishes exactly what
/// its default build publishes.
///
/// The case exists because compiling is not the check. A module the emitter
/// produces can still be ill-formed, and the emitter's own `Ok` says nothing
/// about that — it took a real host assembler to reject a `--par` build of
/// `percent_decode.wf` and `sha256_abc.wf` whose phis named a block their
/// world never emitted. So each unit is *linked*, which is the step that
/// rejected them, and the case covers the whole corpus rather than the one
/// program written for this path: those two were the programs no case here
/// compiled with overlap.
///
/// Linking every unit is the cheap half. The expensive half — a second
/// compilation of the default build and four executions — is spent only on the
/// units the lowering changes, and naming the runtime is exactly that
/// condition: a module that hands nothing out is emitted byte for byte as its
/// default build, which that program's own case already links and runs.
///
/// What the comparison reaches varies by program. Most of the corpus states
/// its result in claims and publishes an exit status, so a run that ends
/// successfully with an empty record channel is the assertion that every claim
/// held under that schedule. `wfgrep.wf` is a command entry that reports its
/// usage when invoked with no arguments, so for that one unit this case
/// reaches the argument-handling path only and the link is what carries it;
/// its search path is covered with real arguments in `wfgrep.rs`.
#[test]
fn every_corpus_program_links_under_par_and_publishes_its_default_bytes() {
    let mut beyond_this_target: Vec<String> = Vec::new();
    for unit in CORPUS_UNITS {
        let named = unit.join(" + ");
        let llvm = match try_compile_programs_with_overlap(unit) {
            Ok(llvm) => llvm,
            // A target with no approved [SYS-14] directory-enumeration row does
            // not compile the programs that walk a directory, and says so
            // itself. Reading that report keeps every other corpus program
            // covered on such a host instead of taking the whole case away
            // from it; every other kind of failure is still a failure here.
            Err(failure) if failure.kind() == CompilationFailureKind::TargetQualification => {
                beyond_this_target.push(named);
                continue;
            }
            Err(failure) => panic!("{named} must compile under --par: {failure}"),
        };
        // Linking is the assertion: `build_program` fails the case if the host
        // assembler rejects the module.
        let overlapped = build_program(&llvm);
        if !module_requires_parallel_runtime(&llvm) {
            continue;
        }

        let default = build_program(&compile_programs(unit));
        let reference = default.run_with_workers(None);
        for workers in [Some("1"), Some("4"), None] {
            let spelling = workers.unwrap_or("absent");
            let published = overlapped.run_with_workers(workers);
            assert_eq!(
                published.status.code(),
                reference.status.code(),
                "{named} at WF_WORKERS={spelling} left the default build's exit status; \
                 its record channel said: {}",
                String::from_utf8_lossy(&published.stderr)
            );
            assert_eq!(
                published.stdout, reference.stdout,
                "{named} at WF_WORKERS={spelling} moved a byte of the default build's result"
            );
            assert_eq!(
                published.stderr, reference.stderr,
                "{named} at WF_WORKERS={spelling} moved a byte of the default build's \
                 record channel"
            );
        }
    }
    // Naming them keeps the exemption from spreading: a target may be short a
    // directory-enumeration row, and nothing else in this corpus may quietly
    // stop being covered.
    assert!(
        beyond_this_target
            .iter()
            .all(|unit| unit.contains("dir_walk.wf") || unit.contains("wfgrep.wf")),
        "only the directory-walking programs may be out of a target's reach: \
         {beyond_this_target:?}"
    );
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
