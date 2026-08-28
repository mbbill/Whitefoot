# Batch 0090 — continuous integration on real hosts

Branch: `batch/0090-ci-real-hosts`, from `main` at `79b29665`.
Deliverables: `.github/workflows/`, the Linux and Windows fixes it found in
`compiler/`, the host limits it made explicit, the Linux-hardware section of
`research/investigations/io-model/RESULTS.md`, this record.

## Charter

Every result this repository has ever recorded came from one macOS machine.
Its Linux evidence came from a Docker container on that machine — an aarch64
guest with two virtual CPUs — whose timings are not a statement about Linux
hardware, and whose kernel is reached through a hypervisor. Its Windows
evidence was a cross-link: `make -C compiler completion-windows-cross` proves
a PE imports `CreateIoCompletionPort` and stops there. Nothing had ever run.

So: give the repository continuous integration on hosts it does not own, and
use it to produce the first Linux-kernel-on-real-hardware numbers and the
first native Windows execution evidence.

## The new root entry

`.github/` is a new top-level entry, which is a structural decision under the
repository's own rules. It earns its place on three counts, all of which this
batch demonstrates rather than predicts.

It serves a current compiler capability: the Linux half of the compiler — the
io_uring adapter, the `__linux__` arm of every compiler-owned C unit, the ELF
link path, the stack ledger's assembly reader — had no host that exercised it
except a container the maintainer started by hand, and this batch found five
defects in that half, three of them in the first run and the last two only
because the earlier fixes let a run reach them. No existing directory owns it: a GitHub
workflow is addressed by path, `.github/workflows/*.yml`, and cannot live
anywhere else. And it is removed when the project stops needing a host it does
not own — which, while Linux and Windows are qualification targets, it will
not.

Two files, because they answer two different questions and are useful at
different cadences:

| file | jobs | what it answers |
|---|---|---|
| `.github/workflows/gate.yml` | `gate-linux`, `gate-macos` | does the canonical `make check` pass on a host that is not this machine |
| `.github/workflows/io-hosts.yml` | `completion-linux`, `bench-linux`, `completion-windows` | does the completion I/O model work, and how fast, where only a real kernel can say |

Both trigger on push to any branch and on `workflow_dispatch`, both cancel a
superseded run for the same ref, and both cache the cargo registry and the
build directory. The repository map in `README.md` names the new entry.

One design point worth stating, because it is the difference between evidence
and decoration: `completion-linux` treats an absent io_uring as a **job
failure**. The compiler's own `completion-test` target deliberately tolerates
exit 77 from the native adapter probe, because a build host is allowed to lack
the ring. A job whose entire purpose is native-adapter evidence is not, so the
job runs the probe itself first and fails on 77 rather than letting the
fallback path be reported as an io_uring result.

## What the gate found on the new hosts

Sixteen distinct stops, in three classes, over the batch's runs — five
defects, five tests that were measuring a host rather than the compiler, and
six cases a host cannot reach. None of them was visible from one host, and the later ones were
reachable only because the earlier fixes let a run get that far: a ledger with
no call graph cannot be caught reporting the wrong depth, a job that stops in
`cargo test` never links the program corpus, and one that stops in the program
corpus never reaches the research tests.

### Defects, fixed

**The stack ledger read no call graph at all on x86-64 Linux.**
`backend/stack_ledger.rs` builds the call graph by reading the assembly clang
emits, and it stripped only Darwin's comment markers — `;` and `##`. On
x86-64 ELF a function label is written

```text
wf_spine:                               # @wf_spine
```

which does not end at the colon. No label resolved, so no instruction ever had
an open caller, so the graph was empty. The ledger then reported a genuine
recursion as a chain with no cycle, and the three cases that ask what one level
of `wf_spine` costs failed. This is the shipped ledger — the artifact that says
how deep a recursion may go before the stack is exhausted — not a test fixture.
The fix is one function, `strip_comment`, that knows all four markers the
supported assemblers use: `;` on Darwin arm64, `##` on Darwin x86-64, `//` on
ELF arm64, `#` on ELF x86-64.

**The deterministic test host did not compile under glibc.** The link names
`-std=c11`, which defines `__STRICT_ANSI__`, under which glibc withholds the
traditional default set that `S_IFREG` and `S_IFDIR` live in. Every
compiler-owned C unit already asks for the feature set it needs; this
generated one did not. Eleven `deterministic_target` cases failed to build.

**A never-compiled `__linux__` arm.** The offset-fault fixture in
`backend/tests/exhaustion.rs` calls `pthread_getattr_np` on Linux without
`_GNU_SOURCE`, which is an implicit declaration and an error. The floor
runtime it links against sets that macro for the same call; the fixture did
not, because the arm had never been compiled anywhere.

