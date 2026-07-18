# One-time Rust opportunity pilot

Status: **complete directional pilot; stop after this run**

This directory answers one practical question: do ordinary, popular Rust
projects contain enough optimization shapes relevant to current Whitefoot to
justify moving on? It is disposable research tooling, not a general Rust
analyzer, a preregistered population study, or a source of precise prevalence
estimates.

The result is deliberately asymmetric. A manually confirmed finding is useful
positive evidence. A clean scan is not proof that a project is optimal because
the source recognizers are narrow and approximate.

## Fixed samples

Both ranked inputs were crates.io API snapshots fetched on 2026-07-12. Input
order is the API's download order. `pilot.py` deduplicates repository URLs,
examines the ordered prefix, and stops when the requested eligible count is
reached. It records ineligible and unknown entries rather than silently
discarding them.

- **Source panel (`n=30`):** first 30 repository representatives with at least
  1,000 nonblank production Rust lines under `src/`.
- **Application panel (`n=12`):** first 12 crates in the crates.io
  `command-line-utilities` category satisfying the same line threshold and
  exposing a Cargo binary target.

This popular-crate frame is a useful, arguably conservative sample of reviewed
Rust, but it does not represent every Rust project. Package release archives,
not whole repository checkouts, are the measured objects.

## Instruments

- `pilot.py` safely materializes the ordered archives, inventories production
  source, invokes scanners, and writes one compact JSON ledger.
- `pilot_signals.py` finds approximate indexed-loop, multi-slice alias, and
  serial saturating-recurrence shapes. It also counts `chunks_exact`, `zip`,
  and `get_unchecked` controls. Every record is only a manual-audit candidate.
- `reassociation/` is the stricter syntax miner. Human inspection replaces the
  abandoned semantic-query plan for this one-time run.
- `bounds-ir/` finds first-party optimized functions that still directly call
  Rust's bounds-panic routine.
- `alias-versioning/` finds LLVM runtime alias checks guarding vector loops.
- `effect-attrs/` compares cross-module call-visible effect facts when a build
  captures all relevant LLVM modules. The pilot's single-module captures were
  insufficient for that comparison.

## Exact corpus commands

Run from the repository root. Raw responses, archives, extracted projects, IR,
and output JSON live under `/tmp` or the ignored `work/` directory.

```sh
curl -fsSL \
  -A 'whitefoot-frequency-pilot/1.0 (+https://github.com/mbbill/whitefoot)' \
  -o /tmp/whitefoot-source-ranking.json \
  'https://crates.io/api/v1/crates?page=1&per_page=100&sort=downloads'

curl -fsSL \
  -A 'whitefoot-frequency-pilot/1.0 (+https://github.com/mbbill/whitefoot)' \
  -o /tmp/whitefoot-app-ranking.json \
  'https://crates.io/api/v1/crates?page=1&per_page=100&sort=downloads&category=command-line-utilities'

python3 -B experiments/frequency-study/pilot.py \
  /tmp/whitefoot-source-ranking.json \
  --limit 30 --min-loc 1000 --fetch --trust-crates-io \
  --reassociation-command \
    'cargo run --quiet --offline --locked --manifest-path experiments/frequency-study/reassociation/Cargo.toml --' \
  --work-dir experiments/frequency-study/work/source30 \
  --output experiments/frequency-study/work/source30.json

python3 -B experiments/frequency-study/pilot.py \
  /tmp/whitefoot-app-ranking.json \
  --limit 12 --min-loc 1000 --require-bin --fetch --trust-crates-io \
  --reassociation-command \
    'cargo run --quiet --offline --locked --manifest-path experiments/frequency-study/reassociation/Cargo.toml --' \
  --work-dir experiments/frequency-study/work/app12 \
  --output experiments/frequency-study/work/app12.json
```

`--trust-crates-io` is intentionally narrow: it accepts checksum-less downloads
only from `static.crates.io` and records the bytes' SHA-256. It is convenience
for this throwaway run, not a publication-grade provenance claim.

For an optimized-IR capture, use the project's normal release build with line
tables and textual LLVM IR enabled. Then run, for example:

```sh
python3 -B experiments/frequency-study/bounds-ir/collect.py \
  --source-root /path/to/crate /path/to/ir --pretty

python3 -B experiments/frequency-study/alias-versioning/collect.py \
  --ir /path/to/module.ll --source-root /path/to/crate --pretty

python3 -B experiments/frequency-study/effect-attrs/classify_ir.py \
  --root /path/to/ir /path/to/ir/*.ll --pretty
```

## Manual audit and interpretation

The source audit inspected all high-signal index and alias records, rejecting
test/generated code, already expert-safe shapes, unrelated alias relationships,
and cases current Whitefoot cannot express or annotate. Application candidates were
audited again rather than promoted automatically. Three buildable libraries
(`comrak`, `inferno`, and `crc`) received a small optimized-IR follow-up.

The pilot calls for moving on only directionally: repeated raw source shapes
plus several bounds-check sites worth IR inspection justify further work on
precondition/bounds proofs and real workload ports. It does **not** establish a
whole-program speedup, and it does not support treating checked-law or current
scoped-alias wins as common. Full counts and caveats are in `RESULTS.md`.

## Checks

```sh
make -C experiments/frequency-study check
```

That command runs the pilot, bounds, alias, effect, and reassociation tests,
plus Rust formatting and clippy. There is intentionally no corpus-lock or
results-validation framework.
