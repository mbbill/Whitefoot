# Ordinary host-values migration

This is a local smoke measurement during the v0.58 amendment, based on
`d695f385` plus the current ordinary-library and generator changes. It does not
claim the amendment or native host CI is complete. Maintain this record with
the campaign described in [README.md](README.md); replace its current results
when the generator or measured compiler changes.

The command was `make smoke WORK=/private/tmp/c2-fuzz-run
CARGO_TARGET_DIR=/private/tmp/c2-fuzz-target`, with the ordinary gate-profile
compiler, first seed 1, four jobs and one repetition. The target is 20 accepted
programs; concurrent workers completed 23 before stopping.

| Observation | Result |
| --- | ---: |
| Attempts / accepted / rejected | 25 / 23 / 2 |
| Programs agreeing with their sequential reference | 23 |
| Captured executions / delayed FIFO executions | 483 / 3 |
| Divergences / unstable references / crashes / timeouts | 0 / 0 / 0 / 0 |
| Lowering refusals | 0 |
| PAR-1 pairs permitted / denied | 7 / 22 |
| PAR-2 loops permitted / denied | 6 / 24 |
| Duration reported by the campaign | 0.9 minutes |

Both rejected programs cite INV-1 at a buffer-length invariant backedge. Seed 9
retains scratch outside its file loop: the successful open arm calls a helper
whose unconditional two-state contract preserves the scratch length, while the
failure arm leaves it unchanged. Seed 18 calls the analogous directory helper
before matching its status. Their generated sources remain reproducible with
`generate --seed 9` and `generate --seed 18`; the hoisted storage and status arms
have not been removed. Independent review separated a repaired branch-image
leak from ENT-6's conservative join: the repair preserves the untouched edge,
but does not derive a shared affine length theorem after differing images join.
An element-only helper can instead take an ordinary `MutSlice`, keeping the
owner descriptor outside its write row. This cohort retains its original
helper shapes and is not relabelled as a post-repair measurement.
The campaign exposes these rejections instead of counting them as executions.

All original algorithm families remain in the generator. Acquired owners close
on each path, view construction is ordinary WF source, and mutation helpers
publish length preservation through existing two-state contracts. Legacy claim
syntax is replaced by checked local invariants over the same arithmetic and
indexed mutation. PAR-3 stage counts are explicitly retired by v0.58; PAR-1 and
PAR-2 counts now come from the parallel invocation. The default invocation is
still compiled and executed separately. Worker/helper matrices, repetitions,
reference stability checks, delayed FIFO reads and byte/status comparisons are
unchanged.

`make lint` passed formatting and clippy with warnings denied. No recorded
probe directory exists; it is created only when the oracle retains a finding.
