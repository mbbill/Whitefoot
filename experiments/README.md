# Experiments Index

Every directory is self-contained (sources + Makefile/run script + RESULTS.md
with measured numbers and honest caveats). Binaries/corpora are regenerable
and gitignored. Chronology and decisions live in
`../optimizer-language-research/implementation/decision-gates.md`.

## Fact-channel benchmarks (democ facts vs --no-facts vs Rust/C)
- `effect-attrs-channel/` — channel 2: effect rows -> LLVM fn attributes.
  O(n)->O(1) at opaque boundaries; ties fat LTO at per-file build cost.
- `scoped-alias-channel/` — channel 1: ownership provenance -> alias.scope.
  Short-trip wins, 17x code size vs Rust's guard-versioned loops; parity at
  long trips (Rust recovers via runtime checks).
- `checked-law-channel/` — channel 3: FN-4 discharged laws license
  reassociation. 3.3x over the obvious fold; refutes false laws compile-time.
- `frequency-study/` — completed one-time directional scan of popular Rust
  sources/applications; points the next real port at relational bounds proofs.

## Port studies (real programs; D9 confidence-gate evidence)
- `default-floor/` — D9a protocol: a fixed low-tier model's first
  correctness-green xlang artifact versus an exact unmodified shipped Rust
  library. First result: Terra xlang beats `percent-encoding` 2.3.2
  `percent_decode` by 1.653x [1.631, 1.667], with facts-on/off practical parity;
  see `default-floor/percent-decode/RESULTS.md`. Independent replication target
  `default-floor/utf8parse/` is preregistered but not yet launched.
- `port-study/binary-trees/` — floor-raising result: the slow shape is
  unrepresentable; ~11% checked-semantics tax vs identical-shape Rust.
- `port-study/wc/` — full-counts 0.23s vs GNU 0.48 / uutils-Rust 0.56 on a
  426MB corpus (regenerate: see RESULTS); -l honest gap vs memchr/bytecount.
- `port-study/wc-chunk-summary/` — ordered-monoid parallel wc. NEGATIVE
  result for channel attribution (Rust expresses the same algebra); reached
  C/Rust parity after the OWN-1 Bool-copy amendment (220->134ms).
- `port-study/base64/` — first const-array consumer; 1.6x GNU/uutils,
  ~parity BSD (table-width algorithm gap); PROOF-1 discharges 15/27 bounds
  sites and improves the kernel 2.50 -> 2.93 GB/s, with PROOF-2 debt isolated.

## Gate fixtures
- `codegen-vs-rust-c/` — the splitmix scalar-backend-parity fixture (LIVE:
  referenced by the parity gate; do not archive).

## Earlier corpus-era studies
Moved to `../archive/experiments/` (scatter residual, guarded-plan
measurements); conclusions absorbed into the corpus notes and THE-PLAN's
evidence ledger.
