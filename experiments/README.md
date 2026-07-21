# Experiments Index

Every directory is self-contained (sources + Makefile/run script + RESULTS.md
with measured numbers and honest caveats). Binaries/corpora are regenerable
and gitignored. Chronology and decisions live in
`../optimizer-language-research/implementation/decision-gates.md`.

## Historical fact-channel benchmarks

These results used the now-archived democ implementation. They remain measured
evidence and regression requirements for the Rust compiler, but their harnesses
are not active compiler gates in the current foundation. They return only
through the later roadmap tranche that owns the relevant backend or
real-project evidence.

- `effect-attrs-channel/` — channel 2: effect rows -> LLVM fn attributes.
  O(n)->O(1) at opaque boundaries; ties fat LTO at per-file build cost.
- `scoped-alias-channel/` — channel 1: ownership provenance -> alias.scope.
  Short-trip wins, 17x code size vs Rust's guard-versioned loops; parity at
  long trips (Rust recovers via runtime checks).
- `checked-law-channel/` — channel 3: FN-4 discharged laws license
  reassociation. 3.3x over the obvious fold; refutes false laws compile-time.
- `frequency-study/` — completed one-time directional scan of popular Rust
  sources/applications; points the next real port at relational bounds proofs.

## Paused expressiveness evidence

- `data-layout-owning-sequence/` — historical E0.1 research and rejected
  isolated-prototype evidence. D11 leaves all work paused before bounded G0-Core
  and a later exact dense-family Lock A; neither is yet authorized, and the old
  protocol does not restart automatically. The evidence separates fixed SoA/AoS
  layout from initialized-prefix ownership and growth, protects the compiler's
  current SoA as the zero-tax baseline, and forbids feature-flagged dual
  semantics.

## Port studies (real programs; D9 confidence-gate evidence)
- `zlib-core-kernels/` — deferred RFC 1951 kernel handoff. Ordinary scalar
  lowering is not competitive for short-period match overlap and trails the
  pinned all-literal Huffman projection; bounds-check elision alone does not
  close either gap. Two unchanged-source stage-0 prototypes recover isolated
  performance through periodic expansion and a guarded six-symbol bit window.
  The directory preserves corrected raw results, compiler patches, LLVM and
  ARM64 snapshots, candidate writer patterns, proof obligations, and production
  pickup gates. These are feasibility results, not complete proofs or a
  whole-inflate claim.
- `default-floor/` — D9a protocol: a fixed low-tier model's first
  correctness-green Whitefoot artifact versus an exact unmodified shipped Rust
  library. Two separately preregistered results are complete: Terra Whitefoot beats
  `percent-encoding` 2.3.2 `percent_decode` by 1.653x [1.631, 1.667] and
  one-shot `utf8parse` 0.2.2 by 1.098x [1.085, 1.145]. Neither result is a
  proof-elision win; see the aggregate claim boundary in
  `default-floor/RESULTS.md` and the two target-specific reports beneath it.
- `port-study/binary-trees/` — floor-raising result: the slow shape is
  unrepresentable; ~11% checked-semantics tax vs identical-shape Rust.
- `port-study/wc/` — full-counts 0.27s vs GNU 0.48 / uutils-Rust 0.56 on a
  426MB corpus (regenerate: see RESULTS); -l honest gap vs memchr/bytecount.
- `port-study/wc-chunk-summary/` — ordered-monoid parallel wc. NEGATIVE
  result for channel attribution (Rust expresses the same algebra); reached
  C/Rust parity after the OWN-1 Bool-copy amendment (220->134ms).
- `port-study/base64/` — first const-array consumer; 1.6x GNU/uutils,
  ~parity BSD (table-width algorithm gap); PROOF-1 discharges 15/27 bounds
  sites and improves the kernel 2.50 -> 2.93 GB/s, with PROOF-2 debt isolated.

## Preserved code-generation fixtures

- `codegen-vs-rust-c/` — the splitmix scalar-backend-parity evidence. Its old
  democ runner is dormant. Phase 2 does not execute it; a later,
  entrance-gated backend tranche must bind the fixture to a Rust harness
  adapter before it can regain gate authority.

## Earlier corpus-era studies
Moved to `../archive/experiments/` (scatter residual, guarded-plan
measurements); conclusions absorbed into the corpus notes and THE-PLAN's
evidence ledger.
