use super::support::{
    build_program, build_program_with_driver_arguments, compile_sources,
    compile_sources_with_cli_parallel_defaults,
};
#[cfg(unix)]
use super::support::{build_program_with_driver, compile_program};

#[cfg(unix)]
fn dense_expected(words: usize, seed: u64, rounds: u64) -> u64 {
    let mut state = seed;
    let mut values = (0..words)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state
        })
        .collect::<Vec<_>>();
    state = seed;
    for _ in 0..rounds {
        for value in &mut values {
            state = state
                .wrapping_mul(2_862_933_555_777_941_757)
                .wrapping_add(*value);
            *value = state;
        }
    }
    values.into_iter().fold(0_u64, |digest, value| {
        digest.wrapping_mul(1_099_511_628_211).wrapping_add(value)
    })
}

#[cfg(unix)]
#[test]
fn dense_scalar_and_record_updates_match_the_complete_sequence_digest() {
    let mut llvm = compile_program("containers/dense.wf");
    for name in ["scalar16", "scalar256", "wide16"] {
        let definition = format!("define i64 @wf_dense_{name}(");
        assert_eq!(llvm.matches(&definition).count(), 1);
    }
    for name in ["main", "wf__main_body"] {
        let definition = format!("define i32 @{name}(");
        assert_eq!(llvm.matches(&definition).count(), 1);
        llvm = llvm.replacen(&definition, &format!("define i32 @unused_{name}("), 1);
    }
    llvm.push_str("\ndeclare i32 @wf__main_body(i32, ptr)\n");
    let driver = r#"
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
extern uint64_t wf_dense_scalar16(uint64_t, uint64_t);
extern uint64_t wf_dense_scalar256(uint64_t, uint64_t);
extern uint64_t wf_dense_wide16(uint64_t, uint64_t);
extern int wf__floor_run(int, char **);
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    uint64_t (*kernels[])(uint64_t, uint64_t) = {
        wf_dense_scalar16, wf_dense_scalar256, wf_dense_wide16
    };
    const uint64_t rounds[] = {0, 1, 3, 4, 8};
    for (unsigned kernel = 0; kernel < 3; ++kernel)
        for (unsigned round = 0; round < 5; ++round)
            for (uint64_t at = 0; at < 12; ++at) {
                uint64_t seed = UINT64_C(7640891576956012809)
                    + at * UINT64_C(11400714819323198485);
                printf("%" PRIu64 "\n", kernels[kernel](seed, rounds[round]));
            }
    return 0;
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;
    let program = build_program_with_driver(&llvm, Some(driver));
    let output = program.run_with_workers(Some("1"));
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let mut expected = String::new();
    for words in [16, 256, 64] {
        for rounds in [0, 1, 3, 4, 8] {
            for at in 0..12_u64 {
                let seed = 7_640_891_576_956_012_809_u64
                    .wrapping_add(at.wrapping_mul(11_400_714_819_323_198_485));
                expected.push_str(&format!("{}\n", dense_expected(words, seed, rounds)));
            }
        }
    }
    assert_eq!(output.stdout, expected.as_bytes());
}

#[cfg(unix)]
#[test]
fn maps_leaf_splits_pages_and_compact_owner_regressions_execute() {
    for name in [
        "hashmap",
        "ordered",
        "packed-page",
        "priority",
        "ordered-runtime-gap",
        "boxed-helper-gap",
        "shared-option-view",
    ] {
        let module = compile_program(&format!("containers/{name}.wf"));
        let output = super::support::compile_and_run(&module);
        assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
        assert!(output.stdout.is_empty(), "{name}: {output:?}");
        assert!(output.stderr.is_empty(), "{name}: {output:?}");
    }
}

#[test]
fn grow_vector_executes_and_releases_every_allocation_in_both_lowering_modes() {
    // Filesystem locations and source-envelope logical names are independent.
    let sources: [(&str, &[u8]); 2] = [
        (
            "lib/containers/grow-vector.wf",
            include_bytes!("../../../lib/containers/grow-vector.wf"),
        ),
        (
            "containers/grow-vector-program.wf",
            include_bytes!("../../../tests/programs/containers/grow-vector-program.wf"),
        ),
    ];
    let modes = [
        ("sequential", compile_sources(&sources)),
        (
            "parallel",
            compile_sources_with_cli_parallel_defaults(&sources),
        ),
    ];

    for (mode, llvm) in modes {
        let output = build_program(&llvm).run_with_workers(None);
        assert_eq!(output.status.code(), Some(0), "{mode}: {output:?}");
        assert!(output.stdout.is_empty(), "{mode}: {output:?}");
        assert!(output.stderr.is_empty(), "{mode}: {output:?}");

        assert!(llvm.contains("@malloc("), "{mode}: missing allocator calls");
        assert!(llvm.contains("@free("), "{mode}: missing release calls");
        assert_eq!(llvm.matches("define i32 @main(").count(), 1, "{mode}");
        let observed = llvm
            .replace("@malloc(", "@wf_observe_allocate(")
            .replace("@free(", "@wf_observe_release(")
            .replace("@main(", "@wf_fixture_main(");
        let observer =
            include_str!("../../../tests/programs/containers/grow-vector-allocation-observer.c");
        let observed_program = build_program_with_driver_arguments(
            &observed,
            Some(observer),
            &[
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-Wno-override-module",
            ],
        );
        let output = observed_program.run_with_workers(None);
        assert_eq!(output.status.code(), Some(0), "{mode}: {output:?}");
        assert!(output.stderr.is_empty(), "{mode}: {output:?}");
        assert_eq!(
            output.stdout,
            b"vector allocation observer: 25 allocations, each released exactly once\n",
            "{mode}: {output:?}"
        );

        if mode == "parallel" {
            // Reuse this native image for observer controls: simultaneous
            // registration and cross-worker release, then three independent
            // wrong ledgers. No extra WF compilation or C build is needed.
            let concurrent =
                observed_program.run_with_workers_and_arguments(None, &[b"concurrent"]);
            assert_eq!(concurrent.status.code(), Some(0), "{concurrent:?}");
            assert!(concurrent.stderr.is_empty(), "{concurrent:?}");
            assert_eq!(
                concurrent.stdout,
                b"vector allocation observer: 32 allocations, each released exactly once\n"
            );
            for (argument, message) in [
                ("double-release", "allocation released twice"),
                (
                    "foreign-release",
                    "release did not return an allocated address",
                ),
                (
                    "missing-release",
                    "every allocation is released exactly once",
                ),
            ] {
                let output =
                    observed_program.run_with_workers_and_arguments(None, &[argument.as_bytes()]);
                assert_eq!(output.status.code(), Some(1), "{argument}: {output:?}");
                assert_eq!(
                    output.stderr,
                    format!("vector allocation observer: {message}\n").as_bytes(),
                    "{argument}: {output:?}"
                );
            }
        }
    }
}
