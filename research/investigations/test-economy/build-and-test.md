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

Measurements and the selected implementation will be recorded here as they
become available.
