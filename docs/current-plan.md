# Current Plan

Status: ACTIVE — ripgrep 15.2.0 is the owner-selected umbrella target;
`RG-BASE` approved by the owner on 2026-08-04

Derived from: [Direction Outline revision 4](roadmap.md), items `CAND-8` and
`PERF-1`

This approval covers only the written `RG-BASE` `Do`, verification, acceptance,
and stop boundary. Selecting ripgrep and the 2x objective does not itself
authorize a Whitefoot port, compiler or specification changes, or
parallel-runtime work.

## Umbrella target

Build a Whitefoot-written command-line search tool that is credible as a
replacement for ripgrep's primary line-oriented recursive regex-search use and
reaches at least **2.00x** end-to-end speedup over pinned ripgrep 15.2.0 on a
preregistered representative suite.

The objective is the broad product comparison, not one currently suspected
opportunity. A milestone may implement or investigate a smaller vertical slice,
but success on one pattern, file shape, output mode, or microkernel does not
rename or complete the target. The persistent upstream pin, claim boundary,
source-architecture findings, open opportunity map, and attribution rules live
in the [ripgrep flagship frame](../research/notes/ripgrep-flagship-frame.md).

## Milestone

### RG — 2x ripgrep

Reach a correctness-green, directly runnable Whitefoot search executable whose
frozen primary command surface and representative benchmark suite satisfy the
flagship frame's 2x product rule. The suite must exercise real source trees and
large text, one and many files, representative matcher families, traversal and
ignore work, result production, and controlled cache states. It compares the
same selected inputs, matches, relevant ordering, rendered or canonical output,
diagnostics, exit status, hardware, thread and memory envelope.

No complete implementation route is predicted in advance. Each later Current
Plan will either establish one real product path, resolve one blocker, or test
one attributed performance opportunity, then return to the unchanged umbrella
comparison.

## Why this is the selected pressure

Ripgrep is already known as a fast Rust tool. A reproducible 2x replacement is
an immediately understandable result with low user trial cost: install one CLI
and run the same search. Its end-to-end path also combines several Whitefoot
directions that otherwise remain disconnected:

```text
CLI and filesystem boundary
  -> ignore-aware traversal
  -> regex and byte matching
  -> checked buffers and result construction
  -> declared parallel work and deterministic failure
  -> target code and observed wall time
```

The pinned source is not an easy baseline. It already uses work-stealing
parallel traversal, `regex-automata`, literal extraction, SIMD-capable byte
search, buffered and whole-input strategies, per-worker printers, and recent
directory-walk improvements. The project therefore supplies real evidence
about whether Whitefoot's proof, floor, runtime, and backend ideas can beat a
mature safe-systems implementation.

## Proposed current step

### [ ] RG-BASE — Freeze the fair comparator and baseline opportunity map

- **Why:** the 2x target is meaningless until its representative work,
  correctness oracle, stronger upstream comparator, target and aggregate rule
  are fixed before any Whitefoot performance result can influence them. A
  source audit has identified the layers but has not measured their importance
  on the available target.
- **Pinned upstream:** ripgrep 15.2.0, commit
  `e89fff89ac9af12e8d4ce9d5fd07beb408ca730f`, Unlicense or MIT. Use its
  committed dependency lock. Build the default regex surface with the exact
  recorded Rust toolchain in `release-lto`, once for the distributable target
  baseline and once with the strongest reproducible native target settings.
  Keep the official release executable as the user baseline. Optional PCRE2
  linkage must not change default-engine work.
- **Target:** begin with the available Apple M4 macOS aarch64 machine because
  it is a current supported Whitefoot host and can run the experiment now.
  Record exact model class, core count, memory, OS, power state, compiler,
  target features and executable identities without publishing device serials
  or other unique host identifiers. The resulting claim is target-specific;
  another target requires a later frozen replication, not an extrapolation.
- **Suite-selection rule:** before timing, derive a compact suite from pinned
  upstream benchmark families plus real repository-search behavior. It must
  include at least two independently sourced real code trees and one large-text
  corpus; single- and many-file cases; literal, required-literal regex,
  no-required-literal regex, case-insensitive and Unicode matching; low and
  material match counts; default ignore/file filtering; and normal result
  production. No one case or family may dominate the aggregate. Record why
  every case represents user work and freeze corpus and command bytes before
  reading comparative timing.
