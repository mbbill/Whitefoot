# Whitefoot

Whitefoot is an experiment. Most programming languages were designed for humans to write, and more and more code is written by AI. The question here: if you designed a systems language for that writer instead of a human one, what would change, and how far could you push it?

An AI writer shifts the tradeoffs. It doesn't mind verbosity, doesn't need familiar syntax, and can follow rules a human would find too strict. So Whitefoot makes decisions a language for humans would not. Most of them trace back to one idea, and most come without a benchmark attached, just a choice and a reason.

## The idea underneath

An optimizer is only as fast as the facts it is handed. "These pointers never overlap," "this function has no side effects," "this operation is associative": each is a fact the author knew and the source usually drops. Compilers spend enormous effort guessing them back, through alias analysis, escape analysis, and JIT speculation, and the facts that matter most cannot be guessed safely at all. A human writer wants the freedom to leave facts unstated. An AI writer does not need that freedom. So Whitefoot asks it to state everything once, in a form the checker verifies and the optimizer can trust. Keeping the facts is the whole bet.

That bet turns into a pile of unusual decisions.

## What's unusual about it

**Safety is a side effect, not the goal.** The bug classes that make AI-written code scary, memory corruption, data races, silent overflow, use-after-free, are not caught here. They are unwritable: no sequence of tokens means those things and compiles, and there is no `unsafe` to reach for. The useful part is that the machinery which makes them unwritable, ownership and exclusivity, is the same machinery that hands the optimizer its aliasing facts. Safety and speed come out of the same rules, so you stop trading one for the other.

**Every check stays on, and only a proof turns one off.** Every array access is bounds-checked, an overflow-checked add can trap, and a writer cannot assert a fact into existence. The one way a check comes out is a machine proof at compile time. Here is base64: a single `requires` line states that the output buffer is big enough, and the compiler carries that fact through the loop.

```
fn encode ['r] (out: &uniq 'r buffer<u8>, src: own buffer<u8>) -> own u64
    reads('r), writes('r), traps requires {
  ...
  check ile<u64>(required_src_len, required_covered_src) else trap "base64 output capacity";
} { ... the obvious indexed loop ... }
```

That line lets the prover remove all 27 bounds checks inside the loop, so the same source runs at 2.48 GB/s with the checks and 4.23 GB/s once they are proven away. The entry check stays, and traps even a C caller that passes a short buffer. Against Rust on the same task and machine:

| implementation | throughput | vs Whitefoot |
|---|---:|---:|
| **Whitefoot**, obvious loop plus one checked `requires` line | **4.285 GB/s** | baseline |
| Rust, obvious indexed loop | 2.673 GB/s | 1.60x slower |
| Rust, obvious loop plus `assert!` up front | 2.677 GB/s | 1.60x slower |
| Rust, expert `chunks_exact/zip` restructure | 4.297 GB/s | tie |
| Rust, `unsafe` indexed | 4.111 GB/s | 1.04x slower |

The `assert!` that should let the optimizer drop the checks does nothing; the obvious Whitefoot loop plus one checked line lands with hand-restructured expert Rust. It is one kernel on one machine, and expert Rust ties it, so it is only a hint that the idea might carry.

