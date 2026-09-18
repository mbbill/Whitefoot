# Candidate x1 revision 4 — engineering task: recursive tree rewrite over Box-linked nodes

Derived against the frozen rule set in `CANDIDATE-X1.md` (revision 4, 2026-09-18) and
only that file. This file replaces round one's `matrix-x1/task-tree-rewrite.md`, which
was derived against revision 2. Section 0 says exactly what changed and why; the rest of
the file is self-contained and does not require reading the round-one file.

The task: a recursive rewrite over Box-linked nodes with two inline
`Option<Box<Node>>` children — traverse by reference through variant payload paths,
replace a subtree with the atomic update, hand a subtree to another function by
ownership, clean up the removed parts, with an early-exit error path; plus the iterative
find-then-mutate form and its recursive or index-based equivalent.

Nothing under "Not in this candidate" is used: no `with` block, no `&uniq`/`&mut`
marker, no region, store brand or lifetime, no `take`/`put` hole, no partial move that
leaves an owner alive, no quantified fact over elements, no channel, no destructor, no
trap, no returned reference, no header-plus-tail block, and no `par` statement — every
overlap claim below is written as two adjacent statements with an explicit "may overlap"
or "may not overlap" judgment.

Cost convention from the protocol: runtime performance is the only cost. Verbosity,
extra parameters, duplicated code and proof annotations are not costs. "One move of `T`"
is a relocation of `sizeof(T)` bytes, which Rule 1's consequence — "any value can be
relocated by copying its bytes (memmove, realloc), because nothing inside it points
anywhere" — makes a plain byte copy with no fixup.

---

## 0. What changed from round one, and why

Round one derived this task against revision 2. Six revision-4 changes hit it directly;
four of them delete round-one findings, one deletes a round-one *program*, and one is new
cost.

**0.1 The node shape the task names is now writable. Round one's headline gap G1 is
closed, and with it the largest cost in round one's table.** Revision 2's path grammar
had no step into an enum payload, so `Option<Box<Node>>` children could not be traversed
by reference and round one was forced onto a second shape — a child *window*
(`kids: Box<Slots<Box<Node>>>`) — which costs one extra dependent load per hop and one
extra allocation per node against the C++ baseline. Revision 4's Rule 2 adds the step:

> "A path starts at a local variable or a parameter and continues through fields, `*`
> (Box content), `[i]` (index, Rule 7), `[lo..hi]` (range, Rule 7), or the payload of an
> enum variant. A payload step is available only under the refinement fact that the enum
> currently holds that variant, which a `match` or `if let` on the enum establishes in
> the selected arm and which any write to the enum invalidates."

The fixed-arity node of §2 is therefore used throughout, and round one's `+1 dependent
load per hop, +1 allocation per node` vs C++ is gone. Round one's derivation of the
window shape is not reproduced; it is no longer the task's shape.

**0.2 The atomic update is now defined, and round one's strongest counterexample is
mostly defused.** Revision 2 exhibited one unary, index-only example, and round one's C1
was that the one zero-cost in-place rewrite primitive could not take arguments, could not
fail, and could not name a destination for the removed subtree — so every interesting
(allocating) rewrite fell back on a four-operation window dance. Revision 4's Rule 6:

```text
place = f(place, args...)  writes(place)            // atomic in-place update: the old value enters f by value,
                                                    // f's result is committed, no program point lies between
node.left = insert(node.left, k)                    // f is total and returns the place's type; f's row must not
c.f = reopen(c.f)                                   // overlap any prefix of place; failure is an enum in the place
```

`node.left = insert(node.left, k)` is this task's operation verbatim: any owned place,
extra arguments, and an effect row on `f`. What survives as a cost is much smaller and is
this round's strongest counterexample (§9, C2): the row restriction "f's row must not
overlap any prefix of place" refuses an `f` that consults the *sibling* child while
rewriting this one.

**0.3 Round one's four-operation `exchange` is deleted.** Revision 2 had only
`pop`/`swap_remove` for getting a value out of storage, so an order-preserving extraction
cost four window operations (round one's §3.1) and a fallible rewrite cost four more on
the error path. Revision 4 gives `swap` on any two places and `replace(&r[k], x) -> own T`
on window slots, so extraction is one operation. Round one's `+3 moves of 8B, +4 header
stores per extraction` and `+4 window ops on the error path` are both gone. With the
fixed-arity shape there is no window in the tree at all, and the extraction is
`swap(&n.left, &hole)` — exactly Rust's `mem::replace`.

**0.4 A round-one program is now rejected. This is the round's new cost.** Round one's
split find-then-mutate walked the tree with an iterative cursor, `n = &(*n.kids[j])` in
a loop, and argued it was admitted by Rule 2's "A reference may be rebound". Revision 4
adds:

> "A path has a static shape: a loop-carried rebinding may change only the index values
> inside the path, never extend the path through itself."
>
> ```text
> loop { p = &(*p.kids)[0] }           // rejected: the path would grow without bound; use recursion or indices
> ```

So the iterative by-reference descent — the form the task explicitly asks for — is
refused. §6 gives the two admitted equivalents and §8 prices them; §9 C1 is built on it.

**0.5 Partial moves are decided, which makes the by-value (ML-style) rewrite writable
and shows it is the expensive form.** Round one's G3 was an outright contradiction
(Rule 8's `close(move c.f)` against Rules 6 and 12's no-partial-move sentences).
Revision 4's Rule 6: "A move out of a field or out of Box content consumes the whole
owner: the owner ceases to exist, its other affine parts are released", with
`let Conn { f, g, .. } = move c`. §5 uses this and finds it costs one allocation and one
free per rewritten node, so the in-place atomic update of §3 stays the primary form.

**0.6 Rule 16 is confirmed, and round one's G5 stops biting this task.** The index-based
equivalent of §6.3 is no longer conditional on a pending ruling. Round one's G5 (index
expressions inside a contract) was load-bearing only because round one's nodes carried a
nested *window* of children whose length a callee wanted to state; the fixed-arity node
has no nested measure, so the precondition round one could not write is not needed. G5
survives in a narrowed form (§11, G-C) and costs one reloaded generation check per call
under its strict reading.

**0.7 Spelling.** Measures are read-only pseudo-fields throughout: `(*log).len`,
`(*log).room`, `entry(*log).len`. `len_of` does not appear. Window parts (`r.next`,
`r.last`, `r.filled`, `r.free`) appear in the rows of the log and worklist helpers.

Unchanged from round one: Rule 4 still forbids returning the found reference, so a *split*
find-then-mutate still re-descends (§8); Rule 8's compiler-emitted release still has no
stated depth bound (§11, G-D), and §0.4 makes that worse by forcing the program's own
descent into recursion too.

---

## 1. The question this task tests

Can a program that descends an owned binary tree through `Option<Box<Node>>` edges,
replaces a subtree at a data-determined position with a freshly allocated one, hands the
removed subtree to another function by ownership, disposes of it, and exits early on
error, be written under revision 4 at the cost of an idiomatic C++/Rust tree rewrite — in
the fused recursive form, in the split find-then-mutate form, and with the iterative
cursor the task names?

---

## 2. Types

```text
enum Op  { Lit, Var, Add, Mul }
enum Err { Oom, NotFound, TooDeep }              // no payloads: Copy

