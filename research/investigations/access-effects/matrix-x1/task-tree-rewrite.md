# Candidate x1 engineering task — recursive tree rewrite over Box-linked nodes

Derived against the frozen rule set in `CANDIDATE-X1.md` (2026-09-18) and only that
file. The engineering-task row quoted from `PROGRAMS.md` is "Recursive tree
transformation — Visit and rewrite an owned tree, transfer subtrees across helpers and
return a new root while disposing of removed resources", with couplings "Recursive
layout/functions, ownership transfer, generics, replacement and cleanup" and costs
"Deep copies, temporary allocation, address stability and proof annotations". P19's
declared depth bound is touched; P3's "a relation to a deleted node must not silently
become a relation to a replacement node" is touched by the index-based form.

Nothing under "Not in this candidate" or "Deferred" is used: no `with` block, no
`uniq`/`mut` marker, no region or store brand, no `take`/`put` hole, no quantified
fact over elements, no channel, no destructor, no trap, no header-plus-tail block, no
returned reference. `FixedVector<T, n>` is under "Pending owner ruling" and is not
used. Rule 16 is used only in the section whose heading says so.

Cost convention from the protocol: runtime performance is the only cost. Verbosity,
extra parameters, duplicated code and proof annotations are not costs. "One move of
`T`" is a relocation of `sizeof(T)` bytes, which Rule 1's consequence — "any value can
be relocated by copying its bytes (memmove, realloc), because nothing inside it points
anywhere" — makes a plain byte copy with no fixup.

---

## 1. The question this task tests

Can a program that descends an owned tree, replaces a subtree at a data-determined
position, hands the removed subtree to another function by ownership, disposes of what
is removed, and exits early on an error, be written under x1 at the cost of an
idiomatic C++/Rust tree rewrite — in both the fused recursive form and the split
find-then-mutate form?

---

## 2. Choosing the node representation, before any program is written

The task says "Box-linked nodes", so the first question is which of the two obvious
Box-linked shapes x1 can actually name. This is not a stylistic choice; the rule text
decides it.

**Shape 1 — fixed arity, optional inline children (the C++ shape).**

```text
struct Node { op: Op, lit: Int, left: Option<Box<Node>>, right: Option<Box<Node>> }
```

Rule 1 accepts the type: every payload is owned, and `Option<Box<Node>>` is not one of
the rejected forms (`Option<&T>` is rejected "whatever T is"; a Box payload is owned).
Rule 12 even endorses the representation for storage — "slots: DynBox<Option<Entry>>
// non-Copy payloads: Option, using a null niche where the type has one".

But the traversal cannot be written. Rule 2 defines the path grammar exhaustively:

> "A path starts at a local variable or a parameter and continues through fields, `*`
> (Box content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7)."

There is no step for "the payload of an enum variant". `&n.left` names the whole
`Option<Box<Node>>` field; `&(*n.left)` requires a step from an `Option<Box<Node>>` to
its `Box<Node>` payload, and that step is not in the list. Rule 9 repeats the same
grammar for effect rows — "each path starts at a reference parameter and may continue
through fields, `*`, and whole-index or range positions supplied as arguments" — so an
effect row cannot name a variant payload either, which means no helper can declare
that it writes one optional child and not the other. Rule 4's `match find(&v, k) {
Some(i) => ... }` is the only `match` in the rule file and it binds a Copy `u64` out of
an owned result, not a payload path inside storage.

This is gap **G1** below. It is not a rejection I may decide; it is the absence of a
grammar step the shape needs, and I do not add one.

**Shape 2 — child count is data, edges are Boxes (the shape used from here on).**

```text
enum Op  { Lit, Var, Add, Mul }
enum Err { Oom, NotFound, TooDeep }

