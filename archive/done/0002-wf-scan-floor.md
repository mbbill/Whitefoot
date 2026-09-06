# 0002 — WF-SCAN-FLOOR

Status: DONE — integrated into `main` on 2026-08-05

Authority: separate owner-approved bounded research advancing `PERF-1`,
`FLOOR-1`, and `FLOOR-2` without authorizing specification, compiler, proof,
intrinsic, runtime, matcher, or system-capability changes.

## Outcome

The active v0.17 language and ordinary compiler express both frozen
same-algorithm scanner shapes at practical parity with same-Clang C:

- full Boolean-dataflow scan: 0.999258x C [0.996948, 1.002261];
- four early-exit scans: 1.000848x C [0.998128, 1.008784].

LLVM removes the guard-dominated bounds trap from both optimized Whitefoot
targets. The full target has the same width-16 vector/reduction structure as C;
the early target has the same four scalar loops. This closes only the narrow
single-buffer language-floor question. It is not matcher, I/O, end-to-end grep,
or 2x-ripgrep evidence.

## Landed commits

- `a965cb4b611b03273a68d1e98e7670c3ae4626e6` — frozen preregistration;
- `e9ae083b6a28e2e022fa0354b5782d79ced809aa` — result, raw evidence, and MCTS
  facts; and
- `28b5280` — merge into `main`.

## Canonical evidence

- `research/experiments/wfgrep-scan-floor/PROTOCOL.md`
- `research/experiments/wfgrep-scan-floor/RESULTS.md`
- `research/experiments/wfgrep-scan-floor/raw/wf-scan-floor-1.jsonl`
- `mcts_mem/whitefoot/pattern-doctrine.md`
- `mcts_mem/whitefoot/checks-and-proofs.md`

## Validation

The create-once AC-powered run retained all 180 scheduled samples and passed
the independent correctness oracle. Order, position, outlier, and
leave-one-order-out review did not change the frozen practical-parity
classification. The experiment-local gate, MCTS-Mem lint, and complete
repository `make check` gate passed before integration.

## Follow-up boundary

Future wfgrep work should take its next hypothesis from a measured matcher,
I/O, reconstruction, output, or parallel-pipeline gap. This completed record is
independent of `docs/ongoing/0001-system-capability-architecture.md`.
