# Batch 0093 — the gate inside five minutes

Branch: `batch/0093-gate-budget`, from `integration/2026-08-28` at `c2c19549`
with `batch/0090-ci-real-hosts` merged in.

Deliverables: the stage split, the stage table and the core-dump disposition in
`Makefile` and `compiler/Makefile`; the restructured sampling cases in
`compiler/src/backend/tests/`; the rebuilt `.github/workflows/gate.yml`; this
record.

## Charter

The canonical gate must finish within five minutes on every host, local and
CI, or one iteration costs half a day. Measured on the merge this branch starts
from:

| host | the gate | wall |
|---|---|---|
| `ubuntu-24.04`, x86-64, 4 cores | `gate-linux`, one `make check` | 21 min 34 s |
| `macos-14`, arm64, 3 cores | `gate-macos`, one `make check` | 5 min 35 s |
| the maintainer's machine, macOS arm64, 10 cores, warm target | `make check` | 3 min 14 s |

Nothing may be weakened to reach the budget: no case deleted, disabled,
narrowed or unwired, no assertion removed, no normative expectation rewritten,
and no wall-clock assertion added inside a test.

## Where the time was

Two independent measurements, one per host, on the tree this branch starts
from.

**The stage breakdown**, read off the raw job logs of gate run
[33142220230](https://github.com/mbbill/Whitefoot/actions/runs/33142220230)
(`af61110c`) by the timestamp of each stage's first line:

| stage | Linux | macOS | maintainer's machine |
|---|---|---|---|
| `repository-invariants` … `spec-digest-sync` | 1.4 s | 2.3 s | 2 s |
| `conformance` (the corpus's own runner) | 0.3 s | 0.4 s | 1 s |
| `compiler/format` | 2.7 s | 1.0 s | — |
| `compiler/lint` | 16.3 s | 14.6 s | — |
| `compiler/test` | **1157.9 s** | 216.9 s | — |
| `compiler/docs` | 6.9 s | 7.7 s | — |
| `compiler/spec` | 15.3 s | 16.0 s | — |
| `compiler/completion-test` | 4.5 s | 1.5 s | — |
| `compiler` (all of the above) | 1203.6 s | 257.7 s | 122 s |
| `research-tests` | 12.9 s | 12.0 s | 9 s |
| `conformance-run` | 51.4 s | 48.9 s | 60 s |
| **`make check`** | **1269.6 s** | **321.3 s** | **194 s** |

`compiler/test` is nine tenths of the Linux gate. Inside it, the gate-profile
build is 86 s, the program corpus 54 s, and the library suite **1016 s**. The
same library suite on the macOS runner beside it is **55 s**, on a host with
one *fewer* core.

**The per-case ranking**, from the same logs, as the gap between one case
reporting and the previous one. Under `--test-threads=$(nproc)` a gap is a
lower bound on the wall of the case that closed it, which for these five is
close to exact because nothing else was still running:

| case | Linux | macOS | runs it makes, and of what |
|---|---|---|---|
| `parallel::the_repeat_reports_a_lowering_whose_joins_were_removed` | 352 s | 1.5 s | one link, then 12 runs of a build whose joins were struck out — a program expected to die |
| `trap_latch::a_racing_pair_of_false_claims_writes_exactly_one_record` | 210 s | 0.5 s | one link, then 40 runs of a program that aborts at a false claim |
| `trap_latch::a_single_false_claim_reports_the_same_bytes_at_every_worker_count` | 138 s | 0.4 s | one link, then 17 runs, all aborting |
| `trap_latch::the_sequential_schedule_names_one_claim_every_run` | 42 s | 0.3 s | one link, then 16 runs, all aborting |
| `trap_latch::the_latch_is_what_keeps_the_record_single` | 20 s | 0.3 s | one link, then 8 runs of a build with the latch defeated, all aborting |
| `exhaustion::a_frame_larger_than_the_guard_region_is_still_reported` | > 60 s | 19.5 s | two links, then two runs that recurse until the stack is gone |

Every one of them spawns a program that **dies**. Nothing else in the suite
differs between the two hosts by more than the ratio of their cores. Per spawn
the difference is 18 milliseconds on macOS against about 5 seconds on Linux —
two and a half orders of magnitude, on a host that is not two and a half orders
of magnitude slower at anything else.

## What each restructured case still proves

No case lost an assertion. Three of the four changes below cut the *number* of
runs a healthy host makes without changing the predicate the case decides; the
fourth cuts the number of links, which no case ever asserted anything about.

### `parallel::the_repeat_reports_a_lowering_whose_joins_were_removed`

The property: a lowering with both `wf__par_join` calls struck out is caught by
the byte comparison the repeat above it makes — at least one of twelve runs of
the injected build must disagree with the intact reference. Detection is
per-run and measured at about four in five, which is why the case samples at
all.

The requirement is **existential**, so the loop now stops at the first
disagreement. Twelve is unchanged as the bound, and it is the bound the
*undetected* direction pays: a comparison that cannot see a missing join still
makes all twelve runs and still fails, at exactly the false-green probability
the case was written for (0.2 ^ 12, below one in two hundred million). What the
change removes is eleven further runs of a program whose whole purpose is to
crash.

### The two grant observations, and the two in `loop_split`

The property: the parallel runtime is granted at least one lane, so that every
other case in those modules is not passing against a runtime that granted
nothing. `GRANT_OBSERVATION_RUNS` is 32 and `GRANT_RUNS` is 4 because a steal
is a scheduling event and one run samples the host rather than the lowering.

Every caller asserts `granted > 0`, which is existential again, so
`CountedProgram::grants_over_runs` stops at the first granted lane. The
predicate is identical on every input sequence: the total is positive exactly
when some run of the sample was granted a lane, and a runtime that grants
nothing still makes all 32 (or all 4) runs and still returns zero. The bounds
did not move.

### `CountedProgram`: one link, many runs

`run_counting_grants` linked a module, ran it once and deleted the executable;
`grants_over_runs` linked the same module again. A case that asks one program
several questions — what it grants at four lanes, what it grants with
`WF_WORKERS` absent, that each named opt-out grants nothing — therefore linked
the same executable up to **five times**, and clang compiles the whole parallel
runtime, the exhaustion floor and the observer on each. The two free functions
are replaced by one `CountedProgram` fixture that links once and answers as
many questions as the case has. The link is exactly the link that was there;
the runs are exactly the runs that were there.

While converting the call sites, one doc-comment defect from batch 0090 is
fixed in place: the paragraph describing what `grants_over_runs` returns had
come to sit above `a_steal_is_observable`, whose own first line followed it.

## The gate, split by stage

`make check` was one target with nine prerequisites and no visibility into
where its wall went. It is now a loop over the same nine stages that times each
one and ends with the table, and `make -C compiler check` does the same for its
own six. Nothing about what runs changed: `CHECK_STAGES` and `COMPILER_STAGES`
are the same targets in the same order, each still a target of its own.

The compiler's `test` target, which was one `cargo test --profile gate
--all-targets`, is now four targets that partition it by cost class:

| target | what it runs |
|---|---|
| `test-unit` | the library suite minus the sampling modules |
| `test-sampling` | `exhaustion`, `loop_split`, `parallel`, `stackless`, `trap_latch` — every module whose cases link a program and run it many times |
| `test-corpus` | the three integration targets and the binaries' own unit tests |
| `test-partition` | the proof that the first two are a partition |

A scheduling split that quietly drops cases is the worst kind of green, so
`test-partition` is not a comment. It lists the library suite three ways —
whole, with the sampling modules skipped, and with only the sampling modules —
and fails unless the two halves sum exactly to the whole and the sampled half
is non-empty. It then compares the integration targets `test-corpus` names
against the `tests/*.rs` files cargo actually discovers, so a new integration
file cannot fall out of the gate by being no one's business. Both checks cost a
`--list` and no test time. On Linux the line reads
`library split: 1377 cases = 1312 fast + 65 sampled`; on macOS, which has
twelve more cases behind `#[cfg(target_os = "macos")]`, `1389 = 1324 + 65`.

`.github/workflows/gate.yml` then runs one job per stage per host instead of
one `make check` per host — twelve jobs, each of which runs exactly the make
targets `check` runs and nothing beside them. One host's `make check` is a sum;
six jobs on that host are a maximum. There is no shared build job: the compiler
crate has no dependencies, so a build is 40 to 70 seconds per job and paying it
in parallel beats serialising behind an artifact upload and download.

Two other things the measurement named:

**`conformance-run` was building and running the compiler unoptimized.** It is
the only `cargo test` in the gate that did not name `--profile gate`, so it
made a second, unoptimized build of the whole crate and then ran five hundred
compilations in it. With the profile the other targets already use, it went
from 48.9 s to 28.5 s on the macOS runner and from 49.5 s to 23.6 s on the
Linux one, and locally from 87 s to 42 s — the local figure larger because
locally the gate build already exists and the dev build did not.

**The cache was warming nothing and evicting everything.** Its key ended in
`github.sha`, so the exact key missed on every commit and every job uploaded a
fresh several-hundred-megabyte entry that the next commit could not use.
`compiler/target` was in it, and could not help: the crate has no dependencies
at all, so a cached target directory is reusable only by a commit that changed
no compiler source. It is out of the cache now. What stays is the crate
registry and the research experiments' build directory — crates with real
dependencies whose lock files change perhaps twice a year — keyed on the lock
files alone, so the exact key hits and nothing is written on an ordinary
commit. Restore fell from 17.6 s to under a second.
