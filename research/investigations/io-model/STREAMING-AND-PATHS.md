# Streaming chunk loops, and bytes to path

One design for the two capabilities the third blind writer's programs blocked
on. It is a synthesis: three independent designs were written against
`integration/2026-08-28b` at `16228216` — `proof-first`, `runtime-first`,
`writer-first` — and reviewed by two independent judges. This file is the
single design the project will implement. Every idea in it is attributed in
§C.1, and every idea that was proposed and rejected is in §C.2 with the
concrete program or schedule that killed it.

**Base revision.** Every `spec/kernel-spec.md`, `compiler/`, `tests/`, `docs/`
and `research/` line number below is `16228216`'s, the revision the three source
designs and both reviews were written against. Read the spec through
`git show 16228216:spec/kernel-spec.md` if the working tree has moved.

**Contents.** §0 the two problems and the target inventory. **Part A** — A.1 the
program and the three walls, A.2 the decision, A.3 the writer-facing example, A.4
the specification delta, A.5 the judgment, A.6 the lowering, A.7 the ledger, A.8
safety and the attacks, A.9 oracle and falsifier, A.10 what the writer writes
differently, A.11 the batch plan, A.12 open questions. **Part B** — same order,
B.1 through B.11. **§C** provenance: what survived and from which design, and what
was rejected with the counterexample that rejected it. **§D** the fifteen-loop
scoreboard and what neither part fixes. **§E** the two new `docs/patterns.md`
entries.

**Status.** Design record. No approval is asserted, no batch is claimed, and
nothing here is merged. §A.11 and §B.10 are the batch plan; §A.12 and §B.11 are
the decisions the owner still owns, each with a recommendation.

---

## 0. The two problems, and the one object they share

**Problem A.** A loop that reads one file in chunks, folds each chunk, and
discards it stays sequential. [PAR-3] cuts the body at the first may-suspend
submission (`spec/kernel-spec.md:2056`) and requires every edge leaving the body
to occur in the prologue (`:2057`); but the `ReadEnd` break is selected by the
read's own outcome, which `:2058` puts in the remainder, and the next read's
offset depends on this read's result, which none of `:2060`'s three dispositions
covers. The compiler says so in as many words
(`compiler/src/semantic/staged_permission.rs:365-366`): "one file's chunk loop
stays sequential". Big files must stream fast; that sentence is the wall.

**Problem B.** No operation turns bytes a program read into something an open
can resolve as a path. A `directory_next` record's name is one path component
(`spec/kernel-spec.md:2676`), `open_file` and `open_directory` validate their
caller-owned range as one component (`:2683`), and `spec/kernel-spec.md:2680`
declares no route from read bytes to a `RelativePath`. So the third writer's
"open every file named in a list file" tool cannot open `sample/a.txt`
(`writer3/sizes.wf:117`) and its walker descends component by component over an
affine `buffer_vacant<DirectoryRead>` stack (`writer3/largest.wf:55,78,91,198`).

**The shared object.** Both problems are one missing [PAR-3] disposition: a
place rooted outside the loop that the *prologue reads* and the *remainder
writes*. Part A's chunk cursor is that place with the read's own payload in its
recurrence; Part B's line cursor is that place without one. They are one
disposition with two proof grades (§A.5.2), and the cheap grade is the one that
makes Part B's list loop stage at all. That is why the two parts are one design.

## 0.1 What the target can already do, at what price

This table is the discipline the whole of Part A is derived from. Nothing in it
is proposed; all of it is in the tree at `16228216`. It comes from
`runtime-first` §A.0, which is the sharpest engineering page in the three source
designs, and it settles one question by itself.

| capability | where | marginal cost |
|---|---|---|
| positioned read that moves no cursor | `pread` / `IORING_OP_READ`; `spec/kernel-spec.md:2626` | one SQE |
| K operations continuously outstanding | `WF_BRIDGE_OPERATION_CAPACITY`, `WF_BRIDGE_SLOT_COUNT` = 64 (`bridge.c:40-41`); the Linux ring is sized to 64 | none; already sized |
| non-blocking harvest of one token | `wf__completion_file_take` (`bridge.h:144`) | one CQE peek |
| blocking join of one token | `wf__completion_file_join` (`bridge.h:129`) | one wait |
| **discard a harvested result** | drop the slot; nothing to undo | **zero** |
| depth query, once per loop entry | `wf__completion_window(span, slot_bytes, ceiling)` (`LOOP-PIPELINE.md:690`), modelled on `wf__par_split_budget` (`par_runtime.c:943`) | one call per loop entry |
| K private destination slots, one allocation at loop entry | `LOOP-PIPELINE.md:721-741` | one heap allocation |
| hand a pure per-iteration fold to another lane | `wf__par_claim` / `wf__par_publish` / `wf__par_join` (`emitter/parallel.rs:106,430,443`) | one claim |
| whole path passed to `openat` | `file_adapter.c:172-190`; the path is staged per operation record at `:611` | zero; already there |
| cancel an in-flight read | `IORING_OP_ASYNC_CANCEL` on Linux; **nothing on Darwin** — a helper blocked in `pread` cannot be interrupted (`FIRST-PRINCIPLES.md:944-948`) | an SQE, a CQE, and a race |

Read the last two rows of the read half together. **Discard is free and
cancellation is not, and on one of the live targets cancellation does not
exist.** A design that needs cancellation to be correct cannot ship on Darwin.
Nothing below ever needs it: every discard in Part A is join-then-drop (§A.6.3).

The `openat` row is the whole of Part B's runtime story: the adapter already
takes a NUL-terminated path of any component count and already stages it into
per-operation storage. Part B's runtime diff is zero lines, and §B.9 makes that
a checked review invariant rather than a claim.

---

# Part A — the streaming chunk loop

## A.1 The program, and the three walls

`writer3/sizes.wf:9-27`, written by a writer given only the spec and
`docs/patterns.md`, and `writer3/largest.wf:9-27` is the same function byte for
byte:

```whitefoot
fn count_file['f, 'd](file: &'f ReadFile, scratch: &uniq 'd buffer<u8>) -> total: own u64
reads(file, scratch), writes(scratch) {
  let sum = 0_u64;
  loop @chunk {
    region 'c {
      match read_chunk<'f, 'c>(file: file, scratch: &uniq 'c deref(scratch), offset: sum) {
        ReadBytes(next: taken) => { set sum = sum +wrap taken; }
        ReadEnd() => { break @chunk; }
        ReadFailed(error: problem) => { break @chunk; }
      }
    }
  }
  return sum;
}
```

This is the owner's 读取-处理-丢弃 loop. It is the shape of `wc`, `cksum`,
`md5sum`, and `grep` over one file. Three separate [PAR-3] conditions refuse it,
and a design that removes only one buys nothing.

**Wall 1 — the exit is in the remainder.** `spec/kernel-spec.md:2057` requires
every leaving edge to occur in P; `:2058` puts an edge the cut statement takes on
its own submission's outcome in E. Both `break @chunk` statements are exactly
that edge.

**Wall 2 — the cursor is on both sides of the cut.** `sum` is read in P (it is
the offset actual, evaluated at c) and written in E. None of
`spec/kernel-spec.md:2060`'s three alternatives covers it, so the loop denies
with `staged_permission.rs:1618`'s reason: *"the body reaches it on both sides of
the cut, so no single segment serializes it"*.

**Wall 3 — the destination is enclosing storage.** `scratch` is a `&uniq`
parameter, so it is rooted outside L, and the read retains a borrow of it past
its own submission — condition 3, `spec/kernel-spec.md:2059`.

