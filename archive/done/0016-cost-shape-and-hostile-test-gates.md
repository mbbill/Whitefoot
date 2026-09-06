# 0016 — Cost-shape and hostile test gates

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main, 2026-08-06; closes the first-slice
  implementation waves
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work items 2
  and 4, wave 8

## Outcome

§9.1 is standing machine evidence anchored on the real wfgrep module
(`compiler/src/backend/tests/cost_shape.rs`): structural gates over every
approved implementation body and the optimized `main` (no dispatch/indirect
call, zero-allocation leases, the RelativePath retype returning the same
value, one openat/read site and five write sites each alone in their block,
close-diagnostic never read, exactly four calloc matching four buffer_new
with no malloc/realloc/memset anywhere, all allocation before the first
transfer); behavioural gates on the deterministic host (3,000 matches = 2
host writes summing exactly to output; 6,000 bytes through 9 host calls;
release rows observed — two closes, never fd 1/2); the four 0013 injection
gates plus the lead-authorized Accept(0) WriteZero behavioural case; §12.2
compile-time items verified as existing corpus coverage. The SYS-7 mapper is
fully outlined into main.cold.* — zero wrapper symbols survive in main. The
§11 buffer-initialization stop condition was measured under preregistration
(research/experiments/buffer-initialization-cost/): parity 1.0014
[0.9982, 1.0083] against the uninitialized native control, one-page
initialization 28.76 ns — no input size makes it material; the condition
did not fire. Honest correction recorded in the gate file: 0015's "memchr
recognized" note was wrong — the newline scan is a scalar byte loop
retaining its bounds trap (the one @memchr is relative_path's NUL check);
no §9.1 row requires memchr and the per-byte-call rejection holds. That
scalar scan with its retained trap is the first named PROOF-1 pressure
candidate for the performance phase.

## Evidence and validation

- Landed commits: `ee2e424` (claim), `af22463` (gates), `c85a165`
  (preregistered research), `23afaa5` (record). Both gates green by unpiped
  exit codes; lib tests 427 → 438.
- Exact-count caveat recorded in the gates: transfer-site counts equal
  source-site counts; a future optimizer duplicating a site requires
  re-deriving the gate from source, never relaxing it.
