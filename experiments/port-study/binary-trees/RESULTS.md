# Port Study Pilot: binary-trees (leg B)

Status: MEASURED + ADVERSARIALLY VERIFIED 2026-07-10 (3-lens panel:
equivalence / fairness / claim-honesty; all equivalence and fairness claims
HOLD; the original headline was corrected per the claim-honesty lens).

## Setup

Checksum-equivalent binary-trees workload (depth 21, min 4, benchmark-game
constants, sequential, single-author references — NOT tuned Benchmarks Game
entries; the game's competitive entries are parallel + bumpalo and would beat
everything here). Note: game rules require per-node-allocation semantics, so
this is NOT a claim about the public benchmark — the arena variants (in both
languages) design the allocation work away; that redesign is the thing under
study. Whitefoot: democ -> clang -O2. Rust: rustc -C opt-level=3. Apple M4,
medians of 3.

| variant | time | note |
|---|---:|---|
| Whitefoot, facts | 0.71s | the shape v0's borrow rules steer to (bottom-up SoA pool) |
| Whitefoot, no-facts control | 1.03s | same IR minus alias/effect metadata (verified metadata-only diff) |
| Rust obvious (Box per node) | 8.64s | default allocator; first-draft shape |
| Rust expert (index arena, recursive) | 1.54s | &mut reborrow build — inexpressible in Whitefoot v0 |
| Rust, identical bottom-up shape | 0.64s | layout-exact control for the Whitefoot port |

## Verified findings (what may honestly be claimed)

1. **The 12x is a SHAPE effect, not a language effect.** Rust-vs-Rust shows
   the same gap (0.64 vs 8.64 = 13.5x). The honest Whitefoot claim is the
   inverse: **the borrow rules exclude the slow shape, so Whitefoot's floor is
   high** — the naive-Box design is unrepresentable, the port lands on the
   fast design by construction. "Whitefoot is 12x faster than Rust" is the wrong
   reading; identical-shape Rust is FASTER tha Whitefoot.
2. **Facts are worth 1.45x on a real program** (0.71 vs 1.03), and the
   facts/nofacts diff is aliasing/effect metadata ONLY (verified in IR): the
   gap is optimization freedom, not semantics.
3. **The checked-semantics tax vs identical-shape Rust is ~11%** (0.71 vs
   0.64, facts enabled) — and the panel notes part of that is not codegen
   immaturity but Whitefoot doing MORE: overflow-trapping arithmetic on every op
   where release Rust wraps silently, plus bounds checks both sides pay.
   Without facts the tax is 61% — the channels are what make the checked
   semantics affordable.
4. **"Only expressible shape" was an overclaim** (panel): slower shapes are
   expressible; linear threading could plausibly express the recursive build
   verbosely; and the greenlit batch-1 deltas (OWN-6 reborrow-through-holder,
   OWN-14 result reborrows) would legalize the recursive arena shape once
   implemented. Correct statement: v0's no-reborrow rule STEERS the port to
   the bottom-up shape, which happens to be the fast one.
5. Fairness flags all point the conservative direction: clang -O2 (Whitefoot) vs
   opt-level=3 (Rust); zero-fill folded to calloc on both sides (verified);
   Whitefoot carries strictly more runtime checks; equivalence of trees,
   checksums, and iteration counts traced and confirmed; all assertions are
   hard failures and all binaries exit 0.

## Leg-B pilot verdict

The port answered both feasibility questions: a canonical program fits the
subset TODAY (three walls hit, all instructive: TYPE-6 sibling-loop binders,
OWN-11 per-iteration regions, no-reborrow forcing bottom-up build — none cost
performance), and the study design works (shape-isolating controls + facts
on/off + adversarial verification). Next targets: fannkuch-redux (pure index
compute); VM/codec class blocked on const-array codegen (and interpreters
re-scoped per the dispatch analysis — see gates 2026-07-10).