struct Node {
    op:    Op,
    lit:   Int,
    left:  Option<Box<Node>>,                     // Rule 1: owned payload, accepted
    right: Option<Box<Node>>,
}

struct Binding { name: Int, lit: Int }
struct Env     { defs: Box<Slots<Binding>> }      // the inlining source, a separate root
struct Report  { fails: u64, code: Err, hits: u64 }
```

Rule 1 accepts `Node`: "Every struct, enum, tuple, `Array`, `Slots`, `Ring`, `Box`, and
generic instantiation holds only owned values." `Option<Box<Node>>` is an owned payload;
only `Option<&T>` is "rejected, whatever T is". `Node` contains no linear part, so by
Rule 8 it is affine: "at scope exit the compiler releases their memory recursively (Box,
Slots, Ring, Array) and runs no user code."

The rewrite log is a window on the heap, `log: &Box<Slots<Int>>`, so the storage is
`*log` and its measures are `(*log).len`, `(*log).room` (Rule 6: "`r` stands for the
storage, e.g. `*b`"; "`r.room` (equal to `r.cap - r.len`)").

`Report` carries the two outputs that cannot ride in the atomic update's result, because
Rule 6 fixes that result: "f is total and returns the place's type". `fails`/`code` is
the error channel and `hits` is the "did a rewrite happen" channel. §8 prices this; §9
C3 examines whether the in-place alternative ("failure is an enum in the place") is
cheaper, and finds it is not for this task.

---

## 3. The strongest program: fused recursive rewrite by reference

The strongest case is the one where static analysis provably cannot help:

- the rewrite target is found by a data-dependent search, so no child selector is a
  literal and no index is a constant;
- the replacement's *shape is read off the removed subtree*, so the fallible allocation
  cannot be hoisted above the point where the subtree is reached;
- the removed subtree is handed to a separate function by ownership across a signature,
  so the checker sees only rows and contracts (Rule 10: "Recursion is checked through
  contracts, never by unfolding bodies");
- the error path can fire *after* the place has been reached, and the tree must be left
  structurally intact;
- the descent is recursive and its depth is data, and the candidate promises no trap.

### 3.1 Lookup and replacement construction

```text
fn lookup(env: &Env, name: Int) -> Option<u64>
    reads(*env.defs)
    contract { ensures when Some: result < (*env.defs).len; }

