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
differs between the two hosts by more than the ratio of their cores. Taking the
racing-claim case, whose forty runs are forty aborts and nothing else: 0.5
seconds on the macOS runner against 403.7 on the Linux one, which is 12
milliseconds a run against 10 seconds — nearly three orders of magnitude, on a
host that is not three orders of magnitude slower at anything else.

## The cause: a piped `kernel.core_pattern`

The correlation above says the cost follows dying programs. It does not say
what the death costs, and the obvious first guess was wrong, which is why this
was measured rather than assumed.

The guess was the core-file limit. It is already zero on the runner —
`ulimit -c` reports `0` in every step — so on the face of it no core is
written at all. It is written anyway. When `kernel.core_pattern` begins with a
pipe, the kernel *ignores* `RLIMIT_CORE`, because the limit bounds the size of
a file and a pipe writes no file; `ubuntu-24.04` ships the pattern

```text
|/usr/lib/systemd/systemd-coredump %P %u %g %s %t 9223372036854775808 %h %d
```

so every aborting test program is handed to `systemd-coredump`, which reads the
dying process, compresses a core, stores it and journals it — with the dying
task waiting for it.

Four arms, one case
(`trap_latch::a_racing_pair_of_false_claims_writes_exactly_one_record`, which
runs an aborting program forty times), one tree, one `ubuntu-24.04` runner,
gate run [33143948885](https://github.com/mbbill/Whitefoot/actions/runs/33143948885).
The last column is `coredumpctl list | wc -l` after the arm, and the host began
with `No coredumps found.`:

| arm | disposition | case wall | dumps recorded |
|---|---|---|---|
| 1 | the host as it comes (soft limit already 0, hard limit `unlimited`) | 403.7 s | 41 lines — one header, **40 dumps** |
| 2 | `ulimit -c 0` set explicitly | 397.7 s | 81 — **40 more** |
| 3 | `ulimit -c 1`, the value `fs/coredump.c` reads as "skip this dump" | 401.7 s | 121 — **40 more** |
| 4 | `sysctl -w kernel.core_pattern=core`, soft limit 0 | **0.45 s** | 121 — **none** |

Forty dumps per run of the case, one per aborting program, under every value of
`RLIMIT_CORE`; none, and nine hundred times faster, once the pattern names a
file. The limit is a no-op here and the pattern is the whole of it. Arm 3 is
recorded because it was this batch's second guess and it is false: the kernel's
own skip-the-pipe value did not skip anything on this kernel, so nothing a
process can do without root declines the dump.

`.github/workflows/gate.yml` therefore sets the pattern in one Linux-only step,
where root is available, and `Makefile` and `compiler/Makefile` keep
`ulimit -c 0` as the portable half — correct on every host whose pattern names
a file, which is where a limit of zero is what declines the dump. Stated
plainly: on a Linux host whose pattern is a pipe and where nothing can change
it, the sampling job pays the dumps again and the five-minute budget does not
hold; the budget as measured rests on that CI step. No case
asserts anything about a core file; each asserts the signal, the exit status
and the record channel, and none of those moves.

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
did not move. One observation is given up and should be named: the old free
function asserted exit status 0 on every one of the N runs, and the early exit
asserts it only on the runs it makes. No case's own property depends on the
later runs (byte identity across worker counts is asserted separately at each
call site), but a counted program that granted on run 1 and exited non-zero on
run 7 would no longer be noticed here.

### `the_latch_is_what_keeps_the_record_single`: a bound sized for a bad draw

This one is a case the batch made flaky and then fixed, and it is worth
stating plainly because the flake was a consequence of the split.

The control defeats the latch and requires that at least one of its runs write
*two* records — which needs two threads to reach the writer, which needs the
pool to grant the handed-out call a lane and the losing thread to get a CPU
before the winner's `abort` finishes. Both are scheduling events, so the
detection rate is the host's. Eight runs were plenty while the case ran among
1389 others on a loaded machine; with the sampling cases running as their own
group there is less on the machine to slow the winner down, and the control
caught nothing in eight runs in one of this batch's three gate runs on
`macos-14`.

The rate was measured rather than guessed: a temporary widening of the loop to
200, with the count printed, reported **117 of 200** on the three-core runner —
a rate of 0.585, at which eight runs would miss about once in a thousand. But
eight runs *did* miss, on that same runner label, which is the finding: the
rate is not a constant of the host, it is a constant of the machine that run
drew and of what else was on it. A bound sized from 0.585 would be a bound
sized from the case that never fails.

`DEFEATED_LATCH_RUNS` is therefore 200, which misses only where the rate has
fallen below about two percent — an order of magnitude under anything
observed. It costs about two seconds of process spawns, measured as 18.9
seconds for the macOS sampling half at eight runs against 20.3 at two hundred,
and every one of the 200 is still checked, so the wider sample widens what the
control verifies rather than diluting it.

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

### `run_with_closed_output`: a race the program could win

The Linux `corpus` job found a second one, in the harness rather than in a
case. `raw_deflate::each_boundary_and_decode_outcome_reaches_its_own_status`
ends by publishing into a destination with no reader and requires the status
[SYS-7] gives that, code 8. It reported code 0 on the four-core runner.

The helper spawned the program with standard output on a pipe and closed the
read end *afterwards*:

```rust
let mut child = Command::new(&self.executable)... .stdout(Stdio::piped()).spawn()...;
drop(child.stdout.take());
```

A pipe holds 64 KiB on Linux, and this program's output fits. So the program
could publish its whole result into the buffer and exit 0 before this process
closed anything, and the case then reported a success status for a destination
that had a reader for as long as it mattered. The property was never wrong; the
harness sometimes failed to create the condition the property is about.

The read end is now closed *before* the spawn, so there is no reader from
before the program exists and its first write reaches a closed destination on
every run and on every host. `std::io::pipe` is what makes that expressible
without a race. Nothing about what the case asserts moves.

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

## The budget, where it cannot flake

Three mechanisms, none of them inside a test:

- **`timeout-minutes: 8` on every CI job.** Eight is the ceiling, not the
  target; the target is five. A job that reaches eight has a defect worth
  failing over rather than waiting out, and this batch's first restructured run
  proved the mechanism by failing exactly the one job that was still over.
- **A "ten largest gaps" step at the end of every job**, written to the job
  summary, so the case that grew is named in the run that grew it. It is a
  ranking of *gaps* and says so: cargo runs a target's cases on as many threads
  as the host has cores, so a gap is a lower bound on the wall of the case that
  closed it. That is enough for the job it has — a case that goes from
  milliseconds to minutes closes a minutes-long gap on any host — and it costs
  no test time, which a real per-case measurement on stable Rust would. The
  step runs `if: always()`, so a job that was cancelled at its timeout still
  reports what it had reached.
- **`make check` ends with the wall of each stage**, and `make -C compiler
  check` with its own six. That is the local half of the same signal.

No test asserts a wall time, and none acquired one here. A wall-clock assertion
inside a test fails on a loaded machine and passes on an idle one, which makes
it a flake generator wearing a budget's clothes.

## Results

### The gate, before and after, per host

| | before, one `make check` | after, the slowest job | after, sum of jobs |
|---|---|---|---|
| `ubuntu-24.04` | 21 min 34 s | **2 min 23 s** | 10 min 3 s |
| `macos-14` | 5 min 35 s | **1 min 47 s** | 7 min 26 s |
| maintainer's machine, warm | 3 min 14 s | **2 min 26 s** | — |

The Linux "after" is the `corpus` job at 143 s; the macOS one is `corpus` at
107 s. Every job on both hosts is inside the five-minute target and none comes
within two and a half minutes of the eight-minute ceiling. The sum column is
what the one-job-per-host shape would still cost, and is what the split turns
into a maximum.

### Per job, gate run [33145853135](https://github.com/mbbill/Whitefoot/actions/runs/33145853135)

| job | Linux | macOS |
|---|---|---|
| `static` | 64 s | 54 s |
| `unit` | 117 s | 96 s |
| `sampling` | 134 s | 94 s |
| `corpus` | 143 s | 107 s |
| `conformance` | 105 s (red, the six documented cases) | 72 s |
| `research` | 40 s | 23 s |

Every job is green on both hosts except `conformance (ubuntu-24.04)`, which is
red on exactly the six target-qualification cases named under *Not done* and
was red on those before this batch.

### Per stage, the same content

| stage | Linux before | Linux after | macOS before | macOS after |
|---|---|---|---|---|
| library suite, fast half | (one suite, 1016 s) | 31.6 s | (one suite, 55 s) | 28.9 s |
| library suite, sampling half | (in the same 1016 s) | **54.1 s** | (in the same 55 s) | 20.3 s |
| integration targets | 54 s | 56 s | 65 s | 66 s |
| `conformance-run` | 49.5 s | 23.6 s | 48.9 s | 28.5 s |
| `research-tests` | 12.9 s | 12 s | 12.0 s | 10 s |

The local figure is a warm target on an otherwise idle machine, which is what
the "before" figure it is compared against was. It is worth saying what that
excludes, because this batch measured it by accident: a `make check` that
shared the ten cores with a second full gate on the same machine took 4 min 51
s on the same revision. The gate is CPU-bound now rather than
waiting-for-a-handler bound, so its wall is the machine's to give.

The local gate ends `== WHITEFOOT ALL TESTS GREEN ==` with its own table:

```text
repository-invariants             1 s
approval-history-integrity        0 s
spec-append-only                  0 s
spec-archive-integrity            1 s
spec-digest-sync                  0 s
conformance                       0 s
compiler                        122 s
research-tests                    6 s
conformance-run                  50 s
```

### The cases that were the problem

Linux, the five that owned the gate, before and after:

| case | before | after |
|---|---|---|
| `parallel::the_repeat_reports_a_lowering_whose_joins_were_removed` | 352 s | under 1 s |
| `trap_latch::a_racing_pair_of_false_claims_writes_exactly_one_record` | 210 s | under 1 s |
| `trap_latch::a_single_false_claim_reports_the_same_bytes_at_every_worker_count` | 138 s | under 1 s |
| `trap_latch::the_sequential_schedule_names_one_claim_every_run` | 42 s | under 1 s |
| `trap_latch::the_latch_is_what_keeps_the_record_single` | 20 s | under 1 s |
| `exhaustion::a_frame_larger_than_the_guard_region_is_still_reported` | over 60 s | 34.2 s |

None of them appears in the sampling job's ten-largest-gaps report any more
except the last, which is now the largest case in the suite and is the cost of
recursing until a stack is gone twice — the recursion's own time, not a
handler's.

## The offset-fault row the x86-64 runner lost about once in sixteen

The gate of ba95aa93 failed one case on `sampling (ubuntu-24.04)`, run
[33147446051](https://github.com/mbbill/Whitefoot/actions/runs/33147446051):

```
panicked at src/backend/tests/exhaustion.rs:546:9:
assertion `left == right` failed: a wild fault 16384 bytes below the stack
must keep its own signal: ExitStatus(unix_wait_status(0))
  left: None
 right: Some(11)
```

`only_a_fault_within_the_probe_stride_is_read_as_an_exhausted_stack` writes
four pages below the running thread's stack and requires the floor to leave it
alone — SIGSEGV, no record — because four pages is past anything a probed
descent can reach. Nothing faulted: the process wrote its word and exited 0. So
there was no fault to classify and this is neither a floor defect nor a flaky
assertion. The row's premise, that the memory below a stack is nothing, was
false on that run.

### The change that fixes it was already written, on another branch

This branch forked at 7ec7bc1a. Batch 0090 hit the same failure at that same
commit and fixed it in 25ac56ef, *let the offset-fault fixture own the memory
below its stack*, which is an ancestor of nothing here: the fixture text on this
branch was byte-identical to the one that commit repaired, and
`git diff 25ac56ef^ ba95aa93 -- compiler/src/backend/tests/exhaustion.rs` is
empty. The failure is not this batch's restructuring, and the fix is that
commit's fixture change carried over unchanged rather than a second fix written
beside it. The other commit this branch never received, bc4f09a4, resizes the
same latch control this batch resized from its own measurement of the same
runner, and is superseded here rather than adopted.

### What the x86-64 runner has under the entry stack

25ac56ef read its account from this project's aarch64 Linux container, whose
64 KiB guard never loses the row. A temporary diagnostic ran the pre-fix body
200 times on the runner itself, printing the address written, the thread's own
`sigaltstack`, and `/proc/self/maps`. Gate runs
[33148142621](https://github.com/mbbill/Whitefoot/actions/runs/33148142621) and
[33148390275](https://github.com/mbbill/Whitefoot/actions/runs/33148390275),
`sampling (ubuntu-24.04)`: 14 of 200 and 11 of 200 runs completed the write —
25 of 400, about one run in sixteen, against a case that runs the row once per
gate.

A run whose write completed:

```
low=7fc36b9f0000 target=7fc36b9ec000 altstack=7fc36b9df000+10000
HIT  7fc36b9df000-7fc36b9ef000 rw-p 00000000 00:00 0
```

A run whose write faulted:

```
low=7f6d41400000 target=7f6d413fc000 altstack=7f6d817cc000+10000
```

The mapping the write landed in is the floor's own alternate signal stack,
named by address rather than inferred from its size: `ss_sp` is exactly the
mapping's first byte and `ss_size` is exactly its 64 KiB. `wf__floor_run`
creates the entry stack at 1 GiB, glibc puts a one-page `---p` guard under it,
`pthread_getattr_np` reports `low` as the top of that guard, and
`wf__floor_attach_thread` then maps the 64 KiB alternate stack — after the
stack block exists, so the kernel's top-down search offers it the gap directly
underneath. When it takes that gap, `low - 16384` is inside a writable mapping
three pages under the guard. When it does not — the faulted sample puts it
3.8 MiB *above* the 1 GiB block's top, in a gap the placement left there — the
memory under the guard is nothing and the row passes. Both rows past the stride
were exposed, not only the one that failed: with the alternate stack at
`[low - 0x11000, low - 0x1000)` the 64 KiB row's target is inside it too. The
case fails on the four-page row first because the loop reaches it first.

Nothing here says the floor misclassifies. Every generated definition carries
the probe, so a descent's first touch below the stack is at most one page under
it and lands on the guard; a mapping *below* the guard is not something a
descent can step into. It is only the fixture's premise that the placement
falsified. Batch 0090 already carries that placement forward as a floor
observation in its own record, which this batch does not edit.

### The fix, and what it measures at

The fixture now owns the memory it writes into. Its faulting thread is its own:
it reserves 16 MiB + 64 KiB + 1 MiB `PROT_NONE` in one mapping, makes the top
megabyte its stack with `pthread_attr_setstack` — which leaves no guard, so
`pthread_getattr_np` reports exactly the address the fixture chose — attaches to
the floor exactly as a pool lane does, reads its bounds, and only then unmaps
the pad below itself, so that between the `munmap` and the write nothing in the
process maps anything. Every address any row names is then inside a hole the
fixture opened. The pad is unmapped rather than left `PROT_NONE` because the
rows assert the host's own signal, and Darwin reports a protected page as
SIGBUS where an unmapped one is SIGSEGV.

The same temporary diagnostic, 40 runs of every row of the case, on both hosts:

| row | ubuntu-24.04, two runs | macos-14, 16 KiB pages |
|---|---|---|
| half a page below | 80/80 abort, `{"resource":"stack"}` | 40/40 |
| one page below | 80/80 abort, `{"resource":"stack"}` | 40/40 |
| four pages below | 80/80 SIGSEGV, no bytes | 40/40 |
| 64 KiB below | 80/80 SIGSEGV, no bytes | 40/40 |
| 16 MiB below | 80/80 SIGSEGV, no bytes | 40/40 |

Every assertion in the case is what it was: the in-band rows still require the
floor's abort and its exact record, the rows past the stride still require
SIGSEGV and an empty channel. The diagnostic was deleted with the same commit
that carried its result into this record.

## Judgment calls

- **The five sampling modules are named by measurement, not by subject.**
  `exhaustion`, `loop_split`, `parallel`, `stackless` and `trap_latch` are in
  `SAMPLING_MODULES` because their cases link a program and run it many times,
  which is the cost class the split exists to schedule. The alternative — move
  them into their own `tests/*.rs` integration target so cargo schedules them
  as a separate binary — would have meant making a large part of the compiler's
  internals public to reach them, which is a change to the crate's shape made
  for a scheduler's benefit.
- **The partition is checked, not asserted.** A list of module prefixes in a
  Makefile is exactly the kind of thing that goes stale silently: rename a
  module and its cases quietly move into the fast half. The arithmetic
  `whole = fast + sampled` cannot notice that on its own, because `--skip M`
  and the filter `M` are exact complements whatever `M` names (an independent
  verifier renamed a module and the sum still held); so `test-partition` also
  requires every name in `SAMPLING_MODULES` to match at least one case on its
  own, which fails on a renamed or deleted module. It costs a few `--list`
  runs and no test time.
- **Early exit only where the claim is existential.** Three of the trap-latch
  cases assert a property of *every* run — a racing pair writes exactly one
  record, the sequential schedule names one claim every run, one false claim
  reports the same bytes at every worker count — and they still make every run
  they made before. Their cost was cut by removing what each run was paying
  for, not by sampling less. Only the four grant observations and the
  joins-removed control, whose assertions are `> 0` and `at least one`, stop
  early.
- **The gap ranking says it is a gap ranking.** An exact per-case table on
  stable Rust means either `--test-threads=1`, which costs more wall than the
  report saves, or nightly's `--report-time`. A ranking of gaps under the
  ordinary parallel run costs nothing and catches the regression that matters —
  a case that grows by minutes. Calling it a duration table would have been the
  comfortable lie.
- **No test learned to look at a clock.** The budget is enforced by
  `timeout-minutes` outside the process. A wall-clock assertion inside a test
  fails on a loaded machine and passes on an idle one.
- **The offset-fault fixture was adopted, not rewritten.** Batch 0090 had
  already diagnosed and repaired the same failure in 25ac56ef, on text
  identical to this branch's. Writing a second fix here — reserving the region
  with `MAP_FIXED_NOREPLACE`, or placing the address relative to the observed
  mapping — would have left two shapes of the same fixture to reconcile at the
  merge, for no property the adopted one lacks. What this batch adds is the
  evidence a container could not give: the runner's own rate, and the mapping
  named by address rather than by size.

## Not done

- The two eight-lane grant controls in `compiler/src/backend/tests/loop_split.rs`
  (`a_split_loop_carries_its_captures_and_a_second_combine` and
  `every_admitted_combine_splits_and_publishes_the_unsplit_bytes`), which
  batch 0090 put behind `a_steal_is_observable(8)`, run on no CI host: the
  runners have three and four cores. They still run on the maintainer's
  ten-core machine and print their host-limited line elsewhere, but the defect
  they exist to catch is invisible to CI until a wider runner exists.

- **The Linux gate is still red at `conformance-run`, on the documented
  target-qualification gap, and this batch does not touch it.** Six cases reach
  `TargetQualification(MissingMapping(Operation(12)))` because Linux has no
  approved [SYS-14] directory-enumeration row: `sys14-list-outcome-exhaustive`,
  `sys14-list-zero-range`, `sys14-directory-release`, `sys14-entry-kind-closed`,
  `accept-sysfile-two-permits-shared-directory` and
  `accept-par3-staged-denied-opaque-cursor`. Batch 0090 recorded five; the sixth
  is an `accept` case added since. The corpus is compiler- and host-independent
  by design and giving it a per-target axis would be a change to conformance
  evidence made to turn a job green.
- **The conformance adapter still runs its five hundred cases in one serial
  case.** At the gate profile that is 24 to 29 seconds, comfortably inside the
  job, and making it concurrent would be a change to conformance collection
  wiring made for speed rather than for evidence.
- **The gate-profile build is now the floor of four jobs.** 40 to 70 seconds on
  Linux and 50 to 55 on macOS, paid four times in parallel. Nothing here
  removes it: the compiler crate has no dependencies, so there is nothing a
  cache can warm, and sharing one build between jobs costs more in artifact
  transfer than it saves.
- **The two stack-exhaustion cases still run programs until the stack is
  gone.** That is what they are, and the time is the recursion's, not a
  handler's.

## Approval classes

Under the four rules in `AGENTS.md`:

- **Rule 2** — the exact revision to be merged needs owner approval, as every
  merge does.
- **Rule 3** — the exact revision must pass `make check`. It does, on the
  maintainer's machine, ending `== WHITEFOOT ALL TESTS GREEN ==`;
  `cargo clippy --all-targets --locked --offline -- -D warnings` and
  `cargo fmt --all -- --check` are clean.
- **No specification change.** `spec/kernel-spec.md` is untouched and no
  archive moves.
- **No conformance verdict change.** No case, manifest, adapter or runner under
  `tests/conformance/` is touched, and the adapter reports the same tally on
  each host it reported before: `Pass=509 Skip=1` locally and on macOS,
  `Pass=503 Fail=6 Skip=1` on Linux.
- **One conformance-adjacent edit, flagged rather than decided here.** The
  `conformance-run` recipe in `Makefile` is collection wiring, and this batch
  changes exactly two things in it: it names `--profile gate`, and it prefixes
  `NO_CORE_DUMPS`. Neither touches a case, a manifest, the adapter or the
  runner, and the reported verdicts are identical on every host. If the owner
  reads that as conformance evidence under rule 4, the merge record is those
  two edits to that one recipe and nothing else. This record does not write an
  approval entry; `governance/APPROVALS.md` is the owner's at merge time.
- **The CI files are `.github/workflows/gate.yml`, `io-hosts.yml` and the new
  `io-bench.yml`.** The three bench jobs moved out of `io-hosts.yml` into
  `io-bench.yml`, which runs on `workflow_dispatch` and on pushes that touch
  the completion runtime, the emitter, or the bench bundle; their steps are
  byte-identical. `io-hosts.yml` keeps the two correctness jobs on every push,
  under a minute each. A push therefore never waits on a fifteen-minute
  measurement that sets no red or green.
