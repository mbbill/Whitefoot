# Park on miss: one scheduler for compute hand-outs and I/O completions

Status: design settled, no open issue; not implemented. Derived with the
owner on 2026-09-02 and 2026-09-03, and re-anchored on 2026-09-04. Every claim
about the current runtime below cites the file and line it was read from at
`main` = `30602914` (specification v0.41 ACTIVE), on branch
`io/t4-resource-relations`; every sentence that is an inference rather than a
reading says so. The revision is named because the citations are read by
pulling up the line, and this tree moves. The runtime and emitter citations
were carried from the `ddcfdc47` reading and re-checked; the specification
citations were re-read at v0.41 and renumbered (§9).

Supersedes, once adopted: the stackless (continuation-frame) direction in
`FIRST-PRINCIPLES.md` §15 and §18.4 item 2, the selective stackless slice in
`compiler/src/backend/emitter/stackless.rs`, both writer-frame schedulers
(`compiler/src/backend/completion/writer_scheduler.c` and
`writer_scheduler_windows.c`), both parallel runtimes
(`compiler/src/backend/par_runtime.c` and `par_runtime_windows.c`, replaced
by the one scheduler core of section 7), the two roadmap
"widen stackless lowering" items, and two claims of `LOOP-PIPELINE.md` §3.4:
that the staged driver blocks on the oldest slot's join rather than suspending
(`LOOP-PIPELINE.md:822-825`), and the stage cutting with spill to the slot
record that was to carry a suspension without a stack switch
(`LOOP-PIPELINE.md:827-834`) — a stack that holds its own live values needs
neither. Those are edited in place in the batch that implements this design,
not before. Roadmap `BOUND-1`'s Windows policy of zero blocking fallback is
*not* superseded: this design removes the per-operation fallback everywhere
rather than adding one.

Five owner rulings govern everything below and are folded in rather than argued
again. **One stack class.** The first implementation has a single class, sized
as today's lane stack, in a fixed count set at startup; every stack carries
a guard page; there is no class test, no class-mismatch switch and no fifth
stack state. Proof-sized classes and the link-time class table are a named
later step (§5), taken only when the stack count reaches a size at which one
class is wasteful.
**Nothing is allocated at run time.** The runtime grows nothing: stacks come
from one reservation made at the core's entry, or a static array, carved into
equal slots with a free list; a stack's state header lives at its base; the ready
queue is an intrusive list threaded through those headers; and a miss that
finds no stack does not allocate one — the thread stays where it is (§2).
**Nothing runs above an I/O join.** The purpose the design serves is that a
completed I/O's continuation is never buried; measured against that purpose,
work nested above a frame that is waiting for a *compute* target delays no I/O,
because that frame is waiting for exactly the work the nesting produces. So the
exhausted path — no free stack and no READY stack — splits by what the join is
waiting for, and only the I/O arm is required to nest nothing (§2's fourth
line, §3, §7's `wf__par_wait` bullet).
**No per-operation blocking fallback, anywhere.** Emitted code has one lowering
for every I/O operation, submit then join, and the direct family leaves it (§8);
an operation with no kernel completion form is executed inside the runtime's
engine and publishes a completion like any other (§7.1); no refusal path is left
to fall to (§2, §7). Reproduction
follows from the same ruling: completion order is a kernel event and therefore
an *input*, so constitution T3's sequential world is realized by replaying
recorded inputs under §11's controlled scheduler, and the guarantee is stated as
**with identical external inputs, including completion order, the internal
execution is identical** — never as unqualified determinism (§11, §13).
**The completion record lives in the frame that submits the operation, and it
runs a compute hand-out slot's protocol, completed by the kernel instead of a
thread.** The two storages stay different — a compute slot is the lane's, and
keeps decision 6's refusal (§1, §7) — but the emitter reserves the record in
the frame (§5) and the join reads it with the compute slot's own protocol
(`par_runtime.c:448-463`, §6); it is sound
because the group's join precedes every read and every exit edge
(`parallel.rs:619-623`), so the frame outlives the operations it submits and a
parked stack's memory stays in place. Nothing that existed because records were
pool objects survives: no slot array, `WF_COMPLETION_SLOT_CAPACITY`, token,
generation, publication lock, milestone, dependent registration,
`wf_completion_depend`, claim, capacity notification, result copy, or refusal
(§5, §7). N operations cost N records in one frame, which the stack ledger
counts from the clang frame size, so the ceiling is the stack.

§11 carries the obligation the owner attached to these rulings: this state
machine's tests are part of the implementation batch, not a follow-up to it.

One issue of the review closed on a ruling that governs this design from outside
it. **Constitution T4**, resource dependencies are API relations: every finite
resource a system operation consumes is an owned value in that operation's
signature, drawn from a factory whose capacity is source-visible, so an
operation holding its resource cannot fail for want of it, and an
implementation never waits, awards, retries, or keeps a ledger to hide an
outcome the sequential program does not produce
(`docs/constitution.md`, T4, owner ruling 2026-09-04).

Every other issue the review rounds raised (ids 1 to 57) is settled and folded
into the section that owns it, with `reviews/park-on-miss-decisions.json` the
record and this the map. The NOTIFIED-at-commit arm enqueues, so
exactly one of the two transitions into READY does (§6, S7, §11-A5); both ends
of the free list are atomic (§3's I3, §7); a thread's host stack is not a
Whitefoot stack and every scheduler loop runs on a pool stack (§5, §7); §7.1's
primitive set is seven and §11's configurations are derived rather than chosen;
the exit-status post is a third epoch-bump transition, tested inside step 4's
capture-to-park window as the ready-list test is (§5, §6, S17, §11 items 16 and
20); target progress is the one primitive that may block, because with no helper
the bounded pass runs a host `open` or `close` inline (§7.1, §11 item 17); the
stack reservation and the worker pool start at different moments, the stack
count is its own setting and the pool's setting fixes only its floor (§5, §7,
§11); the stack free list is inside the core's one mutex and an EMPTY stack is
pushed back only after the switch, from the stack switched to (§5, §6, §11 item
18); completion members keep the join positions the emitter gives them today
(§4, §8); the Windows twins are named in the files they live in,
`wf_floor_windows.c` and `windows_bridge.c` (§3, §5, §7);
`wf__floor_attach_thread` splits into a per-thread alternate stack and per-stack
bounds written from the reservation record (§5, §7); I4 gains the power-of-two
half the deque's mask needs (§3); the floor's argument is stated (§5, §11 item
21); and every specification citation is read at v0.41 (§9). Every question
those rounds asked about a *refusal* path — where it waits, what it reverts to,
which platform it differs on — was answered by the rulings above, which remove
the paths instead.

That fixes the order of work. **Park on miss is implemented after the backed
`FilePermit` batch** (roadmap `BOUND-1`): that batch gives `FileFactory` a
capacity fixed at start and never larger than what the target provides, makes
`reserve_file` return a `Result`, and returns the permit or its credit on
release, after which an `open` holding a permit cannot fail for want of a
descriptor. The descriptor-retirement ledger, its award order, and the
retire-and-retry loop are deleted by that batch, and **nothing in this design
ports them**. Earlier drafts of this plan carried them as a stated exception
and spent a review round on how a thread should wait for a relation the API had
not stated; that text is gone rather than corrected.

## 0. The schedule that motivates this

Today's compute pool waits by helping. A thread that joins a task another
thread took runs whatever other work it can find, as an ordinary nested call
on its own stack (`compiler/src/backend/par_runtime.c:469-510`,
`wf__par_wait` calling `wf__par_execute` at `par_runtime.c:483`, which is
`slot->run(slot->frame)` at `par_runtime.c:457`).
To return to the join, everything stacked above it must first return.

For compute that is harmless: a continuation that is ready but buried under
other compute is not a loss, because every thread is busy anyway. For I/O it
is a throughput bound. Walk one thread through four independent iterations of
`read -> parse -> request -> write`:

```text
job1: submit read1 ; join(read1) misses -> help: job2
  job2: submit read2 ; join(read2) misses -> help: job3
    job3: submit read3 ; join(read3) misses -> help: job4
      job4: submit read4 ; join(read4) misses -> nothing left, wait
read1..3 complete            job1..3 are ready and buried; the thread waits for read4
read4 completes              job4: parse, submit request4, join misses, wait
                             requests in flight: 1, while three reads sit completed
```

On K threads only the top-of-stack join of each thread is live, so every
dependent stage after the first has at most K operations in flight, whatever
the scheduling policy. Helping is not a latency nit here; it collapses the
pipeline depth of every stage after the first to the thread count.

That schedule is not by itself this design's ground, because the staged
[PAR-3] pipeline already lowers exactly it: `LOOP-PIPELINE.md:229-277` draws
the same four iterations with K target operations continuously outstanding on
one lane, and the runtime's window is half the operation capacity, that is 32
(`bridge.c:857`). The ground is three shapes that pipeline never reaches.

1. A [PAR-1] window with a `may-suspend` member that is not a loop at all —
   two independent reads handed out in one group in straight-line code. The
   staged permission governs loops and does not reach this, so the join buries
   the thread.
2. A `may-suspend` call reached through recursion, or through a compute
   hand-out: the thunk's callee does I/O and runs nested on a lane's stack.
   There is no slot ring there at all.
3. A loop whose body does I/O and whose places the staged permission denies —
   the `denied` row of its disposition table (`LOOP-PIPELINE.md:350-355`),
   worked at its §2.3 and §2.5 (§2.4's loop does no I/O and is not a case
   here). It is lowered with the join at the back-edge and one operation in
   flight, which is the schedule above.

The claim differs between them and is worth stating exactly rather than
rounding up. For the first two, park on miss raises depth directly, because
each waiting continuation becomes a stack rather than a thread. For the third
it does not raise that loop's own in-flight depth — that needs a slot ring —
it frees the thread, so other stacks proceed.

The fix has one shape: a wait must not be tied to the one stack the thread
owns. The three places a waiting continuation can live are an OS thread (no
helping, compensate with more threads), a separate stack (switch away from
it), or a compiler-built frame (the stackless transform). This design takes
the second, because it needs no compiler transform and no coloring, and
because every Whitefoot frame is static
(`compiler/src/backend/stack_ledger.rs:38-42`), which lets a stack be sized by
proof rather than by guess.

## 1. The model

Every call the checker permits to overlap, and every I/O operation, is the
same thing to the scheduler: at the call site the work is handed away, the
caller keeps running, and the caller stops only at the join, which the
emitter places before any value of the group is read and before any exit
edge (`compiler/src/backend/emitter/parallel.rs:619-623`, the
`emit_overlap_joins` doc comment; the normative statement is [PAR-1] at
`spec/kernel-spec.md:1984`, that no edge out of s1 leaves the enclosing block
or function without first reaching s2). Nothing ever
waits at an I/O call itself. A `read` returns to its caller the moment the
operation has been accepted. A full submission queue does not change that, and
§7 says why.

```text
                     compute hand-out                    I/O operation
the record           a slot in the lane's pool           a block of the caller's frame
call site            push onto own deque, continue       submit to the completion source, continue
join, target done    read the result                     read the result
join, target missed  see section 2                       see section 2
who can finish it    any thread, including this one      only the completion source
```

**One protocol over two storages.** A compute slot is one of the lane's 64
(`par_runtime.c:184-207`), refused with NULL when the lane has none (`:757-780`);
an I/O record is the frame's and cannot be refused (§5). What they share is the
state-and-waiter protocol the join runs over either (`par_runtime.c:448-463`,
§6). Of the rows above only the last differs at run time, after a miss.

## 2. The rule

```text
join(X):
  X is done                                    -> read it
  X is the newest entry on this thread's deque -> pop it, run it here as an ordinary call
  otherwise, and a stack is available          -> park this stack
  otherwise, by what X is waiting for:
    X is an I/O operation                      -> wait in place; nothing runs above this join
    X is a compute hand-out                    -> run own deque, then steals, above this join
```

The first three lines do not test what X is, and no line tests a stack class:
there is one class (§5). Line one is one rule for both kinds — read the record's
state, and its result if DONE — differing only in who stored DONE, a thread or
the kernel (§6). An I/O operation is never on the deque, so it takes the first
or the third line. A compute hand-out another thread took and has not
finished takes the third line as well; "run it yourself" is undefined for it,
and helping instead would reopen the burial through a compute join. Line two
also tests that X's home lane is this thread's (§4). Only the fourth line asks
what X is waiting for, and §3 says why the two arms are not the same question.

**No refusal path exists.** Owner rulings, 2026-09-04: there is no record
resource to refuse, because the record is a block of the submitting frame and
the stack is its only ceiling (§5), and ring backpressure stays the engine's
(§7). Emitted code
therefore has **one lowering for every I/O operation: submit, then join** —
there is no second arm to fall to, on any platform. Two refusals remain and
neither is an I/O wait: a refused compute slot runs the call inline on the owner
(`parallel.rs:655-660`, §7), which blocks nothing, and honest target exhaustion
stays the operation's own typed error under constitution T4 (§3).

"Park this stack" means: register the stack as the waiter of X, switch to
another stack, and continue there. There are two kinds of switch target and
they are not the same thing, which the rule has to say because only one of
them continues a scheduler loop. A free stack from the pool is empty: the
thread enters the scheduler loop on it. A READY stack is not empty: switching
to it returns into that stack's own join, which resumes where it parked. Both
satisfy the third line, so a miss that finds no free stack but does find a
READY stack parks normally. The thread that parked a stack does not switch
back to it on purpose; a scheduler stack it moved to is used until it is empty
and then returns to the pool.

The fourth line is the refusal the fixed stack pool requires, and it is an
outcome of its own rather than a detail of the third. The runtime allocates
nothing at run time, so the stack count is set at startup, and a miss can
find no free stack *and* no READY stack. What the thread does then is the one
place in the rule where the target's kind matters, and the owner's ruling of
2026-09-03 is the reason: this design exists so that a completed I/O's
continuation is never buried, and work nested above a frame that is waiting for
a *compute* target buries no such continuation, because that frame is waiting
for exactly the work the nesting produces (§3).

**I/O target.** The thread stays on this stack and nothing runs above the join.
It flushes the deferred doorbell (`wf_bridge_flush_target`,
`bridge.c:810-817`), runs one bounded target pass and progress
(`wf_bridge_progress`, `bridge.c:445-477`, which reaches the kernel through
`wf_linux_io_uring_progress`, defined at `linux_io_uring.c:1113`,
and runs the target
pass itself when the helper count is zero, `bridge.c:473-475`, pinned to zero
on Linux with a ready ring at `bridge.c:172-176`), drains ready completions,
and only then sleeps on the one primitive (`bridge.c:502-530`). That is today's
join loop with helping removed, and §6's step 4 is the same sequence — the same
one, not a matching one: the arm runs step 4's window, so it tests the ready
list where step 4 tests it, after the epoch capture and the progress pass and
before the park, and on every wake as well. A READY
stack that appeared while the thread was flushing or reaping is a switch
target, and the thread then parks this stack with its own I/O as the wake and
switches to it. That is the third line taken late, not a fifth outcome, and it
is why the arm's precondition is tested again after every pass rather than once
on entry.

**Compute target.** Today's `wf__par_wait` survives here and only here: pop
this thread's own deque from the newest end (`par_runtime.c:478`), then steal
from the oldest end of other lanes (`wf__par_find`, `par_runtime.c:480`, whose
`wf__par_steal` takes from `top`, `par_runtime.c:402-415`), run what it gets as
an ordinary nested call above the join (`par_runtime.c:482-485`), until the
target's state is DONE (`par_runtime.c:473-474`). Which pop that is matters:
the running thread's own (`wf__par_pop(lane)` with the thread's own lane), not
`wf__par_join`'s pop of `target->home` (`par_runtime.c:818-819`), which in the
crossed case below is a foreign deque that I2 forbids. A nested entry that
itself misses on I/O with no stack available takes the I/O arm above, so
nothing runs above *that* join (S23).

The crossed case is what makes the compute arm necessary rather than merely
convenient. Two stacks resumed on foreign threads, each joining a hand-out on
the other's home lane, with no free stack and nothing READY, leave each thread
holding the other's only engine (S22). Waiting in place on both would move
neither, because a compute hand-out moves only when some thread reaches
priority 2 or 3 of §6's loop and neither thread is in that loop. The steal is
what resolves it: each thread's `wf__par_find` reaches the other's lane and
runs the entry it is waiting for. Sizing the pool past the state is not an
alternative, because compute-parked stacks are bounded only by the lanes times
`WF_PAR_LANE_SLOTS` (`par_runtime.c:94`, `:113`) at a 1 GiB reservation each.

