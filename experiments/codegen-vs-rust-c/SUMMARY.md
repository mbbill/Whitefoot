# Codegen/perf experiment: xlang vs C / C++ / Rust (2026-07-08)

**R0 evidence.** First non-trivial head-to-head of the bootstrap toolchain against C, C++,
Rust. Machine: Apple M4 (arm64), macOS. democ snapshot @ git `6893813`.

## Framing (no overclaim)
democ has **no optimizer of its own** — it emits LLVM IR text and shells to `clang -O2`.
So this isolates ONE question: does xlang's *automatic* emission of optimizer facts
(`noalias`/`readonly` from `&uniq`/`&`; defined-wrapping / no-UB arithmetic) give the
**shared LLVM backend** more to work with than un-annotated C, and reach **parity** with
Rust? Constitution's honest hypothesis: *parity with Rust + non-defeasible noalias*, NOT
"faster than Rust." Tried to falsify as a skeptic.

## Toolchains
xlang (democ `.ll` → `/usr/bin/clang -O2`) and C/C++ use the **same** Apple clang 21 / LLVM 21
binary (cleanest possible pair). Rust = rustc 1.91.1 / LLVM 21.1.2. Constraint: democ's subset
has no arrays/heap/floats/strings/I-O, so kernels are scalar + borrows, constant-fed, result
consumed via `check…else trap`; correctness verified bit-identical in all languages before timing.

## Results

**Kernel A — splitmix64 mixing ×2·10⁸, xor-accumulated (parity test).**
Hot loop is **byte-identical** across xlang/C/C++/Rust (9 register-only arm64 instructions;
democ's in-loop `alloca`s fully promoted by clang SROA/mem2reg). Runtime (median): xlang 3.017,
C 3.021, C++ 3.063, Rust 3.109 ns/iter → **full parity, within noise. No regression.**

**Kernel B — `accumulate(&uniq acc, & addend, n)` noalias demonstration.**
democ emits `define void @accumulate(ptr noalias %acc, ptr noalias readonly %addend, i64 %n)`
automatically & non-defeasibly from the borrow modes.
- *B1, non-affine body* (`acc=(acc^*addend)*K`): xlang hoists the invariant `*addend` load and
  keeps `*acc` in a register → loop body **identical to C-`restrict` and Rust** (4 instr vs naive
  C's 6 with reload/store). BUT **runtime parity** on the M4 P-core (0.93 ns/iter all round):
  single-accumulator loops are latency-bound; the eliminated load/store hide under OoO +
  store-to-load forwarding. **Codegen/µop/code-size/energy win, not P-core wall-clock.**
- *B-add, affine reduction* (`*acc += *addend`): noalias changes **complexity**. Naive C keeps an
  O(n) vectorized loop behind a runtime alias-check; C-`restrict` and **xlang** collapse it to a
  single `madd` = **O(1)**. Runtime: naive C **60.88 ms** vs C-restrict **2.75** vs **xlang 2.92**
  (10⁹ iters) → **≈22× faster, automatic**, where C needs a hand-written/forgettable/unchecked
  `restrict`. Genuine, measured optimizer-fact win.

## Honest verdict
"Optimizer-first" pays off **at exactly the constitution's claimed level, no more**:
- **Parity** with C/C++/Rust everywhere the scalar subset can express the kernel; **no regression
  anywhere** (democ's naive IR costs nothing at the shared backend).
- **Strictly beats un-annotated C**, automatically & non-defeasibly (noalias from the checked type).
- Does **not** beat Rust — the overclaim was correctly not observed.
- **Where the big wall-clock wins live is not yet expressible:** scalar single-accumulator noalias
  is often latency-hidden; the large payoffs (disjoint-buffer auto-vectorization, array reductions,
  in-order cores) need **arrays/pools/slices we haven't built**. That payoff is **projected, not
  demonstrated** — re-measure (same skeptic discipline) once memory features land.

Artifacts: `scratchpad/exp/` (xl/c/cpp/rs sources, `asm/*.s` hot loops, binaries, `time_it.py`);
democ snapshot `scratchpad/exp-prototype/`.
