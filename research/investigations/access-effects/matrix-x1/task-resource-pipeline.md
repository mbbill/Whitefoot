# Engineering task under candidate x1 — resource-processing pipeline

Derived against the frozen rule set in `CANDIDATE-X1.md` (2026-09-18) and only that
file. The engineering-task framing and the discriminating programs are quoted from
`PROGRAMS.md` where a section touches them (row "Resource-processing pipeline",
plus P8, P9, P10, P14, P15, P18). Nothing under "Not in this candidate", nothing
under "Deferred to a future concurrency and layout round", and neither item under
"Pending owner ruling" is assumed; Rule 16 is used only in §2.6 and §4.3 and the
dependence is stated in §9.

Task text: *Resource processing pipeline: a linear resource (file-like) passes
through several stages, each stage may fail with a typed outcome, loops and early
exits must close the resource exactly once on every path; two independent
resources processed in parallel.*

`PROGRAMS.md` excludes device interaction from this discussion — "IO, FFI and
device interaction are excluded. Resource examples can use ordinary memory and
explicitly linear program values." The resource below is therefore an explicitly
linear program value with a trusted-base `close`, and no step of the derivation
depends on `close` performing IO. Where the *latency* of `close` changes a cost
entry, §5 says so and marks the entry conditional.

Cost convention from the protocol, item 7: runtime performance is the only cost.
Verbosity, extra parameters and duplicated code are not costs. One **move** means
one relocation of `sizeof(T)` bytes; Rule 1's stated consequence — "any value can
be relocated by copying its bytes (memmove, realloc), because nothing inside it
points anywhere" — makes every move a plain byte copy with no fixup, directly
comparable to a C++ move constructor or a Rust `memcpy`.

---

## 1. The question this task tests

