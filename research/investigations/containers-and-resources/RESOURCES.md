# Resources: providers, the envelope, and resource-closed programs

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> second draft, after falsifier round 1; this file has been brought to that draft
> and carries no rule text of its own.

The resource half of the batch 0116 drafting round, reduced to the material
`DESIGN.md` does not carry: the goals and non-goals it was written against, and
the three writer-facing walkthroughs. The laws, the rules, the amendment table,
the startup protocol, the worked programs, the open questions and the whole
verified-versus-reasoned register moved; the map below says where.

Tree read: `batch/0116-containers-and-resources` at `main` a40c7e70, spec **v0.40
ACTIVE**. Bare four-digit line numbers are `spec/kernel-spec.md` at that tip.
Nothing here is implemented.

Provenance: the owner's four rulings of 2026-09-01 (heap as an explicit capability
value handed to `main`; `resource-closed` as a compiler-derived,
writer-requirable property with one language and no dialects; no recursion that
accumulates frames and no depth certificates; typed failures with a runtime that
acquires everything before `main` and never allocates after) are decisions, not
proposals. The supporting analysis those rulings were made against is extracted
beside this file in `EVIDENCE-owner-discussion-2026-08-31.md`.

## What moved to `DESIGN.md`

```text
| this file's section                  | where it now lives                       |
|--------------------------------------|------------------------------------------|
| 2. The laws (L1-L8)                  | DESIGN.md section 2, as L1-L9            |
| 3.1 [PROV]                           | DESIGN.md section 3.2                    |
| 3.2 [RES]                            | DESIGN.md section 3.3                    |
| 3.3 [STK]                            | DESIGN.md section 3.4                    |
| 3.4 [RUN]                            | DESIGN.md section 3.5                    |
| 3.5 amendment table                  | DESIGN.md section 3.13, merged           |
| 3.6 the pool seam                    | DESIGN.md section 3.11                   |
| 4. How E is computed                 | DESIGN.md sections 3.3.1 and 3.3.2       |
| 5. The startup protocol              | DESIGN.md [RUN-5]                        |
| 7. Two worked programs               | DESIGN.md section 4                      |
| 8. Open questions                    | DESIGN.md section 5, merged and renumbered|
| the verified/reasoned register       | DESIGN.md section 6, re-run and extended |
```

## What round 2 changed, so this file is not read as current

`DESIGN.md`'s first draft was falsified in four passes, and six of the resource
half's decisions did not survive. The walkthroughs below are written against the
surviving design; if one contradicts `DESIGN.md`, `DESIGN.md` wins.

- **The judgment is two-stage.** `source-resource-closed` is a source judgment over
  program text alone — heap-freedom, acyclicity, the per-domain symbolic
  summaries, no reentrancy. The concrete envelope is a target-stage
  materialization whose failure is a qualification failure citing no language
  rule. The first draft made a source rejection depend on the register allocator,
  the optimizer, and the linked runtime.
- **`E` is a table over the lane counts the target supports, not a single list**,
  because parallel permission is never an obligation and `W = 1` must always be
  legal; and `PreStart` may descend the table rather than reporting a start
  failure at the largest row.
- **A tail edge is one whose caller frame is dead**, not one written in a return
  statement. The syntactic conditions admitted a mutual tail recursion carrying a
  live borrow of a caller local, which compiles today (probes `f2b`,
  `f8_tailframe`) and whose rewrite would alias or unbound the frame.
- **A reserved extent lives in the reserving activation's frame**, one per
  activation. Static placement plus a framed-out row made one extent invisible to
  [PAR-1] and shared by every overlapped activation.
- **Saturation is unreachable, not answered.** Every runtime store is a covered
  store with a published capacity, and the program's peak demand on it is composed
  by the same algebra as a pool's. The first draft licensed inline execution,
  which nests a chain no stack term counts, and waiting, which is a hang.
- **Every provider-owned release names its provider.** A heap free and a pool
  release carry comparable footprints; the first draft gave the first an empty row
  and the second a `writes(pool)` row, which made two concurrent frees invisible to
  [PAR-1] and left nothing ordering a `Heap`'s death after its allocations'.

Two smaller corrections: the store-state vocabulary `live`, `capacity` and
`remaining` is retired into the one measure algebra `len`, `cap`, `room`, so a
provider and a container are read with the same three terms; and `L8` is split, so
that a store's own refusal is an ordinary fact on the `Err` edge and the checked
spelling of an acquisition is worth having.

Vocabulary note: the walkthroughs below were written before the two drafts were
unified and spelled the container operations `fixed_vector_new`,
`fixed_vector_push` and `heap_vector_new`. They now use `DESIGN.md` section 3.12's
chosen spellings.

## 1. Goals and non-goals

