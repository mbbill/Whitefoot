# E0.1: data layout and owning sequences

This experiment asks two questions that are separate unless the selected initializer
requires a builder:

1. ca Whitefoot store a flat record in a fixed buffer without changing the
   performance or semantics of existing primitive buffers and SoA programs?
2. ca Whitefoot express an affine, initialized-prefix owning sequence without
   adding initialization, growth, drop, or alias overhead to the no-grow hot
   path?

It does **not** assume that AoS should replace the compiler's current SoA tapes.
The present compiler is the protected baseline.  Capability adoption and wfc
layout migration have separate decision gates.

Current status: historical research evidence; no work is authorized. D11
replaces the earlier monolithic upstream gate with a bounded G0-Core followed
by an exact lock for each family. E0.1 remains suspended before Lock A. G0-Core
plus the dense-family Lock A are necessary but not sufficient for a later owner
decision to lift that pause, and the old fixed-record paired protocol does not
restart automatically. A dense-family lock must explicitly retain, revise, or
supersede every relevant arm and measurement here. This tree
contains a baseline-only native harness for non-scoring self-test/smoke runs. The
first unconditional candidate was executed only in a disposable worktree and failed
hostile review on affine-fill semantics despite green repository tests. Its exact
reviewed source is now archived here; no executable candidate semantics entered the
production toolchain. Separately authorized repairs at `7438e17` (checker) and
`50a1ddd` (parser) enforce existing language rules and do not select an E0.1 design.
The specification, wfc layout, and teaching remain unchanged until explicit owner
confirmation. Code-shape and raw-IR identity were compared, but no scored timing or
performance comparison exists.

Files:

- `RESEARCH_REPORT.md` is the historical owner-review report and evidence record.
- `OWNERSHIP_ROUTE_PROTOCOL.md` is the suspended historical paired protocol for deciding whether
  declarative Copy or affine fixed-storage with a full-initialization-only
  transient builder should advance to the existing layout experiment. It may not
  enter Lock A, is not preregistered, and authorizes no candidate implementation
  or external run.
- `OWNERSHIP_ROUTE_HOSTILE_REVIEW.md` records the three-scope adversarial review,
  all blocking dispositions, and the exact protocol hash that passed final
  re-review. The pass qualifies the draft for owner review only.
- `PROTOCOL.md` preserves superseded arms, workloads, measurements, and decision
  rules as input to a future dense-family lock; it is not an active protocol.
- `FLAT_DESIGN_CANDIDATE.md` records the reopened ownership/initialization candidate
  space and explicitly forbids a feature flag; it selects no route.
- `BASELINE.md` records the pre-prototype source, layout, memory, and verification
  facts.
- `RESEARCH.md` records primary-source constraints and opposing data-layout
  evidence, with design inferences labeled as such.
- `HOSTILE_REVIEW_0.md` is the pre-prototype attack memo and its required
  resolutions.
- `HOSTILE_REVIEW_1.md` is the independent post-prototype rejection report;
  it records semantic, target, surface, and benchmark-protocol blockers.
- `REVIEW_RESPONSE.md` disposes the owner-advisor review, including durability,
  design-history, grammar, storage-taxonomy, and pattern-doctrine findings.
- `DETACHED_CANDIDATE.patch` is the exact 57,547-byte binary diff reviewed by
  `HOSTILE_REVIEW_1.md`, based on Git `58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1`;
  its SHA-256 is
  `bed070414f9552ea105857404d6d1296b98542a28cc65fa6899a197830e6774e`.
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
