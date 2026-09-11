# Design tree change log

Newest first. One entry per approved tree change. Format:
`skill/templates/log-entry.md`.

## 2026-09-11 Migrate the root and the checks-and-proofs subtree

Nodes: +tree, +tree/checks-and-proofs, +tree/checks-and-proofs/obligation-discharge, +tree/checks-and-proofs/obligation-discharge/goal-decomposition, +tree/checks-and-proofs/obligation-discharge/loop-fact-retention, +tree/checks-and-proofs/obligation-discharge/writer-trap-surface, +tree/checks-and-proofs/certificate-fold, +tree/checks-and-proofs/requires-entry-contract, +tree/checks-and-proofs/requires-entry-contract/requirement-enforcement
Origin: migration
Summary: Pilot migration from `mcts_mem/whitefoot.md` and `mcts_mem/whitefoot/checks-and-proofs/` at commit 3016842. Live summary bullets became decisions; `.alt` nodes and `replaced` moves became `Rejected:` lines with their recorded reasons; dated facts, measurements, and pitfalls were dropped, since their owners are the results records and `compiler/README.md`. One dated statement was carried as a decision because no live bullet stated it: an integer-typed named const is an affine atom. One dated statement was not carried because the active PRF-1 redundancy rule contradicts it: the 2026-07-11 principle that an unused explicit check is never a hard failure; it appears as a rejected alternative instead. Where a live bullet carried no recorded reason, the reason was taken from the nearest recorded rationale and should be confirmed. The pilot exists to calibrate the leanness filters before the remaining twelve subtrees are migrated. This entry transcribes; it decides nothing new. The owner reviews it as a tree diff.
Code: none; this change touches only the design tree.
