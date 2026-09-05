//! The §11 gate of `research/investigations/io-model/PARK-ON-MISS.md`.
//!
//! The gate is not that a scheduler test passes. It is that the core is
//! enumerable and that the enumeration passes: the park protocol compiled
//! against the replacement primitives of §7.1, every interleaving of those
//! primitive steps walked at the four configurations §11 derives from §5's
//! floor, and every invariant checked after every step rather than once at
//! the end. A hand written test that walks one interleaving is evidence that
//! the interleaving behaves, and it is not this gate.

use std::collections::BTreeMap;
use std::process::Command;

use super::test_directory;

/// The scheduler core's contract.
const SCHED_CORE_HEADER: &str = include_str!("../sched/core.h");
/// The seven primitives of §7.1 the core is written against, which the
/// enumerator supplies in place of the host's.
const SCHED_PRIM_HEADER: &str = include_str!("../sched/prim.h");
/// The stack switch a park and a resume go through.
const SCHED_SWITCH_HEADER: &str = include_str!("../sched/switch.h");
/// The enumerator's own contract: one schedule's body, its reset, its end
/// check, and the arms it exists to reach.
const SCHED_ENUMERATE_HEADER: &str = include_str!("../sched/enumerate.h");
/// The park protocol under enumeration.
const SCHED_CORE_SOURCE: &str = include_str!("../sched/core.c");
/// The exhaustive interleaving enumerator.
const SCHED_ENUMERATE_SOURCE: &str = include_str!("../sched/enumerate.c");
/// The §10 schedules the enumerator sweeps.
const SCHED_SCHEDULES_SOURCE: &str = include_str!("../sched/schedules.c");

/// The files one enumerator build needs. `prim_host.c` is deliberately absent:
/// the enumerator is the primitive layer here, and no thread is created.
const ENUMERATOR_UNITS: [(&str, &str); 7] = [
    ("core.h", SCHED_CORE_HEADER),
    ("prim.h", SCHED_PRIM_HEADER),
    ("switch.h", SCHED_SWITCH_HEADER),
    ("enumerate.h", SCHED_ENUMERATE_HEADER),
    ("core.c", SCHED_CORE_SOURCE),
    ("enumerate.c", SCHED_ENUMERATE_SOURCE),
    ("schedules.c", SCHED_SCHEDULES_SOURCE),
];

/// The constants §11 fixes: two lane slots, because I4's second half wants a
/// power of two and never three; ceilings that admit every swept
/// configuration; and the idle window's bounded spin pinned to one round with
/// no yield round, which `compiler/Makefile`'s `sched-enumerate` pins the same
/// way and for the same two reasons. A spin round here is not a delay but a
/// repetition of the window's looks, and every look is a primitive step the
/// search branches on: one round costs 9.4 times the states at (2,4) and a
/// second costs 24 times the states at (2,3). A yield round is worse than
/// costly -- the enumerator makes a yield block until another process writes,
/// so a yield in front of the park forces every device completion ahead of the
/// park and the one thread then never sleeps on the primitive, which S10a
/// asserts it does.
const ENUMERATOR_DEFINES: [&str; 6] = [
    "-DWF_SCHED_ENUMERATE",
    "-DWF_SCHED_LANE_SLOTS=2u",
    "-DWF_SCHED_MAX_THREADS=4u",
    "-DWF_SCHED_MAX_STACKS=8u",
    "-DWF_SCHED_IDLE_SPIN_ROUNDS=1u",
    "-DWF_SCHED_IDLE_YIELD_ROUNDS=0u",
];

/// Reached at every swept configuration: a park, the two states it passes
/// through, the resume that ends it, and a stack that empties to the pool and
/// is handed out again (group A items 1 to 4 and 8).
const EVERY_CONFIGURATION: [&str; 6] = [
    "parks",
    "suspended",
    "ready_from_suspended",
    "resume",
    "empty",
    "take",
];

