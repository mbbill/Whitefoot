# M3 Implementation Gate Audit

Status: local audit complete; not decision-ready.

This file records whether the four xlang-pending M3 tasks are harness artifacts
or real language/toolchain blockers.

## Current Verdict

Do not move forward into self-hosting yet.

Continue only the bounded M3 decision sprint. The next funded unit of work is
not `compiler/src`; it is the minimum democ/subset-S work needed to run the same
task set for xlang and Rust, followed by weak/middle/strong model trials.

If that minimum work is not worth doing, stop or pivot now.

## Blocker Audit

| task | local finding | harness issue? | minimum unblock |
|---|---|---:|---|
| `checked_integer_parser` | The task needs byte-string values, byte iteration, `u64` checked arithmetic, and recoverable parser errors. democ can tokenize string literals only for doc/trap text and has no byte buffer/slice value path. | No | Land byte-string values, `buffer`/`slice` or fixed array views, `u64` ops, and enough Result ergonomics to write a parser. |
| `arena_ast_builder` | The task is a direct proxy for `compiler/PLAN.md`'s AST module: pools plus handles. democ has neither pool/handle operations nor the fixed-capacity arena profile. | No | Specify and implement fixed-capacity `pool<T>`/`handle<T>` operations used by subset S, then add codegen/checking for push/at/len/handle equality. |
| `buffer_index_kernel` | The task maps to existing pending OP-4/OP-9 conformance cases. democ has no `buffer_new`, `index`, or `len` lowering. | No | Implement `buffer<T>`, runtime length, checked `index`, and `len`; promote OP-4/OP-9 pending cases. |
| `error_propagation_chain` | A superficial parser/codegen patch for `try` would be misleading because democ currently erases `Result<T,E>` type arguments before type checking. It could accept happy paths while missing the ERR-3 same-`E` rejection. | No | Preserve enough `Result<T,E>` type information through parse/type lowering, implement ERR-3 same-error checking, then lower `try` to Ok-bind/Err-return. |

## Why The Sprint Cannot Decide Yet

The current reference run proves only this:

- Rust reference submissions pass all seven current tasks.
- xlang reference submissions pass the three currently expressible smoke tasks.
- xlang cannot yet express four minimum-sprint tasks in the current democ subset.

That is a real project finding, but it is not the M3 verdict. The M3 thesis is
about model-written submissions, especially weak-model submissions. No
weak/middle/strong generated suite has been run yet.

## Next Gate

Run this sequence before any self-hosting compiler work:

1. Choose whether to implement the four minimum unblocks above.
2. If yes, promote the corresponding conformance cases from pending to runnable
   as each feature lands.
3. Require the xlang reference suite to pass all minimum M3 tasks.
4. Run weak, middle, and strong model suites with identical prompt and repair
   budgets for Rust and xlang.
5. Apply the pass/fail criteria in `DECISION_SPRINT.md`.

## Stop Conditions

Stop or pivot if any of these happen:

- The four minimum tasks require language features outside the stated subset-S
  implementation plan.
- The unblocks become a broad compiler rewrite before any model evidence exists.
- After the fair model run, Rust plus ordinary guardrails is within the sprint's
  weak-tier margin.
- xlang's prompt/spec tax makes first-shot success worse enough to erase any
  correctness or performance gain.

