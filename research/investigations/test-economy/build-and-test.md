# Build and test cost

This investigation makes local compiler iteration and CI verification practical
without losing the behavior each maintained check protects. It owns the current
measurements and reasons for changes to the build/test entry points; update it
when those costs or choices change. Retire it when a successor measurement
supersedes the comparison.

## Question and criterion

The interrupted scatter investigation spent 8 minutes building an optimized
compiler and over 21 minutes in a filtered Cargo test command without starting a
test. Part of the latter waited for a same-target build lock. Other-worktree
tests and extreme host load make those times unsuitable as a normal baseline.
The recovered 21m27s command includes an estimated 7m16s lock wait; its log
never reaches `Running unittests`. A saved process snapshot also shows another
worktree's optimized library tests, and recorded load averages reach roughly
200–240. This establishes overlapping work, not which agent owned it or the
fraction attributable to memory, thermal state, disk or scheduling. Starting
additional heavy commands in that state and leaving them without a bounded
stop were orchestration defects. The controls below address those defects;
the isolated measurements establish the normal construction cost separately.

Before changing build settings, classify elapsed and CPU time separately for:

- Rust compiler and test-executable construction, including profile and cache;
- Whitefoot frontend/proof/emission work and Clang/native linking;
- test case execution, including repeated schedules and child binaries;
- experimental measurement protocols and their data/cache preparation; and
- process/build-lock waiting and CI setup or queueing.

Compare the same source revision and workload on an otherwise idle host with
one heavy command at a time. A change is useful when it removes demonstrated
duplicate work, reduces an attributed cost, or removes a completed experiment
from the set of maintained executable tools while preserving its useful
evidence. Fewer assertions or weaker oracles are not a performance result.
Compare cold compiler artifacts separately from reused artifacts; retain the
exact commands, profile, exit status and measured limitations. Re-run only
affected evidence after an edit, then run the canonical full gate for the final
revision.

## Baseline conditions

- Base: `277a184473875e28da2e1aa9461c9cc99fc9d63f` (`main`).
- Host: MacBookPro18,3, 8 logical CPUs, 32 GiB RAM; macOS 26.6.2.
- Toolchain: rustc/cargo 1.98.1; Apple Clang 21.0.0.
- Fresh worktree, with the existing Cargo registry available; no prebuilt
  compiler target directory. A fresh target is not a flushed OS page cache.
- Initial limits: Cargo build jobs 2, Make jobs 2, Rust test threads 2.
- Measure with monotonic wall time and process-tree CPU/resource accounting.
  Record abnormal commands and diagnose their owned process tree; an
  infrastructure timeout is an incomplete check, never a language rejection.

The initial hypotheses are excessive independent local concurrency, repeated
whole-program setup, large proof-heavy cases mixed into the nominally fast
library suite, and unconditional experiment rebuilds. The container-family
check currently rebuilds 15 programs in three compilation modes from a shell
loop even when outputs exist. Completed research-tool tests and long IO
measurement workflows also require a purpose review. These are leads, not
assumed causes or authorization to drop necessary coverage.

## Local baseline, September 14

`/usr/bin/time -l` measures command elapsed/user/system time. A temporary
supervisor records owned descendants every 30 seconds and stops an overlong
command's process group. Commands below use `--locked --offline`; no
third-party dependency download is included. Measurements are one pass, not
confidence intervals. The compiler crate has no dependencies.

| Construction only | Profile | Wall s | User s | System s | Peak RSS GB |
|---|---|---:|---:|---:|---:|
| `cargo build --bin whitefootc` | gate | 43.34 | 73.29 | 1.22 | 1.20 |
| `cargo test --all-targets --no-run`, compiler above present | gate | 76.79 | 139.35 | 2.42 | 1.81 |
| `cargo build --bin whitefootc` | dev | 15.43 | 19.79 | 1.19 | 1.25 |
| `cargo test --all-targets --no-run`, dev compiler above present | test/debug | 26.73 | 37.53 | 3.27 | 1.90 |

The optimized compiler plus all optimized test executables therefore cost
120.13 seconds in this sequential construction experiment; the equivalent
unoptimized construction costs 42.16 seconds. This is not the time to run the
tests. `gate` inherits release optimization while retaining debug assertions
and overflow checks. No profile setting changes in this work.