fn build_replacement(old: &Node, env: &Env) -> Result<Box<Node>, Err>
    reads(old), reads(*env.defs)
    contract { ensures when Err: true; }
{
    match lookup(env, old.lit) {
        Some(i) => {
            d = &(*env.defs)[i]                              // i < (*env.defs).len from the ensures
            nd = Node { op: Lit, lit: d.lit, left: None, right: None }
            match Box::new(nd) {
                Ok(b)          => Ok(move b),
                Err((_, back)) => Err(Oom)                   // `back` is the Node, affine, released here
            }
        }
        None => Err(NotFound)
    }
}
```

Rule 4 is obeyed exactly: "A function that 'finds' something returns an index or other
owned data; the caller forms the reference", and `lookup`'s signature is Rule 4's own
shape. `d = &(*env.defs)[i]` needs `i < (*env.defs).len`, discharged by the `ensures when
Some` clause (Rule 11: "`ensures when Variant:` for result-routed relations"), so Rule 7's
runtime alternative is not used here: no compare, no branch.

Rule 5's fallible allocation is the revision-3/4 form — "A fallible allocation that takes
a by-value payload hands it back on failure" — so the `Err` arm names the Node again and
its affine release is the compiler's (Rule 8). Nothing is lost and nothing is written by
the writer.

### 3.2 Cleanup: the removed subtree leaves by ownership

```text
fn retire(sub: own Option<Box<Node>>, log: &Box<Slots<Int>>)
    writes((*log).next), writes((*log).len)
    contract {
        requires (*log).room > 0;
        ensures  (*log).len == entry(*log).len + 1;
    }
{
    match &sub {
        Some(b) => { place_back(&(*log), (*b).lit) }
        None    => { place_back(&(*log), 0) }
    }
    // `sub` is affine and unconsumed: Rule 8 releases the whole subtree recursively
    // at scope exit and runs no user code. No destructor exists and none is needed.
}
```

`place_back(&r, x)` has the row `writes(r.next), writes(r.len)` and the contract
`requires r.room > 0; ensures r.len == entry(r).len + 1` (Rule 6), instantiated at
`r := *log`. `match &sub` takes the payload step of Rule 2 on a by-value local: `b` names
`sub.Some.0` under the fact that `sub` is `Some`, and `place_back` writes a different
root, so nothing invalidates `b` (Rule 3: invalidation requires a write to a proper
prefix of `b`'s path).

### 3.3 The atomic update: the `f` of `place = f(place, args...)`

```text
fn rewrite_child(child: own Option<Box<Node>>, env: &Env,
                 log: &Box<Slots<Int>>, rep: &Report) -> own Option<Box<Node>>
    reads(*env.defs),
    writes((*log).next), writes((*log).len),
    writes(rep.fails), writes(rep.code), writes(rep.hits)
    contract {
        requires (*log).room > 0;
        requires rep.fails == 0;
        ensures  rep.hits <= entry(rep).hits + 1;
        ensures  (*log).len == entry(*log).len + (rep.hits - entry(rep).hits);
    }
{
    match &child {
        Some(b) => {
            if (*b).op == Var {
                match build_replacement(&(*b), env) {
                    Ok(fresh) => {
                        retire(move child, log)              // ownership leaves this frame
                        rep.hits = rep.hits + 1
                        return Some(move fresh)              // committed into the place
                    }
                    Err(e) => {
                        rep.fails = 1
                        rep.code  = e
                        return move child                    // the original occupant goes back
                    }
                }
            }
        }
        None => {}
    }
    move child
}
```

`f` is total (every path returns an `Option<Box<Node>>`), it returns the place's type,
and its row touches only `*env.defs`, `*log` and `rep` — three roots that are not the
tree. That is what makes it legal as the `f` of an atomic update on `n.left`; see the
trace in §4(c).

### 3.4 The recursive rewrite, with the early-exit error path

```text
fn rewrite_at(n: &Node, env: &Env, log: &Box<Slots<Int>>, rep: &Report, budget: u64) -> u64
    reads(*env.defs),
    writes(n.left), writes(n.right),
    writes((*log).next), writes((*log).len),
    writes(rep.fails), writes(rep.code), writes(rep.hits)
    contract {
        requires (*log).room > 0;
        requires rep.fails == 0;
        ensures  result <= 1;
        ensures  (*log).len == entry(*log).len + result;
        ensures  rep.hits == entry(rep).hits + result;
    }
{
    if budget == 0 { rep.fails = 1; rep.code = TooDeep; return 0 }

    h0 = rep.hits
    n.left = rewrite_child(n.left, env, log, rep)            // atomic update: writes(n.left)
    if rep.fails > 0 { return 0 }
    if rep.hits > h0 { return 1 }

    n.right = rewrite_child(n.right, env, log, rep)          // atomic update: writes(n.right)
    if rep.fails > 0 { return 0 }
    if rep.hits > h0 { return 1 }

    match &n.left {
        Some(b) => {
            r = rewrite_at(&(*b), env, log, rep, budget - 1)
            if rep.fails > 0 { return 0 }
            if r == 1 { return 1 }
        }
        None => {}
    }
    match &n.right {
        Some(b) => {
            r2 = rewrite_at(&(*b), env, log, rep, budget - 1)
            if rep.fails > 0 { return 0 }
            if r2 == 1 { return 1 }
        }
        None => {}
    }
    0
}
```

The caller:

```text
rep = Report { fails: 0, code: NotFound, hits: 0 }
k   = rewrite_at(&(*root), &env, &log, &rep, 64)
if rep.fails > 0 { return Err(rep.code) }
```

---

## 4. Rule-by-rule trace at the interesting points

**(a) The payload step, and why the traversal is now a traversal.** `match &n.left {
Some(b) => ... }` is Rule 2's own example transposed:

> ```text
> match &n.left {
>     Some(child) => {   // child names the payload path n.left.Some.0 under the fact "n.left is Some"
>         n.left = None  // the fact is gone: child is invalid from here
>     }
>     None => {}
> }
> ```

`b` names `n.left.Some.0`, a `Box<Node>`; `&(*b)` extends it through `*` ("Box content")
to the heap `Node`. Rule 2's "A reference may be rebound … r names (*v)[i].x; a reference
extends a path, it does not point at p" is the sentence that makes `&(*b)` a path in the
caller's storage rather than a pointer to a local. At run time this is one load of the
child pointer and one discriminant test — and the discriminant test is the null-niche
test (Rule 12: "non-Copy payloads: Option, using a null niche where the type has one"),
identical to Rust's `if let Some(b) = &n.left` and to C++'s `if (n->left)`.

**(b) The recursive call does not invalidate the payload fact.** Inside `Some(b)`, the
call `rewrite_at(&(*b), ...)` substitutes to `writes(n.left.Some.0.*.left)` and
`writes(n.left.Some.0.*.right)`. Rule 2 says the payload fact dies on "any write to the
enum" — the enum is `n.left`, and what was written is strictly below `n.left.Some.0`.
Rule 3 states the same containment from the reference side: "Writing the storage at p's
path or below it (a content write) does not invalidate p." So `b` survives the call. This
is load-bearing in §7, where both children's references must be live at once.

**(c) The atomic update, clause by clause.** The statement is
`n.left = rewrite_child(n.left, env, log, rep)`.

- *Shape.* Rule 6: "`place = f(place, args...)`  `writes(place)`", exhibited as
  `node.left = insert(node.left, k)` — a struct field reached through a reference
  parameter, with an extra argument. That is exactly this statement.
- *No hole.* "the old value enters f by value, f's result is committed, no program point
  lies between". This is why the update does not collide with Rule 6's "A move out of a
  field or out of Box content consumes the whole owner" or with Rule 12's "No place is
  ever partially moved: every path is either wholly present or the program cannot name
  it": there is no program point at which `n.left` is absent, so `n` is never partial and
  is never consumed. Writing `old = move n.left` instead would demand the opposite and is
  refused; see §5.
- *Totality and result type.* "f is total and returns the place's type". `rewrite_child`
  returns `own Option<Box<Node>>` on every path, including the failure path, where it
  returns the value it received.
- *Row restriction.* "f's row must not overlap any prefix of place". `place` is `n.left`;
  its prefixes are `n.left` and `n`. `rewrite_child`'s substituted row names `*env.defs`,
  `*log`, `rep.fails`, `rep.code`, `rep.hits` — three roots distinct from `n`, so
  Rule 10.1's obligation is discharged by "different roots". §9 C2 is what happens when a
  rewrite needs to read `n.right` here.
- *Effect coverage.* Rule 9: "A function body is checked against its own row: every
  statement's effect and every callee's substituted row must be covered by the declared
  row." The statement's effect is `writes(n.left)`, declared. `rewrite_child`'s row is
  declared too, term by term.
- *Fact invalidation.* The update writes `n.left`, so any payload fact on `n.left` held
  before it is gone (Rule 2). The program forms `&n.left`'s payload only afterwards, in
  the `match` of the descent.

**(d) The ownership handoff, and why no region is needed.** Inside `rewrite_child`,
`retire(move child, log)` is Rule 10.2 — "A by-value argument contributes a consumption
(`move`) or a read (copy) of its place to this comparison" — comparing a consumption of
the local place `child` against `writes((*log).next)`, `writes((*log).len)`. Different
roots, so disjoint. The subtree being transferred is a *whole local* at that moment,
because the atomic update delivered it by value; Rule 6's "consuming a whole local
(`move x`)" is the admitted form. No brand, region or lifetime appears, and the transfer
is one 8-byte `Option<Box<Node>>` move.

The reference `b` formed at `match &child` dies at this move: Rule 3, "a proper prefix of
p's path is … moved out of", with `child` a proper prefix of `child.Some.0`. The program
does not use `b` afterwards; it uses `fresh`, a fresh local. Rule 3's own example is the
same situation: "`p = &*b`; `c = move b`  // b is a proper prefix of *b: p invalid, even
though the Node did not move".

**(e) The handoff that is refused, and it should be.** Handing the tree away while
holding a reference into it is Rule 10's third example transposed:

```text
p = &(*root).left
retire(move root, log)                 // rejected: move root writes the prefix root of p's path
```

Rejected, matching "`put(slot, move b)`  // rejected: move b writes the prefix b of
slot's path (Rule 3)". A C++ program that does this is a use-after-free. No rewrite is
owed.

**(f) The log contract across recursion, and why the inequality had to become an
equality.** `rewrite_child`'s `ensures` is
`(*log).len == entry(*log).len + (rep.hits - entry(rep).hits)`, an affine comparison over
a measure and integer values, which is Rule 11's first admitted fact form. A weaker
`ensures (*log).len <= entry(*log).len + 1` would not do: after the first
`rewrite_child`, the caller must re-establish `(*log).room > 0` for the second call and
for the recursion, and from an inequality it cannot. With the equality, the dominating
test `if rep.hits > h0 { return 1 }` gives `rep.hits == h0` on the fall-through edge
(Rule 11: "refinement facts from a dominating branch"), hence `(*log).len ==
entry(*log).len`, hence `room` is what it was, hence `requires (*log).room > 0` transfers
to the callee. Rule 11's "Across a call, a fact known before the call about a measure of
an argument survives as a fact about `entry(p)` of that argument, which is how an
`ensures` of the shape `r.len == entry(r).len + 1` connects to what the caller knew" is
the sentence that makes this chain legal.

`rewrite_at`'s own `ensures (*log).len == entry(*log).len + result` is checked at each of
its five exits: `TooDeep` and both `fails` exits return 0 with no `place_back` executed;
the two `return 1` exits have exactly one, either in this frame (via `rewrite_child`) or
in the callee (via its identical `ensures`). No quantified fact is used anywhere, as
Rule 11 demands: "There are no quantified facts over array elements".

**(g) Recursion is checked through contracts.** Rule 10: "Recursion is checked through
contracts, never by unfolding bodies." `rewrite_at` calls itself and sees only its own
`requires`/`ensures`; the `budget` parameter is ordinary data and `budget - 1` is an
ordinary affine expression. Nothing in the rule set requires the budget — it is the
program's own bound, and §9 C1 explains why the no-trap promise makes it mandatory in
practice and what it costs.

**(h) The early exit.** Two exits fire after work has been done. Memory safety on both is
free and needs no flag: `Node` is affine, and Rule 8 says "Affine values are consumed at
most once; at scope exit the compiler releases their memory recursively … and runs no
user code." *Structural* integrity is free too, and this is a genuine improvement over
the revision-2 derivation: because the atomic update never exposes a hole, the failure
path of `rewrite_child` returns the original child into the place with one store, and no
restoration sequence exists at all. Safe Rust's `n.left.take()` form must store `None`
first and store the original back on failure; the atomic update stores once, on either
path.

**(i) Allocation.** `Box::new(nd)` is Rule 14's form — "Allocation returns a `Result` and
never traps" — and Rule 5's payload-returning shape. One predictable branch per
allocation.

**(j) Cleanup.** `retire` consumes `sub` and never passes it on, so Rule 8 releases the
subtree recursively. Zero instructions written by the program. The depth of that release
is data and unbounded; see §11 G-D and §9 C4, and §6.4 for the allocation-free flattening
that revision 4's general `swap` makes possible.

---

## 5. The moves that are refused, and the two zero-cost replacements

The form a writer reaches for first is a plain extraction through the reference
parameter:

```text
old = move n.left                      // n: &Node
```

Rule 6 decides it: "A move out of a field or out of Box content consumes the whole owner:
the owner ceases to exist, its other affine parts are released". The owner here is the
`Node` the caller owns, reached through a reference parameter, and Rule 12 forbids the
result — "No place is ever partially moved: every path is either wholly present or the
program cannot name it". Two exhibited forms replace it, both at the cost of Rust's
`mem::replace`:

```text
// (1) the atomic update: one store, the place is never incomplete
n.left = rewrite_child(n.left, env, log, rep)

