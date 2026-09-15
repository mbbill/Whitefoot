# Build and test cost

This is the process inventory and timing report for PR #66. It explains what
constructs Rust artifacts, what executes Rust tests, what compiles Whitefoot
sources, what links native images, and what executes those images. It owns the
measurement record and the reasons for the verification changes; update it or
retire it when a successor measurement supersedes these results.

## Reading the numbers

**There is no single operation called "build and test".** A typical backend test
has this call chain:

```text
cargo test --profile gate
  rustc: construct the Rust test executable, if stale
  libtest: execute a selected Rust test
    whitefoot::compile: read/check/prove/lower WF source and emit LLVM
    clang: compile LLVM and native support, then link a new executable
    generated executable: run inputs and report bytes/status
    Rust assertions: compare those observations with the oracle
```

The second step includes the work beneath it. Calling its entire duration
"Rust unit test execution" is correct; calling it time spent only in Rust
assertions, only in WF execution, or only in compilation is incorrect.
`whitefootc --emit-llvm` stops before Clang. `whitefootc -o program` also invokes
Clang. `program` executes the result. Rust `gate`/`dev` profiles configure the
Rust implementation of the compiler; they are **not WF optimization options**.

Unless stated otherwise, numbers are **elapsed seconds**, not CPU seconds.
The local host is a MacBookPro18,3, 8 logical CPUs, 32 GiB RAM, macOS 26.6.2,
Rust 1.98.1 and Apple Clang 21.0.0. Local builds/tests use two build jobs and
two test threads, one heavy command at a time. A fresh target is not a flushed
OS page cache. Cargo's registry is already available; offline builds include
no downloads. These are individual observations, not confidence intervals.

| Evidence | Revision and artifact state | What its numbers can answer |
|---|---|---|
| Isolated Rust construction | `277a1844`, fresh target for each profile; tests built after the compiler | Cold compiler/test construction, with Cargo unit timings |
| Complete cold gate | `15cab3f5`, fresh compiler/research artifacts, helper timing enabled | Actual cold entry-point cost: 1353.25 s |
| Complete warm gate | `e98c673a`, reusable Rust/native/experiment artifacts | Actual warm entry-point cost: 648.54 s |
| Delivered-code full gate | `f3858780`, Rust cached, IO checked images invalidated by the Makefile edit | 786.97 s, including 133.85 s in benchmark-program construction/checks |
| Detailed test attribution | `f3858780` compiler and test payload; methodology below | Individual test and compiler/native phase costs |
| Final-code automatic CI | `f3858780`, GitHub-hosted VMs, build jobs 2 and host-sized test pools | Each CI job's own construction, execution and setup |

These artifact states must not be merged into a claimed before/after speedup.
In particular, the full gate does not construct every profile in the order of
the isolated construction experiment.

## 1. Constructing the Rust compiler and test executables

All compiler targets belong to **one Cargo package, `whitefoot` 0.1.0**, defined
in `compiler/Cargo.toml`. It has no external crate dependencies: one library,
three binary targets, four integration-test targets and the `build.rs` custom
build target. Research Rust models
listed later are separate `rustc` invocations, not this package's unit tests.

| Construction command, run in `compiler/` | Profile | Wall | User CPU | System CPU | Executes test cases? |
|---|---|---:|---:|---:|---|
| `cargo build --bin whitefootc --profile gate --locked --offline --timings` | Optimized Rust, debug assertions and overflow checks retained | 43.34 | 73.29 | 1.22 | No |
| `cargo test --all-targets --no-run --profile gate --locked --offline --timings`, after that build | Same | 76.79 | 139.35 | 2.42 | No |
| `cargo build --bin whitefootc --locked --offline --timings` | `dev`, unoptimized Rust | 15.43 | 19.79 | 1.19 | No |
| `cargo test --all-targets --no-run --locked --offline --timings`, after dev build | Default test/debug | 26.73 | 37.53 | 3.27 | No |

The first two commands total **120.13 s to construct the compiler and all test
executables**, not to execute any tests. The dev equivalents total 42.16 s.
`make -C compiler build` exposes the first operation; `test-build` exposes the
second. Neither is an extra prerequisite run before every `make check`:
Cargo constructs missing artifacts when each owning command needs them.

Cargo's unit timings identify exactly where construction goes:

| Rust target/unit | Gate unit wall | Debug unit wall | Role |
|---|---:|---:|---|
| `compiler/build.rs`, compile / execute | 0.22 / 0.32 | 0.29 / 0.46 | Construct and run Cargo's build script, including derived specification identity; cached in the subsequent test-build commands |
| Ordinary `whitefoot` library | 41.93 | 14.19 | The compiler implementation used by `whitefootc` and integration tests |
| `whitefootc` binary, after that library | 0.72 | 0.45 | CLI executable |
| `whitefoot` library with `--test` | **69.36** | **23.38** | One executable containing all 1662 library tests |
| `programs` integration-test executable | 4.33 | 1.28 | The 110 program tests |
| `canonical_corpus` integration-test executable | 1.93 | 0.79 | Canonical-source checks |
| `conformance` integration-test executable | 2.06 | 0.87 | Native conformance adapter |
| `snapshot` integration-test executable | 0.42 | 0.33 | Recorded-verdict adapter |
| `whitefoot-grammar-tables` binary tests | 3.29 | 0.98 | One grammar-table consistency test |
| `whitefoot-spec` binary tests | 1.63 | 0.63 | Nine rule-scanner tests |
| `whitefootc` binary tests | 1.06 | 0.55 | Fourteen CLI/target-policy tests |
| Ordinary grammar-table/spec binaries, also built by all-targets | 3.31 / 0.96 | 1.02 / 0.50 | Non-test executable targets |

These are **per-unit wall durations within two concurrent Cargo jobs**. Do not
sum the rows into command wall time. The test-enabled library starts 7.40 s into
the gate all-targets command and takes 69.36 s: this is the critical path behind
the 76.79 s total. It is not 76.79 s for unit tests alone.

Why is it expensive? This is a large optimized compiler unit with its tests,
not dependency downloads: a path inventory finds 130,777 lines in 139
non-test-named Rust files and 82,626 lines in 117 test-named files, including
comments/generated code. A test-name filter changes execution selection only;
it still needs the containing test-enabled library. The ordinary library and
test-enabled library are distinct Rust compilation units. Cargo timings locate
that unit; they do **not** identify which internal rustc/LLVM pass is expensive.
Pass-level attribution remains unmeasured.

