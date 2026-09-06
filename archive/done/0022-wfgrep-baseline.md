# 0022 — Zero-change wfgrep baseline

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 1
  (PERF-1)

## Outcome

The first preregistered performance measurement of a Whitefoot program
(protocol frozen before any number; null-comparison precision gates 0.21 to
0.98% half-width against a 2% gate; the sub-percent RG-BASE position bias
measured and disclosed). Headline grep/wfgrep ratios (median, 95% CI):
large-file 0.647 [0.643, 0.653] and no-match 0.656 — material losses,
user-compute bound; many-small-files 0.605 — dominated by the host's
per-open cost for unsigned local binaries (same-provenance C control pays
the identical cost; an environment layer, not a Whitefoot layer);
match-dense 1.105 — a material WIN (the comparator's regex engine churns
per byte while wfgrep is density-flat with batched output); process floor
1.43 ms vs grep's 1.68 ms. Attribution: the preregistered scan-trap suspect
is REFUTED as primary and confirmed secondary — the retained per-byte trap
ceiling is ~18% of compute; free elision of both traps would still leave
~0.77 — the primary term is the scalar double-walk shape (separate newline
scan and literal-match loops, neither vectorized by LLVM), an
algorithm/source-shape/lowering question. PROOF-1's feed is real but
bounded. Scope honestly limited to this host's BSD grep; no
language-vs-C, ripgrep, traversal, or cross-host claim. The measured bytes
are the frozen pre-refactor wfgrep (SHA in the record header); task 0021's
refactor is behavior-identical with §9.1 gates held.

## Evidence and validation

- Landed commits: `9307157` (preregistration), `10a3b5d` (results).
  Canonical evidence: `research/experiments/wfgrep-baseline/` (PROTOCOL,
  MANIFEST, runner, open-control, RESULTS, SHA-pinned raw record). Both
  gates green by unpiped exit codes.
