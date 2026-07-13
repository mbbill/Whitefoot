# utf8parse benchmark harness

This harness implements the build, corpus, process-order, raw-recording, and
statistical rules frozen in `PROTOCOL.md`.  It has two deliberately separate
entry points.

## Non-scoring validation

Run this before a generated source exists, or after changing harness plumbing:

```sh
python3 benchmark.py smoke --out-dir /tmp/utf8parse-smoke
```

Smoke mode builds the Rust binary with the `smoke-shim` feature, so both xlang
slots call a Rust-backed validation shim.  It uses a small corpus and one copy
of each order permutation. Every JSON campaign, corpus, schedule, and block
record says `not_a_score: true`; it writes `SMOKE_ONLY.json`, and it neither
invokes `analyze.py` nor emits a
performance verdict.  Smoke corpora are capped at 16 MiB. Its timings are only
evidence that process isolation,
ordering, buffer validation, digest validation, and raw logging work.

## Preregistered scoring campaign

Only run this after the first correctness-green source has been frozen and its
byte-for-byte SHA-256 has been recorded:

```sh
/Applications/Xcode.app/Contents/Developer/Library/Frameworks/Python3.framework/Versions/3.9/bin/python3.9 benchmark.py score \
  --trace-manifest /absolute/path/to/run/frozen/trace-manifest.json \
  --out-dir /absolute/path/to/new-campaign-directory \
  --acknowledge-preregistered-score
```

The orchestrator derives the candidate path and SHA-256 only from that frozen
manifest. Before any proof report or build, it verifies `config.json`,
`trace.jsonl`, every completed-round record and artifact hash, `source.sha256`,
and the frozen source. It copies the manifest, config, trace, source, and source
hash into `generation-freeze/`, then revalidates the original binding after the
last measurement block.

There are no scoring knobs for corpus size or repetition count.  Score mode
compiles the same source facts-on and facts-off, using Clang `-O3` with the
generic/default CPU target, then links the ordinary Cargo release build.  The
timed Rust arm calls `rust-baseline::parse_into` directly.  Native-CPU flags,
Cargo profile overrides, and environment-provided Rust flags are rejected or
removed and recorded before measurement.

The final link command is Cargo's ordinary release path via
`cargo rustc --bin bench --release --locked --offline`; the only trailing
`rustc` arguments are one `-C link-arg=...` for each preregistered xlang object.
The verbose log must show `opt-level=3` and no native-CPU, LTO, or PGO flag.
Cargo, rustc, and Clang use the same absolute, hash-locked executables as the
generation launcher. The preflight also hashes the cached `utf8parse` 0.2.2 `.crate`,
byte-compares every packaged file with Cargo's extracted registry source tree,
and confirms the verbose build actually referenced that verified tree.

Only after the generation freeze binding passes, each facts mode is compiled
once without and once with a fresh proof-report sink. The LLVM IR must be
byte-identical across report/no-report calls. Both structured reports, their
hashes and summaries, and the invariant result are recorded before linking.

The campaign directory is append-free: the command refuses to overwrite it.
It contains at least:

- `metadata.json`: generation binding, repository dirty manifest, crate/source
  identity, proof accounting, hashes, host/tool/power state, complete build
  commands, corpus identity, and campaign validity;
- `schedule.json`: the frozen permutation schedule;
- `raw.jsonl`: one fsync'd record from each fresh process block, including all
  three elapsed nanosecond counts and before/after power/thermal observations;
- `block-NN.log`: native stdout/stderr for each process;
- `analysis.json`: the 10,000-resample stratified bootstrap report, raw sample
  table, order/position summaries, ratios, intervals, and preregistered verdict.

Any process failure, malformed or missing row, output mismatch, source/corpus
hash mismatch, power-source transition, available thermal-state transition,
or interruption changes `metadata.json` to `status: invalid`.  Logs are kept;
the directory cannot be resumed or silently repaired.  A rerun must use a new
directory and record its reason alongside the study results. Scoring blocks
have no duration timeout: a slow sample alone is retained and never invalidates
the campaign.

A permitted complete rerun must add both
`--rerun-of-invalid-campaign /path/to/prior-run` and a nonempty
`--rerun-reason '...'`. The prior metadata must say `status: invalid`; its path,
hash, and the appended reason are bound into the new metadata before measuring.

## Direct Rust driver interface

`harness/src/bin/bench.rs` also exposes two internal commands used by the
orchestrator:

```text
bench prepare-corpus --mode score|smoke --bytes N --output PATH
bench run-block --mode score|smoke --corpus PATH --expected-sha256 HEX \
  --block-index N --order facts-on,facts-off,rust
```

The scoring binary refuses smoke mode and the smoke-shim binary refuses score
mode, preventing a validation build from entering the primary campaign.