**Goals.** Turn every resource a Whitefoot program can exhaust into a named value
it must hold in order to consume, so that "this subtree never touches the heap"
and "this program's peak demand is this list of regions and slot counts" are facts
a signature and a compiler judgment carry rather than facts a reviewer hopes are
true; give the writer one declaration that turns the second fact into a
compilation requirement, so a program intended for a bounded machine fails at
compile time rather than at three in the morning; make every way of failing to get
a resource a typed value that returns the affine inputs it did not consume, so no
reachable path in an accepted program is a trap, an abort, or a silent promotion
to a bigger store; and put the compiler-derived cleanup, the `par` runtime, and
the target adapter *inside* the same envelope as the writer's code, because a
guarantee that stops at the edge of generated code is not a guarantee.

**Non-goals.** This design does not promise that a program terminates, that it
meets a deadline, that it gets CPU time, that a file exists, that a disk has
space, that a network answers, or that a host does not kill it; it does not bound
how many times a program acts, only what it holds at once and what it consumes
irreversibly; it does not make a general-purpose heap safe, and it deliberately
refuses to give a bounded general heap the resource-closed label, because total
free bytes do not answer a request for a contiguous aligned extent; it does not
add a depth certificate, a resource solver, a search for allocator placements, or
any acceptance path with a budget or a timeout; it does not define the container
operations that consume providers, which are `CONTAINERS.md`'s; and it does not
attempt the `par` continuation-frame redesign that `DESIGN.md` section 5's Q11
shows is the real obstacle to a resource-closed program that uses parallelism.

## 2. The writer's view

The declaration is one marker on the entry:

```wf-design
resource_closed command fn main() -> status: own ExitStatus pure {
```

Everything else the writer sees is a diagnostic. Each of the three walkthroughs
below is the same shape: a writer who did not know about any of this writes the
obvious program, gets one error that names the offending structure, and makes one
structural change. Every rule id cited exists in `DESIGN.md` section 3.

### 2.1 From "it needs a growable vector" to a fixed store

```wf-design
resource_closed command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  doc "Collects every decoded record.";
  let records = seq_heap<Record>();
  ...
}
```

```text
error [RES-5]: this program cannot be resource-closed: it reaches the general heap
  --> records.wf:1:33
   |
 1 | resource_closed command fn main(command.heap as heap: own Heap) ...
   |                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the Heap enters here
   |
   = heap-reaching path:
       main              records.wf:1:1
         seq_reserve     records.wf:9:19
   = a general store cannot appear in an envelope: total free bytes do not decide
     whether the next contiguous aligned request has a home
   = restructure: give the store a capacity the compiler can check --
     FixedVector<Record, N> over frame storage, or PoolVector over a
     Pool<'p, Record, N> reserved by pool_static; or drop the resource_closed
     marker and handle OutOfMemory as a value
```

The fix is a capacity decision, which is the decision the writer was avoiding:

```wf-design
resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Collects up to 512 decoded records.";
  let records = seq_fixed<Record, 512>();
  ...
}
```

Note the path in the diagnostic runs from `main` to the operation, not to
`buffer_new`. The `Heap` is reached through a parameter, and [PROV-4] roots the
reaching relation in the **selected type at the leaf** of each `allocates` path,
so a provider carried inside an aggregate is still found. The first draft rooted
it in the formal's own type, which a struct field defeats — verified today by
probe `f5b`, where an opaque affine capability travels in a struct field and both
rows stay silent about it.

### 2.2 From recursive descent to an explicit work list

```wf-design
fn parse_node['a](input: &'a Span<'a, u8>, at: own u64) -> result: own Node pure {
  doc "Parses one node, recursing into its children.";
  ...
  let child = parse_node<'a>(input: input, at: next);
  ...
}
```

```text
error [STK-2]: this program cannot be resource-closed: its call graph has a cycle
  --> parse.wf:1:1
   |
 1 | fn parse_node['a](input: &'a Span<'a, u8>, at: own u64) -> result: own Node pure {
   | ^^^^^^^^^^^^^^^^^ this function is in the cycle
   |
   = cycle, in call order:
       parse_node   parse.wf:5:15  -> parse_node
   = the call at parse.wf:5:15 is not a tail edge: this frame is not dead at the
     jump -- its result is used at parse.wf:6:3 -- so [STK-1] cannot rewrite the
     component into a loop
   = a recursion depth bound is not accepted here: this version admits no depth
     certificate, so a cycle has no finite stack envelope
   = restructure: carry the pending nodes in an explicit FixedVector<Frame, N>
     work list and loop, or make every recursive call a tail edge whose caller
     frame is dead at the jump
```

The last line is a round-2 change and it is not cosmetic. "Make every recursive
call the complete return expression" is advice that a writer can follow and still
be refused, because a member that borrows its own local and passes the borrow
forward keeps its frame live across the jump; probes `f2b` and `f8_tailframe`
compile today and are exactly that shape. Naming the real premise is what lets the
diagnostic be acted on.

The fix turns the implicit stack into a declared one, and the declared one is an
envelope item the writer chose:

