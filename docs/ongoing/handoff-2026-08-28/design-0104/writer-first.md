# Writer-first: the streaming chunk loop, and bytes to path

Design record, 2026-08-28. Read-only against `integration/2026-08-28b` at
`16228216`; every line number below is that revision's. This is a design, not
an implementation plan already agreed: §A.13 and §B.13 are the decisions it
does not take.

**Method.** Both parts are measured by one question: does a program a blind
writer actually wrote become fast, or become writable, unchanged or after one
change `docs/patterns.md` already teaches? The programs are the five of
`docs/done/0098-blind-writer.md`, the third writer's two at
`$SCRATCH/wf-0100-verify/writer/work/` (`sizes.wf`,
`largest.wf`), and the four chunk loops
`docs/done/0100-writer-defaults-2.md:762` names — `treelines.wf:99`,
`checksum.wf:107`, `tests/programs/dir_walk.wf:262`,
`tests/programs/wfgrep.wf:598`. The scoreboard is §C.

---

# Part A — the streaming chunk loop

## A.0 The program this is measured by

`$SCRATCH/wf-0100-verify/writer/work/sizes.wf:9-27`, written
by a writer given only the spec and `docs/patterns.md`. It is `wc -c` over one
file, and `largest.wf:9-27` is the same function byte for byte:

```whitefoot
fn count_file['f, 'd](file: &'f ReadFile, scratch: &uniq 'd buffer<u8>) -> total: own u64 reads(file, scratch), writes(scratch) {
  let sum = 0_u64;
  loop @chunk {
    region 'c {
      match read_chunk<'f, 'c>(file: file, scratch: &uniq 'c deref(scratch), offset: sum) {
        ReadBytes(next: taken) => {
          set sum = sum +wrap taken;
        }
        ReadEnd() => {
          break @chunk;
        }
        ReadFailed(error: problem) => {
          break @chunk;
        }
      }
    }
  }
  return sum;
}
```

This is the owner's "读取-处理-丢弃" loop. It reads one window, folds it,
discards it, and advances. It is the shape of `wc`, `cksum`, `md5sum`, `grep`
over one file, and of every program that must stream a large file.

## A.1 Why it is denied today — three walls, not one

Each wall is a separate condition of [PAR-3], and a design that removes only
one buys nothing. Naming all three is the whole point of starting from the
writer's own text.

**Wall 1 — the exit is in the remainder.** `spec/kernel-spec.md:2057`:

> Every edge that leaves B — a `return_stmt`, a `give_stmt` delivering outside
> B, a `break_stmt` naming L or a loop enclosing L, and a `let_stmt` selecting
> `propagate_let_rhs` [FN-1, GIVE-1, ERR-3] — occurs in P.

Both `break @chunk` statements sit in the `ReadEnd` and `ReadFailed` arms of
the cut call's own outcome, so they are in E. The compiler already knows the
remedy it prints cannot be taken —
`compiler/src/semantic/staged_permission.rs:365`:

> "…Where the exit is selected by the may-suspend call's own outcome — a
> read-to-EOF loop's `ReadEnd` break is — it cannot be taken before the
> submission and PAR-3 cannot stage that loop as written…"

**Wall 2 — the cursor is on both sides of the cut.** `sum` is read in P (it is
the `offset` actual, evaluated at c) and written in E (`set sum = sum +wrap
taken`). `spec/kernel-spec.md:2060` gives a place rooted outside L exactly
three conditions, and `sum` meets none, so the loop is denied with
`staged_permission.rs:1618`'s reason: *"the body reaches it on both sides of
the cut, so no single segment serializes it"*.

**Wall 3 — the destination is enclosing storage.** `scratch` is a `&uniq`
parameter of `count_file`, so it is rooted outside L; the read writes it and
retains a borrow of it past its own submission, which is condition 3
(`spec/kernel-spec.md:2059`). This wall is the one `docs/patterns.md` P15
already teaches away — "construct the per-iteration scratch **inside** the
loop body" (`docs/patterns.md:346`).

Walls 1 and 2 are language questions. Wall 3 is a taught form.

## A.2 The writer-facing example, as it would compile

