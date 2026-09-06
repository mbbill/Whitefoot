# 0043 — derive canonical-corpus exclusions from the manifest

This is frozen coordination history, not execution authority.

- **Status:** `DONE` (2026-08-09)
- **Authority:** the active obligation-discharge plan's complete-gate
  validation and the protected canonical-renderer/pre-semantic-reject rulings
- **Outcome:** the native canonical-corpus gate is green on v0.23 without a
  filename list or fixed count. An exact conformance path may fail before a
  tree only when its typed manifest expectation is `Reject`; a derived source
  may be non-canonical only when that exact expectation cites `FORM-*`.

## Landed work

- `356943b` refreshed the claimed task onto Direction Outline revision 20 and
  its carried execution scope.
- `201c2d3` reused the existing Rust manifest reader, mapped complete case
  paths to typed expectations, separated manifest-authorized and unexpected
  failures, and added closed-edge canaries. Every derived file still checks
  canonical output and idempotent round-trip before classification, so a
  semantic reject receives no rendering exemption.
- No manifest row, `.wf` source, verdict, status, cited rule, parser, renderer,
  or compiler semantic changed.

## Validation and remaining boundary

The pre-fix focused test reproduced 423 files as 402 round trips, one derived
non-canonical source, and 20 underived sources. After the change,
`cargo test --test canonical_corpus --locked --offline` passed 3/3,
`cargo clippy --test canonical_corpus --locked --offline -- -D warnings`
passed, `make -C compiler check` passed, and `make check` exited 0. An
independent read-only review found no P1/P2 issue. The stale task 0040 closure
must refresh to `201c2d3`; the ACTIVE plan may now proceed to the bounded
ENT-5/stable-file candidate.
