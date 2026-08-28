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
except a container the maintainer started by hand, and this batch found three
defects in that half within one run. No existing directory owns it: a GitHub
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

## What the first run found

Five distinct stops, in three classes. None of them was visible from one host.

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

That third one is the same shape as batch 0085's `bridge.c` finding — code
written for a platform, shipped, and never once put through a compiler for
that platform. It is the standing argument for this batch.

### Tests that read one optimizer's choices as the rule

**`system_io` rejected the wrong symbol.** The transfer-path case asserted
that no `@wf.sys.` symbol survives a call in the optimized entry. What it means
is that no *approved implementation* survives — the qualification condition
[QUAL-3]. The [SYS-7] error-class mapper `@wf.sys.io.error` is a pure function
on the failure arms and carries no transfer, and whether the optimizer leaves
it as a call or outlines it into a `.cold.` region is that optimizer's choice:
clang 21 on Darwin outlines it, clang 18 on Linux does not. The assertion now
names it, so it is about the transfer path rather than about one clang.

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

### Host limits, now declared

Each of these is a case that cannot be reached on a host, stated as a
precondition rather than left to fail. None deletes an assertion; where the
host can reach the case, the assertion is exactly what it was.

| what | where | the limit, and why |
|---|---|---|
| the whole §9.1 cost census | `#[cfg(target_os = "macos")] mod cost_shape;` in `backend/tests.rs` | every case compiles `wfgrep`, which walks directories. Linux has no approved [SYS-14] enumeration row: `getdents64` writes no per-entry name length and the portable record the emitted shim fills needs one, so `backend/qualification.rs` reports `MissingMapping(Operation(12))` rather than pretending the facility is there. There is no `wfgrep` module on Linux to take a census of. |
| `directory_source_open_uses_the_typed_completion_route` | `backend/tests/completion.rs` | the same row, and the same `#[cfg]` its two enumeration siblings already carried |
| the four-lane steal observations | `a_steal_is_observable` in `backend/tests/parallel.rs`, used by `parallel.rs` and `loop_split.rs` | a steal is observable only if a worker reaches the offer before the offering thread finishes the work itself, which needs a core that is not already carrying a lane. Measured: zero over the whole sample on the three-core `macos-14` runner, non-zero on every four-core host. A zero there is a fact about the host. |
| the alias-versioning calibration | `research/experiments/frequency-study/alias-versioning/` | the recorded fingerprint counts LLVM's own runtime alias-check shape — 26 conflict predicates, 52 pointer comparisons — and the vectorizer decides that shape per target. Discriminated rather than assumed: rustc 1.96.0, 1.97.1 and 1.98.0 all reproduce it exactly on `aarch64-apple-darwin`, so the precondition recorded is the target, not the version. |

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

Three findings, all reproduced on all three runners.

**The hand-written io_uring baseline equals the blocking loop at every depth
from 2 to 32.** The whole 68 MiB tree is in the page cache, so a `pread` never
sleeps and there is no latency for a ring to hide.

**C is slower than S.** By 3 percent in run 2 and 5 percent in runs 1 and 3,
against a within-run spread of about 2 percent, at every helper count and on
both the four-wide and the eight-wide program. This is the first host on which
the completion lowering costs more than it returns, and it is the first host
on which the standing bar's first half — C at least as fast as S — is missed.

**The container was measuring something else.** Wall time against child CPU
time separates the two hosts cleanly:

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
