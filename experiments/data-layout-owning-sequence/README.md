# E0.1: data layout and owning sequences

This experiment asks two separate questions, in this order:

1. can xlang store a flat record in a fixed buffer without changing the
   performance or semantics of existing primitive buffers and SoA programs?
2. can xlang express an affine, initialized-prefix owning sequence without
   adding initialization, growth, drop, or alias overhead to the no-grow hot
   path?

It does **not** assume that AoS should replace the compiler's current SoA tapes.
The present compiler is the protected baseline.  Capability adoption and xlc
layout migration have separate decision gates.

Current status: research plus isolated, non-production experiments.  This tree
contains a baseline-only native harness for non-scoring self-test/smoke runs.
An unconditional candidate exists only in a disposable worktree and has failed
hostile review on affine-fill semantics despite green repository tests.  The
production tree, specification, xlc, and teaching remain unchanged until
explicit owner confirmation after report review.  No candidate comparison or
scored result exists.

Files:

- `RESEARCH_REPORT.md` is the owner-review report and decision surface.
- `PROTOCOL.md` proposes arms, workloads, measurements, and decision rules; it
  is a draft until owner approval.
- `FLAT_DESIGN_CANDIDATE.md` describes the smallest no-new-syntax,
  no-implicit-Copy design candidate and explicitly forbids a feature flag.
- `BASELINE.md` records the pre-prototype source, layout, memory, and verification
  facts.
- `RESEARCH.md` records primary-source constraints and opposing data-layout
  evidence, with design inferences labeled as such.
- `HOSTILE_REVIEW_0.md` is the pre-prototype attack memo and its required
  resolutions.
- `HOSTILE_REVIEW_1.md` is the independent post-prototype rejection report;
  it records semantic, target, surface, and benchmark-protocol blockers.
- `HARNESS.md`, `run_baseline.py`, `native/fsoa_sample.c`, and `schemas/`
  define the reproducible native cold-wrapper `F-SOA` recorder.  This is
  isolated experiment infrastructure, not production implementation.
- `PROTOCOL_AMENDMENTS.md` records the equal-arm native stack requirement found
  while validating the baseline harness.
- `RESULTS.md` records the detached candidate's limited code-shape evidence and
  blocking hostile-review findings.  There is no measured performance result;
  smoke elapsed values are explicitly non-scoring.

No external model run is authorized by this directory.  A future default-writer
gate requires a new experiment-specific disclosure authorization; permission
given for an earlier port may not be reused.
