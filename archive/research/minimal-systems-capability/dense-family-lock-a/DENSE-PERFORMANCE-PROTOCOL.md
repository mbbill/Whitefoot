# Dense Family Lock A performance protocol

Status: research-only frozen protocol; candidate construction, Candidate Freeze B, pilot execution, scoring, held-out access, and production work are not authorized.

## Frozen input boundary

- Exact contract registry: 303 rows, SHA-256 3016206708a63b858b655e81de0e5c08e21055b0c4aa4c1ce37c2561c73e3418.
- Contract generator SHA-256: 31dda5ccfd33202860022946fdf456404b104a14118ffcd286406595e2da2d06.
- Candidate operations SHA-256: e6309a06014fbc573974786e91597344d29c665b463bd4589553d1e8a55812bd.
- Candidate bindings SHA-256: efb9336256340fd4b49177ba738fa3de1099f3e9f8157e77b6232979afee01d3.
- Candidate lifecycle SHA-256: 63f6c1e0cad521ea718b40b9570857e2401a2e83e4d36d1d58785b7276a041df.
- Common substrate SHA-256: 99fa5fc0c0ad44033c360027a0b2d5caf2bdb65253013776995f21e145e28e3f.
- OD4 policy SHA-256: 073c206be16fb9d85cfd7d90bd3743c21633b30ef1a93fd21828d3a1a5938bdd.

Any input-byte change blocks regeneration until this protocol is reviewed and explicitly repinned.

## Exact coverage

- Exact dispositions: {'EXCLUDED': 9, 'FUNCTIONAL_ONLY': 144, 'STRUCTURAL_ONLY': 13, 'TIMED_PRIMARY': 137}.
- Standalone operation gates: 97 including the scoped-consume protocol gate; every executable exact member has an independent same-shape Rust-floor gate.
- Matrix cells: 520 total, 502 primary timed; roles {'ARITHMETIC_STRUCTURAL': 3, 'BYTE_BOUNDARY_PRIMARY': 10, 'OD4_SCOPED_PRIMARY': 4, 'PAYLOAD_SEPARATOR_PRIMARY': 18, 'PRIMARY_OPERATION': 470, 'PROTECTED_STRUCTURAL': 15}.
- Timed payload separators: {'AFFINE24': 2, 'AFFINE256': 2, 'AFFINE64': 2, 'BEHAVIOR': 2, 'ROW24': 2, 'ROW56': 2, 'U64': 2, 'U8': 2, 'ZST': 2}.
- Sort shapes, edit indices, retain 10/50/90, clone_from length relations, splice range/replacement relations, traversal modes, byte boundaries, payloads, targets, and exact operations are never pooled.
- B-FIX, B-P2, H-FLATSET, W-SMALL, and W-GAP each have AArch64, x86-64, and i686 structural cells.
- ZST operation latency is timed where registered; all allocator call and byte fields are exact structural zero and can never supply benefit.

## Owner branches and common substrate

- Active power branches: 8 = OD1(2) x OD2(2) x OD3(2), all under OD0 common substrate, OD4 eager plus scoped consume, and OD5 no crossover.
- Blocked or reopen-required alternatives: 4 (OD0 separate, OD4 eager-only, OD4 persistent lazy, OD5 crossover).
- All five arms bind the same sealing, generic-call, reborrow, result-provenance, checked allocator, affine interval, owning cursor, and cost hashes. A private allocator or cursor rejects the arm.

## Statistics and endpoints

- One global alpha family totals 0.01: 0.005 for the 25 directed noninferiority claims and 0.005 for one global Holm benefit family. No candidate pair or cell receives a fresh alpha allocation.
- Every selected target and exact primary cell must independently have the directed candidate/Rust raw scheduled-mixture NI decision pass at 1/5000; exact integer cross-products are used and no aggregate masks an operation.
- Power uses six byte-identical Rust pseudo-treatments in the exact Williams and five-salt schedule, retains four raw whole-cycle supports, integrates the exact hidden treatment mapping, injects exact 17/20 timing or 4/5 memory alternatives, and computes a conservative union-bound selection-power lower bound by exact finite DP.
- NI and benefit use the globally valid scheduled-mixture tail p-value: p=1 when 2*s<=n and the exact Binomial(n,1/2) upper tail otherwise. Benefit then uses one global exact-rational Holm step-down family. Threshold ties and unusable fixed slots count as failures.
- Period, predecessor, row, salt, and cycle summaries are descriptive only; no fitted adjustment, logarithm, residual, or covariance enters inference, power, dominance, or selection.
- Trace and per-operation p50/p95/p99 plus target hardware counters are descriptive. Missing counters are explicit NOT_AVAILABLE evidence, never zero or imputed.

## Construction boundary

- Explicit operational blockers: 27.
- `earliest_blocked_stage` is cumulative across REFERENCE_PILOT, CANDIDATE_CONSTRUCTION, and CANDIDATE_FREEZE_B; descriptive counter reporting is a nonselection side stage.
- Per-branch reference-pilot prerequisite counts: [8, 9]; cumulative construction-gate counts: [21, 22]. The extra prerequisite is the x86 runner in dual-native branches.
- The reference pilot must close as feasible before the first candidate prompt; its row names the complete per-branch prerequisite manifest, not a two-blocker shortcut.
- Author, service, disclosure, custody, equal resources, and repair rules are not yet defensibly frozen; no arbitrary default is supplied.
- After a later exact freeze, inability to build or pass within the frozen equal protocol is a mechanism result. It does not permit another author, service, mechanism, resource extension, or tuning round.

## Generated artifact hashes