A separate fresh candidate `cargo test --profile gate --all-targets --no-run`
takes 116.26 s, with its test-enabled library at 114.63 s and ordinary library
at 72.01 s while sharing the construction window. Different scheduling/build
order explains why these unit durations are not directly comparable with the
sequential compiler-then-tests experiment. No Rust-build speedup is claimed.

Routine verification does not need a second dev `whitefootc` executable.
`gate` already retains the assertions/overflow checks. Clippy and rustdoc still
use their own dev-profile checking/documentation work; that is not an extra
unoptimized compiler used to compile WF programs.

## 2. Executing the Rust tests

This table is the exact complete-gate reading at `f3858780`. Cargo reports
0.00 s construction before these test groups because the artifacts are present.
The durations below begin **after construction**.

| Rust executable / selection | Executed tests | Test wall | What actually runs |
|---|---:|---:|---|
| `src/lib.rs`, excluding the three sampling modules | 1588 | **196.09** | Frontend/proof/IR assertions, Clang builds and generated-program oracles |
| Same `src/lib.rs` executable, only sampling modules | 74 | **29.23** | Exhaustion, loop splitting and parallel scheduling cases |
| `src/bin/grammar_tables/main.rs` tests | 1 | 0.32 | Regenerated grammar tables agree with checked-in tables |
| `src/bin/spec.rs` tests | 9 | 0.00 | Specification rule scanning, malformed/duplicate input handling |
| `src/bin/whitefootc.rs` tests | 14 | 0.00 | CLI options, target/runtime-link policy and error handling |
| `tests/canonical_corpus.rs` | 3 | 0.72 | Corpus parse/render round trips, permitted exclusions, exact normative example bytes |
| `tests/programs.rs` | 110 | **241.74** | Real WF programs, independent byte/status/filesystem/network oracles and parallel controls |
| `tests/conformance.rs`, ordinary Cargo invocation | 0; 1 ignored | 0.00 | Adapter is deliberately selected separately by root `conformance-run` |
| `tests/snapshot.rs`, ordinary Cargo invocation | 0; 1 ignored | 0.00 | Adapter is deliberately selected separately by root `snapshot-run` |
| `tests/conformance.rs --ignored`, root adapter invocation | 1 Rust harness; 805 corpus entries | **118.75** | 803 pass, 1 tracked Xfail (expected reject, reached Unsupported), 1 pending skip |
| `tests/snapshot.rs --ignored`, root adapter invocation | 1 Rust harness; 484 sources | **19.17** | Compile each source and compare accept/reject; no Clang or native execution |

`test-unit` and `test-sampling` do **not** build two crates or two library-test
executables. They partition one executable's 1662 tests. `test-corpus` means all
three binaries plus all four integration targets above; it is not a synonym
for the 110 `programs` cases. It executes 137 non-ignored tests in total.

### Library modules and individual slow tests

A separate run of the existing optimized test executables records each case's
own elapsed time and the existing optional helper trace. There is no Cargo or
rustc construction in this run. Stable libtest rejects `--report-time` directly;
`RUSTC_BOOTSTRAP=1` is set **only when executing the already-built harness** to
unlock `-Z unstable-options --format json --report-time`. It does not rebuild a
compiler, change a WF option or select a verdict. All original filters and two
test threads are retained. The run has an owned 900-second deadline. Its
initial `nice` request was sandbox-denied; the owned process group was lowered
to nice 10 after approximately two minutes, without restarting tests.

The library selections pass in **194.676 s (1588 tests)** and **30.562 s (74 tests)**.
Their module inventories and cumulative per-test intervals are:

| Library module / selection | Tests | Cumulative case wall | Longest case | Purpose |
|---|---:|---:|---:|---|
| `backend` | 270 | 262.333 | 41.738 | LLVM shape, native linking/execution, resource/ownership/runtime oracles |
| `driver` | 38 | 0.950 | 0.411 | Compiler pipeline, option/report/diagnostic boundaries |
| `lexer` | 33 | 0.001 | 0.000 | Tokens, literals, malformed bytes and token locations |
| `lowering` | 28 | 1.598 | 1.330 | Checked-program to executable IR and cleanup lowering |
| `prelude` | 1 | 0.007 | 0.007 | Prelude metadata/source consistency |
| `resolution` | 71 | 0.077 | 0.015 | Scopes, names, imports and duplicate/unknown-name diagnostics |
| `semantic` | 1042 | 85.732 | 58.881 | Typing, ownership/borrows, contracts, arithmetic domains and proof derivations |
| `source` | 12 | 0.000 | 0.000 | Source envelopes, paths and source limits |
| `spec` | 5 | 0.009 | 0.005 | Embedded specification identity and integrity |
| `syntax` | 88 | 0.998 | 0.389 | Parsing, grammar classification, canonical rendering/finalization |
| `backend::tests::exhaustion` (sampling) | 23 | 11.193 | 2.407 | Stack/resource exhaustion and cleanup/fault classification |
| `backend::tests::loop_split` (sampling) | 17 | 12.501 | 2.713 | Loop grain, associative combines, worker counts and ownership |
| `backend::tests::parallel` (sampling) | 34 | 31.226 | 12.453 | Grant/join behavior, refusal policies, recursion controls and result ABI |

The program attribution run takes 247.607 s with 101 passes and 9 environment
failures: eight loopback cases cannot bind under the sandbox, and the deep-tree
wfgrep case reaches the inherited descriptor limit. Those nine cases then pass
in a focused 46.535 s run outside the sandbox with descriptor limit 4096. The
247.607 s attempt is **not** a successful 110-case suite measurement; use the
241.74 s complete-gate result above for that question.

The 110 program tests have these modules (all main-attempt case identities are
counted; the nine failed timings are retained separately in the data):

| `programs` module | Cases |
|---|---:|
| `binary` | 1 |
| `generics` | 5 |
| `hashing` | 1 |
| `heap` | 5 |
| `image` | 1 |
| `network` | 12 |
| `numerics` | 4 |
| `parallel` | 43 |
| `raw_deflate` | 4 |
| `runs` | 6 |
| `signal` | 1 |
| `stream` | 4 |
| `support` | 1 |
| `text` | 3 |
| `traversal` | 6 |
| `wfgrep` | 12 |
| `wide_scan` | 1 |

Selected individual measurements (successful attempts):