struct Node {
    op:   Op,
    lit:  Int,
    kids: DynBox<Box<Node>>,      // Rule 1: owned payloads only
}
```

Every step this task needs is in Rule 2's grammar: `n.kids` is a field, `n.kids[i]` is
an index (Rule 7), `*n.kids[i]` is Box content. Absence of a child is the absence of a
slot, which is Rule 12's own prescription — "Structures whose occupancy is decided by
data keep that occupancy as data" — and Rule 6 supplies the window operations that move
an `own Box<Node>` in and out of the block. `Node` contains no linear part, so by
Rule 8 it is affine: "at scope exit the compiler releases their memory recursively
(Box, DynBox) and runs no user code."

Shape 2 is what the derivation uses. Its cost against Shape 1 is priced in §8 and it is
the reason the task is not zero-cost.

---

## 3. The strongest program

The strongest case is the one where static analysis provably cannot help:

- the rewrite target is found by a data-dependent search, so no index is a literal;
- the replacement's *shape depends on the removed subtree's contents*, so the fallible
  allocation cannot be hoisted above the extraction;
- sibling order is semantically significant (`Add` is written commutative here only for
  brevity; `Mul` of a matrix expression is not), so a reordering removal is wrong;
- the removed subtree is handed to a separate function by ownership, across a
  signature, so the checker sees only rows and contracts (Rule 10: "Recursion is
  checked through contracts, never by unfolding bodies");
- there is an early-exit error path that can fire *after* the extraction.

### 3.1 The order-preserving exchange

Rule 6 fixes the move-out forms:

> "There is no `take` operation and no partial move out of any place: the only ways to
> move a value out of storage are consuming a whole local (`move x`), the window
> operations, and the atomic update."

A child slot is not a whole local, so the extraction must be a window operation. The
window operations that yield a value are `pop` (last slot) and `swap_remove` (any slot,
moving the last slot into the hole). `swap_remove` alone reorders. Order is restored by
running the window four times:

```text
// Replace slot i of a child block by `fresh` and return the previous occupant,
// leaving every other slot at its original index.
fn exchange(kids: &DynBox<Box<Node>>, i: u64, fresh: own Box<Node>) -> own Box<Node>
    writes(kids)
    contract {
        requires i < len_of(kids);
        ensures  len_of(kids) == len_of(deref(entry(kids)));
    }
{
    old  = swap_remove(kids, i)        // len L-1; slot i := former last; old is out
    push_nogrow(kids, move fresh)      // len L;   slot L-1 := fresh
    tail = swap_remove(kids, i)        // len L-1; slot i := fresh; tail = former last
    push_nogrow(kids, move tail)       // len L;   slot L-1 := former last
    move old
}
```

Both index cases are correct. For `i < L-1`: after step 1 slot `i` holds the former
slot `L-1`; step 2 appends `fresh` at `L-1`; step 3 returns the former slot `L-1` and
moves `fresh` from `L-1` into `i`; step 4 puts the former slot `L-1` back at `L-1`.
For `i == L-1`: step 1 is a plain pop, step 2 places `fresh` at `L-1`, step 3 pops
`fresh` back out, step 4 places it at `L-1` again; `old` is still the original
occupant. Nothing but slot `i` changes, and `len_of` returns to `L`.

### 3.2 The fused recursive rewrite, with the early-exit path

```text
fn build_replacement(old: &Node) -> Result<Box<Node>, Err>
    reads(old)
{
    // the replacement's shape is read off the removed subtree: the allocation
    // cannot be hoisted above the extraction
    if old.op == Add && len_of(old.kids) == 2 {
        Ok(Box::new(Node { op: Lit, lit: old.lit, kids: DynBox::new<Box<Node>>(0)? })?)
    } else {
        Ok(Box::new(Node { op: Var, lit: old.lit, kids: DynBox::new<Box<Node>>(0)? })?)
    }
}

fn retire(sub: own Box<Node>, log: &DynBox<Int>)
    writes(log)
    contract {
        requires len_of(log) < cap_of(log);
        ensures  len_of(log) == len_of(deref(entry(log))) + 1;
    }
{
    push_nogrow(log, (*sub).lit)
    // `sub` is affine and unconsumed here: Rule 8 releases the whole subtree
    // recursively at scope exit and runs no user code
}

fn rewrite_at(n: &Node, key: Int, budget: u64, log: &DynBox<Int>) -> Result<u64, Err>
    writes(n), writes(log)
    contract {
        requires len_of(log) < cap_of(log);
        ensures  result <= 1;
        ensures  len_of(log) == len_of(deref(entry(log))) + result;
    }
{
    if budget == 0 { return Err(TooDeep) }              // P19-style declared bound
    m = len_of(n.kids)
    for i in 0..m {
        invariant a: len_of(n.kids) == m;
        invariant b: len_of(log) == len_of(deref(entry(log)));

        c = &(*n.kids[i])                              // Rule 2: field, index, Box content
        if c.op == Var && c.lit == key {
            old = swap_remove(&n.kids, i)              // c is invalid from here (Rule 3)
            match build_replacement(&(*old)) {
                Ok(fresh) => {
                    push_nogrow(&n.kids, move fresh)
                    t = swap_remove(&n.kids, i)
                    push_nogrow(&n.kids, move t)       // §3.1's dance, spelled out here
                    retire(move old, log)              // ownership leaves this frame
                    return Ok(1)
                }
                Err(e) => {
                    push_nogrow(&n.kids, move old)     // restore on the error path
                    t2 = swap_remove(&n.kids, i)
                    push_nogrow(&n.kids, move t2)
                    return Err(e)
                }
            }
        }
        r = rewrite_at(&(*n.kids[i]), key, budget - 1, log)?
        if r == 1 { return Ok(1) }
    }
    Ok(0)
}
```

### 3.3 The split find-then-mutate form

Rule 4 forbids the C++ `Node**` and the Rust `&mut Box<Node>` result outright:

> "A function that 'finds' something returns an index or other owned data; the caller
> forms the reference."

For a tree the owned datum is a *path* of child indices, so the search records a path
and the mutation re-descends it. The path buffer is an inline `array<u64, 64>` with a
declared depth bound, so no allocation is added (Rule 7: "an inline `array<T, N>`
(constant length, always fully initialized)"; `a[i]` "requires i < 8" is a constant
comparison the checker discharges without a runtime test).

```text
struct Path { steps: array<u64, 64>, d: u64 }

fn find_path(n: &Node, key: Int, p: &Path, depth: u64) -> Bool
    reads(n), writes(p)
    contract { requires depth <= 64; ensures result == true implies len_ok(p); }
{
    m = len_of(n.kids)
    for i in 0..m {
        invariant a: len_of(n.kids) == m;
        c = &(*n.kids[i])
        if c.op == Var && c.lit == key {
            if depth < 64 { p.steps[depth] = i; p.d = depth + 1; return true }
            return false
        }
        if depth + 1 <= 64 {
            if find_path(&(*n.kids[i]), key, p, depth + 1) {
                p.steps[depth] = i                     // unwind writes the prefix
                return true
            }
        }
    }
    false
}

