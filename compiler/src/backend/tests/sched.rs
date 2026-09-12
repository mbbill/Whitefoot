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

/// The wait station really parks and really wakes, and this host's cost of
/// doing so is printed beside the cost of one spin round.
///
/// `WF_PAR_SPIN_ROUNDS` is a count of misses standing in for a length of time,
/// and it is only meaningful against those two numbers. The probe measures them
/// through the core's own `wf__par_signal`, `posted` flag and
/// `wf_prim_wait_sleep`, so the constant beside them can be re-derived on any
/// host rather than inherited. Nothing here bounds an elapsed time: the
/// assertion is that a round genuinely parked, and the figures are printed for
/// a reader.
#[test]
fn the_wait_station_parks_and_reports_this_hosts_park_and_wake_cost() {
    let directory = test_directory();
    let executable = build_probe(&directory, include_str!("../sched/wake_probe.c"), &[]);
    let output = Command::new(&executable)
        .output()
        .expect("run native park-and-wake probe");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(report.contains("park-and-wake probe: PASS"), "{report}");
    assert!(report.contains("park_and_wake_ns="), "{report}");
    assert!(report.contains("spin_round_floor_ns="), "{report}");
    std::fs::remove_dir_all(directory).expect("remove native probe");
}

/// The budget one call into an ordinary recursive component starts from.
///
/// The answer is `floor(log2(64 * lanes))` — six private levels per lane's
/// worth of sequential leaves — so it rises by one per doubling of the pool
/// and holds at one lane's answer when there is no pool at all. The ceiling is
/// unreachable through the lane count on any supported host, so the last
/// configuration reaches it through the leaves-per-lane constant instead.
#[test]
fn recursion_budget_follows_the_pool_width_and_stops_at_its_ceiling() {
    let directory = test_directory();
    let executable = build_probe(
        &directory,
        include_str!("../sched/recursion_budget_probe.c"),
        &[],
    );
    for (workers, budget) in [
        ("0", "6"),
        ("1", "6"),
        ("2", "7"),
        ("4", "8"),
        ("8", "9"),
        ("16", "10"),
    ] {
        let output = Command::new(&executable)
            .arg(budget)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run native recursion budget probe");
        assert!(
            output.status.success(),
            "workers={workers}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("recursion budget probe: PASS"));
    }
    let clamped = build_probe(
        &directory,
        include_str!("../sched/recursion_budget_probe.c"),
        &["-DWF_PAR_RECURSION_LEAVES_PER_LANE=(1ull << 40)"],
    );
    for workers in ["1", "4"] {
        let output = Command::new(&clamped)
            .arg("24")
            .env("WF_WORKERS", workers)
            .output()
            .expect("run clamped recursion budget probe");
        assert!(
            output.status.success(),
            "workers={workers}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_dir_all(directory).expect("remove native probe");
}

/// What the idle window's admission test asks the host, and what it does with
/// the answer.
///
/// The window opens only for a pool that fits the CPUs this process may run on
/// AND only where those CPUs are alike, so the core reads `wf_prim_cpu_levels`
/// once at pool start. The probe checks that the primitive answers at all — at
/// least one level, and the same count twice — and then the direction of the
/// rule that holds on every host: a window that opened did so on a fitting
/// pool and on uniform CPUs, at the compiled length. Nothing here depends on
/// this host being uniform or asymmetric, and the oversubscribed arm is run
/// beside the fitting one because the two admission tests are independent.
///
/// The second build is the A/B twin's arm, `WF_PAR_IDLE_WINDOW_ON_ASYMMETRIC`,
/// which withdraws the machine test and nothing else: there a fitting pool must
/// open the window whatever the levels are. The third is the shape every
/// `WF_SCHED_TEST` build compiles — the window's constant at zero — where no
/// pool opens a window at all, which is the arrival the park-protocol probes
/// above are written against.
#[test]
fn the_idle_window_asks_whether_this_hosts_cpus_are_alike() {
    let directory = test_directory();
    for (definitions, workers, expected) in [
        (&[][..], "2", None),
        (&[][..], "8", None),
        (&["-DWF_PAR_IDLE_WINDOW_ON_ASYMMETRIC=1"][..], "2", None),
        (&["-DWF_PAR_IDLE_WINDOW_ON_ASYMMETRIC=1"][..], "8", None),
        (&["-DWF_PAR_IDLE_WINDOW_US=0"][..], "2", Some("window_us=0")),
    ] {
        let executable = build_probe(
            &directory,
            include_str!("../sched/cpu_levels_probe.c"),
            definitions,
        );
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run native cpu levels probe");
        assert!(
            output.status.success(),
            "{definitions:?} workers={workers}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report = String::from_utf8_lossy(&output.stdout);
        assert!(
            report.contains("cpu levels probe: PASS"),
            "{definitions:?} workers={workers}: {report}"
        );
        if let Some(text) = expected {
            assert!(report.contains(text), "{definitions:?}: {report}");
        }
    }
    std::fs::remove_dir_all(directory).expect("remove native probe");
}
