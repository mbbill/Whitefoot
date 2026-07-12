# M3 Current Results

Status: local reference scaffold plus blocker audit; not decision-ready.

Last local command:

```
python3 m3/harness/run.py --suite reference --out /private/tmp/xlang-m3-reference.jsonl
python3 m3/harness/score.py /private/tmp/xlang-m3-reference.jsonl
```

## Reference Summary

| suite | language | runnable | passed | pending | current meaning |
|---|---:|---:|---:|---:|---|
| reference | Rust | 7 | 7 | 0 | Rust can express and pass every current task prompt. |
| reference | xlang | 7 | 7 | 0 | xlang passes every current task prompt (arena via the STOR-1 shape-C append-only pool, pending owner ratification). |

The six runnable xlang tasks are:

- `checked_loop_sum`
- `value_match_result`
- `noalias_add`
- `checked_integer_parser` (unblocked 2026-07-09: buffer<u8>, Result payloads, signed literals)
- `buffer_index_kernel` (unblocked 2026-07-09: buffer_new/index/len; OOB path shown as a bounds-guarded branch; the executed-trap variant is conformance case op4-trap-index-oob)
- `error_propagation_chain` (unblocked 2026-07-09: try/ERR-3, same-E enforced)

- `arena_ast_builder` (unblocked 2026-07-09: append-only struct-of-arrays pool —
  STOR-1 shape C in `optimizer-language-research/notes/stor1-ruling-request.md`;
  no recycling so no UAF class; owner ratification of the shape still requested)

Verification-semantics note: Rust references print `ok` and are checked on stdout;
xlang has no print surface in the kernel subset, so xlang references are checked on
exit code (`expected_xlang: {"exit": 0}` in `tasks.jsonl`). Every xlang assertion
routes failure through `check ... else trap` (abort, nonzero exit), and the
discriminator was negative-tested: mutating the expected sum in
`buffer_index_kernel.xl` makes the harness report `fail exit -5 != 0`.

## Decision Readiness

The current evidence is **not** sufficient for a continue/stop decision.

Two independent blockers remain:

1. **Language/toolchain surface blocker: CLEARED 2026-07-09** — xlang runs all seven tasks (arena via STOR-1 shape C, ruling memo filed).
2. **Model evidence blocker**: no weak/middle/strong generated submissions have been run yet. This is now the dominant blocker.

The strict scorer makes this explicit:

```
python3 m3/harness/score.py /private/tmp/xlang-m3-reference.jsonl \
  --required-suite weak --required-suite middle --required-suite strong \
  --require-decision-ready
```

Expected current result: nonzero exit with `decision_ready: false`.

The harness now also supports multiple generated trials per task and
`--min-trials-per-task` readiness checks, so model-tier evidence can be judged
against the fixed-budget protocol rather than a single cherry-picked source.

## Current Interpretation

This is not a pass for xlang and not a final failure of the project thesis. It is a
clear finding that the project cannot honestly run the stated M3 decision sprint
until the missing xlang subset lands or the sprint is narrowed with an explicit
owner decision.

The next decision is therefore smaller than continue/stop:

- either resolve STOR-1 (pool/handle owner ruling) so `arena_ast_builder` can be
  expressed, then run the model-tier sprint on all seven tasks,
- or run the model-tier sprint on the six runnable tasks now and report the arena
  task as an explicit expressibility gap.

See `IMPLEMENTATION_GATES.md` for the original blocker audit (2026-07-08). Its
key finding held up: the pending tasks were real democ/subset gaps, not harness
mistakes. The 2026-07-09 unblock implemented `try` with `Result<T,E>` type
arguments preserved, exactly to avoid the ERR-3 false-positive risk it flagged
(conformance case err3-neg-try-different-error-type is green).

## Working Recommendation

Do not proceed to self-hosting compiler work yet.

The minimum M3 unblock is now done except for pool/handle support, which is
gated on an owner ruling (STOR-1 rejects untyped slot recycling; the compliant
shape needs to be decided, not just coded). The next unit of work is Phase C:
run the weak/middle/strong model sprint on the six runnable tasks under the
fixed prompt/repair budget, alongside the Phase B channel benchmarks.

## Phase B status (2026-07-09)

All three fact channels are built into democ and measured against real rustc
with opaque boundaries (details in `optimizer-language-research/implementation/decision-gates.md`):

| channel | delta vs Rust obvious shape | delta vs expert Rust |
|---|---|---|
| effect rows -> fn attrs (2) | O(n) -> O(1) at opaque boundaries vs default build | tie vs fat-LTO, at per-file build cost |
| scoped alias from ownership (1) | 2.0x at n=8, parity >= 32; 17x code size | 1.17x at n=8, parity above |
| checked-law reassociation (3) | 3.3x (sat-add reduction) | tie — but expert shape is UNCHECKED; false laws are refuted compile-time in xlang |

The R0 condition ">= 1 robust Phase-B delta" is plausibly met (channels 2 and 3).
The remaining R0 leg is the W1 distributional claim: the Phase C model-tier
sprint on the six runnable tasks. That requires actual weak/middle/strong model
API runs — owner infrastructure.
