# Proof-first: the streaming chunk loop, and bytes to path

Design record, 2026-08-28. Written against `integration/2026-08-28b` at
`16228216`; every citation below was read in that revision (via
`git show integration/2026-08-28b:<path>`), and every `spec/kernel-spec.md`
line number is that revision's.

**The angle.** Both parts are derived from the proof, not from the desired
program. Part A starts from what the [PAR-3] judgment can already establish
about a loop body — the cut, the exits, the loans, the per-place dispositions —
and asks for the smallest additional *provable* fact that admits the streaming
chunk loop. Part B starts from the four facts an open needs about its path
bytes before it may reach a host, and asks which of them a value must carry and
which the operation already discharges.

The two parts share one new object: a **carried place**, a place rooted outside
the loop that the prologue reads and the remainder writes. Part A needs it with
speculation; Part B's list-file loop needs it without. It is one disposition
with two proof grades, and that is the whole of the language delta for Part A.

---

# Part A — the streaming chunk loop

## A.0 What the judgment proves today, and the exact wall

The program the owner named — read a chunk, fold it, discard it — is this, and
it is what the third blind writer wrote unprompted
(`$SCRATCH/wf-0100-verify/writer/work/sizes.wf:9-26`):

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

Three separate [PAR-3] conditions refuse it, and they are not the same defect.

**Wall 1 — the exit (condition 2).** `spec/kernel-spec.md:2057-2058` says,
verbatim:

> Every edge that leaves B — a `return_stmt`, a `give_stmt` delivering outside
> B, a `break_stmt` naming L or a loop enclosing L, and a `let_stmt` selecting
> `propagate_let_rhs` [FN-1, GIVE-1, ERR-3] — occurs in P.
> An edge the statement performing c takes on the outcome of that submission,
> which is the edge a `let_stmt` selecting `propagate_let_rhs` at c takes, is an
> edge of E and not of P.

The `ReadEnd` break is exactly that second sentence's edge. The compiler already
says so in as many words:
`compiler/src/semantic/staged_permission.rs:366` —

> "PAR-3 cannot stage this loop as written: the submission's own outcome selects
> this edge, so no rewrite takes it before the submission. The shapes staged
> today are a fixed-trip bounded loop and a per-file loop over names; one file's
> chunk loop stays sequential"

