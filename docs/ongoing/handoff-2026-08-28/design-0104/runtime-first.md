# Runtime-first: the streaming chunk loop, and bytes to path

Design record, 2026-08-28. Written against `integration/2026-08-28b` at
`16228216`, which carries batches 0093-0100. Every citation below was read in
that revision; a `file:line` in this document means that revision's bytes, not
the current worktree's (`compiler/src/semantic/staged_permission.rs` and
`docs/patterns.md` both differ in the dirty `codex/io-first-principles`
worktree, so line numbers were taken from a clean export of `16228216`).

**Method.** Both parts are derived in one direction only: start from what the
target and the completion runtime can already do at close to zero marginal
cost, and expose exactly that and nothing more. Where a design needs a
mechanism the runtime does not have, that is recorded as a cost, not hidden in
a language feature. The two parts turn out to be very different sizes under
that discipline, and the asymmetry is the main finding:

- **Part A** needs **no new runtime mechanism at all** — no cancellation, no new
  submit path, no second queue. It needs one language rule that says a read the
  source order never performs, into storage the writer cannot name, publishes
  nothing. The whole cost is in the compiler.
- **Part B** needs **no new runtime mechanism and no new language type** — the
  host call `file_adapter.c:172` already makes takes a whole multi-component
  path and does not care how many components it has. The whole cost is two
  amended specification sentences and one generalized validation.

Neither part adds a writer-visible knob, a batch API, a depth, or an attribute.

---

# Part A — the streaming chunk loop

## A.0 What the runtime can already do, at what price

This is the inventory the rest of Part A is derived from. Nothing below is
proposed; all of it is in the tree at `16228216`.

| capability | where | marginal cost |
|---|---|---|
| positioned read that does not move a cursor | `pread` / `IORING_OP_READ`; `spec/kernel-spec.md:2626` | one SQE |
| K operations continuously outstanding | `WF_BRIDGE_OPERATION_CAPACITY` and `WF_BRIDGE_SLOT_COUNT` = 64, ring sized to 64 (`LOOP-PIPELINE.md:703-707`) | none; already sized |
| non-blocking harvest of one token | `wf__completion_file_take` (`bridge.h:144`, cited `LOOP-PIPELINE.md:806`) | one CQE peek |
| blocking join of one token | `wf__completion_file_join` (`bridge.h:129`) | one wait |
| **discard a harvested result** | drop the slot; nothing to undo | **zero** |
| the depth query, asked once per loop entry | `wf__completion_window(span, slot_bytes, ceiling)` (`LOOP-PIPELINE.md:690`), modelled on `wf__par_split_budget` (`par_runtime.c:943`) | one call per loop entry |
| K private destination slots allocated once at loop entry | `LOOP-PIPELINE.md:721-741` | one heap allocation |
| hand a pure per-iteration fold to another lane | `wf__par_claim` / `wf__par_publish` / `wf__par_join` (`emitter/parallel.rs:106,430,443`) | one claim |
| cancel an in-flight read | `IORING_OP_ASYNC_CANCEL` on Linux; **nothing on Darwin** — a helper blocked in `pread` cannot be interrupted (`FIRST-PRINCIPLES.md:944-948`) | an SQE, a CQE, and a race |

Read the last two rows together. **Discard is free and cancellation is not**,
and on one of the three targets cancellation does not exist. A design that
needs cancellation to be correct is a design that cannot ship on Darwin. Part A
is built so that it never needs it.

## A.1 The writer-facing example, as it would compile

This is the third blind writer's own shape, with the one change `docs/patterns.md`
P15 already asks for (the destination allocated inside the body, not hoisted).
Nothing else moves: the `loop`, the hand-carried offset, the `match` on the
read's outcome, and the `ReadEnd` break are exactly what an unguided writer
wrote in `$SCRATCH/wf-0100-verify/writer/work/sizes.wf:11-25`.

```whitefoot
fn count_file['f](file: &'f ReadFile) -> total: own u64
reads(file), allocates(heap) {
  let sum = 0_u64;
  let offset = 0_u64;
  loop @chunk {
    let chunk = buffer_new(65536_u64, 0_u8);
    region 'c {
      match read_at<'f, 'c>(file: file, destination: &uniq 'c chunk,
                            file_offset: offset, start: 0_u64, end: 65536_u64) {
        ReadBytes(next: produced) => {
          let digest = 0_u64;
          region 'd {
            set digest = fold_bytes<'d>(source: &'d chunk,
                                        produced: produced, seed: 0_u64);
          }
          set sum = sum +wrap digest;
          set offset = offset +wrap produced;
        }
        ReadEnd() => { break @chunk; }
        ReadFailed(error: problem) => { break @chunk; }
      }
    }
  }
  return sum;
}
```

Under this design the compiler prints nothing (a granted staged verdict is
silent — `docs/patterns.md:445-450`), the runtime keeps K reads of this one file
in flight, the fold of chunk i runs on a compute lane while chunk i+1 is in
the kernel, and `sum` commits in chunk order because retirement is ordered.

The `seed: 0_u64` matters and the writer should be taught it (A.12). A rolling
fold written `seed: sum` is still admitted and still pipelines its **reads** —
the offsets do not depend on the fold — but each fold then waits for the
previous one, so nothing goes to a compute lane and the win is the I/O half
only. That is a true statement about the program the writer wrote, not a
restriction the design adds.

## A.2 Why that loop is sequential today, in the rule's own words

Two sentences of [PAR-3] do it, both quoted verbatim from
`spec/kernel-spec.md`:

> `spec/kernel-spec.md:2056` — "There is one program point c of B such that
> every statement of B either executes before c on every path through B or is
> reached only through c, and c is the argument evaluation and submission of the
> first `may-suspend` action of B in program order. Write P for the statements
> up to and including c and E for the rest."

> `spec/kernel-spec.md:2057` — "Every edge that leaves B — a `return_stmt`, a
> `give_stmt` delivering outside B, a `break_stmt` naming L or a loop enclosing
> L, and a `let_stmt` selecting `propagate_let_rhs` [FN-1, GIVE-1, ERR-3] —
> occurs in P."

The `break @chunk` in the `ReadEnd()` arm is reached only through the read's
submission, so the checker puts it in E
(`compiler/src/semantic/staged_permission.rs:1047-1058` assigns the segment by a
real dominator/post-dominator query, and `:1109-1113` records the first leaving
edge it finds in E), and condition 2 denies. The compiler already tells the
writer this is unfixable rather than pretending otherwise
(`staged_permission.rs:365`):

> "take every early return, break, or propagate in the prologue, before the
> body's first I/O submission. Where the exit is selected by the may-suspend
> call's own outcome — a read-to-EOF loop's `ReadEnd` break is — it cannot be
> taken before the submission and PAR-3 cannot stage that loop as written: the
> shapes staged today are a fixed-trip bounded loop and a per-file loop over
> names, and one file's chunk loop stays sequential"

The same wall is already on the owner's desk as `docs/done/0100-writer-defaults-2.md:755-765`
(W4), which names the two things the current judgment has no form for: "a way to
carry the offset across the cut as a value the prologue may compute for
iteration i+1 before iteration i's read completes, and a way for a short read to
cancel the operations already in flight. Both are language questions."

**The runtime-first reading disagrees with the second half of that sentence.**
A short read does not need to cancel anything. It needs to *discard*, and
discard is the free row of the table in A.0.

## A.3 The mechanism: predict, prefetch, validate, discard