One change from A.0: the chunk buffer moves inside the loop, which is P15
verbatim. That change also deletes the `read_chunk` helper, because a
`buffer_new` inside the body is an own binding and `&uniq 'c data` is an
ordinary borrow rather than the reborrow [OWN-6] forced the helper for
(`docs/done/0098-blind-writer.md:296`). The loop is shorter than what the
writer wrote.

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
        ReadEnd() => {
          break @chunk;
        }
        ReadFailed(error: problem) => {
          break @chunk;
        }
      }
    }
  }
  return total;
}
```

A folding version replaces `set total = total +wrap taken;` with
`set digest = fold_bytes<'fold>(source: &'fold data, produced: taken, seed: digest);`
and is admitted on the same terms: `digest` is read and written in E alone, so
it is `serialized-E`, and [PAR-3] commits the remainder's writes in iteration
order, so the fold needs no associativity, no identity, and no combination
tree (`spec/kernel-spec.md:2070`).

## A.3 The spec sentences

Two amendments to [PAR-3] and one to the file resource's contract. No new
rule, no grammar production, no keyword, no operation, no writer-visible
marker.

### A.3.1 [PAR-3] — the terminating exit

Amend `spec/kernel-spec.md:2057`. The existing sentence, verbatim:

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
> when every action of P performs, for every place rooted outside B, no write
> and no consumption of an `own` value, and when every write of P is to a place
> this rule replicates or to a place rooted in a binding B itself introduces.
> An action performs a write, for this condition, exactly where its own
> contract fixes that it does [SYS-2, EFF-2]; an action whose write the
> implementation does not resolve denies the exit rather than admitting it.
>
> When an execution of one iteration leaves L through a terminating exit, the
> overlapped execution completes every operation a later iteration submitted,
> performs no segment E of any such iteration, delivers no outcome of such an
> operation to a source binding, and produces none of that iteration's
> observables. It produces exactly the observables the source-order execution
> produces before the exit and produces none after it.

`spec/kernel-spec.md:2058` — "An edge the statement performing c takes on the
outcome of that submission … is an edge of E and not of P" — stays exactly as
written. It is what makes the `ReadEnd` break an edge of E; the new sentence
is what admits an edge of E, and the two are consistent.

### A.3.2 [PAR-3] — the predicted place

Amend `spec/kernel-spec.md:2060`. The existing sentence, verbatim:

> Every place rooted in a binding declared outside L that a footprint of B
> reaches satisfies one of exactly three conditions, and a place satisfying
> none denies permission. Either no footprint of B writes it and every loan on
> it is shared; or every footprint element and every loan touching it belongs
> to one of P and E alone and no loan on it is retained past c; or this rule
> replicates it.

becomes "…one of exactly four conditions…" with a fourth alternative "…or this
rule predicts it", followed by:

> This rule predicts a place rooted in a binding declared outside L only when
> its type is copy [OWN-1]; when every footprint element touching it is a read
> of P or the target of one `set` statement of E whose target is that whole
> binding; when that `set`'s right-hand side is one operation total on its
> type applied to that binding and to one further operand; when every path
> through E that a later iteration's prologue can follow executes that `set`
> exactly once; and when the contract of the action at c fixes one value that
> operand cannot exceed and which the prologue evaluates before that action's
> outcome is published.
>
> An implementation may run the prologue of a later iteration on the value the
> `set` produces from that fixed value in place of the value the remainder
> computes. Before it produces any observable of that later iteration it
> compares the two, and where they differ it completes and discards that
> iteration and every iteration after it exactly as a terminating exit
> discards one and performs their prologues again on the value the remainder
> computed. The place therefore holds at every program point of every
> iteration the value the source-order execution holds there, and which
> prologue an implementation performed, discarded, or performed twice is not
> observable.

For `read_at` the fixed value is the one `spec/kernel-spec.md:2564` already
states — "Every successful transfer payload is an absolute endpoint `next` …
and satisfies `start <= next <= end`" — so the operand `taken` cannot exceed
the `end` actual, and the predicted cursor is `offset +wrap end`. Nothing in
source says so; the bound comes from the operation contract.

### A.3.3 [SYS-10] and [SYS-11] — a `ReadFile` is a regular file

A discarded prologue is invisible only if a positioned read leaves no state
and can be performed again. [SYS-11] already asserts it —
`spec/kernel-spec.md:2626`: "The explicit offset removes an implicit byte
cursor, so the operation observes but does not advance the `ReadFile` state" —
and `research/investigations/io-model/FIRST-PRINCIPLES.md:374-377` scopes it:
"MMIO, device files, virtual files, and read-and-clear state cannot be
silently admitted under the positioned-read contract."

Today one of the two constructors enforces that and the other does not.
[SYS-14] requires `open_file` to inspect (`spec/kernel-spec.md:2683-2685`):
inspection failure returns its class, a directory returns `IsDirectory`, and
"every other successfully inspected non-regular object returns
`Err(Other(code: 0_u32, origin: 0_u8))`". `open_read` has no such sentence, so
a `RelativePath` naming a fifo, a character device, or a `/proc` file yields a
`ReadFile` today. Add to [SYS-10]:

> `open_read` performs the same descriptor-status inspection before publication
> that [SYS-14] requires of `open_file`, with the same outcomes, so every
> `ReadFile` a program holds names a regular file and every positioned read of
> it is repeatable at the same arguments with no state change and no
> observation.

This is a correctness fix independent of Part A — it closes the hole
FIRST-PRINCIPLES §6 names — and it is the premise the discard rests on. It is
a semantic-ID change for `open_read` under [META-5] and moves the outcome for
a non-regular object from `Ok` to `Err`; conformance cases that open a
non-regular object through `open_read` change with it. I found none in the
tree, but the merge must check.