**Wall 2 — the carried offset (condition 5).** `sum` is rooted outside the
loop, read in P (it is `read_at`'s `file_offset`) and written in E. Neither of
`spec/kernel-spec.md:2060`'s three alternatives — read-only, one-segment,
replicated — covers it, so it denies. `docs/done/0100-writer-defaults-2.md:755-765`
records this as the open point W4 and states the two missing pieces exactly:

> "a way to carry the offset across the cut as a value the prologue may compute
> for iteration i+1 before iteration i's read completes, and a way for a short
> read to cancel the operations already in flight. Both are language questions."

**Wall 3 — the hoisted scratch (condition 3).** `scratch` is a parameter borrow
the body writes, and `read_at` retains a borrow of it past its own submission.
This one already has a taught repair (`docs/patterns.md:334-366`, P15): build
the scratch inside the body. It is the only one of the three the writer fixes.

## A.1 The proof, derived

Walls 1 and 2 have one root. Ask what iteration i+1's prologue actually needs
before iteration i's read has completed:

```text
  read i+1 needs:  file (a shared loan, unchanged)
                   a destination     -> iteration-own, replicated       [have]
                   file_offset       -> depends on read i's `next`      [MISSING]
                   whether to run at all -> depends on read i's arm     [MISSING]
```

Both missing facts are *the same submission's outcome*. So the question is not
"can the compiler derive the stride" — on a regular file it provably cannot,
because [SYS-8] fixes only `start <= next <= end`
(`spec/kernel-spec.md:2564`) and states outright that

> A short success is not end of input; only `ReadEnd` states that no byte was
> available at the observed end. (`spec/kernel-spec.md:2556`)

The question is whether the implementation may **guess** those two facts, use
the guess to run a later prologue, and **throw the work away** when the guess is
wrong. That is a smaller ask than a derived stride and it is exactly what the
hand-written C prefetch loop does.

For that to be sound, three things must be proven, and each is a static
property of declared contracts — no entailment fact is consulted:

**(D) The submission is discardable.** Performing the operation and never
delivering its outcome changes no state any operation of this specification
observes. For `read_at` this is not a hope: `spec/kernel-spec.md:2554` fixes it —

> `read_at` performs a positioned read beginning at `file_offset` and never
> observes or changes an implicit file cursor.

— its declared effect row is `reads(file, destination), writes(destination)`
(`spec/kernel-spec.md:2275`), and `destination` is the replicated slot. It is
the only `may-suspend` member of [SYS-2] with that shape: `write_once` changes
the output, `directory_next` advances a cursor
(`spec/kernel-spec.md:2568`), and the four opens create a resource — which is
the case [PAR-3] already legislates at `spec/kernel-spec.md:2072`.

**(R) The prologue is replayable.** Every action of P is the construction of
iteration-own storage, a pure `never-suspends` operation, or a discardable
submission. Then running P again after a wrong guess produces the same state as
running it once with the right values. In the chunk loop P is exactly
`buffer_new` plus the `read_at` submission, so this holds. In the per-file loop
of `many_files_narrow.wf` it does **not** — P holds `reserve_file` and an
`open_file` — so that loop keeps today's judgment untouched.

**(C) The place is a carry.** A place rooted outside L, of copy element type,
never borrowed, read in P only as an operand, and written in E only by `set`
statements whose right-hand side is one total pure operation of [OP-1] over that
place's prior value, entry-stable terms, and the delivered outcome's payload
binders. `sum` in `count_file` is `sum +wrap taken`: total, pure, and a function
of exactly the prior value and one payload binder.

**The two grades of (C).** A carry's recurrence either closes in P or it does
not, and the two grades cost different things:

| grade | condition | example | what lowering does |
|---|---|---|---|
| **carried-closed** | every operand of the recurrence is available in P — no payload binder appears in it | `set begin = index +wrap 1_u64;` in the list-file loop (Part B) | hoist the recurrence above the submission; E reads the slot's saved copy |
| **carried-predicted** | the recurrence names a payload binder of an undelivered submission | `set sum = sum +wrap taken;` | P runs on a predicted value; a mismatch at commit discards and replays every later iteration |

`carried-closed` needs nothing from (D) or (R): no work is ever thrown away, so
nothing has to be discardable. `carried-predicted` needs both.

**The soundness sentence, in one line.** *A speculative prologue's observable
contribution is admitted only when every value it consumed equals the value the
source-order execution produces at that point; otherwise the implementation
delivers no outcome of it, performs no remainder for it, and executes it again.*
That is the whole safety argument, and it is the same shape [PAR-3] already
uses for host resources at `spec/kernel-spec.md:2072`:

> An overlapped execution delivers for each operation of L an outcome that
> operation could deliver in the source-order execution at that point, so an
> implementation whose overlap holds more such resources at once than the
> source-order execution holds completes the earlier iterations and performs the
> operation again at the source-order resource footprint before delivering any
> outcome.

Note what the rule does **not** say: it says nothing about *how* the
implementation predicts. Prediction quality is a runtime policy, exactly as K
is (`research/investigations/io-model/LOOP-PIPELINE.md:681-720`, "The runtime
chooses it, once per loop entry, and the writer never sees it"). An
implementation that predicts nothing runs K = 1 and conforms.

## A.2 The writer-facing example, as it would compile

```whitefoot
fn count_file['f](file: &'f ReadFile) -> total: own u64
reads(file), allocates(heap) {
  let sum = 0_u64;
  loop @chunk {
    let block = buffer_new(65536_u64, 0_u8);       // iteration-own: replicated
    let room = len(block);
    region 'c {
      match read_at<'f, 'c>(file: file, destination: &uniq 'c block,
                            file_offset: sum, start: 0_u64, end: room) {
        ReadBytes(next: taken) => { set sum = sum +wrap taken; }
        ReadEnd() => { break @chunk; }
        ReadFailed(error: problem) => { break @chunk; }
      }
    }
  }
  return sum;
}
```

and the fold version the owner's phrase names — read, process, discard:

```whitefoot
fn checksum_file['f](file: &'f ReadFile) -> digest: own u64
reads(file), allocates(heap) {
  let sum = 0_u64;
  let at = 0_u64;
  loop @chunk {
    let block = buffer_new(65536_u64, 0_u8);
    let room = len(block);
    region 'c {
      match read_at<'f, 'c>(file: file, destination: &uniq 'c block,
                            file_offset: at, start: 0_u64, end: room) {
        ReadBytes(next: taken) => {
          region 'f2 { set sum = sum +wrap fold_bytes<'f2>(source: &'f2 block,
                                                           produced: taken, seed: sum); }
          set at = at +wrap taken;
        }
        ReadEnd() => { break @chunk; }
        ReadFailed(error: problem) => { break @chunk; }
      }
    }
  }
  return sum;
}
```

`at` is `carried-predicted`. `sum` is **not** a carry: it is written in E and
never read in P, so it is an ordinary ordered write, already covered by
`spec/kernel-spec.md:2064`. That distinction matters — it is why a
non-associative or float fold is admitted here with no algebra, exactly as
`docs/patterns.md:379-384` states for the per-file loop.

## A.3 Spec sentences

Four amendments, one rule touched twice, one operation-table property added.
**No new rule, no new grammar production, no new keyword, no new operation, no
new type, no new outcome, no writer-visible marker.** Rule count stays 137.

### A.3.1 [SYS-2] — one operation-table property

Today's sentence, verbatim (`spec/kernel-spec.md:2293`):

> The target contract is `never-suspends` for `args_count`, `arg_get`,
> `host_bytes_len`, `host_copy_bytes`, `host_utf8_len`, `host_copy_utf8`,
> `relative_path`, `exit_status`, and `reserve_file`. It is `may-suspend` for
> `open_read`, `read_at`, `write_once`, `open_directory`,
> `open_directory_source`, `directory_next`, and `open_file`.

Add, immediately after `spec/kernel-spec.md:2294`:

> Exactly one `may-suspend` operation is **discardable**: `read_at`. Performing
> a `read_at` attempt and delivering no outcome of it changes no state any
> operation of this specification observes and produces no value, because the
> operation is positioned, observes and changes no cursor, and its whole
> declared change is to the `[start, next)` range of the `destination` an
> attempt that is discarded does not deliver. Every other `may-suspend`
> operation is not discardable: `write_once` changes the output state,
> `directory_next` advances the enumeration cursor, and the four opens create a
> resource. An operation added by a later version is not discardable unless its
> own record says so.

This is one entry in a table [SYS-2] already carries, in the same class as
`never-suspends`/`may-suspend`. It is the whole of Part A's operation-side
delta.

### A.3.2 [PAR-3] — the exit condition

Today, verbatim (`spec/kernel-spec.md:2058`):

> An edge the statement performing c takes on the outcome of that submission,
> which is the edge a `let_stmt` selecting `propagate_let_rhs` at c takes, is an
> edge of E and not of P.

Keep that sentence unchanged — it is a true statement about where the edge is —
and add, after it:

> An edge of E that the statement performing c takes on the outcome of that
> submission is admitted, and no other edge of E is, exactly when every
> `may-suspend` action of B is discardable [SYS-2] and every other action of P
> is the construction of storage rooted in a binding B itself introduces or one
> `never-suspends` operation whose declared effect row reaches no place rooted
> outside B. An implementation may then execute the prologue of an iteration
> the source-order execution does not reach, provided it delivers no outcome of
> any submission of that prologue, performs no segment E for it, and carries
> every compiler-derived release of the bindings that prologue introduced before
> the loop's edge is taken.

### A.3.3 [PAR-3] — the fourth disposition

Today, verbatim (`spec/kernel-spec.md:2060`):

> Every place rooted in a binding declared outside L that a footprint of B
> reaches satisfies one of exactly three conditions, and a place satisfying none
> denies permission. Either no footprint of B writes it and every loan on it is
> shared; or every footprint element and every loan touching it belongs to one
> of P and E alone and no loan on it is retained past c; or this rule replicates
> it.

Amend the enumeration to four, by replacing "exactly three" with "exactly four"
and appending one alternative:

> … or this rule replicates it; or this rule **carries** it.
>
> This rule carries a place only when its element type is copy [OWN-1], no loan
> of B is ever formed on it, every read of it by a footprint of P is an operand
> read, and every write of it by a footprint of B is one `set` statement whose
> target is that whole place and whose right-hand side is one total operation of
> [OP-1] applied to that place's prior value, to terms no statement of B writes,
> and to the payload binders of B's own outcome matches. A carried place is
> committed at the end of each iteration's segment E, in the order of the
> iterations that perform it, and holds at L's continuation exactly the value
> the source-order execution leaves in it.

### A.3.4 [PAR-3] — the speculative prologue

Add after `spec/kernel-spec.md:2065` ("Every read E performs of a place rooted
outside B that a footprint of B writes likewise occurs in the order of the
iterations that perform it."):

> An implementation may execute the segment P of an iteration using, for a
> place this rule carries, a value other than the one the iterations before it
> have committed. It may deliver an outcome of a submission of that segment, and
> may perform that iteration's segment E, only when every value that segment P
> consumed equals the value the source-order execution produces at that point.
> Otherwise it delivers no outcome of any submission of that segment, performs
> no segment E for it, carries the compiler-derived releases of the bindings
> that segment introduced, and executes that iteration's segment P again with the
> committed values. Which values an implementation uses, how many iterations it
> executes in that way, and how many attempts of a discardable operation it
> performs are not observable.

And extend the existing non-observability sentence
(`spec/kernel-spec.md:2079`), which today reads:

> The number of operations an implementation keeps outstanding, the identity of
> the host thread that executes a segment, whether any overlap was performed at
> all, the storage an implementation gives a replicated place, and the storage an
> implementation reuses across iterations for a construction whose value the body
> releases without observing it, are not observable, and no rule of this
> specification is stated in terms of them.

by inserting, after "whether any overlap was performed at all", the clause
"**, the number of attempts of a discardable operation an implementation
performs and discards, the values it uses in a segment P whose outcome it does
not deliver,**".

### A.3.5 META-5 delta shape

> Numbered rules +0/-0 (137 remain); grammar productions +0/-0; writer operation
> spellings +0/-0; system operations and declaration records +0/-0 (203 remain);
> opaque system nominal spellings +0/-0; entry forms +0/-0; runtime-trap families
> +0/-0; exception clauses +0/-0. [SYS-2] is amended to name which of its
> `may-suspend` operations is discardable, one entry in a per-operation table it
> already carries. [PAR-3] is amended to admit an edge of E that the cut's own
> submission selects when every submission of B is discardable and the rest of P
> is replayable; to carry a fourth place disposition whose recurrence is one
> total [OP-1] operation; and to admit a segment P executed on values later
> iterations may have to discard. No ownership, effect, release, or trap rule
> changes. No accepted program becomes rejected: the permitted-overlap set only
> widens, so no conformance verdict moves.

## A.4 The judgment

`compiler/src/semantic/staged_permission.rs` keeps its seven conditions; two of
them gain an alternative, and one new one is added. Everything below is read
from typing, declared effect rows, resolved places and the statement graph —
**no entailment fact** — so the module's stated invariant
(`staged_permission.rs:162-166`) survives unchanged, and stage 2's
fact-consuming byte-range analysis is not needed for any of it.

**Condition 2, amended.** Today `StagedDenial::ExitInRemainder` carries
`selected_by_submission: bool` (`staged_permission.rs:296`). The flag stops being
a denial reason and becomes a *precondition query*: when it is set, ask

- **(2a) discardable**: every `may-suspend` call of B resolves to a [SYS-2] row
  marked discardable. One table lookup on the semantic ID; no name test.
- **(2b) replayable**: every other statement of P is a construction whose
  resulting binding B introduces, or a call whose target action is
  `never-suspends` and whose [EFF-2]-projected footprint reaches no place rooted
  outside B. `Program::footprint` / `call_projection`
  (`compiler/src/semantic/permission.rs:1116,1244`) already computes exactly
  that set.

Both hold ⇒ admit the edge. Either fails ⇒ today's denial, with today's text.

**Condition 5, fourth disposition.** `Disposition`
(`staged_permission.rs:228-241`) gains `Carried(Grade)` with
`Grade ∈ {Closed, Predicted}`, spelled `carried` and `carried-predicted`
(`spelling()`, `staged_permission.rs:250`). A place lands there when, over its
whole [OWN-7] overlap class (the module already unions flags over the class,
`staged_permission.rs:97-124`):

1. its element type is copy — `is_copy_element` already exists
   (`staged_permission.rs:1638`);
2. no `Loan` of any statement of B names it — `Footprint::loans`
   (`compiler/src/semantic/permission.rs:214`) is the whole query;
3. every read of it by a footprint of P is an operand read
   (`collect_operand_reads`, `permission.rs`), never a written or consumed
   element;
4. every write of it by a footprint of B is `set_target_place`-rooted at that
   whole place, with a right-hand side that is one [OP-1] application whose
   operands are that place, bindings no statement of B writes, and outcome
   payload binders of B's own matches. This is the same *shape* test [PAR-2]'s
   accumulator condition already performs
   (`compiler/src/semantic/loop_permission.rs`), minus the associativity,
   commutativity and identity requirements, which this rule does not use.

Grade is `Closed` when no payload binder occurs in any of those right-hand
sides, `Predicted` otherwise. `Predicted` additionally requires (2a) and (2b).

**Condition 8, new — fail closed on the recurrence.** A `set` of a candidate
carry whose right-hand side this judgment does not resolve to one [OP-1]
application over admitted operands denies. A partial operation, a `Result`
route, a call, or a second `set` on a different path all deny; a `+defined`,
`+checked` or `+sat` right-hand side denies because re-executing it is not the
same as executing it once. `+wrap`, `*wrap`, `-wrap`, the bit operations, and
`imin`/`imax` are admitted; the float operations are admitted too, because this
rule re-executes rather than recombines and therefore uses no associativity.

**What still denies, and must.** A chunk loop that publishes each chunk holds an
exclusive loan on `Output` in E, which condition 4 denies
(`staged_permission.rs:326`) and this design does not touch. A body whose
recurrence is `set at = at +defined taken` denies at condition 8. A loop whose P
opens a file and whose exit is selected by the read's outcome denies at (2b) —
`reserve_file` writes the enclosing factory, so replaying it is not free.

## A.5 Lowering sketch

The chassis is the one `research/investigations/io-model/LOOP-PIPELINE.md`
§§3.1-3.4 already designs; three things are added.

**Depth comes from the window query, unchanged.**
`wf__completion_window(span, slot_bytes, ceiling)`
(LOOP-PIPELINE.md:681-720) is asked once at loop entry. A `loop_stmt` has no
span; it passes `0` for "unknown", and the runtime answers from its own capacity
exactly as it does for a counted loop — `WF_BRIDGE_OPERATION_CAPACITY` and
`WF_BRIDGE_SLOT_COUNT` are 64 (`compiler/src/backend/completion/bridge.c:40-41`),
the Linux ring is sized to the same 64 entries. `K = 1` remains a legal answer
that reproduces the sequential program exactly, so the query can never make a
program fail. There is no source spelling, environment variable or attribute for
K, and none is added.

**Per-slot record.** For each of K slots: the destination ring slot (one heap
allocation at loop entry, restored per iteration under the slot invariant of
LOOP-PIPELINE.md §3.3), the completion token, the outcome payload, the stage,
and — new — **the carried values this slot's prologue consumed**. The last is
what makes the mismatch check possible; it is a few scalars in an
`alloca [KMAX x slot]` in the entry block, not language-visible heap
(LOOP-PIPELINE.md §3.2), and therefore not part of `allocates(heap)`.

**The driver.** Two registers per carried place: `committed` (the source-order
value, advanced only by in-order commit) and `speculative` (advanced by each
prologue). The loop is:

```text
loop {
  while a slot is free and no exit is pending:
      P(next): build the slot, submit read_at(file, slot, file_offset = speculative)
      record consumed = speculative
      speculative = predict(speculative, slot.end - slot.start)   // runtime policy
  join or take the oldest busy slot
  if consumed != committed:            // mispredicted
      discard this slot and every later one (see below); speculative = committed;
      continue
  deliver the outcome, run E: fold, then committed = f(committed, payload)
  if the outcome selected an exit edge: discard every later slot; take the edge
}
```

`predict` is the runtime's; the obvious policy is "the read succeeds in full",
i.e. `speculative + (end - start)`, which is right for every chunk of a regular
file except the last. The rule admits any policy including "predict nothing"
(K = 1).

**Discarding a slot — the part most likely to be got wrong.** A slot whose
`read_at` is still in flight owns its destination buffer *at the target*, until
the operation's `terminal` milestone. So discarding is **not** "forget it":

1. Stop issuing new prologues.
2. For each slot to discard, in any order: obtain its terminal transition —
   `wf__completion_file_take` non-blocking
   (`compiler/src/backend/completion/bridge.h:144`), falling back to
   `wf__completion_file_join` (`bridge.h:129`). An implementation may first ask
   the target to cancel (`IORING_OP_ASYNC_CANCEL`); it must still reach terminal.
3. Drop the outcome without inspecting it. Do not map an error, do not publish a
   `ReadFailed`, do not advance any carried place.
4. Run that slot's compiler-derived releases for the bindings its prologue
   introduced ([STOR-3], `spec/kernel-spec.md:651-660`), restore the ring slot,
   and mark it free.

Only after every discarded slot reaches terminal may the loop take an exit edge
or free the ring. **A target buffer released before its operation's terminal
milestone is a use-after-free, and it is the single defect this design's review
must look for first.**

**Where `ReadEnd` lands.** On the owner lane, at the in-order commit of the slot
that produced it, exactly where the source-order execution produces it. It is
delivered to the writer's `match` as the outcome of that iteration's `read_at`;
it triggers the discard of every later slot; then the `break @chunk` edge is
taken, and the carried `at`/`sum` places hold their committed (source-order)
values at the loop's continuation.

**What the writer sees on a short read.** Nothing. Slot i delivers
`ReadBytes(next)` with `next < end`; the writer's `ReadBytes` arm runs with that
exact `next`; slots i+1.. are discarded and re-submitted at
`committed + (next - start)`. The published bytes are the source-order bytes.
The only cost is one wasted `pread` per short read, and on a regular file a
short read is followed almost always by `ReadEnd` on the very next chunk.

**The three prerequisite bugs are the same ones.** LOOP-PIPELINE.md §3.6's two
latent defects (the adapter retaining the caller's path pointer,
`bridge.c:722`/`linux_io_uring.c:495`; `%component` being one static buffer per
call site, `compiler/src/backend/emitter/system.rs:1952`) and §3.10's
match-scrutinee gap (`compiler/src/lowering/builder.rs:739-757`, which is why a
`match read_at(...)` scrutinee submits nothing today) are prerequisites here
too. The chunk loop writes its read as a `match` scrutinee, so §3.10 is not
optional for Part A — it is the first thing that must land.

## A.6 The ledger line a writer sees

The format is fixed by `compiler/src/semantic/permission_ledger.rs:238` for the
stage line and `:251` for the place lines. For `checksum_file` above:

```text
PAR stage       checksum.wf:6    loop  permitted   staged at read_at<'f, 'c>(file: file, destination: &uniq 'c block, file_offset: at, start: 0_u64, end: room); 3 places classified
PAR place       checksum.wf:6    read-only         file            no footprint of the body writes it and every loan on it is shared
PAR place       checksum.wf:6    carried-predicted at              the body advances it by one total operation over the read's own outcome, so the prologue runs on a value a later commit may correct
PAR place       checksum.wf:6    serialized-E      sum             every footprint touching it belongs to the remainder, and the remainder's writes commit in iteration order
```

and the denial a writer gets when the recurrence is not admitted — the one new
denial text, condition 8:

```text
PAR stage       checksum.wf:6    loop  denied      condition 8: the body advances `at` with an operation this judgment cannot re-execute; instead, advance a place the prologue reads with one total operation — `+wrap` and the bit operations are total, `+defined` and `+checked` are not — or leave this loop sequential, at set at = at +defined taken;
```

Two things about this table are deliberate. First, `carried-predicted` prints
its cost in the reason, because the writer should know that a short read costs
one wasted read — that is teaching, not a knob. Second,
`docs/patterns.md:459-464` currently tells the writer that one file's chunk loop
cannot be staged; that paragraph is **superseded in place** by this design, and
the same change replaces P15's closing sentence about `treelines.wf`,
`checksum.wf`, `dir_walk.wf` and `wfgrep.wf`
(`docs/done/0100-writer-defaults-2.md:764-765`). A design that leaves that
paragraph standing has not landed.

## A.7 The safety argument

**No memory corruption.** Every destination is a replicated ring slot the loop
owns; a discarded slot is not released until its operation reaches terminal
(A.5). [SYS-8]'s two static range obligations
(`spec/kernel-spec.md:2540-2545`) are discharged per call exactly as today, and
the sanitized-count rule (`spec/kernel-spec.md:2570`) still bounds `next`.
Nothing about speculation touches either.

**No data race.** The only places two segments reach are: `file` (shared loans
only — `spec/kernel-spec.md:2060`'s first alternative), the replicated slots
(one per in-flight iteration), and the carried places, which live in exactly two
owner-lane registers written only by the owner lane. The fold, if handed to a
compute lane, is handed the slot as its frame exactly as
LOOP-PIPELINE.md §3.5 hands one, and joins before the commit that reads it.

**No uninitialized read.** A ring slot holds the value its `buffer_new`
constructed over its whole capacity (LOOP-PIPELINE.md §3.3's slot invariant), so
the iteration-own case needs no coverage proof — which matters, because
`spec/kernel-spec.md:2069` currently forbids deriving a written byte from
[SYS-8]'s "may have changed" wording, so [PAR-3]'s replication clause cannot yet
privatize a *hoisted* buffer at all. Part A does not need it to.

**No claim is removed.** Nothing here reads or weakens a `claim`. A false claim
inside a body with slots in flight is [PAR-3]'s existing erroneous-execution
case verbatim (`spec/kernel-spec.md:2074-2078`): one [DIAG-3] record, abort
without unwinding, in-flight operations abandoned by process teardown, and **no
latch on the correct path**.

**The two honest limits.** (i) A discarded `pread` still touches the host: it
updates `st_atime` under the host's own policy and consumes I/O bandwidth.
Neither is a Whitefoot state place and neither is reachable through any
operation [SYS-2] declares, but the discardability sentence should be read as a
claim about *this specification's* observables, and A.3.1 words it that way
rather than as a claim about the host. (ii) A file another process modifies
between a discarded attempt and its replay can deliver different bytes. That is
already outside the model — `spec/kernel-spec.md:2627` says "Environment-created
changes to the same physical file do not merge or mutate Whitefoot places" —
and a sequential program re-reading the same offset has the same exposure.

**Why pipes are not a hazard.** `read_at` lowers to `pread`
(`compiler/src/backend/emitter/system.rs:409,1440`) and to `IORING_OP_READ` with
an explicit `off` (`compiler/src/backend/completion/linux_io_uring.c:512,515`).
Both fail with `ESPIPE` on a non-seekable object, and `ESPIPE` (29) appears in
neither errno class table (`compiler/src/backend/qualification.rs`,
`LINUX_ERROR_CLASSES` and `DARWIN_ERROR_CLASSES`), so it reaches source as
`Other`. **A pipe cannot be read by `read_at` at all today**, so no discarded
attempt can ever consume a byte from a stream. When a positioned-stream or
`Source` type is added, its read must not be marked discardable — that is a
one-line obligation on the future record, and A.3.1's closing sentence states
it.

## A.8 The differential oracle

Three oracles, in increasing cost, each answering a different question.

**O1 — identity, compiler against itself.** One program, four builds:
`--no-overlap` (the S line, K = 1 by construction), default, `--par`, and
facts-off. All four publish byte-identical output over a corpus chosen to force
every prediction outcome:

| file | size | what it exercises |
|---|---|---|
| exact | 4 x 65536 | no short read, no mispredict |
| ragged | 4 x 65536 + 1 | one short read at the last chunk |
| short | 3 | one chunk, immediate `ReadEnd` on chunk 1 |
| empty | 0 | `ReadEnd` on chunk 0, every prefetched slot discarded |
| big | 1 GiB | steady state, K deep |

**A green run of O1 proves nothing on its own** — the identity holds trivially
if the pipeline never fired. So O1 is paired with a mechanism counter:
`wf__completion_file_submissions()` (`bridge.h:161`) must be at least
`ceil(size / 65536) + 1` for `big`, and a new
`wf__completion_file_discarded()` counter must be **0** for `exact` and **≥ 1**
for `ragged` and `empty`. That pairing is the test; the identity alone is not.

**O2 — identity, compiler against C.** A standalone C program doing the same
K-deep prefetching `pread` loop over the same files, publishing the same
checksum. Compiler-independent, and it is the oracle that would catch a wrong
`next` (a sanitized-count defect) that O1 cannot see because both sides of O1
share the defect.

**O3 — the observable-attempt oracle.** `strace -f -e trace=pread64,io_uring_enter`
(Linux) on `exact`: every `pread` offset the program issues must be a multiple
of 65536 and every offset in `[0, size)` must appear at least once. On `ragged`:
exactly one offset appears twice — the one re-issued after the short read — and
no offset outside `[0, size + 65536)` is ever issued. This is the oracle that
proves the discard-and-replay is doing what the rule says rather than something
that happens to produce the same bytes.

## A.9 The falsifier

**F1 — free, before any measurement.** `whitefootc --par-ledger` on
`checksum_file` above must print the four-line table of A.6, `permitted`, with
`at` classified `carried-predicted`. A denial names its condition and its node.
If the denial is condition 8, the recurrence shape test is wrong about what
`+wrap` over a payload binder looks like. Costs nothing and fires first.

**F2 — the submission counter.** `wf__completion_file_submissions()` is **0**
today for a `match read_at(…)` scrutinee (`builder.rs:739-757`) and must be
`ceil(size/65536) + 1` after. If it is 0, nothing actualized whatever the ledger
said, and the gap is in lowering, not judgment.

**F3 — the wall clock, and this is the owner's bar.** One new maintained
program, `read_heavy.wf`: open one file, fold it in 64 KiB chunks, publish one
checksum. Against a hand-written C loop that keeps the same number of `pread`s
in flight through io_uring, on the Linux CI runner, over a 1 GiB file:

```text
  REQUIRED  Linux  cold cache  C.read_heavy  <=  1.10 x  the C prefetch loop
  REQUIRED  Linux  warm cache  C.read_heavy  <=  1.10 x  the C sequential pread loop
  control   both   S.read_heavy (--no-overlap) unchanged before/after within spread
  control   both   published checksum bytes identical on every recorded run
```

The two REQUIRED lines are different bars on purpose. Cold, depth is everything
and the pipeline should track C closely. **Warm, depth buys nothing and the ring
costs about 3x a blocking syscall in CPU on this host**
(LOOP-PIPELINE.md:69-72: 2.5 us/op through the ring against 0.82 us/op direct),
so the warm line is the one at risk, and the honest answer if it fails is a
runtime policy that uses the direct path when the window query returns a small
K — not a language change. **Both numbers must be probed before a line of
compiler is written**, exactly as LOOP-PIPELINE.md §5.5 probed before batch 0089.

**F4 — the leak test.** `many_files_narrow.wf` and `many_files_wide8.wf` must
be byte-unchanged. Neither loop's cut submission is discardable, so neither
reaches any new path. A regression there means the (2a) table lookup is keyed on
something other than the semantic ID.

**F5 — the discriminating pair.** Two maintained programs differing in one
token, both with checked published checksums, one permitted and one denied:

```whitefoot
  set at = at +wrap taken;      // permitted, carried-predicted
  set at = at +defined taken;   // denied, condition 8
```

`+defined` attaches a domain obligation to every application
(`spec/kernel-spec.md`, [PAR-2]'s own reasoning about the same operator set), so
re-executing it is not the same as executing it once. A privatization or
speculation test that passes because the mechanism never fired proves nothing.

**F6 — facts-off identity.** Acceptance and published bytes must not move with
the entailment state degraded. This judgment reads no fact, so F6 should be free
— and it is pinned by a test, not asserted in a comment.

## A.10 What it costs

| component | file | lines |
|---|---|---|
| match-scrutinee call results (prerequisite, standalone) | `compiler/src/lowering/builder.rs` | ~80 |
| discardable flag on the [SYS-2] record + resolution | `compiler/src/resolution/catalog.rs`, `semantic/` | ~40 |
| condition 2 alternative (2a)+(2b) | `semantic/staged_permission.rs` | ~120 |
| `Carried(Grade)` disposition + condition 8 recurrence shape test | `semantic/staged_permission.rs` | ~260 |
| judgment tests (each condition, each grade, the denied pair) | `semantic/tests/` | ~300 |
| ledger `carried` / `carried-predicted` rows and denial text | `semantic/permission_ledger.rs` | ~60 |
| pipeline IR: slots, ring, driver, speculative/committed registers, discard | new `lowering/builder/pipeline.rs` | ~900 |
| back-edge-tolerant joins, discard and drain blocks | `backend/emitter/completion.rs`, `emitter.rs` | ~400 |
| per-operation-record path storage; slot-indexed completion storage (prerequisites) | `completion/bridge.c`, `emitter/completion.rs`, `emitter/system.rs` | ~380 |
| window query + weak fallback | `completion/bridge.c`, `emitter/completion.rs` | ~60 |
| `wf__completion_file_discarded()` counter + cancel path | `completion/bridge.c`, `linux_io_uring.c` | ~120 |
| backend tests (discard-before-release, drain on exit, `--no-overlap` parity) | `backend/tests/` | ~450 |
| `read_heavy.wf`, the C oracle, harness wiring | `research/experiments/io-completion-bench/` | ~300 |
| conformance cases and verdicts | `conformance/` | ~200 |
| spec, `docs/patterns.md` P15 supersession, `docs/done/` | `spec/`, `docs/` | ~250 |
| **total** | | **~3,900** |

**New APIs the writer sees: none.** New runtime mechanism: one counter, one
optional cancel, and the discard path — everything else is the pipeline chassis
LOOP-PIPELINE.md already costs. Of the total, ~1,000 lines are the prerequisites
that batch 0095 owes anyway.

## A.11 What the writer must write differently, and why it is not a hidden trick

**One thing, and it is already the taught form.** The destination buffer moves
from above the loop to inside the body:

```whitefoot
  let scratch = buffer_new(65536_u64, 0_u8);      // before: hoisted, denied at condition 3
  loop @chunk { … read_at(destination: &uniq 'c scratch, …) … }

  loop @chunk {                                   // after
    let block = buffer_new(65536_u64, 0_u8);      // iteration-own, replicated
    … read_at(destination: &uniq 'c block, …) …
  }
```

That is `docs/patterns.md:334-366` (P15) verbatim, landed before this design and
for an independent reason. Nothing about the offset, the exit, the depth, or the
prefetch appears in the source. There is no batch call, no window, no
`prefetch(…)`, no attribute, no `par loop`, and no environment variable.

**Why moving the buffer is not a hidden trick.** Three reasons, and the third is
the one that settles it:

1. It is an *ownership statement the writer means*: this storage belongs to this
   iteration. It is written in the same place a writer would write it in any
   language that had no allocation cost to worry about.
2. It removes a real order dependence rather than hiding one. With one reused
   buffer, a short read leaves the previous chunk's bytes above `next`, and a
   fold that reads past `next` publishes them — `docs/patterns.md:342-344` says
   exactly this. The per-iteration form is the one whose output does not depend
   on the schedule.
3. **It costs nothing at runtime and the specification already says so.**
   `spec/kernel-spec.md:2079` puts "the storage an implementation reuses across
   iterations for a construction whose value the body releases without observing
   it" outside the observable set, so the implementation allocates the K slots
   once at loop entry and restores them. The writer writes the honest form and
   the compiler is *permitted*, not obliged, to make it free.

**One thing the writer must know, and it belongs in `docs/patterns.md`, not in
the source.** A helper that takes the scratch as a parameter —
`fn count_file(file: &ReadFile, scratch: &uniq buffer<u8>)`, which is what the
blind writer wrote — pushes the buffer back outside the loop from the judgment's
point of view. The repair is for the helper to construct its own buffer and
declare `allocates(heap)`, which is a better signature anyway: it stops the
caller from being able to hand in a buffer whose leftover bytes change the
answer. P15 gains that paragraph.

**And one thing the compiler must not do.** It must not silently rewrite a
hoisted buffer into a per-iteration one. That is the transformation
`spec/kernel-spec.md:2069` already forbids deriving without a coverage proof,
and a whole-place "write before read" rule would accept the loop-carried-scratch
program of LOOP-PIPELINE.md §2.3 and silently miscompile it. **Warn and teach**
— the denial at condition 3 already names the buffer and the repair
(`staged_permission.rs:423`).

## A.12 Open questions for the owner (Part A)

1. **Is a discarded host attempt acceptable?** This is the one genuinely new
   thing in Part A: the implementation performs `pread`s the source-order
   execution never performs. Nothing in [SYS-2] observes them, and `read_at`'s
   own contract makes them free of state change, but they are real syscalls with
   real `atime` and real bandwidth. The alternative that avoids them entirely is
   `carried-closed` only — which admits Part B's list loop and **not** the
   chunk loop, because a chunk loop's offset provably depends on the outcome.
   **Approve discardable attempts, or ship `carried-closed` only and leave the
   chunk loop sequential?**

2. **The warm-cache bar.** Cold, the pipeline should track the C prefetch loop.
   Warm, depth buys nothing and the ring costs ~3x a blocking syscall in CPU on
   the measured container (LOOP-PIPELINE.md:69-72), so a warm 1 GiB fold may be
   *slower* pipelined than sequential. The clean answer is a runtime policy —
   the window query returns K = 1 and the direct path is taken — but that is a
   policy the runtime cannot make well without knowing whether the file is in
   cache. **Is "warm-cache parity within 10 %, cold-cache within 10 % of C" the
   right bar, or should warm-cache regression be forbidden outright?**

3. **The streaming copy loop stays denied.** `read chunk, write chunk` holds an
   exclusive loan on `Output` in the remainder, and condition 4 denies it
   (`staged_permission.rs:326`). That is the standing D1/W5 question
   (`docs/done/0100-writer-defaults-2.md:766-772`), not this design's, but Part
   A makes it much more visible: `cat`, `cp` and every filter are exactly that
   shape, and after Part A lands the *fold* is fast and the *copy* is not.
   **Does the per-iteration-publish question become the next project?**

4. **Float recurrences.** Condition 8 admits `fadd.strict` on a carried place,
   because this rule re-executes rather than recombines and therefore uses no
   associativity — unlike [PAR-2], which explicitly refuses float folds
   (`spec/kernel-spec.md`, [PAR-2]'s admitted-operation paragraph). That is
   correct but it is the first place in the language where a float fold rides a
   permitted overlap. **Confirm, or exclude floats for now?**

5. **Should `carried-predicted` be reported even when granted?** A granted loop
   says nothing today (`docs/patterns.md:445-450`). But a `carried-predicted`
   loop has a performance cliff a writer might want to know about — a file
   served by a filesystem that always short-reads would mispredict every chunk.
   A `PAR place` line exists for it in `--par-ledger`; the question is whether it
   should also reach the default channel. **Silent, or a note?**

---

# Part B — bytes to path

## B.0 The proof a path must carry, and who already discharges it

Start from the host call, not from the type. Before `openat(dirfd, p, flags)`
may run, four facts must hold about `p`:

| # | fact | why it is required | who proves it today |
|---|---|---|---|
| 1 | the bytes are readable memory of a stated length | memory safety | [SYS-8]'s two static obligations, `spec/kernel-spec.md:2540-2545` |
| 2 | the bytes contain no NUL inside the range | a NUL truncates the C string and opens a *different file* | the operation, `spec/kernel-spec.md:2683`; emitter `component_validation`, `compiler/src/backend/emitter/system.rs:1730-1755` |
| 3 | the bytes name no target root | otherwise `dirfd` is bypassed and the "root" is meaningless | [PATH-1], `spec/kernel-spec.md:2387` for a `RelativePath`; the separator refusal for a name range |
| 4 | the backing outlives the call | dangling pointer | [HOST-3]'s command-lifetime snapshot for a `RelativePath`, `spec/kernel-spec.md:2378`; an ordinary borrow for a name range |

Facts 1, 2 and 3 are **already discharged for bytes a program read** — that is
the finding. `sizes.wf`, the third blind writer's list-file tool, opens files
whose names it read out of a file, right now, on this branch:

```whitefoot
match open_file<'g2, 'n2>(permit: move permit2, root: &'g2 cwd,
                          name: &'n2 content, start: begin, end: index) {
```

(`$SCRATCH/wf-0100-verify/writer/work/sizes.wf:117`, where
`content` is a 64 KiB buffer holding the whole list file). It works for
`flat.txt` — `a.txt`, `b.txt`, `big.bin`. It fails for `list.txt` —
`sample/a.txt` — and for exactly one reason: `spec/kernel-spec.md:2676`

> A name is one path component: it is never empty, never longer than the
> target's component limit, and contains no NUL and no target separator, so no
> record a program reads can name more than one component.

**So the missing proof is not about lifetime, ownership, or provenance. It is
one validation: that a byte range is a well-formed relative path rather than a
well-formed single component.** Everything the record at
`docs/done/0100-writer-defaults-2.md:787-801` frames as a lease-model problem —

> "the only `RelativePath` constructor is `relative_path(value: own HostString)`,
> the only `HostString` source is `arg_get` … The question for the owner is
> whether the lease model should gain a second backing class"

— is a problem only for the `RelativePath` *route*. The name-range route is
already open, already borrow-backed, already validated, and already
pipeline-friendly. It needs one more validation mode, not a second backing class.

**Why not a new type.** `spec/kernel-spec.md:2381` predicts what a new type
would cost, and predicts it correctly:

> A producer whose backing is not command-lifetime yields no value of this type:
> it introduces a distinct owned-backing string resource with its own release
> action and its own type contract, because storage class is a function of type
> [STOR-1] and one type carries exactly one release action.

A path over program-owned bytes is therefore either (a) a new *owned* string
resource with its own allocation and release — a heap copy of every path, a new
release row in [SYS-5], and a new affine value in every loop body; or (b) a
*region-bearing* opaque nominal, the first one in the language, which collides
with [STOR-5]'s borrow-free storage and with [HOST-3]:2379's "a lease is neither
a borrow nor a region-bearing type". Both are large, and neither buys a fact the
range route does not already have. **Design B takes neither.**

## B.1 The writer-facing form, as it would compile

Two operations are added, differing from their existing siblings only in the
validation they perform and in the parameter's name:

```whitefoot
fn open_file_path['c, 'n](permit: own FilePermit, root: &'c DirectoryRead,
                          path: &'n buffer<u8>, start: own u64, end: own u64)
  -> result: own Result<ReadFile, IoError>
  reads(permit, root, path), writes(permit);

fn open_directory_path['c, 'n](permit: own FilePermit, root: &'c DirectoryRead,
                               path: &'n buffer<u8>, start: own u64, end: own u64)
  -> result: own Result<DirectoryRead, IoError>
  reads(permit, root, path), writes(permit);
```

The list-file tool the blind writer could not finish, written against them —
this is the whole loop, and it stages (B.6):

```whitefoot
  // `content[0..filled]` holds the list file; one line names one path.
  let total = 0_u64;
  let begin = 0_u64;
  for @scan index in 0_u64..filled {
    let byte = content[index];
    let eol = ieq(byte, 10_u8);
    if eol {
      let nonempty = ilt(begin, index);
      if nonempty {
        region 'g {
          let permit = reserve_file<'g>(factory: &uniq 'g files);
          region 'p {
            match open_file_path<'g, 'p>(permit: move permit, root: &'g cwd,
                                         path: &'p content, start: begin, end: index) {
              Ok(value: target) => {
                region 'q { set total = total +wrap count_file<'q>(file: &'q target); }
              }
              Err(error: problem) => { }
            }
          }
        }
      }
      set begin = index +wrap 1_u64;
    }
  }
```

`sample/a.txt` opens. `sample/../sample/a.txt` opens the same file.
`/etc/hosts` returns `Err(InvalidPath(code: 0_u32, origin: 0_u8))` before any
host call. `sample//a.txt` likewise. The `buffer_vacant<DirectoryRead>` +
`replace` descent of
`$SCRATCH/wf-0100-verify/writer/work/largest.wf:55,78,91,198`
is no longer the only way to reach a nested file, and it costs one permit and
one host call instead of one per component.

**Why not widen `open_file` itself.** Because the refusal of a separator is a
guarantee existing programs lean on. A walker that hands an *enumerated entry
name* to `open_file` relies on `spec/kernel-spec.md:2676` to stop a hostile or
merely surprising directory entry from naming something else. Widening the
existing operation would remove that net silently from every call site. Two
operations, each with one fixed contract, is the shape the language already uses
for `open_read` (a `RelativePath`) beside `open_file` (a name range).

## B.2 Spec sentences

### B.2.1 [PATH-1] — what the deferral actually defers

Today, verbatim (`spec/kernel-spec.md:2391`):

> A path component type, an absolute path type, and every operation that
> decomposes, enumerates, joins, or displays a path are DEFERRED additions with
> their own deltas [META-5].

That sentence is unchanged and this design adds none of those four operations.
Add one sentence after it, because a reader will otherwise take the deferral to
cover more than it does:

> Admitting one externally supplied multi-component relative path is not path
> algebra: this specification declares no operation that decomposes, joins,
> normalizes, or displays a path, and an operation that validates a caller-owned
> code-unit range as one relative path assembles nothing.

### B.2.2 [SYS-14] — the sentence that has to move

Today, verbatim (`spec/kernel-spec.md:2680-2681`):

> An entry name reaches source only as those bytes.
> This specification declares no operation turning an enumerated name into a
> `HostString` or a `RelativePath`, because a name's backing is not the
> command-lifetime argument snapshot [HOST-3] and a path value is an inline
> lease over that snapshot [PATH-1].
> `open_directory` and `open_file` therefore take a caller-owned name range
> rather than a path value, and path composition remains the DEFERRED addition
> [PATH-1] states.

Keep the first two sentences exactly — they remain true, and they are the reason
this design adds no type. Replace the third with:

> `open_directory` and `open_file` therefore take a caller-owned name range
> rather than a path value, and `open_directory_path` and `open_file_path` take
> a caller-owned range validated as one complete relative path. Neither route
> yields a path value, and path composition remains the DEFERRED addition
> [PATH-1] states.

### B.2.3 [SYS-14] — the new validation, beside the existing one

Today, verbatim (`spec/kernel-spec.md:2683`):

> Each then validates `[start, end)` as one component before any host call: a
> component that is empty, longer than the target's component limit, or
> containing a NUL or a target separator yields
> `Err(InvalidPath(code: 0_u32, origin: 0_u8))`, no host call, and no resource
> value.

Add, after it:

> `open_directory_path` and `open_file_path` instead validate `[start, end)` as
> one complete relative path before any host call. The range is one or more
> components separated by exactly one target separator each. It yields
> `Err(InvalidPath(code: 0_u32, origin: 0_u8))`, no host call, and no resource
> value when it is empty, when it begins with a target-root prefix [PATH-1],
> when it contains a NUL, when any component is empty — so a leading, trailing,
> or repeated separator is refused — when any component is longer than the
> target's component limit, or when the whole range is longer than the target's
> path limit. Every admitted code unit is preserved exactly, including `.` and
> `..` components: this operation performs no normalization, canonicalization,
> case folding, prefix stripping, or component collapse [PATH-1].
> The target's path limit used by this version's Darwin-family approved
> implementations is 1024 bytes and by its Linux-family approved implementations
> 4096 bytes [QUAL-1].

And, for the resolution promise, after `spec/kernel-spec.md:2687` ("On success
`open_directory` returns an independent `DirectoryRead` … a symbolic link is not
followed by either operation."):

> `open_directory_path` and `open_file_path` resolve their whole validated range
> in one act of the target's own directory-relative resolution [PATH-2], never
> by concatenating a prefix and never by opening the components in turn. A
> symbolic link named by the range's last component is not followed; a symbolic
> link, `.` component, `..` component, or mount transition named by an earlier
> component is followed exactly as the surrounding process namespace follows it,
> so a resolved object may lie outside the directory the `root` value names.
> That is the process equivalence [PATH-2] already fixes for this type and it is
> not a confinement claim: this specification promises no relation between the
> resolved object and the root, and promises nothing about a rename or a link
> change performed by another party while the resolution runs. A confined
> directory state type, whose operations would promise both, remains the
> DEFERRED addition [PATH-2] states.

### B.2.4 [SYS-8] — the range-bearing family

`spec/kernel-spec.md:2537` names the complete range-bearing set. Both new
operations join it, so its opening sentence gains two names and every clause
that already covers `open_file` — the two static obligations, the empty-range
rule at `spec/kernel-spec.md:2549`, the observed range — covers them unchanged.
`spec/kernel-spec.md:2294`'s `loan-released(path)` sentence likewise gains them:
forming the request copies the admitted range into compiler-owned storage, so
the borrow is released before target transfer, exactly as for a name.

### B.2.5 META-5 delta shape (Part B)

> Numbered rules +0/-0 (137 remain); grammar productions +0/-0; keywords +0/-0;
> opaque system nominal spellings +0/-0; outcome types +0/-0; `IoError` classes
> +0/-0 (28 remain); writer operation spellings **+2/-0**; system operations and
> declaration records **+2/-0** (205 remain); runtime-trap families +0/-0.
> [SYS-14] is amended to declare `open_directory_path` and `open_file_path`,
> which differ from `open_directory` and `open_file` only in validating their
> caller-owned range as one complete relative path rather than as one component,
> and to state that both resolve that range in one act of the target's own
> directory-relative facility with the process equivalence [PATH-2] already
> fixes. [PATH-1] is amended by one sentence distinguishing admission of an
> externally supplied path from path algebra, which remains deferred. [SYS-8]'s
> range-bearing set gains both names, with no change to any clause. No new type,
> backing class, release action, error class, or static obligation is added, and
> no accepted program changes meaning.

## B.3 The judgment

**There is none, and that is the point.** The static proof a path operation
needs is exactly the proof [SYS-8] already demands of every range-bearing
operation, verbatim (`spec/kernel-spec.md:2540-2542`):

> Every call to a member of this family carries exactly two independent [ENT-6]
> obligations in this order: `start <= end`, then `end <= len(deref(buffer))` …
> Both obligations are queried in the caller's pre-transfer state and must be
> derived independently; neither is a premise for the other.
> A refuted or unproved obligation rejects the call under [ERR-4].

Those two are what protect memory. Everything else about a path — separators,
NULs, roots, empty components, limits — is a property of *content*, not of
storage, and content is checked at runtime and reported as a typed outcome.
That split is already the language's: `spec/kernel-spec.md:2543` says "There is
no operation-internal range check, runtime fallback, or range trap", and
`spec/kernel-spec.md:2683` puts the content check in the operation.

So Part B adds **zero** semantic-checker code. The resolution catalog gains two
rows; the checker sees two more members of a family it already handles.

## B.4 Lowering

`compiler/src/backend/emitter/system.rs:1730-1755` already emits the component
validation as a straight-line scan:

```llvm
measure:  %oversize = icmp ugt i64 %extent, {component_limit}
          %vacant   = icmp eq  i64 %extent, 0
          br i1 %unusable, label %invalid, label %scan.entry
scan:     %byte = load i8 ...
          %terminating = icmp eq i32 %byte.value, 0
          %separating  = icmp eq i32 %byte.value, {root}
          br i1 %refused, label %invalid, label %scan.step
```

The path validation is the same loop with three edits, and it is the whole
backend delta:

1. `%oversize` compares against the target's **path** limit, and a running
   per-component counter compares against the component limit and resets at each
   separator.
2. `%separating` no longer refuses: it checks that the previous byte was not a
   separator (no empty component) and that the index is not 0 (no leading
   separator), then resets the component counter.
3. after the scan, refuse when the last component is empty (trailing separator).

`%terminating` (the NUL check) is unchanged, and so is the copy into the bounded
stack slot, the NUL terminator, and the single `openat` — except that the slot is
sized `path_limit + 1` rather than `component_limit + 1`, and the flags are the
existing `component_file_open_flags` / `component_directory_open_flags`, which
already carry `O_DIRECTORY`, `O_NOFOLLOW` and `O_NONBLOCK`
(`compiler/src/backend/qualification.rs:917,948,966`). `SystemTarget` gains one
field, `path_limit`, beside `component_limit`
(`qualification.rs`, the `for_triple` tuple).

`open_file_path`'s post-open descriptor-status inspection is `open_file`'s,
unchanged (`spec/kernel-spec.md:2685-2686`): a successfully inspected directory
returns `Err(IsDirectory(...))`, every other non-regular object `Err(Other(...))`,
after exactly one native close attempt.

## B.5 The errno mapping

**No new class, and no new table row.** Verified against
`compiler/src/backend/qualification.rs`, `LINUX_ERROR_CLASSES` and
`DARWIN_ERROR_CLASSES`:

| condition | Linux | Darwin | [SYS-7] class | reached |
|---|---|---|---|---|
| empty range, NUL, root prefix, empty/oversize component, oversize path | — | — | `InvalidPath(0, 0)` | before any host call |
| a component does not exist | `ENOENT` 2 | 2 | `NotFound` | host |
| an intermediate component is not a directory | `ENOTDIR` 20 | 20 | `NotDirectory` | host |
| search permission missing on an intermediate directory | `EACCES` 13 | 13 | `PermissionDenied` | host |
| the last component is a symbolic link (`O_NOFOLLOW`) | `ELOOP` 40 | `ELOOP` 62 | `InvalidPath` | host |
| a symbolic-link cycle in an intermediate component | `ELOOP` 40 | 62 | `InvalidPath` | host |
| the host's own path or component limit is exceeded | `ENAMETOOLONG` 36 | 63 | `InvalidPath` | host |
| descriptor exhaustion | `EMFILE`/`ENFILE` 24/23 | 24/23 | `ResourceExhausted` | host |
| the resolved object is a directory (`open_file_path`) | — | — | `IsDirectory(0, 0)` | post-open inspection |

The one entry worth arguing is `ELOOP → InvalidPath`. It is already the mapping
on both targets, it is already what a symlinked *component* name produces today,
and it means a refused final symlink and a bad path shape share one class. That
is a real loss of distinction, and [SYS-7] anticipates it
(`spec/kernel-spec.md:2528`): `code` carries the native errno and `origin`
carries `ORIGIN_DIRECTORY_OPEN` (`qualification.rs`), so a program that wants to
tell them apart reads the detail and reports it — it just cannot branch
portably on it. `spec/kernel-spec.md:2531` fixes exactly that: "The detail is
diagnostic data, not a portable discriminator".

## B.6 Composition — with the permit ceremony, and with the pipeline

**The permit ceremony is unchanged.** `open_file_path` takes
`permit: own FilePermit`, exhibits `reads(permit, root, path), writes(permit)`,
and consumes the permit on every outcome — one permit, one attempt,
`spec/kernel-spec.md:2605`. A multi-component path costs **one** permit and
**one** host call. The component-descent workaround costs one permit and one
host call per component and leaves an intermediate `DirectoryRead` to release at
each level; that is the concrete saving, and it is why `largest.wf`'s
`buffer_vacant<DirectoryRead>` stack is a walker's tool and not a path opener's.

**The pipeline stages the list loop.** Running B.1's loop against the [PAR-3]
conditions as `compiler/src/semantic/staged_permission.rs` implements them:

| place | disposition | why |
|---|---|---|
| `content` | **read-only** | the body never writes it; `open_file_path`'s `&'p content` is shared, and two shared loans deny nothing (`spec/kernel-spec.md:2060`, first alternative) |
| `cwd` | **read-only** | same |
| `files` | **serialized-P** | every access is `reserve_file`'s inline `&uniq` loan, which ends when that `never-suspends` operation returns (`spec/kernel-spec.md:2603`); prologues run in index order and never overlap (`spec/kernel-spec.md:2063`) |
| `begin` | **carried-closed** (Part A) | `set begin = index +wrap 1_u64;` names no payload binder, so the recurrence closes in P and is hoisted above the submission |
| `total` | ordered write | written in E only, committed in iteration order (`spec/kernel-spec.md:2064`) |

Every exit is in P (there is none in the body at all), the cut is
`open_file_path`, and the loop is **permitted**. Note that `begin` is the reason
this loop needs Part A at all: without the `carried` disposition it is a place
the prologue reads and the remainder writes, which
`spec/kernel-spec.md:2060`'s three alternatives do not cover, and the loop
denies at condition 5. **That is the sharpest argument for building Part A's
`carried-closed` grade even if the owner declines speculation**: it is what makes
the ordinary "scan a buffer, act on each record" loop stage, and it needs no
discarded host attempt at all.

## B.7 The safety argument

**What is promised.** Exactly four things, and they are the four of B.0:
the range is proven in bounds before the call ([SYS-8]); no NUL reaches the host
inside it, so the file opened is the file the bytes name; no target-root prefix
reaches the host, so the `root` value is never bypassed; and the whole range is
resolved by the target's own directory-relative facility, never by string
concatenation against an ambient working directory
(`spec/kernel-spec.md:2398`, which already makes prefix concatenation a
qualification failure rather than an implementation choice).

**What is not promised, said out loud.** The resolved object may lie outside the
directory `root` names. `..` is preserved and followed. An intermediate symlink
is followed. A rename or link change performed by another party during
resolution is not excluded. **There is no TOCTOU the language promises away**,
and B.2.3 says so in the rule rather than leaving a reader to infer it from
`spec/kernel-spec.md:2396-2397`:

> The value bound to the command's working-directory entry input is
> process-equivalent: resolution follows `.` and `..` components, symbolic links,
> reparse points, and mount transitions exactly as the surrounding process
> namespace does, so a resolved object may lie outside the directory that value
> names.
> That is the complete promise this type makes, and it is not a confinement
> claim.

**Then what does "no path traversal by accident" mean here?** Precisely this:
*a program can only open a path it named.* It cannot acquire one by
concatenation the language performed on its behalf (there is none), by a
truncation at an embedded NUL (refused), by an absolute path silently escaping a
relative open (refused), or by an empty component collapsing two separators into
something the program did not write (refused). What it *can* do is open
`../../etc/passwd` when its own input says `../../etc/passwd` — and for a
list-file utility, an `xargs`, or a build tool, that is the correct behavior, not
a defect.

**The case this does not serve, and the honest sequencing.** A program that
treats its list file as *untrusted* and must confine every open beneath a root
is exactly what `spec/kernel-spec.md:2399` defers:

> A confined directory state type, one guaranteeing that lexical traversal,
> links, mount transitions, and rename races cannot escape a granted root, is a
> DEFERRED addition with its own distinct contract [META-5]; a value's
> confinement promise is fixed by its type and never changes at runtime.

That sentence also rules out the tempting shortcut: confinement may not be a
flag on the operation or on the value, because a value's promise is fixed by its
type. So the future shape is a distinct `ConfinedDirectory` nominal whose opens
lower to `openat2(RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS)`
on Linux and which **fails qualification** on Darwin, which has no equivalent —
a target with no such facility fails qualification for those IDs rather than
emulating them (`spec/kernel-spec.md:2419-2421`). Emulating it by per-component
`openat` with `O_NOFOLLOW` is *not* equivalent: it still races on rename between
components. **Do not build per-component descent and call it confinement.** That
is the single most likely wrong turn in Part B, and the reason this design
resolves in one host call.