// (2) swap against a local hole, when the removed value must be named in this frame
hole = None
swap(&n.left, &hole)                   // Rule 6: "swap(p: &T, q: &T)  writes(p), writes(q)"; distinct roots
retire(move hole, log)                 // `hole` is a whole local: Rule 6's admitted move
```

Form (2) is nominally two loads and two stores where Rust's `mem::replace` is one load
and one store, but `hole` is a fresh local with the constant value `None`, so the load of
`hole` and the store into it fold away in any backend that propagates the constant. The
worst case that survives folding is +1 store. Form (1) has no such residue and is what
§3 uses.

The by-value (ML-style) rewrite is now writable too, because revision 4 decided the
partial-move question:

```text
fn rewrite_owned(b: own Box<Node>, env: &Env) -> Result<Box<Node>, Err>
{
    nd = move *b                                        // Rule 6: "unbox: c is consumed … the cell is freed"
    let Node { op, lit, left, right } = move nd         // Rule 6: "several fields at once; `..` covers the rest"
    l2 = match left  { Some(lb) => Some(rewrite_owned(move lb, env)?), None => None }
    r2 = match right { Some(rb) => Some(rewrite_owned(move rb, env)?), None => None }
    match Box::new(Node { op, lit, left: l2, right: r2 }) {
        Ok(nb)         => Ok(move nb),
        Err((_, back)) => Err(Oom)
    }
}
```

It is accepted and it is the expensive form: `move *b` frees the cell and the rebuild
allocates a new one, so every visited node costs one free and one allocation that the
in-place form does not. That is not a cost the *candidate* imposes — a Rust or ML program
written this way pays it too — but it is the reason §3's in-place form is the answer to
this task and this one is not.

---

## 6. Find-then-mutate: the iterative form is refused, and its two equivalents

### 6.1 The iterative cursor the task names — refused

```text
// REFUSED
cur = &(*root)
k   = 0
while k < d {
    match &cur.left {
        Some(b) => { cur = &(*b) }                 // rejected
        None    => { break }
    }
    k = k + 1
}
```

Rule 2:

> "A path has a static shape: a loop-carried rebinding may change only the index values
> inside the path, never extend the path through itself."
>
> ```text
> loop { p = &(*p.kids)[0] }           // rejected: the path would grow without bound; use recursion or indices
> ```

`cur = &(*b)` where `b` names `cur.left.Some.0` extends `cur`'s own path through itself,
by two steps per iteration. The rejection is unambiguous and it is the same rejection the
exhibited example carries. Note that even without this sentence Rule 2's join rule —
"At a control-flow join, a reference variable's target is the set of paths it may name" —
would leave `cur` with an infinite target set at the loop header, so the clause is
consistent with the rest of the rule, not an extra restriction bolted on.

This is the one place in the task where revision 4 is *stricter* than revision 2, and it
lands on the exact form C, C++ and Rust all use for tree descent
(`while (n) n = n->left;`). The rule file's own cost record says
"A rebinding of a reference that extends its own path in a loop is refused; recursion or
indices cost the same loads." Same loads, yes. §9 C1 is about what else it costs.

### 6.2 Equivalent one: recursion, both halves

Rule 4 forbids carrying the found position out as a reference — "A function that 'finds'
something returns an index or other owned data; the caller forms the reference" — so the
search returns a path of child selectors, and the mutation replays it. Both halves must
recurse: the search because it must unwind to write the prefix, the mutation because of
§6.1.

```text
struct Path { steps: Array<u64, 64>, d: u64 }       // 0 = left, 1 = right; Array is always fully initialized

fn find_path(n: &Node, key: Int, p: &Path, depth: u64) -> u64
    reads(n), writes(p.steps), writes(p.d)
    contract { requires depth < 64; ensures result <= 1; ensures p.d <= 64; }
{
    match &n.left {
        Some(b) => {
            if (*b).op == Var && (*b).lit == key { p.steps[depth] = 0; p.d = depth + 1; return 1 }
            if depth + 1 < 64 {
                if find_path(&(*b), key, p, depth + 1) == 1 { p.steps[depth] = 0; return 1 }
            }
        }
        None => {}
    }
    match &n.right {
        Some(b) => {
            if (*b).op == Var && (*b).lit == key { p.steps[depth] = 1; p.d = depth + 1; return 1 }
            if depth + 1 < 64 {
                if find_path(&(*b), key, p, depth + 1) == 1 { p.steps[depth] = 1; return 1 }
            }
        }
        None => {}
    }
    0
}

