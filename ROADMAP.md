# Roadmap (updated 2026-07-07: build plan ratified)

Read first: CONSTITUTION.md -> spec/kernel-spec-v0.3.md (v0.3.1) -> spec/derivation-ledger.md -> optimizer-language-research/implementation/decision-gates.md.

## Build track (ordered; each milestone gates the next)
- **M0 — FR reconciliation of spec §5** (BLOCKS M1: the compiler core implements §5): map OWN-1..13 onto Featherweight Rust (K003), fix divergences, adopt FR model-check method. Output: §5 ratifiable + reconciliation memo.
- **M1 — reference compiler** (grow prototype/democ, stay Python — disposable per owner ruling): byte-exact canonical parser (FORM enforcement), full checker (add OWN-13, slices, copy/affine), structs/enums/match/loop, monomorphization, full op table, effects validation (EFF-2 syntactic exhibits), elaborated artifact (DIAG-2: drops+checks surfaced), LLVM backend. ACCEPTANCE: compiles spec EX-1 end-to-end; dangle-class programs rejected with rule IDs.
- **M2 — verification harness**: spec-CI rule<->test-vector binding; FR-style model checking (random program gen + interpreter as differential oracle: accepted programs never exhibit UAF/double-uniq at runtime).
- **M3 — AI-codegen harness** (= M1+M2 in a generate->check->score loop, W1 model tiers): unblocks the R3 experiment battery — loop-form first, then conditional/no-if, statement-match, TYPE-5 interior, no-comments, byte-format reject-vs-canonicalize, FN-3 interfaces back-fill.
- **M4 — self-hosting** (endgame, owner ruling): compiler rewritten in xlang; the ultimate R0 dogfood.

## Parallel research track (fills gaps, never blocks build)
Effect exemplars for §9 (Koka/Cyclone/MLKit/DPJ); D3 lexicon audit (Ok/Err/Some/None, box, fn/let, 'r sigil); FFI dossier (D4-narrowed: C-ABI out, pinning, foreign_shared); trap batch-replay spec validated vs LLVM LoopVectorize; remaining card debt (J001/J002, A003, N002 verbatim, OWN-9 noalias benchmark — partially discharged by democ 1-vs-2-loads measurement); DIAG-3 field schemas.

## Standing disciplines
META-5 (delta + selection ground), META-6 (derivation ledger; tools/spec_ci.py green before every spec change), R0 Rust test, W1 tiering, weakest-chain queue in derivation-ledger.md.