### A.3.1 Why condition 2 is the wrong thing to relax

Condition 2 exists for one reason, and the implementation states it
(`staged_permission.rs:56-59`):

> "With K iterations in flight, an iteration's decision to leave is otherwise
> taken after later iterations have already submitted operations that the
> source-order execution never performs, and a submitted target operation is an
> externally observable transition that is not rolled back."

That reason is a property of the *operation*, not of the loop. `open_file`
creates a descriptor and burns a permit. `write_once` publishes bytes.
`directory_next` advances a cursor. But `read_at` on a `ReadFile`:

- exhibits `reads(file, destination), writes(destination)` — `file` is in
  `reads` and **not** in `writes` (`spec/kernel-spec.md:2626`);
- "The explicit offset removes an implicit byte cursor, so the operation
  observes but does not advance the `ReadFile` state" (`spec/kernel-spec.md:2626`);
- "On `ReadEnd` and on `ReadFailed` no byte of the buffer changes"
  (`spec/kernel-spec.md:2566`).

So the complete Whitefoot-observable effect of one `read_at` is on its
`destination`. **A `read_at` into storage the writer cannot name changes no
Whitefoot state place.** It is not the "externally observable transition"
condition 2 protects against; it is the same class of thing [PAR-3] already
declines to observe at `spec/kernel-spec.md:2079` — "The number of operations
an implementation keeps outstanding … are not observable, and no rule of this
specification is stated in terms of them."

**Therefore the design does not relax condition 2 in general.** It says that a
read the implementation performs *ahead* of the loop, into replicated storage,
is not an action of B at all — so there is nothing for condition 2 to refuse.
`open_file` in a remainder stays denied, exactly as it is today; `wide8`'s
returning `Err` arm (`LOOP-PIPELINE.md:590-596`) stays denied.

### A.3.2 The four steps

Write `P_step` for the prologue of B with its submission removed: the guard, the
per-iteration construction, and the evaluation of the read's four arguments. It
is a pure function of the loop's carried state and the previous iteration's
outcome.

1. **Predict.** After submitting iteration i's read, the owner lane runs
   `P_step(state_i, OPTIMISTIC)` where `OPTIMISTIC` is the outcome
   `ReadBytes(next: end_i)` — a full read. That yields iteration i+1's four
   argument values and its private slot, and the lane submits that read. Repeat
   to depth K.
2. **Prefetch.** Up to K reads of the same file, at successive predicted
   offsets, into K private slots, are outstanding. Nothing about them is
   writer-visible.
3. **Validate.** When iteration i's read retires with its actual outcome, the
   lane runs `P_step(state_i, ACTUAL)` and compares the resulting argument tuple
   with the tuple slot i+1 was submitted with. **Equal** means the prediction
   held: slot i+1's already-harvested (or in-flight) result *is* iteration i+1's
   result and is published as such. **Unequal** means the prediction failed.
4. **Discard.** On a mismatch every slot after i is dropped — harvested and
   thrown away if complete, joined and thrown away if in flight — and the lane
   resumes prologue submission from `state_{i+1}` computed from the actual
   outcome. Nothing is cancelled. Nothing is rolled back, because nothing
   writer-visible was written.

This is exactly what a hand-written prefetching C loop does, and it is why the
falsifier in A.10 compares against one. The `OPTIMISTIC` outcome is not a guess
about the world; it is the one outcome for which the recurrence closes without
waiting, and every wrong guess is detected at the point the source order would
have detected it.

### A.3.3 Where `ReadEnd` lands

The prefetch of iteration i+1 may itself return `ReadEnd`. That result is
**retained, not published**. When retirement reaches i+1 and validation passes,
the retained `ReadEnd` becomes iteration i+1's outcome and the loop takes its
`break @chunk` — at exactly the iteration the source order takes it, with
exactly the observables the source order produced before it. When validation
fails, the retained `ReadEnd` is discarded like any other prefetched result and
the re-issued read decides.

On a regular file the common case is that a `ReadEnd` prefetch is *right*: the
last full chunk's read returns `produced == end`, the prediction holds, and the
`ReadEnd` the prefetcher already found is the loop's exit. The loop pays one
extra read for the whole file, which is what `cat` pays.

### A.3.4 Short reads: what a regular file gives, what a pipe does not

The prediction `produced == end` is only useful if a short read is rare and a
wrong prediction is cheap. Both halves are target facts and neither is a
language promise:

- **Regular file.** `pread(2)` transfers the lesser of the requested count and
  the bytes remaining, so `produced < end` means end-of-file or a concurrent
  truncation. Prediction fails at most once per file, on the last chunk. This is
  the whole reason the design is worth building.
- **FIFO, socket, terminal, character device.** A positioned read is `ESPIPE`;
  where a read succeeds at all it returns whatever is available, so `produced <
  end` says nothing, prediction fails on nearly every iteration, and every
  prefetch is wasted work. Worse, on a read-and-clear device a repeated read is
  not free — `FIRST-PRINCIPLES.md:374-378` is explicit that "MMIO, device files,
  virtual files, and read-and-clear state cannot be silently admitted under the
  positioned-read contract."
- **A file another process extends or truncates during the loop.** Prediction
  fails, the slots are discarded, and the loop resumes from the true offset.
  Correct, and slow only while the race lasts.