Can a resource that the compiler is forbidden to release (Rule 8: "the compiler
never releases them") travel through a sequence of fallible stages, inside a loop
with several early exits and at least one stage whose typed outcome decides
whether the stage kept or returned the resource, such that the resource is closed
exactly once on every path — with no runtime drop flag, no sentinel, and no
occupancy tag — and can two such pipelines run in `par` at no cost?

The subsidiary question, which is where the cost actually lives: once "two
independent resources" becomes "N resources" or "several stages overlapped over
one resource", what does Rule 13's disjointness judgment plus Rule 6's closed set
of move-out operations still admit?

---

## 2. Programs

### 2.1 The resource and the trusted base

```text
linear type File;                                  // Rule 8: "Linearity is declared on
fn close(f: own File)                              // external-resource types"
fn open(name: &DynBox<u8>) -> Result<own File, Code>   reads(name)
```

`open`'s `Err` arm carries no resource, because nothing was created on that edge;
this is the only stage in the whole program whose failure arm is trivially safe.
`Code` and `u64` are Copy.

### 2.2 Two stage interfaces, and which one the rules prefer

**Form R (the resource stays in the driver's frame, stages take a reference).**

```text
enum Fill { Block(u64), Eof, Torn(Code) }              // Copy payloads: a Copy enum
enum Dec  { Wrote(u64), NeedMore, Corrupt(Code) }      // Copy
enum Emit { Flushed, Stalled(u64), Refused(Code) }     // Copy

fn fill(f: &File, buf: &DynBox<u8>) -> Fill            writes(f), writes(buf)
    contract {
        requires len_of(buf) == 0;
        ensures when Block: result > 0;
        ensures when Block: len_of(buf) == result;
        ensures when Eof:   len_of(buf) == 0;
        ensures when Torn:  len_of(buf) == 0;
    }

fn decode(src: &DynBox<u8>, hist: &DynBox<u8>, dst: &DynBox<u8>) -> Dec
    reads(src), writes(hist), writes(dst)
    contract {
        requires len_of(dst) == 0;
        requires cap_of(dst) >= 4 * len_of(src);       // the format's expansion bound
        requires len_of(hist) == 32768;
        ensures when Wrote:    len_of(dst) == result;
        ensures when Wrote:    result > 0;
        ensures when NeedMore: len_of(dst) == 0;
        ensures when Corrupt:  len_of(dst) == 0;
        ensures len_of(hist) == 32768;                 // the adaptive window survives every arm
    }

fn crc(src: &DynBox<u8>, acc: &u64)                    reads(src), writes(acc)

fn emit(sink: &File, src: &DynBox<u8>) -> Emit         writes(sink), reads(src)
    contract { ensures when Stalled: result < len_of(src); }
```

`fill` declares `writes(f)`, not `reads(f)`, because consuming input advances the
resource's own state; that choice is what makes the `par` questions in §3.8 real
rather than vacuous.

**Form T (typestate: the stage owns the resource and returns it in its outcome).**

```text
enum Take { Kept(Code), Back(own File, u64) }          // an enum with an owned linear
                                                       // payload — see gap G2
fn probe(f: own File, mode: u64) -> Take
    contract { ensures when Back: result.1 <= mode; }
```

Rule 15 states which of the two the rules prefer for a large resource: "Passing a
large value in and back out is expressed as a reference parameter with a `writes`
entry, not as move-and-return." Form T is therefore the right shape only when the
resource is register-sized *or* when the stage genuinely needs to decide, at
runtime, whether to close it — which is exactly the case the task names and which
Form R cannot express, because `close` takes `own File` and a stage holding only
`&File` can never consume it.

### 2.3 The ordinary program: a five-stage streaming loop with eight exit paths

```text
enum Run { Done(u64, u64), Short(u64), Damaged(Code), NoRoom(Code), SinkLost(Code) }
                                                       // every payload Copy: Run is Copy

fn run(f: own File, sink: own File, blk: u64) -> Run
{
    buf = match DynBox::new<u8>(blk) {
        Ok(b)  => move b,
        Err(c) => { close(move f); close(move sink); return NoRoom(c) }        // exit 1
    }
    dst = match DynBox::new<u8>(4 * blk) {
        Ok(b)  => move b,
        Err(c) => { close(move f); close(move sink); return NoRoom(c) }        // exit 2
    }
    hist = match DynBox::filled<u8>(32768, 0) {
        Ok(b)  => move b,
        Err(c) => { close(move f); close(move sink); return NoRoom(c) }        // exit 3
    }
    acc   = 0
    total = 0

    loop {
        invariant e1: len_of(buf) == 0;
        invariant e2: len_of(dst) == 0;
        invariant e3: len_of(hist) == 32768;

        match fill(&f, &buf) {
            Eof      => { break }                                              // exit 8 below
            Torn(c)  => { close(move f); close(move sink); return Damaged(c) } // exit 4
            Block(n) => { }
        }

        match decode(&buf, &hist, &dst) {
            Corrupt(c) => { close(move f); close(move sink); return Damaged(c) }   // exit 5
            NeedMore   => { truncate(&buf, 0); continue }
            Wrote(m)   => {
                crc(&dst, &acc)
                match emit(&sink, &dst) {
                    Refused(c) => { close(move f); close(move sink); return SinkLost(c) } // exit 6
                    Stalled(k) => { total = total + k
                                    close(move f); close(move sink); return Short(total) } // exit 7
                    Flushed    => { total = total + m }
                }
                truncate(&dst, 0)
            }
        }
        truncate(&buf, 0)
    }
    close(move f)                                                              // exit 8
    close(move sink)
    Done(total, acc)
}
```

Eight exit paths; each closes both resources exactly once, and no path closes
either twice. No drop flag, no `has_file` boolean, no sentinel `File` value, and
no tag anywhere: the branches the writer already needed for the typed outcomes
carry the whole obligation. This is `PROGRAMS.md` P8's stated requirement — "no
runtime drop flag; the writer's own branches carry the state" — met literally.

Note what is **not** written: `buf = DynBox::new<u8>(blk)?`. `?` returns from the
enclosing function on the `Err` edge with `f` and `sink` live and unconsumed,
which Rule 8's "must be consumed by an explicit operation on every exit path"
refuses. Every `?` in a scope where a linear value is live becomes the `match`
above. The source is longer; the emitted branch count is identical, because the
`Err` test exists in both forms and the cleanup call sits on the taken edge.

### 2.4 The strongest program: variant-decided ownership carried across a loop back edge

This is the case where no static analysis can help. Whether the resource is still
owned by the driver is decided by a runtime variant returned from a separately
compiled stage; the loop index into the stage table is data; and the local holding
the resource is absent in the middle of the loop body and present again on one of
the two back edges.

```text
fn probe(f: own File, mode: u64) -> Take
{
    if unusable(&f, mode) {                    // unusable(f: &File, m: u64) -> Bool   reads(f)
        close(move f)                          // the stage discharges it itself
        return Kept(E_UNUSABLE)                // ... and says so in the variant
    }
    k = measure(&f, mode)                      // measure(f: &File, m: u64) -> u64     reads(f)
    Back(move f, k)                            // ... or hands it back in the variant
}

fn adaptive(f0: own File, modes: &DynBox<u64>) -> Run    reads(modes)
{
    f = move f0
    i = 0
    loop {
        invariant m1: i <= len_of(modes);

        if i >= len_of(modes) {
            close(move f); return Short(0)                       // exit A: resource present
        }
        match probe(move f, modes[i]) {                          // f leaves the frame here
            Kept(c)    => { return Damaged(c) }                  // exit B: resource absent,
                                                                 //         nothing to close
            Back(g, k) => {
                f = move g                                       // present again (gap G4)
                if k > 0 { close(move f); return Done(k, 0) }    // exit C: resource present
                i = i + 1
            }
        }
    }
}
```

Three exits with three different ownership states, one of them decided by a
variant the caller cannot see through, and a back edge on which the resource must
be present again. In C++ this is the classic `goto cleanup` shape whose bug is
double-close on the `Kept` path; in Rust it is `let f = match probe(f, m) { … }`
with `Drop` and, because ownership is conditional, a compiler-inserted drop flag.

### 2.5 The task's parallel requirement: two independent resources

**Owning form** — each arm carries its own pair of resources and closes them:

```text
struct Pair<A, B> { x: A, y: B }                       // Rule 1: owned fields only

fn two(a: own File, sa: own File, b: own File, sb: own File, blk: u64) -> Pair<Run, Run>
{
    par { ra = run(move a, move sa, blk);  rb = run(move b, move sb, blk) }
    Pair { x: ra, y: rb }
}
```

**Reference form** — the driver keeps the obligation and the arms only work:

```text
fn run_ref(f: &File, sink: &File, blk: u64) -> Run     writes(f), writes(sink)
    // the body of §2.3 with every close(...) and the by-value parameters removed;
    // the failure arms just return, because the caller still owns both resources

fn two_ref(a: own File, sa: own File, b: own File, sb: own File, blk: u64) -> Pair<Run, Run>
{
    par { ra = run_ref(&a, &sa, blk);  rb = run_ref(&b, &sb, blk) }
    close(move a); close(move sa); close(move b); close(move sb)
    Pair { x: ra, y: rb }
}
```

Both are accepted (§3.8) and both are free. The reference form is the one that
scales, because it separates *working on* a resource from *discharging* it — the
distinction that decides every remaining parallel question in §4.

### 2.6 N resources: a table of files

```text
files: DynBox<File>                 // linear by Rule 6: "A DynBox whose element type
outs:  DynBox<u64>                  // is linear is itself linear (Rule 8)"

fn work(f: &File, slot: &u64)  writes(f), writes(slot)

// (a) parallel processing — the admitted counted-loop family, per-element map:
//     each iteration touches files[i] and outs[i] for distinct i
par-map over i in 0..n { work(&files[i], &outs[i]) }

// (b) the two-arm range form:
par { span(&files[0..mid], &outs[0..mid]);  span(&files[mid..n], &outs[mid..n]) }

fn span(fs: &DynBox<File>, os: &DynBox<u64>)  writes(fs), writes(os)
    contract { requires len_of(os) == len_of(fs); }

// (c) the discharge, which is NOT parallel (see §3.9 and gap G6):
while len_of(files) > 0 { g = pop(&files); close(move g) }
```

(b) uses indices as handles into a DynBox, which is the Rule 16 pattern; §9 states
the dependence.

### 2.7 Overlapping the stages of one resource: double-buffered nested `par`

The stages of §2.2 are sequentially dependent per block through `hist` (the
adaptive window), so no range split of the input can parallelize `decode`. The
only overlap available is stage overlap. `par { A; B }` takes two arms as written,
and n arms are reached by nesting at no runtime cost.

```text
// unrolled by two so that no local is ever rebound: A and B keep fixed roles
loop {
    invariant f1: len_of(bufA) == 0;
    invariant f2: len_of(dstB) == 0;

    par { r1 = fill(&f, &bufA);
          par { r2 = decode(&bufB, &hist, &dstB);
                par { crc(&dstC, &acc);
                      r4 = emit(&sink, &dstD) } } }
    // -- barrier --   then match r1, r2, r4 sequentially; each failure arm closes
    //                 both resources and returns, exactly as in §2.3
    rotate_roles_by_unrolling                     // the second half of the unrolled body
    par { r1 = fill(&f, &bufB);
          par { r2 = decode(&bufA, &hist, &dstA);
                par { crc(&dstD, &acc);
                      r4 = emit(&sink, &dstC) } } }
    // -- barrier --   same handling
}
```

Four arms, four disjoint roots per arm, one barrier per block. This is the
"batched fork-join with two buffers" that the rule file's "Known costs already
recorded" section names as the available form; §5 and §6 price it.

---

## 3. Rule-by-rule trace at the interesting points

### 3.1 The types are well formed

- Rule 1: "Every struct, enum, tuple, array, slice-like value, Box, DynBox, and
  generic instantiation holds only owned values." `Fill`, `Dec`, `Emit`, `Run`,
  `Pair<Run, Run>` and `Take` hold owned values only. In particular `Take::Back`
  holds `own File`, not `&File`, so Rule 1's rejection of `enum Bad2 { A(&Int), B }`
  does not reach it. Accepted.
- Rule 8: `File` is linear by declaration. `DynBox<File>` in §2.6 is linear, which
  Rule 6 restates: "A DynBox whose element type is linear is itself linear
  (Rule 8)."
- `Take` is an **enum** with a linear payload. Rule 8's propagation sentence names
  "a struct, array, Box, or DynBox"; it does not name enum or tuple, although
  Rule 1's aggregate list does. The entire failure-accounting design of Form T
  rests on `Take` being linear — see gap **G2** and the unsafe variant U4 in §3.10.

### 3.2 Closed exactly once on every exit path

Rule 8, the governing sentence: "Linear values must be consumed by an explicit
operation on every exit path; the compiler never releases them."

The eight exits of §2.3:

| Exit | Where | `f` | `sink` |
|---|---|---|---|
| 1 | `buf` allocation fails | `close(move f)` on the arm | `close(move sink)` on the arm |
| 2 | `dst` allocation fails | same | same |
| 3 | `hist` allocation fails | same | same |
| 4 | `fill` returns `Torn` | same | same |
| 5 | `decode` returns `Corrupt` | same | same |
| 6 | `emit` returns `Refused` | same | same |
| 7 | `emit` returns `Stalled` | same | same |
| 8 | `fill` returns `Eof`, loop breaks | after the loop | after the loop |

The `NeedMore` and `Flushed` arms are not exits; they reach the back edge with both
resources present, which is the same state as the loop header, so nothing is owed
there. At-least-once therefore holds by enumeration.

At-most-once needs the converse, and the rule text states it only for affine
values: Rule 8 says "Affine values are consumed at most once" and then states the
linear obligation without an upper bound. The three-class sentence "A type is copy,
affine, or linear" makes the classes disjoint, so the affine sentence does not
cover `File`. See gap **G1**. The nearest text that supplies the bound is Rule 12's
"every path is either wholly present or the program cannot name it" together with
Rule 6's "consuming a whole local (`move x`)": after `close(move f)` the local is
not present, so `close(move f)` a second time names nothing. The derivation below
uses that reading; the whole point of the task depends on it.

**Rule 14 matters here in a way that is easy to miss.** "Allocation and release
carry no effect entry and never make two parallel arms conflict. Allocation
returns a `Result` and never traps." Because the failed `DynBox::new` has no
effect entry, it writes nothing, invalidates no fact (Rule 11: "A fact that
mentions a path is invalidated when that path is written"), and — crucially — is
not itself an exit. The `Err` arm is an ordinary branch on which the writer's own
`close` calls sit. There is no hidden unwinding edge, so the enumeration above is
complete: Rule 14's "never traps" is what makes "every exit path" a finite,
writer-visible set. In C++ the same claim requires reasoning about every
potentially-throwing expression in the loop body.

### 3.3 The loop with no ownership change (§2.3)

- Rule 11 admits the three loop-header invariants: each is `invariant name:
  affine_expr compare_op affine_expr` over the measures `len_of`, which Rule 11
  lists first among the fact forms ("affine comparisons over measures and integer
  values ... loop-header invariants").
- `e1` is re-established on the `NeedMore` edge by `truncate(&buf, 0)` whose
  `ensures len_of(buf) == n` gives `len_of(buf) == 0`, and on the `Flushed` edge by
  the same call at the bottom of the body.
- `e2` is re-established on the `Corrupt`/`NeedMore` edges by `decode`'s own
  `ensures when …: len_of(dst) == 0`, and on the `Flushed` edge by
  `truncate(&dst, 0)`.
- `e3` is re-established by `decode`'s `ensures len_of(hist) == 32768`, which is
  stated outside the `when` clauses and therefore holds on every arm. Without it
  Rule 11's invalidation sentence would kill the fact at the `writes(hist)` call
  and `decode`'s own `requires len_of(hist) == 32768` would fail on the second
  iteration. The repair is a contract line; it costs nothing at runtime, because
  Rule 6's `len_of` "is a runtime number stored in the block header" that nobody
  has to reload for the proof — the proof is erased.
- Rule 12: "At a join, facts are intersected (a fact survives only if it holds on
  every incoming edge)." Both back edges carry `e1`, `e2`, `e3`, so the header
  facts survive.

### 3.4 The strongest program's loop (§2.4), step by step

**Bounds.** `modes[i]` requires `i < len_of(modes)` by Rule 7 ("Every index must be
proved in bounds"). The dominating branch `if i >= len_of(modes) { … return }` has
a false edge carrying `i < len_of(modes)` — Rule 11's "refinement facts from a
dominating branch" over a single affine comparison. No disequality step and no
extra compare beyond the one the writer wrote: Rule 7's own example is exactly this
shape, "if i < len_of(buf) { use(&buf[i]) } // the test establishes the fact; one
compare, no trap".

**The invariant.** `m1: i <= len_of(modes)` is re-established after `i = i + 1`
from `i < len_of(modes)` on that path. `len_of(modes)` is not written anywhere in
the loop — `probe`'s row is empty of `modes` and its by-value argument is a copy
(Rule 9: "A by-value parameter has no effect entry: the call site records the
consumption (for `move`) or the read (for a copy) of the argument's place") — so
Rule 11's invalidation clause never fires on the measure.

**The call.** `probe(move f, modes[i])` substituted under Rule 10: clause 2, "A
by-value argument contributes a consumption (`move`) or a read (copy) of its place
to this comparison." The call contributes a consumption of the root `f` and a read
of `modes[i]`. Clause 1 compares them pairwise: different roots, disjoint,
accepted. There is no live reference into `f` at this point, so clause 3 has
nothing to invalidate.

**Inside `probe`.** `unusable(&f, mode)` forms a reference to the by-value
parameter, the call returns, the reference is dead, and only then does
`close(move f)` run. Had the reference still been live, Rule 10's third example
would apply directly — "put(slot, move b) // rejected: move b writes the prefix b
of slot's path (Rule 3)" — because the move of `f` writes the prefix `f` of the
reference's path and Rule 3 invalidates it. As written the program never uses a
reference after the move, so nothing is rejected, and the ordering costs nothing:
the same instruction order is what a C++ or Rust implementation emits.

**The back edge.** This is the interesting point. On the `Kept` arm the function
returns, contributing no back edge. On the `Back` arm, `f = move g` makes `f`
present again and the back edge carries "f present"; the entry edge carries "f
present" from `f = move f0`. So a flow-sensitive presence analysis accepts the
loop, and Rule 12's sentence "No place is ever partially moved: every path is
either wholly present or the program cannot name it" is the text that gives that
analysis its notion of presence.

What the rule set does **not** provide is a way for the writer to *state* it.
Rule 11 enumerates the fact forms — "affine comparisons over measures and integer
values, refinement facts from a dominating branch, loop-header invariants
`invariant name: affine_expr compare_op affine_expr`, explicit `use` steps inside
an `invariant`, and callee contracts" — and none of them can say "the linear local
`f` is present here". `invariant` takes an affine comparison, and presence is not a
comparison between affine expressions. So if the automatic presence analysis ever
fails to see through a loop of this shape, there is no written step that repairs
it. See gap **G4**. Nothing in §2.4 needs the repair; the gap is about what the
writer can do when a harder shape does.

**Re-binding a consumed local.** `f = move g` assigns to a local that is currently
absent. Rule 6's list of moves *out of* storage is closed; moving *into* a place is
not restricted by it, and Rule 6 itself writes an overwrite of a place in the
growth path ("replaces the old one"). There is no old value to release here
because the place is absent, so Rule 8's release rules are not reached. This one is
decided by the text, not a gap.

**Both resources' obligations across the variant.** Exit B (`Kept`) does not close
anything and must not: the resource was consumed inside `probe` on that path.
Exits A and C do close, and must. Because Rule 8's obligation is per exit path and
not per join, no intersection is taken and no drop flag is needed — the variant tag
that the writer already tests *is* the ownership evidence, and it is a tag the
program needed anyway for its typed outcome. This is the sharpest positive result
of the whole task: **the typed-outcome enum and the ownership evidence are the same
runtime word.** Rust must additionally prove that the `File` is not dropped twice
across the `match`, and for a conditionally-owned local in a loop rustc's standard
answer is a drop flag on the stack plus a branch at the scope end.

### 3.5 Effect rows and Rule 9/10 at each stage call

- Rule 9: "An effect row lists `reads(path)` and `writes(path)` where each path
  starts at a reference parameter." Every row in §2.2 does. `buf`, `dst`, `hist`,
  `acc`, `total`, `i` are locals of `run`/`adaptive`, so they contribute no entry
  to those functions' own rows; `run` and `adaptive` declare nothing about them,
  which is correct because a caller cannot name them.
- Rule 9's body clause: "A function body is checked against its own row: every
  statement's effect and every callee's substituted row must be covered by the
  declared row." `run_ref`'s declared row is `writes(f), writes(sink)`; its callees
  substitute to `writes(f), writes(buf)`, `reads(buf), writes(hist), writes(dst)`,
  `reads(dst), writes(acc)` and `writes(sink), reads(dst)`, whose entries outside
  `{f, sink}` all name locals. Covered.
- Rule 10 clause 1 at `decode(&buf, &hist, &dst)`: `reads(buf)`, `writes(hist)`,
  `writes(dst)` — three distinct roots, "different roots" disjointness, accepted.
  The same call written `decode(&buf, &hist, &buf)` is **rejected**: `reads(buf)`
  and `writes(buf)` overlap and one is a write. That rejection costs nothing,
  because an in-place stage is expressed by giving it one reference and declaring
  the write:

  ```text
  fn filter(b: &DynBox<u8>)  writes(b)      // in-place, one buffer, accepted
  ```
  which is the idiomatic in-place form and saves the same buffer and pass that C++
  saves. The refusal is of *aliasing two parameters*, not of in-place work.
- Rule 10 clause 1 at `emit(&sink, &dst)`: `writes(sink)`, `reads(dst)` — distinct
  roots. Accepted. Had `sink` been the same resource as `f` — a read-write pipe
  over one descriptor — the call `emit(&f, &dst)` inside a body that also holds
  `fill(&f, …)` would still be accepted, because Rule 10 compares effects *within
  one call*, not across statements; the two statements are sequential and ordered
  by source order.

### 3.6 Reference validity around the close (Rule 3, Rule 10 clause 3)

Rule 3: validity "is established when p is formed and invalidated when any proper
prefix of p's path is written, moved out of, replaced, or freed ... Writing the
storage at p's path or below it (a content write) does not invalidate p."

- `fill(&f, &buf)` declares `writes(f)`. `&f` names the root path `f`; the write is
  *at* the path, not at a proper prefix, so nothing that names `f` or below is
  invalidated. The driver's own `f` is untouched as a place.
- `close(move f)` is a move out of the root. Any reference formed from `f` and
  still live would be invalidated by Rule 3 and refused by Rule 10 clause 3 ("A
  live reference outside the call whose path has a proper prefix among the call's
  write paths becomes invalid after the call"). In §2.3 and §2.4 no reference into
  `f` outlives the call that created it, because Rule 4 forbids storing or
  returning one anyway: "A reference may be bound to a local, passed as a call
  argument, and used within the function that formed it or received it."
- `truncate(&buf, 0)` declares `writes(buf)`. Any live `&buf[k]` has `buf` as a
  proper prefix and is invalidated. The loop holds none across the truncate, so
  nothing is refused. Each new iteration re-forms whatever it needs; Rule 4's cost
  model is one address computation, "one address computation" as its own example
  says, and here it is zero because the driver passes `&buf` whole.

### 3.7 Bounds inside the stages

Rule 7's requirement — "Every index must be proved in bounds; when the proof is
unavailable the program tests the measure, which is ordinary data" — applies inside
`fill`, `decode`, `crc` and `emit`, not in the driver. Each of those walks
`0..len_of(src)` or `0..cap_of(dst)` with the loop counter as the only index, so
the bound is the loop's own affine invariant and no compare survives into the
emitted code. Against **C++** this is parity (no check either way). Against **safe
Rust** it removes one compare-and-branch per element access, unless LLVM elides it
— which it does for the simple counted forms and does not reliably for the
`hist`-window indexing inside an LZ77-style decoder, where the offset is data.
That is a real, if modest, win for the candidate on the hottest loop of this task.

### 3.8 The par blocks

Rule 13: "`par { A; B }` is accepted when A's write paths are disjoint from B's
read and write paths and vice versa, using the same path-overlap and
index/range-disjointness judgment as Rule 10. Read/read overlap is allowed.
Allocation and release are not effects (Rule 14)."

- **§2.5 owning form.** Arm A contributes consumptions of `a` and `sa`; arm B of
  `b` and `sb` (Rule 10 clause 2, imported by Rule 13's "the same … judgment as
  Rule 10"). Four distinct roots. Accepted.
- **§2.5 reference form.** Arm A: `writes(a), writes(sa)`. Arm B: `writes(b),
  writes(sb)`. Different roots. Accepted.
- **Allocation inside both arms.** `run` and `run_ref` each allocate three buffers.
  Rule 14: "Allocation and release carry no effect entry and never make two
  parallel arms conflict … Addresses are not observable, so allocator concurrency
  does not affect program determinism." This is the sentence that makes the whole
  parallel requirement free — without it, a shared heap would be a write path in
  both arms and every parallel pipeline in this task would be rejected. Rule 14's
  own example is exactly this program: `par { a = build(&x)?; b = build(&y)? }
  // both allocate: accepted`.
- **§2.6(a) per-element map.** Rule 13 names the family: "The existing counted-loop
  forms (per-element maps, adjacent ranges passed to a helper, admitted reductions)
  are expressed with range references", and its example `par { bump(&v[i]);
  bump(&v[j]) } // needs i != j` is the same judgment for two explicit indices.
  Accepted for distinct `i`.
- **§2.6(b) adjacent ranges.** Rule 13's own example form: `par {
  kernel(&v[0..mid], &out[0..mid]); kernel(&v[mid..n], &out[mid..n]) } // disjoint
  ranges: accepted`. Accepted.
- **§2.7 nested four-arm par.** Arm 1 `writes(f), writes(bufA)`; arm 2
  `reads(bufB), writes(hist), writes(dstB)`; arm 3 `reads(dstC), writes(acc)`;
  arm 4 `writes(sink), reads(dstD)`. All roots distinct across arms. Accepted, one
  pair at a time through the nesting. The unrolling in §2.7 exists so that no
  buffer local is rebound between phases; it is a source transformation with no
  runtime effect.
- **`crc` accumulating into `acc`.** With the crc split across arms it would be a
  reduction, and Rule 13 admits only "admitted reductions" from the existing fixed
  table. CRC32 combination is not integer wrap-add, min, max, bitwise or Boolean,
  so it is not on that table and the crc stays in one arm, computed in block order.
  That is the correct answer and costs nothing here, because one arm of four is not
  the critical path.

### 3.9 What the rules refuse, and why

- **`par { push-style consumption from the same container }`.** Two arms both
  calling `pop(&files)` substitute to `writes(files)` twice: same root, one is a
  write, not disjoint. Rejected by Rule 13. Correct and necessary — the block
  header's `len_of` is a single runtime number.
- **Draining a range in parallel.** The natural rewrite is to give each arm a range
  reference and let it consume its own elements. Rule 6 types every window
  operation on the whole block — `fn pop<T>(buf: &DynBox<T>) -> own T writes(buf)`
  — and Rule 7 says a range reference is one of the "Indexable things", with
  `len_of(part) == hi - lo`. Whether a range reference *is* a `&DynBox<T>` and
  therefore admits `pop`/`truncate` is not stated, and the two readings differ
  sharply: under the permissive one `pop(&files[lo..hi])` typechecks against a
  single block header whose `len_of` belongs to the whole block. See gap **G6**.
  The derivation uses the tight reading (window operations need the whole block),
  under which parallel drain is refused and §5 prices the rewrite.
- **Fan-in to one sink.** `par { emit(&sink, &dstA); emit(&sink, &dstB) }`
  substitutes to `writes(sink)` in both arms: same root, rejected by Rule 13. There
  is no escape, because "Not in this candidate" lists "channels or atomics", so an
  internally-synchronized writer cannot be expressed at all. §5 and §6 price the
  rewrite.
- **Software-pipelining a stage against itself.** Overlapping `decode` of block k
  with `decode` of block k+1 requires two writers of `hist`: same root, rejected.
  This is correct — the adaptive window genuinely serializes the stage — and it is
  the reason §2.7 overlaps *different* stages rather than iterations.
- **`?` inside a `par` arm while a linear value is live.** Rule 14's example
  contains `par { a = build(&x)?; b = build(&y)? }`. If arm A takes the `Err` edge
  and returns from the enclosing function, the rule text does not say what happens
  to arm B, nor whether a linear value live in the enclosing frame has met Rule 8's
  "on every exit path" obligation on that edge. See gap **G5**. §2.7 sidesteps it
  entirely: the arms return outcome values and every `match`, `close` and `return`
  sits after the join. That rewrite costs one block of work in the sibling arm when
  a failure occurs (§5 row 10) and nothing at all otherwise.

### 3.10 Unsafe variants: five attempts to lose or double-close the resource

- **U1, double close.** `close(move f); close(move f)`. The second call names a
  local that is not present. Rule 6's enumeration of moves out of storage names
  "consuming a whole local (`move x`)" as a one-shot event and Rule 12 says "every
  path is either wholly present or the program cannot name it". Rejected — under
  the reading of gap **G1**.
- **U2, close in both arms.** `par { close(move f); close(move f) }`. Two
  consumptions of the root `f`; Rule 10 clause 2 turns each into an effect on `f`
  and clause 1 rejects two overlapping effects one of which is a write. Rejected.
- **U3, close while the sibling reads.** `par { close(move f); n = measure(&f, 0) }`.
  Consumption of `f` versus `reads(f)`: overlap, one is a write. Rejected by
  Rule 13. Note this is stronger than the sequential rule needs: the same two
  statements in source order are fine, because the reference is formed before the
  move.
- **U4, leak by discarding the outcome.** `probe(move f, m);` with the returned
  `Take` value dropped on the floor. Whether this is rejected is exactly gap
  **G2**: if linearity propagates through an enum, the `Take` value is linear,
  unconsumed, and refused; if it does not, the `File` inside `Back` is lost
  silently with no diagnostic and no memory error. This is the single most
  load-bearing unresolved sentence for this task — every typed outcome that carries
  the resource back is an enum.
- **U5, park the resource in a container to dodge the obligation.**
  `push_nogrow(&pool, move f)` and then let `pool` go out of scope. Rule 6 says the
  container inherits the obligation: "A DynBox whose element type is linear is
  itself linear (Rule 8) and must be truncated to zero by the program before it can
  go out of scope." So the dodge fails — but the prescribed discharge is itself
  undecided, because `truncate` has no stated behavior for the linear values in the
  slots it removes: releasing them contradicts Rule 8's "the compiler never
  releases them", and not releasing them loses them. See gap **G7**. The safe
  reading, and the one §2.6(c) uses, is `while len_of(files) > 0 { g = pop(&files);
  close(move g) }`, after which the truncate-to-zero requirement is already
  satisfied.

---

## 4. Parallel opportunities: admitted and refused

### 4.1 Admitted, at zero cost

1. **Two (or any fixed number of) independent resources**, owning form or reference
   form. Rule 13, different roots. This is the task's literal parallel requirement
   and it is free.
2. **Per-element map over N resources** with `work(&files[i], &outs[i])` for
   distinct `i`. Rule 13's per-element family.
3. **Adjacent ranges** `span(&files[0..mid], …)` / `span(&files[mid..n], …)`.
   Rule 13's example form verbatim.
4. **Read/read overlap**: any number of arms computing statistics over the same
   buffer. Rule 13, "Read/read overlap is allowed."
5. **Stage overlap over one resource** via §2.7's nested `par` with per-stage
   buffers. Admitted because the stages' roots are distinct.
6. **Allocation in every arm simultaneously.** Rule 14, explicitly.
7. **Independent per-resource `close` for a fixed arity**: `par { close(move a);
   close(move b) }` — distinct roots, accepted. This matters when `close` is not
   free.

### 4.2 Refused

1. **Parallel drain of a container of linear resources.** Two `pop(&files)` calls
   conflict on the root; a per-range drain depends on gap **G6** and is refused
   under the tight reading.
2. **Fan-in of K workers into one sink.** `writes(sink)` in two arms.
3. **Elastic producer/consumer between stages.** No channel exists in the
   candidate; the rule file itself defers one ("A channel primitive in the trusted
   base (ownership-transfer queue) for producer/consumer pipelines").
4. **Cooperative early abort of a par arm.** No atomics, so no shared flag; a
   sibling arm cannot be told to stop.
5. **CRC (or any non-tabled reduction) split across arms.** Rule 13 admits only the
   "admitted reductions".
6. **Overlapping a stateful stage with itself across blocks.** Correct refusal.

### 4.3 Refusals that C++ and Rust also make

Items 1 and 2 of §4.2 are refused by Rust's `&mut` for the same structural reason,
and Rust's escape is `Mutex`/`Arc`/atomics, which the candidate does not have.
Item 6 is refused by every language that respects the data dependence. Only items
2, 3 and 4 represent capability the baselines have and the candidate does not, and
item 3 is the expensive one.

---

## 5. Cost table versus an idiomatic C++ or Rust implementation

Baseline: C++17 with RAII (`std::unique_ptr`-style `File` wrapper, `std::thread`
plus `std::mutex` for fan-in, a bounded `concurrent_queue` for pipelining) or Rust
with `Drop`, `&mut`, `rayon` and `crossbeam::channel`. Deltas are per the named
operation. "—" means no difference.

| # | Operation | Baseline form | x1 form | Δ loads | Δ branches | Δ copies/moves | Δ allocations | Δ parallelism |
|---|---|---|---|---|---|---|---|---|
| 1 | Stage call, resource by reference | `stage(&mut f, &mut buf)` | `fill(&f, &buf)` `writes(f), writes(buf)` | — | — | — | — | — |
| 2 | Stage call, resource by value, register-sized | `fn(File) -> Result<File, E>` returned in registers | `probe(move f, m) -> Take` | — | — | — | — | — |
| 3 | Stage call, resource large | sret pointer, one memcpy in, one out | same, or Rule 15's reference form ("expressed as a reference parameter with a `writes` entry") | — | — | — | — | — |
| 4 | Typed-outcome branch | `match result { … }` | `match fill(…) { … }` | — | — | — | — | — |
| 5 | Close on an unconditional exit path | C++ dtor at scope end; Rust `Drop` | `close(move f)` written on the path | — | — | — | — | — |
| 6 | Close where ownership is conditional (§2.4) | C++ hand-written `goto cleanup` plus a `bool owned`; Rust: compiler-inserted drop flag | the variant tag the program already tests | — | **−1** | — | — | — |
| 7 | Drop-flag storage for the conditional local | 1 stack byte per conditionally-owned local (Rust) | none | — | — | — | — | — |
| 8 | `?` on a fallible step while a resource is live | `?` plus `Drop` (Rust); RAII (C++) | `match` with `close` on the `Err` arm | — | — | — | — | — |
| 9 | Element access inside a stage's inner loop | C++ raw pointer: none. Safe Rust: 1 cmp+branch when not elided | Rule 7 proof from the loop's affine invariant | — | **0 vs C++, −1 vs safe Rust** | — | — | — |
| 10 | Scratch buffers | allocated once outside the loop | same; Rule 14 gives allocation no effect entry, so it never serializes arms | — | — | — | — | — |
| 11 | In-place filter stage | `filter(&mut b)` | `filter(b: &DynBox<u8>) writes(b)` | — | — | — | — | — |
| 12 | Two independent resources in parallel | 2 threads | `par { run(move a, move sa, blk); run(move b, move sb, blk) }` | — | — | — | — | — |
| 13 | N resources, processing only | `par_iter_mut()` | per-element map over `&files[i]` | — | — | — | — | — |
| 14 | **N resources, closing** | `par_iter().for_each(drop)` | `while len_of(files) > 0 { close(move pop(&files)) }` | — | — | — | — | **N closes serialized** |
| 15 | **N resources, closing, K-way rewrite** | as above | split `files` into K owned DynBoxes by `pop`+`push_nogrow`, then `par` over the K owners | — | — | **+2N moves of `sizeof(File)`** | **+K** | recovers K-way |
| 16 | **Fan-in of K workers to one sink** | `Mutex<Writer>`, streaming; ~1 uncontended lock per block | per-worker output DynBox, sequential concatenation after the join | — | — | **+1 full n-byte copy** | **+K** (or +1 oversized) | streaming lost: first-output latency becomes total latency; peak output memory ≈ 2× |
| 17 | Fan-in where per-worker output sizes are known in advance | same | `par` over disjoint ranges of one output DynBox (Rule 13's example) | — | — | — | — | — |
| 18 | Fan-in where sizes are not known (a decoder) | as #16 | either #16, or a sizing pass first | — | — | see #16 | see #16 | or **+1 full decode pass** |
| 19 | **Stage-to-stage overlap, one stateful resource** | 4 threads, bounded channels, depth-d queues | §2.7 nested `par`, 4 buffer sets, one barrier per block | — | — | — | +3 buffer sets (one-time) | **+1 fork-join barrier per block; E[max] instead of max[E] across stages** |
| 20 | Early abort of a par arm when a sibling fails | shared atomic flag polled per block | not expressible; arms complete their block, driver matches after the join | — | — | — | — | **≤1 block of wasted work, once per failing run** |
| 21 | Double close / leak on a rare exit path | C++: a live bug class (`goto cleanup` mistakes); Rust: prevented | statically refused (gaps G1, G2 permitting) | — | — | — | — | — |

**Grounds for the two quantified rows** (predictions with stated reasoning, not
measurements — `PROGRAMS.md`: "During prose design, cost entries are predictions
with explicit grounds, not runtime measurements"):

- **#19, the barrier.** A four-way fork-join barrier costs two cache-line
  round-trips plus, when the workers are parked, a futex wake per worker: roughly
  0.5–2 µs spinning, 1–5 µs parked, on a current x86 server part. At a 64 KiB block
  with per-stage times in the 20–60 µs range the barrier alone is **2–10%**.
- **#19, the jitter.** A barrier pays E[max(t₁…t₄)] per block; a channel pipeline
  with a queue of depth d pays approximately max(E[tᵢ]) once the queues fill. For
  four stages whose per-block times vary with the data (compression ratio varies
  block to block) at σ/µ ≈ 0.3, E[max of 4] ≈ µ(1 + 1.0σ/µ) ≈ 1.3 µ, i.e. a further
  **~25–30%**. Combined range for #19: **10–40% throughput on a jittery
  multi-stage stream**, and ~2–10% on a perfectly uniform one.
- **#14, conditional on the resource.** For a linear *program* value whose `close`
  is a few instructions, N serialized closes is noise. For an external resource
  whose release blocks, the loss is N × close-latency and becomes the dominant
  term. `PROGRAMS.md` excludes IO from this discussion, so the row is stated both
  ways and the second way is marked conditional.

**Rows where the candidate is cheaper than the baseline:** #6, #7 and #9 against
Rust, and #21 against C++. Row #6 is the substantive one: Rust's answer to §2.4's
conditional ownership is a drop flag, an extra stack slot and an extra branch at
the scope end; x1 reuses the variant tag the program already computes.

---

## 6. The strongest counterexample against the candidate for this task

**A stateful stream decoder over one large resource.**

```text
open → fill (64 KiB blocks) → inflate (32 KiB adaptive history) → crc32 → emit
```

Every stage is sequentially dependent on its predecessor per block, and `inflate`
is sequentially dependent on itself through `hist`, so **no range split of the
input parallelizes anything**. The only available parallelism is stage overlap.
A C++ or Rust implementation runs four threads joined by two bounded queues of
depth 4–8: the queues decouple the stages, so a block whose inflate takes 1.6× the
mean is absorbed by the queue and the pipeline's throughput stays at
1/max(E[tᵢ]). It scales to the slowest stage and no further, which is the point.

Under candidate x1 the queues are unavailable by construction. "Not in this
candidate" lists "channels or atomics"; "Deferred to a future concurrency and
layout round" lists "A channel primitive in the trusted base (ownership-transfer
queue) for producer/consumer pipelines"; and "Known costs already recorded" states
the position plainly: "Lock-free rings are not expressible; batched fork-join with
two buffers is the available form."

The best rewrite is §2.7, and it is a good rewrite: four arms, four buffer sets,
all roots disjoint, Rule 13 accepts it, and the overlap is genuinely four-way. What
it cannot do is absorb jitter. Every block pays a barrier and every block pays the
maximum of four stage times rather than the maximum of four means. The cost is
row #19: **2–10% on a uniform stream, 10–40% on a jittery one**, and it is
unavoidable — there is no second rewrite, because the mechanism that would fix it
is the one the candidate names as deferred.

Two smaller counterexamples, both real:

- **The fan-in.** K workers decoding K independent resources into one output. Rule
  13 refuses `writes(sink)` in two arms and there is no mutex. The rewrite buffers
  each worker's whole output and concatenates after the join: one extra full copy
  of the output and roughly 2× peak output memory (row #16). The escape — disjoint
  output ranges, row #17 — needs the sizes in advance, which a decoder does not
  have without a full extra pass.
- **The N-way drain.** A server holding N linear resources cannot release them in
  parallel: `pop` writes the whole block, two arms conflict, and a per-range drain
  runs into gap G6. The K-way rewrite costs 2N moves and K allocations (row #15).

And the counterexample the candidate **survives**, which is the task's headline
requirement: no arrangement of the eight exits in §2.3, the three exits in §2.4, or
the eight failure arms across both, loses a resource or closes one twice, and none
of it uses a flag, a sentinel or a tag. Rule 14's "Allocation returns a `Result`
and never traps" makes the set of exit paths finite and writer-visible; Rule 8's
"on every exit path" is then a per-path enumeration a reader can perform by hand;
and the typed outcome the task required anyway is the same runtime word that
carries the ownership evidence. `PROGRAMS.md` P8's requirement — "no runtime drop
flag; the writer's own branches carry the state" — and P18's — every failure arm
"must free nothing twice and leave earlier blocks owned" — are both met without a
byte of metadata, and row #6 shows the candidate beating Rust on exactly the shape
the task is built around.

---

## 7. Rules consistent, and task achievable — recorded separately

**Rules consistent for this task: yes for §2.3–§2.7 as written, with the seven
gaps in §8, and with one textual conflict.** Nothing in §2 is admitted by one rule
and refused by another. The conflict is gap **G3**: Rule 8's own worked example
`fn use_it(c: own Conn) { ...; close(move c.f) }` performs a partial move out of
the place `c.f`, while Rule 6 states "There is no `take` operation and no partial
move out of any place" and Rule 12 restates it. This task meets the same question
in the form `match probe(…) { Back(g, k) => … }`, which must move `own File` out of
the matched value. The two sentences cannot both be read literally. That is a
consistency defect in the text, not a contradiction between two admitted
derivations — and it is the same defect the container task recorded as its G5,
reached from a different direction, which strengthens the case that it is one
missing operation rather than two local oversights.

**Task achievable: yes, with cost, and the cost is entirely in the parallel
extension.** Taken literally — one linear resource, several fallible stages, a
loop with early exits, closed exactly once on every path, and two independent
resources in `par` — the task is achievable at **zero** runtime cost, and at
negative cost against Rust on the conditional-ownership shape (rows #6, #7, #9).
Taken as the engineering capability the row in `PROGRAMS.md` describes, with
"independent work remains eligible to overlap" read to include N resources and
overlapped stages, the task carries real cost: rows #14/#15, #16/#18 and #19.

**Per-operation verdicts:**

| Operation | Verdict |
|---|---|
| Linear resource type, stage signatures, effect rows, contracts | `accepted-fine` |
| Five-stage loop with eight exit paths, close exactly once (§2.3) | `accepted-fine` |
| `?` refused while a linear value is live; `match`+`close` rewrite | `rejected-zero-cost` |
| Typed outcome carrying the resource back (`Take::Back`) | `undecided-rule-gap` (G2, G3) |
| Stage that closes the resource itself and says so in its variant | `accepted-fine` (given G2, G3) |
| Loop-carried resource moved out and returned inside the body (§2.4) | `accepted-fine`; `undecided-rule-gap` (G4) for the case where the automatic presence analysis needs help |
| Exactly-once as an upper bound on `close` | `undecided-rule-gap` (G1) |
| Reference validity around `close`; aliased stage parameters | `accepted-fine` (the aliased call is `rejected-zero-cost`: in-place work is a one-reference signature) |
| Bounds inside the stage loops | `accepted-fine` |
| Two independent resources in `par`, owning or reference form | `accepted-fine` |
| N resources, parallel processing via per-element or range references | `accepted-fine` |
| N resources, parallel close | `rejected-real-cost` (+2N moves, +K allocations) |
| Window operations on a range reference | `undecided-rule-gap` (G6) |
| Discharging a `DynBox<File>` via `truncate` | `undecided-rule-gap` (G7) |
| Fan-in to one sink, sizes unknown | `rejected-real-cost` (+1 full copy, 2× peak memory, streaming lost) |
| Fan-in to disjoint output ranges, sizes known | `accepted-fine` |
| Stage-to-stage overlap over one stateful resource | `rejected-real-cost` (barrier per block; 10–40% on jittery stages) |
| Early abort of a par arm | `rejected-real-cost` (≤1 block wasted per failing run) |
| `?` inside a `par` arm with a live linear value | `undecided-rule-gap` (G5) |
| Non-tabled reduction (crc32) split across arms | `rejected-zero-cost` (one arm of four is not the critical path here) |

**Headline verdict: `accepted-fine` for the task as stated, `rejected-real-cost`
for its parallel generalization.** The sequential core — the part the task actually
names — is the best result recorded for this candidate so far: it is free, it beats
Rust on conditional ownership, and it meets P8 and P18 literally. The generalization
to N resources and to overlapped stages is where the missing channel and the
missing parallel-drain form become visible, and the largest single number in this
task is the 10–40% of §6.

---

## 8. Rule gaps

Each gap quotes the exact sentence and names what the text does not decide. Per the
protocol, none is resolved here.

**G1 — is a linear value consumed *at most* once?** Rule 8:

> "Affine values are consumed at most once; at scope exit the compiler releases their memory recursively (Box, DynBox) and runs no user code. Linear values must be consumed by an explicit operation on every exit path; the compiler never releases them."

Missing: the upper bound for linear values. "At most once" is stated for affine
only, and Rule 8's opening sentence "A type is copy, affine, or linear" makes the
classes disjoint, so the affine sentence does not reach `File`. The task's phrase
is "close the resource exactly once on every path"; the at-least-once half is
explicit and the at-most-once half is not. Rule 12's "every path is either wholly
present or the program cannot name it" together with Rule 6's "consuming a whole
local (`move x`)" supplies it indirectly, and this derivation uses that reading.
No runtime cost rides on the answer; the whole safety claim does.

**G2 — does linearity propagate through an enum or a tuple?** Rule 8:

> "Linearity is declared on external-resource types and propagates through aggregates: a struct, array, Box, or DynBox containing a linear part is linear."

Missing: enum and tuple, both of which Rule 1 lists as aggregates ("Every struct,
enum, tuple, array, slice-like value, Box, DynBox, and generic instantiation holds
only owned values"). Every typed outcome in this task that hands the resource back
is an enum with an `own File` payload. If linearity does not propagate through it,
unsafe variant U4 — calling `probe(move f, m)` and discarding the result — loses
the resource silently, with no diagnostic and no memory error, on exactly the
failure paths the task exists to check. The generic phrase "propagates through
aggregates" points at propagation; the enumerated list does not say it.

**G3 — how does a linear payload leave a matched value?** Rule 6:

> "There is no `take` operation and no partial move out of any place: the only ways to move a value out of storage are consuming a whole local (`move x`), the window operations, and the atomic update."

and Rule 12: "No place is ever partially moved: every path is either wholly present
or the program cannot name it." Against Rule 8's own example:

> `fn use_it(c: own Conn) { ...; close(move c.f) }   // consuming the struct as a whole and closing its File is required`

Missing: a whole-value destructuring form. `match probe(…) { Back(g, k) => … }`
must move `own File` out of the matched enum value, which is neither a whole local,
a window operation, nor an atomic update. Rule 8's comment "consuming the struct as
a whole" gestures at the operation that would make both sentences true at once, but
no rule provides it. Consequence for this task: Form T — and with it the entire
"stage decides whether it kept the resource" capability the task names — is not
derivable from the text as written. Form R remains available at zero cost for
everything except that capability.

**G4 — can the writer state that a linear local is present?** Rule 11:

> "Facts are the existing WF forms: affine comparisons over measures and integer values, refinement facts from a dominating branch, loop-header invariants `invariant name: affine_expr compare_op affine_expr`, explicit `use` steps inside an `invariant`, and callee contracts."

Missing: any form that says "the linear local `f` is present at this program
point". Presence is not a comparison between affine expressions, so `invariant`
cannot carry it across a loop back edge and no `use` step can establish it. §2.4's
loop is simple enough for an automatic flow-sensitive analysis, which Rule 12's
presence sentence implicitly assumes; the gap is that when a harder shape defeats
that analysis there is no written repair, and the rule set nowhere states what the
analysis is.

**G5 — what does `?` in a `par` arm do to the sibling arm and to live linear
values?** Rule 14:

> `par { a = build(&x)?; b = build(&y)? }     // both allocate: accepted`

Missing: the meaning of the early return. If arm A's `Err` edge returns from the
enclosing function, the rule text does not say whether arm B is awaited, abandoned
or forbidden, nor whether Rule 8's "on every exit path" obligation is discharged
for a linear value live in the enclosing frame on that edge. §2.7 avoids the
question by returning outcome values from every arm and matching after the join,
which costs one block of sibling work per failing run (row #20) and nothing
otherwise.

**G6 — do the window operations apply to a range reference?** Rule 6 types them on
the whole block:

> `fn pop<T>(buf: &DynBox<T>) -> own T           writes(buf)`

Rule 7 makes a range reference indexable:

> "Indexable things: an inline `array<T, N>` (constant length, always fully initialized), a `DynBox<T>` (Rule 6), a range reference into either, and a `const` table."

and Rule 13 passes one as an argument: `kernel(&v[0..mid], &out[0..mid])`. Missing:
the type of a range reference, and therefore whether `pop(&files[lo..hi])` or
`truncate(&files[lo..hi], 0)` typechecks. It matters because Rule 6 also says
`len_of` is "a runtime number stored in the block header" — one number for the
whole block — so under the permissive reading two arms popping from disjoint ranges
would both update the same header while Rule 13 judged them disjoint. This
derivation uses the tight reading (window operations need the whole block), under
which parallel drain is refused and row #15 prices the rewrite.

**G7 — what does `truncate` do with linear values in the slots it removes?**
Rule 6:

> "A DynBox whose element type is linear is itself linear (Rule 8) and must be truncated to zero by the program before it can go out of scope."

with the operation's own contract stating only the measure:

> `fn truncate<T>(buf: &DynBox<T>, n: u64)       writes(buf)`
> `    contract { requires n <= len_of(buf);         ensures len_of(buf) == n; }`

Missing: the fate of the values in slots `[n, len_of)` when `T` is linear.
Releasing them contradicts Rule 8's "the compiler never releases them"; not
releasing them loses them, which is the leak the linearity exists to prevent.
Rule 6's neighbouring sentence for scope exit — "the compiler releases slots
`[0, len_of)` recursively" — shows the release behavior for the affine case and is
silent here. §2.6(c) sidesteps it with a `pop`-and-`close` loop, after which the
truncate-to-zero requirement is already satisfied; the sidestep costs nothing, but
the prescribed route is undecided.

---

## 9. Dependence on Rule 16

**Dependent: yes, and confined to §2.6 — the N-resource table. The task as stated,
and everything in §2.3, §2.4, §2.5 and §2.7, derives under Rules 1–15 alone.**

The pipeline itself names its resources with locals (`f`, `sink`, `a`, `b`, `sa`,
`sb`). Locals have identity by construction, Rule 3 decides reference validity
against them, and Rule 8's obligation attaches to the local. No handle, index or
generation number appears anywhere in the sequential program or in the two-resource
`par`, so Rule 16 is not consulted.

Rule 16 enters once the resources live in a `DynBox<File>` and workers address them
by index:

1. **§2.6(a) and (b) are the Rule 16 pattern.** "A pool is a DynBox plus indices
   used as handles". Worker arms name `files[i]` and `outs[i]`, and Rule 13's
   per-element and range families judge disjointness on those indices. Rule 4 is
   what forces indices in the first place — "A function that 'finds' something
   returns an index or other owned data; the caller forms the reference" — so any
   scheduler that selects resources by search hands out indices, not references.
2. **Identity after a removal is Rule 16's ruling, and this task can hit it.** If
   the driver removes a finished resource with `swap_remove(&files, k)`, every
   index above `k` renumbers. Rule 16 is the only place in the rule set that states
   a disposition: "A stale index that is still in bounds names the current occupant
   of that slot: a logic error, not a memory error. Programs that need to detect it
   keep a generation number as data." For a *linear* resource the consequence is
   worth stating plainly: a stale index cannot cause a double close, because the
   close obligation lives with the container and is discharged by `pop`, not by the
   index — but it can cause the pipeline to process one resource twice and another
   not at all, which Rule 16 classifies as a logic error. Without Rule 16 the
   candidate has no stated position on handle identity in a mutated resource table,
   and §2.6 would be underivable rather than merely costly.
3. **Row #15's K-way split is a Rule 16 rewrite**, since each worker ends up owning
   a pool of its own and addressing it by index.

The generation-number remedy Rule 16 names costs one word per slot plus one load
and one compare per dereference, and it is not needed by any program in §2 — none
of them removes an element while a worker holds an index for it.
