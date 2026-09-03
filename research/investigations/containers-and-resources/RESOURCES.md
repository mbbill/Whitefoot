# Resources: providers, the envelope, and resource-closed programs

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first.

The resource half of the batch 0116 drafting round, reduced to the material
`DESIGN.md` does not carry: the goals and non-goals it was written against, the
three writer-facing walkthroughs, and this file's own verified-versus-reasoned
register. The laws, the rules, the amendment table, the startup protocol, the
worked programs and the open questions all moved; the map below says where.

Tree read: `batch/0116-containers-and-resources` at `main` a40c7e70, spec
**v0.40 ACTIVE**. Bare four-digit line numbers are `spec/kernel-spec.md` at that
tip; every other citation names its file. Nothing here is implemented.

Provenance: the owner's four rulings of 2026-09-01 (heap as an explicit
capability value handed to `main`; `resource-closed` as a compiler-derived,
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
| 3.1 [PROV]                           | DESIGN.md section 3.1                    |
| 3.2 [RES]                            | DESIGN.md section 3.2                    |
| 3.3 [STK]                            | DESIGN.md section 3.3                    |
| 3.4 [RUN]                            | DESIGN.md section 3.4                    |
| 3.5 amendment table                  | DESIGN.md section 3.12, merged           |
| 3.6 the pool seam                    | DESIGN.md section 3.10                   |
| 4. How E is computed                 | DESIGN.md sections 3.2.1 and 3.2.2       |
| 5. The startup protocol              | DESIGN.md [RUN-5]                        |
| 7. Two worked programs               | DESIGN.md section 4                      |
| 8. Open questions                    | DESIGN.md section 5, merged and renumbered|
```

Vocabulary note: the walkthroughs kept below were written before the two drafts
were unified, so they spell the container operations `fixed_vector_new`,
`fixed_vector_push` and `heap_vector_new`. `DESIGN.md` section 3.11 records the
chosen spellings (`seq_fixed`, `seq_place`, `seq_heap`) and why. The diagnostics
and the reasoning are unaffected.

## 1. Goals and non-goals

**Goals.** Turn every resource a Whitefoot program can exhaust into a named value it
must hold in order to consume, so that "this subtree never touches the heap" and
"this program's peak demand is this list of regions and slot counts" are facts a
signature and a compiler judgment carry rather than facts a reviewer hopes are
true; give the writer one declaration that turns the second fact into a
compilation requirement, so a program intended for a bounded machine fails at
compile time rather than at three in the morning; make every way of failing to
get a resource a typed value that returns the affine inputs it did not consume,
so no reachable path in an accepted program is a trap, an abort, or a silent
promotion to a bigger store; and put the compiler-derived cleanup, the `par`
runtime, and the target adapter *inside* the same envelope as the writer's code,
because a guarantee that stops at the edge of generated code is not a guarantee.

**Non-goals.** This design does not promise that a program terminates, that it
meets a deadline, that it gets CPU time, that a file exists, that a disk has
space, that a network answers, or that a host does not kill it; it does not
bound how many times a program acts, only what it holds at once and what it
consumes irreversibly; it does not make a general-purpose heap safe, and it
deliberately refuses to give a bounded general heap the resource-closed label,
because total free bytes do not answer a request for a contiguous aligned
extent; it does not add a depth certificate, a resource solver, a search for
allocator placements, or any acceptance path with a budget or a timeout; it does
not define the container operations that consume providers, which are
`CONTAINERS.md`'s; and it does not attempt the `par` continuation-frame redesign
that `DESIGN.md` section 5's Q14 shows is the real obstacle to a resource-closed program that
uses parallelism.


## 2. The writer's view

The declaration is one marker on the entry:

```wf-draft
resource_closed command fn main() -> status: own ExitStatus pure {
```

Everything else the writer sees is a diagnostic. Each of the three walkthroughs
below is the same shape: a writer who did not know about any of this writes the
obvious program, gets one error that names the offending structure, and makes one
structural change.

### 2.1 From "it needs a growable vector" to a fixed store

```wf-draft
resource_closed command fn main(command.heap as heap: own Heap) -> status: own ExitStatus allocates(heap) {
  doc "Collects every decoded record.";
  let records = heap_vector_new<Record>(heap: &uniq 'h heap);
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
       main                      records.wf:1:1
         heap_vector_new<Record> records.wf:3:17
           buffer_new            (operation table)
   = a general store cannot appear in an envelope: total free bytes do not decide
     whether the next contiguous aligned request has a home
   = restructure: give the store a capacity the compiler can check --
     FixedVector<Record, N> over static or frame storage, or PoolVector over a
     Pool<'p, Record, N> reserved by pool_static; or drop the resource_closed
     marker and handle OutOfMemory as a value
```

The fix is a capacity decision, which is the decision the writer was avoiding:

```wf-draft
resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Collects up to 512 decoded records.";
  let records = fixed_vector_new<Record, 512>();
  ...
}
```

### 2.2 From recursive descent to an explicit work list

```wf-draft
fn parse_node['a](input: &'a Span<u8>, at: own u64) -> result: own Node pure {
  doc "Parses one node, recursing into its children.";
  ...
  let child = parse_node(input: input, at: next);
  ...
}
```

```text
error [STK-2]: this program cannot be resource-closed: its call graph has a cycle
  --> parse.wf:1:1
   |
 1 | fn parse_node['a](input: &'a Span<u8>, at: own u64) -> result: own Node pure {
   | ^^^^^^^^^^^^^^^^^ this function is in the cycle
   |
   = cycle, in call order:
       parse_node   parse.wf:5:15  -> parse_node
   = the call at parse.wf:5:15 is not in tail position: its result is used at
     parse.wf:6:3, so [STK-1] cannot rewrite the component into a loop
   = a recursion depth bound is not accepted here: this version admits no depth
     certificate, so a cycle has no finite stack envelope
   = restructure: carry the pending nodes in an explicit FixedVector<Frame, N>
     work list and loop, or make every recursive call the complete return
     expression
```

The fix turns the implicit stack into a declared one, and the declared one is an
envelope item the writer chose:

```wf-draft
fn parse['a](input: &'a Span<u8>) -> result: own Result<Node, TooDeep> pure {
  doc "Parses the whole input with an explicit pending-node stack.";
  let pending = fixed_vector_new<Frame, 64>();
  loop @walk {
    ...
  }
}
```

### 2.3 From an unbounded store to a bounded one

```wf-draft
  for @fill (i in 0_u64..count) {
    let blank = Page(bytes: array_new<u8, 4096>(0_u8));
    let page = pool_take(pool: &uniq 'p pages, value: move blank);
    let stored = fixed_vector_push(vector: &uniq 'v kept, value: move page);
  }
```

```text
error [RES-6]: this loop's demand on 'p has no finite bound
  --> pager.wf:12:5
   |
12 |     let page = pool_take(pool: &uniq 'p pages, value: blank);
   |                ^^^^^^^^^ acquires one slot of Pool<'p, Page, 64>
13 |     let stored = fixed_vector_push(vector: &uniq 'v kept, value: move page);
   |                  ^^^^^^^^^^^^^^^^^ retains it past the backedge
   |
   = per iteration: peak 1, delta +1 on 'p
   = the loop runs 0..count, and count is a runtime value with no upper bound in
     this ProofContext, so the peak is unbounded
   = supply one of:
       a requires on count (`requires ile(count, 64_u64);`)
       a structural cutoff in the loop (`if ieq(len(kept), 64_u64) { break @fill; }`)
       an invariant relating the two stores
         (`invariant slots_match_vector: ieq(live(pages), len(kept))`)
       or the checked spelling, and handle PoolExhausted
```

The proved spelling and the checked spelling are both accepted, and both keep
the program resource-closed — this is [RES-8], and it is what the writer picks
between:

```wf-draft
  for @fill (
    i in 0_u64..count,
    invariant kept_fits: ile(len(kept), 64_u64)
  ) {
    let room = ilt(len(kept), 64_u64);
    if room {
      let blank = Page(bytes: array_new<u8, 4096>(0_u8));
      let page = pool_take(pool: &uniq 'p pages, value: move blank);
      let stored = fixed_vector_push(vector: &uniq 'v kept, value: move page);
    } else {
      break @fill;
    }
  }
```

or, when the writer would rather answer the refusal than prove it away:

```wf-draft
    let blank = Page(bytes: array_new<u8, 4096>(0_u8));
    let attempt = pool_take_checked(pool: &uniq 'p pages, value: move blank);
    match attempt {
      Err(value: refused) => {
        let recovered = move refused.rejected;
        break @fill;
      }
      Ok(value: page) => {
        let stored = fixed_vector_push(vector: &uniq 'v kept, value: move page);
      }
    }
```


## 3. What I verified and what I reasoned

This register is kept verbatim as the draft's own evidence. Its cross-references
(sections 2, 3, 3.5, 4.2, 7.1, 7.2, 8, and the law numbers L1 to L8) name the
sections this file carried before `DESIGN.md` superseded them; the map in
"What moved to `DESIGN.md`" translates each one, and the laws L1 to L8 here are
L1 to L9 there. `DESIGN.md` section 6 re-ran every probe below against the gate
compiler and records the verdicts obtained in that session.

**Compiled.** I built the gate-profile `whitefootc` from this tree and ran five
probes. They establish current behaviour, not this design; no program in this
file compiles, because none of its syntax exists.

| probe | program | result | what it establishes |
|---|---|---|---|
| `p1_noinput.wf` | a `command` entry selecting no standard input | **accepted** | 7.1's entry shape is legal today |
| `p2_forever.wf` | an entry whose only statement is a `loop` with no `break` | **rejected**, [FN-1] `FunctionFallthrough` | an unbounded service loop must still leave the function on some edge; 7.1's scheduler loop therefore breaks on an empty queue. This corrected the program I first wrote |
| `p3_rec.wf` | an ordinary self-recursive function called from `main` | **accepted** | recursion is permitted today [FN-6]; [STK-2] is a new restriction on resource-closed programs and retires nothing |
| `p4_undeclared.wf` | a body that allocates while declaring `pure` | **rejected**, [EFF-2] `EffectMismatch`, missing `allocates(heap)` | allocation is already exhibited-and-checked both ways; [PROV-4] changes what the entry names, not whether it is checked |
| `p5_ambient.wf` | a nullary leaf function that allocates a buffer while holding nothing | **accepted** | the heap is ambient today. This is L2's evidence and the single fact the capability half of this design exists to change |
| `p6_unproved.wf` | `buffer_new` on an unbounded runtime length | **rejected** at target layout, `Unrepresentable(RuntimeSizedAllocation)` | allocation *size* is already a static obligation while availability is not — the split 7.2 relies on |

I also ran the existing `--stack-ledger` on `tests/programs/recursive_tree.wf`.
It reports, from the post-codegen assembly of that same compilation, a per-frame
table, a `STACK cycle` row for the recursive function with its bytes-per-level and
the number of levels the runtime's stack holds, and a `STACK chain` row per root —
and one of those chains runs through the compiler's drop glue into
`wf_resource_abort`. Three things in this design are that output's direct
consequences: [STK-3]'s insistence that frames are measured after code generation
(the ledger's own header explains that a pre-codegen number reports zero bytes for
the function a program dies in), [STK-2]'s cycle diagnostic (the ledger already
finds and prints the cycle), and [RES-7]'s claim that the abort site has a
reachable caller today.

**Read, not run.** Every rule citation in the amendment table was read at the line given
in `spec/kernel-spec.md` at a40c7e70: [SCOPE-3] 27-31, [TYPE-2] 352, [SET-2] 508,
[OWN-1] 558, [OWN-5] 580, [STOR-1] 670, [STOR-3] 683, [STOR-5] 718, [STOR-6]
733-761, [OP-1] 793-798, [OP-9] 968-994, [FN-6] 1205, [FN-7] 1210-1253, [EFF-1]
1363-1372, [EFF-2] 1386-1420, [EFF-5] 1444-1450, [PAR-1] 1965-1989, [PAR-2] 2024,
[PAR-3] 2049, [PROG-3] 1499-1509, [SYS-2] 2158-2280, [SYS-8] 2482.

**Reasoned, and not verified anywhere.**

- Every judgment in section 3. None is implemented, and none has been executed
  against a program.
- The composition algebra of 4.2. Its sequence and branch rules are standard and
  I believe them; its `par` rule depends on a runtime profile that does not exist
  yet, and its loop rule's claim that a zero-delta backedge needs no iteration
  bound is the one I would attack first if I were falsifying this file.
- [PROV-7]. The release-site reachability premise is stated, not derived. It is
  the known hole.
- Every byte figure in section 7. They are illustrative; nothing computed them.
- The claim that [STK-1]'s admission conditions are sufficient for a correct
  mutual-tail-recursion rewrite. They are the conditions I could name; I did not
  attempt a proof and I did not enumerate the shapes they reject.
- The claim in [PROV-2] that `Heap` uniqueness is enough to keep `buffer<T>`
  non-region-bearing and its release row empty. This is load-bearing for keeping
  [STOR-1], [STOR-3] and [STOR-5] almost unchanged, and it deserves a falsifier
  of its own: a program that holds two heap-backed buffers across a boundary where
  the `Heap` has been moved.
- Everything about the current runtime's closure. The gaps named in section 2's
  L4 paragraph come from the 2026-09-01 read of the backend sources, not from a
  fresh audit in this session, and [RUN-2] is written as an obligation precisely
  because I cannot certify any existing target meets it.

**Falsifiers this file is asking for, in the order I would run them.**

1. Rewrite one existing corpus program that uses `buffer<T>` against [PROV-3] and
   [RES-7] by hand, and count what the `Result` return costs at every call site.
   If the answer is "every function that touches a buffer grows an error route",
   the operation split of [RES-8] is wrong somewhere.
2. Hand-execute 4.2 on 7.1 and on a program whose loop retains conditionally, and
   check that the branch rule's per-variant retention survives a `propagate` edge.
3. Attack [PROV-2]'s uniqueness argument with the two-buffer program above.
4. Attack [STK-1] with a mutual tail recursion carrying a live child reborrow
   across the jump, and check that the stated conditions reject it.
5. Attack L8 with a fixed append-only log: confirm that the design counts its
   records as a consumable budget and not as an effect flow, and that a program
   which writes to it in an unbounded loop is correctly *not* resource-closed.