Walls 1 and 2 are language questions and this design answers them. Wall 3 is a
taught form: `docs/patterns.md:346` (P15) already says "construct the
per-iteration scratch **inside** the loop body", for a reason that predates any
performance argument (`patterns.md:342-344`: with one reused buffer, after a
short read the bytes beyond `next` are the previous iteration's).

## A.2 The decision

Five decisions, each one taken over a named alternative.

1. **The loop is genuinely staged under [PAR-3].** Condition 2 gains a second
   alternative — a *terminating exit* — so the `ReadEnd` break is admitted where
   the writer wrote it, and the loop then holds the ordinary staged permission:
   E(i) may overlap P(i+1), and the [PAR-3] compute-lane hand-out of
   `LOOP-PIPELINE.md:837-873` applies to the fold. This is taken over
   `runtime-first`'s framing, in which a prefetch is "not an action of B" and
   sits outside [PAR-3] — elegant, but both judges found it grants a permission
   inside a rule whose premise it refutes (§C.2, R-A2), and it then prices itself
   against a C loop whose fold runs on a second thread that its own rule does not
   license.

2. **The offset is a *predicted* place, and the prediction bound comes from the
   operation contract, not from a runtime policy.** `spec/kernel-spec.md:2564`
   already fixes `start <= next <= end` for every successful transfer payload, so
   the operand `taken` cannot exceed the `end` actual — which P evaluates as an
   argument, before c publishes anything. The predicted cursor is
   `offset +wrap end`: provable, deterministic, and stated in the rule. This is
   `writer-first`'s finding and it is the one that turns "the implementation
   guesses" into "the implementation computes a one-sided over-estimate the
   contract fixes".

3. **The disposition has two grades, and the cheap one ships first.**
   `carried-closed` — a recurrence naming no payload binder of an undelivered
   submission — needs no prediction, no comparison, no discard, and no
   discardability property. `carried-predicted` needs all four. This is
   `proof-first`'s split, and it is the design's most useful structural idea: it
   is what makes the ordinary "scan a buffer, act on each record" loop stage,
   including Part B's list loop (§B.7), and it gives the owner a position from
   which speculation can be declined without losing Part B.

4. **Speculation is fenced by five clauses, not two.** A speculative prologue
   runs on a value the source order may never hold, so it must not be able to
   evaluate a `claim`, to apply a partial operation to that value, to create a
   host resource, or to change the state of any resource it names. Both judges
   found that the two designs with only the first two fences abort a correct
   program (§A.8.1). `writer-first`'s conditions 2b.3-2b.5, with its own §D.1
   correction to 2b.3, are taken verbatim.

5. **Discard is join-then-drop, never cancel, and the ring is freed only after
   every discarded slot reaches terminal.** §0.1's last row is the argument; the
   procedure is `proof-first` §A.5's, including its review instruction.

**What the writer writes differently: one line, and it is P15.** The destination
buffer moves inside the loop body. Nothing about the offset, the exit, the depth,
or the prefetch appears in the source. §A.10 is the argument that this is not a
hidden trick.

## A.3 The writer-facing example, as it would compile

One change from A.1: the chunk buffer moves inside the body. That change also
deletes the `read_chunk` helper, because a `buffer_new` inside the body is an own
binding and `&uniq 'c data` is an ordinary borrow rather than the reborrow
[OWN-6] forced the helper for (`docs/done/0098-blind-writer.md:296`). The loop is
shorter than what the writer wrote.

```whitefoot
fn count_file['f](file: &'f ReadFile) -> total: own u64 reads(file), allocates(heap) {
  let total = 0_u64;
  let offset = 0_u64;
  loop @chunk {
    let data = buffer_new(65536_u64, 0_u8);
    region 'c {
      match read_at<'f, 'c>(file: file, destination: &uniq 'c data,
                            file_offset: offset, start: 0_u64, end: 65536_u64) {
        ReadBytes(next: taken) => {
          set total = total +wrap taken;
          set offset = offset +wrap taken;
        }
        ReadEnd() => { break @chunk; }
        ReadFailed(error: problem) => { break @chunk; }
      }
    }
  }
  return total;
}
```

The folding form — the owner's read-process-discard — replaces the first `set`
with a bound call, because [GRAM-9] does not derive a call in an atom position
(`writer3/NOTES.md`, W1; this is the slip `proof-first`'s own example made):

```whitefoot
        ReadBytes(next: taken) => {
          let digest = 0_u64;
          region 'd { set digest = fold_bytes<'d>(source: &'d data, produced: taken, seed: 0_u64); }
          set sum = sum +wrap digest;
          set offset = offset +wrap taken;
        }
```

`offset` is `carried-predicted`. `sum` is **not** a carry — it is written in E
and never read in P, so it is an ordinary ordered write already covered by
`spec/kernel-spec.md:2064`. That distinction is why a non-associative or float
fold is admitted here with no algebra, no identity, and no combination tree: the
remainder commits in iteration order.

A rolling fold written `seed: sum` is still admitted and still pipelines its
*reads* — the offsets do not depend on the fold — but each fold then waits for
the previous one and none goes to a compute lane. That is a true statement about
the program the writer wrote, not a restriction the design adds, and it belongs
in `docs/patterns.md` P19 (§E).

## A.4 The exact specification delta

Two rules amended, one operation-table property added, one existing operation
given an inspection it should already have had. **No new rule** (137 remain),
no grammar production, no keyword, no operation, no type, no outcome, and no
writer-visible marker.

### A.4.1 [PAR-3] — the terminating exit

`spec/kernel-spec.md:2057`, verbatim:

> Every edge that leaves B — a `return_stmt`, a `give_stmt` delivering outside
> B, a `break_stmt` naming L or a loop enclosing L, and a `let_stmt` selecting
> `propagate_let_rhs` [FN-1, GIVE-1, ERR-3] — occurs in P.

becomes:

> Every edge that leaves B — a `return_stmt`, a `give_stmt` delivering outside
> B, a `break_stmt` naming L or a loop enclosing L, and a `let_stmt` selecting
> `propagate_let_rhs` [FN-1, GIVE-1, ERR-3] — occurs in P or is a terminating
> exit.
>
> A terminating exit is an edge leaving B that occurs in E, admitted exactly
> when all of the following hold. Every action of P performs, for every place
> rooted outside B, no write and no consumption of an `own` value. Every write
> of P is to a place rooted in a binding B itself introduces, to a place this
> rule replicates, or to a place this rule carries. The action at c creates no
> host resource and its contract fixes no change to the state of any resource it
> names. P contains no `claim` statement. Every read a statement of P performs of
> a place this rule predicts is either an argument of the action at c or an
> operand of an operation total on its type. An action performs a write, for this
> condition, exactly where its own contract fixes that it does [SYS-2, EFF-2]; an
> action whose write the implementation does not resolve denies the exit rather
> than admitting it.
>
> When an execution of one iteration leaves L through a terminating exit, the
> overlapped execution completes every operation a later iteration submitted,
> performs no segment E of any such iteration, delivers no outcome of such an
> operation to a source binding, and produces none of that iteration's
> observables. It produces exactly the observables the source-order execution
> produces before the exit and produces none after it.

`spec/kernel-spec.md:2058` — "An edge the statement performing c takes on the
outcome of that submission … is an edge of E and not of P" — **stays exactly as
written**. It is what makes the `ReadEnd` break an edge of E; the new paragraph
is what admits an edge of E. The two are consistent and neither is weakened.

### A.4.2 [PAR-3] — the fourth disposition, in two grades

`spec/kernel-spec.md:2060`, verbatim:

> Every place rooted in a binding declared outside L that a footprint of B
> reaches satisfies one of exactly three conditions, and a place satisfying none
> denies permission. Either no footprint of B writes it and every loan on it is
> shared; or every footprint element and every loan touching it belongs to one
> of P and E alone and no loan on it is retained past c; or this rule replicates
> it.

"exactly three" becomes "exactly four", the enumeration gains "…or this rule
carries it", and the following is added:

> This rule carries a place rooted in a binding declared outside L only when its
> element type is copy [OWN-1]; when no loan of B is ever formed on it; when
> every read of it by a footprint of P is an operand read; when every write of it
> by a footprint of B is one `set` statement whose target is that whole place and
> whose right-hand side is one operation total on its type applied to that
> place's prior value and to one further operand; and when every path through E
> that a later iteration's prologue can follow executes that `set` exactly once.
> A carried place is committed at the end of each iteration's segment E, in the
> order of the iterations that perform it, and holds at L's continuation exactly
> the value the source-order execution leaves in it.
>
> A carried place whose recurrence names no payload binder of an outcome the
> implementation has not delivered is *closed*: its value for a later iteration
> is computed in P from terms P already holds, and no rule below applies to it.
>
> A carried place whose recurrence names such a payload binder is *predicted*
> only when, in addition, the contract of the action at c fixes one value that
> further operand cannot exceed and which the prologue evaluates before that
> action's outcome is published.

For `read_at` that fixed value is already stated:
`spec/kernel-spec.md:2564` — "Every successful transfer payload is an absolute
endpoint `next` … and satisfies `start <= next <= end`" — bounds `taken` by the
`end` actual, so the predicted cursor is `offset +wrap end`. Nothing in source
says so; the bound comes from the operation contract.

### A.4.3 [PAR-3] — the speculative prologue, and what a discarded attempt is

Added after `spec/kernel-spec.md:2065`:

> An implementation may execute the segment P of an iteration using, for a place
> this rule predicts, the value the recurrence produces from the fixed value the
> action at c's contract states, in place of the value the remainder computes. It
> may deliver an outcome of a submission of that segment, and may perform that
> iteration's segment E, only when every value that segment P consumed equals the
> value the source-order execution produces at that point. Otherwise it delivers
> no outcome of any submission of that segment, performs no segment E for it,
> carries the compiler-derived releases of the bindings that segment introduced,
> and executes that iteration's segment P again with the committed values.
>
> An implementation performing a segment P it later discards may perform the
> action at c with arguments the source-order execution does not supply. Such an
> action changes no place of the program and no content of any object it names,
> its outcome is delivered nowhere, and it is not an observable of this rule.
> This admits no action whose contract fixes a state change or creates a host
> resource; that condition is stated with the terminating exit.
>
> The place therefore holds at every program point of every iteration the value
> the source-order execution holds there, and which segment P an implementation
> performed, discarded, or performed twice is not observable.

And `spec/kernel-spec.md:2079`, which today reads

> The number of operations an implementation keeps outstanding, the identity of
> the host thread that executes a segment, whether any overlap was performed at
> all, the storage an implementation gives a replicated place, and the storage an
> implementation reuses across iterations for a construction whose value the body
> releases without observing it, are not observable, and no rule of this
> specification is stated in terms of them.

gains, after "whether any overlap was performed at all", the clause

> **, the number of attempts of a discardable operation an implementation
> performs and discards, the values it uses in a segment P whose outcome it does
> not deliver,**

The middle paragraph is the sentence a reader asks for first, and two of the
three source designs did not have it. `spec/kernel-spec.md:2079` makes only the
*number* of outstanding operations unobservable; it says nothing about an
operation the source-order execution never performs at all.

### A.4.4 [SYS-2] — discardable, as an operation-table property

`spec/kernel-spec.md:2293` already carries a per-operation target-contract
property (`never-suspends` / `may-suspend`). Add, immediately after `:2294`:

> A `may-suspend` operation is **discardable** exactly when it creates no host
> resource and its contract fixes no change to the state of any resource it
> names, so that performing an attempt and delivering no outcome of it changes no
> state any operation of this specification observes and produces no value.
> Exactly one operation of this version is discardable: `read_at`, whose transfer
> is positioned, which observes but does not advance the `ReadFile` state
> [SYS-11], and whose whole declared change is to the `[start, next)` range of a
> `destination` an undelivered attempt does not publish. No other `may-suspend`
> operation is: `write_once` changes the output state, `directory_next` advances
> the enumeration cursor [SYS-8], and the four opens create a resource. An
> operation added by a later version is not discardable unless its own record
> says so and it satisfies this test.

Two things about that wording, both of which cost a source design a defect.

The property is stated as a **test** and then applied, rather than asserted for
`read_at` and left there. `writer-first`'s own §D.1 correction found why:
`directory_next` creates no host resource and is still not discardable, because
`spec/kernel-spec.md:2568` fixes that on `ListBytes` "the enumeration cursor
advances past exactly the entries those records name". A discarded
`directory_next` has consumed entries the program will never see. The test that
generalizes is "creates no host resource **and** its contract fixes no change to
the state of any resource it names", and it is what keeps `dir_walk.wf:262` and
`p1_tree_wc.wf:304` correctly denied (§D).

The property is keyed on the **semantic ID**, never on a name, a signature, or a
call-site shape, per `CLAUDE.md`'s rule that a capability is implemented by
grammar and semantic rule. One table lookup discharges condition 2b.3.

### A.4.5 [SYS-10] — `open_read` gains the inspection `open_file` already has

**This lands first, on its own, whether or not the rest of Part A ships.** All
three source designs found it independently and both judges called it a
correctness fix in its own right.

[SYS-14] requires `open_file` to inspect the descriptor before publication
(`spec/kernel-spec.md:2685`): inspection failure returns its class, a directory
returns `IsDirectory`, and "every other successfully inspected non-regular object
returns `Err(Other(code: 0_u32, origin: 0_u8))`". `open_read` has **no such
sentence anywhere** — its eleven mentions are `:2274, 2293, 2294, 2313, 2498,
2519, 2605, 2609, 2611, 2625, 2684`, and none inspects a kind. So a
`RelativePath` naming a FIFO, a character device, a `/proc` file, or a tape
yields a `ReadFile` today, and [SYS-11]'s positioned-read contract
(`spec/kernel-spec.md:2626`) is unsatisfiable for it.
`FIRST-PRINCIPLES.md:374-378` is explicit: "MMIO, device files, virtual files,
and read-and-clear state cannot be silently admitted under the positioned-read
contract."

Add to [SYS-10], after `spec/kernel-spec.md:2611`:

> `open_read` performs the same descriptor-status inspection before publication
> that [SYS-14] requires of `open_file`, with the same outcomes and the same
> single native close attempt, so every `ReadFile` a program holds names a
> regular file and every positioned read of it is repeatable at the same
> arguments with no state change and no observation.

This is a semantic-ID change under [META-5]: an object that reaches source as
`Ok` today reaches it as `Err` after. It is the premise the whole discard
argument rests on — a discarded read on a read-and-clear device destroys data the
program would have seen — and until it lands, `carried-predicted` must be
restricted to a `ReadFile` whose producer inspected the object, which the [PRV-2]
provenance column already carries (`spec/kernel-spec.md:966`). That fallback is
`runtime-first`'s and it is the right bridge, not the destination.

### A.4.6 The [META-5] delta shape

> Numbered rules +0/-0 (137 remain); grammar productions +0/-0; keywords +0/-0;
> writer operation spellings +0/-0; opaque system nominal spellings +0/-0; entry
> forms +0/-0; runtime-trap families +0/-0; exception clauses +0/-0; system
> operations and declaration records +0/-0 (203 remain). [PAR-3] is amended to
> admit a terminating exit under five stated conditions, to carry a fourth place
> disposition in a closed and a predicted grade, and to admit a segment P
> executed on a contract-fixed predicted value whose outcome it may have to
> discard. [SYS-2] is amended to name which of its `may-suspend` operations is
> discardable, by a test rather than by enumeration. [SYS-10] is amended to give
> `open_read` the descriptor-status inspection `open_file` already performs, which
> moves a non-regular object from `Ok` to `Err(Other)`. No ownership, effect,
> release, or trap rule changes. The permitted-overlap set only widens, so no
> [PAR-3] conformance verdict moves; the `open_read` inspection is the one change
> that can move a verdict and the merge must check for one.

## A.5 The judgment — what the compiler must prove

`compiler/src/semantic/staged_permission.rs:36-95` states today's seven
conditions. Condition 2 becomes two, condition 5 gains a fifth disposition in two
grades, and one new condition is added. Everything is read from typing, declared
effect rows, resolved places and the statement graph — **no entailment fact** —
so the module's stated invariant (`staged_permission.rs:162-166`) survives, and
stage 2's fact-consuming byte-range analysis is not needed for any of it.

### A.5.1 Condition 2 becomes 2a and 2b

**2a — an ordinary exit still leaves from P.** Unchanged. It is what keeps
`many_files_wide8.wf` denied for its `Err` arms
(`docs/done/0098-blind-writer.md:65`).

**2b — a terminating exit.** `StagedDenial::ExitInRemainder`'s
`selected_by_submission` flag (`staged_permission.rs:296`) stops being a denial
reason and becomes a precondition query. When it is set, all five must resolve:

| # | clause | how it is discharged |
|---|---|---|
| 2b.1 | every action of P writes no place rooted outside B and consumes no `own` value rooted outside B | the existing `Program::footprint` / `call_projection` survey (`semantic/permission.rs:1116,1244`) |
| 2b.2 | every write of P is to a place B introduces, a replicated place, or a carried place | the same survey against the disposition table |
| 2b.3 | the action at c is discardable [SYS-2] | one table lookup on the semantic ID |
| 2b.4 | P contains no `claim` statement | one statement-kind scan of P |
| 2b.5 | every read a statement of P performs of a **predicted** place is an argument of the action at c or an operand of an operation total on its type | `collect_operand_reads` plus the [OP-1] totality table |

2b.3, 2b.4 and 2b.5 are the three no reader will have predicted, and each closes
an attack a judge landed on a source design (§A.8). 2b.4 and 2b.5 are vacuous for
a `carried-closed` loop, because nothing is predicted; they bind only the
speculative grade.

### A.5.2 Condition 5 gains `carried`, in two grades

`disposition_of` (`staged_permission.rs:1565`) returns four dispositions today.
The fifth is `Carried(Grade)` with `Grade ∈ {Closed, Predicted}`, spelled
`carried` and `carried-predicted` in the ledger. Over the place's whole [OWN-7]
overlap class (the module already unions flags over the class, `:97-124`):

| # | obligation | where it is decided |
|---|---|---|
| 1 | the place's element type is copy [OWN-1] | `is_copy_element` (`staged_permission.rs:1638`) |
| 2 | no `Loan` of any statement of B names it | `Footprint::loans` (`semantic/permission.rs:214`) |
| 3 | every read of it by a footprint of P is an operand read | the existing survey |
| 4 | every write of it by a footprint of B is one `set` at that whole place, whose right-hand side is one [OP-1] application over that place and one further operand | one IR pattern match |
| 5 | every path through E a later prologue can follow executes that `set` exactly once | the body's own `Flow`, the CFG condition 1 already queries |
| 6 | *(Predicted only)* the [SYS-2] row for the action at c fixes a bound on the further operand, and P evaluates that bound before c publishes an outcome | one [SYS-2] table entry |

`Grade` is `Closed` when no payload binder of an undelivered outcome occurs in
any admitted right-hand side, `Predicted` otherwise. `Predicted` additionally
requires 2b.3, 2b.4 and 2b.5.

Obligation 6 is the only new machinery, and for the whole first slice it is one
table entry: `read_at`'s [SYS-8] payload relation. Two real writer programs fall
on the two sides of it and both are wanted:
`read_heavy_narrow.wf:209` writes `set position_0 = position_0 +wrap 8_u64;` —
the operand is a literal, obligation 6 is trivial, and the prediction is *exact*
and never discards; `writer3/sizes.wf:15` writes `set sum = sum +wrap taken;` —
the operand is the arm binder, obligation 6 discharges against `end`, and the
prediction is a one-sided over-estimate a short read refutes. A design admitting
only the first would leave the owner's loop exactly where it is.

### A.5.3 Condition 8, new — fail closed on the recurrence

A `set` of a candidate carry whose right-hand side this judgment does not resolve
to one [OP-1] application over admitted operands **denies**. A partial operation,
a `Result` route, a call, or a second `set` on a different path all deny.
`+defined`, `+checked` and `+sat` deny, because re-executing an application that
carries a domain obligation is not the same as executing it once. `+wrap`,
`-wrap`, `*wrap`, the bit operations and `imin`/`imax` are admitted; the float
operations are admitted too, because this rule **re-executes rather than
recombines** and therefore uses no associativity — unlike [PAR-2], which refuses
float folds for exactly the reason this rule does not need.

### A.5.4 What still denies, and must

- A body publishing per iteration holds an exclusive loan on `Output` in E, which
  condition 4 denies (`staged_permission.rs:326`). Untouched. That is D1 (§D).
- `set at = at +defined taken` denies at condition 8.
- A loop whose P opens a file denies at 2b.1 (`reserve_file` writes the enclosing
  factory) and at 2b.3 (an open creates a host resource).
- A `directory_next` batch loop denies at 2b.3 by the [SYS-2] test of §A.4.4.
- `writer3/sizes.wf:74-93`'s `@slurp` loop stays sequential, correctly: it
  *accumulates* into one buffer with `start: filled`, and the program keeps every
  byte. Nothing here is a fold-and-discard.

## A.6 The lowering

`LOOP-PIPELINE.md` §§3.1-3.4 already designs the ring, the slot record, the
in-order commit and the stage outlining, and this design changes none of it. What
it adds is the driver's behaviour for a predicted cursor and for a terminating
exit, plus one runtime policy.

### A.6.1 Depth comes from the runtime, once, at loop entry

`LOOP-PIPELINE.md:690` fixes the query
`wf__completion_window(span, slot_bytes, ceiling)`, with the discipline copied
from `wf__par_split_budget` (`par_runtime.c:943`): asked once per loop entry,
never per iteration. A `for_stmt` supplies its trip count as `span`. **A
`loop_stmt` has no trip count**, and the owner's chunk loop is one, so it passes
`span = 0` meaning unbounded and the runtime answers from its own capacity —
`WF_BRIDGE_OPERATION_CAPACITY` and `WF_BRIDGE_SLOT_COUNT` are 64
(`bridge.c:40-41`). **That is the entire language surface for K: there is none.**
`K = 1` is always a legal answer and reproduces the sequential program exactly,
so the query can never make a program fail.

For `count_file` as written in §A.3, `slot_bytes` is 65,536 for `data` plus the
slot scalars (`offset`, `total`, `taken`, the stage tag, the completion token) —
call it 65,600. K = 8 is 525 KiB reserved once; K = 32 is 2.1 MiB.

A loop whose slot storage alone exceeds the static ceiling declines at compile
time and says so, exactly as `compiler/src/lowering.rs:1163`'s `Decline` reports
a lane frame that does not fit. A writer who calls `buffer_new(16777216_u64, …)`
inside the loop gets K = 1 and a reported decline, not 512 MiB of invisible
memory.

### A.6.2 The driver

One block per role, all on the owner lane, no stack switching — `stackless.rs`
stays out of the critical path exactly as `LOOP-PIPELINE.md:813-825` argues.
Two registers per carried place: `committed`, advanced only by in-order commit,
and `speculative`, advanced by each prologue.

```text
entry:    K = wf__completion_window(0, slot_bytes, ceiling)
          allocate the ring of K slots; committed = the source binding's value
          speculative = committed; misses = 0
issue:    while a slot is free, no terminating exit is pending, and K > 1:
            run P on `speculative` for sequence n; record consumed[n] = speculative
            submit read_at into slot n's private destination
            speculative = recurrence(speculative, end_actual)      // A.4.2's bound
            n += 1
harvest:  wf__completion_file_take on the oldest busy slot (bridge.h:144);
          if not ready, wf__completion_file_join it (bridge.h:129)
commit:   for the oldest slot in sequence order, and only that one:
            if consumed[j] != committed:                            // mispredicted
              discard slots j..n-1; speculative = committed; misses += 1
              if misses >= m: K = 1                                 // A.6.4
              continue
            deliver its outcome to its own `match`
            run E: the arms' writes to serialized-E places, then the carry `set`
            committed = the value E computed; misses = 0
            release the slot (restore, LOOP-PIPELINE.md:760-771)
            if the outcome selected a terminating exit: goto drain
drain:    join every slot with sequence > j, discard its outcome, restore it,
          then free the ring, then take the edge
```

### A.6.3 Discarding a slot — the part most likely to be got wrong

A slot whose `read_at` is still in flight owns its destination buffer *at the
target*, until the operation's `terminal` milestone. Discarding is **not** "forget
it". The procedure, from `proof-first` §A.5 verbatim:

1. Stop issuing new prologues.
2. For each slot to discard, in any order: obtain its terminal transition —
   `wf__completion_file_take` non-blocking (`bridge.h:144`), falling back to
   `wf__completion_file_join` (`bridge.h:129`). An implementation *may* first ask
   the target to cancel; it must still reach terminal, and §0.1's last row is why
   cancellation is never the correct path and never required.
3. Drop the outcome **without inspecting it**. Do not map an error, do not
   publish a `ReadFailed`, do not advance any carried place.
4. Run that slot's compiler-derived releases for the bindings its prologue
   introduced ([STOR-3], `spec/kernel-spec.md:651-660`), restore the ring slot,
   and mark it free.

Only after every discarded slot reaches terminal may the loop take an exit edge
or free the ring. **A target buffer released before its operation's terminal
milestone is a use-after-free, and it is the single defect this design's review
must look for first.**

This is the same shape `spec/kernel-spec.md:2071` already requires for an exit
from P — "every operation of an earlier iteration still outstanding is completed
and its segment E performed before that edge is taken" — with "and its segment E
performed" replaced by "and its outcome discarded" for iterations after the exit.
It costs one join per in-flight slot at loop exit: once per loop, not per
iteration.

### A.6.4 The mispredict backoff — a runtime policy, not a language surface

A misprediction discards slots `j..n-1`, so it costs **K-1 wasted reads**, not
one. Two source designs said "one" and both judges caught it. For a whole-file
stream the only mismatch is the last partial chunk, so the cost is K-1 wasted
reads per *file* against a stream of thousands, and `read_heavy_narrow.wf`'s
constant stride mispredicts never.

But `spec/kernel-spec.md:2556` — "A short success is not end of input" —
explicitly permits a filesystem that caps every transfer below the requested
extent: NFS, FUSE, an overlay, or a `pread` interrupted by a signal. Such a
source mispredicts on *every* chunk, and at K = 32 the loop would issue 32 reads
per delivered chunk and run roughly 30x slower than the sequential build. So:

> After `m` consecutive mispredicting commits the implementation sets K to 1 for
> the remainder of that loop entry, and issues no further speculative prologue.

`m = 2` is the proposed constant. K = 1 is a legal answer of the window query by
construction (§A.6.1), so this is a policy inside the runtime's own answer, not a
rule, not a spelling, and not a writer-visible knob. §A.9's falsifier bounds the
adversarial case at 1.2x the sequential build, which is what makes the policy
checkable rather than asserted.

### A.6.5 Where `ReadEnd` lands

Nowhere new. The outcome of slot j is delivered to slot j's own `match` during
that slot's commit, in sequence order, so `ReadEnd` reaches exactly the `match`
the writer wrote, in exactly the iteration that would have seen it sequentially.
A speculative prologue that *itself* returned `ReadEnd` has its result retained,
not published: if its slot commits, that retained `ReadEnd` is its iteration's
outcome; if it is discarded, the re-issued read decides. On a regular file the
common case is that the prefetched `ReadEnd` is right, and the loop pays one
extra read for the whole file — which is what `cat` pays.

What is different is only what happens *after* the `break` is taken: §A.6.2's
drain. A writer reading their own source sees one `break` and one exit; the
pipeline never delivers a `ReadEnd` early, to the wrong arm, or twice.

## A.7 The ledger a writer sees

`compiler/src/semantic/permission_ledger.rs:238` and `:251` fix the two line
shapes; `:494`'s `staged_detail` fixes what follows a permitted verdict. Three
additive changes, and one deletion.

The disposition column is `{:<12}` today and `carried-predicted` is 17, so the
column widens to 17 in the same change.

```text
PAR stage       sizes.wf:11    loop  permitted   staged at read_at(file: file, …); 5 places classified
PAR place       sizes.wf:11    replicated        let data = buffer_new(65536_u64, 0_u8);   each in-flight
                                                 iteration gets its own storage; no statement after the loop reads it
PAR place       sizes.wf:11    carried-predicted let offset = 0_u64;   the remainder advances it by at most the
                                                 read's own `end`, so a later prologue runs on that bound and is
                                                 discarded if the remainder disagrees
PAR place       sizes.wf:11    serialized-E      let total = 0_u64;    written only in the remainder, committed in
                                                 iteration order
PAR place       sizes.wf:11    read-only         &'f file              no footprint of the body writes it
PAR exit        sizes.wf:14    terminating       break @chunk;         selected by the read's own outcome; reads
                                                 already in flight are completed and discarded
PAR ring        sizes.wf:11    loop  65,600 bytes per slot (65,536 destination, 64 slot record); up to 8 slots,
                                                 allocated once at loop entry; the number of slots is chosen by the
                                                 runtime at entry and has no source spelling
```

**`PAR exit` is a third line kind** rather than a sixth disposition, because the
thing classified is an edge and not a place, and because a writer who has just
read `EXIT_SELECTED_BY_SUBMISSION` (`staged_permission.rs:366`) telling them
their loop cannot be staged needs the line that says it now can. The
notice-channel predicate at `permission_ledger.rs:265` extends unchanged: a
`PAR exit` line is a notice exactly when the loop is denied.

**`PAR ring` is a fourth line kind**, and it exists for exactly one reason — the
owner's ruling that anything the compiler allocates is allocated once at loop
entry and stated. It prints the per-slot size **broken down**, the fact that the
allocation happens once, and the fact that the count is the runtime's. It prints
on a *granted* loop, which is a deliberate exception to "a granted loop says
nothing"; the exception is bounded by keeping it on `--par-ledger` only, so an
ordinary build stays silent (`docs/patterns.md:445-450`).

**Nesting is stated too.** When the remainder of a staged loop calls a function
whose own body holds a staged loop, K outer iterations in flight each carry an
inner ring, and the storage is K_outer × K_inner. `spec/kernel-spec.md:2072`'s
retire-and-retry clause already covers the correctness of the host-resource half;
none of the three source designs put the storage anywhere a writer could read it.
The `PAR ring` line of the outer loop therefore names it:

```text
PAR ring        list.wf:14     for   1,104 bytes per slot (1,024 staged path, 80 operation record); up to 32 slots,
                                                 allocated once at loop entry; each slot's remainder enters
                                                 count_file, whose own ring is up to 8 slots of 65,600 bytes
```

**Two denial texts are deleted in the same change.** `EXIT_IN_REMAINDER` and
`EXIT_SELECTED_BY_SUBMISSION` (`staged_permission.rs:365-366`) exist only to be
honest about a limit this design removes. Leaving them would be a compiler telling
a writer to stop trying at exactly the shape it now compiles — the same failure
`docs/done/0100-writer-defaults-2.md:137-167` (W4) was raised to fix. The
paragraph at `docs/patterns.md:459-464`, which tells the writer that one file's
chunk loop cannot be staged, is superseded in place by P19 (§E).

One new denial text is added, for condition 8:

```text
PAR stage       checksum.wf:6  loop  denied      condition 8: the body advances `at` with an operation this judgment
                                                 cannot re-execute; advance a place the prologue reads with one total
                                                 operation — `+wrap` and the bit operations are total, `+defined` and
                                                 `+checked` are not — or leave this loop sequential, at
                                                 set at = at +defined taken;
```

## A.8 The safety argument, and the attacks that shaped it

Safety is not negotiable. Every subsection below is an attack a judge landed on
one of the three source designs, stated as the program or schedule that exhibits
it, followed by the clause of this design that closes it. The ones that landed
are the reason condition 2b has five clauses where two source designs had two.

### A.8.1 A speculative prologue fires a `claim` the source order never reaches

**The attack** (both judges, independently; against `proof-first` and
`runtime-first`). Neither design constrained what a speculated prologue *computes
from* the predicted value — one required only that P's reads of the carried place
be operand reads, the other constrained only P's writes. A `claim` writes nothing,
so this loop was admitted by both:

```whitefoot
  loop @chunk {
    let block = buffer_new(65536_u64, 0_u8);
    claim bounded: ilt(at, 1073741824_u64) because "this tool only reads files under 1 GiB";
    region 'c {
      match read_at<'f, 'c>(file: file, destination: &uniq 'c block,
                            file_offset: at, start: 0_u64, end: 65536_u64) {
        ReadBytes(next: taken) => { set at = at +wrap taken; }
        ReadEnd() => { break @chunk; }
        ReadFailed(error: problem) => { break @chunk; }
      }
    }
  }
```

On a file of 1 GiB minus one byte the source order never reaches `at >= 2^30`.
With K = 32 the speculative cursor runs about 2 MiB ahead, and in the last 2 MiB
the predicted prologue evaluates the claim **false**. [PAR-3]'s
erroneous-execution clauses then fire verbatim (`spec/kernel-spec.md:2074-2078`):
one [DIAG-3] record and a whole-process abort. **A correct program dies because
the compiler guessed** — the exact inverse of the owner's ruling that a claim is
never removed for speed, since here a claim is *added* to an execution that never
performs it. `spec/kernel-spec.md:2077` read the other way is the sentence that
is broken: "A correct program executes no false `claim`, so the impossible branch
cannot narrow or surcharge its execution."

**Closed by condition 2b.4: P containing a `claim` denies the terminating exit.**
The loop is refused, not the claim. That is the fail-closed direction
(`staged_permission.rs:90-95`), it never removes or weakens a written claim, and
it costs the repository's writers nothing: `docs/done/0098-blind-writer.md:40`
records "Zero `claim` statements in 1,694 lines" across the five blind-writer
programs, and neither `writer3/sizes.wf` nor `writer3/largest.wf` writes one. It
is a wall no program in this tree stands behind. §A.12 Q3 records the narrower
rule for the day one does.

### A.8.2 A speculative prologue trips an unwritten trap

**The attack** (judge 2, FF-7; against `runtime-first`, and left to luck by
`proof-first`). The same hazard without a `claim`, through the obligations that
are discharged statically over the values the source-order execution produces:

```whitefoot
  loop @chunk {
    let block = buffer_new(65536_u64, 0_u8);
    let mark = table[offset];        // [OP-4] residual `offset < len(table)`
    region 'c { match read_at<'f,'c>(file: file, destination: &uniq 'c block,
                                     file_offset: offset, start: 0_u64, end: 65536_u64) { … } }
  }
```

`at +defined step`, `limit -defined at`, and `deref(table)[at / 4096_u64]` whose
[ENT-6] `SubscriptBounds` obligation was discharged from a loop-carried fact are
the same shape. The proof does not cover a value the execution never produces.

**Closed by condition 2b.5:** a read of a predicted place in P is either an
argument of the action at c or an operand of an operation total on its type. That
is exactly what the writers' loops already do — `sizes.wf:13` passes `sum` as
`offset`; `read_heavy_narrow.wf:187-188` passes `position_0` through `block_at`
and `*wrap` — and anything else denies. Note the direction: 2b.5 is blunt, and it
denies some loops that would in fact be safe (an element access in P is neither an
argument of c nor a total operation). That is the safe direction and it is stated
rather than discovered.

### A.8.3 A discarded read on a device that is not a regular file

**The attack** (judge 1, FA-4; against `proof-first`, which declared `read_at`
discardable *unconditionally* over every `ReadFile`). `open_read` performs no
descriptor-status inspection (§A.4.5), so `open_read(root: &cwd, path)` on
`dev/urandom`, a `/proc` file, a seekable character device, or a tape yields a
`ReadFile` today and `pread` on those succeeds. A discarded read on a
read-and-clear device destroys data the program would have seen.

**Closed by §A.4.5**, which gives `open_read` the inspection, and by the
[PRV-2] provenance restriction as the bridge until it lands. The pipe half is
closed twice over: `read_at` lowers to `pread` (`emitter/system.rs:409,1440`) and
to `IORING_OP_READ` with an explicit `off` (`linux_io_uring.c:512,515`), both
positioned, both `ESPIPE` on a pipe; and `ESPIPE` (29) is in neither target's
class table, so it reaches source as `Other` — a pipe cannot be read by `read_at`
at all today. But an accident is not a promise, which is why §A.4.5 is a
precondition of `carried-predicted` and not a footnote.

### A.8.4 The mispredict storm

**The attack** (judge 1, FA-5; against all three). A source that short-reads on
every chunk — permitted by `spec/kernel-spec.md:2556` and produced by NFS, FUSE,
an overlay, or a signal-interrupted `pread` — makes every prefetch waste. At
K = 32 that is 32 reads per delivered chunk. No source design had a backoff and
none measured the case.

**Closed by §A.6.4's runtime backoff** (K collapses to 1 after m consecutive
mispredicting commits) and by the falsifier line of §A.9.3 that bounds
`WF_IO_SHORT_READ_EVERY=1` at 1.2x the sequential build. Both are runtime policy;
neither is spec text.

### A.8.5 The compute lane must be licensed by the rule that grants it

**The attack** (judge 1, FA-2; against `runtime-first`). That design's read-ahead
ran only for loops [PAR-3] *denies* at condition 2, so the loop held no staged
permission and no segment of iteration i could overlap iteration i+1 — yet the
design claimed "the fold of chunk i runs on a compute lane while chunk i+1 is in
the kernel" and priced itself against a C loop with the fold on a second thread.
The bar was unearnable by construction on a CPU-bound fold
(`LOOP-PIPELINE.md:0.1`: 65.65 ms user against 61.42 ms sys for `C.wide8`).

**Closed by decision 1 of §A.2**: condition 2b admits the exit, so the loop holds
the ordinary staged permission, so `spec/kernel-spec.md:2066`'s "the segment E of
one iteration with overlapping execution against the segment P of any later
iteration" licenses the hand-out, and `LOOP-PIPELINE.md:837-873`'s
`wf__par_claim` path applies unchanged. The C bar of §A.9.3 is therefore fair.

### A.8.6 A discardable operation that consumes something

**The attack** (`writer-first`'s own §D.1, found while drafting).
`directory_next` creates no host resource and is still not discardable:
`spec/kernel-spec.md:2568` fixes that on `ListBytes` "the enumeration cursor
advances past exactly the entries those records name". A prefetched, discarded
`directory_next` eats entries.

**Closed by §A.4.4's test**, which reads "creates no host resource **and** its
contract fixes no change to the state of any resource it names". This is a
genuine limit, not an oversight to be fixed later: a prefetching enumeration needs
an operation whose speculative advance is recoverable, which
`spec/kernel-spec.md:2627` already anticipates — "a persistent read-ahead Source
is a separate system type rather than hidden state in this type".

### A.8.7 The ordinary four

**No memory corruption.** Every in-flight iteration writes exactly one place: its
own ring slot. Slots are disjoint by construction and the ring is one allocation,
so disjointness is an address-range fact and not an alias analysis. The ring is
freed only by the drain, which joins every busy slot first (§A.6.3), so no target
write ever lands in freed memory. [SYS-8]'s two static obligations
(`spec/kernel-spec.md:2540`) are discharged per call exactly as today, and the
sanitized-count rule (`:2570`) still bounds `next`.

**No data race.** The only places two segments reach are `file` (shared loans
only, `spec/kernel-spec.md:2060`'s first alternative), the replicated slots (one
per in-flight iteration), and the carried places, which live in two owner-lane
registers written only by the owner lane. A fold handed to a compute lane is
handed its slot as its frame and joins before the commit that reads it.

**No uninitialized read.** A ring slot holds the value its `buffer_new`
constructed over its whole capacity (`LOOP-PIPELINE.md:751-753`), so the
iteration-own case needs no coverage proof. This matters, because
`spec/kernel-spec.md:2069` currently forbids deriving a written byte from
[SYS-8]'s "may have changed" wording, so [PAR-3]'s replication clause cannot yet
privatize a *hoisted* buffer at all. Stage 1 of Part A does not need it to;
§A.11's stage 2 does, and it owes the [SYS-8] amendment (§A.12 Q5).

**No claim is removed or cheapened.** Nothing here reads a trap latch, quiesces a
slot before a claim, or orders anything to make a defective execution
reproducible — `spec/kernel-spec.md:2077-2078` forbids exactly that.

### A.8.8 The two honest limits

(i) A discarded `pread` still touches the host: it updates `st_atime` under the
host's own policy and consumes I/O bandwidth. Neither is a Whitefoot state place
and neither is reachable through any operation [SYS-2] declares, but §A.4.4 is
worded as a claim about *this specification's* observables and not about the
host, deliberately.

(ii) A validated prefetch reads its bytes earlier in wall-clock time than the
source order would. For a file no other process mutates — the assumption any chunk
loop already makes — the bytes are identical. For a file another process is
writing, the program may publish bytes from an earlier moment. [SYS-11] already
declines to relate the two (`spec/kernel-spec.md:2627`: "Environment-created
changes to the same physical file do not merge or mutate Whitefoot places"), and
[PAR-3] already admits "an outcome that operation could deliver in the source-order
execution at that point" (`:2072`). Whether *at that point* covers an earlier read
is §A.12 Q2, and it is the only place in Part A that touches a promise.

## A.9 The differential oracle and the falsifier

### A.9.1 The oracles

The pipeline is correct trivially when nothing mispredicts, so an oracle that
only runs real files proves almost nothing. Every identity test below is paired
with something that proves the mechanism ran — the green-is-not-coverage rule of
`LOOP-PIPELINE.md:386-388`.

**O1 — the forced-misprediction schedule sweep.** The repository already has the
instrument: the deterministic test target (`qualification.rs:1013`) answers the
file facility from a scripted in-process host. Script it to answer a fixed
sequence of `read_at` calls with a chosen pattern — full×n then `ReadEnd`;
full×n, one short, then `ReadEnd`; **short at every chunk**; `ReadFailed` at chunk
3; `ReadEnd` at chunk 0 — and run `count_file` at K = 1, 2, 3, 8, 64, comparing
the returned total, the published bytes and the exit status against a build with
no pipeline. This is the whole correctness argument for the prediction, it needs
no real host, and it is the cheapest instrument in the set (`writer-first`).

**O2 — the fault-injected mispredict path on a real filesystem.**
`WF_IO_SHORT_READ_EVERY=n` in the adapter, in the same class as `WF_IO_HELPERS`
(`bridge.c:90`) — a target-policy knob, never a language surface, exactly as
`LOOP-PIPELINE.md:975-979` argues for the ring-off knob. It is the only
deterministic route to the discard path against a real host
(`runtime-first`). O1 and O2 fail differently and both are built.

**O3 — the observable-attempt census.** `strace -f -e trace=pread64,io_uring_enter`
on a file of exactly n full chunks: every offset issued is a multiple of the
chunk size and every offset in `[0, size)` appears at least once. On a ragged
file: exactly one offset appears twice — the one re-issued after the short read —
and no offset outside `[0, size + chunk)` is ever issued. **This is the only
oracle in the set that proves discard-and-replay happened rather than that the
bytes matched** (`proof-first`).

**O4 — the Unix reference.** `docs/done/0100-writer-defaults-2.md` records
`largest.wf`'s ten rows as byte-identical to
`find … -exec stat … | sort -rn | head -10`, and `sizes.wf`'s output against
per-file `wc -c`. Re-run both. Neither reference knows anything about this
compiler.

**O5 — the same file, every K.** `read_heavy_narrow.wf` publishes a folded
checksum; every K from 1 to 64 must publish identical bytes. This catches a
commit-order defect O1's short schedules miss.

### A.9.2 The counters

Two counters beside `wf__completion_file_submissions()` (`bridge.h:161`):
`wf__completion_readahead_attempts()` and `wf__completion_readahead_discarded()`.
For a file of exactly n full chunks at depth K, published reads are exactly n+1
(n full plus the `ReadEnd`), and attempts minus discards equals n+1. **Assert both
numbers, not their difference.** `LOOP-PIPELINE.md:1807-1814` records why: the
pipeline design's own falsifier F2 was wrong as written, because the operation
counts were assumed rather than measured.

### A.9.3 The falsifier

**F1 — the ledger, free and first.** `whitefootc --par-ledger` on
`research/experiments/io-completion-bench/programs/read_heavy_narrow.wf`
**byte-unchanged** prints `permitted` for all eight lane loops, with `data`
replicated, `position_N` `carried-predicted`, `sum` and `bytes` serialized-E, and
a `PAR ring` line. And on `writer3/sizes.wf` with only the P15 move of §A.3, the
`@chunk` loop prints `permitted` with a `PAR exit … terminating` line. If either
denies, the judgment is wrong and no measurement is worth taking.

**F2 — the submission counter.** `wf__completion_file_submissions()` is **0**
today for a `match read_at(…)` scrutinee (`lowering/builder.rs:739-757` submits
nothing) and must be `ceil(size / chunk) + 1` after. It separates a judgment
failure from a lowering failure and costs one line.

**F3 — the wall clock, and this is the owner's bar.** The reference is a
hand-written C loop keeping K positioned reads of one file continuously in flight
through io_uring, with the fold on a second thread, same chunk size, same file,
same tree — **built and measured first**, in one interleaved plan with the
Whitefoot binaries, exactly as Probe B of `LOOP-PIPELINE.md:1368-1371` was.

```text
  REQUIRED  Linux 4 KiB uncached   C.narrow.after   <=  1.15 x  the hand-written prefetch loop
  REQUIRED  Linux 64 KiB cold      C.narrow.after   <=  1.15 x  the hand-written prefetch loop
  REQUIRED  Linux  WF_IO_SHORT_READ_EVERY=1         <=  1.20 x  the same binary at WF_IO_RING=0 WF_IO_HELPERS=0
  control   Linux  WF_IO_RING=0 WF_IO_HELPERS=0 on the same binary is unmoved
  control   Linux  --no-overlap build is bit-exact with the default build's published bytes
  control   macOS  C.narrow.after <= C.narrow.before, and no worse
  control   both   published checksum bytes identical on every recorded run
```

Four deliberate choices in those lines, all of them corrections to a source
design.

**The bar is against a program the mechanism must earn.**
`LOOP-PIPELINE.md:1824-1833` records that the pipeline design's REQUIRED Linux bar
"is met on this container today by an environment variable, so it no longer
discriminates between a working pipeline and no pipeline". A bar restated as
"beat an existing Whitefoot line" repeats that mistake. The recorded medians —
`RESULTS.md:706-731` gives `C.narrow.default` 3058.12 ms and `C.wide8.default`
1463.43 ms at 4 KiB uncached; `:758-783` gives 1664.13 and 1228.53 at 64 KiB cold —
are the *context*, not the bar: they say a 2.09x and a 1.35x improvement is what
success looks like with no change to the program.

**`--no-overlap` is not the Linux sequential control.**
`LOOP-PIPELINE.md:1824-1833` measured that its binaries still report
`enters=8192`. `WF_IO_RING=0 WF_IO_HELPERS=0` is the honest control;
`--no-overlap` stays as a lowering check.

**macOS is a no-regression bar, not a win.** The host serializes most concurrency
and saturates at the existing helper cap (`LOOP-PIPELINE.md:1574-1579`). Promising
a macOS win would be dishonest.

**The adversarial line is REQUIRED.** Without it §A.6.4's backoff is a paragraph
nobody checks.

**F4 — the leak test.** `read_heavy_wide8.wf` unchanged, and
`many_files_wide8.wf` keeps exactly the [PAR-1] window overlap it has today
(`docs/done/0098-blind-writer.md:65`, 1534 differing IR lines). A regression means
the new conditions leaked into the counted judgment.

**F5 — the discriminating pair.** Two maintained programs differing in one token,
both with checked published checksums, one permitted and one denied:

```whitefoot
  set at = at +wrap taken;      // permitted, carried-predicted
  set at = at +defined taken;   // denied, condition 8
```

A prediction test that passes because prediction never fired proves nothing.

**F6 — facts-off identity.** Acceptance and published bytes do not move with the
entailment state degraded. This judgment reads no fact by construction, so F6
should be free — and free things are pinned by a test, not asserted in a comment.

**F7 — the concurrent-truncation case.** Truncate the file mid-loop from a second
process. An admissibility test, not a byte-identity test: the program must not
trap, must not read past the new end, and must terminate. It is the test that
catches a validation comparing the wrong thing.

**F8 — the scoreboard.** §D. `sizes.wf` and `largest.wf` compile after their one
taught change and produce byte-identical output to the record in
`docs/done/0100-writer-defaults-2.md`.

**What falsifies the design.** If `C.narrow.after` lands outside 1.15x while the
counters show the attempts and discards they should, the mechanism is right and
the lowering is losing the time — reopen the driver. If the counters show few
attempts, the judgment denied and F1's ledger says why. If the program is fast and
F7 fails, the design is wrong and must not ship.

## A.10 What the writer writes differently, and why it is not a hidden trick

**For `read_heavy_narrow.wf`: nothing, at stage 2.** Its `data` buffer is declared
above all eight lane loops, so wall 3 applies, and it is exactly the case stage-2
privatization by interval proof exists for — the buffer's contents after the loop
are never read. This is the design's strongest writer claim and also its riskiest:
it depends on the derived range analysis `LOOP-PIPELINE.md:1412` budgets at ~800
lines **and** on a [SYS-8] amendment (§A.12 Q5), because
`spec/kernel-spec.md:2069` says a contract stating only which bytes *may* have
changed establishes no written byte, and [SYS-8]:2565 is exactly such a contract.
At stage 1 the program needs the same one-line move as the next one.

**For `writer3/sizes.wf` and `largest.wf`: one change, already taught.** P15
(`docs/patterns.md:346`). Applying it to `count_file` deletes the `scratch`
parameter and the `read_chunk` helper that existed only to satisfy [OWN-6]'s
one-statement reborrow region, and changes the effect row from
`reads(file, scratch), writes(scratch)` to `reads(file), allocates(heap)`. **That
is a signature change, not a loop-body change**: the edits are four lines in
`sizes.wf` (the `scratch` construction at `:56`, the call at `:120`, the header at
`:9`, the helper at `:3-7`) and four in `largest.wf` (`:62`, `:231`, `:9`,
`:3-7`), and [EFF-2]'s `expected_row` diagnostic prints the replacement row — the
diagnostic the same writer called "the single best diagnostic I met: it prints the
row to write". A source design that said "it needs the scratch moved inside the
loop and nothing else" undercounted this, and a helper whose *caller* folds the
chunk cannot be repaired by having the helper construct its own buffer, because a
buffer the helper constructs dies at the helper's return.

**Why this is not a hidden trick.** Four reasons, in the order the owner's rulings
state them.

1. *No batch API and no scheduling knob.* The writer writes the same loop, with
   the same `read_at`, the same `match`, the same `break`. There is no
   `read_ahead`, no depth argument, no attribute, no pragma, no environment
   variable a program must set to be fast. K is a runtime answer (§A.6.1) and
   `WF_IO_SHORT_READ_EVERY` is a test-only target-policy knob in the
   `WF_IO_HELPERS` class, never reachable from source.
2. *The compiler does not rewrite the writer's program.* It does not hoist the
   buffer, privatize a caller's `&uniq` argument behind their back, or turn the
   `break` into something else. Where the loop as written cannot be staged, it is
   not staged and the ledger names the place that stopped it. The one thing the
   compiler does that the source does not spell is allocate K slots, and §A.7 puts
   that number — broken down, including the nested case — on the writer's screen.
3. *Warning and teaching over silent transformation.* The change the writer makes
   is a change P15 already asks for, for a reason P15 already gives
   (`docs/patterns.md:342-344`: with one reused buffer, after a short read the
   bytes beyond `next` are the previous iteration's, so the program is genuinely
   order-dependent). The pattern was right before this design existed; this design
   is what makes taking it pay. And the compiler **must not** silently rewrite a
   hoisted buffer into a per-iteration one: `spec/kernel-spec.md:2069` forbids
   deriving that without a coverage proof, and a whole-place "write before read"
   rule would accept the loop-carried-scratch program of `LOOP-PIPELINE.md` §2.3
   and silently miscompile it. Warn and teach; the denial at condition 3 already
   names the buffer and the repair (`staged_permission.rs:423`).
4. *The alternative is worse.* A compiler that silently privatized `count_file`'s
   `&uniq scratch` parameter would change what the *caller* observes in that
   buffer after the call — and the row declares `writes(scratch)`, so the caller is
   entitled to those bytes. Stage 2 privatizes only where the interval proof shows
   nobody reads them. That proof is the difference between an optimization and a
   lie.

`spec/kernel-spec.md:2079` already puts "the storage an implementation reuses
across iterations for a construction whose value the body releases without
observing it" outside the observable set, so the writer writes the honest form and
the implementation is *permitted*, not obliged, to make it free.

## A.11 Implementation plan

Four batches. The first two are independently valuable and the third is the one
that needs the owner's answer to §A.12 Q1.

**A0 — `open_read`'s descriptor-status inspection. ~90 lines + ~80 conformance.**
`backend/emitter/system.rs`, `qualification.rs`, one [SYS-10] sentence, one
conformance case. Lands alone, before anything else, because it is a correctness
fix in its own right (§A.4.5) and because it is the premise A2 rests on. It is the
one item in Part A that changes an existing semantic ID and therefore carries a
[META-5] and a conformance cost; the merge records the specification bytes under
rule 4.

**A1 — `carried-closed`, no speculation. ~430 lines.** The `Carried(Closed)`
disposition, condition 8's recurrence shape test, the ledger spellings and column
width, and the judgment tests. **No terminating exit, no prediction, no discard,
no discardability property, and no host attempt the source order does not
perform.** It is what makes the ordinary "scan a buffer, act on each record" loop
stage, and it is what Part B's list loop needs (§B.7). If the owner declines
speculation, this is the half that still ships.

| component | file | lines |
|---|---|---|
| `Carried(Grade)` disposition, obligations 1-5 | `semantic/staged_permission.rs` | ~180 |
| condition 8 recurrence shape test and its denial text | `semantic/staged_permission.rs` | ~60 |
| ledger spellings, column width | `semantic/permission_ledger.rs` | ~30 |
| judgment tests, including every denial | `semantic/tests/staged_permission.rs` | ~160 |

**A2 — the terminating exit and `carried-predicted`. ~1,620 lines**, as a rider on
batch 0095's pipeline chassis (`LOOP-PIPELINE.md:1396-1424` prices that chassis at
~5,600, ~3,400 of it stage 1). Building A2 without the chassis means building the
chassis.

| component | file | lines |
|---|---|---|
| condition 2b, all five clauses | `semantic/staged_permission.rs` | ~120 |
| `discardable` on the [SYS-2] record and its resolution | `resolution/catalog.rs`, `semantic/` | ~40 |
| obligation 6, the contract-derived bound | `semantic/staged_permission.rs` | ~70 |
| judgment tests for both, including the fail-closed cases | `semantic/tests/` | ~190 |
| predict / compare / discard / reseed in the driver; the drain's discard path | `lowering/builder/pipeline.rs` | ~250 |
| the mispredict backoff | `completion/bridge.c` | ~30 |
| `PAR exit` and `PAR ring` lines; deleted denial texts | `semantic/permission_ledger.rs` | ~80 |
| two read-ahead counters | `completion/bridge.c`, `bridge.h` | ~40 |
| O1's scripted-target sweep and O2's fault knob | `backend/tests/`, `completion/bridge.c` | ~300 |
| conformance cases and verdicts | `tests/conformance/` | ~120 |
| the hand-written C prefetch ceiling and bench wiring | `research/experiments/io-completion-bench/` | ~200 |
| spec, `docs/patterns.md` P19, `docs/done/` record | `spec/`, `docs/` | ~180 |

**A3 — stage-2 privatization by interval proof.** Not this design's, but this
design names its precondition: the [SYS-8] amendment of §A.12 Q5. Without it
`read_heavy_narrow.wf` cannot pipeline byte-unchanged, and the claim must not be
made.

**The shared prerequisite neither part charges.** `LOOP-PIPELINE.md:1401` already
prices it: per-operation-record path storage and the `loan-released(name)`
milestone at `begin_submit`, ~180 lines. Part A needs it for the K-slot ring; Part
B needs it so K in-flight opens hold independent path copies (§B.7). Built once,
it serves both, and building either part without it produces a pipeline that races
the caller's name buffer. If only one part ships, it ships with this.

## A.12 Open questions for the owner — Part A

Each records a recommendation; none is decided here.

**Q1. Is a discarded host attempt acceptable?** This is the one genuinely new
thing in Part A: the implementation performs `pread`s the source-order execution
never performs. Nothing [SYS-2] declares observes them and §A.4.4's test makes
them free of state change, but they are real syscalls with real `atime` and real
bandwidth. The alternative that avoids them entirely is A1 alone — which admits
Part B's list loop and **not** the chunk loop, because a chunk loop's offset
provably depends on the outcome.
*Recommendation: approve, and ship A1 first anyway.* A1 is independently useful
and gives the owner a real fallback rather than a rhetorical one.

**Q2. Does a validated prefetch satisfy "at that point"?** `spec/kernel-spec.md:2072`
admits "an outcome that operation could deliver in the source-order execution at
that point". A prefetch reads earlier in wall clock. [SYS-11]:2627 already declines
to relate two reads of one physical file.
*Recommendation: state it explicitly in the §A.4.3 paragraph rather than relying
on the inference.* It costs one sentence and it is the only promise Part A
touches.

**Q3. Is the `claim`-in-P exclusion (2b.4) too blunt?** A claim whose predicate
does not reach a predicted place is harmless, and the narrower condition is
derivable from the same footprint survey.
*Recommendation: ship the blunt version.* It is one line, no program in this
repository pays for it (`docs/done/0098-blind-writer.md:40`), and narrowing later
is a widening of the permitted set, which moves no verdict.

**Q4. Should `ReadFailed` be a terminating exit?** It is admitted by the same
argument as `ReadEnd`, and both writers wrote `break @chunk` in both arms. But a
failing read is where a writer might later want to retry, and discarding K-1
in-flight reads on the first `EINTR`-class failure may not be what they want.
*Recommendation: admit it.* Refusing it would deny the writers' actual programs,
and a retry loop is a different shape that will need its own judgment anyway.

**Q5. The [SYS-8] amendment stage 2 owes.** `spec/kernel-spec.md:2069` says a byte
counts as written "only where that contract fixes that the footprint changes it: a
contract stating only which bytes of a buffer *may* have changed [SYS-8]
establishes no written byte", and [SYS-8]:2565 is exactly such a contract. So
`read_at` supplies no replication coverage today and no hoisted buffer can be
privatized.
*Recommendation: amend [SYS-8]:2565 to state the change exactly on the successful
edge, as part of A3, and do not claim `read_heavy_narrow.wf` pipelines
byte-unchanged before it lands.*

**Q6. Float recurrences.** Condition 8 admits `fadd.strict` on a carried place,
because this rule re-executes rather than recombines and uses no associativity —
unlike [PAR-2], which refuses float folds.
*Recommendation: confirm.* It is correct, and it is the first place in the
language where a float fold rides a permitted overlap, so it should be a decision
rather than a consequence.

**Q7. Should the window be capped by device queue depth rather than by memory?**
For one file, 64 outstanding 64 KiB reads is 4 MiB and a queue depth the device may
not reward. `wf__completion_window` answers from bridge capacity alone.
*Recommendation: leave it as runtime policy and measure with F3 at several K.* It
is deliberately not a language surface either way.

---

# Part B — bytes to path

## B.1 The defect, stated as an asymmetry rather than a feature request

`writer3/sizes.wf` reads a list file into `content`, splits it on newlines, and
for each line opens the file it names. Line 117, verbatim:

```whitefoot
match open_file<'g2, 'n2>(permit: move permit2, root: &'g2 cwd, name: &'n2 content, start: begin, end: index) {
```

**That call works.** It is `open_file`'s ordinary caller-owned name range
(`spec/kernel-spec.md:2681`), the writer found it without a diagnostic, and it
opens `a.txt`, `b.txt`, `empty.txt`, `big.bin`. It returns
`Err(InvalidPath(code: 0_u32, origin: 0_u8))` for `sample/a.txt`, because
[SYS-14]:2683 validates the range as one *component* and a separator is refused
before any host call (`emitter/system.rs:1747`).

Forty-seven lines earlier, the same program opens a multi-component relative path
successfully:

```whitefoot
match open_read<'g, 'p>(permit: move permit, root: &'g cwd, path: &'p path) {
```

`path` came from `arg_get` through `relative_path`. [PATH-1]:2390 "preserves every
admitted code unit exactly — including `.` and `..` components and every
separator", and `open_read`'s qualified flags are `file_open_flags: 0`
(`qualification.rs:996`), documented at `:733` as "`open_read`'s
namespace-following relative-path open". So `sizes ../nested/list.txt` already
resolves two components and a `..` through one `openat`.

**The same bytes open a file when they arrive on argv and do not when the program
reads them out of a file.** That is the whole of Part B. Nothing about resolution
is missing, nothing about validation is missing, and no host capability is missing
— `file_adapter.c:172-190` already hands `openat` a whole path and `:611` already
stages it per operation record. What is missing is a *producer*: a route from
program-owned bytes to an open.

The workaround is recorded (`docs/done/0100-writer-defaults-2.md:787-801`) and
visible in `writer3/largest.wf`: a `buffer_vacant<DirectoryRead>(512_u64)` stack
at `:55`, `replace stack[k] = Some<DirectoryRead>(…)` at `:78`, `:91` and `:200`,
a 65,536-byte `spath` buffer holding one 128-byte path slot per level at `:56`, a
parallel `slen` at `:57`, a hand-built separator write at `:173`, and three copy
loops at `:94-103`, `:160-168` and `:202-211` whose entire job is to reassemble a
path the program then cannot use as a path. About sixty lines of a 216-line
program are that workaround.

## B.2 The decision

**Two new operations, `open_file_path` and `open_directory_path`. `open_file` and
`open_directory` are untouched. No new type.**

Four decisions, each over a named alternative.

1. **Add beside, do not widen.** Widening `open_file`'s admitted range in place is
   the smallest change and it is refused. §B.8.1 is the attack: `writer3/sizes.wf`
   **unmodified** would open `/etc/passwd` from a list-file line reading
   `../../../etc/passwd`, where today it returns `Err(InvalidPath)` before any host
   call. Nobody edited the program, nobody opted in, and no new spelling appears in
   the source. That is precisely the owner's "no path traversal by accident". Two
   operations cost two catalog rows and they buy an "off" position, which is what
   makes the differential oracle of §B.9.1 exist at all.

2. **No new type.** `spec/kernel-spec.md:2680` states the exclusion and
   [HOST-3]:2381 predicts what a different producer would cost: "a distinct
   owned-backing string resource with its own release action and its own type
   contract, because storage class is a function of type [STOR-1]". That is a new
   nominal, a [SYS-5] release row, a [STOR-1] storage class, a heap copy per path,
   and a new affine value inside every loop body — hence a new place for [PAR-3] to
   classify, so §B.7's staging argument would have to be redone. And no writer
   program in this repository wants to *hold* a path; they want to open one. The
   operation takes a caller-owned byte range and validates it as a relative path,
   exactly as `open_file` takes a caller-owned byte range and validates it as a
   component.

3. **One path semantics: validate by exactly [PATH-1]'s test, plus the length
   limits.** `.`, `..`, repeated separators and a trailing separator are
   **admitted**. This is `runtime-first`'s principle and it is the one both judges
   endorsed against the other two designs: a stricter byte route creates a second
   grade of well-formed path, which is the defect [META-2] and [META-4] exist to
   prevent. §B.8.2 is why refusing `..` in particular gets safety *backwards*.

4. **Symbolic links are followed, at every component including the last.** The
   path route carries no `O_NOFOLLOW`, so it agrees with `open_read`, which is what
   the same bytes already do on the argv route. Adding `O_NOFOLLOW` would make
   `open_file_path` refuse a symlinked file that `open_read` opens — a difference no
   writer could predict from the operation's name, and a *new* asymmetry created
   while removing one.

## B.3 The writer-facing form, as it would compile

Two operations, named parallel to the pair [SYS-14] already declares:

```text
open_file_path['p, 'n](permit: own FilePermit, root: &'p DirectoryRead,
                       path: &'n buffer<u8>, start: own u64, end: own u64)
    -> result: own Result<ReadFile, IoError>
    reads(permit, root, path), writes(permit)

open_directory_path['p, 'n](permit: own FilePermit, root: &'p DirectoryRead,
                            path: &'n buffer<u8>, start: own u64, end: own u64)
    -> result: own Result<DirectoryRead, IoError>
    reads(permit, root, path), writes(permit)
```

`writer3/sizes.wf:117` becomes, in full, with every surrounding line unchanged:

```whitefoot
region 'g2 {
  let permit2 = reserve_file<'g2>(factory: &uniq 'g2 files);
  region 'n2 {
    match open_file_path<'g2, 'n2>(permit: move permit2, root: &'g2 cwd,
                                   path: &'n2 content, start: begin, end: index) {
      Ok(value: target) => {
        region 'q {
          let size = count_file<'q>(file: &'q target);
          set total = total +wrap size;
        }
      }
      Err(error: problem) => {
        region 'w4 {
          let sent = write_once<'w4, 'w4>(output: &uniq 'w4 out, source: &'w4 punct,
                                          start: 1_u64, end: 2_u64);
        }
      }
    }
  }
}
```

**One identifier changes: `open_file` becomes `open_file_path`, and `name:`
becomes `path:`.** The permit ceremony is the ceremony the writer already wrote.
The region structure is the structure they already wrote. The `Err` arm is the arm
they already wrote and it still prints `?` — for `InvalidPath` now as well as for
`NotFound`. `list.txt` is used unchanged and `flat.txt`, the workaround file, is
deleted in the same change: a workaround kept beside its fix is rot.

`sample/a.txt` opens. `./sample/a.txt` opens. `sample/../sample/a.txt` opens the
same object. `sample//a.txt` opens. `/etc/hosts` returns
`Err(InvalidPath(0, 0))` before any host call. `../../etc/passwd` opens
`/etc/passwd` — and §B.8 is the argument that this is correct rather than a hole.

`largest.wf`'s descent collapses too, and this is the optional half. Keeping the
full path in `full` as the program already does, the directory push at `:198` and
the file open at `:228` both become opens against the *root* `cwd` over
`full[0..flen]`, and then `stack`, `spath`, `slen`, `cur`, the hand-built
separator write at `:173`, the three copy loops, and `buffer_vacant<DirectoryRead>`
all become dead: about sixty lines. It gains one cost — opening `a/b/c/d.txt` from
the root re-resolves four components per open where the descent resolved one — and
§B.5 is why both routes must exist and neither replaces the other.

## B.4 The exact specification delta

Three rules amended, one paragraph added, one guarantee added, four lists
extended. No new rule (137 remain), no new type, no new backing class, no new
release action, no new `IoError` class, no new static obligation. **Writer
operation spellings +2; declaration records +2 operations, +4 region parameters,
+10 value parameters.**

### B.4.1 [SYS-14] — the exclusion sentence

`spec/kernel-spec.md:2680-2681`, verbatim:

> This specification declares no operation turning an enumerated name into a
> `HostString` or a `RelativePath`, because a name's backing is not the
> command-lifetime argument snapshot [HOST-3] and a path value is an inline lease
> over that snapshot [PATH-1].
> `open_directory` and `open_file` therefore take a caller-owned name range rather
> than a path value, and path composition remains the DEFERRED addition [PATH-1]
> states.

becomes:

> This specification declares no operation turning an enumerated name, or any
> other bytes a program reads, into a `HostString` or a `RelativePath`, because
> such backing is not the command-lifetime argument snapshot [HOST-3] and a path
> value is an inline lease over that snapshot [PATH-1].
> `open_directory` and `open_file` therefore take a caller-owned name range rather
> than a path value; `open_directory_path` and `open_file_path` take a
> caller-owned relative-path range on the same terms and resolve it through the
> target's own directory-relative facility [PATH-2]. Neither route yields a path
> value, no operation of this version composes, decomposes, joins, normalizes, or
> displays one, and path algebra remains the DEFERRED addition [PATH-1] states.

`spec/kernel-spec.md:2676` — "A name is one path component … so no record a
program reads can name more than one component" — **stays unchanged**. It states a
host fact about an enumeration record, not a restriction on opens. §B.11 Q6 asks
the owner to confirm the reading, because its final clause was written when it
also implied what could be opened, and after this change it no longer does.

`spec/kernel-spec.md:2687`'s symlink sentence also stays unchanged: it is about
`open_directory` and `open_file`, whose contracts do not move.

### B.4.2 [SYS-14] — the new paragraph

Added after `spec/kernel-spec.md:2687`:

> `open_file_path` and `open_directory_path` each consume one `FilePermit`, borrow
> one `DirectoryRead` as `root` through `&`, borrow one caller-owned initialized
> `buffer<u8>` as `path` through `&`, and name a half-open range `[start, end)` in
> it [SYS-8]. Each first discharges [SYS-8]'s two static range obligations; neither
> has a runtime range check or `traps` effect.
>
> Each then validates `[start, end)` as one relative path before any host call, by
> exactly the test [PATH-1] fixes for a relative path constructed from a host
> string, together with two stated length limits. A component is one maximal
> non-empty run of code units containing no target separator. The range yields
> `Err(InvalidPath(code: 0_u32, origin: 0_u8))`, no host call, and no resource
> value when it is empty, when it contains a NUL code unit, when it begins with a
> target-root prefix [PATH-1], when a component is longer than the target's
> component limit, or when the range is longer than the target's path limit. A `.`
> component, a `..` component, a repeated separator, and a trailing separator are
> admitted unchanged. Validation preserves every admitted code unit exactly and
> performs no normalization, canonicalization, case folding, prefix stripping, or
> component collapse [PATH-1]. This validation is not a confinement check and no
> rule of this specification makes one of it.
>
> A valid range resolves through the target's own directory-relative facility in
> exactly one host attempt, whatever its component count, and a failure of that
> attempt yields the target-mapped [SYS-7] error. Resolution is process-equivalent
> [PATH-2]: `.` and `..` components, symbolic links, reparse points, and mount
> transitions are followed exactly as the surrounding process namespace follows
> them, in every component and in the last, so a resolved object may lie outside
> the directory `root` names. That is the complete promise these operations make.
> The confined directory state type remains DEFERRED [PATH-2].
>
> After `open_file_path` obtains a provisional descriptor, descriptor-status
> inspection is required before publication, with the outcomes and the single
> native close attempt this rule already fixes for `open_file`. On success
> `open_file_path` returns an independent `ReadFile` for the resolved regular file
> and `open_directory_path` returns an independent `DirectoryRead` for the resolved
> directory.
>
> The path limit is target data fixed by that target's qualification record
> [QUAL-1]; the limit used by this version's approved implementations is 1023 code
> units on both families.

The last sentence follows the precedent of `spec/kernel-spec.md:2677`, which
states the component limits in the specification rather than leaving them
implicit. A writer whose list file holds a 1500-byte path must be able to read what
will happen.

### B.4.3 [PATH-2] — the resolution sentence

`spec/kernel-spec.md:2395`, verbatim:

> A directory-relative operation resolves either one relative path value or one
> caller-supplied single path component [SYS-14]; both are resolved through the
> target's own directory-relative facility and neither is concatenated onto a
> prefix.

becomes:

> A directory-relative operation resolves one relative path value, one
> caller-supplied single path component, or one caller-supplied relative path
> range [SYS-14]; all three are resolved through the target's own
> directory-relative facility, all three are admitted by the same [PATH-1] test,
> and none is concatenated onto a prefix.

This is the amendment a reader is most likely to miss, because [PATH-2] is not
where anyone looks for a [SYS-14] operation, and the sentence is an exhaustive
"either … or" that a third form silently violates. It was found by
`runtime-first`'s own in-place correction and independently by `writer-first`;
`proof-first` missed it.

### B.4.4 [PATH-1] — one added sentence, and one recommended clause

Add after `spec/kernel-spec.md:2391`:

> Admitting one externally supplied multi-component relative path is not path
> algebra: this specification declares no operation that decomposes, joins,
> normalizes, or displays a path, and an operation validating a caller-owned
> code-unit range as one relative path assembles nothing.

**Recommended, and it is §B.11 Q2:** [PATH-1]:2387 has no length clause, so
`relative_path` over a 4000-byte argument succeeds and the length becomes a host
outcome. Adding the path limit to [PATH-1] is what makes "one path semantics"
literally true rather than true in five clauses out of six. It may move a
`relative_path` conformance verdict, which is why it is a question.

### B.4.5 [QUAL-2] — the path limit as a stated target guarantee

Following `spec/kernel-spec.md:2420`'s fourth guarantee, a fifth:

> The fifth is a stated path limit for the directory-relative semantic IDs: a
> qualified target names the greatest length, in code units, of a relative path its
> directory-relative facility accepts, and the compiler's admitted validation
> refuses a longer range before any host call so that the length is never a host
> outcome.

[QUAL-2]'s third guarantee (`spec/kernel-spec.md:2419`) already covers "the
target's own directory-relative resolution facility [PATH-2] for every semantic ID
that resolves a relative path or one caller-supplied component against a
`DirectoryRead`", so the two new IDs join an existing guarantee rather than adding
a sixth.

### B.4.6 The four list amendments

Each adds two names to a set the specification already enumerates. `writer-first`
found the third; no other design did, and without it a helper returning
`own Result<ReadFile, IoError>` cannot `propagate` a path open.

- `spec/kernel-spec.md:2537` — the complete range-bearing system-operation set.
- `spec/kernel-spec.md:2549` — the empty-range rule for `open_directory` and
  `open_file`; §B.4.2's first refusal is that sentence.
- `spec/kernel-spec.md:2519` — the `propagate` chain, "exactly `open_read`,
  `write_once`, `open_directory`, `open_directory_source`, and `open_file`".
- `spec/kernel-spec.md:2605`, `:2609`, `:2611` — [SYS-10]'s permit list, borrow
  list, and fresh-owner list.

`spec/kernel-spec.md:2293`'s `may-suspend` list gains both names, and `:2294`'s
`loan-released(path)` sentence gains their `path` borrow beside `open_read`'s
`path` and `open_file`'s `name` — forming the request copies the admitted range
into compiler-owned storage, so the borrow is released before target transfer,
exactly as for a name.

### B.4.7 The [SYS-2] inventory arithmetic, done rather than asserted

`spec/kernel-spec.md:2285` counts parameters as records:

> The inventory is therefore exactly eighteen nominal types, forty enum-variant
> constructors, sixty-three variant fields, sixteen operations, twenty-two
> operation region parameters, and forty-four operation value parameters.

Each new operation carries two region parameters and five value parameters
(`permit`, `root`, `path`, `start`, `end`), exactly as `open_file` does
(`resolution/catalog.rs:947-973`). So the sentence becomes "**eighteen
operations, twenty-six** operation region parameters, and **fifty-four** operation
value parameters", and the record count moves from 203 to **219**, not to 205. A
count that does not move by exactly that arithmetic is a defect. (`proof-first`
caught its own error here; a design that says "+2 records" has not counted.)

### B.4.8 The [META-5] delta shape

> Numbered rules +0/-0 (137 remain); grammar productions +0/-0; keywords +0/-0;
> opaque system nominal spellings +0/-0 (ten remain); outcome types +0/-0;
> `IoError` classes +0/-0 (28 remain); runtime-trap families +0/-0; entry forms
> +0/-0. Writer operation spellings **+2/-0**; system operations and declaration
> records **+2 operations, +4 region parameters, +10 value parameters** (219
> remain). [SYS-14] is amended to declare `open_file_path` and
> `open_directory_path`, which differ from `open_file` and `open_directory` only
> in validating their caller-owned range as one complete relative path rather than
> as one component and in not refusing a symbolic link at the final component, and
> to state that both resolve that range in one act of the target's own
> directory-relative facility with the process equivalence [PATH-2] already fixes.
> [PATH-2] is amended to admit a third resolved form. [PATH-1] is amended by one
> sentence distinguishing admission of an externally supplied path from path
> algebra. [QUAL-2] gains a fifth stated target guarantee. [SYS-8]'s range-bearing
> set, [SYS-2]'s `may-suspend` and `loan-released` sentences, [ERR-3]'s propagate
> chain, and [SYS-10]'s three lists each gain both names, with no change to any
> clause. **No existing operation's contract changes**, so no accepted program
> changes meaning and no conformance verdict moves.

## B.5 The judgment, the permit ceremony, and why both routes stay

**There is no new judgment, and that is the design's strongest claim on a
writer's time.** A call to `open_file_path` carries exactly the two [ENT-6]
obligations `spec/kernel-spec.md:2540` states for every member of the
range-bearing family: `start <= end`, then `end <= len(deref(path))`, "queried in
the caller's pre-transfer state" and "derived independently". These are the
obligations `writer3/sizes.wf` already discharges at line 117 for `open_file`,
with the same operands, by the same ordinary `if` branches, against the same
hoisted length fact P16 teaches (`docs/patterns.md:491`). Substituting the new
operation changes no proof the writer must produce.

Everything §B.4.2 refuses is refused at **runtime** and reported as a typed
`Err(InvalidPath)`, not at compile time — because the bytes are not available at
compile time, and because [ERR-4]'s recoverable-outcome route is what a program
handling a malformed list file needs. There is no new entailment fact, no new
obligation class, no new `claim`, no new [DIAG-2] retention, and no new rule for
the checker. Part B adds **zero** semantic-checker lines; the resolution catalog
gains two rows and the checker sees two more members of a family it handles.

The one thing the compiler must not do is treat a validation refusal as a source
rejection. `CLAUDE.md` is explicit — "Compiler capability, an internal error, a
timeout, or an unimplemented feature is not a source-language rejection" — and so
is [QUAL-1]:2408. A path is data.

**The permit ceremony is unchanged, and one attempt is one permit.**
`spec/kernel-spec.md:2605`: "A `FilePermit` authorizes exactly one attempt …
and consumes it on every success or recoverable-failure outcome." A
multi-component open is one attempt, so it costs one permit and one host call,
whatever the depth. That is the answer to the obvious objection that a
four-component path should cost four permits: the permit is not a resource meter,
and `:2605` says so outright — "Reserving it promises no native descriptor,
handle-table entry, kernel memory, or host quota" — while actual host pressure
still arrives as `ResourceExhausted` in the typed result. A pre-host refusal
consumes the permit too, which is the rule `open_file` already follows for an
invalid component (`:2683` with [SYS-10] still consuming).

| route over `sample/deep/a.txt` | permits | descriptors live at the leaf | host calls |
|---|---|---|---|
| descent, `writer3/largest.wf:196-200` per level | one per component | one per live level (`spec/kernel-spec.md:2691`) | one per component |
| one range, this design | **one** | **one** | **one** |

**Both routes stay, and the writer chooses.** `open_file` by single component
keeps two properties the path route does not offer: its terminal symbolic link is
not followed (`spec/kernel-spec.md:2687`; `component_file_open_flags` carries
`O_NOFOLLOW`, `0x0002_0800` on Linux x86-64 and `0x0000_0104` on Darwin,
`qualification.rs:922/971`), and each level's descriptor is held, so a descent
resolves each component exactly once and a walker that opens many files under one
directory pays one `openat` per file rather than one per component per file. There
is a third, purely diagnostic difference: the descent tells the writer *which*
component failed, because each level has its own outcome, where one call returns
one `NotFound` for the whole range. `docs/patterns.md` P18 (§E) is where that
choice is taught rather than left to be discovered.

## B.6 The lowering and the errno map

### B.6.1 What is emitted

`compiler/src/backend/emitter/system.rs` already emits every piece.

**The scan.** `component_validation` (`system.rs:1730`) is one byte loop refusing
NUL and the separator. `path_validation` is the same single pass with one counter
and one preamble block added:

```text
measure:
  %oversize = icmp ugt i64 %extent, {path_limit}      ; was component_limit
  %vacant   = icmp eq  i64 %extent, 0
  br i1 %unusable, label %invalid, label %rooted
rooted:                                               ; [PATH-1] target-root prefix
  %first = load i8, ptr %text, align 1
  %absolute = icmp eq i8 %first, {root_prefix}
  br i1 %absolute, label %invalid, label %scan
scan:
  %index = phi i64 [ 0, %rooted ], [ %index.next, %scan.step ]
  %run   = phi i64 [ 0, %rooted ], [ %run.next,   %scan.step ]
  %byte  = load i8, ptr (%text + %index), align 1
  %terminating = icmp eq i8 %byte, 0
  br i1 %terminating, label %invalid, label %scan.step
scan.step:
  %separating = icmp eq i8 %byte, {root_prefix}
  %run.next = select i1 %separating, i64 0, i64 (%run + 1)
  %long = icmp ugt i64 %run.next, {component_limit}
  br i1 %long, label %invalid, label %scan.next
```

Three facts about that block. It is **one pass**, so the per-byte cost of an open
is unchanged and §B.9.3's control bar can require the difference to be
unmeasurable. It reads **only target data** — `root_prefix`
(`qualification.rs:990`, `b'/'`), `component_limit` (`:731`), and the new path
limit — so it is not a source-shape test. And it runs **before the copy and
therefore before the host call**, the property `emitter/system.rs:1727-1729`
states today. A Windows-family target's prefix set is a sequence rather than one
code unit, so its `%rooted` block would test a sequence; no such target qualifies
(`qualification.rs:917-984` lists four Unix triples) and this design fixes nothing
about it.

`invalid_component` (`system.rs:1704`) is reused unchanged and already yields
`InvalidPath(0, 0)` with both detail fields zero, because no native facility ran.

**The staging slot, and why the path limit is 1023.** `%component = alloca
[component_limit + 1]` (`system.rs:1957`, `:1965`) becomes `%path = alloca
[path_limit + 1]`. The number is chosen **by the runtime, because the runtime has
already chosen it**: `file_adapter.h:65` is `#define WF_FILE_PATH_CAPACITY 1024u`,
whose comment reads "1024 is Darwin's whole `PATH_MAX`. Storage is bounded and
static because a submission may not allocate", and `:70`'s `wf_file_stage_path`
**refuses rather than truncates** a longer name ("truncating a name would resolve
a different file"), which demotes the open off the completion path to the direct
blocking path and counts it in `wf__completion_file_demoted_opens`
(`file_adapter.h:59-64`, `bridge.h:171`).

So: **1023 on both families**, so every admitted path fits `path_storage[1024]`
(`file_adapter.h:287`, `linux_io_uring.h:83`) with its NUL, no admitted open ever
demotes for length, and the record layout does not move. The alternative — the
host's own `PATH_MAX`, 4096 on Linux — would either raise `WF_FILE_PATH_CAPACITY`
to 4096 (4 KiB × 64 slots of static per-ring storage where 1 KiB × 64 is enough)
or admit paths that always demote, silently falling out of exactly the pipeline
Part A is about. One source design chose 4096, sized a 4097-byte stack slot, and
published +3841 B of frame for paths that structurally could not use the
completion path; both judges called it. §B.9.2's F-B4 turns the choice into a
checked assertion.

The slot stays a **fixed** `alloca`, not a dynamic one sized to the range.
`stack_ledger.rs:38-43` says what a dynamic frame costs: "Every frame this
compiler has ever emitted is `static`, which is what makes the arithmetic below
exact rather than an estimate". Trading an exact stack ledger for a kilobyte in
the one function that opens a path is the wrong trade in a language whose stack
story is a published number. On Linux the slot grows from 256 B to 1024 B; on
Darwin it grows by one byte.

**Per-slot storage.** `emitter/system.rs:1959-1964` already states in the tree
that a submitting wrapper must index this buffer per outstanding operation rather
than share one alloca. That is true today for one component and does not become
more true for a path; it is batch 0095's own per-slot work, and the adapter side
already holds it per record — `native_adapter_probe.c:484-485` asserts
`entry->request.buffer.path == entry->path_storage`.

**The host call: zero new lines.** `file_adapter.c:172-190` passes
`request->operation.open_at.path` to `openat` as a whole NUL-terminated path and
never looks at it; `linux_io_uring.c:631` stages it the same way. **The runtime
change for Part B is zero lines**, and §B.9.2's F-B8 makes that a review invariant
rather than a claim.

**The flags.** Two new fields beside the four at `qualification.rs:731-740`:
`path_file_open_flags` = `O_NONBLOCK` (`0x0000_0800` Linux, `0x0000_0004` Darwin)
and `path_directory_open_flags` = `O_DIRECTORY` (`0x0001_0000` Linux,
`0x0010_0000` Darwin) — the existing `component_*` values with `O_NOFOLLOW`
removed, per decision 4 of §B.2.

**The inspection.** `expected_kind = WF_FILE_EXPECT_FILE`, which already triggers
the `fstat` at `file_adapter.c:195-200`. `O_DIRECTORY` binds the final component,
so an interior non-directory reaches source as `NotDirectory`.

### B.6.2 The errno map

**No new `IoError` class, and not one row of either target table changes.**
[SYS-7] fixes twenty-eight classes (`spec/kernel-spec.md:2523`) and nine files in
the tree match `IoError` exhaustively — five conformance cases,
`tests/programs/wfgrep.wf`, and three research programs — so a new class would
cost every one of them for no writer benefit. What changes is only *which*
existing classes a multi-component range can reach.

| native | Linux / Darwin | reached by | class | table row |
|---|---|---|---|---|
| — | — | empty range, NUL, root prefix, over-limit component, over-limit path | `InvalidPath(0, 0)` | before any host call |
| `ENOENT` | 2 / 2 | any component missing — **interior is new** | `NotFound` | `qualification.rs:320` / `:281` |
| `EACCES` | 13 / 13 | search permission on an **interior** directory is new | `PermissionDenied` | `:321` / `:282` |
| `ENOTDIR` | 20 / 20 | an interior component is not a directory; also `a/b/` where `b` is a regular file | `NotDirectory` | `:323` / `:284` |
| `ELOOP` | 40 / 62 | symlink-depth exhaustion (new; the final-component refusal is gone with `O_NOFOLLOW`) | `InvalidPath` | `:329` / `:290` |
| `ENAMETOOLONG` | 36 / 63 | not reachable for the supplied range — validation refuses first — but still reachable when interior symlink expansion exceeds the host's own limit | `InvalidPath` | `:329` / `:290` |
| `EMFILE` / `ENFILE` | 24 / 23 | unchanged | `ResourceExhausted` | `:341` / `:305` |
| `EISDIR` | 21 / 21 | unchanged; `open_file_path` normally synthesizes `IsDirectory` from its own inspection | `IsDirectory` | `:324` / `:285` |
| `EXDEV` | 18 / 18 | not reachable from `openat`; listed to show it is unmoved | `CrossDevice` | `:345` / `:309` |
| anything else | — | unchanged | `Other` (`spec/kernel-spec.md:2525`) | — |

`ELOOP` mapping to `InvalidPath` on both families is the fact that makes this
cheap: a symlink loop reaches source as the same class as a syntactically invalid
path, so a writer's `Err(InvalidPath)` arm covers both and no exhaustive match
anywhere gains an arm.

**Five distinct pre-host refusals share one indistinguishable value**,
`InvalidPath(code: 0_u32, origin: 0_u8)`. That is [SYS-7]:2529's "Each field is
zero when the target supplies no value for it", and the two zeros are themselves
the message: `code` is the native errno (`:2528`) and `ORIGIN_NONE` with `code`
zero is the operation saying *this never reached the host*, which is the first
fact a writer debugging a refused path needs. `:2531` makes the detail diagnostic
data and not a portable discriminator. Whether the reasons should be
distinguishable is §B.11 Q3; teaching the reading is P18's job.

**`ENAMETOOLONG` is the reason [QUAL-2] gains a guarantee, not the reason it does
not need one.** The validation makes the *supplied* length a source outcome
instead of a host outcome; it cannot make symlink expansion one, and the spec must
not pretend it can.

Nothing here is a claim or a trap. Every row is a typed `Err` value under [ERR-4]
(`spec/kernel-spec.md:1463`), reached through the same cold outcome mapper
[QUAL-3]:2426 already fixes for these operations.

## B.7 Composition with the pipeline, and the ledger

The program Part B unblocks is a loop over paths read from a list file, and the
question is whether that loop stages under [PAR-3] with Part A's judgment. It
does — and, this is the finding that makes the two parts one design, it stages
*more cheaply* than the generated-name loop P15 was written for.

Take `writer3/sizes.wf`'s scan loop as it becomes with `open_file_path`. Its body
reserves a permit, opens a path out of `content`, reads the file, and folds:

| place | disposition | which clause of `spec/kernel-spec.md:2060` |
|---|---|---|
| `content`, the list buffer | read-only | first: no footprint of B writes it and every loan on it is shared |
| `cwd`, the root | read-only | first |
| `files`, the factory | serialized-P | second: every element belongs to P alone, and `reserve_file`'s loan "ends when that inline operation returns" (`:2603`) |
| `begin`, the line cursor | **carried-closed** | §A.4.2 — the recurrence `set begin = index +wrap 1_u64;` names no payload binder |
| `total` | serialized-E | written in E only, committed in iteration order (`:2064`) |

Three things about that table.

**`begin` is the reason this loop needs Part A at all.** Without the `carried`
disposition it is a place the prologue reads and the remainder writes, which none
of `:2060`'s three alternatives covers, and the loop denies at condition 5.
**That is the sharpest argument for building A1 even if the owner declines
speculation**: `carried-closed` is what makes the ordinary "scan a buffer, act on
each record" loop stage, and it needs no discarded host attempt at all.

**A path in a read-only buffer needs no replication.** The condition that would
have denied is `spec/kernel-spec.md:2059`: `open_file_path` retains a borrow of
`content` past its own submission. Its third alternative admits it — "on a place
no footprint of B writes" — and `content` is exactly that, because the list file
is read once before the loop. Where P15's per-iteration `name` buffer needs a ring
slot each (`LOOP-PIPELINE.md:726`, 512 B of ring), the list-file loop needs none.

**Two mechanical facts must hold and both already do.** [SYS-2]:2294 releases the
name borrow before target transfer, and `file_adapter.c:611` copies the path into
the per-operation record before the call, so K in-flight opens hold K independent
copies of their own path bytes and never race the caller's buffer. The per-slot
cost is `WF_FILE_PATH_CAPACITY` = 1024 bytes, which at K = 32 is 32 KiB, and the
`PAR ring` line of §A.7 is where the writer reads that number — broken down, so
1024 of the 1104 bytes is visibly the staged path.

```text
PAR stage       sizes.wf:102   for   permitted   staged at open_file_path(permit: move permit2, …);
                                                 5 places classified
PAR place       sizes.wf:102   read-only         &'n2 content    no footprint of the body writes it; the open's
                                                 retained borrow is shared
PAR place       sizes.wf:102   read-only         &'g2 cwd        the root is borrowed shared by every open
PAR place       sizes.wf:102   serialized-P      &uniq 'g2 files the factory loan ends when reserve_file returns
PAR place       sizes.wf:102   carried           let begin = 0_u64;  advanced from terms the prologue already
                                                 holds, so the recurrence is hoisted above the submission and no
                                                 outcome is predicted
PAR place       sizes.wf:102   serialized-E      let total = 0_u64;  written only in the remainder, committed in
                                                 iteration order
PAR ring        sizes.wf:102   for   1,104 bytes per slot (1,024 staged path, 80 operation record); up to 32 slots,
                                                 allocated once at loop entry; the number of slots is chosen by the
                                                 runtime at entry and has no source spelling
```

**The writer's own loop still denies, for a reason Part B does not touch.**
`writer3/sizes.wf:102`'s `for @scan` writes each name to `out` *before* opening it
(`:109`), so the cut lands on that `write_once` and the retained exclusive loan on
`out` denies at condition 3 (`staged_permission.rs:60-72`). That is D1 (§D), it is
about `Output` and not about paths, and neither part of this design fixes or
worsens it. Saying so is the difference between a design that composes and a design
that claims to.

**What the ledger deliberately does not gain.** A per-call-site line for the
`%path` alloca of a non-staged open. It is stack storage inside a wrapper the
compiler inlines, it is the same size for every open in the program, and a line per
open site would be noise of the kind `docs/patterns.md:443-450` keeps off the
default channel. The number belongs in the target's qualification data beside
`component_limit`, and §B.11 Q5 asks whether a `--qualification-report` should
print it. The **`STACK` ledger** does move, by one kilobyte per opening frame on
Linux, and `stack_ledger.rs:94-137`'s `cycle` line is where a recursive walker that
opens by path at every level reads its own depth as a number instead of
discovering it as a crash.

## B.8 The safety argument, and the attacks that shaped it

The owner's ruling: expressive power, performance, and safety, all three, with no
path traversal by accident. Part B's safety argument has an unusual shape, because
two of the three source designs failed it in *opposite* directions and the
corrections point at each other.

### B.8.1 Widening `open_file` turns existing call sites into traversals

**The attack** (both judges, independently; against `runtime-first`, which widened
`open_file`'s admitted range in place). The program is the writer's own,
unmodified — `writer3/sizes.wf:117`:

```whitefoot
match open_file<'g2, 'n2>(permit: move permit2, root: &'g2 cwd,
                          name: &'n2 content, start: begin, end: index) {
```

`content` holds a list file the program read. **Today** a line reading
`../../../etc/passwd` returns `Err(InvalidPath(0,0))` before any host call and the
program prints `?`. **After widening**, the same binary, the same source, the same
list file opens `/etc/passwd` and publishes its size. Nobody edited the program,
nobody opted in, no new spelling appears in the source. That is exactly the
owner's "no path traversal by accident".

The separator refusal at `spec/kernel-spec.md:2683` is today the only thing
standing between an unaudited name and an escape, and widening removes it from
every call site at once. A second, smaller loss: a walker handing an *enumerated*
entry name to `open_file` currently gets a belt-and-braces check against
[SYS-14]:2676's one-component guarantee, which is a shim/target property the
language otherwise takes on trust; widening deletes that check too.

The corpus proves it as well as the program does. `tests/conformance/manifest.jsonl`
holds `sys14-open-directory-component`, whose whole content is that behaviour: "A
range containing a target separator is not one path component, so it yields
InvalidPath with both detail fields zero, no host call, and no DirectoryRead
result; the consumed FilePermit is burned on that failure." The widening design
asserted in its own [META-5] delta that "no conformance verdict moves" and
contradicted that on the next page. Both cannot stand.

**Closed by decision 1 of §B.2**: two new operations, `open_file` and
`open_directory` untouched. `sys14-open-directory-component` stays byte-unmodified
and is named load-bearing in §B.9.2.

### B.8.2 Refusing `..` gets safety backwards

**The attack** (judge 2, FF-4; against `writer-first`, which refused a `..`
component before any host call while following symbolic links at every component).
Two list-file lines, the same program:

```text
  ../shared/data.txt     ->  Err(InvalidPath(0,0)), no host call
  cache/data.txt         ->  opens /etc/shadow, if `cache` is a symlink to /etc
                             and `data.txt` is a symlink to shadow
```

The refused form is the one that cannot escape anything the process could not
already reach through its own `..`; the admitted form is the one that actually
leaves the root. A check that stops the harmless case and passes the escaping one
is worse than no check, because it reads as a confinement promise that is not one —
and that design's own text conceded it ("bounds what the supplied code units can
name and **bounds nothing a link names**") and shipped the refusal anyway.

The expressiveness cost is real and measured against the writers' own tooling: a
list produced by `find ../shared -type f` is unopenable, a build tool's dependency
list is unopenable, and the escape hatch offered ("open the parent explicitly")
does not exist, because `open_directory_path` refused `..` too and `open_directory`
takes one component. The only route back is the argv route, which is exactly the
asymmetry Part B exists to remove. `run-syspath-dotdot-preserved` is a live
conformance case whose whole content is that `..` is preserved on the argv route,
so the two routes would then disagree about what a relative path is.

**Closed by decision 3 of §B.2**: validate by exactly [PATH-1]'s test plus the
length limits. `.`, `..`, repeated separators and trailing separators are admitted
and resolved by the target. The same argument disposes of the *empty-component*
refusal `proof-first` shipped: `sample//a.txt` and `sample/` would be refused on
the byte route and admitted on the argv route, for no safety gain.

### B.8.3 So what does "no path traversal by accident" mean here?

Precisely this: **a program can only open a path it named.** It cannot acquire one
by concatenation the language performed on its behalf (`spec/kernel-spec.md:2398`
forbids prefix concatenation outright and makes it a qualification failure rather
than an implementation choice); by truncation at an embedded NUL (refused, and a
NUL would silently open a *different file*); or by an absolute path silently
escaping a relative open (refused before any host call, [PATH-1]:2387's own test).
What it *can* do is open `../../etc/passwd` when its own input says
`../../etc/passwd` — and for a list-file utility, an `xargs`, or a build tool, that
is the correct behaviour, not a defect.

The comparison that settles whether this is acceptable is not against an imagined
confined language; it is against the operation the writer can call today.
`open_read(permit, root: &cwd, path)` with `path` from
`relative_path(arg_get(1))` follows links, admits `..`, and admits any depth.
`open_file_path` does exactly the same three things. **The new operation is
neither safer nor less safe than the existing one on the same input class; it is
the same operation reached from a different source of bytes.** Authority in this
language comes from the `DirectoryRead` a program holds and the `FilePermit` it
consumed, not from where the bytes came from, and bytes a program read out of a
file it opened are not more trustworthy than bytes the kernel handed it in `argv`.

The honest statement of the delta, and the one P18 must carry: **an untrusted list
file becomes as dangerous as an untrusted argument vector, which is exactly as
dangerous as it already was.**

### B.8.4 No TOCTOU the language promises away, and none it opens

Every refusal in §B.4.2 is a property of the *byte sequence*, decided before the
host call, over memory the caller owns under a live shared loan. Nothing about the
filesystem is examined and then relied on, so there is no check to invalidate. The
bytes are then copied into the operation record (`file_adapter.c:611`, asserted
per record by `native_adapter_probe.c:484-485`), so they cannot change between
validation and `openat` either, even in a pipelined loop where the caller's buffer
is live.

Two positive properties survive multi-component paths unchanged, and both are worth
stating as safety claims rather than as resolution claims. **The published resource
is the object that was inspected**: `open_file_path` opens, then inspects the
descriptor it got (`spec/kernel-spec.md:2685`), so a `ReadFile` reaching source is a
regular file the process opened, whatever the path did during resolution. Symlink
following at interior components cannot smuggle a directory, a FIFO, or a device
into a `ReadFile`; it can only change *which* regular file. And the language does
not promise the path still resolves there afterwards, because the program holds a
`ReadFile`, not a name.

**No silent normalization, and that is the safe direction.** Collapsing `a/../b`
lexically would be *less* safe, not more: with a symlinked `a`, the collapsed path
names a different object than the one the host resolves, so a compiler that
normalized would open one file while its diagnostics named another. Refusing to
normalize keeps the language's account of which object it opened identical to the
kernel's, which is [PATH-1]:2390's position for the same reason.

### B.8.5 Do not build per-component descent and call it confinement

This is the single most likely wrong turn in Part B and it is worth a named
warning. A per-component `openat` chain with `O_NOFOLLOW` **is not** confinement:
it costs one syscall per component (0.85 us on the Linux runner, 116 us on the
macOS host — `RESULTS.md:280-286`), holds one descriptor per live level
([SYS-14]:2691), and still races on rename between two components. The only
primitives that actually confine are `openat2(RESOLVE_BENEATH |
RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS)` on Linux 5.6 and later, and Darwin's
`O_RESOLVE_BENEATH`, whose own SDK header marks it "only for `open(2)`" — a probe,
not a fact.

`spec/kernel-spec.md:2399` already fixes where the promise must live: "a value's
confinement promise is fixed by its type and never changes at runtime", so
confinement may never be a flag on an operation or on a value. And [QUAL-2]:2413
forbids emulating a guarantee: "when it cannot supply one, it fails qualification
for that ID and compilation stops rather than admitting the operation under a
weaker guarantee". So a `ConfinedDirectory` cannot ship an emulated fallback; if
the Darwin probe fails it qualifies on Linux and fails qualification on Darwin,
and the language acquires a construct that exists on one of its two live targets.
**That cost, not the code, is what keeps the confined type out of this design.**

**The entry condition, stated so it can be checked rather than argued.** Start the
confined type when all three hold: (1) a real program in the tree consumes path
bytes it did not produce and whose corpus the writer cannot audit — the list-file
tool is *not* that program, since its list is the writer's own file; (2) the
Darwin `O_RESOLVE_BENEATH`-with-`openat` question is answered by a probe in
`research/experiments/`, not by a header comment; and (3) the writer-facing shape
is settled — a confined value produced by an operation over a `DirectoryRead`,
every open over it returning the same `IoError` set with one reachable addition,
and no implicit conversion relating the two directory types [TYPE-4].

Until then the honest position is the one §B.8.3 states out loud and P18 must
teach: **the language confines nothing, on either route, and a program that must
confine writes the check itself** — an ordinary scan over the same range the open
will take, refusing a `..` component. That is a pattern, not a rule, and it is
deliberately not built into the validation, because a lexical `..` refusal is not
confinement (§B.8.2) and building it in would buy a promise the language could not
keep.

### B.8.6 The ordinary three

**No memory corruption.** The two [SYS-8] obligations (`spec/kernel-spec.md:2540`)
are discharged statically before the call, exactly as for `open_file` today; the
copy is bounded by the same `end - start` on one side and by a `path_limit + 1`
destination on the other; the scan refuses an over-limit extent before the memcpy;
and the staging function refuses rather than truncates (`file_adapter.h:70`).
`spec/kernel-spec.md:2682`'s "neither operation has a runtime range check or
`traps` effect" extends verbatim, and `:2545`'s "The target is never asked to
validate a source pointer or source range" still holds, because the target is
handed a NUL-terminated copy the compiler owns.

**No data race and no retained caller storage.** The path borrow is shared and
ends at `loan-released(path)` (`spec/kernel-spec.md:2294`), which the copy makes
true rather than promises; `:1435` binds the adapter to it. A path is longer than a
component and is copied by the same memcpy into the same record, so nothing about
that argument is component-shaped.

**No new claim.** The writer writes zero `claim` statements to use this, which
matters because a claim is never removed for speed and adding a new reason to write
one is a real cost.

## B.9 The differential oracle, the tests, and the falsifier

### B.9.1 The two oracles, and why the design's shape depends on one of them

**O-B1 — the `Inventory` nested-prefix differential.** The repository already owns
the standing shape for a new system surface, and it states it itself
(`compiler/src/resolution/catalog.rs:414-421`):

> The three states are strictly nested prefixes of the tables below, taken in
> normative order, so a state is a length rather than a set of independent
> features: every declaration ordinal an earlier state assigns keeps exactly that
> value in a later one. That is what lets one differential test show that switching
> a candidate off leaves every earlier program's emitted module byte-identical.

Part B ships as one more nested prefix: a `PATH_OPEN` switch beside
`TRAVERSAL_SURFACE` (`catalog.rs:399`) and `OPEN_BY_NAME` (`:412`), and a fifth
`Inventory` variant. Both new rows are appended **last** to `SYSTEM_OPERATIONS`, so
`open_file`'s ordinal 190 (`resolution/tests.rs:1751`) and every ordinal below it
are untouched. With the switch off, every program that compiles today compiles to a
byte-identical module, and `system_index_helpers_agree_with_the_preorder_entity_map`
(`resolution/tests.rs:2197-2216`) already iterates the inventory states and pins
the ordinal map under each; the change adds the new state to that loop.

**The oracle costs one enum variant and it is total — and it is also the argument
that settles §B.2's first decision: a widened operation has no "off" position and
therefore no differential at all.**

**O-B2 — the descent is a compiler-independent reference implementation that
already runs.** The component-by-component descent the third blind writer wrote by
hand (`writer3/largest.wf:196-200`) is exactly path resolution, in Whitefoot, with
no knowledge of this feature. Every admitted multi-component open has an equivalent
descent, and the oracle is that they agree — **with the divergences asserted, not
excused**:

```text
  one call:  open_file_path(root, "c1/c2/.../cn")
  descent:   open_directory(root, "c1") -> ... -> open_file(dn-1, "cn")
```

The two routes **must** differ at a symbolic link, at *both* ends: the descent's
`component_directory_open_flags` and `component_file_open_flags` carry
`O_NOFOLLOW` (`qualification.rs:919-984`), and the path route carries none
(§B.2 decision 4). So an interior symlink and a final symlink each produce
`InvalidPath` from the descent and `Ok` from the one call, and the fixture must
contain both cases and the test must assert both sides. **A test suite in which
the two routes agree everywhere is a test suite whose fixture has no symlink in
it, and it proves nothing.**

Fixture cases, each with both routes: a plain two-level file; a three-level file; a
missing final component; a missing interior component; an interior regular file
used as a directory (`ENOTDIR`); an unsearchable interior directory (`EACCES`); a
symlinked final component (**asserted divergence**); a symlinked interior component
(**asserted divergence**); `a/./b`; `a/../a/b`; `a//b`; `a/b/` where `b` is a
directory and where `b` is a file.

**O-B3 — the host oracle, which the compiler cannot be witness for.** §B.6.2's
errno table is a claim about what the *host* does. A twenty-line C program beside
`research/experiments/io-completion-bench/read_baseline.c` calls
`openat(dirfd, path, O_RDONLY | O_NONBLOCK)` — the exact flag word
`path_file_open_flags` carries — over the same corpus of shapes on the same fixture
tree, and publishes the raw `errno`; the Whitefoot program publishes the [SYS-7]
class. The pair must agree on every row of both families. This is the only test
that catches an errno mapped to the wrong class, a family divergence (`ELOOP` is 40
on Linux and 62 on Darwin), or a validation stricter than the rule says.

**O-B4 — the mechanism count, because identity alone proves nothing.** Opening a
nested file two ways and getting the same checksum holds trivially if the path
route silently fell back to a descent. So: `strace -f -e trace=openat` must show
*n* `openat` calls for the descent of an *n*-component path and **exactly one** for
the path route, and the permit census must show *n* `reserve_file` against one.

### B.9.2 The tests, and where each lives

| what it pins | home | kind |
|---|---|---|
| two catalog rows, their ordinals, the new inventory prefix | `resolution/tests.rs` (extend the state loop at `:2197-2216`) | unit |
| signature, effect row, [SYS-8] family membership, permit consumption, `propagate` chain | `semantic/tests/` beside the `open_file` cases | unit |
| the two [ENT-6] range obligations on `path`, proved and refuted | `semantic/tests/entailment.rs` | unit |
| the emitted scan: every refusal shape, the slot size, the single `openat`, the flag word | `backend/tests/system_io.rs` | executed backend |
| symlink behaviour at the final and interior components; an escaping `..` | `backend/tests/system_io.rs` (builds its own tree) | executed backend |
| the validation table and the errno classes, compiler-independently | `tests/conformance/cases/sys14-open-*-path-*.wf` + `manifest.jsonl` | corpus |
| the list loop stages with `begin` carried | `semantic/tests/` + a `--par-ledger` golden | unit + ledger |
| a real program end to end | `tests/programs/list_open.wf` | maintained program |
| the wall clock, the `openat` census, the descent pairs | `research/experiments/io-completion-bench/` + the C oracle | benchmark |
| the taught form and the `InvalidPath(0, 0)` reading | `docs/patterns.md` P18 | documentation |

New conformance cases, with the fixture schema unchanged
(`{"path": hex, "bytes": hex}` or `{"path": hex, "directory": true}`,
`tests/conformance/runner.py:51`; the adapter already runs `create_dir_all(parent)`,
`compiler/tests/conformance/adapter.rs:118-126`, so a nested fixture path works
today):

| id | expect | what it pins |
|---|---|---|
| `sys14-open-file-path-multi-component` | run, exit 0 | `sample/a.txt` from a `buffer<u8>` opens and reads |
| `sys14-open-file-path-root-prefix` | run, `InvalidPath` | a leading `/` refused before any host call, with **no fixture arranged** |
| `sys14-open-file-path-parent-component` | run, exit 0 | `sample/../sample/a.txt` opens the same object — the anti-falsifier for a later `RESOLVE_BENEATH` "helpfully" added |
| `sys14-open-file-path-self-component` | run, exit 0 | `./a.txt` admitted |
| `sys14-open-file-path-repeated-separator` | run, exit 0 | `sample//a.txt` admitted |
| `sys14-open-file-path-nul` | run, `InvalidPath` | an interior NUL refused |
| `sys14-open-file-path-component-too-long` | run, `InvalidPath` | 256 bytes in one component on Linux |
| `sys14-open-file-path-over-limit` | run, `InvalidPath` | a 1024-byte range refused |
| `sys14-open-file-path-empty-range` | run, `InvalidPath` | [SYS-8]:2549 for the new pair |
| `sys14-open-file-path-directory` | run, `IsDirectory` | the inspection [SYS-14]:2685 requires |
| `sys14-open-directory-path-not-directory` | run, `NotDirectory` | `sample/a.txt` through the directory opener |

The **`arrange`-free refusal cases are deliberate**: a case that arranges nothing
and still expects `InvalidPath(0, 0)` distinguishes "refused before the host" from
"the host said no", which is otherwise untestable from inside the program.

**Two existing cases are load-bearing and must stay green with their `.wf` bytes
and their manifest rows unmodified.** Naming them here is the point, so a later
reader knows that editing either is the governance breach `CLAUDE.md` names and not
a fix:

- `sys14-open-directory-component` — if this needs editing, someone widened
  `open_file`/`open_directory` and silently removed the net of §B.8.1.
- `sys14-no-path-from-bytes` — a `reject` at [TYPE-5] for handing a `buffer<u8>` to
  `relative_path`. It stays a reject: there is still no path *value* from bytes. Its
  `doc` sentence quotes the exclusion sentence §B.4.1 amends, so the doc changes
  even though the verdict does not — and that is conformance content under merge
  rule 4.

**The one honest gap.** The FIXTURE schema cannot arrange a symbolic link, so
O-B2's asserted divergences and §B.4.2's symlink sentence are testable only inside
the compiler's own backend tests. Two ways out, to be chosen deliberately rather
than drifted into: add `{"path": hex, "symlink": hex}` to the schema, about 30 lines
across `runner.py` and `adapter.rs`, which makes the promise compiler-independently
testable; or leave it to the backend test. **This design recommends the schema
delta** (§B.11 Q4), because a symlink promise checked only by the compiler that
makes it is exactly the shape of evidence this project's own rules distrust.

Also required, and free: **F-B4**, `wf__completion_file_demoted_opens()` reads
**zero** after any program whose opens are all admitted paths — the assertion that
catches a path limit chosen above `WF_FILE_PATH_CAPACITY` (§B.6.1). **F-B5**, two
opens outstanding at once carry different staged path bytes, extending
`native_adapter_probe.c:484-485` to two records claimed before either completes;
without it a shared `%component` alloca passes every sequential test and opens the
wrong file under the pipeline. **F-B8**, the diff under
`compiler/src/backend/completion/` is **empty** — a review invariant on Part B's
central claim, and the cheapest check in the file.

### B.9.3 The falsifier

Part B claims expressive power at zero runtime cost, so it has two bars and they
fail differently.

```text
  REQUIRED  expressiveness  writer3/sizes.wf opens every entry of its own list.txt with one
                            identifier changed, publishes the same total as per-file `wc -c`,
                            and flat.txt is deleted
  REQUIRED  expressiveness  largest.wf loses buffer_vacant<DirectoryRead>, replace, spath, slen,
                            the separator write at :173 and the three copy loops, and prints
                            output byte-identical to the record in docs/done/0100
  REQUIRED  mechanism       exactly one openat per admitted path, counted by strace/dtruss, on
                            both families
  REQUIRED  Linux  10k depth-4 opens from a list  <=  0.40 x  the same program written as a
                   component descent, holding 1 live descriptor where the descent holds 4
  control   Linux  10k single-component opens     <=  1.02 x  before this change
  control   both   every O-B2 pair agrees except the two asserted symlink cases
  control   both   wf__completion_file_demoted_opens() == 0
  control   both   the diff under compiler/src/backend/completion/ is empty
```

The 0.40 figure is arithmetic, not ambition: a depth-4 open is four `openat` calls
and four descriptor closes in the descent and one of each here, and
`RESULTS.md:280-286` gives 0.85 us per `openat` on the Linux runner and 116 us on
the macOS host, where `RESULTS.md:401` already records the walker workload as
"almost entirely open-bound".

The **expressiveness lines fire before any measurement and they are the ones that
matter**: if `flat.txt` is still needed, the design failed at the thing it was
built for and no benchmark rescues it.

**What falsifies the design.** If an O-B2 pair diverges anywhere but the two
asserted symlink cases, the semantics is wrong and the batch stops — that is the
only outcome in Part B that does. If the single-component control exceeds 1.02x,
the scan of §B.6.1 is not one pass. If the depth-4 bar misses while O-B2 passes,
the win was overstated and the honest response is to correct the claim in
`docs/done/`, not to tune the bar — which is the failure
`LOOP-PIPELINE.md:1824-1833` records for the pipeline design's own REQUIRED line.

## B.10 Implementation plan

**One batch, ~1,450 lines added and ~150 deleted, no chassis required.** Part B lands with or without
batch 0095; only its per-slot staged buffer depends on the pipeline, and that row
is zero if 0095 lands first because it is the same change.

| component | file | lines |
|---|---|---|
| `path_validation` beside `component_validation`: root-prefix block, run counter, path-limit compare | `backend/emitter/system.rs:1723-1760` | ~110 |
| two operation emitters, reusing `emit_open_file`'s body; `%path` slot sizing | `backend/emitter/system.rs:1950-2110` | ~160 |
| two [SYS-2] catalog rows and their parameters (`catalog.rs:940-973` is the 28-line model) | `resolution/catalog.rs` | ~70 |
| `PATH_OPEN` switch, fifth `Inventory` prefix, the table-length arms | `resolution/catalog.rs:388-470` | ~40 |
| target path-limit datum, two flag fields, accessors, four triple rows | `backend/qualification.rs:727-800, 917-984` | ~60 |
| [SYS-8] range-bearing family membership and `propagate` chain (table entries, not rules) | `semantic/`, `resolution/` | ~20 |
| semantic checking | — | **0** (§B.5) |
| resolution and semantic unit tests | `resolution/tests.rs`, `semantic/tests/` | ~110 |
| executed backend tests, including the symlink tree | `backend/tests/system_io.rs` | ~200 |
| conformance cases, fixtures and verdicts | `tests/conformance/` | ~210 |
| symlink fixture schema, if approved (§B.11 Q4) | `runner.py`, `conformance/adapter.rs` | ~30 |
| the C `openat` oracle, the descent-pair harness, the bench row | `research/experiments/` | ~150 |
| `list_open.wf` and its expected output | `tests/programs/` | ~90 |
| spec: [SYS-14] two edits, [PATH-2]:2395, [PATH-1], [QUAL-2], four lists, the [META-5] delta | `spec/kernel-spec.md` | ~60 |
| `docs/patterns.md` P18, `docs/done/` record | `docs/` | ~140 |
| **deleted**: `flat.txt`, `largest.wf`'s descent stack (`:55, :78, :91, :173, :196-200`) | writer programs | **−150** |
| **total** | | **~1,450 added, ~150 deleted** |
| **runtime C change** | `completion/file_adapter.c`, `linux_io_uring.c` | **0** |

The zero is the point, and F-B8 checks it. `openat` already takes a whole path,
the operation record already carries 1024 bytes of path storage, the staging
function already refuses rather than truncates, and the probe already asserts the
per-record binding. **Part B is a compiler and specification change over a runtime
that has been ready for it since the completion adapter was written.**

Part B **changes the specification's promises** where Part A mostly changes what
an implementation may do: two [SYS-14] edits and one new paragraph, one [PATH-2]
sentence, one [PATH-1] sentence, one [QUAL-2] guarantee, and about a dozen
conformance cases are specification and conformance content, so merge rule 4 puts
the exact specification bytes and the exact added, modified, deleted, or renamed
conformance content into `governance/APPROVALS.md` at merge time.

## B.11 Open questions for the owner — Part B

**Q1. Is 1023 the right path limit, and is one number for both families right?**
§B.6.1 recommends 1023 code units on every Unix-family target, because it is
exactly `WF_FILE_PATH_CAPACITY − 1` and it makes the demotion path unreachable for
admitted paths. The cost is that Linux accepts 4096 and the language refuses at
1024 — a stated narrowing no program in the tree notices.
*Recommendation: 1023 everywhere, with F-B4 as the standing check.*

**Q2. Does [PATH-1] gain the same length test?** §B.2's whole argument is one path
semantics, but [PATH-1]:2387 has no length clause today, so `relative_path` over a
4000-byte argument succeeds and the length becomes a host outcome. Adding the limit
keeps one semantics; leaving it out means the two routes differ in exactly one
clause, which §B.2 promised they would not.
*Recommendation: amend [PATH-1] too, and check whether a `relative_path`
conformance verdict moves.*

**Q3. Should `InvalidPath` distinguish its reasons?** Five distinct pre-host
refusals return `InvalidPath(0, 0)` (§B.6.2). A portable discriminator would change
a deliberately closed 28-class set and touch the nine files that match `IoError`
exhaustively. A cheaper half-measure is to let the compiler-owned refusal put a
compiler-owned value in `code`, which `spec/kernel-spec.md:2528` currently reserves
for the target's native error code.
*Recommendation: leave the zeros and teach the reading in P18.* The two zeros
already carry the one fact that matters — this never reached the host.

**Q4. The symlink fixture.** Approving `{"path": hex, "symlink": hex}` in the
conformance FIXTURE schema costs about 30 lines and makes §B.4.2's symlink sentence
and O-B2's asserted divergences compiler-independently testable; declining it
leaves a normative sentence pinned only by the compiler that implements it.
*Recommendation: approve the schema delta.*

**Q5. Should a `--qualification-report` print the target data?** The path limit,
the component limit, and the flag words are compiler-internal target data
([QUAL-1]:2411) that a writer currently cannot read. §B.7 declines to put the
`%path` alloca on the `PAR` ledger, so there is no other place.
*Recommendation: yes, as a separate small change, not in this batch.*

**Q6. Confirm that `directory_next`'s record stays one component.**
`spec/kernel-spec.md:2676` is kept unchanged by §B.4.1 because it states a host
fact about an enumeration record, not a restriction on opens. Its final clause —
"so no record a program reads can name more than one component" — was written when
it also implied what could be opened, and after this change it no longer does.
*Recommendation: keep the sentence and keep the clause*, because it remains true
of the record; but the owner should confirm the reading rather than inherit it.

**Q7. `open_directory_source` keeps its single-component reach.** The design adds
the path form for the two openers a program uses to reach a file or a directory
and leaves the enumeration opener taking one component, because a walker reaches a
subdirectory through the entry name it just enumerated. That means one of the three
openers has a different reach from the other two.
*Recommendation: add the third when a program needs it*, on the project's own rule
that material earns its place before it is created.

**Q8. Does the owned-backing path type still get built?** [HOST-3]:2381 describes
it and §B.2 declines it. The trigger that would change the answer is a program that
must *hold* a path across a buffer's reuse — a sort of paths, a deduplicating
walker. `largest.wf` avoids it by holding bytes; a program that could not would
want the type.
*Recommendation: leave it deferred and record this trigger, so the next writer's
program decides it rather than an argument.*

**Q9. Standard input is still unreachable, and this design makes it more so.**
D7 (`docs/done/0098-blind-writer.md:411-436`) records that `cat`, `wc`, `sort` and
`grep -` are unwritable because there is no route to a `ReadFile` for an already-open
descriptor, and §A.4.5's inspection makes `/dev/stdin` *refused* rather than merely
useless. If the owner wants filters, that is a third operation and its own design.
*Recommendation: record it as a consequence of A0 and design it separately.*

---

# C — Provenance

The three source designs and the two reviews are the evidence behind every choice
above. This section names what came from where, and — more usefully — what was
proposed and rejected, with the concrete program or schedule that killed it.

## C.1 What survived, and where it came from

| # | idea | from | why it is here |
|---|---|---|---|
| 1 | conditions 2b.4 and 2b.5 — no `claim` in a speculated prologue, and a predicted place read only as an argument of c or an operand of a total operation | `writer-first` | the only fence against §A.8.1 and §A.8.2; both judges called it non-negotiable |
| 2 | the discardability *test* — "creates no host resource **and** its contract fixes no change to the state of any resource it names" | `writer-first` §D.1, its own self-correction | generalizes to operations not yet declared, and is why `directory_next` stays denied |
| 3 | the contract-derived prediction bound — [SYS-8]:2564 bounds the operand by the `end` actual | `writer-first` | turns a runtime guess into a provable one-sided over-estimate stated in the rule |
| 4 | the terminating exit as a second alternative to condition 2, keeping `spec/kernel-spec.md:2058` unchanged | `writer-first` | makes the loop genuinely staged, which is what licenses the compute lane (§A.8.5) |
| 5 | two grades of carry — `carried-closed` and `carried-predicted` | `proof-first` | lets the cheap half ship alone, and it is what Part B's list loop needs (§B.7) |
| 6 | the [SYS-2] `discardable` property as the home of the fact, keyed on semantic ID | `proof-first` | one table lookup, with a closing sentence obliging any future operation to opt in |
| 7 | the discard procedure — terminal before release, outcome dropped uninspected, releases run, slot restored, ring freed last — and its review instruction | `proof-first` §A.5, verbatim | the one line an implementer most needs: releasing a target buffer before terminal is a use-after-free |
| 8 | the `strace` offset census (O3) | `proof-first` | the only oracle that proves discard-and-replay *happened* rather than that the bytes matched |
| 9 | the `Inventory` nested-prefix differential, and the argument that a widened operation has no "off" position | `proof-first` | the repository's own oracle, free and total, and it settles §B.2's first decision |
| 10 | naming `sys14-open-directory-component` and `sys14-no-path-from-bytes` as load-bearing anti-falsifiers | `proof-first` | the anti-governance-breach framing `CLAUDE.md` asks for |
| 11 | the warning against silent privatization of a hoisted buffer — warn and teach | `proof-first` §A.11 | the owner's own ruling, and it names the program a naive rule would miscompile |
| 12 | the target-capability table, and join-and-drop as its consequence | `runtime-first` §A.0 | settles cancellation in one line: Darwin cannot interrupt a helper blocked in `pread` |
| 13 | one path semantics — validate by exactly [PATH-1]'s test | `runtime-first` §B.3.3 | the right answer to both §B.8.2 attacks, and the one principle the other two designs each violated in a different place |
| 14 | path limit 1023 = `WF_FILE_PATH_CAPACITY − 1`, with `wf__completion_file_demoted_opens() == 0` as the free assertion | `runtime-first` §B.8.3 | every number from the runtime that has to carry it |
| 15 | the descent-equivalence oracle with the symlink divergence *asserted* | `runtime-first` §B.12.1 | "a test suite in which the two routes agree everywhere is a test suite whose fixture has no symlink in it" |
| 16 | the two-outstanding-opens staged-path probe (F-B5) and the empty C diff (F-B8) | `runtime-first` | a latent-bug catcher for a bug the tree's own comment admits, and a checkable central claim |
| 17 | `WF_IO_SHORT_READ_EVERY=n` as a target-policy fault knob | `runtime-first` | the only deterministic route to the discard path on a real filesystem |
| 18 | the measurement discipline wholesale — `WF_IO_RING=0 WF_IO_HELPERS=0` as the Linux control, both counters asserted not their difference, macOS as no-regression, the bar against a ceiling built first | `runtime-first` | `--no-overlap` is not a sequential control on Linux, and the pipeline design's own REQUIRED bar stopped discriminating |
| 19 | the confined type's three-part entry condition, with the Darwin header comment treated as a probe | `runtime-first` §B.7.4 | the most disciplined deferral in the set |
| 20 | the `PAR ring` line, with its per-slot figure broken down | `runtime-first` §B.9 | the owner's ruling that what the compiler allocates is stated |
| 21 | the [PATH-2]:2395 amendment | `runtime-first` §B.7.2 and `writer-first` §B.8 | an exhaustive "either … or" a third form silently violates |
| 22 | the [ERR-3]:2519 propagate-chain amendment | `writer-first` | a writer-visible hole no other design caught |
| 23 | the fifteen-loop scoreboard, and its conclusion that D1 is worth more than either part | `writer-first` §C | the acceptance instrument for the batch |
| 24 | the `open_read` descriptor-status inspection | all three, independently | a correctness fix worth landing before either part |

Three items are this synthesis's own, because the judges found gaps all three
designs shared: **the mispredict backoff** of §A.6.4 with its REQUIRED falsifier
line (no design had one, and a filesystem that short-reads every chunk makes K = 32
issue 32 reads per delivered chunk); **the nested K×K disclosure** in the `PAR ring`
line of §A.7 (a staged loop whose remainder calls a function that itself pipelines
holds K_outer × K_inner slots, and no design put that where a writer could read it);
and the **[SYS-2] inventory arithmetic** of §B.4.7 done rather than asserted.

## C.2 What was rejected, and the counterexample that rejected it

| # | proposal | from | why it is not here |
|---|---|---|---|
| R-A1 | run read-ahead *outside* [PAR-3], on the ground that a prefetch "is not an action of B" | `runtime-first` §A.3.1 | the cleanest idea about *what* is happening, and it grants a permission inside a rule whose premise it refutes: [PAR-3] opens "Permission holds for a loop L exactly when all of the following hold", the design applies read-ahead only to loops that fail condition 2, and its added text then borrows "the place this rule replicates" and `spec/kernel-spec.md:2079`'s storage-reuse clause. Both judges. |
| R-A2 | price the mechanism against a C loop with the fold on a second thread | `runtime-first` §A.10 | unearnable by construction under R-A1: with no staged permission the owner lane must join the fold before the back edge, so the hand-out buys nothing. Fixed here by §A.2 decision 1 rather than by lowering the bar. |
| R-A3 | prediction as an unconstrained runtime guess, with the rule silent on how | `proof-first`, `runtime-first` | replaced by §A.4.2's contract-derived bound. The guess version needs a sentence saying the guess is unobservable; the contract version does not need it for the value, only for the schedule. |
| R-A4 | declare `read_at` discardable unconditionally over every `ReadFile` | `proof-first` §A.3.1 | `open_read` performs no descriptor-status inspection, so a `ReadFile` may name `/dev/urandom`, a `/proc` file, a seekable character device, or a tape, and a discarded read on a read-and-clear device destroys data the program would have seen (`FIRST-PRINCIPLES.md:374-378`). Closed by §A.4.5. |
| R-A5 | "the only cost is one wasted `pread` per short read" | `proof-first` §A.5, `runtime-first` §A.3.3 | the cost is K−1, and on a source that short-reads every chunk it is K−1 *per chunk*. §A.6.4 and the REQUIRED adversarial line. |
| R-A6 | admit only a loop-invariant stride, and teach writers to write a bounded loop over block indices | `writer-first` §A.11 Q1, as its own counter-proposal | it is a smaller judgment and it leaves the owner's loop exactly where it is: the fast form would need a file size the language has no operation to obtain. |
| R-A7 | cancel in-flight reads on a mispredict or an exit | considered by all three | Darwin cannot interrupt a helper blocked in `pread` (`FIRST-PRINCIPLES.md:944-948`), and `IORING_OP_ASYNC_CANCEL` costs an SQE, a CQE and a race with the completion it is trying to stop. Join-and-drop is free (§0.1). |
| R-A8 | let the compiler silently privatize a hoisted destination buffer | tempting, refused by `proof-first` explicitly | `spec/kernel-spec.md:2069` forbids deriving the coverage from [SYS-8]'s "may have changed" wording, and a whole-place "write before read" rule would accept the loop-carried-scratch program of `LOOP-PIPELINE.md` §2.3 and miscompile it. |
| R-A9 | repair the writer's `read_chunk` helper by having it construct its own buffer | `proof-first` §A.11 | works for `count_file`, which discards the bytes; fails for any helper whose *caller* folds the chunk, because a buffer the helper constructs dies at its return. The honest statement is §A.10's: the helper disappears and the signature changes. |
| R-B1 | widen `open_file` and `open_directory` in place | `runtime-first` §B.3.1 | §B.8.1. `writer3/sizes.wf:117` unmodified opens `/etc/passwd` from a list-file line, and `sys14-open-directory-component`'s whole content is the behaviour it deletes. Both judges; judge 1 called it the one flaw in the set worth blocking on. |
| R-B2 | refuse a `..` component before any host call | `writer-first` §B.3 | §B.8.2. It stops the case that cannot escape and passes the case that does, breaks a `find ../shared`-generated list, offers an escape hatch that does not exist, and makes `open_file_path` and `open_read` disagree about what a relative path is. |
| R-B3 | refuse an empty component — a leading, doubled or trailing separator | `proof-first` §B.2.3, `writer-first` §B.3 | not forced by anything, and it creates two grades of well-formed path: `sample//a.txt` refused on the byte route and admitted on the argv route, for no safety gain. `proof-first` raised it as its own open question and shipped it as the default. |
| R-B4 | keep `O_NOFOLLOW` on the path route | `proof-first` §B.4, `runtime-first` (inherited) | it makes a list-file line naming a symlinked file return `ELOOP → InvalidPath` while the identical bytes on `argv` open it — a *new* asymmetry created while removing one. |
| R-B5 | path limit 4096 on the Linux family, with a 4097-byte stack slot | `proof-first` §B.2.3 | `WF_FILE_PATH_CAPACITY` is 1024 and `wf_file_stage_path` refuses rather than truncates, so a 1100-byte path is admitted, compiles, opens — and silently demotes off the completion path, in a design whose Part A is about keeping operations in flight. Both judges. |
| R-B6 | a new owned `PathBuffer` type over program-owned bytes | considered and refused by all three | [HOST-3]:2381 predicts the price: a new nominal, a [SYS-5] release row, a [STOR-1] storage class, a heap copy per path, and a new affine value inside every loop body — hence a new place for [PAR-3] to classify. No writer program in the tree wants to *hold* a path. |
| R-B7 | per-component `openat` with `O_NOFOLLOW`, called confinement | the obvious wrong turn | §B.8.5. It still races on rename between two components, and `spec/kernel-spec.md:2399` fixes the promise to a *type*, never a flag. |
| R-B8 | a new `IoError` class for a path refusal | considered | `ELOOP` already maps to `InvalidPath` on both families, so a writer's existing arm covers it; a new class would touch the nine files that match `IoError` exhaustively for no writer benefit. |
| R-B9 | `--no-overlap` as the Linux sequential control | `proof-first`, `writer-first` | its binaries still report `enters=8192` (`LOOP-PIPELINE.md:1824-1833`). Correct as a lowering check, wrong as a baseline. |
| R-B10 | a REQUIRED bar of the shape "beat `C.wide8`'s own median" | `writer-first` §A.8.2 F3 | `LOOP-PIPELINE.md` §9.6 item 5 records that the predecessor bar of that shape is met on this container by setting an environment variable, "so it no longer discriminates between a working pipeline and no pipeline". §A.9.3 states the bar against a ceiling that must be built first and keeps the medians as context. |

---

# D — The scoreboard, and what neither part fixes

Every loop a blind writer actually wrote, and what each part does to it. "Taught
change" means P15's per-iteration scratch and nothing else. This table is
`writer-first` §C, carried forward unchanged because it is the acceptance
instrument for the batch; line references are each program's own at `16228216`.

| # | program | the loop | today | after Part A | after Part B |
|---|---|---|---|---|---|
| 1 | `blind-writer/2026-08-28/programs/p1_tree_wc.wf:194` | `@chunks`, read-fold-discard | denied: exit in E, carried `offset`, hoisted `chunk` | **pipelines**, taught change | — |
| 2 | `p1_tree_wc.wf:304` | `@batches`, `directory_next` | denied | **still denied** (§A.8.6) | — |
| 3 | `p1_tree_wc.wf:335` | `@records`, per-entry with output | denied | still denied (D1) | — |
| 4 | `p2_tree_grep.wf:156` | `@chunks`, sliding window | denied, same three walls | **pipelines**, taught change | — |
| 5 | `p3_checksum.wf:134` | `@chunks`, flag-then-break | denied | **pipelines**, taught change | list from a file now possible |
| 6 | `p3_checksum.wf:189` | `@each`, per-file with output | denied | still denied (D1) | — |
| 7 | `p4_copy_count.wf:117` | `@pump`, read then write | denied | **still denied**: the `write_once` in E holds an exclusive loan on enclosing `Output` (condition 4) | — |
| 8 | `p5_two_outputs.wf:114` | report loop | denied | still denied (D1) | — |
| 9 | `writer3/sizes.wf:11` (`count_file`) | `@chunks`, `ReadEnd` break | denied — **the owner's own case** | **pipelines**, taught change | — |
| 10 | `writer3/sizes.wf:117` | — | `sample/a.txt` unopenable | — | **opens**, one identifier |
| 11 | `writer3/largest.wf:11` (`count_file`) | same as 9 | denied | **pipelines**, taught change | — |
| 12 | `writer3/largest.wf:55-211` | the descent | ~60 lines of workaround | — | **optional**: the descent collapses |
| 13 | `tests/programs/dir_walk.wf:262` | `@batches`, `directory_next` | denied | **still denied** (§A.8.6) | — |
| 14 | `tests/programs/wfgrep.wf:598` | chunk loop | denied | **pipelines**, taught change | — |
| 15 | `io-completion-bench/programs/read_heavy_narrow.wf` | eight `@lane_N` | denied | **pipelines byte-unchanged** at A3; taught change at A2 | — |

**Part A fixes six of the eight read-fold loops** — every one whose body does not
publish per iteration — and one of them byte-unchanged once A3 lands. **Part B
fixes the one program that could not be written at all** and shortens a second by
a quarter. `writer3/sizes.wf:74-93`'s `@slurp` loop stays sequential and should:
it accumulates into one buffer the program keeps.

**What neither part fixes, and this is the honest half.** Five of the fifteen rows
are the same defect: a loop that writes to one `Output` per iteration is denied by
condition 4, and no amount of read pipelining changes it. That is **D1**, the
standing open point `docs/done/0100-writer-defaults-2.md:766-775` records, and it
is the largest remaining gap in the writer's experience — a `wc` that prints one
line per file is the shape everyone writes first, and `cat`, `cp` and every filter
are exactly it. The working rewrite (fold a total, publish after the loop) changes
the program's output ordering guarantees, which is why it is an open question and
not a pattern.

**If the owner is choosing what to build after these two, D1 is worth more than
either.** After Part A lands, the *fold* is fast and the *copy* is not, which makes
D1 more visible rather than less. Two further rows are the enumeration analogue
(§A.8.6), which needs an operation whose speculative advance is recoverable —
`spec/kernel-spec.md:2627` already anticipates it as "a separate system type rather
than hidden state in this type", and both blocked programs are directory-bound
rather than device-bound, so the payoff is smaller than Part A's.

---

# E — What `docs/patterns.md` gains

`docs/patterns.md` ends at P17 (`:541`), so the new entries are P18 and P19. This
section exists because the owner's ruling is to prefer warning and teaching over
silent transformation: both parts change what a writer should write, and a change
nobody is taught is a change nobody takes.

**P15 is amended, not replaced.** Its problem statement (`docs/patterns.md:336-345`)
is about a loop that opens one file per iteration. The same paragraph now covers
the loop that reads one *chunk* per iteration, which is the shape it was always
really about, and its closing sentence — that reusing one buffer "is also what makes
the program genuinely order-dependent" — becomes the lead rather than the aside,
because §A.10's argument is exactly that the pattern was right before there was any
performance reason to take it. The worked example gains §A.3's `count_file`
rewrite beside the existing per-file one. The paragraph at `docs/patterns.md:459-464`
telling the writer that one file's chunk loop cannot be staged is **superseded in
place**, in the same change that deletes `EXIT_IN_REMAINDER` and
`EXIT_SELECTED_BY_SUBMISSION`.

**P18 — open by path when the bytes came from outside, by component when you are
walking.** *The problem:* a program has path bytes and two operations that take
them, and nothing in either signature says which to use. *The pattern:* `open_file`
and `open_directory` take one component, refuse a terminal symbolic link
(`spec/kernel-spec.md:2687`), and resolve one component per host call, so a walker
that descends and holds a descriptor per level uses them, and gets a per-component
diagnostic for free. `open_file_path` and `open_directory_path` take a whole
relative path, refuse an absolute path and a NUL before any host call, follow links
exactly as the rest of the process namespace does, and resolve in one host call
whatever the depth, so a program whose path arrived as data — a list file, later
standard input — uses them. *The trade, stated:* one call and no per-component
diagnostic, against one call per level and no link-following at the leaf. *Also
teach:* the language confines nothing on either route; an untrusted list file is
exactly as dangerous as an untrusted argument vector; a program that must confine
writes the `..` scan itself (§B.8.5), and `InvalidPath(code: 0, origin: 0)` means
*this never reached the host* while a nonzero `code` means the host refused.
*Replaces:* the component-by-component descent every writer builds when they
discover `open_file` refuses a separator, of which `writer3/largest.wf:55-211` is a
sixty-line instance.

**P19 — let the loop read ahead: do not hoist the destination, do not hoist the
cursor.** *The problem:* the two habits that cost a streaming loop its pipeline are
the two every systems programmer brings — allocate the buffer once above the loop,
and keep the cursor in a variable the whole function can see. *The pattern:*
construct the destination inside the body (P15), and let the cursor be a place the
body reads before the read and writes after it, advanced by one *total* operation
against the read's own endpoint. *Then say what the writer gets:* `--par-ledger`
prints `replicated` for the destination, `carried-predicted` for the cursor,
`terminating` for the `ReadEnd` break, and a `PAR ring` line with the window it
reserved. *Show the denied variant beside it* — a cursor advanced by `+defined`, or
by anything the read's contract does not bound — because a pattern that only shows
the accepted form teaches nothing about the boundary. *And one honest cost:* a
rolling fold written `seed: sum` still pipelines its reads but keeps its folds on
the owner lane, because each fold waits for the previous one; that is a property of
the program, not a restriction the compiler adds. This is the pattern the deleted
denial texts used to substitute for.

---

**Both parts, in one sentence each.** Part A admits the streaming chunk loop by
adding one alternative to [PAR-3]'s exit condition and one disposition in two
grades, fenced by five clauses that keep a speculated prologue from tripping a
claim, a domain obligation, a host resource, or a state change — so the
implementation may compute the next offset from the bound the read's own contract
fixes, prefetch, compare at in-order commit, and discard what it guessed wrong,
which is what the hand-written C loop already does and what no writer-visible knob
is needed for. Part B admits a multi-component path by adding two operations beside
the two that exist, validating a caller-owned byte range by exactly [PATH-1]'s test
so the language has one notion of a well-formed path, and resolving it in one host
call — which removes the asymmetry between bytes from `argv` and bytes from a file
and needs no new type, no new judgment, no new error class, and no runtime line.
They share one object, the carried place, and Part B's list loop is the argument for
building the cheap half of it even if the speculative half is declined.