What the compute arm drops from today's `wf__par_wait` is the sleep, not the
work-finding loop. The lane mutex, the `waiter` store and the
condition-variable wait (`par_runtime.c:500-507`) go, because §6 replaces the
lane signal with the ready publication; the Windows twin's writer help
(`WF_PAR_WRITER_HELP_ONCE`, `par_runtime_windows.c:625-628`) and its
`WaitOnAddress` sleep (`par_runtime_windows.c:648-655`) go with the twin
itself. The spin and yield rounds (`par_runtime.c:487-495`) stay as the
empty-handed pause. That an empty-handed turn is bounded is an inference rather
than a reading: an empty turn means nothing is stealable anywhere, so the
target is in some thread's hands and that thread is running it — to its end, or
down to an I/O wait that its own arm ends — and the arm is therefore re-testing
a condition another thread is moving. It is a spin while that lasts, which is
the price of the exhausted path and one of the things §11 asks a test to reach
deliberately.

## 3. Why nothing is buried

The principle, in the form the owner ruled on 2026-09-03: **a completed I/O's
continuation is never buried.** Nothing runs above a frame that is waiting for
I/O. That is the purpose the whole design serves, and it is the measure every
other rule here is checked against.

Invariant I1 is the stronger property the design holds almost everywhere, and
the way it delivers the principle. On every stack, every frame that is stopped
at a join is waiting for something that transitively includes the frame above
it.

The rule preserves I1 because the only thing ever run above a join frame is
the join's own target (line two), and a target's completion is exactly what
the join needs. A parked stack therefore contains no frame that is ready: a
frame under the park point is waiting for the frame above it, which is
waiting for the event. When the event arrives, the stack is ready as a whole
and any thread may run it.

The loss the motivating schedule shows is a frame that is ready and not
runnable. Under I1 that state does not exist. A frame that is not ready and
not runnable is not a loss.

I1 has one stated exception in this design, and it is stated rather than
hidden because the principle above survives it. The compute arm of §2's fourth
line runs deque and stolen work above a join that is not its target, which is
an I1 violation of exactly the shape `wf__par_wait` has today. It is admitted
because the frame below is waiting for a compute hand-out, not for I/O, so the
nesting delays no completion: the frame is waiting for exactly the work the
nesting produces or for work of the same population. It is bounded twice over —
only while no stack can be had, and only above a compute join — and a nested
entry that reaches an I/O join with no stack takes the I/O arm, where nothing
runs above it. Above an I/O join the exception does not apply at all, on any
path.

Today's runtime violates I1 in three places on POSIX. The design removes two of
them outright and narrows the third to the exception above:

- `wf__par_join` (`par_runtime.c:816-847`) runs same-group siblings above the
  join when the target is not the newest entry. That is a *bounded* violation
  over one group, not the unbounded burial the next two allow, and it is
  bounded because the members of one group are disjoint by the judgment that
  permitted them.
