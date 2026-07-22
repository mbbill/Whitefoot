# Code generation and performance tests

Performance is a project pillar, but this directory's current runner targets a
retired compiler and is not an active gate. Its sources and manifests preserve
useful hypotheses and negative controls until the LLVM backend exists. See
`DORMANT.md` for the historical runner contract.

The replacement belongs beside the real backend and must exercise unchanged
Whitefoot source through the normal compiler path. It has four layers:

1. executable correctness for lowered programs and runtime checks;
2. facts-off tests proving that every required check remains;
3. facts-on/facts-off comparisons proving that only justified checks disappear,
   with near-miss programs that must retain them; and
4. runtime and code-shape measurements kept under `research/`, because noisy
   timing is experimental evidence rather than an every-commit invariant.

Before the old runner moves to `archive/`, the new Rust harness must map each
retained high-value case to one of those layers, run through the current
compiler, and preserve the negative controls. Cases that encode retired
compiler APIs or product-policy machinery may be archived with the runner;
the underlying semantic and performance hypothesis must either be mapped or
explicitly rejected with a recorded reason.

Do not make this corpus an optimizer dispatch table. Production compilation
must implement grammar and semantic rules, never case names, source hashes, or
manifest identities.