fn mutate_at_path(n: &Node, p: &Path, k: u64, env: &Env,
                  log: &Box<Slots<Int>>, rep: &Report) -> u64
    reads(p.steps), reads(p.d), reads(*env.defs),
    writes(n.left), writes(n.right),
    writes((*log).next), writes((*log).len),
    writes(rep.fails), writes(rep.code), writes(rep.hits)
    contract {
        requires k < p.d; requires p.d <= 64;
        requires (*log).room > 0; requires rep.fails == 0;
        ensures  result <= 1;
        ensures  (*log).len == entry(*log).len + result;
    }
{
    h0 = rep.hits
    if k + 1 == p.d {
        if p.steps[k] == 0 { n.left  = rewrite_child(n.left,  env, log, rep) }
        else               { n.right = rewrite_child(n.right, env, log, rep) }
        if rep.fails > 0 { return 0 }
        if rep.hits > h0 { return 1 }
        return 0
    }
    if p.steps[k] == 0 {
        match &n.left  { Some(b) => { return mutate_at_path(&(*b), p, k + 1, env, log, rep) }  None => { return 0 } }
    }
    match &n.right     { Some(b) => { return mutate_at_path(&(*b), p, k + 1, env, log, rep) }  None => { return 0 } }
}
```

`p.steps[k]` needs `k < 64` (Rule 7: "`a: Array<Int, 8>;  a[i]  // requires i < 8`"),
discharged from `requires k < p.d` and `requires p.d <= 64` by affine chaining, which is
Rule 11's first fact form. No runtime bounds compare is emitted on the path buffer.

The path may be stale if anything changed between the two halves; here nothing does, and
the `None` arms are the structural test that a stale path would hit. That test is one
discriminant check per level, which the descent performs anyway.

### 6.3 Equivalent two: indices into a pool (Rule 16, confirmed)

Rule 16 is confirmed in revision 4, so this form needs no condition. It restores the
iterative loop, because a pool index is an *index inside a static path shape*, which is
exactly what Rule 2 still permits to vary.

```text
const NONE: u64 = 0xFFFF_FFFF_FFFF_FFFF

struct PNode { op: Op, lit: Int, gen: u64, l: u64, r: u64 }
struct Pool  { nodes: Box<Slots<PNode>>, free: Box<Slots<u64>> }

fn pool_find(p: &Pool, root: u64, key: Int) -> Option<u64>
    reads(*p.nodes)
    contract { requires root < (*p.nodes).len; ensures when Some: result < (*p.nodes).len; }
{
    id = root
    loop {
        if id >= (*p.nodes).len { return None }        // Rule 7: "the test establishes the fact; one compare, no trap"
        n = &(*p.nodes)[id]                            // path shape (*p.nodes)[.] is fixed; only the index varies
        if n.op == Var && n.lit == key { return Some(id) }
        if n.l == NONE { return None }
        id = n.l                                       // n still names the old slot: Rule 2's index snapshot
    }
}

fn pool_set_child(p: &Pool, parent: u64, which: u64, fresh: u64) -> u64
    writes((*p.nodes)[parent].l), writes((*p.nodes)[parent].r)
    contract { requires parent < (*p.nodes).len; requires which < 2; }
{
    if which == 0 { old = (*p.nodes)[parent].l; (*p.nodes)[parent].l = fresh; return old }
    old = (*p.nodes)[parent].r; (*p.nodes)[parent].r = fresh; return old
}
```

Three clauses carry this form. Rule 2's static shape: "a loop-carried rebinding may
change only the index values inside the path" — `(*p.nodes)[id]` keeps its shape while
`id` varies, so the loop is admitted where §6.1's was not. Rule 2's snapshot: "An index
expression inside a path is evaluated when the reference is formed; the path records that
value, and later assignments to the variables the expression used do not change it" — so
after `id = n.l` the reference `n` still names the previous slot and must be re-formed,
which the loop head does. Rule 9's index-through-argument clause: "an index enters an
effect only through an argument, evaluated once at the call" — `parent` is an argument,
so `writes((*p.nodes)[parent].l)` is a legal row, and it is slot-precise, which is what
makes §7's item 5 parallel.

What the pool form costs and gives up is in §8 and §9 C3.

### 6.4 Allocation-free bounded-depth cleanup, newly expressible

Rule 8's release is recursive and its depth is data (§11 G-D). Revision 4's general
`swap` — "Two operations apply to any owned place, not only to window slots" — makes the
classic flattening writable with no allocation at all, by threading the pending list
through the nodes' own `left` fields:

```text
fn link(x: own Option<Box<Node>>, head: &Option<Box<Node>>)
    writes(head)
    // caller's obligation: x's own left is None (see §11 G-A: the rule set cannot state this)
{
    match &x { Some(xb) => { swap(&(*xb).left, head) }  None => {} }   // head := None, x.left := old list
    head = move x                                                       // releases the None that was there
}

fn demolish(sub: own Option<Box<Node>>)
{
    head = move sub
    while true {
        cur = None
        swap(&cur, &head)
        match &cur {
            Some(b) => {
                l = None;  swap(&(*b).left,  &l)
                r = None;  swap(&(*b).right, &r)
                link(move l, &head)
                link(move r, &head)
            }
            None => { break }
        }
        // `cur` now holds a childless node: Rule 8's release at scope exit is O(1), not recursive
    }
}
```

Four swaps per node — roughly eight stores — buys an O(1) stack for the release of an
arbitrarily deep tree. Under revision 2 this had to be a worklist `Box<Slots<...>>`,
whose `push` can fail, and whose failure path fell back on exactly the unbounded
recursive release it was avoiding. That failure mode is gone.

---

## 7. Overlaps the rules admit and refuse

Rule 13, revision 4: "Two adjacent statements of one block may overlap when the first's
write paths are disjoint from the second's read and write paths and vice versa, using the
same path-overlap and index/range-disjointness judgment as Rule 10; read/read overlap is
allowed; a by-value consumption counts as a write of the argument's place." Rule 14:
"Allocation and release carry no effect entry and never prevent two statements from
overlapping."

**Admitted.**

1. *Two read-only analyses of the same tree.*
   `s1 = size(&(*root)); s2 = depth(&(*root))` — both rows are `reads`. Rule 13's first
   example is this: "`s1 = stats(&v); s2 = stats(&v)`  // read/read: may overlap".

2. *The two children of one node, rewritten in place.*
   ```text
   n.left  = rewrite_child(n.left,  env, logA, repA)
   n.right = rewrite_child(n.right, env, logB, repB)
   // may overlap: writes(n.left) and writes(n.right) are distinct fields
   ```
   Rule 10's "`two(&a.x, &a.y)`  // accepted: distinct fields". Both statements allocate;
   Rule 14 exempts allocation. This is the form revision 2 could not reach at all: with
   the children in one `Slots` block, every mutating operation declared the whole block,
   and two such statements were never disjoint. The fixed-arity shape that revision 4's
   payload step unlocked is what makes divide-and-conquer over a binary tree admissible.

