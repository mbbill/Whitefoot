# Whitefoot

Whitefoot is a systems language that an AI writes and a human approves. Four bug classes simply don't compile: memory corruption, data races, silent overflow, and uninitialized reads. There is no `unsafe` escape to reach for. The checks that rule those bugs out also give the optimizer facts it can trust, so checked code runs at C speed. Every check stays in unless a machine proof takes it out.

Base64 shows the bargain in one function. Every array read and write below is bounds-checked at runtime. No `unsafe`, no `get_unchecked`, no way to switch a check off by hand.

```
fn encode ['r] (out: &uniq 'r buffer<u8>, src: own buffer<u8>) -> own u64
    reads('r), writes('r), traps requires {
  ...
  check ile<u64>(required_src_len, required_covered_src) else trap "base64 output capacity";
} {
  ...                      // the obvious indexed loop: 3 reads, 4 writes per step
}
```

One `requires` line states that the output buffer is big enough. It runs once, at entry. A machine proof carries that fact into the loop and clears all 27 bounds checks inside it. The same source runs at 2.48 GB/s with the checks in place and 4.23 GB/s after the proof removes them, a 1.71x gain. Output stays byte-identical. The entry check still traps an undersized buffer, even when C code calls in.

Rust runs the same task on the same machine, full RFC semantics, isolated processes:

| implementation | throughput | vs Whitefoot |
|---|---:|---:|
| **Whitefoot**, obvious loop plus one checked `requires` line | **4.285 GB/s** | baseline |
| Rust, obvious indexed loop | 2.673 GB/s | 1.60x slower |
| Rust, obvious loop plus `assert!` up front | 2.677 GB/s | 1.60x slower |
| Rust, expert `chunks_exact/zip` restructure | 4.297 GB/s | tie |
| Rust, `unsafe` indexed | 4.111 GB/s | 1.04x slower |

The two middle rows carry the lesson. An `assert!` that should let the optimizer drop the checks buys nothing, because LLVM cannot tie it to the loop. To reach the fast class in Rust you restructure into `chunks_exact`, an idiom you have to already know, and even `unsafe` indexing lands a little behind. Whitefoot reaches it from the obvious loop plus one checked line, and keeps every bound. Speed comes from proof, never from dropping a check.

For the full argument, read [docs/why-whitefoot.md](docs/why-whitefoot.md): effect rows the optimizer trusts across opaque boundaries, ownership-driven guard-free vectorization, checked algebraic laws, and the cases where Whitefoot loses.

Each highlight below links to a RESULTS.md under `experiments/`:
- Default floor: the first correctness-green program from a `gpt-5.6-terra` run beat two released Rust crates by 1.653x and 1.098x on locked workloads, every bounds check still in place.
- wc: byte-identical output under LC_ALL=C, about 2x faster than GNU coreutils on the default invocation.
- Classifier kernel: i1 dataflow ties C and safe Rust.
- Checked algebraic laws: 3.3x on reductions, and a false law fails to compile.

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

Both gates run green or the build fails. `make check` covers spec CI, rule tests, soundness probes, performance pins, the codegen-parity corpus, and the conformance suite. `make -C compiler check` runs the wfc test stack, including the self-parse gate. Every finished step commits with one line in the decision log, so a fresh session resumes from `git log` and the log tail alone.
