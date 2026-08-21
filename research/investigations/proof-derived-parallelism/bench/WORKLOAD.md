# The paired workload

One algorithm, one data structure, one operation sequence, written twice: once
in Whitefoot and once in Rust. Everything that is not the language is held
fixed, so a difference in the numbers is a difference in the two toolchains and
their two schedulers rather than a difference in what was computed.

## Data structure

```
enum LNode {
  Leaf(w: f64, h: f64, out: f64);
  Branch(left: box<LNode>, right: box<LNode>, w: f64, h: f64, out: f64);
}
```

The Rust twin is the same shape node for node:

```rust
enum LNode {
    Leaf   { w: f64, h: f64, out: f64 },
    Branch { left: Box<LNode>, right: Box<LNode>, w: f64, h: f64, out: f64 },
}
```

`Box<LNode>` is deliberate. A `Vec` arena with index children is the faster
Rust idiom for a tree of this kind and would have removed one pointer chase per
node, but it would also have changed the memory layout the two sides walk. The
primary comparison keeps the pointer tree on both sides. That choice is stated
again under limitations.

## Tree builders

Balanced: `build(depth, w)` splits every node into two children of `depth - 1`,
scaling the left child's width by `1.0009765625` and the right child's by
`0.9990234375`. `depth = 0` is a leaf. A depth-`d` tree has `2^(d+1) - 1` nodes.

Skewed: `build(depth, w, phase)` is the same, except that at every other level
(`phase == 0`) the right child is built three levels shallower than the left,
saturating at a leaf. The phase alternates each level, so the skew repeats all
the way down and the tree is deep and lopsided rather than merely deep.

Both builders are deterministic and both are transcribed statement for
statement into Rust.

## The layout pass

Per node, one combined pass modeled on `tests/programs/par_layout.wf`:

1. `cascade(inh, w, h)` — a straight-line float cascade of about two dozen
   operations: padding and border resolution, a content width, a font size, a
   line count through `fceil`, a square root, a clamp pair, a `ffloor`
   fractional adjustment.
2. `measure_words(words, font)` — a loop over one shared `buffer<f64>` metric
   table, scaling every entry by the font size, tracking the widest and the
   running total. The loop is bounded by `len(deref(words))`, the table's own
   length. That is what discharges its index obligation without a `claim`,
   which is what keeps the whole call closure claim-free, which is what makes
   the fold's child pair actualizable.
3. The bottom-up fold `layout(node, words, inh)` recurses into both children
   with the same `child_inh`, sums the two results, adds its own contribution,
   and writes the sum into the node's own `out` slot through `&uniq`.

The two child calls are adjacent `let` statements, which is the shape the
permission judgment reads.

## Operation mapping, Whitefoot to Rust

| Whitefoot | Rust | note |
|---|---|---|
| `fadd.strict` `fsub.strict` `fmul.strict` `fdiv.strict` | `+` `-` `*` `/` | strict IEEE-754 binary64 on both sides; no fast-math, no reassociation, no contraction |
| `ffma.strict(a, b, c)` | `a.mul_add(b, c)` | both are `llvm.fma`: one rounding |
| `fsqrt.strict` | `.sqrt()` | both `llvm.sqrt` |
| `fceil` `ffloor` `fabs` | `.ceil()` `.floor()` `.abs()` | roundToIntegral, staying in f64 |
| `fmin` `fmax` | `.min()` `.max()` | **the one inexact cell** — see below |
| `fgt` | `>` | |
| `reinterpret<f64, u64>` | `f64::to_bits` | |
| `+wrap` `-wrap` on `u64` | `wrapping_add` `wrapping_sub` | |
| `len(deref(words))` | `words.len()` | |

The inexact cell: the kernel specification maps `fmin`/`fmax` to
`llvm.minimum`/`llvm.maximum` — IEEE-2019, NaN-propagating, `-0.0` ordered
below `+0.0`, deterministic — and explicitly refuses `llvm.minnum`/`maxnum`
because their signed-zero tie is unspecified. Rust's `f64::min`/`f64::max` are
exactly the `minnum`/`maxnum` family. The two agree on every input that is not
a NaN and not a signed-zero tie. This workload produces neither: the three
guarded values (`content`, `mixed`, `capped`) are finite and positive at every
node. The claim is checked rather than asserted — the Rust side was rebuilt
with `f64::minimum`/`f64::maximum` and re-run on all twelve configurations; see
`logs/followups.txt` section 4.

## Output

Both sides publish the final fold value as the sixteen lowercase hexadecimal
digits of its f64 bit pattern, most significant first, followed by a newline.
That makes the result comparable across languages, worker counts, and runs by
`cmp` on the bytes rather than by a tolerance.