**Effects are part of every signature, and checked both ways.** A function declares what it does: `pure`, or `reads('r)`, `writes('r)`, `allocates(...)`, `traps`. The checker rejects a function that does more than it declares, and also one that declares more than it does, so the row is a fact rather than a hope. Two things fall out. The optimizer trusts the declared effects across a compiled boundary it cannot see into, and you can read a system's data flow off its signatures: grep for `writes('world)` and you find every place the world can change.

**Ownership is the whole aliasing story, with no exceptions.** An exclusive borrow (`&uniq`) means no other path reaches that memory, which is exactly the `noalias` an optimizer wants, and it holds everywhere because there is no `RefCell`, no `Cell`, no interior mutability at all. One shared-mutable hole would make every aliasing fact conditional, so the language does not have the hole. Whatever you would reach for `RefCell` to do, you do with a different architecture (below).

**You can hand the compiler an algebra, and it checks it.** State that an operation is associative or commutative, and the compiler verifies the claim against the operation's real semantics before it uses it, then reassociates your plain serial loop into parallel accumulators. Declare a law that is not true, signed saturating add claiming associativity, and the program does not compile.

```
law associative(combine);
law commutative(combine);
law identity(combine, 0_u64);
```

A hand-written multi-accumulator loop in C or Rust makes the same associativity claim in silence, and nothing checks it. Here the claim is a checked fact, and a wrong one is a compile error.

**There is exactly one way to write anything, down to the byte.** No formatting freedom (the toolchain rejects non-canonical input instead of reformatting it), no infix operators, no precedence to memorize. Overflow behavior is part of the operation's name, chosen at each call site, with no default to forget:

```
iadd.wrap<u64>(a, b)      // wraps, by declaration
iadd.trap<u64>(a, b)      // traps on overflow
iadd.checked<u64>(a, b)   // returns Result<u64, Overflow>
```

`match` is the only conditional, `loop` plus `break` the only iteration, and there are no comments (documentation is a structured field). This looks punishing for a human and costs an AI nothing but tokens. In return: debug and release compile the same program, a diff shows only real changes, the same source always produces the same bytes, and every rejection cites one rule and one location, which is feedback a model can act on.

**Big data lives in pools; you pass handles, not references.** Instead of borrowing into a structure, code holds a handle, a plain index, a claim ticket rather than the thing itself. Node links in a tree or graph are handles, so the self-referential structs that push real Rust projects into `Pin` or `unsafe` are impossible to write here, and most code needs no borrow reasoning at all. Slots recycle with generation counters, so a stale handle traps instead of silently reading whoever moved in.

**The set of architectures is closed and taught, and a few kernels are sealed.** Whatever a language lets you represent, someone eventually writes, so Whitefoot does not leave the slow shapes representable: the blessed program-scale patterns are a fixed, taught catalog rather than an open field. A short list of kernels that genuinely cannot be written in checked code (a hash table's control bytes, a lock-free queue) ship with trusted internals, and that list is the whole trust surface, roughly ten audited pieces instead of `unsafe` scattered across every crate. Nothing in the language reaches that layer, and anything later machine-proved leaves it.

## What actually works so far

A handful of measurements look good; most of the language is unfinished or unvalidated. The numbers here come from one dev machine, several are dry runs rather than shipped code, and plenty of the design is still open. Read this as a research direction with some early evidence, not a finished tool. Each result has a RESULTS.md under `experiments/`.

- Default floor: the first correctness-green program a fixed model produced came out ahead of two released Rust crates by about 1.65x and 1.10x on locked workloads, bounds checks still in place. Two data points, not a trend.
- wc: byte-identical output under LC_ALL=C, roughly 2x GNU coreutils on the default invocation.
- A classifier kernel reaches parity with C and safe Rust once its boolean dataflow lowers to wide vectors.
- Checked algebraic laws give 3.3x on a reduction (see above), and a law that isn't true fails to compile.
- The sealed kernels are validated as shapes, not shipped: a single-producer/single-consumer queue beat a mature Rust crate on latency in a dry run while trailing it on throughput.

Open questions run through most of it: hand-tuned SIMD still beats the naive shapes, whether these patterns show up often in real code is unknown, and several announced pieces are designs under validation rather than shipped features. The longer write-up, including where Whitefoot loses, is in [docs/why-whitefoot.md](docs/why-whitefoot.md).

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

Two gates guard the repo, and both must pass. `make check` covers spec CI, rule tests, soundness probes, performance pins, the codegen-parity corpus, and the conformance suite. `make -C compiler check` runs the wfc test stack, including the self-parse gate. Every finished step commits with one line in the decision log, so a fresh session resumes from `git log` and the log tail alone.
