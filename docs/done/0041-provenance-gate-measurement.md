# 0041 — measure the provenance gate on the boundary-fed deflate sites

This is frozen coordination history, not execution authority.

- **Status:** `DONE` — the measurement completed with a negative prerequisite
  result; a negative result was an explicitly valid task outcome.
- **Authority:** the ACTIVE stage-5a plan derived from Direction Outline
  revision 22; owner approval 2026-08-07 advancing the provenance measurement;
  and the held `governance/spec-evolution/provenance-gate-candidate.md`.
- **Owner / workspace:** Codex lead /
  `/Users/bytedance/do_not_scan/wf-0045-final-activation`, branch
  `codex/0045-ent5-activation`.
- **Base revision:** `e5db43d`; claimed by `41aa352` after task 0045's terminal
  v0.24 acceptance baseline.

## Outcome

The frozen four-file boundary compilation unit contains 23 named claims and
33 subscript obligations. Applying the held PRV-1/PRV-2/PRV-3 rules literally
classifies 18 obligation subjects as external. Six of those prove in the
unasserted state; the other 12 obligations belong to ten claims that the gate
would reject. Fifteen subjects are internal.

The candidate's required canonical-Huffman result is only 2/3:

- `order_slot_in_offsets` is gated;
- `ordered_in_symbols` is gated;
- `destination_in_symbols` is missed because an external index affects only
  write addresses and control, while the internal right-hand-side count values
  keep `counts`, `offsets`, and `destination` internal under the drafted rules.

The measurement also found a PRV-2 diagnostic gap: a derived column retains a
set of required parameter positions but no mapping from one position to the
several obligations its diagnostic is required to carry. The material O3
`requires` bypass remains unchanged. The candidate therefore returns to rule
review under its own §8 condition; this task did not edit or activate it.

## Landed work

- `41aa352` — lifecycle claim refreshed onto the terminal ENT-5 baseline.
- This closure change — the complete result in `PROBE-TAINT.md`, Direction
  Outline revision 23, and this move from `docs/ongoing/` to `docs/done/`.

## Canonical evidence

- `research/investigations/obligation-discharge/PROBE-TAINT.md` owns the frozen
  inputs, full 23-claim table, 33-obligation cross-check, propagation lineage,
  canonical hit/miss result, precision lenses, PRV-2 call projection, and
  honest repair costs.
- `docs/roadmap.md` revision 23 owns the current landscape and records that
  stage 5a is negative rather than silently rolling into stage 6.

## Validation

- Activation `f4c7e60c47bdea620eea5a00be89ff54d7678cc9`; active-spec SHA-256
  `53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.
- Held-candidate SHA-256
  `62f9fbb98d69777f5cacacb8f63fd4a922eed4bef6e49d7a66f71df7827fb47b`.
- Two independent source walks agree on 33 obligations, 18 external subjects,
  12 rejected obligation nodes, ten affected claims, and the 2/3 canonical
  result.
- The existing checker independently reproduces the same five v0.24 redundant
  claim advisories. The measurement changed no specification, compiler,
  protected corpus, active plan, or held-candidate byte.
- `make check`, `make repository-invariants`, and `git diff --check` complete
  with exit status 0 on the closure tree.

## Remaining dependency

The terminal current plan requires owner disposition after a negative result.
No later stage is authorized by this record, and this record does not choose
the disposition or sequence a repair. Wfgrep remains parked.