```wf-design
fn parse['a](input: &'a Span<'a, u8>) -> result: own Result<Node, TooDeep> pure {
  doc "Parses the whole input with an explicit pending-node stack.";
  let pending = seq_fixed<Frame, 64>();
  loop @walk {
    ...
  }
}
```

### 2.3 From an unbounded store to a bounded one

```wf-design
  for @fill (i in 0_u64..count) {
    let blank_bytes = array_new<u8, 4096>(0_u8);
    let blank = Page(bytes: blank_bytes);
    let page = pool_take<'p, 'c>(pool: &uniq 'c pages, value: move blank);
    set kept = seq_place(vector: move kept, value: move page);
  }
```

```text
error [RES-6]: this loop's demand on 'p has no finite bound
  --> pager.wf:12:16
   |
12 |     let page = pool_take<'p, 'c>(pool: &uniq 'c pages, value: move blank);
   |                ^^^^^^^^^ acquires one slot of Pool<'p, Page, 64>
13 |     set kept = seq_place(vector: move kept, value: move page);
   |                ^^^^^^^^^ retains it past the backedge
   |
   = per iteration, on the fallthrough exit: peak 1, delta +1 on 'p
   = the loop runs 0..count, and count is a runtime value with no upper bound in
     this ProofContext, so the peak is unbounded
   = supply one of:
       a requires on count (`requires ile(count, 64_u64);`)
       a structural cutoff in the loop (`let n = len(kept); let full =
         ieq(n, 64_u64); if full { break @fill; }`)
       an invariant relating the two stores
         (`invariant slots_match: ile(len(pages), len(kept))`)
       or the checked spelling, and handle PoolExhausted
```

The proved spelling and the checked spelling are both accepted, and both keep the
program resource-closed — this is [RES-8], and it is what the writer picks between:

```wf-design
  for @fill (
    i in 0_u64..count,
    invariant kept_fits: ile(len(kept), 64_u64)
  ) {
    let spare = room(kept);
    let more = igt(spare, 0_u64);
    if more {
      let blank_bytes = array_new<u8, 4096>(0_u8);
      let blank = Page(bytes: blank_bytes);
      let page = pool_take<'p, 'c>(pool: &uniq 'c pages, value: move blank);
      set kept = seq_place(vector: move kept, value: move page);
    } else {
      break @fill;
    }
  }
```

or, when the writer would rather answer the refusal than prove it away:

```wf-design
    let blank_bytes = array_new<u8, 4096>(0_u8);
    let blank = Page(bytes: blank_bytes);
    let attempt = pool_take_checked<'p, 'c>(pool: &uniq 'c pages, value: move blank);
    match attempt {
      Ok(value: page) => {
        set kept = seq_place(vector: move kept, value: move page);
      }
      Err(error: refused) => {
        let recovered = move refused.rejected;
        break @fill;
      }
    }
```

The fourth repair is the one the first draft offered and could not deliver, twice
over. Its result type `Result<slot<'p, Page>, PoolExhausted<Page>>` was rejected by
[FN-2] as a region-bearing generic argument and by [STOR-5] as a region-bearing
enum payload, so no checked acquisition anywhere in the design was a program;
`DESIGN.md` [CNT-6] admits it, because the instance's own type names `'p` and the
instance is therefore itself confined. And under the draft's L8 the `Err` arm was
unreachable in the abstract demand semantics, so the `break` never executed and
the loop's backedge delta stayed `+1`; the split L8 lets the judgment read the
store's own `room(pages) = Z` on that edge, which is a fact about the store rather
than a claim about the program's survival.

The proved repair also depends on `room` being readable. Under the first draft's
L15 it was not, so a dominating branch could bound `len(kept)` and never reach a
goal over the pool's spare, and the escape hatch the diagnostic names was
unwritable in both of its forms.

## 3. Evidence

The verified-versus-reasoned register that stood here has moved to `DESIGN.md`
section 6, where every probe was re-run and eleven more were added. Three of the
new ones bear directly on this half: `r1_relend` and `r1_relend_affine` show that
a helper holding `&uniq 'b P` cannot lend it onward at all today, which is why
`DESIGN.md` [PROV-9] exists and without which no function but `main` could
allocate; and `r1_ambient` re-confirms that the heap is ambient today, which is the
single fact this half exists to change.

The `--stack-ledger` read of `tests/programs/recursive_tree.wf` is worth keeping
here in one sentence, because three rules rest on it: it reports `main` and the
entry body as two **disjoint roots** rather than one chain, and the entry chain
runs through the compiler's drop glue into `wf_resource_abort`. That is why
[STK-3] quantifies over a context's whole chain in both directions rather than
over the writer's call graph from `main`, why [STK-4] must give the guard-page
handler's alternate stack an item of its own, and why [RES-7]'s claim that the
abort site has a reachable caller today is a reading of emitted code rather than
an assumption.

Nothing in this file is verified. Every program above is design text and compiles
nowhere.
