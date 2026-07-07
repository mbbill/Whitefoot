# xlang

**A systems language written by AI, built to out-optimize the languages humans settle for.**

An optimizer is only as fast as the facts it is handed. Which pointers alias, what a function touches, how each number may overflow — humans won't write these down, so mainstream languages leave them implicit and the compiler spends its life *guessing* (or giving up and emitting the slow, safe thing). An AI writer will state every one of them, for free. xlang is the language that collects those facts and hands them straight to the backend.

Here is what that buys you — three worked examples of one repeatable method. They're a sample, not the feature list; the real ambition is the whole catalog below them.

---

## Orientation

You need four things to read the examples:

- Functions declare **region parameters** in `[...]`, each value's **mode** — `own`, `&'r` (shared), `&uniq 'r` (exclusive) — and an **effect row**: `pure`, `reads('r)`, `writes('r)`, `allocates(...)`, `traps`.
- Arithmetic is a prefix call that **names its overflow behavior**: `iadd.wrap` (wraps), `iadd.trap` (traps on overflow), `iadd.checked` (returns a `Result`).
- There are no operators and no `if`/`while`/`for`. Conditionals are `match`; loops are `loop` + `break`.

Everything is explicit and verbose — on purpose. Verbosity costs a machine writer nothing and gives the compiler everything.

---

## 1. Exclusive borrows → no reloads

```
fn twice_read ['r] (a: &'r i32, b: &uniq 'r i32) -> own i32 writes('r) {
  set deref(b) = iadd.trap<i32>(deref(a), 1_i32);   # write through b
  let s: own i32 = iadd.wrap<i32>(deref(a), deref(a)); # read a again
  return s;
}
```

`a` is a **shared** borrow and `b` is a **unique** (exclusive) borrow, both in region `'r`. By construction they cannot name the same location, and the checker proves it. So after the store through `b`, the compiler *knows* `a` is unchanged and keeps it in a register: it reads `a` **once**, not twice.

In C, `b` might alias `a`, so the compiler must reload `a` after the write. Rust's `&`/`&mut` carry the same exclusivity in principle — but rustc has repeatedly had to *turn the corresponding LLVM `noalias` off* after it triggered miscompiles, and its lifetimes are inferred by elision rather than named. xlang states the region up front and the checker's entire job is to keep that disjointness fact true, so the optimizer can lean on it every time. (In the prototype, dropping these facts turns that one load back into two.)

## 2. Effect rows → calls that optimize like values

```
fn weight (x: own i32) -> own i32 pure {
  return imul.wrap<i32>(x, x);
}

# ... elsewhere:
let a: own i32 = weight(k);
let b: own i32 = weight(k);   # same argument
```

`weight` is declared `pure`, and the checker verifies the body reads nothing, writes nothing, allocates nothing, and cannot trap. That promise lives **in the signature**, so a caller can collapse the two `weight(k)` calls into one, reorder them, or hoist a `pure` call out of a loop — *without ever inlining `weight`*.

Rust and C have no effect in the signature. To do the same, the compiler must inline the callee and *prove* it has no side effects; across a real call boundary it usually can't, so the call stays and the work repeats. A `reads('r)` function gets the weaker-but-still-powerful version: reuse its result as long as nothing writes `'r` in between. These are facts the optimizer normally has to reconstruct — here they are given.

## 3. Per-operation numeric modes → safety that costs nothing

```
loop @l {
  match ige<i32>(i, 5_i32) {
    True() => { break @l; }
    False() => { }
  }
  set sum = iadd.trap<i32>(sum, i);   # overflow-TRAPPING add
  set i = iadd.trap<i32>(i, 1_i32);
}
```

Every add here is `.trap` — it *traps* on overflow instead of silently wrapping. In this bounded loop the compiler proves overflow is impossible and deletes every check; the whole guarded loop folds to a constant. You wrote fully-checked arithmetic and paid nothing for the checks.

Rust ties overflow checking to the **build mode**: on in debug, off in release. So you either pay for checks everywhere or turn safety off to ship fast — and worse, debug and release are optimizing *different programs*, so the build you profile isn't the build you tested. In xlang the mode lives on the operation. It means the same thing in every build, the optimizer always sees the real program, and any check it can discharge is free. Safety and speed stop trading against each other.

---

## Three examples, one method — and a lot more coming

Every example above is the same move: take a fact the optimizer normally has to guess and make it an explicit, checked part of the language. Aliasing, effects, and overflow intent are just the three that fit in five lines. They are not the feature set — they are three points on a much larger surface we are working through on purpose:

- **Data layout & arrays** — shapes and strides carried in the type, so bounds checks fall away and loops vectorize.
- **Dispatch** — closed-world monomorphization and exhaustive `match`: every call is direct and inlinable, no vtables, no devirtualization guesswork.
- **Allocation** — ownership and regions make stack and arena placement a *guarantee*, not an escape-analysis gamble, and there is no GC to outrun.
- **Concurrency** — data-race-freedom by construction, which is exactly what keeps a compiler's reordering and non-interference reasoning sound.
- **Numeric & algebraic law** — per-node float modes, plus properties like associativity as *checked* facts the optimizer may actually use (reassociation, vectorized reductions).
- **Definedness** — no undefined behavior; each operation's poison/trap contract is stated, not lurking.

The catalog is open and still growing, deliberately. Any optimization a language *could* unlock — if only the writer would state the fact behind it — is a candidate to bring in here, because the writer is a machine that will state it without complaint. That is the real goal: **not three clever features, but a language that surfaces every fact we know how to optimize on — more than any compiler could infer, because the writer never minded writing them down.**

## Safe by construction — because that is what keeps the facts true

The optimizer is only allowed to trust those facts because they can't be violated. Data races, use-after-free, dangling references, and uninitialized reads are rejected at compile time citing a rule (example 1's disjointness is one such check), and there is **no writer-reachable `unsafe`** to break the guarantees from the inside. Safety here isn't a separate feature competing with speed — it is the thing that lets the compiler rely on `&uniq`, on `pure`, on a discharged bounds check, and go fast. This matters for an AI writer specifically: an unattended model can't debug a race that shows up at runtime, and it can't quietly cast the safety away to make a red program go green.

---

## Status

A design-and-evidence project with a running prototype, not a finished language.

- **Spec:** [`spec/kernel-spec-v0.4.md`](spec/kernel-spec-v0.4.md). Every rule is traced back to the goals in [`spec/derivation-ledger.md`](spec/derivation-ledger.md); forms not yet validated are flagged as provisional, not hidden.
- **Ownership core:** formally reconciled to **Featherweight Rust** (a soundness-proven calculus) as a strict, sound subset, and cross-checked by a generative model checker over tens of thousands of random programs with zero soundness violations.
- **Compiler:** a demo compiler lowers a growing subset to LLVM and reproduces the wins above.
- **The open bet:** that *weak* models write correct, fast code more reliably here than in mainstream languages is the hypothesis under test — the measurement harness is the next milestone, not a finished result.

## Read next

- [`CONSTITUTION.md`](CONSTITUTION.md) — the goal (performance, with the Rust test, above AI-writability) and the rules behind every decision.
- [`spec/kernel-spec-v0.4.md`](spec/kernel-spec-v0.4.md) — the current kernel spec.
- [`ROADMAP.md`](ROADMAP.md) — where this is going, and [`optimizer-language-research/`](optimizer-language-research/) — the evidence corpus behind it.