| Test | Case wall |
|---|---:|
| `semantic::tests::entailment::frozen_real_sources_retain_complete_proof_roots_without_counted_false_positives` | 58.881 |
| `backend::tests::cost_shape::the_output_batch_costs_one_host_write_per_full_batch` | 41.738 |
| `backend::tests::cost_shape::the_reused_buffers_are_initialized_once_at_allocation` | 39.449 |
| `semantic::tests::entailment::generic_counted_roots_are_deterministic_across_twenty_analyses` | 9.371 |
| `backend::tests::slices::stable_scatter_matches_an_independent_oracle_and_hands_out_output_work` | 6.342 |
| `backend::tests::parallel::recursive_controls_preserve_scalar_and_destination_results` | 12.453 |
| `backend::tests::parallel::heap_box_loop_keeps_provider_order_and_updates_borrowed_owners` | 6.426 |
| `backend::tests::loop_split::every_admitted_combine_splits_and_publishes_the_unsplit_bytes` | 2.713 |
| `programs::runs::the_fixed_run_library_proves_and_runs` | 90.593 |
| `programs::parallel::corpus_par_wfgrep` | 86.208 |
| `programs::parallel::corpus_par_fixed_run_library` | 86.038 |
| `programs::wfgrep::a_match_across_a_read_boundary_keeps_its_line_number` | 46.643 |
| `programs::parallel::corpus_par_dir_walk` | 17.294 |
| `programs::parallel::corpus_par_raw_deflate_boundary` | 15.905 |
| `programs::raw_deflate::each_boundary_and_decode_outcome_reaches_its_own_status` | 15.676 |

[Per-case timing data](test-times.tsv) retains every attempt, including the nine
environment failures and their successful focused rechecks. The 24 binary-target
tests and 3 canonical-corpus tests were also timed individually in a separate
prebuilt-harness invocation (all pass, about one second total); their rows are
included. The 21 standalone research Rust tests are represented by the suite
aggregates below, not individual rows in this compiler/corpus table. It is this report's
measurement companion, not a new gate target or a performance threshold; update
or retire it with this report.


Module cumulative times count each test's own wall interval. With two test
threads, two tests can overlap; a test waiting for a shared `OnceLock` also
counts that wait. These sums are not suite wall, CPU time or exclusive work.
The companion per-case table distinguishes absent phase observations from zero.

The slow cases explain very different work:

- `semantic::tests::entailment::frozen_real_sources_retain_complete_proof_roots_without_counted_false_positives`
  analyzes UTF-8, a four-file DEFLATE bundle and wfgrep, then validates retained
  derivations and counted-loop roots. It does not run those WF programs.
- The two `backend::tests::cost_shape` cases share one wfgrep compilation and
  optimized module through `OnceLock`. They check initialization/allocation
  shape and output batching. Both case clocks can include the same shared
  compilation window; they are not two independent 40-second compiles.
- `generic_counted_roots_are_deterministic_across_twenty_analyses` really does
  repeat the same source analysis twenty times, comparing normalized proof
  ledgers for two generic instances. Its repetition protects determinism;
  this report does not establish that twenty is the minimum useful sample.
- `recursive_controls_preserve_scalar_and_destination_results` has
  2 self/mutual recursion forms × 2 scalar/aggregate return forms × 9 policies:
  **36 emitted/native images**, each executed under 2 supplied budgets ×
  2 grant/refusal outcomes, **144 native runs**. This is a policy/ABI matrix,
  not one twelve-second native algorithm.
- `an_overlapped_program_reports_one_byte_sequence_at_every_worker_count`
  runs 4 worker settings × 5 repetitions, plus a separately compiled sequential
  reference. Grant observations elsewhere stop after a successful observation,
  with a ceiling of 32 runs; a runtime that grants nothing still fails.
- `loop_split::a_split_loop_publishes_one_byte_sequence_at_every_worker_count`
  checks nine explicit worker settings and the unset default, then counted
  grant observations. Some split fixtures have 400,000 loop iterations to
  expose actual work sharing. Those are native execution, not Rust construction.
- `programs::runs::the_fixed_run_library_proves_and_runs` compiles a 375-line
  source with four-element runtime capacity. Its slow part is proof compilation,
  not a huge runtime input. It covers fixed-run construction, transposition,
  checked boundaries and draining while preserving order and ownership.
- `programs::parallel::the_default_compilation_of_the_demo_names_no_runtime`
  does contain substantial native computation: `par_layout.wf` performs 800
  tree traversals for each of two folds per batch, and this test runs the
  ordinary form, a four-batch invocation and a worker-setting control. Its
  batch argument is part of the output-equivalence check.

## 3. WF compilation, Clang construction and native execution

The optional helper records calls **inside** the tests, not Rust construction.
`whitefoot-compile` covers the compiler pipeline through LLVM emission;
`semantic-check` stops at checked semantics. `native-build` constructs/links
native images; `native-run` includes process launch/wait and program work.
Helpers do not cover every custom subprocess, explicit no-overlap compilation
or direct compiler call. Parent/child intervals and parallel intervals can
also overlap. Consequently their totals cannot be subtracted from suite wall
to manufacture an "assertion time" or exact phase percentage.

| Attribution invocation / phase | Observed calls | Cumulative helper wall |
|---|---:|---:|
| unit / `native-build` | 390 | 51.857 |
| unit / `native-run` | 330 | 78.760 |
| unit / `semantic-check` | 1368 | 67.093 |
| unit / `semantic-test-assertions` | 1368 | 0.066 |
| unit / `whitefoot-compile` | 221 | 56.316 |
| sampling / `native-build` | 110 | 11.557 |
| sampling / `native-run` | 28 | 7.228 |
| sampling / `whitefoot-compile` | 16 | 0.224 |
| programs / `native-build` | 94 | 9.938 |
| programs / `native-run` | 101 | 32.835 |
| programs / `whitefoot-compile` | 53 | 260.496 |
| programs / `whitefoot-compile-par` | 41 | 179.989 |
| programs-retry / `native-build` | 11 | 2.437 |
| programs-retry / `native-run` | 1 | 0.411 |
| programs-retry / `whitefoot-compile` | 7 | 46.305 |
| programs-retry / `whitefoot-compile-par` | 1 | 0.281 |

The program-attempt rows include work before environment failures; the retry
rows are a separate invocation and may reconstruct shared images. This is not
a single successful suite's phase total. Unit proof-assertion helper time is
only 0.066 s across 1368 callbacks; deleting those assertions would save almost
nothing. Other assertions and direct compiler paths are outside that helper.

For the slow program cases in the first attribution attempt:

