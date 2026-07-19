# Whitefoot

Whitefoot is an experimental systems language built around a question: what changes when the code writer is an AI and the human's job is to approve the result?

Most languages bargain with a human writer. They trade away information for convenience: do not make me state which pointers overlap, what a function can touch, or which algebraic laws an operation obeys. Compilers then spend enormous effort trying to recover those facts through alias analysis, escape analysis, and speculation. Some of the most valuable facts cannot be recovered safely at all.

An AI writer changes that bargain. It does not mind verbosity, need familiar syntax, or benefit from an escape hatch. Whitefoot uses that freedom to make memory corruption, data races, uninitialized reads, and accidental overflow unrepresentable, then reuses the same proofs to optimize checked code. The full design argument, including comparisons with Rust, measured wins, losses, and open questions, is in [Why Whitefoot?](docs/why-whitefoot.md).

This is a programming-language research and compiler project under construction, not an application-development tool. The production compiler, `wfc`, is being written in Whitefoot and is not yet self-hosting or available as a general-purpose command-line tool. The current implementation sequence is in [THE-PLAN.md](THE-PLAN.md).

## The idea underneath

An optimizer is only as fast as the facts it can prove. Whitefoot asks the writer to state those facts in forms the checker can verify, then preserves them all the way to machine code. The optimizer may trust a fact only after the checker proves it. This creates one chain of evidence:

```text
source declaration -> checker proof -> optimizer fact -> machine code
```

Safety checks remain part of the language semantics. A proof may remove a redundant check; a source-level option may not weaken it for speed.

## A small example: proving checks away

The base64 experiment begins with an ordinary indexed loop and one checked entry condition. Here is the `requires` excerpt:

```text
requires {
  let required_out_len: own u64 = len<u8>(deref(out));
  let required_src_len: own u64 = len<u8>(src);
  let required_out_groups: own u64 = ishr.wrap<u64>(required_out_len, 2_u32);
  let required_covered_src: own u64 = imul.wrap<u64>(required_out_groups, 3_u64);
  check ile<u64>(required_src_len, required_covered_src)
      else trap "base64 output capacity";
}
```

The repository contains the [complete source](experiments/port-study/base64/b64.wf). The prover discharges all 27 lowered bounds-check sites while retaining the entry trap. On the recorded 384 MB Apple M4 harness, the same source measured 2.480 GB/s with facts disabled and 4.233 GB/s with the proof enabled.

A separate balanced comparison measured the proved Whitefoot loop against four Rust implementations:

| implementation | throughput | Whitefoot / variant |
|---|---:|---:|
| Whitefoot, indexed loop plus checked `requires` | 4.285 GB/s | 1.000 |
| Rust, ordinary indexed loop | 2.673 GB/s | 1.602 |
| Rust, indexed loop plus an upfront `assert!` | 2.677 GB/s | 1.604 |
| Rust, safe `chunks_exact`/`zip` restructuring | 4.297 GB/s | 0.997 |
| Rust, `unsafe` indexed loop | 4.111 GB/s | 1.040 |

This is one kernel on one machine. Expert safe Rust ties it after restructuring the loop; the narrower result is that Whitefoot's direct indexed shape reaches the same performance class once its checked relation becomes a proof. The full methodology and caveats are in the [base64 results](experiments/port-study/base64/RESULTS.md).

## What follows from the bet

Keeping those facts turns into a set of unusual language decisions.

**Ownership is the aliasing model.** An exclusive borrow, `&uniq`, means no other usable path reaches that memory. There is no `unsafe` escape and no interior-mutability exception that makes the optimizer's `noalias` fact conditional.

**Effects are exact.** Every function declares `pure` or an exact combination of `reads`, `writes`, `allocates`, and `traps`. The checker rejects both undeclared effects and effects that were declared but not exhibited, so callers and the optimizer can treat the row as a fact.

**Overflow behavior is local and visible.** Operations such as `iadd.wrap`, `iadd.trap`, and `iadd.checked` have distinct spellings. There is no build mode that silently changes the program's arithmetic semantics.

**Algebraic laws are checked.** A program may declare that an operation is associative, commutative, or has an identity. The compiler verifies the law before using it to reassociate a reduction; a false law is a compile-time error.

**The syntax and architecture vocabulary are closed.** `match` is the conditional form, `loop` plus `break` is the iteration form, and canonical source has one byte-level spelling. Larger programs use the documented shapes in [PATTERNS.md](PATTERNS.md), such as command buffers, structure-of-arrays pools, linear threading, and checked-law reductions.

## Try the current toolchain

The shortest runnable path uses `prototype/democ`, the stage-0 compiler. From the repository root:

```sh
make examples
```

This compiles and runs two small Whitefoot programs that exercise checked arithmetic, exhaustive `match`, regions, loops, and traps.

Run the full repository gate with:

```sh
make check
make -C compiler check
```

The first command checks the specification, reference semantics, soundness model, optimizer facts, codegen parity, conformance suite, and compiler bootstrap. The second reruns the focused `wfc` gate, including the whole-compiler self-parse.

## Current status and evidence

Whitefoot is a research prototype with measured pieces, not a finished language implementation.

- `prototype/democ` provides the stage-0 compiler and reference codegen path.
- `prototype/checker` provides the reference semantic checker and generative soundness model.
- `compiler/` contains `wfc`, the production compiler under construction. Its current whole-program unit has 535 functions: 45 classify clean, 490 are legal but not yet supported by body semantics, and none are semantic rejects. LLVM lowering currently emits 15 functions.
- The repository records two preregistered weak-writer comparisons against released Rust crates: 1.653x for percent decoding and 1.098x for one-shot UTF-8 parsing. These are two data points, not a general trend.
- The checked-law experiment measured a 3.3x improvement over the serial reduction shape. The `wc` port measured 0.27 seconds against GNU's 0.48 seconds and uutils Rust's 0.56 seconds on its pinned 426 MB workload, while documenting a separate `-l` weakness.
- Proposed sealed components and concurrency work remain research evidence outside the current implementation scope. They are not shipped language features.

Each experiment is self-contained and records its setup, results, and limitations under [experiments/](experiments/README.md).

## Repository guide

| If you want to... | Start here |
|---|---|
| See the current state and authorized work | [THE-PLAN.md](THE-PLAN.md) |
| Read the language specification | [spec/kernel-spec-v0.8.md](spec/kernel-spec-v0.8.md) |
| Understand why each rule exists | [spec/derivation-ledger.md](spec/derivation-ledger.md) |
| Read the project laws and blessed program shapes | [CONSTITUTION.md](CONSTITUTION.md) and [PATTERNS.md](PATTERNS.md) |
| Inspect the production compiler | [compiler/](compiler/README.md) |
| Inspect the stage-0 compiler and checker | [prototype/](prototype/) |
| Review measured evidence | [experiments/](experiments/README.md) |
| Follow the append-only implementation record | [optimizer-language-research/implementation/decision-gates.md](optimizer-language-research/implementation/decision-gates.md) |
| Work in the repository as an agent | [AGENTS.md](AGENTS.md) |

## License

Whitefoot is available under the [MIT License](LICENSE).
