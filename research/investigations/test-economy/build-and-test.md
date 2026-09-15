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

## Selected implementation and remaining validation

The pending [verification amendment](../../../design/amendments/compiler-verification-cost.md)
records the material choices. The live tree and specification are unchanged.

- A shared guard owns one host-wide lock and one cancellable process group,
  defaults to two build jobs/two test threads, prints phase wall/user/system
  time and 30-second progress, and bounds a command at 30 minutes unless a
  longer protocol explicitly sets its limit. It reports infrastructure failure,
  never language rejection. Raw commands bypass it; guidance therefore names
  the wrapper as well as guarded targets. SIGKILL can leave a stale lock that
  must be inspected rather than silently stolen.
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
  and many-files scripts. Read-heavy logs now label construction and verification
  phases before the four timing tables.

Candidate optimized construction and guard tests have passed. Full candidate
gate, incremental invalidation checks, local protocols and current-branch CI
results will replace this validation status before completion.
