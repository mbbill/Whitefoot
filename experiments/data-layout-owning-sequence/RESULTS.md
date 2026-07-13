# E0.1 results ledger

Status: research plus isolated non-production experimentation.  No production
implementation or scored timing has started.

Completed:

- pre-prototype repository verification is green;
- exact compiler source/count/layout/memory baseline is recorded;
- layout, initialization, growth, allocation, owner/lifetime, and capacity-policy
  confounds are identified; valid causal controls are not yet complete;
- the baseline-only native harness passes self-test and a two-process non-scoring
  smoke with frozen source/IR/executable/correctness hashes;
- an unconditional candidate was built only in the detached worktree
  `/private/tmp/xlang-e01a-candidate`; its repository-wide `make check`, 73
  checker tests, 10,000-case modelcheck, field-only IR shape, two 64-bit target
  layout folds, and four unchanged-source raw-IR pins pass;
- independent hostile review rejected that candidate as a production design.

The candidate's green tests do not close E0.1a.  Known blockers include:

- `buffer_new<Record>(n, move seed)` evaluates one affine record then stores it
  N times, contradicting `Flat != Copy`; a nested move inside an outer fresh
  constructor has the same contraction;
- whole-row use/move is not rejected by the checker in every expression context;
- the disposable backend's object-size and pointer facts are correct only for
  its frozen 64-bit experiment targets, not a target-generic or 32-bit claim;
- sanitizer, allocation-fault, cross-architecture execution, and complete
  internal-tape equivalence gates have not run.

No performance result, language adoption, xlc migration, or default-teaching
claim exists.  `BASELINE.md` numbers are static accounting and harness smoke
elapsed values are explicitly non-scoring.

Next stop: owner review of `RESEARCH_REPORT.md`, especially the unresolved
record-initialization semantics and corrected attribution protocol.  No further
candidate iteration implies production authorization.  Production E0.1a still
requires explicit confirmation, and E0.1b remains closed.
