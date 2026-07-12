# Roadmap (updated 2026-07-08: decision sprint added)

Read first: CONSTITUTION.md -> spec/kernel-spec-v0.6.md -> DECISION_SPRINT.md -> spec/derivation-ledger.md -> optimizer-language-research/implementation/decision-gates.md.

## Immediate decision gate

- **M3 decision sprint**: run the bounded AI-codegen validation in `DECISION_SPRINT.md` before committing to the self-hosting compiler track. The compiler plan remains the forward implementation path if the sprint passes.
- **AI-native parallelism investigation**: run the first-principles branch in `experiments/ai-native-parallelism/` before treating parallelism as a project pillar. This branch tests source bundles with AI-authored plans/proof obligations/guards, not DPJ-style human-maintained annotations.

## Build track (ordered; each milestone gates the next)
- **M0 — FR reconciliation of spec §5** (BLOCKS M1: the compiler core implements §5): map OWN-1..13 onto Featherweight Rust (K003), fix divergences, adopt FR model-check method. Output: §5 ratifiable + reconciliation memo; v0.6's additive `give`/OWN-13 clause remains a tracked ratification item until its proof note lands.
- **M1 — reference compiler** (grow prototype/democ, stay Python — disposable per owner ruling): byte-exact canonical parser (FORM enforcement), full checker (add OWN-13, slices, copy/affine), structs/enums/match/loop, monomorphization, full op table, effects validation (EFF-2 syntactic exhibits), elaborated artifact (DIAG-2: drops+checks surfaced), LLVM backend. ACCEPTANCE: compiles spec EX-1 end-to-end; dangle-class programs rejected with rule IDs.
- **M2 — verification harness**: spec-CI rule<->test-vector binding; FR-style model checking (random program gen + interpreter as differential oracle: accepted programs never exhibit UAF/double-uniq at runtime).
- **M3 — AI-codegen harness** (= M1+M2 in a generate->check->score loop, W1 model tiers): unblocks the R3 experiment battery — loop-form first, then conditional/no-if, statement-match, TYPE-5 interior, no-comments, byte-format reject-vs-canonicalize, FN-3 interfaces back-fill.
- **M4 — self-hosting** (endgame, owner ruling): compiler rewritten in xlang; the ultimate R0 dogfood.

## Parallel research track (fills gaps, never blocks build)
Effect exemplars for §9 (Koka/Cyclone/MLKit/DPJ); D3 lexicon audit (Ok/Err/Some/None, box, fn/let, 'r sigil); FFI dossier (D4-narrowed: C-ABI out, pinning, foreign_shared); trap batch-replay spec validated vs LLVM LoopVectorize; remaining card debt (J001/J002, A003, N002 verbatim, OWN-9 noalias benchmark — partially discharged by democ 1-vs-2-loads measurement); DIAG-3 field schemas.

## Standing disciplines
META-5 (delta + selection ground), META-6 (derivation ledger; tools/spec_ci.py green before every spec change), R0 Rust test, W1 tiering, weakest-chain queue in derivation-ledger.md.

## Post-audit plan of record (2026-07-09)
Phase 0 hygiene -> A M3-unblock (buffer/index/len, try, bytes, pool ruling) -> B channels: effect-attrs BUILT+MEASURED 2026-07-09 (O(n)->O(1) at opaque boundaries; ties fat-LTO Rust; experiments/effect-attrs-channel); REMAINING: scoped alias metadata; FN-4 law consumer incl. injectivity for scatter with discriminating benchmarks vs rustc -> C M3 W1 sprint -> R0 decision gate. Diagnosis: the negative audit tested only channels where rustc emits identical facts; 3 of 4 constitutional P0 channels remain unbuilt/untested.
