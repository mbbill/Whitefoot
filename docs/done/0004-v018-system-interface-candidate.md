# v0.18 system-interface candidate

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — v0.18 activated at `9768bae`, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 1,
  derived from Outline revision 7

## Outcome

Produced, verified, and activated `spec/kernel-spec-v0.18.md` (SHA-256
`307a758e41366531c71dc8736bddc466054dbeba37f6e6db13f0859787711a28`), the
BOUND-1 first-command-slice batch: 25 new rules, 13 modified, the Route C
system-declaration domain with syntactic conditional visibility, and the
system inventory (7 opaque types, 7 outcome enums / 39 constructors, 11
operations). Five delta packages were drafted in parallel and integrated
serially; a hostile integration review raised 17 findings, all applied; the
native verifier (task 0005) verified the candidate grammar; the owner gave
exact-byte approval on 2026-08-05 with all eight judgment items accepted;
activation landed the spec, the promoted native grammar, the derivation
ledger rows (21 derived / 4 existence-only), and explicit
unsupported-capability gates for the whole surface.

## Evidence and validation

- `governance/APPROVALS.md` 2026-08-05 entry; `spec/kernel-spec-v0.18.md`;
  `spec/derivation/derivation-ledger.md`; the architecture dossier and
  31-issue review record under
  `research/investigations/system-capability-architecture/`.
- Landed commits: `7cc6302` (integration), `85c0f5c` (review fixes),
  `4ea068a`/`a9c6e1a` (verifier, task 0005), `776dc97` (approval),
  `110d8c4` (ledger), `9768bae` (activation).
- Validation: native verifier green on the active spec (64/74/75);
  `make -C compiler check` and `make check` green (119 rules, full suite);
  v0.17-accepted behavior byte-identical; no protected conformance verdict
  changed.

## Follow-ups

- Implementation proceeds through planned tasks 0006-0016.
- `tests/conformance/runner.py` still pins v0.17 (correct until task 0014
  lands v0.18 cases); the conservative unsupported gates are converted to
  real FN-7 judgments by tasks 0006/0008.
- The offline grammar-table generator note lives in `docs/done/0005`.
