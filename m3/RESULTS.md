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
| reference | xlang | 6 | 6 | 1 | xlang passes six of seven minimum-sprint tasks; only the arena task is unexpressed. |

The six runnable xlang tasks are:

- `checked_loop_sum`
- `value_match_result`
- `noalias_add`
- `checked_integer_parser` (unblocked 2026-07-09: buffer<u8>, Result payloads, signed literals)
- `buffer_index_kernel` (unblocked 2026-07-09: buffer_new/index/len; OOB path shown as a bounds-guarded branch; the executed-trap variant is conformance case op4-trap-index-oob)
- `error_propagation_chain` (unblocked 2026-07-09: try/ERR-3, same-E enforced)

The one xlang-pending task is:

- `arena_ast_builder`: current democ lacks the `pool<T>`/`handle<T>` or arena-backed AST shape required by `compiler/PLAN.md`. Note STOR-1 explicitly rejects untyped index-pool recycling; the xlang shape needs an owner ruling before implementation.

Verification-semantics note: Rust references print `ok` and are checked on stdout;
xlang has no print surface in the kernel subset, so xlang references are checked on
exit code (`expected_xlang: {"exit": 0}` in `tasks.jsonl`). Every xlang assertion
routes failure through `check ... else trap` (abort, nonzero exit), and the
discriminator was negative-tested: mutating the expected sum in
`buffer_index_kernel.xl` makes the harness report `fail exit -5 != 0`.

## Decision Readiness

The current evidence is **not** sufficient for a continue/stop decision.

Two independent blockers remain:

1. **Language/toolchain surface blocker (mostly cleared 2026-07-09)**: xlang now runs six of the seven tasks, including all four previously-blocked minimum-sprint tasks except `arena_ast_builder` (which awaits the STOR-1 pool ruling).
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