3. *Fork-join recursion into both subtrees.*
   ```text
   match &n.left { Some(lb) =>
     match &n.right { Some(rb) => {
       rewrite_at(&(*lb), env, logA, repA, budget - 1)
       rewrite_at(&(*rb), env, logB, repB, budget - 1)
       // may overlap: the substituted rows write below n.left.Some.0 and below n.right.Some.0,
       // which differ at their first field step
     } None => {} } None => {} }
   append(&(*logA), &(*logB))       // at the join: one memcpy (Rule 6)
   ```
   Both payload facts are live simultaneously because a `match` is a read, not a write,
   and Rule 2 invalidates a payload fact only on "any write to the enum" (§4b). The join's
   `append` has the contract `requires dst.room >= src.len; ensures dst.len ==
   entry(dst).len + entry(src).len, src.len == 0`, and it moves exactly the bytes a single
   shared log would have written, so the per-arm log is free.

4. *Structurally distinct rotations.* `swap(&(*lb).left, &(*rb).right)` where `lb` and
   `rb` were reached through `n.left` and `n.right`: the two paths differ at their first
   field step, so they are disjoint by shape without any index reasoning. Rust needs
   field-splitting gymnastics or `split_at_mut` for the equivalent; here the path shapes
   decide it.

5. *Two pool edits at proved-distinct parents.*
   `pool_set_child(&p, i, 0, a); pool_set_child(&p, j, 1, b)` with the fact `i != j` —
   Rule 10's "`two(&r[i], &r[j])`  // accepted only with the fact `i != j`", available
   because §6.3's row is slot-precise.

**Refused.**

6. *Rewriting a child while reading it.*
   `n.left = rewrite_child(n.left, ...); k = size_of_child(&n.left)` — `writes(n.left)`
   meets `reads(n.left)`. Rule 13's "`push(&v, 1); stats(&v)`  // may not overlap".
   Correct; C++ would be racy.

7. *Two rewrites through one log or one report.* Both statements write `(*log).len` and
   `rep.hits`. Refused, and repaired at zero runtime cost by item 3's per-arm log and
   `append` — the concatenation writes the same bytes the shared log would have.

8. *Two allocating rewrites into one pool.* `pool_alloc` writes `(*p.nodes).next` and
   `(*p.nodes).len`, so two arms allocating from one pool never overlap. The rule file
   records this: "Pool allocation from one pool serializes against every other access to
   that pool; per-worker pools are the standard form." The repair is not free here; see
   §9 C3.

9. *Fail-fast between the two children.* This one is a trade, not a defect, and it is
   worth naming because revision 4 made overlap a property of *adjacent* statements:
   ```text
   n.left  = rewrite_child(n.left,  env, logA, repA)
   if repA.fails > 0 { return 0 }                      // this statement separates them
   n.right = rewrite_child(n.right, env, logB, repB)
   ```
   With the check between them the two updates are not adjacent and the permission of
   item 2 is not available. Moving the check after both restores the overlap and executes
   the second update even when the first failed — which, since "The program's meaning is
   its sequential meaning", is a genuine change of program, not a scheduling choice. The
   cost of fail-fast is therefore the parallelism of item 2; the cost of overlap is one
   wasted subtree rewrite on the failure path. A C++/Rust `rayon::join` faces the same
   choice, so this is not a candidate-specific cost.

---

## 8. Cost table

Baselines. **C** = C++ `struct Node { Op op; int64_t lit; Node* left; Node* right; }`
with `unique_ptr`-equivalent ownership. **R** = safe Rust
`struct Node { op: Op, lit: i64, left: Option<Box<Node>>, right: Option<Box<Node>> }`
with `Box::try_new` — the same shape, bit for bit, since revision 4.

| Operation | C baseline | R baseline | x1 revision 4 | Δ vs R | Δ vs C |
|---|---|---|---|---|---|
| Node layout | 2 raw pointers | 2 niche `Option`s | 2 niche `Option`s (Rule 12) | 0 | 0 |
| Descend one edge | load + test + branch | `if let Some(b) = &n.left` | `match &n.left { Some(b) => … }` | 0 | 0 |
| Bounds/validity per hop | none | none | none (no index on the tree path) | 0 | 0 |
| In-place child rewrite | `n->left = f(old)`: 1 load, 1 store | `n.left = f(n.left.take())`: +1 store of `None` | atomic update: 1 load, 1 store | **−1 store** | 0 |
| Fallible rewrite, failure path | 1 branch | store `None`, store original back | atomic update returns the original: 1 store | **−1 store** | 0 |
| Second output of the rewrite (did it fire, did it fail) | returned in registers | returned in registers | must ride in `Report` (Rule 6: "f … returns the place's type") | +1 store +1 load +1 compare, folded by any inliner | same |
| Extract a child to name it in this frame | `Node* old = n->left; n->left = 0;` | `n.left.take()` | `hole = None; swap(&n.left, &hole)` | 0 after constant folding; worst case +1 store | same |
| Hand the subtree to a helper | move an 8B pointer | move an 8B `Option<Box>` | `retire(move …)`: 8B move | 0 | 0 |
| Dispose of the removed subtree | recursive `~unique_ptr` | recursive `Drop` | Rule 8 recursive release | 0 | 0 |
| …with an O(1) stack | hand-written flattening | hand-written flattening | §6.4 `swap` chain, **no allocation** | 0 vs careful R (+4 swaps/node vs naive R) | same |
| Allocation | `new` throws | `try_new`: 1 branch | `Box::new(…)`: 1 branch (Rule 14) | 0 | +1 predictable branch |
| Self-imposed depth bound | none (guard-page trap) | none (guard-page trap) | `budget` compare per level, because the candidate promises no trap | **+1 compare +1 branch per level** | same |
| **Iterative cursor descent** | `while (n) n = n->left;`: 1 register update + 1 load/hop | same | **refused** (Rule 2 static shape); recursion instead | **+1 call +1 return per hop, +O(d) stack** | same |
| …via the pool instead | — | — | §6.3 loop | +1 bounds compare +1 branch, +1 multiply-add per hop; −pointer load | same as R; +compare/branch vs C |
| **Split find-then-mutate, per pair** | find returns `Node**`; mutate is 1 store | find returns `&mut Option<Box<Node>>` | Rule 4 forbids; replay the path recursively | **+d loads, +d tests, +d calls/returns** | same |
| …via the pool instead | — | — | `pool_find` → `id`, then one slot write | +1 bounds compare (hoistable) | same as R |
| Rewrite that must read the sibling child | free | free (field-split borrows) | §9 C2: swap out, read, swap back | **+2 stores per rewrite** (strict reading; 0 under the other) | same |
| Parallel rewrite of the two subtrees | programmer's word | `rayon::join` + field split | admitted (§7 items 2, 3) | 0 | 0 |
| …with one shared log | shared buffer + lock | shared buffer + lock | refused; per-arm log + `append` | 0 (same bytes) | 0 |
| Parallel allocation into one pool | bump pointer + atomic | same | refused (§7 item 8) | per-worker pools: +renumber pass or +1 branch/hop | same |
| By-value (ML) rewrite | — | rebuild, alloc per node | §5, accepted | 0 | 0 |

