---
name: whitefoot-test-profile
description: Never run plain `cargo test` in Whitefoot — the gate uses `--profile gate` (release + debug-assertions); dev-profile front-end checks of wfgrep.wf take >6 min and look like a hang
metadata:
  type: project
  originSessionId: ab659700-c818-4271-84f2-e403a962b810
  modified: 2026-08-27T03:00:00.000Z
---

2026-08-27: I ran `cargo test` (dev profile) on the I/O branch to verify
codex's numbers and it "hung" for 10+ minutes; the owner asked what broke.
Nothing did. `compiler/Cargo.toml` has `[profile.gate]` (inherits release,
debug-assertions + overflow-checks on) and `compiler/Makefile` `test:` runs
`cargo test --profile gate --all-targets --locked --offline` (since commit
09e4cff2, 2026-08-16). The cost_shape tests type-check the real
`tests/programs/wfgrep.wf` (~1400 lines); the entailment closure
(`entailment/state.rs` `close_with_excluded_term`, `DenseClosureBounds`) is
the hot spot: 26 s optimized, >6 min unoptimized. Same on main (fee33565)
and on the 2026-08-22 compiler, so it is the checker's baseline cost, not a
regression.

**How to apply:** verify with `make -C compiler test` (or
`cargo test --profile gate ...`), or `make check` at the root. Reuse the
gate binaries under `compiler/target/gate/` for single-test timing. When a
test run looks hung, check the profile before bisecting. The 26 s optimized
check of wfgrep is a real compiler-performance item (ENT closure per kill
event), worth a batch when it starts hurting.