**The stack ledger promised twice the recursion depth x86-64 has.** With the
call graph readable, the ledger's own end-to-end case — build the recursion
just inside the reported ceiling, build it just outside, run both — failed on
the inside half: the ledger reported 134,217,728 levels of the tight spine on
the one-gigabyte runtime stack and the program died at 134,083,510. The cause
is that `-fstack-usage` reports what a function allocates for itself, and the
two qualified architectures put the return address on opposite sides of that
line. Emitting one module and compiling it for both targets shows it directly:
`wf_spine` reports 8 bytes on x86-64 and its whole prologue is `pushq %rax`,
with the return address the caller's `call` pushed sitting above the 8; it
reports 16 on arm64, with a prologue of `stp x29, x30, [sp, #-16]!` that
stores the return address inside the 16. One activation costs sixteen bytes on
both machines and only the reporting differs, so on x86-64 the ledger was
dividing the runtime's stack by half a level's cost.

Every frame now carries the return address the report leaves out — eight bytes
on x86-64, none on arm64 — so a row is what one activation costs and the level
count divides by that. Over-promising depth is the dangerous direction for this
artifact: a writer who believes it writes a recursion the machine cannot run.

The rule arrived as a `cfg!` inside the ledger, and the next Linux run showed
why that was the wrong shape: two cases in the ledger's own module read one
synthetic arm64 report and assert the rows it produces, and on an x86-64 host
the ledger was adding that host's eight bytes to that report's numbers. A pure
text-processing function had a hidden input. `stack_ledger` now takes the
`Architecture` its report describes: the fixture cases name `Arm64`, which is
what their fixture is, and the driver and the compile-and-run cases name
`Architecture::HOST`, which `backend/target.rs` already establishes is the only
architecture this compiler emits for.

The end-to-end case still runs, and beside it `a_row_is_what_one_activation_costs`
states the arithmetic from a synthetic report without compiling anything —
both architectures at once, from either machine — pinning the eight bytes in
the frame row, the cycle row and the division alike. A second case pins
`Architecture::HOST` against `std::env::consts::ARCH` rather than repeating the
ledger's own `cfg!`, so a ledger that named the wrong architecture cannot
satisfy itself.

**No link named the math library.** With the compiler's own tests green, the
run reached the program corpus, and five of its cases did not link, across
three programs. A
Whitefoot module reaches libm without asking for it: the backend lowers a
rounding to `roundevenf` and a fused multiply-add to `fma`, and the host
optimizer turns other float arithmetic into `ceil` and `floor`.
`grayscale_pixels`, `feedback_controller` and `par_layout` each end with one of
those undefined. Darwin serves them from the library every program already
links, and says nothing; an ELF host keeps them in libm and the link fails.
This is the shipped driver's own link path, not a test harness: `whitefootc`
on that host would have failed the same way on the same programs.
`HOST_LINK_LIBRARIES` now names the library once, beside
`HOST_OPTIMIZATION_ARGUMENTS` and for the same one-definition reason, and the
driver and both test link paths pass it. On Darwin it resolves to a stub that
is already linked, so this stays one link path rather than two.

The third of those is the same shape as batch 0085's `bridge.c` finding — code
written for a platform, shipped, and never once put through a compiler for
that platform. It is the standing argument for this batch.

### Tests that were measuring the host

**`system_io` rejected the wrong symbol.** The transfer-path case asserted
that no `@wf.sys.` symbol survives a call in the optimized entry. What it means
is that no *approved implementation* survives — the qualification condition
[QUAL-3]. The [SYS-7] error-class mapper `@wf.sys.io.error` is a pure function
on the failure arms and carries no transfer, and whether the optimizer leaves
it as a call or outlines it into a `.cold.` region is that optimizer's choice:
clang 21 on Darwin outlines it, clang 18 on Linux does not. The assertion now
names it, so it is about the transfer path rather than about one clang.

**The overlapped world was bounded by a multiple of one host's register
allocator.** `the_shipped_default_keeps_a_deep_recursion` asked that one
activation of the world an unconfigured `--par` binary runs in cost no more
than twice one activation of the sequential clone beside it. The lowering
costs the same on both architectures and the baseline does not: 48 bytes an
overlapped level on each, against 32 a sequential level on arm64 and 16 on
x86-64, so the identical lowering reads as a ratio of 1.5 on one host and 3 on
the other. The bound of two was a fact about which values the arm64 allocator
had to spill, not about the hand-out. It is now the overhead itself — six
machine words, against a measured two on arm64 and four on x86-64 — which is a
property of what the claim keeps live across `wf__par_claim`. The mechanism
the case guards, that the claim's record belongs to the lane so a refused
hand-out builds nothing, is held exactly and with no tolerance by
`handing_a_call_out_adds_no_stack_slot`.

**A grant observation was sampling the runner's schedule.** Two cases assert
that the parallel runtime is granted at least one lane — the existential claim
without which every other parallel case would pass just as well against a
runtime that granted nothing. A grant is a steal, and a steal is a scheduling
event: the offering thread can finish the work itself before any pool thread
reaches the offer, and on a busy machine it often does. Five runs were enough
on this machine and not on the runners: the three-core macOS runner totalled
zero over the default pool's five runs in one gate run and was granted on the
first run of the next, and the four-core Linux runner lost the four-worker one
the same way. Both now sample `GRANT_OBSERVATION_RUNS` — thirty-two runs of one
linked module, which costs a fraction of a second. A runtime that grants
nothing still totals zero over all of them, so what the assertion refuses is
unchanged.