**This is where the design meets a real gap in the specification.**
`open_file` inspects the descriptor before publishing and refuses anything that
is not a regular file (`spec/kernel-spec.md:2685`: "a successfully inspected
directory returns `Err(IsDirectory(...))`, and every other successfully
inspected non-regular object returns `Err(Other(...))`"). `open_read` has **no
such sentence anywhere in `spec/kernel-spec.md`** — every mention of it is at
`:2274, 2293, 2294, 2313, 2498, 2519, 2605, 2609, 2611, 2625, 2684`, and none
inspects a kind. So a `ReadFile` obtained through `open_read` on a FIFO is a
`ReadFile` on which the positioned-read contract of `spec/kernel-spec.md:2626`
is not satisfiable.

That is a defect independent of this design; it becomes load-bearing here
because read-ahead admissibility rests on the resource really being a
positioned-read resource. Two ways to close it, and the owner picks (A.13
question 3): give `open_read` the same inspection `open_file` already has, or
admit read-ahead only for a `ReadFile` whose producer inspected the object.
**Until one of them lands, read-ahead is admitted only for a `ReadFile`
produced by `open_file`, and that is a compiler-side provenance fact the
existing [PRV-2] provenance column already carries** (`spec/kernel-spec.md:966`
lists it in the callable boundary).

## A.4 The spec sentences

Three rules are touched. Rule count is unchanged if the permission is added to
[PAR-3]; it becomes 138 if the owner prefers a separate [PAR-4] (A.13
question 1). Nothing else changes: no grammar production, no keyword, no
operation, no type, no outcome, no writer-visible marker.

### A.4.1 [PAR-3] — the read-ahead clause

The sentence being amended, verbatim:

> `spec/kernel-spec.md:2057` — "Every edge that leaves B — a `return_stmt`, a
> `give_stmt` delivering outside B, a `break_stmt` naming L or a loop enclosing
> L, and a `let_stmt` selecting `propagate_let_rhs` [FN-1, GIVE-1, ERR-3] —
> occurs in P."

It is **not** edited. The following paragraphs are added after
`spec/kernel-spec.md:2072`, whose host-resource sentence they extend:

> An implementation may additionally perform, for a `may-suspend` action of B
> which the read-ahead conditions below admit, further attempts of that same
> operation at argument values it computes for iterations L has not reached.
> Such an attempt is not an action of B, is not a submission at c, and its
> outcome is never delivered to a source expression unless the validation
> condition below holds for it; the edges an iteration takes are therefore
> judged by the conditions above as though no such attempt existed.
>
> Read-ahead is admitted for one action a of B exactly when all of the
> following hold. B holds no `may-suspend` action other than a. The resource
> place a names satisfies the first disposition condition above — no footprint
> of B writes it and every loan on it is shared — and a's operation contract
> names that place in `reads` and not in `writes`, states that the operation
> does not advance it, and states that an attempt whose outcome is discarded
> changes no state place. The place a writes is one this rule replicates. No
> footprint of the statements up to and including c writes a place rooted in a
> binding declared outside L.
>
> An attempt's outcome is delivered to the iteration that reaches a exactly
> when every argument value that iteration computes for a from the outcomes it
> actually observed equals the value the attempt was performed with; otherwise
> the attempt's outcome is discarded, every later attempt is discarded, and
> that iteration performs a at the values it computed. A discarded attempt
> produces no observable of this specification, and the number of attempts an
> implementation performs, discards, or has outstanding is not observable.

### A.4.2 [SYS-11] — the discard sentence

To follow `spec/kernel-spec.md:2626` ("The explicit offset removes an implicit
byte cursor, so the operation observes but does not advance the `ReadFile`
state"), one sentence, which is the entry read-ahead needs and the existing text
does not supply:

> A `read_at` whose outcome is discarded and whose `destination` no source
> expression reaches changes no state place of the program, so an
> implementation may perform one at any offset without changing what any
> execution of the program observes.

### A.4.3 [SYS-11] or [QUAL-2] — which `ReadFile` is a positioned-read resource

The sentence the design needs and the specification does not have. `open_file`
has its inspection at `spec/kernel-spec.md:2685`; `open_read` has none. One
sentence in [SYS-11], mirroring the [QUAL-2] discipline at
`spec/kernel-spec.md:2413` ("when it cannot supply one, it fails qualification
for that ID and compilation stops … rather than admitting the operation under a
weaker guarantee"):

> `open_read` performs the same descriptor-status inspection before publication
> that `open_file` performs [SYS-14], with the same outcomes, so every
> `ReadFile` a program can hold names a regular file and a positioned read of it
> is repeatable and advances no host state.

That sentence is worth landing whether or not read-ahead ships: without it a
`ReadFile` may name a FIFO and `read_at`'s own contract at
`spec/kernel-spec.md:2626` is unsatisfiable for it.

### A.4.4 META-5 delta shape

> numbered rules +0/-0 (137 remain) — or +1/-0 (138) if the read-ahead
> permission is spelled [PAR-4]; grammar productions +0/-0; unique fixed
> lowercase grammar atoms +0/-0; writer operation spellings +0/-0; opaque system
> nominal spellings +0/-0; runtime-trap families +0/-0; entry forms +0/-0;
> system operations and declaration records +0/-0 (203 remain); exception
> clauses +0/-0. [PAR-3] is amended to admit an implementation performing
> further attempts of one admitted `may-suspend` action at argument values it
> computes ahead of the loop, whose outcomes are delivered only on an exact
> argument match and are otherwise discarded without observable effect.
> [SYS-11] is amended to state that a discarded positioned read whose
> destination no source expression reaches changes no state place, and to give
> `open_read` the descriptor-status inspection `open_file` already performs. No
> accepted program becomes rejected and no permitted overlap is withdrawn: the
> permitted set only widens, so no conformance verdict moves.

## A.5 The judgment

The read-ahead judgment runs **after** [PAR-3]'s staged judgment and only for a
loop [PAR-3] would deny at condition 2 with `selected_by_submission` set or with
the only leaving edge reached through the cut. It reuses `Survey`, `Footprint`,
`Loan`, and `call_projection` unchanged
(`compiler/src/semantic/staged_permission.rs:168-177` already imports all of
them), and — like the three judgments beside it — it consults typing, declared
effect rows, resolved places, and the statement graph, and **never the
entailment fact state** (`staged_permission.rs:162-166`). Facts-on and
facts-off compilation therefore produce the same read-ahead table by
construction, and F6 of A.9 pins it.

Six conditions. A condition that cannot be resolved denies, on the same
one-sided reading `staged_permission.rs:90-95` states.

**(R1) One suspending action, and it is admitted for read-ahead.** B holds
exactly one `may-suspend` action a. Its operation's declaration record marks it
read-ahead-admitted; in this slice `read_at` is the only member, and the record
is spec data, never a name test (`spec/kernel-spec.md:2404` — "no source
function name or spelling … ever selects, adds, or removes one").

**(R2) The resource is observed, not advanced.** a's resource argument's
resolved place takes [PAR-3]'s `read-only` disposition — no footprint of B
writes it and every loan on it is shared (`spec/kernel-spec.md:2060`, first
alternative) — and the resource value's provenance [PRV-2] is an `open_file`
result, or [SYS-11] has gained the inspection sentence of A.4.3, so the object
is a regular file.

**(R3) The destination is replicated.** a's destination argument's resolved
place takes [PAR-3]'s `replicated` disposition. In this slice that means storage
B itself introduces (`staged_permission.rs:150-160`: "The replicated disposition
here admits exactly the constructions B itself introduces"). When batch 0095's
stage 2 lands, a hoisted buffer proved iteration-private
(`LOOP-PIPELINE.md:477-479`) qualifies too, and nothing here changes.

**(R4) The prologue writes nothing outside the loop.** No footprint of P writes
a place rooted in a binding declared outside L. This is what makes a speculative
run of P discardable: everything it writes lives in the slot, and dropping the
slot drops it.

**(R5) The carried state is copy-typed and closes over the outcome.** Let the
*carried state* be every binding declared outside L that P reads and some
footprint of E writes — in A.1 that is `offset` and nothing else. Each such
binding has a copy element type [OWN-1], and every statement of E that writes it
is a `set` whose right-hand side is one `never-suspends` pure operation over that
binding, the outcome's own components, and terms no statement of B writes. `set
offset = offset +wrap produced;` is that shape. The implementation forms the
predicted carried state by evaluating exactly those statements with the predicted
outcome.

Note what R5 does **not** ask. It asks for no constant stride, no induction
variable, no trip count, and no entailment fact. It does not need to *prove* the
prediction right — it needs the successor's arguments to be computable from a
predicted outcome and comparable against the actual one. This is the difference
between the design and the "constant stride so read i+1's start is known before
read i completes" framing: constant stride is one program shape that satisfies
R5, and R5 admits every shape whose offset update is a pure step function of the
outcome, including `offset = offset + produced`, `offset = offset + produced *
2`, and a bounded window that shrinks.

**(R6) The argument tuple is comparable.** Every argument of a other than the
resource is a value of copy type whose equality is bit equality, and the
resource argument's resolved place is the same place for both iterations. Three
`u64` comparisons and one place identity in A.1.

### What denies, and why that is right

- **The accumulating shape** — the writer's own `sizes.wf:74-93`, whose read is
  `read_at(file_offset: filled, start: filled, end: capacity)` into a hoisted
  `content` — denies at **R3**: the destination is enclosing storage the body
  writes, and the loop's continuation reads it. Correct: the bytes of chunk i+1
  land in the same buffer the program keeps, so a discarded prefetch would have
  overwritten data the program observes. A.12 says what the writer does instead.
- **A body that opens a file and then reads it** denies at **R1**: two
  suspending actions. That loop is already the shape [PAR-3] stages today, and
  it should keep staging under [PAR-3] rather than acquiring a second mechanism.
- **A body whose prologue writes an enclosing counter** denies at **R4**. A
  speculative prologue would double-count it.
- **A `directory_next` loop** denies at **R1/R2**: the cursor is
  `writes(source)` (`spec/kernel-spec.md:2666`) and an extra `getdents` consumes
  entries. `LOOP-PIPELINE.md:466-470` already calls this "the ownership model
  working, not a gap", and read-ahead does not reopen it.
- **A `write_once` loop** denies at **R1/R2**: a speculative write publishes
  bytes. The standing D1 (`docs/done/0100-writer-defaults-2.md:766-772`) is
  untouched by this design and stays open.

## A.6 The lowering

Everything below sits on top of the pipeline chassis batch 0095 is building
(`LOOP-PIPELINE.md:679-979`). The read-ahead loop is the *simplest* case of that
chassis, because R1 gives it one submission per iteration and therefore one
stage: no state-machine outlining is needed at all, and `stackless.rs` stays out
of the critical path exactly as `LOOP-PIPELINE.md:813-825` insists.

**Prefetch depth.** `K = wf__completion_window(span, slot_bytes, ceiling)`,
asked once at loop entry, the same query and the same call site the pipeline
already specifies (`LOOP-PIPELINE.md:690`), modelled on
`wf__par_split_budget(span, weight)` whose module doc fixes the discipline —
"asked once per loop entry, never per iteration" (`par_runtime.c:943`, cited
`LOOP-PIPELINE.md:687`). For a `loop_stmt` the span is unknown, so the compiler
passes `span = 0` and the runtime answers from its own capacity. `K = 1` is
always a legal answer and reproduces the sequential program exactly, so the
query can never make a program fail. There is no environment variable, no
attribute, and no source spelling for K.

**Slots.** K records, one heap allocation at loop entry, each holding: the
private destination buffer (the construction B introduces, allocated once and
restored per iteration by the tiered restore of `LOOP-PIPELINE.md:760-771`), the
submitted argument tuple, the completion token, the harvested outcome, and the
predicted carried state. `slot_bytes` for A.1 is 65,536 + 32 + tokens.

**The driver.** One loop on the owner lane, the same shape as
`LOOP-PIPELINE.md:262-269` with validation added:

```text
loop {
  if the oldest busy slot's token is ready (wf__completion_file_take):
      harvest it
      if it is the retiring iteration:
          recompute its successor's argument tuple from the ACTUAL outcome
          if it equals the tuple slot+1 was submitted with: keep going
          else: discard every later slot (take-or-join, then drop), reseed
          hand this slot's fold to a compute lane (wf__par_claim)
          commit this slot's accumulator writes in iteration order
          if this outcome takes a leaving edge: drain and leave
  else if a slot is free and the predicted prologue did not take a leaving edge:
      run P_step with the OPTIMISTIC outcome into that slot and submit
  else:
      wf__completion_file_join the oldest busy slot
}
```

**What happens to a prefetched read when the loop exits.** It is **joined and
dropped**, never cancelled. Three reasons, in the runtime-first order:
(1) `IORING_OP_ASYNC_CANCEL` costs an SQE and a CQE and races the completion it
is trying to stop, so the drain would need the completion path anyway;
(2) Darwin has no way to interrupt a helper blocked in `pread`
(`FIRST-PRINCIPLES.md:944-948`), so cancellation cannot be the correct path on
one of three targets; (3) R2 restricts the resource to a regular file, where an
in-flight `pread` completes in microseconds. The drain is exactly
`LOOP-PIPELINE.md:906-912`'s drain with the tails suppressed, because a
discarded slot has no tail.

**Where a false claim leaves it.** Unchanged from `LOOP-PIPELINE.md:913-920`
and `spec/kernel-spec.md:2074-2078`: one [DIAG-3] record, abort without
unwinding, in-flight operations abandoned by process teardown, and the correct
path pays nothing to make the defective one reproducible.

## A.7 The ledger line a writer sees

The existing format is unchanged (`compiler/src/semantic/permission_ledger.rs:238`
for the `PAR stage` line and `:251` for `PAR place`). Two disposition spellings
are added to `Disposition::spelling` (`staged_permission.rs:250-258`), and one
new line kind is added because the owner requires that anything the compiler
allocates is stated:

```text
PAR stage       sizes.wf:11  loop  permitted  read-ahead at read_at<'f, 'c>(file: file,
                destination: &uniq 'c chunk, file_offset: offset, start: 0_u64,
                end: 65536_u64); 4 places classified
PAR place       sizes.wf:11  observed      file            the resource is read and never
                advanced, so a read the source order never performs publishes nothing
PAR place       sizes.wf:11  replicated    &uniq 'c chunk  storage the body introduces; one
                private slot per read in flight
PAR place       sizes.wf:11  predicted     offset          recomputed from the outcome; a read
                that returns short discards every later slot and resumes from the true value
PAR place       sizes.wf:11  serialized-E  sum             written in the remainder and
                committed in iteration order
PAR ring        sizes.wf:11  loop  65568 bytes per slot, allocated once at loop entry; the
                number of slots is chosen by the runtime at entry and has no source spelling
```

The `PAR ring` line is the one new thing and it exists for exactly one reason:
the owner's ruling that the compiler does no hidden tricks and that anything it
allocates is stated. It prints the per-slot size, the fact that the allocation
happens once, and the fact that the count is not the writer's. It prints on a
**granted** loop, which is a deliberate exception to the rule that a granted
loop says nothing on the default channel (`docs/patterns.md:445-450`) — the
ring line goes to `--par-ledger` only, so an ordinary build is still silent.

A denied read-ahead loop keeps saying what it says today, and the condition-2
sentence at `staged_permission.rs:365-366` — the one that currently ends "one
file's chunk loop stays sequential" — is rewritten in the same change, because
leaving it in place after the loop stages would be the exact failure
`docs/done/0100-writer-defaults-2.md:137-167` (W4) was raised to fix: a remedy
the writer cannot take, or worse, a statement that is no longer true.

## A.8 The safety argument

Six claims, each with the rule that carries it.

1. **No memory corruption.** Every prefetch writes into one slot's private
   buffer, whose length the compiler fixed and whose range the read carries.
   [SYS-8]'s two static obligations (`spec/kernel-spec.md:2540`) are discharged
   for the source call and the prefetch reuses that same admitted extent; "The
   target is never asked to validate a source pointer or source range"
   (`spec/kernel-spec.md:2545`) still holds because the range is the same one.
2. **No data race.** One slot is written by one target operation and read by at
   most one lane, after the drain-before-resume the runtime already guarantees
   ("acquire drain before result or returned loans become visible",
   `FIRST-PRINCIPLES.md:970`). The resource is held under a shared loan by R2,
   and "two overlapping shared loans deny nothing" (`spec/kernel-spec.md:1999`).
3. **No uninitialized read.** A slot's buffer is fully initialized by
   `buffer_new` once at loop entry, and the tiered restore
   (`LOOP-PIPELINE.md:751-753`: "A released ring slot holds the value its
   `buffer_new` constructed, over its whole capacity") re-establishes that
   before reuse. A discarded prefetch leaves bytes in a slot no source
   expression reaches.
4. **The published bytes equal the source order's.** Validation is exact
   argument equality (R6), so a delivered outcome is the outcome of a read
   whose four arguments are the ones the source-order execution computes. A
   mismatch delivers nothing. `sum` commits in iteration order under [PAR-3]'s
   own clause (`spec/kernel-spec.md:2064`).
5. **A claim is never removed and never cheapened.** Nothing here reads a trap
   latch, quiesces a slot before a claim, or orders anything to make a defective
   execution reproducible — `spec/kernel-spec.md:2078` forbids exactly that and
   `LOOP-PIPELINE.md:913-920` records that batch 0078 already deleted one
   instance of the shape.
6. **The one thing the design does *not* promise.** A validated prefetch reads
   its bytes **earlier in wall-clock time** than the source order would. For a
   file no other process mutates during the loop — which is the assumption any
   chunk loop already makes — the bytes are identical. For a file another
   process is writing, the program may publish bytes from an earlier moment
   than the source order would have seen. [SYS-11] already declines to relate
   the two: "Environment-created changes to the same physical file do not merge
   or mutate Whitefoot places [EFF-5]" (`spec/kernel-spec.md:2627`), and
   [PAR-3] already admits "an outcome that operation could deliver in the
   source-order execution at that point" (`spec/kernel-spec.md:2072`). Whether
   *at that point* covers an earlier read is the one genuine narrowing question
   in Part A, and it is A.13 question 2 rather than a sentence I write for the
   owner.

## A.9 The differential oracle

The oracle is a set of paired runs, not a single golden output. A byte-identity
test that passes because read-ahead never fired proves nothing — the
green-is-not-coverage rule applies here exactly as `LOOP-PIPELINE.md:386-388`
applies it to privatization — so every identity test below is paired with a
counter assertion that proves the mechanism ran.

**F1 — the ledger, free and first.** `whitefootc --par-ledger` on the A.1
program must print a granted read-ahead verdict naming `file` observed,
`chunk` replicated, `offset` predicted, `sum` serialized-E, and one `PAR ring`
line. It costs nothing and it separates a judgment failure from everything
downstream.

**F2 — the counters.** Two new counters beside
`wf__completion_file_submissions()` (`bridge.h:161`): `..._readahead_attempts()`
and `..._readahead_discarded()`. For a file of exactly n full chunks read at
depth K, published reads must be exactly n+1 (n full plus the `ReadEnd`), and
attempts minus discards must equal n+1. Assert both numbers, not their
difference. `LOOP-PIPELINE.md:1807-1814` records why this matters: the
falsifier F2 of the pipeline design was **wrong as written** because the
operation counts were assumed rather than measured.

**F3 — the size sweep, three ways.** The same program over files of size 0, 1,
chunk-1, chunk, chunk+1, 2*chunk, K*chunk, K*chunk+1, and one larger than the
runtime's own capacity, each published byte-identical under (a) the default
build, (b) `--no-overlap`, and (c) `WF_IO_RING=0 WF_IO_HELPERS=0`. Note that
`--no-overlap` is **not** a sequential reference on Linux —
`LOOP-PIPELINE.md:1824-1833` measured that its binaries still report
`enters=8192` — so (c) is the honest control and (b) is a lowering check.

**F4 — the mismatch path, fault-injected.** A regular filesystem produces a
short non-EOF read essentially never, so the discard-and-resume path is
untested without help. Add `WF_IO_SHORT_READ_EVERY=n` to the adapter, in the
same class as `WF_IO_HELPERS` (`bridge.c:90`) — a target-policy knob, never a
language surface, exactly as `LOOP-PIPELINE.md:975-979` argues for the ring-off
knob. With it, every size in F3 must still publish identical bytes for n in
{1, 2, 3, K, K+1}, and `..._readahead_discarded()` must be nonzero.

**F5 — the concurrent-truncation case.** Truncate the file mid-loop from a
second process. This is an admissibility test, not a byte-identity test: the
program must not trap, must not read past the new end, and must terminate. It
is the test that would catch a validation that compares the wrong thing.

**F6 — facts-off identity.** Acceptance, the read-ahead table, and the
published bytes must be unchanged with the entailment state degraded. Pinned by
a test, not by a comment (`LOOP-PIPELINE.md:1355-1357`). The judgment reads no
fact by construction (A.5), so this test is cheap and its failure would mean the
judgment leaked.

**F7 — the leak test.** `many_files_wide8.wf` and `many_files_narrow.wf` must be
byte-unchanged in emitted IR, because neither is a read-ahead loop. A regression
there means the new judgment widened [PAR-3]'s.

## A.10 The falsifier

**The bar: a read-heavy single-file program on the Linux CI runner must approach
the hand-written prefetching C loop.**

The hand-written loop has to be written first, and it is the same discipline as
Probe B of the pipeline design (`LOOP-PIPELINE.md:1368-1371`): one thread
keeping K positioned reads of one file continuously in flight through io_uring,
the fold on a second thread, same chunk size, same file, same tree, built and
run interleaved in one plan with the Whitefoot binaries.

```text
  REQUIRED  Linux  C.read_heavy.after  <=  1.15 x  the hand-written prefetch loop
  REQUIRED  Linux  C.read_heavy.after  <=  0.65 x  C.read_heavy.before
  control   Linux  WF_IO_RING=0 WF_IO_HELPERS=0 on the same binary is unmoved
  control   both   published checksum bytes identical on every recorded run
  control   macOS  C.read_heavy.after  <=  C.read_heavy.before, and no worse
```

Two deliberate choices in those lines.

First, **the bar is stated against a program the mechanism has to earn**, not
against an existing Whitefoot line. `LOOP-PIPELINE.md:1824-1833` records what
happens otherwise: the pipeline design's REQUIRED Linux bar of 119.47 ms turned
out to be reachable on the container by setting an environment variable, "so it
no longer discriminates between a working pipeline and no pipeline".

Second, **macOS is a no-regression bar, not a win.** The measured record on the
read-heavy workload is `C.narrow` 3058.12 ms against `C.wide8` 1463.43 ms
(`docs/done/0098-blind-writer.md:88-89`), and the pipeline design's own macOS
finding is that the host serializes most concurrency and saturates at the
existing helper cap (`LOOP-PIPELINE.md:1574-1579`). Promising a macOS win here
would be dishonest.

**What falsifies the design.** If `C.read_heavy.after` lands outside 1.15x of
the hand-written loop while F2 shows the attempts and discards it should, the
mechanism is right and the *lowering* is losing the time — reopen the driver.
If F2 shows few attempts, the *judgment* denied and the ledger says why. If the
program is fast and F5 fails, the design is wrong and must not ship.

## A.11 What it costs

| component | file | lines |
|---|---|---|
| read-ahead judgment R1-R6, reusing `Survey`/`Footprint`/`Loan` | new `compiler/src/semantic/read_ahead.rs` | ~350 |
| judgment tests, including every denial of A.5 | `compiler/src/semantic/tests/` | ~250 |
| two disposition spellings, the `PAR ring` line, the rewritten condition-2 sentence | `semantic/permission_ledger.rs`, `semantic/staged_permission.rs` | ~90 |
| predicted-prologue re-materialization and the argument-tuple record | `lowering/builder/pipeline.rs` (the pipeline's own file) | ~200 |
| validate, discard, reseed in the driver; drain with tails suppressed | `backend/emitter/completion.rs` | ~150 |
| two counters and the short-read fault knob | `completion/bridge.c`, `bridge.h` | ~60 |
| backend tests (slot shape, discard on exit, `--no-overlap` parity) | `compiler/src/backend/tests/` | ~250 |
| conformance cases and verdicts for the amended rules | `conformance/` | ~150 |
| the hand-written C prefetch ceiling and the bench wiring | `research/experiments/io-completion-bench/` | ~200 |
| spec, `docs/patterns.md` P15 rewrite, `docs/done/` | `spec/`, `docs/` | ~200 |
| **total** | | **~1,900** |

**New APIs: none.** No new operation, no new type, no new outcome, no new
writer spelling. **New runtime mechanism: none** — two counters and one
test-only fault knob are instrumentation, not mechanism. No cancellation path,
no second queue, no new submit route, no change to the completion protocol
core, whose 34.9-35.6 ns/op round trip `LOOP-PIPELINE.md:972-974` says must not
be touched.

The estimate assumes the pipeline chassis of batch 0095 exists. On its own the
read-ahead loop would need the chassis too (K slots, the ring, back-edge
tolerant joins), which `LOOP-PIPELINE.md:1398-1417` prices at ~3,400 lines for
stage 1. **Part A is a ~1,900-line rider on work already scheduled, not a
parallel mechanism**, and that is the strongest argument for sequencing it
immediately after batch 0095 rather than designing it separately.

## A.12 What the writer must write differently, and why that is not a hidden trick

**One thing, and it is already the documented pattern.** The destination buffer
must be constructed **inside** the loop body rather than hoisted above it. That
is `docs/patterns.md:346` — "construct the per-iteration scratch **inside** the
loop body" — landed for the per-file loop and unchanged here. Everything else
in the natural loop stays: the `loop`, the hand-carried offset, the `match`, the
`ReadEnd` break, the ordinary `set sum = sum +wrap digest`.

The writer's own program shows the difference exactly.
`$SCRATCH/wf-0100-verify/writer/work/sizes.wf:11-25`'s
`count_file` reads into a `scratch` passed in from `main:56` and folds a byte
count; it needs the scratch moved inside the loop and nothing else.
`sizes.wf:74-93`'s `@slurp` loop is the other shape — it *accumulates* into one
buffer with `start: filled` — and it stays sequential, correctly, because the
program keeps every byte it read.

**Three reasons this is not a hidden trick**, taking the owner's rulings in
order.

1. **Nothing is transformed silently.** The compiler never rewrites the
   accumulating shape into the discarding one. It refuses to read ahead, and
   `--par-ledger` names `chunk`/`content` and the condition. The teaching lives
   in `docs/patterns.md` and in the diagnostic, which is the ruling's own
   preference — warn and teach rather than transform.
2. **Everything the compiler allocates is stated.** One allocation, at loop
   entry, printed on the `PAR ring` line with its per-slot size and an explicit
   statement that the count is the runtime's. There is no per-iteration
   allocation hidden behind a restore that the writer cannot see, and there is
   no depth the writer could have set.
3. **The speculative work is countable.** `..._readahead_attempts()` and
   `..._readahead_discarded()` make the read-ahead visible to a bench or a
   writer who wants to know what their loop cost. A mechanism whose waste
   cannot be counted is a mechanism nobody can argue with.

And one honest cost the writer pays for the *fold*, not for the reads: a rolling
fold written `seed: sum` still pipelines its reads but keeps its folds on the
owner lane (A.1). That is a property of the program, and the right response is a
sentence in `docs/patterns.md`, not a compiler transformation.

## A.13 Open questions for the owner — Part A

1. **[PAR-3] amended, or a new [PAR-4]?** A.4.1 amends [PAR-3] and holds the
   rule count at 137. The honest cost is that [PAR-3] would then carry a
   permission about a cut and dispositions *and* a permission about speculative
   attempts, which share the disposition vocabulary but not the schedule. This
   is the same choice `LOOP-PIPELINE.md:1589-1596` put to the owner for the
   staged permission itself. **Which?**
2. **Does a validated prefetch satisfy "at that point"?** [PAR-3]'s
   `spec/kernel-spec.md:2072` admits "an outcome that operation could deliver in
   the source-order execution at that point". A prefetch reads earlier in wall
   clock. For a file no other process mutates the outcome is identical; for a
   file under concurrent external write it may not be. [SYS-11]'s
   `spec/kernel-spec.md:2627` already declines to relate two reads of one
   physical file. **Is that enough, or does [PAR-3] need a sentence saying so
   explicitly?** This is the only place in Part A that touches a promise.
3. **`open_read` performs no descriptor-status inspection.** `open_file` does
   (`spec/kernel-spec.md:2685`); `open_read` does not, in any of its eleven
   mentions. So a `ReadFile` may name a FIFO on which `read_at`'s own contract
   at `:2626` is unsatisfiable. **Give `open_read` the same inspection
   (recommended, and worth landing regardless), or restrict read-ahead to
   `open_file`-produced resources by provenance?**
4. **The short-read fault knob.** F4 is the only deterministic route to the
   discard path. **Acceptable as a target-policy knob in the `WF_IO_HELPERS`
   class?** Without it that path ships untested.
5. **Sequencing.** Part A is a rider on batch 0095's chassis (A.11). **Should it
   be scheduled as the batch immediately after the pipeline lands, or held until
   the pipeline's own falsifier F3 reports?**

---

# Part B — bytes to path

## B.0 What the runtime can already do, at what price

| capability | where | marginal cost |
|---|---|---|
| open a **multi-component** path relative to a directory descriptor | `openat(dirfd, path, flags)` at `compiler/src/backend/completion/file_adapter.c:172` and `:182` | **zero** — the call already takes a whole path and never inspects it for separators |
| stage the caller's bytes into compiler-owned storage before submission | `compiler/src/backend/emitter/system.rs:2059` memcpys the admitted `[start, end)` range and NUL-terminates it (`LOOP-PIPELINE.md:374-377`) | one memcpy, already paid |
| release the name borrow before target transfer | `spec/kernel-spec.md:2294` already publishes `loan-released(name)` at `begin_submit` | none |
| refuse a symlink at the final component | `O_NOFOLLOW` | none |
| refuse a symlink at **any** component, Darwin | `O_NOFOLLOW_ANY = 0x20000000`, `MacOSX.sdk/usr/include/sys/fcntl.h:158` on this host | one flag bit |
| confine resolution beneath the directory, Darwin | `O_RESOLVE_BENEATH = 0x00001000`, same header `:128` — **but its comment reads "only for open(2)"**, so it needs a runtime probe before it is relied on with `openat` | one flag bit, if it probes |
| confine resolution beneath the directory, Linux ≥ 5.6 | `openat2(dirfd, path, {resolve: RESOLVE_BENEATH\|RESOLVE_NO_SYMLINKS\|RESOLVE_NO_MAGICLINKS}, size)`, race-free, kernel-enforced; no glibc wrapper, so `syscall(SYS_openat2, ...)` | one syscall, same as `openat` |
| confine resolution with neither of those | a per-component `openat(..., O_NOFOLLOW \| O_DIRECTORY)` chain | n+1 syscalls **and it is not race-free** — a component can be replaced between two calls |

The first row is the whole of Part B's easy half: **the one-component
restriction is a source-side validation, not a host limit.** The runtime code
that opens `a.txt` opens `sample/a.txt` with no change at all.

The bottom four rows are Part B's hard half: confinement is a real target
capability that two of the three targets supply *conditionally* and the
fallback does not supply race-freedom at all. That asymmetry is why B.3 splits
the design in two and ships only the first half.

## B.1 The writer-facing example, as it would compile

This is the strongest statement Part B can make, and it is literally true:
**the third blind writer's program needs no edit at all.** The lines below are
`$SCRATCH/wf-0100-verify/writer/work/sizes.wf:114-122`,
byte for byte, reading one line of a list file out of `content` at
`[begin, index)`:

```whitefoot
                            region 'g2 {
                              let permit2 = reserve_file<'g2>(factory: &uniq 'g2 files);
                              region 'n2 {
                                match open_file<'g2, 'n2>(permit: move permit2, root: &'g2 cwd,
                                                          name: &'n2 content, start: begin, end: index) {
                                  Ok(value: target) => {
                                    region 'q {
                                      let size = count_file<'q, 'q>(file: &'q target, scratch: &uniq 'q scratch);
```

Today that returns `Err(InvalidPath(code: 0_u32, origin: 0_u8))` for every line
of the writer's own `list.txt`, whose entries are `sample/a.txt`,
`sample/b.txt`, `sample/empty.txt`, `sample/big.bin`, `sample/missing.txt`. The
writer's workaround was to build a second list file, `flat.txt`, holding
`a.txt`, `b.txt`, `empty.txt`, `big.bin`, `missing.txt`, and to run the tool
from inside the sample directory. Under this design `list.txt` works and
`flat.txt` is deleted.

For a directory tree the same change collapses `largest.wf`'s hand-written
descent. That program holds `buffer_vacant<DirectoryRead>(512_u64)`
(`largest.wf:55`), pushes `cwd` into slot 0 with `replace stack[0_u64] =
Some<DirectoryRead>(...)` (`:78`), takes a level back out with `replace
stack[sp] = None<DirectoryRead>()` (`:91`), builds a path by hand writing a
separator with `set full[plen] = 47_u8;` (`:173`), and descends one component
at a time through `open_directory(..., name: &'n2 batch, start: p3, end: nend)`
(`:198`) pushing each child back onto the affine stack (`:200`). Every one of
those lines exists to work around the one-component rule. None of them is
needed once a name range may hold a path.

## B.2 Why it is refused today, in the rules' own words

Three sentences, verbatim.

> `spec/kernel-spec.md:2676` — "A name is one path component: it is never
> empty, never longer than the target's component limit, and contains no NUL
> and no target separator, so no record a program reads can name more than one
> component."

> `spec/kernel-spec.md:2683` — "Each then validates `[start, end)` as one
> component before any host call: a component that is empty, longer than the
> target's component limit, or containing a NUL or a target separator yields
> `Err(InvalidPath(code: 0_u32, origin: 0_u8))`, no host call, and no resource
> value."

> `spec/kernel-spec.md:2680-2681` — "This specification declares no operation
> turning an enumerated name into a `HostString` or a `RelativePath`, because a
> name's backing is not the command-lifetime argument snapshot [HOST-3] and a
> path value is an inline lease over that snapshot [PATH-1]. `open_directory`
> and `open_file` therefore take a caller-owned name range rather than a path
> value, and path composition remains the DEFERRED addition [PATH-1] states."

The value-side route is closed by the same lease argument:

> `spec/kernel-spec.md:2385` — "A relative path is an opaque value whose code
> units are admitted by construction from one host string and are never
> assembled, split, or concatenated as source text."

> `spec/kernel-spec.md:2381` — "A producer whose backing is not
> command-lifetime yields no value of this type: it introduces a distinct
> owned-backing string resource with its own release action and its own type
> contract, because storage class is a function of type [STOR-1] and one type
> carries exactly one release action."

And the cost is already recorded as an owner open point,
`docs/done/0100-writer-defaults-2.md:787-801` (B6): "a utility that takes its
work list from a file has no route from the bytes it read to a file it can
open, and the writer's workaround reached for `buffer_vacant` and `replace`
over a `DirectoryRead` held per path component. The question for the owner is
whether the lease model should gain a second backing class … or whether a work
list is meant to arrive through `Args`."

**Both of B6's alternatives are wrong**, and that is the finding of Part B. A
work list is obviously not meant to arrive through `Args`; and the lease model
does not need a second backing class, because the thing the program needs is not
a stored path value.

## B.3 The design: one path semantics, no new type, no new operation

### B.3.1 The choice, and why the smallest one wins

Three shapes were considered.

| shape | what it adds | verdict |
|---|---|---|
| **B-plain.** Generalize `open_file`'s and `open_directory`'s admitted range from one component to one relative path. | one amended validation sentence | **ship this** |
| **B-owned.** A new owned `PathBuffer` type built from a buffer range, with `allocates(heap)`, its own release action, and its own [SYS-5] completion policy. | one type, one storage contract, one release action, one constructor, one consumer per open | refused — see below |
| **B-confined.** A confined directory type whose opens cannot resolve outside it, per [PATH-2]'s deferred addition. | one type, three target facilities, one qualification record, one target that fails | **defer, with a named entry condition** (B.7) |

**B-owned is refused** on the repository's own residue-hunt axis: it is a value
whose only use is to be an argument. Nothing in the writer's program stores a
path, returns one, puts one in a struct, or compares two. The bytes are already
in a buffer the program owns; wrapping them in a second affine value with a
second release action buys the writer a `move` ceremony and buys the language a
storage class. [HOST-3]:2381 says a second backing class *would* be the shape if
a value were needed — and no program needs one.

**B-plain is what the runtime already does.** `file_adapter.c:172` passes the
staged bytes to `openat` unexamined. The entire difference between `a.txt` and
`sample/a.txt` today is `spec/kernel-spec.md:2683`'s separator check.

### B.3.2 The validation, stated once

Amend `spec/kernel-spec.md:2683`. The admitted range is one **relative path**,
and — this is the load-bearing decision — it is validated by **exactly
[PATH-1]'s test and no other**, so the byte route and the argument route have
one path semantics:

- non-empty;
- contains no NUL code unit — [PATH-1]:2387's first clause;
- begins with no target-root prefix, where "The exact target-root prefix set is
  target data fixed by that target's qualification record [QUAL-1]; a
  Unix-family leading separator and a Windows-family drive or UNC prefix are
  members of their targets' sets" (`spec/kernel-spec.md:2388`) — [PATH-1]'s
  second clause, reused verbatim rather than restated;
- splits at the target separator into components each non-empty and no longer
  than the target's component limit, which `spec/kernel-spec.md:2677` already
  fixes at 1023 bytes for Darwin-family and 255 for Linux-family;
- is no longer than the target's **path limit**, which is a new target datum and
  must be named in [QUAL-2]'s guarantee list, because otherwise the host returns
  `ENAMETOOLONG` and the language would be checking one limit and not the other.

### B.3.3 What is deliberately **not** validated

`.` and `..` components, a repeated separator, and a trailing separator are
**admitted**. Refusing them here would make the byte route stricter than the
argument route for no stated reason, and would create a second path semantics —
the exact defect [META-2] and [META-4] exist to prevent. [PATH-1] is explicit
that construction "preserves every admitted code unit exactly — including `.`
and `..` components and every separator — and performs no normalization,
canonicalization, case folding, prefix stripping, or component collapse"
(`spec/kernel-spec.md:2390`).

The consequence must be said out loud, in the spec and in
`docs/patterns.md`: **the validation is not a confinement check.** A program
whose list file contains `../../etc/passwd` opens `/etc/passwd`, exactly as a
program whose *argument* is `../../etc/passwd` does today through `arg_get` +
`relative_path` + `open_read`. That is not a new hole; it is the promise
[PATH-2] already makes:

> `spec/kernel-spec.md:2396-2397` — "The value bound to the command's
> working-directory entry input is process-equivalent: resolution follows `.`
> and `..` components, symbolic links, reparse points, and mount transitions
> exactly as the surrounding process namespace does, so a resolved object may
> lie outside the directory that value names. That is the complete promise this
> type makes, and it is not a confinement claim."

Authority in this language comes from the `DirectoryRead` a program holds and
the `FilePermit` it consumed, not from where the bytes came from. Bytes a
program read out of a file it opened are not less trustworthy than bytes the
kernel handed it in `argv` — both are external input. Making the byte route
stricter would be a category error dressed as safety.

## B.4 The spec sentences

Two rules amended, one guarantee added, no rule added, no type added, no
operation added.

### B.4.1 [SYS-14] — the validated range

`spec/kernel-spec.md:2676` is **kept unchanged**: a `directory_next` record's
name really is one component, and that sentence is about the enumeration record,
not about the open. `spec/kernel-spec.md:2683` is replaced by:

> Each then validates `[start, end)` as one relative path before any host call,
> by exactly the test [PATH-1] fixes for a relative path constructed from a host
> string: a range that is empty, contains a NUL code unit, begins with a
> target-root prefix, is longer than the target's path limit, or splits at the
> target separator into a component that is empty or longer than the target's
> component limit yields `Err(InvalidPath(code: 0_u32, origin: 0_u8))`, no host
> call, and no resource value. `.` and `..` components, repeated separators, and
> a trailing separator are admitted unchanged and are resolved by the target's
> own directory-relative facility [PATH-2]; this validation is not a confinement
> check and no rule of this specification makes one of it.

`spec/kernel-spec.md:2680-2681`'s exclusion sentence is replaced by:

> This specification declares no operation turning an enumerated name or bytes a
> program read into a `HostString` or a `RelativePath`, because their backing is
> not the command-lifetime argument snapshot [HOST-3] and a path value is an
> inline lease over that snapshot [PATH-1]. `open_directory` and `open_file`
> instead take a caller-owned range holding one relative path, so a program
> composes a path in a buffer it owns and hands the range to the open; a stored
> path value with its own backing class remains the DEFERRED addition [PATH-1]
> states.

`spec/kernel-spec.md:2687`'s symlink sentence is amended, because with several
components it is no longer complete:

> On success `open_directory` returns an independent `DirectoryRead` for the
> named directory and `open_file` returns an independent `ReadFile` for the
> named regular file; a symbolic link **at the final component** is not followed
> by either operation, and a symbolic link at any earlier component is followed
> exactly as the target's own directory-relative resolution follows it [PATH-2].

### B.4.2 [QUAL-2] — the path limit

To follow `spec/kernel-spec.md:2420`'s fourth guarantee, a fifth:

> The fifth is a stated path limit for the directory-relative semantic IDs: a
> qualified target names the greatest length, in code units, of a relative path
> its directory-relative facility accepts, and the compiler's admitted
> validation refuses a longer range before any host call so that the length is
> never a host outcome.

### B.4.3 META-5 delta shape

> numbered rules +0/-0 (137 remain); grammar productions +0/-0; unique fixed
> lowercase grammar atoms +0/-0; writer operation spellings +0/-0; opaque system
> nominal spellings +0/-0 (ten remain); runtime-trap families +0/-0; entry forms
> +0/-0; system operations and declaration records +0/-0 (203 remain); exception
> clauses +0/-0. [SYS-14] is amended so `open_file`'s and `open_directory`'s
> admitted `name` range holds one relative path validated by exactly [PATH-1]'s
> test rather than one path component, so those two operations reach a
> multi-component path a program composed in its own buffer; its symlink
> sentence is amended to distinguish the final component from earlier ones; and
> its exclusion sentence is amended to name the range route it now provides.
> [QUAL-2] gains a fifth stated target guarantee, a path limit, so a range
> longer than the target accepts is refused before any host call. **A program
> accepted before is accepted now with the same outcome**: a single component is
> a one-component relative path and takes the same validation branch, so no
> conformance verdict moves and no published byte changes. No confinement
> promise is added, and [PATH-2]'s deferred confined directory type is
> unaffected.

## B.5 The judgment

There is no new static judgment. This is worth stating plainly because it is the
main structural argument for B-plain: the range's *content* is runtime data, and
the two obligations the call already carries are the only static ones.

Every member of the range-bearing family already carries, per
`spec/kernel-spec.md:2540`, "exactly two independent [ENT-6] obligations in this
order: `start <= end`, then `end <= len(deref(buffer))`". Those are unchanged.
The path validation is a runtime check that returns a typed `Err(InvalidPath)`,
which is the correct classification under [ERR-4]: an "expected environment and
input failure" is a value (`spec/kernel-spec.md:1463`). It is not a claim, not a
trap, and not a source rejection.

What *does* change in the compiler is one static fact used for storage sizing:
the staged path buffer per call site (`emitter/system.rs:1952`'s `%component`)
is sized by the target's component limit today and must be sized by the path
limit instead. `LOOP-PIPELINE.md:889-900` already requires that buffer to become
slot-indexed for the pipeline; this change rides that one and does not create a
second.

## B.6 Resolution: one host call, and what it promises about symlinks and races

**One host call, not a per-component chain.** `openat(dirfd, "sample/a.txt",
O_RDONLY | O_NOFOLLOW)` is what `file_adapter.c:172` already emits, and it is
one syscall regardless of component count. A per-component `O_PATH` chain would
cost n+1 syscalls, would need n+1 completion records in the pipeline, and would
buy nothing the target's own resolution does not already do — and
`spec/kernel-spec.md:2398` forbids the alternative direction outright: "a target
implements directory-relative resolution with its own directory-relative
facility, never by concatenating a prefix onto a path and resolving the result
against an ambient working directory, and a target with no directory-relative
facility fails qualification for the directory-relative semantic IDs [QUAL-1]
rather than emulating them."

**The symlink promise, precisely.** `O_NOFOLLOW` refuses only a symlink at the
final component, so `spec/kernel-spec.md:2687`'s current sentence — "a symbolic
link is not followed by either operation" — is exactly true for one component
and would be false for several. B.4.1 amends it rather than letting the
implementation quietly weaken it, which is the [QUAL-2]:2413 discipline applied
to a sentence rather than to a target.

**The race promise: none, and that is stated.** Between the validation and the
`openat`, and between one `openat` and the next, the filesystem may change.
`open_file` already answers this the only way a single call can: it opens, then
inspects the descriptor it got (`spec/kernel-spec.md:2685`), so the object it
publishes is the object it opened — not an object it looked up twice. That is a
genuine TOCTOU-freedom property of the *published resource*, and it survives
multi-component paths unchanged, because it is a property of the descriptor and
not of the path. What no single `openat` can promise is that the *path* still
names that object afterwards, and the language does not promise it: the program
holds a `ReadFile`, not a name.

So the honest statement, and the one that belongs in `docs/patterns.md`: **the
language promises the object you opened is the object you inspected. It does
not promise the path is confined, and it does not promise the path still resolves
there.** Confinement is a property of the directory value's type
(`spec/kernel-spec.md:2399`: "a value's confinement promise is fixed by its type
and never changes at runtime"), which is B.7.