/// Group C item 11 at §5's floor, S = T + 1: the first park empties the free
/// list, so the next miss with nothing READY takes the fourth line, on both
/// of its arms.
const AT_THE_FLOOR: [&str; 2] = ["exhausted_io", "exhausted_compute"];

/// The arms a second thread makes reachable: the NOTIFIED window of group A
/// item 5 on both record kinds, the cancel of item 6, a resume by a thread
/// that is not the parking one, a steal, a READY stack found inside the I/O
/// arm (item 12), and each place a publication can land relative to the
/// parking window.
const AT_TWO_THREADS: [&str; 12] = [
    "notified",
    "ready_from_notified",
    "cancel_suspending_io",
    "cancel_suspending_compute",
    "resume_foreign",
    "steals",
    "start_after_park",
    "post_by_worker",
    "post_in_window",
    "post_while_asleep",
    "post_elsewhere",
    "late_parks",
];

/// Item 7's arm, a cancel consuming a notification, is unreachable by the
/// claim protocol: a publisher notifies only a registration it has claimed
/// and a parker cancels only one it took back. The enumerator fails an
/// execution that takes the transition; the count stays zero everywhere.
const NEVER: [&str; 2] = ["cancel_notified_io", "cancel_notified_compute"];

/// One sweep: the coverage counts the enumerator printed for each schedule it
/// admitted at that configuration.
struct Report {
    configuration: String,
    schedules: BTreeMap<String, BTreeMap<String, u64>>,
}

impl Report {
    /// One coverage key summed over every schedule of the sweep.
    fn total(&self, key: &str) -> u64 {
        let mut named = false;
        let mut total = 0;
        for counts in self.schedules.values() {
            if let Some(count) = counts.get(key) {
                named = true;
                total += *count;
            }
        }
        assert!(named, "the enumerator reports no `{key}` count");
        total
    }

    /// Asserts the sweep reached every named arm somewhere in its schedules.
    fn reaches(&self, arms: &[&str]) {
        for arm in arms {
            assert!(
                self.total(arm) > 0,
                "no interleaving at {} reached `{arm}`",
                self.configuration
            );
        }
    }

    /// Asserts no interleaving of the sweep reached any named arm.
    fn never_reaches(&self, arms: &[&str]) {
        for arm in arms {
            assert_eq!(
                self.total(arm),
                0,
                "an interleaving at {} reached `{arm}`",
                self.configuration
            );
        }
    }
}

/// The `key=value` fields of one enumerator line.
fn fields(line: &str) -> BTreeMap<&str, &str> {
    line.split_whitespace()
        .filter_map(|field| field.split_once('='))
        .collect()
}

/// Builds the enumerator and sweeps every interleaving of one configuration.
/// `search` is the enumerator's search: `state`, the gate's explicit-state
/// search, or `dfs`, the re-executing walk of every interleaving that is the
/// reference at one thread.
fn enumerate(threads: u32, stacks: u32, search: &str) -> Report {
    let directory = test_directory();
    for (name, source) in ENUMERATOR_UNITS {
        std::fs::write(directory.join(name), source).expect("write one enumerator unit");
    }
    let executable = directory.join("enumerate");
    let compiled = Command::new("/usr/bin/clang")
        .current_dir(&directory)
        .args([
            "-std=c11",
            "-O2",
            "-g",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-Wpedantic",
        ])
        .args(ENUMERATOR_DEFINES)
        .args(["core.c", "enumerate.c", "schedules.c", "-o"])
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        compiled.status.success(),
        "the enumerator must compile:\n{}",
        String::from_utf8_lossy(&compiled.stderr)
    );

    let configuration = format!("threads={threads} stacks={stacks}");
    let threads_argument = threads.to_string();
    let stacks_argument = stacks.to_string();
    let mut command = Command::new(&executable);
    command.current_dir(&directory).args([
        "--threads",
        &threads_argument,
        "--stacks",
        &stacks_argument,
        "--search",
        search,
    ]);
    let swept = command.output().expect("run the enumerator");
    let printed = String::from_utf8_lossy(&swept.stdout).into_owned();
    assert!(
        swept.status.success(),
        "the sweep at {configuration} did not pass:\n{}{printed}",
        String::from_utf8_lossy(&swept.stderr)
    );

    let mut schedules: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for line in printed.lines() {
        let named = fields(line);
        let Some(name) = named.get("schedule") else {
            continue;
        };
        let counts: BTreeMap<String, u64> = named
            .iter()
            .filter_map(|(key, value)| Some(((*key).to_owned(), value.parse().ok()?)))
            .collect();
        // A bounded execution is one the sweep cut short, so the schedule's
        // interleavings were sampled rather than enumerated.
        assert_eq!(
            counts.get("bounded"),
            Some(&0),
            "schedule {name} left executions at the step bound"
        );
        schedules.insert((*name).to_owned(), counts);
    }
    let announced = printed
        .lines()
        .find(|line| line.starts_with("enumerate: PASS"))
        .expect("a sweep that exits zero announces its pass");
    assert_eq!(
        fields(announced)
            .get("schedules")
            .and_then(|count| count.parse::<usize>().ok()),
        Some(schedules.len()),
        "the sweep swept schedules whose coverage it did not report"
    );

    std::fs::remove_dir_all(&directory).expect("remove the enumerator directory");
    Report {
        configuration,
        schedules,
    }
}