**The stackless migration observation was a rare race sampled as if it were
not.** The migration half of that case asks a scheduler worker to claim a ready
frame inside a window one call wide, before the writer thread resumes it
itself. On this machine, with ten cores, it happens within a run or two, and
the sample was sixteen runs; the batch raised it to ninety-six when a Linux
runner reached zero. That was still the wrong order of magnitude. Measured
across this batch's gate runs, both runners reached one migration in some runs
and none across a whole ninety-six-run sample in others, which puts the rate
near one attempt in a hundred and made ninety-six attempts a coin flip — every
lost flip a red gate reporting a scheduler defect that was not there. The
sample is now a thousand and twenty-four attempts, which miss a one-in-a-
hundred event about once in a thousand gate runs. The loop stops at the first
migration, so the cost is paid only where the event is rare: 16 milliseconds a
run here, so a host that sees one immediately spends nothing and a host that
never does spends about twenty seconds.

**The wfgrep cost census had a hole, and the second macOS toolchain exposed
it.** The `macos-14` runner ships Xcode 15.4; this machine has Apple clang 21.
On the runner the census saw `@exit` and had no row for it. It was never
absent from the program: `wf__main_body`'s `start.failure` arm exits with the
fixed start-failure status when the host refuses the initial working
directory. The newer clang outlines that arm into `@wf__main_body.cold.5`,
whose body the census explicitly does not scan; the older one leaves the call
where the census reads it. Verified directly by emitting `wfgrep.wf` and
running it through this machine's clang: the call is there, inside the cold
outline. The census now accounts for it on the same terms as
`wf_resource_abort` — `noreturn`, on no success path — and asserts the
`noreturn` declaration rather than arguing it.

