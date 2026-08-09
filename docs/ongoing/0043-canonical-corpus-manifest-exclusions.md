# 0043 — derive canonical-corpus exclusions from the manifest

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** the active obligation-discharge plan's complete-gate
  validation; the 2026-08-08 canonical-renderer and pre-semantic-reject rulings
  in `governance/APPROVALS.md`; owner instruction of 2026-08-09 to implement
  the current ENT-5 goal through restored verification
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`, branch
  `codex/0043-canonical-corpus`
- **Base revision:** `50e2c35`
- **Dependency:** v0.23 activation `a01bc70`; independent of terminal task 0042

## Goal and direction

Restore the native canonical-corpus gate after v0.23 without a filename list or
fixed count. A source that cannot derive may be excluded only when its exact
conformance path has a manifest `reject` expectation. A derived non-canonical
source may be excluded only when that exact path has a manifest `FORM-*`
rejection. Every other source must derive and render canonically; derived
semantic rejects continue through the ordinary round-trip checks.

## Method and scope

- Reuse the existing typed Rust manifest reader and map exact conformance case
  paths to expectations.
- Replace `DELIBERATELY_NONCANONICAL` and `underived.is_empty()` with two
  manifest-derived classifications and two unexpected-failure sets.
- Add focused canaries for both allowed and forbidden classification edges.
- Expected touch set: `compiler/tests/canonical_corpus.rs` and, only if needed
  to reuse the existing test module without duplication, its local module
  declarations. Do not change the manifest, any `.wf` case, verdict, status,
  cited rule, parser, renderer, or compiler semantics.

## Progress

- Completed: reproduced the 423-file split on v0.23 — 402 round trips, one
  derived FORM-2 non-canonical case, and 20 manifest-declared pre-tree rejects.
- Current: implement the manifest-derived classifier and focused unit canaries.
- Next: run the focused integration test, compiler gate, and complete repository
  gate; then close this record and make the landed result a prerequisite of the
  stale 0040 closure.

## Validation and stop condition

- `cargo test --test canonical_corpus --locked --offline`
- `make -C compiler check`
- `make check`

Stop on any need to edit protected conformance material, change an expectation,
weaken rendering/idempotence checks, or classify by filename or a fixed count.

## Closure

Pending.