/// §5's floor for one thread, and the configuration a host at the floor
/// actually runs.
#[test]
fn enumeration_holds_at_one_thread_two_stacks() {
    let report = enumerate(1, 2, "state");
    report.reaches(&EVERY_CONFIGURATION);
    report.reaches(&AT_THE_FLOOR);
    report.never_reaches(&NEVER);
}

/// The explicit-state search prunes a state it has seen; the re-executing
/// walk prunes nothing. At one thread the walk is feasible, and the two must
/// reach exactly the same arms on every schedule, which is the check that the
/// pruning, and the ample steps the search takes alone, lose no behaviour.
#[test]
fn explicit_state_search_reaches_what_the_full_walk_reaches() {
    let walked = enumerate(1, 2, "dfs");
    let searched = enumerate(1, 2, "state");
    assert_eq!(
        walked.schedules.keys().collect::<Vec<_>>(),
        searched.schedules.keys().collect::<Vec<_>>(),
        "the two searches swept different schedules"
    );
    for (name, walked_counts) in &walked.schedules {
        let searched_counts = &searched.schedules[name];
        for (key, count) in walked_counts {
            if key == "executions" || key == "max_steps" || key == "states" || key == "pruned" {
                continue;
            }
            assert_eq!(
                *count > 0,
                searched_counts.get(key).copied().unwrap_or(0) > 0,
                "schedule {name}: the walk and the search disagree on whether `{key}` is reached"
            );
        }
    }
}

/// One stack above the floor, which is what S19's two parks on one thread
/// need.
#[test]
fn enumeration_holds_at_one_thread_three_stacks() {
    let report = enumerate(1, 3, "state");
    report.reaches(&EVERY_CONFIGURATION);
    report.never_reaches(&NEVER);
}

/// The floor again at two threads, where the fourth line is reached with a
/// second thread able to publish into the parking window.
#[test]
fn enumeration_holds_at_two_threads_three_stacks() {
    let report = enumerate(2, 3, "state");
    report.reaches(&EVERY_CONFIGURATION);
    report.reaches(&AT_THE_FLOOR);
    report.reaches(&AT_TWO_THREADS);
    report.never_reaches(&NEVER);
}

/// The largest configuration §11 derives: two stacks parked while two threads
/// run, which is what S22's crossed pair and S23's nested miss need.
#[test]
fn enumeration_holds_at_two_threads_four_stacks() {
    let report = enumerate(2, 4, "state");
    report.reaches(&EVERY_CONFIGURATION);
    report.reaches(&AT_TWO_THREADS);
    report.never_reaches(&NEVER);
}
