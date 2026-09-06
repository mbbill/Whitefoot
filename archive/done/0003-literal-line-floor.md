# 0003 — WF-LITERAL-LINE

Status: DONE — integrated into `main` on 2026-08-05

Authority: separate owner-approved bounded research on 2026-08-05 advancing
`PERF-1`, `FLOOR-1`, and `FLOOR-2` without authorizing specification,
compiler, proof, intrinsic, runtime, system-capability, regex, thread, or
end-to-end `wfgrep` changes.

## Outcome

The active v0.17 language expresses the frozen runtime-haystack/runtime-needle
literal line contract and all four controls produced the same exact records.
The same-Clang primary ratio was C/Whitefoot 0.953500
[0.922344, 0.960942]: a precise directional C advantage, but neither strict
practical parity nor a material Whitefoot loss under the frozen bands.

Pinned Rust `memmem` measured 0.136389x the same-toolchain naive Rust time
[0.134852, 0.138298], or descriptively 7.33x throughput, with the expected
AArch64 NEON packed-pair mechanism. Because the primary practical-parity
prerequisite was not met, this is not promoted to the protocol's formal
material algorithmic-ceiling classification. It is not ripgrep, end-to-end,
or 2x-wfgrep evidence.

## Landed commits

- `7adb0faa55678997fdad3ddef15a311579c9d80a` — frozen preregistration,
  apparatus, and pre-timing code-shape inspection; and
- `10a7a23f05766ab34ce375271011fc2104fe19e9` — result and retained raw
  evidence; and
- `b44b737` — admitted MCTS facts and validation closure; `main` was
  fast-forwarded through this linear history.

## Canonical evidence

- `research/experiments/literal-line-floor/PROTOCOL.md`
- `research/experiments/literal-line-floor/CODE_SHAPE.md`
- `research/experiments/literal-line-floor/RESULTS.md`
- `research/experiments/literal-line-floor/raw/wf-literal-line-floor-1.jsonl`
- admitted facts in `mcts_mem/whitefoot/pattern-doctrine.md` and
  `mcts_mem/whitefoot/checks-and-proofs.md`

## Validation

The create-once AC-powered run retained all 128 scheduled samples. Each
variant occupied each process position eight times; every work identity and
hash matched; order-class and time-block sensitivity checks preserved the
directions. The experiment-local gate, MCTS-Mem lint after tree edits, and
complete repository `make check` gate passed before integration.

## Follow-up boundary

Do not open proof or compiler work from the unresolved roughly 5% scalar gap.
The much larger observed lever is the pinned library algorithm, but this result
authorizes only a future bounded proposal to test an efficient taught literal
search pattern or library capability. It remains independent of the ongoing
system-capability architecture work.