- **Correctness:** compare selected paths, matches, offsets or line/column
  records as applicable, context, relevant ordering, diagnostics and exit
  status with pinned ripgrep. Force stable non-color output. For intentionally
  unordered parallel output, compare a canonical record multiset while timing
  each executable's ordinary output path; deterministic modes compare exact
  bytes. Reject a case whose results cannot be compared independently of
  timing.
- **Measurement:** time complete process invocation, pattern construction,
  traversal, open/read, matching, formatting and consumed output. Separate
  controlled warm and cold cases; do not claim cold-cache results when macOS
  cache state cannot be restored symmetrically. Hold thread cap and a stated
  memory envelope equal. Interleave executions, use enough repetitions for a
  stable confidence interval, preregister timeout and outlier handling, and
  choose the faster upstream executable per case before any Whitefoot result.
- **Profile:** for every material family, divide elapsed work only as far as
  supported evidence permits among startup/pattern compilation, traversal and
  ignore handling, file I/O, matching, line/context accounting, output, worker
  scheduling and waiting, allocation, and kernel time. Record CPU utilization,
  bytes searched, result volume and peak memory; use target-available sampling
  or counters without inventing unavailable PMU precision.
- **Do:** create one self-contained `research/experiments/ripgrep/` baseline
  bundle containing the authorization packet, exact upstream and corpus
  identities, runner, correctness comparison, raw measurements, profile
  evidence and `RESULTS.md`. Bulk corpora, cloned upstream source and build
  products stay under `/Users/bytedance/do_not_scan`. The runner must be wired
  to one documented gate command and must not become general benchmark
  infrastructure.
- **Verify:** rerun the frozen suite from a clean experiment checkout; verify
  every output comparison before accepting timings; inspect that the faster of
  official and native-LTO ripgrep is used per case; and hostile-review the
  suite for a hidden favorable mode, omitted result work, mismatched regex or
  ignore semantics, asymmetric cache state, thread or memory advantage,
  corpus leakage, and an aggregate dominated by one case.
- **Accept:** the exact comparator, suite, target envelope, aggregate 2x rule,
  per-case regression guard, baseline timings and evidence-backed opportunity
  map are frozen and independently reviewable. The result may show little
  apparent headroom; that redirects later work but does not lower the umbrella
  goal. Completion authorizes no Whitefoot search implementation.
- **Stop:** stop after the upstream-only baseline and review. Do not write
  Whitefoot search source, prototype an alternative matcher or walker, change
  the compiler or specification, or design the parallel language/runtime in
  this step. If fair correctness or timing cannot be frozen, record the exact
  blocker and return for owner review instead of substituting an easier claim.

## Expected reverse pressure, not predetermined answers

The baseline may prioritize any combination of matcher strategy, pattern
specialization, I/O and traversal, file- or range-level work decomposition,
result construction, allocation, target code, or measurement noise. It may
also show that a suspected direction has no useful share of end-to-end time.

Later proposals must follow the evidence:

- a real CLI first makes `BOUND-1` concrete;
- a correctness-green accepted path first invokes `PERF-1` and the floor audit;
- measured parallel pressure may propose `PAR-1` and `PAR-4` together with the
  exact task, non-interference, determinism, failure and cost boundary;
- `PAR-2` or `PAR-3` enters only for a demonstrated subrange or reduction need;
- a proof or strategy consumer enters only after its exact retained-check,
  alias, effect or structural pressure is observed; and
- dynamic matcher, queue, result or buffer storage routes through the relevant
  storage item rather than an rg-specific compiler form.

The project is not a line-by-line translation obligation. A Whitefoot-native
algorithm is welcome when it preserves the frozen consumer behavior and
resource envelope. A compiler path keyed to ripgrep, a pattern, function,
corpus, source shape or benchmark identity is always disallowed.

## Not authorized

- No corpus download, benchmark execution, or experiment bundle before
  `RG-BASE` approval.
- No Whitefoot search implementation, regex engine, directory walker, CLI,
  filesystem boundary, threading runtime, parallel construct, or optimizer
  fact.
- No compiler, specification, conformance, project-law, or protected-evidence
  change.
- No reduction of the flagship to `--sort`, one large file, fixed-string
  search, a private agent trace, or another favorable subset.
- No unrelated project as a prerequisite.

## Parallel research

None. `RG-BASE` is the one proposed bounded research step.

## Completion

After independent review of the baseline bundle, update the Direction Outline
with only established measurements and replace this file with the next
`PROPOSED` ripgrep-derived step. That proposal may attempt the first faithful
current-language product path or investigate the single highest-decision-value
blocker exposed by the baseline. Only a later owner decision changes it to
`ACTIVE`.
