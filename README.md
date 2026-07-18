# Whitefoot

A systems language for AI-written, human-approved code. The checker makes the
memory-corruption, data-race, silent-overflow, and uninitialized-read bug
classes unrepresentable — no unsafe escape exists — and its proofs feed the
optimizer, so checked code runs at C-class speed: safety checks are always on
unless a machine-verified proof discharges them.

Here is what that means in one program.

**This is base64. Every array read and write in it is bounds-checked at
runtime — and the language has no `unsafe`, no `get_unchecked`, no way for a
writer to turn a check off by hand.**

```
fn encode ['r] (out: &uniq 'r buffer<u8>, src: own buffer<u8>) -> own u64
    reads('r), writes('r), traps requires {
  ...
  check ile<u64>(required_src_len, required_covered_src) else trap "base64 output capacity";
} {
  ...                      // the obvious indexed loop: 3 reads, 4 writes per step
}
```

One line of contract — *the output buffer is big enough* — is checked once on
entry. A machine proof then carries that fact into the loop and discharges all
27 bounds checks inside it. Same source, checks retained versus proven away:
2.48 → 4.23 GB/s (**1.71x**), output byte-identical, the entry check still live
(an undersized buffer traps at the boundary — even for a C caller).

Now the same work in Rust, same machine, full RFC semantics, isolated
processes:

| implementation | throughput | vs Whitefoot |
|---|---:|---:|
| **Whitefoot** — obvious loop + one checked `requires` line | **4.285 GB/s** | — |
| Rust — obvious indexed loop | 2.673 GB/s | 1.60x slower |
| Rust — obvious loop + `assert!` up front | 2.677 GB/s | 1.60x slower |
| Rust — expert `chunks_exact/zip` restructure | 4.297 GB/s | tie |
| Rust — `unsafe` indexed | 4.111 GB/s | 1.04x slower |

Read the middle two rows. The `assert!` that *looks* like it should let the
optimizer drop the checks recovers **nothing** — LLVM cannot connect it to the
loop. Only an expert who knows the `chunks_exact` restructuring reaches the
fast class, and even `unsafe` indexing is slightly slower. In Whitefoot the
**obvious** shape plus one checked line gets there, with every bound still
enforced.

That is the whole idea in one program: **speed is earned by proof, never
bought by weakening a check.**

The full argument — effect rows the optimizer trusts across opaque boundaries,
ownership-driven guard-free vectorization, checked algebraic laws, and what it
honestly does *not* beat — is in **[docs/why-whitefoot.md](docs/why-whitefoot.md)**.

Highlights so far (each with a RESULTS.md under `experiments/`):
- default floor: first-green `gpt-5.6-terra` Whitefoot programs beat the ordinary
  public paths of two released Rust crates by 1.653x and 1.098x on locked
  workloads, with every reported Whitefoot bounds check retained.
- wc: byte-identical under LC_ALL=C, ~2x GNU coreutils on default invocation.
- The classifier-kernel study: i1-dataflow parity with C and safe Rust.
- Checked algebraic laws: 3.3x on reductions, with FALSE laws refuted at
  compile time — the transform Rust must take on faith.

## Where things are

| I want to... | Go to |
|---|---|
| Understand the project state | `THE-PLAN.md`, then the tail of `optimizer-language-research/implementation/decision-gates.md` |
| Work in this repo as an agent | `CLAUDE.md` |
| Read the law / the doctrine | `CONSTITUTION.md` / `PATTERNS.md` |
| Read the language spec | `spec/kernel-spec-v0.6.md` (+ `spec/derivation-ledger.md` for why each rule exists) |
| The production compiler (wfc, self-hosting) | `compiler/` |
| The stage-0 compiler + checker | `prototype/` |
| Run all verification | `make check` and `make -C compiler check` |
| Measured evidence | `experiments/` (index in its README) |
| Owner rulings | `optimizer-language-research/notes/user-directives.md` |
| Superseded history | `archive/` |

## Verification

Two gates, both required green: `make check` (spec CI, rule tests, soundness
probes, performance pins, codegen parity corpus, conformance suite) and
`make -C compiler check` (the wfc test stack, including the self-parse gate).
Every completed unit of work commits with a one-line entry in the decision
log — the repo is designed so that any fresh session can resume from
`git log` plus the log tail alone.