- `wf__par_wait` (`par_runtime.c:469-510`) steals and runs unrelated work
  above the join, through `wf__par_find` at `par_runtime.c:479-486`. This is
  the one the design narrows rather than removes: it survives on the exhausted
  path with a compute target and nowhere else (§2's fourth line, §7), so the
  unbounded burial it allows today is reachable only when no stack can be had,
  and never above an I/O join.
- The I/O join `wf__completion_file_join` (`completion/bridge.c:1692-1725`)
  calls `wf_bridge_progress`, which calls `wf__par_help_once`
  (`bridge.c:476`), which runs a compute hand-out above the I/O join
  (`par_runtime.c:799-814`).

Windows has four, and they are not the same four in the same places. Two are
the parallel runtime's, twins of the first
two above: `wf__par_join` (`par_runtime_windows.c:943-964`) and `wf__par_wait`
(`:614-658`). A third is the Windows-only one: `wf__par_wait` calls
`WF_PAR_WRITER_HELP_ONCE` inside its wait loop
(`par_runtime_windows.c:625-628`), which runs a writer frame above a compute
join. The fourth is the twin of the POSIX I/O-join violation, and it lives where
that one does — in the completion bridge, not the parallel runtime: the
`wf__par_help_once` calls at `windows_bridge.c:772` (inside the record-capacity
wait §7 deletes), `:793` (inside
`wf__windows_completion_progress_for_retirement`) and
`:1231` (inside `wf__writer_run_root`). The Windows I/O joins themselves
(`windows_bridge.c:1138-1153`, `:1155-1180`) do *not* call it, which is why the
Windows count differs from the POSIX one in kind and not only in number. So the
count is three on POSIX and four on Windows.

So the design carries **one** stated exception and no others: the compute arm
of §2's fourth line, above. It is bounded by a resource state — no stack — and
it runs nothing above a frame waiting for an I/O completion, which is why the
principle at the top of this section survives it. Nothing else in emitted code
can run a host call above a live frame either, because the direct family is no
longer there to run: emitted code submits and joins, and that is all (§2, §8).

An earlier draft carried a second. `wf_bridge_retire_and_retry_direct` drives
the engines above a live frame while it waits for a descriptor, and the plan
kept it as an exception it could not remove. It is not an exception any more,
because the site itself is gone: under constitution T4 the backed `FilePermit`
batch deletes the descriptor-retirement ledger, its award order, and that loop
before this design is implemented (header, roadmap `BOUND-1`). An `open`
holding its permit cannot fail for want of a descriptor, so no frame waits for
one, and the principle holds with nothing to except.

That is worth stating as the test rather than as a repair, because it is how
every later device API is to be judged: **if overlap can invent an outcome the
sequential program never produces, a resource is missing from the API, and the
answer is on the API, never in the scheduler** (constitution T4). The descriptor
was the instance. The scheduler's job is to place work, and a scheduler asked
to hide an outcome is a scheduler covering for a relation the checker was never
told about.

Invariant I2, which §4's property rests on: a deque is pushed and popped only
by its owning thread, so Chase-Lev keeps a single owner even though stacks
migrate between threads. Nothing in this design may pop a foreign deque.

Invariant I3, which the release path rests on: exactly one thread ever pops a
given lane's free list — `wf__par_acquire_lane` reads `wf__par_self`
(`par_runtime.c:758`) and never `slot->home` — while any thread may push onto
it, because a resumed stack releases on whichever thread resumed it. That
asymmetry is what makes the release safe without a tag or a generation counter
over the head at `par_runtime.c:225`: ABA on a single-consumer, multi-producer
stack needs two pops, and there is only ever one popper.

I3 answers ABA and nothing else, and the difference matters. A single popper
does not make a plain read-modify-write safe against a concurrent pusher: that
is a lost update, a different hazard with a different fix. Both ends of the free
list are therefore atomic — a release compare-exchange push and an
acquire-then-compare-exchange pop (§7) — and I3 is what says neither needs a tag
on top of that.

Invariant I4, which the unchecked push rests on: a lane's deque buffer holds
exactly as many cells as its lane has slots, and a slot is on the deque at most
once. That is why `wf__par_push` needs no fullness test
(`par_runtime.c:356-359`), and nothing in the tree states it today — the two
lengths merely happen to be the same constant (`par_runtime.c:222`, `:238`,
threaded at `:592-598`). **I4 has a second half the wrap depends on: the slot
count is a power of two**. The deque does not index
modulo its length, it masks — `bottom & (WF_PAR_LANE_SLOTS - 1)` at
`par_runtime.c:358` in the push, `:382` in the owner pop and `:409` in the
steal — so the coupling is necessary and not sufficient: at capacity 3 the two
lengths still match while `bottom & 2` folds two live entries onto one cell.
The tree states this today in exactly one place, and it is a place this design
deletes: `par_runtime_windows.c:61-64`. `par_runtime.c` carries no
`_Static_assert` at all, so without this clause the property leaves the
repository with the file. The core therefore states both properties here and
asserts **both** with `_Static_assert`, on the model of `bridge.c:55-58`, so
neither a later change that gives the deque its own constant nor one that picks
a non-power-of-two slot count can silently reintroduce an overwrite.

## 4. Join order and the deque

The deque is Chase-Lev: the owner pushes at the newest end
(`par_runtime.c:356-359`) and pops from it (`par_runtime.c:367-393`),
thieves take from the oldest end (`par_runtime.c:402-415`). Stolen entries are therefore always a prefix of
the publish order and the owner's own entries a suffix; the two ends never
leave a hole between them.

The emitter today joins a group in publish order (`parallel.rs:631`,
`for pending in std::mem::take(&mut self.handed_out)`). This design reverses
it for compute members: publish J1, J2, J3, join J3, J2, J1. Then at every
join the target is either the newest entry of the deque or it has been
stolen, never present and blocked by something newer. The runtime needs no
notion of a group; it looks at the newest end once.

One function, `compute_join_order`, is the single definition of a group's
compute join order, carrying the reason above in its doc comment. Three sites
encode that order today and all three consume it: `emit_overlap_joins`
(`parallel.rs:624-670`), `overlap_join_tail` (which uses `handed_out().last()`
as the group's exit label, `emitter.rs:2234-2239`, documented at `:2228-2233`),
and `block_exit_label` (which replays the same queue for phi predecessors). Two
structures carry the order — `IrOverlap::handed_out`, the accessor over the IR
member list, which lives in the lowering module and not the emitter
(`compiler/src/lowering.rs:1086-1091`, on `struct IrOverlap` at `:1070`), and
`FunctionEmitter::handed_out`, the emitter queue (`parallel.rs:495-503`,
`631`) — and both go through the one function. Reversing only the loop in
`parallel.rs` would leave the other two naming a block that is no longer a
predecessor.

An I/O member holds no deque entry, so the deque places no constraint on its
join, and **completion members keep the order they have today**. That order is
the publish queue itself:
`emit_overlap_joins` walks `handed_out` once and dispatches by kind in place,
the completion arm joining the member where it sits (`parallel.rs:631-637`)
and the compute arm falling through (`:639-668`), so a completion member
published before a compute member is joined before it. The one member that
moves is one a later step names in `wait_for`: `emit_completion_dependencies`
removes it from the queue by position and joins it at that step
(`completion.rs:244-252`). An earlier draft said instead that an unnamed member
is joined after every compute member, which is neither what the emitter does
nor something any dependency edge fixes; the rule is now stated as the code
has it, and no emitter change follows (§8 is unchanged).

`compute_join_order` therefore defines the order of **compute members only**,
and this is where that is said, so the plan's single definition point does not
leave one of the two kinds unassigned. The reversal does not disturb an
interleaved completion join, and this was checked rather than assumed: an I/O
join takes line one, line three, or the I/O arm of §2's fourth line, and never
line two, so it pops nothing and leaves the deque's newest end exactly as it
found it. What it can do is park — or, with no stack to be had, wait in place
on the fourth line with nothing above it — and a park between two compute joins
is the counterexample §4 already prices below at one park: the same cost,
reached by a different route.

The property holds with a precondition, not unconditionally, because a deque
belongs to a lane and not to a call chain (`par_runtime.c:213-226`,
`par_runtime.c:356-359`). Stated in full: at a join taken on the target's home
thread, with no other stack having pushed onto that lane between this stack's
publishes, the target is the newest entry or has been stolen. The
counterexample is stack A publishing J1 and J2, parking, the same thread
resuming stack B which publishes K1, and A joining J2 against a newest end
that is K1. There the rule falls to line three, which costs one park and
nothing else.

The deque therefore stays per thread and line two gains a home test: X's home
lane must be this thread's. The cost is one thread-local load and a compare —
`wf__par_join` loads `target->home` today (`par_runtime.c:818`) but never
reads `wf__par_self` (`par_runtime.c:276`), so the load is new — and what it
buys is the removal of the cross-thread owner-side pop `par_runtime.c:819`
would otherwise perform once a stack can migrate. That is invariant I2 (§3),
and it is what keeps Chase-Lev's owner side single-owner.

Join order is not observable: [PAR-1] fixes every value to the source-order
result (`spec/kernel-spec.md:1987`) and states that the schedule is not an
observation (`spec/kernel-spec.md:1993`). The reversal is an emitter choice.

## 5. Stacks

A stack has four states.

```text
RUNNING  --join misses, park-->  PARKED  --event arrives-->  READY  --a thread switches to it-->  RUNNING
RUNNING  --unwinds to the scheduler loop at its bottom-->  EMPTY  --pushed after the switch, from the stack switched to-->  pool
```

Every stack's bottom frame is a scheduler loop. Nothing below the loop is
ever returned into. This is a hard requirement, found while walking the entry
case: the entry stack today bottoms out in `wf__floor_entry` on the entry
pthread (`compiler/src/backend/wf_floor.c:290-295`), and `wf__floor_run`
joins that pthread for the exit status (`wf_floor.c:327`). A parked entry stack resumed by
another thread and run to its end would return from a function that thread
never called. So `wf__main_body` runs on a fiber whose bottom is the loop;
when it returns, the status is posted and the entry thread, which is by then
running its own scheduler loop on a pool stack, sees the program is done,
switches back to its host stack and returns the status from there.

That last clause is the rule the plan owes, and it is one rule for two questions: which stack a thread's scheduler loop
runs on, and what happens to the stack the thread started life on. **A thread's
host stack is not a Whitefoot stack.** The entry thread's process stack and
each worker's pthread stack — reserved by platform item 1 of §7 at
`par_runtime.c:659-668` and run today by `wf__par_worker_main`
(`par_runtime.c:522-527`) — are outside the carve: never on the free list,
never on the ready list, never parked, never READY, and carrying no Whitefoot
frame at all. Every thread's first act is to switch to a pool stack, and every
scheduler loop runs on a pool stack, so §6's steps 2 and 3, which run work
nested on the stack they are on, always nest onto a stack the four states
above cover. A host stack is entered at most twice: once at thread start, to
take a pool stack and switch to it, and once at the end. Only the entry thread
has that second entry — it switches back when the status is posted and executes
`return call.status` (`wf_floor.c:328`) — because workers are created detached
(`par_runtime.c:669`) and never return through anything. The pool stack the
entry thread leaves at that second entry is pushed back to the free list from
the host stack, after the switch, for the same reason every EMPTY push happens
after its switch (below). The extra loop exit that tests "the status is posted"
belongs to the entry thread alone, and it sits **inside §6's step 4**: after
the wake epoch is captured and before the park, in the same window as the flush
and the progress pass, and not merely on the far side of a wake (§6). And no
thread ever switches to another thread's host
stack, which is what keeps the return on the thread that made the call.

An earlier draft left the entry thread's loop on the host stack, and that was
the same defect this paragraph exists to prevent, one level down: §6's loop
would run stolen work nested there, that work could miss and park the host
stack, and a worker resuming it would run `wf__floor_run`'s return on a thread
that never called it. The other repair — leaving the entry thread asleep on its
host stack — does not survive the pool-off world, because with no workers there
would be nobody left to run the entry's pool stack (S10a).

There is no run-time conditional here: no join has a pool-off behaviour, and
§7 records why one could not exist even if it were wanted. What selects the
entry's shape is a *link-time* fact, which is a different thing and the
distinction from the runtime world query §7 refuses: the linker resolves it
once, and no runtime unit has to ask anything.

**Windows has its own floor file and its own instance of all of this**. Everything above is derived against `wf_floor.c`,
and `wf_floor_windows.c` is the third twin file, beside the two §7 already
names. It has its own `wf__floor_entry` (`wf_floor_windows.c:136-141`), which
runs `wf__main_body` on a thread `wf__floor_run` (`:143-175`) creates with
`_beginthreadex` (`:157-164`); it waits for the exit status with
`WaitForSingleObject(thread, INFINITE)` (`:170`) and returns it at `:174`. So
the hard requirement is the same there — a parked entry stack resumed by
another thread would run that return on a thread that never made the call — and
the wake this design removes on Windows is the `WaitForSingleObject`, not a
`pthread_join`.

What changes there is what changes here: the weak core-symbol test selects the
entry's shape at link time, the entry runs `wf__main_body` on a pool stack whose
bottom is the scheduler loop, its host stack is entered at most twice (with
`ConvertThreadToFiber` at that first entry and `ConvertFiberToThread` at the
second, §7's platform item 3 — for the entry thread, "thread start" is
`wf__floor_run` in this file), and the exit wake becomes the status-post epoch
bump of §6. What does **not** change is the one place Windows differs: a failed
`_beginthreadex` calls `abort()` (`:166`) where `wf_floor.c` falls back to
running the body on the host thread, because "failure to reserve its specified
stack makes the native backend unavailable" (`:152-156`). That abort stays
exactly as it is. §5's conclusion below, that the POSIX fallbacks become
unreachable rather than an alternative shape, therefore has no Windows
counterpart to reach: there was never a second shape there.

`wf_floor.c` is compiled into every link, so the two fallbacks at
`wf_floor.c:314-316` and `wf_floor.c:317-321` are not deleted and stay where
they are; the one change to the file is the test below, which is a source change
and is named as one. `wf__floor_run` tests a weak
core symbol — `wf__sched_entry_stack`, whose
weak definition answers null — on the pattern this tree already uses three
times: `bridge.c:119-123`, `parallel.rs:175`, and the scheduler hooks at
`writer_scheduler.c:55-57`, one weak definition per line. `wf_floor.c:40-42`
already describes the pattern
for `wf__main_body` itself.

Symbol present, which is every link that contains the core: the entry takes a
stack from the pool reservation and runs on it, `pthread_create` is not on the
entry path at all, the failure the two fallbacks cover therefore cannot arise,
and they are unreachable rather than an alternative shape. Symbol absent: the
function is exactly today's, fallbacks included, and such a link cannot park
because it has no core, so the hard requirement above has nothing to protect
there.

Sizing, and where the stacks come from. There is **one class**, sized as
today's lane stack (`wf__floor_stack_bytes`, 1 GiB reserved with pages
committed on touch, `wf_floor.c:55,66`; the Windows floor reserves the same
number at `wf_floor_windows.c:30,33`, and `par_runtime_windows.c:807` reads it
for every worker), and **every stack carries a guard page**. The count is a
startup setting like `WF_WORKERS` and never grows: the runtime allocates
nothing at run time. One reservation, or a static array, is carved into equal
slots with a free list; a stack's state header lives at its base, so a stack
handle and a header pointer are the same address. Getting a stack is a
free-list pop; a miss that finds the list empty does not allocate, it takes
§2's fourth line. A reservation that cannot be made refuses to start, which is
what `par_runtime.c:659-666` already does on POSIX and
`par_runtime_windows.c:809-811` does fatally on Windows.

**Two starts, and they are not the same moment.**
An earlier draft said "at pool start" and meant the worker pool's, which is a
point neither shape this design needs actually reaches: `wf__par_start` runs
only under `pthread_once` inside `wf__par_attach` (`par_runtime.c:729-730`),
attach is reached only from `wf__par_acquire_lane` (`par_runtime.c:770`) and
from the writer-scheduler hook §7 deletes (`par_runtime.c:742-746`), and
`wf__par_requested_lanes` answers 0 for `WF_WORKERS` under 2
(`par_runtime.c:642-643`) so `wf__par_start` returns before preparing anything
(`par_runtime.c:655-657`). A completion-only program never acquires a lane at
all, and S10a parks with no workers; both need a stack.

So the **stack reservation is made at the core's entry** — in `wf__floor_run`,
on the branch where the core symbol resolves (§5 above) — before
`wf__main_body` runs, unconditionally in every core link. **Worker-thread start
stays exactly where it is**, lazy at the first lane acquisition. The two starts
are independent in *time* — a link may have stacks and never a lane — and the
two counts are independent too, above one floor the pool's setting fixes.

**The stack count is its own startup setting, and its floor is the thread count
plus one.** A setting below the floor is raised to it; above the floor the count is free, and nothing ties it to `WF_WORKERS`. The
thread count is the lane count when the pool is on — lane 0 belongs to the
calling thread, so `WF_WORKERS=n` is n threads in total
(`par_runtime.c:673-677`) — and one when it is off, since `WF_WORKERS` under 2
answers 0 lanes (`par_runtime.c:642-643`, `wf__par_requested_lanes` at
`:634-649`) and the entry thread is still a thread. So every core link reserves
at least two stacks.

The floor and not the count is what carries the argument, and the distinction
matters because sizing the count from the pool's setting would fix S at exactly
T + 1 — one parked stack in the whole process, which is not the design §0, §11
and §12 describe. What the floor has to carry is §11's arithmetic plus one fact
about thread start. Every thread holds exactly one pool stack while it runs, so
with T threads the free stacks are S − T and a first park needs S ≥ T + 1 (§11).
And a thread's first act is to take a pool stack and switch to it, which is
*not* a join: §2's fourth line does not cover it, so a worker that starts and
finds the free list empty has no rule to fall to.

The floor removes that state, and the argument takes three facts rather than
one. The first is the bound an earlier draft stated
alone, and it is the weakest of the three: `wf__par_start` may start fewer lanes
than requested — it stops at a `pthread_create` failure and publishes the lower
count (`par_runtime.c:688-695`) — and never more, so a reservation whose floor
is computed from the requested count before any lane exists is a bound the pool
cannot outgrow. That bounds T. It does not by itself keep the free list
non-empty at a pop, because parked stacks come out of that same list: at the
floor S = T + 1, the last thread to start finds a stack only while at most one
is parked. Two facts about *when* the pool starts supply that.

**Before the pool starts, only one stack can be parked.** There is one thread,
and it parks only from its own scheduler loop, where priorities 2 and 3 find
nothing: no lane exists yet, because `wf__par_requested_lanes` answers 0 below
`WF_WORKERS=2` (`par_runtime.c:642-643`) and `wf__par_self` is NULL until
attach. So the one stack that can be parked at that moment is the entry's.

**During the create loop, no started worker can park either.** The pool starts
from inside a hand-out — `wf__par_acquire_lane` calls `wf__par_attach`, which
runs `wf__par_start` under `pthread_once` (`par_runtime.c:770`, `:729-730`) —
and the emitter's hand-out is acquire, store the operands, publish, with no join
between (`parallel.rs:478` to `:492`). So while `wf__par_start` is still
creating threads (`par_runtime.c:678-694`) every prepared lane's deque is empty,
and a worker that has already started cannot steal, miss, and take a stack.

Both facts are contingent, which is the reason to write them down and to assert
the conclusion rather than the premises: a scheduler stack that could run work
before the first lane exists, or an eager pool start moved ahead of the entry's
first hand-out, would remove one of them, and at the floor a thread start would
then find the list empty with no rule to fall to. §11 item 21 asserts the
conclusion directly. The alternative was to cap the lane count by the stacks
reserved; it is not taken, because it makes the worker pool read a core capacity
at its own start and still leaves this floor to be enforced somewhere.

**The completion record is a block of the submitting frame, and the stack
ledger already counts it.** That is the owner's fifth ruling (header), and the
stack floor above is the only sizing rule this design owes. The runtime owns the
layout — an atomic state (PENDING or DONE), the waiter, the typed result of
value and error code, the operation kind, and a platform block whose three forms
§7 anchors — and the emitter reserves that block where it reserves an
operation's other storage today (§8). So a group of N operations costs N records
in one frame and nesting costs one set per live frame; every Whitefoot frame is
static (`stack_ledger.rs:38-42`), clang sizes it and the ledger reads that size,
so the ceiling is the stack and nothing else.
An earlier draft sized a record pool from `wf__completion_window`
(`bridge.c:845-887`), which its own comment calls a per-loop query — "asked once
per loop entry and never per iteration" (`bridge.h:30-35`) — and not a bound on
one stack's outstanding operations, which a straight-line [PAR-1] group of N
members reaches at whatever N the source states. The query itself does not
survive either, because every input it has is deleted here: the slot constant at
`bridge.c:857` and the ring and entry capacities it clamps to
(`bridge.c:858-864` over `linux_io_uring.c:1440-1450`). **The window becomes an
emitter depth choice, not a resource-derived bound.** With no pooled
per-operation resource left, nothing in the runtime bounds how many iterations
one loop may carry, so the number the staged [PAR-3] lowering consumes
(`lowering.rs:1211-1234`) is the compiler's own choice of depth against the
frame it costs. The one thing the runtime still answers is the floor it answers
today: one is always legal and reproduces the sequential program exactly
(`bridge.h:30-36`).

The record holds the *loans* of the buffer and the path until the join rather
than copies, which the specification already permits — "Submission may retain
only the loans recorded for that call" (`spec/kernel-spec.md:1456`) — so the
fixed path buffer (`WF_FILE_PATH_CAPACITY`, `file_adapter.h:65`), the demotion
of a path that does not fit (`bridge.c:1049-1059`) and the 256-byte result copy
(`WF_COMPLETION_RESULT_CAPACITY`, `contract.h:30`) all disappear. §12 prices the
frame growth this costs.

On cost, the measurement
at `par_runtime.c:865-875` prices a different move and is not borrowed: its 17
and 18 percent on `par_layout.wf` is the cost of starting *worker threads*
early, so that they "spin alongside that work"; the stack reservation starts no
thread. It is one mapping with pages committed on touch plus a carve of the
free list, and that this is not the measured cost is an inference from what
that comment attributes the cost to, not a reading. `WF_WORKERS=0` and `1`
therefore keep their meaning — no lanes, no hand-outs, the sequential world —
over a stack pool that exists and parks (S10a).

The stack free list has a protocol, and it is not I3's. Its poppers are three
kinds and the list is complete here: any thread taking §2's third line pops one;
every worker pops one at its own start, because a thread's first act is to take
a pool stack and switch to it; and the entry thread makes that same start-time
pop on its own path, at the core's entry, just after the reservation and before
`wf__main_body` runs. Any thread whose scheduler stack empties pushes onto it.
So it is many-producer many-consumer; I3's premise is a single popper per lane
and does not transfer, and with two poppers the read-head, read-successor, swing
sequence is open to ABA, whose failure is one stack handed to two threads.
**The core's one mutex covers this list as well as the ready list.** §6's
argument for affording that lock carries over unchanged, because the two lists
are touched on the same two events: a park takes a stack from the free list and
may link one onto the ready list, and a resume unlinks one and may return one.
So in the steady state the lock is still taken once per park and once per
resume, not once per DONE and not once per completion; beside that steady state
stand the start-time pops named above — one per thread, plus the entry's — which
happen once each and are not on any repeated path. The named alternative stays
named: a tagged lock-free stack, with the tag doing the work the single-popper
argument cannot do here, if §12's park-cost measurement shows the lock on the
park path.

The mutex says who may touch the list's words. It does not say *when* a stack
may be offered, and that is a separate rule the design needs: **an EMPTY stack
is pushed to the pool after the switch, from the stack switched to**, exactly as
step 5's commit is made from the target stack (§6). A push issued before the
switch hands the stack to a second thread while the first is still executing on
it, and the lock does not prevent that — the pusher holds the lock legitimately
and publishes a stack that is still in use. The entry thread's last pool stack
follows the same rule at its second entry: it switches to the host stack first
and pushes the pool stack from there (§5 above).

One class means line two of the rule needs no class test, there is no
class-mismatch switch, and no fifth stack state: a scheduler-loop stack
switched away from with nothing pending is EMPTY, and the thread that switched
to the next stack pushes it back to the free list from there.

Proof-sized classes are a **named later step**, not part of this design, and
they are taken only if the parked population can exceed tens (§12), which the
stack count alone decides. When they are, the class cannot come from
the emitter reading the ledger: the stack ledger is produced from clang's
output after the emitter has finished (`whitefootc.rs:282-310`, which already
runs clang), and its chain bound excludes every runtime and libc frame and
gives a cyclic component zero bytes (`stack_ledger.rs:446`). So the class comes
from a link-time table the ledger pass emits as a constant array the runtime
reads at start: a thunk's class is its callee's chain bound plus one fixed
constant for the frames the ledger excludes (scheduler loop, `wf__par_execute`,
thunk, submit path); any entry whose chain passes a cyclic component is the
large class; no table means the large class.

Bounds for the exhaustion floor are per stack on POSIX, set at every switch,
**and the switch writes them from the reservation record rather than asking
pthread**. Today they are three thread-locals
captured from pthread (`wf_floor.c:72-74`, `wf__floor_capture_bounds` at
`wf_floor.c:202-226`), and the handler needs them because it classifies a
delivered signal against an address range (`wf_floor.c:150-158`). That query is
wrong for a pool stack, and silently so: a Whitefoot stack is a slot of the
core's own carve and no thread's pthread stack, so the query answers the *host*
stack's bounds, the range test at `wf_floor.c:155-158` fails for a pool-stack
overflow, and the handler hands it to the default disposition as "not this
mechanism's fault class" (`wf_floor.c:160-165`) — the floor off for exactly the
stacks this design adds. The carve knows each slot's base and size, so the
switch writes the target stack's own low and high from that record, and
`wf__floor_capture_bounds` survives only for a host stack. S8 depends on this:
it asserts one record under the single latch for an overflow that has to be
classified as a guard hit first. Windows needs none of this: its floor is a
vectored
exception handler that classifies `EXCEPTION_STACK_OVERFLOW` by its code
(`wf_floor_windows.c:5,73,104`) and reserves the guarantee with
`SetThreadStackGuarantee` (`wf_floor_windows.c:121`), so a switch there
carries no per-stack bounds at all. What Windows does need, found on the real
host after the port landed, is the guarantee *on each pool fiber*: Windows
keeps it for the calling thread or fiber, a fiber takes it only when it is set
from inside that fiber, and a thread's guarantee does not carry to a fiber it
runs. So the floor's attach runs once at every pool fiber's first frame
(`sched/prim_windows.c`, `wf_prim_fiber_main`) beside every thread's start;
before it did, an overflow on a pool fiber had only what was left of the guard
page under the handler and was classified or not by where in the page the
descent stopped.

The parallel runtime holds two more thread-locals, `wf__par_self` and
`wf__par_attached` (`par_runtime.c:276,280`); those stay per thread and must
be re-read after a switch rather than cached across one.

State that migrates with a stack is not found by searching for
`_Thread_local`, so the method is a review of what is reasoned about per
thread, not a grep. Applied to the runtime this design inherits, the review
finds nothing left to carry: the one piece it used to find was the
descriptor-retirement waiter, a stack local linked into a process-wide award
order, and both are deleted by the backed `FilePermit` batch this design
follows (header). The floor's bounds above are per stack, and the parallel
runtime's two thread-locals are per thread and re-read after every switch.

## 6. Wake protocol

A stack's park is the writer-frame handshake already in the tree, applied to a
stack instead of a frame (`completion/writer_scheduler.c:102-181`):

```text
RUNNING --begin park--> SUSPENDING --commit--> SUSPENDED --event--> READY (enqueued by the event)
                        SUSPENDING --event--> NOTIFIED  --commit--> READY (enqueued by the commit)
```

Both arms enqueue, and an earlier draft said the second one did not. The reason it must is the same one the tree has:
`wf__writer_commit_suspend` finds NOTIFIED, compare-exchanges to READY, and
calls `wf_writer_enqueue` (`writer_scheduler.c:166-178`, the enqueue at
`:177`), because its caller returns immediately afterwards. A stack is in a
stronger version of that position — step 5 puts the commit on the *target*
stack, so at the instant it runs, the stack that reached READY is executing on
no thread at all. Without the enqueue it would be READY, out of the pool, and
on no list, which is a lost wake on the ordinary arm rather than a rare one.
The alternative, switching back to it, would need a RUNNING-to-READY edge for
the stack being abandoned that §5's diagram does not have.

The switch sequence is written here once and referenced everywhere else. It
has five steps, and the middle one is not optional.

```text
1. mark SUSPENDING                     (wf__writer_begin_suspend, writer_scheduler.c:102-118)
2. register the wake                   (the waiter store into the record)
3. re-read the record's state
4. already satisfied -> cancel here and continue; never switch
5. otherwise         -> switch, then commit on the target stack
```

**It is written once because there is one record** (§5): the join runs these
five steps whether a thread or the kernel will store DONE.

Step 2 must follow step 1, because `wf__writer_scheduler_ready` aborts on any
phase but SUSPENDING or SUSPENDED (`writer_scheduler.c:121-151`, abort at
`:150`). Step 3 is what today's code has and an earlier draft of this section
dropped: the compute publisher stores DONE sequentially consistent and then
loads the waiter (`par_runtime.c:455-463`), and its reason
(`par_runtime.c:448-454`) names the other half — the waiter's own store of
`waiter` before its read of the state, which `wf__par_wait` performs at
`par_runtime.c:501-503`. Without step 3 a DONE that lands before the waiter
store is lost and the stack is parked forever on an event that already
happened. The I/O arm reads the same two fields of the same record and gets the
same answer, with the drain in the publisher's place.

Step 5's commit never precedes the switch, because until the switch happens the
parking thread is still executing on the stack a resumer would take. Commit is
called from the target stack; begin and cancel are called from the old one,
which is the point of step 4 — `wf__writer_cancel_suspend` takes SUSPENDING
back to RUNNING (`writer_scheduler.c:183-194`), a transition only a stack that
is still running can want.

Step 4 needs one widening over the ported handshake, because both sides may
see each other: the waiter reads DONE *and* the publisher loaded the waiter,
in which case the phase is already NOTIFIED when the cancel runs. The ported
arm is a strong compare-exchange from SUSPENDING that aborts on anything else
(`writer_scheduler.c:183-194`, Windows twin
`writer_scheduler_windows.c:219-229`). So the core's cancel arm accepts NOTIFIED and takes
it back to RUNNING, consuming the notification, the way
`wf__writer_commit_suspend` already accepts NOTIFIED
(`writer_scheduler.c:161-179`), and keeps the abort for every other phase.
Consuming is sound because that notification was never enqueued and the
cancelling stack has already read the condition it announces.

The registration is cleared on **every** exit from the park — cancel,
commit-to-NOTIFIED, and resume — exactly as `wf__par_wait` clears it today
(`par_runtime.c:505`). Nothing else does: `wf__par_release`
(`par_runtime.c:849-855`) returns the slot to the free list without touching
`waiter`, and `wf__par_prepare` sets the field once at pool start
(`par_runtime.c:595`). A stale pointer surviving onto a re-acquired slot is
not a leak but a fault: the next publisher either aborts, because the stack it
names is in a phase `wf__writer_scheduler_ready` refuses
(`writer_scheduler.c:150`), or enqueues it, resuming a stack for an event that
has not happened (`writer_scheduler.c:137-147`).

The lock question, stated and left to measurement. The handshake above — the
SUSPENDING and NOTIFIED phases, the cancel arm, and the re-check that catches a
DONE landing before the registration — exists to keep a lock off the park path,
and the alternative is a lock taken before the switch and released on the target
stack, which collapses the five phases to RUNNING, PARKED and READY and removes
S21's interleavings entirely, at the cost of one lock per park and one per
publish. This plan does not choose between them by argument: the lock-free
handshake stays as written, and the locked form is the fallback if §11's
enumerator finds a hole in it or cannot bound its state space. If both are
built, §12 measures the park cost of each.

The event side is one call, made once per event, by exactly one publisher: a
compute hand-out's `wf__par_execute` when it stores DONE
(`par_runtime.c:455-463`, replacing the condvar signal; the Windows twin at
`par_runtime_windows.c:601-610` wakes by address instead and has no `waiter`
field to repurpose, `par_runtime_windows.c:68-74`), and the completion
drain for an I/O record (where `wf__writer_scheduler_ready` is called today,
`completion/runtime.c:653` and `:744`). Two threads cannot both resume one READY
stack because the transition happens once, inside the CAS that wins it.

The I/O park needs no registration call. Step 2's waiter store goes into the
record the frame already holds, and the drain publishes it exactly as
`wf__par_execute` does: store DONE, load the waiter, mark that stack READY,
having found the record by its address rather than by a lookup (§7). So
`wf_completion_depend`, the slot's `publication_lock`, `dependent_registered`,
`dependent_frame`, the milestone requirement and `wf_completion_wait_to_consume`
are deleted rather than ported (§7).

The ready queue is an intrusive singly-linked list threaded through a
next-ready field in each stack's state header, with the head in the scheduler
core, guarded by one mutex. Its capacity is the stack count by construction:
a stack is linked at most once per park. Two transitions reach READY — the
event's SUSPENDED-to-READY compare-exchange, and the commit's
NOTIFIED-to-READY — and they are mutually exclusive for one park: which one a
park takes is fixed by whether the event lands before or after the commit, and
only that one runs at all. There is no loser here, and the reason is worth
stating exactly: on the NOTIFIED arm the commit's
first compare-exchange, SUSPENDING to SUSPENDED, *fails*
(`writer_scheduler.c:157-163`), and it is that failure — which writes NOTIFIED
back into `expected` — that reaches the second, NOTIFIED to READY, which
succeeds and enqueues (`:166-178`, the enqueue at `:177`). What makes it a
loserless arm is that the event's own SUSPENDED-to-READY compare-exchange never
ran at all on this path. So the property is simply that exactly one of the two transitions enqueues, and the
at-most-once bound holds over both arms rather than over one. There is no array to overflow, no count to compare
against a capacity, and no abort. The lock is deliberate and cheap here: it is taken once
per park and once per resume — not once per DONE and not once per completion,
because only a stack that parked has a waiter to publish, the way `slot->waiter`
is read today (`par_runtime.c:199-201`) — and every park is already paying a
context switch. A stack returns to the free pool only from EMPTY, and that push
is made after the switch, from the stack switched to (§5); it is linked
only between its park and its resume, so a linked header is never recycled.

The mutex stays. A lock-free push and pop is a later measurement against the
2.2 microsecond park-and-wake figure (`par_runtime.c:124-125`), not a
precondition and not a correctness question. The removal also frees a coupling
outright rather than rehousing it, because neither the ready list nor the
records have a capacity any more (§7).

A thread with nothing to run sleeps on one primitive: the completion park
that already exists (`wf_bridge_park`, `bridge.c:502-530`, io_uring on Linux
and the runtime's own epoch park elsewhere). A push bumps nothing. The epoch
is bumped only on a transition that would otherwise leave a thread asleep
beside work: a READY stack entering an empty ready queue, or a push onto the
deque of a lane whose idle bitmap is non-empty — which is exactly the test
`wf__par_publish` already performs (`par_runtime.c:788-791`); and the exit
status being posted. Routing every push through the
epoch would put a global read-modify-write back on the hand-out path the deque
exists to keep local.

The status post is a transition and not a third event source, and the
difference matters: it publishes no stack READY, it only makes every sleeper
re-check. It needs to exist because the thread that posts the status is not
always the thread that returns it. After main's stack parks, the entry thread
may be asleep at step 4 on an unbounded park (`wf_bridge_park` passes
UINT32_MAX on both arms, `bridge.c:502-530`) while a worker resumes main's
stack, runs it to its return, and posts. Nothing else in this section would
wake the entry thread, the worker would then empty main's stack and sleep too,
and `wf__floor_run`'s return (`wf_floor.c:327-328`) would never run — a hang
where today `pthread_join` (`wf_floor.c:327`) is the wake this design removes
with the `pthread_create` it joins. The post therefore goes through
`wf_completion_notify_scheduler` (`runtime.c:366-392`: the epoch bump at
`:367`, the announced-sleeper test at `:368-371`, the host wake and the
signal-or-broadcast at `:383-392`), reached publicly through
`wf_completion_notify_compute` (`runtime.c:937-947`, the call at `:946`) — the
same call this section already uses for the other transitions.

Where the entry thread *tests* for that post is the other half of the answer,
and "on every wake" is not enough by itself. The park sleeps whenever the
current epoch still equals the captured one: `wf_completion_park_if_unchanged`
compares on entry (`runtime.c:1004-1008`), announces itself and compares again
(`:1022-1031`), and then waits while the two are equal (`:1033-1046`); the
Linux arm announces under the same lock (`linux_io_uring.c:1287-1291`) and
rechecks the captured epoch and the completion queue before it sleeps
(`:1318-1331`). So a post that lands *before* the capture is folded into the
epoch the capture reads, the park compares equal, and the thread sleeps through
a status that was already posted. Only a test on the far side of the capture
sees it. The entry thread's extra loop exit (§5) is therefore tested **inside
step 4's capture-to-park window**, in exactly the position, and for exactly the
reason, that the flush and the progress pass occupy.

So the event sources are two: a compute hand-out's DONE store and a completion
drain, and both publish through one call. A third source appeared in earlier
drafts — a descriptor return, announced by the retirement ledger — and it is
gone with the ledger itself (header). A completion that happens to carry a
descriptor back is an ordinary completion and reaches the drain like any other.

Scheduler loop, in priority order:

```text
1. a READY stack        -> switch to it; the stack switched away from, if EMPTY, is
                           pushed to the pool from the stack switched to, after the switch
2. own deque newest     -> run it nested here
3. a steal              -> run it nested here
4. nothing              -> capture the wake epoch; flush and progress; re-test the
                           ready list; on the entry thread, test the exit status;
                           then park on that epoch
```

Step 4 is a window of ordered actions and not one action. The io_uring doorbell
is deferred and a park with staged SQEs is a hang, so: capture the wake epoch;
flush the deferred doorbell and run one bounded progress pass — together the
target progress primitive of §7.1 (`wf_bridge_flush_target`, `bridge.c:810-817`,
and `wf_bridge_progress`, `bridge.c:445-477`) — restarting the loop if either
made progress; re-test the ready list and take priority 1 if it is no longer
empty; on the entry thread, test the exit status and leave the loop if
it is posted (§5); only then park on the captured epoch. Everything in that
window is there for one reason: a transition that raises the epoch before the
capture cannot stop the park, because the capture reads the raised value and
the park then compares equal. Every condition that must not be slept through is
therefore tested on this side of the capture — the flush, the progress pass, the
ready list, and the status.

The ready-list re-test is inside the window and not only at priority 1, and
that is what makes §2's I/O arm and this step one sequence rather than two that
resemble each other: that arm already
re-checks the ready list before it does anything else on every wake, because a
READY stack that appeared while the thread was flushing or reaping is a switch
target. Tested only at priority 1, a stack enqueued between that test and the
capture leaves this thread asleep beside it. That state is a stranded READY
stack rather than a lost wake — every enqueuer returns to a priority-1 test
before it can sleep, so some thread picks the stack up when it next goes idle —
but it is a ready continuation held behind a sleeping thread, which is the state
§3's principle exists to forbid, so it is closed here rather than argued away.
§11 carries the matching invariant. Today's join loop has the same
epoch-then-progress-then-park order (`bridge.c:1697-1721`) but makes no explicit
flush call, because on Linux the kick happens inside
`wf_linux_io_uring_progress`, which is what that adapter's own safety argument
rests on (`linux_io_uring.c:713-717`). The core makes the flush explicit
because it is not inside the adapter that hides it.
The I/O arm of §2's fourth line is the same sequence.

## 7. Runtime changes, with anchors

The shape is one scheduler core and a thin platform layer. The core owns the
deque, the stack states, the ready queue and the park protocol, and is the
only place any of those are written down. It replaces both parallel runtimes
(`par_runtime.c`, `par_runtime_windows.c`) and both writer schedulers
(`completion/writer_scheduler.c`, `completion/writer_scheduler_windows.c`),
which are four implementations of two mechanisms today.

### 7.1 The core's boundary, and why it is a deliverable

The core reaches shared state through a closed set of primitives and through
nothing else. The set is:

1. Atomic load, atomic store, and compare-exchange, each with its stated
   memory ordering.
2. The stack switch.
3. The sleep and the wake on the one primitive.
4. The stack reservation, made once at the core's entry (§5).
5. Lock and unlock of the core's one mutex, which guards both the ready list
   and the stack free list (§5). The mutex says who may touch those words; it
   does not say when a stack may be offered, which is why the EMPTY-to-pool
   push is placed after the switch rather than merely inside the lock (§5, §6).
6. The yield, which the compute arm of §2's fourth line keeps
   (`sched_yield` at `par_runtime.c:493`, inside `:487-495`).
7. Target progress and the drain: flush the deferred doorbell, run one bounded
   target pass, drain ready completions — each drained completion storing DONE
   into its record and publishing that record's waiter READY. §6's step 4 and
   §2's I/O arm both call it. It is the one primitive that may block: with no
   helper the bounded pass executes a queued host `open` or `close` on the
   calling thread. It is also the I/O join's publisher, which is why the set is
   seven: the join registers through no call of its own, the record being the
   frame's and its waiter store primitive 1 (§6).

Everything that touches shared state through those seven is the core: §6's
stack state machine, the slot states, the deque, the free list, and the ready
list. Everything else — the adapters, the rings, the bridge's own queues, the
floor — is outside it and reaches its own state its own way.

Primitive 7 is the one that carries another unit's state, and it is in the set
rather than outside it because the core calls it on the path that matters most.
`wf_bridge_flush_target` (`bridge.c:810`) and `wf_bridge_progress`
(`bridge.c:445`) are both static today, so the bridge exports one entry point
where it has three statics. The floor-bounds answer above does not transfer to
it: the switch owns the bounds it writes, and progress writes state the core
does not own. What makes it a primitive rather than a leak is that its *effect*
on core state is statable in one line — progress reaches the drain, whose
`wf__writer_scheduler_ready` call (`runtime.c:653`) is one of §6's two event
sources — so the enumerator can model it without modelling the adapter (§11).

Its second property has to be stated with it, because a model that leaves it
out is wrong in the direction that hides defects: **primitive 7 may block.**
Under the zero-helper policy the bounded target pass runs the queued operation
on the calling thread — `wf_bridge_progress` calls
`wf_bridge_target_progress_one` when the helper count is zero
(`bridge.c:473-475`, pinned to zero on Linux with a ready ring at
`bridge.c:172-176`), that is `wf_file_adapter_progress` with a budget of one
(`bridge.c:198-201`, `file_adapter.c:1135-1149`), and `wf_file_run_work`
(`file_adapter.c:731-742`) reaches the host `openat` at `file_adapter.c:196-203`
or `close` at `:273` there. So the thread inside primitive 7 is a thread that
may be inside a host call, and §11 item 17 models it that way. That is now the
*only* place a host call is made: an operation with no kernel completion form is
executed here, or on an adapter's own execution thread where the target has no
completion source at all, and it publishes a completion like any other. It is
the runtime's engine running under the scheduler loop's rules, not a path an
emitted program can take (§2, §8).

Items 5 and 6 are here because §6 and §2 put them inside the core, and an
earlier draft's set of four left them out. They are
not reducible to item 1 for the purpose this boundary serves: a lock blocks, so
the enumerator schedules its acquisition as a point with its own wait state
rather than as an atomic; and a yield is a scheduling point that may also do
nothing, so a core that is correct only because a yield ran is a core the
enumerator rejects. Supplying the lock through the replacement header has a
second effect worth naming: the locked fallback form of §6 becomes enumerable
on the same terms as the lock-free handshake, which is what makes the choice
between them a measurement rather than a preference.

The boundary is a deliverable rather than a style preference, for two reasons.
The first is §11: a core that names its primitives in one header can be
compiled against a replacement header and driven by a controlled scheduler,
and a core that reaches a mutex or an atomic anywhere else cannot. The second
is that this is the boundary a later rewrite of the non-core runtime in
Whitefoot would keep. The specification already draws it in the same place:
"Native completion rings, readiness tables, helper mailboxes, and device queues
are target-private protocol state and are never exposed as ordinary shared
Whitefoot storage" (`spec/kernel-spec.md:1456`). The core is protocol state of
exactly that kind, so it stays target-private on either side of such a rewrite,
and the line does not have to be redrawn later.

Where shared state lives today, by lines that name an atomic
(`__atomic_` or `atomic_`) and lines that name a `pthread_mutex`:

```text
                        lines   atomic lines   mutex lines
completion/runtime.c     1787       146            76
completion/bridge.c      1925         7             0
completion/linux_io_uring.c 1555      65            25
completion/file_adapter.c 1250       38            22
par_runtime.c             985        40            17
completion/writer_scheduler.c 252     15             5
wf_floor.c                329         0             0
```

Those seven are 8,083 lines of the 23,039 lines of shipped runtime C in the
tree (29 units, excluding the test harness and the default probe). The two the
core absorbs are the last two but one: `par_runtime.c` and
`writer_scheduler.c`, 1,237 lines together, plus their Windows twins
(`par_runtime_windows.c` at 1,041 lines and `writer_scheduler_windows.c` at
288, whose synchronization is `Interlocked`, `WaitOnAddress` and
`WakeByAddress` rather than the C11 atomics, 24 and 7 lines respectively). The
rest of the count is adapter and bridge state that stays where it is, and it is
an upper bound rather than a residue: the completion core's 146 atomic and 76
mutex lines are its slot records and its publication lock, and both go with the
record pool this design deletes (§7's bullets), as the retirement ledger's share
goes with the backed `FilePermit` batch (header). `wf_floor.c` has none of
either, which is what makes it the one unit already on the far side of this
boundary.

Where the core is linked. It is not staged by the predicate
the two parallel runtimes are staged by. `compile_executable` writes the
parallel runtime only under `module_requires_parallel_runtime`
(`whitefootc.rs:328-340`), a text test for the lane-acquisition symbol
(`parallel.rs:160-163`), and Windows gates identically
(`whitefootc.rs:501-509`). But the park half the core absorbs is staged
differently today: `writer_scheduler.c`, which owns the ready queue
(`writer_scheduler.c:45-49`) and the whole handshake
(`writer_scheduler.c:102-194`), is written and compiled inside the
completion-required block (`whitefootc.rs:377`, `:396-397`), and on Windows it
is in the unconditional compile list (`whitefootc.rs:142`). A completion-only
program therefore links the bridge, the floor and the handshake and no deque —
and under this design its every I/O join parks a stack. So the merged core
needs the union of the two predicates,
`module_requires_parallel_runtime(llvm) || module_requires_completion_runtime(llvm)`
at `whitefootc.rs:328` and `:501`, which is the only staging that preserves
what each merged part has today. The core is not moved into the floor: the
dependency already runs from the parallel runtimes into the floor and not back
(`par_runtime.c:313`, `:520`, `:527`, `:665`; `par_runtime_windows.c:123-124`,
`:667`, `:807`, `:873`), and the floor's invariant — one exhaustion record
under one latch — is not the core's. `module_requires_writer_scheduler`'s
source fork (`whitefootc.rs:330-335`, `:503-507`) disappears with the writer
scheduler, leaving one core source under one predicate. The core defines
`wf__sched_entry_stack` strongly; the floor carries the weak null answer, which
is what selects the entry shape at link time (§5).

The platform layer supplies exactly four things and nothing else:

1. Thread creation with a reserved stack. POSIX is
   `pthread_attr_setstacksize` as `par_runtime.c:665` does it; Windows is
   `_beginthreadex` as `par_runtime_windows.c:807-818` does it. The stack this
   reserves is the thread's *host* stack and is not a Whitefoot stack: the
   thread switches to a pool stack before it runs anything, and its scheduler
   loop lives there (§5). The reservation
   stays large because the floor's handler and the switch itself run on it, not
   because Whitefoot frames do.
2. One wait and wake primitive. Linux is the io_uring park
   (`bridge.c:502-530`) with an `eventfd` or `msg_ring` wake; macOS is the
   epoch park already in the tree (`wf_completion_park_if_unchanged`,
   reached from `bridge.c:521`); Windows is the completion port itself,
   with a READY stack or a new offer delivered as a
   `PostQueuedCompletionStatus` packet, which is the pattern
   `windows_blocking.c` already uses at lines 319, 572, 612, 817 and 869.
   The Windows form is the one that removes a whole class of problem: a
   ready stack and an I/O completion arrive on one queue, so there is no
   second wake path to keep consistent.
3. The context switch. POSIX is a hand-written switch of callee-saved
   registers and the stack pointer, per architecture, because `swapcontext`
   carries a `sigprocmask` syscall per switch that this design cannot pay at
   completion rates. Windows is `CreateFiberEx` with a stack reservation and
   `SwitchToFiber`, plus `ConvertThreadToFiber` at thread start and
   `ConvertFiberToThread` at the end: a Windows thread's host stack has to
   become a fiber before anything can switch back to it, which is exactly the
   second entry §5 gives the entry thread. The per-switch update of the floor's
   three bounds (`wf_floor.c:72-74`, per stack under §5) is part of this
   primitive rather than a primitive of its own — it is state the switch owns
   and nothing else writes — so the replacement header §11 compiles against
   supplies it with the switch. The switch writes them from the target slot's
   own base and size in the reservation record, never from a pthread query
   (§5); the alternate signal stack is not part of
   this primitive and stays at thread start (§7's bullet below).
4. Stack allocation, which is one reservation made at the core's entry and
   carved into equal slots with a free list (section 5), not a call per stack.
   The platform supplies the reservation and the guard page; the core supplies
   the free list. It is not the worker pool's start and does not wait for one.
   Its size is its own startup setting; the one thing it takes from the pool's
   setting is a *floor*: the core reads the
   `WF_WORKERS` setting `wf__par_requested_lanes` reads
   (`par_runtime.c:634-649`) and reserves at least the thread count plus one,
   so at least two stacks in every core link, and raises a smaller setting to
   that floor (section 5). Nothing here allocates after start.

Two platform facts fall out of that split rather than being decided by it.
POSIX sets the floor's three bounds at every switch; Windows sets none,
because its floor classifies by exception code rather than by address
(section 5), and instead arms each pool fiber's own emergency stack once, at
the fiber's first frame, because a stack guarantee there is per thread or
fiber (section 7's floor bullet). And POSIX keeps the sequential world while Windows has none
(`par_runtime_windows.c:8-13`), so a defect in this design is escapable on one
platform and not the other. That difference is operational and not a fork in
the design: no join has a pool-off behaviour and none can exist (the last
bullet below), so both platforms run the one rule and a POSIX user merely has
somewhere to stand while a defect is fixed.

The anchors below are the POSIX file's, and each has a Windows twin — but in
*two* files, not one. An earlier draft promised one file for all of them, which
sent a reader looking for the design's largest Windows edit where it is not.

A **parallel-runtime** bullet's twin is in `par_runtime_windows.c`, which
`whitefootc.rs:501-510` stages into the link whenever a module hands work out:
`wf__par_join` at `943-964`, `wf__par_wait` at `614-658`, `wf__par_help_once` at
`926-941`, `wf__par_execute` at `601-610`, `wf__par_release` at `966-972`,
`wf__par_publish` at `916-924`, and `wf__par_acquire_lane` at `893-914`. None of
them is ported: the core replaces both files, so each twin is deleted rather
than edited twice, and the three differences that would otherwise have to be
carried through every bullet stop existing — the missing `waiter` field
(`par_runtime_windows.c:68-74`) and the `WakeByAddressAll` wake become the
core's one ready queue, and `wf__par_wait`'s `WaitOnAddress` sleep
(`par_runtime_windows.c:648-655`) becomes the completion port on every path that
parks, while the one path that does not park — the compute arm of §2's fourth
line — is written once in the core with no sleep in it at all. Deleting four
implementations of two mechanisms is the largest single reason to prefer this
shape over porting the design twice.

A **completion-runtime** bullet's twin is in `completion/windows_bridge.c`, and
it has its own bullet below. And the **floor** bullet's twin is in a third file,
`wf_floor_windows.c`, which is where the entry-shape
change of §5 lands on Windows; it too has its own bullet.

- `wf__par_join` (`par_runtime.c:816-847`): becomes §2's rule. "Newest entry"
  is a read of `bottom - 1` and a compare, no atomic; the pop that follows is
  the existing one.
- `wf__par_wait` (`par_runtime.c:469-510`): deleted on every path except pool
  exhaustion with a compute target, where it is the compute arm of §2's fourth
  line and the only engine a crossed compute target has. What survives is the
  work-finding loop and its empty-handed pause: the DONE test
  (`par_runtime.c:473-474`), the own-deque pop (`:478`), the steal through
  `wf__par_find` (`:480`), the nested run (`:482-485`), and the spin and yield
  rounds (`:487-495`). What is removed from those lines is the sleep and its
  registration — the lane mutex, the `waiter` store, the `pthread_cond_wait`
  loop and the clearing store (`par_runtime.c:500-507`) — because §6 replaces
  the lane signal with the ready publication, and the Windows twin's writer
  help inside the same loop (`WF_PAR_WRITER_HELP_ONCE`,
  `par_runtime_windows.c:625-628`) and its `WaitOnAddress` sleep
  (`par_runtime_windows.c:648-655`), which go with the twin file. On every
  other path the spin, yield and condvar phases are replaced by the park, as
  the bullet above and below describe.
- `wf__par_help_once` (`par_runtime.c:799-814`) and its four call sites,
  `wf_bridge_progress` (`bridge.c:476`) and `windows_bridge.c:772,793,1231`:
  deleted. Compute is run from the scheduler loop only.
- `wf_bridge_target_work_needs_this_thread` (`bridge.c:496-500`) and its three
  call sites in the join loops (`bridge.c:1708`, `:1749`, `:1794`): deleted
  with those loops, and nothing in the core replaces it. It exists today to
  stop a scheduler parking while it is the only thread that can execute a
  queued target request, which is the zero-helper configuration and nothing
  else (`bridge.c:480-495`). Dropping it is safe under the order §6 fixes, in
  two parts. The thread parks only after a pass that found nothing: step 4 and
  §2's I/O arm run the flush and the bounded target pass immediately before
  every park and restart the loop if either made progress, and with no helper
  that pass *is* the engine (`bridge.c:473-475`, `:198-201`), so a pass
  reporting nothing is a pass that found the target queue empty. And a request
  enqueued after that pass but before the park cannot be slept through either:
  an accepted target submit announces itself (`bridge.c:648`,
  `wf_completion_notify_target` at `runtime.c:949-959`), which raises the wake
  epoch at `runtime.c:367` above the one step 4 captured before the pass, so
  the park returns immediately instead of sleeping. Both parts are the
  capture-to-park window doing its job, so this is an ordering consequence
  rather than a behaviour the design gives up.
- `wf__par_execute` (`par_runtime.c:455-463`): the DONE store publishes the
  waiter's stack READY instead of signalling a lane.
- `wf__par_release` (`par_runtime.c:849-855`): `lane->free_head` is written
  with plain stores, and the field comment at `par_runtime.c:223-225` says
  free-list writes "both run on the owning thread, so it needs no atomic". A
  resumed stack reaches its release on whichever thread resumed it, whatever
  line the join took, so the home test of §4 does not reach it. It becomes a
  CAS push onto `slot->home`'s free list — and **both ends become atomic**, not
  only the push. `free_head` becomes an atomic int, and the pop in
  `wf__par_acquire_lane` becomes a compare-exchange loop: read the head with
  acquire, read that slot's `next_free`, compare-exchange the head to it, retry
  on failure. The push's CAS is a release store, which is what publishes the
  `next_free` the popper then reads; the head read is acquire. Both are
  primitive 1 of §7.1 with their orderings stated.
  Today's pop is a plain load and a plain store of the same word
  (`par_runtime.c:775-780`; the range an earlier draft cited, `:772-778`, is
  the null-lane branch above it), which loses a concurrent push outright —
  owner reads head A, a foreign release compare-exchanges B on, owner stores
  A's successor, and B is gone from that lane's supply of 64 forever
  (`par_runtime.c:592-598`), leaving a lane that refuses hand-outs
  permanently instead of transiently. The field comment at
  `par_runtime.c:223-224` — free-list writes "both run on the owning thread, so
  it needs no atomic" — is the premise this design removes, so it is superseded
  rather than quoted as support. No tag and no generation counter is needed
  over the head, and invariant I3 (§3) is why: `wf__par_acquire_lane` reads
  `wf__par_self` (`par_runtime.c:758`) and never `slot->home`, so exactly one
  thread ever pops a given free list while any thread may push it, and ABA on a
  single-consumer stack needs two pops.
- Foreign join: a resumed stack may reach `wf__par_join` on a thread that is
  not `target->home`. Then line two is skipped (the deque is not its to
  pop); the join takes line one, three or four. That is line two's home test
  (§4), not a special case.
- `wf__par_publish` (`par_runtime.c:784`) and `wf__par_acquire_lane`
  (`par_runtime.c:757`, the function v0.39 spelled `wf__par_claim`):
  unchanged. With one stack class the frame carries no class and no table is
  read; both gain a class parameter only at §5's later step.
- Slots: `WF_PAR_LANE_SLOTS = 64` (`par_runtime.c:113`) and
  `WF_PAR_FRAME_BYTES = 256` (`par_runtime.c:102`) bound outstanding hand-outs
  per lane, and the same constant is the deque's capacity — the buffer is
  `WF_PAR_LANE_SLOTS` long (`par_runtime.c:222`), the free list is threaded
  with exactly that many slots (`par_runtime.c:592-598`), and a slot is on the
  deque at most once, which is the whole reason `wf__par_push` needs no
  fullness test (`par_runtime.c:356-359`). Nothing grows. A parked stack holds
  its slots until its joins release them, so a lane whose slots are all held by
  parked stacks refuses every further hand-out (`par_runtime.c:774-778`) and
  the emitter runs those calls inline on the refused edge
  (`parallel.rs:655-660`) — a lane reverting to the sequential schedule, which
  is a schedule the program is already correct under, for the reason stated at
  `par_runtime.c:103-111`: the refused edge is the same call the granted edge
  makes. The refusal is per lane and self-clearing, since the release that
  returns a slot runs when the parked stack resumes. The coupling between the
  two capacities is invariant I4 (§3) and a `_Static_assert` in the core, on
  the model of `bridge.c:55-58`, because it is what keeps the push correct with
  no fullness test and nothing else states it. The core carries a **second**
  `_Static_assert` beside it, that the slot count is
  a power of two, because the deque masks rather than divides
  (`par_runtime.c:358`, `:382`, `:409`) and the only file asserting that today
  is `par_runtime_windows.c:61-64`, which this list deletes. Every slot on a lane's ring is
  that lane's even when stacks migrate, because acquire-to-publish is one
  straight line with no join between (`parallel.rs:478`, `:492`).
- `wf__completion_file_join` and its two siblings (`bridge.c:1692-1811`): the
  park becomes a stack park on the frame's own record, step 2's waiter store
  taking the place of the token wait (`bridge.c:1711`, `:1752`, `:1797`) (§6).
- The Windows completion twins, in `completion/windows_bridge.c` and not in
  `par_runtime_windows.c`: the two live I/O joins, `wf__completion_file_join`
  (`windows_bridge.c:1138-1153`) and `wf__completion_file_open_join`
  (`:1155-1180`), take the same conversion as their POSIX counterpart above —
  each is today a loop that drains the token, tries to take, and otherwise calls
  `wf_windows_bridge_progress(observed, INFINITE)`, and each becomes a stack
  park. The third name is not a third join: `wf__completion_file_status_join`
  (`:1182-1197`) discards its arguments and calls `abort()`, so it is a stub
  that has never run, and this design converts nothing there — it is listed only
  so a reader who greps the three POSIX joins is not left looking for a Windows
  loop that does not exist. `wf__writer_run_root` (`:1225-1242`) goes with the
  writer scheduler it pumps, and the record-capacity wait
  `wf__completion_wait_core_capacity` (`:755-780`) is deleted as dead code: with
  no record pool (§5) there is no record refusal to reach it. The three
  `wf__par_help_once` calls this list deletes on
  Windows are inside those last two and
  `wf__windows_completion_progress_for_retirement` (`:772`, `:793`, `:1231`).
- **The record's pool machinery: deleted, not answered**, because the record is
  a block of the submitting frame (§5). That is the slot array and its count
  (`contract.h:189-190`) and the slot record itself (`contract.h:145-170`) with
  its `publication_lock`, `generation`, `milestones`, `dependent_registered`,
  `dependent_frame` and 256-byte result union (`contract.h:30`); the token
  (`contract.h:57-60`) and `WF_COMPLETION_SLOT_CAPACITY`
  (`writer_scheduler.h:21`); the claim with its `WAIT_CAPACITY` answer
  (`runtime.c:234-239`) and the submit-side refusal that consumes it
  (`bridge.c:612-614`); the capacity loops in `wf_bridge_submit_file` and its
  io_uring twin with their parks (`bridge.c:640-657`, `:694-719`), the adapter
  and ring capacity answers they consume (`file_adapter.c:1109-1117`,
  `linux_io_uring.c:608-628`), and `wf_completion_notify_capacity` with its six
  callers (`bridge.c:1361`, `file_adapter.c:505`, `:557`, `:810`,
  `linux_io_uring.c:925`, `:1209`); `wf_completion_depend` (`runtime.c:784-833`)
  with its duplicate refusal and its ALREADY_READY arm; and the
  path-does-not-fit demotion with its counter (`bridge.c:1049-1059`,
  `WF_FILE_PATH_CAPACITY` at `file_adapter.h:65`), which goes because the record
  holds the loan (§5).
- **The engine's own per-operation pools go with it**, because they are sized by
  the same constant and hold the same object. `WF_BRIDGE_OPERATION_CAPACITY` is
  the deleted slot constant (`bridge.c:42-44` over `writer_scheduler.h:21`), and
  every fixed array it sizes is disposed of here rather than left standing:
  `wf_bridge_slots` (`bridge.c:61`) and `wf_bridge_linux_entries`
  (`bridge.c:103`) are **deleted**, because the frame's record *is* the adapter
  entry — the SQE is built from the record straight into the ring, where
  `entry->request` supplies it today (`linux_io_uring.c:505-526`); and
  `wf_bridge_queue` (`bridge.c:63`) is **deleted** because an adapter-thread
  target queues pending work on an intrusive list through the records instead
  (`file_adapter.c:1020`, `:1050`, `queue_capacity` at `:1109`). So
  `wf_linux_reserve_entry` (`linux_io_uring.c:425-450`) and its `WAIT_CAPACITY`
  answer (`:624-628`) go with the loops that consumed them, and the ring's depth
  is **resized to a ring parameter** — a throughput choice, not a bound on
  operations in flight — so `io_uring_setup`'s `entry_capacity`
  (`linux_io_uring.c:217-232`) stops being read from the slot constant. **The
  same census runs on Windows**, where the constant reaches the bridge through
  `WF_WINDOWS_BRIDGE_CAPACITY` (`windows_bridge.c:23`) and sizes
  `wf_windows_bridge_slots` (`:44`) and `wf_windows_bridge_entries` (`:46`),
  with two static assertions tying it to the blocking and ready capacities
  (`:29-36`); all of it is deleted with the POSIX arrays, because there the
  `OVERLAPPED` is the record's own platform block, so no entry pool is left for
  `wf_windows_reserve_entry` (`windows_iocp.c:218-241`) to draw from and its
  `WAIT_CAPACITY` answer (`:263`), the capacity loop that consumes it
  (`windows_bridge.c:872-875`) and the window twin (`:884`) go with it.
  Neither queue can then refuse an operation. A full submission queue is emptied by the
  submitting call's own `io_uring_enter`, which consumes it synchronously
  because this design uses no SQPOLL, so submit waits on a syscall and never on
  an event (§1). Completion-queue overflow is absorbed by `IORING_SETUP_CQSIZE`
  with the NODROP feature the adapter already requires and refuses a ring
  without (`linux_io_uring.c:227-228`). Windows has no submission ring to fill,
  which is true but is not why its pool is gone: the record replacing the entry
  is.
- **Every submit path ends in a published record**, the property that replaces
  the not-pending verdict now that no inline arm consumes it (§8). Either the
  kernel accepts the operation and the drain publishes its terminal, or the
  engine executes it and publishes an inline terminal — the shape the tree
  already has on the Windows immediate-failure arm, where the entry owns the
  unique terminal publication because Windows promises no packet
  (`windows_iocp.c:462-471`) — so the adapter result that is neither TARGET_OWNS
  nor WAIT_CAPACITY (`bridge.c:651-652`) publishes instead of answering 0. The
  one remaining path is not an operation outcome: a bridge that cannot
  initialize (`bridge.c:608-610`) leaves no engine to run the operation and no
  drain to publish it, so it is a trusted-computing-base failure and terminates
  deterministically where the floor does. A refused native ring is *not* that
  case and is not a submit failure: `WF_IO_NO_NATIVE_RING` routes the process to
  the POSIX adapter (`bridge.c:261`, `:280`), `wf_bridge_ensure_file` still
  succeeds, and the gate runs that configuration
  (`compiler/Makefile:293-297`, under the `completion-test` stage at `:232`). The submit path becomes: fill the frame's record, submit or
  execute, return with the record in flight or published — never with a 0 the
  caller must interpret.
- **Exactly one terminal completion per submission, and the record still there
  for it.** This replaces the identity checks the token pool carried — the
  `user_data` range and in-flight tests (`linux_io_uring.c:1015-1022`), the
  Windows range-and-stride test (`windows_iocp.c:474-486`) and the core's
  stale-generation drop (`runtime.c:428-437`) — with an impossibility rather
  than an unchecked write. The count: a resubmission publishes nothing and
  re-arms the same operation (`linux_io_uring.c:1029-1040`), so several CQEs may
  name one record while exactly one is terminal, and the bullet above keeps that
  count off zero. The lifetime: `emit_terminator` joins every outstanding
  completion before any terminator (`emitter.rs:1732-1740`), the stronger form
  of §1's placement rule, so no exit edge and therefore no frame teardown
  precedes the join that consumes the terminal. §11 carries it as an item.
- **What survives, and the one change inside it.** The adapters and the rings
  keep their own state on the far side of §7.1's boundary. The drain
  changes in one way: it finds the
  record by address instead of by token, so `linux_io_uring.c:526` submits the
  record's address as `user_data` where an entry index sits today (decoded at
  `:1010`), and the Windows entry's `OVERLAPPED`-first layout
  (`windows_iocp.h:101-109`, asserted at `windows_iocp.c:59-62`) is embedded in
  the frame's record instead of a pool entry.
- `wf_floor_windows.c`, the Windows floor: its
  `wf__floor_run` (`:143-175`) takes the same entry-shape change as
  `wf_floor.c`'s. The weak core-symbol test selects the shape at link time; the
  entry runs `wf__main_body` on a pool stack whose bottom is the scheduler loop
  rather than on the thread `_beginthreadex` makes at `:157-164`; its host stack
  is entered at most twice, with `ConvertThreadToFiber` at the first entry and
  `ConvertFiberToThread` at the second (platform item 3 above); and the
  `WaitForSingleObject(thread, INFINITE)` that is the exit wake today (`:170`)
  is replaced by the status-post epoch bump of §6, the way `pthread_join`
  (`wf_floor.c:327`) is on POSIX. The `abort()` on a failed thread creation
  (`:166`) is **not** ported away: Windows has no fallback shape and the file
  says why (`:152-156`), so where §5 concludes that the POSIX fallbacks become
  unreachable, here there is nothing to conclude.
- `wf__floor_attach_thread` splits in two, because it is two things. The
  **per-thread half** is the alternate signal
  stack: it maps 64 KiB (`wf_floor.c:238-239`) and installs it with
  `sigaltstack` (`:246`), and it stays exactly where it is called today, once at
  thread start (`par_runtime.c:527`, and `wf_floor.c:279` for the first thread).
  Putting that on a switch would be an `mmap` per switch and a leaked mapping
  per switch, which the header's second ruling forbids. The **per-stack half**
  is the three bounds, which move into the switch as platform item 3 already
  says, written from the reservation record rather than read from pthread (§5).
- `writer_scheduler.c`, `writer_scheduler_windows.c`, `emitter/stackless.rs`,
  `tests/stackless.rs`: retired. Their handshake moves to the stack park;
  the emitter emits the plain synchronous ABI for every function.
  `WF_WRITER_READY_CAPACITY` (`writer_scheduler.h:22`) and the assertion tying
  it to the slot count (`bridge.c:55-58`) go with the file, as
  `WF_COMPLETION_SLOT_CAPACITY` (`:21`) goes with the record pool above.
- The cancel arm of the core's park protocol accepts NOTIFIED and returns it to
  RUNNING, consuming the notification, on the model of the commit arm
  (`writer_scheduler.c:161-179`); every other phase still aborts. This is a
  property of the core, not an edit: `writer_scheduler.c` and its Windows twin
  are retired above, so the ported arm is written once with the
  wider condition rather than changed in place. The ported version
  (`writer_scheduler.c:183-194`, `writer_scheduler_windows.c:219-229`) is a
  strong compare-exchange from SUSPENDING that aborts on anything else, and
  that is correct only because cancel has one caller today — the emitter's
  inline path when a submit returned not-pending (`stackless.rs:761`), where
  the operation never reached a completion source and no publisher can exist.
  §6's step 4 reaches cancel after a live registration, so the precondition is
  gone. The core also clears the waiter registration on every park exit (§6).
- There is no pool-off behaviour of any join, and none can exist. The I/O
  joins are runtime functions (`bridge.c:1692`, `1727`, `1768`);
  `wf__par_pool_active` is called only from the emitted bootstrap
  (`system.rs:3145`, `system.rs:3280`) and no source under `completion/`
  references it; the only parallel symbol `completion/` names is
  `wf__par_help_once`, whose weak answer at `bridge.c:119-123` exists to keep
  completion-only programs independent of that runtime, and which this list
  deletes. So §2's rule is unconditional and the sequential clone world
  (`parallel.rs:35`, "Two worlds, selected once") changes nothing about it:
  with no workers a miss still parks its stack and the one thread sleeps on the
  primitive it sleeps on today. Windows has no such world in any case
  (`par_runtime_windows.c:8-13`, `:749-760`).

## 8. Emitter changes

- Join order of compute members reversed, through the one `compute_join_order`
  function that `emit_overlap_joins`, `overlap_join_tail` and
  `block_exit_label` all consume. Completion members are not reordered at all:
  they keep their publish-queue positions, and the one a later step names keeps
  being moved to that step by `emit_completion_dependencies` as it is today, so
  this bullet is the whole emitter change (section 4). `compute_join_order` is
  consumed by `emit_overlap_joins` (`parallel.rs:624-670`), `overlap_join_tail`
  (`emitter.rs:2234-2239`) and `block_exit_label`; the member list it orders is
  `IrOverlap::handed_out` in the lowering module (`lowering.rs:1086-1091`).
  Two more emitter changes follow the owner's fourth ruling (header), and the
  bullet above is otherwise the whole of it.
- **The frame carries the record where it carries the token today.** The emitter
  already reserves an entry-block element per outstanding operation — the token,
  the result slot, the raw value and error, an open's outcome, a cursor's
  position, an open's staged path — through `completion_entry_slot`
  (`completion.rs:313-335`, the reason at `:287-296`), the token itself as
  `[2 x i64]` at `:665`, `:829` and `:1135`. That set collapses to one opaque
  block of the runtime's stated size and alignment, whose address is what submit
  is given and what the join reads (§5, §6); the emitted module holds one opaque
  pointer and "never learns this layout" (`par_runtime.c:185-207`). The per-site
  outstanding-count rule is unchanged; only what one element is changes.
- **How the size and alignment reach the emitter**, which nothing states today
  and which a wrong number turns into a kernel write past the reservation. They
  are one ABI constant of the completion contract, stated in the completion
  header beside the other public capacities and used by the emitter to reserve
  the block; the C side asserts its own record layout against that constant with
  a `_Static_assert` on the model of `bridge.c:55-58`, so a layout that outgrows
  or out-aligns the emitter's number fails the build instead of the program, and
  §11 carries the matching item. Two things it is *not*. The compute slot is not
  the precedent: there the emitted call passes the *program's* frame size into
  the runtime and is refused above `WF_PAR_FRAME_BYTES` (`parallel.rs:472-479`,
  `par_runtime.c:761-762`), which crosses the other way. And the number cannot
  come from the stack ledger, which runs clang on LLVM the emitter has already
  produced (`whitefootc.rs:282-310`) — §5's own ordering fact. The ledger prices
  the block (§5, §12); the header states its size.
- **One lowering for every I/O operation: submit, then join.** The direct family
  leaves emitted code, and it has two anchors rather than one. The qualification
  dispatch names five — close, open_at, status, pread and write, each spelled
  `wf__completion_file_*_direct` (`qualification.rs:233-291`) — while the sixth,
  `wf__completion_directory_next_direct`, is declared and called by the
  directory-list lowering (`emitter/system.rs:542`, `:2735`) and defined at
  `windows_runtime.c:1617`; an earlier draft counted six in the first range
  alone. With them go the inline arm (`completion.rs:764-783`), its three
  callers (`:764`, `:924`, `:1183`), and the eligibility branch that selects it
  (`:749`). An operation with no kernel completion form is executed inside the
  runtime's engine and publishes a completion like any other (§7.1), so the
  emitter needs no second arm for it.
- The Windows verdict fork goes with it, as dead code rather than as a policy
  change: `emit_completion_submit_verdict`'s Windows arm
  (`completion.rs:438-526`), its `wf__completion_wait_core_capacity` declaration
  (`:80`) and the runtime function it names (`windows_bridge.c:755-780`), the
  six label helpers the fork alone uses (`:1353-1383`),
  `emit_windows_completion_materialization` (`:531`), and the same block in
  `stackless.rs:473` with that file. `completion/bridge.h:28` is deleted with
  them; the verdict enum (`bridge.h:18-22`) stays, because the runtime type is
  unchanged, and its comment (`bridge.h:11-22`), which today describes the retry
  policy, is rewritten to say that a capacity verdict is unreachable once the
  record is the frame's (§5). In `tests/completion.rs`, only assertions of the
  retry shape are retired — the declaration entry at `:478`, and those of
  `windows_core_pressure_materializes_the_oldest_owned_result_and_retries`
  (fn at `:496`) — with the honest note this repository requires: the rule they
  test was changed by a recorded owner ruling, not to make a check pass. The
  assertion at `:550`, that core pressure must never become direct execution,
  **stays**: it is what this design now guarantees everywhere.
- No stack class. With one class the emitter passes nothing extra at a
  hand-out; the link-time class table is section 5's later step and its
  producer is the ledger pass in `whitefootc.rs:282-310`, not the emitter.
- `StacklessPlan` and the writer-frame ABI removed; every `may-suspend`
  function keeps the synchronous ABI it has in the sequential world.
- Nothing about the permission judgment, the window, [PAR-2], or [PAR-3]
  changes. Which iterations become jobs is decided exactly as today.
- The staged [PAR-3] driver keeps its one-function lowering, and its waits
  become parks of the one stack that runs it. `IrCompletionPipeline` holds
  its entry, carrying blocks, feeder and drain as block ids in one function
  (`compiler/src/lowering.rs:1211-1227`, `lowering.rs:1290-1309`) and its
  per-iteration storage as an entry-block reservation (`lowering.rs:1324`),
  and `LOOP-PIPELINE.md` §3.4 states that "every stage runs on the owner
  lane, there is no stack switch, no cross-thread resume". So an iteration
  never gets its own stack, no stack ever waits on another stack, and
  [PAR-3]'s ordering clauses (`spec/kernel-spec.md:2045-2047,2053`) hold for
  the reason they hold today: they are one function's control flow on one
  stack. Park on miss changes only which thread that stack sits on, which
  [PAR-3] states is not an observation (`spec/kernel-spec.md:2058`). This
  holds while the staged lowering keeps that shape; outlining iterations onto
  their own stacks would reopen `spec/kernel-spec.md:2046-2048` and `2055`, so
  it is recorded here as a constraint on future work.
  What changes for the pipeline is exactly one call: its driver blocks through
  `wf__completion_file_join`, emitted at `completion.rs:564`, `584`, `1292` and
  `1312`, and that wait becomes a park of the driver's own stack.
  `LOOP-PIPELINE.md` §3.4 states the opposite and is superseded in exactly two
  places, both named in the list at the top of this file. The first is that the
  driver blocks on the oldest slot's join rather than suspending, because "with
  K slots outstanding the owner lane has nothing else to run"
  (`LOOP-PIPELINE.md:822-825`). The second is the stage cutting and spill to the
  slot record that was to carry a suspension without a switch
  (`LOOP-PIPELINE.md:827-834`): a stack that holds its own live values needs
  neither, so the stages disappear rather than being implemented. The reversal
  is a generalization rather than a change of judgment: §3.4's premise holds for
  a lane running one pipeline alone and not for a lane that also holds a deque
  and other stacks, and §3.4 priced suspension as the stackless transform
  (`LOOP-PIPELINE.md:815-822`), which this design removes rather than pays.
  Its emitter changes 1 and 3 (`LOOP-PIPELINE.md:797-803`, `:808-811`) stand
  unchanged; change 2 (`:804-807`) keeps its first and last steps and loses
  "run that slot's next stage". Its instruction against re-introducing
  `stackless.rs` (`LOOP-PIPELINE.md:832-835`) is carried out here rather than
  reversed. Nothing in working code changes: the shipped lowering has no stage
  concept at all (`lowering.rs:1210-1234`).

## 9. Specification

This section was read at `main` = `30602914`, where the active specification is
v0.41 (`spec/kernel-spec.md:1-6`) and its section 13 begins at
`spec/kernel-spec.md:1966`. The header names the revision for the runtime
citations and this section names it again, because the specification is the one
subject here that is a live document: v0.41 respelled the comparison operators
and states that "No rule's semantics changes ... the accepted-program set is
unchanged up to respelling" (`spec/kernel-spec.md:6`), so every clause quoted
below reads as it did at v0.40 and only its line number moved. This plan is
also implemented after a further specification change (roadmap `BOUND-1`,
header), which is the second reason to date the reading rather than assert a
version.

[PAR-1] already states that the number of workers, the host thread, the
schedule, and whether an overlap was performed are not observations
(`spec/kernel-spec.md:1993`), and that a `may-suspend` member "selects
completion lowering when the implementation actualizes the window"
(`spec/kernel-spec.md:1983`). Park on miss is a completion lowering. Loan
retention until `loan-released(path)` is unchanged: a parked stack holds its
borrows exactly as a blocked thread does today (`spec/kernel-spec.md:1983`).

**DONE is the `loan-released` point**, which has to be said because §7 deletes
the milestones. The four are already one product today — `RESULT_READY`,
`PAYLOAD_RELEASED`, `RESOURCE_RELEASED` and `TERMINAL` as
`WF_COMPLETION_OWNERSHIP_COMPLETE` (`contract.h:35-44`) — and DONE *is* that
product: a drain stores DONE only when the target is finished with every
referent of the call, so none stores DONE while the target still holds the
buffer or the path. That matters more, not less, after §5 spends [EFF-5]'s
permission to drop the path copy, because the adapter now retains the writer's
referent where it retained a copy. The record in a frame is not the exposure the
same sentence forbids, either: what it forbids is target-private protocol state
"exposed as ordinary shared Whitefoot storage" (`spec/kernel-spec.md:1456`), and
the record is an opaque block no source form can name, address, read, or alias,
holding no Whitefoot place. The frame is where its bytes live, not a way to
observe them.

No fiber unwinding support is needed, and the specification has supported that
more strongly since v0.40 than v0.39 did. The `claim` statement and its runtime
trap are gone. v0.40 is what retired them, and its META-5 delta now reads from
the archive (`spec/kernel-spec-v0.40.md:6`): it retires TRAP-1, DIAG-3 and the CLM rules,
removes the `claim_stmt` production and the `claim` atom, and records
"runtime-trap families +0/-1 (0 remain)". v0.41's own delta
(`spec/kernel-spec.md:6`) records "+0/-0 (0 remain)", which is the same state
one version on. [EFF-4] states that accepted source
has no writer-reachable abort effect, exception, unwinding edge, or hidden
runtime proof fallback (`spec/kernel-spec.md:1446`), [ERR-1] makes recoverable
errors values dispatched by `match` (`spec/kernel-spec.md:1464`) and adds, in a
sentence of its own, "No exceptions, no unwinding, no panic values"
(`spec/kernel-spec.md:1465`) — that second line is the one this design leans
on, and it is why it is cited separately from the head of the rule — and
[PAR-1] now
states directly that the overlap identity holds in every source execution
because accepted source contains no writer-reachable proof-failure branch
(`spec/kernel-spec.md:1989`) and that no completion or fast path pays for a
runtime proof fallback (`spec/kernel-spec.md:1992`). A parked stack therefore
has no language-level edge that could unwind it, for every accepted program
rather than for one trap family.

What remains is the resource floor, which is not a language rule: a stack
overflow writes one record under a single process-wide latch
(`wf_floor.c:121-132`) and aborts, and parked stacks are neither unwound nor
cleaned up. Exhaustion of the resources an implementation spends on
overlapping stays a resource condition under [SCOPE-3]
(`spec/kernel-spec.md:1995`).

[PAR-3]'s ordering clauses (`spec/kernel-spec.md:2045-2047,2053`) survive
unchanged, for the reason section 8 gives: the staged driver is one function
on one stack, so index order is its own control flow and is never delegated
to the scheduler. [PAR-3] states that the identity of the host thread that
executes a segment is not an observation (`spec/kernel-spec.md:2058`), which
is the only thing park on miss changes about it.

## 10. Schedules to walk

Each is a concrete schedule to walk step by step. A reviewer checks whether
any of them can lose a wake, run a stack twice, bury a ready frame, or
corrupt a deque.

- S1. One thread, four independent iterations of the four-stage chain. Every
  iteration parks at its first join; the stage-two depth must reach four.
- S2. A publishes J1, J2 and runs its inline member three frames deep to a
  read; the read's join misses. A's stack parks with J1, J2 still on the
  deque; the same thread runs them on the new stack; A resumes and joins
  them done.
- S3. A compute child stolen and unfinished at its join. Line three, not a
  help. The thief's DONE store marks A READY.
- S4. A parked stack resumed on a foreign thread with an unjoined hand-out
  still on the home deque. The foreign join must not pop; the home lane or a
  thief runs it; DONE marks the stack READY.
- S5. A same-group sibling run inline above a join parks on its own I/O. The
  whole stack parks; the frame below needs the sibling anyway.
- S6. Every thread parked, nothing runnable, operations in flight. Every
  thread must sleep on the one primitive, and one completion must wake at
  least one thread and never none; a woken thread that finds nothing returns
  to sleep. (Not "exactly one": the primitive already broadcasts,
  `runtime.c:383-392`, the broadcast itself at `:391`, and the bar that matters
  is no lost wake.)
- S7. The completion arrives between begin park and commit. The commit takes
  NOTIFIED to READY and enqueues the stack exactly once; exactly one thread
  dequeues it, and the parking thread does not switch back to it.
- S8. A resource record written while stacks are parked: one thread's stack
  overflows while others hold parked stacks. One record under the single
  latch (`wf_floor.c:121-132`), abort, no unwinding, no cleanup, no second
  record. The overflow to walk is a *pool* stack's, because the record is
  written only after the handler classifies the fault as a guard hit against
  that stack's own bounds (`wf_floor.c:155-158`); a pool stack whose bounds
  still came from a pthread query would fall through to the default disposition
  (`:160-165`) and this schedule would assert a record that is never written
  (§5).
- S9. A lane's whole slot set held by parked stacks. Later hand-outs on that
  lane refuse (`par_runtime.c:774-778`) and run inline
  (`parallel.rs:655-660`); the deque cannot overflow because it is sized by
  the same constant; the stacks resume and release; hand-outs are granted
  again. Nothing deadlocks; the cap is a resource condition.
- S10a. POSIX, pool off (`WF_WORKERS=0` or `1`). This asserts an equality, not
  a second mechanism: the one thread parks its stack, finds nothing at
  priorities 1 to 3, and sleeps on the same primitive at `bridge.c:502-530` it
  sleeps on today, so observable behaviour equals today's. Check first that the
  stacks exist to park on: `wf__par_requested_lanes` answers 0 here
  (`par_runtime.c:642-643`) and `wf__par_start` returns without preparing a lane
  (`par_runtime.c:655-657`), so the park depends on the stack reservation being
  the core's entry rather than the worker pool's start (§5), and on at least two
  stacks existing with no worker — which is §5's floor at its smallest, the one
  thread plus one, and this schedule is what that floor is for.
- S10b. Windows. There is no pool-off state
  (`par_runtime_windows.c:974-977`) and no sequential world to fall back to
  (`par_runtime_windows.c:8-13`), so every defect in this design is reached
  with no escape. There is no question left to answer: Windows takes the same
  one path POSIX does, because no join has a pool-off behaviour (§7).
- S11. Retired for the first implementation, and the reason is technical
  rather than convenience: with one stack class (§5) there is no class
  mismatch, no extra switch and nothing to walk. Reinstate it verbatim when
  §5's later step introduces classes.
- S12. Nested groups: a member of an inner group publishes its own group.
  Reverse join order keeps every target newest-or-stolen at each level.
- S13. An iteration exits by `propagate` after having parked and resumed.
  Cleanup on the normal exit edge runs on whatever thread resumed it.
- S14. One stack parks, resumes, and parks again on a later join. Two
  parks, two resumes, no state carried between them.
- S15. Two threads reach priority 1 with one READY stack on the list. This is a *consumer* schedule, and it is stated
  against the mechanism §6 actually has: the core's one mutex serializes the two
  unlinks, one thread takes the stack, the other finds the list empty and falls
  to priority 2, and no stack is resumed twice. An earlier draft wrote it as two
  threads racing to enqueue one stack inside "the winning CAS", which is the
  pre-decision-23 array queue and also a publisher-side race §6 shows cannot
  arise; the CAS that wins is the phase transition, and it is item 10's subject,
  not this one's.
- S16. A stack parks with its own entries on the deque; thieves take some;
  it resumes elsewhere; its joins find each target stolen or done.
- S17. The entry stack parks on main's first I/O; a worker resumes it and runs
  main to its return; the entry thread is meanwhile in step 4. Walk the wake
  rather than assuming it, and walk it in the order the two threads touch the
  epoch, because that order is the whole of the answer. The worker posts
  through `wf_completion_notify_compute` (`runtime.c:937-947`, the call at
  `:946`), which raises the epoch (`runtime.c:367`) and only then reads
  `parked_schedulers` (`:368-371`). That leaves two cases and both end on the
  same line. If the entry thread has already captured its epoch and announced
  itself, the post finds an announced sleeper and signals or broadcasts it
  (`:383-392`); it wakes, restarts the loop, finds nothing at priorities 1 to 3,
  captures again, and tests the status inside that new capture-to-park window.
  If the post lands before the entry thread's capture, no sleeper is announced
  and none needs to be: the capture reads the already-raised epoch, and the
  status test that follows it in the same window sees the post without the
  thread ever sleeping. Either way the entry thread finds the status posted,
  switches to its host stack, pushes the pool stack it just left back to the
  free list, and executes `return call.status` (`wf_floor.c:328`, the last line
  of `wf__floor_run`). Walk the third case as well — the worker empties main's
  stack and sleeps before the entry thread wakes — which is the second case with
  a longer gap: the post is already in the epoch, so the entry thread's own
  capture-and-test still reaches the return.
  Add the completion-only twin: a program that does I/O and hands nothing out,
  checking that `wf__sched_entry_stack` resolves strongly, that the entry takes
  its stack from the reservation the core's entry makes, and that no lane is
  ever started because the module never names `wf__par_acquire_lane` — which is
  the link shape that made the reservation a separate start (§5). Add the
  no-core twin: a program with neither
  predicate runs today's `wf__floor_run` unchanged and never parks.

- S18. A group of N reads holds N records in one frame. The emitter walks the
  publish queue once and joins each completion member where it sits
  (`parallel.rs:631-637`), so all N are outstanding together, each owning its own
  block, and a nested group adds its own N one frame up. Nothing is claimed, so
  what this walks is that every record's address stays valid to its join, which
  the join's placement before any read and any exit edge gives
  (`parallel.rs:619-623`).
- S19. Two stacks of one thread park on two different events that complete
  in the opposite order. Each resumes independently.
- S20. A `may-suspend` inline member that never actually misses (the
  completion is already there at the join). Zero switches; the fast path is
  a take.
- S21. The event lands before the registration, which S7 does not walk (S7 is
  the arm where it lands between begin and commit). Compute: the thief stores
  DONE and loads `waiter` while it is still NULL, so it publishes nothing. Walk
  both sub-cases of §6's step 3 — the parking stack re-reads DONE and the
  publisher loaded NULL, so it cancels and continues; and the parking stack
  re-reads DONE *and* the publisher loaded the waiter, so the phase is already
  NOTIFIED and the cancel must consume it rather than abort. The stack must run
  exactly once and never be enqueued. Then release the slot and acquire it again
  on the same lane, checking that the waiter field the cancel cleared
  (§6) leaves the next publisher nothing stale to name. The I/O twin is the
  same walk with the drain in the thief's place, on the same two fields of the
  same kind of record (§5).
- S22. A crossed pair on the exhausted path: two stacks resumed on foreign
  threads, each joining a hand-out on the other's home lane, with no free stack
  and nothing READY. Walked to the end under §2's fourth line: both targets are
  compute hand-outs, so both threads take the compute arm. Thread T1 pops its
  own deque (`par_runtime.c:478`) and finds nothing of its own, then steals
  (`par_runtime.c:480`); `wf__par_find` walks every other lane
  (`par_runtime.c:420-445`) and reaches T2's home lane, whose oldest end holds
  the entry T1 is waiting for; `wf__par_steal` takes it (`par_runtime.c:402-415`)
  and T1 runs it nested above its own join (`par_runtime.c:482-485`). T1's
  target reaches DONE and T1's join returns. Symmetrically for T2, and if both
  steal at once each takes the other's entry, which is progress on both. No
  foreign deque is popped from the owner end, so I2 holds: `wf__par_steal` takes
  from `top` with a compare-exchange, which is the thief side Chase-Lev permits
  from any thread. The one thing the arm may not do here is what
  `wf__par_join` does — pop `target->home` (`par_runtime.c:818-819`) — which on
  this schedule is an owner-side pop of a foreign deque.
- S23. A nested entry on the compute arm that itself misses on I/O with no
  stack: on the exhausted path, thread T runs a stolen entry above its compute
  join, and that entry submits a read and joins it. The read is not on any
  deque and no stack is free, so the inner join takes the I/O arm: T flushes
  (`bridge.c:810-817`), progresses (`bridge.c:445-477`), drains, and sleeps
  (`bridge.c:502-530`) with nothing run above the read's join. The stack now
  holds two waiting frames, the outer compute join and the inner I/O join. Only
  §3's one exception is in play: what sits above the compute join is compute
  work of the population that join is waiting on, and nothing at all sits above
  the I/O join. On the wake T re-checks the ready list
  first: a READY stack found there is switched to, with the read as this stack's
  wake, and the outer compute join is then a parked frame like any other.

## 11. Test obligations the implementation is gated on

Recorded with the owner's ruling of 2026-09-03: this state machine is large
enough that its tests are part of the batch that implements it, not a follow-up
to it. The list below is derived from §5 and §6's own state diagram and from
§10's schedules, and carries nothing the state machine does not have.

**The gate is not "these tests pass". The gate is that the core is enumerable
and the enumeration passes.** Concretely: the protocol core compiles against a
replacement primitives header — the seven primitives of §7.1 and nothing
else — inside a cargo test; a controlled scheduler in
that test enumerates *every* interleaving of those primitive operations for
bounded configurations; and every item below is checked as an invariant at
every step of every interleaving, not once at the end of a run. A hand-written
test that walks one interleaving is evidence that the interleaving behaves, and
it is not the gate.

The configurations are derived, not chosen. Under
§5's rule that every thread runs its loop on a pool stack and a stack leaves a
thread only at a switch, T running threads hold T stacks, so free stacks are
S − T. Line three needs a free stack or a READY stack, and a READY stack exists
only after some park, so a first park needs S ≥ T + 1; P stacks parked at once
needs S ≥ T + P. Two of the schedules need two stacks parked while two threads
run — S22's crossed pair and S23's nested miss — which is what fixes the largest
configuration at (T=2, S=4). S19 and group A item 4 do not: S19's two parks are
one thread's, which (T=1, S=3) reaches, and item 4 only needs the resuming
thread to differ from the parking one. Group C item 11 needs the one-thread
floor itself, which is (T=1, S=2): there the first park empties the pool, so the
fourth line is reached after one park rather than after two, and that is the
configuration a host at the floor actually runs. So
the sweep is
**(T=1, S=2), (T=1, S=3), (T=2, S=3) and (T=2, S=4)**,
each with two slots and with one and then two lanes. Two is not an arbitrary
reduction: I4's second half requires a power-of-two slot count, so the
enumerator's smaller constant is constrained to 2 or 4 and never 3
(§3). T and S are free of each
other only above one bound, and §5 is why: the stack count is its own startup
setting, but its floor is the thread count plus one, so **S ≥ T + 1 holds in
every link a host produces** while the count above that floor is not tied to
`WF_WORKERS` at all. The enumerator therefore sweeps
no configuration with S ≤ T, and not merely because such a run would be dull.
An earlier draft said two
threads and two stacks, which is S = T: free stacks are zero at every step, no
park is reachable at all, and group A items 2 to 5, 7, 9 and 10 and schedules
S3, S4, S7, S13 to S16, S19 and S21 would never be entered. That configuration
is now excluded twice over, by this derivation and by the runtime's own floor.
Group C item 11 reaches the fourth line the way a host reaches it instead: at
the floor, S = T + 1, the free list is empty after the first park, and the next
miss with nothing READY takes the fourth line.

The reason is plain and worth saying rather than implying: ordinary tests
sample interleavings. They cannot cover this state machine, because the
schedules that break it are the rare ones — a DONE that lands in a two-
instruction window, a READY stack that appears between a flush and a sleep —
and a sampling test that passes tells you the sample missed them, not that they
are absent. Enumeration is the only form of coverage this machine has. A batch
that implements this design and leaves an item below outside the enumeration,
or leaves the enumeration failing, is not finished.

The items are therefore the list of properties to check and the configurations
to reach, not a list of separate test programs.

The starting position is worse than it looks, and that is the reason for the
list. The handshake §6 ports has no behavioural test in the tree today: the
only tests that name `wf__writer_begin_suspend` assert on emitted IR text
(`compiler/src/backend/tests/stackless.rs:99`, `:119`, `:138`), and nothing
drives its phases. Every item in group A is therefore new work rather than a
port of an existing case.

Three harness kinds are named once here and referred to by letter.

- **Kind A, single-thread deterministic.** One process, one thread, no worker
  pool — the shape `completion.rs:1303` and `:1396` already use
  (`WF_WORKERS=0`), or a C harness case that creates no thread. Exists today.
- **Kind B, real threads.** Two homes exist. The compiler-independent C
  harness `compiler/src/backend/completion/harness.c` links the completion
  runtime directly and is built and run by `compiler/Makefile:235-247` at three
  helper counts, with an address-sanitizer arm at `:369-370` and a
  thread-sanitizer arm at `:408-414`; its device for making two threads race is
  the release gate at `harness.c:550-563`, used by
  `test_exactly_one_terminal_under_race` (`harness.c:565`). The other is a
  compiled program run over a sweep of `WF_WORKERS`, as
  `exhaustion.rs:326-341` and `loop_split.rs:649-663` do. Exists today.
- **Kind C, forced ordering.** A run in which one thread is held at a named
  point until another has passed a second named point, so a specific
  interleaving is produced rather than hoped for. Under the gate above this is
  not a fourth harness but one output of the enumerator: an interleaving the
  enumerator can name is one it can replay, and "Kind C" below means "the
  enumerator must reach this ordering", not "write a bespoke race". *Neither
  the enumerator nor a forced-ordering harness exists in the tree today*; the
  closest is Kind B's release gate, which starts threads together and makes a
  race likely without forcing an order. The materials are here: the replacement
  header is the seam, the tree's own seam pattern is the weak symbol, used
  three times (`bridge.c:121`, `runtime.c:15`, `writer_scheduler.c:55-57`), and
  the observer-unit pattern of `link_counting_grants`
  (`compiler/src/backend/tests/parallel.rs:1692-1740`, the test file and not
    the emitter's `parallel.rs`) is how a test reads a runtime counter back at
    exit.

**Group A — every state and every transition of §5 and §6.** Kind A drives one
side of each; where an event has to arrive from elsewhere the item says so.
None of these check I1 to I4, which are properties of what runs above a join;
each checks the park protocol's own property, that a park produces exactly one
resume and loses no wake.

1. RUNNING to SUSPENDING at begin, and step 2's registration landing after it
   rather than before. Kind A, with the wrong order asserted to be refused:
   the core keeps the abort the ported arm has for any phase but SUSPENDING or
   SUSPENDED (`writer_scheduler.c:121-151`, abort at `:150`).
2. SUSPENDING to SUSPENDED, committed on the target stack after the switch and
   never before it. Kind A with two stacks; the assertion is that the commit is
   observed after the switch, which is what makes the stack safe to take.
3. SUSPENDED to READY with an enqueue. One transition, not two that resemble
   each other: the compute publisher and the drain store DONE and load the
   waiter of the same record (§5, §6). Kind B, one per side.
4. READY to RUNNING when a thread switches to it, including the case where the
   resuming thread is not the parking thread.  Kind B.
5. SUSPENDING to NOTIFIED and NOTIFIED to READY at commit, with the commit
   enqueueing the stack exactly once. Kind C; this
   is S7, and Kind B cannot place the event inside the window reliably.
6. SUSPENDING to RUNNING by cancel, when step 3 finds the record already DONE.
   Kind A, with the completion drained, or the compute target finished, before
   the join runs, so the re-read is deterministic.
7. NOTIFIED to RUNNING by cancel, consuming the notification. Kind C; this is
   the arm the ported code aborts on (`writer_scheduler.c:183-194`) and the one
   widening §7 records, so a test that never reaches it leaves the widening
   unexercised.
8. RUNNING to EMPTY to the pool: a scheduler-loop stack that empties returns to
   the free list and is handed out again. Kind A, asserting the same stack is
   reused rather than a second one taken.
9. The waiter registration cleared on every park exit — cancel,
   commit-to-NOTIFIED, and resume — followed by a release of the slot and a
   re-acquire on the same lane, asserting the next publisher finds nothing
   stale to name. Kind A for the sequence, Kind B for the re-acquire; this
   also checks I3, because the release runs on whichever thread resumed the
   stack while the acquire runs on the lane's own. The enumerator must model
   the free-list pop as its constituent primitive operations — the head read,
   the successor read, and the swing — and not as one atomic step
   modelling it as one step is exactly what hides
   the lost update the plain pop had, so this item also checks that no push is
   ever dropped.
10. A stack is linked into the ready queue at most once: **exactly one of the
    two transitions into READY enqueues**, with both arms reached. That is §6's property, and it is stated as a
    property rather than as a race on purpose. There is exactly one publisher
    per record: the kernel delivers a completion once, and the drain that
    receives it stores DONE and loads the waiter once, which is
    `wf__par_execute`'s own pair (`par_runtime.c:455-463`) — so an item that asked the
    enumerator to reach two publishers racing for one stack would be asking it
    to reach a state the design forbids, and the way an unreachable gate item
    gets satisfied is by adding the path that reaches it. Kind C for the
    ordering; item 5 keeps the NOTIFIED arm, and S15 is now the consumer
    schedule.

**Group B — every schedule of §10 as an executable test, named by its
S-number.** S11 is retired for the first implementation (§10 says why) and is
the only S-number with no test. The rest are one test each.

```text
S1   Kind A                     I1        S13  Kind B                     —
S2   Kind A, pool >= 2          I1, I2    S14  Kind A                     —
S3   Kind B                     I1        S15  Kind B, two consumers      —
S4   Kind B                     I2        S16  Kind B                     I2
S5   Kind A                     I1        S17  Kind B + link checks       —
S6   Kind B                     —         S18  Kind A, N in one frame     —
S7   Kind C (group A item 5)    —         S19  Kind B                     —
S8   Kind B                     —         S20  Kind A                     —
S9   Kind B, slots exhausted    I4        S21  Kind C                     —
S10a Kind A                     —         S22  Kind B, pool exhausted     I1, I2
S10b Kind B, on Windows         —         S23  Kind B, pool exhausted     I1
S12  Kind A                     I1
```

A dash means the schedule checks no invariant of I1 to I4 and checks something
else the section names: S6 the wake bar, S8 the floor's single record, S10a the
equality with today's behaviour, S13 cleanup on the resuming thread, S15 that
two consumers of one READY stack are serialized by the core's mutex and no stack
is resumed twice, S18 the frame's record set, S20 the
no-miss fast path, and S7, S14, S17, S19, S21 the park protocol.

**Group C — the five the owner named, because a single ordinary run does not
reach them, and nine more the review rounds added (items 16 to 24).**

11. The fourth line's two arms, each run at §5's floor — two stacks with one
    thread, three with two — so exhaustion is reached deterministically rather
    than under load: park the single free stack, then miss again with nothing
    READY. Kind A for the one-thread runs and Kind B for the
    two-thread runs; the stack count is the startup setting of §5, and a pool
    at the floor is what makes that second miss take the fourth line. The I/O arm asserts
    that nothing Whitefoot-level runs above the join while the thread sleeps
    (I1); the compute arm asserts that the target reaches DONE through the
    thread's own deque and then a steal, and that no owner-side pop of a foreign
    deque occurs (I2).
12. The ready-list re-check inside the I/O arm's sleep: a READY stack is made
    to appear while the thread is between its flush and its sleep, and the
    assertion is that the thread parks this stack with its I/O as the wake and
    switches, rather than sleeping past it. Kind C, because the appearance has
    to land inside that window; the observation that a sleeper exists is
    already available in the harness
    (`wf_completion_parked_scheduler_count`, `harness.c:733`). Checks I1: a
    READY stack left unswitched is a ready continuation buried behind a sleep.
13. S21's DONE-before-the-waiter-store interleavings, both sub-cases, under a
    harness that forces each ordering rather than hoping for it. Kind C, and
    this is the item that makes the enumerator worth building: the publisher's DONE
    store and its load of `waiter` (`par_runtime.c:455-463`) and the parking
    stack's registration and re-read (`par_runtime.c:496-505`) have to be
    interleaved both ways, and the second way is what reaches group A item 7.
    Checks the park protocol; no invariant of I1 to I4.
14. The crossed case, S22: two stacks resumed on foreign threads, each joining
    a hand-out on the other's home lane, with the pool exhausted and nothing
    READY. Kind B with the pool sized to one and two threads pinned, so the
    state is reached by construction rather than by luck. Checks I2 — the
    resolution must come from `wf__par_steal` on the thief end and never from
    an owner-side pop — and I1's stated exception, which the run must stay
    inside.
15. S17's three links, which are link-time facts and not schedules: the
    completion-only program (I/O, no hand-out) where `wf__sched_entry_stack`
    must resolve strongly, the entry must start on a stack from the core-entry
    reservation (§5), and no lane may ever be started; the hand-out-only program; and the no-core program
    with neither predicate, which must run today's `wf__floor_run` unchanged
    and never park. Kind B for the first two and Kind A for the third, each
    with a symbol check on the linked executable of the kind
    `compiler/Makefile:236-240` already performs. Checks no invariant of I1 to
    I4; it checks that the staging predicate of §7 admits exactly the three
    link shapes §5 describes.
16. Termination: no reachable state has every thread asleep with the exit
    status posted. This is a state assertion checked
    at every step, not a timeout — the defect it catches is a missing wake, and
    a wake that is missing is missing in a state the enumerator can name. It
    subsumes the schedule S17 walks and belongs with S6's bar, which is the
    same shape one event earlier: one completion wakes at least one thread and
    never none. The enumerator must be able to place the status post on both
    sides of the entry thread's epoch capture, because the two sides are what
    §6 distinguishes: a post before the capture is caught only by the status
    test inside the capture-to-park window, and a post after it is caught by
    the wake. Enumerator, over the configurations above.
17. Target progress as a nondeterministic operation. Wherever the core calls
    primitive 7 the enumerator may deliver any subset of the outstanding
    completions — each storing DONE into its record and publishing its waiter —
    may report progress or none, and may block, because the bounded pass runs a
    host call inline (§7.1's anchors). A model in which it always returns
    promptly is a model of something the runtime is not. That is strictly more
    schedules than a host produces, the right direction for a gate; the item is
    that every §11 invariant holds under all of them.
18. The stack free list with two poppers and two pushers. Group A item 9
    models the *lane slot* list,
    whose single popper makes its constituent-operation modelling sound; this
    one is many-producer many-consumer, and the assertion is that no stack is
    ever held by two threads and none is lost. Under the folded answer the list
    is inside the core's one mutex, so the enumerator schedules the lock as a
    wait state; if §12's measurement replaces it with a tagged lock-free stack,
    this item is what the tag has to satisfy. The push the enumerator must
    reach is the one §5 and §6 place *after* the switch, issued from the stack
    switched to: a push scheduled before the switch is the defect this item
    exists to catch, because it offers a stack a thread is still running on.
19. The record's protocol, enumerated **once** for both kinds. A compute slot
    and an I/O record are the same object (§5), so the enumerator models one
    publisher-and-waiter pair — DONE stored then the waiter loaded, against the
    waiter stored then the state re-read (`par_runtime.c:448-463`, `:496-505`) —
    and reaches it from both sides, `wf__par_execute` and a completion delivered
    inside primitive 7. Two models would be the defect this item prevents: the
    arm not enumerated is the arm whose interleavings are unchecked. Both step 3
    outcomes must be reachable from each side, which is what puts §6's cancel arm
    and its NOTIFIED widening under the enumeration rather than in prose.
20. No reachable state has every thread asleep with a non-empty ready list. A
    state assertion checked at every step, beside
    item 16's status assertion. It is expected to hold by construction once the
    ready-list test sits inside step 4's capture-to-park window (§6), and that
    is the reason to check it rather than a reason not to: it is the invariant a
    later reordering of that window would break first, and the defect it names —
    a ready continuation held behind a sleeping thread — is the one §3's
    principle forbids.
21. Every thread start obtains a stack: no reachable state has a thread taking
    its first pool stack while the free list is empty. Checked at every step over the swept
    configurations, all of which satisfy §5's floor S ≥ T + 1. This is the
    property §5 calls unrecoverable if violated — a thread start is not a join,
    so §2's fourth line does not cover it — and until now nothing asserted it:
    item 18 asserts the list never hands one stack to two threads and never
    loses one, which is a different property. The enumerator must reach the
    orderings §5's argument rules out by construction, a park before the pool
    starts and a park by an already-started worker during the create loop, so
    that the assertion is a check rather than a restatement of the reasoning.
22. One terminal completion per submission, and the record alive for it (§7).
    The enumerator counts terminal publications per accepted submission and
    asserts it is exactly one, with the resubmission arm reached so that several
    deliveries for one record are walked and only one is terminal
    (`linux_io_uring.c:1029-1040`), and with a submission that produces no
    kernel completion reached so the count is never zero. Beside it a compiler
    test asserts the lifetime half: no terminator is emitted while a completion
    is outstanding (`emitter.rs:1732-1740`). This is what replaces the token's
    range, stride and generation checks, so an item that never reaches the
    resubmission arm leaves the replacement unexercised.
23. The record's layout agreement (§8). A test asserts that the block the
    emitter reserves matches the runtime's `sizeof` and `_Alignof` for the
    record, and that the C `_Static_assert` fails the build when they disagree.
    The enumerator cannot cover this: it compiles the core against a replacement
    header and never touches the real layout, which is why the item is a
    compiler test beside the enumeration rather than inside it.
24. Replay. The controlled scheduler this section already requires is also the
    replay harness: it records the external inputs of a run — the data and the
    completion order — and replays them. The guarantee to check is **with
    identical external inputs, including completion order, the internal
    execution is identical**, and it is stated that way rather than as
    determinism, because completion order is a kernel event and therefore an
    input (header, §13). This is how constitution T3's sequential world is
    realized for a program that overlaps.

## 12. Measurements before a line of the compiler changes

- Park-and-resume against nested helping at a compute miss, on a compute-only
  program. Baseline in the tree: `par_layout.wf` at 0.3984 s (`WF_WORKERS=4`)
  and 0.3752 s (`W=8`), the comment at `par_runtime.c:871-878`. Bar: within noise.
  If it is not, the fallback is to allow nested runs of `never-suspends`
  jobs at a miss, which needs the target-action bit at the hand-out.
- Park cost of the lock-free handshake against the locked form of §6, if both
  are built. The comparison is one park and one publish each, against the same
  2.2 microsecond park-and-wake figure the next item uses. It is a measurement
  and not the reason to prefer either: §11's enumerator decides whether the
  lock-free form stands at all.
- The context switch itself: cost of one save/restore of callee-saved
  registers and the stack pointer, against the 2.2 microsecond park-and-wake
  figure the tree measured (`par_runtime.c:124-125`). A switch that is not
  well under that number removes the design's reason to exist.
- The four-stage chain in C on io_uring, 1000 files, 8 threads, in four shapes:
  nested helping, thread compensation, stack switch, and the shipped staged
  pipeline as lowered today (one lane, K slots, blocking on the oldest slot's
  join). Stage-two in-flight depth and wall time. The fourth shape is what makes
  this a comparison against the tree rather than against nested helping alone;
  the bar is that park on miss is not slower than the pipeline on the pipeline's
  own program and beats nested helping on §0's three shapes.
- The smallest stack count at which the pool stops refusing for the corpus
  programs, and what refusal costs below it — the one startup setting left (§5).
- Record memory, now per frame rather than per pool (§5). Run the stack ledger
  over `tests/programs` before and after, and report the growth per frame that
  holds a group and the growth of the deepest chain bound, which is what a stack
  is sized against. That growth replaces today's fixed 64 records
  (`bridge.c:42-44`) and is the price of having no refusal path.
- The ledger's chain bound per hand-out entry across `tests/programs`, to
  see how many classes real programs would need — input to §5's later step,
  not to the first implementation.

## 13. Deliberately not in this design

- Any change to which statements or iterations may overlap.
- Any writer-visible task, future, callback, or scheduling marker.
- A completion-record pool of any shape. The record lives in the submitting
  frame (§5), so there is no pool to size, grow, refuse from, or index with a
  token, and reintroducing one reintroduces the bound this review found false.
- Moving compute hand-out slots into frames. Only the I/O record moves; the
  compute slot stays the lane's with decision 6's refusal (§1), because a
  refused hand-out runs the call inline on the owner and costs nothing, where a
  refused I/O operation has no inline arm to fall to. Giving compute the same
  treatment is a possible later simplification and is not taken here.
- Growth of anything at run time: the stack count, the slot capacity and the
  deque capacity are all fixed at start and refuse rather than grow.
- The direct family in emitted code, and any per-operation blocking fallback.
  Emitted code submits and joins (§2, §8); an operation with no kernel
  completion form runs inside the runtime's engine (§7.1).
- Unqualified determinism. Completion order is a kernel event and therefore an
  input; constitution T3's sequential world is realized by *replay* — with
  identical external inputs, including completion order, the internal execution
  is identical (§11).
- Proof-sized stack classes and the link-time class table. Named as a later
  step in §5, taken only if the parked population can exceed tens.
