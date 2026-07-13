# Native F-SOA baseline harness

`run_baseline.py` is non-production experiment infrastructure.  It records only
the existing facts-off `F-SOA` baseline; it contains no AoS candidate, language
implementation, comparison, or performance verdict.

## Timing boundary

Every sample is a new native process and contains exactly one call to
`xlc_frontend_run`.  The two monotonic clock reads immediately surround that
call.  Reading and hashing the corpus and executable, computing the correctness
digest, serializing JSON, process startup, and process-exit reclamation are
outside the interval.

This is a **cold public-wrapper** interval.  It includes allocation, eager tape
filling, lexing, parsing, and semantic frontend work performed by the wrapper.
It is neither retained nor phase-isolated.

## Non-scoring validation

From the repository root:

```sh
python3 -B experiments/data-layout-owning-sequence/run_baseline.py self-test
python3 -B experiments/data-layout-owning-sequence/run_baseline.py smoke \
  --out-dir /tmp/xlang-fsoa-smoke
```

`self-test` builds in a temporary directory, launches two independent smoke
processes, checks their reports and digests, and attacks the source hash,
executable hash, dirty-tree, sample-digest, and score-lock rejection paths.
`smoke` preserves two samples at a new output path.  Both are marked
`not_a_score: true`; their elapsed values are harness diagnostics, not evidence
for a performance claim.

The CLI exposes guarded `freeze-lock` and `score` commands so the artifact
contract can be reviewed, but this restoration does not authorize or run them.
A future scored baseline requires a separately approved protocol, a committed
clean tree, an exact external lock, and explicit acknowledgement.

## Frozen build

The source is constructed exactly like
`compiler/test_lexer.py::compiler_source`, then pinned to 1,029,044 bytes and
SHA-256
`17c28914ec3cd109f0411cc8a83423623c1541be239e753e91144a66bea93f65`.
Stage 0 runs facts-off.  The harness rejects forbidden optimizer facts and any
raw IR differing from the 1,860,733-byte SHA-256
`23baa6cce795a7c8c21b66af2c2c01dbbeade8e40b5fe7dda64966db9f8e615a`.
It links with `/usr/bin/clang -O2`, without native-CPU, LTO, or PGO flags.

Per [A1](PROTOCOL_AMENDMENTS.md), every native arm must also use
`-Wl,-stack_size,0x4000000`.  The script records and binds the 64 MiB value and
fails closed on non-Darwin hosts until an equal Linux mechanism is defined.

## Artifacts

A completed campaign directory contains:

```text
manifest.json
raw.jsonl
compiler-source.xl
compiler-source.sha256
build/
  build.json
  clang.stdout
  clang.stderr
  fsoa.ll
  fsoa_sample
samples/
  sample-000.stdout
  sample-000.stderr
  ...
```

`raw.jsonl` is append-and-fsync per completed process.  Each sample verifies and
records the corpus and running-executable SHA-256 before the clock starts.
`manifest.json` is atomically replaced after every state transition and sample;
an exception marks it `invalid`.  `schemas/` defines the v1 sample, manifest,
and lock formats.

Correctness is the exact frozen public `FrontendReport` plus a SHA-256 digest of
its eleven fields encoded as little-endian unsigned 64-bit values after the
domain separator `xlang-fsoa-frontend-report-v1\0`.  Digesting occurs after the
clock stops.

## Limitations

- The public wrapper retains thirty tape allocations.  Fresh process exit
  bounds the leak per sample, but this cannot measure retained steady state or
  destruction.
- The interval combines allocation, initialization, and every frontend phase;
  it cannot attribute a change to layout, initialization, or allocator work.
- The digest covers the public report, not internal tape contents.  Full
  field-by-field equivalence remains mandatory for a future candidate.
- There is no AoS arm, balanced ordering, phase fixture, RSS/physical-footprint
  accounting, allocator counter, hardware counter, thermal record, or
  statistical analysis.
- The 64 MiB stack reservation is equal virtual capacity, not measured live
  memory; physical-memory analysis must use touched/resident pages.