- DENSE-PERFORMANCE-ALGORITHMS.tsv: 136786 bytes, SHA-256 65c531a4b77338d1b651b539c2ca8c6aa2e122f92168b3c234a814973a3ef887.
- DENSE-PERFORMANCE-ALLOCATORS.tsv: 1556 bytes, SHA-256 cc218b4330640a9cad12eb5c9c2c14e5cfdaf912fdfd3bf6cb96e753d6f5e5df.
- DENSE-PERFORMANCE-BLOCKERS.tsv: 13658 bytes, SHA-256 3fda2ab4a24189f58ddd83e425a2504be4b52f3a7830721e413015695d0f8af3.
- DENSE-PERFORMANCE-COMMON-SUBSTRATE.tsv: 7176 bytes, SHA-256 23b2b1f652f7d28f6d9722ae082f10bef8c797c59a63d507fba579ede87fbc3e.
- DENSE-PERFORMANCE-CONTROLS.tsv: 5077 bytes, SHA-256 1bc594003697c3868e0ac42720de56df1680922f22426b090ad12fa029eeea09.
- DENSE-PERFORMANCE-COUNTER-POLICIES.tsv: 2494 bytes, SHA-256 89a9212a6982b4e37c211e2dcc3641be55bd64fcdaa9e1d70720761c16768117.
- DENSE-PERFORMANCE-DISPOSITIONS.tsv: 238023 bytes, SHA-256 5899086d5ed5b9ac75397b21994d741972f2fef41a443a7e577f0e33e87bb8c6.
- DENSE-PERFORMANCE-DISTRIBUTIONS.tsv: 9240 bytes, SHA-256 54dc16eec707b918321872ce7c05afee4ceafe258a87a8080df998c4a1be1c72.
- DENSE-PERFORMANCE-ENDPOINTS.tsv: 4863 bytes, SHA-256 0018182e1e15f1a18725742cf0fa3ee08dbf05e900603e29b13a4faa74934c5a.
- DENSE-PERFORMANCE-FACTS-POLICIES.tsv: 1231 bytes, SHA-256 899925e9744f472732a8efa102cd53306772706f0fb9ead369969bcae2e319cc.
- DENSE-PERFORMANCE-FAILURE-POLICIES.tsv: 815 bytes, SHA-256 bd6331cc05d6af867a77a416644a92ed143cab669c9145cea4e39089128af742.
- DENSE-PERFORMANCE-GENERATORS.tsv: 769 bytes, SHA-256 bf9ff60553e0157ba6a92fbf0cfa672e81b0b8668db8abdcc3c56c631f543f9e.
- DENSE-PERFORMANCE-GROWTH-POLICIES.tsv: 1813 bytes, SHA-256 768645fd68ed1de1699dac7f92d30b51ebb3ba99efc1b8a294368255f0f4c431.
- DENSE-PERFORMANCE-INPUT-DESCRIPTORS.jsonl: 1382806 bytes, SHA-256 880a1231311bb8e1132e7d0237c844455cd6f7c99a30eb8bff4d396ad8d12722.
- DENSE-PERFORMANCE-INPUTS.jsonl: 265676 bytes, SHA-256 65ed14b61cf2f016bf75c49abe165e5a6b2245e451acafb67d693512bc83119f.
- DENSE-PERFORMANCE-LAYOUTS.tsv: 1445 bytes, SHA-256 de5a054494d944ab70cd3aac56c2dfff6ddfa6f2271f030064ed2642ff198768.
- DENSE-PERFORMANCE-MATRIX.tsv: 963557 bytes, SHA-256 8d9f376ed9cb90f3fe8aa9c55c3861bc775430d948521a264630178cac28a187.
- DENSE-PERFORMANCE-OPERATION-GATES.tsv: 71499 bytes, SHA-256 2a6faa63f95b9140c0f8203022c9526a83681bf4f27b6fd0a270b151b35245a9.
- DENSE-PERFORMANCE-OWNER-BRANCHES.tsv: 5478 bytes, SHA-256 c48bbc27e51db13094cd4486dddf09001b8c2725cd38c2922f7dbbf3b94f09f5.
- DENSE-PERFORMANCE-PAYLOADS.tsv: 2356 bytes, SHA-256 68ccb0cb9e85e2b83e6cdb4e586a8fd789f264966f1d9f71881b42338dfd52e0.
- DENSE-PERFORMANCE-REFERENCE-ROUTES.tsv: 7325 bytes, SHA-256 ae89b2b13e4e438de70a70006c43d8d0f7044227f54dc81bf5df513c094cf6a7.
- DENSE-PERFORMANCE-REPETITIONS.tsv: 1097 bytes, SHA-256 0ad5eb53324573507e9f4db7f5afdabd29c053334595ae6d2e40a3b03397c359.
- DENSE-PERFORMANCE-SCHEDULES.tsv: 10917 bytes, SHA-256 06e0a002f799fa82dc4e77d9f46914fb1762ac9659ab6a18d0a145c24f3bbc94.
- DENSE-PERFORMANCE-STATISTICS.json: 172659 bytes, SHA-256 08987857f7e52ab258536b8d8763d689e3ad1feb243b369241b3d82e286ab4b0.
- DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv: 4770 bytes, SHA-256 6196995de9b7f1ad2518c38c730a4c081ddb465a2bb00cff52734e9b351c9f32.
- DENSE-PERFORMANCE-TARGETS.tsv: 1489 bytes, SHA-256 9dcc86dc85fdcf87c77f7c1e6c411c22761b44aac50a7973348df335997dcf94.
- DENSE-PERFORMANCE-WARMUPS.tsv: 643 bytes, SHA-256 0b048f6ab95c9c48d6bb963f0a4be6d648258b86c88dd6944ed0251ad908c41b.

The registry summary adds the report hash and source-authority hashes. Independent hostile review remains required before any authorization.