Two numbers dominate, and both are new this round because round one's two dominant
numbers (the extra dependent load per hop from the child window, and the four-operation
order-preserving extraction) no longer exist:

- **+1 call/return per hop** for any descent that a C, C++ or Rust program writes as a
  loop, plus O(depth) stack where the loop used O(1);
- **+d loads, tests and calls per find-then-mutate pair**, unchanged from round one,
  because Rule 4 still forbids returning the found reference and Rule 3 would have
  invalidated it across the mutating call anyway. The fused form of §3 has this term at
  exactly zero, and so does the pool form of §6.3.

Everything else in the table is zero against safe Rust, and two entries are *negative*:
the atomic update removes the `Option::take` store that safe Rust pays on both the
success and the failure path.

Proof annotations — the `invariant`-free contracts, the `ensures` relations, the `budget`
parameter's declaration — cost nothing at run time. The `budget` *test* is counted above,
because it executes.

---

## 9. Counterexamples

**C1 (strongest). The candidate refuses the iterative cursor, prices the refusal as "same
loads", and does not price the two things that actually differ: the call and the stack.**

The rule file's own cost record says: "A rebinding of a reference that extends its own
path in a loop is refused; recursion or indices cost the same loads." Loads, yes. But the
refused form is

```text
while (n) { n = n->left; }
```

which is one dependent load and one register update per hop, and the admitted form is a
self-call per hop. Against that, the recursion adds the call and the return, the argument
setup for `env`, `log`, `rep` and `budget`, whatever callee-saved registers the frame
spills, and — the part that is not an instruction count — O(depth) of stack where the
loop used O(1). For a compiler pass walking a left-associated expression chain of a
million `Add` nodes, or a DOM walk, or a dominator-tree descent, that is the hottest loop
in the program turned into a recursion.

The consequence compounds with the candidate's central promise. There are no runtime
traps, so the stack depth cannot be answered by the guard page the C and Rust baselines
rely on; the program must carry a `budget`, which costs one compare and one predicted
branch *per hop* — a cost neither baseline pays — and the program must have a correct
bound, which for a tree built from untrusted input is a property of the input, not of the
code. The pool form of §6.3 escapes all of it and restores the loop exactly, which is the
honest answer, but it is the answer that C3 is about.

This is the one place where revision 4 is strictly worse than revision 2 *for this task*,
and it is not a rule defect: Rule 2's join rule needed the static-shape clause, or a
rebound reference at a loop header would have had an infinite target set. The cost is
real and it is charged to the language, not to the writer.

**C2 (rule gap with a priced consequence). The atomic update cannot consult the sibling
it is rewriting next to.**

Rule 6: "f's row must not overlap any prefix of `place`". For
`n.left = fold(n.left, &n.right)` — constant-folding a binary node, which is *the*
canonical tree rewrite — `place` is `n.left`, whose prefixes are `n.left` and `n`. The
argument path `n.right` is below `n`, so under Rule 10's overlap judgment (one path a
prefix of the other) `f`'s substituted row overlaps the prefix `n` and the update is
refused. Under a narrower reading — that only the prefixes' *own* storage is at stake, so
a disjoint sibling field is fine — it is accepted with no cost at all. The rule text does
not separate the two readings; the quoted sentence is in §11 as G-B and I do not resolve
it.

Priced both ways. Under the narrow reading: zero. Under the strict reading, the rewrite is

```text
h = None
swap(&n.left, &h)                       // +1 store: the place holds None for the duration
v = fold_value(&h, &n.right)            // now h and n.right are different roots
swap(&n.left, &h)                       // +1 store, and the place is whole again
```

which is +2 stores per rewrite against both baselines, and it reintroduces exactly the
hole the atomic update exists to prevent — during `fold_value` the tree is missing a
child, so any function called there must not observe it. Safe Rust has neither cost:
`&mut n.left` and `&n.right` are disjoint field borrows and the borrow checker splits
them without help. That is the sharpest surviving comparison in this task: revision 4's
path-overlap judgment is *more* precise than Rust's for two references (§7 item 4), and
the atomic update's row restriction is *less* precise than Rust's for one place plus one
neighbour.

A zero-cost escape exists whenever the needed sibling data is Copy: hoist it before the
update (`s = n.right_lit` is a load the fold needed anyway) and pass it by value. That
covers arithmetic folding and fails exactly when the fold must consume or restructure the
sibling.

**C3 (capability, unchanged in substance from round one, now unconditional). The rewrite
that removes C1's per-hop call also removes the ownership guarantee and the free
parallelism.**

Rule 16 is confirmed, so §6.3's pool is a real option and it is the only form that gives
back the iterative cursor and the O(1) find-then-mutate. It pays twice.

*Identity.* Every edge becomes a `u64`, and `u64` is Copy. After
`old = pool_set_child(&p, parent, 0, fresh)` the caller holds an index to a detached
subtree, and nothing in the rule set stops it from being reinstalled, kept past the slot's
reuse, or dropped. Rule 16 says so: "A stale index that is still in bounds names the
current occupant of that slot: a logic error, not a memory error." Under §3's owning form
x1 gives the opposite for free: Rule 3 invalidates the reference, Rule 8 releases the
subtree exactly once. The mitigation Rule 16 names — "Programs that need to detect it keep
a generation number as data" — costs one load of `gen`, one compare and one branch at
every edge dereference, plus an error arm threaded through every traversal's return type
because there are no traps. That is a per-hop cost on the task's hottest operation, worse
than the per-hop call it was introduced to remove.

*Parallelism.* §7 item 2 gives the owned form a free fork-join over the two children,
because `n.left` and `n.right` are distinct fields. The pool form loses it as soon as an
arm allocates, because both arms write `(*p.nodes).next` and `(*p.nodes).len` (§7 item 8).
Per-worker pools restore it, and then a subtree built in worker A and linked from worker
B needs either a `(pool, index)` handle — one extra branch per hop, permanently — or a
renumbering pass over every node after the join, which is O(nodes) of pure overhead that
neither baseline pays.

So the candidate offers a three-way trade for this task and no corner of it is free: the
owned form is safe and pays the call per hop and `+d` per split find-mutate; the pool form
is fast and gives up the identity guarantee; the guarded pool form is safe and pays per
hop, and it also has to choose between a wide handle and a merge pass to get the
parallelism the owned form had for nothing.