**The offset-fault case assumed the memory below a stack was empty, and once
in ten Linux runs it was not.**
`only_a_fault_within_the_probe_stride_is_read_as_an_exhausted_stack` faults at
chosen distances below the floor's entry stack and requires the two in-band
rows to end in the floor's abort with the stack record, and the three
past-the-stride rows — four pages, 64 KiB, 16 MiB — to keep `SIGSEGV` with no
record. Nine Linux gates passed it; the gate of `7ec7bc1a`
([33137459268](https://github.com/mbbill/Whitefoot/actions/runs/33137459268))
did not: the four-page row exited 0 — no signal, no record. Nothing faulted,
so the floor never ran; the write four pages below the stack found a writable
page. The fixture had assumed the memory below a stack is empty, and the
floor's own layout says otherwise: `wf__floor_attach_thread` maps a 64 KiB
read-write alternate signal stack *after* the entry stack, and the kernel's
top-down search drops it into the first gap below the stack block.
`/proc/self/maps` from the same fixture linked against the same floor, in this
project's Linux container (glibc 2.39, the runner's), shows it:

```text
f1d709640000-f1d709650000 rw-p    the entry thread's alternate signal stack
f1d709650000-f1d709660000 ---p    the guard glibc puts under the stack
f1d709660000-f1d749660000 rw-p    the 1 GiB entry stack
```

The container never lost the row because its guard is 64 KiB, so four pages
below the stack is still inside the guard; x86-64 glibc's guard is one page,
so there the write lands in the alternate stack in every run where the kernel
has placed it under the stack rather than in a hole higher up, which the
runner's ten gates that reached the row did once. That is not a floor defect
— the floor classifies faults, and no fault happened — and not a host limit
to declare, because the premise was never the host's to keep. The fixture now
keeps it: the fault comes from a thread the fixture creates on a stack it
allocates at the top of one reservation, attached to the floor exactly as a
pool lane is, and the thread unmaps the 16 MiB below its stack after
attaching and reading its bounds — both map memory, and the same search would
put either into a fresh hole — and just before it writes. The pad is unmapped
rather than left `PROT_NONE` because Darwin reports a protected page as
`SIGBUS` and an unmapped one as `SIGSEGV`, and the rows assert the host's own
signal for a pointer into nothing. The assertions are byte-for-byte what they
were. Verified by running the shipped fixture text against the shipped floor:
in the Linux container, twenty of twenty runs on every row — abort and record
at half a page and one page below, `SIGSEGV` and no record at four pages,
64 KiB and 16 MiB; on this machine, whose page is 16 KiB, the same at 8 KiB
and 16 KiB against 64 KiB and 16 MiB.

**The latch control was a race sampled at one host's rate.**
`the_latch_is_what_keeps_the_record_single` defeats the trap latch with a
one-token injection and requires that at least one of eight runs show both
racing threads writing a record before the winner's `abort` takes the process
down. On this machine, ten cores, it is caught 200 of 200 runs. On the
three-core `macos-14` runner it is a race the second thread often loses, and
the gate of `25ac56ef`
([33142388164](https://github.com/mbbill/Whitefoot/actions/runs/33142388164))
caught none in eight, after twelve gates that had caught at least one — a
rate near one run in four. The record's not-done list had named this class
as one no run had lost; one now had. The same move as the migration
observation: the first eight runs still always happen, every caught one
checked in full, and after them the loop stops at the first catch or at 512
attempts. At one in four that misses about never; at one in fifty, about once
in thirty thousand gates; a host that never catches spends about ten seconds,
at twenty milliseconds an attempt, before failing honestly. Every assertion
is what it was. Batch 0093, the gate time budget, is restructuring the
process-spawning sampling cases in `trap_latch`, `parallel`, `stackless` and
`exhaustion` with red/green verification; this resizing is the minimal one and
is handed to that batch, whose shape supersedes it.

### Host limits, now declared

Each of these is a case that cannot be reached on a host, stated as a
precondition rather than left to fail. None deletes an assertion; where the
host can reach the case, the assertion is exactly what it was.

| what | where | the limit, and why |
|---|---|---|
| the whole §9.1 cost census | `#[cfg(target_os = "macos")] mod cost_shape;` in `backend/tests.rs` | every case compiles `wfgrep`, which walks directories. Linux has no approved [SYS-14] enumeration row: `getdents64` writes no per-entry name length and the portable record the emitted shim fills needs one, so `backend/qualification.rs` reports `MissingMapping(Operation(12))` rather than pretending the facility is there. There is no `wfgrep` module on Linux to take a census of. |
| `directory_source_open_uses_the_typed_completion_route` | `backend/tests/completion.rs` | the same row, and the same `#[cfg]` its two enumeration siblings already carried |
| the four- and eight-lane steal observations | `a_steal_is_observable` in `backend/tests/parallel.rs`, used by `parallel.rs` and by three sites in `loop_split.rs` | a steal is observable only if a worker reaches the offer before the offering thread finishes the work itself, which needs a core that is not already carrying a lane. Measured: zero over the whole sample on the three-core `macos-14` runner, non-zero on every four-core host. A zero there is a fact about the host. Two eight-lane sites in `loop_split.rs` were left without the guard their sibling in the same file carries, and the macOS runner failed one of them; they carry it now. |
| the alias-versioning calibration | `research/experiments/frequency-study/alias-versioning/` | the recorded fingerprint counts LLVM's own runtime alias-check shape — 26 conflict predicates, 52 pointer comparisons — and the vectorizer decides that shape per target. Discriminated rather than assumed: rustc 1.96.0, 1.97.1 and 1.98.0 all reproduce it exactly on `aarch64-apple-darwin`, so the precondition recorded is the target, not the version. |
| the directory-walking program cases | `mod wfgrep;` in `tests/programs.rs`, five of the eight cases in `tests/programs/traversal.rs`, and the three support helpers only they call | the same missing enumeration row: `dir_walk.wf` and `wfgrep.wf` do not compile on Linux, so seventeen cases — the twelve in `wfgrep` and the five in `traversal` that build `dir_walk.wf` — were reporting the target table's gap as a test failure. The three traversal cases that reject inline source at a numbered rule, `an_enumeration_handle_is_not_usable_after_it_is_moved`, `program_bytes_still_cannot_become_a_path_value` and `an_enumeration_match_that_omits_an_outcome_is_rejected`, run on both hosts, because every stage reaches a source rule before target qualification. `b58d724d` shipped this with a second `#[cfg]` on `mod traversal;` itself, which hid all eight: the Linux gate of `196525e7` ran 34 program cases against macOS's 54, twenty apart, and this record said seventeen. An independent reading of the run logs caught it; the module-level attribute is gone, the five per-case attributes stay, and the run under Results shows the Linux count with the three restored. The corpus-wide `--par` case keeps every other program covered on Linux instead of standing down: it reads the compiler's own `TargetQualification` report, records which units it could not compile, and asserts that they are only the two that walk a directory. |
| the effect-attrs IR-validity probe | `test_multiline_probe_is_valid_llvm` in `research/experiments/frequency-study/effect-attrs/tests/` | every fixture there is written in the `memory(...)` attribute dialect, which is the attribute the experiment classifies and which LLVM 16 introduced. The `macos-14` runner's Apple clang 15 rejects them at the parser with "expected top-level entity", which says nothing about whether they are well formed. The probe asks the host compiler with a one-line `memory(none)` module and states the limit when the answer is no, so what it skips on is what the toolchain understands rather than a version string. |

### The gap that stays open

Five conformance cases reach `TargetQualification(MissingMapping(Operation(12)))`
on Linux and therefore fail:

```text
sys14-list-outcome-exhaustive
sys14-list-zero-range
sys14-directory-release
sys14-entry-kind-closed
accept-sysfile-two-permits-shared-directory
```

Batch 0085 named the first four. The fifth is new here and is an `accept`
case rather than a `run` case; 0085's count of four was one short.

Nothing in this batch changes them. The corpus is compiler-independent and
host-independent by design: it has a `pending`/`xfail` axis for what *this
toolchain* cannot reach, and no axis for what one target cannot reach, because
a case that passes on Darwin and not on Linux is not a property of the corpus.
Giving it one would be a change to conformance evidence made to turn a job
green, which is exactly the move the project forbids. `gate-linux` therefore
ends red at `conformance-run`, on five named cases, all of them the one
documented target-qualification gap, and it stays red until Linux gains a
directory-enumeration row. That is a real gap in the compiler, honestly
reported, and the job's value is everything it checks before reaching it.

## Results

### The runs this record stands on

Both workflows run on every push. These are the runs of `196525e7`, the last
commit of the batch's first Linux picture that changes code, and of every
code-changing commit after it, oldest first:

| run | job | host | outcome |
|---|---|---|---|
| [gate 33133768976](https://github.com/mbbill/Whitefoot/actions/runs/33133768976) | `gate-linux` | ubuntu-24.04, x86-64, 4 CPUs, clang 18.1.3, stable 1.98.0 | red, and red only where this record says it is: `== WHITEFOOT COMPILER GATE GREEN ==`, 1320 library cases, 34 program cases and every research suite pass, and `conformance-run` then reports `Pass=497  Fail=5  Skip=1` on the five named cases |
| | `gate-macos` | macos-14, arm64, 3 CPUs, Apple clang 15.0.0, stable 1.96.0 | green |
| [io-hosts 33133768971](https://github.com/mbbill/Whitefoot/actions/runs/33133768971) | `completion-linux` | ubuntu-24.04 | green |
| | `bench-linux` | ubuntu-24.04, AMD EPYC 7763, `/dev/sda1` ext4 | green: N.direct 101.57, S.wide 123.15, C.wide.default 130.73 milliseconds, so C.wide 6 percent slower than S.wide. An earlier revision of this row put 119.31, 141.91 and 147.85 here; those are the numbers of [io-hosts 33131919667](https://github.com/mbbill/Whitefoot/actions/runs/33131919667) on `e7720a0a`, an EPYC 9V74 runner, and the table under *The Linux-hardware bench* now names every run beside its own numbers |
| | `completion-windows` | windows-2025 | green |
| [gate 33137459268](https://github.com/mbbill/Whitefoot/actions/runs/33137459268) on `7ec7bc1a` | `gate-linux` | ubuntu-24.04, x86-64, 4 CPUs | red inside the library suite, where "red only on the five cases" did not hold: 1319 library cases pass and `only_a_fault_within_the_probe_stride_is_read_as_an_exhausted_stack` fails on its four-page row with exit 0 — the finding under *Tests that were measuring the host* — so neither `tests/programs.rs` nor the conformance run was reached; the library suite took 980 s |
| | `gate-macos` | macos-14 | green |
| [io-hosts 33137459242](https://github.com/mbbill/Whitefoot/actions/runs/33137459242) on `7ec7bc1a` | `completion-linux` | ubuntu-24.04 | green |
| | `bench-linux` | ubuntu-24.04, AMD EPYC 9V74, `/dev/sda1` ext4 | green: N.direct 118.79, S.wide 140.86, C.wide.default 148.71 milliseconds, C.wide 5.6 percent slower than S.wide |
| | `completion-windows` | windows-2025 | green |
| [gate 33142388164](https://github.com/mbbill/Whitefoot/actions/runs/33142388164) on `25ac56ef` | `gate-linux` | ubuntu-24.04, x86-64, 4 CPUs | red, and red only where this record says it is: `== WHITEFOOT COMPILER GATE GREEN ==`, 1320 library cases in 1105.59 s with `only_a_fault_within_the_probe_stride_is_read_as_an_exhausted_stack` green on x86-64, 37 program cases — `an_enumeration_handle_is_not_usable_after_it_is_moved`, `an_enumeration_match_that_omits_an_outcome_is_rejected` and `program_bytes_still_cannot_become_a_path_value` among them, the three `7ec7bc1a` restored — and every research suite pass, and `conformance-run` then reports `Pass=497  Fail=5  Skip=1` on the five named cases |
| | `gate-macos` | macos-14, arm64, 3 CPUs | red on one case, `trap_latch::the_latch_is_what_keeps_the_record_single`, with 1331 of 1332 library cases passing: the three-core sampling limit under *Tests that were measuring the host*. The resizing in this record's head is the minimal one; batch 0093, the gate time budget, is restructuring the process-spawning sampling cases in `trap_latch`, `parallel`, `stackless` and `exhaustion` with red/green verification, and this case is handed to it |
| [io-hosts 33142388146](https://github.com/mbbill/Whitefoot/actions/runs/33142388146) on `25ac56ef` | `completion-linux` | ubuntu-24.04 | green |
| | `bench-linux` | ubuntu-24.04, AMD EPYC 7763, `/dev/sda1` ext4 | green: N.direct 102.62, S.wide 124.62, C.wide.default 128.94 milliseconds, C.wide 3.5 percent slower than S.wide |
| | `completion-windows` | windows-2025 | green |

`make check` on the maintainer's machine — macOS on arm64, ten cores, Apple
clang 21, stable 1.97.1 — is green on the same tree, which is the third host
and the only one where the whole gate passes.

The head of the branch carries the latch-control resizing and this record's
final revision. Its own runs are not named here: the batch's CI iteration
closed on the runs of `25ac56ef`, and a further round on runner sampling would
collide with batch 0093.

### Linux completion I/O, on a real kernel

`completion-linux`, ubuntu-24.04, kernel `6.17.0-1022-azure`, x86-64, 4 CPUs,
`kernel.io_uring_disabled=0`, clang 18.1.3:

```text
native adapter probe                  target=linux-io-uring status=pass
completion harness, helpers 0/1/4     PASS
harness, WF_REQUIRE_LINUX_IO_URING=1  PASS at helpers 0, 1 and 4
completion-sanitize (ASan + UBSan)    PASS
completion-core-read-tsan             PASS
core/read bridge isolation            PASS
pure-compute link boundary            PASS
```

Batch 0085 had to fall back to gcc for the Linux sanitizers, because the bench
image carried clang 18 without `libclang_rt` and had no working package
mirror. On the runner `libclang-rt-18-dev` installs, so every line above is
clang, which is the compiler `whitefootc` itself invokes.

### Windows IOCP, executed

`completion-windows`, Windows Server 2025 Datacenter build 10.0.26100, x64,
LLVM clang 20.1.8 targeting `x86_64-pc-windows-msvc`:

```text
windows-native-completion-probe status=pass
native-adapter-probe target=windows-iocp status=pass
```

This is the first execution of the Windows completion path anywhere. It closes
exactly one of the two reasons Windows qualification was fail-closed — no
Windows host existed. The other reason is unchanged: the IOCP wake packet is
neither coalesced nor persistent for every already-announced waiter.
Qualification stays fail-closed and `implemented` does not move.

### The Linux-hardware bench

`bench-linux`, three separately provisioned ubuntu-24.04 runners, AMD EPYC
9V74, 4 CPUs, kernel 6.17, tree on the runner's own `ext4` local disk, medians
of nine after two warm-ups. The full tables are in
`research/investigations/io-model/RESULTS.md`; the shape is:

```text
                     run 1     run 2     run 3
N.direct            119.69     94.24     94.68
N.pool4              34.04     26.61     27.14     best N
N.uring32           118.69     94.45     94.75
S.wide              142.15    112.07    112.19
C.wide.default      149.47    115.92    118.14
```

`bench-linux` runs on every push, and each run draws its own runner, so the
branch has thirteen readings on three CPU models. Every one of them, beside the
commit it ran on, the host and disk the job itself reported, and the three
lines the finding turns on, medians in milliseconds:

| run | commit | host, disk | N.direct | S.wide | C.wide.default | C.wide against S.wide |
|---|---|---|---|---|---|---|
| [33114336424](https://github.com/mbbill/Whitefoot/actions/runs/33114336424), run 1 above | `7a1c73a5` | EPYC 9V74, `sda1` | 119.69 | 142.15 | 149.47 | +5.1% |
| [33115297530](https://github.com/mbbill/Whitefoot/actions/runs/33115297530), run 2 above | `804ed782` | EPYC 9V74, `nvme0n1p1` | 94.24 | 112.07 | 115.92 | +3.4% |
| [33118248259](https://github.com/mbbill/Whitefoot/actions/runs/33118248259), run 3 above | `2c342009` | EPYC 9V74, `nvme0n1p1` | 94.68 | 112.19 | 118.14 | +5.3% |
| [33121457101](https://github.com/mbbill/Whitefoot/actions/runs/33121457101) | `79aa36ea` | EPYC 9V74, `sda1` | 119.24 | 142.00 | 147.66 | +4.0% |
| [33127604146](https://github.com/mbbill/Whitefoot/actions/runs/33127604146) | `9adf0067` | EPYC 7763, `sda1` | 103.37 | 122.49 | 128.22 | +4.7% |
| [33128887536](https://github.com/mbbill/Whitefoot/actions/runs/33128887536) | `7c644216` | EPYC 7763, `sda1` | 103.51 | 124.76 | 128.72 | +3.2% |
| [33131534867](https://github.com/mbbill/Whitefoot/actions/runs/33131534867) | `75ce03d4` | EPYC 9V74, `sda1` | 118.84 | 141.49 | 147.14 | +4.0% |
| [33131919667](https://github.com/mbbill/Whitefoot/actions/runs/33131919667) | `e7720a0a` | EPYC 9V74, `sda1` | 119.31 | 141.91 | 147.85 | +4.2% |
| [33133174447](https://github.com/mbbill/Whitefoot/actions/runs/33133174447) | `6fc4c71b` | EPYC 9V74, `sda1` | 119.41 | 141.73 | 147.00 | +3.7% |
| [33133768971](https://github.com/mbbill/Whitefoot/actions/runs/33133768971) | `196525e7` | EPYC 7763, `sda1` | 101.57 | 123.15 | 130.73 | +6.2% |
| [33135242838](https://github.com/mbbill/Whitefoot/actions/runs/33135242838) | `db7d997b` | Xeon Platinum 8573C, `nvme0n1p1` | 77.25 | 95.32 | 101.49 | +6.5% |
| [33137459242](https://github.com/mbbill/Whitefoot/actions/runs/33137459242) | `7ec7bc1a` | EPYC 9V74, `sda1` | 118.79 | 140.86 | 148.71 | +5.6% |
| [33142388146](https://github.com/mbbill/Whitefoot/actions/runs/33142388146) | `25ac56ef` | EPYC 7763, `sda1` | 102.62 | 124.62 | 128.94 | +3.5% |

The completion build loses to the sequential build on all thirteen, by 3 to 7
percent, on three CPU models and both disk kinds. The io_uring reading is not
as portable, and the tabulated runners are the ones on which it holds: on the
EPYC 9V74 the ring equals the loop at every depth; on the Xeon it is within 8
percent of it (N.uring32 83.18 against N.direct 77.25); on all four EPYC 7763
runs the ring at depth 4 and above sits at 125 to 128 ms against a 102 to 104
ms loop, a quarter slower, while depth 2 is nearly equal (105 to 107). Why
that CPU pays for a deeper ring is not settled here.

Three findings, all reproduced on the three tabulated runners.

**The hand-written io_uring baseline equals the blocking loop at every depth
from 2 to 32.** The whole 68 MiB tree is in the page cache, so a `pread` never
sleeps and there is no latency for a ring to hide.

**On the real runner, C.wide is 3 to 5 percent slower than S.wide.** By 3
percent in run 2 and 5 percent in runs 1 and 3, against a within-run spread of
about 2 percent, at every helper count and on both the four-wide and the
eight-wide program. The completion build loses to the sequential build here.
This is the first host on which the completion lowering costs more than it
returns, and the first on which the standing bar's first half — C at least as
fast as S — is missed.

**The container's advantage was a wait the runner does not have.** Wall time
against child CPU time separates the two hosts cleanly:

```text
                        wall     user+sys    CPU/wall
runner  N.direct       94.24       94.08       1.00
        S.wide        112.07      111.61       1.00
        C.wide        115.92      115.82       1.00
container N.direct     72.04       72.28       1.00
        S.wide        337.29      221.98       0.66
        C.wide        146.73      132.67       0.90
```

On the runner every line is CPU-saturated. In the container the C baseline is
too, but Whitefoot's own sequential build is not: 115 ms of its 337 ms is time
the process was not running, and the completion build recovers most of it.
Overlap was repaying a wait that the container has and the runner does not.
What produces that wait is not settled here — the container's virtualized
block path and a two-vCPU guest running a lane pool plus a writer are both
consistent with the numbers, and discriminating them wants a run with the pool
disabled on both hosts, which this batch did not do.

The consequence for the design is not that overlap is worthless. It is that
the value of the completion lowering is a property of the host's I/O latency,
and this repository has now measured a host where that latency is zero. The
batch-0084 section already listed "any workload whose operations genuinely
wait" among what its numbers do not cover; this is the other side of that
sentence, measured.

## Approval classes

- **No specification change.** `spec/kernel-spec.md` is untouched; the active
  identity remains v0.37.
- **No conformance change.** No case, manifest line, adapter, runner, or
  collection wiring is added, modified, deleted, or renamed. The five failing
  Linux cases are reported, not annotated.
- **New root entry:** `.github/`, justified above and named in the repository
  map in `README.md`.
- Ordinary compiler, test, research, and documentation changes otherwise.

## Judgment calls

1. **No `rust-toolchain.toml`.** The gate's lint result turned out to be a
   function of the clippy version: `macos-14` ships stable 1.96.0, the Linux
   image ships 1.98.0, and this machine runs 1.97.1. Each rejected something
   the others accept. Pinning would have made CI reproducible at the cost of
   freezing the maintainer's compiler, so instead the four flagged sites are
   written the way all three accept — verified under each — and every job
   prints its own `cargo clippy --version` so a future red is attributable in
   one line rather than one run.
2. **`gate-macos` included.** The local machine already covers macOS, so this
   job is optional by charter. It was kept because it is a second macOS
   toolchain, and it immediately earned its place: the `@exit` census hole and
   both steal observations are things only a different macOS host could show.
3. **The stackless migration observation samples wider, and the link moved out
   of its loop.** It was sixteen link-and-run attempts; it is now one link and
   ninety-six runs. That is cheaper and a wider sample, and it exposed
   something the old shape hid: on the three-core macOS runner the observation
   fails either way, and it passed there before only because a fresh link
   before each run widened the window. Recorded as a limit rather than papered
   over.
4. **The five conformance failures are left red.** Reasoned above. The
   alternative — a host axis in a compiler-independent corpus — is a change to
   conformance evidence whose only purpose would be a green badge.
5. **`linux-bench.sh` was parameterized rather than copied.** `ROOT`, `OUT`
   and `CLANG` now come from the environment with the container's paths as
   defaults, so the container run and the native runner run execute the same
   bytes. A second native script would have been a second protocol.
6. **The math library is named on both hosts, not on the one that needs it.**
   A `cfg` would have made the link path a pair of paths differing by target,
   for a flag Darwin resolves to a stub it already links. One link line is
   worth more than the flag it saves.
7. **The overlapped world's bound is bytes, not a multiple.** Raising the
   multiple until x86-64 passed would have set the bar at the number the host
   produces, which is how a case stops discriminating. What the hand-out keeps
   live across the claim is a fixed, small set of values, so the overhead is
   the quantity with a meaning, and it is the same quantity on both
   architectures.
8. **The corpus-wide `--par` case reads the compiler's report instead of
   standing down.** The blunt move was `#[cfg(target_os = "macos")]` over the
   whole case, which would have taken twenty-three programs' Linux link
   coverage away to excuse two. Reading `TargetQualification` and then
   asserting that the excused units are exactly the two that walk a directory
   keeps the coverage and keeps the exemption from spreading.
9. **The effect-attrs fixtures were not rewritten for the older clang.** The
   pre-LLVM-16 spelling would have deleted the experiment's subject: it
   classifies the `memory(...)` attribute. The probe declares what the host
   toolchain cannot read instead, and the assertion is unchanged on every host
   that can read it.
10. **The temporary Linux diagnostic step is gone with the picture it was for.**
    `make check` stops at the first failing target and cargo at the first
    failing test binary, so one run would otherwise have revealed one layer of
    Linux stops at a time; the step ran the whole suite again with
    `--no-fail-fast`, which is what made four rounds enough. It doubled the
    job — 35 minutes of `make check` and 17 more of diagnosis — and now that
    the only red left is the conformance gap, `make check`'s own output names
    it.
11. **The offset-fault case was neither excused nor loosened; its fixture now
    owns the premise it had been borrowing.** A `#[cfg]` would have said the
    band's boundary is unobservable on Linux, which is false, and a
    conditional assertion — if it faulted, then — would have let the row pass
    while testing nothing. The row's claim is about where the floor draws its
    band, and that claim needs a fault at a known address; a fixture that
    makes the memory below its own stack empty is the only way to have one on
    every layout.

## What this batch did not do

- It did not add a Linux directory-enumeration row. That is a real compiler
  capability, it needs a record model that does not assume a per-entry name
  length, and it is the single change that would make `gate-linux` green.
- It did not close Windows qualification. The wake-packet coalescing question
  is untouched.
- It did not explain the container's non-CPU-bound sequential line. It
  established that the effect is the container's and not Linux's, and named
  the experiment that would separate the two candidates.
- It did not run the pipe workload or the macOS bench in CI. The local machine
  covers macOS, and the pipe workload discriminated nothing in batch 0084.
- It did not make the scheduler observations deterministic. The grant,
  migration and latch-control cases still sample a race; what changed is that
  the samples are now sized from the rate the runners actually show rather
  than from the rate this machine shows. Two of the same class were left as
  they are because no run of this batch lost them: the corpus case
  `the_claim_bearing_fold_is_granted_lanes_and_publishes_the_same_bytes`,
  whose five attempts each pay for their own link, and the join-less
  comparison in `backend/tests/parallel.rs`, which asks twelve runs to
  disagree. A host slower than any measured here can still lose either — the
  three-core macOS runner lost the latch control once, at a sample this
  machine had never lost, before it was resized.
- It did not touch the five cases that own most of the library suite's time
  on the four-CPU runner, where the suite takes 980 s against about 90 s on
  this machine. This is the input to a follow-up on test economy, so the next
  batch does not rediscover it. Measured on the gate of `7ec7bc1a` as the gap
  between a case's result line and the one before it — under four test
  threads that is a lower bound on the case's own time, not the time itself:
  `parallel::the_repeat_reports_a_lowering_whose_joins_were_removed`, 412 s,
  one link and twelve process runs of a join-less lowering asked to disagree
  with the reference;
  `trap_latch::a_racing_pair_of_false_claims_writes_exactly_one_record`,
  210 s, forty runs of two threads racing to trap;
  `trap_latch::a_single_false_claim_reports_the_same_bytes_at_every_worker_count`,
  138 s, four worker counts by four runs;
  `trap_latch::the_sequential_schedule_names_one_claim_every_run`, 42 s, two
  sequential settings by eight runs. Those four are about 13 of the suite's
  16 minutes. The fifth,
  `exhaustion::a_frame_larger_than_the_guard_region_is_still_reported`, links
  and runs a recursion into a 1 GiB stack twice, probed and ablated; it shows
  no gap of its own because other cases finish around it, and it completed
  103 s after the suite began among the first cases started, so a minute or
  more of that is its own. Every one of the five samples schedules or
  exhausts a stack on purpose; what a follow-up has to decide is how many
  samples each purpose needs on a slow host, not whether to keep them. Batch
  0093, the gate time budget, is that follow-up.
- It did not move the floor's alternate signal stack. The map above shows it
  mapped read-write directly under the entry stack's guard, and on x86-64
  glibc that guard is one page. Generated frames probe their pages and cannot
  step over it, so nothing this batch measured is unsafe; a runtime or libc
  frame larger than a page, running near the bottom of the stack, would write
  into the alternate stack without a fault. Mapping the alternate stack
  before the entry stack, or with a guard of its own, is a small change a
  follow-up should weigh with that picture in front of it.