fn mutate_at_path(root: &Node, p: &Path, log: &DynBox<Int>) -> Result<u64, Err>
    writes(root), writes(log), reads(p)
    contract { requires p.d >= 1; requires p.d <= 64; requires len_of(log) < cap_of(log); }
{
    // the second descent: a reference cannot be carried out of find_path (Rule 4)
    // and could not have survived the call anyway (Rule 3)
    n = &(*root)
    for k in 0..(p.d - 1) {
        invariant s: k < p.d;
        j = p.steps[k]                                 // Rule 7: k < 64 is constant
        if j >= len_of(n.kids) { return Err(NotFound) } // the path may be stale: data test
        n = &(*n.kids[j])                              // rebinding a reference (Rule 2)
    }
    last = p.steps[p.d - 1]
    if last >= len_of(n.kids) { return Err(NotFound) }
    match build_replacement(&(*n.kids[last])) {
        Ok(fresh) => { old = exchange(&n.kids, last, move fresh); retire(move old, log); Ok(1) }
        Err(e)    => Err(e)
    }
}
```

`n = &(*n.kids[j])` inside the loop is Rule 2's rebinding — "A reference may be
rebound" — and at the loop header the target set is the union of the paths reached on
each incoming edge, so "every check on it must hold for every member of the set": the
checks used are `len_of(n.kids)` reads and an index proved by the immediately preceding
test, both of which hold for every member.

---

## 4. Rule-by-rule trace at the interesting points

**(a) Forming the child reference.** `c = &(*n.kids[i])` uses exactly three of Rule 2's
four continuation forms: `.kids` is a field, `[i]` is "index, Rule 7", `*` is "Box
content". Rule 7 requires the index fact: "buf: DynBox<Int>; buf[i] // requires i <
len_of(buf)". Here `i < m` comes from the counted loop and `len_of(n.kids) == m` from
`invariant a`, which is one of Rule 11's admitted forms ("loop-header invariants
`invariant name: affine_expr compare_op affine_expr`"). No runtime compare is emitted;
Rule 7's alternative — "if i < len_of(buf) { use(&buf[i]) } // the test establishes the
fact; one compare, no trap" — is not needed in the fused form and *is* needed in
`mutate_at_path`, where the path is data and the length relation was not carried.

**(b) The extraction invalidates the child reference, and that is fine.** `old =
swap_remove(&n.kids, i)` substitutes to `writes(n.kids)` (Rule 6 declares
`fn swap_remove<T>(buf: &DynBox<T>, k: u64) -> own T  writes(buf)`; `k` is by value and
by Rule 9 "A by-value parameter has no effect entry"). `n.kids` is a proper prefix of
`c`'s path `n.kids[i].*`, so Rule 10.3 applies — "A live reference outside the call
whose path has a proper prefix among the call's write paths becomes invalid after the
call (Rule 3)" — and `c` is invalid afterwards. The program never uses `c` again; it
uses `&(*old)`, a fresh path rooted at the new local. Rule 3's second example is the
same situation read from the other side: "p = &*b; c = move b; // b is a proper prefix
of *b: p invalid, even though the Node did not move".

**(c) Extraction turns an interior path into a root, and that is what makes the
handoff checkable.** Before the window operation, the subtree is at `n.kids[i]`, a path
under the same root as everything else in the frame. After it, the subtree is the local
`old`. At `retire(move old, log)`, Rule 10.2 applies — "A by-value argument contributes
a consumption (`move`) or a read (copy) of its place to this comparison" — and the
comparison is between a consumption of the place `old` and `writes(log)`. Different
roots, so Rule 10.1's disjointness obligation is discharged by "different roots". No
region, brand or lifetime is needed for the ownership transfer because the transferred
thing stopped being interior. This is the mechanism by which the task's "hand a subtree
to another function by ownership" works at all under x1, and it costs one window
operation.

**(d) The rejection x1 exhibits for this task is real and correctly aimed.** Rule 10's
third example is the tree case in miniature:

```text
slot = &(*b).next
put(slot, move b)                            // rejected: move b writes the prefix b of slot's path
```

The analogue here is handing the whole tree away while holding a reference into it:

```text
p = &(*root).kids
retire(move root, log)                       // rejected, same clause
```

Rejected, and a C++ program that does this is a use-after-free. No rewrite is owed; the
program simply forms `p` after the handoff, or does not hand `root` away.

**(e) The recursive call.** `rewrite_at(&(*n.kids[i]), key, budget - 1, log)`
substitutes to `writes(*n.kids[i])`, `writes(log)`. Rule 10.1 compares the two: roots
`n` and `log` differ, so they are disjoint. Rule 9's body obligation — "every
statement's effect and every callee's substituted row must be covered by the declared
row" — is met because `writes(*n.kids[i])` is below the declared `writes(n)`. Rule 10's
"Recursion is checked through contracts, never by unfolding bodies" means the callee is
seen only through `ensures result <= 1; ensures len_of(log) == len_of(deref(entry(log)))
+ result`.

**(f) The loop invariant survives the recursive call, and this is load-bearing.** After
the recursive call returns, `invariant a: len_of(n.kids) == m` must still hold or the
next iteration's index is unproved. The callee's substituted write path is
`*n.kids[i]`, which is *below* `n.kids`, not `n.kids` itself. Rule 11 says a fact "is
invalidated when that path is written"; the path mentioned by the fact is `n.kids`, and
Rule 9 fixes what was written: "`writes` covers writing, replacing, moving out of, and
freeing the storage at the path", the path being `*n.kids[i]`. Rule 3 states the same
containment from the reference side: "Writing the storage at p's path or below it (a
content write) does not invalidate p." A write strictly below `n.kids` therefore leaves
the block header alone and the invariant stands. Two independent sentences agree here,
so I read it as decided, not as a gap — but it is worth naming, because it is the
single fact that keeps a recursive rewrite free of re-reading `len_of` at every level.

**(g) `invariant b` and the log contract.** `push_nogrow` requires `len_of(log) <
cap_of(log)` (Rule 6). On the recursing path nothing was pushed in this frame, so
`invariant b` carries `len_of(log) == len_of(deref(entry(log)))`, and the caller's
`requires len_of(log) < cap_of(log)` transfers to the callee unchanged. On the
rewriting path the frame pushes once and returns `1`, matching `ensures len_of(log) ==
len_of(deref(entry(log))) + result`. After a recursive call returns `r`, the program
returns immediately when `r == 1`, so `invariant b` is re-established at the header
only when `r == 0`, where the callee's `ensures` says the length is unchanged. Every
step uses only Rule 11's admitted forms: affine comparisons over measures, a loop-header
invariant and callee contracts. No quantified fact over elements is used, as Rule 11
requires: "There are no quantified facts over array elements".

**(h) The early-exit path.** Two exits fire after state has been disturbed. `Err(TooDeep)`
fires before anything is touched. `Err(e)` from `build_replacement` fires while `old` is
a live local holding a removed subtree. Rule 8 settles the memory question with no code
and no flag: `Node` is affine, and "Affine values are consumed at most once; at scope
exit the compiler releases their memory recursively (Box, DynBox) and runs no user
code." So even if the program simply returned `Err(e)` and dropped `old`, there is no
leak and no double free. The program instead puts `old` back, because leaving the tree
short one child is a *semantic* defect, not a safety one; the restoration costs a second
four-operation dance and is the honest price of a fallible rewrite that must be
transactional. `push_nogrow` after `swap_remove` is provable: `swap_remove` ensures
`len_of(buf) == len_of(deref(entry(buf))) - 1`, and Rule 6's window structure — "Slots
`[0, len_of)` hold values; slots `[len_of, cap_of)` hold nothing" — gives `len_of <=
cap_of`, so `len_of < cap_of` holds after the removal.

**(i) Allocation.** `Box::new(...)?` and `DynBox::new<Box<Node>>(0)?` are Rule 14's
form: "Allocation returns a `Result` and never traps." Each is one predictable branch
plus the propagation the writer already declared in `-> Result<..., Err>`.

**(j) Cleanup of the removed subtree.** `retire` consumes `sub` and never passes it on,
so Rule 8 releases the whole subtree recursively at scope exit. No destructor exists
and none is needed; Rule 8 says so twice ("runs no user code", "there are no
destructors" in Rule 5). This part of the task is free.

**(k) Cleanup, the deep-tree case.** Rule 8's release is *recursive* and the tree's
depth is data. The rule file bounds the program's own recursion only by what the writer
declares (the `budget` parameter above, P19's form), and says nothing about the depth of
the release the compiler emits. This is gap **G4**. The flattening rewrite is
expressible and is priced in §8:

```text
fn demolish(sub: own Box<Node>, work: &Vector<Box<Node>>) -> Result<u64, Err>
    writes(work)
{
    push(work, move sub)?
    while len_of(work) > 0 {
        b = pop(work)                                   // own Box<Node>
        while len_of((*b).kids) > 0 {
            k = pop(&(*b).kids)                         // own Box<Node>
            push(work, move k)?
        }
        // b is now a childless affine Box: Rule 8 releases it at scope exit
    }
    Ok(0)
}
```

`pop(&(*b).kids)` is a path from the local `b` through `*` and a field, so the window
operation applies to it (Rule 2, Rule 6). The loop needs no invariant beyond the
condition, which is Rule 11's own second example: "while len_of(buf) > 0 { pop(&buf) }
// no knowledge: the loop condition is the test".

---

## 5. The Box-linked shape the task literally names, and where it stops

Shape 1 (`left: Option<Box<Node>>`) is traced here for completeness. It reaches three
places where the rule text does not decide the case, and by the protocol I stop at each.

**G1 — no path step into a variant payload.** Quoted in §2. Without it, `&(*n.left)`
is not a path, so neither the traversal nor an effect row that distinguishes the two
children can be written.

**G2 — the scope and shape of the atomic update.** Rule 6 introduces it as one of the
three move-out forms and exhibits exactly one syntax:

> "buf[k] = f(buf[k])                   // atomic in-place update: the old value goes
> into f by value, f's result is committed, no program point lies between; requires k <
> len_of(buf)"

Three things are undetermined for this task. (i) Whether the form applies to a path
that is not a DynBox index — `n.left = f(n.left)` on a struct field. The sentence that
names it as a general move-out form is general; the only exhibition, and its `requires`
clause, are DynBox-specific. (ii) Whether `f` may take parameters besides the old value.
The exhibition is unary. (iii) Whether `f` may return a `Result`. The text says "f's
result is committed" into the slot, which reads as requiring the slot's type.

The consequence for this task is exact and is the strongest counterexample in §9: under
the unary total reading, the atomic update is the zero-cost in-place rewrite form — and
it cannot receive the materials for a replacement, cannot name a destination for the
removed subtree, and cannot fail.

**G3 — partial move out of an owned local.** The by-value recursive rewrite, which is
the form a Rust or ML programmer writes first —

```text
fn rewrite(n: own Node) -> own Node { l = move n.left; ... Node { left: Some(rewrite(move l)), ... } }
```

— needs `move n.left` out of an owned local. Rule 6 says "There is no `take` operation
and no partial move out of any place", Rule 12 says "No place is ever partially moved:
every path is either wholly present or the program cannot name it", and Rule 8's own
example does it: "fn use_it(c: own Conn) { ...; close(move c.f) }   // consuming the
struct as a whole and closing its File is required". The rule file states both and
ranks neither. This gap is not specific to this task — it is the same sentence pair
recorded for value classes — but the by-value recursive tree rewrite depends on it
completely, so it is marked here too.

Shape 2 needs none of G1, G2 or G3: it moves values only through the window operations,
which are exhibited without ambiguity, and it names children only through field, index
and `*` steps, which are all in Rule 2's list. That is why §3 uses it.

---

## 6. The index-based equivalent (depends on Rule 16)

This section, and only this section, uses Rule 16, which the rule file marks
"provisional, pending owner ruling".

```text
struct PNode { op: Op, lit: Int, gen: u64, kids: DynBox<u64> }
struct Pool  { nodes: DynBox<PNode>, free: DynBox<u64> }

