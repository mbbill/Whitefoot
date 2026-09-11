//! Native evidence for the delivered current-stack compute core. The old
//! enumerator modeled managed-stack suspension, migration and ready lists;
//! those mechanisms are retired, not approximated by these tests. The frozen
//! unified checkpoint retains that model and its historical results.

use std::path::Path;
use std::process::Command;

use super::test_directory;

fn build_probe(directory: &Path, source: &str, definitions: &[&str]) -> std::path::PathBuf {
    for (name, text) in [
        ("core.h", crate::SCHED_CORE_HEADER),
        ("core.c", crate::SCHED_CORE_SOURCE),
        ("entry.h", crate::SCHED_ENTRY_HEADER),
        ("entry.c", crate::SCHED_ENTRY_SOURCE),
        ("prim.h", crate::SCHED_PRIM_HEADER),
        ("prim_host.c", crate::SCHED_PRIM_HOST_SOURCE),
        ("probe.c", source),
    ] {
        std::fs::write(directory.join(name), text).expect("stage native runtime probe");
    }
    let executable = directory.join("probe");
    let output = Command::new("clang")
        .args([
            "-std=c11", "-O2", "-g", "-Wall", "-Wextra", "-Werror", "-pthread",
        ])
        .args(definitions)
        .args([
            directory.join("probe.c"),
            directory.join("entry.c"),
            directory.join("prim_host.c"),
        ])
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("build native runtime probe");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

#[test]
fn current_stack_nested_join_refusal_and_completion_lifetimes() {
    let directory = test_directory();
    let executable = build_probe(
        &directory,
        include_str!("../sched/smoke.c"),
        &["-DWF_SCHED_TEST"],
    );
    for (workers, mode) in [
        ("1", "normal"),
        ("4", "normal"),
        ("4", "owner-fail"),
        ("4", "worker-fail"),
        ("4", "partial"),
        ("4", "partial-three"),
        ("4", "startup-delayed"),
        ("4", "startup-partial"),
    ] {
        let output = Command::new(&executable)
            .arg(mode)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run native runtime scenario");
        assert!(
            output.status.success(),
            "workers={workers} mode={mode}: {output:?}"
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("compute smoke: PASS"));
    }
    std::fs::remove_dir_all(directory).expect("remove native probe");
}

#[test]
fn concurrent_ring_wrap_and_live_counters_with_and_without_statistics() {
    let directory = test_directory();
    for stats in ["-DWF_SCHED_STATS=1", "-DWF_SCHED_STATS=0"] {
        let executable = build_probe(
            &directory,
            include_str!("../sched/deque_probe.c"),
            &["-DWF_SCHED_LANE_SLOTS=8u", stats],
        );
        let output = Command::new(&executable)
            .output()
            .expect("run concurrent deque probe");
        assert!(output.status.success(), "{stats}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("sched deque probe: PASS"));
    }
    std::fs::remove_dir_all(directory).expect("remove native probe");
}
