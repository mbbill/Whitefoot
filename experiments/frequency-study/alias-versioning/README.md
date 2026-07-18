# Alias-versioning frequency calibration

This directory contains a **calibration-only** collector for the Leg-A
alias-guard frequency proxy. It does not contain a crate corpus and it does not
make a frequency claim.

Run the checked calibration from the repository root:

```sh
python3 -B experiments/frequency-study/alias-versioning/collect.py --pretty
python3 -B -m unittest discover -s experiments/frequency-study/alias-versioning -p 'test_*.py'
```

The collector compiles the existing
`experiments/scoped-alias-channel/rust_kernels.rs` benchmark at `-O3` into a
temporary optimized LLVM IR file, deletes that IR when finished, and writes a
single JSON document. On the calibrated rustc 1.91.1 / LLVM 21.1.2 toolchain,
the two obvious/rebound kernels produce two first-party runtime alias-versioned
loops. Success requires the complete calibration fingerprint: two raw, two
validated, and two first-party memchecks; zero rejections; 26 conflict
predicates; and 52 pointer comparisons. Any fingerprint drift exits 1;
compiler or input failure exits 2.

An existing optimized IR file can be inspected without compiling the fixture:

```sh
python3 -B experiments/frequency-study/alias-versioning/collect.py \
  --ir path/to/crate.ll --source-root path/to/crate --pretty
```

## What is counted

A candidate must be an actual basic block whose name starts with LLVM's
`vector.memcheck` convention, and its CFG must reach a `vector.body` block.
Mentions in comments or metadata do not count. The report records optimized
function instances, memcheck successors, conflict predicates, pointer
comparisons, and the outermost inlining debug location. `--source-root` marks
and relativizes first-party locations. LLVM comments are stripped with quoted
strings and identifiers preserved before any of those facts are selected. CFG
reachability uses a finite seen-set over every parsed block, with no depth
cutoff. Empty, comment-only, or arbitrary non-IR input is rejected with exit
status 2 instead of being reported as a clean zero; recognizable LLVM modules
containing declarations but no definitions remain valid zero-function inputs.
Malformed or nested definitions, duplicate block labels, and unrecognized
top-level text are also rejected rather than partially analyzed.

## Limits before a real corpus run

- `vector.memcheck` is an LLVM naming convention, not a stable API; the
  calibration deliberately fails closed on toolchain drift.
- This is a static optimized-instance count. It says nothing about runtime
  hotness, trip counts, or whether checks matter to application performance.
- A runtime memcheck is only a candidate for Whitefoot's channel. Same-owner
  subviews, raw pointers, unsafe code, and FFI may require facts Whitefoot does not
  currently provide; every real-corpus hit needs source audit.
- Monomorphization can produce several optimized instances from one source
  loop. A corpus collector must report both instance and unique-source counts.
- Inlined standard-library or dependency code is not first-party merely
  because it appears in the root object. Debug-location attribution is useful
  but imperfect for macros and generated code.
- This calibration does not define a corpus sampler, denominator, code-size
  counterfactual, confidence interval, or promotion threshold. Those belong in
  the separately reviewed Leg-A protocol.