fn alloc_node(p: &Pool, x: own PNode) -> u64
    writes(p.nodes)
    contract { requires len_of(p.nodes) < cap_of(p.nodes);
               ensures  result == len_of(deref(entry(p.nodes)));
               ensures  len_of(p.nodes) == result + 1; }

fn find_id(p: &Pool, root: u64, key: Int, budget: u64) -> Option<u64>
    reads(p.nodes)
    contract { requires root < len_of(p.nodes);
               ensures when Some: result < len_of(p.nodes); }

fn set_child(p: &Pool, parent: u64, k: u64, fresh: u64) -> Result<u64, Err>
    writes(p.nodes)
    contract { requires parent < len_of(p.nodes); requires fresh < len_of(p.nodes); }
{
    if k >= len_of(p.nodes[parent].kids) { return Err(NotFound) }   // runtime test: see G5
    old = p.nodes[parent].kids[k]
    p.nodes[parent].kids[k] = fresh                                 // u64 is Copy: one store
    Ok(old)
}
```

`alloc_node` and `find_id` are Rule 16's own shapes — its example is
`fn alloc<T>(a: &Arena<T>, x: own T) -> u64  writes(a.buf)` with the same two `ensures`
clauses — and `find_id` returning an index is what Rule 4 demands anyway.

What this form buys: the mutation after a search is `O(1)`, because `p.nodes[id]` is a
path reached by one index instead of a re-descent. That removes the whole second-descent
cost of §3.3, which is the single largest number in §8.

What it costs, beyond arithmetic: the ownership discipline stops being checked. Rule 16
states the consequence in its own words —

> "A stale index that is still in bounds names the current occupant of that slot: a
> logic error, not a memory error. Programs that need to detect it keep a generation
> number as data."

so "hand a subtree to another function by ownership" degrades to handing over a `u64`,
which is Copy: nothing prevents the caller from keeping and reusing it. Cleanup degrades
from Rule 8's automatic recursive release to a program-maintained free list, and a
missed push is a leak (not a memory error, and not detected). §9's counterexample C2 is
built on this.

**G5 — index expressions inside a contract.** `set_child`'s natural precondition is
`requires k < len_of(p.nodes[parent].kids)`. Rule 9 says:

> "Signatures never contain index expressions; an index enters an effect only through
> an argument, evaluated once at the call."

The first clause says *signatures*, the second narrows the ban to *effects*. A contract
is part of a signature. If the ban covers contracts, then no callee can state a
precondition about a nested window's length, and the check must happen at run time —
which is what `set_child` above is forced to do. That runtime form costs one dependent
load of the inner block's header, one compare and one branch per edge write, plus an
error arm at every call site, because there are no traps. If the ban covers only effect
rows, the precondition is writable and the check is free. The rule text does not
separate the two readings, so I do not choose; §8 prices the expensive reading and
names the cheap one.

---

## 7. Parallel opportunities: admitted and refused

Rule 13's judgment is "A's write paths are disjoint from B's read and write paths and
vice versa, using the same path-overlap and index/range-disjointness judgment as Rule
10", with "Read/read overlap is allowed" and, from Rule 14, "Allocation and release are
not effects" and "never make two parallel arms conflict".

**Admitted.**

1. Parallel read-only traversal of the same tree:
   `par { a = eval(&n, env); b = eval(&n, env) }` — both arms are `reads`, and Rule 13's
   first example is exactly this: "par { s1 = stats(&v); s2 = stats(&v) } // read/read:
   accepted".
2. Parallel rewriting *inside* two distinct subtrees:
   `par { rewrite_in(&(*n.kids[i]), ...); rewrite_in(&(*n.kids[j]), ...) }` with the
   fact `i != j` in hand. Rule 10's "two(&v[i], &v[j]) // accepted only with the fact i
   != j" and Rule 13's "par { bump(&v[i]); bump(&v[j]) } // needs i != j". With literal
   `0` and `1` the fact is immediate; with data-determined `i` and `j` the writer
   carries `i < j` from the loop that produced them.
3. Level-wise fork-join over a child block by adjacent ranges:
   `par { pass(&n.kids[0..mid]); pass(&n.kids[mid..m]) }`, which is Rule 13's fourth
   example verbatim in shape. Each arm may index its own range (`part[k]` requiring
   `k < len_of(part)`, Rule 7) and may write through the Boxes it reaches.
4. Parallel *replace-and-discard* at proved-distinct slots:
   `par { n.kids[i] = move a; n.kids[j] = move b }` with `i != j`. These are statements
   whose write paths are the whole-index positions `n.kids[i]` and `n.kids[j]`, and the
   recursive release of the two old subtrees is allocator work, which Rule 14 exempts.
5. Parallel allocation in both arms while building replacements, by Rule 14's example
   "par { a = build(&x)?; b = build(&y)? } // both allocate: accepted".

**Refused.**

6. Parallel *extracting* edits on one node's child block:
   `par { exchange(&n.kids, i, a); exchange(&n.kids, j, b) }` is refused for every `i`
   and `j`, including proved-distinct ones. `exchange`'s row is `writes(kids)`, because
   Rule 9 forbids an index in a signature — "Signatures never contain index expressions"
   — so a window operation's declared write is the whole block. Two identical write
   paths are not disjoint. This is the one parallel form the C++/Rust baseline has and
   x1 does not: Rust reaches it with `split_at_mut` plus `mem::replace(&mut part[k],
   fresh)`, which is a per-slot exclusive write that also yields the old value. x1's
   range references give the per-slot *write* (item 4) but not the per-slot
   *extraction*, because extraction exists only as a whole-block window operation.
7. Rewriting while reading: `par { eval(&n, env); exchange(&n.kids, i, x) }` — `reads(n)`
   against `writes(n.kids)` overlap. Rule 13's "par { push(&v, 1); stats(&v) } //
   rejected: writes(v) overlaps reads(v)". This refusal is correct; C++ would be racy.
8. Two independent rewrite passes over one tree sharing one log:
   `par { rewrite_at(&root, k1, b, log); rewrite_at(&root, k2, b, log) }` — both
   `writes(root)` and both `writes(log)`. Correct on the tree (the two searches may land
   on the same node) and, for the log, repaired at no runtime cost by giving each arm
   its own log buffer and concatenating after the join — the concatenation is `R` moves
   of `sizeof(Int)` for `R` recorded entries, which is the same traffic a single shared
   log would have written.

**Cost of refusal 6.** The extractions are serialized ahead of the parallel region and
the expensive per-subtree work stays parallel. Each serialized extraction is the
four-operation dance of §3.1 on L1-hot memory, so the serial residue is `O(1)` per
rewrite against `O(size of subtree)` of parallel work. Real, and Amdahl-negligible for
any rewrite that does more than relink — but it is a refusal, not a zero-cost rewrite,
because the baseline form runs the extractions concurrently.

---

## 8. Cost table

Baselines. **R** = Rust `struct Node { op: Op, lit: i64, kids: Vec<Box<Node>> }` with
`Box::try_new`, the shape-for-shape equivalent. **C** = C++ `struct Node { Op op;
int64_t lit; Node* left; Node* right; }` with `unique_ptr`-equivalent ownership, the
fixed-arity fast shape. Δ is the x1 cost above the named baseline.

| Operation | Baseline form | x1 form | Δ vs R | Δ vs C |
|---|---|---|---|---|
| Descend one level | load ptr, deref | load `kids` handle, index, load Box, deref | 0 | +1 dependent load |
| Bounds on that index | R bounds-checks a `Vec` index identically | proved from the counted loop (§4a) — no compare | −1 compare where R's check is not elided | 0 (C has none either) |
| Bounds on a *data* index (§3.3, §6) | R: same runtime check | `if j >= len_of(n.kids)` (Rule 7's "one compare, no trap") | 0 | +1 dependent header load, +1 compare, +1 branch |
| Node memory | R: `Vec` header 24B + a separate block | `DynBox` block header + slots | 0 | +1 allocation per node, +header bytes per node |
| Full traversal of `n` nodes | `n` hops | `n` hops | 0 | +`n` dependent loads |
| Replace a child, discard the old | `kids[i] = fresh` | `n.kids[i] = move fresh` (Rule 6: "Assigning buf[k] = x where the old value is affine releases the old value") | 0 | 0 |
| In-place *shrinking* rewrite | `fold_in_place(&mut *kids[i])` | atomic update `n.kids[i] = f(n.kids[i])`, reusing the Box and `truncate(&(*x).kids, 0)` | 0 | 0 |
| Extract a child, order irrelevant | `kids.swap_remove(i)` | `swap_remove(&n.kids, i)` | 0 | +1 header store |
| **Extract a child, order preserved** | `mem::replace(&mut kids[i], fresh)`: 1 load, 1 store | `exchange`: 4 window ops | **+3 moves of 8B, +4 header stores (~+6 instructions)** | same |
| Transactional error path after extraction | `?` — the extracted value is dropped, nothing to restore | second `exchange` to put the old child back | **+4 window ops on the error path only** | same |
| Hand a subtree to a helper | `f(old_box)` — move an 8B pointer | `retire(move old, log)` — move an 8B Box | 0 | 0 |
| Dispose of a removed subtree | recursive `Drop` | Rule 8 recursive release | 0 | 0 |
| …with a bounded stack (G4) | R has the same unbounded-`Drop` problem; the careful form is the same worklist | `demolish` worklist | 0 vs careful R; +1 `Vector` allocation, +1 push, +1 pop per node vs naive R | same |
| Allocation | R `Box::try_new`: 1 branch | `Box::new(...)?`: 1 branch (Rule 14) | 0 | +1 predictable branch (C `new` throws) |
| **Split find-then-mutate, per pair** | find returns `&mut Box<Node>` / `Node**`; mutate is immediate | find writes `array<u64,64>`; mutate re-descends `d` levels | **+`d` dependent loads, +`d` compares, +`d` branches, +`d` stores** | same |
| Split find-then-mutate under Rule 16 | same | `find_id` → `u64`, then `p.nodes[id]` | +1 shift/multiply, +1 add, +1 bounds compare (hoistable) | same, plus the identity loss of C2 |
| Fused recursive rewrite (find+mutate) | one descent | one descent | 0 | 0 apart from the per-hop row above |
| Parallel payload rewrite of two subtrees | `rayon` over `split_at_mut` | admitted (§7 items 2–4) | 0 | 0 |
| **Parallel extracting edits on one node** | `split_at_mut` + `mem::replace` in two threads | refused (§7 item 6); extractions serialize | **lost: `O(1)` serial per rewrite** | same |

Two numbers dominate. The `d` dependent loads per find-then-mutate pair is the only
term that scales with tree depth, and it is exactly zero in the fused recursive form
and exactly zero under Rule 16. The `+1 dependent load per hop` against baseline **C**
is structural: it is the price of G1 — since an optional inline child is not nameable,
children must live in a window, and a window is a second allocation reached through a
second pointer. Against baseline **R**, which has the same shape, the whole table is
zero except the order-preserving extraction and the split find-then-mutate.

Proof annotations (`invariant a`, `invariant b`, the `ensures` relations, the `budget`
parameter) cost nothing at run time and are not counted, per the protocol.

---

## 9. Counterexamples

**C1 (performance, the strongest one). The zero-cost in-place rewrite form cannot be
used by any rewrite that allocates, and every interesting rewrite allocates.**

The candidate's one form that replaces a value in storage without moving it is the
atomic update, `buf[k] = f(buf[k])`, whose contract is "the old value goes into f by
value, f's result is committed, no program point lies between". It is the perfect tree
rewrite primitive: the old subtree arrives by ownership, `f` may take it apart and
return the new node, and the slot is written once. It is zero cost against C++ and
Rust — and for a *shrinking* rewrite (`Add(Lit a, Lit b)` → `Lit (a+b)`, reusing the
node's own Box and `truncate`ing its child block) it is fully usable.

A *growing* rewrite is not. Replacing `Var x` with the inlined body of `x` allocates,
Rule 14 makes every allocation fallible — "Allocation returns a `Result` and never
traps" — and the atomic update's `f` must return the slot's type, so the failure has
nowhere to go. Nor can the materials arrive: `f` is exhibited as unary, so it cannot
receive the body to inline, the environment to consult, or a destination for the
subtree it removes. The rewrite must therefore hoist the allocation above the rewrite
point — but the strongest program's decision of *what* to allocate is read off the node
it has not yet reached (`build_replacement(&(*old))`), so hoisting is not available
either. What remains is §3.1's four-operation exchange, plus a second four-operation
exchange on the error path. Against `mem::replace` that is `+3` eight-byte moves,
`+4` header stores and, on failure, `+4` more window operations, per rewrite. For a
compiler pass doing `R` rewrites this is `O(R)` extra L1-hot instructions — bounded, not
proportional to the tree — and if the pass is written in the split find-then-mutate form
the task explicitly asks for, it is `O(R·d)` extra dependent loads on top.

Note what this counterexample does *not* claim: it is not a deep copy, not an extra
allocation per rewrite, and not lost asymptotic behavior. The candidate's value
semantics (Rule 1's relocation consequence) keep every subtree transfer an eight-byte
Box move, which was the thing most at risk in a language with no references in
aggregates. The damage is a small constant on the rewrite itself.

**C2 (capability/safety). The rewrite that removes C1's depth term removes the
guarantee the owning form provides, and `PROGRAMS.md` names that guarantee as a
requirement.**

Rule 16's pool makes find-then-mutate `O(1)` (§6). It does so by making every edge a
`u64`. `u64` is Copy, so after `old = set_child(&p, parent, k, fresh)` the caller holds
an index to a subtree that is no longer attached, and nothing in the rule set prevents
that index from being installed somewhere else, kept past the slot's reuse, or silently
dropped. Rule 16 is explicit that this is intended: "A stale index that is still in
bounds names the current occupant of that slot: a logic error, not a memory error."
`PROGRAMS.md`'s mutable-compiler-graph row requires the opposite: "a relation to a
deleted node must not silently become a relation to a replacement node." Under §3's
owning form, x1 gives that requirement for free — Rule 3 invalidates the reference and
Rule 8 releases the subtree exactly once. Under §6's pool form, x1 gives it up.

The mitigation Rule 16 names — "Programs that need to detect it keep a generation
number as data" — costs one dependent load of `gen`, one compare and one branch at every
edge dereference, and, because there are no traps, an error arm threaded through every
traversal function's return type. That is a per-hop cost on the hottest operation in the
whole task, which is worse than C1's per-rewrite cost. So the candidate offers a real
dilemma for this task: the owning form is safe and pays `O(R·d)` on split
find-then-mutate; the pool form is fast and pays the identity guarantee; the guarded
pool form is safe and pays per hop.

**C3 (smaller, and it is a gap rather than a refusal). Cleanup has no depth bound.**
The program can bound its own recursion with a `budget` parameter and prove it at every
recursive call, as §3.2 does. The recursive release Rule 8 emits for an affine tree has
no budget, no contract and no trap. A degenerate tree — a chain of a million `Add`
nodes, which is exactly what a parser produces from a long left-associated expression —
is released by a compiler-emitted recursion whose depth is data. The program can avoid
its own recursion (§4k's `demolish`) but it cannot opt out of the compiler's, short of
never letting an affine tree go out of scope in one piece. Rust has the same defect and
answers it with an abort; x1 has promised there is no trap to abort with.

---

## 10. Rules consistent, and task achievable — recorded separately

**Rules consistent for the form actually used (§3, Shape 2): yes.** Every step is
covered by an exhibited clause, and the two sentences that could have collided at §4f —
Rule 11's invalidation and Rule 3's content-write exemption — point the same way. The
refusals in §4d and §7 items 6–8 are coherent applications of Rule 10.1 and Rule 13, not
accidents.

**Rules consistent for the form the task names literally (§5, Shape 1): not decided.**
G1 leaves an optional child unnameable; G2 leaves the one zero-cost rewrite primitive
without a stated scope; G3 is an outright contradiction between Rule 8's example and the
no-partial-move sentences of Rules 6 and 12.

**Task achievable: yes, with cost.** All five parts land:

- *traverse* — accepted, zero cost against the shape-for-shape Rust baseline;
- *replace a subtree* — accepted, zero cost when the old value is discarded or when the
  rewrite shrinks in place; `+3` moves and `+4` header stores when the old value must
  come out with sibling order intact;
- *hand a subtree to another function by ownership* — accepted, and the mechanism is
  clean: the window operation converts an interior path into a root, after which Rule
  10.2 discharges the transfer by root-distinctness (§4c). Zero cost beyond the
  extraction itself;
- *clean up removed parts* — accepted, zero cost, no destructor and no drop flag; with
  gap G4 on release depth and a `+1 allocation, +2 header stores per node` flattening
  rewrite if the depth must be bounded;
- *early-exit error path* — accepted; memory safety is free (Rule 8 on affine locals),
  and structural restoration costs a second four-operation exchange on the failure path
  only.

The split find-then-mutate form the task asks for is where the cost concentrates:
`+d` dependent loads, compares, branches and stores per find-mutate pair, because Rule 4
forbids returning the found reference and Rule 3 would have invalidated it across the
mutating call anyway. The recursive/fused equivalent removes that term entirely, and the
Rule 16 index-based equivalent removes it at the price of C2.

**Verdict: `undecided-rule-gap`** for the Box-linked shape the task names, because G1
and G2 decide whether the task is zero-cost or real-cost and I am not permitted to
resolve them. **`rejected-real-cost`** for the sub-question the gap-free rewrite
answers: the fixed-arity, inline-optional-child, `mem::replace`-and-`split_at_mut`
program is not writable, and the best available rewrite carries `+1` dependent load per
hop and `+1` allocation per node against the C baseline, `+3` moves and `+4` header
stores per order-preserving extraction against both baselines, `+d` loads per split
find-mutate pair, and the lost parallel extraction of §7 item 6. **`accepted-fine`** for
traversal, discard-replacement, in-place shrinking rewrite, ownership handoff, cleanup,
and the parallel forms of §7 items 1–5.

---

## 11. Rule gaps, quoted

**G1 — no path step into a variant payload.**
> "A path starts at a local variable or a parameter and continues through fields, `*`
> (Box content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7)." (Rule 2)

and, for effect rows,

> "each path starts at a reference parameter and may continue through fields, `*`, and
> whole-index or range positions supplied as arguments" (Rule 9)

Missing: whether an enum variant's payload is reachable as a path, and if so by what
step. Without it, `Option<Box<Node>>` children are not traversable by reference,
although Rule 12 recommends `Option` with a null niche as a storage representation.

**G2 — scope and shape of the atomic update.**
> "buf[k] = f(buf[k])                   // atomic in-place update: the old value goes
> into f by value, f's result is committed, no program point lies between; requires k <
> len_of(buf)" (Rule 6)

Missing: whether the form applies to a non-index path such as a struct field; whether
`f` may take parameters other than the old value; whether `f` may return a `Result`.
All three decide C1.

**G3 — partial move out of an owned local.**
> "There is no `take` operation and no partial move out of any place: the only ways to
> move a value out of storage are consuming a whole local (`move x`), the window
> operations, and the atomic update." (Rule 6)
> "No place is ever partially moved: every path is either wholly present or the program
> cannot name it." (Rule 12)
> "fn use_it(c: own Conn) { ...; close(move c.f) }   // consuming the struct as a whole
> and closing its File is required" (Rule 8)

Missing: a ranking. The by-value recursive rewrite `fn rewrite(n: own Node) -> own Node`
is admitted under the third sentence and rejected under the first two.

**G4 — depth of recursion and of compiler-emitted release.**
> "Recursion is checked through contracts, never by unfolding bodies." (Rule 10)
> "at scope exit the compiler releases their memory recursively (Box, DynBox) and runs
> no user code" (Rule 8)

Missing: any statement about stack depth. This is silence rather than a contradiction,
but it is load-bearing for this task because the tree's depth is data and the candidate
promises no trap.

**G5 — index expressions inside a contract.**
> "Signatures never contain index expressions; an index enters an effect only through an
> argument, evaluated once at the call." (Rule 9)

Missing: whether the first clause's "signatures" includes the `contract` block or only
the effect row. It decides whether a precondition about a nested window's length
(`requires k < len_of(p.nodes[parent].kids)`) is writable, and therefore whether §6's
edge write pays a runtime load, compare, branch and error arm.

---

## 12. Dependence on Rule 16

**Sections 1–5 and 7–9's C1: no dependence.** They use only Rules 1–15.

**Section 6 and counterexample C2: full dependence.** The index-based equivalent the
task asks for is exactly Rule 16's pool-as-usage, which the rule file marks "provisional,
pending owner ruling" and which "reverses the current design tree's rejection of the
arena-index-pool pattern". If Rule 16 is not adopted, this task loses its only
constant-time split find-then-mutate form and the `+d` dependent loads per find-mutate
pair become unavoidable for that style; the fused recursive form is unaffected either
way. If Rule 16 is adopted, this task is the place where its stated cost — "A stale
index that is still in bounds names the current occupant of that slot: a logic error,
not a memory error" — collides with `PROGRAMS.md`'s requirement that "a relation to a
deleted node must not silently become a relation to a replacement node".