The candidate's fresh `cargo test --profile gate --all-targets --no-run
--timings` costs 116.26 seconds. Cargo attributes its critical path to the
test-enabled library (114.63 seconds); the ordinary library takes 72.01
seconds while sharing the two-job construction window. Integration test links
take 0.40–4.19 seconds each. This is a different build order, not evidence of a
Rust compilation speedup. Cargo timings identify units, not rustc's internal
optimization passes. The observed minute-scale optimized build is normal;
the interrupted multi-worktree build and build-lock wait were not a cold-build
baseline.

For scale, the candidate has 130,777 lines in 139 non-test-named Rust files
and 82,626 lines in 117 test-named files under `compiler/src` and
`compiler/tests` (a path-based inventory, including comments/generated code,
not a compiler cost model). The compiler crate has no external crates to
build. A Cargo test-name filter selects execution only after the containing
test binary is compiled; it does not turn that large optimized unit into a
small construction task. Internal rustc pass-level attribution was not
measured, so neither source size nor Cargo unit timings establish which LLVM
optimization pass is expensive.

With both profiles constructed above but fresh experiment/native artifacts,
the unchanged `make check` passes in **1127.30 seconds (18m47.30s)**, using
1430.46 user seconds and 134.43 system seconds. Stage counters round to seconds:

| Canonical stage | Wall s | Work performed |
|---|---:|---|
| Repository invariants | 1 | Repository structure and rule consistency |
| Spec archive / prose integrity | 0 / 0 | Immutable archive and current authority checks |
| Design lint | 5 | Tree/amendment form and change boundary |
| Conformance structure/coverage | 1 | Python corpus inventory, not compiler execution |
| Compiler: format / lint | 1 / 11 | Formatting and dev-profile Clippy |
| Compiler: test partition | 0 | Build/list cached harnesses and prove collection coverage |
| Compiler: unit | 276 | 1,588 cases, including Whitefoot proofs and Clang/native runs |
| Compiler: sampling | 61 | 74 cases, including repeated native schedules |
| Compiler: corpus | 254 | 110 real-program cases and the other Cargo integration/bin targets |
| Compiler: docs / spec / completion | 6 / 1 / 5 | Rust docs, dev spec checker, native C runtime harnesses |
| Research tests | 202 | Experiment/reference construction and execution, mixed Rust/C/Python |
| Benchmark programs | 150 | Whitefoot/native construction and bounded correctness checks |
| Native conformance adapter | 134 | 803 pass, 1 expected compiler failure, 1 skip |
| Recorded-verdict snapshot | 19 | Recompile 484 sources; all verdicts unchanged |

This is a **prebuilt-Rust gate**, not an exact clean full-gate measurement.
Do not add every separate build measurement above and call it a cold gate:
the canonical gate does not build the profiles and artifacts in that order.

## What the slow work protects

| Work | Purpose and observed cost source | Disposition |
|---|---|---|
| Library proof tests | Source acceptance, proof roots and obligation metadata; some analyze whole real programs | Keep. A runtime output oracle cannot replace proof metadata assertions |
| `frozen_real_sources_retain_complete_proof_roots_without_counted_false_positives` | UTF-8, four-file DEFLATE bundle and wfgrep; checks retained derivations and counted-loop groups | Keep all bundles; compilation and assertion time can now be measured separately |
| Backend `cost_shape` tests | Shared wfgrep emission; checks initialization/allocation shape and output batching with a deterministic host observer | Keep; different claim from the corpus's filesystem behavior |
| Backend native links | Recompile the same immutable runtime units for hundreds of separate emitted programs | Reuse immutable objects within the test process; retain fresh modules, observers and macro-interposed builds |
| Schedule sampling | Repeated grant/join/exhaustion interleavings and ownership cleanup | Keep repetition; no shorter loop or removed schedule is claimed as a speedup |
| Program corpus | Real bytes, filesystem/TCP behavior, CLI arguments, compiler modes and worker routes | Keep the independent oracles and inputs |
| Conformance / snapshot | Normative verdicts and recorded compiler verdicts, respectively | Keep separate compilation; neither a timeout nor a cached unit-test verdict substitutes |
| Container families | 15 sources in default, `--par`, `--no-overlap`, plus instrumented allocation and ownership controls | Keep all 45 executions; rebuild only when source/compiler/invocation changes |
| IO/compute `programs-check` | Current benchmark source compilation, lowering assertions and bounded native reference checks | Keep. IO outputs must depend on compiler and Makefile, not just source |
| Frequency-study miners | Completed Rust source/IR opportunity pilot, with parser/miner self-tests | Reproduce with `historical-tool-tests`; does not exercise today's Whitefoot compiler |
| Default-floor runner and four Rust crates | Completed fixed-model trajectory protocol, frozen reference/harness self-tests | Same explicit reproduction target; current UTF-8/DEFLATE/program oracles stay |
| Full IO timing matrices | Storage/cache/concurrency measurements with native controls | Manual experiment; program compilation and platform correctness stay automatic |

The obsolete `base64-dedup.md` note is removed under its own retirement
condition: the native conformance adapter now belongs to `make check`. Its old
claim that removing a library execution removes all gate execution no longer
holds. This work still keeps those library assertions and the independent
adapter; it removes redundant immutable C construction, not their evidence.

## A compiler hotspot, not native execution

On the baseline gate compiler, `--emit-llvm` for
`io-completion-bench/programs/read_heavy_wide8.wf` takes **58.29 seconds**
(56.39 user, 0.52 system, 2.88 GB peak RSS). No Clang or generated executable
is invoked. A three-second sample after 20 seconds has 2,008 stacks under the
semantic flow checker; 1,229 leaf samples (61.2%) are in
`close_with_excluded_term`, with contradiction checks and proof/hash bookkeeping
also visible. This is a sampled window, not a whole-run percentage profile.

The same baseline compiles `tests/programs/wfgrep.wf` to LLVM in 39.49 seconds
(39.02 user, 0.43 system). The same source through the prebuilt debug compiler
takes 535.38 seconds: **13.56 times the elapsed analysis/emission cost**. The
generated LLVM files compare byte-for-byte equal; neither command includes Rust
construction, Clang or program execution. Proof
closure optimization is already being developed in [PR 65](https://github.com/mbbill/Whitefoot/pull/65);
this PR records attribution without overlapping that algorithm change.

## CI baseline and scope

These completed GitHub-hosted runs measured revision `70c86e5b`, based on the
same `main` but with scatter-attribution changes. They are direct CI log/API
observations, **not a matched before/after comparison with the local baseline**.
Job durations include setup/cleanup but exclude queueing; parallel jobs do not
add up to workflow latency. The unit jobs failed the then-current scatter IR
assertion; their completed elapsed times are not successful gate evidence.

| Gate job | Linux job s | macOS job s | Main-step construction / execution evidence |
|---|---:|---:|---|
| static | 78 | 100 | Dev Clippy/docs/spec plus native runtime builds and runs |
| unit | 437 | 327 | Gate test build 75 / 90 s; cases 339.23 / 218.28 s |
| sampling | 147 | 154 | Gate test build 56.92 / 92 s; cases 55.80 / 45.55 s |
| corpus | 379 | 427 | Gate build 57.46 / 73 s; program cases 295.33 / 327.67 s |
| conformance + snapshot | 130 | 154 | Adapter build 40.97 / 59.42 s; run 44.30 / 51.55 s; snapshot 22.20 / 24.18 s |
| research | 225 | 205 | Compiler build 32.98 / 40.40 s; then mixed experiment construction/execution |
| benchmark programs | 239 | 213 | Compiler build 40.65 / 42.97 s; then Whitefoot/native builds and checks |

Source: [gate run](https://github.com/mbbill/Whitefoot/actions/runs/34902635670).

| Other CI job | Job wall | What happens inside |
|---|---:|---|
| Compute benchmark, Linux | 267 s | Dependencies 28 s; images 54 s; verify 24 s; protocol 131 s |
| Compute benchmark, macOS | 224 s | Dependencies 19 s; images 67 s; verify 15 s; protocol 102 s |
| Compute regression, Linux | 336 s | Baseline compiler 42 s; deps 30 s; both image sets 60 s; verify 27 s; paired protocol 153 s |
| IO host correctness, Linux | 42 s | Native C harnesses and ASan/UBSan/TSAN routes |
| IO host correctness, Windows | 259 s | Default-dev compiler plus real IOCP 67 s, TCP 66 s, component buffers 26 s, worker probes 27 s |
| IO many-files/network, Linux | 343 s | File protocol with builds 288 s; network protocol 30 s |
| IO read-heavy, Linux | 30m52s | Read protocol, including builds, data/cache preparation and repeated reads |
| IO read + many-files, macOS | 32m21s | Read protocol then many-files protocol, detailed below |
| IO measurement, Windows | 526 s | Compiler/references plus full protocol 487 s |

Sources: [compute benchmark](https://github.com/mbbill/Whitefoot/actions/runs/34902635711),
[compute regression](https://github.com/mbbill/Whitefoot/actions/runs/34902644033),
[IO hosts](https://github.com/mbbill/Whitefoot/actions/runs/34902635693),
[IO benchmark](https://github.com/mbbill/Whitefoot/actions/runs/34902635675).

The completed macOS IO log separates a 62-second gate compiler build,
approximately 10m05s of remaining read-image construction/data/verification,
472s for the 64 KiB uncached table, 440s for the 4 KiB uncached table, 49s/14s
for the warm tables, and approximately 194s of many-files build/verification
followed by 76s of timed passes. It is a 3-CPU/7-GiB VM, not the local Mac.
The protocol uses 17 macOS or 20 Linux configurations, four tables, seven
recorded rounds plus two warmups: 612 or 720 calls over a 512-MiB dataset.
Each call makes 32,768 reads. The uncached tables are substantial real device
work, not a slow Cargo test. Dropping them from automatic pushes changes when
an experiment is requested, not the repetitions inside that experiment.

## Candidate local results

With fresh compiler and research artifacts, candidate `15cab3f5` completes the
canonical `make check` in **1353.25 seconds (22m33.25s)**, using 1812.16 user
seconds and 97.09 system seconds. Optional helper timing is enabled for this
attribution run. The later `d05ba1cb` changes dedicated-CI thread selection,
native dependency build limits and reproduction guidance, not this local gate's
compiler or test payload. The stage counters below round to seconds.

| Candidate cold stage | Wall s | Construction versus execution |
|---|---:|---|
| Invariants / archive / prose / design / corpus structure | 11 / 0 / 0 / 6 / 0 | Includes the new guard's cancellation and exclusion falsifiers |
| Compiler format / lint | 2 / 11 | Rust formatting and dev Clippy |
| Compiler partition | 79 | Cold optimized library-test construction, then collection checks |
| Compiler unit | 219 | 1,588 cases execute in 218.16 s |
| Compiler sampling | 33 | 74 cases execute in 32.64 s |
| Compiler corpus | 482 | 51.08 s Cargo construction; approximately 430 s in program/integration cases |
| Compiler docs / spec / completion | 9 / 20 / 6 | Rust docs, cold dev checker, C construction and execution |
| Research | 145 | Containers 142.19 s; proof-use-cost 1.56 s; ripgrep 0.21 s; DEFLATE oracle 0.31 s |
| Benchmark programs | 161 | Whitefoot/native construction and bounded checks, not full timing protocols |
| Native conformance adapter | 150 | 803 pass, 1 expected compiler failure, 1 skip; test wall 149.46 s |
| Recorded-verdict snapshot | 19 | 484 unchanged verdicts; test wall 19.57 s |

The baseline's prebuilt-Rust 18m47s and the candidate's cold 22m33s are different
artifact states; they do **not** establish an overall speedup or regression.
Unit and sampling execution improved in these single passes, but the corpus
was slower. The baseline and candidate `whitefootc` executables compare
byte-for-byte equal. An isolated prebuilt wfgrep corpus case takes 41.32 s on
baseline, 41.44 s on candidate with logging off, and 43.01 s with logging on.
The full-suite corpus slowdown is not reproduced by that isolated comparison;
its context/concurrency contribution remains unresolved. Do not attribute it
to a production compiler change or claim that logging explains it.

The helper trace identifies what to investigate rather than treating every
slow case as a large runtime loop:

| Observed helper work | Calls | Aggregate helper wall s | Interpretation |
|---|---:|---:|---|
| Unit semantic checks / assertions | 1368 / 1368 | 64.639 / 0.065 | Removing proof metadata assertions would save almost nothing |
| Unit Whitefoot compile / native build / native run | 221 / 390 / 330 | 55.631 / 52.815 / 123.799 | Repeated compile/link and launch work inside `cargo test` |
| Sampling native build / native run | 110 / 28 | 11.308 / 8.370 | Shared helpers only; custom schedule subprocesses are outside this trace |
| Corpus default compile / parallel compile | 53 / 41 | 578.527 / 185.872 | Whole-program proof and emission dominate the measured helpers |
| Corpus native build / native run | 97 / 101 | 13.862 / 55.522 | Small fraction of traced program work is Clang linking |

These rows overlap across threads and can nest; their sum is **not** suite
elapsed time. The frozen-real-source test spends 1.460 / 15.139 / 39.745 s
analyzing UTF-8 / DEFLATE / wfgrep. The corpus's fixed-run library source is
375 lines with a four-element runtime capacity, yet one compile takes
215.576 s in the cold full-suite context. That is compiler work, not an
unbounded generated-program workload.

Before the native launch probe, the criterion was a large first-launch-only
wall/CPU gap for an otherwise trivial executable. Three fresh `int main(void)
{ return 0; }` images take 1.29 / 0.29 / 0.28 s on first launch; each of their
two subsequent launches prints 0.00 s at `/usr/bin/time -p` resolution. User and
system time also print 0.00 s. This isolates host first-launch overhead from
test input/algorithm work; it does not identify the responsible host service.

With existing example artifacts, the unchanged container-family check takes
81.86 s and the candidate 1.89 s, a **97.7% reduction**. Both execute all 45
source/mode pairs and the observer checks. Changed-source rejection, a changed
compiler that deliberately fails, and all-three-mode invalidation are verified;
the IO program target also invokes the changed compiler rather than a stale
executable. Completed research instruments still pass through
`make historical-tool-tests` in 8.16 s with their Rust artifacts already built.

## Local performance protocols

The complete local compute protocol uses five kernels, every available
implementation and emitted width, five passes and `CALLS=5` warm calls plus
the separately recorded first call per process. These
commands run one at a time at `nice -n 10`, with `JOBS=2`,
`CARGO_BUILD_JOBS=2` and `RUST_TEST_THREADS=2`; benchmark worker widths are
unchanged. They measure workflow cost, not a scheduler improvement.
The retained cell lists contain 139 combinations across the five kernels:
695 timed process invocations and 4,170 recorded first/warm calls. This
explains why the protocol can take minutes after image construction has
finished.

| Compute command | Wall s | User s | System s | Work performed |
|---|---:|---:|---:|---|
| `make deps`, sources cached, fresh build outputs | 13.78 | 17.14 | 3.40 | Verify pinned sources; build oneTBB, Rayon and the Parlay availability probe |
| `make build`, optimized compiler already present | 9.45 | 7.00 | 1.83 | Whitefoot modules, native objects and five kernel images |
| `make verify` | 70.07 | 409.84 | 4.14 | Every form at every selected width against the independent kernel oracle |
| `make compare` | 229.62 | 1231.91 | 15.31 | Full requested timed passes, reduction and tables |

An initial fresh-source dependency fetch was stopped after 459.41 s with only
0.19 user and 0.08 system seconds: it was waiting in Git's oneTBB download,
before any build. The subsequent run uses existing clean checkouts verified at
the exact oneTBB and Parlay pins, with Cargo offline. The interrupted download
is not included in the construction times or reported as a successful fetch.

The local many-files sequence exposes another duplicate: `build` takes about
129 s, then `verify` takes about 142 s and recompiles the same Whitefoot images.
Those images depend directly on the phony `compiler` target. Before changing
this dependency, the criterion is that Cargo must still refresh the compiler
before image freshness is considered, unchanged images must skip construction,
and source/compiler/Makefile changes must invalidate default and sequential
images. Verification and timing keep the same programs, bytes and repetitions.

| Local IO command, before the many-files dependency fix | Wall s | User s | System s | Work performed |
|---|---:|---:|---:|---|
| Full read-heavy script, 7 rounds + 2 warmups | 1288.52 | 367.06 | 142.07 | Eight Whitefoot/native images, verification and all four storage tables |
| Many-files `make build` | 128.95 | 125.79 | 2.95 | Native tools and ten default/sequential Whitefoot images |
| Many-files `make verify` after that build | 141.51 | 128.37 | 4.91 | Repeated image construction, generated input and output checks |
| Many-files `make bench`, 7 rounds + 2 warmups | 200.86 | 152.28 | 67.28 | Repeated construction, verification and full requested native timing passes |

All commands pass. The read-heavy script's prebuilt Rust compiler is a no-op;
native references/data take about 4 s, and its eight Whitefoot/native builds
take 349 s (38–58 s each). The remaining time is output verification and
repeated reads. All four local table labels are confirmed before and after
measurement. Linux-only engines/sanitizers and native Windows routes are
measured on their real CI hosts, not emulated on this Mac.

After inserting the real compiler file between Cargo's phony refresh and the
image dependencies, an unchanged many-files `build` takes **0.22 s** and
`verify` takes **6.46 s**, versus 128.95 / 141.51 s above. Cargo still checks
the compiler first. The required rebuild after the Makefile edit takes
128.19 s, then the same images and output oracles are reused. Source,
compiler and Makefile invalidation apply to default, sequential, read-heavy
and checked images; the real-file dependency also orders a parallel Make
correctly when the compiler file does not yet exist.
The complete seven-round/two-warmup many-files protocol subsequently takes
70.62 s (27.18 user, 64.07 system), versus 200.86 s before the fix, a 64.8%
elapsed reduction. Its input generation, output checks and timing calls are
unchanged. This comparison measures the command's removed build work, not a
faster IO runtime.

## Candidate CI results

All 14 jobs of [gate run 34913902582](https://github.com/mbbill/Whitefoot/actions/runs/34913902582)
pass at `d05ba1cb`. Each VM uses its host-sized Rust test pool; local verification
defaults to two test threads. Job times include setup and are not CPU seconds.

| Gate job | Linux s | macOS s | Construction / execution inside the job, Linux then macOS |
|---|---:|---:|---|
| Static | 97 | 90 | Dev Clippy 22.64 / 20.43 s, docs 7.93 / 8.95 s, dev spec checker 19.04 / 22.28 s; C runtime harnesses also build and run |
| Unit | 273 | 335 | Gate test construction 90 / 126 s; 1,588 cases 148.80 / 190.44 s |
| Sampling | 152 | 149 | Gate test construction 100 / 121 s; 74 cases 25.86 / 12.92 s |
| Corpus | 428 | 351 | Gate construction 79 / 73 s; 110 real-program cases 318.54 / 258.69 s |
| Conformance and snapshot | 169 | 150 | Adapter construction 58.37 / 68 s; adapter execution 58.96 / 41.28 s; snapshot execution 28.76 / 22.87 s |
| Active research | 275 | 223 | Compiler construction 50.78 / 63 s; container construction and checks 202.45 / 139.02 s |
| Benchmark programs | 291 | 324 | Compiler construction 52.01 / 75 s; complete target 258.90 / 308.49 s, remainder mostly Whitefoot/native source construction |

These are successful current-branch readings, not controlled speed ratios
against the older scatter branch or different hosted machines.

| Other current-branch job | Wall s | Main components | Result |
|---|---:|---|---|
| Compute, Linux / macOS | 274 / 223 | Dependency builds 31 / 19; kernel images 55 / 65; verification 25 / 16; timed passes 131 / 103 | Pass |
| Compute regression, Linux | 354 | Merge-base compiler 45; dependencies 29; both arms 63; verification 27; timed passes 155 | Pass |
| IO host correctness, Linux / Windows | 72 / 259 | Native adapter/sanitizer probes; Windows includes Whitefoot compilation and IOCP/TCP execution | Pass |
| Manual IO file/network, Linux | 355 | File protocol 293; network protocol 30 | Pass |
| Manual read-heavy, Linux | 3995 | Read protocol 3965, including construction | Pass; one cache label refused |
| Manual read + many-files, macOS | 1146 | Read protocol 929; many-files protocol 199 | Pass |
| Manual IO, Windows | 463 | Native measurement protocol 418 | Pass |

Sources: [compute](https://github.com/mbbill/Whitefoot/actions/runs/34913902615),
[regression](https://github.com/mbbill/Whitefoot/actions/runs/34913903615),
[IO hosts](https://github.com/mbbill/Whitefoot/actions/runs/34913902627),
[manual IO](https://github.com/mbbill/Whitefoot/actions/runs/34912406944).
The first three use `d05ba1cb`; the deliberately dispatched IO matrix uses
`35227ab3`, before the later build-failure propagation and phase-label fixes.

Windows host correctness deliberately remains a separate platform check. Its
first real-program step includes a 46.92 s dev-profile Rust build and about
19 s of subsequent source/native construction and execution. The 68 s TCP
step compiles three images before exercising the two engines; its first echo
output appears about 58 s into the step, while refused-connection runs each
take about 4 s. These are not 68 seconds in Cargo or one long echo loop.
This work does not assume that a cold optimized Windows compiler would pay
for its extra construction in this smaller source set; that total would need
a matched profile comparison before changing the workflow's profile.

The unusually long Linux IO job spends 36.70 s building Rust, about 570 s on
read images/data/verification, 1068 s on the 64 KiB uncached table, 2239 s on
the 4 KiB uncached table, and 46 / 5 s on the warm tables. The 64 KiB probe
confirms uncached before the table but refuses that label afterward; a green
workflow does not turn that table into valid uncached performance evidence.
The 4 KiB uncached probes both pass. This is storage-protocol variability,
not an hour-long Rust build or Cargo test. Preserve the full requested protocol
and its honest labels, but do not run it automatically on unrelated pushes.

## Selected implementation and validation

The pending [verification amendment](../../../design/amendments/compiler-verification-cost.md)
records the material choices. The live tree and specification are unchanged.

- A shared guard owns one host-wide lock and one cancellable process group,
  defaults to two build jobs/two test threads, prints phase wall/user/system
  time and 30-second progress, and bounds a command at 30 minutes unless a
  longer protocol explicitly sets its limit. It reports infrastructure failure,
  never language rejection. Raw commands bypass it; guidance therefore names
  the wrapper as well as guarded targets. SIGKILL can leave a stale lock that
  must be inspected rather than silently stolen.
  Dedicated CI VMs retain a host-sized Rust test pool; the two-thread default
  is for interactive local work. Native oneTBB construction now respects the
  supplied `JOBS` limit rather than independently consuming every CPU.
- Test helpers optionally append per-phase TSV timing. They do not cache
  compiler verdicts, alter callbacks or claim to time every custom subprocess.
- Native object reuse and dependency-tracked container/IO images avoid
  unnecessary or stale construction while preserving cases and modes.
- Completed research instruments retain their tests under an explicit target;
  the active gate no longer downloads/caches their third-party crates in every
  CI job. Full IO timing matrices are dispatched deliberately.
- IO protocol compiler builds no longer pipe into `tail`, which masked Cargo
  failure under POSIX `sh` and could continue with an old executable. Injected
  Cargo failure exits at construction and preserves prior output in the read
  and many-files scripts; the network script passes the same check with a fake
  Linux port-range lookup. Read-heavy logs now label construction and verification
  phases before the four timing tables.

Candidate optimized construction, the cold canonical gate, incremental
invalidation, retained historical-tool reproduction and the CI runs above pass.
Guard falsifiers cover original exit status, nested ownership, competing
invocations, parallelism defaults, timeout, signal cancellation and orphan
cleanup on macOS and Linux. No specification, normative case, verdict,
schedule count or proof assertion is changed.

Conformance invocation wiring now passes through the guard. It retains the
same Cargo adapter command, ignored-case opt-in, full source inventory and
verdict interpretation. A wrapper deadline or child failure is a failed or
incomplete verification command, never a normative source rejection. This
distinction is the selection ground for the wiring change.