| Case | WF compile/emission | Native construction | Native launch/execution | Whole case |
|---|---:|---:|---:|---:|
| `programs::runs::the_fixed_run_library_proves_and_runs` | 90.222 | 0.106 | 0.262 | 90.593 |
| `programs::parallel::corpus_par_fixed_run_library` | 85.890 | 0.114 | Not observed | 86.038 |
| `programs::parallel::corpus_par_wfgrep` | 85.225 | 0.544 | 0.434 | 86.208 |
| `programs::wfgrep::a_match_across_a_read_boundary_keeps_its_line_number` | 46.178 | 0.236 | 0.224 | 46.643 |
| `programs::parallel::the_default_compilation_of_the_demo_names_no_runtime` | 0.262 | 0.106 | 9.839 | 10.208 |


### A compiler hotspot, not native execution

| Standalone command using prebuilt `whitefootc` | WF compile/emission wall | User / system CPU | Clang | Native program execution |
|---|---:|---:|---|---|
| `--emit-llvm tests/programs/wfgrep.wf`, gate compiler | **39.49** | 39.02 / 0.43 | None | None |
| Same source/options, dev compiler | **535.38** | 529.76 / 2.13 | None | None |
| `--emit-llvm .../read_heavy_wide8.wf`, gate compiler | **58.29** | 56.39 / 0.52 | None | None |

The wfgrep LLVM outputs compare byte-for-byte equal. The 13.56× ratio is the
speed of the **Rust compiler executing proof analysis/emission**, not generated
wfgrep runtime. Routine dev compiler execution therefore saves construction
seconds but can spend minutes per real-source compile.