**C4 (gap, narrowed since round one). Release still has no depth bound, but the program
can now fix it with no allocation.** Rule 8's release is recursive and the tree's depth is
data; the rule file says nothing about stack depth for it, and there is no trap to abort
with. §6.4's `swap`-chained flattening answers it in the program at four swaps per node,
and — this is the round-two improvement — it needs no allocation, so unlike revision 2's
worklist it has no failure path that falls back on the recursion it was avoiding. What
remains is that a writer who simply lets an affine tree go out of scope gets the
unbounded release, silently.

---

## 10. Rules consistent, and task achievable — recorded separately

**Rules consistent for the program actually written (§3, §6.2, §6.4): yes.** Every step is
covered by an exhibited clause. Two pairs of sentences that could have collided point the
same way: Rule 11's fact invalidation and Rule 3's content-write exemption agree that a
write below `n.left.Some.0` leaves the payload fact on `n.left` alone (§4b); and Rule 6's
atomic update and Rule 12's no-hole sentence agree because "no program point lies
between" (§4c). The refusals in §4e, §6.1 and §7 items 6–8 are coherent applications of
Rules 2, 10.1 and 13, not accidents.

**Rules consistent for the sibling-consulting rewrite (§9 C2): not decided.** One sentence,
quoted at G-B, decides between zero cost and +2 stores per rewrite, and I do not resolve
it.

**Task achievable: yes, with cost.** Part by part:

- *traverse by reference through variant payload paths* — accepted, **zero cost** against
  both baselines. This is the part round one could not write at all.
- *replace a subtree with the atomic update* — accepted, **zero cost** against C++ and
  **one store better** than safe Rust, with the caveat of C2 when the sibling must be
  read, and a folded-away `Report` store for the second output.
- *hand a subtree to another function by ownership* — accepted, zero cost. The mechanism
  is clean: the atomic update delivers the old value as a whole local, after which
  Rule 10.2's by-value comparison is discharged by root-distinctness. No region, brand or
  lifetime appears anywhere.
- *clean up removed parts* — accepted, zero cost, no destructor and no drop flag; the
  bounded-stack variant is now allocation-free (§6.4). G-D (release depth) remains open.
- *early-exit error path* — accepted, zero cost; memory safety is free (Rule 8 on affine
  locals) and structural integrity is free (the atomic update never exposes a hole).
- *the iterative find-then-mutate form* — **refused**, with real cost: the recursive
  equivalent adds a call and a return per hop plus O(depth) stack plus the `budget`
  compare per hop; the pool equivalent restores the loop and pays C3.
- *the split find-then-mutate form* — accepted, at `+d` loads, tests and calls per pair;
  exactly zero in the fused form of §3 and in the pool form of §6.3.

**Verdicts.** `accepted-fine` for the traversal, the atomic-update replacement, the
ownership handoff, the cleanup, the early-exit path, the fused recursive rewrite, and the
parallel forms of §7 items 1–5. `rejected-real-cost` for the iterative by-reference
find-then-mutate cursor of §6.1 (+1 call and +1 return per hop, +1 budget compare per hop,
O(depth) stack) and for the split find-then-mutate (+d per pair).
`undecided-rule-gap` for the sibling-consulting atomic update of §9 C2, priced at 0 or +2
stores per rewrite depending on the reading of one sentence.

---

## 11. Rule gaps, quoted

**G-A — a precondition cannot state that an enum parameter holds a variant.**

> "A payload step is available only under the refinement fact that the enum currently
> holds that variant, which a `match` or `if let` on the enum establishes in the selected
> arm and which any write to the enum invalidates." (Rule 2)
>
> "Contracts use `requires`, `ensures`, and `ensures when Variant:` for result-routed
> relations" (Rule 11)

Missing: whether a `requires` may state a refinement fact about a *parameter* (`requires
p is Some`), so that a callee could declare `writes(p.Some.0)` and the caller could
discharge it. Rule 11 exhibits variant routing only on the *result*.

*Not load-bearing for this task, and stated as such.* Every callee here takes the node
itself (`&Node`) and does its own `match`, which is one discriminant test that both
baselines also execute — so the workaround is free. It does cost §6.4's `link` the ability
to state its precondition ("x's own left is None"), which is therefore a program
obligation the rule set cannot check. That is a logic obligation, not a safety one: if it
is violated, a well-formed subtree is released early, which is a wrong answer, not a
memory error.

**G-B — the scope of the atomic update's row restriction (load-bearing, §9 C2).**

> "f is total and returns the place's type; f's row must not overlap any prefix of place;
> failure is an enum in the place" (Rule 6)

Missing: whether "overlap any prefix of place" is judged by Rule 10's ordinary path
overlap — under which every path sharing the place's root overlaps the root prefix, so `f`
may touch nothing under that root, including a disjoint sibling field — or whether only
the prefixes' own storage is meant. It decides whether `n.left = fold(n.left, &n.right)`
is accepted at zero cost or costs +2 stores per rewrite.

A second, smaller question in the same sentence: "failure is an enum in the place" reads
as *the* failure mechanism, while "f's row must not overlap any prefix of place"
presupposes that `f` may have a row and therefore may report out of band, which is what
§3.3 does. This program does not depend on the resolution — the in-place alternative is
writable, as `enum Child { Empty, Sub(Box<Node>), Failed(Box<Node>, Err) }` — but it is
the *more* expensive one: three variants with a Box payload cannot all fit the null niche,
so the field grows from 8 to 16 bytes and every node grows by 16, costing cache on every
hop of every traversal. The out-of-band `Report` keeps the 8-byte niche.

**G-C — index expressions inside a contract (narrowed since round one).**

> "Signatures never contain index expressions; an index enters an effect only through an
> argument, evaluated once at the call." (Rule 9)

Missing: whether "signatures" includes the `contract` block or only the effect row. The
effect row is settled by the second clause — `writes((*p.nodes)[parent].l)` in §6.3 is
legal because `parent` is an argument — but an `ensures` such as
`ensures (*p.nodes)[parent].gen == entry(*p.nodes)[parent].gen` is not decided. It is no
longer load-bearing for the *tree* shape, which has no nested measure; it costs the pool
form of §6.3 the ability to carry a generation across a call, so a guarded pool must
reload and re-compare `gen` after every call that might have touched the pool: +1 load, +1
compare, +1 branch per call under the strict reading, zero under the other.

**G-D — no depth bound on the compiler-emitted release (unchanged, and now compounded).**

> "at scope exit the compiler releases their memory recursively (Box, Slots, Ring, Array)
> and runs no user code" (Rule 8)
>
> "There is no writer-accessible unsafe escape or runtime trap." (project statement,
> restated in the candidate's evaluation criteria as "no runtime traps")

Missing: any statement about stack depth, for the release or for the program's own
recursion. Round one recorded this for the release only. Revision 4 compounds it, because
Rule 2's static-shape clause now forces the *program's* descent into recursion too
(§6.1), so both the traversal and the release are unbounded recursions over data-shaped
depth in a language that promises there is no trap to catch the overflow. §6.4 fixes the
release side inside the program at four swaps per node; the traversal side is fixed only
by the `budget` compare per hop (§9 C1) or by moving to indices (§6.3, §9 C3).