For `read_heavy_wide8`, a three-second sample after twenty seconds finds 2008
semantic-flow stacks, including 1229 leaf samples (61.2%) in
`close_with_excluded_term`. Peak RSS is 2.88 GB. This is one sampled window,
not a whole-command profile. Proof-closure optimization remains separate in
[PR #65](https://github.com/mbbill/Whitefoot/pull/65).

A prebuilt isolated wfgrep corpus case takes 41.32 s on baseline, 41.44 s on
candidate without helper logging and 43.01 s with logging. The logged case
splits into **40.917 s WF compilation, 0.852 s native construction and 1.224 s
native launch/execution**. It checks a match crossing a read boundary and its
line number; it does not spend forty seconds searching files.

The earlier cold full-suite trace reports 215.576 s for fixed-run compilation
and 104.173 s for one shared wfgrep compilation. The complete cold program
suite takes about 430 s, versus 239.08/241.74 s in the subsequent warm gates.
Baseline/candidate `whitefootc` binaries are byte-identical and the isolated
wfgrep comparison does not reproduce that slowdown. Its context/concurrency
contribution remains unresolved; neither production changes nor logging have
been established as its cause.

A trivial first-launch probe also separates host overhead from algorithms:
three fresh `int main(void) { return 0; }` executables take 1.29/0.29/0.28 s on
first launch, then 0.00 s at `time -p` resolution on repeat launches; reported
CPU time is 0.00 s. This identifies first-launch overhead but not the service
responsible, and cannot explain every native-run interval.

### Conformance and snapshot internals

The manifest contains 821 rows: 16 rule annotations and 805 cases. The cases
are 330 runnable `run`, 95 runnable `accept`, 378 runnable `reject`, one
tracked Xfail whose normative expectation is `Reject(OP-4)` but whose compiler
result is `Unsupported`, and one pending acceptance. Thus the native adapter
attempts **804 compiler verdicts and 330 native constructions/executions**;
it does not launch 805 programs. The pending case is not compiled.

To split these previously opaque adapters, temporary copies of their Rust
harnesses add `Instant` observations around the existing compiler call,
link helper, spawn/wait interval and complete case. They link the existing
`f3858780` `whitefoot` rlib with `rustc --test -C opt-level=3
-C debug-assertions=yes -C overflow-checks=yes`; compiler construction is not
repeated. Original iteration, arrangements, matching and assertions remain.
Temporary adapters are observation tools, not committed gate replacements.
They run serially outside the sandbox, with descriptor limit 4096 and nice 10.

| Operation | Count | Elapsed | Boundary |
|---|---:|---:|---|
| Construct temporary conformance Rust harness | 1 | 1.22 | Links the already-built compiler library; not a cold Cargo build |
| Run conformance harness | 1 | **120.54** | Full test interval; 803 pass, 1 tracked Xfail (expected reject, reached Unsupported), 1 pending skip |
| WF compile/check/emission inside it | 804 | **25.514** | No Clang or program execution |
| Native construction inside it | 330 | **23.001** | Emit/stage native inputs, obtain shared runtime objects, Clang/link and native-build cleanup |
| Native launch/execution inside it | 330 | **70.567** | Spawn, provide input and wait for exit |
| Case setup/assertions/cleanup and adapter overhead | — | About 1.46 | Remainder of this serial run; not a separately timed command |
| Construct temporary snapshot Rust harness | 1 | 0.36 | Links the same existing compiler library |
| Run snapshot harness | 1 | **19.06** | All 484 verdicts unchanged |
| Read and compile individual snapshot cases | 484 | **19.007** | No Clang/native image execution |

The conformance phase intervals are non-overlapping in one serial loop, so
adding them is meaningful here. Their native-run total is elapsed launch/wait
and program work, **not 70.567 CPU seconds spent in algorithms**. The trivial
launch probe above establishes a host-overhead component but not its total
share in these 330 different processes.

| Slow conformance case | Whole case | WF compile | Native build | Native run |
|---|---:|---:|---:|---:|
| `x-base64-rfc-vectors-run` | 4.518 | 4.158 | 0.087 | 0.265 |
| `liv2-pos-two-elements-of-two-inner-runs-are-distinct-targets` | 1.389 | 1.070 | 0.070 | 0.245 |
| `blk3-pos-full-array-owning-round-trip` | 1.142 | 0.825 | 0.078 | 0.237 |
| `fn8-pos-affine-requirement-images` | 1.014 | 0.016 | 0.770 | 0.226 |
| `run-sysout-redirect-same-sink-order` | 0.793 | 0.448 | 0.081 | 0.256 |

The slowest snapshot case is `indexing__adversary-r1__06-binary-search-accept`
at 1.245 s. It compiles a binary-search proof case; it executes no
binary-search program. The per-case data includes every attempted corpus row.


## 4. Exactly what the local entry points execute

| Entry point | Actual work | Included in root `make check`? |
|---|---|---|
| Root `make check` | Ten stages below, sequentially, with nested compiler stages | The canonical complete entry point |
| `make -C compiler check` | Format, Clippy, partition, unit, sampling, corpus, rustdoc, spec binary, native completion harnesses | Yes |
| Root `make static` | Repository invariants, immutable spec archives, current-prose authority checks, design-lint tests/lint | All four stages are included |
| `make -C compiler static` | Format, Clippy, rustdoc, spec binary, **C runtime construction/execution** | Same component targets are included; the `static` alias itself is not called |
| `make -C compiler test` | Partition plus both library selections plus binary/integration tests | Same components; ignored corpus adapters still require root targets |
| `make -C compiler build` / `test-build` | Construct compiler / all test executables without executing Rust tests | On-demand construction already occurs in the owning gate commands |
| Root `make historical-tool-tests` | Retired frequency-study and default-floor instrument self-tests | No; explicit reproduction |
| Compute `make verify` / `compare` / `verdict` | All-form oracle execution / timed comparison / regression decision | Full protocols no; verdict-rule self-tests yes |
| IO `make bench` and read/network scripts | Data construction, verification and repeated IO measurement | No; bounded program/native checks yes |

`static` is a historical target name, not a precise description of its
contents. Root `static` even runs the guard's subprocess tests. Compiler
`static` builds and runs C programs and the Rust spec-checker binary. Neither
alias is a substitute for the complete gate.

The exact `f3858780` root run takes **786.97 s (13m06.97s)**, with 1046.94 user
and 72.46 system CPU seconds. Below are inner command `time -p` readings;
wrappers and stage transitions explain the small difference from their sum.
Nested compiler rows replace, rather than add to, the 479.93 s compiler stage.

| Stage / command | Wall | Specific operation and coverage |
|---|---:|---|
| `repository-invariants` | 11.18 | Run `.github/test-run-check.sh`: status propagation, nested ownership, competing invocation rejection, concurrency defaults, timeout, signals and orphan cleanup; compare AGENTS/CLAUDE; scan tracked content/paths for machine-local names |
| `spec-append-only` | 0.05 | Git diff against local `main`; reject modifications/deletions of released spec archives |
| `spec-prose-integrity` | 0.08 | Search current guidance for copied spec digests and stale active-version declarations |
| `design-lint` | 6.38 | 17 Python tests for the tree linter, then tree/amendment form and authorized-change-boundary checks; 57 nodes, depth 3 |
| `conformance` | 0.21 | 25 Python runner tests; manifest/source/rule coverage, 129/129 rules; no WF compilation |
| Compiler `format`: `cargo fmt --all -- --check` | 1.10 | Rust formatting, no type checking or test execution |
| Compiler `lint`: `cargo clippy --all-targets --locked --offline -- -D warnings` | 0.13 | Rust type/lint checking of library, bins and tests; cached here; no test cases run |
| Compiler `test-partition` | 0.32 | Six `--lib -- --list` selections; prove 1588 + 74 = 1662, each named sampling module is nonempty, and all four integration targets are declared; cold runs build the library-test executable here |
| Compiler `test-unit` | 196.13 | Cargo selection excluding `backend::tests::{exhaustion,loop_split,parallel}`; 1588 tests |
| Compiler `test-sampling` | 29.27 | Cargo selection of those three modules; 74 tests |
| Compiler `test-corpus` | 242.93 | Three binary-test and four integration-test executables; 137 tests execute, two adapters remain ignored in this invocation |
| Compiler `docs`: `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps` | 2.55 | Build Rust API documentation and reject rustdoc warnings; not Markdown lint and not doctest execution (`doctest=false`) |
| Compiler `spec`: `cargo run --profile gate --bin whitefoot-spec` | 0.34 | Execute the specification scanner/identity checks; 129 rules |
| Compiler `completion-test` | 5.75 | Compile and run the C runtime probes described below; no Rust tests or WF source compilation |
| `research-tests` | 15.94 | Current compiler witnesses plus independent Rust/C/Python models/oracles; inventory below |
| `bench-programs` | 133.85 | Refresh gate compiler; IO native oracles and 12 WF image constructions; compute lowering/link witnesses; not a full benchmark timing protocol |
| `conformance-run` | 118.79 | Compile/check 804 non-pending cases, link/run the 330 run cases through one Rust harness |
| `snapshot-run` | 19.21 | Compile 484 recorded sources through one Rust harness, compare verdicts, no native linking/running |

### Native completion programs and current research tests

A separate recipe/CC observer reruns these short native/research targets with
current artifacts. It leaves compiler options and assertions unchanged, but
Ruby observer startup inflates the outer targets: the instrumented completion
target takes 9.08 s versus 5.75 s in the ordinary full gate. Use the individual
child-command readings below for attribution, not 9.08 s as a new normal-cost
claim. Shell-recipe timings include their shell; C-build timings start inside
the observer immediately before invoking `cc`.

| C artifact / operation | C construction | Execution / check |
|---|---:|---|
| `core-read-probe` | 0.468 | 0.247 execution; symbol isolation separately checked |
| `bridge-default-probe` | 0.496 | 0.315 execution on the shipped helper policy |
| `ordinary-values-probe` | 0.590 | Helpers 0 / 2: 0.234 / 0.009 execution |
| `sched-smoke` | 0.214 | Seven startup modes 0.274; explicit one-worker run 0.007 |
| `sched-deque-probe`, stats on / off | 0.207 / 0.207 | Combined build+run shell loop 1.157; loop overhead and the two runs are not separately isolated |
| Full completion `harness` | 0.683 | Helpers 0 / 1 / 4: 0.346 / 0.100 / 0.096; no-cache variant 0.097 |
| `pure-compute` | 0.053 | `nm` link-boundary check 0.018; no execution |
| Linux ring source on macOS | 0.024 | `-fsyntax-only`; no executable |

All native probes pass. In the separately observed warm research call, the
container sub-makes cost: authority 0.428, foundation 1.041, lifecycle 12.723,
dense 0.675, costs 0.390 and families 3.697 s. These include observer startup,
so they are not a replacement for the ordinary 13.36 s container total. They
locate the work: lifecycle's compile/run/negative-control loop itself is
12.503 s; the 45 cached family executions take 0.229 s before their additional
observers. The standalone proof-use runner's Rust construction is 0.410 s and
its fixture checks 0.960 s. The crafted compute-verdict rule takes 0.118 s.

Foundation's whole `layout` executable takes 0.323 s; its 81 reported timed
samples sum to 0.224 s. That is a small residual exploratory measurement,
separate from the large proof-compilation costs.


The compiler completion target includes these concrete native C artifacts:
`core-read-probe` (isolated completion core/read and bridge-symbol boundary),
`bridge-default-probe` (shipped helper policy and file/TCP paths),
`ordinary-values-probe` (text/ranges/files/directories/TCP/close-reuse and credit
transfer, helpers 0 and 2), `sched-smoke` (startup/workers), two deque probes
(stats on/off, each with 200,000 tasks), and the full completion `harness`
(helpers 0, 1, 4 and no-cache policy). `pure-compute` is constructed and checked
with `nm` for unwanted completion symbols, not executed. macOS syntax-checks
the Linux ring source; Linux additionally builds/runs the native ring probe and
runs the required-ring harness when the probe is available. Sanitizers belong
to the separate Linux IO-host workflow, not this ordinary local target.

`research-tests` is not another large Cargo unit suite:

| Owner and command | Concrete cases/programs | Final-code local wall |
|---|---|---:|
| `proof-use-cost check` | Build a standalone optimized Rust runner; refresh `whitefootc`; 7 accepted WF fixtures including the 4096-use ceiling and 2 rejection controls | 1.57 |
| `container-representation check` | Six subdirectories detailed below, cached artifacts plus continuing assertions and lifecycle recompilation | 13.36 |
| `ripgrep test` | 22 Python runner/oracle tests | 0.16 |
| `raw-deflate-default-shape/test_oracle.py` | 19 Python independent DEFLATE-oracle tests | 0.27 |
| Compute `verdict-test` | 14 crafted-table regression-rule tests in shell/AWK; no performance measurement | Included in 15.94 s total; not separately timed in that run |

Container subdirectories execute:

- `authority`: `model.rs` **7 Rust tests** and the model's 510 bounded live sets;
  `membership.rs` **2 Rust tests** plus identity/generation/lifecycle traces.
- `foundation`: `model.rs` **5**, `construction.rs` **1**, `rust-baseline.rs`
  **2 Rust tests**, their model executables, C layout checks and a WF large-result
  witness. The current `layout` invocation also prints 9-sample/1200-iteration
  copy timings; these are observations without a regression verdict, not extra
  correctness cases. This residual mixing is identified explicitly below.
- `lifecycle`: **7 accepted WF programs**, constructed and executed; **8 rejected
  WF fixtures**, checked for exact rule/detail and absent executable output.
- `dense`: 16/256/4096 elements and a 16×4-lane form, native reference comparisons,
  a retained-boundary control, an inline-view witness and IR/assembly checks.
- `costs`: C map-layout/sparse-ownership and Rust map controls, in `check` mode;
  allocation, membership, payload and ownership observations, not timed passes.
- `families`: **15 WF sources × 3 modes = 45 executions**, plus allocation/growth/
  behavior/ABI observers, retained/inlined boundary controls and one negative
  compiler case. Rust `priority-interface.rs` has **1 test** and
  `owning-growth-abi.rs` **3 tests**.

That is **21 standalone Rust research tests in seven test executables**, each
suite finishing at 0.00 s in this local run. They are compiled with `rustc
--test`, not Cargo crates; the surrounding WF/native construction and model
program execution explain the experiment's larger elapsed time.

`bench-programs` has two exact owners:

- IO `programs-check`: `ordinary-check` generates a 4-file/16-KiB-bound tree and
  runs the public/body native callers; construct all **12** `programs/*.wf`
  images. Linux also runs `uring-check`, **7 deterministic traces × 2 send
  policies**. The WF image construction does not execute the full IO workloads.
- Compute `programs-check`: **10 kernel sources** (`mandelbrot`, `quadrature`,
  `records`, `fir`, `stencil`, `prefix`, `histogram`, `merge_sort`, `bfs`,
  `radix_scatter`) in parallel/sequential forms, checking publish-site presence,
  symbols and real-runtime links; **`range_split.wf`** additionally constructs
  and executes both forms. `module-symbols-test` and `baseline-entry-test` check
  this machinery. This is 11 sources, not the five timed kernels of `compare`.

## 5. What CI actually executes and how long it takes

[Gate run 34924294748](https://github.com/mbbill/Whitefoot/actions/runs/34924294748)
is the successful `f3858780` reading. `gate.yml` uses seven jobs on each of
Linux and macOS. Each VM constructs its own required artifacts; a compiler
built in the research job is not available to the unit job. The unit and
sampling jobs consequently construct the same test-enabled library on separate
VMs. Local `make check` reuses it between those stages.

Times below separate **job total**, **main step**, and **construction/execution
inside the main step**. Job total includes setup/cleanup but excludes queueing.
The remainder is job minus main step, not a measurement of network time alone.
Columns use Linux / macOS order. Cargo reports durations above a minute rounded
to seconds. Parallel job times are not summed into workflow latency.

| CI job and exact command | Job total | Main step | Setup/other | Inside the main step: construction then execution |
|---|---:|---:|---:|---|
| `static`: root `make static` then `make -C compiler static` | 119 / 132 | 98 / 117 | 21 / 15 | Clippy 16.30 / 24.65; rustdoc 8.23 / 11.38; **gate spec binary/library build 51.13 / 55.58**; repository checks and native C construction/runs also execute |
| `unit`: `make -C compiler test-partition test-unit` | 318 / 327 | 298 / 311 | 20 / 16 | Library-test build **99 / 137**; 1588 tests **197.60 / 172.54** |
| `sampling`: `make -C compiler test-sampling` | 194 / 152 | 155 / 140 | 39 / 12 | Same library-test construction **130 / 123**; 74 tests **24.99 / 15.38** |
| `corpus`: `make -C compiler test-corpus` | 239 / 309 | 220 / 296 | 19 / 13 | Rust bin/integration construction **51.10 / 59.35**; 110 program cases **167.97 / 234.38**; other 27 cases 0.79 / 1.15 |
| `conformance`: `make conformance && make conformance-run && make snapshot-run` | 134 / 174 | 110 / 158 | 24 / 16 | Adapter/library build **46.77 / 75**; adapter **43.78 / 54.87**; snapshot build 0.29 / 0.63 and execution **19.20 / 25.18**; Python structure checks also run |
| `research`: `make research-tests` | 289 / 254 | 268 / 237 | 21 / 17 | Compiler build **53.42 / 63** inside proof-use check 63.69 / 67.78; containers **203.01 / 166.58**, including their Rust/WF/native construction and checks |
| `bench-programs`: `make bench-programs` | 215 / 361 | 191 / 341 | 24 / 20 | Compiler build **41.19 / 81**; remaining work constructs WF/native images and runs the specific bounded oracles above |

Every gate job checks out the tree, installs/records the host toolchain, names
`main`, sets scratch/test parallelism and (Linux) declines core dumps. The
static job also chooses the design-review base. Linux toolchain setup costs
10–26 s across these jobs; macOS 1–3 s. Checkout costs 5–8 s. The job's
"ten largest gaps" output is **not per-test elapsed timing**: concurrent tests
can finish inside one another's intervals.

The static job now pays a cold gate-library build for `whitefoot-spec` on its
separate VM. In the local full gate that library is already built. Changing
this binary to `gate` therefore removes a local duplicate dev executable but
does not promise that the isolated static CI job is faster.

### Other automatic workflows

| Workflow / operation | Linux | macOS | Windows | What the operation does |
|---|---:|---:|---:|---|
| Compute correctness: whole job | 151 | 104 | — | Push-triggered build plus oracle checks; full timing steps skipped |
| Build pinned scheduler dependencies | 41 | 17 | — | oneTBB/Rayon and Parlay availability; dependency work, not WF tests |
| Build kernel images | 54 | 59 | — | Rust compiler if missing, WF modules, native runtime/references and links |
| Verify forms and widths | 24 | 13 | — | Execute each form/width against independent kernel oracles |
| Paired compute regression: whole job | 350 | — | — | PR check comparing this tree with its merge base on one VM |
| Build merge-base compiler | 49 | — | — | Rust `gate whitefootc` for the baseline arm |
| Dependencies / both image sets | 27 / 61 | — | — | Dependency construction, WF/native construction for both arms |
| Verify both arms / timed paired passes | 28 / 152 | — | — | Correctness first, then five paired measurement passes |
| IO platform correctness: whole job | 43 | — | 207 | Native platform/runtime behavior, not the full storage protocol |

Sources: [compute correctness](https://github.com/mbbill/Whitefoot/actions/runs/34924294745),
[paired regression](https://github.com/mbbill/Whitefoot/actions/runs/34924297881),
[IO hosts](https://github.com/mbbill/Whitefoot/actions/runs/34924294754).
All pass. The paired regression covers 15 kernel/width blocks, with no suspects
or CPU reports. Its unchanged rule needs a >3% paired median slowdown and
4/5 adverse pairs at two widths among 1/2/4 workers to fail a kernel; the CPU
signal is report-only. This performance decision is separate from `make check`.

Linux IO-host work is C, not Cargo tests: native ring probe 1 s; completion
harness 8 s; required-ring execution under 1 s; ASan/UBSan builds/runs 7 s;
isolated ThreadSanitizer 5 s; bridge/ring ThreadSanitizer 3 s. Setup and the
host toolchain account for the rest. The gate's deterministic reference fixture
also passes all 14 traces; its two builds plus trace execution take about
0.582 s in the timestamped Linux benchmark-program log.

Windows IO-host steps are explicit:

| Windows step | Wall | Construction versus execution |
|---|---:|---|
| Deque reuse/counters | 9 | C construction and execution, stats on/off |
| CPU performance levels / startup-frame probes | 1 / 1 | C platform probes |
| Default-route probe | 3 build + 1 run | Run native IOCP and refused-ring adapter routes |
| Native adapter probe | 2 build + <1 run | Actual Windows adapter |
| Native namespace probe | 1 build + <1 run | Native path/code-unit behavior |
| Ordinary values / strict runtime check | 4 / 2 | C construction/execution / C syntax-warning checks |
| Bridge failure probe | 3 build + <1 run | Deliberate initialization refusal must fail stop |
| First real WF IOCP program | 94 | **88 s Rust gate compiler construction**, then about 6 s WF/native construction and execution |
| TCP routes | 24 | Construct WF images, then run both native-ring and adapter engines |
| HostString raw units | 4 | Construct/run WF boundary witness |
| Component-buffer open | 8 | Construct/run WF native-component witness |
| Native parallel workers | 14 | Construct/run WF worker/grant probes |
| Compute-thread overflow floor | 13 | Construct/run WF floor classification probes |

Later Windows steps reuse that gate compiler. These step totals do not further
separate each WF compile from its Clang invocation and native run; those inner
boundaries were not logged. No debug `whitefootc` is built by this workflow.

## 6. Full measurement protocols, outside the gate

Compute `compare` is five kernels and 139 implementation/width cells across
five passes: **695 process invocations and 4170 recorded first/warm calls**.
With prebuilt compiler and cached pinned source checkouts, the local sequence is:

| Command | Wall | User / system CPU | Work |
|---|---:|---:|---|
| `make deps`, fresh native dependency outputs | 13.78 | 17.14 / 3.40 | Construct pinned native/Rust scheduler dependencies |
| `make build` | 9.45 | 7.00 / 1.83 | WF/native kernel images |
| `make verify` | 70.07 | 409.84 / 4.14 | Native correctness execution across forms/widths |
| `make compare` | 229.62 | 1231.91 / 15.31 | Full timed passes and tables |

An initial fresh oneTBB source fetch was stopped after 459.41 s with 0.19 user
and 0.08 system seconds, before compilation. That was network waiting. The
completed sequence uses clean existing checkouts verified at the pinned commits.

Local read-heavy IO costs **1288.52 s (21m28.52s)**: the Rust compiler is already
built; native references/data take about 4 s; eight WF/native image builds take
about 349 s (38–58 s each); the remainder is verification and repeated reads.
The protocol has 17 macOS or 20 Linux configurations × four tables ×
(7 measured rounds + 2 warmups): 612/720 calls, a 512-MiB dataset and 32,768
reads per call. It is not a long-running Cargo test. All four local cache
labels were confirmed before and after measurement.

Full IO matrices now require manual dispatch. The completed
[manual run](https://github.com/mbbill/Whitefoot/actions/runs/34912406944)
at `35227ab3` has these job readings; this is older protocol evidence, not a
rerun at `f3858780`:

| Manual IO job | Job wall | Main work |
|---|---:|---|
| Linux many-files/network | 355 | File protocol 293; network protocol 30; setup remainder |
| Linux read-heavy | **3995 (66m35s)** | Rust construction 36.70; about 570 image/data/verification; uncached tables 1068 and 2239; warm tables 46 and 5 |
| macOS read + many-files | 1146 | Read protocol 929; many-files 199 |
| Windows measurement | 463 | Native protocol including construction 418 |

The Linux 64-KiB table's post-probe refused its uncached label. A green workflow
is not a valid uncached claim for that table. The 4-KiB uncached probes pass.
These long jobs are storage protocol work and variability, not hour-long Rust
builds. Automatic IO-host correctness and the separate compute-regression
verdict remain enabled.

## 7. Causes, changes and remaining limitations

The interrupted scatter investigation's recovered 21m27s filtered Cargo
command never reaches `Running unittests`; about 7m16s is build-lock waiting.
Another worktree's optimized library tests are visible, and saved load averages
reach about 200–240. This establishes overlapping work, not which agent owned
it or an exact thermal/memory/disk attribution. An earlier compiler construction
also took eight minutes. Those overloaded commands are not normal baselines.
Starting additional heavy commands and leaving them without a bounded stop
were orchestration defects.

The selection criterion was recorded before changes: classify construction,
proof/emission, native execution, protocols and waiting separately; compare the
same workload under controlled concurrency; remove demonstrated duplicate work
without fewer assertions, shorter schedule samples or cached verdicts. Before
changing IO dependencies, the criterion additionally required Cargo refresh
before image freshness, reuse when unchanged, and invalidation on source,
compiler and Makefile changes.

| Demonstrated issue | Change already in PR #66 | Measured effect / boundary |
|---|---|---|
| Overlapping local heavy commands and invisible lock waits | Host-wide owner, two-job/two-test defaults, 30 s progress, phase wall/CPU/status, cancellable process group, default 30 min deadline | Guard failure/status/nesting/exclusion/timeout/signal/orphan falsifiers pass; deadlines fail verification, never reject source |
| Recompiling identical native runtime support for separate tests | Reuse immutable objects within the test process; fresh emitted modules, observers and special macro builds remain | Unit helper trace exposes remaining link/run cost; no matched whole-suite speedup claimed |
| Rebuilding 45 unchanged container-family images | Source/compiler/Makefile dependencies; execute all 45 images and observers every check | Warm command **81.86 → 1.89 s**, 97.7% reduction |
| IO image depended directly on phony Cargo refresh | Real compiler-file dependency, refreshed before image freshness | Warm `build` **128.95 → 0.22 s**; `verify` **141.51 → 6.46 s**; required rebuild 128.19 s |
| Repeated construction inside the many-files timing command | Same dependency fix; unchanged inputs and 7+2 rounds | Full command **200.86 → 70.62 s**, 64.8% reduction; not a faster IO runtime |
| Routine unoptimized proof execution | Windows/spec checker use existing assertion-retaining `gate` | Same wfgrep LLVM, 39.49 versus 535.38 s compiler execution; no WF optimization flag added |
| Automatic exploratory compute/IO timing without a regression verdict | Dispatch-only scoreboards; keep correctness and paired performance decisions | Final compute push skips compare/publication/upload; IO correctness remains automatic |
| Historical miner/model-trajectory self-tests treated as compiler checks | Explicit `historical-tool-tests`, with documented retirement grounds | Retained reproduction passes in 8.16 s with Rust artifacts present; current compiler witnesses remain |
| Cargo failure masked by `... \| tail` in IO scripts | Preserve Cargo exit status and prior output; label phases | Injected Cargo failure stops all three scripts before later construction |
| Missing automatic reference-oracle caller | Linux IO `programs-check` calls deterministic `uring-check` | Both send policies, all 14 traces pass; macOS lacks Linux headers and is not claimed as a trace run |

Further audit observations must not be hidden behind a claim that every cost
has been removed:

- Whole-program proof compilation remains the largest measured source cost;
  PR #65 owns that algorithm work. This report does not change compiler semantics.
- The `lifecycle` check still constructs its seven accepted images each time;
  negative cases must still obtain a current compiler verdict. Its construction
  is distinguishable from the 21 fast standalone Rust research tests.
- `foundation/layout` still mixes correctness with copy-timing output in `check`.
  That timing has no regression decision, so it does not meet the owner's CI
  criterion as a measurement. Its cost and retained correctness work are exposed
  here; separating that small experiment is follow-up work, not a claimed saving.
- Full rustc pass attribution, the anomalous cold-context corpus slowdown, the
  responsible first-launch host service and the inner Windows compile/link/run
  breakdown remain unmeasured or unresolved. No subtraction of overlapping
  timings fills those gaps.

## Validation and design state

The exact code revision `f3858780` passes root `make check` in 786.97 s and all
four automatic workflows linked above. The pure warm run at `e98c673a` takes
648.54 s; the cold candidate takes 1353.25 s. Baseline prebuilt-Rust/fresh-native
`make check` takes 1127.30 s. These are explicit artifact states, not a matched
whole-gate speedup. The 152.79 s interrupted warm observer run hit a temporary
Ruby process-argument UTF-8 decoding error; cleanup worked, byte parsing fixed
the observer, and replacement/final runs passed. It is not a repository test
failure or a completed timing result.

Independent completion review and DCR at `f3858780` found no unresolved finding
in their scope after owner-directed fixes to the missing Linux reference
caller and automatic exploratory compute timing. The primary assessment agrees
with both fixes. The reporting follow-up independently checks the actual target/case inventory,
per-case and per-phase data, temporary adapter diffs and exact CI logs. Its
ordinary findings corrected the omitted build-script units, clarified the
tracked Xfail and added the 27 fast compiler cases to the data. Root `make
static` passes in 16.66 s; no production code or maintained test logic changes
in this follow-up, so the full compiler gate was not repeated for prose/data.
The lifecycle rebuild and foundation timing remain explicitly identified
follow-up work, not a new blanket claim that every CI cost meets the criterion.

The [pending amendment](../../../design/amendments/compiler-verification-cost.md)
proposes a new `design/compiler/verification.md` node: owned local verification,
immutable/dependency-correct construction reuse, existing gate-profile
verification, and automatic correctness/regression decisions with explicit
historical reproduction/exploratory measurement. It rejects weakened normative
cases, shorter sampling and shared verdict caches. The live tree remains at
57 nodes, depth 3, net change zero; owner ruling on the exact amendment remains
pending. No specification, normative case or verdict changed.

Conformance invocation wiring passes through the guard but retains the same
Cargo adapter command, ignored-case opt-in, inventory and interpretation. A
wrapper deadline or child failure is failed/incomplete verification, never a
normative rejection. That preservation is the selection ground for the wiring
change. Timing-only temporary adapters below are attribution evidence, not a
replacement for the canonical conformance gate.
